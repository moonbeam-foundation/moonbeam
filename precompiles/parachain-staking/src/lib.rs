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
use pallet_evm::Precompile;
use precompile_utils::{error, Gasometer, InputReader, OutputBuilder, RuntimeHelper};
use sp_core::{H160, U256};
use sp_std::convert::{TryFrom, TryInto};
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;
use sp_std::vec;

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
			// constants
			[0xc9, 0xf5, 0x93, 0xb2] => Self::min_nomination(target_gas),
			// storage getters
			[0x97, 0x99, 0xb4, 0xe7] => Self::points(input, target_gas),
			[0x4b, 0x1c, 0x4c, 0x29] => Self::candidate_count(target_gas),
			[0x0a, 0xd6, 0xa7, 0xbe] => Self::collator_nomination_count(input, target_gas),
			[0xda, 0xe5, 0x65, 0x9b] => Self::nominator_nomination_count(input, target_gas),
			// role verifiers
			[0x8e, 0x50, 0x80, 0xe7] => Self::is_nominator(input, target_gas),
			[0x85, 0x45, 0xc8, 0x33] => Self::is_candidate(input, target_gas),
			[0x8f, 0x6d, 0x27, 0xc7] => Self::is_selected_candidate(input, target_gas),
			// runtime methods (setters)
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

impl<Runtime> ParachainStakingWrapper<Runtime>
where
	Runtime: parachain_staking::Config + pallet_evm::Config + frame_system::Config,
	Runtime::AccountId: From<H160>,
	BalanceOf<Runtime>: TryFrom<U256> + TryInto<u128> + Debug,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<parachain_staking::Call<Runtime>>,
{
	// Constants

	fn min_nomination(target_gas: Option<u64>) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let min_nomination: u128 = <<Runtime as parachain_staking::Config>::MinNomination as Get<
			BalanceOf<Runtime>,
		>>::get()
		.try_into()
		.map_err(|_| ExitError::Other("Amount is too large for provided balance type".into()))?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: OutputBuilder::new().write_u256(min_nomination).build(),
			logs: vec![],
		})
	}

	// Storage Getters

	fn points(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let round = input.read_u32()?;

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let points: u32 = parachain_staking::Pallet::<Runtime>::points(round);

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
		let candidate_count: u32 = <parachain_staking::Pallet<Runtime>>::candidate_pool()
			.0
			.len() as u32;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
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

	// Role Verifiers

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

	// Runtime Methods (setters)

	fn join_candidates(
		mut input: InputReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(2)?;

		let amount = input.read_balance::<BalanceOf<Runtime>>()?;
		let collator_candidate_count = input.read_u32()?;
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

		let collator_candidate_count = input.read_u32()?;

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

		let amount = input.read_balance::<BalanceOf<Runtime>>()?;

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

		let amount = input.read_balance::<BalanceOf<Runtime>>()?;

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
		let amount = input.read_balance::<BalanceOf<Runtime>>()?;
		let collator_nomination_count = input.read_u32()?;
		let nominator_nomination_count = input.read_u32()?;

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

		let nomination_count = input.read_u32()?;

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
		let amount = input.read_balance::<BalanceOf<Runtime>>()?;

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
		let amount = input.read_balance::<BalanceOf<Runtime>>()?;

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
			logs: vec![],
		})
	}
}
