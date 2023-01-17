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
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{H160, U256};
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
	/// Vote in a poll.
	///
	/// Parameters:
	/// * poll_index: Index of poll
	/// * aye: Yes or no vote
	/// * vote_amount: Balance locked for vote
	/// * conviction: Conviction multiplier for length of vote lock
	#[precompile::public("vote(uint32,bool,uint256,uint8)")]
	fn vote(
		handle: &mut impl PrecompileHandle,
		poll_index: u32,
		aye: bool,
		vote_amount: U256,
		conviction: u8,
	) -> EvmResult {
		let poll_index = Self::u32_to_index(poll_index).in_field("pollIndex")?;
		let vote_amount = Self::u256_to_amount(vote_amount).in_field("voteAmount")?;
		let conviction = Self::u8_to_conviction(conviction).in_field("conviction")?;

		let vote = AccountVote::Standard {
			vote: Vote { aye, conviction },
			balance: vote_amount,
		};

		log::trace!(target: "conviction-voting-precompile",
			"Voting {:?} on poll {:?}, with conviction {:?}",
			aye, poll_index, conviction
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ConvictionVotingCall::<Runtime>::vote { poll_index, vote }.into();

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("removeVote(uint32)")]
	fn remove_vote(handle: &mut impl PrecompileHandle, poll_index: u32) -> EvmResult {
		let index = Self::u32_to_index(poll_index).in_field("pollIndex")?;

		log::trace!(
			target: "conviction-voting-precompile",
			"Removing vote from poll {:?}",
			index
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ConvictionVotingCall::<Runtime>::remove_vote { class: None, index };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("removeOtherVote(address,uint16,uint32)")]
	fn remove_other_vote(
		handle: &mut impl PrecompileHandle,
		target: Address,
		track_id: u16,
		poll_index: u32,
	) -> EvmResult {
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

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ConvictionVotingCall::<Runtime>::remove_other_vote {
			target,
			class,
			index,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

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
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ConvictionVotingCall::<Runtime>::delegate {
			class,
			to,
			conviction,
			balance: amount,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}
	#[precompile::public("undelegate(uint16)")]
	fn undelegate(handle: &mut impl PrecompileHandle, track_id: u16) -> EvmResult {
		let class = Self::u16_to_track_id(track_id).in_field("trackId")?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ConvictionVotingCall::<Runtime>::undelegate { class };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}
	#[precompile::public("unlock(uint16,address)")]
	fn unlock(handle: &mut impl PrecompileHandle, track_id: u16, target: Address) -> EvmResult {
		let class = Self::u16_to_track_id(track_id).in_field("trackId")?;
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
}
