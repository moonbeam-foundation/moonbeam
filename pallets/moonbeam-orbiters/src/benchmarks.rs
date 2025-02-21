// Copyright 2019-2025 PureStake Inc.
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

use crate::{
	AccountLookupOverride, BalanceOf, Call, CollatorPoolInfo, CollatorsPool, Config, CurrentRound,
	ForceRotation, MinOrbiterDeposit, OrbiterPerRound, Pallet,
};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::traits::{Currency, Get, OnInitialize, ReservableCurrency};
use frame_system::RawOrigin;
use sp_runtime::traits::{Saturating, StaticLookup};

const MIN_ORBITER_DEPOSIT: u32 = 10_000;
const USER_SEED: u32 = 999666;

fn init<T: Config>() {
	let min_orbiter_deposit: BalanceOf<T> = MIN_ORBITER_DEPOSIT.into();
	MinOrbiterDeposit::<T>::put(min_orbiter_deposit);
}

/// Create a funded user.
fn create_funded_user<T: Config>(string: &'static str, n: u32, balance: u32) -> T::AccountId {
	const SEED: u32 = 0;
	let user = account(string, n, SEED);
	let _ = T::Currency::make_free_balance_be(&user, balance.into());
	let _ = T::Currency::issue(balance.into());
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

benchmarks! {
	collator_add_orbiter {
		init::<T>();
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
		init::<T>();
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
		init::<T>();
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
		init::<T>();
		let orbiter_account: T::AccountId = create_funded_user::<T>("ORBITER", USER_SEED, 20_000);
	}: _(RawOrigin::Signed(orbiter_account.clone()))
	verify {
		assert_eq!(T::Currency::reserved_balance(&orbiter_account), MIN_ORBITER_DEPOSIT.into());
	}
	orbiter_unregister {
		// We make it dependent on the number of collator in the orbiter program
		let n in 0..100;

		init::<T>();

		for i in 0..n {
			let _: T::AccountId = create_collator::<T>("COLLATOR", USER_SEED + i, 10_000);
		}
		let orbiter_account: T::AccountId = create_orbiter::<T>("ORBITER", USER_SEED, 20_000);
	}: _(RawOrigin::Signed(orbiter_account), n)
	verify {

	}
	add_collator {
		init::<T>();
		let collator_account: T::AccountId = create_funded_user::<T>("COLLATOR", USER_SEED, 10_000);
		let collator_lookup: <T::Lookup as StaticLookup>::Source =
			T::Lookup::unlookup(collator_account.clone());

	}: _(RawOrigin::Root, collator_lookup.clone())
	verify {

	}
	remove_collator {
		init::<T>();
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

	on_initialize {
		// We make it dependent on the number of collator in the orbiter program
		let x in 0..100;

		init::<T>();

		let round = CurrentRound::<T>::get()
			.saturating_add(T::MaxRoundArchive::get())
			.saturating_add(1u32.into());

		// Force worst case
		<CurrentRound<T>>::put(round);

		let round_to_prune: T::RoundIndex = 1u32.into();
		for i in 0..x {
			let collator_account: T::AccountId = create_funded_user::<T>("COLLATOR", USER_SEED-i, 10_000);
			// It does not rellay matter that the orbiter is the collator for the sake of the benchmark
			<OrbiterPerRound<T>>::insert(round_to_prune, collator_account.clone(), collator_account);
		};
	}: { Pallet::<T>::on_initialize(<frame_system::Pallet<T>>::block_number()); }
	verify {
		let collator_account: T::AccountId = create_funded_user::<T>("COLLATOR", USER_SEED, 10_000);
		assert!(
			<OrbiterPerRound<T>>::get(round_to_prune, collator_account).is_none(), "Should have been removed"
		);

	}

	distribute_rewards {
		init::<T>();

		let round_to_pay: T::RoundIndex = 1u32.into();
		let collator: T::AccountId = create_funded_user::<T>("COLLATOR", USER_SEED, 10_000);
		let orbiter: T::AccountId = create_funded_user::<T>("COLLATOR", USER_SEED -1, 10_000);

		// Worst case, orbiter exists
		<OrbiterPerRound<T>>::insert(round_to_pay, &collator, &orbiter);

	}: { Pallet::<T>::distribute_rewards(round_to_pay, collator.clone(), 1_000u32.into()); }
	verify {
		assert_eq!(
			T::Currency::total_balance(&orbiter), 11_000u32.into()
		);
		assert_eq!(
			T::Currency::total_balance(&collator), 10_000u32.into()
		);

	}

	on_new_round {
		init::<T>();

		// We want to simulate worst case:
		// An orbiter for a collator exists
		// we rotate to a new orbiter
		let round_to_rotate: T::RoundIndex = 1u32.into();

		// Worst case, orbiter exists
		let collator: T::AccountId = create_funded_user::<T>("COLLATOR", USER_SEED, 10_000);
		let old_orbiter: T::AccountId = create_funded_user::<T>("COLLATOR", USER_SEED-1, 10_000);
		let new_orbiter: T::AccountId = create_funded_user::<T>("COLLATOR", USER_SEED-2, 10_000);

		let mut collator_pool_info = CollatorPoolInfo::<T::AccountId>::default();
		collator_pool_info.add_orbiter(old_orbiter.clone());
		collator_pool_info.add_orbiter(new_orbiter.clone());
		// To put old_orbiter in place
		let rotate_result = collator_pool_info.rotate_orbiter();

		// Worst case: forced rotation
		ForceRotation::<T>::put(true);


		assert!(
			rotate_result.maybe_old_orbiter.is_none()
		);

		assert_eq!(
			rotate_result.maybe_next_orbiter, Some(old_orbiter)
		);

		<CollatorsPool<T>>::insert(&collator, &collator_pool_info);

	}: { Pallet::<T>::on_new_round(round_to_rotate); }
	verify {
		assert_eq!(
			AccountLookupOverride::<T>::get(&collator), Some(None)
		);
		assert_eq!(
			AccountLookupOverride::<T>::get(&new_orbiter).expect("must exist"), Some(collator)
		);

	}
}

#[cfg(test)]
mod tests {
	use crate::benchmarks::*;
	use crate::mock::Test;
	use frame_support::assert_ok;
	use parity_scale_codec::Encode;
	use sp_io::TestExternalities;
	use sp_runtime::BuildStorage;

	pub fn new_test_ext() -> TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default()
			.build_storage()
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

	#[test]
	fn bench_on_initialize() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_on_initialize());
		});
	}

	#[test]
	fn bench_distribute_rewards() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_distribute_rewards());
		});
	}

	#[test]
	fn bench_on_new_round() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_on_new_round());
		});
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::benchmarks::tests::new_test_ext(),
	crate::mock::Test
);
