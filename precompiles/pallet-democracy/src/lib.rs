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

use fp_evm::{Context, ExitSucceed, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::Currency;
use pallet_democracy::{AccountVote, Call as DemocracyCall, Vote};
use pallet_evm::AddressMapping;
use pallet_evm::Precompile;
use precompile_utils::{
	Address, Bytes, EvmData, EvmDataReader, EvmDataWriter, EvmResult, FunctionModifier, Gasometer,
	RuntimeHelper,
};
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

#[precompile_utils::generate_function_selector]
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

impl<Runtime> Precompile for DemocracyWrapper<Runtime>
where
	Runtime: pallet_democracy::Config + pallet_evm::Config + frame_system::Config,
	BalanceOf<Runtime>: TryFrom<U256> + TryInto<u128> + Debug + EvmData,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<DemocracyCall<Runtime>>,
	Runtime::Hash: From<H256>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> EvmResult<PrecompileOutput> {
		log::trace!(target: "democracy-precompile", "In democracy wrapper");

		let mut gasometer = Gasometer::new(target_gas);
		let gasometer = &mut gasometer;

		let (mut input, selector) = EvmDataReader::new_with_selector(gasometer, input)?;
		let input = &mut input;

		gasometer.check_function_modifier(
			context,
			is_static,
			match selector {
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
			},
		)?;

		match selector {
			// Storage Accessors
			Action::PublicPropCount => Self::public_prop_count(gasometer),
			Action::DepositOf => Self::deposit_of(input, gasometer),
			Action::LowestUnbaked => Self::lowest_unbaked(gasometer),
			Action::OngoingReferendumInfo => Self::ongoing_referendum_info(input, gasometer),
			Action::FinishedReferendumInfo => Self::finished_referendum_info(input, gasometer),

			// Dispatchables
			Action::Propose => Self::propose(input, gasometer, context),
			Action::Second => Self::second(input, gasometer, context),
			Action::StandardVote => Self::standard_vote(input, gasometer, context),
			Action::RemoveVote => Self::remove_vote(input, gasometer, context),
			Action::Delegate => Self::delegate(input, gasometer, context),
			Action::UnDelegate => Self::un_delegate(gasometer, context),
			Action::Unlock => Self::unlock(input, gasometer, context),
			Action::NotePreimage => Self::note_preimage(input, gasometer, context),
			Action::NoteImminentPreimage => Self::note_imminent_preimage(input, gasometer, context),
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

	fn public_prop_count(gasometer: &mut Gasometer) -> EvmResult<PrecompileOutput> {
		// Fetch data from pallet
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let prop_count = DemocracyOf::<Runtime>::public_prop_count();
		log::trace!(target: "democracy-precompile", "Prop count from pallet is {:?}", prop_count);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(prop_count).build(),
			logs: Default::default(),
		})
	}

	fn deposit_of(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		// Bound check
		input.expect_arguments(gasometer, 1)?;
		let prop_index: u32 = input.read(gasometer)?;

		// Fetch data from pallet
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let deposit = DemocracyOf::<Runtime>::deposit_of(prop_index)
			.ok_or_else(|| gasometer.revert("No such proposal in pallet democracy"))?
			.1;

		log::trace!(
			target: "democracy-precompile",
			"Deposit of proposal {:?} is {:?}", prop_index, deposit
		);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(deposit).build(),
			logs: Default::default(),
		})
	}

	fn lowest_unbaked(gasometer: &mut Gasometer) -> EvmResult<PrecompileOutput> {
		// Fetch data from pallet
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let lowest_unbaked = DemocracyOf::<Runtime>::lowest_unbaked();
		log::trace!(
			target: "democracy-precompile",
			"lowest unbaked referendum is {:?}", lowest_unbaked
		);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(lowest_unbaked).build(),
			logs: Default::default(),
		})
	}

	// This method is not yet implemented because it depends on
	// https://github.com/paritytech/substrate/pull/9565 which has been merged into Substrate
	// master, but is not on the release branches that we are following
	fn ongoing_referendum_info(
		_input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		Err(gasometer
			.revert("This method depends on https://github.com/paritytech/substrate/pull/9565"))
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
		_input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		Err(gasometer
			.revert("This method depends on https://github.com/paritytech/substrate/pull/9565"))
	}

	// The dispatchable wrappers are next. They dispatch a Substrate inner Call.
	fn propose(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check
		input.expect_arguments(gasometer, 2)?;

		let proposal_hash = input.read::<H256>(gasometer)?.into();
		let amount = input.read::<BalanceOf<Runtime>>(gasometer)?;

		log::trace!(
			target: "democracy-precompile",
			"Proposing with hash {:?}, and amount {:?}", proposal_hash, amount
		);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::propose {
			proposal_hash,
			value: amount,
		};

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn second(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check
		input.expect_arguments(gasometer, 2)?;

		let proposal = input.read(gasometer)?;
		let seconds_upper_bound = input.read(gasometer)?;

		log::trace!(
			target: "democracy-precompile",
			"Seconding proposal {:?}, with bound {:?}", proposal, seconds_upper_bound
		);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::second {
			proposal,
			seconds_upper_bound,
		};

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn standard_vote(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check
		input.expect_arguments(gasometer, 4)?;

		let ref_index = input.read(gasometer)?;
		let aye = input.read(gasometer)?;
		let balance = input.read(gasometer)?;
		let conviction = input
			.read::<u8>(gasometer)?
			.try_into()
			.map_err(|_| gasometer.revert("Conviction must be an integer in the range 0-6"))?;
		let vote = AccountVote::Standard {
			vote: Vote { aye, conviction },
			balance,
		};

		log::trace!(target: "democracy-precompile",
			"Voting {:?} on referendum #{:?}, with conviction {:?}",
			aye, ref_index, conviction
		);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::vote { ref_index, vote };

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn remove_vote(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check
		input.expect_arguments(gasometer, 1)?;

		let referendum_index = input.read(gasometer)?;

		log::trace!(
			target: "democracy-precompile",
			"Removing vote from referendum {:?}",
			referendum_index
		);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::remove_vote {
			index: referendum_index,
		};

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn delegate(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check
		input.expect_arguments(gasometer, 3)?;

		let to: H160 = input.read::<Address>(gasometer)?.into();
		let to = Runtime::AddressMapping::into_account_id(to);
		let conviction = input
			.read::<u8>(gasometer)?
			.try_into()
			.map_err(|_| gasometer.revert("Conviction must be an integer in the range 0-6"))?;
		let balance = input.read(gasometer)?;

		log::trace!(target: "democracy-precompile",
			"Delegating vote to {:?} with balance {:?} and {:?}",
			to, conviction, balance
		);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::delegate {
			to,
			conviction,
			balance,
		};

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn un_delegate(gasometer: &mut Gasometer, context: &Context) -> EvmResult<PrecompileOutput> {
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::undelegate {};

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn unlock(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check
		input.expect_arguments(gasometer, 1)?;

		let target: H160 = input.read::<Address>(gasometer)?.into();
		let target = Runtime::AddressMapping::into_account_id(target);

		log::trace!(
			target: "democracy-precompile",
			"Unlocking democracy tokens for {:?}", target
		);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::unlock { target };

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn note_preimage(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let encoded_proposal: Vec<u8> = input.read::<Bytes>(gasometer)?.into();

		log::trace!(
			target: "democracy-precompile",
			"Noting preimage {:?}", encoded_proposal
		);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::note_preimage { encoded_proposal };

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn note_imminent_preimage(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let encoded_proposal: Vec<u8> = input.read::<Bytes>(gasometer)?.into();

		log::trace!(
			target: "democracy-precompile",
			"Noting imminent preimage {:?}", encoded_proposal
		);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::note_imminent_preimage { encoded_proposal };

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}
}
