// Copyright 2025 Moonbeam foundation
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

use core::marker::PhantomData;

use frame_support::traits::{
	fungible::{self, NativeOrWithId},
	tokens::ConversionFromAssetBalance,
};
use moonbeam_core_primitives::{AssetId, Balance};
#[cfg(feature = "runtime-benchmarks")]
use pallet_moonbeam_foreign_assets::AssetCreate;
use pallet_xcm_weight_trader::RELATIVE_PRICE_DECIMALS;
use sp_runtime::traits::MaybeEquivalence;
use xcm::v5::Location;

pub struct AssetRateConverter<T, NativeAsset, ForeignAssets>(
	PhantomData<(T, NativeAsset, ForeignAssets)>,
);
impl<
		T: frame_system::Config + pallet_xcm_weight_trader::Config,
		NativeAsset: fungible::Mutate<T::AccountId> + fungible::Inspect<T::AccountId>,
		#[cfg(not(feature = "runtime-benchmarks"))] ForeignAssets: pallet_moonbeam_foreign_assets::AssetInspect + MaybeEquivalence<Location, AssetId>,
		#[cfg(feature = "runtime-benchmarks")] ForeignAssets: pallet_moonbeam_foreign_assets::AssetInspect
			+ MaybeEquivalence<Location, AssetId>
			+ AssetCreate,
	> ConversionFromAssetBalance<Balance, NativeOrWithId<AssetId>, Balance>
	for AssetRateConverter<T, NativeAsset, ForeignAssets>
{
	type Error = pallet_xcm_weight_trader::Error<T>;

	fn from_asset_balance(
		balance: Balance,
		asset_kind: NativeOrWithId<AssetId>,
	) -> Result<Balance, Self::Error> {
		match asset_kind {
			NativeOrWithId::Native => Ok(balance),
			NativeOrWithId::WithId(asset_id) => {
				let location = ForeignAssets::convert_back(&asset_id)
					.ok_or(pallet_xcm_weight_trader::Error::<T>::AssetNotFound)?;
				let relative_price =
					pallet_xcm_weight_trader::Pallet::<T>::get_asset_relative_price(&location)
						.ok_or(pallet_xcm_weight_trader::Error::<T>::AssetNotFound)?;
				Ok(balance
					.checked_mul(relative_price)
					.ok_or(pallet_xcm_weight_trader::Error::<T>::PriceOverflow)?
					.checked_div(10u128.pow(RELATIVE_PRICE_DECIMALS))
					.ok_or(pallet_xcm_weight_trader::Error::<T>::PriceOverflow)?)
			}
		}
	}

	/// Set a conversion rate to `1` for the `asset_id`.
	#[cfg(feature = "runtime-benchmarks")]
	fn ensure_successful(asset_id: NativeOrWithId<AssetId>) {
		use frame_support::{assert_ok, traits::OriginTrait};
		use xcm::opaque::v5::Junction::Parachain;
		match asset_id {
			NativeOrWithId::Native => (),
			NativeOrWithId::WithId(asset_id) => {
				if let None = ForeignAssets::convert_back(&asset_id) {
					let location = Location::new(1, [Parachain(1000)]);
					let root = <T as frame_system::Config>::RuntimeOrigin::root();

					assert_ok!(ForeignAssets::create_asset(asset_id, location.clone()));

					assert_ok!(pallet_xcm_weight_trader::Pallet::<T>::add_asset(
						root,
						location.clone(),
						1u128,
					));
				}
			}
		}
	}
}
// type Error = Error;
// fn from_asset_balance(
// 	asset_balance: Balance,
// 	asset_kind: NativeOrWithId<AssetId>,
// ) -> Result<Balance, Error> {
// 	match asset_kind {
// 		NativeOrWithId::Native => Ok(asset_balance),
// 		NativeOrWithId::WithId(asset_id) => {
// 			let location =
// 				T::AssetIdentifier::convert_back(&asset_id).ok_or(Error::<T>::AssetNotFound)?;
// 			let relative_price = Pallet::<T>::get_asset_relative_price(&location)
// 				.ok_or(Error::AssetNotFound)?;
// 			Ok(asset_balance
// 				.checked_mul(relative_price)
// 				.ok_or(Error::PriceOverflow)?
// 				.checked_div(10u128.pow(RELATIVE_PRICE_DECIMALS))
// 				.ok_or(Error::PriceOverflow)?)
// 		}
// 	}
// }

// /// Set a conversion rate to `1` for the `asset_id`.
// #[cfg(feature = "runtime-benchmarks")]
// fn ensure_successful(asset_id: NativeOrWithId<AssetId>) {
// 	use frame_support::{assert_ok, traits::OriginTrait};
// 	use pallet_moonbeam_foreign_assets::AssetCreate;
// 	use xcm::opaque::v5::Junction::Parachain;
// 	match asset_id {
// 		NativeOrWithId::Native => (),
// 		NativeOrWithId::WithId(asset_id) => {
// 			if let None = T::AssetIdentifier::convert_back(&asset_id) {
// 				let location = Location::new(1, [Parachain(1000)]);
// 				let root = <T as frame_system::Config>::RuntimeOrigin::root();

// 				assert_ok!(T::AssetCreator::create_asset(asset_id, location.clone()));

// 				assert_ok!(Self::add_asset(root, location.clone(), 1u128,));
// 			}
// 		}
// 	}
// }
