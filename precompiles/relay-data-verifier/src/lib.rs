// Copyright 2025 Moonbeam Foundation.
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

//! Precompile for verifying relay entries against a relay block number.

#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;
use cumulus_primitives_core::relay_chain::BlockNumber as RelayBlockNumber;
use fp_evm::{PrecompileFailure, PrecompileHandle};
use frame_support::{ensure, traits::ConstU32};
use pallet_precompile_benchmarks::WeightInfo as TWeightInfo;
use precompile_utils::prelude::*;
use sp_core::H256;
use sp_std::vec::Vec;
use storage_proof_primitives::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);
pub const ARRAY_LIMIT: u32 = 2048;
pub const KEY_LENGTH_LIMIT: u32 = 256;

pub type GetCallDataLimit = ConstU32<CALL_DATA_LIMIT>;
pub type GetKeyLengthLimit = ConstU32<KEY_LENGTH_LIMIT>;
pub type GetArrayLimit = ConstU32<ARRAY_LIMIT>;

pub type RawKey = BoundedBytes<GetKeyLengthLimit>;

/// Relay Data Verifier precompile.
pub struct RelayDataVerifierPrecompile<Runtime, WeightInfo>(PhantomData<(Runtime, WeightInfo)>);

#[precompile_utils::precompile]
impl<Runtime, WeightInfo> RelayDataVerifierPrecompile<Runtime, WeightInfo>
where
	Runtime: frame_system::Config + pallet_relay_storage_roots::Config + pallet_evm::Config,
	WeightInfo: TWeightInfo,
{
	/// Verify the storage entry using the provided relay block number and proof. Return the value
	/// of the storage entry if the proof is valid and the entry exists.
	#[precompile::public("verifyEntry(uint32,(bytes32,bytes[]),bytes)")]
	#[precompile::public("verify_entry(uint32,(bytes32,bytes[]),bytes)")]
	fn verify_entry(
		handle: &mut impl PrecompileHandle,
		relay_block_number: RelayBlockNumber,
		proof: ReadProof,
		key: RawKey,
	) -> EvmResult<UnboundedBytes> {
		// Charge gas for storage proof verification
		let weight = WeightInfo::verify_entry(proof.proof.len() as u32);
		handle.record_external_cost(Some(weight.ref_time()), Some(0), Some(0))?;

		// Get the storage root of the relay block
		let storage_root = Self::get_storage_root(handle, relay_block_number)?;

		// One read per key
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Read and return the value associated ton the key
		verify_entry(storage_root, proof.to_raw_proof(), key.as_bytes())
			.map_err(map_err)
			.map(UnboundedBytes::from)
	}

	/// Verify the storage entries using the provided relay block number and proof. Return the
	/// values of the storage entries in the same order of keys, if the proof is valid and the
	/// entries exist.
	#[precompile::public("verifyEntries(uint32,(bytes32,bytes[]),bytes[])")]
	#[precompile::public("verify_entries(uint32,(bytes32,bytes[]),bytes[])")]
	fn verify_entries(
		handle: &mut impl PrecompileHandle,
		relay_block_number: RelayBlockNumber,
		proof: ReadProof,
		keys: BoundedVec<RawKey, GetArrayLimit>,
	) -> EvmResult<BoundedVec<UnboundedBytes, GetArrayLimit>> {
		ensure!(keys.len() > 0, revert("Keys must not be empty"));

		//  Charge gas for storage proof verification
		let weight = WeightInfo::verify_entry(proof.proof.len() as u32);
		handle.record_external_cost(Some(weight.ref_time()), Some(0), Some(0))?;

		// Get the storage root of the relay block
		let storage_root = Self::get_storage_root(handle, relay_block_number)?;

		// Charge one db read per key
		handle.record_cost(
			(keys.len() as u64).saturating_mul(RuntimeHelper::<Runtime>::db_read_gas_cost()),
		)?;

		// Read and return the values associated ton the keys
		let keys = Vec::from(keys);
		let keys: Vec<_> = keys.iter().map(|x| x.as_bytes()).collect();
		verify_entries(storage_root, proof.to_raw_proof(), &keys)
			.map_err(map_err)
			.map(|x| x.into_iter().map(UnboundedBytes::from).collect::<Vec<_>>())
			.map(|x| BoundedVec::from(x))
	}

	#[precompile::public("latestRelayBlockNumber()")]
	#[precompile::public("latest_relay_block_number()")]
	#[precompile::view]
	fn latest_relay_block(handle: &mut impl PrecompileHandle) -> EvmResult<RelayBlockNumber> {
		let weight = WeightInfo::latest_relay_block();
		handle.record_external_cost(Some(weight.ref_time()), Some(weight.proof_size()), Some(0))?;

		pallet_relay_storage_roots::RelayStorageRootKeys::<Runtime>::get()
			.last()
			.cloned()
			.ok_or(revert("No relay block found"))
	}

	fn get_storage_root(
		handle: &mut impl PrecompileHandle,
		relay_block_number: RelayBlockNumber,
	) -> EvmResult<H256> {
		handle.record_db_read::<Runtime>(84)?;
		pallet_relay_storage_roots::RelayStorageRoot::<Runtime>::get(relay_block_number)
			.ok_or(revert("Block number not present"))
	}
}

fn map_err(err: ProofError) -> PrecompileFailure {
	match err {
		ProofError::RootMismatch => revert("Root Mismatch"),
		ProofError::Proof => revert("Invalid Proof"),
		ProofError::Absent => revert("Value is not present"),
		ProofError::BlockNumberNotPresent => revert("Block number not present"),
	}
}

#[derive(Clone, Debug, solidity::Codec)]
pub struct ReadProof {
	// Block Hash used to generate the proof
	pub at: H256,
	// A storage proof
	pub proof: BoundedVec<BoundedBytes<GetCallDataLimit>, GetArrayLimit>,
}

impl ReadProof {
	pub fn to_raw_proof(self) -> Vec<Vec<u8>> {
		Vec::from(self.proof)
			.iter()
			.map(|x| x.as_bytes().to_vec())
			.collect()
	}
}
