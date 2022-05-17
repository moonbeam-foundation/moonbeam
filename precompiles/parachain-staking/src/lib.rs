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

//! Precompile to call parachain-staking runtime methods via the EVM

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(test, feature(assert_matches))]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use fp_evm::{Context, PrecompileHandle, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::{Currency, Get};
use pallet_evm::AddressMapping;
use precompile_utils::{
	check_function_modifier, revert, succeed, Address, EvmData, EvmDataReader, EvmDataWriter,
	EvmResult, FunctionModifier, RuntimeHelper,
};
use sp_core::H160;
use sp_std::{convert::TryInto, fmt::Debug, marker::PhantomData, vec::Vec};

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
	Round = "round()",
	// DEPRECATED
	CollatorNominationCount = "collator_nomination_count(address)",
	// DEPRECATED
	NominatorNominationCount = "nominator_nomination_count(address)",
	CandidateDelegationCount = "candidate_delegation_count(address)",
	DelegatorDelegationCount = "delegator_delegation_count(address)",
	SelectedCandidates = "selected_candidates()",
	// DEPRECATED
	IsNominator = "is_nominator(address)",
	IsDelegator = "is_delegator(address)",
	IsCandidate = "is_candidate(address)",
	IsSelectedCandidate = "is_selected_candidate(address)",
	DelegationRequestIsPending = "delegation_request_is_pending(address,address)",
	DelegatorExitIsPending = "delegator_exit_is_pending(address)",
	CandidateExitIsPending = "candidate_exit_is_pending(address)",
	CandidateRequestIsPending = "candidate_request_is_pending(address)",
	JoinCandidates = "join_candidates(uint256,uint256)",
	// DEPRECATED
	LeaveCandidates = "leave_candidates(uint256)",
	ScheduleLeaveCandidates = "schedule_leave_candidates(uint256)",
	ExecuteLeaveCandidates = "execute_leave_candidates(address,uint256)",
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

// TODO: Migrate to precompile_utils::Precompile.
impl<Runtime> pallet_evm::Precompile for ParachainStakingWrapper<Runtime>
where
	Runtime: parachain_staking::Config + pallet_evm::Config,
	BalanceOf<Runtime>: EvmData,
	Runtime::AccountId: Into<H160>,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<parachain_staking::Call<Runtime>>,
{
	fn execute(
		handle: &mut impl PrecompileHandle,
		input: &[u8],
		_target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> EvmResult<PrecompileOutput> {
		let (mut input, selector) = EvmDataReader::new_with_selector(input)?;
		let input = &mut input;

		check_function_modifier(
			context,
			is_static,
			match selector {
				Action::IsNominator
				| Action::IsDelegator
				| Action::IsCandidate
				| Action::IsSelectedCandidate
				| Action::Points
				| Action::MinNomination
				| Action::MinDelegation
				| Action::CandidateCount
				| Action::CollatorNominationCount
				| Action::CandidateDelegationCount
				| Action::NominatorNominationCount
				| Action::DelegatorDelegationCount
				| Action::SelectedCandidates => FunctionModifier::View,
				_ => FunctionModifier::NonPayable,
			},
		)?;

		// Return early if storage getter; return (origin, call) if dispatchable
		let (origin, call) = match selector {
			// DEPRECATED
			Action::MinNomination => return Self::min_delegation(handle),
			Action::MinDelegation => return Self::min_delegation(handle),
			Action::Points => return Self::points(handle, input),
			Action::CandidateCount => return Self::candidate_count(handle),
			Action::Round => return Self::round(handle),
			// DEPRECATED
			Action::CollatorNominationCount => {
				return Self::candidate_delegation_count(handle, input)
			}
			// DEPRECATED
			Action::NominatorNominationCount => {
				return Self::delegator_delegation_count(handle, input)
			}
			Action::CandidateDelegationCount => {
				return Self::candidate_delegation_count(handle, input)
			}
			Action::DelegatorDelegationCount => {
				return Self::delegator_delegation_count(handle, input)
			}
			Action::SelectedCandidates => return Self::selected_candidates(handle),
			// DEPRECATED
			Action::IsNominator => return Self::is_delegator(handle, input),
			Action::IsDelegator => return Self::is_delegator(handle, input),
			Action::IsCandidate => return Self::is_candidate(handle, input),
			Action::IsSelectedCandidate => return Self::is_selected_candidate(handle, input),
			Action::DelegationRequestIsPending => {
				return Self::delegation_request_is_pending(handle, input)
			}
			Action::DelegatorExitIsPending => {
				return Self::delegator_exit_is_pending(handle, input)
			}
			Action::CandidateExitIsPending => {
				return Self::candidate_exit_is_pending(handle, input)
			}
			Action::CandidateRequestIsPending => {
				return Self::candidate_request_is_pending(handle, input)
			}
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

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, origin, call)?;

		Ok(succeed([]))
	}
}

impl<Runtime> ParachainStakingWrapper<Runtime>
where
	Runtime: parachain_staking::Config + pallet_evm::Config,
	BalanceOf<Runtime>: EvmData,
	Runtime::AccountId: Into<H160>,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<parachain_staking::Call<Runtime>>,
{
	// Constants

	fn min_delegation(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let min_nomination: u128 = <<Runtime as parachain_staking::Config>::MinDelegation as Get<
			BalanceOf<Runtime>,
		>>::get()
		.try_into()
		.map_err(|_| revert("Amount is too large for provided balance type"))?;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(min_nomination).build()))
	}

	// Storage Getters

	fn points(
		handle: &mut impl PrecompileHandle,
		input: &mut EvmDataReader,
	) -> EvmResult<PrecompileOutput> {
		// Read input.
		input.expect_arguments(1)?;
		let round = input.read::<u32>()?;

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let points: u32 = parachain_staking::Pallet::<Runtime>::points(round);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(points).build()))
	}

	fn candidate_count(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let candidate_count: u32 = <parachain_staking::Pallet<Runtime>>::candidate_pool()
			.0
			.len() as u32;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(candidate_count).build()))
	}

	fn round(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let round: u32 = <parachain_staking::Pallet<Runtime>>::round().current;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(round).build()))
	}

	fn candidate_delegation_count(
		handle: &mut impl PrecompileHandle,
		input: &mut EvmDataReader,
	) -> EvmResult<PrecompileOutput> {
		// Read input.
		input.expect_arguments(1)?;
		let address = input.read::<Address>()?.0;
		let address = Runtime::AddressMapping::into_account_id(address);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let result =
			if let Some(state) = <parachain_staking::Pallet<Runtime>>::candidate_info(&address) {
				let candidate_delegation_count: u32 = state.delegation_count;

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
		Ok(succeed(EvmDataWriter::new().write(result).build()))
	}

	fn delegator_delegation_count(
		handle: &mut impl PrecompileHandle,
		input: &mut EvmDataReader,
	) -> EvmResult<PrecompileOutput> {
		// Read input.
		input.expect_arguments(1)?;
		let address = input.read::<Address>()?.0;
		let address = Runtime::AddressMapping::into_account_id(address);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
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
		Ok(succeed(EvmDataWriter::new().write(result).build()))
	}

	fn selected_candidates(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let selected_candidates: Vec<Address> =
			parachain_staking::Pallet::<Runtime>::selected_candidates()
				.into_iter()
				.map(|address| Address(address.into()))
				.collect();

		// Build output.
		Ok(succeed(
			EvmDataWriter::new().write(selected_candidates).build(),
		))
	}

	// Role Verifiers

	fn is_delegator(
		handle: &mut impl PrecompileHandle,
		input: &mut EvmDataReader,
	) -> EvmResult<PrecompileOutput> {
		// Read input.
		input.expect_arguments(1)?;
		let address = input.read::<Address>()?.0;
		let address = Runtime::AddressMapping::into_account_id(address);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_delegator = parachain_staking::Pallet::<Runtime>::is_delegator(&address);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(is_delegator).build()))
	}

	fn is_candidate(
		handle: &mut impl PrecompileHandle,
		input: &mut EvmDataReader,
	) -> EvmResult<PrecompileOutput> {
		// Read input.
		input.expect_arguments(1)?;
		let address = input.read::<Address>()?.0;
		let address = Runtime::AddressMapping::into_account_id(address);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_candidate = parachain_staking::Pallet::<Runtime>::is_candidate(&address);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(is_candidate).build()))
	}

	fn is_selected_candidate(
		handle: &mut impl PrecompileHandle,
		input: &mut EvmDataReader,
	) -> EvmResult<PrecompileOutput> {
		// Read input.
		input.expect_arguments(1)?;
		let address = input.read::<Address>()?.0;
		let address = Runtime::AddressMapping::into_account_id(address);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_selected = parachain_staking::Pallet::<Runtime>::is_selected_candidate(&address);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(is_selected).build()))
	}

	fn delegation_request_is_pending(
		handle: &mut impl PrecompileHandle,
		input: &mut EvmDataReader,
	) -> EvmResult<PrecompileOutput> {
		// Read input.
		input.expect_arguments(2)?;

		// First argument is delegator
		let delegator = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Second argument is candidate
		let candidate = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// If we are not able to get delegator state, we return false
		// Users can call `is_delegator` to determine when this happens
		let pending =
			<parachain_staking::Pallet<Runtime>>::delegation_request_exists(&candidate, &delegator);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(pending).build()))
	}

	fn delegator_exit_is_pending(
		handle: &mut impl PrecompileHandle,
		input: &mut EvmDataReader,
	) -> EvmResult<PrecompileOutput> {
		// Read input.
		input.expect_arguments(1)?;

		// Only argument is delegator
		let delegator = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// If we are not able to get delegator state, we return false
		// Users can call `is_delegator` to determine when this happens
		let pending = if let Some(state) =
			<parachain_staking::Pallet<Runtime>>::delegator_state(&delegator)
		{
			state.is_leaving()
		} else {
			log::trace!(
				target: "staking-precompile",
				"Delegator state for {:?} not found, so pending exit is false",
				delegator
			);
			false
		};

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(pending).build()))
	}

	fn candidate_exit_is_pending(
		handle: &mut impl PrecompileHandle,
		input: &mut EvmDataReader,
	) -> EvmResult<PrecompileOutput> {
		// Read input.
		input.expect_arguments(1)?;

		// Only argument is candidate
		let candidate = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// If we are not able to get delegator state, we return false
		// Users can call `is_candidate` to determine when this happens
		let pending =
			if let Some(state) = <parachain_staking::Pallet<Runtime>>::candidate_info(&candidate) {
				state.is_leaving()
			} else {
				log::trace!(
					target: "staking-precompile",
					"Candidate state for {:?} not found, so pending exit is false",
					candidate
				);
				false
			};

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(pending).build()))
	}

	fn candidate_request_is_pending(
		handle: &mut impl PrecompileHandle,
		input: &mut EvmDataReader,
	) -> EvmResult<PrecompileOutput> {
		// Read input.
		input.expect_arguments(1)?;

		// Only argument is candidate
		let candidate = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// If we are not able to get candidate metadata, we return false
		// Users can call `is_candidate` to determine when this happens
		let pending =
			if let Some(state) = <parachain_staking::Pallet<Runtime>>::candidate_info(&candidate) {
				state.request.is_some()
			} else {
				log::trace!(
					target: "staking-precompile",
					"Candidate metadata for {:?} not found, so pending request is false",
					candidate
				);
				false
			};

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(pending).build()))
	}

	// Runtime Methods (dispatchables)

	fn join_candidates(
		input: &mut EvmDataReader,
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
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
		input: &mut EvmDataReader,
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
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
		input: &mut EvmDataReader,
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
		// Read input.
		input.expect_arguments(1)?;
		let candidate = input.read::<Address>()?.0;
		let candidate = Runtime::AddressMapping::into_account_id(candidate);
		let candidate_delegation_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::execute_leave_candidates {
			candidate,
			candidate_delegation_count,
		};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn cancel_leave_candidates(
		input: &mut EvmDataReader,
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
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
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::go_offline {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn go_online(
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::go_online {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn candidate_bond_more(
		input: &mut EvmDataReader,
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
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
		input: &mut EvmDataReader,
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
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
		input: &mut EvmDataReader,
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
		// Read input.
		input.expect_arguments(1)?;
		let candidate = input.read::<Address>()?.0;
		let candidate = Runtime::AddressMapping::into_account_id(candidate);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::execute_candidate_bond_less { candidate };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn cancel_candidate_bond_less(
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::cancel_candidate_bond_less {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn delegate(
		input: &mut EvmDataReader,
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
		// Read input.
		input.expect_arguments(4)?;
		let candidate = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let amount: BalanceOf<Runtime> = input.read()?;
		let candidate_delegation_count = input.read()?;
		let delegation_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::delegate {
			candidate,
			amount,
			candidate_delegation_count,
			delegation_count,
		};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn schedule_leave_delegators(
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::schedule_leave_delegators {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn execute_leave_delegators(
		input: &mut EvmDataReader,
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
		// Read input.
		input.expect_arguments(2)?;
		let delegator = input.read::<Address>()?.0;
		let delegator = Runtime::AddressMapping::into_account_id(delegator);
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
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::cancel_leave_delegators {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn schedule_revoke_delegation(
		input: &mut EvmDataReader,
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
		// Read input.
		input.expect_arguments(1)?;
		let collator = input.read::<Address>()?.0;
		let collator = Runtime::AddressMapping::into_account_id(collator);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::schedule_revoke_delegation { collator };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn delegator_bond_more(
		input: &mut EvmDataReader,
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
		// Read input.
		input.expect_arguments(2)?;
		let candidate = input.read::<Address>()?.0;
		let candidate = Runtime::AddressMapping::into_account_id(candidate);
		let more: BalanceOf<Runtime> = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::delegator_bond_more { candidate, more };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn schedule_delegator_bond_less(
		input: &mut EvmDataReader,
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
		// Read input.
		input.expect_arguments(2)?;
		let candidate = input.read::<Address>()?.0;
		let candidate = Runtime::AddressMapping::into_account_id(candidate);
		let less: BalanceOf<Runtime> = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call =
			parachain_staking::Call::<Runtime>::schedule_delegator_bond_less { candidate, less };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn execute_delegation_request(
		input: &mut EvmDataReader,
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
		// Read input.
		input.expect_arguments(2)?;
		let delegator = input.read::<Address>()?.0;
		let delegator = Runtime::AddressMapping::into_account_id(delegator);
		let candidate = input.read::<Address>()?.0;
		let candidate = Runtime::AddressMapping::into_account_id(candidate);

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
		input: &mut EvmDataReader,
		context: &Context,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		parachain_staking::Call<Runtime>,
	)> {
		// Read input.
		input.expect_arguments(1)?;
		let candidate = input.read::<Address>()?.0;
		let candidate = Runtime::AddressMapping::into_account_id(candidate);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::cancel_delegation_request { candidate };

		// Return call information
		Ok((Some(origin).into(), call))
	}
}
