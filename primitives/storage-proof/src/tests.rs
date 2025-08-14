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

use crate::{ProofError, StorageProofChecker};
use cumulus_primitives_core::relay_chain;
use frame_support::assert_ok;
use parity_scale_codec::Encode;
use sp_core::H256;
use sp_runtime::traits::HashingFor;
use sp_std::collections::btree_map::BTreeMap;
use sp_trie::PrefixedMemoryDB;
use std::{fs::File, io::Write};

// Storage Root: 767caa877bcea0d34dd515a202b75efa41bffbc9f814ab59e2c1c96716d4c65d
pub const STORAGE_ROOT: &[u8] = &[
	118, 124, 170, 135, 123, 206, 160, 211, 77, 213, 21, 162, 2, 183, 94, 250, 65, 191, 251, 201,
	248, 20, 171, 89, 226, 193, 201, 103, 22, 212, 198, 93,
];

// Timestamp key: f0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb
pub const TIMESTAMP_KEY: &[u8] = &[
	240, 195, 101, 195, 207, 89, 214, 113, 235, 114, 218, 14, 122, 65, 19, 196, 159, 31, 5, 21,
	244, 98, 205, 207, 132, 224, 241, 214, 4, 93, 252, 187,
];

// Total Issuance Key: c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80
pub const TOTAL_ISSUANCE_KEY: &[u8] = &[
	194, 38, 18, 118, 204, 157, 31, 133, 152, 234, 75, 106, 116, 177, 92, 47, 87, 200, 117, 228,
	207, 247, 65, 72, 228, 98, 143, 38, 75, 151, 76, 128,
];

// Treasury Approval Key: 89d139e01a5eb2256f222e5fc5dbe6b33c9c1284130706f5aea0c8b3d4c54d89
pub const TREASURY_APPROVALS_KEY: &[u8] = &[
	137, 209, 57, 224, 26, 94, 178, 37, 111, 34, 46, 95, 197, 219, 230, 179, 60, 156, 18, 132, 19,
	7, 6, 245, 174, 160, 200, 179, 212, 197, 77, 137,
];

pub fn mock_proof() -> Vec<Vec<u8>> {
	use parity_scale_codec::Decode;
	Vec::decode(&mut &include_bytes!("../proof").to_vec()[..]).unwrap()
}

#[test]
fn test_storage_proof_checker_valid() {
	let proof = mock_proof();
	let storage_root = H256::from_slice(STORAGE_ROOT);

	assert_ok!(StorageProofChecker::<HashingFor<relay_chain::Block>>::new(
		storage_root,
		proof
	));
}

#[test]
fn test_storage_proof_checker_root_mismatch() {
	let proof = mock_proof();
	// A different storage root
	let storage_root = H256::from_slice(
		hex::decode("767caa877bcea0d34dd515a202b75efa41bffbc9f814ab59e2c1c96716d4c65e")
			.unwrap()
			.as_slice(),
	);

	assert_eq!(
		StorageProofChecker::<HashingFor<relay_chain::Block>>::new(storage_root, proof)
			.unwrap_err(),
		ProofError::RootMismatch
	);
}

#[test]
fn test_storage_proof_read_entries() {
	let proof = mock_proof();
	let storage_root = H256::from_slice(STORAGE_ROOT);
	let proof_checker =
		StorageProofChecker::<HashingFor<relay_chain::Block>>::new(storage_root, proof).unwrap();

	let timestamp = proof_checker.read_entry(TIMESTAMP_KEY).unwrap();
	let total_issuance = proof_checker.read_entry(TOTAL_ISSUANCE_KEY).unwrap();
	let approvals = proof_checker.read_entry(TREASURY_APPROVALS_KEY).unwrap();

	assert_eq!(
		timestamp,
		1_708_190_328_000u64.encode(),
		"Timestamp does not match the expected value"
	);
	assert_eq!(
		total_issuance,
		14_123_366_426_803_276_130u128.encode(),
		"Total issuance does not match the expected value"
	);
	assert_eq!(
		approvals,
		vec![607, 608, 609, 610, 611, 612, 613, 614, 615, 616, 617, 618, 619, 620, 621, 622, 623]
			.encode(),
		"Treasury Approvals does not match the expected value"
	);
}

#[test]
fn test_storage_proof_checker_absent() {
	let proof = mock_proof();
	let storage_root = H256::from_slice(STORAGE_ROOT);

	let proof_checker =
		StorageProofChecker::<HashingFor<relay_chain::Block>>::new(storage_root, proof).unwrap();

	// A key that is not present in the proof
	let key =
		hex::decode("89d139e01a5eb2256f222e5fc5dbe6b33c9c1284130706f5aea0c8b3d4c54d2c").unwrap();
	let value = proof_checker.read_entry(&key);
	assert_eq!(value.err(), Some(ProofError::Absent));
}

pub fn build_mocked_proof(
	entries: Vec<(Vec<u8>, Vec<u8>)>,
	keys: Vec<Vec<u8>>,
) -> (H256, Vec<Vec<u8>>) {
	let (db, root) = PrefixedMemoryDB::<HashingFor<relay_chain::Block>>::default_with_root();
	let state_version = Default::default();
	let mut backend = sp_state_machine::TrieBackendBuilder::new(db, root).build();

	entries.into_iter().for_each(|(key, value)| {
		backend.insert(vec![(None, vec![(key, Some(value))])], state_version);
	});

	let root = *backend.root();
	let proof = sp_state_machine::prove_read(backend, keys).expect("prove read");

	(root, proof.into_iter_nodes().collect())
}

// Generate mocked proofs for the benchmarks. The proofs are generated for a set of
// keys and values, and then stored in a file. The proofs are then used in the benchmarks
// to simulate the proofs obtained from the relay chain.
#[test]
fn test_mocked_storage_proof() {
	// This set of entries generates proofs with number of nodes in proof increasing by 100 for
	// each entry (Number of Proof Node, Number of Entries)
	let entries: Vec<(u32, u32)> = vec![
		(100, 95),
		(200, 190),
		(300, 270),
		(400, 320),
		(500, 370),
		(600, 420),
		(700, 470),
		(800, 530),
		(900, 630),
		(1000, 730),
		(1100, 830),
		(1200, 930),
		(1300, 1030),
		(1400, 1130),
		(1500, 1230),
		(1600, 1330),
		(1700, 1430),
		(1800, 1530),
		(1900, 1630),
		(2000, 1730),
	];

	let mut proofs = BTreeMap::new();
	entries.into_iter().for_each(|(i, x)| {
		let keys: Vec<Vec<u8>> = (1..x as u128).into_iter().map(|y| y.encode()).collect();
		let entries = keys
			.iter()
			.enumerate()
			.map(|(i, key)| (key.clone(), (i as u128).encode()))
			.collect();

		let (state_root, proof) = build_mocked_proof(entries, keys);
		proofs.insert(i, (state_root, proof));
	});

	let mut file = File::create(format!("benchmark_proofs")).unwrap();
	file.write_all(&proofs.encode()).unwrap();
}
