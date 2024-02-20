// Copyright 2024 Moonbeam Foundation.
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
use precompile_utils::prelude::*;
use sp_core::H256;
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod proof;
mod weights;
use proof::{ProofError, StorageProofChecker};
use weights::{SubstrateWeight, WeightInfo};

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);
pub const ARRAY_LIMIT: u32 = 512;
pub const KEY_LENGTH_LIMIT: u32 = 256;

pub type GetCallDataLimit = ConstU32<CALL_DATA_LIMIT>;
pub type GetKeyLengthLimit = ConstU32<KEY_LENGTH_LIMIT>;
pub type GetArrayLimit = ConstU32<ARRAY_LIMIT>;

pub type RawKey = BoundedBytes<GetKeyLengthLimit>;

/// Relay Data Verifier precompile.
pub struct RelayDataVerifierPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> RelayDataVerifierPrecompile<Runtime>
where
	Runtime: frame_system::Config + pallet_relay_storage_roots::Config + pallet_evm::Config,
{
	/// Verify the storage entry using the provided relay block number and proof. Return the value
	/// of the storage entry if the proof is valid and the entry exists.
	#[precompile::public("verifyEntry(uint32,(bytes32,bytes[]),bytes)")]
	#[precompile::public("verify_entry(uint32,(bytes32,bytes[]),bytes)")]
	fn verify_entry(
		_handle: &mut impl PrecompileHandle,
		relay_block_number: RelayBlockNumber,
		proof: ReadProof,
		key: RawKey,
	) -> EvmResult<UnboundedBytes> {
		Self::do_verify_entry(relay_block_number, proof, key)
	}

	/// Verify the storage entries using the provided relay block number and proof. Return the
	/// values of the storage entries in the same order of keys, if the proof is valid and the
	/// entries exist.
	#[precompile::public("verifyEntries(uint32,(bytes32,bytes[]),bytes[])")]
	#[precompile::public("verify_entries(uint32,(bytes32,bytes[]),bytes[])")]
	fn verify_entries(
		_handle: &mut impl PrecompileHandle,
		relay_block_number: RelayBlockNumber,
		proof: ReadProof,
		keys: BoundedVec<RawKey, GetArrayLimit>,
	) -> EvmResult<BoundedVec<UnboundedBytes, GetArrayLimit>> {
		Self::do_verify_entries(relay_block_number, proof, keys)
	}

	#[precompile::public("latestRelayBlockNumber()")]
	#[precompile::public("latest_relay_block_number()")]
	#[precompile::view]
	fn latest_relay_block(handle: &mut impl PrecompileHandle) -> EvmResult<RelayBlockNumber> {
		let weight = <SubstrateWeight<Runtime> as WeightInfo>::latest_relay_block();
		handle.record_external_cost(Some(weight.ref_time()), Some(weight.proof_size()), Some(0))?;

		pallet_relay_storage_roots::RelayStorageRootKeys::<Runtime>::get()
			.last()
			.cloned()
			.ok_or(revert("No relay block found"))
	}

	/// Returns the storage root at the given relay block number stored on-chain. Use the pallet
	/// `pallet_relay_storage_roots` to store the storage roots on-chain.
	fn get_storage_root(relay_block_number: RelayBlockNumber) -> EvmResult<H256> {
		let storage_root =
			pallet_relay_storage_roots::RelayStorageRoot::<Runtime>::get(relay_block_number)
				.ok_or(revert(
					"Storage root is not stored on chain for the given relay block number",
				))?;

		Ok(storage_root)
	}

	pub fn do_verify_entry(
		relay_block_number: RelayBlockNumber,
		proof: ReadProof,
		key: RawKey,
	) -> EvmResult<UnboundedBytes> {
		let storage_root = Self::get_storage_root(relay_block_number)?;
		let proof_checker = StorageProofChecker::new(storage_root, proof.to_raw_proof())?;

		let value = proof_checker.read_entry(key.as_bytes())?;

		Ok(value.into())
	}

	pub fn do_verify_entries(
		relay_block_number: RelayBlockNumber,
		proof: ReadProof,
		keys: BoundedVec<RawKey, GetArrayLimit>,
	) -> EvmResult<BoundedVec<UnboundedBytes, GetArrayLimit>> {
		let keys = Vec::from(keys);
		ensure!(!keys.is_empty(), revert("Keys must not be empty"));

		let storage_root = Self::get_storage_root(relay_block_number)?;
		let proof_checker = StorageProofChecker::new(storage_root, proof.to_raw_proof())?;

		let mut values = Vec::new();
		for key in keys {
			let value: Vec<u8> = proof_checker.read_entry(key.as_bytes())?;
			values.push(value.into());
		}

		Ok(values.into())
	}
}

impl From<ProofError> for PrecompileFailure {
	fn from(err: ProofError) -> Self {
		match err {
			ProofError::RootMismatch => revert("Root Mismatch"),
			ProofError::Proof => revert("Invalid Proof"),
			ProofError::Absent => revert("Value is not present"),
		}
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
