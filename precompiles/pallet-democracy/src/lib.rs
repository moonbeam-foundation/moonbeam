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
use pallet_democracy::{AccountVote, Call as DemocracyCall, Vote};
use pallet_evm::AddressMapping;
use pallet_evm::Precompile;
use precompile_utils::{
	error, EvmData, EvmDataReader, EvmDataWriter, EvmResult, Gasometer, RuntimeHelper, Address,
};
// TODO there is a warning about H160 not being used. But when I remove it I get errors.
use sp_core::{H160, H256, U256};
use sp_std::convert::{TryFrom, TryInto};
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;

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
			[0x56, 0xfd, 0xf5, 0x47] => Self::public_prop_count(target_gas),
			// [0xb0, 0x89, 0xf2, 0x02] => Self::public_props(target_gas),
			[0xa3, 0x03, 0x05, 0xe9] => Self::deposit_of(input, target_gas),
			[0x03, 0x88, 0xf2, 0x82] => Self::lowest_unbaked(target_gas),
			[0x8b, 0x93, 0xd1, 0x1a] => Self::ongoing_referendum_info(input, target_gas),
			[0xb1, 0xfd, 0x38, 0x37] => Self::finished_referendum_info(input, target_gas),
			
			// Now the dispatchables
			[0x78, 0x24, 0xe7, 0xd1] => Self::propose(input, target_gas, context),
			[0xc7, 0xa7, 0x66, 0x01] => Self::second(input, target_gas, context),
			[0x35, 0xcd, 0xe7, 0xae] => Self::standard_vote(input, target_gas, context),
			[0x20, 0x42, 0xf5, 0x0b] => Self::remove_vote(input, target_gas, context),
			[0x01, 0x85, 0x92, 0x1e] => Self::delegate(input, target_gas, context),
			[0xcb, 0x37, 0xb8, 0xea] => Self::un_delegate(target_gas, context),
			[0x2f, 0x6c, 0x49, 0x3c] => Self::unlock(input, target_gas, context),
			_ => {
				println!("Failed to match function selector in democracy precompile");
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

	fn public_prop_count(target_gas: Option<u64>) -> EvmResult<PrecompileOutput> {
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

	fn public_props(_target_gas: Option<u64>) -> EvmResult<PrecompileOutput> {
		todo!()
	}

	fn deposit_of(mut input: EvmDataReader, target_gas: Option<u64>) -> EvmResult<PrecompileOutput> {
		todo!()
	}

	fn lowest_unbaked(target_gas: Option<u64>) -> EvmResult<PrecompileOutput> {
		todo!()
	}

	fn ongoing_referendum_info(mut input: EvmDataReader, target_gas: Option<u64>) -> EvmResult<PrecompileOutput> {
		todo!()
	}

	fn finished_referendum_info(mut input: EvmDataReader, target_gas: Option<u64>) -> EvmResult<PrecompileOutput> {
		todo!()
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
		let call = DemocracyCall::<Runtime>::propose(proposal_hash.into(), amount);

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
		println!("in second");
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(2)?;

		// Woah! I do't even need type annotations!
		let proposal_index = input.read()?;
		let seconds_upper_bound = input.read()?;

		log::trace!(target: "democracy-precompile", "Seconding proposal {:?}, with bound {:?}", proposal_index, seconds_upper_bound);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::second(proposal_index, seconds_upper_bound);

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

	fn standard_vote(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// TODO Why arethese not printing??
		println!("in standard vote");
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(4)?;

		let ref_index = input.read()?;
		let aye = input.read()?;
		let balance = input.read()?;
		let conviction = input
			.read::<u8>()?
			.try_into()
			.map_err(|_| error("Conviction must be an integer in the range 0-6"))?;
		let account_vote = AccountVote::Standard {
			vote: Vote { aye, conviction },
			balance,
		};

		println!(
			"Voting {:?} on referendum #{:?}, with conviction {:?}",
			aye, ref_index, conviction
		);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::vote(ref_index, account_vote);

		//TODO would be slightly ncer to pass a mutable gasometer.
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

	fn remove_vote(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(1)?;

		let ref_index = input.read()?;

		println!("Removing vote from referendum {:?}", ref_index);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::remove_vote(ref_index);

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

	fn delegate(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(3)?;

		let to_address: H160 = input.read::<Address>()?.into();
		let to_account = Runtime::AddressMapping::into_account_id(to_address);
		let conviction = input
			.read::<u8>()?
			.try_into()
			.map_err(|_| error("Conviction must be an integer in the range 0-6"))?;
		let balance = input.read()?;

		println!("Delegating vote to {:?} with balance {:?} and {:?}", to_account, conviction, balance);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::delegate(to_account, conviction, balance);

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

	fn un_delegate(
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		println!("Undelegating vote");

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::undelegate();

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

	fn unlock(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(1)?;

		let target_address: H160 = input.read::<Address>()?.into();
		let target_account = Runtime::AddressMapping::into_account_id(target_address);

		println!("Unlocking democracy tokens for {:?}", target_account);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::unlock(target_account);

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
