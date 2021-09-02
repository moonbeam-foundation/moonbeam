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

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;
pub use pallet::*;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
pub mod tests;

#[pallet]
pub mod pallet {

	use frame_support::pallet_prelude::*;
	use frame_system::{ensure_root, pallet_prelude::*};
	use parity_scale_codec::HasCompact;
	use sp_runtime::traits::AtLeast32BitUnsigned;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	// The registrar trait. We need to comply with this
	pub trait AssetRegistrar<T: Config> {
		// How to create an asset
		fn create_asset(asset: T::AssetId, min_balance: T::Balance) -> DispatchResult;
		fn destroy_asset(asset: T::AssetId) -> DispatchResultWithPostInfo;
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Asset Id. This will be used to register the asset in Assets
		type AssetId: Member + Parameter + Default + Copy + HasCompact + MaxEncodedLen;

		/// The Asset Kind.
		type AssetType: Parameter + Member + Ord + PartialOrd + Into<Self::AssetId> + Default;

		/// The units in which we record balances.
		type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

		/// The trait we use to register Assets
		type AssetRegistrar: AssetRegistrar<Self>;
	}

	/// An error that can occur while executing the mapping pallet's logic.
	#[pallet::error]
	pub enum Error<T> {
		ErrorCreatingAsset,
		ErrorDestroyingAsset,
		AssetAlreadyExists,
		MultiLocationAlreadyExists,
		AssetDoestNotExist,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		XcmAssetRegistered(T::AssetType, T::AssetId),
		XcmAssetDestroyed(T::AssetId),
	}

	#[pallet::storage]
	#[pallet::getter(fn asset_id_to_type)]
	pub type AssetIdToAssetType<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetId, T::AssetType>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn xcm_asset_register(
			origin: OriginFor<T>,
			asset: T::AssetType,
			min_amount: T::Balance,
		) -> DispatchResult {
			ensure_root(origin)?;
			let asset_id: T::AssetId = asset.clone().into();
			ensure!(
				AssetIdToAssetType::<T>::get(&asset_id).is_none(),
				Error::<T>::AssetAlreadyExists
			);
			T::AssetRegistrar::create_asset(asset_id, min_amount)
				.map_err(|_| Error::<T>::ErrorCreatingAsset)?;
			AssetIdToAssetType::<T>::insert(&asset_id, &asset);

			Self::deposit_event(Event::XcmAssetRegistered(asset, asset_id));
			Ok(())
		}
		#[pallet::weight(0)]
		pub fn xcm_asset_destroy(origin: OriginFor<T>, asset_id: T::AssetId) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(
				AssetIdToAssetType::<T>::get(asset_id).is_some(),
				Error::<T>::AssetDoestNotExist
			);
			T::AssetRegistrar::destroy_asset(asset_id)
				.map_err(|_| Error::<T>::ErrorDestroyingAsset)?;
			AssetIdToAssetType::<T>::remove(asset_id);

			Self::deposit_event(Event::XcmAssetDestroyed(asset_id));

			Ok(())
		}
	}
}
