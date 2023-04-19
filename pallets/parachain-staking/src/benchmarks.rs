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
use crate::{
	AwardedPts, BalanceOf, Call, CandidateBondLessRequest, Config, DelegationAction, Pallet,
	ParachainBondConfig, ParachainBondInfo, Points, Range, RewardPayment, Round, ScheduledRequest,
	Staked, TopDelegations,
};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, vec};
use frame_support::traits::{Currency, Get, OnFinalize, OnInitialize};
use frame_system::RawOrigin;
use sp_runtime::{Perbill, Percent};
use sp_std::vec::Vec;

/// Minimum collator candidate stake
fn min_candidate_stk<T: Config>() -> BalanceOf<T> {
	<<T as Config>::MinCollatorStk as Get<BalanceOf<T>>>::get()
}

/// Minimum delegator stake
fn min_delegator_stk<T: Config>() -> BalanceOf<T> {
	<<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get()
}

/// Create a funded user.
/// Extra + min_candidate_stk is total minted funds
/// Returns tuple (id, balance)
fn create_funded_user<T: Config>(
	string: &'static str,
	n: u32,
	extra: BalanceOf<T>,
) -> (T::AccountId, BalanceOf<T>) {
	const SEED: u32 = 0;
	let user = account(string, n, SEED);
	let min_candidate_stk = min_candidate_stk::<T>();
	let total = min_candidate_stk + extra;
	T::Currency::make_free_balance_be(&user, total);
	T::Currency::issue(total);
	(user, total)
}

/// Create a funded delegator.
fn create_funded_delegator<T: Config>(
	string: &'static str,
	n: u32,
	extra: BalanceOf<T>,
	collator: T::AccountId,
	min_bond: bool,
	collator_delegator_count: u32,
) -> Result<T::AccountId, &'static str> {
	let (user, total) = create_funded_user::<T>(string, n, extra);
	let bond = if min_bond {
		min_delegator_stk::<T>()
	} else {
		total
	};
	Pallet::<T>::delegate(
		RawOrigin::Signed(user.clone()).into(),
		collator,
		bond,
		collator_delegator_count,
		0u32, // first delegation for all calls
	)?;
	Ok(user)
}

/// Create a funded collator.
fn create_funded_collator<T: Config>(
	string: &'static str,
	n: u32,
	extra: BalanceOf<T>,
	min_bond: bool,
	candidate_count: u32,
) -> Result<T::AccountId, &'static str> {
	let (user, total) = create_funded_user::<T>(string, n, extra);
	let bond = if min_bond {
		min_candidate_stk::<T>()
	} else {
		total
	};
	Pallet::<T>::join_candidates(
		RawOrigin::Signed(user.clone()).into(),
		bond,
		candidate_count,
	)?;
	Ok(user)
}

// Simulate staking on finalize by manually setting points
fn parachain_staking_on_finalize<T: Config>(author: T::AccountId) {
	let now = <Round<T>>::get().current;
	let score_plus_20 = <AwardedPts<T>>::get(now, &author).saturating_add(20);
	<AwardedPts<T>>::insert(now, author, score_plus_20);
	<Points<T>>::mutate(now, |x| *x = x.saturating_add(20));
}

/// Run to end block and author
fn roll_to_and_author<T: Config>(round_delay: u32, author: T::AccountId) {
	let total_rounds = round_delay + 1u32;
	let round_length: T::BlockNumber = Pallet::<T>::round().length.into();
	let mut now = <frame_system::Pallet<T>>::block_number() + 1u32.into();
	let end = Pallet::<T>::round().first + (round_length * total_rounds.into());
	while now < end {
		parachain_staking_on_finalize::<T>(author.clone());
		<frame_system::Pallet<T>>::on_finalize(<frame_system::Pallet<T>>::block_number());
		<frame_system::Pallet<T>>::set_block_number(
			<frame_system::Pallet<T>>::block_number() + 1u32.into(),
		);
		<frame_system::Pallet<T>>::on_initialize(<frame_system::Pallet<T>>::block_number());
		Pallet::<T>::on_initialize(<frame_system::Pallet<T>>::block_number());
		now += 1u32.into();
	}
}

const USER_SEED: u32 = 999666;
struct Seed {
	pub inner: u32,
}
impl Seed {
	fn new() -> Self {
		Seed { inner: USER_SEED }
	}

	pub fn take(&mut self) -> u32 {
		let v = self.inner;
		self.inner += 1;
		v
	}
}

benchmarks! {
	// MONETARY ORIGIN DISPATCHABLES
	set_staking_expectations {
		let stake_range: Range<BalanceOf<T>> = Range {
			min: 100u32.into(),
			ideal: 200u32.into(),
			max: 300u32.into(),
		};
	}: _(RawOrigin::Root, stake_range)
	verify {
		assert_eq!(Pallet::<T>::inflation_config().expect, stake_range);
	}

	set_inflation {
		let inflation_range: Range<Perbill> = Range {
			min: Perbill::from_perthousand(1),
			ideal: Perbill::from_perthousand(2),
			max: Perbill::from_perthousand(3),
		};

	}: _(RawOrigin::Root, inflation_range)
	verify {
		assert_eq!(Pallet::<T>::inflation_config().annual, inflation_range);
	}

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

	// ROOT DISPATCHABLES

	set_total_selected {
		Pallet::<T>::set_blocks_per_round(RawOrigin::Root.into(), 101u32)?;
	}: _(RawOrigin::Root, 100u32)
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
		let mut candidate_count = 1u32;
		for i in 2..x {
			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>(
				"collator",
				seed,
				0u32.into(),
				true,
				candidate_count
			)?;
			candidate_count += 1u32;
		}
		let (caller, min_candidate_stk) = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
	}: _(RawOrigin::Signed(caller.clone()), min_candidate_stk, candidate_count)
	verify {
		assert!(Pallet::<T>::is_candidate(&caller));
	}

	// This call schedules the collator's exit and removes them from the candidate pool
	// -> it retains the self-bond and delegator bonds
	schedule_leave_candidates {
		let x in 3..1_000;
		// Worst Case Complexity is removal from an ordered list so \exists full list before call
		let mut candidate_count = 1u32;
		for i in 2..x {
			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>(
				"collator",
				seed,
				0u32.into(),
				true,
				candidate_count
			)?;
			candidate_count += 1u32;
		}
		let caller: T::AccountId = create_funded_collator::<T>(
			"caller",
			USER_SEED,
			0u32.into(),
			true,
			candidate_count,
		)?;
		candidate_count += 1u32;
	}: _(RawOrigin::Signed(caller.clone()), candidate_count)
	verify {
		assert!(Pallet::<T>::candidate_info(&caller).unwrap().is_leaving());
	}

	execute_leave_candidates {
		// x is total number of delegations for the candidate
		let x in 2..(<<T as Config>::MaxTopDelegationsPerCandidate as Get<u32>>::get()
		+ <<T as Config>::MaxBottomDelegationsPerCandidate as Get<u32>>::get());
		let candidate: T::AccountId = create_funded_collator::<T>(
			"unique_caller",
			USER_SEED - 100,
			0u32.into(),
			true,
			1u32,
		)?;
		// 2nd delegation required for all delegators to ensure DelegatorState updated not removed
		let second_candidate: T::AccountId = create_funded_collator::<T>(
			"unique__caller",
			USER_SEED - 99,
			0u32.into(),
			true,
			2u32,
		)?;
		let mut delegators: Vec<T::AccountId> = Vec::new();
		let mut col_del_count = 0u32;
		for i in 1..x {
			let seed = USER_SEED + i;
			let delegator = create_funded_delegator::<T>(
				"delegator",
				seed,
				min_delegator_stk::<T>(),
				candidate.clone(),
				true,
				col_del_count,
			)?;
			Pallet::<T>::delegate(
				RawOrigin::Signed(delegator.clone()).into(),
				second_candidate.clone(),
				min_delegator_stk::<T>(),
				col_del_count,
				1u32,
			)?;
			Pallet::<T>::schedule_revoke_delegation(
				RawOrigin::Signed(delegator.clone()).into(),
				candidate.clone()
			)?;
			delegators.push(delegator);
			col_del_count += 1u32;
		}
		Pallet::<T>::schedule_leave_candidates(
			RawOrigin::Signed(candidate.clone()).into(),
			3u32
		)?;
		roll_to_and_author::<T>(2, candidate.clone());
	}: _(RawOrigin::Signed(candidate.clone()), candidate.clone(), col_del_count)
	verify {
		assert!(Pallet::<T>::candidate_info(&candidate).is_none());
		assert!(Pallet::<T>::candidate_info(&second_candidate).is_some());
		for delegator in delegators {
			assert!(Pallet::<T>::is_delegator(&delegator));
		}
	}

	cancel_leave_candidates {
		let x in 3..1_000;
		// Worst Case Complexity is removal from an ordered list so \exists full list before call
		let mut candidate_count = 1u32;
		for i in 2..x {
			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>(
				"collator",
				seed,
				0u32.into(),
				true,
				candidate_count
			)?;
			candidate_count += 1u32;
		}
		let caller: T::AccountId = create_funded_collator::<T>(
			"caller",
			USER_SEED,
			0u32.into(),
			true,
			candidate_count,
		)?;
		candidate_count += 1u32;
		Pallet::<T>::schedule_leave_candidates(
			RawOrigin::Signed(caller.clone()).into(),
			candidate_count
		)?;
		candidate_count -= 1u32;
	}: _(RawOrigin::Signed(caller.clone()), candidate_count)
	verify {
		assert!(Pallet::<T>::candidate_info(&caller).unwrap().is_active());
	}

	go_offline {
		let caller: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into(),
			true,
			1u32
		)?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(!Pallet::<T>::candidate_info(&caller).unwrap().is_active());
	}

	go_online {
		let caller: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into(),
			true,
			1u32
		)?;
		Pallet::<T>::go_offline(RawOrigin::Signed(caller.clone()).into())?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(Pallet::<T>::candidate_info(&caller).unwrap().is_active());
	}

	candidate_bond_more {
		let more = min_candidate_stk::<T>();
		let caller: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			more,
			true,
			1u32,
		)?;
	}: _(RawOrigin::Signed(caller.clone()), more)
	verify {
		let expected_bond = more * 2u32.into();
		assert_eq!(
			Pallet::<T>::candidate_info(&caller).expect("candidate was created, qed").bond,
			expected_bond,
		);
	}

	schedule_candidate_bond_less {
		let min_candidate_stk = min_candidate_stk::<T>();
		let caller: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			min_candidate_stk,
			false,
			1u32,
		)?;
	}: _(RawOrigin::Signed(caller.clone()), min_candidate_stk)
	verify {
		let state = Pallet::<T>::candidate_info(&caller).expect("request bonded less so exists");
		assert_eq!(
			state.request,
			Some(CandidateBondLessRequest {
				amount: min_candidate_stk,
				when_executable: 3,
			})
		);
	}

	execute_candidate_bond_less {
		let min_candidate_stk = min_candidate_stk::<T>();
		let caller: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			min_candidate_stk,
			false,
			1u32,
		)?;
		Pallet::<T>::schedule_candidate_bond_less(
			RawOrigin::Signed(caller.clone()).into(),
			min_candidate_stk
		)?;
		roll_to_and_author::<T>(2, caller.clone());
	}: {
		Pallet::<T>::execute_candidate_bond_less(
			RawOrigin::Signed(caller.clone()).into(),
			caller.clone()
		)?;
	} verify {
		assert_eq!(
			Pallet::<T>::candidate_info(&caller).expect("candidate was created, qed").bond,
			min_candidate_stk,
		);
	}

	cancel_candidate_bond_less {
		let min_candidate_stk = min_candidate_stk::<T>();
		let caller: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			min_candidate_stk,
			false,
			1u32,
		)?;
		Pallet::<T>::schedule_candidate_bond_less(
			RawOrigin::Signed(caller.clone()).into(),
			min_candidate_stk
		)?;
	}: {
		Pallet::<T>::cancel_candidate_bond_less(
			RawOrigin::Signed(caller.clone()).into(),
		)?;
	} verify {
		assert!(
			Pallet::<T>::candidate_info(&caller).unwrap().request.is_none()
		);
	}

	delegate {
		let x in 3..<<T as Config>::MaxDelegationsPerDelegator as Get<u32>>::get();
		let y in 2..<<T as Config>::MaxTopDelegationsPerCandidate as Get<u32>>::get();
		// Worst Case is full of delegations before calling `delegate`
		let mut collators: Vec<T::AccountId> = Vec::new();
		// Initialize MaxDelegationsPerDelegator collator candidates
		for i in 2..x {
			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>(
				"collator",
				seed,
				0u32.into(),
				true,
				collators.len() as u32 + 1u32,
			)?;
			collators.push(collator.clone());
		}
		let bond = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
		let extra = if (bond * (collators.len() as u32 + 1u32).into()) > min_candidate_stk::<T>() {
			(bond * (collators.len() as u32 + 1u32).into()) - min_candidate_stk::<T>()
		} else {
			0u32.into()
		};
		let (caller, _) = create_funded_user::<T>("caller", USER_SEED, extra.into());
		// Delegation count
		let mut del_del_count = 0u32;
		// Nominate MaxDelegationsPerDelegators collator candidates
		for col in collators.clone() {
			Pallet::<T>::delegate(
				RawOrigin::Signed(caller.clone()).into(), col, bond, 0u32, del_del_count
			)?;
			del_del_count += 1u32;
		}
		// Last collator to be delegated
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into(),
			true,
			collators.len() as u32 + 1u32,
		)?;
		// Worst Case Complexity is insertion into an almost full collator
		let mut col_del_count = 0u32;
		for i in 1..y {
			let seed = USER_SEED + i;
			let _ = create_funded_delegator::<T>(
				"delegator",
				seed,
				0u32.into(),
				collator.clone(),
				true,
				col_del_count,
			)?;
			col_del_count += 1u32;
		}
	}: _(RawOrigin::Signed(caller.clone()), collator, bond, col_del_count, del_del_count)
	verify {
		assert!(Pallet::<T>::is_delegator(&caller));
	}

	schedule_leave_delegators {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into(),
			true,
			1u32
		)?;
		let (caller, _) = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		let bond = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
		Pallet::<T>::delegate(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone(),
			bond,
			0u32,
			0u32
		)?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(
			Pallet::<T>::delegation_scheduled_requests(&collator)
				.iter()
				.any(|r| r.delegator == caller && matches!(r.action, DelegationAction::Revoke(_)))
		);
	}

	execute_leave_delegators {
		let x in 2..<<T as Config>::MaxDelegationsPerDelegator as Get<u32>>::get();
		// Worst Case is full of delegations before execute exit
		let mut collators: Vec<T::AccountId> = Vec::new();
		// Initialize MaxDelegationsPerDelegator collator candidates
		for i in 1..x {
			let seed = USER_SEED - i;
			let collator = create_funded_collator::<T>(
				"collator",
				seed,
				0u32.into(),
				true,
				collators.len() as u32 + 1u32
			)?;
			collators.push(collator.clone());
		}
		let bond = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
		let need = bond * (collators.len() as u32).into();
		let default_minted = min_candidate_stk::<T>();
		let need: BalanceOf<T> = if need > default_minted {
			need - default_minted
		} else {
			0u32.into()
		};
		// Fund the delegator
		let (caller, _) = create_funded_user::<T>("caller", USER_SEED, need);
		// Delegation count
		let mut delegation_count = 0u32;
		let author = collators[0].clone();
		// Nominate MaxDelegationsPerDelegators collator candidates
		for col in collators {
			Pallet::<T>::delegate(
				RawOrigin::Signed(caller.clone()).into(),
				col,
				bond,
				0u32,
				delegation_count
			)?;
			delegation_count += 1u32;
		}
		Pallet::<T>::schedule_leave_delegators(RawOrigin::Signed(caller.clone()).into())?;
		roll_to_and_author::<T>(2, author);
	}: _(RawOrigin::Signed(caller.clone()), caller.clone(), delegation_count)
	verify {
		assert!(Pallet::<T>::delegator_state(&caller).is_none());
	}

	cancel_leave_delegators {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into(),
			true,
			1u32
		)?;
		let (caller, _) = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		let bond = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
		Pallet::<T>::delegate(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone(),
			bond,
			0u32,
			0u32
		)?;
		Pallet::<T>::schedule_leave_delegators(RawOrigin::Signed(caller.clone()).into())?;
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(Pallet::<T>::delegator_state(&caller).unwrap().is_active());
	}

	schedule_revoke_delegation {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into(),
			true,
			1u32
		)?;
		let (caller, _) = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		let bond = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
		Pallet::<T>::delegate(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone(),
			bond,
			0u32,
			0u32
		)?;
	}: _(RawOrigin::Signed(caller.clone()), collator.clone())
	verify {
		assert_eq!(
			Pallet::<T>::delegation_scheduled_requests(&collator),
			vec![ScheduledRequest {
				delegator: caller,
				when_executable: 3,
				action: DelegationAction::Revoke(bond),
			}],
		);
	}

	delegator_bond_more {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into(),
			true,
			1u32
		)?;
		let (caller, _) = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		let bond = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
		Pallet::<T>::delegate(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone(),
			bond,
			0u32,
			0u32
		)?;
	}: _(RawOrigin::Signed(caller.clone()), collator.clone(), bond)
	verify {
		let expected_bond = bond * 2u32.into();
		assert_eq!(
			Pallet::<T>::delegator_state(&caller).expect("candidate was created, qed").total,
			expected_bond,
		);
	}

	schedule_delegator_bond_less {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into(),
			true,
			1u32
		)?;
		let (caller, total) = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		Pallet::<T>::delegate(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone(),
			total,
			0u32,
			0u32
		)?;
		let bond_less = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
	}: _(RawOrigin::Signed(caller.clone()), collator.clone(), bond_less)
	verify {
		let state = Pallet::<T>::delegator_state(&caller)
			.expect("just request bonded less so exists");
		assert_eq!(
			Pallet::<T>::delegation_scheduled_requests(&collator),
			vec![ScheduledRequest {
				delegator: caller,
				when_executable: 3,
				action: DelegationAction::Decrease(bond_less),
			}],
		);
	}

	execute_revoke_delegation {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into(),
			true,
			1u32
		)?;
		let (caller, _) = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		let bond = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
		Pallet::<T>::delegate(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone(),
			bond,
			0u32,
			0u32
		)?;
		Pallet::<T>::schedule_revoke_delegation(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone()
		)?;
		roll_to_and_author::<T>(2, collator.clone());
	}: {
		Pallet::<T>::execute_delegation_request(
			RawOrigin::Signed(caller.clone()).into(),
			caller.clone(),
			collator.clone()
		)?;
	} verify {
		assert!(
			!Pallet::<T>::is_delegator(&caller)
		);
	}

	execute_delegator_bond_less {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into(),
			true,
			1u32
		)?;
		let (caller, total) = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		Pallet::<T>::delegate(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone(),
			total,
			0u32,
			0u32
		)?;
		let bond_less = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
		Pallet::<T>::schedule_delegator_bond_less(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone(),
			bond_less
		)?;
		roll_to_and_author::<T>(2, collator.clone());
	}: {
		Pallet::<T>::execute_delegation_request(
			RawOrigin::Signed(caller.clone()).into(),
			caller.clone(),
			collator.clone()
		)?;
	} verify {
		let expected = total - bond_less;
		assert_eq!(
			Pallet::<T>::delegator_state(&caller).expect("candidate was created, qed").total,
			expected,
		);
	}

	cancel_revoke_delegation {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into(),
			true,
			1u32
		)?;
		let (caller, _) = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		let bond = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
		Pallet::<T>::delegate(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone(),
			bond,
			0u32,
			0u32
		)?;
		Pallet::<T>::schedule_revoke_delegation(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone()
		)?;
	}: {
		Pallet::<T>::cancel_delegation_request(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone()
		)?;
	} verify {
		assert!(
			!Pallet::<T>::delegation_scheduled_requests(&collator)
			.iter()
			.any(|x| &x.delegator == &caller)
		);
	}

	cancel_delegator_bond_less {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into(),
			true,
			1u32
		)?;
		let (caller, total) = create_funded_user::<T>("caller", USER_SEED, 0u32.into());
		Pallet::<T>::delegate(RawOrigin::Signed(
			caller.clone()).into(),
			collator.clone(),
			total,
			0u32,
			0u32
		)?;
		let bond_less = <<T as Config>::MinDelegatorStk as Get<BalanceOf<T>>>::get();
		Pallet::<T>::schedule_delegator_bond_less(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone(),
			bond_less
		)?;
		roll_to_and_author::<T>(2, collator.clone());
	}: {
		Pallet::<T>::cancel_delegation_request(
			RawOrigin::Signed(caller.clone()).into(),
			collator.clone()
		)?;
	} verify {
		assert!(
			!Pallet::<T>::delegation_scheduled_requests(&collator)
				.iter()
				.any(|x| &x.delegator == &caller)
		);
	}

	// ON_INITIALIZE

	prepare_staking_payouts {
		let reward_delay = <<T as Config>::RewardPaymentDelay as Get<u32>>::get();
		let round = reward_delay + 2u32;
		let payout_round = round - reward_delay;
		// may need:
		//  <Points<T>>
		//  <Staked<T>>
		//  <ParachainBondInfo<T>>
		//  ensure parachain bond account exists so that deposit_into_existing succeeds
		<Points<T>>::insert(payout_round, 100);
		<Staked<T>>::insert(payout_round, min_candidate_stk::<T>());

		// set an account in the bond config so that we will measure the payout to it
		let account = create_funded_user::<T>(
			"parachain_bond",
			0,
			min_candidate_stk::<T>(),
		).0;
		<ParachainBondInfo<T>>::put(ParachainBondConfig {
			account,
			percent: Percent::from_percent(50),
		});

	}: { Pallet::<T>::prepare_staking_payouts(round); }
	verify {
	}

	get_rewardable_delegators {
		let y in 0..<<T as Config>::MaxDelegationsPerDelegator as Get<u32>>::get(); // num delegators

		let high_inflation: Range<Perbill> = Range {
			min: Perbill::one(),
			ideal: Perbill::one(),
			max: Perbill::one(),
		};
		Pallet::<T>::set_inflation(RawOrigin::Root.into(), high_inflation.clone())?;
		Pallet::<T>::set_blocks_per_round(RawOrigin::Root.into(), 101u32)?;
		Pallet::<T>::set_total_selected(RawOrigin::Root.into(), 100u32)?;

		let collator = create_funded_collator::<T>(
			"collator",
			0,
			min_candidate_stk::<T>() * 1_000_000u32.into(),
			true,
			1,
		)?;

		// create delegators
		for i in 0..y {
			let seed = USER_SEED + i + 1;
			let delegator = create_funded_delegator::<T>(
				"delegator",
				seed,
				min_candidate_stk::<T>() * 1_000_000u32.into(),
				collator.clone(),
				true,
				i,
			)?;
		}

		let mut _results = None;

	}: { _results = Some(Pallet::<T>::get_rewardable_delegators(&collator)); }
	verify {
		let counted_delegations = _results.expect("get_rewardable_delegators returned some results");
		assert!(counted_delegations.uncounted_stake == 0u32.into());
		assert!(counted_delegations.rewardable_delegations.len() as u32 == y);
		let top_delegations = <TopDelegations<T>>::get(collator.clone())
			.expect("delegations were set for collator through delegate() calls");
		assert!(top_delegations.delegations.len() as u32 == y);
	}

	select_top_candidates {
		let x in 0..50; // num collators
		let y in 0..<<T as Config>::MaxDelegationsPerDelegator as Get<u32>>::get(); // num delegators

		let high_inflation: Range<Perbill> = Range {
			min: Perbill::one(),
			ideal: Perbill::one(),
			max: Perbill::one(),
		};
		Pallet::<T>::set_inflation(RawOrigin::Root.into(), high_inflation.clone())?;
		Pallet::<T>::set_blocks_per_round(RawOrigin::Root.into(), 101u32)?;
		Pallet::<T>::set_total_selected(RawOrigin::Root.into(), 100u32)?;

		let mut seed = USER_SEED + 1;

		for _ in 0..x {
			let collator = create_funded_collator::<T>(
				"collator",
				seed,
				min_candidate_stk::<T>() * 1_000_000u32.into(),
				true,
				999999,
			)?;
			seed += 1;

			// create delegators
			for _ in 0..y {
				let delegator = create_funded_delegator::<T>(
					"delegator",
					seed,
					min_candidate_stk::<T>() * 1_000_000u32.into(),
					collator.clone(),
					true,
					9999999,
				)?;
				seed += 1;
			}
		}

	}: { Some(Pallet::<T>::select_top_candidates(1)); }
	verify {
	}

	pay_one_collator_reward {
		// y controls number of delegations, its maximum per collator is the max top delegations
		let y in 0..<<T as Config>::MaxTopDelegationsPerCandidate as Get<u32>>::get();

		// must come after 'let foo in 0..` statements for macro
		use crate::{
			DelayedPayout, DelayedPayouts, AtStake, CollatorSnapshot, BondWithAutoCompound, Points,
			AwardedPts,
		};

		let before_running_round_index = Pallet::<T>::round().current;
		let initial_stake_amount = min_candidate_stk::<T>() * 1_000_000u32.into();

		let mut total_staked = 0u32.into();

		// initialize our single collator
		let sole_collator = create_funded_collator::<T>(
			"collator",
			0,
			initial_stake_amount,
			true,
			1u32,
		)?;
		total_staked += initial_stake_amount;

		// generate funded collator accounts
		let mut delegators: Vec<T::AccountId> = Vec::new();
		for i in 0..y {
			let seed = USER_SEED + i;
			let delegator = create_funded_delegator::<T>(
				"delegator",
				seed,
				initial_stake_amount,
				sole_collator.clone(),
				true,
				delegators.len() as u32,
			)?;
			delegators.push(delegator);
			total_staked += initial_stake_amount;
		}

		// rather than roll through rounds in order to initialize the storage we want, we set it
		// directly and then call pay_one_collator_reward directly.

		let round_for_payout = 5;
		<DelayedPayouts<T>>::insert(&round_for_payout, DelayedPayout {
			// NOTE: round_issuance is not correct here, but it doesn't seem to cause problems
			round_issuance: 1000u32.into(),
			total_staking_reward: total_staked,
			collator_commission: Perbill::from_rational(1u32, 100u32),
		});

		let mut delegations: Vec<BondWithAutoCompound<T::AccountId, BalanceOf<T>>> = Vec::new();
		for delegator in &delegators {
			delegations.push(BondWithAutoCompound {
				owner: delegator.clone(),
				amount: 100u32.into(),
				auto_compound: Percent::zero(),
			});
		}

		<AtStake<T>>::insert(round_for_payout, &sole_collator, CollatorSnapshot {
			bond: 1_000u32.into(),
			delegations,
			total: 1_000_000u32.into(),
		});

		<Points<T>>::insert(round_for_payout, 100);
		<AwardedPts<T>>::insert(round_for_payout, &sole_collator, 20);

	}: {
		let round_for_payout = 5;
		// TODO: this is an extra read right here (we should whitelist it?)
		let payout_info = Pallet::<T>::delayed_payouts(round_for_payout).expect("payout expected");
		let result = Pallet::<T>::pay_one_collator_reward(round_for_payout, payout_info);
		// TODO: how to keep this in scope so it can be done in verify block?
		assert!(matches!(result.0, RewardPayment::Paid));
	}
	verify {
		// collator should have been paid
		assert!(
			T::Currency::free_balance(&sole_collator) > initial_stake_amount,
			"collator should have been paid in pay_one_collator_reward"
		);
		// nominators should have been paid
		for delegator in &delegators {
			assert!(
				T::Currency::free_balance(&delegator) > initial_stake_amount,
				"delegator should have been paid in pay_one_collator_reward"
			);
		}
	}

	base_on_initialize {
		let collator: T::AccountId = create_funded_collator::<T>(
			"collator",
			USER_SEED,
			0u32.into(),
			true,
			1u32
		)?;
		let start = <frame_system::Pallet<T>>::block_number();
		parachain_staking_on_finalize::<T>(collator.clone());
		<frame_system::Pallet<T>>::on_finalize(start);
		<frame_system::Pallet<T>>::set_block_number(
			start + 1u32.into()
		);
		let end = <frame_system::Pallet<T>>::block_number();
		<frame_system::Pallet<T>>::on_initialize(end);
	}: { Pallet::<T>::on_initialize(end); }
	verify {
		// Round transitions
		assert_eq!(start + 1u32.into(), end);
	}

	set_auto_compound {
		// x controls number of distinct auto-compounding delegations the prime collator will have
		// y controls number of distinct delegations the prime delegator will have
		let x in 0..<<T as Config>::MaxTopDelegationsPerCandidate as Get<u32>>::get();
		let y in 0..<<T as Config>::MaxDelegationsPerDelegator as Get<u32>>::get();

		use crate::auto_compound::AutoCompoundDelegations;

		let min_candidate_stake = min_candidate_stk::<T>();
		let min_delegator_stake = min_delegator_stk::<T>();
		let mut seed = Seed::new();

		// initialize the prime collator
		let prime_candidate = create_funded_collator::<T>(
			"collator",
			seed.take(),
			min_candidate_stake,
			true,
			1,
		)?;

		// initialize the prime delegator
		let prime_delegator = create_funded_delegator::<T>(
			"delegator",
			seed.take(),
			min_delegator_stake * (y+1).into(),
			prime_candidate.clone(),
			true,
			0,
		)?;

		// have x-1 distinct auto-compounding delegators delegate to prime collator
		// we directly set the storage, since benchmarks don't work when the same extrinsic is
		// called from within the benchmark.
		let mut auto_compounding_state = <AutoCompoundDelegations<T>>::get_storage(&prime_candidate);
		for i in 1..x {
			let delegator = create_funded_delegator::<T>(
				"delegator",
				seed.take(),
				min_delegator_stake,
				prime_candidate.clone(),
				true,
				i,
			)?;
			auto_compounding_state.set_for_delegator(
				delegator,
				Percent::from_percent(100),
			);
		}
		auto_compounding_state.set_storage(&prime_candidate);

		// delegate to y-1 distinct collators from the prime delegator
		for i in 1..y {
			let collator = create_funded_collator::<T>(
				"collator",
				seed.take(),
				min_candidate_stake,
				true,
				i+1,
			)?;
			Pallet::<T>::delegate(
				RawOrigin::Signed(prime_delegator.clone()).into(),
				collator,
				min_delegator_stake,
				0,
				i,
			)?;
		}
	}: {
		Pallet::<T>::set_auto_compound(
			RawOrigin::Signed(prime_delegator.clone()).into(),
			prime_candidate.clone(),
			Percent::from_percent(50),
			x,
			y+1,
		)?;
	}
	verify {
		let actual_auto_compound = <AutoCompoundDelegations<T>>::get_storage(&prime_candidate)
			.get_for_delegator(&prime_delegator);
		let expected_auto_compound = Some(Percent::from_percent(50));
		assert_eq!(
			expected_auto_compound,
			actual_auto_compound,
			"delegation must have an auto-compound entry",
		);
	}

	delegate_with_auto_compound {
		// x controls number of distinct delegations the prime collator will have
		// y controls number of distinct auto-compounding delegations the prime collator will have
		// z controls number of distinct delegations the prime delegator will have
		let x in 0..(<<T as Config>::MaxTopDelegationsPerCandidate as Get<u32>>::get()
		+ <<T as Config>::MaxBottomDelegationsPerCandidate as Get<u32>>::get());
		let y in 0..<<T as Config>::MaxTopDelegationsPerCandidate as Get<u32>>::get()
		+ <<T as Config>::MaxBottomDelegationsPerCandidate as Get<u32>>::get();
		let z in 0..<<T as Config>::MaxDelegationsPerDelegator as Get<u32>>::get();

		use crate::auto_compound::AutoCompoundDelegations;

		let min_candidate_stake = min_candidate_stk::<T>();
		let min_delegator_stake = min_delegator_stk::<T>();
		let mut seed = Seed::new();

		// initialize the prime collator
		let prime_candidate = create_funded_collator::<T>(
			"collator",
			seed.take(),
			min_candidate_stake,
			true,
			1,
		)?;

		// initialize the future delegator
		let (prime_delegator, _) = create_funded_user::<T>(
			"delegator",
			seed.take(),
			min_delegator_stake * (z+1).into(),
		);

		// have x-1 distinct delegators delegate to prime collator, of which y are auto-compounding.
		// we can directly set the storage here.
		let auto_compound_z = x * y / 100;
		for i in 1..x {
			let delegator = create_funded_delegator::<T>(
				"delegator",
				seed.take(),
				min_delegator_stake,
				prime_candidate.clone(),
				true,
				i,
			)?;
			if i <= y {
				Pallet::<T>::set_auto_compound(
					RawOrigin::Signed(delegator.clone()).into(),
					prime_candidate.clone(),
					Percent::from_percent(100),
					i+1,
					i,
				)?;
			}
		}

		// delegate to z-1 distinct collators from the prime delegator
		for i in 1..z {
			let collator = create_funded_collator::<T>(
				"collator",
				seed.take(),
				min_candidate_stake,
				true,
				i+1,
			)?;
			Pallet::<T>::delegate(
				RawOrigin::Signed(prime_delegator.clone()).into(),
				collator,
				min_delegator_stake,
				0,
				i,
			)?;
		}
	}: {
		Pallet::<T>::delegate_with_auto_compound(
			RawOrigin::Signed(prime_delegator.clone()).into(),
			prime_candidate.clone(),
			min_delegator_stake,
			Percent::from_percent(50),
			x,
			y,
			z,
		)?;
	}
	verify {
		assert!(Pallet::<T>::is_delegator(&prime_delegator));
		let actual_auto_compound = <AutoCompoundDelegations<T>>::get_storage(&prime_candidate)
			.get_for_delegator(&prime_delegator);
		let expected_auto_compound = Some(Percent::from_percent(50));
		assert_eq!(
			expected_auto_compound,
			actual_auto_compound,
			"delegation must have an auto-compound entry",
		);
	}

	mint_collator_reward {
		let mut seed = Seed::new();
		let collator = create_funded_collator::<T>(
			"collator",
			seed.take(),
			0u32.into(),
			true,
			1,
		)?;
		let original_free_balance = T::Currency::free_balance(&collator);
	}: {
		Pallet::<T>::mint_collator_reward(1u32.into(), collator.clone(), 50u32.into())
	}
	verify {
		assert_eq!(T::Currency::free_balance(&collator), original_free_balance + 50u32.into());
	}
}

#[cfg(test)]
mod tests {
	use crate::benchmarks::*;
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
			assert_ok!(Pallet::<Test>::test_benchmark_set_staking_expectations());
		});
	}

	#[test]
	fn bench_set_inflation() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_set_inflation());
		});
	}

	#[test]
	fn bench_set_parachain_bond_account() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_set_parachain_bond_account());
		});
	}

	#[test]
	fn bench_set_parachain_bond_reserve_percent() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_set_parachain_bond_reserve_percent());
		});
	}

	#[test]
	fn bench_set_total_selected() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_set_total_selected());
		});
	}

	#[test]
	fn bench_set_collator_commission() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_set_collator_commission());
		});
	}

	#[test]
	fn bench_set_blocks_per_round() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_set_blocks_per_round());
		});
	}

	#[test]
	fn bench_join_candidates() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_join_candidates());
		});
	}

	#[test]
	fn bench_schedule_leave_candidates() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_schedule_leave_candidates());
		});
	}

	#[test]
	fn bench_execute_leave_candidates() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_execute_leave_candidates());
		});
	}

	#[test]
	fn bench_cancel_leave_candidates() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_cancel_leave_candidates());
		});
	}

	#[test]
	fn bench_go_offline() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_go_offline());
		});
	}

	#[test]
	fn bench_go_online() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_go_online());
		});
	}

	#[test]
	fn bench_candidate_bond_more() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_candidate_bond_more());
		});
	}

	#[test]
	fn bench_schedule_candidate_bond_less() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_schedule_candidate_bond_less());
		});
	}

	#[test]
	fn bench_execute_candidate_bond_less() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_execute_candidate_bond_less());
		});
	}

	#[test]
	fn bench_cancel_candidate_bond_less() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_cancel_candidate_bond_less());
		});
	}

	#[test]
	fn bench_delegate() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_delegate());
		});
	}

	#[test]
	fn bench_schedule_leave_delegators() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_schedule_leave_delegators());
		});
	}

	#[test]
	fn bench_execute_leave_delegators() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_execute_leave_delegators());
		});
	}

	#[test]
	fn bench_cancel_leave_delegators() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_cancel_leave_delegators());
		});
	}

	#[test]
	fn bench_schedule_revoke_delegation() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_schedule_revoke_delegation());
		});
	}

	#[test]
	fn bench_delegator_bond_more() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_delegator_bond_more());
		});
	}

	#[test]
	fn bench_schedule_delegator_bond_less() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_schedule_delegator_bond_less());
		});
	}

	#[test]
	fn bench_execute_revoke_delegation() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_execute_revoke_delegation());
		});
	}

	#[test]
	fn bench_execute_delegator_bond_less() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_execute_delegator_bond_less());
		});
	}

	#[test]
	fn bench_cancel_revoke_delegation() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_cancel_revoke_delegation());
		});
	}

	#[test]
	fn bench_cancel_delegator_bond_less() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_cancel_delegator_bond_less());
		});
	}

	#[test]
	fn bench_base_on_initialize() {
		new_test_ext().execute_with(|| {
			assert_ok!(Pallet::<Test>::test_benchmark_base_on_initialize());
		});
	}
}

impl_benchmark_test_suite!(
	Pallet,
	crate::benchmarks::tests::new_test_ext(),
	crate::mock::Test
);
