// Copyright 2019-2022 PureStake Inc.
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
use frame_benchmarking::{account, benchmarks};
use frame_support::{
	traits::{Currency, Get},
	BoundedVec,
};
use frame_system::RawOrigin;
use sp_core::{H160, H256};

/// Create a funded user.
fn create_funded_user<T: Config>(string: &'static str, n: u32, balance: u32) -> T::AccountId {
	const SEED: u32 = 0;
	let user = account(string, n, SEED);
	T::Currency::make_free_balance_be(&user, balance.into());
	T::Currency::issue(balance.into());
	user
}

benchmarks! {
	clear_suicided_storage {
		let caller = create_funded_user::<T>("caller", 0, 100);
		// a is the number of addresses to be used in the test
		let a in 0 .. GetArrayLimit::get();
		// l is the limit of the number of storage entries to be deleted
		let l in 0 .. 32330;
		let e in 0 .. 32330;
		// Create the addresses to be used in the test
		let mut addresses = BoundedVec::<H160, GetArrayLimit>::new();

		// Create the storage entries to be deleted
		for i in 0..=a {
			// let entries = rand::random::<u32>() % e + 1;
			let address = H160::repeat_byte(i as u8);
			addresses.try_push(address).unwrap();
			for j in 0..e {
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
