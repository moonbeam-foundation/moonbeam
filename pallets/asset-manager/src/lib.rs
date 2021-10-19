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

//! # Asset Manager Pallet
//!
//! This pallet allows to register new assets for different purposes. The main purpose as of today
//! is the usage of such assets in xcm-operations, such as minting/burning when tokens are received
//! or sent by the chain
//!
//! The assumption is we work with AssetTypes, which for now represent xcm assets but in the future
//! can represent other kind of assets that we would like to register
//!
//! The main reason to have an asset-manager is to avoid users creating assets with assetIds that
//! we later cannot use xcm operations. The idea is that instead of looking storing the assetType
//! -> assetId association we make the assetId computable from the assetType, e.g., by hashing
//! it. That is the main reason why this pallet only stores the opposit association, i.e.,
//! assetId -> assetType. This is important, e.g., in xcm, where receiving assets does not imply
//!  any db read, but sending to another chain does require this db read.
//!
//! This pallet has two storage items: AssetIdType, which holds a mapping from AssetId->AssetType
//! AssetIdUnitsPerSecond: an AssetId->u128 mapping that holds how much each AssetId should be
//! charged per unit of second, i.e., allowing to pay for execution in the associated Asset
//!
//! This pallet has two extrinsics: register_asset, which registers an Asset in this pallet and
//! creates the asset as dictated by the AssetRegistrar trait. set_asset_units_per_second: which
//! sets the unit per second that should be charged for a particular asset.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;
pub use pallet::*;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
pub mod tests;

#[pallet]
pub mod pallet {

	use frame_support::{pallet_prelude::*, PalletId};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::HasCompact;
	use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned};

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// The AssetManagers's pallet id
	pub const PALLET_ID: PalletId = PalletId(*b"asstmngr");

	/// The registrar trait. This instructs how to create an asset
	pub trait AssetRegistrar<T: Config> {
		/// How to create an asset
		/// asser: the assetId of the associated asset
		/// min_balance: the existential deposit
		/// metadata: Other information related to the asset. It can be decimals, name, etc.
		fn create_asset(
			asset: T::AssetId,
			min_balance: T::Balance,
			metadata: T::AssetRegistrarMetadata,
		) -> DispatchResult;
	}

	// We implement this trait to be able to get the AssetType
	impl<T: Config> xcm_primitives::AssetTypeGetter<T::AssetId, T::AssetType> for Pallet<T> {
		/// asset_id: the assetId for which we want to get the assetType
		fn get_asset_type(asset_id: T::AssetId) -> Option<T::AssetType> {
			AssetIdType::<T>::get(asset_id)
		}
	}

	// We implement this trait to be able to get the units_per_second
	impl<T: Config> xcm_primitives::UnitsToWeightRatio<T::AssetId> for Pallet<T> {
		fn get_units_per_second(asset_id: T::AssetId) -> Option<u128> {
			AssetIdUnitsPerSecond::<T>::get(asset_id)
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Asset Id. This will be used to create the asset and to associate it with
		/// a assetType
		type AssetId: Member + Parameter + Default + Copy + HasCompact + MaxEncodedLen;

		/// The Asset Metadata we want to store. This can represent name, decimals, etc
		type AssetRegistrarMetadata: Member + Parameter;

		/// The Asset Kind. This represents the different kind of assets we want to register
		/// It needs to be convertible to an assetId
		type AssetType: Parameter + Member + Ord + PartialOrd + Into<Self::AssetId> + Default;

		/// The units in which we record balances.
		type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

		/// The asset Registrar.
		type AssetRegistrar: AssetRegistrar<Self>;

		/// Origin that is allowed to create and modify asset information
		type AssetModifierOrigin: EnsureOrigin<Self::Origin>;
	}

	/// An error that can occur while executing the mapping pallet's logic.
	#[pallet::error]
	pub enum Error<T> {
		ErrorCreatingAsset,
		AssetAlreadyExists,
		AssetDoesNotExist,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		AssetRegistered(T::AssetId, T::AssetType, T::AssetRegistrarMetadata),
		UnitsPerSecondChanged(T::AssetId, u128),
	}

	/// Stores the assetId -> AssetType association.
	#[pallet::storage]
	#[pallet::getter(fn asset_id_type)]
	pub type AssetIdType<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetId, T::AssetType>;

	/// Stores the units per second. Not all assets might contain units per second, hence the
	/// different storage. If an asset does not contain units per second, it cannot be used to
	/// pay for execution fee
	#[pallet::storage]
	#[pallet::getter(fn asset_id_units_per_second)]
	pub type AssetIdUnitsPerSecond<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetId, u128>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register new asset with the asset manager
		#[pallet::weight(0)]
		pub fn register_asset(
			origin: OriginFor<T>,
			asset: T::AssetType,
			metadata: T::AssetRegistrarMetadata,
			min_amount: T::Balance,
		) -> DispatchResult {
			// Ensure the origin matches AssetModifierOrigin
			T::AssetModifierOrigin::ensure_origin(origin)?;

			// Compute assetId from asset
			let asset_id: T::AssetId = asset.clone().into();

			// Ensure such an assetId does not exist
			ensure!(
				AssetIdType::<T>::get(&asset_id).is_none(),
				Error::<T>::AssetAlreadyExists
			);

			// Create the asset as instructed by AssetRegistrar
			T::AssetRegistrar::create_asset(asset_id, min_amount, metadata.clone())
				.map_err(|_| Error::<T>::ErrorCreatingAsset)?;

			// Insert the association assetId->assetType
			AssetIdType::<T>::insert(&asset_id, &asset);

			Self::deposit_event(Event::AssetRegistered(asset_id, asset, metadata));
			Ok(())
		}

		/// Change/Set the amount of units we are charging per execution second for a given AssetId
		#[pallet::weight(0)]
		pub fn set_asset_units_per_second(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			units_per_second: u128,
		) -> DispatchResult {
			// Ensure the origin matches AssetModifierOrigin
			T::AssetModifierOrigin::ensure_origin(origin)?;

			// Ensure such an assetId does not exist
			ensure!(
				AssetIdType::<T>::get(&asset_id).is_some(),
				Error::<T>::AssetDoesNotExist
			);

			// Write the units per second in the corresponding asset
			AssetIdUnitsPerSecond::<T>::insert(&asset_id, &units_per_second);

			Self::deposit_event(Event::UnitsPerSecondChanged(asset_id, units_per_second));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// The account ID of AssetManager
		pub fn account_id() -> T::AccountId {
			PALLET_ID.into_account()
		}
	}
}
