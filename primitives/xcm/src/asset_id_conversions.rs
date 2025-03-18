// Copyright 2019-2025 PureStake Inc.
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

use sp_runtime::traits::MaybeEquivalence;
use sp_std::marker::PhantomData;
use xcm::v3::Location;
use xcm_executor::traits::ConvertLocation;

/// Converter struct implementing `AssetIdConversion` converting a numeric asset ID
/// (must be `TryFrom/TryInto<u128>`) into a Location Value and vice versa through
/// an intermediate generic type AssetType.
/// The trait bounds enforce is that the AssetTypeGetter trait is also implemented for
/// AssetIdInfoGetter
pub struct AsAssetType<AssetId, AssetType, AssetIdInfoGetter>(
	PhantomData<(AssetId, AssetType, AssetIdInfoGetter)>,
);
impl<AssetId, AssetType, AssetIdInfoGetter> MaybeEquivalence<Location, AssetId>
	for AsAssetType<AssetId, AssetType, AssetIdInfoGetter>
where
	AssetId: Clone,
	AssetType: From<Location> + Into<Option<Location>> + Clone,
	AssetIdInfoGetter: AssetTypeGetter<AssetId, AssetType>,
{
	fn convert(id: &Location) -> Option<AssetId> {
		AssetIdInfoGetter::get_asset_id(id.clone().into())
	}
	fn convert_back(what: &AssetId) -> Option<Location> {
		AssetIdInfoGetter::get_asset_type(what.clone()).and_then(Into::into)
	}
}
impl<AssetId, AssetType, AssetIdInfoGetter> MaybeEquivalence<xcm::v5::Location, AssetId>
	for AsAssetType<AssetId, AssetType, AssetIdInfoGetter>
where
	AssetId: Clone,
	AssetType: From<Location> + Into<Option<Location>> + Clone,
	AssetIdInfoGetter: AssetTypeGetter<AssetId, AssetType>,
{
	fn convert(id: &xcm::v5::Location) -> Option<AssetId> {
		let v3_location =
			xcm_builder::WithLatestLocationConverter::<xcm::v3::Location>::convert(id)?;
		AssetIdInfoGetter::get_asset_id(v3_location.clone().into())
	}
	fn convert_back(what: &AssetId) -> Option<xcm::v5::Location> {
		let v3_location: Location =
			AssetIdInfoGetter::get_asset_type(what.clone()).and_then(Into::into)?;
		xcm_builder::WithLatestLocationConverter::convert_back(&v3_location)
	}
}
impl<AssetId, AssetType, AssetIdInfoGetter> ConvertLocation<AssetId>
	for AsAssetType<AssetId, AssetType, AssetIdInfoGetter>
where
	AssetId: Clone,
	AssetType: From<Location> + Into<Option<Location>> + Clone,
	AssetIdInfoGetter: AssetTypeGetter<AssetId, AssetType>,
{
	fn convert_location(id: &xcm::v5::Location) -> Option<AssetId> {
		let v3_location =
			xcm_builder::WithLatestLocationConverter::<xcm::v3::Location>::convert(id)?;
		AssetIdInfoGetter::get_asset_id(v3_location.clone().into())
	}
}

/// Defines the trait to obtain a generic AssetType from a generic AssetId and vice versa
pub trait AssetTypeGetter<AssetId, AssetType> {
	// Get asset type from assetId
	fn get_asset_type(asset_id: AssetId) -> Option<AssetType>;

	// Get assetId from assetType
	fn get_asset_id(asset_type: AssetType) -> Option<AssetId>;

	// Set assetId and assetType
	#[cfg(feature = "runtime-benchmarks")]
	fn set_asset_type_asset_id(asset_type: AssetType, asset_id: AssetId);
}

/// This trait ensure we can convert AccountIds to CurrencyIds
/// We will require Runtime to have this trait implemented
pub trait AccountIdToCurrencyId<Account, CurrencyId> {
	// Get assetId from account
	fn account_to_currency_id(account: Account) -> Option<CurrencyId>;
}
