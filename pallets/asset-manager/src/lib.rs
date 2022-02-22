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

//! TODO Doc comments for the pallet
//! # Asset Manager Pallet
//!
//! This pallet allows to register new assets if certain conditions are met
//! The main goal of this pallet is to allow moonbeam to register XCM assets
//! The assumption is we work with AssetTypes, which can then be comperted to AssetIds
//!
//! This pallet has three storage items: AssetIdType, which holds a mapping from AssetId->AssetType
//! AssetTypeUnitsPerSecond: an AssetType->u128 mapping that holds how much each AssetType should be
//! charged per unit of second, in the case such an Asset is received as a XCM asset. Finally,
//! AssetTypeId holds a mapping from AssetType -> AssetId.
//!
//! This pallet has three extrinsics: register_asset, which registers an Asset in this pallet and
//! creates the asset as dictated by the AssetRegistrar trait. set_asset_units_per_second: which
//! sets the unit per second that should be charged for a particular asset.
//! change_existing_asset_type: which allows to update the correspondence between AssetId and
//! AssetType

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;
pub use pallet::*;
#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;
pub mod migrations;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
pub mod tests;
pub mod weights;

#[pallet]
pub mod pallet {

	use crate::weights::WeightInfo;
	use frame_support::{pallet_prelude::*, PalletId};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::HasCompact;
	use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// The AssetManagers's pallet id
	pub const PALLET_ID: PalletId = PalletId(*b"asstmngr");

	// The registrar trait. We need to comply with this
	pub trait AssetRegistrar<T: Config> {
		// How to create an asset
		fn create_asset(
			asset: T::AssetId,
			min_balance: T::Balance,
			metadata: T::AssetRegistrarMetadata,
			// Wether or not an asset-receiving account increments the sufficient counter
			is_sufficient: bool,
		) -> DispatchResult;
	}

	// We implement this trait to be able to get the AssetType and units per second registered
	impl<T: Config> xcm_primitives::AssetTypeGetter<T::AssetId, T::AssetType> for Pallet<T> {
		fn get_asset_type(asset_id: T::AssetId) -> Option<T::AssetType> {
			AssetIdType::<T>::get(asset_id)
		}

		fn get_asset_id(asset_type: T::AssetType) -> Option<T::AssetId> {
			AssetTypeId::<T>::get(asset_type)
		}
	}

	impl<T: Config> xcm_primitives::UnitsToWeightRatio<T::AssetType> for Pallet<T> {
		fn get_units_per_second(asset_type: T::AssetType) -> Option<u128> {
			AssetTypeUnitsPerSecond::<T>::get(asset_type)
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Asset Id. This will be used to register the asset in Assets
		type AssetId: Member + Parameter + Default + Copy + HasCompact + MaxEncodedLen;

		/// The Asset Metadata we want to store
		type AssetRegistrarMetadata: Member + Parameter + Default;

		/// The Asset Kind.
		type AssetType: Parameter + Member + Ord + PartialOrd + Into<Self::AssetId> + Default;

		/// The units in which we record balances.
		type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

		/// The trait we use to register Assets
		type AssetRegistrar: AssetRegistrar<Self>;

		/// Origin that is allowed to create and modify asset information
		type AssetModifierOrigin: EnsureOrigin<Self::Origin>;

		type WeightInfo: WeightInfo;
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
		/// New asset with the asset manager is registered
		AssetRegistered {
			asset_id: T::AssetId,
			asset: T::AssetType,
			metadata: T::AssetRegistrarMetadata,
		},
		/// Changed the amount of units we are charging per execution second for a given asset
		UnitsPerSecondChanged {
			asset_type: T::AssetType,
			units_per_second: u128,
		},
		/// Changed the xcm type mapping for a given asset id
		AssetTypeChanged {
			asset_id: T::AssetId,
			new_asset_type: T::AssetType,
		},
		/// Removed all information related to an assetId
		AssetRemoved {
			asset_id: T::AssetId,
			asset_type: T::AssetType,
		},
	}

	/// Mapping from an asset id to asset type.
	/// This is mostly used when receiving transaction specifying an asset directly,
	/// like transferring an asset from this chain to another.
	#[pallet::storage]
	#[pallet::getter(fn asset_id_type)]
	pub type AssetIdType<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetId, T::AssetType>;

	/// Reverse mapping of AssetIdType. Mapping from an asset type to an asset id.
	/// This is mostly used when receiving a multilocation XCM message to retrieve
	/// the corresponding asset in which tokens should me minted.
	#[pallet::storage]
	#[pallet::getter(fn asset_type_id)]
	pub type AssetTypeId<T: Config> = StorageMap<_, Blake2_128Concat, T::AssetType, T::AssetId>;

	/// Stores the units per second for local execution for a AssetType.
	/// This is used to know how to charge for XCM execution in a particular
	/// asset
	/// Not all assets might contain units per second, hence the different storage
	#[pallet::storage]
	#[pallet::getter(fn asset_type_units_per_second)]
	pub type AssetTypeUnitsPerSecond<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetType, u128>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register new asset with the asset manager
		#[pallet::weight(T::WeightInfo::register_asset())]
		pub fn register_asset(
			origin: OriginFor<T>,
			asset: T::AssetType,
			metadata: T::AssetRegistrarMetadata,
			min_amount: T::Balance,
			is_sufficient: bool,
		) -> DispatchResult {
			T::AssetModifierOrigin::ensure_origin(origin)?;

			let asset_id: T::AssetId = asset.clone().into();
			ensure!(
				AssetIdType::<T>::get(&asset_id).is_none(),
				Error::<T>::AssetAlreadyExists
			);
			T::AssetRegistrar::create_asset(asset_id, min_amount, metadata.clone(), is_sufficient)
				.map_err(|_| Error::<T>::ErrorCreatingAsset)?;

			AssetIdType::<T>::insert(&asset_id, &asset);
			AssetTypeId::<T>::insert(&asset, &asset_id);

			Self::deposit_event(Event::AssetRegistered {
				asset_id,
				asset,
				metadata,
			});
			Ok(())
		}

		/// Change the amount of units we are charging per execution second for a given AssetId
		#[pallet::weight(T::WeightInfo::set_asset_units_per_second())]
		pub fn set_asset_units_per_second(
			origin: OriginFor<T>,
			asset_type: T::AssetType,
			units_per_second: u128,
		) -> DispatchResult {
			T::AssetModifierOrigin::ensure_origin(origin)?;

			ensure!(
				AssetTypeId::<T>::get(&asset_type).is_some(),
				Error::<T>::AssetDoesNotExist
			);

			AssetTypeUnitsPerSecond::<T>::insert(&asset_type, &units_per_second);

			Self::deposit_event(Event::UnitsPerSecondChanged {
				asset_type,
				units_per_second,
			});
			Ok(())
		}

		/// Change the xcm type mapping for a given assetId
		/// We also change this if the previous units per second where pointing at the old
		/// assetType
		#[pallet::weight(T::WeightInfo::change_existing_asset_type())]
		pub fn change_existing_asset_type(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			new_asset_type: T::AssetType,
		) -> DispatchResult {
			T::AssetModifierOrigin::ensure_origin(origin)?;

			let previous_asset_type =
				AssetIdType::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			// Insert new asset type info
			AssetIdType::<T>::insert(&asset_id, &new_asset_type);
			AssetTypeId::<T>::insert(&new_asset_type, &asset_id);

			// Remove previous asset type info
			AssetTypeId::<T>::remove(&previous_asset_type);

			if let Some(units) = AssetTypeUnitsPerSecond::<T>::get(&previous_asset_type) {
				// Remove previous asset type info
				AssetTypeUnitsPerSecond::<T>::remove(&previous_asset_type);
				AssetTypeUnitsPerSecond::<T>::insert(&new_asset_type, units);
			}

			Self::deposit_event(Event::AssetTypeChanged {
				asset_id,
				new_asset_type,
			});
			Ok(())
		}

		/// Remove a given aassetId -> assetType association
		#[pallet::weight(T::WeightInfo::remove_existing_asset_type())]
		pub fn remove_existing_asset_type(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
		) -> DispatchResult {
			T::AssetModifierOrigin::ensure_origin(origin)?;

			let asset_type =
				AssetIdType::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			// Remove from AssetIdType
			AssetIdType::<T>::remove(&asset_id);
			// Remove from AssetTypeId
			AssetTypeId::<T>::remove(&asset_type);
			// Remove previous asset type units per second
			AssetTypeUnitsPerSecond::<T>::remove(&asset_type);

			Self::deposit_event(Event::AssetRemoved {
				asset_id,
				asset_type,
			});
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
