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
use frame_support::traits::{Currency, Get, ReservableCurrency}; // OnInitialize, OnFinalize
use frame_system::RawOrigin;
use sp_runtime::Perbill;
use sp_std::vec::Vec;

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

/// Create a funded collator. Base amount is MinCollatorStk == default_balance but the
/// last parameter `extra` represents how much additional balance is minted to the collator.
fn create_funded_collator<T: Config>(
	string: &'static str,
	n: u32,
	extra: BalanceOf<T>,
) -> Result<T::AccountId, &'static str> {
	let user = create_funded_user::<T>(string, n, extra);
	Pallet::<T>::join_candidates(
		RawOrigin::Signed(user.clone()).into(),
		default_balance::<T>(),
		500u32, // larger than collator candidates are ever expected to be
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

	set_total_selected {}: _(RawOrigin::Root, 100u32)
	verify {
		assert_eq!(Pallet::<T>::total_selected(), 100u32);
	}

	set_max_collator_candidates {}: _(RawOrigin::Root, 500u32)
	verify {
		assert_eq!(Pallet::<T>::candidate_count().max_collator_candidates, 500u32);
	}

	set_collator_commission {}: _(RawOrigin::Root, Perbill::from_percent(33))
	verify {
		assert_eq!(Pallet::<T>::collator_commission(), Perbill::from_percent(33));
	}

	set_blocks_per_round {}: _(RawOrigin::Root, 1200u32)
	verify {
		assert_eq!(Pallet::<T>::round().length, 1200u32);
	}

	force_leave_candidates {
		let (max_candidates, max_nominators) = (
			<<T as Config>::MaxCollatorCandidates as Get<u32>>::get(),
			<<T as Config>::MaxNominatorsPerCollator as Get<u32>>::get(),
		);
		let x in 3..<<T as Config>::MaxCollatorCandidates as Get<u32>>::get();
		let y in 3..<<T as Config>::MaxNominatorsPerCollator as Get<u32>>::get();
		// Worst Case Complexity is removal from an ordered list so \exists full list before call
		for i in 2..x {
			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>("collator", seed, 0u32.into())?;
		}
		let caller: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, 0u32.into())?;
		let mut nominators: Vec<T::AccountId> = Vec::new();
		let mut col_nom_count = 0u32;
		// Worst Case Complexity is also leaving collator that is full of nominations
		for j in 2..y {
			let seed = USER_SEED + j;
			let nominator = create_funded_user::<T>("nominator", seed, 0u32.into());
			let bond = <<T as Config>::MinNominatorStk as Get<BalanceOf<T>>>::get();
			Pallet::<T>::nominate(
				RawOrigin::Signed(nominator.clone()).into(),
				caller.clone(),
				bond,
				col_nom_count,
				0u32
			)?;
			col_nom_count += 1u32;
			nominators.push(nominator.clone());
		}
	}: _(RawOrigin::Root, caller.clone(), max_candidates, max_nominators)
	verify {
		assert!(!Pallet::<T>::is_candidate(&caller));
		// all nominators that were only nominators for this candidate are no longer nominators
		for nom in nominators {
			assert!(!Pallet::<T>::is_nominator(&nom));
		}
	}

	// USER DISPATCHABLES

	join_candidates {
		let max_candidates = <<T as Config>::MaxCollatorCandidates as Get<u32>>::get();
		let x in 3..<<T as Config>::MaxCollatorCandidates as Get<u32>>::get();
		// Worst Case Complexity is insertion into an ordered list so \exists full list before call
		for i in 2..x {
			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>("collator", seed, 0u32.into())?;
		}
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
	}: _(RawOrigin::Signed(caller.clone()), default_balance::<T>(), max_candidates)
	verify {
		assert!(Pallet::<T>::is_candidate(&caller));
	}

	// This call schedules the collator's exit and removes them from the candidate pool
	// -> it retains the self-bond and nominator bonds
	leave_candidates {
		let max_candidates = <<T as Config>::MaxCollatorCandidates as Get<u32>>::get();
		let x in 3..<<T as Config>::MaxCollatorCandidates as Get<u32>>::get();
		// Worst Case Complexity is removal from an ordered list so \exists full list before call
		let mut collator_count = 2u32;
		for i in 2..x {
			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>("collator", seed, 0u32.into())?;
			collator_count += 1u32;
		}
		let caller: T::AccountId = create_funded_collator::<T>("caller", USER_SEED, 0u32.into())?;
	}: _(RawOrigin::Signed(caller.clone()), collator_count, 0u32)
	verify {
		assert!(Pallet::<T>::collator_state(&caller).unwrap().is_leaving());
	}

	go_offline {
		let caller: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, 0u32.into())?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(!Pallet::<T>::collator_state(&caller).unwrap().is_active());
	}

	go_online {
		let caller: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, 0u32.into())?;
		Pallet::<T>::go_offline(RawOrigin::Signed(caller.clone()).into())?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(Pallet::<T>::collator_state(&caller).unwrap().is_active());
	}

	candidate_bond_more {
		let balance = default_balance::<T>();
		let caller: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, balance)?;
	}: _(RawOrigin::Signed(caller.clone()), balance)
	verify {
		let expected_bond = balance * 2u32.into();
		assert_eq!(T::Currency::reserved_balance(&caller), expected_bond);
	}

	candidate_bond_less {
		let balance = default_balance::<T>();
		let caller: T::AccountId = create_funded_collator::<T>("collator", USER_SEED, balance)?;
		Pallet::<T>::candidate_bond_more(RawOrigin::Signed(caller.clone()).into(), balance)?;
	}: _(RawOrigin::Signed(caller.clone()), balance)
	verify {
		assert_eq!(T::Currency::reserved_balance(&caller), balance);
	}

	nominate {
		let max_nominations = <<T as Config>::MaxCollatorsPerNominator as Get<u32>>::get();
		let max_nominators = <<T as Config>::MaxNominatorsPerCollator as Get<u32>>::get();
		let x in 3..<<T as Config>::MaxCollatorsPerNominator as Get<u32>>::get();
		let y in 2..<<T as Config>::MaxNominatorsPerCollator as Get<u32>>::get();
		// Worst Case is full of nominations before calling `nominate`
		let mut collators: Vec<T::AccountId> = Vec::new();
		// Initialize MaxCollatorsPerNominator collator candidates
		for i in 2..x {
			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>("collator", seed, 0u32.into())?;
			collators.push(collator.clone());
		}
		let bond = <<T as Config>::MinNominatorStk as Get<BalanceOf<T>>>::get();
		let extra = if (bond * (collators.len() as u32 + 1u32).into()) > default_balance::<T>() {
			(bond * (collators.len() as u32 + 1u32).into()) - default_balance::<T>()
		} else {
			0u32.into()
		};
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, extra.into());
		// Nomination count
		let mut nom_nom_count = 0u32;
		// Nominate MaxCollatorsPerNominators collator candidates
		for col in collators {
			Pallet::<T>::nominate(
				RawOrigin::Signed(caller.clone()).into(), col, bond, 0u32, nom_nom_count
			)?;
			nom_nom_count += 1u32;
		}
		// Last collator to be nominated
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into()
		)?;
		// Worst Case Complexity is insertion into an almost full collator
		let mut col_nom_count = 0u32;
		for i in 1..y {
			let seed = USER_SEED + i;
			let nominator = create_funded_user::<T>("nominator", seed, 0u32.into());
			Pallet::<T>::nominate(
				RawOrigin::Signed(nominator.clone()).into(),
				collator.clone(),
				bond,
				col_nom_count,
				0u32,
			)?;
			col_nom_count += 1u32;
		}
	}: _(RawOrigin::Signed(caller.clone()), collator, bond, col_nom_count, nom_nom_count)
	verify {
		assert!(Pallet::<T>::is_nominator(&caller));
	}

	leave_nominators {
		let x in 2..<<T as Config>::MaxCollatorsPerNominator as Get<u32>>::get();
		// Worst Case is full of nominations before exit
		let mut collators: Vec<T::AccountId> = Vec::new();
		// Initialize MaxCollatorsPerNominator collator candidates
		for i in 1..x {
			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>("collator", seed, 0u32.into())?;
			collators.push(collator.clone());
		}
		let bond = <<T as Config>::MinNominatorStk as Get<BalanceOf<T>>>::get();
		let need = bond * (collators.len() as u32).into();
		let default_minted = default_balance::<T>();
		let need: BalanceOf<T> = if need > default_minted {
			need - default_minted
		} else {
			0u32.into()
		};
		// Fund the nominator
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, need);
		let nomination_count = collators.len() as u32;
		// Nomination count
		let mut nom_count = 0u32;
		// Nominate MaxCollatorsPerNominators collator candidates
		for col in collators {
			Pallet::<T>::nominate(RawOrigin::Signed(caller.clone()).into(), col, bond, 0u32, nom_count)?;
			nom_count += 1u32;
		}
	}: _(RawOrigin::Signed(caller.clone()), nomination_count)
	verify {
		assert!(!Pallet::<T>::is_nominator(&caller));
	}

	revoke_nomination {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into()
		)?;
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		let bond = <<T as Config>::MinNominatorStk as Get<BalanceOf<T>>>::get();
		Pallet::<T>::nominate(RawOrigin::Signed(caller.clone()).into(), collator.clone(), bond, 0u32, 0u32)?;
	}: _(RawOrigin::Signed(caller.clone()), collator)
	verify {
		assert!(!Pallet::<T>::is_nominator(&caller));
	}

	nominator_bond_more {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into()
		)?;
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		let bond = <<T as Config>::MinNominatorStk as Get<BalanceOf<T>>>::get();
		Pallet::<T>::nominate(RawOrigin::Signed(caller.clone()).into(), collator.clone(), bond, 0u32, 0u32)?;
	}: _(RawOrigin::Signed(caller.clone()), collator, bond)
	verify {
		let expected_bond = bond * 2u32.into();
		assert_eq!(T::Currency::reserved_balance(&caller), expected_bond);
	}

	nominator_bond_less {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into()
		)?;
		let caller: T::AccountId = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		let total = default_balance::<T>();
		Pallet::<T>::nominate(RawOrigin::Signed(caller.clone()).into(), collator.clone(), total, 0u32, 0u32)?;
		let bond_less = <<T as Config>::MinNominatorStk as Get<BalanceOf<T>>>::get();
	}: _(RawOrigin::Signed(caller.clone()), collator, bond_less)
	verify {
		let expected = total - bond_less;
		assert_eq!(T::Currency::reserved_balance(&caller), expected);
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
	fn bench_set_blocks_per_round() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_blocks_per_round::<Test>());
		});
	}

	#[test]
	fn bench_set_collator_commission() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_collator_commission::<Test>());
		});
	}

	#[test]
	fn bench_set_max_collator_candidates() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_max_collator_candidates::<Test>());
		});
	}

	#[test]
	fn bench_set_total_selected() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_total_selected::<Test>());
		});
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
	fn bench_force_leave_candidates() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_force_leave_candidates::<Test>());
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
