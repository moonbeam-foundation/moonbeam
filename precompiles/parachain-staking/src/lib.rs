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

//! Precompile to call parachain-staking runtime methods via the EVM

#![cfg_attr(not(feature = "std"), no_std)]

use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::{Currency, Get};
use pallet_evm::AddressMapping;
use pallet_evm::GasWeightMapping;
use pallet_evm::Precompile;
use precompile_utils::{
	error, EvmResult, Gasometer, InputReader, LogsBuilder, OutputBuilder, RuntimeHelper,
};
use sp_core::{H160, U256};
use sp_std::convert::{TryFrom, TryInto};
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;
use sp_std::{vec, vec::Vec};

type BalanceOf<Runtime> = <<Runtime as parachain_staking::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

/// A precompile to wrap the functionality from parachain_staking.
///
/// EXAMPLE USECASE:
/// A simple example usecase is a contract that allows donors to donate, and stakes all the funds
/// toward one fixed address chosen by the deployer.
/// Such a contract could be deployed by a collator candidate, and the deploy address distributed to
/// supporters who want to donate toward a perpetual nomination fund.
pub struct ParachainStakingWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for ParachainStakingWrapper<Runtime>
where
	Runtime: parachain_staking::Config + pallet_evm::Config,
	Runtime::AccountId: From<H160>,
	BalanceOf<Runtime>: TryFrom<U256> + Debug,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<parachain_staking::Call<Runtime>>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let input = InputReader::new(input)?;

		match input.selector() {
			[0x8e, 0x50, 0x80, 0xe7] => Self::is_nominator(input, target_gas),
			[0x85, 0x45, 0xc8, 0x33] => Self::is_candidate(input, target_gas),
			[0x8f, 0x6d, 0x27, 0xc7] => Self::is_selected_candidate(input, target_gas),
			[0xc9, 0xf5, 0x93, 0xb2] => Self::min_nomination(target_gas),
			[0x97, 0x99, 0xb4, 0xe7] => Self::points(input, target_gas),
			[0x4b, 0x1c, 0x4c, 0x29] => Self::candidate_count(target_gas),
			[0x0a, 0xd6, 0xa7, 0xbe] => Self::collator_nomination_count(input, target_gas),
			[0xda, 0xe5, 0x65, 0x9b] => Self::nominator_nomination_count(input, target_gas),
			[0x0a, 0x1b, 0xff, 0x60] => Self::join_candidates(input, target_gas, context),
			[0x72, 0xb0, 0x2a, 0x31] => Self::leave_candidates(input, target_gas, context),
			[0x76, 0x7e, 0x04, 0x50] => Self::go_offline(target_gas, context),
			[0xd2, 0xf7, 0x3c, 0xeb] => Self::go_online(target_gas, context),
			[0x28, 0x9b, 0x6b, 0xa7] => Self::candidate_bond_less(input, target_gas, context),
			[0xc5, 0x7b, 0xd3, 0xa8] => Self::candidate_bond_more(input, target_gas, context),
			[0x49, 0xdf, 0x6e, 0xb3] => Self::nominate(input, target_gas, context),
			[0xb7, 0x1d, 0x21, 0x53] => Self::leave_nominators(input, target_gas, context),
			[0x4b, 0x65, 0xc3, 0x4b] => Self::revoke_nomination(input, target_gas, context),
			[0xf6, 0xa5, 0x25, 0x69] => Self::nominator_bond_less(input, target_gas, context),
			[0x97, 0x1d, 0x44, 0xc8] => Self::nominator_bond_more(input, target_gas, context),
			_ => Err(error("no selector found")),
		}
	}
}

/// Parses an H160 account address from a 256 bit (32 byte) buffer. Only the last 20 bytes are used.
fn parse_account(input: &[u8]) -> Result<H160, ExitError> {
	const PADDING_SIZE_BYTES: usize = 12;
	const ACCOUNT_SIZE_BYTES: usize = 20;
	const TOTAL_SIZE_BYTES: usize = PADDING_SIZE_BYTES + ACCOUNT_SIZE_BYTES;

	if input.len() != TOTAL_SIZE_BYTES {
		log::trace!(target: "staking-precompile",
			"Unable to parse address. Got {} bytes, expected {}",
			input.len(),
			TOTAL_SIZE_BYTES,
		);
		return Err(ExitError::Other(
			"Incorrect input length for account parsing".into(),
		));
	}

	Ok(H160::from_slice(
		&input[PADDING_SIZE_BYTES..TOTAL_SIZE_BYTES],
	))
}

/// Parses an amount of ether from a 256 bit (32 byte) slice. The balance type is generic.
/// TODO: move to precompile-utils
fn u256_to_amount<Balance: TryFrom<U256>>(input: U256) -> Result<Balance, ExitError> {
	Ok(input
		.try_into()
		.map_err(|_| ExitError::Other("Amount is too large for provided balance type".into()))?)
}

/// Parses Weight Hint: u32 from a U256
/// TODO: move to precompile-utils
fn u256_to_u32(input: U256) -> Result<u32, ExitError> {
	Ok(input
		.try_into()
		.map_err(|_| ExitError::Other("Too large for u32".into()))?)
}

impl<Runtime> ParachainStakingWrapper<Runtime>
where
	Runtime: parachain_staking::Config + pallet_evm::Config + frame_system::Config,
	Runtime::AccountId: From<H160>,
	BalanceOf<Runtime>: TryFrom<U256> + TryInto<u128> + Debug,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<parachain_staking::Call<Runtime>>,
{
	// TODO: reorder
	// constants, storage getters, role verifiers, setters

	fn is_nominator(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let address = input.read_address::<Runtime::AccountId>()?;

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_nominator = parachain_staking::Pallet::<Runtime>::is_nominator(&address.into());

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: OutputBuilder::new().write_bool(is_nominator).build(),
			logs: vec![],
		})
	}

	fn is_candidate(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let address = input.read_address::<Runtime::AccountId>()?;

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_candidate = parachain_staking::Pallet::<Runtime>::is_candidate(&address.into());

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: OutputBuilder::new().write_bool(is_candidate).build(),
			logs: vec![],
		})
	}

	fn is_selected_candidate(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let address = input.read_address::<Runtime::AccountId>()?;

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_selected =
			parachain_staking::Pallet::<Runtime>::is_selected_candidate(&address.into());

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: OutputBuilder::new().write_bool(is_selected).build(),
			logs: vec![],
		})
	}

	fn min_nomination(target_gas: Option<u64>) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let raw_min_nomination: u128 = <
			<Runtime as parachain_staking::Config>::MinNomination
				as Get<BalanceOf<Runtime>>
			>::get().try_into()
				.map_err(|_|
					ExitError::Other("Amount is too large for provided balance type".into())
				)?;
		let min_nomination: U256 = raw_min_nomination.into();

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			// TODO: can use raw_min_nomination
			output: OutputBuilder::new().write_u256(min_nomination).build(),
			logs: vec![],
		})
	}

	fn points(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let round = u256_to_u32(input.read_u256()?)?;

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let points: u32 = parachain_staking::Pallet::<Runtime>::points(round);

		// TODO: is this necessary
		// Make sure the round number fits in a u32
		// if round_u256.leading_zeros() < 256 - 32 {
		// 	return Err(ExitError::Other(
		// 		"Round is too large. 32 bit maximum".into(),
		// 	));
		// }
		// let round: u32 = round_u256.low_u32();

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: OutputBuilder::new().write_u256(points).build(),
			logs: vec![],
		})
	}

	fn candidate_count(target_gas: Option<u64>) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let raw_candidate_count: u32 = <parachain_staking::Pallet<Runtime>>::candidate_pool()
			.0
			.len() as u32;
		let candidate_count: U256 = raw_candidate_count.into();

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			// TODO: can use raw
			output: OutputBuilder::new().write_u256(candidate_count).build(),
			logs: vec![],
		})
	}

	fn collator_nomination_count(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let address = input.read_address::<Runtime::AccountId>()?;

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let result: U256 =
			if let Some(state) = <parachain_staking::Pallet<Runtime>>::collator_state2(&address) {
				let collator_nomination_count: u32 = state.nominators.0.len() as u32;

				log::trace!(
					target: "staking-precompile",
					"Result from pallet is {:?}",
					collator_nomination_count
				);
				collator_nomination_count.into()
			} else {
				log::trace!(
					target: "staking-precompile",
					"Collator {:?} not found, so nomination count is 0",
					address
				);
				U256::zero()
			};

		// TODO: check length and same for all of the others, can panic if len < 32

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: OutputBuilder::new().write_u256(result).build(),
			logs: vec![],
		})
	}

	fn nominator_nomination_count(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let address = input.read_address::<Runtime::AccountId>()?;

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let result: U256 =
			if let Some(state) = <parachain_staking::Pallet<Runtime>>::nominator_state(&address) {
				let nominator_nomination_count: u32 = state.nominations.0.len() as u32;

				log::trace!(
					target: "staking-precompile",
					"Result from pallet is {:?}",
					nominator_nomination_count
				);

				nominator_nomination_count.into()
			} else {
				log::trace!(
					target: "staking-precompile",
					"Nominator {:?} not found, so nomination count is 0",
					address
				);
				U256::zero()
			};

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: OutputBuilder::new().write_u256(result).build(),
			logs: vec![],
		})
	}

	// The dispatchable wrappers are next. They return a substrate inner Call ready for dispatch.

	fn join_candidates(
		mut input: InputReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(2)?;

		let amount = u256_to_amount::<BalanceOf<Runtime>>(input.read_u256()?)?;
		let collator_candidate_count = u256_to_u32(input.read_u256()?)?;
		// let amount = parse_amount::<BalanceOf<Runtime>>(&input[..32])?;
		// let collator_candidate_count = parse_weight_hint(&input[32..])?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call =
			parachain_staking::Call::<Runtime>::join_candidates(amount, collator_candidate_count);

		// Dispatch call (if enough gas).
		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
			// TODO: log amount bonded
			logs: vec![],
		})
	}

	fn leave_candidates(
		mut input: InputReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let collator_candidate_count = u256_to_u32(input.read_u256()?)?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::leave_candidates(collator_candidate_count);

		// Dispatch call (if enough gas).
		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
			// TODO: log amount bonded
			logs: vec![],
		})
	}

	fn go_offline(
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::go_offline();

		// Dispatch call (if enough gas).
		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
			logs: vec![],
		})
	}

	fn go_online(
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::go_online();

		// Dispatch call (if enough gas).
		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
			logs: vec![],
		})
	}

	fn candidate_bond_more(
		mut input: InputReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let amount = u256_to_amount(input.read_u256()?)?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::candidate_bond_more(amount);

		// Dispatch call (if enough gas).
		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
			// TODO: log amount bonded
			logs: vec![],
		})
	}

	fn candidate_bond_less(
		mut input: InputReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let amount = u256_to_amount(input.read_u256()?)?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::candidate_bond_less(amount);

		// Dispatch call (if enough gas).
		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
			// TODO: log amount bonded
			logs: vec![],
		})
	}

	fn nominate(
		mut input: InputReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(4)?;

		let collator = input.read_address::<Runtime::AccountId>()?;
		let amount = u256_to_amount::<BalanceOf<Runtime>>(input.read_u256()?)?;
		let collator_nomination_count = u256_to_u32(input.read_u256()?)?;
		let nominator_nomination_count = u256_to_u32(input.read_u256()?)?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::nominate(
			collator.into(),
			amount,
			collator_nomination_count,
			nominator_nomination_count,
		);

		// Dispatch call (if enough gas).
		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
			// TODO: log amount bonded
			logs: vec![],
		})
	}

	fn leave_nominators(
		mut input: InputReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let nomination_count = u256_to_u32(input.read_u256()?)?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::leave_nominators(nomination_count);

		// Dispatch call (if enough gas).
		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
			// TODO: log amount bonded
			logs: vec![],
		})
	}

	fn revoke_nomination(
		mut input: InputReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let collator = input.read_address::<Runtime::AccountId>()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::revoke_nomination(collator.into());

		// Dispatch call (if enough gas).
		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
			// TODO: log amount bonded
			logs: vec![],
		})
	}

	fn nominator_bond_more(
		mut input: InputReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(2)?;

		let collator = input.read_address::<Runtime::AccountId>()?;
		let amount = u256_to_amount(input.read_u256()?)?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::nominator_bond_more(collator.into(), amount);

		// Dispatch call (if enough gas).
		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
			// TODO: log amount bonded
			logs: vec![],
		})
	}

	fn nominator_bond_less(
		mut input: InputReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(2)?;

		let collator = input.read_address::<Runtime::AccountId>()?;
		let amount = u256_to_amount(input.read_u256()?)?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::nominator_bond_less(collator.into(), amount);

		// Dispatch call (if enough gas).
		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
			// TODO: log amount bonded
			logs: vec![],
		})
	}
}
