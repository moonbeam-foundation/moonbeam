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
use frame_support::traits::{Currency, Get, ReservableCurrency};
use frame_system::RawOrigin;
use sp_runtime::traits::StaticLookup;

/// Create a funded user.
fn create_funded_user<T: Config>(string: &'static str, n: u32, balance: u32) -> T::AccountId {
	const SEED: u32 = 0;
	let user = account(string, n, SEED);
	T::Currency::make_free_balance_be(&user, balance.into());
	T::Currency::issue(balance.into());
	user
}

/// Create a funded user and register it as a collator in orbiter program
fn create_collator<T: Config>(string: &'static str, n: u32, balance: u32) -> T::AccountId {
	let collator_account: T::AccountId = create_funded_user::<T>(string, n, balance);
	let collator_lookup: <T::Lookup as StaticLookup>::Source =
		T::Lookup::unlookup(collator_account.clone());
	Pallet::<T>::add_collator(RawOrigin::Root.into(), collator_lookup)
		.expect("fail to register collator");
	collator_account
}

/// Create a funded user ard register it as an orbiter
fn create_orbiter<T: Config>(string: &'static str, n: u32, balance: u32) -> T::AccountId {
	let orbiter_account: T::AccountId = create_funded_user::<T>(string, n, balance);
	Pallet::<T>::orbiter_register(RawOrigin::Signed(orbiter_account.clone()).into())
		.expect("fail to register orbiter");
	orbiter_account
}

const MIN_ORBITER_DEPOSIT: u32 = 10_000;
const USER_SEED: u32 = 999666;

benchmarks! {
	collator_add_orbiter {
		let collator_account: T::AccountId = create_collator::<T>("COLLATOR", USER_SEED, 10_000);

		// To test the worst case, we pre-fill the collator pool to the maximum size minus one
		for i in 1..T::MaxPoolSize::get() {
			let orbiter_account: T::AccountId =
				create_orbiter::<T>("ORBITER", USER_SEED + i, 20_000);
			let orbiter_lookup: <T::Lookup as StaticLookup>::Source =
				T::Lookup::unlookup(orbiter_account.clone());
			Pallet::<T>::collator_add_orbiter(
				RawOrigin::Signed(collator_account.clone()).into(),
				orbiter_lookup.clone()
			).expect("fail to add orbiter");
		}

		let orbiter_account: T::AccountId = create_orbiter::<T>("ORBITER", USER_SEED, 20_000);
		let orbiter_lookup: <T::Lookup as StaticLookup>::Source =
			T::Lookup::unlookup(orbiter_account.clone());
	}: _(RawOrigin::Signed(collator_account), orbiter_lookup)
	verify {

	}
	collator_remove_orbiter {
		let collator_account: T::AccountId = create_collator::<T>("COLLATOR", USER_SEED, 10_000);

		// orbiter_lookup must be initialized with an account id
		let mut orbiter_lookup: <T::Lookup as StaticLookup>::Source =
			T::Lookup::unlookup(collator_account.clone());

		// To test the worst case, we pre-fill the collator pool to the maximum size
		for i in 0..T::MaxPoolSize::get() {
			let orbiter_account: T::AccountId =
				create_orbiter::<T>("ORBITER", USER_SEED + i, 20_000);
			orbiter_lookup = T::Lookup::unlookup(orbiter_account.clone());
			Pallet::<T>::collator_add_orbiter(
				RawOrigin::Signed(collator_account.clone()).into(),
				orbiter_lookup.clone()
			).expect("fail to add orbiter");
		}

	}: _(RawOrigin::Signed(collator_account), orbiter_lookup)
	verify {

	}
	orbiter_leave_collator_pool {
		let collator_account: T::AccountId = create_collator::<T>("COLLATOR", USER_SEED, 10_000);

		// orbiter_account must be initialized with an account id
		let mut orbiter_account: T::AccountId = collator_account.clone();

		// To test the worst case, we pre-fill the collator pool to the maximum size
		for i in 0..T::MaxPoolSize::get() {
			orbiter_account = create_orbiter::<T>("ORBITER", USER_SEED + i, 20_000);
			let orbiter_lookup: <T::Lookup as StaticLookup>::Source =
				T::Lookup::unlookup(orbiter_account.clone());
			Pallet::<T>::collator_add_orbiter(
				RawOrigin::Signed(collator_account.clone()).into(),
				orbiter_lookup.clone()
			).expect("fail to add orbiter");
		}

		let collator_lookup: <T::Lookup as StaticLookup>::Source =
			T::Lookup::unlookup(collator_account.clone());
	}: _(RawOrigin::Signed(orbiter_account), collator_lookup)
	verify {

	}
	orbiter_register {
		let orbiter_account: T::AccountId = create_funded_user::<T>("ORBITER", USER_SEED, 20_000);
	}: _(RawOrigin::Signed(orbiter_account.clone()))
	verify {
		assert_eq!(T::Currency::reserved_balance(&orbiter_account), MIN_ORBITER_DEPOSIT.into());
	}
	orbiter_unregister {
		// We make it dependent on the number of collator in the orbiter program
		let n in 0..100;
		for i in 0..n {
			let _: T::AccountId = create_collator::<T>("COLLATOR", USER_SEED + i, 10_000);
		}
		let orbiter_account: T::AccountId = create_orbiter::<T>("ORBITER", USER_SEED, 20_000);
	}: _(RawOrigin::Signed(orbiter_account), n)
	verify {

	}
	add_collator {
		let collator_account: T::AccountId = create_funded_user::<T>("COLLATOR", USER_SEED, 10_000);
		let collator_lookup: <T::Lookup as StaticLookup>::Source =
			T::Lookup::unlookup(collator_account.clone());

	}: _(RawOrigin::Root, collator_lookup.clone())
	verify {

	}
	remove_collator {
		let collator_account: T::AccountId = create_collator::<T>("COLLATOR", USER_SEED, 10_000);

		for i in 0..T::MaxPoolSize::get() {
			let orbiter_account: T::AccountId =
				create_orbiter::<T>("ORBITER", USER_SEED + i, 20_000);
			let orbiter_lookup: <T::Lookup as StaticLookup>::Source =
				T::Lookup::unlookup(orbiter_account.clone());
			Pallet::<T>::collator_add_orbiter(
				RawOrigin::Signed(collator_account.clone()).into(),
				orbiter_lookup.clone()
			).expect("fail to add orbiter");
		}

		let collator_lookup: <T::Lookup as StaticLookup>::Source =
			T::Lookup::unlookup(collator_account.clone());
	}: _(RawOrigin::Root, collator_lookup.clone())
	verify {

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
	fn bench_collator_add_orbiter() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_collator_add_orbiter());
		});
	}

	#[test]
	fn bench_collator_remove_orbiter() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_collator_remove_orbiter());
		});
	}

	#[test]
	fn bench_orbiter_leave_collator_pool() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_orbiter_leave_collator_pool());
		});
	}

	#[test]
	fn bench_orbiter_register() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_orbiter_register());
		});
	}

	#[test]
	fn bench_orbiter_unregister() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_orbiter_unregister());
		});
	}

	#[test]
	fn bench_add_collator() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_add_collator());
		});
	}

	#[test]
	fn bench_remove_collator() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_remove_collator());
		});
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::benchmarks::tests::new_test_ext(),
	crate::mock::Test
);
