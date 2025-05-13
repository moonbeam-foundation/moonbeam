// Copyright 2019-2025 PureStake Inc.
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

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_support::sp_runtime::Percent;
use frame_support::traits::{Currency, Get};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{H160, U256};
use sp_runtime::traits::Dispatchable;
use sp_std::{convert::TryInto, marker::PhantomData, vec::Vec};

type BalanceOf<Runtime> = <<Runtime as pallet_parachain_staking::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

/// A precompile to wrap the functionality from parachain_staking.
///
/// EXAMPLE USECASE:
/// A simple example usecase is a contract that allows donors to donate, and stakes all the funds
/// toward one fixed address chosen by the deployer.
/// Such a contract could be deployed by a collator candidate, and the deploy address distributed to
/// supporters who want to donate toward a perpetual nomination fund.
pub struct ParachainStakingPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> ParachainStakingPrecompile<Runtime>
where
	Runtime: pallet_parachain_staking::Config + pallet_evm::Config,
	Runtime::AccountId: Into<H160>,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_parachain_staking::Call<Runtime>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
	<Runtime as pallet_evm::Config>::AddressMapping: AddressMapping<Runtime::AccountId>,
{
	// Constants
	#[precompile::public("minDelegation()")]
	#[precompile::public("min_delegation()")]
	#[precompile::view]
	fn min_delegation(_handle: &mut impl PrecompileHandle) -> EvmResult<u128> {
		let min_nomination: u128 =
			<<Runtime as pallet_parachain_staking::Config>::MinDelegation as Get<
				BalanceOf<Runtime>,
			>>::get()
			.try_into()
			.map_err(|_| revert("Amount is too large for provided balance type"))?;

		Ok(min_nomination)
	}

	// Storage Getters
	#[precompile::public("points(uint256)")]
	#[precompile::view]
	fn points(handle: &mut impl PrecompileHandle, round: Convert<U256, u32>) -> EvmResult<u32> {
		let round = round.converted();
		// AccountsPayable: Twox64Concat(8) + RoundIndex(4) + RewardPoint(4)
		handle.record_db_read::<Runtime>(16)?;
		let points: u32 = pallet_parachain_staking::Pallet::<Runtime>::points(round);

		Ok(points)
	}

	#[precompile::public("awardedPoints(uint32,address)")]
	#[precompile::view]
	fn awarded_points(
		handle: &mut impl PrecompileHandle,
		round: u32,
		candidate: Address,
	) -> EvmResult<u32> {
		// AccountsPayable: Twox64Concat(8) + RoundIndex(4) + Twox64Concat(8) + AccountId(20)
		// + RewardPoint(4)
		handle.record_db_read::<Runtime>(44)?;

		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		let points = <pallet_parachain_staking::Pallet<Runtime>>::awarded_pts(&round, &candidate);

		Ok(points)
	}

	#[precompile::public("candidateCount()")]
	#[precompile::public("candidate_count()")]
	#[precompile::view]
	fn candidate_count(handle: &mut impl PrecompileHandle) -> EvmResult<u32> {
		// CandidatePool: UnBoundedVec(AccountId(20) + Balance(16))
		// TODO CandidatePool is unbounded, we account for a theoretical 200 pool.
		handle.record_db_read::<Runtime>(7200)?;
		// Fetch info.
		let candidate_count: u32 = <pallet_parachain_staking::Pallet<Runtime>>::candidate_pool()
			.0
			.len() as u32;

		// Build output.
		Ok(candidate_count)
	}

	#[precompile::public("round()")]
	#[precompile::view]
	fn round(handle: &mut impl PrecompileHandle) -> EvmResult<u32> {
		// Round: RoundInfo(RoundIndex(4) + BlockNumber(4) + 4)
		handle.record_db_read::<Runtime>(12)?;
		let round: u32 = <pallet_parachain_staking::Pallet<Runtime>>::round().current;

		Ok(round)
	}

	#[precompile::public("candidateDelegationCount(address)")]
	#[precompile::public("candidate_delegation_count(address)")]
	#[precompile::view]
	fn candidate_delegation_count(
		handle: &mut impl PrecompileHandle,
		candidate: Address,
	) -> EvmResult<u32> {
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);
		// CandidateInfo: Twox64Concat(8) + AccountId(20) + CandidateMetadata(105)
		handle.record_db_read::<Runtime>(133)?;
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

		Ok(result)
	}

	#[precompile::public("candidateAutoCompoundingDelegationCount(address)")]
	#[precompile::view]
	fn candidate_auto_compounding_delegation_count(
		handle: &mut impl PrecompileHandle,
		candidate: Address,
	) -> EvmResult<u32> {
		// AutoCompoundingDelegations:
		// Blake2128(16) + AccountId(20)
		// + BoundedVec(
		// 	AutoCompoundConfig * (MaxTopDelegationsPerCandidate + MaxBottomDelegationsPerCandidate)
		// )
		handle.record_db_read::<Runtime>(
			36 + (
				22 * (<Runtime as pallet_parachain_staking::Config>::MaxTopDelegationsPerCandidate::get()
				+ <Runtime as pallet_parachain_staking::Config>::MaxBottomDelegationsPerCandidate::get())
				as usize),
		)?;

		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		let count =
			<pallet_parachain_staking::Pallet<Runtime>>::auto_compounding_delegations(&candidate)
				.len() as u32;

		Ok(count)
	}

	#[precompile::public("delegatorDelegationCount(address)")]
	#[precompile::public("delegator_delegation_count(address)")]
	#[precompile::view]
	fn delegator_delegation_count(
		handle: &mut impl PrecompileHandle,
		delegator: Address,
	) -> EvmResult<u32> {
		let delegator = Runtime::AddressMapping::into_account_id(delegator.0);
		// CandidateInfo:
		// Twox64Concat(8) + AccountId(20) + Delegator(56 + MaxDelegationsPerDelegator)
		handle.record_db_read::<Runtime>(
			84 + (<Runtime as pallet_parachain_staking::Config>::MaxDelegationsPerDelegator::get()
				as usize),
		)?;
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

		Ok(result)
	}

	#[precompile::public("selectedCandidates()")]
	#[precompile::public("selected_candidates()")]
	#[precompile::view]
	fn selected_candidates(handle: &mut impl PrecompileHandle) -> EvmResult<Vec<Address>> {
		// TotalSelected
		handle.record_db_read::<Runtime>(4)?;
		let total_selected = pallet_parachain_staking::Pallet::<Runtime>::total_selected();
		// SelectedCandidates: total_selected * AccountId(20)
		handle.record_db_read::<Runtime>(20 * (total_selected as usize))?;
		let selected_candidates: Vec<Address> =
			pallet_parachain_staking::Pallet::<Runtime>::selected_candidates()
				.into_iter()
				.map(|address| Address(address.into()))
				.collect();

		Ok(selected_candidates)
	}

	#[precompile::public("delegationAmount(address,address)")]
	#[precompile::view]
	fn delegation_amount(
		handle: &mut impl PrecompileHandle,
		delegator: Address,
		candidate: Address,
	) -> EvmResult<U256> {
		// DelegatorState:
		// Twox64Concat(8) + AccountId(20) + Delegator(56 + MaxDelegationsPerDelegator)
		handle.record_db_read::<Runtime>(
			84 + (<Runtime as pallet_parachain_staking::Config>::MaxDelegationsPerDelegator::get()
				as usize),
		)?;
		let (candidate, delegator) = (
			Runtime::AddressMapping::into_account_id(candidate.0),
			Runtime::AddressMapping::into_account_id(delegator.0),
		);
		let amount = pallet_parachain_staking::Pallet::<Runtime>::delegator_state(&delegator)
			.and_then(|state| {
				state
					.delegations
					.0
					.into_iter()
					.find(|b| b.owner == candidate)
			})
			.map_or(
				U256::zero(),
				|pallet_parachain_staking::Bond { amount, .. }| amount.into(),
			);

		Ok(amount)
	}

	// Role Verifiers
	#[precompile::public("isInTopDelegations(address,address)")]
	#[precompile::view]
	fn is_in_top_delegations(
		handle: &mut impl PrecompileHandle,
		delegator: Address,
		candidate: Address,
	) -> EvmResult<bool> {
		let (candidate, delegator) = (
			Runtime::AddressMapping::into_account_id(candidate.0),
			Runtime::AddressMapping::into_account_id(delegator.0),
		);
		// TopDelegations:
		// Twox64Concat(8) + AccountId(20) + Balance(16)
		// + (AccountId(20) + Balance(16) * MaxTopDelegationsPerCandidate)
		handle.record_db_read::<Runtime>(
			44 + ((36
				* <Runtime as pallet_parachain_staking::Config>::MaxTopDelegationsPerCandidate::get(
				)) as usize),
		)?;
		let is_in_top_delegations = pallet_parachain_staking::Pallet::<Runtime>::top_delegations(
			&candidate,
		)
		.map_or(false, |delegations| {
			delegations
				.delegations
				.into_iter()
				.any(|b| b.owner == delegator)
		});

		Ok(is_in_top_delegations)
	}

	#[precompile::public("isDelegator(address)")]
	#[precompile::public("is_delegator(address)")]
	#[precompile::view]
	fn is_delegator(handle: &mut impl PrecompileHandle, delegator: Address) -> EvmResult<bool> {
		let delegator = Runtime::AddressMapping::into_account_id(delegator.0);
		// DelegatorState:
		// Twox64Concat(8) + AccountId(20) + Delegator(56 + MaxDelegationsPerDelegator)
		handle.record_db_read::<Runtime>(
			84 + (<Runtime as pallet_parachain_staking::Config>::MaxDelegationsPerDelegator::get()
				as usize),
		)?;
		let is_delegator = pallet_parachain_staking::Pallet::<Runtime>::is_delegator(&delegator);

		Ok(is_delegator)
	}

	#[precompile::public("isCandidate(address)")]
	#[precompile::public("is_candidate(address)")]
	#[precompile::view]
	fn is_candidate(handle: &mut impl PrecompileHandle, candidate: Address) -> EvmResult<bool> {
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// CandidateInfo: Twox64Concat(8) + AccountId(20) + CandidateMetadata(105)
		handle.record_db_read::<Runtime>(133)?;
		let is_candidate = pallet_parachain_staking::Pallet::<Runtime>::is_candidate(&candidate);

		Ok(is_candidate)
	}

	#[precompile::public("isSelectedCandidate(address)")]
	#[precompile::public("is_selected_candidate(address)")]
	#[precompile::view]
	fn is_selected_candidate(
		handle: &mut impl PrecompileHandle,
		candidate: Address,
	) -> EvmResult<bool> {
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// TotalSelected
		handle.record_db_read::<Runtime>(4)?;
		let total_selected = pallet_parachain_staking::Pallet::<Runtime>::total_selected();
		// SelectedCandidates: total_selected * AccountId(20)
		handle.record_db_read::<Runtime>(20 * (total_selected as usize))?;
		let is_selected =
			pallet_parachain_staking::Pallet::<Runtime>::is_selected_candidate(&candidate);

		Ok(is_selected)
	}

	#[precompile::public("delegationRequestIsPending(address,address)")]
	#[precompile::public("delegation_request_is_pending(address,address)")]
	#[precompile::view]
	fn delegation_request_is_pending(
		handle: &mut impl PrecompileHandle,
		delegator: Address,
		candidate: Address,
	) -> EvmResult<bool> {
		let delegator = Runtime::AddressMapping::into_account_id(delegator.0);
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// DelegationScheduledRequests:
		// Blake2128(16) + AccountId(20)
		// + Vec(
		// 	ScheduledRequest(20 + 4 + DelegationAction(18))
		//	* (MaxTopDelegationsPerCandidate + MaxBottomDelegationsPerCandidate)
		// )
		handle.record_db_read::<Runtime>(
			36 + (
				42 * (<Runtime as pallet_parachain_staking::Config>::MaxTopDelegationsPerCandidate::get()
				+ <Runtime as pallet_parachain_staking::Config>::MaxBottomDelegationsPerCandidate::get())
				as usize),
		)?;

		// If we are not able to get delegator state, we return false
		// Users can call `is_delegator` to determine when this happens
		let pending = <pallet_parachain_staking::Pallet<Runtime>>::delegation_request_exists(
			&candidate, &delegator,
		);

		Ok(pending)
	}

	#[precompile::public("candidateExitIsPending(address)")]
	#[precompile::public("candidate_exit_is_pending(address)")]
	#[precompile::view]
	fn candidate_exit_is_pending(
		handle: &mut impl PrecompileHandle,
		candidate: Address,
	) -> EvmResult<bool> {
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// CandidateInfo: Twox64Concat(8) + AccountId(20) + CandidateMetadata(105)
		handle.record_db_read::<Runtime>(133)?;

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

		Ok(pending)
	}

	#[precompile::public("candidateRequestIsPending(address)")]
	#[precompile::public("candidate_request_is_pending(address)")]
	#[precompile::view]
	fn candidate_request_is_pending(
		handle: &mut impl PrecompileHandle,
		candidate: Address,
	) -> EvmResult<bool> {
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// CandidateInfo: Twox64Concat(8) + AccountId(20) + CandidateMetadata(105)
		handle.record_db_read::<Runtime>(133)?;

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

		Ok(pending)
	}

	#[precompile::public("delegationAutoCompound(address,address)")]
	#[precompile::view]
	fn delegation_auto_compound(
		handle: &mut impl PrecompileHandle,
		delegator: Address,
		candidate: Address,
	) -> EvmResult<u8> {
		let delegator = Runtime::AddressMapping::into_account_id(delegator.0);
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// AutoCompoundingDelegations:
		// Blake2128(16) + AccountId(20)
		// + BoundedVec(
		// 	AutoCompoundConfig * (MaxTopDelegationsPerCandidate + MaxBottomDelegationsPerCandidate)
		// )
		handle.record_db_read::<Runtime>(
			36 + (
				22 * (<Runtime as pallet_parachain_staking::Config>::MaxTopDelegationsPerCandidate::get()
				+ <Runtime as pallet_parachain_staking::Config>::MaxBottomDelegationsPerCandidate::get())
				as usize),
		)?;

		let value = <pallet_parachain_staking::Pallet<Runtime>>::delegation_auto_compound(
			&candidate, &delegator,
		);

		Ok(value.deconstruct())
	}

	// Runtime Methods (dispatchables)

	#[precompile::public("joinCandidates(uint256,uint256)")]
	#[precompile::public("join_candidates(uint256,uint256)")]
	fn join_candidates(
		handle: &mut impl PrecompileHandle,
		amount: U256,
		candidate_count: Convert<U256, u32>,
	) -> EvmResult {
		let amount = Self::u256_to_amount(amount).in_field("amount")?;
		let candidate_count = candidate_count.converted();

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::join_candidates {
			bond: amount,
			candidate_count,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("scheduleLeaveCandidates(uint256)")]
	#[precompile::public("schedule_leave_candidates(uint256)")]
	fn schedule_leave_candidates(
		handle: &mut impl PrecompileHandle,
		candidate_count: Convert<U256, u32>,
	) -> EvmResult {
		let candidate_count = candidate_count.converted();

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::schedule_leave_candidates {
			candidate_count,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("executeLeaveCandidates(address,uint256)")]
	#[precompile::public("execute_leave_candidates(address,uint256)")]
	fn execute_leave_candidates(
		handle: &mut impl PrecompileHandle,
		candidate: Address,
		candidate_count: Convert<U256, u32>,
	) -> EvmResult {
		let candidate_count = candidate_count.converted();
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::execute_leave_candidates {
			candidate,
			candidate_delegation_count: candidate_count,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("cancelLeaveCandidates(uint256)")]
	#[precompile::public("cancel_leave_candidates(uint256)")]
	fn cancel_leave_candidates(
		handle: &mut impl PrecompileHandle,
		candidate_count: Convert<U256, u32>,
	) -> EvmResult {
		let candidate_count = candidate_count.converted();

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_parachain_staking::Call::<Runtime>::cancel_leave_candidates { candidate_count };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("goOffline()")]
	#[precompile::public("go_offline()")]
	fn go_offline(handle: &mut impl PrecompileHandle) -> EvmResult {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::go_offline {};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("goOnline()")]
	#[precompile::public("go_online()")]
	fn go_online(handle: &mut impl PrecompileHandle) -> EvmResult {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::go_online {};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("candidateBondMore(uint256)")]
	#[precompile::public("candidate_bond_more(uint256)")]
	fn candidate_bond_more(handle: &mut impl PrecompileHandle, more: U256) -> EvmResult {
		let more = Self::u256_to_amount(more).in_field("more")?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::candidate_bond_more { more };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("scheduleCandidateBondLess(uint256)")]
	#[precompile::public("schedule_candidate_bond_less(uint256)")]
	fn schedule_candidate_bond_less(handle: &mut impl PrecompileHandle, less: U256) -> EvmResult {
		let less = Self::u256_to_amount(less).in_field("less")?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::schedule_candidate_bond_less { less };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("executeCandidateBondLess(address)")]
	#[precompile::public("execute_candidate_bond_less(address)")]
	fn execute_candidate_bond_less(
		handle: &mut impl PrecompileHandle,
		candidate: Address,
	) -> EvmResult {
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_parachain_staking::Call::<Runtime>::execute_candidate_bond_less { candidate };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("cancelCandidateBondLess()")]
	#[precompile::public("cancel_candidate_bond_less()")]
	fn cancel_candidate_bond_less(handle: &mut impl PrecompileHandle) -> EvmResult {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::cancel_candidate_bond_less {};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("delegateWithAutoCompound(address,uint256,uint8,uint256,uint256,uint256)")]
	fn delegate_with_auto_compound(
		handle: &mut impl PrecompileHandle,
		candidate: Address,
		amount: U256,
		auto_compound: u8,
		candidate_delegation_count: Convert<U256, u32>,
		candidate_auto_compounding_delegation_count: Convert<U256, u32>,
		delegator_delegation_count: Convert<U256, u32>,
	) -> EvmResult {
		if auto_compound > 100 {
			return Err(
				RevertReason::custom("Must be an integer between 0 and 100 included")
					.in_field("auto_compound")
					.into(),
			);
		}

		let amount = Self::u256_to_amount(amount).in_field("amount")?;
		let auto_compound = Percent::from_percent(auto_compound);
		let candidate_delegation_count = candidate_delegation_count.converted();
		let candidate_auto_compounding_delegation_count =
			candidate_auto_compounding_delegation_count.converted();
		let delegator_delegation_count = delegator_delegation_count.converted();

		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::delegate_with_auto_compound {
			candidate,
			amount,
			auto_compound,
			candidate_delegation_count,
			candidate_auto_compounding_delegation_count,
			delegation_count: delegator_delegation_count,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("scheduleRevokeDelegation(address)")]
	#[precompile::public("schedule_revoke_delegation(address)")]
	fn schedule_revoke_delegation(
		handle: &mut impl PrecompileHandle,
		candidate: Address,
	) -> EvmResult {
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::schedule_revoke_delegation {
			collator: candidate,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("delegatorBondMore(address,uint256)")]
	#[precompile::public("delegator_bond_more(address,uint256)")]
	fn delegator_bond_more(
		handle: &mut impl PrecompileHandle,
		candidate: Address,
		more: U256,
	) -> EvmResult {
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);
		let more = Self::u256_to_amount(more).in_field("more")?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_parachain_staking::Call::<Runtime>::delegator_bond_more { candidate, more };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("scheduleDelegatorBondLess(address,uint256)")]
	#[precompile::public("schedule_delegator_bond_less(address,uint256)")]
	fn schedule_delegator_bond_less(
		handle: &mut impl PrecompileHandle,
		candidate: Address,
		less: U256,
	) -> EvmResult {
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);
		let less = Self::u256_to_amount(less).in_field("less")?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::schedule_delegator_bond_less {
			candidate,
			less,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("executeDelegationRequest(address,address)")]
	#[precompile::public("execute_delegation_request(address,address)")]
	fn execute_delegation_request(
		handle: &mut impl PrecompileHandle,
		delegator: Address,
		candidate: Address,
	) -> EvmResult {
		let delegator = Runtime::AddressMapping::into_account_id(delegator.0);
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::execute_delegation_request {
			delegator,
			candidate,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("cancelDelegationRequest(address)")]
	#[precompile::public("cancel_delegation_request(address)")]
	fn cancel_delegation_request(
		handle: &mut impl PrecompileHandle,
		candidate: Address,
	) -> EvmResult {
		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_parachain_staking::Call::<Runtime>::cancel_delegation_request { candidate };

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("setAutoCompound(address,uint8,uint256,uint256)")]
	fn set_auto_compound(
		handle: &mut impl PrecompileHandle,
		candidate: Address,
		value: u8,
		candidate_auto_compounding_delegation_count: Convert<U256, u32>,
		delegator_delegation_count: Convert<U256, u32>,
	) -> EvmResult {
		if value > 100 {
			return Err(
				RevertReason::custom("Must be an integer between 0 and 100 included")
					.in_field("value")
					.into(),
			);
		}

		let value = Percent::from_percent(value);
		let candidate_auto_compounding_delegation_count_hint =
			candidate_auto_compounding_delegation_count.converted();
		let delegation_count_hint = delegator_delegation_count.converted();

		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_parachain_staking::Call::<Runtime>::set_auto_compound {
			candidate,
			value,
			candidate_auto_compounding_delegation_count_hint,
			delegation_count_hint,
		};

		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("getDelegatorTotalStaked(address)")]
	#[precompile::view]
	fn get_delegator_total_staked(
		handle: &mut impl PrecompileHandle,
		delegator: Address,
	) -> EvmResult<U256> {
		// DelegatorState:
		// Twox64Concat(8) + AccountId(20) + Delegator(56 + MaxDelegationsPerDelegator)
		handle.record_db_read::<Runtime>(
			84 + (<Runtime as pallet_parachain_staking::Config>::MaxDelegationsPerDelegator::get()
				as usize),
		)?;

		let delegator = Runtime::AddressMapping::into_account_id(delegator.0);

		let amount = <pallet_parachain_staking::Pallet<Runtime>>::delegator_state(&delegator)
			.map(|state| state.total)
			.unwrap_or_default();

		Ok(amount.into())
	}

	#[precompile::public("getCandidateTotalCounted(address)")]
	#[precompile::view]
	fn get_candidate_total_counted(
		handle: &mut impl PrecompileHandle,
		candidate: Address,
	) -> EvmResult<U256> {
		// CandidateInfo: Twox64Concat(8) + AccountId(20) + CandidateMetadata(105)
		handle.record_db_read::<Runtime>(133)?;

		let candidate = Runtime::AddressMapping::into_account_id(candidate.0);

		let amount = <pallet_parachain_staking::Pallet<Runtime>>::candidate_info(&candidate)
			.map(|state| state.total_counted)
			.unwrap_or_default();

		Ok(amount.into())
	}

	fn u256_to_amount(value: U256) -> MayRevert<BalanceOf<Runtime>> {
		value
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").into())
	}
}
