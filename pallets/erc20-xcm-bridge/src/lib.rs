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
mod erc20_trap;
mod errors;

use frame_support::pallet;

pub use erc20_trap::AssetTrapWrapper;
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
	use xcm::latest::{Error as XcmError, MultiAsset, MultiLocation, Result as XcmResult};
	use xcm_executor::traits::{Convert, Error as MatchError};
	use xcm_executor::Assets;

	const ERC20_TRANSFER_GAS_LIMIT: u64 = 200_000;
	const ERC20_TRANSFER_SELECTOR: [u8; 4] = [0xa9, 0x05, 0x9c, 0xbb];

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		type AccountIdConverter: Convert<MultiLocation, H160>;
		type Erc20MultilocationPrefix: Get<MultiLocation>;
		type EvmRunner: Runner<Self>;
	}

	impl<T: Config> Pallet<T> {
		fn erc20_transfer(
			erc20_contract_address: H160,
			from: H160,
			to: H160,
			amount: U256,
		) -> Result<(), Erc20TransferError> {
			// ERC20.transfer method hash
			let mut input = ERC20_TRANSFER_SELECTOR.to_vec();
			// append receiver address
			input.extend_from_slice(H256::from(to).as_bytes());
			// append amount to be transferred
			input.extend_from_slice(H256::from_uint(&amount).as_bytes());

			let exec_info = T::EvmRunner::call(
				from,
				erc20_contract_address,
				input,
				U256::default(),
				ERC20_TRANSFER_GAS_LIMIT,
				None,
				None,
				None,
				Default::default(),
				false,
				false,
				&fp_evm::Config::london(),
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
		// For optimization reasons, the asset we want to deposit has not really been withdrawn,
		// we have just traced from which account it should have been withdrawn.
		// So we will retrieve these information and make the transfer from the origin account.
		fn deposit_asset(what: &MultiAsset, who: &MultiLocation) -> XcmResult {
			let Erc20Asset {
				contract_address,
				amount,
				maybe_holder,
			} = Erc20Matcher::<T>::matches_erc20(what)?;

			let from = maybe_holder.ok_or(MatchError::AssetIdConversionFailed)?;

			let beneficiary = T::AccountIdConverter::convert_ref(who)
				.map_err(|()| MatchError::AccountIdConversionFailed)?;

			frame_support::storage::with_storage_layer(|| {
				Self::erc20_transfer(contract_address, from, beneficiary, amount)
					.map_err(DepositError::Erc20TransferError)
			})
			.map_err(Into::into)
		}

		fn internal_transfer_asset(
			asset: &MultiAsset,
			from: &MultiLocation,
			to: &MultiLocation,
		) -> Result<Assets, XcmError> {
			let Erc20Asset {
				contract_address,
				amount,
				maybe_holder,
			} = Erc20Matcher::<T>::matches_erc20(asset)?;

			ensure!(
				maybe_holder.is_none(),
				XcmError::from(MatchError::AssetIdConversionFailed)
			);

			let from = T::AccountIdConverter::convert_ref(from)
				.map_err(|()| MatchError::AccountIdConversionFailed)?;

			let to = T::AccountIdConverter::convert_ref(to)
				.map_err(|()| MatchError::AccountIdConversionFailed)?;

			Self::erc20_transfer(contract_address, from, to, amount)?;

			Ok(asset.clone().into())
		}

		// Since we don't control the erc20 contract that manages the asset we want to withdraw,
		// we can't really withdraw this asset, we can only transfer it to another account.
		// It would be possible to transfer the asset to a dedicated account that would reflect
		// the content of the xcm holding, but this would imply to perform two evm calls instead of
		// one (1 to withdraw the asset and a second one to deposit it).
		// In order to perform only one evm call, we just trace the origin of the asset,
		// and then the transfer will only really be performed in the deposit instruction.
		fn withdraw_asset(what: &MultiAsset, who: &MultiLocation) -> Result<Assets, XcmError> {
			let Erc20Asset {
				contract_address: _,
				amount: _,
				maybe_holder,
			} = Erc20Matcher::<T>::matches_erc20(what)?;

			ensure!(
				maybe_holder.is_none(),
				XcmError::from(MatchError::AssetIdConversionFailed)
			);

			let who = T::AccountIdConverter::convert_ref(who)
				.map_err(|()| MatchError::AccountIdConversionFailed)?;

			Ok(Erc20Matcher::<T>::insert_holder(what, who)?.into())
		}
	}
}
