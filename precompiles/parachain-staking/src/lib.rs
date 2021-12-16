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

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use fp_evm::{Context, ExitError, ExitSucceed, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::{Currency, Get};
use pallet_evm::AddressMapping;
use pallet_evm::Precompile;
use precompile_utils::{
	error, Address, EvmData, EvmDataReader, EvmDataWriter, Gasometer, RuntimeHelper,
};
use sp_std::convert::TryInto;
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;
use sp_std::vec;

type BalanceOf<Runtime> = <<Runtime as parachain_staking::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	// DEPRECATED
	MinNomination = "min_nomination()",
	MinDelegation = "min_delegation()",
	Points = "points(uint256)",
	CandidateCount = "candidate_count()",
	// DEPRECATED
	CollatorNominationCount = "collator_nomination_count(address)",
	// DEPRECATED
	NominatorNominationCount = "nominator_nomination_count(address)",
	CandidateDelegationCount = "candidate_delegation_count(address)",
	DelegatorDelegationCount = "delegator_delegation_count(address)",
	// DEPRECATED
	IsNominator = "is_nominator(address)",
	IsDelegator = "is_delegator(address)",
	IsCandidate = "is_candidate(address)",
	IsSelectedCandidate = "is_selected_candidate(address)",
	JoinCandidates = "join_candidates(uint256,uint256)",
	// DEPRECATED
	LeaveCandidates = "leave_candidates(uint256)",
	ScheduleLeaveCandidates = "schedule_leave_candidates(uint256)",
	ExecuteLeaveCandidates = "execute_leave_candidates(address)",
	CancelLeaveCandidates = "cancel_leave_candidates(uint256)",
	GoOffline = "go_offline()",
	GoOnline = "go_online()",
	// DEPRECATED
	CandidateBondLess = "candidate_bond_less(uint256)",
	ScheduleCandidateBondLess = "schedule_candidate_bond_less(uint256)",
	CandidateBondMore = "candidate_bond_more(uint256)",
	ExecuteCandidateBondLess = "execute_candidate_bond_less(address)",
	CancelCandidateBondLess = "cancel_candidate_bond_less()",
	// DEPRECATED
	Nominate = "nominate(address,uint256,uint256,uint256)",
	Delegate = "delegate(address,uint256,uint256,uint256)",
	// DEPRECATED
	LeaveNominators = "leave_nominators(uint256)",
	ScheduleLeaveDelegators = "schedule_leave_delegators()",
	ExecuteLeaveDelegators = "execute_leave_delegators(address,uint256)",
	CancelLeaveDelegators = "cancel_leave_delegators()",
	// DEPRECATED
	RevokeNomination = "revoke_nomination(address)",
	ScheduleRevokeDelegation = "schedule_revoke_delegation(address)",
	// DEPRECATED
	NominatorBondLess = "nominator_bond_less(address,uint256)",
	ScheduleDelegatorBondLess = "schedule_delegator_bond_less(address,uint256)",
	// DEPRECATED
	NominatorBondMore = "nominator_bond_more(address,uint256)",
	DelegatorBondMore = "delegator_bond_more(address,uint256)",
	ExecuteDelegationRequest = "execute_delegation_request(address,address)",
	CancelDelegationRequest = "cancel_delegation_request(address)",
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
	BalanceOf<Runtime>: EvmData,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<parachain_staking::Call<Runtime>>,
{
	fn execute(
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let (input, selector) = EvmDataReader::new_with_selector(input)?;

		// Return early if storage getter; return (origin, call) if dispatchable
		let (origin, call) = match selector {
			// DEPRECATED
			Action::MinNomination => return Self::min_delegation(target_gas),
			Action::MinDelegation => return Self::min_delegation(target_gas),
			Action::Points => return Self::points(input, target_gas),
			Action::CandidateCount => return Self::candidate_count(target_gas),
			// DEPRECATED
			Action::CollatorNominationCount => {
				return Self::candidate_delegation_count(input, target_gas)
			}
			// DEPRECATED
			Action::NominatorNominationCount => {
				return Self::delegator_delegation_count(input, target_gas)
			}
			Action::CandidateDelegationCount => {
				return Self::candidate_delegation_count(input, target_gas)
			}
			Action::DelegatorDelegationCount => {
				return Self::delegator_delegation_count(input, target_gas)
			}
			// DEPRECATED
			Action::IsNominator => return Self::is_delegator(input, target_gas),
			Action::IsDelegator => return Self::is_delegator(input, target_gas),
			Action::IsCandidate => return Self::is_candidate(input, target_gas),
			Action::IsSelectedCandidate => return Self::is_selected_candidate(input, target_gas),
			// runtime methods (dispatchables)
			Action::JoinCandidates => Self::join_candidates(input, context)?,
			// DEPRECATED
			Action::LeaveCandidates => Self::schedule_leave_candidates(input, context)?,
			Action::ScheduleLeaveCandidates => Self::schedule_leave_candidates(input, context)?,
			Action::ExecuteLeaveCandidates => Self::execute_leave_candidates(input, context)?,
			Action::CancelLeaveCandidates => Self::cancel_leave_candidates(input, context)?,
			Action::GoOffline => Self::go_offline(context)?,
			Action::GoOnline => Self::go_online(context)?,
			// DEPRECATED
			Action::CandidateBondLess => Self::schedule_candidate_bond_less(input, context)?,
			Action::ScheduleCandidateBondLess => {
				Self::schedule_candidate_bond_less(input, context)?
			}
			Action::CandidateBondMore => Self::candidate_bond_more(input, context)?,
			Action::ExecuteCandidateBondLess => Self::execute_candidate_bond_less(input, context)?,
			Action::CancelCandidateBondLess => Self::cancel_candidate_bond_less(context)?,
			// DEPRECATED
			Action::Nominate => Self::delegate(input, context)?,
			Action::Delegate => Self::delegate(input, context)?,
			// DEPRECATED
			Action::LeaveNominators => Self::schedule_leave_delegators(context)?,
			Action::ScheduleLeaveDelegators => Self::schedule_leave_delegators(context)?,
			Action::ExecuteLeaveDelegators => Self::execute_leave_delegators(input, context)?,
			Action::CancelLeaveDelegators => Self::cancel_leave_delegators(context)?,
			// DEPRECATED
			Action::RevokeNomination => Self::schedule_revoke_delegation(input, context)?,
			Action::ScheduleRevokeDelegation => Self::schedule_revoke_delegation(input, context)?,
			// DEPRECATED
			Action::NominatorBondLess => Self::schedule_delegator_bond_less(input, context)?,
			Action::ScheduleDelegatorBondLess => {
				Self::schedule_delegator_bond_less(input, context)?
			}
			// DEPRECATED
			Action::NominatorBondMore => Self::delegator_bond_more(input, context)?,
			Action::DelegatorBondMore => Self::delegator_bond_more(input, context)?,
			Action::ExecuteDelegationRequest => Self::execute_delegation_request(input, context)?,
			Action::CancelDelegationRequest => Self::cancel_delegation_request(input, context)?,
		};
		// Initialize gasometer
		let mut gasometer = Gasometer::new(target_gas);
		// Dispatch call (if enough gas).
		let used_gas =
			RuntimeHelper::<Runtime>::try_dispatch(origin, call, gasometer.remaining_gas()?)?;
		gasometer.record_cost(used_gas)?;
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
			logs: vec![],
		})
	}
}

impl<Runtime> ParachainStakingWrapper<Runtime>
where
	Runtime: parachain_staking::Config + pallet_evm::Config,
	BalanceOf<Runtime>: EvmData,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<parachain_staking::Call<Runtime>>,
{
	// Constants

	fn min_delegation(target_gas: Option<u64>) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let min_nomination: u128 = <<Runtime as parachain_staking::Config>::MinDelegation as Get<
			BalanceOf<Runtime>,
		>>::get()
		.try_into()
		.map_err(|_| error("Amount is too large for provided balance type"))?;

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
			output: EvmDataWriter::new().write(candidate_count).build(),
			logs: vec![],
		})
	}

	fn candidate_delegation_count(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;
		let address = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let result =
			if let Some(state) = <parachain_staking::Pallet<Runtime>>::candidate_state(&address) {
				let candidate_delegation_count: u32 = state.delegators.0.len() as u32;

				log::trace!(
					target: "staking-precompile",
					"Result from pallet is {:?}",
					candidate_delegation_count
				);
				candidate_delegation_count
			} else {
				log::trace!(
					target: "staking-precompile",
					"Candidate {:?} not found, so delegation count is 0",
					address
				);
				0u32
			};

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(result).build(),
			logs: vec![],
		})
	}

	fn delegator_delegation_count(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;
		let address = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let result =
			if let Some(state) = <parachain_staking::Pallet<Runtime>>::delegator_state(&address) {
				let delegator_delegation_count: u32 = state.delegations.0.len() as u32;

				log::trace!(
					target: "staking-precompile",
					"Result from pallet is {:?}",
					delegator_delegation_count
				);

				delegator_delegation_count
			} else {
				log::trace!(
					target: "staking-precompile",
					"Delegator {:?} not found, so delegation count is 0",
					address
				);
				0u32
			};

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(result).build(),
			logs: vec![],
		})
	}

	// Role Verifiers

	fn is_delegator(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;
		let address = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_delegator = parachain_staking::Pallet::<Runtime>::is_delegator(&address);

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(is_delegator).build(),
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
		let address = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

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
		let address = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

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

	// Runtime Methods (dispatchables)

	fn join_candidates(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(2)?;
		let bond: BalanceOf<Runtime> = input.read()?;
		let candidate_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::join_candidates {
			bond,
			candidate_count,
		};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn schedule_leave_candidates(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(1)?;
		let candidate_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call =
			parachain_staking::Call::<Runtime>::schedule_leave_candidates { candidate_count };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn execute_leave_candidates(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(1)?;
		let candidate = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::execute_leave_candidates { candidate };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn cancel_leave_candidates(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(1)?;
		let candidate_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::cancel_leave_candidates { candidate_count };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn go_offline(
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::go_offline {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn go_online(
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::go_online {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn candidate_bond_more(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(1)?;
		let more: BalanceOf<Runtime> = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::candidate_bond_more { more };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn schedule_candidate_bond_less(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(1)?;
		let less: BalanceOf<Runtime> = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::schedule_candidate_bond_less { less };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn execute_candidate_bond_less(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(1)?;
		let candidate = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::execute_candidate_bond_less { candidate };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn cancel_candidate_bond_less(
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::cancel_candidate_bond_less {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn delegate(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(4)?;
		let collator = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let amount: BalanceOf<Runtime> = input.read()?;
		let candidate_delegation_count = input.read()?;
		let delegation_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::delegate {
			collator,
			amount,
			candidate_delegation_count,
			delegation_count,
		};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn schedule_leave_delegators(
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::schedule_leave_delegators {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn execute_leave_delegators(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(2)?;
		let delegator = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let delegation_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::execute_leave_delegators {
			delegator,
			delegation_count,
		};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn cancel_leave_delegators(
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::cancel_leave_delegators {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn schedule_revoke_delegation(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(1)?;
		let collator = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::schedule_revoke_delegation { collator };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn delegator_bond_more(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(2)?;
		let candidate = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let more: BalanceOf<Runtime> = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::delegator_bond_more { candidate, more };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn schedule_delegator_bond_less(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(2)?;
		let candidate = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let less: BalanceOf<Runtime> = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call =
			parachain_staking::Call::<Runtime>::schedule_delegator_bond_less { candidate, less };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn execute_delegation_request(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(2)?;
		let delegator = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let candidate = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::execute_delegation_request {
			delegator,
			candidate,
		};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn cancel_delegation_request(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(1)?;
		let candidate = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::cancel_delegation_request { candidate };

		// Return call information
		Ok((Some(origin).into(), call))
	}
}
