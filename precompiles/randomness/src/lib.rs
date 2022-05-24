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

use fp_evm::{Context, ExitSucceed, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use nimbus_primitives::NimbusId;
use pallet_evm::AddressMapping;
use pallet_evm::Precompile;
use pallet_randomness::Call as RandomnessCall;
use precompile_utils::{EvmDataReader, EvmResult, FunctionModifier, Gasometer, RuntimeHelper};
use sp_core::crypto::UncheckedFrom;
use sp_core::H256;
use sp_std::{fmt::Debug, marker::PhantomData};

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

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
	fn execute(
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> EvmResult<PrecompileOutput> {
		log::trace!(target: "randomness-precompile", "In randomness wrapper");

		let mut gasometer = Gasometer::new(target_gas);
		let gasometer = &mut gasometer;

		let (mut input, selector) = EvmDataReader::new_with_selector(gasometer, input)?;
		let input = &mut input;

		gasometer.check_function_modifier(context, is_static, FunctionModifier::NonPayable)?;

		match selector {
			Action::RequestBabeRandomnessOneEpochAgo => {
				Self::request_babe_randomness_one_epoch_ago(input, gasometer, context)
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
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check
		input.expect_arguments(gasometer, 3)?;
		let contract_address = context.caller;
		let refund_address = input.read::<Address>(gasometer)?;
		let fee = input.read::<U256>(gasometer)?;
		let salt = input.read::<H256>(gasometer)?;
		let epoch_index = input.read::<U256>(gasometer)?;
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
	fn request_local_randomness(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check
		input.expect_arguments(gasometer, 3)?;
		let contract_address = context.caller;
		let refund_address = input.read::<Address>(gasometer)?;
		let fee = input.read::<U256>(gasometer)?;
		let salt = input.read::<H256>(gasometer)?;
		let block_number = input.read::<U256>(gasometer)?;
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
	// precompile
	fn fulfill() {
		// ::prepare fulfill
		// subcall
		// revert() if subcall OOG
		// ::post fulfill
	}
	// TODO: increase request fee
	// TODO: execute request expiration
}
