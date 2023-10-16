// Copyright 2019-2022 PureStake Inc.
// This file is part of Moonbeam.

// Moonbeam is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonbeam is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>.

// We need to know how to charge for incoming assets
// We assume AssetIdInfoGetter is implemented and is capable of getting how much units we should
// charge for a given asset
// This takes the first fungible asset, and takes whatever UnitPerSecondGetter establishes
// UnitsToWeightRatio trait, which needs to be implemented by AssetIdInfoGetter

use cumulus_primitives_core::XcmContext;
use frame_support::{
	pallet_prelude::Weight,
	traits::{tokens::fungibles::Mutate, Get},
	weights::constants::WEIGHT_REF_TIME_PER_SECOND,
};
use sp_runtime::traits::Zero;
use sp_std::marker::PhantomData;
use xcm::latest::{
	AssetId as xcmAssetId, Error as XcmError, Fungibility, MultiAsset, MultiLocation,
};

use xcm_builder::TakeRevenue;
use xcm_executor::traits::{MatchesFungibles, WeightTrader};

pub struct FirstAssetTrader<
	AssetType: From<MultiLocation> + Clone,
	AssetIdInfoGetter: UnitsToWeightRatio<AssetType>,
	R: TakeRevenue,
>(
	Weight,
	Option<(MultiLocation, u128, u128)>, // id, amount, units_per_second
	PhantomData<(AssetType, AssetIdInfoGetter, R)>,
);
impl<
		AssetType: From<MultiLocation> + Clone,
		AssetIdInfoGetter: UnitsToWeightRatio<AssetType>,
		R: TakeRevenue,
	> WeightTrader for FirstAssetTrader<AssetType, AssetIdInfoGetter, R>
{
	fn new() -> Self {
		FirstAssetTrader(Weight::zero(), None, PhantomData)
	}
	fn buy_weight(
		&mut self,
		weight: Weight,
		payment: xcm_executor::Assets,
		_context: &XcmContext,
	) -> Result<xcm_executor::Assets, XcmError> {
		// can only call one time
		if self.1.is_some() {
			// TODO: better error
			return Err(XcmError::NotWithdrawable);
		}

		assert_eq!(self.0, Weight::zero());
		let first_asset = payment
			.clone()
			.fungible_assets_iter()
			.next()
			.ok_or(XcmError::TooExpensive)?;

		// We are only going to check first asset for now. This should be sufficient for simple token
		// transfers. We will see later if we change this.
		match (first_asset.id, first_asset.fun) {
			(xcmAssetId::Concrete(id), Fungibility::Fungible(_)) => {
				let asset_type: AssetType = id.clone().into();
				// Shortcut if we know the asset is not supported
				// This involves the same db read per block, mitigating any attack based on
				// non-supported assets
				if !AssetIdInfoGetter::payment_is_supported(asset_type.clone()) {
					return Err(XcmError::TooExpensive);
				}
				if let Some(units_per_second) = AssetIdInfoGetter::get_units_per_second(asset_type)
				{
					// TODO handle proof size payment
					let amount = units_per_second.saturating_mul(weight.ref_time() as u128)
						/ (WEIGHT_REF_TIME_PER_SECOND as u128);

					// We dont need to proceed if the amount is 0
					// For cases (specially tests) where the asset is very cheap with respect
					// to the weight needed
					if amount.is_zero() {
						return Ok(payment);
					}

					let required = MultiAsset {
						fun: Fungibility::Fungible(amount),
						id: xcmAssetId::Concrete(id.clone()),
					};
					let unused = payment
						.checked_sub(required)
						.map_err(|_| XcmError::TooExpensive)?;

					self.0 = weight;
					self.1 = Some((id, amount, units_per_second));

					return Ok(unused);
				} else {
					return Err(XcmError::TooExpensive);
				};
			}
			_ => return Err(XcmError::TooExpensive),
		}
	}

	// Refund weight. We will refund in whatever asset is stored in self.
	fn refund_weight(&mut self, weight: Weight, _context: &XcmContext) -> Option<MultiAsset> {
		if let Some((id, prev_amount, units_per_second)) = self.1.clone() {
			let weight = weight.min(self.0);
			self.0 -= weight;
			let amount = units_per_second * (weight.ref_time() as u128)
				/ (WEIGHT_REF_TIME_PER_SECOND as u128);
			let amount = amount.min(prev_amount);
			self.1 = Some((
				id.clone(),
				prev_amount.saturating_sub(amount),
				units_per_second,
			));
			Some(MultiAsset {
				fun: Fungibility::Fungible(amount),
				id: xcmAssetId::Concrete(id.clone()),
			})
		} else {
			None
		}
	}
}

/// Deal with spent fees, deposit them as dictated by R
impl<
		AssetType: From<MultiLocation> + Clone,
		AssetIdInfoGetter: UnitsToWeightRatio<AssetType>,
		R: TakeRevenue,
	> Drop for FirstAssetTrader<AssetType, AssetIdInfoGetter, R>
{
	fn drop(&mut self) {
		if let Some((id, amount, _)) = self.1.clone() {
			if amount > 0 {
				R::take_revenue((id, amount).into());
			}
		}
	}
}

/// XCM fee depositor to which we implement the TakeRevenue trait
/// It receives a fungibles::Mutate implemented argument, a matcher to convert MultiAsset into
/// AssetId and amount, and the fee receiver account
pub struct XcmFeesToAccount<Assets, Matcher, AccountId, ReceiverAccount>(
	PhantomData<(Assets, Matcher, AccountId, ReceiverAccount)>,
);
impl<
		Assets: Mutate<AccountId>,
		Matcher: MatchesFungibles<Assets::AssetId, Assets::Balance>,
		AccountId: Clone,
		ReceiverAccount: Get<AccountId>,
	> TakeRevenue for XcmFeesToAccount<Assets, Matcher, AccountId, ReceiverAccount>
{
	fn take_revenue(revenue: MultiAsset) {
		match Matcher::matches_fungibles(&revenue) {
			Ok((asset_id, amount)) => {
				let ok = Assets::mint_into(asset_id, &ReceiverAccount::get(), amount).is_ok();
				debug_assert!(ok, "`mint_into` cannot generally fail; qed");
			}
			Err(_) => log::debug!(
				target: "xcm",
				"take revenue failed matching fungible"
			),
		}
	}
}

// Defines the trait to obtain the units per second of a give asset_type for local execution
// This parameter will be used to charge for fees upon asset_type deposit
pub trait UnitsToWeightRatio<AssetType> {
	// Whether payment in a particular asset_type is suppotrted
	fn payment_is_supported(asset_type: AssetType) -> bool;
	// Get units per second from asset type
	fn get_units_per_second(asset_type: AssetType) -> Option<u128>;
	#[cfg(feature = "runtime-benchmarks")]
	fn set_units_per_second(_asset_type: AssetType, _fee_per_second: u128) {}
}

#[cfg(test)]
mod test {
	use super::*;
	use cumulus_primitives_core::XcmHash;
	use xcm::latest::{AssetId, Fungibility, Junction, Junctions};
	use xcm_executor::Assets;

	const ARBITRARY_ML: MultiLocation = MultiLocation {
		parents: 0u8,
		interior: Junctions::Here,
	};
	const ARBITRARY_ID: AssetId = AssetId::Concrete(ARBITRARY_ML);

	impl UnitsToWeightRatio<MultiLocation> for () {
		fn payment_is_supported(_asset_type: MultiLocation) -> bool {
			true
		}
		fn get_units_per_second(_asset_type: MultiLocation) -> Option<u128> {
			// return WEIGHT_REF_TIME_PER_SECOND to cancel the division out in buy_weight()
			// this should make weight and payment amounts directly comparable
			Some(WEIGHT_REF_TIME_PER_SECOND as u128)
		}
	}

	#[test]
	fn test_buy_weight_accounts_weight_properly() {
		let amount = 1000u128;

		let mut payment = Assets::new();
		let multilocation = MultiLocation {
			parents: 0u8,
			interior: Junctions::Here,
		};
		payment.subsume(MultiAsset {
			id: AssetId::Concrete(multilocation),
			fun: Fungibility::Fungible(amount),
		});

		let mut trader: FirstAssetTrader<MultiLocation, (), ()> = FirstAssetTrader::new();
		let ctx = XcmContext {
			origin: Some(multilocation),
			message_id: XcmHash::default(),
			topic: None,
		};
		let unused = trader
			.buy_weight((amount as u64).into(), payment.clone(), &ctx)
			.expect("can buy weight once");
		assert!(unused.is_empty());
		assert_eq!(trader.0, 1000u64.into());
	}

	#[test]
	fn cant_call_buy_weight_twice() {
		let mut trader: FirstAssetTrader<MultiLocation, (), ()> = FirstAssetTrader::new();

		// should be able to buy once
		let mut asset_one_payment = Assets::new();
		let multilocation = MultiLocation {
			parents: 0u8,
			interior: Junctions::X1(Junction::Parachain(1000)),
		};
		asset_one_payment.subsume(MultiAsset {
			id: AssetId::Concrete(multilocation),
			fun: Fungibility::Fungible(100u128),
		});
		let ctx = XcmContext {
			origin: Some(multilocation),
			message_id: XcmHash::default(),
			topic: None,
		};
		let buy_one_results = trader
			.buy_weight(100u64.into(), asset_one_payment.clone(), &ctx)
			.expect("can buy weight once");
		assert_eq!(buy_one_results.fungible.len(), 0); // no unused amount
		assert_eq!(trader.0, 100u64.into());
		assert_eq!(
			trader.1,
			Some((
				MultiLocation {
					parents: 0u8,
					interior: Junctions::X1(Junction::Parachain(1000))
				},
				100,
				WEIGHT_REF_TIME_PER_SECOND as u128
			))
		);

		// but not twice
		let mut asset_two_payment = xcm_executor::Assets::new();
		let multi_location = MultiLocation {
			parents: 0u8,
			interior: Junctions::X1(Junction::Parachain(1001)),
		};
		asset_two_payment.subsume(MultiAsset {
			id: AssetId::Concrete(multi_location),
			fun: Fungibility::Fungible(10_000u128),
		});
		let ctx = XcmContext {
			origin: Some(multi_location),
			message_id: XcmHash::default(),
			topic: None,
		};
		assert_eq!(
			trader.buy_weight(10_000u64.into(), asset_two_payment.clone(), &ctx),
			Err(XcmError::NotWithdrawable),
		);

		// state should be unchanged
		assert_eq!(trader.0, 100u64.into());
		assert_eq!(
			trader.1,
			Some((
				MultiLocation {
					parents: 0u8,
					interior: Junctions::X1(Junction::Parachain(1000))
				},
				100,
				WEIGHT_REF_TIME_PER_SECOND as u128
			))
		);
	}

	#[test]
	fn can_call_refund_weight_with_all_weight() {
		let amount = 1000u128;

		let mut payment = Assets::new();
		payment.subsume(MultiAsset {
			id: ARBITRARY_ID,
			fun: Fungibility::Fungible(amount),
		});

		let mut trader: FirstAssetTrader<MultiLocation, (), ()> = FirstAssetTrader::new();
		let ctx = XcmContext {
			origin: Some(ARBITRARY_ML),
			message_id: XcmHash::default(),
			topic: None,
		};
		let unused = trader
			.buy_weight((amount as u64).into(), payment.clone(), &ctx)
			.expect("can buy weight once");
		assert!(unused.is_empty());
		assert_eq!(trader.0, 1000u64.into());

		assert_eq!(
			trader.refund_weight(1000u64.into(), &ctx),
			Some(MultiAsset {
				fun: Fungibility::Fungible(1000),
				id: ARBITRARY_ID,
			})
		);
	}

	#[test]
	fn can_call_refund_multiple_times() {
		let amount = 1000u128;

		let mut payment = Assets::new();
		payment.subsume(MultiAsset {
			id: ARBITRARY_ID,
			fun: Fungibility::Fungible(amount),
		});

		let mut trader: FirstAssetTrader<MultiLocation, (), ()> = FirstAssetTrader::new();
		let ctx = XcmContext {
			origin: Some(ARBITRARY_ML),
			message_id: XcmHash::default(),
			topic: None,
		};
		let unused = trader
			.buy_weight((amount as u64).into(), payment.clone(), &ctx)
			.expect("can buy weight once");
		assert!(unused.is_empty());
		assert_eq!(trader.0, 1000u64.into());

		assert_eq!(
			trader.refund_weight(100u64.into(), &ctx),
			Some(MultiAsset {
				fun: Fungibility::Fungible(100),
				id: ARBITRARY_ID,
			})
		);

		// should reflect 100 weight and 100 currency deducted
		assert_eq!(trader.0, 900u64.into());
		assert_eq!(trader.1.clone().unwrap().1, 900);

		// can call again
		assert_eq!(
			trader.refund_weight(200u64.into(), &ctx),
			Some(MultiAsset {
				fun: Fungibility::Fungible(200),
				id: ARBITRARY_ID,
			})
		);

		// should reflect another 200 weight and 200 currency deducted
		assert_eq!(trader.0, 700u64.into());
		assert_eq!(trader.1.clone().unwrap().1, 700);
	}

	#[test]
	fn refund_weight_caps_weight() {
		let amount = 1000u128;

		let mut payment = Assets::new();
		payment.subsume(MultiAsset {
			id: ARBITRARY_ID,
			fun: Fungibility::Fungible(amount),
		});
		let ctx = XcmContext {
			origin: Some(ARBITRARY_ML),
			message_id: XcmHash::default(),
			topic: None,
		};
		let mut trader: FirstAssetTrader<MultiLocation, (), ()> = FirstAssetTrader::new();
		let unused = trader
			.buy_weight((amount as u64).into(), payment.clone(), &ctx)
			.expect("can buy weight once");
		assert!(unused.is_empty());
		assert_eq!(trader.0, 1000u64.into());

		// can't call with more weight
		assert_eq!(
			trader.refund_weight(9999u64.into(), &ctx),
			Some(MultiAsset {
				fun: Fungibility::Fungible(1000),
				id: ARBITRARY_ID,
			})
		);
		assert_eq!(trader.0, Weight::zero());
	}

	#[test]
	fn refund_weight_caps_currency() {
		let amount = 1000u128;

		let mut payment = Assets::new();
		payment.subsume(MultiAsset {
			id: ARBITRARY_ID,
			fun: Fungibility::Fungible(amount),
		});

		let ctx = XcmContext {
			origin: Some(ARBITRARY_ML),
			message_id: XcmHash::default(),
			topic: None,
		};
		let mut trader: FirstAssetTrader<MultiLocation, (), ()> = FirstAssetTrader::new();
		let unused = trader
			.buy_weight((amount as u64).into(), payment.clone(), &ctx)
			.expect("can buy weight once");
		assert!(unused.is_empty());
		assert_eq!(trader.0, 1000u64.into());

		// adjust weight so that it will allow a higher amount -- we want to see that the currency
		// (self.1.1) is capped even when weight is not
		trader.0 = trader.0.saturating_add(1000u64.into());

		// can't call with more weight
		assert_eq!(
			trader.refund_weight(1500u64.into(), &ctx),
			Some(MultiAsset {
				fun: Fungibility::Fungible(1000),
				id: ARBITRARY_ID,
			})
		);
		assert_eq!(trader.0, 500u64.into()); // still thinks we have unreturned weight
	}
}
