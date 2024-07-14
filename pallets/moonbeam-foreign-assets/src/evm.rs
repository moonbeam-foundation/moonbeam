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

use crate::{AssetId, Error, Pallet};
use ethereum_types::{H160, U256};
use fp_evm::{ExitReason, ExitSucceed};
use frame_support::ensure;
use pallet_evm::Runner;
use sp_runtime::DispatchError;
use xcm::latest::Error as XcmError;

const ERC20_CREATE_INIT_CODE_MAX_SIZE: usize = 16 * 1024;
const FOREIGN_ASSETS_PREFIX: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
const FOREIGN_ASSET_ERC20_CREATE_GAS_LIMIT: u64 = 500_000;
const FOREIGN_ASSET_ERC20_TRANSFER_GAS_LIMIT: u64 = 500_000;
const FOREIGN_ASSET_ERC20_MINT_INTO_GAS_LIMIT: u64 = 500_000;
const FOREIGN_ASSET_ERC20_BURN_FROM_GAS_LIMIT: u64 = 500_000;
const FOREIGN_ASSET_ERC20_PAUSE_GAS_LIMIT: u64 = 500_000;
const FOREIGN_ASSET_ERC20_UNPAUSE_GAS_LIMIT: u64 = 500_000;

pub(crate) enum EvmError {
	ContractCreationFail,
	ContractTransferFail,
	ContractReturnInvalidValue,
	DispatchError(DispatchError),
	EvmCallFail,
}

impl From<DispatchError> for EvmError {
	fn from(e: DispatchError) -> Self {
		Self::DispatchError(e)
	}
}

impl From<EvmError> for XcmError {
	fn from(error: EvmError) -> XcmError {
		match error {
			EvmError::ContractCreationFail => {
				XcmError::FailedToTransactAsset("Erc20 contract transfer fail")
			}
			EvmError::ContractTransferFail => {
				XcmError::FailedToTransactAsset("Erc20 contract transfer fail")
			}
			EvmError::ContractReturnInvalidValue => {
				XcmError::FailedToTransactAsset("Erc20 contract return invalid value")
			}
			EvmError::DispatchError(err) => {
				log::debug!("dispatch error: {:?}", err);
				Self::FailedToTransactAsset("storage layer error")
			}
			EvmError::EvmCallFail => XcmError::FailedToTransactAsset("Fail to call erc20 contract"),
		}
	}
}

pub(crate) struct EvmCaller<T: crate::Config>(core::marker::PhantomData<T>);

impl<T: crate::Config> EvmCaller<T> {
	/// Deploy foreign asset erc20 contract
	pub(crate) fn erc20_create(asset_id: AssetId, decimals: u8) -> Result<(), Error<T>> {
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
			Pallet::<T>::account_id(),
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

	pub(crate) fn erc20_mint_into(
		erc20_contract_address: H160,
		beneficiary: H160,
		amount: U256,
	) -> Result<(), EvmError> {
		todo!()
	}

	pub(crate) fn erc20_transfer(
		erc20_contract_address: H160,
		from: H160,
		to: H160,
		amount: U256,
	) -> Result<(), EvmError> {
		todo!()
	}

	pub(crate) fn erc20_burn_from(
		erc20_contract_address: H160,
		who: H160,
		amount: U256,
	) -> Result<(), EvmError> {
		todo!()
	}

	// Call contract selector "pause"
	pub(crate) fn erc20_pause(asset_id: AssetId) -> Result<(), Error<T>> {
		todo!()
	}

	// Call contract selector "unpause"
	pub(crate) fn erc20_unpause(asset_id: AssetId) -> Result<(), Error<T>> {
		todo!()
	}
}
