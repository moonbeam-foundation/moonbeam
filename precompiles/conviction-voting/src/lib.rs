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
use sp_core::{H256, U256};
use sp_std::marker::PhantomData;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

type BalanceOf<Runtime> = <<Runtime as pallet_conviction_voting::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

/// A precompile to wrap the functionality from pallet-conviction-voting.
pub struct ConvictionVotingPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> ConvictionVotingPrecompile<Runtime>
where
	Runtime: pallet_conviction_voting::Config + pallet_evm::Config + frame_system::Config,
	BalanceOf<Runtime>: TryFrom<U256>,
	<Runtime as frame_system::Config>::Hash: TryFrom<H256>,
	<Runtime as frame_system::Config>::Call:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::Call: From<ConvictionVotingCall<Runtime>>,
	<<Runtime as pallet_conviction_voting::Config>::Polls as Polling<
		Tally<
			<<Runtime as pallet_conviction_voting::Config>::Currency as Currency<
				<Runtime as frame_system::Config>::AccountId,
			>>::Balance,
			<Runtime as pallet_conviction_voting::Config>::MaxTurnout,
		>,
	>>::Index: TryFrom<u32>,
{
	/// Vote in a poll.
	///
	/// Parameters:
	/// * poll_index: Index of poll
	/// * aye: Yes or no vote
	/// * vote_amount: Balance locked for vote
	/// * conviction: Conviction multiplier for length of vote lock
	#[precompile::public("standardVote(uint256,bool,uint256,uint256)")]
	fn standard_vote(
		handle: &mut impl PrecompileHandle,
		poll_index: SolidityConvert<U256, u32>,
		aye: bool,
		vote_amount: U256,
		conviction: SolidityConvert<U256, u8>,
	) -> EvmResult {
		let poll_index = poll_index.converted();
		let vote_amount = Self::u256_to_amount(vote_amount).in_field("voteAmount")?;

		let conviction: Conviction = conviction.converted().try_into().map_err(|_| {
			RevertReason::custom("Must be an integer between 0 and 6 included")
				.in_field("conviction")
		})?;

		let vote = AccountVote::Standard {
			vote: Vote { aye, conviction },
			balance: vote_amount,
		};

		log::trace!(target: "conviction-voting-precompile",
			"Voting {:?} on poll #{:?}, with conviction {:?}",
			aye, poll_index, conviction
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = ConvictionVotingCall::<Runtime>::vote {
			poll_index: poll_index
				.try_into()
				.map_err(|_| revert("Poll index does not match type"))?,
			vote,
		}
		.into();

		<RuntimeHelper<Runtime>>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}
	// TODO
	// * delegate
	// * undelegate
	// * unlock
	// * remove_vote
	// * remove_other_vote

	fn u256_to_amount(value: U256) -> MayRevert<BalanceOf<Runtime>> {
		value
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").into())
	}
}
