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

use fp_evm::{Context, ExitSucceed, PrecompileHandle, PrecompileHandleExt, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use nimbus_primitives::NimbusId;
use pallet_evm::AddressMapping;
use pallet_evm::Precompile;
use pallet_randomness::Call as RandomnessCall;
use precompile_utils::{
	revert, EvmDataReader, EvmResult, FunctionModifier, Gasometer, PrecompileHandleExt,
	RuntimeHelper,
};
use sp_core::crypto::UncheckedFrom;
use sp_core::H256;
use sp_std::{fmt::Debug, marker::PhantomData};

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

/// Constant gas limit for the subcall to provide randomness
/// The callback assumes only enough gas to store the randomness and emit an event
/// Users should implement logic that consumes the randomness in a separate function that is not
/// directly invoked by this subcall
pub const SUBCALL_GAS_LIMIT: u64 = 6000u64; // TODO: calculate what this should be

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	RequestBabeRandomnessOneEpochAgo = "request_randomness(address,uint256,bytes32,uint256)",
}

/// A precompile to wrap the functionality from pallet author mapping.
pub struct RandomnessWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for RandomnessWrapper<Runtime>
where
	Runtime: pallet_randomness::Config + pallet_evm::Config + frame_system::Config,
	<Runtime as frame_system::Config>::Hash: From<H256>,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		log::trace!(target: "randomness-precompile", "In randomness wrapper");

		let selector = handle.read_selector()?;

		// No funds are transfered to the precompile address.
		// Transfers will directly be made on the behalf of the user by the precompile.
		handle.check_function_modifier(FunctionModifier::NonPayable)?;

		gasometer.check_function_modifier(context, is_static, FunctionModifier::NonPayable)?;

		match selector {
			Action::RequestBabeRandomnessOneEpochAgo => {
				Self::request_babe_randomness_one_epoch_ago(handle)
			}
		}
	}
}

impl<Runtime> RandomnessWrapper<Runtime>
where
	Runtime: pallet_randomness::Config + pallet_evm::Config + frame_system::Config,
	// TODO: why is this specific of a bound required
	// do we use frame_system::pallet::Config in pallet_randomness?
	<Runtime as frame_system::Config>::Hash: From<H256>,
{
	fn request_babe_randomness_one_epoch_ago(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		let contract_address = handle.context().caller;
		let refund_address = input.read::<Address>()?;
		let fee = input.read::<U256>()?;
		let salt = input.read::<H256>()?;
		let epoch_index = input.read::<U256>()?;
		let request = pallet_randomness::Request {
			refund_address,
			contract_address,
			fee,
			salt,
			info: RequestType::BabeOneEpochAgo(epoch_index),
		};
		// log::trace!(
		// 	target: "randomness-precompile",
		// 	"Requesting randomness {:?}", request
		// );
		pallet_randomness::Pallet::<Runtime>::request_randomness(request)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	} // TODO: 2 epochs ago once confirmed we want it
	/// Make request for local VRF randomness
	fn request_local_randomness(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		let contract_address = context.caller;
		let refund_address = input.read::<Address>()?;
		let fee = input.read::<U256>()?;
		let salt = input.read::<H256>()?;
		let block_number = input.read::<U256>()?;
		let request = pallet_randomness::Request {
			refund_address,
			contract_address,
			fee,
			salt,
			info: RequestType::Local(block_number),
		};
		// log::trace!(
		// 	target: "randomness-precompile",
		// 	"Requesting randomness {:?}", request
		// );
		pallet_randomness::Pallet::<Runtime>::request_randomness(request)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	pub fn provide_randomness(
		handle: &mut impl PrecompileHandle,
		contract: H160,
		randomness: H256,
	) -> (ExitReason, Vec<u8>) {
		handle.call(
			contract,
			None,
			EvmDataWriter::new().write(randomness).build(),
			Some(SUBCALL_GAS_LIMIT),
			false,
			&Context {
				caller: handle.context().address, // precompile address
				address: contract,
				apparent_value: U256::zero(),
			},
		)
	}
	// precompile
	// only provide storage deposit, don't provide gas_limit
	// the hardcoded gas_limit is enough just to store the randomness and maybe emit an event
	// refund cost of execution to one who calls it, what has not been used
	fn fulfill(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		// read all the inputs
		// ::prepare fulfill --> check no state changes
		let before_remaining_gas = handle.remaining_gas();
		// add GasLimit + call_cost() and ensure caller passed in limit is above
		// if not enough, should not be refunded and should revert
		match Self::provide_randomness(handle) {}
		// revert() if subcall OOG
		let after_remaining_gas = handle.remaining_gas();
		// if 0 ==> reverted because of OOG of the precompile
		// must revert
		if after_remaining_gas.is_zero() {
			// precompile OOG so must revert and consuming contract loses their deposit sans refund
			return Err(revert("OOG so deposit"));
		}
		let cost_of_execution = before_remaining_gas
			.checked_sub(after_remaining_gas)
			.ok_or(revert("Before remaining gas < After remaining gas"))?
			.checked_mul(pallet_base_fee::Pallet::base_fee_per_gas())
			.ok_or(revert("Multiply cost of execution by base fee overflowed"))?;
		// must refund it by the requester
		// ::post fulfill
	}
	// TODO: increase request fee
	// TODO: execute request expiration
}
