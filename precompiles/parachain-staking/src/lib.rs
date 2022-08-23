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

use fp_evm::{Precompile, PrecompileHandle, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::{Currency, Get};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::H160;
use sp_std::{convert::TryInto, fmt::Debug, marker::PhantomData, vec::Vec};

type BalanceOf<Runtime> = <<Runtime as pallet_parachain_staking::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	MinDelegation = "minDelegation()",
	Points = "points(uint256)",
	CandidateCount = "candidateCount()",
	Round = "round()",
	CandidateDelegationCount = "candidateDelegationCount(address)",
	DelegatorDelegationCount = "delegatorDelegationCount(address)",
	SelectedCandidates = "selectedCandidates()",
	IsDelegator = "isDelegator(address)",
	IsCandidate = "isCandidate(address)",
	IsSelectedCandidate = "isSelectedCandidate(address)",
	DelegationRequestIsPending = "delegationRequestIsPending(address,address)",
	CandidateExitIsPending = "candidateExitIsPending(address)",
	CandidateRequestIsPending = "candidateRequestIsPending(address)",
	JoinCandidates = "joinCandidates(uint256,uint256)",
	ScheduleLeaveCandidates = "scheduleLeaveCandidates(uint256)",
	ExecuteLeaveCandidates = "executeLeaveCandidates(address,uint256)",
	CancelLeaveCandidates = "cancelLeaveCandidates(uint256)",
	GoOffline = "goOffline()",
	GoOnline = "goOnline()",
	ScheduleCandidateBondLess = "scheduleCandidateBondLess(uint256)",
	CandidateBondMore = "candidateBondMore(uint256)",
	ExecuteCandidateBondLess = "executeCandidateBondLess(address)",
	CancelCandidateBondLess = "cancelCandidateBondLess()",
	Delegate = "delegate(address,uint256,uint256,uint256)",
	ScheduleRevokeDelegation = "scheduleRevokeDelegation(address)",
	ScheduleDelegatorBondLess = "scheduleDelegatorBondLess(address,uint256)",
	DelegatorBondMore = "delegatorBondMore(address,uint256)",
	ExecuteDelegationRequest = "executeDelegationRequest(address,address)",
	CancelDelegationRequest = "cancelDelegationRequest(address)",

	// deprecated in favor of batch util
	ScheduleLeaveDelegators = "scheduleLeaveDelegators()",
	ExecuteLeaveDelegators = "executeLeaveDelegators(address,uint256)",
	CancelLeaveDelegators = "cancelLeaveDelegators()",

	// deprecated
	DeprecatedMinDelegation = "min_delegation()",
	DeprecatedCandidateCount = "candidate_count()",
	DeprecatedCandidateDelegationCount = "candidate_delegation_count(address)",
	DeprecatedDelegatorDelegationCount = "delegator_delegation_count(address)",
	DeprecatedSelectedCandidates = "selected_candidates()",
	DeprecatedIsDelegator = "is_delegator(address)",
	DeprecatedIsCandidate = "is_candidate(address)",
	DeprecatedIsSelectedCandidate = "is_selected_candidate(address)",
	DeprecatedDelegationRequestIsPending = "delegation_request_is_pending(address,address)",
	DeprecatedCandidateExitIsPending = "candidate_exit_is_pending(address)",
	DeprecatedCandidateRequestIsPending = "candidate_request_is_pending(address)",
	DeprecatedJoinCandidates = "join_candidates(uint256,uint256)",
	DeprecatedScheduleLeaveCandidates = "schedule_leave_candidates(uint256)",
	DeprecatedExecuteLeaveCandidates = "execute_leave_candidates(address,uint256)",
	DeprecatedCancelLeaveCandidates = "cancel_leave_candidates(uint256)",
	DeprecatedGoOffline = "go_offline()",
	DeprecatedGoOnline = "go_online()",
	DeprecatedScheduleCandidateBondLess = "schedule_candidate_bond_less(uint256)",
	DeprecatedCandidateBondMore = "candidate_bond_more(uint256)",
	DeprecatedExecuteCandidateBondLess = "execute_candidate_bond_less(address)",
	DeprecatedCancelCandidateBondLess = "cancel_candidate_bond_less()",
	DeprecatedScheduleLeaveDelegators = "schedule_leave_delegators()",
	DeprecatedExecuteLeaveDelegators = "execute_leave_delegators(address,uint256)",
	DeprecatedCancelLeaveDelegators = "cancel_leave_delegators()",
	DeprecatedScheduleRevokeDelegation = "schedule_revoke_delegation(address)",
	DeprecatedScheduleDelegatorBondLess = "schedule_delegator_bond_less(address,uint256)",
	DeprecatedDelegatorBondMore = "delegator_bond_more(address,uint256)",
	DeprecatedExecuteDelegationRequest = "execute_delegation_request(address,address)",
	DeprecatedCancelDelegationRequest = "cancel_delegation_request(address)",
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
	Runtime: pallet_parachain_staking::Config + pallet_evm::Config,
	BalanceOf<Runtime>: EvmData,
	Runtime::AccountId: Into<H160>,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_parachain_staking::Call<Runtime>>,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let selector = handle.read_selector()?;

		handle.check_function_modifier(match selector {
			// Views
			Action::IsDelegator
			| Action::IsCandidate
			| Action::IsSelectedCandidate
			| Action::MinDelegation
			| Action::Points
			| Action::CandidateCount
			| Action::Round
			| Action::CandidateDelegationCount
			| Action::DelegatorDelegationCount
			| Action::SelectedCandidates
			| Action::DelegationRequestIsPending
			| Action::CandidateExitIsPending
			| Action::CandidateRequestIsPending
			| Action::DeprecatedIsDelegator
			| Action::DeprecatedIsCandidate
			| Action::DeprecatedIsSelectedCandidate
			| Action::DeprecatedMinDelegation
			| Action::DeprecatedCandidateCount
			| Action::DeprecatedCandidateDelegationCount
			| Action::DeprecatedDelegatorDelegationCount
			| Action::DeprecatedSelectedCandidates
			| Action::DeprecatedDelegationRequestIsPending
			| Action::DeprecatedCandidateExitIsPending
			| Action::DeprecatedCandidateRequestIsPending => FunctionModifier::View,
			// Non-payables
			Action::JoinCandidates
			| Action::ScheduleLeaveCandidates
			| Action::ExecuteLeaveCandidates
			| Action::CancelLeaveCandidates
			| Action::GoOffline
			| Action::GoOnline
			| Action::ScheduleCandidateBondLess
			| Action::CandidateBondMore
			| Action::ExecuteCandidateBondLess
			| Action::CancelCandidateBondLess
			| Action::Delegate
			| Action::ScheduleLeaveDelegators
			| Action::ExecuteLeaveDelegators
			| Action::CancelLeaveDelegators
			| Action::ScheduleRevokeDelegation
			| Action::ScheduleDelegatorBondLess
			| Action::DelegatorBondMore
			| Action::ExecuteDelegationRequest
			| Action::CancelDelegationRequest
			| Action::DeprecatedJoinCandidates
			| Action::DeprecatedScheduleLeaveCandidates
			| Action::DeprecatedExecuteLeaveCandidates
			| Action::DeprecatedCancelLeaveCandidates
			| Action::DeprecatedGoOffline
			| Action::DeprecatedGoOnline
			| Action::DeprecatedScheduleCandidateBondLess
			| Action::DeprecatedCandidateBondMore
			| Action::DeprecatedExecuteCandidateBondLess
			| Action::DeprecatedCancelCandidateBondLess
			| Action::DeprecatedScheduleLeaveDelegators
			| Action::DeprecatedExecuteLeaveDelegators
			| Action::DeprecatedCancelLeaveDelegators
			| Action::DeprecatedScheduleRevokeDelegation
			| Action::DeprecatedScheduleDelegatorBondLess
			| Action::DeprecatedDelegatorBondMore
			| Action::DeprecatedExecuteDelegationRequest
			| Action::DeprecatedCancelDelegationRequest => FunctionModifier::NonPayable,
		})?;

		match selector {
			Action::MinDelegation | Action::DeprecatedMinDelegation => Self::min_delegation(handle),
			Action::Points => return Self::points(handle),
			Action::CandidateCount | Action::DeprecatedCandidateCount => {
				Self::candidate_count(handle)
			}
			Action::Round => return Self::round(handle),
			Action::CandidateDelegationCount | Action::DeprecatedCandidateDelegationCount => {
				Self::candidate_delegation_count(handle)
			}
			Action::DelegatorDelegationCount | Action::DeprecatedDelegatorDelegationCount => {
				Self::delegator_delegation_count(handle)
			}
			Action::SelectedCandidates | Action::DeprecatedSelectedCandidates => {
				Self::selected_candidates(handle)
			}
			Action::IsDelegator | Action::DeprecatedIsDelegator => Self::is_delegator(handle),
			Action::IsCandidate | Action::DeprecatedIsCandidate => Self::is_candidate(handle),
			Action::IsSelectedCandidate | Action::DeprecatedIsSelectedCandidate => {
				Self::is_selected_candidate(handle)
			}
			Action::DelegationRequestIsPending | Action::DeprecatedDelegationRequestIsPending => {
				Self::delegation_request_is_pending(handle)
			}
			Action::CandidateExitIsPending | Action::DeprecatedCandidateExitIsPending => {
				Self::candidate_exit_is_pending(handle)
			}
			Action::CandidateRequestIsPending | Action::DeprecatedCandidateRequestIsPending => {
				Self::candidate_request_is_pending(handle)
			}
			// runtime methods (dispatchables)
			Action::JoinCandidates | Action::DeprecatedJoinCandidates => {
				Self::join_candidates(handle)
			}
			Action::ScheduleLeaveCandidates | Action::DeprecatedScheduleLeaveCandidates => {
				Self::schedule_leave_candidates(handle)
			}
			Action::ExecuteLeaveCandidates | Action::DeprecatedExecuteLeaveCandidates => {
				Self::execute_leave_candidates(handle)
			}
			Action::CancelLeaveCandidates | Action::DeprecatedCancelLeaveCandidates => {
				Self::cancel_leave_candidates(handle)
			}
			Action::GoOffline | Action::DeprecatedGoOffline => Self::go_offline(handle),
			Action::GoOnline | Action::DeprecatedGoOnline => Self::go_online(handle),
			Action::ScheduleCandidateBondLess | Action::DeprecatedScheduleCandidateBondLess => {
				Self::schedule_candidate_bond_less(handle)
			}
			Action::CandidateBondMore | Action::DeprecatedCandidateBondMore => {
				Self::candidate_bond_more(handle)
			}
			Action::ExecuteCandidateBondLess | Action::DeprecatedExecuteCandidateBondLess => {
				Self::execute_candidate_bond_less(handle)
			}
			Action::CancelCandidateBondLess | Action::DeprecatedCancelCandidateBondLess => {
				Self::cancel_candidate_bond_less(handle)
			}
			Action::Delegate => Self::delegate(handle),
			Action::ScheduleLeaveDelegators | Action::DeprecatedScheduleLeaveDelegators => {
				Self::schedule_leave_delegators(handle)
			}
			Action::ExecuteLeaveDelegators | Action::DeprecatedExecuteLeaveDelegators => {
				Self::execute_leave_delegators(handle)
			}
			Action::CancelLeaveDelegators | Action::DeprecatedCancelLeaveDelegators => {
				Self::cancel_leave_delegators(handle)
			}
			Action::ScheduleRevokeDelegation | Action::DeprecatedScheduleRevokeDelegation => {
				Self::schedule_revoke_delegation(handle)
			}
			Action::ScheduleDelegatorBondLess | Action::DeprecatedScheduleDelegatorBondLess => {
				Self::schedule_delegator_bond_less(handle)
			}
			Action::DelegatorBondMore | Action::DeprecatedDelegatorBondMore => {
				Self::delegator_bond_more(handle)
			}
			Action::ExecuteDelegationRequest | Action::DeprecatedExecuteDelegationRequest => {
				Self::execute_delegation_request(handle)
			}
			Action::CancelDelegationRequest | Action::DeprecatedCancelDelegationRequest => {
				Self::cancel_delegation_request(handle)
			}
		}
	}
}

impl<Runtime> ParachainStakingWrapper<Runtime>
where
	Runtime: pallet_parachain_staking::Config + pallet_evm::Config,
	BalanceOf<Runtime>: EvmData,
	Runtime::AccountId: Into<H160>,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_parachain_staking::Call<Runtime>>,
{
	// Constants

	fn min_delegation(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let min_nomination: u128 =
			<<Runtime as pallet_parachain_staking::Config>::MinDelegation as Get<
				BalanceOf<Runtime>,
			>>::get()
			.try_into()
			.map_err(|_| revert("Amount is too large for provided balance type"))?;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(min_nomination).build()))
	}

	// Storage Getters

	fn points(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { round: u32 });

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let points: u32 = pallet_parachain_staking::Pallet::<Runtime>::points(round);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(points).build()))
	}

	fn candidate_count(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let candidate_count: u32 = <pallet_parachain_staking::Pallet<Runtime>>::candidate_pool()
			.0
			.len() as u32;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(candidate_count).build()))
	}

	fn round(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let round: u32 = <pallet_parachain_staking::Pallet<Runtime>>::round().current;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(round).build()))
	}

	fn candidate_delegation_count(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { candidate: Address });
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let result = if let Some(state) =
			<pallet_parachain_staking::Pallet<Runtime>>::candidate_info(&candidate)
		{
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
				candidate
			);
			0u32
		};

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(result).build()))
	}

	fn delegator_delegation_count(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { delegator: Address });
		let delegator = Runtime::AddressMapping::into_account_id(delegator.0);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let result = if let Some(state) =
			<pallet_parachain_staking::Pallet<Runtime>>::delegator_state(&delegator)
		{
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
				delegator
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
			pallet_parachain_staking::Pallet::<Runtime>::selected_candidates()
				.into_iter()
				.map(|address| Address(address.into()))
				.collect();

		// Build output.
		Ok(succeed(
			EvmDataWriter::new().write(selected_candidates).build(),
		))
	}

	// Role Verifiers

	fn is_delegator(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { delegator: Address });
		let delegator = Runtime::AddressMapping::into_account_id(delegator.0);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_delegator = pallet_parachain_staking::Pallet::<Runtime>::is_delegator(&delegator);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(is_delegator).build()))
	}

	fn is_candidate(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { candidate: Address });
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_candidate = pallet_parachain_staking::Pallet::<Runtime>::is_candidate(&candidate);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(is_candidate).build()))
	}

	fn is_selected_candidate(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { candidate: Address });
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_selected =
			pallet_parachain_staking::Pallet::<Runtime>::is_selected_candidate(&candidate);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(is_selected).build()))
	}

	fn delegation_request_is_pending(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {delegator: Address, candidate: Address});
		let delegator = Runtime::AddressMapping::into_account_id(delegator.0);
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// If we are not able to get delegator state, we return false
		// Users can call `is_delegator` to determine when this happens
		let pending = <pallet_parachain_staking::Pallet<Runtime>>::delegation_request_exists(
			&candidate, &delegator,
		);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(pending).build()))
	}

	fn candidate_exit_is_pending(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { candidate: Address });
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// If we are not able to get delegator state, we return false
		// Users can call `is_candidate` to determine when this happens
		let pending = if let Some(state) =
			<pallet_parachain_staking::Pallet<Runtime>>::candidate_info(&candidate)
		{
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
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { candidate: Address });
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// If we are not able to get candidate metadata, we return false
		// Users can call `is_candidate` to determine when this happens
		let pending = if let Some(state) =
			<pallet_parachain_staking::Pallet<Runtime>>::candidate_info(&candidate)
		{
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

	fn join_candidates(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {amount: BalanceOf<Runtime>, candidate_count: u32});
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::join_candidates {
			bond: amount,
			candidate_count,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn schedule_leave_candidates(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { candidate_count: u32 });

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::schedule_leave_candidates {
			candidate_count,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn execute_leave_candidates(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {candidate: Address, candidate_count: u32});
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::execute_leave_candidates {
			candidate,
			candidate_delegation_count: candidate_count,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn cancel_leave_candidates(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { candidate_count: u32 });

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_parachain_staking::Call::<Runtime>::cancel_leave_candidates { candidate_count };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn go_offline(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::go_offline {};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn go_online(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::go_online {};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn candidate_bond_more(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {more: BalanceOf<Runtime>});

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::candidate_bond_more { more };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn schedule_candidate_bond_less(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {less: BalanceOf<Runtime>});

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::schedule_candidate_bond_less { less };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn execute_candidate_bond_less(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { candidate: Address });
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_parachain_staking::Call::<Runtime>::execute_candidate_bond_less { candidate };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn cancel_candidate_bond_less(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::cancel_candidate_bond_less {};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn delegate(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			candidate: Address,
			amount: BalanceOf<Runtime>,
			candidate_delegation_count: u32,
			delegator_delegation_count: u32
		});
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::delegate {
			candidate,
			amount,
			candidate_delegation_count,
			delegation_count: delegator_delegation_count,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn schedule_leave_delegators(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::schedule_leave_delegators {};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn execute_leave_delegators(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {delegator: Address, delegator_delegation_count: u32});
		let delegator = Runtime::AddressMapping::into_account_id(delegator.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::execute_leave_delegators {
			delegator,
			delegation_count: delegator_delegation_count,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn cancel_leave_delegators(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::cancel_leave_delegators {};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn schedule_revoke_delegation(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { candidate: Address });
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::schedule_revoke_delegation {
			collator: candidate,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn delegator_bond_more(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {candidate: Address, more: BalanceOf<Runtime>});
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_parachain_staking::Call::<Runtime>::delegator_bond_more { candidate, more };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn schedule_delegator_bond_less(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {candidate: Address, less: BalanceOf<Runtime>});
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::schedule_delegator_bond_less {
			candidate,
			less,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn execute_delegation_request(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {delegator: Address, candidate: Address});
		let delegator = Runtime::AddressMapping::into_account_id(delegator.0);
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::execute_delegation_request {
			delegator,
			candidate,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn cancel_delegation_request(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { candidate: Address });
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_parachain_staking::Call::<Runtime>::cancel_delegation_request { candidate };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}
}
