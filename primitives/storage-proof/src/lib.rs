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

#![cfg_attr(not(feature = "std"), no_std)]

use cumulus_primitives_core::relay_chain;
use sp_core::H256;
use sp_runtime::traits::{Hash, HashingFor};
use sp_state_machine::{Backend, TrieBackend, TrieBackendBuilder};
use sp_std::vec::Vec;
use sp_trie::{HashDBT, MemoryDB, StorageProof, EMPTY_PREFIX};

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq)]
pub enum ProofError {
	// The storage root in the proof does not match the expected storage root.
	RootMismatch,
	// The proof is invalid.
	Proof,
	// The key is not present in the proof.
	Absent,
	// Block number is not present
	BlockNumberNotPresent,
}

pub type RawStorageProof = Vec<Vec<u8>>;

/// A storage proof checker. It is used to verify a storage proof against a well-known storage root,
/// and return the value of the storage item if the proof is valid.
#[derive(Debug)]
pub struct StorageProofChecker<H>
where
	H: Hash,
{
	trie_backend: TrieBackend<MemoryDB<H>, H>,
}

impl<H: Hash> StorageProofChecker<H> {
	/// Create a new storage proof checker. Returns an error if the given `storage_root` is not
	/// present in the proof.
	pub fn new(
		storage_root: H::Out,
		raw_proof: impl IntoIterator<Item = Vec<u8>>,
	) -> Result<Self, ProofError> {
		let storage_proof = StorageProof::new(raw_proof);
		let db = storage_proof.into_memory_db::<H>();

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

	pub fn read_entries(&self, keys: &[&[u8]]) -> Result<Vec<Vec<u8>>, ProofError> {
		let mut values = Vec::new();
		for key in keys {
			let value = self.read_entry(key)?;
			values.push(value);
		}
		Ok(values)
	}
}

pub fn verify_entry(
	expected_root: H256,
	proof: impl IntoIterator<Item = Vec<u8>>,
	key: &[u8],
) -> Result<Vec<u8>, ProofError> {
	let proof_checker =
		StorageProofChecker::<HashingFor<relay_chain::Block>>::new(expected_root, proof)?;

	proof_checker.read_entry(key)
}

pub fn verify_entries(
	expected_root: H256,
	proof: impl IntoIterator<Item = Vec<u8>>,
	keys: &[&[u8]],
) -> Result<Vec<Vec<u8>>, ProofError> {
	let proof_checker =
		StorageProofChecker::<HashingFor<relay_chain::Block>>::new(expected_root, proof)?;

	proof_checker.read_entries(keys)
}
