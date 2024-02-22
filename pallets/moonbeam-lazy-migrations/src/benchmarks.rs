// Copyright 2024 Moonbeam foundation
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
use crate::{Call, Config, GetArrayLimit, Pallet};
use core::cmp::max;
use frame_benchmarking::{account, benchmarks};
use frame_support::BoundedVec;
use frame_system::RawOrigin;
use sp_core::{H160, H256};

benchmarks! {
	clear_suicided_storage {
		let caller = account("caller", 1, 100);
		// a is the number of addresses
		let a in 1 .. 100;
		// l is the limit of the number of storage entries to be deleted
		let l in 1 .. 1000;

		// Create the addresses to be used in the test
		let mut addresses = BoundedVec::<H160, GetArrayLimit>::new();

		// Create the storage entries to be deleted
		for i in 0..a {
			let address = account("address", i, i);
			addresses.try_push(address).expect("Cannot add more addresses to address list");
			let n = max(1, l/a);
			for j in 0..n {
				pallet_evm::AccountStorages::<T>::insert(
					address,
					H256::from_low_u64_be(j as u64),
					H256::from_low_u64_be(j as u64),
				);
			}
		}
	}:_(
		RawOrigin::Signed(caller),
		addresses,
		l
	)
	verify {
	}
}
