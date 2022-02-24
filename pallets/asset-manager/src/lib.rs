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
//! and control the creation of local assets
//! The assumption is we work with AssetTypes, which can then be comperted to AssetIds
//!
//! This pallet has four storage items: AssetIdType, which holds a mapping from AssetId->AssetType
//! AssetTypeUnitsPerSecond: an AssetType->u128 mapping that holds how much each AssetType should be
//! charged per unit of second, in the case such an Asset is received as a XCM asset. Finally,
//! AssetTypeId holds a mapping from AssetType -> AssetId. Finally LocalAssetCreationauthorization
//! which holds authorizations for specific accounts to create local assets
//!
//! This pallet has five extrinsics: register_asset, which registers an Asset in this pallet and
//! creates the asset as dictated by the AssetRegistrar trait. set_asset_units_per_second: which
//! sets the unit per second that should be charged for a particular asset.
//! change_existing_asset_type: which allows to update the correspondence between AssetId and
//! AssetType
//! register_local_asset: which creates a local asset with a specific owner
//! authorize_local_asset: which gives authorization to an account to register a local asset

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
	use sp_std::vec::Vec;

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
			asset: T::AssetId,
			min_balance: T::Balance,
			metadata: T::AssetRegistrarMetadata,
			// Wether or not an asset-receiving account increments the sufficient counter
			is_sufficient: bool,
		) -> DispatchResult;

		// Create a local asset, meaning an asset whose reserve chain is our chain
		// This are created as non-sufficent by default
		fn create_local_asset(
			asset: T::AssetId,
			account: T::AccountId,
			min_balance: T::Balance,
			owner: T::AccountId,
		) -> DispatchResult;
	}

	// The local asset id creator. We cannot let users choose assetIds for their assets
	// because they can look for collisions in the EVM.
	pub trait LocalAssetIdCreator<T: Config> {
		// How to create an assetId from an AccountId
		fn create_asset_id_from_account(creator: T::AccountId) -> T::AssetId;
	}

	// We implement this trait to be able to get the AssetType and units per second registered
	impl<T: Config> xcm_primitives::AssetTypeGetter<T::AssetId, T::ForeignAssetType> for Pallet<T> {
		fn get_asset_type(asset_id: T::AssetId) -> Option<T::ForeignAssetType> {
			AssetIdType::<T>::get(asset_id)
		}

		fn get_asset_id(asset_type: T::ForeignAssetType) -> Option<T::AssetId> {
			AssetTypeId::<T>::get(asset_type)
		}
	}

	impl<T: Config> xcm_primitives::UnitsToWeightRatio<T::ForeignAssetType> for Pallet<T> {
		fn payment_is_supported(asset_type: T::ForeignAssetType) -> bool {
			SupportedFeePaymentAssets::<T>::get()
				.binary_search(&asset_type)
				.is_ok()
		}
		fn get_units_per_second(asset_type: T::ForeignAssetType) -> Option<u128> {
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

		/// The Foreign Asset Kind.
		type ForeignAssetType: Parameter + Member + Ord + PartialOrd + Into<Self::AssetId> + Default;

		/// The units in which we record balances.
		type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

		/// The trait we use to register Assets
		type AssetRegistrar: AssetRegistrar<Self>;

		/// Origin that is allowed to create and modify asset information for foreign assets
		type ForeignAssetModifierOrigin: EnsureOrigin<Self::Origin>;

		/// Origin that is allowed to create and modify asset information for local assets
		type LocalAssetModifierOrigin: EnsureOrigin<Self::Origin>;

		/// Means of creating local asset Ids
		type LocalAssetIdCreator: LocalAssetIdCreator<Self>;

		type WeightInfo: WeightInfo;
	}

	/// Struct containing info about pending Local Asset to be registered
	/// Includes the owner and the minimum balance
	#[derive(Default, Clone, Encode, Decode, RuntimeDebug, PartialEq, scale_info::TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct LocalAssetInfo<T: Config> {
		pub owner: T::AccountId,
		pub min_balance: T::Balance,
	}

	/// An error that can occur while executing the mapping pallet's logic.
	#[pallet::error]
	pub enum Error<T> {
		ErrorCreatingAsset,
		AssetAlreadyExists,
		AssetDoesNotExist,
		NotAuthorizedToCreateLocalAssets,
		TooLowNumAssetsWeightHint,
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
		ForeignAssetTypeChanged {
			asset_id: T::AssetId,
			new_asset_type: T::ForeignAssetType,
		},
		/// Removed all information related to an assetId
		AssetRemoved {
			asset_id: T::AssetId,
			asset_type: T::AssetType,
		},
		/// Supported asset type for fee payment removed
		SupportedAssetRemoved { asset_type: T::ForeignAssetType },
		LocalAssetAuthorizationGiven {
			creator: T::AccountId,
			owner: T::AccountId,
			min_balance: T::Balance,
		},
		LocalAssetRegistered {
			asset_id: T::AssetId,
			creator: T::AccountId,
			owner: T::AccountId,
		},
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

	/// Stores the units per second for local execution for a AssetType.
	/// This is used to know how to charge for XCM execution in a particular
	/// asset
	/// Not all assets might contain units per second, hence the different storage
	#[pallet::storage]
	#[pallet::getter(fn asset_type_units_per_second)]
	pub type AssetTypeUnitsPerSecond<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ForeignAssetType, u128>;

	/// Stores the authorization for a particular user to be able to create a local asset
	/// This authorization needs to be given by root origin
	#[pallet::storage]
	#[pallet::getter(fn local_asset_creation_authorization)]
	pub type LocalAssetCreationAuthorization<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, LocalAssetInfo<T>>;

	// Supported fee asset payments
	#[pallet::storage]
	#[pallet::getter(fn supported_fee_payment_assets)]
	pub type SupportedFeePaymentAssets<T: Config> =
		StorageValue<_, Vec<T::ForeignAssetType>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register new asset with the asset manager
		#[pallet::weight(T::WeightInfo::register_foreign_asset())]
		pub fn register_foreign_asset(
			origin: OriginFor<T>,
			asset: T::ForeignAssetType,
			metadata: T::AssetRegistrarMetadata,
			min_amount: T::Balance,
			is_sufficient: bool,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			let asset_id: T::AssetId = asset.clone().into();
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

			AssetIdType::<T>::insert(&asset_id, &asset);
			AssetTypeId::<T>::insert(&asset, &asset_id);

			Self::deposit_event(Event::ForeignAssetRegistered {
				asset_id,
				asset,
				metadata,
			});
			Ok(())
		}

		/// Register a new local asset
		/// We need a previously given authorization to be able to create local assets
		#[pallet::weight(T::WeightInfo::register_local_asset())]
		pub fn register_local_asset(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// If no authorization was given, error
			let asset_info = LocalAssetCreationAuthorization::<T>::get(&who)
				.ok_or(Error::<T>::NotAuthorizedToCreateLocalAssets)?;

			// Create the assetId with LocalAssetIdCreator
			let asset_id = T::LocalAssetIdCreator::create_asset_id_from_account(who.clone());

			// Create local asset
			T::AssetRegistrar::create_local_asset(
				asset_id,
				who.clone(),
				asset_info.min_balance,
				asset_info.owner.clone(),
			)
			.map_err(|_| Error::<T>::ErrorCreatingAsset)?;

			// Remove the previous authorization
			LocalAssetCreationAuthorization::<T>::remove(&who);

			Self::deposit_event(Event::LocalAssetRegistered {
				asset_id,
				creator: who,
				owner: asset_info.owner,
			});
			Ok(())
		}

		/// Authorize an account to be able to create a local asset
		/// LocalAssetModifierOrigin needs to dispatch this call
		#[pallet::weight(T::WeightInfo::authorize_local_asset())]
		pub fn authorize_local_asset(
			origin: OriginFor<T>,
			creator: T::AccountId,
			owner: T::AccountId,
			min_balance: T::Balance,
		) -> DispatchResult {
			T::LocalAssetModifierOrigin::ensure_origin(origin)?;

			let local_asset_info = LocalAssetInfo::<T> {
				owner: owner.clone(),
				min_balance: min_balance.clone(),
			};

			LocalAssetCreationAuthorization::insert(&creator, local_asset_info);

			Self::deposit_event(Event::LocalAssetAuthorizationGiven {
				creator,
				owner,
				min_balance,
			});
			Ok(())
		}

		/// Change the amount of units we are charging per execution second for a given AssetType
		#[pallet::weight(T::WeightInfo::set_asset_units_per_second(*num_assets_weight_hint))]
		pub fn set_asset_units_per_second(
			origin: OriginFor<T>,
			asset_type: T::ForeignAssetType,
			units_per_second: LocalAssetCreationAuthorizationu128,
			num_assets_weight_hint: u32,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			ensure!(
				AssetTypeId::<T>::get(&asset_type).is_some(),
				Error::<T>::AssetDoesNotExist
			);

			// Grab supported assets
			let mut supported_assets = SupportedFeePaymentAssets::<T>::get();

			ensure!(
				num_assets_weight_hint >= (supported_assets.len() as u32),
				Error::<T>::TooLowNumAssetsWeightHint
			);

			// Only if the asset is not supported we need to push it
			if let Err(index) = supported_assets.binary_search(&asset_type) {
				supported_assets.insert(index, asset_type.clone());
				SupportedFeePaymentAssets::<T>::put(supported_assets);
			}

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
		#[pallet::weight(T::WeightInfo::change_existing_asset_type(*num_assets_weight_hint))]
		pub fn change_existing_asset_type(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			new_asset_type: T::ForeignAssetType,
			num_assets_weight_hint: u32,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			// Grab supported assets
			let mut supported_assets = SupportedFeePaymentAssets::<T>::get();

			ensure!(
				num_assets_weight_hint >= (supported_assets.len() as u32),
				Error::<T>::TooLowNumAssetsWeightHint
			);

			let previous_asset_type =
				AssetIdType::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			// Insert new asset type info
			AssetIdType::<T>::insert(&asset_id, &new_asset_type);
			AssetTypeId::<T>::insert(&new_asset_type, &asset_id);

			// Remove previous asset type info
			AssetTypeId::<T>::remove(&previous_asset_type);

			// Change AssetTypeUnitsPerSecond
			if let Some(units) = AssetTypeUnitsPerSecond::<T>::get(&previous_asset_type) {
				// Only if the old asset is supported we need to remove it
				if let Ok(index) = supported_assets.binary_search(&previous_asset_type) {
					supported_assets.remove(index);
				}

				// Only if the new asset is not supported we need to push it
				if let Err(index) = supported_assets.binary_search(&new_asset_type) {
					supported_assets.insert(index, new_asset_type.clone());
				}

				// Insert supported fee payment assets
				SupportedFeePaymentAssets::<T>::put(supported_assets);

				// Remove previous asset type info
				AssetTypeUnitsPerSecond::<T>::remove(&previous_asset_type);
				AssetTypeUnitsPerSecond::<T>::insert(&new_asset_type, units);
			}

			Self::deposit_event(Event::ForeignAssetTypeChanged {
				asset_id,
				new_asset_type,
			});
			Ok(())
		}

		#[pallet::weight(T::WeightInfo::remove_supported_asset(*num_assets_weight_hint))]
		pub fn remove_supported_asset(
			origin: OriginFor<T>,
			asset_type: T::ForeignAssetType,
			num_assets_weight_hint: u32,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			// Grab supported assets
			let mut supported_assets = SupportedFeePaymentAssets::<T>::get();

			ensure!(
				num_assets_weight_hint >= (supported_assets.len() as u32),
				Error::<T>::TooLowNumAssetsWeightHint
			);

			// Only if the old asset is supported we need to remove it
			if let Ok(index) = supported_assets.binary_search(&asset_type) {
				supported_assets.remove(index);
			}

			// Insert
			SupportedFeePaymentAssets::<T>::put(supported_assets);

			// Remove
			AssetTypeUnitsPerSecond::<T>::remove(&asset_type);

			Self::deposit_event(Event::SupportedAssetRemoved { asset_type });
			Ok(())
		}

		/// Remove a given assetId -> assetType association
		#[pallet::weight(T::WeightInfo::remove_existing_asset_type(*num_assets_weight_hint))]
		pub fn remove_existing_asset_type(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			num_assets_weight_hint: u32,
		) -> DispatchResult {
			T::AssetModifierOrigin::ensure_origin(origin)?;

			// Grab supported assets
			let mut supported_assets = SupportedFeePaymentAssets::<T>::get();

			ensure!(
				num_assets_weight_hint >= (supported_assets.len() as u32),
				Error::<T>::TooLowNumAssetsWeightHint
			);

			let asset_type =
				AssetIdType::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			// Remove from AssetIdType
			AssetIdType::<T>::remove(&asset_id);
			// Remove from AssetTypeId
			AssetTypeId::<T>::remove(&asset_type);
			// Remove previous asset type units per second
			AssetTypeUnitsPerSecond::<T>::remove(&asset_type);

			// Only if the old asset is supported we need to remove it
			if let Ok(index) = supported_assets.binary_search(&asset_type) {
				supported_assets.remove(index);
			}

			// Insert
			SupportedFeePaymentAssets::<T>::put(supported_assets);

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
