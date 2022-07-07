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

use frame_support::{
	traits::{tokens::fungibles::Mutate, Get},
	weights::{constants::WEIGHT_PER_SECOND, Weight},
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
	Option<(MultiLocation, u128, u128)>,
	PhantomData<(AssetType, AssetIdInfoGetter, R)>,
);
impl<
		AssetType: From<MultiLocation> + Clone,
		AssetIdInfoGetter: UnitsToWeightRatio<AssetType>,
		R: TakeRevenue,
	> WeightTrader for FirstAssetTrader<AssetType, AssetIdInfoGetter, R>
{
	fn new() -> Self {
		FirstAssetTrader(0, None, PhantomData)
	}
	fn buy_weight(
		&mut self,
		weight: Weight,
		payment: xcm_executor::Assets,
	) -> Result<xcm_executor::Assets, XcmError> {
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
					let amount = units_per_second.saturating_mul(weight as u128)
						/ (WEIGHT_PER_SECOND as u128);

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
					self.0 = self.0.saturating_add(weight); // XXX: should do all sanity checks first

					// In case the asset matches the one the trader already stored before, add
					// to later refund

					// Else we are always going to subtract the weight if we can, but we latter do
					// not refund it

					// In short, we only refund on the asset the trader first successfully was able
					// to pay for an execution
					let new_asset = match self.1.clone() {
						Some((prev_id, prev_amount, units_per_second)) => {
							if prev_id == id.clone() {
								Some((id, prev_amount.saturating_add(amount), units_per_second))
							} else {
								None
							}
						}
						None => Some((id, amount, units_per_second)),
					};

					// Due to the trait bound, we can only refund one asset.
					if let Some(new_asset) = new_asset {
						self.0 = self.0.saturating_add(weight); // XXX: this is duplicated
						self.1 = Some(new_asset);
					};
					return Ok(unused);
				} else {
					return Err(XcmError::TooExpensive);
				};
			}
			_ => return Err(XcmError::TooExpensive),
		}
	}

	// Refund weight. We will refund in whatever asset is stored in self.
	fn refund_weight(&mut self, weight: Weight) -> Option<MultiAsset> {
		if let Some((id, prev_amount, units_per_second)) = self.1.clone() {
			let weight = weight.min(self.0);
			self.0 -= weight;
			let amount = units_per_second * (weight as u128) / (WEIGHT_PER_SECOND as u128);
			self.1 = Some((
				id.clone(),
				prev_amount.saturating_sub(amount), // XXX: should handle this going negative
				units_per_second,
			));
			Some(MultiAsset {
				fun: Fungibility::Fungible(amount), // XXX: amount should be checked 
                                                    // XXX: also, can't be 0 or will panic in debug
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
			R::take_revenue((id, amount).into());
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
				if !amount.is_zero() {
					let ok = Assets::mint_into(asset_id, &ReceiverAccount::get(), amount).is_ok();
					debug_assert!(ok, "`mint_into` cannot generally fail; qed");
				}
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
}

#[cfg(test)]
mod test {
	use super::*;
	use xcm::latest::{AssetId, Junction, Junctions, Fungibility};
    use xcm_executor::Assets;

	impl UnitsToWeightRatio<MultiLocation> for () {
		fn payment_is_supported(_asset_type: MultiLocation) -> bool { true }
		fn get_units_per_second(_asset_type: MultiLocation) -> Option<u128> {
			// return WEIGHT_PER_SECOND to cancel the division out in buy_weight()
			// this should make weight and payment amounts directly comparable
			Some(WEIGHT_PER_SECOND as u128)
		}
	}

	#[test]
	fn test_buy_weight_accounts_weight_properly() {
		let amount = 1000u128;

		let mut payment = Assets::new();
		payment.subsume(MultiAsset {
			id: AssetId::Concrete(MultiLocation {
				parents: 0u8,
				interior: Junctions::Here,
			}),
			fun: Fungibility::Fungible(amount),
		});

		let mut trader: FirstAssetTrader<MultiLocation, (), ()> = FirstAssetTrader::new();
		let unused = trader.buy_weight(amount as Weight, payment.clone()).expect("can buy weight");
		assert!(unused.is_empty());
		assert_eq!(trader.0, 1000u64);
	}

    #[test]
    fn cant_mint_when_calling_buy_weight_with_different_types() {

		let mut trader: FirstAssetTrader<MultiLocation, (), ()> = FirstAssetTrader::new();

        // first buy_weight in one asset with a small amount
        let mut asset_one_payment = Assets::new();
		asset_one_payment.subsume(MultiAsset {
			id: AssetId::Concrete(MultiLocation {
				parents: 0u8,
				interior: Junctions::X1(Junction::Parachain(1000)),
			}),
			fun: Fungibility::Fungible(100u128),
		});
        let buy_one_results = trader.buy_weight(100u32 as Weight, asset_one_payment.clone())
            .expect("can buy weight");
        assert_eq!(buy_one_results.fungible.len(), 0);

        // then buy_weight in a second asset with a larger amount
        let mut asset_two_payment = xcm_executor::Assets::new();
		asset_two_payment.subsume(MultiAsset {
			id: AssetId::Concrete(MultiLocation {
				parents: 0u8,
				interior: Junctions::X1(Junction::Parachain(1001)),
			}),
			fun: Fungibility::Fungible(10_000u128),
		});
        let buy_two_results = trader.buy_weight(10_000u32 as Weight, asset_two_payment.clone())
            .expect("can buy weight again"); // TODO: this should fail!
        assert_eq!(buy_two_results.fungible.len(), 0);

        let result = trader.refund_weight(9_999u32 as Weight);

        // TODO: assert about weight refunded and FirstAssetTrader's state
    }

}
