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

#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::no_effect)]

use crate::{Config, Pallet};
use frame_benchmarking::benchmarks;
use frame_support::{traits::Get, BoundedVec};
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

fn fill_relay_storage_roots<T: Config>() -> Vec<u32> {
	// Initialize keys BoundedVec for RelayStorageRoots
	let mut keys = BoundedVec::<u32, _>::default();
	for i in 0..T::MaxStorageRoots::get() {
		let relay_root = H256::from_low_u64_be(i as u64);
		pallet_relay_storage_roots::RelayStorageRoot::<T>::insert(i, relay_root);
		keys.try_push(i)
			.expect("Keys should not exceed MaxStorageRoots");
	}
	pallet_relay_storage_roots::RelayStorageRootKeys::<T>::put(keys.clone());
	keys.to_vec()
}

benchmarks! {
	verify_entry {
		// x is the number of nodes in the proof
		let x in 100..2000 => 100;

		let mocked_proofs: BTreeMap<u32, (H256, Vec<Vec<u8>>)> =
			BTreeMap::decode(&mut &include_bytes!("../benchmark_proofs").to_vec()[..])
			.expect("Failed to decode mocked proofs");


		// if x is not multiple of 100, we will use the proof for the closest multiple of 100
		let x = (x / 100) * 100;
		let (state_root, mocked_proof) =
			mocked_proofs.get(&x).expect("Not Found").clone();

		// Set the state root for the relay block in the relay storage roots pallet
		let relay_block = 10;
		pallet_relay_storage_roots::RelayStorageRoot::<T>::insert(relay_block, state_root);

		let key = 2u128.encode();
	}:{
		Pallet::<T>::verify(relay_block, mocked_proof, &key)
			.expect("Should verify the entry without error.");
	}

	latest_relay_block {
		let keys = fill_relay_storage_roots::<T>();
	}:{
		Pallet::<T>::latest_relay_block().expect("There should be at least one relay block entry.");
	}
	verify {
		assert_eq!(
			Pallet::<T>::latest_relay_block()
				.expect("Should return the latest relay block without error."),
			keys.last().cloned().expect("There should be at least one key")
		);
	}
}
