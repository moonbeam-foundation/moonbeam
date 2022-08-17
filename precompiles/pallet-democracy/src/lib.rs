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

//! Precompile to interact with pallet democracy through an evm precompile.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(assert_matches)]

use fp_evm::{PrecompileHandle, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::{ConstU32, Currency};
use pallet_democracy::{AccountVote, Call as DemocracyCall, Conviction, Vote};
use pallet_evm::{AddressMapping, Precompile};
use precompile_utils::prelude::*;
use sp_core::{H160, H256, U256};
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker::PhantomData,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type BalanceOf<Runtime> = <<Runtime as pallet_democracy::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

type DemocracyOf<Runtime> = pallet_democracy::Pallet<Runtime>;

pub const ENCODED_PROPOSAL_SIZE_LIMIT: u32 = 2u32.pow(16);
type GetEncodedProposalSizeLimit = ConstU32<ENCODED_PROPOSAL_SIZE_LIMIT>;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	PublicPropCount = "publicPropCount()",
	DepositOf = "depositOf(uint256)",
	LowestUnbaked = "lowestUnbaked()",
	OngoingReferendumInfo = "ongoingReferendumInfo(uint256)",
	FinishedReferendumInfo = "finishedReferendumInfo(uint256)",
	Propose = "propose(bytes32,uint256)",
	Second = "second(uint256,uint256)",
	StandardVote = "standardVote(uint256,bool,uint256,uint256)",
	RemoveVote = "removeVote(uint256)",
	Delegate = "delegate(address,uint256,uint256)",
	UnDelegate = "unDelegate()",
	Unlock = "unlock(address)",
	NotePreimage = "notePreimage(bytes)",
	NoteImminentPreimage = "noteImminentPreimage(bytes)",

	// deprecated
	DeprecatedPublicPropCount = "public_prop_count()",
	DeprecatedDepositOf = "deposit_of(uint256)",
	DeprecatedLowestUnbaked = "lowest_unbaked()",
	DeprecatedOngoingReferendumInfo = "ongoing_referendum_info(uint256)",
	DeprecatedFinishedReferendumInfo = "finished_referendum_info(uint256)",
	DeprecatedStandardVote = "standard_vote(uint256,bool,uint256,uint256)",
	DeprecatedRemoveVote = "remove_vote(uint256)",
	DeprecatedUnDelegate = "un_delegate()",
	DeprecatedNotePreimage = "note_preimage(bytes)",
	DeprecatedNoteImminentPreimage = "note_imminent_preimage(bytes)",
}

/// A precompile to wrap the functionality from pallet democracy.
///
/// Grants evm-based DAOs the right to vote making them first-class citizens.
///
/// For an example of a political party that operates as a DAO, see PoliticalPartyDao.sol
pub struct DemocracyWrapper<Runtime>(PhantomData<Runtime>);

// TODO: Migrate to precompile_utils::Precompile.
impl<Runtime> Precompile for DemocracyWrapper<Runtime>
where
	Runtime: pallet_democracy::Config + pallet_evm::Config + frame_system::Config,
	BalanceOf<Runtime>: TryFrom<U256> + TryInto<u128> + Debug + EvmData,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<DemocracyCall<Runtime>>,
	Runtime::Hash: From<H256>,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		log::trace!(target: "democracy-precompile", "In democracy wrapper");

		let selector = handle.read_selector()?;

		handle.check_function_modifier(match selector {
			Action::Propose
			| Action::Second
			| Action::StandardVote
			| Action::RemoveVote
			| Action::Delegate
			| Action::UnDelegate
			| Action::Unlock
			| Action::NotePreimage
			| Action::NoteImminentPreimage
			| Action::DeprecatedStandardVote
			| Action::DeprecatedRemoveVote
			| Action::DeprecatedUnDelegate
			| Action::DeprecatedNotePreimage
			| Action::DeprecatedNoteImminentPreimage => FunctionModifier::NonPayable,
			_ => FunctionModifier::View,
		})?;

		match selector {
			// Storage Accessors
			Action::PublicPropCount | Action::DeprecatedPublicPropCount => {
				Self::public_prop_count(handle)
			}
			Action::DepositOf | Action::DeprecatedDepositOf => Self::deposit_of(handle),
			Action::LowestUnbaked | Action::DeprecatedLowestUnbaked => Self::lowest_unbaked(handle),
			Action::OngoingReferendumInfo | Action::DeprecatedOngoingReferendumInfo => {
				Self::ongoing_referendum_info(handle)
			}
			Action::FinishedReferendumInfo | Action::DeprecatedFinishedReferendumInfo => {
				Self::finished_referendum_info(handle)
			}

			// Dispatchables
			Action::Propose => Self::propose(handle),
			Action::Second => Self::second(handle),
			Action::StandardVote | Action::DeprecatedStandardVote => Self::standard_vote(handle),
			Action::RemoveVote | Action::DeprecatedRemoveVote => Self::remove_vote(handle),
			Action::Delegate => Self::delegate(handle),
			Action::UnDelegate | Action::DeprecatedUnDelegate => Self::un_delegate(handle),
			Action::Unlock => Self::unlock(handle),
			Action::NotePreimage | Action::DeprecatedNotePreimage => Self::note_preimage(handle),
			Action::NoteImminentPreimage | Action::DeprecatedNoteImminentPreimage => {
				Self::note_imminent_preimage(handle)
			}
		}
	}
}

impl<Runtime> DemocracyWrapper<Runtime>
where
	Runtime: pallet_democracy::Config + pallet_evm::Config + frame_system::Config,
	BalanceOf<Runtime>: TryFrom<U256> + TryInto<u128> + Debug + EvmData,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<DemocracyCall<Runtime>>,
	Runtime::Hash: From<H256>,
{
	// The accessors are first. They directly return their result.
	fn public_prop_count(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		// Fetch data from pallet
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let prop_count = DemocracyOf::<Runtime>::public_prop_count();
		log::trace!(target: "democracy-precompile", "Prop count from pallet is {:?}", prop_count);

		Ok(succeed(EvmDataWriter::new().write(prop_count).build()))
	}

	fn deposit_of(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { prop_index: u32 });

		// Fetch data from pallet
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let deposit = DemocracyOf::<Runtime>::deposit_of(prop_index)
			.ok_or_else(|| revert("No such proposal in pallet democracy"))?
			.1;

		log::trace!(
			target: "democracy-precompile",
			"Deposit of proposal {:?} is {:?}", prop_index, deposit
		);

		Ok(succeed(EvmDataWriter::new().write(deposit).build()))
	}

	fn lowest_unbaked(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		// Fetch data from pallet
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let lowest_unbaked = DemocracyOf::<Runtime>::lowest_unbaked();
		log::trace!(
			target: "democracy-precompile",
			"lowest unbaked referendum is {:?}", lowest_unbaked
		);

		Ok(succeed(EvmDataWriter::new().write(lowest_unbaked).build()))
	}

	// This method is not yet implemented because it depends on
	// https://github.com/paritytech/substrate/pull/9565 which has been merged into Substrate
	// master, but is not on the release branches that we are following
	fn ongoing_referendum_info(_handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		Err(revert(
			"This method depends on https://github.com/paritytech/substrate/pull/9565",
		))
		// let mut gasometer = Gasometer::new(target_gas);

		// // Bound check
		// input.expect_arguments(1)?;
		// let ref_index: u32 = input.read()?;

		// // Fetch data from pallet
		// gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		// let ref_status = match DemocracyOf::<Runtime>::referendum_info(ref_index) {
		// 	Some(ReferendumInfo::Ongoing(ref_status)) => ref_status,
		// 	Some(ReferendumInfo::Finished{..}) => Err(error("Referendum is finished"))?,
		// 	None => Err(error("failed to get ongoing (or finished for that matter) referendum"))?,
		// };
		// log::trace!(
		// 	target: "democracy-precompile",
		// 	"Ongoing Referendum info for ref {:?} is {:?}", ref_index, ref_status
		// );

		// // Write data
		// //TODO woof, between private fields and generic types, this is pretty complicated
		// let threshold_u8: u8 = match ref_status.threshold {
		// 	VoteThreshold::SuperMajorityApprove => 0,
		// 	VoteThreshold::SuperMajorityAgainst => 1,
		// 	VoteThreshold::SimpleMajority => 2,
		// };

		// let output = EvmDataWriter::new()
		// 	.write(ref_status.end)
		// 	.write(ref_status.proposal_hash)
		// 	.write(threshold_u8)
		// 	.write(ref_status.delay)
		// 	.write(ref_status.tally.ayes)
		// 	.write(ref_status.tally.nays)
		// 	.write(ref_status.tally.turnout);

		// Ok(PrecompileOutput {
		// 	exit_status: ExitSucceed::Returned,
		// 	cost: gasometer.used_gas(),
		// 	output: output.build(),
		// 	logs: Default::default(),
		// })
	}

	// This method is not yet implemented because it depends on
	// https://github.com/paritytech/substrate/pull/9565 which has been merged into Substrate
	// master, but is not on the release branches that we are following
	fn finished_referendum_info(
		_handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		Err(revert(
			"This method depends on https://github.com/paritytech/substrate/pull/9565",
		))
	}

	// The dispatchable wrappers are next. They dispatch a Substrate inner Call.
	fn propose(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {proposal_hash: H256, value: BalanceOf<Runtime>});
		let proposal_hash = proposal_hash.into();

		log::trace!(
			target: "democracy-precompile",
			"Proposing with hash {:?}, and amount {:?}", proposal_hash, value
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::propose {
			proposal_hash,
			value,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn second(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		// substrate arguments are u32 but Solidity one are uint256.
		// We parse as uint32 to properly reject overflowing values.
		read_args!(handle, {prop_index: u32, seconds_upper_bound: u32});

		log::trace!(
			target: "democracy-precompile",
			"Seconding proposal {:?}, with bound {:?}", prop_index, seconds_upper_bound
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::second {
			proposal: prop_index,
			seconds_upper_bound,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn standard_vote(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			ref_index: u32,
			aye: bool,
			vote_amount: BalanceOf<Runtime>,
			conviction: u8
		});
		let conviction: Conviction = conviction.try_into().map_err(|_| {
			RevertReason::custom("Must be an integer between 0 and 6 included")
				.in_field("conviction")
		})?;

		let vote = AccountVote::Standard {
			vote: Vote { aye, conviction },
			balance: vote_amount,
		};

		log::trace!(target: "democracy-precompile",
			"Voting {:?} on referendum #{:?}, with conviction {:?}",
			aye, ref_index, conviction
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::vote { ref_index, vote };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn remove_vote(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { ref_index: u32 });

		log::trace!(
			target: "democracy-precompile",
			"Removing vote from referendum {:?}",
			ref_index
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::remove_vote { index: ref_index };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn delegate(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			representative: Address,
			conviction: u8,
			amount: BalanceOf<Runtime>
		});
		let conviction: Conviction = conviction.try_into().map_err(|_| {
			RevertReason::custom("Must be an integer between 0 and 6 included")
				.in_field("conviction")
		})?;

		let to = Runtime::AddressMapping::into_account_id(representative.into());

		log::trace!(target: "democracy-precompile",
			"Delegating vote to {:?} with balance {:?} and {:?}",
			to, conviction, amount
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::delegate {
			to,
			conviction,
			balance: amount,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn un_delegate(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::undelegate {};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn unlock(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { target: Address });
		let target: H160 = target.into();
		let target = Runtime::AddressMapping::into_account_id(target);

		log::trace!(
			target: "democracy-precompile",
			"Unlocking democracy tokens for {:?}", target
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::unlock { target };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn note_preimage(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			encoded_proposal: BoundedBytes<GetEncodedProposalSizeLimit>
		});
		let encoded_proposal = encoded_proposal.into_vec();

		log::trace!(
			target: "democracy-precompile",
			"Noting preimage {:?}", encoded_proposal
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::note_preimage { encoded_proposal };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn note_imminent_preimage(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			encoded_proposal: BoundedBytes<GetEncodedProposalSizeLimit>
		});
		let encoded_proposal = encoded_proposal.into_vec();

		log::trace!(
			target: "democracy-precompile",
			"Noting imminent preimage {:?}", encoded_proposal
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::note_imminent_preimage { encoded_proposal };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}
}
