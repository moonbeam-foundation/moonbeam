// Copyright 2019-2024 PureStake Inc.
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

//! Precompile for verifying relay data against a relay block number.

#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use cumulus_primitives_core::relay_chain::BlockNumber as RelayBlockNumber;
use fp_evm::PrecompileHandle;
use frame_support::traits::ConstU32;
use precompile_utils::prelude::*;
use sp_core::H256;
use sp_std::vec;

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);
pub const ARRAY_LIMIT: u32 = 2u32.pow(9);

type GetCallDataLimit = ConstU32<CALL_DATA_LIMIT>;
type GetArrayLimit = ConstU32<ARRAY_LIMIT>;
type RawStorageProof = BoundedVec<BoundedBytes<GetCallDataLimit>, GetArrayLimit>;
type RawKey = BoundedBytes<GetCallDataLimit>;

/// Relay Data Verifier precompile.
pub struct RelayDataVerifierPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> RelayDataVerifierPrecompile<Runtime> {
	#[precompile::public("verify(uint32,bytes[],bytes)")]
	#[precompile::view]
	fn verify(
		_handle: &mut impl PrecompileHandle,
		_relay_block_number: RelayBlockNumber,
		_proof: RawStorageProof,
		_key: RawKey,
	) -> EvmResult<UnboundedBytes> {
		// TODO: to be implemented
		Ok(vec![].into())
	}

	#[precompile::public("verifyBatch(uint32,bytes[],bytes[])")]
	#[precompile::public("verify_batch(uint32,bytes[],bytes[])")]
	#[precompile::view]
	fn verify_batch(
		_handle: &mut impl PrecompileHandle,
		_relay_block_number: RelayBlockNumber,
		_proof: RawStorageProof,
		_keys: BoundedVec<RawKey, GetArrayLimit>,
	) -> EvmResult<BoundedVec<UnboundedBytes, GetArrayLimit>> {
		// TODO: to be implemented
		Ok(vec![vec![].into()].into())
	}

	#[precompile::public("lastRelayBlock()")]
	#[precompile::public("last_relay_block()")]
	#[precompile::view]
	fn last_relay_block(
		_handle: &mut impl PrecompileHandle,
	) -> EvmResult<(RelayBlockNumber, H256)> {
		// TODO: to be implemented
		Ok((0, H256::zero()))
	}
}
