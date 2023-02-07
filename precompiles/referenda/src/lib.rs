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
use pallet_referenda::{Call as ReferendaCall, DecidingCount, ReferendumCount, TracksInfo};
use parity_scale_codec::Encode;
use precompile_utils::prelude::*;
use sp_core::U256;
use sp_std::{boxed::Box, marker::PhantomData, vec::Vec};

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

/// A precompile to wrap the functionality from pallet-referenda.
pub struct ReferendaPrecompile<Runtime, GovOrigin: TryFrom<u16>>(PhantomData<(Runtime, GovOrigin)>);

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
	Runtime::BlockNumber: Into<U256>,
	TrackIdOf<Runtime>: TryFrom<u16> + TryInto<u16>,
	BalanceOf<Runtime>: Into<U256>,
	GovOrigin: TryFrom<u16>,
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
			.filter_map(|x| {
				if let Ok(track_id) = x.0.try_into() {
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
	fn track_info(
		handle: &mut impl PrecompileHandle,
		track_id: u16,
	) -> EvmResult<(
		UnboundedBytes,
		U256,
		U256,
		U256,
		U256,
		U256,
		U256,
		UnboundedBytes,
		UnboundedBytes,
	)> {
		// Fetch data from runtime
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let track_id: TrackIdOf<Runtime> = track_id
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("Track id type").into())
			.in_field("trackId")?;
		let tracks = Runtime::Tracks::tracks();
		let index = tracks
			.binary_search_by_key(&track_id, |x| x.0)
			.unwrap_or_else(|x| x);
		let track_info = &tracks[index].1;

		Ok((
			track_info.name.as_bytes().into(),
			track_info.max_deciding.into(),
			track_info.decision_deposit.into(),
			track_info.prepare_period.into(),
			track_info.decision_period.into(),
			track_info.confirm_period.into(),
			track_info.min_enactment_period.into(),
			track_info.min_approval.encode().into(),
			track_info.min_support.encode().into(),
		))
	}

	// Helper function for submitAt and submitAfter
	fn submit(
		handle: &mut impl PrecompileHandle,
		track_id: u16,
		proposal: BoundedBytes<GetCallDataLimit>,
		at: bool,
		block_number: u32,
	) -> EvmResult<u32> {
		// for read of referendumCount to get the referendum index
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let referendum_index = ReferendumCount::<Runtime>::get();
		let proposal_origin: GovOrigin = track_id.try_into().map_err(|_| {
			RevertReason::custom("Origin does not exist for TrackId").in_field("trackId")
		})?;
		let proposal_origin: Box<OriginOf<Runtime>> = Box::new(proposal_origin.into());
		let proposal: BoundedCallOf<Runtime> = Bounded::Inline(
			frame_support::BoundedVec::try_from(proposal.as_bytes().to_vec()).map_err(|_| {
				RevertReason::custom("Proposal input is not a runtime call").in_field("proposal")
			})?,
		);
		let enactment_moment = if at {
			DispatchTime::At(block_number.into())
		} else {
			DispatchTime::After(block_number.into())
		};

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
		Self::submit(handle, track_id, proposal, true, block_number)
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
		Self::submit(handle, track_id, proposal, false, block_number)
	}

	/// Post the Decision Deposit for a referendum.
	///
	/// Parameters:
	/// * index: The index of the submitted referendum whose Decision Deposit is yet to be posted.
	#[precompile::public("placeDecisionDeposit(uint32)")]
	fn place_decision_deposit(handle: &mut impl PrecompileHandle, index: u32) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = ReferendaCall::<Runtime>::place_decision_deposit { index }.into();

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;
		Ok(())
	}

	/// Refund the Decision Deposit for a closed referendum back to the depositor.
	///
	/// Parameters:
	/// * index: The index of a closed referendum whose Decision Deposit has not yet been refunded.
	#[precompile::public("refundDecisionDeposit(uint32)")]
	fn refund_decision_deposit(handle: &mut impl PrecompileHandle, index: u32) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let call = ReferendaCall::<Runtime>::refund_decision_deposit { index }.into();

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;
		Ok(())
	}
}
