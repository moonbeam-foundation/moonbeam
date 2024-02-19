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

use cumulus_primitives_core::relay_chain;
use frame_support::sp_runtime::traits::HashingFor;
use sp_core::H256;
use sp_state_machine::{Backend, TrieBackend, TrieBackendBuilder};
use sp_std::vec::Vec;
use sp_trie::{HashDBT, MemoryDB, StorageProof, EMPTY_PREFIX};

#[derive(Debug, PartialEq)]
pub enum ProofError {
	// The storage root in the proof does not match the expected storage root.
	RootMismatch,
	// The proof is invalid.
	Proof,
	// The key is not present in the proof.
	Absent,
}
/// A storage proof checker. It is used to verify a storage proof against a well-known storage root,
/// and return the value of the storage item if the proof is valid.
#[derive(Debug)]
pub struct StorageProofChecker {
	trie_backend:
		TrieBackend<MemoryDB<HashingFor<relay_chain::Block>>, HashingFor<relay_chain::Block>>,
}

impl StorageProofChecker {
	/// Create a new storage proof checker. Returns an error if the given `storage_root` is not
	/// present in the proof.
	pub fn new(
		storage_root: H256,
		raw_proof: impl IntoIterator<Item = Vec<u8>>,
	) -> Result<Self, ProofError> {
		let storage_proof = StorageProof::new(raw_proof);
		let db = storage_proof.into_memory_db::<HashingFor<relay_chain::Block>>();
		if !db.contains(&storage_root, EMPTY_PREFIX) {
			return Err(ProofError::RootMismatch);
		}
		let trie_backend = TrieBackendBuilder::new(db, storage_root).build();

		Ok(Self { trie_backend })
	}

	/// Returns the value of the storage given the key, if the proof is valid.
	/// Returns `Err` if the proof is invalid, or if the value specified by the key according to the
	/// proof is not present.
	pub fn read_entry(&self, key: &[u8]) -> Result<Vec<u8>, ProofError> {
		self.trie_backend
			.storage(key)
			.map_err(|_| ProofError::Proof)?
			.ok_or(ProofError::Absent)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{
		hex_to_bytes, mock_raw_proof, STORAGE_ROOT_HEX, TIMESTAMP_KEY_HEX, TOTAL_ISSUANCE_KEY_HEX,
		TREASURY_APPROVAL_KEY_HEX,
	};
	use parity_scale_codec::Decode;

	fn construct_proof() -> Vec<Vec<u8>> {
		// Mock a storage proof obtained from the relay chain using the
		// state_getReadProof RPC call, for the following keys:
		// TimeStamp:
		// 0xf0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb
		// Balances (Total Issuance):
		// 0xc2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80
		// Treasury Approvals
		// 0x89d139e01a5eb2256f222e5fc5dbe6b33c9c1284130706f5aea0c8b3d4c54d89
		// at Block Hash:
		// 0x1272470f226fc0e955838262e8dd17a7d7bad6563739cc53a3b1744ddf0ea872
		mock_raw_proof()
	}

	#[test]
	fn test_storage_proof_checker_valid() {
		let proof = construct_proof();
		let storage_root = H256::from_slice(&hex_to_bytes(STORAGE_ROOT_HEX));

		assert!(StorageProofChecker::new(storage_root, proof).is_ok());
	}

	#[test]
	fn test_storage_proof_checker_root_mismsatch() {
		let proof = construct_proof();
		// A different storage root
		let storage_root = H256::from_slice(&hex_to_bytes(
			"767caa877bcea0d34dd515a202b75efa41bffbc9f814ab59e2c1c96716d4c65e",
		));

		assert_eq!(
			StorageProofChecker::new(storage_root, proof).unwrap_err(),
			ProofError::RootMismatch
		);
	}

	#[test]
	fn test_storage_proof_read_entries() {
		let proof = construct_proof();
		let storage_root = H256::from_slice(&hex_to_bytes(STORAGE_ROOT_HEX));
		let proof_checker = StorageProofChecker::new(storage_root, proof).unwrap();

		let key1 = hex_to_bytes(TIMESTAMP_KEY_HEX);
		let key2 = hex_to_bytes(TOTAL_ISSUANCE_KEY_HEX);
		let key3 = hex_to_bytes(TREASURY_APPROVAL_KEY_HEX);

		let timestamp: u64 =
			u64::decode(&mut &proof_checker.read_entry(&key1).unwrap()[..]).unwrap();
		let total_issuance: u128 =
			u128::decode(&mut &proof_checker.read_entry(&key2).unwrap()[..]).unwrap();
		let approvals: Vec<u32> =
			Vec::<u32>::decode(&mut &proof_checker.read_entry(&key3).unwrap()[..]).unwrap();

		assert_eq!(
			timestamp, 1_708_190_328_000,
			"Timestamp does not match the expected value"
		);
		assert_eq!(
			total_issuance, 14_123_366_426_803_276_130,
			"Total issuance does not match the expected value"
		);
		assert_eq!(
			approvals,
			vec![
				607, 608, 609, 610, 611, 612, 613, 614, 615, 616, 617, 618, 619, 620, 621, 622, 623
			],
			"Value 3 does not match the expected value"
		);
	}

	#[test]
	fn test_storage_proof_checker_absent() {
		let proof = construct_proof();
		let storage_root = H256::from_slice(&hex_to_bytes(STORAGE_ROOT_HEX));

		let proof_checker = StorageProofChecker::new(storage_root, proof).unwrap();

		// A key that is not present in the proof
		let key = hex_to_bytes("89d139e01a5eb2256f222e5fc5dbe6b33c9c1284130706f5aea0c8b3d4c54d2c");
		let value = proof_checker.read_entry(&key);
		assert_eq!(value.err(), Some(ProofError::Absent));
	}
}
