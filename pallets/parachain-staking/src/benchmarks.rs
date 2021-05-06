// Copyright 2019-2020 PureStake Inc.
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
use crate::{Call, Config, Pallet, Range};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelist_account};
use frame_support::traits::{Currency, ReservableCurrency};
use frame_system::RawOrigin;
use sp_runtime::Perbill;

/// Create a funded user.
pub fn create_funded_user<T: Config>(
	string: &'static str,
	n: u32,
	balance_factor: u32,
) -> T::AccountId {
	const SEED: u32 = 0;
	let user = account(string, n, SEED);
	let balance = T::Currency::minimum_balance() * balance_factor.into();
	T::Currency::make_free_balance_be(&user, balance);
	T::Currency::issue(balance);
	user
}

/// Create a funded collator.
pub fn create_funded_collator<T: Config>(
	string: &'static str,
	n: u32,
	balance_factor: u32,
	bond: u32,
) -> Result<T::AccountId, &'static str> {
	const SEED: u32 = 0;
	let user = account(string, n, SEED);
	let balance = T::Currency::minimum_balance() * balance_factor.into();
	T::Currency::make_free_balance_be(&user, balance);
	T::Currency::issue(balance);
	Pallet::<T>::join_candidates(RawOrigin::Signed(user.clone()).into(), bond.into())?;
	Ok(user)
}

const USER_SEED: u32 = 999666;

benchmarks! {
	set_inflation {
		let inflation_range: Range<Perbill> = Range {
			min: Perbill::from_perthousand(1),
			ideal: Perbill::from_perthousand(2),
			max: Perbill::from_perthousand(3),
		};

	}: _(RawOrigin::Root, inflation_range)
	verify {
	}

	join_candidates {
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 100);
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()), 20u32.into())
	verify {
		assert!(Pallet::<T>::is_candidate(&caller));
	}

	leave_candidates {
		let caller: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, 100, 10)?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		// TODO: roll_2_rounds and ensure is_candidate == false
		assert!(Pallet::<T>::collator_state(&caller).unwrap().is_leaving());
	}

	go_offline {
		let caller: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, 100, 10)?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(!Pallet::<T>::collator_state(&caller).unwrap().is_active());
	}

	go_online {
		let caller: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, 100, 10)?;
		Pallet::<T>::go_offline(RawOrigin::Signed(caller.clone()).into())?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(Pallet::<T>::collator_state(&caller).unwrap().is_active());
	}

	candidate_bond_more {
		let caller: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, 100, 10)?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()), 5u32.into())
	verify {
		assert_eq!(T::Currency::reserved_balance(&caller), 15u32.into());
	}

	candidate_bond_less {
		let caller: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, 100, 20)?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()), 5u32.into())
	verify {
		assert_eq!(T::Currency::reserved_balance(&caller), 15u32.into());
	}

	nominate {
		let collator: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, 100, 10)?;
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 100);
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()), collator, 20u32.into())
	verify {
		assert!(Pallet::<T>::is_nominator(&caller));
	}

	leave_nominators {
		let collator: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, 100, 10)?;
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 100);
		Pallet::<T>::nominate(RawOrigin::Signed(caller.clone()).into(), collator, 10u32.into())?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(!Pallet::<T>::is_nominator(&caller));
	}

	revoke_nomination {
		let collator: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, 100, 10)?;
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 100);
		Pallet::<T>::nominate(RawOrigin::Signed(caller.clone()).into(), collator.clone(), 10u32.into())?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()), collator)
	verify {
		assert!(!Pallet::<T>::is_nominator(&caller));
	}

	nominator_bond_more {
		let collator: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, 100, 10)?;
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 100);
		Pallet::<T>::nominate(RawOrigin::Signed(caller.clone()).into(), collator.clone(), 10u32.into())?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()), collator, 10u32.into())
	verify {
		assert_eq!(T::Currency::reserved_balance(&caller), 20u32.into());
	}

	nominator_bond_less {
		let collator: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, 100, 10)?;
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 100);
		Pallet::<T>::nominate(RawOrigin::Signed(caller.clone()).into(), collator.clone(), 10u32.into())?;
		whitelist_account!(caller); // TODO: why is this line necessary, copy pasta-ed
	}: _(RawOrigin::Signed(caller.clone()), collator, 5u32.into())
	verify {
		assert_eq!(T::Currency::reserved_balance(&caller), 5u32.into());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::Test;
	use frame_support::assert_ok;
	use sp_io::TestExternalities;

	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		TestExternalities::new(t)
	}

	#[test]
	fn bench_set_inflation() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_inflation::<Test>());
		});
	}

	#[test]
	fn bench_join_candidates() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_join_candidates::<Test>());
		});
	}

	#[test]
	fn bench_leave_candidates() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_leave_candidates::<Test>());
		});
	}

	#[test]
	fn bench_go_offline() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_go_offline::<Test>());
		});
	}

	#[test]
	fn bench_go_online() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_go_online::<Test>());
		});
	}

	#[test]
	fn bench_candidate_bond_more() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_candidate_bond_more::<Test>());
		});
	}

	#[test]
	fn bench_candidate_bond_less() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_candidate_bond_less::<Test>());
		});
	}

	#[test]
	fn bench_nominate() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_nominate::<Test>());
		});
	}

	#[test]
	fn bench_leave_nominators() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_leave_nominators::<Test>());
		});
	}

	#[test]
	fn bench_revoke_nomination() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_revoke_nomination::<Test>());
		});
	}

	#[test]
	fn bench_nominator_bond_more() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_nominator_bond_more::<Test>());
		});
	}

	#[test]
	fn bench_nominator_bond_less() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_nominator_bond_less::<Test>());
		});
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::benchmarks::tests::new_test_ext(),
	crate::mock::Test
);
