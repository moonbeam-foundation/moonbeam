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

//! Benchmarking

use crate::{Call, Config, Pallet};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::traits::{Currency, ReservableCurrency};
use frame_system::RawOrigin;
use sp_runtime::traits::One;

/// Create a funded user.
fn create_funded_user<T: Config>(string: &'static str, n: u32, balance: u32) -> T::AccountId {
	const SEED: u32 = 0;
	let user = account(string, n, SEED);
	T::Currency::make_free_balance_be(&user, balance.into());
	T::Currency::issue(balance.into());
	user
}

const MIN_ORBITER_DEPOSIT: u32 = 10_000;
const USER_SEED: u32 = 999666;

benchmarks! {
	orbiter_register {
		let orbiter_account: T::AccountId = create_funded_user::<T>("TEST", 20_000, USER_SEED);
	}: _(RawOrigin::Signed(orbiter_account.clone()))
	verify {
		assert_eq!(T::Currency::reserved_balance(&orbiter_account), MIN_ORBITER_DEPOSIT.into());
	}
	force_update_min_orbiter_deposit {
	}: _(RawOrigin::Root, One::one())
	verify {
		assert_eq!(Pallet::<T>::min_orbiter_deposit(), One::one());
	}
}

#[cfg(test)]
mod tests {
	use crate::benchmarks::*;
	use crate::mock::Test;
	use frame_support::assert_ok;
	use parity_scale_codec::Encode;
	use sp_io::TestExternalities;

	pub fn new_test_ext() -> TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		let min_orbiter_deposit_prefix =
			frame_support::storage::storage_prefix(b"MoonbeamOrbiters", b"MinOrbiterDeposit");
		t.top.insert(
			min_orbiter_deposit_prefix.to_vec(),
			(MIN_ORBITER_DEPOSIT as crate::mock::Balance).encode(),
		);
		TestExternalities::new(t)
	}

	#[test]
	fn bench_orbiter_register() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_orbiter_register());
		});
	}

	#[test]
	fn bench_force_update_min_orbiter_deposit() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_force_update_min_orbiter_deposit());
		});
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::benchmarks::tests::new_test_ext(),
	crate::mock::Test
);
