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

//! Precompile to interact with pallet_collective instances.

#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use fp_evm::Log;
use frame_support::{
	dispatch::Dispatchable,
	dispatch::{GetDispatchInfo, Pays, PostDispatchInfo},
	sp_runtime::traits::Hash,
	traits::ConstU32,
	weights::Weight,
};
use pallet_evm::AddressMapping;
use parity_scale_codec::DecodeLimit as _;
use precompile_utils::prelude::*;
use sp_core::{Decode, Get, H160, H256};
use sp_std::{boxed::Box, vec::Vec};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Solidity selector of the Executed log.
pub const SELECTOR_LOG_EXECUTED: [u8; 32] = keccak256!("Executed(bytes32)");

/// Solidity selector of the Proposed log.
pub const SELECTOR_LOG_PROPOSED: [u8; 32] = keccak256!("Proposed(address,uint32,bytes32,uint32)");

/// Solidity selector of the Voted log.
pub const SELECTOR_LOG_VOTED: [u8; 32] = keccak256!("Voted(address,bytes32,bool)");

/// Solidity selector of the Closed log.
pub const SELECTOR_LOG_CLOSED: [u8; 32] = keccak256!("Closed(bytes32)");

pub fn log_executed(address: impl Into<H160>, hash: H256) -> Log {
	log2(address.into(), SELECTOR_LOG_EXECUTED, hash, Vec::new())
}

pub fn log_proposed(
	address: impl Into<H160>,
	who: impl Into<H160>,
	index: u32,
	hash: H256,
	threshold: u32,
) -> Log {
	log4(
		address.into(),
		SELECTOR_LOG_PROPOSED,
		who.into(),
		H256::from_slice(&EvmDataWriter::new().write(index).build()),
		hash,
		EvmDataWriter::new().write(threshold).build(),
	)
}

pub fn log_voted(address: impl Into<H160>, who: impl Into<H160>, hash: H256, voted: bool) -> Log {
	log3(
		address.into(),
		SELECTOR_LOG_VOTED,
		who.into(),
		hash,
		EvmDataWriter::new().write(voted).build(),
	)
}

pub fn log_closed(address: impl Into<H160>, hash: H256) -> Log {
	log2(address.into(), SELECTOR_LOG_CLOSED, hash, Vec::new())
}

type GetProposalLimit = ConstU32<{ 2u32.pow(16) }>;
type DecodeLimit = ConstU32<8>;

pub struct CollectivePrecompile<Runtime, Instance: 'static>(PhantomData<(Runtime, Instance)>);

#[precompile_utils::precompile]
impl<Runtime, Instance> CollectivePrecompile<Runtime, Instance>
where
	Instance: 'static,
	Runtime: pallet_collective::Config<Instance> + pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + Decode,
	Runtime::RuntimeCall: From<pallet_collective::Call<Runtime, Instance>>,
	<Runtime as pallet_collective::Config<Instance>>::Proposal: From<Runtime::RuntimeCall>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::AccountId: Into<H160>,
	H256: From<<Runtime as frame_system::Config>::Hash>
		+ Into<<Runtime as frame_system::Config>::Hash>,
{
	#[precompile::public("execute(bytes)")]
	fn execute(
		handle: &mut impl PrecompileHandle,
		proposal: BoundedBytes<GetProposalLimit>,
	) -> EvmResult {
		let proposal: Vec<_> = proposal.into();
		let proposal_hash: H256 = hash::<Runtime>(&proposal);

		let log = log_executed(handle.context().address, proposal_hash);
		handle.record_log_costs(&[&log])?;

		let proposal_length: u32 = proposal.len().try_into().map_err(|_| {
			RevertReason::value_is_too_large("uint32")
				.in_field("length")
				.in_field("proposal")
		})?;

		let proposal =
			Runtime::RuntimeCall::decode_with_depth_limit(DecodeLimit::get(), &mut &*proposal)
				.map_err(|_| {
					RevertReason::custom("Failed to decode proposal").in_field("proposal")
				})?
				.into();
		let proposal = Box::new(proposal);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			pallet_collective::Call::<Runtime, Instance>::execute {
				proposal,
				length_bound: proposal_length,
			},
		)?;

		log.record(handle)?;

		Ok(())
	}

	#[precompile::public("propose(uint32,bytes)")]
	fn propose(
		handle: &mut impl PrecompileHandle,
		threshold: u32,
		proposal: BoundedBytes<GetProposalLimit>,
	) -> EvmResult<u32> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let proposal: Vec<_> = proposal.into();
		let proposal_length: u32 = proposal.len().try_into().map_err(|_| {
			RevertReason::value_is_too_large("uint32")
				.in_field("length")
				.in_field("proposal")
		})?;

		let proposal_index = pallet_collective::Pallet::<Runtime, Instance>::proposal_count();
		let proposal_hash: H256 = hash::<Runtime>(&proposal);

		// In pallet_collective a threshold < 2 means the proposal has been
		// executed directly.
		let log = if threshold < 2 {
			log_executed(handle.context().address, proposal_hash)
		} else {
			log_proposed(
				handle.context().address,
				handle.context().caller,
				proposal_index,
				proposal_hash,
				threshold,
			)
		};

		handle.record_log_costs(&[&log])?;

		let proposal =
			Runtime::RuntimeCall::decode_with_depth_limit(DecodeLimit::get(), &mut &*proposal)
				.map_err(|_| {
					RevertReason::custom("Failed to decode proposal").in_field("proposal")
				})?
				.into();
		let proposal = Box::new(proposal);

		{
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_collective::Call::<Runtime, Instance>::propose {
					threshold,
					proposal,
					length_bound: proposal_length,
				},
			)?;
		}

		log.record(handle)?;

		Ok(proposal_index)
	}

	#[precompile::public("vote(bytes32,uint32,bool)")]
	fn vote(
		handle: &mut impl PrecompileHandle,
		proposal_hash: H256,
		proposal_index: u32,
		approve: bool,
	) -> EvmResult {
		// TODO: Since we cannot access ayes/nays of a proposal we cannot
		// include it in the EVM events to mirror Substrate events.
		let log = log_voted(
			handle.context().address,
			handle.context().caller,
			proposal_hash,
			approve,
		);
		handle.record_log_costs(&[&log])?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			pallet_collective::Call::<Runtime, Instance>::vote {
				proposal: proposal_hash.into(),
				index: proposal_index,
				approve,
			},
		)?;

		log.record(handle)?;

		Ok(())
	}

	#[precompile::public("close(bytes32,uint32,uint64,uint32)")]
	fn close(
		handle: &mut impl PrecompileHandle,
		proposal_hash: H256,
		proposal_index: u32,
		proposal_weight_bound: u64,
		length_bound: u32,
	) -> EvmResult<bool> {
		// Because the actual log cannot be built before dispatch, we manually
		// record it first (`executed` and `closed` have the same cost).
		handle.record_log_costs_manual(2, 0)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let post_dispatch_info = RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			pallet_collective::Call::<Runtime, Instance>::close {
				proposal_hash: proposal_hash.into(),
				index: proposal_index,
				proposal_weight_bound: Weight::from_ref_time(proposal_weight_bound),
				length_bound,
			},
		)?;

		// We can know if the proposal was executed or not based on the `pays_fee` in
		// `PostDispatchInfo`.
		let (executed, log) = match post_dispatch_info.pays_fee {
			Pays::Yes => (true, log_executed(handle.context().address, proposal_hash)),
			Pays::No => (false, log_closed(handle.context().address, proposal_hash)),
		};
		log.record(handle)?;

		Ok(executed)
	}

	#[precompile::public("proposalHash(bytes)")]
	#[precompile::view]
	fn proposal_hash(
		_handle: &mut impl PrecompileHandle,
		proposal: BoundedBytes<GetProposalLimit>,
	) -> EvmResult<H256> {
		let proposal: Vec<_> = proposal.into();
		let hash = hash::<Runtime>(&proposal);

		Ok(hash)
	}

	#[precompile::public("proposals()")]
	#[precompile::view]
	fn proposals(handle: &mut impl PrecompileHandle) -> EvmResult<Vec<H256>> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let proposals = pallet_collective::Pallet::<Runtime, Instance>::proposals();
		let proposals: Vec<_> = proposals.into_iter().map(|hash| hash.into()).collect();

		Ok(proposals)
	}

	#[precompile::public("members()")]
	#[precompile::view]
	fn members(handle: &mut impl PrecompileHandle) -> EvmResult<Vec<Address>> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let members = pallet_collective::Pallet::<Runtime, Instance>::members();
		let members: Vec<_> = members.into_iter().map(|id| Address(id.into())).collect();

		Ok(members)
	}

	#[precompile::public("isMember(address)")]
	#[precompile::view]
	fn is_member(handle: &mut impl PrecompileHandle, account: Address) -> EvmResult<bool> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let account = Runtime::AddressMapping::into_account_id(account.into());

		let is_member = pallet_collective::Pallet::<Runtime, Instance>::is_member(&account);

		Ok(is_member)
	}

	#[precompile::public("prime()")]
	#[precompile::view]
	fn prime(handle: &mut impl PrecompileHandle) -> EvmResult<Address> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let prime = pallet_collective::Pallet::<Runtime, Instance>::prime()
			.map(|prime| prime.into())
			.unwrap_or(H160::zero());

		Ok(Address(prime))
	}
}

pub fn hash<Runtime>(data: &[u8]) -> H256
where
	Runtime: frame_system::Config,
	H256: From<<Runtime as frame_system::Config>::Hash>,
{
	<Runtime as frame_system::Config>::Hashing::hash(data).into()
}
