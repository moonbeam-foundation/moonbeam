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

//! This pallet is designed for benchmarking precompile functions. It should not be used in
//! production.

use cumulus_primitives_core::relay_chain;
use frame_support::sp_runtime::traits::HashingFor;
use parity_scale_codec::Encode;
use sp_core::H256;
use sp_std::collections::btree_map::BTreeMap;
use sp_trie::PrefixedMemoryDB;
use std::{fs::File, io::Write};

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
fn benchmark_mocked_storage_proof() {
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
