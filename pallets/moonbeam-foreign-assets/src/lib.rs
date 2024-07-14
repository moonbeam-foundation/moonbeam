// Copyright Moonsong Labs
// This file is part of Moonkit.

// Moonkit is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonkit is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonkit.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use weights::WeightInfo;

use frame_support::pallet;

#[cfg(any(test, feature = "runtime-benchmarks"))]
pub mod benchmarks;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
pub mod tests;
pub mod weights;

/// Trait for the OnForeignAssetRegistered hook
pub trait ForeignAssetCreatedHook<ForeignAsset, AssetId, AssetBalance> {
	fn on_asset_created(foreign_asset: &ForeignAsset, asset_id: &AssetId);
}

impl<ForeignAsset, AssetId, AssetBalance>
	ForeignAssetCreatedHook<ForeignAsset, AssetId, AssetBalance> for ()
{
	fn on_asset_created(_foreign_asset: &ForeignAsset, _asset_id: &AssetId) {}
}

/// Trait for the OnForeignAssetDeregistered hook
pub trait ForeignAssetDestroyedHook<ForeignAsset, AssetId> {
	fn on_asset_destroyed(foreign_asset: &ForeignAsset, asset_id: &AssetId);
}

impl<ForeignAsset, AssetId> ForeignAssetDestroyedHook<ForeignAsset, AssetId> for () {
	fn on_asset_destroyed(_foreign_asset: &ForeignAsset, _asset_id: &AssetId) {}
}

#[pallet]
pub mod pallet {
	use super::*;
	use ethereum_types::{H160, U256};
	use fp_evm::{ExitReason, ExitSucceed};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_evm::{GasWeightMapping, Runner};
	use sp_runtime::traits::{AccountIdConversion, MaybeEquivalence};

	const ERC20_CREATE_INIT_CODE_MAX_SIZE: usize = 16 * 1024;
	const FOREIGN_ASSETS_PREFIX: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
	const FOREIGN_ASSET_ERC20_CREATE_GAS_LIMIT: u64 = 500_000;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// The moonbeam foreign assets's pallet id
	pub const PALLET_ID: frame_support::PalletId = frame_support::PalletId(*b"forgasst");

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		type AccountId: Parameter + Into<H160> + IsType<<Self as frame_system::Config>::AccountId>;

		/// EVM runner
		type EvmRunner: Runner<Self>;

		/// The Foreign Asset Kind.
		type ForeignAsset: Parameter + Member + Ord + PartialOrd;

		/// Origin that is allowed to create a new foreign assets
		type ForeignAssetCreatorOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Origin that is allowed to modify asset information for foreign assets
		type ForeignAssetModifierOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Origin that is allowed to freeze all tokens of a foreign asset
		type ForeignAssetFreezerOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Origin that is allowed to unfreeze all tokens of a foreign asset that was previously
		/// frozen
		type ForeignAssetUnfreezerOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Origin that is allowed to create and modify asset information for foreign assets
		type ForeignAssetDestroyerOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Hook to be called when new foreign asset is registered.
		type OnForeignAssetCreated: ForeignAssetCreatedHook<
			Self::ForeignAsset,
			AssetId,
			AssetBalance,
		>;

		/// Hook to be called when foreign asset is de-registered.
		type OnForeignAssetDestroyed: ForeignAssetDestroyedHook<Self::ForeignAsset, AssetId>;

		/// Maximum nulmbers of differnt foreign assets
		type MaxForeignAssets: Get<u32>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	pub type AssetBalance = U256;
	pub type AssetId = u128;

	/// An error that can occur while executing the mapping pallet's logic.
	#[pallet::error]
	pub enum Error<T> {
		AssetAlreadyExists,
		AssetDoesNotExist,
		AssetAlreadyFrozen,
		AssetNotFrozen,
		Erc20ContractCreationFail,
		TooManyForeignAssets,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New asset with the asset manager is registered
		ForeignAssetCreated {
			asset_id: AssetId,
			foreign_asset: T::ForeignAsset,
		},
		/// Changed the xcm type mapping for a given asset id
		ForeignAssetTypeChanged {
			asset_id: AssetId,
			new_foreign_asset: T::ForeignAsset,
		},
		/// Removed all information related to an assetId
		ForeignAssetRemoved {
			asset_id: AssetId,
			foreign_asset: T::ForeignAsset,
		},
		// Freezes all tokens of a given asset id
		ForeignAssetFrozen {
			asset_id: AssetId,
			foreign_asset: T::ForeignAsset,
		},
		// Thawing a previously frozen asset
		ForeignAssetUnfrozen {
			asset_id: AssetId,
			foreign_asset: T::ForeignAsset,
		},
		/// Removed all information related to an assetId and destroyed asset
		ForeignAssetDestroyed {
			asset_id: AssetId,
			foreign_asset: T::ForeignAsset,
		},
	}

	/// Mapping from an asset id to a Foreign asset type.
	/// This is mostly used when receiving transaction specifying an asset directly,
	/// like transferring an asset from this chain to another.
	#[pallet::storage]
	#[pallet::getter(fn foreign_asset_for_id)]
	pub type AssetIdToForeignAsset<T: Config> =
		CountedStorageMap<_, Blake2_128Concat, AssetId, T::ForeignAsset, OptionQuery>;

	/// Reverse mapping of AssetIdToForeignAsset. Mapping from a foreign asset to an asset id.
	/// This is mostly used when receiving a multilocation XCM message to retrieve
	/// the corresponding asset in which tokens should me minted.
	#[pallet::storage]
	#[pallet::getter(fn asset_id_for_foreign)]
	pub type ForeignAssetToAssetId<T: Config> =
		StorageMap<_, Blake2_128Concat, T::ForeignAsset, AssetId>;

	#[pallet::storage]
	#[pallet::getter(fn frozen_assets)]
	pub type FrozenAssets<T: Config> =
		StorageValue<_, BoundedVec<AssetId, T::MaxForeignAssets>, ValueQuery>;

	impl<T: Config> Pallet<T> {
		/// The account ID of this pallet
		pub fn account_id() -> H160 {
			let account_id: <T as Config>::AccountId = PALLET_ID.into_account_truncating();
			account_id.into()
		}

		/// Deploy foreign asset erc20 contract
		fn create_erc20_contract(asset_id: AssetId, decimals: u8) -> Result<(), Error<T>> {
			// Get init code
			let mut init = Vec::with_capacity(ERC20_CREATE_INIT_CODE_MAX_SIZE);
			init.extend_from_slice(include_bytes!("../resources/foreign_erc20_initcode.bin"));

			// Add constructor parameters
			// (0x6D6f646c617373746d6E67720000000000000000, 18, MTT, MyBigToken)
			//0x0000000000000000000000006d6f646c617373746d6e677200000000000000000000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000034d54540000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a4d79426967546f6b656e00000000000000000000000000000000000000000000

			// Compute contract address
			let mut buffer = [0u8; 20];
			buffer[..4].copy_from_slice(&FOREIGN_ASSETS_PREFIX);
			buffer[4..].copy_from_slice(&asset_id.to_be_bytes());
			let contract_address = H160(buffer);

			let exec_info = T::EvmRunner::create_force_address(
				Self::account_id(),
				init,
				U256::default(),
				FOREIGN_ASSET_ERC20_CREATE_GAS_LIMIT,
				None,
				None,
				None,
				Default::default(),
				true,
				false,
				None,
				None,
				&<T as pallet_evm::Config>::config(),
				contract_address,
			)
			.map_err(|_| Error::Erc20ContractCreationFail)?;

			ensure!(
				matches!(
					exec_info.exit_reason,
					ExitReason::Succeed(ExitSucceed::Returned | ExitSucceed::Stopped)
				),
				Error::Erc20ContractCreationFail
			);

			Ok(())
		}

		// Call contract selector "pause"
		fn pause(asset_id: AssetId) -> Result<(), Error<T>> {
			todo!()
		}

		// Call contract selector "unpause"
		fn unpause(asset_id: AssetId) -> Result<(), Error<T>> {
			todo!()
		}

		fn start_destroy(asset_id: AssetId) -> Result<(), Error<T>> {
			// Freeze asset
			Self::pause(asset_id)?;

			todo!()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create new asset with the ForeignAssetCreator
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::create_foreign_asset())]
		pub fn create_foreign_asset(
			origin: OriginFor<T>,
			foreign_asset: T::ForeignAsset,
			asset_id: AssetId,
			decimals: u8,
		) -> DispatchResult {
			T::ForeignAssetCreatorOrigin::ensure_origin(origin)?;

			// Ensure such an assetId does not exist
			ensure!(
				AssetIdToForeignAsset::<T>::get(&asset_id).is_none(),
				Error::<T>::AssetAlreadyExists
			);

			ensure!(
				AssetIdToForeignAsset::<T>::count() < T::MaxForeignAssets::get(),
				Error::<T>::TooManyForeignAssets
			);

			// TODO submit create eth-xcm call
			Self::create_erc20_contract(asset_id, decimals)?;

			// Insert the association assetId->foreigAsset
			// Insert the association foreigAsset->assetId
			AssetIdToForeignAsset::<T>::insert(&asset_id, &foreign_asset);
			ForeignAssetToAssetId::<T>::insert(&foreign_asset, &asset_id);

			T::OnForeignAssetCreated::on_asset_created(&foreign_asset, &asset_id);

			Self::deposit_event(Event::ForeignAssetCreated {
				asset_id,
				foreign_asset,
			});
			Ok(())
		}

		/// Change the xcm type mapping for a given assetId
		/// We also change this if the previous units per second where pointing at the old
		/// assetType
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::change_existing_asset_type())]
		pub fn change_existing_asset_type(
			origin: OriginFor<T>,
			asset_id: AssetId,
			new_foreign_asset: T::ForeignAsset,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			let previous_foreign_asset =
				AssetIdToForeignAsset::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			// Insert new foreign asset info
			AssetIdToForeignAsset::<T>::insert(&asset_id, &new_foreign_asset);
			ForeignAssetToAssetId::<T>::insert(&new_foreign_asset, &asset_id);

			// Remove previous foreign asset info
			ForeignAssetToAssetId::<T>::remove(&previous_foreign_asset);

			Self::deposit_event(Event::ForeignAssetTypeChanged {
				asset_id,
				new_foreign_asset,
			});
			Ok(())
		}

		/// Remove a given assetId -> foreignAsset association
		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::remove_existing_asset_type())]
		pub fn remove_existing_asset_type(
			origin: OriginFor<T>,
			asset_id: AssetId,
		) -> DispatchResult {
			T::ForeignAssetDestroyerOrigin::ensure_origin(origin)?;

			let foreign_asset =
				AssetIdToForeignAsset::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			// Remove from AssetIdToForeignAsset
			AssetIdToForeignAsset::<T>::remove(&asset_id);
			// Remove from ForeignAssetToAssetId
			ForeignAssetToAssetId::<T>::remove(&foreign_asset);

			Self::deposit_event(Event::ForeignAssetRemoved {
				asset_id,
				foreign_asset,
			});
			Ok(())
		}

		/// Freeze a given foreign assetId
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::destroy_foreign_asset())]
		pub fn freeze_foreign_asset(origin: OriginFor<T>, asset_id: AssetId) -> DispatchResult {
			T::ForeignAssetFreezerOrigin::ensure_origin(origin)?;

			let foreign_asset =
				AssetIdToForeignAsset::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			let mut frozen_assets: Vec<_> = FrozenAssets::<T>::get().into();
			let index = match frozen_assets.binary_search_by(|i| i.cmp(&asset_id)) {
				Ok(_) => return Err(Error::<T>::AssetAlreadyFrozen.into()),
				Err(index) => index,
			};

			Self::pause(asset_id.clone())?;

			frozen_assets.insert(index, asset_id);
			let frozen_assets_bounded: BoundedVec<_, T::MaxForeignAssets> = frozen_assets
				.try_into()
				.map_err(|_| Error::<T>::TooManyForeignAssets)?;
			FrozenAssets::<T>::put(frozen_assets_bounded);

			Self::deposit_event(Event::ForeignAssetFrozen {
				asset_id,
				foreign_asset,
			});
			Ok(())
		}

		/// Freeze a given foreign assetId
		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::destroy_foreign_asset())]
		pub fn unfreeze_foreign_asset(origin: OriginFor<T>, asset_id: AssetId) -> DispatchResult {
			T::ForeignAssetUnfreezerOrigin::ensure_origin(origin)?;

			let foreign_asset =
				AssetIdToForeignAsset::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			let mut frozen_assets: Vec<_> = FrozenAssets::<T>::get().into();
			let index = match frozen_assets.binary_search_by(|i| i.cmp(&asset_id)) {
				Ok(index) => index,
				Err(_) => return Err(Error::<T>::AssetNotFrozen.into()),
			};

			Self::unpause(asset_id.clone())?;

			frozen_assets.remove(index);
			let frozen_assets_bounded: BoundedVec<_, T::MaxForeignAssets> = frozen_assets
				.try_into()
				.map_err(|_| Error::<T>::TooManyForeignAssets)?;
			FrozenAssets::<T>::put(frozen_assets_bounded);

			Self::deposit_event(Event::ForeignAssetUnfrozen {
				asset_id,
				foreign_asset,
			});
			Ok(())
		}

		/// Destroy a given foreign assetId
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::destroy_foreign_asset())]
		pub fn destroy_foreign_asset(origin: OriginFor<T>, asset_id: AssetId) -> DispatchResult {
			T::ForeignAssetDestroyerOrigin::ensure_origin(origin)?;

			let foreign_asset =
				AssetIdToForeignAsset::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			// Important: this starts the destruction process,
			// making sure the assets are non-transferable anymore
			// make sure the destruction process is completable by other means
			Self::start_destroy(asset_id.clone())?;

			// Remove from AssetIdToForeignAsset
			AssetIdToForeignAsset::<T>::remove(&asset_id);
			// Remove from ForeignAssetToAssetId
			ForeignAssetToAssetId::<T>::remove(&foreign_asset);

			T::OnForeignAssetDestroyed::on_asset_destroyed(&foreign_asset, &asset_id);

			Self::deposit_event(Event::ForeignAssetDestroyed {
				asset_id,
				foreign_asset,
			});
			Ok(())
		}
	}

	impl<T: Config> MaybeEquivalence<T::ForeignAsset, AssetId> for Pallet<T> {
		fn convert(foreign_asset: &T::ForeignAsset) -> Option<AssetId> {
			Pallet::<T>::asset_id_for_foreign(foreign_asset.clone())
		}
		fn convert_back(id: &AssetId) -> Option<T::ForeignAsset> {
			Pallet::<T>::foreign_asset_for_id(id.clone())
		}
	}
}
