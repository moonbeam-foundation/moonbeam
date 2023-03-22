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
use frame_support::traits::{Currency, Polling};
use pallet_conviction_voting::Call as ConvictionVotingCall;
use pallet_conviction_voting::{AccountVote, Conviction, Tally, Vote};
use pallet_evm::{AddressMapping, Log};
use precompile_utils::prelude::*;
use sp_core::{H160, H256, U256};
use sp_runtime::traits::StaticLookup;
use sp_std::marker::PhantomData;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type BalanceOf<Runtime> = <<Runtime as pallet_conviction_voting::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;
type IndexOf<Runtime> = <<Runtime as pallet_conviction_voting::Config>::Polls as Polling<
	Tally<
		<<Runtime as pallet_conviction_voting::Config>::Currency as Currency<
			<Runtime as frame_system::Config>::AccountId,
		>>::Balance,
		<Runtime as pallet_conviction_voting::Config>::MaxTurnout,
	>,
>>::Index;
type ClassOf<Runtime> = <<Runtime as pallet_conviction_voting::Config>::Polls as Polling<
	Tally<
		<<Runtime as pallet_conviction_voting::Config>::Currency as Currency<
			<Runtime as frame_system::Config>::AccountId,
		>>::Balance,
		<Runtime as pallet_conviction_voting::Config>::MaxTurnout,
	>,
>>::Class;

/// Solidity selector of the Vote log, which is the Keccak of the Log signature.
pub(crate) const SELECTOR_LOG_VOTED: [u8; 32] =
	keccak256!("Voted(uint32,address,bool,uint256,uint8)");

/// Solidity selector of the Vote Split log, which is the Keccak of the Log signature.
pub(crate) const SELECTOR_LOG_VOTE_SPLIT: [u8; 32] =
	keccak256!("VoteSplit(uint32,address,uint256,uint256)");

/// Solidity selector of the Vote Split Abstained log, which is the Keccak of the Log signature.
pub(crate) const SELECTOR_LOG_VOTE_SPLIT_ABSTAINED: [u8; 32] =
	keccak256!("VoteSplitAbstained(uint32,address,uint256,uint256,uint256)");

/// Solidity selector of the VoteRemove log, which is the Keccak of the Log signature.
pub(crate) const SELECTOR_LOG_VOTE_REMOVED: [u8; 32] = keccak256!("VoteRemoved(uint32,address)");

/// Solidity selector of the VoteRemoveOther log, which is the Keccak of the Log signature.
pub(crate) const SELECTOR_LOG_VOTE_REMOVED_OTHER: [u8; 32] =
	keccak256!("VoteRemovedOther(uint32,address,address,uint16)");

/// Solidity selector of the Delegate log, which is the Keccak of the Log signature.
pub(crate) const SELECTOR_LOG_DELEGATED: [u8; 32] =
	keccak256!("Delegated(uint16,address,address,uint256,uint8)");

/// Solidity selector of the Undelegate log, which is the Keccak of the Log signature.
pub(crate) const SELECTOR_LOG_UNDELEGATED: [u8; 32] = keccak256!("Undelegated(uint16,address)");

/// Solidity selector of the Unlock log, which is the Keccak of the Log signature.
pub(crate) const SELECTOR_LOG_UNLOCKED: [u8; 32] = keccak256!("Unlocked(uint16,address)");

/// A precompile to wrap the functionality from pallet-conviction-voting.
pub struct ConvictionVotingPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> ConvictionVotingPrecompile<Runtime>
where
	Runtime: pallet_conviction_voting::Config + pallet_evm::Config + frame_system::Config,
	BalanceOf<Runtime>: TryFrom<U256>,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::RuntimeCall: From<ConvictionVotingCall<Runtime>>,
	IndexOf<Runtime>: TryFrom<u32>,
	ClassOf<Runtime>: TryFrom<u16>,
{
	/// Internal helper function for vote* extrinsics exposed in this precompile.
	fn vote(
		handle: &mut impl PrecompileHandle,
		poll_index: u32,
		vote: AccountVote<U256>,
	) -> EvmResult {
		let caller = handle.context().caller;
		let (poll_index, vote, event) = Self::log_vote_event(handle, poll_index, vote)?;

		let origin = Runtime::AddressMapping::into_account_id(caller);
		let call = ConvictionVotingCall::<Runtime>::vote { poll_index, vote }.into();

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		event.record(handle)?;

		Ok(())
	}

	/// Vote yes in a poll.
	///
	/// Parameters:
	/// * poll_index: Index of poll
	/// * vote_amount: Balance locked for vote
	/// * conviction: Conviction multiplier for length of vote lock
	#[precompile::public("voteYes(uint32,uint256,uint8)")]
	fn vote_yes(
		handle: &mut impl PrecompileHandle,
		poll_index: u32,
		vote_amount: U256,
		conviction: u8,
	) -> EvmResult {
		Self::vote(
			handle,
			poll_index,
			AccountVote::Standard {
				vote: Vote {
					aye: true,
					conviction: Self::u8_to_conviction(conviction).in_field("conviction")?,
				},
				balance: vote_amount,
			},
		)
	}

	/// Vote no in a poll.
	///
	/// Parameters:
	/// * poll_index: Index of poll
	/// * vote_amount: Balance locked for vote
	/// * conviction: Conviction multiplier for length of vote lock
	#[precompile::public("voteNo(uint32,uint256,uint8)")]
	fn vote_no(
		handle: &mut impl PrecompileHandle,
		poll_index: u32,
		vote_amount: U256,
		conviction: u8,
	) -> EvmResult {
		Self::vote(
			handle,
			poll_index,
			AccountVote::Standard {
				vote: Vote {
					aye: false,
					conviction: Self::u8_to_conviction(conviction).in_field("conviction")?,
				},
				balance: vote_amount,
			},
		)
	}

	/// Vote split in a poll.
	///
	/// Parameters:
	/// * poll_index: Index of poll
	/// * aye: Balance locked for aye vote
	/// * nay: Balance locked for nay vote
	#[precompile::public("voteSplit(uint32,uint256,uint256)")]
	fn vote_split(
		handle: &mut impl PrecompileHandle,
		poll_index: u32,
		aye: U256,
		nay: U256,
	) -> EvmResult {
		Self::vote(handle, poll_index, AccountVote::Split { aye, nay })
	}

	/// Vote split in a poll.
	///
	/// Parameters:
	/// * poll_index: Index of poll
	/// * aye: Balance locked for aye vote
	/// * nay: Balance locked for nay vote
	#[precompile::public("voteSplitAbstain(uint32,uint256,uint256,uint256)")]
	fn vote_split_abstain(
		handle: &mut impl PrecompileHandle,
		poll_index: u32,
		aye: U256,
		nay: U256,
		abstain: U256,
	) -> EvmResult {
		Self::vote(
			handle,
			poll_index,
			AccountVote::SplitAbstain { aye, nay, abstain },
		)
	}

	#[precompile::public("removeVote(uint32)")]
	fn remove_vote(handle: &mut impl PrecompileHandle, poll_index: u32) -> EvmResult {
		let caller = handle.context().caller;
		let event = log2(
			handle.context().address,
			SELECTOR_LOG_VOTE_REMOVED,
			H256::from_low_u64_be(poll_index as u64), // poll index,
			EvmDataWriter::new()
				.write::<Address>(Address(caller))
				.build(),
		);
		handle.record_log_costs(&[&event])?;

		let index = Self::u32_to_index(poll_index).in_field("pollIndex")?;

		log::trace!(
			target: "conviction-voting-precompile",
			"Removing vote from poll {:?}",
			index
		);

		let origin = Runtime::AddressMapping::into_account_id(caller);
		let call = ConvictionVotingCall::<Runtime>::remove_vote { class: None, index };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		event.record(handle)?;

		Ok(())
	}

	#[precompile::public("removeOtherVote(address,uint16,uint32)")]
	fn remove_other_vote(
		handle: &mut impl PrecompileHandle,
		target: Address,
		track_id: u16,
		poll_index: u32,
	) -> EvmResult {
		let caller = handle.context().caller;

		let event = log2(
			handle.context().address,
			SELECTOR_LOG_VOTE_REMOVED_OTHER,
			H256::from_low_u64_be(poll_index as u64), // poll index,
			EvmDataWriter::new()
				.write::<Address>(Address(caller))
				.write::<Address>(target)
				.write::<u16>(track_id)
				.build(),
		);
		handle.record_log_costs(&[&event])?;

		let class = Self::u16_to_track_id(track_id).in_field("trackId")?;
		let index = Self::u32_to_index(poll_index).in_field("pollIndex")?;

		let target = Runtime::AddressMapping::into_account_id(target.into());
		let target: <Runtime::Lookup as StaticLookup>::Source =
			Runtime::Lookup::unlookup(target.clone());

		log::trace!(
			target: "conviction-voting-precompile",
			"Removing other vote from poll {:?}",
			index
		);

		let origin = Runtime::AddressMapping::into_account_id(caller);
		let call = ConvictionVotingCall::<Runtime>::remove_other_vote {
			target,
			class,
			index,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		event.record(handle)?;

		Ok(())
	}

	#[precompile::public("delegate(uint16,address,uint8,uint256)")]
	fn delegate(
		handle: &mut impl PrecompileHandle,
		track_id: u16,
		representative: Address,
		conviction: u8,
		amount: U256,
	) -> EvmResult {
		let caller = handle.context().caller;

		let event = log2(
			handle.context().address,
			SELECTOR_LOG_DELEGATED,
			H256::from_low_u64_be(track_id as u64), // track id,
			EvmDataWriter::new()
				.write::<Address>(Address(caller))
				.write::<Address>(representative)
				.write::<U256>(amount)
				.write::<u8>(conviction)
				.build(),
		);
		handle.record_log_costs(&[&event])?;

		let class = Self::u16_to_track_id(track_id).in_field("trackId")?;
		let amount = Self::u256_to_amount(amount).in_field("amount")?;
		let conviction = Self::u8_to_conviction(conviction).in_field("conviction")?;

		log::trace!(target: "conviction-voting-precompile",
			"Delegating vote to {:?} with balance {:?} and conviction {:?}",
			representative, amount, conviction
		);

		let representative = Runtime::AddressMapping::into_account_id(representative.into());
		let to: <Runtime::Lookup as StaticLookup>::Source =
			Runtime::Lookup::unlookup(representative.clone());
		let origin = Runtime::AddressMapping::into_account_id(caller);
		let call = ConvictionVotingCall::<Runtime>::delegate {
			class,
			to,
			conviction,
			balance: amount,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		event.record(handle)?;

		Ok(())
	}
	#[precompile::public("undelegate(uint16)")]
	fn undelegate(handle: &mut impl PrecompileHandle, track_id: u16) -> EvmResult {
		let caller = handle.context().caller;

		let event = log2(
			handle.context().address,
			SELECTOR_LOG_UNDELEGATED,
			H256::from_low_u64_be(track_id as u64), // track id,
			EvmDataWriter::new()
				.write::<Address>(Address(caller))
				.build(),
		);
		handle.record_log_costs(&[&event])?;

		let class = Self::u16_to_track_id(track_id).in_field("trackId")?;
		let origin = Runtime::AddressMapping::into_account_id(caller);
		let call = ConvictionVotingCall::<Runtime>::undelegate { class };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		event.record(handle)?;

		Ok(())
	}
	#[precompile::public("unlock(uint16,address)")]
	fn unlock(handle: &mut impl PrecompileHandle, track_id: u16, target: Address) -> EvmResult {
		let class = Self::u16_to_track_id(track_id).in_field("trackId")?;

		let event = log2(
			handle.context().address,
			SELECTOR_LOG_UNLOCKED,
			H256::from_low_u64_be(track_id as u64), // track id,
			EvmDataWriter::new().write::<Address>(target).build(),
		);
		handle.record_log_costs(&[&event])?;

		let target: H160 = target.into();
		let target = Runtime::AddressMapping::into_account_id(target);
		let target: <Runtime::Lookup as StaticLookup>::Source =
			Runtime::Lookup::unlookup(target.clone());

		log::trace!(
			target: "conviction-voting-precompile",
			"Unlocking conviction-voting tokens for {:?}", target
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ConvictionVotingCall::<Runtime>::unlock { class, target };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		event.record(handle)?;

		Ok(())
	}
	fn u8_to_conviction(conviction: u8) -> MayRevert<Conviction> {
		conviction
			.try_into()
			.map_err(|_| RevertReason::custom("Must be an integer between 0 and 6 included").into())
	}
	fn u32_to_index(index: u32) -> MayRevert<IndexOf<Runtime>> {
		index
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("index type").into())
	}
	fn u16_to_track_id(class: u16) -> MayRevert<ClassOf<Runtime>> {
		class
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("trackId type").into())
	}
	fn u256_to_amount(value: U256) -> MayRevert<BalanceOf<Runtime>> {
		value
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").into())
	}
	fn log_vote_event(
		handle: &mut impl PrecompileHandle,
		poll_index: u32,
		vote: AccountVote<U256>,
	) -> EvmResult<(IndexOf<Runtime>, AccountVote<BalanceOf<Runtime>>, Log)> {
		let (contract_addr, caller) = (handle.context().address, handle.context().caller);
		let (vote, event) = match vote {
			AccountVote::Standard { vote, balance } => {
				let event = log2(
					contract_addr,
					SELECTOR_LOG_VOTED,
					H256::from_low_u64_be(poll_index as u64),
					EvmDataWriter::new()
						.write::<Address>(Address(caller))
						.write::<bool>(vote.aye)
						.write::<U256>(balance)
						.write::<u8>(vote.conviction.into())
						.build(),
				);
				(
					AccountVote::Standard {
						vote,
						balance: Self::u256_to_amount(balance).in_field("voteAmount")?,
					},
					event,
				)
			}
			AccountVote::Split { aye, nay } => {
				let event = log2(
					contract_addr,
					SELECTOR_LOG_VOTE_SPLIT,
					H256::from_low_u64_be(poll_index as u64),
					EvmDataWriter::new()
						.write::<Address>(Address(caller))
						.write::<U256>(aye)
						.write::<U256>(nay)
						.build(),
				);
				(
					AccountVote::Split {
						aye: Self::u256_to_amount(aye).in_field("aye")?,
						nay: Self::u256_to_amount(nay).in_field("nay")?,
					},
					event,
				)
			}
			AccountVote::SplitAbstain { aye, nay, abstain } => {
				let event = log2(
					contract_addr,
					SELECTOR_LOG_VOTE_SPLIT_ABSTAINED,
					H256::from_low_u64_be(poll_index as u64),
					EvmDataWriter::new()
						.write::<Address>(Address(caller))
						.write::<U256>(aye)
						.write::<U256>(nay)
						.write::<U256>(abstain)
						.build(),
				);
				(
					AccountVote::SplitAbstain {
						aye: Self::u256_to_amount(aye).in_field("aye")?,
						nay: Self::u256_to_amount(nay).in_field("nay")?,
						abstain: Self::u256_to_amount(abstain).in_field("abstain")?,
					},
					event,
				)
			}
			#[allow(unreachable_patterns)] // in case a new vote variant is added then still works
			_ => return Err(RevertReason::custom("Vote variant unsupported").into()),
		};
		handle.record_log_costs(&[&event])?;
		Ok((Self::u32_to_index(poll_index)?, vote, event))
	}
}
