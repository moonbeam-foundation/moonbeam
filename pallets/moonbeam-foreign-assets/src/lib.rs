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

//! # Moonbeam Foreign Assets pallet
//!
//! This pallets allow to create and manage XCM derivative assets (aka. foreign assets).
//!
//! Each asset is implemented by an evm smart contract that is deployed by this pallet
//! The evm smart contract for each asset is trusted by the runtime, and should
//! be deployed only by the runtime itself.
//!
//! This pallet made several assumptions on theses evm smarts contracts:
//! - Only this pallet should be able to mint and burn tokens
//! - The following selectors should be exposed and callable only by this pallet account:
//!   - burnFrom(address, uint256)
//!   - mintInto(address, uint256)
//!   - pause(address, uint256)
//!   - unpause(address, uint256)
//! - The smart contract should expose as weel the ERC20.transfer selector
//!
//! Each asset has a unique identifier that can never change.
//! This identifier is named "AssetId", it's an integer (u128).
//! This pallet maintain a two-way mapping beetween each AssetId the XCM Location of the asset.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(test, feature = "runtime-benchmarks"))]
pub mod benchmarks;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
pub mod tests;
pub mod weights;

mod evm;

pub use pallet::*;
pub use weights::WeightInfo;

use self::evm::EvmCaller;
use ethereum_types::{H160, U256};
use frame_support::pallet;
use frame_support::pallet_prelude::*;
use frame_support::traits::Contains;
use frame_system::pallet_prelude::*;
use xcm::latest::{
	Asset, AssetId as XcmAssetId, Error as XcmError, Fungibility, Location, Result as XcmResult,
	XcmContext,
};
use xcm_executor::traits::Error as MatchError;

const FOREIGN_ASSETS_PREFIX: [u8; 4] = [0xff, 0xff, 0xff, 0xff];

/// Trait for the OnForeignAssetRegistered hook
pub trait ForeignAssetCreatedHook<ForeignAsset> {
	fn on_asset_created(foreign_asset: &ForeignAsset, asset_id: &AssetId);
}

impl<ForeignAsset> ForeignAssetCreatedHook<ForeignAsset> for () {
	fn on_asset_created(_foreign_asset: &ForeignAsset, _asset_id: &AssetId) {}
}

/// Trait for the OnForeignAssetDeregistered hook
pub trait ForeignAssetDestroyedHook<ForeignAsset> {
	fn on_asset_destroyed(foreign_asset: &ForeignAsset, asset_id: &AssetId);
}

impl<ForeignAsset> ForeignAssetDestroyedHook<ForeignAsset> for () {
	fn on_asset_destroyed(_foreign_asset: &ForeignAsset, _asset_id: &AssetId) {}
}

pub(crate) struct ForeignAssetsMatcher<T>(core::marker::PhantomData<T>);

impl<T: crate::Config> ForeignAssetsMatcher<T> {
	fn match_asset(asset: &Asset) -> Result<(H160, U256, AssetStatus), MatchError> {
		let (amount, location) = match (&asset.fun, &asset.id) {
			(Fungibility::Fungible(ref amount), XcmAssetId(ref location)) => (amount, location),
			_ => return Err(MatchError::AssetNotHandled),
		};

		if let Some((asset_id, asset_status)) = AssetsByLocation::<T>::get(&location) {
			Ok((
				Pallet::<T>::contract_address_from_asset_id(asset_id),
				U256::from(*amount),
				asset_status,
			))
		} else {
			Err(MatchError::AssetNotHandled)
		}
	}
}

#[derive(Decode, Encode, PartialEq, TypeInfo)]
#[cfg_attr(test, derive(Debug))]
pub enum AssetStatus {
	/// All operations are enabled
	Active,
	/// The asset is frozen, but deposit from XCM still work
	FrozenXcmDepositAllowed,
	/// The asset is frozen, and deposit from XCM will fail
	FrozenXcmDepositForbidden,
}

#[pallet]
pub mod pallet {
	use super::*;
	use pallet_evm::Runner;
	use sp_runtime::traits::AccountIdConversion;
	use xcm_executor::traits::ConvertLocation;
	use xcm_executor::traits::Error as MatchError;
	use xcm_executor::AssetsInHolding;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// The moonbeam foreign assets's pallet id
	pub const PALLET_ID: frame_support::PalletId = frame_support::PalletId(*b"forgasst");

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		/// the overarching AccountId type
		type AccountId: Parameter + Into<H160> + IsType<<Self as frame_system::Config>::AccountId>;

		// Convert XCM Location to H160
		type AccountIdConverter: ConvertLocation<H160>;

		/// A filter to forbid some AssetId values, if you don't use it, put "Everything"
		type AssetIdFilter: Contains<AssetId>;

		/// EVM runner
		type EvmRunner: Runner<Self>;

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
		type OnForeignAssetCreated: ForeignAssetCreatedHook<Location>;

		/// Hook to be called when foreign asset is de-registered.
		type OnForeignAssetDestroyed: ForeignAssetDestroyedHook<Location>;

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
		AssetAlreadyFrozen,
		AssetDoesNotExist,
		AssetIdFiltered,
		AssetNotFrozen,
		CorruptedStorageOrphanLocation,
		Erc20ContractCreationFail,
		EvmCallPauseFail,
		EvmCallUnpauseFail,
		EvmInternalError,
		InvalidTicker,
		InvalidTokenName,
		TooManyForeignAssets,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New asset with the asset manager is registered
		ForeignAssetCreated {
			asset_id: AssetId,
			xcm_location: Location,
		},
		/// Changed the xcm type mapping for a given asset id
		ForeignAssetTypeChanged {
			asset_id: AssetId,
			new_xcm_location: Location,
		},
		/// Removed all information related to an assetId
		ForeignAssetRemoved {
			asset_id: AssetId,
			xcm_location: Location,
		},
		// Freezes all tokens of a given asset id
		ForeignAssetFrozen {
			asset_id: AssetId,
			xcm_location: Location,
		},
		// Thawing a previously frozen asset
		ForeignAssetUnfrozen {
			asset_id: AssetId,
			xcm_location: Location,
		},
		/// Removed all information related to an assetId and destroyed asset
		ForeignAssetDestroyed {
			asset_id: AssetId,
			xcm_location: Location,
		},
	}

	/// Mapping from an asset id to a Foreign asset type.
	/// This is mostly used when receiving transaction specifying an asset directly,
	/// like transferring an asset from this chain to another.
	#[pallet::storage]
	#[pallet::getter(fn assets_by_id)]
	pub type AssetsById<T: Config> =
		CountedStorageMap<_, Blake2_128Concat, AssetId, Location, OptionQuery>;

	/// Reverse mapping of AssetsById. Mapping from a foreign asset to an asset id.
	/// This is mostly used when receiving a multilocation XCM message to retrieve
	/// the corresponding asset in which tokens should me minted.
	#[pallet::storage]
	#[pallet::getter(fn assets_by_location)]
	pub type AssetsByLocation<T: Config> =
		StorageMap<_, Blake2_128Concat, Location, (AssetId, AssetStatus)>;

	impl<T: Config> Pallet<T> {
		/// The account ID of this pallet
		pub fn account_id() -> H160 {
			let account_id: <T as Config>::AccountId = PALLET_ID.into_account_truncating();
			account_id.into()
		}

		#[inline]
		pub(crate) fn contract_address_from_asset_id(asset_id: AssetId) -> H160 {
			let mut buffer = [0u8; 20];
			buffer[..4].copy_from_slice(&FOREIGN_ASSETS_PREFIX);
			buffer[4..].copy_from_slice(&asset_id.to_be_bytes());
			H160(buffer)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create new asset with the ForeignAssetCreator
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::create_foreign_asset())]
		pub fn create_foreign_asset(
			origin: OriginFor<T>,
			xcm_location: Location,
			asset_id: AssetId,
			decimals: u8,
			ticker: BoundedVec<u8, ConstU32<256>>,
			name: BoundedVec<u8, ConstU32<256>>,
		) -> DispatchResult {
			T::ForeignAssetCreatorOrigin::ensure_origin(origin)?;

			// Ensure such an assetId does not exist
			ensure!(
				AssetsById::<T>::get(&asset_id).is_none(),
				Error::<T>::AssetAlreadyExists
			);

			ensure!(
				AssetsById::<T>::count() < T::MaxForeignAssets::get(),
				Error::<T>::TooManyForeignAssets
			);

			ensure!(
				T::AssetIdFilter::contains(&asset_id),
				Error::<T>::AssetIdFiltered
			);

			let ticker = core::str::from_utf8(&ticker).map_err(|_| Error::<T>::InvalidTicker)?;
			let name = core::str::from_utf8(&name).map_err(|_| Error::<T>::InvalidTokenName)?;

			EvmCaller::<T>::erc20_create(asset_id, decimals, ticker, name)?;

			// Insert the association assetId->foreigAsset
			// Insert the association foreigAsset->assetId
			AssetsById::<T>::insert(&asset_id, &xcm_location);
			AssetsByLocation::<T>::insert(&xcm_location, (asset_id, AssetStatus::Active));

			T::OnForeignAssetCreated::on_asset_created(&xcm_location, &asset_id);

			Self::deposit_event(Event::ForeignAssetCreated {
				asset_id,
				xcm_location,
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
			new_xcm_location: Location,
		) -> DispatchResult {
			T::ForeignAssetModifierOrigin::ensure_origin(origin)?;

			let previous_location =
				AssetsById::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			// Remove previous foreign asset info
			let (_asset_id, asset_status) = AssetsByLocation::<T>::take(&previous_location)
				.ok_or(Error::<T>::CorruptedStorageOrphanLocation)?;

			// Insert new foreign asset info
			AssetsById::<T>::insert(&asset_id, &new_xcm_location);
			AssetsByLocation::<T>::insert(&new_xcm_location, (asset_id, asset_status));

			Self::deposit_event(Event::ForeignAssetTypeChanged {
				asset_id,
				new_xcm_location,
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

			let xcm_location =
				AssetsById::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			// Remove from AssetsById
			AssetsById::<T>::remove(&asset_id);
			// Remove from AssetsByLocation
			AssetsByLocation::<T>::remove(&xcm_location);

			Self::deposit_event(Event::ForeignAssetRemoved {
				asset_id,
				xcm_location,
			});
			Ok(())
		}

		/// Freeze a given foreign assetId
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::destroy_foreign_asset())]
		pub fn freeze_foreign_asset(
			origin: OriginFor<T>,
			asset_id: AssetId,
			allow_xcm_deposit: bool,
		) -> DispatchResult {
			T::ForeignAssetFreezerOrigin::ensure_origin(origin)?;

			let xcm_location =
				AssetsById::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			let (_asset_id, asset_status) = AssetsByLocation::<T>::get(&xcm_location)
				.ok_or(Error::<T>::CorruptedStorageOrphanLocation)?;

			ensure!(
				asset_status == AssetStatus::Active,
				Error::<T>::AssetAlreadyFrozen
			);

			EvmCaller::<T>::erc20_pause(asset_id)?;

			let new_asset_status = if allow_xcm_deposit {
				AssetStatus::FrozenXcmDepositAllowed
			} else {
				AssetStatus::FrozenXcmDepositForbidden
			};

			AssetsByLocation::<T>::insert(&xcm_location, (asset_id, new_asset_status));

			Self::deposit_event(Event::ForeignAssetFrozen {
				asset_id,
				xcm_location,
			});
			Ok(())
		}

		/// Freeze a given foreign assetId
		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::destroy_foreign_asset())]
		pub fn unfreeze_foreign_asset(origin: OriginFor<T>, asset_id: AssetId) -> DispatchResult {
			T::ForeignAssetUnfreezerOrigin::ensure_origin(origin)?;

			let xcm_location =
				AssetsById::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			let (_asset_id, asset_status) = AssetsByLocation::<T>::get(&xcm_location)
				.ok_or(Error::<T>::CorruptedStorageOrphanLocation)?;

			ensure!(
				asset_status == AssetStatus::FrozenXcmDepositAllowed
					|| asset_status == AssetStatus::FrozenXcmDepositForbidden,
				Error::<T>::AssetNotFrozen
			);

			EvmCaller::<T>::erc20_unpause(asset_id)?;

			AssetsByLocation::<T>::insert(&xcm_location, (asset_id, AssetStatus::Active));

			Self::deposit_event(Event::ForeignAssetUnfrozen {
				asset_id,
				xcm_location,
			});
			Ok(())
		}
	}

	impl<T: Config> xcm_executor::traits::TransactAsset for Pallet<T> {
		// For optimization reasons, the asset we want to deposit has not really been withdrawn,
		// we have just traced from which account it should have been withdrawn.
		// So we will retrieve these information and make the transfer from the origin account.
		fn deposit_asset(what: &Asset, who: &Location, _context: Option<&XcmContext>) -> XcmResult {
			let (contract_address, amount, asset_status) =
				ForeignAssetsMatcher::<T>::match_asset(what)?;

			if let AssetStatus::FrozenXcmDepositForbidden = asset_status {
				return Err(MatchError::AssetNotHandled.into());
			}

			let beneficiary = T::AccountIdConverter::convert_location(who)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			// We perform the evm transfers in a storage transaction to ensure that if it fail
			// any contract storage changes are rolled back.
			frame_support::storage::with_storage_layer(|| {
				EvmCaller::<T>::erc20_mint_into(contract_address, beneficiary, amount)
			})?;

			Ok(())
		}

		fn internal_transfer_asset(
			asset: &Asset,
			from: &Location,
			to: &Location,
			_context: &XcmContext,
		) -> Result<AssetsInHolding, XcmError> {
			let (contract_address, amount, asset_status) =
				ForeignAssetsMatcher::<T>::match_asset(asset)?;

			if let AssetStatus::FrozenXcmDepositForbidden | AssetStatus::FrozenXcmDepositAllowed =
				asset_status
			{
				return Err(MatchError::AssetNotHandled.into());
			}

			let from = T::AccountIdConverter::convert_location(from)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			let to = T::AccountIdConverter::convert_location(to)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			// We perform the evm transfers in a storage transaction to ensure that if it fail
			// any contract storage changes are rolled back.
			frame_support::storage::with_storage_layer(|| {
				EvmCaller::<T>::erc20_transfer(contract_address, from, to, amount)
			})?;

			Ok(asset.clone().into())
		}

		// Since we don't control the erc20 contract that manages the asset we want to withdraw,
		// we can't really withdraw this asset, we can only transfer it to another account.
		// It would be possible to transfer the asset to a dedicated account that would reflect
		// the content of the xcm holding, but this would imply to perform two evm calls instead of
		// one (1 to withdraw the asset and a second one to deposit it).
		// In order to perform only one evm call, we just trace the origin of the asset,
		// and then the transfer will only really be performed in the deposit instruction.
		fn withdraw_asset(
			what: &Asset,
			who: &Location,
			_context: Option<&XcmContext>,
		) -> Result<AssetsInHolding, XcmError> {
			let (contract_address, amount, asset_status) =
				ForeignAssetsMatcher::<T>::match_asset(what)?;
			let who = T::AccountIdConverter::convert_location(who)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			if let AssetStatus::FrozenXcmDepositForbidden | AssetStatus::FrozenXcmDepositAllowed =
				asset_status
			{
				return Err(MatchError::AssetNotHandled.into());
			}

			// We perform the evm transfers in a storage transaction to ensure that if it fail
			// any contract storage changes are rolled back.
			frame_support::storage::with_storage_layer(|| {
				EvmCaller::<T>::erc20_burn_from(contract_address, who, amount)
			})?;

			Ok(what.clone().into())
		}
	}
}
