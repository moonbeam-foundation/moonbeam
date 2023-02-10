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
//! and control the creation of local assets
//! The assumption is we work with AssetTypes, which can then be compared to AssetIds
//!
//! This pallet has five storage items: AssetIdType, which holds a mapping from AssetId->AssetType
//! AssetTypeUnitsPerSecond: an AssetType->u128 mapping that holds how much each AssetType should
//! be charged per unit of second, in the case such an Asset is received as a XCM asset. Finally,
//! AssetTypeId holds a mapping from AssetType -> AssetId. LocalAssetCounter
//! which holds the counter of local assets that have been created so far. And LocalAssetDeposit,
//! which holds a mapping between assetId and assetInfo, i.e., the asset creator (from which
//! we take the deposit) and the deposit amount itself.
//!
//! This pallet has eight extrinsics: register_foreign_asset, which registers a foreign
//! asset in this pallet and creates the asset as dictated by the AssetRegistrar trait.
//! set_asset_units_per_second: which sets the unit per second that should be charged for
//! a particular asset.
//! change_existing_asset_type: which allows to update the correspondence between AssetId and
//! AssetType
//! remove_supported_asset: which removes an asset from the supported assets for fee payment
//! remove_existing_asset_type: which removes a mapping from a foreign asset to an assetId
//! register_local_asset: which creates a local asset with a specific owner
//! destroy_foreign_asset: which destroys a foreign asset and all its associated data
//! destroy_local_asset: which destroys a local asset and all its associated data

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
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ReservableCurrency},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::HasCompact;
	use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned};
	use sp_std::vec::Vec;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// The AssetManagers's pallet id
	pub const PALLET_ID: PalletId = PalletId(*b"asstmngr");

	pub(crate) type DepositBalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Default, Clone, Encode, Decode, RuntimeDebug, PartialEq, scale_info::TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct AssetInfo<T: Config> {
		pub creator: T::AccountId,
		pub deposit: DepositBalanceOf<T>,
	}

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

		// Create a local asset, meaning an asset whose reserve chain is our chain
		// These are created as non-sufficent by default
		fn create_local_asset(
			_asset: T::AssetId,
			_account: T::AccountId,
			_min_balance: T::Balance,
			_is_sufficient: bool,
			_owner: T::AccountId,
		) -> DispatchResult {
			unimplemented!()
		}

		// How to destroy a foreign asset
		fn destroy_foreign_asset(_asset: T::AssetId) -> DispatchResult {
			unimplemented!()
		}

		// How to destroy a local asset
		fn destroy_local_asset(_asset: T::AssetId) -> DispatchResult {
			unimplemented!()
		}

		// Get destroy asset weight dispatch info
		fn destroy_asset_dispatch_info_weight(_asset: T::AssetId) -> Weight;
	}

	// The local asset id creator. We cannot let users choose assetIds for their assets
	// because they can look for collisions in the EVM.
	pub trait LocalAssetIdCreator<T: Config> {
		// How to create an assetId from the local asset counter
		fn create_asset_id_from_metadata(local_asset_counter: u128) -> T::AssetId;
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

	impl<T: Config> xcm_primitives::UnitsToWeightRatio<T::ForeignAssetType> for Pallet<T> {
		fn payment_is_supported(asset_type: T::ForeignAssetType) -> bool {
			SupportedFeePaymentAssets::<T>::get()
				.binary_search(&asset_type)
				.is_ok()
		}
		fn get_units_per_second(asset_type: T::ForeignAssetType) -> Option<u128> {
			AssetTypeUnitsPerSecond::<T>::get(asset_type)
		}
		#[cfg(feature = "runtime-benchmarks")]
		fn set_units_per_second(asset_type: T::ForeignAssetType, fee_per_second: u128) {
			// Grab supported assets
			let mut supported_assets = SupportedFeePaymentAssets::<T>::get();
			// Only if the asset is not supported we need to push it
			if let Err(index) = supported_assets.binary_search(&asset_type) {
				supported_assets.insert(index, asset_type.clone());
				SupportedFeePaymentAssets::<T>::put(supported_assets);
			}
			AssetTypeUnitsPerSecond::<T>::insert(&asset_type, &fee_per_second);
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

		/// Origin that is allowed to create and modify asset information for local assets
		type LocalAssetModifierOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Ways of creating local asset Ids
		type LocalAssetIdCreator: LocalAssetIdCreator<Self>;

		/// The currency mechanism in which we reserve deposits for local assets.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The basic amount of funds that must be reserved for a local asset.
		#[pallet::constant]
		type LocalAssetDeposit: Get<DepositBalanceOf<Self>>;

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
		ForeignAssetTypeChanged {
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
		/// Local asset was created
		LocalAssetRegistered {
			asset_id: T::AssetId,
			creator: T::AccountId,
			owner: T::AccountId,
		},
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

	/// Stores the units per second for local execution for a AssetType.
	/// This is used to know how to charge for XCM execution in a particular
	/// asset
	/// Not all assets might contain units per second, hence the different storage
	#[pallet::storage]
	#[pallet::getter(fn asset_type_units_per_second)]
	pub type AssetTypeUnitsPerSecond<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ForeignAssetType, u128>;

	/// Stores the counter of the number of local assets that have been
	/// created so far
	/// This value can be used to salt the creation of an assetId, e.g.,
	/// by hashing it. This is particularly useful for cases like moonbeam
	/// where letting users choose their assetId would result in collision
	/// in the evm side.
	#[pallet::storage]
	#[pallet::getter(fn local_asset_counter)]
	pub type LocalAssetCounter<T: Config> = StorageValue<_, u128, ValueQuery>;

	/// Local asset deposits, a mapping from assetId to a struct
	/// holding the creator (from which the deposit was reserved) and
	/// the deposit amount
	#[pallet::storage]
	#[pallet::getter(fn local_asset_deposit)]
	pub type LocalAssetDeposit<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AssetId, AssetInfo<T>>;

	// Supported fee asset payments
	#[pallet::storage]
	#[pallet::getter(fn supported_fee_payment_assets)]
	pub type SupportedFeePaymentAssets<T: Config> =
		StorageValue<_, Vec<T::ForeignAssetType>, ValueQuery>;

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

		/// Change the amount of units we are charging per execution second
		/// for a given ForeignAssetType
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::set_asset_units_per_second(*num_assets_weight_hint))]
		pub fn set_asset_units_per_second(
			origin: OriginFor<T>,
			asset_type: T::ForeignAssetType,
			units_per_second: u128,
			num_assets_weight_hint: u32,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			// Ensure such an assetId does not exist
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
		#[pallet::call_index(2)]
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

		#[pallet::call_index(3)]
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
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::remove_existing_asset_type(*num_assets_weight_hint))]
		pub fn remove_existing_asset_type(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			num_assets_weight_hint: u32,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

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
				// Insert
				SupportedFeePaymentAssets::<T>::put(supported_assets);
			}

			Self::deposit_event(Event::ForeignAssetRemoved {
				asset_id,
				asset_type,
			});
			Ok(())
		}

		/// Register a new local asset
		/// No information is stored in this pallet about the local asset
		/// The reason is that we dont need to hold a mapping between the multilocation
		/// and the local asset, as this conversion is deterministic
		/// Further, we dont allow xcm fee payment in local assets
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::register_local_asset())]
		pub fn register_local_asset(
			origin: OriginFor<T>,
			creator: T::AccountId,
			owner: T::AccountId,
			is_sufficient: bool,
			min_balance: T::Balance,
		) -> DispatchResult {
			T::LocalAssetModifierOrigin::ensure_origin(origin)?;

			// Get the deposit amount
			let deposit = T::LocalAssetDeposit::get();

			// Verify we can reserve
			T::Currency::can_reserve(&creator, deposit)
				.then(|| true)
				.ok_or(Error::<T>::NotSufficientDeposit)?;

			// Read Local Asset Counter
			let mut local_asset_counter = LocalAssetCounter::<T>::get();

			// Create the assetId with LocalAssetIdCreator
			let asset_id =
				T::LocalAssetIdCreator::create_asset_id_from_metadata(local_asset_counter);

			// Increment the counter
			local_asset_counter = local_asset_counter
				.checked_add(1)
				.ok_or(Error::<T>::LocalAssetLimitReached)?;

			// Create local asset
			T::AssetRegistrar::create_local_asset(
				asset_id,
				creator.clone(),
				min_balance,
				is_sufficient,
				owner.clone(),
			)
			.map_err(|_| Error::<T>::ErrorCreatingAsset)?;

			// Reserve the deposit, we verified we can do this
			T::Currency::reserve(&creator, deposit)?;

			// Update assetInfo
			LocalAssetDeposit::<T>::insert(
				asset_id,
				AssetInfo {
					creator: creator.clone(),
					deposit,
				},
			);

			// Update local asset counter
			LocalAssetCounter::<T>::put(local_asset_counter);

			Self::deposit_event(Event::LocalAssetRegistered {
				asset_id,
				creator,
				owner,
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
			T::WeightInfo::remove_existing_asset_type(*num_assets_weight_hint)
			.saturating_add(dispatch_info_weight)
		})]
		pub fn destroy_foreign_asset(
			origin: OriginFor<T>,
			asset_id: T::AssetId,
			num_assets_weight_hint: u32,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			T::AssetRegistrar::destroy_foreign_asset(asset_id)
				.map_err(|_| Error::<T>::ErrorDestroyingAsset)?;

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
				// Insert
				SupportedFeePaymentAssets::<T>::put(supported_assets);
			}

			Self::deposit_event(Event::ForeignAssetDestroyed {
				asset_id,
				asset_type,
			});
			Ok(())
		}

		/// Destroy a given local assetId
		/// We do not store anything related to local assets in this pallet other than the counter
		/// and the counter is not used for destroying the asset, so no additional db reads/writes
		/// to be counter here
		#[pallet::call_index(7)]
		#[pallet::weight({
			T::AssetRegistrar::destroy_asset_dispatch_info_weight(
				*asset_id
			)
			.saturating_add(T::DbWeight::get().reads_writes(2, 2))
		})]
		pub fn destroy_local_asset(origin: OriginFor<T>, asset_id: T::AssetId) -> DispatchResult {
			T::LocalAssetModifierOrigin::ensure_origin(origin)?;

			// Get asset creator and deposit amount
			let asset_info =
				LocalAssetDeposit::<T>::get(asset_id).ok_or(Error::<T>::NonExistentLocalAsset)?;

			// Destroy local asset
			T::AssetRegistrar::destroy_local_asset(asset_id)
				.map_err(|_| Error::<T>::ErrorDestroyingAsset)?;

			// Unreserve deposit
			T::Currency::unreserve(&asset_info.creator, asset_info.deposit);

			// Remove asset info
			LocalAssetDeposit::<T>::remove(asset_id);

			Self::deposit_event(Event::LocalAssetDestroyed { asset_id });
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
