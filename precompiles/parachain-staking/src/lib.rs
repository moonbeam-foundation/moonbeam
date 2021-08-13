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

extern crate alloc;
use alloc::format;
use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::{Currency, Get};
use pallet_evm::AddressMapping;
use pallet_evm::Precompile;
use precompile_utils::{error, Address, EvmDataReader, EvmDataWriter, Gasometer, RuntimeHelper};
use sp_core::{H160, U256};
use sp_std::convert::{TryFrom, TryInto};
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;
use sp_std::vec;

type BalanceOf<Runtime> = <<Runtime as parachain_staking::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, num_enum::TryFromPrimitive)]
enum Action {
	MinNomination = "min_nomination()",
	Points = "points(uint256)",
	IsNominator = "is_nominator(address)",
	IsCandidate = "is_candidate(address)",
	IsSelectedCandidate = "is_selected_candidate(address)",
	JoinCandidates = "join_candidates(uint256,uint256)",
	LeaveCandidates = "leave_candidates(uint256)",
	GoOffline = "go_offline()",
	GoOnline = "go_online()",
	CandidateBondLess = "candidate_bond_less(uint256)",
	CandidateBondMore = "candidate_bond_more(uint256)",
	Nominate = "nominate(address,uint256,uint256,uint256)",
	LeaveNominators = "leave_nominators(uint256)",
	RevokeNomination = "revoke_nomination(address)",
	NominatorBondLess = "nominator_bond_less(address,uint256)",
	NominatorBondMore = "nominator_bond_more(address,uint256)",
}

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
		let mut input = EvmDataReader::new(input);

		match &input.read_selector()? {
			// constants
			Action::MinNomination => Self::min_nomination(target_gas),
			// storage getters
			Action::Points => Self::points(input, target_gas),
			// role verifiers
			Action::IsNominator => Self::is_nominator(input, target_gas),
			Action::IsCandidate => Self::is_candidate(input, target_gas),
			Action::IsSelectedCandidate => Self::is_selected_candidate(input, target_gas),
			// runtime methods (dispatchables)
			Action::JoinCandidates => Self::join_candidates(input, target_gas, context),
			Action::LeaveCandidates => Self::leave_candidates(input, target_gas, context),
			Action::GoOffline => Self::go_offline(target_gas, context),
			Action::GoOnline => Self::go_online(target_gas, context),
			Action::CandidateBondLess => Self::candidate_bond_less(input, target_gas, context),
			Action::CandidateBondMore => Self::candidate_bond_more(input, target_gas, context),
			Action::Nominate => Self::nominate(input, target_gas, context),
			Action::LeaveNominators => Self::leave_nominators(input, target_gas, context),
			Action::RevokeNomination => Self::revoke_nomination(input, target_gas, context),
			Action::NominatorBondLess => Self::nominator_bond_less(input, target_gas, context),
			Action::NominatorBondMore => Self::nominator_bond_more(input, target_gas, context),
		}
		// TODO: share for all dispatchables...
		// let outer_call: Runtime::Call = inner_call.into();
		// let info = outer_call.get_dispatch_info();

		// // Make sure enough gas
		// if let Some(gas_limit) = target_gas {
		// 	let required_gas = Runtime::GasWeightMapping::weight_to_gas(info.weight);
		// 	if required_gas > gas_limit {
		// 		return Err(ExitError::OutOfGas);
		// 	}
		// }
		// log::trace!(target: "staking-precompile", "Made it past gas check");

		// // Dispatch that call
		// let origin = Runtime::AddressMapping::into_account_id(context.caller);

		// log::trace!(target: "staking-precompile", "Gonna call with origin {:?}", origin);

		// match outer_call.dispatch(Some(origin).into()) {
		// 	Ok(post_info) => {
		// 		let gas_used = Runtime::GasWeightMapping::weight_to_gas(
		// 			post_info.actual_weight.unwrap_or(info.weight),
		// 		);
		// 		Ok(PrecompileOutput {
		// 			exit_status: ExitSucceed::Stopped,
		// 			cost: gas_used,
		// 			output: Default::default(),
		// 			logs: Default::default(),
		// 		})
		// 	}
		// 	Err(e) => {
		// 		let error_message = format!(
		// 			"Parachain staking call via EVM failed with dispatch error: {:?}",
		// 			e
		// 		);
		// 		Err(error(error_message))
		// 	}
		// }
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
			output: EvmDataWriter::new().write(min_nomination).build(),
			logs: vec![],
		})
	}

	// Storage Getters

	fn points(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let round = input.read::<u32>()?;

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let points: u32 = parachain_staking::Pallet::<Runtime>::points(round);

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(points).build(),
			logs: vec![],
		})
	}

	// Role Verifiers

	fn is_nominator(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let raw_address: H160 = input.read::<Address>()?.into();
		let address: Runtime::AccountId = raw_address.into();

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_nominator = parachain_staking::Pallet::<Runtime>::is_nominator(&address);

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(is_nominator).build(),
			logs: vec![],
		})
	}

	fn is_candidate(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let raw_address: H160 = input.read::<Address>()?.into();
		let address: Runtime::AccountId = raw_address.into();

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_candidate = parachain_staking::Pallet::<Runtime>::is_candidate(&address);

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(is_candidate).build(),
			logs: vec![],
		})
	}

	fn is_selected_candidate(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let raw_address: H160 = input.read::<Address>()?.into();
		let address: Runtime::AccountId = raw_address.into();

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_selected = parachain_staking::Pallet::<Runtime>::is_selected_candidate(&address);

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(is_selected).build(),
			logs: vec![],
		})
	}

	// Runtime Methods (setters)

	fn join_candidates(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(2)?;

		let raw_amount = input.read::<u128>()?;
		let amount: BalanceOf<Runtime> = raw_amount
			.try_into()
			.map_err(|_| error("balance type conversion failed"))?;
		let collator_candidate_count = input.read()?;
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
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let collator_candidate_count = input.read()?;

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
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let raw_amount = input.read::<u128>()?;
		let amount: BalanceOf<Runtime> = raw_amount
			.try_into()
			.map_err(|_| error("balance type conversion failed"))?;

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
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let raw_amount = input.read::<u128>()?;
		let amount: BalanceOf<Runtime> = raw_amount
			.try_into()
			.map_err(|_| error("balance type conversion failed"))?;

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
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(4)?;

		let raw_address: H160 = input.read::<Address>()?.into();
		let collator: Runtime::AccountId = raw_address.into();
		let raw_amount = input.read::<u128>()?;
		let amount: BalanceOf<Runtime> = raw_amount
			.try_into()
			.map_err(|_| error("balance type conversion failed"))?;
		let collator_nomination_count = input.read()?;
		let nominator_nomination_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::nominate(
			collator,
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
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let nomination_count = input.read()?;

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
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;

		let raw_address: H160 = input.read::<Address>()?.into();
		let collator: Runtime::AccountId = raw_address.into();

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::revoke_nomination(collator);

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
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(2)?;

		let raw_address: H160 = input.read::<Address>()?.into();
		let collator: Runtime::AccountId = raw_address.into();
		let raw_amount = input.read::<u128>()?;
		let amount: BalanceOf<Runtime> = raw_amount
			.try_into()
			.map_err(|_| error("balance type conversion failed"))?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::nominator_bond_more(collator, amount);

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
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(2)?;

		let raw_address: H160 = input.read::<Address>()?.into();
		let collator: Runtime::AccountId = raw_address.into();
		let raw_amount = input.read::<u128>()?;
		let amount: BalanceOf<Runtime> = raw_amount
			.try_into()
			.map_err(|_| error("balance type conversion failed"))?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::nominator_bond_less(collator, amount);

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
