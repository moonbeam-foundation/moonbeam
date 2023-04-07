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

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::{Bounded, ConstU32, Currency, QueryPreimage};
use pallet_democracy::{
	AccountVote, Call as DemocracyCall, Conviction, ReferendumInfo, Vote, VoteThreshold,
};
use pallet_evm::AddressMapping;
use pallet_preimage::Call as PreimageCall;
use precompile_utils::prelude::*;
use sp_core::{H160, H256, U256};
use sp_runtime::traits::{Hash, StaticLookup};
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

pub const ENCODED_PROPOSAL_SIZE_LIMIT: u32 = 2u32.pow(16);
type GetEncodedProposalSizeLimit = ConstU32<ENCODED_PROPOSAL_SIZE_LIMIT>;

/// Solidity selector of the Proposed log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_PROPOSED: [u8; 32] = keccak256!("Proposed(uint32,uint256)");

/// Solidity selector of the Seconded log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_SECONDED: [u8; 32] = keccak256!("Seconded(uint32,address)");

/// Solidity selector of the StandardVote log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_STANDARD_VOTE: [u8; 32] =
	keccak256!("StandardVote(uint32,address,bool,uint256,uint8)");

/// Solidity selector of the Delegated log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_DELEGATED: [u8; 32] = keccak256!("Delegated(address,address)");

/// Solidity selector of the Undelegated log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_UNDELEGATED: [u8; 32] = keccak256!("Undelegated(address)");

/// A precompile to wrap the functionality from pallet democracy.
///
/// Grants evm-based DAOs the right to vote making them first-class citizens.
///
/// For an example of a political party that operates as a DAO, see PoliticalPartyDao.sol
pub struct DemocracyPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
#[precompile::test_concrete_types(mock::Runtime)]
impl<Runtime> DemocracyPrecompile<Runtime>
where
	Runtime: pallet_democracy::Config
		+ pallet_evm::Config
		+ frame_system::Config
		+ pallet_preimage::Config,
	BalanceOf<Runtime>: TryFrom<U256> + TryInto<u128> + Into<U256> + Debug + EvmData,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<DemocracyCall<Runtime>>,
	Runtime::RuntimeCall: From<PreimageCall<Runtime>>,
	Runtime::Hash: From<H256> + Into<H256>,
	Runtime::BlockNumber: Into<U256>,
{
	// The accessors are first. They directly return their result.
	#[precompile::public("publicPropCount()")]
	#[precompile::public("public_prop_count()")]
	#[precompile::view]
	fn public_prop_count(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
		// Fetch data from pallet
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let prop_count = DemocracyOf::<Runtime>::public_prop_count();
		log::trace!(target: "democracy-precompile", "Prop count from pallet is {:?}", prop_count);

		Ok(prop_count.into())
	}

	#[precompile::public("depositOf(uint256)")]
	#[precompile::public("deposit_of(uint256)")]
	#[precompile::view]
	fn deposit_of(
		handle: &mut impl PrecompileHandle,
		prop_index: SolidityConvert<U256, u32>,
	) -> EvmResult<U256> {
		let prop_index = prop_index.converted();

		// Fetch data from pallet
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let deposit = DemocracyOf::<Runtime>::deposit_of(prop_index)
			.ok_or_else(|| revert("No such proposal in pallet democracy"))?
			.1;

		log::trace!(
			target: "democracy-precompile",
			"Deposit of proposal {:?} is {:?}", prop_index, deposit
		);

		Ok(deposit.into())
	}

	#[precompile::public("lowestUnbaked()")]
	#[precompile::public("lowest_unbaked()")]
	#[precompile::view]
	fn lowest_unbaked(handle: &mut impl PrecompileHandle) -> EvmResult<U256> {
		// Fetch data from pallet
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let lowest_unbaked = DemocracyOf::<Runtime>::lowest_unbaked();
		log::trace!(
			target: "democracy-precompile",
			"lowest unbaked referendum is {:?}", lowest_unbaked
		);

		Ok(lowest_unbaked.into())
	}

	#[precompile::public("ongoingReferendumInfo(uint32)")]
	#[precompile::view]
	fn ongoing_referendum_info(
		handle: &mut impl PrecompileHandle,
		ref_index: u32,
	) -> EvmResult<(U256, H256, u8, U256, U256, U256, U256)> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let ref_status = match DemocracyOf::<Runtime>::referendum_info(ref_index) {
			Some(ReferendumInfo::Ongoing(ref_status)) => ref_status,
			Some(ReferendumInfo::Finished { .. }) => Err(revert("Referendum is finished"))?,
			None => Err(revert("Unknown referendum"))?,
		};

		let threshold_u8: u8 = match ref_status.threshold {
			VoteThreshold::SuperMajorityApprove => 0,
			VoteThreshold::SuperMajorityAgainst => 1,
			VoteThreshold::SimpleMajority => 2,
		};

		Ok((
			ref_status.end.into(),
			ref_status.proposal.hash().into(),
			threshold_u8.into(),
			ref_status.delay.into(),
			ref_status.tally.ayes.into(),
			ref_status.tally.nays.into(),
			ref_status.tally.turnout.into(),
		))
	}

	#[precompile::public("finishedReferendumInfo(uint32)")]
	#[precompile::view]
	fn finished_referendum_info(
		handle: &mut impl PrecompileHandle,
		ref_index: u32,
	) -> EvmResult<(bool, U256)> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let (approved, end) = match DemocracyOf::<Runtime>::referendum_info(ref_index) {
			Some(ReferendumInfo::Ongoing(_)) => Err(revert("Referendum is ongoing"))?,
			Some(ReferendumInfo::Finished { approved, end }) => (approved, end),
			None => Err(revert("Unknown referendum"))?,
		};

		Ok((approved, end.into()))
	}

	// The dispatchable wrappers are next. They dispatch a Substrate inner Call.
	#[precompile::public("propose(bytes32,uint256)")]
	fn propose(handle: &mut impl PrecompileHandle, proposal_hash: H256, value: U256) -> EvmResult {
		handle.record_log_costs_manual(2, 32)?;

		// Fetch data from pallet
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let prop_count = DemocracyOf::<Runtime>::public_prop_count();

		let value = Self::u256_to_amount(value).in_field("value")?;

		log::trace!(
			target: "democracy-precompile",
			"Proposing with hash {:?}, and amount {:?}", proposal_hash, value
		);

		// This forces it to have the proposal in pre-images.
		// TODO: REVISIT
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let len = <Runtime as pallet_democracy::Config>::Preimages::len(&proposal_hash).ok_or({
			RevertReason::custom("Failure in preimage fetch").in_field("proposal_hash")
		})?;

		let bounded = Bounded::Lookup::<pallet_democracy::CallOf<Runtime>> {
			hash: proposal_hash,
			len,
		};

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::propose {
			proposal: bounded,
			value,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		log2(
			handle.context().address,
			SELECTOR_LOG_PROPOSED,
			H256::from_low_u64_be(prop_count as u64), // proposal index,
			EvmDataWriter::new().write::<U256>(value.into()).build(),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("second(uint256,uint256)")]
	fn second(
		handle: &mut impl PrecompileHandle,
		prop_index: SolidityConvert<U256, u32>,
		seconds_upper_bound: SolidityConvert<U256, u32>,
	) -> EvmResult {
		handle.record_log_costs_manual(2, 32)?;
		let prop_index = prop_index.converted();
		let seconds_upper_bound = seconds_upper_bound.converted();

		log::trace!(
			target: "democracy-precompile",
			"Seconding proposal {:?}, with bound {:?}", prop_index, seconds_upper_bound
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::second {
			proposal: prop_index,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		log2(
			handle.context().address,
			SELECTOR_LOG_SECONDED,
			H256::from_low_u64_be(prop_index as u64), // proposal index,
			EvmDataWriter::new()
				.write::<Address>(handle.context().caller.into())
				.build(),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("standardVote(uint256,bool,uint256,uint256)")]
	#[precompile::public("standard_vote(uint256,bool,uint256,uint256)")]
	fn standard_vote(
		handle: &mut impl PrecompileHandle,
		ref_index: SolidityConvert<U256, u32>,
		aye: bool,
		vote_amount: U256,
		conviction: SolidityConvert<U256, u8>,
	) -> EvmResult {
		handle.record_log_costs_manual(2, 32 * 4)?;
		let ref_index = ref_index.converted();
		let vote_amount_balance = Self::u256_to_amount(vote_amount).in_field("voteAmount")?;

		let conviction_enum: Conviction =
			conviction.clone().converted().try_into().map_err(|_| {
				RevertReason::custom("Must be an integer between 0 and 6 included")
					.in_field("conviction")
			})?;

		let vote = AccountVote::Standard {
			vote: Vote {
				aye,
				conviction: conviction_enum,
			},
			balance: vote_amount_balance,
		};

		log::trace!(target: "democracy-precompile",
			"Voting {:?} on referendum #{:?}, with conviction {:?}",
			aye, ref_index, conviction_enum
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::vote { ref_index, vote };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		log2(
			handle.context().address,
			SELECTOR_LOG_STANDARD_VOTE,
			H256::from_low_u64_be(ref_index as u64), // referendum index,
			EvmDataWriter::new()
				.write::<Address>(handle.context().caller.into())
				.write::<bool>(aye)
				.write::<U256>(vote_amount)
				.write::<u8>(conviction.converted())
				.build(),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("removeVote(uint256)")]
	#[precompile::public("remove_vote(uint256)")]
	fn remove_vote(
		handle: &mut impl PrecompileHandle,
		ref_index: SolidityConvert<U256, u32>,
	) -> EvmResult {
		let ref_index: u32 = ref_index.converted();

		log::trace!(
			target: "democracy-precompile",
			"Removing vote from referendum {:?}",
			ref_index
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::remove_vote { index: ref_index };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("delegate(address,uint256,uint256)")]
	fn delegate(
		handle: &mut impl PrecompileHandle,
		representative: Address,
		conviction: SolidityConvert<U256, u8>,
		amount: U256,
	) -> EvmResult {
		handle.record_log_costs_manual(2, 32)?;
		let amount = Self::u256_to_amount(amount).in_field("amount")?;

		let conviction: Conviction = conviction.converted().try_into().map_err(|_| {
			RevertReason::custom("Must be an integer between 0 and 6 included")
				.in_field("conviction")
		})?;

		log::trace!(target: "democracy-precompile",
			"Delegating vote to {representative:?} with balance {amount:?} and conviction {conviction:?}",
		);

		let to = Runtime::AddressMapping::into_account_id(representative.into());
		let to: <Runtime::Lookup as StaticLookup>::Source = Runtime::Lookup::unlookup(to.clone());
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::delegate {
			to,
			conviction,
			balance: amount,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		log2(
			handle.context().address,
			SELECTOR_LOG_DELEGATED,
			handle.context().caller,
			EvmDataWriter::new()
				.write::<Address>(representative)
				.build(),
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("unDelegate()")]
	#[precompile::public("un_delegate()")]
	fn un_delegate(handle: &mut impl PrecompileHandle) -> EvmResult {
		handle.record_log_costs_manual(2, 0)?;
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::undelegate {};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		log2(
			handle.context().address,
			SELECTOR_LOG_UNDELEGATED,
			handle.context().caller,
			[],
		)
		.record(handle)?;

		Ok(())
	}

	#[precompile::public("unlock(address)")]
	fn unlock(handle: &mut impl PrecompileHandle, target: Address) -> EvmResult {
		let target: H160 = target.into();
		let target = Runtime::AddressMapping::into_account_id(target);
		let target: <Runtime::Lookup as StaticLookup>::Source =
			Runtime::Lookup::unlookup(target.clone());

		log::trace!(
			target: "democracy-precompile",
			"Unlocking democracy tokens for {:?}", target
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = DemocracyCall::<Runtime>::unlock { target };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("notePreimage(bytes)")]
	#[precompile::public("note_preimage(bytes)")]
	fn note_preimage(
		handle: &mut impl PrecompileHandle,
		encoded_proposal: BoundedBytes<GetEncodedProposalSizeLimit>,
	) -> EvmResult {
		let encoded_proposal: Vec<u8> = encoded_proposal.into();

		log::trace!(
			target: "democracy-precompile",
			"Noting preimage {:?}", encoded_proposal
		);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = PreimageCall::<Runtime>::note_preimage {
			bytes: encoded_proposal.into(),
		};
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("noteImminentPreimage(bytes)")]
	#[precompile::public("note_imminent_preimage(bytes)")]
	fn note_imminent_preimage(
		handle: &mut impl PrecompileHandle,
		encoded_proposal: BoundedBytes<GetEncodedProposalSizeLimit>,
	) -> EvmResult {
		let encoded_proposal: Vec<u8> = encoded_proposal.into();

		log::trace!(
			target: "democracy-precompile",
			"Noting imminent preimage {:?}", encoded_proposal
		);

		// To mimic imminent preimage behavior, we need to check whether the preimage
		// has been requested
		// is_requested implies db read
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let proposal_hash = <Runtime as frame_system::Config>::Hashing::hash(&encoded_proposal);
		if !<<Runtime as pallet_democracy::Config>::Preimages as QueryPreimage>::is_requested(
			&proposal_hash.into(),
		) {
			return Err(revert("not imminent preimage (preimage not requested)"));
		};

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = PreimageCall::<Runtime>::note_preimage {
			bytes: encoded_proposal.into(),
		};
		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	fn u256_to_amount(value: U256) -> MayRevert<BalanceOf<Runtime>> {
		value
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").into())
	}
}
