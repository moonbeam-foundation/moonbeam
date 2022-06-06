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

use fp_evm::{Context, ExitReason, ExitSucceed, Log, PrecompileHandle, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use nimbus_primitives::NimbusId;
use pallet_evm::AddressMapping;
use pallet_evm::Precompile;
use pallet_randomness::{BalanceOf, Call as RandomnessCall};
use precompile_utils::{
	call_cost, error, keccak256, revert, Address, EvmData, EvmDataReader, EvmDataWriter, EvmResult,
	FunctionModifier, LogExt, LogsBuilder, PrecompileHandleExt, RuntimeHelper,
};
use sp_core::crypto::UncheckedFrom;
use sp_core::{H160, H256, U256};
use sp_std::{fmt::Debug, marker::PhantomData};

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

pub const LOG_SUBCALL_SUCCEEDED: [u8; 32] = keccak256!("SubcallSucceeded");
pub const LOG_SUBCALL_FAILED: [u8; 32] = keccak256!("SubcallFailed");

pub fn log_subcall_succeeded(address: impl Into<H160>) -> Log {
	LogsBuilder::new(address.into()).log0(LOG_SUBCALL_SUCCEEDED)
}

pub fn log_subcall_failed(address: impl Into<H160>) -> Log {
	LogsBuilder::new(address.into()).log0(LOG_SUBCALL_FAILED)
}

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	RequestBabeRandomnessOneEpochAgo = "request_randomness(address,uint256,bytes32,uint256)",
}

/// A precompile to wrap the functionality from pallet author mapping.
pub struct RandomnessWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for RandomnessWrapper<Runtime>
where
	Runtime: pallet_randomness::Config + pallet_evm::Config + pallet_base_fee::Config,
	<Runtime as frame_system::Config>::Hash: From<H256> + Into<H256>,
	<Runtime as frame_system::Config>::AccountId: From<H160> + Into<H160> + From<Address>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		log::trace!(target: "randomness-precompile", "In randomness wrapper");

		let selector = handle.read_selector()?;

		// No funds are transferred to the precompile address.
		// Transfers will directly be made on behalf of the user by the precompile.
		handle.check_function_modifier(FunctionModifier::NonPayable)?;

		match selector {
			Action::RequestBabeRandomnessOneEpochAgo => {
				Self::request_babe_randomness_one_epoch_ago(handle)
			}
		}
	}
}

impl<Runtime> RandomnessWrapper<Runtime>
where
	Runtime: pallet_randomness::Config + pallet_evm::Config + pallet_base_fee::Config,
	<Runtime as frame_system::Config>::Hash: From<H256> + Into<H256>,
	<Runtime as frame_system::Config>::AccountId: From<H160> + Into<H160>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
{
	fn request_babe_randomness_one_epoch_ago(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		let contract_address = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let refund_address = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let fee: BalanceOf<Runtime> = input.read::<U256>()?.into();
		let gas_limit = input.read::<u64>()?;
		let salt = input.read::<H256>()?;
		let epoch_index = input.read::<u64>()?;
		let request = pallet_randomness::Request {
			refund_address,
			contract_address: contract_address.into(),
			fee,
			gas_limit,
			salt: salt.into(),
			info: pallet_randomness::RequestType::BabeOneEpochAgo(epoch_index),
		};
		// log::trace!(
		// 	target: "randomness-precompile",
		// 	"Requesting randomness {:?}", request
		// );
		pallet_randomness::Pallet::<Runtime>::request_randomness(request)
			.map_err(|_| error("failed to request randomness"))?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: Default::default(),
		})
	} // TODO: 2 epochs ago once confirmed we want it
	/// Make request for local VRF randomness
	fn request_local_randomness(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		let contract_address = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let refund_address = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let fee: BalanceOf<Runtime> = input.read::<U256>()?.into();
		let gas_limit = input.read::<u64>()?;
		let salt = input.read::<H256>()?; // TODO: convert to Hash type or just use H256?
		let block_number = input.read::<U256>()?;
		let request = pallet_randomness::Request {
			refund_address,
			contract_address,
			fee,
			gas_limit,
			salt: salt.into(),
			info: pallet_randomness::RequestType::Local(block_number),
		};
		pallet_randomness::Pallet::<Runtime>::request_randomness(request)
			.map_err(|_| error("failed to request randomness"))?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: Default::default(),
		})
	}
	/// Subcall to store randomness and emit an event by input `contract`
	// TODO: add atomic bool to prevent reentrancy
	pub fn provide_randomness(
		handle: &mut impl PrecompileHandle,
		gas_limit: u64,
		contract: H160,
		randomness: H256,
	) -> EvmResult<(ExitReason, Vec<u8>)> {
		// if not true, revert but keep request in storage (before callback)
		let log_cost = log_subcall_failed(handle.code_address())
			.compute_cost()
			.map_err(|_| revert("failed to compute log cost"))?;
		// Cost of the call itself that the batch precompile must pay.
		let call_cost = call_cost(U256::zero(), <Runtime as pallet_evm::Config>::config());
		// assert gasLimit > overhead call_cost!
		if gas_limit <= call_cost + log_cost {
			return Err(revert("Gas limit must exceed overhead call cost"));
		}
		Ok(handle.call(
			contract,
			None,
			EvmDataWriter::new().write(randomness).build(),
			Some(gas_limit),
			false,
			&Context {
				caller: handle.context().address, // precompile address
				address: contract,
				apparent_value: U256::zero(),
			},
		))
	}
	/// Fulfill a randomness request due to be fulfilled
	fn fulfill(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		// only get requestId for prepare fulfillment
		let id = input.read::<u64>()?;
		// read all the inputs
		let pallet_randomness::FulfillArgs {
			request,
			deposit,
			randomness,
		} = pallet_randomness::Pallet::<Runtime>::prepare_fulfillment(id)
			.map_err(|_| error("failed to prepare fulfillment"))?;
		// assert fee / base_fee > gasLimit
		let fee_as_u256: U256 = request.fee.into();
		let fees_available: U256 =
			fee_as_u256 / pallet_base_fee::Pallet::<Runtime>::base_fee_per_gas();
		if fees_available.into() <= request.gas_limit {
			return Err(revert(
				"Gas limit at current price must exceed fees allotted",
			));
		}
		// get gas before subcall
		let before_remaining_gas = handle.remaining_gas();
		// make subcall
		let (reason, output) = Self::provide_randomness(
			handle,
			request.gas_limit,
			request.contract_address, // TODO: convert from AccountId to H160
			randomness,
		)?;
		// get gas before subcall
		let after_remaining_gas = handle.remaining_gas();
		// cost of execution is before_remaining_gas less after_remaining_gas
		let cost_of_execution = before_remaining_gas
			.checked_sub(after_remaining_gas)
			.ok_or(revert("Before remaining gas < After remaining gas"))?
			.checked_mul(pallet_base_fee::Pallet::<Runtime>::base_fee_per_gas().into())
			.ok_or(revert("Multiply cost of execution by base fee overflowed"))?;
		// Finish fulfillment to
		// refund cost of execution to caller
		// refund excess fee to the refund_address
		// remove request state
		pallet_randomness::Pallet::<Runtime>::finish_fulfillment(
			id,
			request,
			deposit,
			&Runtime::AddressMapping::into_account_id(handle.context().caller),
			cost_of_execution,
		);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: Default::default(),
		})
	}
	// TODO: increase request fee
	// TODO: execute request expiration
}
