// Copyright 2025 Moonbeam Foundation.
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
use xcm::prelude::Parachain;
use xcm_executor::traits::ConvertLocation;
use xcm_executor::traits::Error as MatchError;

const FOREIGN_ASSETS_PREFIX: [u8; 4] = [0xff, 0xff, 0xff, 0xff];

/// Trait for the OnForeignAssetRegistered hook
pub trait ForeignAssetCreatedHook<ForeignAsset> {
	fn on_asset_created(foreign_asset: &ForeignAsset, asset_id: &AssetId);
}

impl<ForeignAsset> ForeignAssetCreatedHook<ForeignAsset> for () {
	fn on_asset_created(_foreign_asset: &ForeignAsset, _asset_id: &AssetId) {}
}

/// Ensure origin location is a sibling
fn convert_location<T>(location: &Location) -> Result<T::AccountId, DispatchError>
where
	T: Config,
{
	match location.unpack() {
		(1, [Parachain(_)]) => T::ConvertLocation::convert_location(location)
			.ok_or(Error::<T>::CannotConvertLocationToAccount.into()),
		_ => Err(DispatchError::BadOrigin.into()),
	}
}
#[derive(Decode, Encode, Debug, PartialEq, TypeInfo, Clone)]
pub enum OriginType {
	XCM(Location),
	Governance,
}

/// Used to convert the success of an EnsureOrigin into `OriginType::Governance`
pub struct MapSuccessToGovernance<Original>(PhantomData<Original>);
impl<O, Original: EnsureOrigin<O, Success = ()>> EnsureOrigin<O>
	for MapSuccessToGovernance<Original>
{
	type Success = OriginType;
	fn try_origin(o: O) -> Result<OriginType, O> {
		Original::try_origin(o)?;
		Ok(OriginType::Governance)
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<O, ()> {
		Original::try_successful_origin()
	}
}

/// Used to convert the success of an EnsureOrigin into `OriginType::XCM`
pub struct MapSuccessToXcm<Original>(PhantomData<Original>);
impl<O, Original: EnsureOrigin<O, Success = Location>> EnsureOrigin<O>
	for MapSuccessToXcm<Original>
{
	type Success = OriginType;
	fn try_origin(o: O) -> Result<OriginType, O> {
		Original::try_origin(o).map(OriginType::XCM)
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<O, ()> {
		Original::try_successful_origin()
	}
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

#[derive(Decode, Debug, Encode, PartialEq, TypeInfo)]
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
	use frame_support::traits::{Currency, ReservableCurrency};
	use pallet_evm::{GasWeightMapping, Runner};
	use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned, Convert};
	use xcm_executor::traits::ConvertLocation;
	use xcm_executor::traits::Error as MatchError;
	use xcm_executor::AssetsInHolding;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// The moonbeam foreign assets's pallet id
	pub const PALLET_ID: frame_support::PalletId = frame_support::PalletId(*b"forgasst");

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config + scale_info::TypeInfo {
		// Convert AccountId to H160
		type AccountIdToH160: Convert<Self::AccountId, H160>;

		/// A filter to forbid some AssetId values, if you don't use it, put "Everything"
		type AssetIdFilter: Contains<AssetId>;

		/// EVM runner
		type EvmRunner: Runner<Self>;

		type ConvertLocation: ConvertLocation<Self::AccountId>;

		/// Origin that is allowed to create a new foreign assets
		type ForeignAssetCreatorOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = OriginType>;

		/// Origin that is allowed to freeze all tokens of a foreign asset
		type ForeignAssetFreezerOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = OriginType>;

		/// Origin that is allowed to modify asset information for foreign assets
		type ForeignAssetModifierOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = OriginType>;

		/// Origin that is allowed to unfreeze all tokens of a foreign asset that was previously
		/// frozen
		type ForeignAssetUnfreezerOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = OriginType>;

		/// Hook to be called when new foreign asset is registered.
		type OnForeignAssetCreated: ForeignAssetCreatedHook<Location>;

		/// Maximum numbers of different foreign assets
		type MaxForeignAssets: Get<u32>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		// Convert XCM Location to H160
		type XcmLocationToH160: ConvertLocation<H160>;

		/// Amount of tokens required to lock for creating a new foreign asset
		type ForeignAssetCreationDeposit: Get<BalanceOf<Self>>;

		/// The balance type for locking funds
		type Balance: Member
			+ Parameter
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo;

		/// The currency type for locking funds
		type Currency: ReservableCurrency<Self::AccountId>;
	}

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
		EvmCallMintIntoFail,
		EvmCallTransferFail,
		EvmInternalError,
		/// Account has insufficient balance for locking
		InsufficientBalance,
		CannotConvertLocationToAccount,
		LocationOutsideOfOrigin,
		AssetNotInSiblingPara,
		InvalidSymbol,
		InvalidTokenName,
		LocationAlreadyExists,
		TooManyForeignAssets,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New asset with the asset manager is registered
		ForeignAssetCreated {
			contract_address: H160,
			asset_id: AssetId,
			xcm_location: Location,
			deposit: Option<BalanceOf<T>>,
		},
		/// Changed the xcm type mapping for a given asset id
		ForeignAssetXcmLocationChanged {
			asset_id: AssetId,
			previous_xcm_location: Location,
			new_xcm_location: Location,
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
		/// Tokens have been locked for asset creation
		TokensLocked(T::AccountId, AssetId, AssetBalance),
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

	/// Mapping from an asset id to its creation details
	#[pallet::storage]
	#[pallet::getter(fn assets_creation_details)]
	pub type AssetsCreationDetails<T: Config> =
		StorageMap<_, Blake2_128Concat, AssetId, AssetDepositDetails<T>>;

	#[derive(Clone, Decode, Encode, Eq, PartialEq, Debug, TypeInfo, MaxEncodedLen)]
	pub struct AssetDepositDetails<T: Config> {
		pub deposit_account: T::AccountId,
		pub deposit: BalanceOf<T>,
	}

	impl<T: Config> Pallet<T> {
		/// The account ID of this pallet
		#[inline]
		pub fn account_id() -> H160 {
			let account_id: T::AccountId = PALLET_ID.into_account_truncating();
			T::AccountIdToH160::convert(account_id)
		}

		/// Compute asset contract address from asset id
		#[inline]
		pub fn contract_address_from_asset_id(asset_id: AssetId) -> H160 {
			let mut buffer = [0u8; 20];
			buffer[..4].copy_from_slice(&FOREIGN_ASSETS_PREFIX);
			buffer[4..].copy_from_slice(&asset_id.to_be_bytes());
			H160(buffer)
		}

		/// This method only exists for migration purposes and will be deleted once the
		/// foreign assets migration is finished.
		pub fn register_foreign_asset(
			asset_id: AssetId,
			xcm_location: Location,
			decimals: u8,
			symbol: BoundedVec<u8, ConstU32<256>>,
			name: BoundedVec<u8, ConstU32<256>>,
		) -> DispatchResult {
			Self::do_create_asset(asset_id, xcm_location, decimals, symbol, name, None)
		}

		/// Mint an asset into a specific account
		pub fn mint_into(
			asset_id: AssetId,
			beneficiary: T::AccountId,
			amount: U256,
		) -> Result<(), evm::EvmError> {
			// We perform the evm call in a storage transaction to ensure that if it fail
			// any contract storage changes are rolled back.
			frame_support::storage::with_storage_layer(|| {
				EvmCaller::<T>::erc20_mint_into(
					Self::contract_address_from_asset_id(asset_id),
					T::AccountIdToH160::convert(beneficiary),
					amount,
				)
			})
			.map_err(Into::into)
		}

		/// Transfer an asset from an account to another one
		pub fn transfer(
			asset_id: AssetId,
			from: T::AccountId,
			to: T::AccountId,
			amount: U256,
		) -> Result<(), evm::EvmError> {
			frame_support::storage::with_storage_layer(|| {
				EvmCaller::<T>::erc20_transfer(
					Self::contract_address_from_asset_id(asset_id),
					T::AccountIdToH160::convert(from),
					T::AccountIdToH160::convert(to),
					amount,
				)
			})
			.map_err(Into::into)
		}

		pub fn balance(asset_id: AssetId, who: T::AccountId) -> Result<U256, evm::EvmError> {
			EvmCaller::<T>::erc20_balance_of(asset_id, T::AccountIdToH160::convert(who))
				.map_err(Into::into)
		}

		/// Approve a spender to spend a certain amount of tokens from the owner account
		pub fn approve(
			asset_id: AssetId,
			owner: T::AccountId,
			spender: T::AccountId,
			amount: U256,
		) -> Result<(), evm::EvmError> {
			// We perform the evm call in a storage transaction to ensure that if it fail
			// any contract storage changes are rolled back.
			frame_support::storage::with_storage_layer(|| {
				EvmCaller::<T>::erc20_approve(
					Self::contract_address_from_asset_id(asset_id),
					T::AccountIdToH160::convert(owner),
					T::AccountIdToH160::convert(spender),
					amount,
				)
			})
			.map_err(Into::into)
		}

		pub fn weight_of_erc20_burn() -> Weight {
			T::GasWeightMapping::gas_to_weight(evm::ERC20_BURN_FROM_GAS_LIMIT, true)
		}
		pub fn weight_of_erc20_mint() -> Weight {
			T::GasWeightMapping::gas_to_weight(evm::ERC20_MINT_INTO_GAS_LIMIT, true)
		}
		pub fn weight_of_erc20_transfer() -> Weight {
			T::GasWeightMapping::gas_to_weight(evm::ERC20_TRANSFER_GAS_LIMIT, true)
		}
		#[cfg(feature = "runtime-benchmarks")]
		pub fn set_asset(asset_location: Location, asset_id: AssetId) {
			AssetsByLocation::<T>::insert(&asset_location, (asset_id, AssetStatus::Active));
			AssetsById::<T>::insert(&asset_id, asset_location);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create new asset with the ForeignAssetCreator
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::create_foreign_asset())]
		pub fn create_foreign_asset(
			origin: OriginFor<T>,
			asset_id: AssetId,
			asset_xcm_location: Location,
			decimals: u8,
			symbol: BoundedVec<u8, ConstU32<256>>,
			name: BoundedVec<u8, ConstU32<256>>,
		) -> DispatchResult {
			let origin_type = T::ForeignAssetCreatorOrigin::ensure_origin(origin.clone())?;

			Self::ensure_origin_can_modify_location(origin_type.clone(), &asset_xcm_location)?;
			let deposit_account = Self::get_deposit_account(origin_type)?;

			Self::do_create_asset(
				asset_id,
				asset_xcm_location,
				decimals,
				symbol,
				name,
				deposit_account,
			)
		}

		/// Change the xcm type mapping for a given assetId
		/// We also change this if the previous units per second where pointing at the old
		/// assetType
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::change_xcm_location())]
		pub fn change_xcm_location(
			origin: OriginFor<T>,
			asset_id: AssetId,
			new_xcm_location: Location,
		) -> DispatchResult {
			let origin_type = T::ForeignAssetModifierOrigin::ensure_origin(origin.clone())?;

			Self::ensure_origin_can_modify_location(origin_type.clone(), &new_xcm_location)?;

			let previous_location =
				AssetsById::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			Self::ensure_origin_can_modify_location(origin_type, &previous_location)?;

			Self::do_change_xcm_location(asset_id, previous_location, new_xcm_location)
		}

		/// Freeze a given foreign assetId
		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::freeze_foreign_asset())]
		pub fn freeze_foreign_asset(
			origin: OriginFor<T>,
			asset_id: AssetId,
			allow_xcm_deposit: bool,
		) -> DispatchResult {
			let origin_type = T::ForeignAssetFreezerOrigin::ensure_origin(origin.clone())?;

			let xcm_location =
				AssetsById::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			Self::ensure_origin_can_modify_location(origin_type, &xcm_location)?;

			Self::do_freeze_asset(asset_id, xcm_location, allow_xcm_deposit)
		}

		/// Unfreeze a given foreign assetId
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::unfreeze_foreign_asset())]
		pub fn unfreeze_foreign_asset(origin: OriginFor<T>, asset_id: AssetId) -> DispatchResult {
			let origin_type = T::ForeignAssetUnfreezerOrigin::ensure_origin(origin.clone())?;

			let xcm_location =
				AssetsById::<T>::get(&asset_id).ok_or(Error::<T>::AssetDoesNotExist)?;

			Self::ensure_origin_can_modify_location(origin_type, &xcm_location)?;

			Self::do_unfreeze_asset(asset_id, xcm_location)
		}
	}

	impl<T: Config> Pallet<T> {
		/// Ensure that the caller origin can modify the location,
		fn ensure_origin_can_modify_location(
			origin_type: OriginType,
			location: &Location,
		) -> DispatchResult {
			match origin_type {
				OriginType::XCM(origin_location) => {
					ensure!(
						location.starts_with(&origin_location),
						Error::<T>::LocationOutsideOfOrigin,
					);
				}
				OriginType::Governance => {
					// nothing to check Governance can change any asset
				}
			};
			Ok(())
		}

		fn get_deposit_account(
			origin_type: OriginType,
		) -> Result<Option<T::AccountId>, DispatchError> {
			match origin_type {
				OriginType::XCM(origin_location) => {
					let deposit_account = convert_location::<T>(&origin_location)?;
					Ok(Some(deposit_account))
				}
				OriginType::Governance => Ok(None),
			}
		}

		pub fn do_create_asset(
			asset_id: AssetId,
			asset_xcm_location: Location,
			decimals: u8,
			symbol: BoundedVec<u8, ConstU32<256>>,
			name: BoundedVec<u8, ConstU32<256>>,
			deposit_account: Option<T::AccountId>,
		) -> DispatchResult {
			ensure!(
				!AssetsById::<T>::contains_key(&asset_id),
				Error::<T>::AssetAlreadyExists
			);

			ensure!(
				!AssetsByLocation::<T>::contains_key(&asset_xcm_location),
				Error::<T>::LocationAlreadyExists
			);

			ensure!(
				AssetsById::<T>::count() < T::MaxForeignAssets::get(),
				Error::<T>::TooManyForeignAssets
			);

			ensure!(
				T::AssetIdFilter::contains(&asset_id),
				Error::<T>::AssetIdFiltered
			);

			let symbol = core::str::from_utf8(&symbol).map_err(|_| Error::<T>::InvalidSymbol)?;
			let name = core::str::from_utf8(&name).map_err(|_| Error::<T>::InvalidTokenName)?;
			let contract_address = EvmCaller::<T>::erc20_create(asset_id, decimals, symbol, name)?;

			let deposit = if let Some(deposit_account) = deposit_account {
				let deposit = T::ForeignAssetCreationDeposit::get();

				// Reserve _deposit_ amount of funds from the caller
				<T as Config>::Currency::reserve(&deposit_account, deposit)?;

				// Insert the amount that is reserved from the user
				AssetsCreationDetails::<T>::insert(
					&asset_id,
					AssetDepositDetails {
						deposit_account,
						deposit,
					},
				);

				Some(deposit)
			} else {
				None
			};

			// Insert the association assetId->foreigAsset
			// Insert the association foreigAsset->assetId
			AssetsById::<T>::insert(&asset_id, &asset_xcm_location);
			AssetsByLocation::<T>::insert(&asset_xcm_location, (asset_id, AssetStatus::Active));

			T::OnForeignAssetCreated::on_asset_created(&asset_xcm_location, &asset_id);

			Self::deposit_event(Event::ForeignAssetCreated {
				contract_address,
				asset_id,
				xcm_location: asset_xcm_location,
				deposit,
			});
			Ok(())
		}

		pub fn do_change_xcm_location(
			asset_id: AssetId,
			previous_xcm_location: Location,
			new_xcm_location: Location,
		) -> DispatchResult {
			ensure!(
				!AssetsByLocation::<T>::contains_key(&new_xcm_location),
				Error::<T>::LocationAlreadyExists
			);

			// Remove previous foreign asset info
			let (_asset_id, asset_status) = AssetsByLocation::<T>::take(&previous_xcm_location)
				.ok_or(Error::<T>::CorruptedStorageOrphanLocation)?;

			// Insert new foreign asset info
			AssetsById::<T>::insert(&asset_id, &new_xcm_location);
			AssetsByLocation::<T>::insert(&new_xcm_location, (asset_id, asset_status));

			Self::deposit_event(Event::ForeignAssetXcmLocationChanged {
				asset_id,
				new_xcm_location,
				previous_xcm_location,
			});
			Ok(())
		}

		pub fn do_freeze_asset(
			asset_id: AssetId,
			xcm_location: Location,
			allow_xcm_deposit: bool,
		) -> DispatchResult {
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

		pub fn do_unfreeze_asset(asset_id: AssetId, xcm_location: Location) -> DispatchResult {
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

			let beneficiary = T::XcmLocationToH160::convert_location(who)
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

			let from = T::XcmLocationToH160::convert_location(from)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			let to = T::XcmLocationToH160::convert_location(to)
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
			let who = T::XcmLocationToH160::convert_location(who)
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

	impl<T: Config> sp_runtime::traits::MaybeEquivalence<Location, AssetId> for Pallet<T> {
		fn convert(location: &Location) -> Option<AssetId> {
			AssetsByLocation::<T>::get(location).map(|(asset_id, _)| asset_id)
		}
		fn convert_back(asset_id: &AssetId) -> Option<Location> {
			AssetsById::<T>::get(asset_id)
		}
	}
}
