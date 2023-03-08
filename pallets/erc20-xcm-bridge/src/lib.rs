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

//! Pallet that allow to transact erc20 tokens trought xcm directly.

#![cfg_attr(not(feature = "std"), no_std)]

mod erc20_matcher;
mod errors;

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {

	use crate::erc20_matcher::*;
	use crate::errors::*;
	use ethereum_types::BigEndianHash;
	use fp_evm::{ExitReason, ExitSucceed};
	use frame_support::pallet_prelude::*;
	use pallet_evm::Runner;
	use sp_core::{H160, H256, U256};
	use sp_std::vec::Vec;
	use xcm::latest::{Error as XcmError, MultiAsset, MultiLocation, Result as XcmResult};
	use xcm_executor::traits::{Convert, Error as MatchError, MatchesFungibles};
	use xcm_executor::Assets;

	const ERC20_TRANSFER_CALL_DATA_SIZE: usize = 4 + 32 + 32; // selector + from + amount
	const ERC20_TRANSFER_SELECTOR: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		type AccountIdConverter: Convert<MultiLocation, H160>;
		type Erc20MultilocationPrefix: Get<MultiLocation>;
		type Erc20TransferGasLimit: Get<u64>;
		type EvmRunner: Runner<Self>;
	}

	impl<T: Config> Pallet<T> {
		pub fn is_erc20_asset(asset: &MultiAsset) -> bool {
			Erc20Matcher::<T::Erc20MultilocationPrefix>::is_erc20_asset(asset)
		}
		pub fn weight_of_erc20_transfer() -> Weight {
			Weight::from_ref_time(
				T::Erc20TransferGasLimit::get().saturating_mul(T::WeightPerGas::get().ref_time()),
			)
		}
		fn erc20_transfer(
			erc20_contract_address: H160,
			from: H160,
			to: H160,
			amount: U256,
		) -> Result<(), Erc20TransferError> {
			let mut input = Vec::with_capacity(ERC20_TRANSFER_CALL_DATA_SIZE);
			// ERC20.transfer method hash
			input.extend_from_slice(&ERC20_TRANSFER_SELECTOR);
			// append receiver address
			input.extend_from_slice(H256::from(to).as_bytes());
			// append amount to be transferred
			input.extend_from_slice(H256::from_uint(&amount).as_bytes());

			let exec_info = T::EvmRunner::call(
				from,
				erc20_contract_address,
				input,
				U256::default(),
				T::Erc20TransferGasLimit::get(),
				None,
				None,
				None,
				Default::default(),
				false,
				false,
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
			let mut bytes = [0u8; 32];
			U256::from(1).to_big_endian(&mut bytes);

			// Check return value to make sure not calling on empty contracts.
			ensure!(
				!exec_info.value.is_empty() && exec_info.value == bytes,
				Erc20TransferError::ContractReturnInvalidValue
			);

			Ok(())
		}
	}

	impl<T: Config> xcm_executor::traits::TransactAsset for Pallet<T> {
		// Erc20 assets can't goes in the Holding register, so they can't be deposited.
		// We still need to match the asset to return the right error.
		fn deposit_asset(what: &MultiAsset, _who: &MultiLocation) -> XcmResult {
			let _ = Erc20Matcher::<T::Erc20MultilocationPrefix>::matches_fungibles(what)?;
			Err(XcmError::NotWithdrawable)
		}

		fn internal_transfer_asset(
			asset: &MultiAsset,
			from: &MultiLocation,
			to: &MultiLocation,
		) -> Result<Assets, XcmError> {
			let (contract_address, amount) =
				Erc20Matcher::<T::Erc20MultilocationPrefix>::matches_fungibles(asset)?;

			let from = T::AccountIdConverter::convert_ref(from)
				.map_err(|()| MatchError::AccountIdConversionFailed)?;

			let to = T::AccountIdConverter::convert_ref(to)
				.map_err(|()| MatchError::AccountIdConversionFailed)?;

			// We perform the evm transfers in a storage transaction to ensure that if it fail
			// any contract storage changes are rolled back.
			frame_support::storage::with_storage_layer(|| {
				Self::erc20_transfer(contract_address, from, to, amount)
			})?;

			Ok(asset.clone().into())
		}

		// Since we don't control the erc20 contract that manages the asset we want to withdraw,
		// we can't really withdraw this asset, we can only transfer it to another account.
		// We still need to match the asset to return the right error.
		fn withdraw_asset(what: &MultiAsset, _who: &MultiLocation) -> Result<Assets, XcmError> {
			let _ = Erc20Matcher::<T::Erc20MultilocationPrefix>::matches_fungibles(what)?;
			Err(XcmError::NotWithdrawable)
		}
	}
}
