// Copyright 2019-2021 PureStake Inc.
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

use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::Currency;
use pallet_democracy::{AccountVote, Call as DemocracyCall, Vote};
use pallet_evm::AddressMapping;
use pallet_evm::Precompile;
use precompile_utils::{
	error, Address, Bytes, EvmData, EvmDataReader, EvmDataWriter, EvmResult, Gasometer,
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
#[derive(Debug, PartialEq, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
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
	Runtime: pallet_democracy::Config + pallet_evm::Config,
	BalanceOf<Runtime>: TryFrom<U256> + Debug + EvmData,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<DemocracyCall<Runtime>>,
	Runtime::Hash: From<H256>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		log::trace!(target: "democracy-precompile", "In democracy wrapper");

		let (input, selector) = EvmDataReader::new_with_selector(input)?;

		match selector {
			// Storage Accessors
			Action::PublicPropCount => Self::public_prop_count(target_gas),
			Action::DepositOf => Self::deposit_of(input, target_gas),
			Action::LowestUnbaked => Self::lowest_unbaked(target_gas),
			Action::OngoingReferendumInfo => Self::ongoing_referendum_info(input, target_gas),
			Action::FinishedReferendumInfo => Self::finished_referendum_info(input, target_gas),

			// Dispatchables
			Action::Propose => Self::propose(input, target_gas, context),
			Action::Second => Self::second(input, target_gas, context),
			Action::StandardVote => Self::standard_vote(input, target_gas, context),
			Action::RemoveVote => Self::remove_vote(input, target_gas, context),
			Action::Delegate => Self::delegate(input, target_gas, context),
			Action::UnDelegate => Self::un_delegate(target_gas, context),
			Action::Unlock => Self::unlock(input, target_gas, context),
			Action::NotePreimage => Self::note_preimage(input, target_gas, context),
			Action::NoteImminentPreimage => {
				Self::note_imminent_preimage(input, target_gas, context)
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

	fn public_prop_count(target_gas: Option<u64>) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

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
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(1)?;
		let prop_index: u32 = input.read()?;

		// Fetch data from pallet
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let deposit = DemocracyOf::<Runtime>::deposit_of(prop_index)
			.ok_or_else(|| error("No such proposal in pallet democracy"))?
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

	fn lowest_unbaked(target_gas: Option<u64>) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

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
		mut _input: EvmDataReader,
		_target_gas: Option<u64>,
	) -> EvmResult<PrecompileOutput> {
		Err(error(
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
		mut _input: EvmDataReader,
		_target_gas: Option<u64>,
	) -> EvmResult<PrecompileOutput> {
		Err(error(
			"This method depends on https://github.com/paritytech/substrate/pull/9565",
		))
	}

	// The dispatchable wrappers are next. They dispatch a Substrate inner Call.
	fn propose(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(2)?;

		let proposal_hash = input.read::<H256>()?;
		let amount = input.read::<BalanceOf<Runtime>>()?;

		log::trace!(
			target: "democracy-precompile",
			"Proposing with hash {:?}, and amount {:?}", proposal_hash, amount
		);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::propose(proposal_hash.into(), amount);

		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn second(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(2)?;

		let proposal_index = input.read()?;
		let seconds_upper_bound = input.read()?;

		log::trace!(
			target: "democracy-precompile",
			"Seconding proposal {:?}, with bound {:?}", proposal_index, seconds_upper_bound
		);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::second(proposal_index, seconds_upper_bound);

		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn standard_vote(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(4)?;

		let ref_index = input.read()?;
		let aye = input.read()?;
		let balance = input.read()?;
		let conviction = input
			.read::<u8>()?
			.try_into()
			.map_err(|_| error("Conviction must be an integer in the range 0-6"))?;
		let account_vote = AccountVote::Standard {
			vote: Vote { aye, conviction },
			balance,
		};

		log::trace!(target: "democracy-precompile",
			"Voting {:?} on referendum #{:?}, with conviction {:?}",
			aye, ref_index, conviction
		);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::vote(ref_index, account_vote);

		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn remove_vote(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(1)?;

		let ref_index = input.read()?;

		log::trace!(target: "democracy-precompile", "Removing vote from referendum {:?}", ref_index);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::remove_vote(ref_index);

		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn delegate(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(3)?;

		let to_address: H160 = input.read::<Address>()?.into();
		let to_account = Runtime::AddressMapping::into_account_id(to_address);
		let conviction = input
			.read::<u8>()?
			.try_into()
			.map_err(|_| error("Conviction must be an integer in the range 0-6"))?;
		let balance = input.read()?;

		log::trace!(target: "democracy-precompile",
			"Delegating vote to {:?} with balance {:?} and {:?}",
			to_account, conviction, balance
		);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::delegate(to_account, conviction, balance);

		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn un_delegate(target_gas: Option<u64>, context: &Context) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::undelegate();

		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn unlock(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		// Bound check
		input.expect_arguments(1)?;

		let target_address: H160 = input.read::<Address>()?.into();
		let target_account = Runtime::AddressMapping::into_account_id(target_address);

		log::trace!(
			target: "democracy-precompile",
			"Unlocking democracy tokens for {:?}", target_account
		);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::unlock(target_account);

		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn note_preimage(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		let encoded_proposal: Vec<u8> = input.read::<Bytes>()?.into();

		log::trace!(
			target: "democracy-precompile",
			"Noting preimage {:?}", encoded_proposal
		);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::note_preimage(encoded_proposal);

		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn note_imminent_preimage(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		let encoded_proposal: Vec<u8> = input.read::<Bytes>()?.into();

		log::trace!(
			target: "democracy-precompile",
			"Noting imminent preimage {:?}", encoded_proposal
		);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = DemocracyCall::<Runtime>::note_imminent_preimage(encoded_proposal);

		let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
			Some(origin).into(),
			call,
			gasometer.remaining_gas()?,
		)?;
		gasometer.record_cost(used_gas)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}
}
