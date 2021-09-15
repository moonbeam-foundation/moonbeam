// Copyright 2019-2021 PureStake Inc.
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

//! The XCM primitive trait implementations

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	traits::{Get, OriginTrait},
	weights::{constants::WEIGHT_PER_SECOND, Weight},
};
use xcm::v0::{
	Error as XcmError,
	Junction::{AccountKey20, Parachain, Parent},
	MultiAsset, MultiLocation, NetworkId,
};
use xcm_builder::TakeRevenue;
use xcm_executor::traits::FilterAssetLocation;
use xcm_executor::traits::WeightTrader;

use sp_runtime::traits::Zero;

use sp_std::borrow::Borrow;
use sp_std::{convert::TryInto, marker::PhantomData};

/// Converter struct implementing `AssetIdConversion` converting a numeric asset ID
/// (must be `TryFrom/TryInto<u128>`) into a MultiLocation Value and Viceversa through
/// an intermediate generic type AssetType.
/// The trait bounds enforce is that the AssetTypeGetter trait is also implemented for AssetIdInfoGetter
pub struct AsAssetType<AssetId, AssetType, AssetIdInfoGetter>(
	PhantomData<(AssetId, AssetType, AssetIdInfoGetter)>,
);
impl<AssetId, AssetType, AssetIdInfoGetter> xcm_executor::traits::Convert<MultiLocation, AssetId>
	for AsAssetType<AssetId, AssetType, AssetIdInfoGetter>
where
	AssetId: From<AssetType> + Clone,
	AssetType: From<MultiLocation> + Into<Option<MultiLocation>> + Clone,
	AssetIdInfoGetter: AssetTypeGetter<AssetId, AssetType>,
{
	fn convert_ref(id: impl Borrow<MultiLocation>) -> Result<AssetId, ()> {
		let asset_type: AssetType = id.borrow().clone().into();
		Ok(AssetId::from(asset_type))
	}
	fn reverse_ref(what: impl Borrow<AssetId>) -> Result<MultiLocation, ()> {
		if let Some(asset_type) = AssetIdInfoGetter::get_asset_type(what.borrow().clone()) {
			if let Some(location) = asset_type.into() {
				Ok(location)
			} else {
				Err(())
			}
		} else {
			Err(())
		}
	}
}

/// Instructs how to convert a 20 byte accountId into a MultiLocation
pub struct AccountIdToMultiLocation<AccountId>(sp_std::marker::PhantomData<AccountId>);
impl<AccountId> sp_runtime::traits::Convert<AccountId, MultiLocation>
	for AccountIdToMultiLocation<AccountId>
where
	AccountId: Into<[u8; 20]>,
{
	fn convert(account: AccountId) -> MultiLocation {
		MultiLocation::X1(AccountKey20 {
			network: NetworkId::Any,
			key: account.into(),
		})
	}
}

// Convert a local Origin (i.e., a signed 20 byte account Origin)  to a Multilocation
pub struct SignedToAccountId20<Origin, AccountId, Network>(
	sp_std::marker::PhantomData<(Origin, AccountId, Network)>,
);
impl<Origin: OriginTrait + Clone, AccountId: Into<[u8; 20]>, Network: Get<NetworkId>>
	xcm_executor::traits::Convert<Origin, MultiLocation>
	for SignedToAccountId20<Origin, AccountId, Network>
where
	Origin::PalletsOrigin: From<frame_system::RawOrigin<AccountId>>
		+ TryInto<frame_system::RawOrigin<AccountId>, Error = Origin::PalletsOrigin>,
{
	fn convert(o: Origin) -> Result<MultiLocation, Origin> {
		o.try_with_caller(|caller| match caller.try_into() {
			Ok(frame_system::RawOrigin::Signed(who)) => Ok(AccountKey20 {
				key: who.into(),
				network: Network::get(),
			}
			.into()),
			Ok(other) => Err(other.into()),
			Err(other) => Err(other),
		})
	}
}

// We need to know how to charge for incoming assets
// This takes the first fungible asset, and takes whatever UnitPerSecondGetter establishes
// UnitsToWeightRatio trait, which needs to be implemented by AssetIdInfoGetter
pub struct FirstAssetTrader<
	AssetId: From<AssetType> + Clone,
	AssetType: From<MultiLocation> + Clone,
	AssetIdInfoGetter: UnitsToWeightRatio<AssetId>,
	R: TakeRevenue,
>(
	Weight,
	Option<(MultiLocation, u128, u128)>,
	PhantomData<(AssetId, AssetType, AssetIdInfoGetter, R)>,
);
impl<
		AssetId: From<AssetType> + Clone,
		AssetType: From<MultiLocation> + Clone,
		AssetIdInfoGetter: UnitsToWeightRatio<AssetId>,
		R: TakeRevenue,
	> WeightTrader for FirstAssetTrader<AssetId, AssetType, AssetIdInfoGetter, R>
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
		match first_asset {
			MultiAsset::ConcreteFungible { id, .. } => {
				let asset_type: AssetType = id.clone().into();
				let asset_id: AssetId = AssetId::from(asset_type);
				if let Some(units_per_second) = AssetIdInfoGetter::get_units_per_second(asset_id) {
					let amount = units_per_second * (weight as u128) / (WEIGHT_PER_SECOND as u128);
					let required = MultiAsset::ConcreteFungible {
						amount,
						id: id.clone(),
					};
					let (unused, _) = payment.less(required).map_err(|_| XcmError::TooExpensive)?;
					self.0 = self.0.saturating_add(weight);

					// In case the asset matches the one the trader already stored before, add
					// to later refund

					// Else we are always going to substract the weight if we can, but we latter do
					// not refund it

					// In short, we only refund on the asset the trader first succesfully was able
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
						self.0 = self.0.saturating_add(weight);
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

	fn refund_weight(&mut self, weight: Weight) -> MultiAsset {
		let result = if let Some((id, prev_amount, units_per_second)) = self.1.clone() {
			let weight = weight.min(self.0);
			self.0 -= weight;
			let amount = units_per_second * (weight as u128) / (WEIGHT_PER_SECOND as u128);
			self.1 = Some((
				id.clone(),
				prev_amount.saturating_sub(amount),
				units_per_second,
			));
			MultiAsset::ConcreteFungible { id, amount }
		} else {
			MultiAsset::None
		};
		result
	}
}

// This defines how multiTraders should be implemented
// The intention is to distinguish between non-self-reserve assets and the reserve asset
pub struct MultiWeightTraders<NativeTrader, OtherTrader> {
	native_trader: NativeTrader,
	other_trader: OtherTrader,
}
impl<NativeTrader: WeightTrader, OtherTrader: WeightTrader> WeightTrader
	for MultiWeightTraders<NativeTrader, OtherTrader>
{
	fn new() -> Self {
		Self {
			native_trader: NativeTrader::new(),
			other_trader: OtherTrader::new(),
		}
	}
	fn buy_weight(
		&mut self,
		weight: Weight,
		payment: xcm_executor::Assets,
	) -> Result<xcm_executor::Assets, XcmError> {
		if let Ok(assets) = self.native_trader.buy_weight(weight, payment.clone()) {
			return Ok(assets);
		}

		if let Ok(assets) = self.other_trader.buy_weight(weight, payment) {
			return Ok(assets);
		}

		Err(XcmError::TooExpensive)
	}
	fn refund_weight(&mut self, weight: Weight) -> MultiAsset {
		let native = self.native_trader.refund_weight(weight);
		match native {
			MultiAsset::ConcreteFungible { amount, .. } if !amount.is_zero() => return native,
			_ => {}
		}

		let other = self.other_trader.refund_weight(weight);
		match other {
			MultiAsset::ConcreteFungible { amount, .. } if !amount.is_zero() => return other,
			_ => {}
		}

		MultiAsset::None
	}
}

pub trait Reserve {
	/// Returns assets reserve location.
	fn reserve(&self) -> Option<MultiLocation>;
}

// Takes the chain part of a MultiAsset
impl Reserve for MultiAsset {
	fn reserve(&self) -> Option<MultiLocation> {
		if let MultiAsset::ConcreteFungible { id, .. } = self {
			match (id.first(), id.at(1)) {
				(Some(Parent), Some(Parachain(id))) => Some((Parent, Parachain(*id)).into()),
				(Some(Parent), _) => Some(Parent.into()),
				(Some(Parachain(id)), _) => Some(Parachain(*id).into()),
				_ => None,
			}
		} else {
			None
		}
	}
}

/// A `FilterAssetLocation` implementation. Filters multi native assets whose
/// reserve is same with `origin`.
pub struct MultiNativeAsset;
impl FilterAssetLocation for MultiNativeAsset {
	fn filter_asset_location(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		if let Some(ref reserve) = asset.reserve() {
			if reserve == origin {
				return true;
			}
		}
		false
	}
}

// Defines the trait to obtain a generic AssetType from a generic AssetId
pub trait AssetTypeGetter<AssetId, AssetType> {
	// Get units per second from asset type
	fn get_asset_type(asset_id: AssetId) -> Option<AssetType>;
}

// Defines the trait to obtain the units per second of a give assetId
// This parameter will be used to charge for fees upon assetId deposit
pub trait UnitsToWeightRatio<AssetId> {
	// Get units per second from asset type
	fn get_units_per_second(asset_id: AssetId) -> Option<u128>;
}
