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
use crate::{Config, Pallet};
use frame_benchmarking::benchmarks;
use frame_support::{traits::Get, BoundedVec};
use sp_core::H256;

benchmarks! {
	latest_relay_block {
		// Initialize keys BoundedVec for RelayStorageRoots
		let mut keys = BoundedVec::<u32, _>::default();
		for i in 0..T::MaxStorageRoots::get() {
			let relay_root = H256::from_low_u64_be(i as u64);
			pallet_relay_storage_roots::RelayStorageRoot::<T>::insert(i, relay_root);
			keys.try_push(i).expect("Keys BoundedVec should not exceed MaxStorageRoots");
		}

		pallet_relay_storage_roots::RelayStorageRootKeys::<T>::put(keys.clone());
		assert!(
			keys.len() as u32 >= T::MaxStorageRoots::get(),
			"Inserted keys should meet MaxStorageRoots configuration."
		);
	}:{
		Pallet::<T>::latest_relay_block().unwrap();
	}
	verify {
		assert_eq!(
			Pallet::<T>::latest_relay_block()
				.expect("Should return the latest relay block without error."),
			keys.last().cloned().expect("There should be at least one key")
		);
	}
}
