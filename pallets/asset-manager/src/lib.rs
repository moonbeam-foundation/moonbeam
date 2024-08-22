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

//! # Asset Manager Pallet
//!
//! This pallet allows to register new assets if certain conditions are met
//! The main goal of this pallet is to allow moonbeam to register XCM assets
//! The assumption is we work with AssetTypes, which can then be compared to AssetIds
//!
//! This pallet has five storage items: AssetIdType, which holds a mapping from AssetId->AssetType
//! AssetTypeUnitsPerSecond: an AssetType->u128 mapping that holds how much each AssetType should
//! be charged per unit of second, in the case such an Asset is received as a XCM asset. Finally,
//! AssetTypeId holds a mapping from AssetType -> AssetId.
//!
//! This pallet has eight extrinsics: register_foreign_asset, which registers a foreign
//! asset in this pallet and creates the asset as dictated by the AssetRegistrar trait.
//! change_existing_asset_type: which allows to update the correspondence between AssetId and
//! AssetType
//! remove_supported_asset: which removes an asset from the supported assets for fee payment
//! remove_existing_asset_type: which removes a mapping from a foreign asset to an assetId
//! destroy_foreign_asset: which destroys a foreign asset and all its associated data

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

pub use crate::weights::WeightInfo;

#[pallet]
pub mod pallet {
	use super::*;
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
		// How to create a foreign asset, meaning an asset whose reserve chain
		// is not our chain
		fn create_foreign_asset(
			_asset: T::AssetId,
			_min_balance: T::Balance,
			_metadata: T::AssetRegistrarMetadata,
			// Wether or not an asset-receiving account increments the sufficient counter
			_is_sufficient: bool,
		) -> DispatchResult {
			unimplemented!()
		}

		// How to destroy a foreign asset
		fn destroy_foreign_asset(_asset: T::AssetId) -> DispatchResult {
			unimplemented!()
		}

		// Get destroy asset weight dispatch info
		fn destroy_asset_dispatch_info_weight(_asset: T::AssetId) -> Weight;
	}

	// We implement this trait to be able to get the AssetType and units per second registered
	impl<T: Config> xcm_primitives::AssetTypeGetter<T::AssetId, T::ForeignAssetType> for Pallet<T> {
		fn get_asset_type(asset_id: T::AssetId) -> Option<T::ForeignAssetType> {
			AssetIdType::<T>::get(asset_id)
		}

		fn get_asset_id(asset_type: T::ForeignAssetType) -> Option<T::AssetId> {
			AssetTypeId::<T>::get(asset_type)
		}
		#[cfg(feature = "runtime-benchmarks")]
		fn set_asset_type_asset_id(asset_type: T::ForeignAssetType, asset_id: T::AssetId) {
			AssetTypeId::<T>::insert(&asset_type, asset_id);
			AssetIdType::<T>::insert(&asset_id, asset_type);
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The Asset Id. This will be used to create the asset and to associate it with
		/// a assetType
		type AssetId: Member + Parameter + Default + Copy + HasCompact + MaxEncodedLen;

		/// The Asset Metadata we want to store
		type AssetRegistrarMetadata: Member + Parameter + Default;

		/// The Foreign Asset Kind.
		type ForeignAssetType: Parameter + Member + Ord + PartialOrd + Into<Self::AssetId> + Default;

		/// The units in which we record balances.
		type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

		/// The asset Registrar.
		/// The trait we use to register Assets
		type AssetRegistrar: AssetRegistrar<Self>;

		/// Origin that is allowed to create and modify asset information for foreign assets
		type ForeignAssetModifierOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		type WeightInfo: WeightInfo;
	}

	/// An error that can occur while executing the mapping pallet's logic.
	#[pallet::error]
	pub enum Error<T> {
		ErrorCreatingAsset,
		AssetAlreadyExists,
		AssetDoesNotExist,
		TooLowNumAssetsWeightHint,
		LocalAssetLimitReached,
		ErrorDestroyingAsset,
		NotSufficientDeposit,
		NonExistentLocalAsset,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New asset with the asset manager is registered
		ForeignAssetRegistered {
			asset_id: T::AssetId,
			asset: T::ForeignAssetType,
			metadata: T::AssetRegistrarMetadata,
		},
		/// Changed the amount of units we are charging per execution second for a given asset
		UnitsPerSecondChanged {
			asset_type: T::ForeignAssetType,
			units_per_second: u128,
		},
		/// Changed the xcm type mapping for a given asset id
		ForeignAssetXcmLocationChanged {
			asset_id: T::AssetId,
			new_asset_type: T::ForeignAssetType,
		},
		/// Removed all information related to an assetId
		ForeignAssetRemoved {
			asset_id: T::AssetId,
			asset_type: T::ForeignAssetType,
		},
		/// Supported asset type for fee payment removed
		SupportedAssetRemoved { asset_type: T::ForeignAssetType },
		/// Removed all information related to an assetId and destroyed asset
		ForeignAssetDestroyed {
			asset_id: T::AssetId,
			asset_type: T::ForeignAssetType,
		},
		/// Removed all information related to an assetId and destroyed asset
		LocalAssetDestroyed { asset_id: T::AssetId },
	}

	/// Mapping from an asset id to asset type.
	/// This is mostly used when receiving transaction specifying an asset directly,
	/// like transferring an asset from this chain to another.
	#[pallet::storage]
	#[pallet::getter(fn asset_id_type)]
	pub type AssetIdType<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetId, T::ForeignAssetType>;

	/// Reverse mapping of AssetIdType. Mapping from an asset type to an asset id.
	/// This is mostly used when receiving a multilocation XCM message to retrieve
	/// the corresponding asset in which tokens should me minted.
	#[pallet::storage]
	#[pallet::getter(fn asset_type_id)]
	pub type AssetTypeId<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ForeignAssetType, T::AssetId>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register new asset with the asset manager
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::register_foreign_asset())]
		pub fn register_foreign_asset(
			origin: OriginFor<T>,
			asset: T::ForeignAssetType,
			metadata: T::AssetRegistrarMetadata,
			min_amount: T::Balance,
			is_sufficient: bool,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			// Compute assetId from asset
			let asset_id: T::AssetId = asset.clone().into();

			// Ensure such an assetId does not exist
			ensure!(
				AssetIdType::<T>::get(&asset_id).is_none(),
				Error::<T>::AssetAlreadyExists
			);
			T::AssetRegistrar::create_foreign_asset(
				asset_id,
				min_amount,
				metadata.clone(),
				is_sufficient,
			)
			.map_err(|_| Error::<T>::ErrorCreatingAsset)?;

			// Insert the association assetId->assetType
			AssetIdType::<T>::insert(&asset_id, &asset);
			AssetTypeId::<T>::insert(&asset, &asset_id);

			Self::deposit_event(Event::ForeignAssetRegistered {
				asset_id,
				asset,
				metadata,
			});
			Ok(())
		}

		/// Change the xcm type mapping for a given assetId
		/// We also change this if the previous units per second where pointing at the old
		/// assetType
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::change_existing_asset_type())]
		pub fn change_existing_asset_type(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			new_asset_type: T::ForeignAssetType,
			_num_assets_weight_hint: u32,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			let previous_asset_type =
				AssetIdType::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			// Insert new asset type info
			AssetIdType::<T>::insert(&asset_id, &new_asset_type);
			AssetTypeId::<T>::insert(&new_asset_type, &asset_id);

			// Remove previous asset type info
			AssetTypeId::<T>::remove(&previous_asset_type);

			Self::deposit_event(Event::ForeignAssetXcmLocationChanged {
				asset_id,
				new_asset_type,
			});
			Ok(())
		}

		/// Remove a given assetId -> assetType association
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::remove_existing_asset_type())]
		pub fn remove_existing_asset_type(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			_num_assets_weight_hint: u32,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			let asset_type =
				AssetIdType::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			// Remove from AssetIdType
			AssetIdType::<T>::remove(&asset_id);
			// Remove from AssetTypeId
			AssetTypeId::<T>::remove(&asset_type);

			Self::deposit_event(Event::ForeignAssetRemoved {
				asset_id,
				asset_type,
			});
			Ok(())
		}

		/// Destroy a given foreign assetId
		/// The weight in this case is the one returned by the trait
		/// plus the db writes and reads from removing all the associated
		/// data
		#[pallet::call_index(6)]
		#[pallet::weight({
			let dispatch_info_weight = T::AssetRegistrar::destroy_asset_dispatch_info_weight(
				*asset_id
			);
			T::WeightInfo::remove_existing_asset_type()
			.saturating_add(dispatch_info_weight)
		})]
		pub fn destroy_foreign_asset(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			_num_assets_weight_hint: u32,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			T::AssetRegistrar::destroy_foreign_asset(asset_id)
				.map_err(|_| Error::<T>::ErrorDestroyingAsset)?;

			let asset_type =
				AssetIdType::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			// Remove from AssetIdType
			AssetIdType::<T>::remove(&asset_id);
			// Remove from AssetTypeId
			AssetTypeId::<T>::remove(&asset_type);

			Self::deposit_event(Event::ForeignAssetDestroyed {
				asset_id,
				asset_type,
			});
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// The account ID of AssetManager
		pub fn account_id() -> T::AccountId {
			PALLET_ID.into_account_truncating()
		}
	}
}
