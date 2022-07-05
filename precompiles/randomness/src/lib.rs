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

//! Precompile to interact with randomness through an evm precompile.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(assert_matches)]

extern crate alloc;

use fp_evm::{
	Context, ExitReason, ExitSucceed, Log, Precompile, PrecompileHandle, PrecompileOutput,
};
use pallet_randomness::{BalanceOf, GetBabeData};
use precompile_utils::{costs::call_cost, prelude::*};
use sp_core::{H160, H256, U256};
use sp_std::{fmt::Debug, marker::PhantomData};

// #[cfg(test)]
// mod mock;
#[cfg(test)]
mod tests;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	RelayBlockNumber = "relayBlockNumber()",
	RelayEpochIndex = "relayEpochIndex()",
	RequestBabeRandomnessCurrentBlock =
		"requestBabeRandomnessCurrentBlock(address,uint256,uint64,bytes32,uint64)",
	RequestBabeRandomnessOneEpochAgo =
		"requestBabeRandomnessOneEpochAgo(address,uint256,uint64,bytes32)",
	RequestBabeRandomnessTwoEpochsAgo =
		"requestBabeRandomnessTwoEpochsAgo(address,uint256,uint64,bytes32)",
	RequestLocalRandomness = "requestLocalRandomness(address,uint256,uint64,bytes32,uint64)",
	FulfillRequest = "fulfillRequest(uint64)",
	IncreaseRequestFee = "increaseRequestFee(uint64,uint256)",
	ExecuteRequestExpiration = "executeRequestExpiration(uint64)",
}

pub const FULFILLMENT_ESTIMATED_COST: u64 = 1000u64; // TODO: get real value from benchmarking
pub const LOG_SUBCALL_SUCCEEDED: [u8; 32] = keccak256!("SubcallSucceeded");
pub const LOG_SUBCALL_FAILED: [u8; 32] = keccak256!("SubcallFailed");

pub fn log_subcall_succeeded(address: impl Into<H160>) -> Log {
	log0(address, LOG_SUBCALL_SUCCEEDED)
}

pub fn log_subcall_failed(address: impl Into<H160>) -> Log {
	log0(address, LOG_SUBCALL_FAILED)
}

/// Reverts if fees and gas_limit are not sufficient to make subcall and cleanup
fn ensure_can_provide_randomness<Runtime>(
	code_address: H160,
	gas_limit: u64,
	request_fee: BalanceOf<Runtime>,
	clean_up_cost: u64,
) -> EvmResult<()>
where
	Runtime: pallet_randomness::Config + pallet_evm::Config + pallet_base_fee::Config,
	BalanceOf<Runtime>: Into<U256>,
{
	// assert fee > gasLimit * base_fee
	let gas_limit_as_u256: U256 = gas_limit.into();
	if let Some(gas_limit_times_base_fee) =
		gas_limit_as_u256.checked_mul(pallet_base_fee::Pallet::<Runtime>::base_fee_per_gas())
	{
		if gas_limit_times_base_fee >= request_fee.into() {
			return Err(revert(
				"Gas limit at current price must be less than fees allotted",
			));
		}
	} else {
		return Err(revert("Gas limit times base fee overflowed U256"));
	}
	let log_cost = log_subcall_failed(code_address)
		.compute_cost()
		.map_err(|_| revert("failed to compute log cost"))?;
	// Cost of the call itself that the batch precompile must pay.
	let call_cost = call_cost(U256::zero(), <Runtime as pallet_evm::Config>::config());
	// assert gasLimit > overhead cost
	let overhead = call_cost + log_cost + clean_up_cost;
	if gas_limit <= overhead {
		return Err(revert("Gas limit must exceed overhead call cost"));
	}
	Ok(())
}

/// Subcall to provide randomness
/// caller must call `ensure_can_provide_randomness` before calling this function
fn provide_randomness(
	handle: &mut impl PrecompileHandle,
	gas_limit: u64,
	contract: H160,
	randomness: H256,
) -> EvmResult<()> {
	let (reason, _) = handle.call(
		contract,
		None,
		EvmDataWriter::new().write(randomness).build(),
		Some(gas_limit),
		false,
		&Context {
			caller: handle.context().address,
			address: contract,
			apparent_value: U256::zero(),
		},
	);
	// Logs
	// We reserved enough gas so this should not OOG.
	match reason {
		ExitReason::Revert(_) | ExitReason::Error(_) => {
			let log = log_subcall_failed(handle.code_address());
			handle.record_log_costs(&[&log])?;
			log.record(handle)?
		}
		ExitReason::Succeed(_) => {
			let log = log_subcall_succeeded(handle.code_address());
			handle.record_log_costs(&[&log])?;
			log.record(handle)?
		}
		_ => (),
	}
	Ok(())
}

/// A precompile to wrap the functionality from pallet-randomness
pub struct RandomnessWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for RandomnessWrapper<Runtime>
where
	Runtime: pallet_randomness::Config + pallet_evm::Config + pallet_base_fee::Config,
	<Runtime as frame_system::Config>::BlockNumber: From<u32>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256>,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		log::trace!(target: "randomness-precompile", "In randomness wrapper");

		let selector = handle.read_selector()?;

		// No funds are transferred to the precompile address.
		// Transfers will directly be made on behalf of the user by the precompile.
		handle.check_function_modifier(FunctionModifier::NonPayable)?;

		match selector {
			Action::RelayBlockNumber => Self::relay_block_number(handle),
			Action::RelayEpochIndex => Self::relay_epoch_index(handle),
			Action::RequestBabeRandomnessCurrentBlock => {
				Self::request_babe_randomness_current_block(handle)
			}
			Action::RequestBabeRandomnessOneEpochAgo => {
				Self::request_babe_randomness_one_epoch_ago(handle)
			}
			Action::RequestBabeRandomnessTwoEpochsAgo => {
				Self::request_babe_randomness_two_epochs_ago(handle)
			}
			Action::RequestLocalRandomness => Self::request_local_randomness(handle),
			Action::FulfillRequest => Self::fulfill_request(handle),
			Action::IncreaseRequestFee => Self::increase_request_fee(handle),
			Action::ExecuteRequestExpiration => Self::execute_request_expiration(handle),
		}
	}
}

impl<Runtime> RandomnessWrapper<Runtime>
where
	Runtime: pallet_randomness::Config + pallet_evm::Config + pallet_base_fee::Config,
	<Runtime as frame_system::Config>::BlockNumber: TryInto<u64> + TryFrom<u64>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256>,
{
	fn relay_block_number(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let relay_block_number: u64 =
			<Runtime as pallet_randomness::Config>::BabeDataGetter::get_relay_block_number()
				.try_into()
				.map_err(|_| revert("storage value is too large for provided block number type"))?;
		Ok(succeed(
			EvmDataWriter::new().write(relay_block_number).build(),
		))
	}
	fn relay_epoch_index(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let relay_epoch_index =
			<Runtime as pallet_randomness::Config>::BabeDataGetter::get_relay_epoch_index();
		Ok(succeed(
			EvmDataWriter::new().write(relay_epoch_index).build(),
		))
	}
	/// Make request for babe randomness current block
	fn request_babe_randomness_current_block(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		let contract_address = handle.context().caller;
		let refund_address = input.read::<Address>()?.0;
		let fee: BalanceOf<Runtime> = input
			.read::<U256>()?
			.try_into()
			.map_err(|_| revert("amount is too large for provided balance type"))?;
		let gas_limit = input.read::<u64>()?;
		let salt = input.read::<H256>()?;
		let blocks_after_current = input.read::<u64>()?;
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let relay_block_number: u64 =
			<Runtime as pallet_randomness::Config>::BabeDataGetter::get_relay_block_number()
				.try_into()
				.map_err(|_| revert("block number overflowed u64"))?;
		let requested_block_number = blocks_after_current
			.checked_add(relay_block_number)
			.ok_or(error("addition result overflowed u64"))?
			.try_into()
			.map_err(|_| revert("u64 addition result overflowed block number type"))?;
		let request = pallet_randomness::Request {
			refund_address,
			contract_address,
			fee,
			gas_limit,
			salt: salt.into(),
			info: pallet_randomness::RequestType::BabeCurrentBlock(requested_block_number),
		};
		pallet_randomness::Pallet::<Runtime>::request_randomness(request)
			.map_err(|e| error(alloc::format!("{:?}", e)))?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: Default::default(),
		})
	}
	/// Make request for babe randomness one epoch ago
	fn request_babe_randomness_one_epoch_ago(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		let contract_address = handle.context().caller;
		let refund_address = input.read::<Address>()?.0;
		let fee: BalanceOf<Runtime> = input
			.read::<U256>()?
			.try_into()
			.map_err(|_| revert("amount is too large for provided balance type"))?;
		let gas_limit = input.read::<u64>()?;
		let salt = input.read::<H256>()?;
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let next_epoch_index =
			<Runtime as pallet_randomness::Config>::BabeDataGetter::get_relay_epoch_index()
				.checked_add(1u64)
				.ok_or(error("Epoch Index (u64) overflowed"))?;
		let request = pallet_randomness::Request {
			refund_address,
			contract_address,
			fee,
			gas_limit,
			salt: salt.into(),
			info: pallet_randomness::RequestType::BabeOneEpochAgo(next_epoch_index),
		};
		pallet_randomness::Pallet::<Runtime>::request_randomness(request)
			.map_err(|e| error(alloc::format!("{:?}", e)))?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: Default::default(),
		})
	}
	/// Make request for babe randomness two epochs ago
	fn request_babe_randomness_two_epochs_ago(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		let contract_address = handle.context().caller;
		let refund_address = input.read::<Address>()?.0;
		let fee: BalanceOf<Runtime> = input
			.read::<U256>()?
			.try_into()
			.map_err(|_| revert("amount is too large for provided balance type"))?;
		let gas_limit = input.read::<u64>()?;
		let salt = input.read::<H256>()?;
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let next_epoch_index =
			<Runtime as pallet_randomness::Config>::BabeDataGetter::get_relay_epoch_index()
				.checked_add(1u64)
				.ok_or(error("Epoch Index (u64) overflowed"))?;
		let request = pallet_randomness::Request {
			refund_address,
			contract_address,
			fee,
			gas_limit,
			salt: salt.into(),
			info: pallet_randomness::RequestType::BabeTwoEpochsAgo(next_epoch_index),
		};
		pallet_randomness::Pallet::<Runtime>::request_randomness(request)
			.map_err(|e| error(alloc::format!("{:?}", e)))?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: Default::default(),
		})
	}
	/// Make request for local VRF randomness
	fn request_local_randomness(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		let contract_address = handle.context().caller;
		let refund_address = input.read::<Address>()?.0;
		let fee: BalanceOf<Runtime> = input
			.read::<U256>()?
			.try_into()
			.map_err(|_| revert("amount is too large for provided balance type"))?;
		let gas_limit = input.read::<u64>()?;
		let salt = input.read::<H256>()?;
		let blocks_after_current = input.read::<u64>()?;
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let current_block_number: u64 = <frame_system::Pallet<Runtime>>::block_number()
			.try_into()
			.map_err(|_| revert("block number overflowed u64"))?;
		let requested_block_number = blocks_after_current
			.checked_add(current_block_number)
			.ok_or(error("addition result overflowed u64"))?
			.try_into()
			.map_err(|_| revert("u64 addition result overflowed block number type"))?;
		let request = pallet_randomness::Request {
			refund_address,
			contract_address,
			fee,
			gas_limit,
			salt: salt.into(),
			info: pallet_randomness::RequestType::Local(requested_block_number),
		};
		pallet_randomness::Pallet::<Runtime>::request_randomness(request)
			.map_err(|e| error(alloc::format!("{:?}", e)))?;
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: Default::default(),
		})
	}
	/// Fulfill a randomness request due to be fulfilled
	fn fulfill_request(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		let request_id = input.read::<u64>()?;
		// read all the inputs
		let pallet_randomness::FulfillArgs {
			request,
			deposit,
			randomness,
		} = pallet_randomness::Pallet::<Runtime>::prepare_fulfillment(request_id)
			.map_err(|e| error(alloc::format!("{:?}", e)))?;
		// check that randomness can be provided
		ensure_can_provide_randomness::<Runtime>(
			handle.code_address(),
			request.gas_limit,
			request.fee,
			FULFILLMENT_ESTIMATED_COST,
		)?;
		// get gas before subcall
		let before_remaining_gas = handle.remaining_gas();
		// make subcall
		provide_randomness(
			handle,
			request.gas_limit,
			request.contract_address.clone().into(),
			H256(randomness),
		)?;
		// get gas after subcall
		let after_remaining_gas = handle.remaining_gas();
		let gas_used: U256 = before_remaining_gas
			.checked_sub(after_remaining_gas)
			.ok_or(revert("Before remaining gas < After remaining gas"))?
			.into();
		// cost of execution is before_remaining_gas less after_remaining_gas
		let cost_of_execution: BalanceOf<Runtime> = gas_used
			.checked_mul(pallet_base_fee::Pallet::<Runtime>::base_fee_per_gas())
			.ok_or(revert("Multiply gas used by base fee overflowed"))?
			.try_into()
			.map_err(|_| revert("amount is too large for provided balance type"))?;
		// Finish fulfillment to
		// refund cost of execution to caller
		// refund excess fee to the refund_address
		// remove request state
		pallet_randomness::Pallet::<Runtime>::finish_fulfillment(
			request_id,
			request,
			deposit,
			&handle.context().caller,
			cost_of_execution,
		);
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: Default::default(),
		})
	}
	/// Increase the fee used to refund fulfillment of the request
	fn increase_request_fee(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		let request_id = input.read::<u64>()?;
		let fee_increase: BalanceOf<Runtime> = input
			.read::<U256>()?
			.try_into()
			.map_err(|_| revert("amount is too large for provided balance type"))?;
		pallet_randomness::Pallet::<Runtime>::increase_request_fee(
			&handle.context().caller,
			request_id,
			fee_increase,
		)
		.map_err(|e| error(alloc::format!("{:?}", e)))?;
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: Default::default(),
		})
	}
	/// Execute request expiration to remove the request from storage
	/// Transfers `fee` to caller and `deposit` back to `contract_address`
	fn execute_request_expiration(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		let request_id = input.read::<u64>()?;
		pallet_randomness::Pallet::<Runtime>::execute_request_expiration(
			&handle.context().caller,
			request_id,
		)
		.map_err(|e| error(alloc::format!("{:?}", e)))?;
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: Default::default(),
		})
	}
}
