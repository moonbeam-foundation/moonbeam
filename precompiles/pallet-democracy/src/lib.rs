// Copyright 2019-2021 PureStake Inc.
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

//! Precompile to interact with pallet democracy through an evm precompile.

#![cfg_attr(not(feature = "std"), no_std)]

use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::Currency;
use pallet_democracy::Call as DemocracyCall;
use pallet_evm::AddressMapping;
use pallet_evm::Precompile;
use precompile_utils::{
	error, EvmDataReader, EvmDataWriter, EvmResult, Gasometer, RuntimeHelper, EvmData,
};
use sp_core::{H160, H256, U256};
use sp_std::convert::{TryFrom, TryInto};
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type BalanceOf<Runtime> = <<Runtime as pallet_democracy::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

type DemocracyOf<Runtime> = pallet_democracy::Pallet<Runtime>;

/// A precompile to wrap the functionality from pallet democracy.
///
/// Grants evm-based DAOs the right to vote making them first-class citizens.
///
/// EXAMPLE USECASE:
/// A political party that citizens delegate their vote to, and the party votes on their behalf.
pub struct DemocracyWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for DemocracyWrapper<Runtime>
where
	Runtime: pallet_democracy::Config + pallet_evm::Config,
	BalanceOf<Runtime>: TryFrom<U256> + Debug + EvmData,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<DemocracyCall<Runtime>>,
	Runtime::Hash: From<H256>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		log::trace!(target: "democracy-precompile", "In democracy wrapper");

		let mut input = EvmDataReader::new(input);

		// Parse the function selector
		// These are the four-byte function selectors calculated from the DemocracyInterface.sol
		// according to the solidity specification
		// https://docs.soliditylang.org/en/v0.8.0/abi-spec.html#function-selector
		match &input.read_selector()? {
			// Check for accessor methods first. These return results immediately
			[0x56, 0xfd, 0xf5, 0x47] => Self::public_prop_count(input, target_gas),
			// Now the dispatchables
			[0x78, 0x24, 0xe7, 0xd1] => Self::propose(input, target_gas, context),
			[0xc7, 0xa7, 0x66, 0x01] => Self::second(input, target_gas, context),
			_ => {
				log::trace!(
					target: "democracy-precompile",
					"Failed to match function selector in democracy precompile"
				);
				Err(error("No democracy wrapper method at given selector"))
			}
		}
	}
}

impl<Runtime> DemocracyWrapper<Runtime>
where
	Runtime: pallet_democracy::Config + pallet_evm::Config + frame_system::Config,
	BalanceOf<Runtime>: TryFrom<U256> + TryInto<u128> + Debug + EvmData,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<DemocracyCall<Runtime>>,
	Runtime::Hash: From<H256>,
{
	// The accessors are first. They directly return their result.

	fn public_prop_count(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> EvmResult<PrecompileOutput> {
		// TODO Ensure there is no additional input passed
		
		let mut gasometer = Gasometer::new(target_gas);

		// Fetch data from pallet
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let prop_count = DemocracyOf::<Runtime>::public_prop_count();
		log::trace!(target: "democracy-precompile", "Result from pallet is {:?}", prop_count);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(prop_count).build(),
			logs: Default::default(),
		})
	}

	// The dispatchable wrappers are next. They return a substrate inner Call ready for dispatch.

	fn propose(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(2)?;

		let proposal_hash = input.read::<H256>()?;
		let amount = input.read::<BalanceOf<Runtime>>()?;

		log::trace!(target: "democracy-precompile", "Proposing with hash {:?}, and amount {:?}", proposal_hash, amount);
		
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::propose(
			proposal_hash.into(),
			amount,
		);

		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;

		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn second(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(2)?;

		// Woah! I do't even need type annotations!
		let proposal_index = input.read()?;
		let seconds_upper_bound = input.read()?;

		log::trace!(target: "democracy-precompile", "Seconding proposal {:?}, with bound {:?}", proposal_index, seconds_upper_bound);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::second(
			proposal_index,
			seconds_upper_bound,
		);

		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;

		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}
}
