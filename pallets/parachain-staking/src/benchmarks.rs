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
use crate::{BalanceOf, Call, Config, Pallet, Range};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::traits::{Currency, Get, OnFinalize, OnInitialize, ReservableCurrency};
use frame_system::RawOrigin;
use nimbus_primitives::EventHandler;
use sp_runtime::{Perbill, Percent};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

/// Default balance amount is minimum collator stake
fn default_balance<T: Config>() -> BalanceOf<T> {
	<<T as Config>::MinCollatorStk as Get<BalanceOf<T>>>::get()
}

/// Create a funded user.
fn create_funded_user<T: Config>(
	string: &'static str,
	n: u32,
	extra: BalanceOf<T>,
) -> T::AccountId {
	const SEED: u32 = 0;
	let user = account(string, n, SEED);
	let default_balance = default_balance::<T>();
	let total = default_balance + extra;
	T::Currency::make_free_balance_be(&user, total);
	T::Currency::issue(total);
	user
}

// /// Create a funded nominator. Base balance is MinCollatorStk == default_balance
// /// but the amount for the nomination is MinNominatorStk << MinCollatorStk => the rest + extra
// /// is free balance for the returned account.
// fn create_funded_nominator<T: Config>(
// 	string: &'static str,
// 	n: u32,
// 	extra: BalanceOf<T>,
// 	collator: T::AccountId,
// 	collator_nominator_count: u32,
// ) -> Result<T::AccountId, &'static str> {
// 	let user = create_funded_user::<T>(string, n, extra);
// 	Pallet::<T>::nominate(
// 		RawOrigin::Signed(user.clone()).into(),
// 		collator,
// 		<<T as Config>::MinNominatorStk as Get<BalanceOf<T>>>::get(),
// 		collator_nominator_count,
// 		0u32, // first nomination for all calls
// 	)?;
// 	Ok(user)
// }

/// Create a funded collator. Base amount is MinCollatorStk == default_balance but the
/// last parameter `extra` represents how much additional balance is minted to the collator.
fn create_funded_collator<T: Config>(
	string: &'static str,
	n: u32,
	extra: BalanceOf<T>,
	collator_count: u32,
) -> Result<T::AccountId, &'static str> {
	let user = create_funded_user::<T>(string, n, extra);
	Pallet::<T>::join_candidates(
		RawOrigin::Signed(user.clone()).into(),
		default_balance::<T>(),
		collator_count,
	)?;
	Ok(user)
}

const USER_SEED: u32 = 999666;

benchmarks! {
	// ROOT DISPATCHABLES

	set_staking_expectations {
		let stake_range: Range<BalanceOf<T>> = Range {
			min: 100u32.into(),
			ideal: 200u32.into(),
			max: 300u32.into(),
		};
	}: _(RawOrigin::Root, stake_range)
	verify {}

	set_inflation {
		let inflation_range: Range<Perbill> = Range {
			min: Perbill::from_perthousand(1),
			ideal: Perbill::from_perthousand(2),
			max: Perbill::from_perthousand(3),
		};

	}: _(RawOrigin::Root, inflation_range)
	verify {}

	set_parachain_bond_account {
		let parachain_bond_account: T::AccountId = account("TEST", 0u32, USER_SEED);
	}: _(RawOrigin::Root, parachain_bond_account.clone())
	verify {
		assert_eq!(Pallet::<T>::parachain_bond_info().account, parachain_bond_account);
	}

	set_parachain_bond_reserve_percent {
	}: _(RawOrigin::Root, Percent::from_percent(33))
	verify {
		assert_eq!(Pallet::<T>::parachain_bond_info().percent, Percent::from_percent(33));
	}

	set_total_selected {}: _(RawOrigin::Root, 100u32)
	verify {
		assert_eq!(Pallet::<T>::total_selected(), 100u32);
	}

	set_collator_commission {}: _(RawOrigin::Root, Perbill::from_percent(33))
	verify {
		assert_eq!(Pallet::<T>::collator_commission(), Perbill::from_percent(33));
	}

	set_blocks_per_round {}: _(RawOrigin::Root, 1200u32)
	verify {
		assert_eq!(Pallet::<T>::round().length, 1200u32);
	}

	// USER DISPATCHABLES

	join_candidates {
		let x in 3..1_000;
		// Worst Case Complexity is insertion into an ordered list so \exists full list before call
		let mut collator_count = 1u32;
		for i in 2..x {
			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>(
				"collator",
				seed,
				0u32.into(),
				collator_count
			)?;
			collator_count += 1u32;
		}
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
	}: _(RawOrigin::Signed(caller.clone()), default_balance::<T>(), collator_count)
	verify {
		assert!(Pallet::<T>::is_candidate(&caller));
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
	fn bench_set_staking_expectations() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_staking_expectations::<Test>());
		});
	}

	#[test]
	fn bench_set_inflation() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_inflation::<Test>());
		});
	}

	#[test]
	fn bench_set_parachain_bond_account() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_parachain_bond_account::<Test>());
		});
	}

	#[test]
	fn bench_set_parachain_bond_reserve_percent() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_parachain_bond_reserve_percent::<Test>());
		});
	}

	#[test]
	fn bench_set_total_selected() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_total_selected::<Test>());
		});
	}

	#[test]
	fn bench_set_collator_commission() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_collator_commission::<Test>());
		});
	}

	#[test]
	fn bench_set_blocks_per_round() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_blocks_per_round::<Test>());
		});
	}

	#[test]
	fn bench_join_candidates() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_join_candidates::<Test>());
		});
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::benchmarks::tests::new_test_ext(),
	crate::mock::Test
);
