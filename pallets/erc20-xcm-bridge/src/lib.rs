// Copyright 2019-2025 PureStake Inc.
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

//! Pallet that allow to transact erc20 tokens through xcm directly.
//!
//! It hosts two distinct asset transactors:
//!
//! - [`Pallet`] (the original "reserve mode" transactor): keeps a single EVM transfer per XCM
//!   message by deferring the actual ERC-20 movement until the in-message `DepositAsset`
//!   instruction. Suitable for reserve-backed transfers.
//! - [`Erc20TeleportTransactor`] (new): treats the ERC-20 supply on Moonbeam as locked in a
//!   runtime-controlled checking address whenever the contract is in the
//!   [`TeleportableErc20s`] whitelist. Performs an actual EVM transfer in `withdraw_asset` /
//!   `deposit_asset`. This is what backs `pallet_xcm::limited_teleport_assets` for the
//!   whitelisted ERC-20s, while the legacy reserve adapter continues to handle every other
//!   ERC-20.
//!
//! Native assets (DEV / GLMR / MOVR) are intentionally not handled here and remain
//! non-teleportable.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod erc20_matcher;
mod erc20_trap;
mod errors;
mod xcm_holding_ext;

use frame_support::pallet;

pub use erc20_trap::AssetTrapWrapper;
pub use pallet::*;
pub use xcm_holding_ext::XcmExecutorWrapper;

#[pallet]
pub mod pallet {

	use crate::erc20_matcher::*;
	use crate::errors::*;
	use crate::xcm_holding_ext::*;
	use core::marker::PhantomData;
	use ethereum_types::BigEndianHash;
	use fp_evm::{ExitReason, ExitSucceed};
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Contains, ContainsPair, EnsureOrigin};
	use frame_system::pallet_prelude::*;
	use pallet_evm::{GasWeightMapping, Runner};
	use sp_core::{H160, H256, U256};
	use sp_std::vec::Vec;
	use xcm::latest::{
		Asset, AssetId, Error as XcmError, Junction, Location, Result as XcmResult, XcmContext,
	};
	use xcm_executor::traits::ConvertLocation;
	use xcm_executor::traits::{Error as MatchError, MatchesFungibles};
	use xcm_executor::AssetsInHolding;

	const ERC20_TRANSFER_CALL_DATA_SIZE: usize = 4 + 32 + 32; // selector + from + amount
	const ERC20_TRANSFER_SELECTOR: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config:
		frame_system::Config<RuntimeEvent: From<Event<Self>>> + pallet_evm::Config
	{
		/// How a Location is converted into an EVM `H160` address (used to match accounts and
		/// the destination chain's sovereign account).
		type AccountIdConverter: ConvertLocation<H160>;
		/// XCM Location prefix used to identify ERC-20 multilocations on this chain (typically
		/// the pallet location, e.g. `(0, [PalletInstance(<this pallet index>)])`).
		type Erc20MultilocationPrefix: Get<Location>;
		/// Default gas limit used for ERC-20 transfers when the asset doesn't override it.
		type Erc20TransferGasLimit: Get<u64>;
		/// EVM runner used to execute ERC-20 calls (transfer / transferFrom).
		type EvmRunner: Runner<Self>;
		/// Origin that can edit the [`TeleportableErc20s`] whitelist.
		type TeleportAdminOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// EVM `H160` address used to lock ERC-20 supply when teleporting whitelisted contracts.
		/// Must be controlled by this runtime; should not be a regular user account.
		type TeleportCheckingAccount: Get<H160>;
	}

	/// Whitelist of ERC-20 contracts that are eligible for teleport semantics. Adding a
	/// contract to this map signals that:
	/// - this runtime locks its supply in `TeleportCheckingAccount` whenever it is sent
	///   cross-chain via XCM,
	/// - any sibling chain that registered the asset's foreign-asset twin with `teleportable:
	///   true` and `reserve = (1, [Parachain(<this para>)])` will accept teleport semantics
	///   for it.
	#[pallet::storage]
	pub type TeleportableErc20s<T: Config> = StorageMap<_, Twox64Concat, H160, (), OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An ERC-20 contract was added to the teleport whitelist.
		TeleportableErc20Added { contract: H160 },
		/// An ERC-20 contract was removed from the teleport whitelist.
		TeleportableErc20Removed { contract: H160 },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The contract is already in the teleport whitelist.
		Erc20AlreadyTeleportable,
		/// The contract is not in the teleport whitelist.
		Erc20NotTeleportable,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Add an ERC-20 contract to the teleport whitelist. Callable only by
		/// `TeleportAdminOrigin`.
		///
		/// The whitelist is checked by [`Erc20TeleportTransactor`] and by the
		/// [`IsTeleportableErc20`] filter, so adding a contract here both:
		/// - admits user-facing `pallet_xcm::limited_teleport_assets` calls carrying that
		///   contract, and
		/// - enables true lock/unlock semantics for it on cross-chain XCM flows.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(15_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(1, 1)))]
		pub fn add_teleportable_erc20(origin: OriginFor<T>, contract: H160) -> DispatchResult {
			T::TeleportAdminOrigin::ensure_origin(origin)?;
			ensure!(
				!TeleportableErc20s::<T>::contains_key(&contract),
				Error::<T>::Erc20AlreadyTeleportable
			);
			TeleportableErc20s::<T>::insert(&contract, ());
			Self::deposit_event(Event::TeleportableErc20Added { contract });
			Ok(())
		}

		/// Remove an ERC-20 contract from the teleport whitelist. Callable only by
		/// `TeleportAdminOrigin`.
		///
		/// Removing a contract immediately stops new outbound teleports for it. It does not
		/// unwind already-locked balances in `TeleportCheckingAccount`; those must still be
		/// claimed via the inbound teleport path. Operators should drain the lock account
		/// before removing widely-used contracts.
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(15_000_000, 0).saturating_add(T::DbWeight::get().reads_writes(1, 1)))]
		pub fn remove_teleportable_erc20(origin: OriginFor<T>, contract: H160) -> DispatchResult {
			T::TeleportAdminOrigin::ensure_origin(origin)?;
			ensure!(
				TeleportableErc20s::<T>::contains_key(&contract),
				Error::<T>::Erc20NotTeleportable
			);
			TeleportableErc20s::<T>::remove(&contract);
			Self::deposit_event(Event::TeleportableErc20Removed { contract });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn is_erc20_asset(asset: &Asset) -> bool {
			Erc20Matcher::<T::Erc20MultilocationPrefix>::is_erc20_asset(asset)
		}
		/// Whether the given ERC-20 contract has been admitted to the teleport whitelist.
		pub fn is_teleportable_erc20(contract: &H160) -> bool {
			TeleportableErc20s::<T>::contains_key(contract)
		}
		pub fn gas_limit_of_erc20_transfer(asset_id: &AssetId) -> u64 {
			let location = &asset_id.0;
			if let Some(Junction::GeneralKey {
				length: _,
				ref data,
			}) = location.interior().into_iter().next_back()
			{
				// As GeneralKey definition might change in future versions of XCM, this is meant
				// to throw a compile error as a warning that data type has changed.
				// If that happens, a new check is needed to ensure that data has at least 18
				// bytes (size of b"gas_limit:" + u64)
				let data: &[u8; 32] = &data;
				if let Ok(content) = core::str::from_utf8(&data[0..10]) {
					if content == "gas_limit:" {
						let mut bytes: [u8; 8] = Default::default();
						bytes.copy_from_slice(&data[10..18]);
						return u64::from_le_bytes(bytes);
					}
				}
			}
			T::Erc20TransferGasLimit::get()
		}
		pub fn weight_of_erc20_transfer(asset_id: &AssetId) -> Weight {
			T::GasWeightMapping::gas_to_weight(Self::gas_limit_of_erc20_transfer(asset_id), true)
		}
		pub(crate) fn erc20_transfer(
			erc20_contract_address: H160,
			from: H160,
			to: H160,
			amount: U256,
			gas_limit: u64,
		) -> Result<(), Erc20TransferError> {
			let mut input = Vec::with_capacity(ERC20_TRANSFER_CALL_DATA_SIZE);
			// ERC20.transfer method hash
			input.extend_from_slice(&ERC20_TRANSFER_SELECTOR);
			// append receiver address
			input.extend_from_slice(H256::from(to).as_bytes());
			// append amount to be transferred
			input.extend_from_slice(H256::from_uint(&amount).as_bytes());

			let weight_limit: Weight = T::GasWeightMapping::gas_to_weight(gas_limit, true);

			let exec_info = T::EvmRunner::call(
				from,
				erc20_contract_address,
				input,
				U256::default(),
				gas_limit,
				None,
				None,
				None,
				Default::default(),
				Default::default(),
				false,
				false,
				Some(weight_limit),
				Some(0),
				&<T as pallet_evm::Config>::config(),
			)
			.map_err(|_| Erc20TransferError::EvmCallFail)?;

			ensure!(
				matches!(
					exec_info.exit_reason,
					ExitReason::Succeed(ExitSucceed::Returned | ExitSucceed::Stopped)
				),
				Erc20TransferError::ContractTransferFail
			);

			// return value is true.
			let bytes: [u8; 32] = U256::from(1).to_big_endian();

			// Check return value to make sure not calling on empty contracts.
			ensure!(
				!exec_info.value.is_empty() && exec_info.value == bytes,
				Erc20TransferError::ContractReturnInvalidValue
			);

			Ok(())
		}
	}

	impl<T: Config> xcm_executor::traits::TransactAsset for Pallet<T> {
		// For optimization reasons, the asset we want to deposit has not really been withdrawn,
		// we have just traced from which account it should have been withdrawn.
		// So we will retrieve these information and make the transfer from the origin account.
		fn deposit_asset(what: &Asset, who: &Location, _context: Option<&XcmContext>) -> XcmResult {
			let (contract_address, amount) =
				Erc20Matcher::<T::Erc20MultilocationPrefix>::matches_fungibles(what)?;

			let beneficiary = T::AccountIdConverter::convert_location(who)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			let gas_limit = Self::gas_limit_of_erc20_transfer(&what.id);

			// Get the global context to recover accounts origins.
			XcmHoldingErc20sOrigins::with(|erc20s_origins| {
				match erc20s_origins.drain(contract_address, amount) {
					// We perform the evm transfers in a storage transaction to ensure that if one
					// of them fails all the changes of the previous evm calls are rolled back.
					Ok(tokens_to_transfer) => frame_support::storage::with_storage_layer(|| {
						tokens_to_transfer
							.into_iter()
							.try_for_each(|(from, subamount)| {
								Self::erc20_transfer(
									contract_address,
									from,
									beneficiary,
									subamount,
									gas_limit,
								)
							})
					})
					.map_err(Into::into),
					Err(DrainError::AssetNotFound) => Err(XcmError::AssetNotFound),
					Err(DrainError::NotEnoughFounds) => Err(XcmError::FailedToTransactAsset(
						"not enough founds in xcm holding",
					)),
					Err(DrainError::SplitError) => Err(XcmError::FailedToTransactAsset(
						"SplitError: each withdrawal of erc20 tokens must be deposited at once",
					)),
				}
			})
			.ok_or(XcmError::FailedToTransactAsset(
				"missing erc20 executor context",
			))?
		}

		fn internal_transfer_asset(
			asset: &Asset,
			from: &Location,
			to: &Location,
			_context: &XcmContext,
		) -> Result<AssetsInHolding, XcmError> {
			let (contract_address, amount) =
				Erc20Matcher::<T::Erc20MultilocationPrefix>::matches_fungibles(asset)?;

			let from = T::AccountIdConverter::convert_location(from)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			let to = T::AccountIdConverter::convert_location(to)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			let gas_limit = Self::gas_limit_of_erc20_transfer(&asset.id);

			// We perform the evm transfers in a storage transaction to ensure that if it fail
			// any contract storage changes are rolled back.
			frame_support::storage::with_storage_layer(|| {
				Self::erc20_transfer(contract_address, from, to, amount, gas_limit)
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
			let (contract_address, amount) =
				Erc20Matcher::<T::Erc20MultilocationPrefix>::matches_fungibles(what)?;
			let who = T::AccountIdConverter::convert_location(who)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			XcmHoldingErc20sOrigins::with(|erc20s_origins| {
				erc20s_origins.insert(contract_address, who, amount)
			})
			.ok_or(XcmError::FailedToTransactAsset(
				"missing erc20 executor context",
			))?;

			Ok(what.clone().into())
		}
	}

	/// `TransactAsset` implementation for whitelisted ERC-20s. Performs real EVM transfers
	/// against `T::TeleportCheckingAccount`:
	///
	/// - `withdraw_asset(asset, who)` → `ERC20.transfer(who → checking)`. Used both for
	///   teleport-out and (incidentally) reserve-out of whitelisted contracts.
	/// - `deposit_asset(holding, beneficiary)` → `ERC20.transfer(checking → beneficiary)`. Used
	///   both for teleport-in and reserve-in of whitelisted contracts.
	/// - `internal_transfer_asset(asset, from, to)` → `ERC20.transfer(from → to)` (same as
	///   the legacy reserve adapter; same-chain transfer never touches the checking address).
	///
	/// Non-whitelisted assets always return `Err(AssetNotFound)`, letting the legacy
	/// `Pallet<T>` adapter (placed after this one in `AssetTransactors`) keep its
	/// single-EVM-call reserve optimisation.
	pub struct Erc20TeleportTransactor<T>(PhantomData<T>);

	impl<T: Config> Erc20TeleportTransactor<T> {
		fn match_whitelisted(asset: &Asset) -> Result<(H160, U256), MatchError> {
			let (contract, amount) =
				Erc20Matcher::<T::Erc20MultilocationPrefix>::matches_fungibles(asset)?;
			if !TeleportableErc20s::<T>::contains_key(&contract) {
				return Err(MatchError::AssetNotHandled);
			}
			Ok((contract, amount))
		}
	}

	impl<T: Config> xcm_executor::traits::TransactAsset for Erc20TeleportTransactor<T> {
		fn can_check_in(_origin: &Location, what: &Asset, _context: &XcmContext) -> XcmResult {
			let _ = Self::match_whitelisted(what)?;
			Ok(())
		}

		fn check_in(_origin: &Location, _what: &Asset, _context: &XcmContext) {}

		fn can_check_out(_dest: &Location, what: &Asset, _context: &XcmContext) -> XcmResult {
			let _ = Self::match_whitelisted(what)?;
			Ok(())
		}

		fn check_out(_dest: &Location, _what: &Asset, _context: &XcmContext) {}

		fn deposit_asset(what: &Asset, who: &Location, _context: Option<&XcmContext>) -> XcmResult {
			let (contract, amount) = Self::match_whitelisted(what)?;
			let beneficiary = T::AccountIdConverter::convert_location(who)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			let gas_limit = Pallet::<T>::gas_limit_of_erc20_transfer(&what.id);
			let checking = T::TeleportCheckingAccount::get();

			frame_support::storage::with_storage_layer(|| {
				Pallet::<T>::erc20_transfer(contract, checking, beneficiary, amount, gas_limit)
			})
			.map_err(Into::into)
		}

		fn withdraw_asset(
			what: &Asset,
			who: &Location,
			_context: Option<&XcmContext>,
		) -> Result<AssetsInHolding, XcmError> {
			let (contract, amount) = Self::match_whitelisted(what)?;
			let from = T::AccountIdConverter::convert_location(who)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			let gas_limit = Pallet::<T>::gas_limit_of_erc20_transfer(&what.id);
			let checking = T::TeleportCheckingAccount::get();

			frame_support::storage::with_storage_layer(|| {
				Pallet::<T>::erc20_transfer(contract, from, checking, amount, gas_limit)
			})?;

			Ok(what.clone().into())
		}

		fn internal_transfer_asset(
			asset: &Asset,
			from: &Location,
			to: &Location,
			_context: &XcmContext,
		) -> Result<AssetsInHolding, XcmError> {
			let (contract, amount) = Self::match_whitelisted(asset)?;
			let from = T::AccountIdConverter::convert_location(from)
				.ok_or(MatchError::AccountIdConversionFailed)?;
			let to = T::AccountIdConverter::convert_location(to)
				.ok_or(MatchError::AccountIdConversionFailed)?;

			let gas_limit = Pallet::<T>::gas_limit_of_erc20_transfer(&asset.id);

			frame_support::storage::with_storage_layer(|| {
				Pallet::<T>::erc20_transfer(contract, from, to, amount, gas_limit)
			})?;

			Ok(asset.clone().into())
		}
	}

	/// Filter that returns `true` only when every input asset is a whitelisted ERC-20.
	///
	/// Implements both:
	/// - `ContainsPair<Asset, Location>` for use as `xcm_executor::Config::IsTeleporter`. The
	///   destination is intentionally ignored: trust on the receiving side is enforced by AH's
	///   per-asset `set_reserves(teleportable: true)`. If you want to also lock the
	///   counterparty here, wrap this with `Case<DevForAssetHub>` or similar in the runtime.
	/// - `Contains<(Location, Vec<Asset>)>` for use as `pallet_xcm::Config::XcmTeleportFilter`,
	///   so user-facing `pallet_xcm::limited_teleport_assets` calls only succeed when carrying
	///   whitelisted ERC-20 contracts.
	pub struct IsTeleportableErc20<T>(PhantomData<T>);

	impl<T: Config> IsTeleportableErc20<T> {
		fn asset_is_whitelisted(asset: &Asset) -> bool {
			match Erc20Matcher::<T::Erc20MultilocationPrefix>::matches_fungibles(asset) {
				Ok((contract, _amount)) => TeleportableErc20s::<T>::contains_key(&contract),
				Err(_) => false,
			}
		}
	}

	impl<T: Config> ContainsPair<Asset, Location> for IsTeleportableErc20<T> {
		fn contains(asset: &Asset, _origin: &Location) -> bool {
			Self::asset_is_whitelisted(asset)
		}
	}

	impl<T: Config> Contains<(Location, Vec<Asset>)> for IsTeleportableErc20<T> {
		fn contains(value: &(Location, Vec<Asset>)) -> bool {
			let (_origin, assets) = value;
			!assets.is_empty() && assets.iter().all(Self::asset_is_whitelisted)
		}
	}
}
