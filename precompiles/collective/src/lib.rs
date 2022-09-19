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
use fp_evm::{Log, Precompile, PrecompileOutput};
use frame_support::{
	dispatch::Dispatchable,
	sp_runtime::traits::Hash,
	traits::ConstU32,
	weights::{GetDispatchInfo, Pays, PostDispatchInfo},
};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{Decode, H160, H256};
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

#[generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	Execute = "execute(bytes)",
	Propose = "propose(uint32,bytes)",
	Vote = "vote(bytes32,uint32,bool)",
	Close = "close(bytes32,uint32,uint64,uint32)",
	ProposalHash = "proposalHash(bytes)",
}

pub struct CollectivePrecompile<Runtime, Instance: 'static>(PhantomData<(Runtime, Instance)>);

impl<Runtime, Instance> Precompile for CollectivePrecompile<Runtime, Instance>
where
	Instance: 'static,
	Runtime: pallet_collective::Config<Instance> + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + Decode,
	Runtime::Call: From<pallet_collective::Call<Runtime, Instance>>,
	<Runtime as pallet_collective::Config<Instance>>::Proposal: From<Runtime::Call>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	H256: From<<Runtime as frame_system::Config>::Hash>
		+ Into<<Runtime as frame_system::Config>::Hash>,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let selector = handle.read_selector()?;

		handle.check_function_modifier(match selector {
			Action::ProposalHash => FunctionModifier::View,
			_ => FunctionModifier::NonPayable,
		})?;

		match selector {
			Action::Execute => Self::contract_execute(handle),
			Action::Propose => Self::propose(handle),
			Action::Vote => Self::vote(handle),
			Action::Close => Self::close(handle),
			Action::ProposalHash => Self::proposal_hash(handle),
		}
	}
}

impl<Runtime, Instance> CollectivePrecompile<Runtime, Instance>
where
	Instance: 'static,
	Runtime: pallet_collective::Config<Instance> + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + Decode,
	Runtime::Call: From<pallet_collective::Call<Runtime, Instance>>,
	<Runtime as pallet_collective::Config<Instance>>::Proposal: From<Runtime::Call>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	H256: From<<Runtime as frame_system::Config>::Hash>
		+ Into<<Runtime as frame_system::Config>::Hash>,
{
	fn contract_execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			proposal: BoundedBytes<GetProposalLimit>
		});

		let proposal: Vec<_> = proposal.into_vec();
		let proposal_hash: H256 = hash::<Runtime>(&proposal);
		let proposal_length: u32 = proposal.len().try_into().map_err(|_| {
			RevertReason::value_is_too_large("uint32")
				.in_field("length")
				.in_field("proposal")
		})?;

		let proposal = Runtime::Call::decode(&mut &*proposal)
			.map_err(|_| RevertReason::custom("Failed to decode proposal").in_field("proposal"))?
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

		let log = log_executed(handle.context().address, proposal_hash);

		handle.record_log_costs(&[&log])?;
		log.record(handle)?;

		Ok(succeed(&[]))
	}

	fn propose(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		read_args!(handle, {
			threshold: u32,
			proposal: BoundedBytes<GetProposalLimit>
		});

		let proposal: Vec<_> = proposal.into_vec();
		let proposal_length: u32 = proposal.len().try_into().map_err(|_| {
			RevertReason::value_is_too_large("uint32")
				.in_field("length")
				.in_field("proposal")
		})?;

		let proposal_index = pallet_collective::Pallet::<Runtime, Instance>::proposal_count();
		let proposal_hash: H256 = hash::<Runtime>(&proposal);
		let proposal = Runtime::Call::decode(&mut &*proposal)
			.map_err(|_| RevertReason::custom("Failed to decode proposal").in_field("proposal"))?
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
		log.record(handle)?;

		Ok(succeed(EvmDataWriter::new().write(proposal_index).build()))
	}

	fn vote(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			proposal_hash: H256,
			proposal_index: u32,
			approve: bool
		});

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

		// TODO: Since we cannot access ayes/nays of a proposal we cannot
		// include it in the EVM events to mirror Substrate events.

		let log = log_voted(
			handle.context().address,
			handle.context().caller,
			proposal_hash,
			approve,
		);
		handle.record_log_costs(&[&log])?;
		log.record(handle)?;

		Ok(succeed(&[]))
	}

	fn close(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			proposal_hash: H256,
			proposal_index: u32,
			proposal_weight_bound: u64,
			length_bound: u32
		});

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let post_dispatch_info = RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			pallet_collective::Call::<Runtime, Instance>::close {
				proposal_hash: proposal_hash.into(),
				index: proposal_index,
				proposal_weight_bound,
				length_bound,
			},
		)?;

		// We can know if the proposal was executed or not based on the `pays_fee` in
		// `PostDispatchInfo`.
		let (executed, log) = match post_dispatch_info.pays_fee {
			Pays::Yes => (true, log_executed(handle.context().address, proposal_hash)),
			Pays::No => (false, log_closed(handle.context().address, proposal_hash)),
		};
		handle.record_log_costs(&[&log])?;
		log.record(handle)?;

		Ok(succeed(EvmDataWriter::new().write(executed).build()))
	}

	fn proposal_hash(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			proposal: BoundedBytes<GetProposalLimit>
		});

		let hash = hash::<Runtime>(&proposal.into_vec());

		Ok(succeed(EvmDataWriter::new().write(hash).build()))
	}
}

pub fn hash<Runtime>(data: &[u8]) -> H256
where
	Runtime: frame_system::Config,
	H256: From<<Runtime as frame_system::Config>::Hash>,
{
	<Runtime as frame_system::Config>::Hashing::hash(data).into()
}
