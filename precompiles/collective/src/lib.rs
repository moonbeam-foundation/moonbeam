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
use fp_evm::{Precompile, PrecompileOutput};
use frame_support::{
	dispatch::Dispatchable,
	sp_runtime::traits::Hash,
	traits::{ConstU32, PalletInfo},
	weights::{GetDispatchInfo, PostDispatchInfo},
};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{Decode, H256};

/// Solidity selector of the Executed log.
pub const SELECTOR_LOG_EXECUTED: [u8; 32] = keccak256!("Executed(bytes32)");

/// Solidity selector of the Proposed log.
pub const SELECTOR_LOG_PROPOSED: [u8; 32] = keccak256!("Proposed(address,uint32,bytes32,uint32)");

/// Solidity selector of the Voted log.
pub const SELECTOR_LOG_VOTED: [u8; 32] = keccak256!("Voted(address,bytes32,bool,uint32,uint32)");

type GetProposalLimit = ConstU32<{ 2u32.pow(16) }>;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	Execute = "execute(bytes,uint32)",
	Propose = "propose(uint32,bytes)",
	Vote = "vote(bytes32,uint32,bool)",
	Close = "close(bytes32,uint32,uint64)",
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
		todo!()
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
		let proposal_hash: H256 = Runtime::Hashing::hash(&proposal).into();
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
			log2(
				handle.context().address,
				SELECTOR_LOG_EXECUTED,
				proposal_hash,
				Vec::new(),
			)
		} else {
			log4(
				handle.context().address,
				SELECTOR_LOG_PROPOSED,
				handle.context().caller,
				H256::from_slice(&EvmDataWriter::new().write(proposal_index).build()),
				proposal_hash,
				EvmDataWriter::new().write(threshold).build(),
			)
		};

		handle.record_log_costs(&[&log])?;
		log.record(handle)?;

		Ok(succeed(&[]))
	}

	fn vote(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			proposal_hash: H256,
			proposal_index: u32,
			approve: bool
		});

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let pallet_index = Self::pallet_index()?;
		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			pallet_collective::Call::<Runtime, Instance>::vote {
				proposal: proposal_hash.into(),
				index: proposal_index,
				approve,
			},
		)?;

		todo!()
	}

	fn close(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		todo!()
	}

	fn proposal_hash(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			proposal: BoundedBytes<GetProposalLimit>
		});

		let hash: H256 = Runtime::Hashing::hash(&proposal.into_vec()).into();

		Ok(succeed(EvmDataWriter::new().write(hash).build()))
	}

	fn pallet_index() -> EvmResult<usize> {
		<Runtime as frame_system::Config>::PalletInfo::index::<
			pallet_collective::Pallet<Runtime, Instance>,
		>()
		.ok_or_else(|| revert("cannot retreive pallet collective index"))
	}
}
