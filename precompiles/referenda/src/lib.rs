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

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(assert_matches)]

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::{
	schedule::DispatchTime, Bounded, ConstU32, Currency, Get, OriginTrait,
};
use pallet_evm::AddressMapping;
use pallet_referenda::{Call as ReferendaCall, DecidingCount, ReferendumCount, TracksInfo, Pallet as Referenda, ReferendumInfo, ReferendumInfoFor};
use parity_scale_codec::Encode;
use precompile_utils::{data::String, prelude::*};
use sp_core::{Hasher, H256, U256};
use sp_std::{boxed::Box, marker::PhantomData, str::FromStr, vec::Vec};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);

type GetCallDataLimit = ConstU32<CALL_DATA_LIMIT>;
type BalanceOf<Runtime> = <<Runtime as pallet_referenda::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;
type TrackIdOf<Runtime> = <<Runtime as pallet_referenda::Config>::Tracks as TracksInfo<
	BalanceOf<Runtime>,
	<Runtime as frame_system::Config>::BlockNumber,
>>::Id;
type BoundedCallOf<Runtime> = Bounded<<Runtime as pallet_referenda::Config>::RuntimeCall>;

type OriginOf<Runtime> =
	<<Runtime as frame_system::Config>::RuntimeOrigin as OriginTrait>::PalletsOrigin;

/// Solidity selector of the SubmittedAt log, which is the Keccak of the Log signature.
pub(crate) const SELECTOR_LOG_SUBMITTED_AT: [u8; 32] =
	keccak256!("SubmittedAt(uint16,uint32,bytes32)");

/// Solidity selector of the SubmittedAfter log, which is the Keccak of the Log signature.
pub(crate) const SELECTOR_LOG_SUBMITTED_AFTER: [u8; 32] =
	keccak256!("SubmittedAfter(uint16,uint32,bytes32)");

/// Solidity selector of the DecisionDepositPlaced log, which is the Keccak of the Log signature.
pub(crate) const SELECTOR_LOG_DECISION_DEPOSIT_PLACED: [u8; 32] =
	keccak256!("DecisionDepositPlaced(uint32,address,uint256)");

/// Solidity selector of the DecisionDepositRefunded log, which is the Keccak of the Log signature.
pub(crate) const SELECTOR_LOG_DECISION_DEPOSIT_REFUNDED: [u8; 32] =
	keccak256!("DecisionDepositRefunded(uint32,address,uint256)");

#[derive(EvmData)]
pub struct TrackInfo {
	name: UnboundedBytes,
	max_deciding: U256,
	decision_deposit: U256,
	prepare_period: U256,
	decision_period: U256,
	confirm_period: U256,
	min_enactment_period: U256,
	min_approval: UnboundedBytes,
	min_support: UnboundedBytes,
}

/// A precompile to wrap the functionality from pallet-referenda.
pub struct ReferendaPrecompile<Runtime, GovOrigin>(PhantomData<(Runtime, GovOrigin)>);

#[precompile_utils::precompile]
impl<Runtime, GovOrigin> ReferendaPrecompile<Runtime, GovOrigin>
where
	Runtime: pallet_referenda::Config + pallet_evm::Config + frame_system::Config,
	OriginOf<Runtime>: From<GovOrigin>,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::RuntimeCall: From<ReferendaCall<Runtime>>,
	<Runtime as frame_system::Config>::Hash: Into<H256>,
	Runtime::BlockNumber: Into<U256>,
	TrackIdOf<Runtime>: TryFrom<u16> + TryInto<u16>,
	BalanceOf<Runtime>: Into<U256>,
	GovOrigin: FromStr,
{
	// The accessors are first. They directly return their result.
	#[precompile::public("referendumCount()")]
	#[precompile::view]
	fn referendum_count(handle: &mut impl PrecompileHandle) -> EvmResult<u32> {
		// Fetch data from pallet
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let ref_count = ReferendumCount::<Runtime>::get();
		log::trace!(target: "referendum-precompile", "Referendum count is {:?}", ref_count);

		Ok(ref_count)
	}

	#[precompile::public("submissionDeposit()")]
	#[precompile::view]
	fn submission_deposit(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
		// Fetch data from pallet
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let submission_deposit = Runtime::SubmissionDeposit::get();
		log::trace!(target: "referendum-precompile", "Submission deposit is {:?}", submission_deposit);

		Ok(submission_deposit.into())
	}

	#[precompile::public("decidingCount(uint16)")]
	#[precompile::view]
	fn deciding_count(handle: &mut impl PrecompileHandle, track_id: u16) -> EvmResult<U256> {
		// Fetch data from pallet
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let track_id: TrackIdOf<Runtime> = track_id
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("Track id type").into())
			.in_field("trackId")?;
		let deciding_count = DecidingCount::<Runtime>::get(track_id);
		log::trace!(
			target: "referendum-precompile", "Track {:?} deciding count is {:?}",
			track_id,
			deciding_count
		);

		Ok(deciding_count.into())
	}

	#[precompile::public("trackIds()")]
	#[precompile::view]
	fn track_ids(handle: &mut impl PrecompileHandle) -> EvmResult<Vec<u16>> {
		// Fetch data from runtime
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let track_ids: Vec<u16> = Runtime::Tracks::tracks()
			.into_iter()
			.filter_map(|(id, _)| {
				if let Ok(track_id) = (*id).try_into() {
					Some(track_id)
				} else {
					None
				}
			})
			.collect();

		Ok(track_ids)
	}

	#[precompile::public("trackInfo(uint16)")]
	#[precompile::view]
	fn track_info(handle: &mut impl PrecompileHandle, track_id: u16) -> EvmResult<TrackInfo> {
		// Fetch data from runtime
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let track_id: TrackIdOf<Runtime> = track_id
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("Track id type").into())
			.in_field("trackId")?;
		let tracks = Runtime::Tracks::tracks();
		let index = tracks
			.binary_search_by_key(&track_id, |(id, _)| *id)
			.unwrap_or_else(|x| x);
		let (_, track_info) = &tracks[index];

		Ok(TrackInfo {
			name: track_info.name.into(),
			max_deciding: track_info.max_deciding.into(),
			decision_deposit: track_info.decision_deposit.into(),
			prepare_period: track_info.prepare_period.into(),
			decision_period: track_info.decision_period.into(),
			confirm_period: track_info.confirm_period.into(),
			min_enactment_period: track_info.min_enactment_period.into(),
			min_approval: track_info.min_approval.encode().into(),
			min_support: track_info.min_support.encode().into(),
		})
	}

	/// Use Runtime::Tracks::tracks to get the origin for input trackId
	fn track_id_to_origin(track_id: TrackIdOf<Runtime>) -> EvmResult<Box<OriginOf<Runtime>>> {
		let tracks = Runtime::Tracks::tracks();
		let index = tracks
			.binary_search_by_key(&track_id, |(id, _)| *id)
			.unwrap_or_else(|x| x);
		let (_, track_info) = &tracks[index];
		let origin = if track_info.name == "root" {
			frame_system::RawOrigin::Root.into()
		} else {
			GovOrigin::from_str(track_info.name)
				.map_err(|_| {
					RevertReason::custom("Custom origin does not exist for {track_info.name}")
						.in_field("trackId")
				})?
				.into()
		};
		Ok(Box::new(origin))
	}

	// Helper function for submitAt and submitAfter
	fn submit(
		handle: &mut impl PrecompileHandle,
		track_id: u16,
		proposal: Vec<u8>,
		enactment_moment: DispatchTime<Runtime::BlockNumber>,
	) -> EvmResult<u32> {
		// record cost to read referendumCount to get the referendum index
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let referendum_index = ReferendumCount::<Runtime>::get();
		let proposal_origin = Self::track_id_to_origin(
			track_id
				.try_into()
				.map_err(|_| RevertReason::value_is_too_large("Track id type").into())
				.in_field("trackId")?,
		)?;
		let proposal: BoundedCallOf<Runtime> =
			Bounded::Inline(frame_support::BoundedVec::try_from(proposal).map_err(|_| {
				RevertReason::custom("Proposal input is not a runtime call").in_field("proposal")
			})?);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = ReferendaCall::<Runtime>::submit {
			proposal_origin,
			proposal,
			enactment_moment,
		}
		.into();

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(referendum_index)
	}

	/// Propose a referendum on a privileged action.
	///
	/// Parameters:
	/// * track_id: The trackId for the origin from which the proposal is to be dispatched.
	/// * proposal: The proposed runtime call.
	/// * block_number: Block number at which proposal is dispatched.
	#[precompile::public("submitAt(uint16,bytes,uint32)")]
	fn submit_at(
		handle: &mut impl PrecompileHandle,
		track_id: u16,
		proposal: BoundedBytes<GetCallDataLimit>,
		block_number: u32,
	) -> EvmResult<u32> {
		let proposal: sp_std::vec::Vec<u8> = proposal.into();
		let hash = <Runtime as frame_system::Config>::Hashing::hash(&proposal);
		handle.record_log_costs_manual(2, 32 * 2)?;

		let referendum_index = Self::submit(
			handle,
			track_id,
			proposal,
			DispatchTime::At(block_number.into()),
		)?;
		let event = log2(
			handle.context().address,
			SELECTOR_LOG_SUBMITTED_AT,
			H256::from_low_u64_be(track_id as u64),
			EvmDataWriter::new()
				.write::<u32>(referendum_index)
				.write::<H256>(hash.into())
				.build(),
		);
		event.record(handle)?;

		Ok(referendum_index)
	}

	/// Propose a referendum on a privileged action.
	///
	/// Parameters:
	/// * track_id: The trackId for the origin from which the proposal is to be dispatched.
	/// * proposal: The proposed runtime call.
	/// * block_number: Block number after which proposal is dispatched.
	#[precompile::public("submitAfter(uint16,bytes,uint32)")]
	fn submit_after(
		handle: &mut impl PrecompileHandle,
		track_id: u16,
		proposal: BoundedBytes<GetCallDataLimit>,
		block_number: u32,
	) -> EvmResult<u32> {
		let proposal: sp_std::vec::Vec<u8> = proposal.into();
		let hash = <Runtime as frame_system::Config>::Hashing::hash(&proposal);
		handle.record_log_costs_manual(2, 32 * 2)?;

		let referendum_index = Self::submit(
			handle,
			track_id,
			proposal,
			DispatchTime::After(block_number.into()),
		)?;
		let event = log2(
			handle.context().address,
			SELECTOR_LOG_SUBMITTED_AFTER,
			H256::from_low_u64_be(track_id as u64),
			EvmDataWriter::new()
				.write::<u32>(referendum_index)
				.write::<H256>(hash.into())
				.build(),
		);

		event.record(handle)?;

		Ok(referendum_index)
	}

	/// Post the Decision Deposit for a referendum.
	///
	/// Parameters:
	/// * index: The index of the submitted referendum whose Decision Deposit is yet to be posted.
	#[precompile::public("placeDecisionDeposit(uint32)")]
	fn place_decision_deposit(handle: &mut impl PrecompileHandle, index: u32) -> EvmResult {
		// Account later `ensure_ongoing` read cost
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		handle.record_log_costs_manual(1, 32 * 3)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = ReferendaCall::<Runtime>::place_decision_deposit { index }.into();

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		// Once the deposit has been succesfully placed, it is available in the ReferendumStatus.
		let ongoing_referendum = Referenda::<Runtime>::ensure_ongoing(index)
			.map_err(|_| RevertReason::custom("Index is not an ongoing referendum").in_field("index"))?;
		let decision_deposit: U256 = if let Some(decision_deposit) = ongoing_referendum.decision_deposit {
			decision_deposit.amount.into()
		} else {
			U256::zero()
		};
		let event = log1(
			handle.context().address,
			SELECTOR_LOG_DECISION_DEPOSIT_PLACED,
			EvmDataWriter::new()
				.write::<u32>(index)
				.write::<Address>(Address(handle.context().caller))
				.write::<U256>(decision_deposit)
				.build(),
		);

		event.record(handle)?;
		Ok(())
	}

	/// Refund the Decision Deposit for a closed referendum back to the depositor.
	///
	/// Parameters:
	/// * index: The index of a closed referendum whose Decision Deposit has not yet been refunded.
	#[precompile::public("refundDecisionDeposit(uint32)")]
	fn refund_decision_deposit(handle: &mut impl PrecompileHandle, index: u32) -> EvmResult {
		handle.record_log_costs_manual(1, 32 * 3)?;
		// Get refunding deposit before dispatch
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let refunded_deposit: U256 = match ReferendumInfoFor::<Runtime>::get(index).ok_or(RevertReason::custom("Referendum index does not exist").in_field("index"))? {
			ReferendumInfo::Ongoing(x) if x.decision_deposit.is_none() => U256::zero(),
			ReferendumInfo::Ongoing(_) => return Err(RevertReason::custom("Cannot refund an ongoing referendum").in_field("index").into()),
			ReferendumInfo::Approved(_, _, Some(d)) | ReferendumInfo::Rejected(_, _, Some(d)) | ReferendumInfo::TimedOut(_, _, Some(d)) | ReferendumInfo::Cancelled(_, _, Some(d)) => d.amount.into(),
			_ => U256::zero(),
		};

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = ReferendaCall::<Runtime>::refund_decision_deposit { index }.into();

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;
		let event = log1(
			handle.context().address,
			SELECTOR_LOG_DECISION_DEPOSIT_REFUNDED,
			EvmDataWriter::new()
				.write::<u32>(index)
				.write::<Address>(Address(handle.context().caller))
				.write::<U256>(refunded_deposit)
				.build(),
		);

		event.record(handle)?;
		Ok(())
	}

	/// Refund the Submission Deposit for a closed referendum back to the depositor.
	///
	/// Parameters:
	/// * index: The index of a closed referendum whose Submission Deposit has not yet been refunded.
	#[precompile::public("refundSubmissionDeposit(uint32)")]
	fn refund_submission_deposit(handle: &mut impl PrecompileHandle, index: u32) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = ReferendaCall::<Runtime>::refund_submission_deposit { index }.into();

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;
		Ok(())
	}
}
