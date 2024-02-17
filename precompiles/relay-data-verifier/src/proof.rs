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
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_state_machine::{Backend, TrieBackend, TrieBackendBuilder};
use sp_std::vec::Vec;
use sp_trie::{HashDBT, MemoryDB, StorageProof, EMPTY_PREFIX};

#[derive(Encode, Decode)]
pub struct ReadProof {
	// Block Hash used to generate the proof
	pub at: H256,
	// A storage proof
	pub proof: Vec<Vec<u8>>,
}

pub enum ProofError {
	// The storage root in the proof does not match the expected storage root.
	RootMismatch,
	// The proof is invalid.
	Proof,
	// The value could not be decoded.
	Decode,
	// The key is not present in the proof.
	Absent,
}
/// A storage proof checker. It is used to verify a storage proof against a well-known storage root,
/// and return the value of the storage item if the proof is valid.
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

	/// Returns the value of the storage given the key, if the proof is valid. If the value
	/// specified by the key according to the proof is empty, the `fallback` value will be returned.
	///
	/// Returns `Err` if the proof is invalid, or if the value specified by the key according to the
	/// proof is not present.
	pub fn read_entry<T>(&self, key: &[u8], fallback: Option<T>) -> Result<T, ProofError>
	where
		T: Decode,
	{
		self.trie_backend
			.storage(key)
			.map_err(|_| ProofError::Proof)?
			.map(|raw_entry| T::decode(&mut &raw_entry[..]).map_err(|_| ProofError::Decode))
			.transpose()?
			.or(fallback)
			.ok_or(ProofError::Absent)
	}
}
