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
use frame_support::traits::Currency;
use pallet_democracy::{AccountVote, Call as DemocracyCall, Vote};
use pallet_evm::{AddressMapping, Precompile};
use precompile_utils::prelude::*;
use sp_core::{H160, H256, U256};
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker::PhantomData,
	vec::Vec,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type BalanceOf<Runtime> = <<Runtime as pallet_democracy::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

type DemocracyOf<Runtime> = pallet_democracy::Pallet<Runtime>;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	PublicPropCount = "public_prop_count()",
	DepositOf = "deposit_of(uint256)",
	LowestUnbaked = "lowest_unbaked()",
	OngoingReferendumInfo = "ongoing_referendum_info(uint256)",
	FinishedReferendumInfo = "finished_referendum_info(uint256)",
	Propose = "propose(bytes32,uint256)",
	Second = "second(uint256,uint256)",
	StandardVote = "standard_vote(uint256,bool,uint256,uint256)",
	RemoveVote = "remove_vote(uint256)",
	Delegate = "delegate(address,uint256,uint256)",
	UnDelegate = "un_delegate()",
	Unlock = "unlock(address)",
	NotePreimage = "note_preimage(bytes)",
	NoteImminentPreimage = "note_imminent_preimage(bytes)",
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
			| Action::NoteImminentPreimage => FunctionModifier::NonPayable,
			_ => FunctionModifier::View,
		})?;

		match selector {
			// Storage Accessors
			Action::PublicPropCount => Self::public_prop_count(handle),
			Action::DepositOf => Self::deposit_of(handle),
			Action::LowestUnbaked => Self::lowest_unbaked(handle),
			Action::OngoingReferendumInfo => Self::ongoing_referendum_info(handle),
			Action::FinishedReferendumInfo => Self::finished_referendum_info(handle),

			// Dispatchables
			Action::Propose => Self::propose(handle),
			Action::Second => Self::second(handle),
			Action::StandardVote => Self::standard_vote(handle),
			Action::RemoveVote => Self::remove_vote(handle),
			Action::Delegate => Self::delegate(handle),
			Action::UnDelegate => Self::un_delegate(handle),
			Action::Unlock => Self::unlock(handle),
			Action::NotePreimage => Self::note_preimage(handle),
			Action::NoteImminentPreimage => Self::note_imminent_preimage(handle),
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
		let mut input = handle.read_input()?;
		// Bound check
		input.expect_arguments(1)?;
		let prop_index: u32 = input.read()?;

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
		let mut input = handle.read_input()?;
		// Bound check
		input.expect_arguments(2)?;

		let proposal_hash = input.read::<H256>()?.into();
		let amount = input.read::<BalanceOf<Runtime>>()?;

		log::trace!(
			target: "democracy-precompile",
			"Proposing with hash {:?}, and amount {:?}", proposal_hash, amount
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::propose {
			proposal_hash,
			value: amount,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn second(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		// Bound check
		input.expect_arguments(2)?;

		let proposal = input.read()?;
		let seconds_upper_bound = input.read()?;

		log::trace!(
			target: "democracy-precompile",
			"Seconding proposal {:?}, with bound {:?}", proposal, seconds_upper_bound
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::second {
			proposal,
			seconds_upper_bound,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn standard_vote(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		// Bound check
		input.expect_arguments(4)?;

		let ref_index = input.read()?;
		let aye = input.read()?;
		let balance = input.read()?;
		let conviction = input
			.read::<u8>()?
			.try_into()
			.map_err(|_| revert("Conviction must be an integer in the range 0-6"))?;
		let vote = AccountVote::Standard {
			vote: Vote { aye, conviction },
			balance,
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
		let mut input = handle.read_input()?;
		// Bound check
		input.expect_arguments(1)?;

		let referendum_index = input.read()?;

		log::trace!(
			target: "democracy-precompile",
			"Removing vote from referendum {:?}",
			referendum_index
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::remove_vote {
			index: referendum_index,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn delegate(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		// Bound check
		input.expect_arguments(3)?;

		let to: H160 = input.read::<Address>()?.into();
		let to = Runtime::AddressMapping::into_account_id(to);
		let conviction = input
			.read::<u8>()?
			.try_into()
			.map_err(|_| revert("Conviction must be an integer in the range 0-6"))?;
		let balance = input.read()?;

		log::trace!(target: "democracy-precompile",
			"Delegating vote to {:?} with balance {:?} and {:?}",
			to, conviction, balance
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::delegate {
			to,
			conviction,
			balance,
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
		let mut input = handle.read_input()?;
		// Bound check
		input.expect_arguments(1)?;

		let target: H160 = input.read::<Address>()?.into();
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
		let mut input = handle.read_input()?;
		let encoded_proposal: Vec<u8> = input.read::<Bytes>()?.into();

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
		let mut input = handle.read_input()?;
		let encoded_proposal: Vec<u8> = input.read::<Bytes>()?.into();

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
