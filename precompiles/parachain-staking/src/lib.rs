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

use fp_evm::{PrecompileHandle, PrecompileOutput};
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
	MinDelegation = "min_delegation()",
	Points = "points(uint256)",
	CandidateCount = "candidate_count()",
	Round = "round()",
	CandidateDelegationCount = "candidate_delegation_count(address)",
	DelegatorDelegationCount = "delegator_delegation_count(address)",
	SelectedCandidates = "selected_candidates()",
	IsDelegator = "is_delegator(address)",
	IsCandidate = "is_candidate(address)",
	IsSelectedCandidate = "is_selected_candidate(address)",
	DelegationRequestIsPending = "delegation_request_is_pending(address,address)",
	CandidateExitIsPending = "candidate_exit_is_pending(address)",
	CandidateRequestIsPending = "candidate_request_is_pending(address)",
	JoinCandidates = "join_candidates(uint256,uint256)",
	ScheduleLeaveCandidates = "schedule_leave_candidates(uint256)",
	ExecuteLeaveCandidates = "execute_leave_candidates(address,uint256)",
	CancelLeaveCandidates = "cancel_leave_candidates(uint256)",
	GoOffline = "go_offline()",
	GoOnline = "go_online()",
	ScheduleCandidateBondLess = "schedule_candidate_bond_less(uint256)",
	CandidateBondMore = "candidate_bond_more(uint256)",
	ExecuteCandidateBondLess = "execute_candidate_bond_less(address)",
	CancelCandidateBondLess = "cancel_candidate_bond_less()",
	Delegate = "delegate(address,uint256,uint256,uint256)",
	ScheduleLeaveDelegators = "schedule_leave_delegators()",
	ExecuteLeaveDelegators = "execute_leave_delegators(address,uint256)",
	CancelLeaveDelegators = "cancel_leave_delegators()",
	ScheduleRevokeDelegation = "schedule_revoke_delegation(address)",
	ScheduleDelegatorBondLess = "schedule_delegator_bond_less(address,uint256)",
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
			| Action::CandidateRequestIsPending => FunctionModifier::View,
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
			| Action::CancelDelegationRequest => FunctionModifier::NonPayable,
		})?;

		// Return early if storage getter; return (origin, call) if dispatchable
		let (origin, call) = match selector {
			Action::MinDelegation => return Self::min_delegation(handle),
			Action::Points => return Self::points(handle),
			Action::CandidateCount => return Self::candidate_count(handle),
			Action::Round => return Self::round(handle),
			Action::CandidateDelegationCount => return Self::candidate_delegation_count(handle),
			Action::DelegatorDelegationCount => return Self::delegator_delegation_count(handle),
			Action::SelectedCandidates => return Self::selected_candidates(handle),
			Action::IsDelegator => return Self::is_delegator(handle),
			Action::IsCandidate => return Self::is_candidate(handle),
			Action::IsSelectedCandidate => return Self::is_selected_candidate(handle),
			Action::DelegationRequestIsPending => {
				return Self::delegation_request_is_pending(handle)
			}
			Action::CandidateExitIsPending => return Self::candidate_exit_is_pending(handle),
			Action::CandidateRequestIsPending => return Self::candidate_request_is_pending(handle),
			// runtime methods (dispatchables)
			Action::JoinCandidates => Self::join_candidates(handle)?,
			Action::ScheduleLeaveCandidates => Self::schedule_leave_candidates(handle)?,
			Action::ExecuteLeaveCandidates => Self::execute_leave_candidates(handle)?,
			Action::CancelLeaveCandidates => Self::cancel_leave_candidates(handle)?,
			Action::GoOffline => Self::go_offline(handle)?,
			Action::GoOnline => Self::go_online(handle)?,
			Action::ScheduleCandidateBondLess => Self::schedule_candidate_bond_less(handle)?,
			Action::CandidateBondMore => Self::candidate_bond_more(handle)?,
			Action::ExecuteCandidateBondLess => Self::execute_candidate_bond_less(handle)?,
			Action::CancelCandidateBondLess => Self::cancel_candidate_bond_less(handle)?,
			Action::Delegate => Self::delegate(handle)?,
			Action::ScheduleLeaveDelegators => Self::schedule_leave_delegators(handle)?,
			Action::ExecuteLeaveDelegators => Self::execute_leave_delegators(handle)?,
			Action::CancelLeaveDelegators => Self::cancel_leave_delegators(handle)?,
			Action::ScheduleRevokeDelegation => Self::schedule_revoke_delegation(handle)?,
			Action::ScheduleDelegatorBondLess => Self::schedule_delegator_bond_less(handle)?,
			Action::DelegatorBondMore => Self::delegator_bond_more(handle)?,
			Action::ExecuteDelegationRequest => Self::execute_delegation_request(handle)?,
			Action::CancelDelegationRequest => Self::cancel_delegation_request(handle)?,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, origin, call)?;

		Ok(succeed([]))
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
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;
		let round = input.read::<u32>()?;

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
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;
		let address = input.read::<Address>()?.0;
		let address = Runtime::AddressMapping::into_account_id(address);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let result = if let Some(state) =
			<pallet_parachain_staking::Pallet<Runtime>>::candidate_info(&address)
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
				address
			);
			0u32
		};

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(result).build()))
	}

	fn delegator_delegation_count(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;
		let address = input.read::<Address>()?.0;
		let address = Runtime::AddressMapping::into_account_id(address);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let result = if let Some(state) =
			<pallet_parachain_staking::Pallet<Runtime>>::delegator_state(&address)
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
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;
		let address = input.read::<Address>()?.0;
		let address = Runtime::AddressMapping::into_account_id(address);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_delegator = pallet_parachain_staking::Pallet::<Runtime>::is_delegator(&address);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(is_delegator).build()))
	}

	fn is_candidate(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;
		let address = input.read::<Address>()?.0;
		let address = Runtime::AddressMapping::into_account_id(address);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_candidate = pallet_parachain_staking::Pallet::<Runtime>::is_candidate(&address);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(is_candidate).build()))
	}

	fn is_selected_candidate(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;
		let address = input.read::<Address>()?.0;
		let address = Runtime::AddressMapping::into_account_id(address);

		// Fetch info.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_selected =
			pallet_parachain_staking::Pallet::<Runtime>::is_selected_candidate(&address);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(is_selected).build()))
	}

	fn delegation_request_is_pending(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
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
		let pending = <pallet_parachain_staking::Pallet<Runtime>>::delegation_request_exists(
			&candidate, &delegator,
		);

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(pending).build()))
	}

	fn candidate_exit_is_pending(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;

		// Only argument is candidate
		let candidate = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

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
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;

		// Only argument is candidate
		let candidate = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

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

	fn join_candidates(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(2)?;
		let bond: BalanceOf<Runtime> = input.read()?;
		let candidate_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::join_candidates {
			bond,
			candidate_count,
		};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn schedule_leave_candidates(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;
		let candidate_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::schedule_leave_candidates {
			candidate_count,
		};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn execute_leave_candidates(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;
		let candidate = input.read::<Address>()?.0;
		let candidate = Runtime::AddressMapping::into_account_id(candidate);
		let candidate_delegation_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::execute_leave_candidates {
			candidate,
			candidate_delegation_count,
		};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn cancel_leave_candidates(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;
		let candidate_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_parachain_staking::Call::<Runtime>::cancel_leave_candidates { candidate_count };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn go_offline(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::go_offline {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn go_online(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::go_online {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn candidate_bond_more(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;
		let more: BalanceOf<Runtime> = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::candidate_bond_more { more };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn schedule_candidate_bond_less(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;
		let less: BalanceOf<Runtime> = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::schedule_candidate_bond_less { less };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn execute_candidate_bond_less(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;
		let candidate = input.read::<Address>()?.0;
		let candidate = Runtime::AddressMapping::into_account_id(candidate);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_parachain_staking::Call::<Runtime>::execute_candidate_bond_less { candidate };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn cancel_candidate_bond_less(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::cancel_candidate_bond_less {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn delegate(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(4)?;
		let candidate = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let amount: BalanceOf<Runtime> = input.read()?;
		let candidate_delegation_count = input.read()?;
		let delegation_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::delegate {
			candidate,
			amount,
			candidate_delegation_count,
			delegation_count,
		};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn schedule_leave_delegators(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::schedule_leave_delegators {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn execute_leave_delegators(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(2)?;
		let delegator = input.read::<Address>()?.0;
		let delegator = Runtime::AddressMapping::into_account_id(delegator);
		let delegation_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::execute_leave_delegators {
			delegator,
			delegation_count,
		};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn cancel_leave_delegators(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::cancel_leave_delegators {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn schedule_revoke_delegation(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;
		let collator = input.read::<Address>()?.0;
		let collator = Runtime::AddressMapping::into_account_id(collator);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_parachain_staking::Call::<Runtime>::schedule_revoke_delegation { collator };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn delegator_bond_more(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(2)?;
		let candidate = input.read::<Address>()?.0;
		let candidate = Runtime::AddressMapping::into_account_id(candidate);
		let more: BalanceOf<Runtime> = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_parachain_staking::Call::<Runtime>::delegator_bond_more { candidate, more };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn schedule_delegator_bond_less(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(2)?;
		let candidate = input.read::<Address>()?.0;
		let candidate = Runtime::AddressMapping::into_account_id(candidate);
		let less: BalanceOf<Runtime> = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::schedule_delegator_bond_less {
			candidate,
			less,
		};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn execute_delegation_request(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(2)?;
		let delegator = input.read::<Address>()?.0;
		let delegator = Runtime::AddressMapping::into_account_id(delegator);
		let candidate = input.read::<Address>()?.0;
		let candidate = Runtime::AddressMapping::into_account_id(candidate);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::execute_delegation_request {
			delegator,
			candidate,
		};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn cancel_delegation_request(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<(
		<Runtime::Call as Dispatchable>::Origin,
		pallet_parachain_staking::Call<Runtime>,
	)> {
		let mut input = EvmDataReader::new_skip_selector(handle.input())?;
		// Read input.
		input.expect_arguments(1)?;
		let candidate = input.read::<Address>()?.0;
		let candidate = Runtime::AddressMapping::into_account_id(candidate);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_parachain_staking::Call::<Runtime>::cancel_delegation_request { candidate };

		// Return call information
		Ok((Some(origin).into(), call))
	}
}
