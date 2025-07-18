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
extern crate alloc;

use crate::{AssetId, Error, Pallet};
use alloc::format;
use ethereum_types::{BigEndianHash, H160, H256, U256};
use fp_evm::{ExitReason, ExitSucceed};
use frame_support::ensure;
use frame_support::pallet_prelude::Weight;
use pallet_evm::{GasWeightMapping, Runner};
use precompile_utils::prelude::*;
use precompile_utils::solidity::codec::{Address, BoundedString};
use precompile_utils::solidity::Codec;
use precompile_utils_macro::keccak256;
use sp_runtime::traits::ConstU32;
use sp_runtime::{DispatchError, SaturatedConversion};
use sp_std::vec::Vec;
use xcm::latest::Error as XcmError;

const ERC20_CALL_MAX_CALLDATA_SIZE: usize = 4 + 32 + 32; // selector + address + uint256
const ERC20_CREATE_MAX_CALLDATA_SIZE: usize = 16 * 1024; // 16Ko

// Hardcoded gas limits (from manual binary search)
const ERC20_CREATE_GAS_LIMIT: u64 = 3_600_000; // highest failure: 3_600_000
pub(crate) const ERC20_BURN_FROM_GAS_LIMIT: u64 = 160_000; // highest failure: 154_000
pub(crate) const ERC20_MINT_INTO_GAS_LIMIT: u64 = 160_000; // highest failure: 154_000
const ERC20_PAUSE_GAS_LIMIT: u64 = 160_000; // highest failure: 150_500
pub(crate) const ERC20_TRANSFER_GAS_LIMIT: u64 = 160_000; // highest failure: 154_000
pub(crate) const ERC20_APPROVE_GAS_LIMIT: u64 = 160_000; // highest failure: 153_000
const ERC20_UNPAUSE_GAS_LIMIT: u64 = 160_000; // highest failure: 149_500
pub(crate) const ERC20_BALANCE_OF_GAS_LIMIT: u64 = 160_000; // Calculated effective gas: max(used: 24276, pov: 150736, storage: 0) = 150736

#[derive(Debug)]
pub enum EvmError {
	BurnFromFail(String),
	BalanceOfFail(String),
	ContractReturnInvalidValue,
	DispatchError(DispatchError),
	EvmCallFail(String),
	MintIntoFail(String),
	TransferFail(String),
}

impl From<DispatchError> for EvmError {
	fn from(e: DispatchError) -> Self {
		Self::DispatchError(e)
	}
}

impl From<EvmError> for XcmError {
	fn from(error: EvmError) -> XcmError {
		match error {
			EvmError::BurnFromFail(err) => {
				log::debug!("BurnFromFail error: {:?}", err);
				XcmError::FailedToTransactAsset("Erc20 contract call burnFrom fail")
			}
			EvmError::BalanceOfFail(err) => {
				log::debug!("BalanceOfFail error: {:?}", err);
				XcmError::FailedToTransactAsset("Erc20 contract call balanceOf fail")
			}
			EvmError::ContractReturnInvalidValue => {
				XcmError::FailedToTransactAsset("Erc20 contract return invalid value")
			}
			EvmError::DispatchError(err) => {
				log::debug!("dispatch error: {:?}", err);
				Self::FailedToTransactAsset("storage layer error")
			}
			EvmError::EvmCallFail(err) => {
				log::debug!("EvmCallFail error: {:?}", err);
				XcmError::FailedToTransactAsset("Fail to call erc20 contract")
			}
			EvmError::MintIntoFail(err) => {
				log::debug!("MintIntoFail error: {:?}", err);
				XcmError::FailedToTransactAsset("Erc20 contract call mintInto fail+")
			}
			EvmError::TransferFail(err) => {
				log::debug!("TransferFail error: {:?}", err);
				XcmError::FailedToTransactAsset("Erc20 contract call transfer fail")
			}
		}
	}
}

#[derive(Codec)]
#[cfg_attr(test, derive(Debug))]
struct ForeignErc20ConstructorArgs {
	owner: Address,
	decimals: u8,
	symbol: BoundedString<ConstU32<64>>,
	token_name: BoundedString<ConstU32<256>>,
}

pub(crate) struct EvmCaller<T: crate::Config>(core::marker::PhantomData<T>);

impl<T: crate::Config> EvmCaller<T> {
	/// Deploy foreign asset erc20 contract
	pub(crate) fn erc20_create(
		asset_id: AssetId,
		decimals: u8,
		symbol: &str,
		token_name: &str,
	) -> Result<H160, Error<T>> {
		// Get init code
		let mut init = Vec::with_capacity(ERC20_CREATE_MAX_CALLDATA_SIZE);
		init.extend_from_slice(include_bytes!("../resources/foreign_erc20_initcode.bin"));

		// Add constructor parameters
		let args = ForeignErc20ConstructorArgs {
			owner: Pallet::<T>::account_id().into(),
			decimals,
			symbol: symbol.into(),
			token_name: token_name.into(),
		};
		let encoded_args = precompile_utils::solidity::codec::Writer::new()
			.write(args)
			.build();
		// Skip size of constructor args (32 bytes)
		init.extend_from_slice(&encoded_args[32..]);

		let contract_adress = Pallet::<T>::contract_address_from_asset_id(asset_id);

		let exec_info = T::EvmRunner::create_force_address(
			Pallet::<T>::account_id(),
			init,
			U256::default(),
			ERC20_CREATE_GAS_LIMIT,
			None,
			None,
			None,
			Default::default(),
			false,
			false,
			None,
			None,
			&<T as pallet_evm::Config>::config(),
			contract_adress,
		)
		.map_err(|err| {
			log::debug!("erc20_create (error): {:?}", err.error.into());
			Error::<T>::Erc20ContractCreationFail
		})?;

		ensure!(
			matches!(
				exec_info.exit_reason,
				ExitReason::Succeed(ExitSucceed::Returned | ExitSucceed::Stopped)
			),
			Error::Erc20ContractCreationFail
		);

		Ok(contract_adress)
	}

	pub(crate) fn erc20_mint_into(
		erc20_contract_address: H160,
		beneficiary: H160,
		amount: U256,
	) -> Result<(), EvmError> {
		let mut input = Vec::with_capacity(ERC20_CALL_MAX_CALLDATA_SIZE);
		// Selector
		input.extend_from_slice(&keccak256!("mintInto(address,uint256)")[..4]);
		// append beneficiary address
		input.extend_from_slice(H256::from(beneficiary).as_bytes());
		// append amount to be minted
		input.extend_from_slice(H256::from_uint(&amount).as_bytes());

		let weight_limit: Weight =
			T::GasWeightMapping::gas_to_weight(ERC20_MINT_INTO_GAS_LIMIT, true);

		let exec_info = T::EvmRunner::call(
			Pallet::<T>::account_id(),
			erc20_contract_address,
			input,
			U256::default(),
			ERC20_MINT_INTO_GAS_LIMIT,
			None,
			None,
			None,
			Default::default(),
			false,
			false,
			Some(weight_limit),
			Some(0),
			&<T as pallet_evm::Config>::config(),
		)
		.map_err(|err| EvmError::MintIntoFail(format!("{:?}", err.error.into())))?;

		ensure!(
			matches!(
				exec_info.exit_reason,
				ExitReason::Succeed(ExitSucceed::Returned | ExitSucceed::Stopped)
			),
			{
				let err = error_on_execution_failure(&exec_info.exit_reason, &exec_info.value);
				EvmError::MintIntoFail(err)
			}
		);

		Ok(())
	}

	pub(crate) fn erc20_transfer(
		erc20_contract_address: H160,
		from: H160,
		to: H160,
		amount: U256,
	) -> Result<(), EvmError> {
		let mut input = Vec::with_capacity(ERC20_CALL_MAX_CALLDATA_SIZE);
		// Selector
		input.extend_from_slice(&keccak256!("transfer(address,uint256)")[..4]);
		// append receiver address
		input.extend_from_slice(H256::from(to).as_bytes());
		// append amount to be transferred
		input.extend_from_slice(H256::from_uint(&amount).as_bytes());

		let weight_limit: Weight =
			T::GasWeightMapping::gas_to_weight(ERC20_TRANSFER_GAS_LIMIT, true);

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
			Some(weight_limit),
			Some(0),
			&<T as pallet_evm::Config>::config(),
		)
		.map_err(|err| EvmError::TransferFail(format!("{:?}", err.error.into())))?;

		ensure!(
			matches!(
				exec_info.exit_reason,
				ExitReason::Succeed(ExitSucceed::Returned | ExitSucceed::Stopped)
			),
			{
				let err = error_on_execution_failure(&exec_info.exit_reason, &exec_info.value);
				EvmError::TransferFail(err)
			}
		);

		// return value is true.
		let bytes: [u8; 32] = U256::from(1).to_big_endian();

		// Check return value to make sure not calling on empty contracts.
		ensure!(
			!exec_info.value.is_empty() && exec_info.value == bytes,
			EvmError::ContractReturnInvalidValue
		);

		Ok(())
	}

	pub(crate) fn erc20_approve(
		erc20_contract_address: H160,
		owner: H160,
		spender: H160,
		amount: U256,
	) -> Result<(), EvmError> {
		let mut input = Vec::with_capacity(ERC20_CALL_MAX_CALLDATA_SIZE);
		// Selector
		input.extend_from_slice(&keccak256!("approve(address,uint256)")[..4]);
		// append spender address
		input.extend_from_slice(H256::from(spender).as_bytes());
		// append amount to be approved
		input.extend_from_slice(H256::from_uint(&amount).as_bytes());
		let weight_limit: Weight =
			T::GasWeightMapping::gas_to_weight(ERC20_APPROVE_GAS_LIMIT, true);

		let exec_info = T::EvmRunner::call(
			owner,
			erc20_contract_address,
			input,
			U256::default(),
			ERC20_APPROVE_GAS_LIMIT,
			None,
			None,
			None,
			Default::default(),
			false,
			false,
			Some(weight_limit),
			Some(0),
			&<T as pallet_evm::Config>::config(),
		)
		.map_err(|err| EvmError::EvmCallFail(format!("{:?}", err.error.into())))?;

		ensure!(
			matches!(
				exec_info.exit_reason,
				ExitReason::Succeed(ExitSucceed::Returned | ExitSucceed::Stopped)
			),
			{
				let err = error_on_execution_failure(&exec_info.exit_reason, &exec_info.value);
				EvmError::EvmCallFail(err)
			}
		);

		Ok(())
	}

	pub(crate) fn erc20_burn_from(
		erc20_contract_address: H160,
		who: H160,
		amount: U256,
	) -> Result<(), EvmError> {
		let mut input = Vec::with_capacity(ERC20_CALL_MAX_CALLDATA_SIZE);
		// Selector
		input.extend_from_slice(&keccak256!("burnFrom(address,uint256)")[..4]);
		// append who address
		input.extend_from_slice(H256::from(who).as_bytes());
		// append amount to be burn
		input.extend_from_slice(H256::from_uint(&amount).as_bytes());

		let weight_limit: Weight =
			T::GasWeightMapping::gas_to_weight(ERC20_BURN_FROM_GAS_LIMIT, true);

		let exec_info = T::EvmRunner::call(
			Pallet::<T>::account_id(),
			erc20_contract_address,
			input,
			U256::default(),
			ERC20_BURN_FROM_GAS_LIMIT,
			None,
			None,
			None,
			Default::default(),
			false,
			false,
			Some(weight_limit),
			Some(0),
			&<T as pallet_evm::Config>::config(),
		)
		.map_err(|err| EvmError::EvmCallFail(format!("{:?}", err.error.into())))?;

		ensure!(
			matches!(
				exec_info.exit_reason,
				ExitReason::Succeed(ExitSucceed::Returned | ExitSucceed::Stopped)
			),
			{
				let err = error_on_execution_failure(&exec_info.exit_reason, &exec_info.value);
				EvmError::BurnFromFail(err)
			}
		);

		Ok(())
	}

	// Call contract selector "pause"
	pub(crate) fn erc20_pause(asset_id: AssetId) -> Result<(), Error<T>> {
		let mut input = Vec::with_capacity(ERC20_CALL_MAX_CALLDATA_SIZE);
		// Selector
		input.extend_from_slice(&keccak256!("pause()")[..4]);

		let weight_limit: Weight = T::GasWeightMapping::gas_to_weight(ERC20_PAUSE_GAS_LIMIT, true);

		let exec_info = T::EvmRunner::call(
			Pallet::<T>::account_id(),
			Pallet::<T>::contract_address_from_asset_id(asset_id),
			input,
			U256::default(),
			ERC20_PAUSE_GAS_LIMIT,
			None,
			None,
			None,
			Default::default(),
			false,
			false,
			Some(weight_limit),
			Some(0),
			&<T as pallet_evm::Config>::config(),
		)
		.map_err(|err| {
			log::debug!("erc20_pause (error): {:?}", err.error.into());
			Error::<T>::EvmInternalError
		})?;

		ensure!(
			matches!(
				exec_info.exit_reason,
				ExitReason::Succeed(ExitSucceed::Returned | ExitSucceed::Stopped)
			),
			{
				let err = error_on_execution_failure(&exec_info.exit_reason, &exec_info.value);
				log::debug!("erc20_pause (error): {:?}", err);
				Error::<T>::EvmCallPauseFail
			}
		);

		Ok(())
	}

	// Call contract selector "unpause"
	pub(crate) fn erc20_unpause(asset_id: AssetId) -> Result<(), Error<T>> {
		let mut input = Vec::with_capacity(ERC20_CALL_MAX_CALLDATA_SIZE);
		// Selector
		input.extend_from_slice(&keccak256!("unpause()")[..4]);

		let weight_limit: Weight =
			T::GasWeightMapping::gas_to_weight(ERC20_UNPAUSE_GAS_LIMIT, true);

		let exec_info = T::EvmRunner::call(
			Pallet::<T>::account_id(),
			Pallet::<T>::contract_address_from_asset_id(asset_id),
			input,
			U256::default(),
			ERC20_UNPAUSE_GAS_LIMIT,
			None,
			None,
			None,
			Default::default(),
			false,
			false,
			Some(weight_limit),
			Some(0),
			&<T as pallet_evm::Config>::config(),
		)
		.map_err(|err| {
			log::debug!("erc20_unpause (error): {:?}", err.error.into());
			Error::<T>::EvmInternalError
		})?;

		ensure!(
			matches!(
				exec_info.exit_reason,
				ExitReason::Succeed(ExitSucceed::Returned | ExitSucceed::Stopped)
			),
			{
				let err = error_on_execution_failure(&exec_info.exit_reason, &exec_info.value);
				log::debug!("erc20_unpause (error): {:?}", err);
				Error::<T>::EvmCallUnpauseFail
			}
		);

		Ok(())
	}

	// Call contract selector "balanceOf"
	pub(crate) fn erc20_balance_of(asset_id: AssetId, account: H160) -> Result<U256, EvmError> {
		let mut input = Vec::with_capacity(ERC20_CALL_MAX_CALLDATA_SIZE);
		// Selector
		input.extend_from_slice(&keccak256!("balanceOf(address)")[..4]);
		// append account address
		input.extend_from_slice(H256::from(account).as_bytes());

		let exec_info = T::EvmRunner::call(
			Pallet::<T>::account_id(),
			Pallet::<T>::contract_address_from_asset_id(asset_id),
			input,
			U256::default(),
			ERC20_BALANCE_OF_GAS_LIMIT,
			None,
			None,
			None,
			Default::default(),
			false,
			false,
			None,
			None,
			&<T as pallet_evm::Config>::config(),
		)
		.map_err(|err| EvmError::EvmCallFail(format!("{:?}", err.error.into())))?;

		ensure!(
			matches!(
				exec_info.exit_reason,
				ExitReason::Succeed(ExitSucceed::Returned | ExitSucceed::Stopped)
			),
			{
				let err = error_on_execution_failure(&exec_info.exit_reason, &exec_info.value);
				EvmError::BalanceOfFail(err)
			}
		);

		let balance = U256::from_big_endian(&exec_info.value);
		Ok(balance)
	}
}

fn error_on_execution_failure(reason: &ExitReason, data: &[u8]) -> String {
	match reason {
		ExitReason::Succeed(_) => alloc::string::String::new(),
		ExitReason::Error(err) => format!("evm error: {err:?}"),
		ExitReason::Fatal(err) => format!("evm fatal: {err:?}"),
		ExitReason::Revert(_) => extract_revert_message(data),
	}
}

/// The data should contain a UTF-8 encoded revert reason with a minimum size consisting of:
/// error function selector (4 bytes) + offset (32 bytes) + reason string length (32 bytes)
fn extract_revert_message(data: &[u8]) -> alloc::string::String {
	const LEN_START: usize = 36;
	const MESSAGE_START: usize = 68;
	const BASE_MESSAGE: &str = "VM Exception while processing transaction: revert";
	// Return base message if data is too short
	if data.len() <= MESSAGE_START {
		return BASE_MESSAGE.into();
	}
	// Extract message length and calculate end position
	let message_len =
		U256::from_big_endian(&data[LEN_START..MESSAGE_START]).saturated_into::<usize>();
	let message_end = MESSAGE_START.saturating_add(message_len);
	// Return base message if data is shorter than expected message end
	if data.len() < message_end {
		return BASE_MESSAGE.into();
	}
	// Extract and decode the message
	let body = &data[MESSAGE_START..message_end];
	match core::str::from_utf8(body) {
		Ok(reason) => format!("{BASE_MESSAGE} {reason}"),
		Err(_) => BASE_MESSAGE.into(),
	}
}
