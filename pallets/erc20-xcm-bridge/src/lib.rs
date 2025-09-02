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

//! Pallet that allow to transact erc20 tokens trought xcm directly.

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
	use ethereum_types::BigEndianHash;
	use fp_evm::{ExitReason, ExitSucceed};
	use frame_support::pallet_prelude::*;
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
	pub trait Config: frame_system::Config + pallet_evm::Config {
		type AccountIdConverter: ConvertLocation<H160>;
		type Erc20MultilocationPrefix: Get<Location>;
		type Erc20TransferGasLimit: Get<u64>;
		type EvmRunner: Runner<Self>;
	}

	impl<T: Config> Pallet<T> {
		pub fn is_erc20_asset(asset: &Asset) -> bool {
			Erc20Matcher::<T::Erc20MultilocationPrefix>::is_erc20_asset(asset)
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
		fn erc20_transfer(
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
}
