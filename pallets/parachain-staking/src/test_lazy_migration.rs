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

//! # Lazy Migration Tests
//! Tests for the migration from Currency (locks) to Fungible (freezes) traits.
//! This module focuses specifically on testing the lazy migration functionality
//! that automatically converts accounts from the old lock-based system to the
//! new freeze-based system when they interact with staking operations.

use crate::mock::{
	query_freeze_amount, AccountId, Balances, ExtBuilder, ParachainStaking, RuntimeOrigin, Test,
};
use crate::set::OrderedSet;
use crate::{
	CandidateInfo, FreezeReason, MigratedCandidates, MigratedDelegators, COLLATOR_LOCK_ID,
	DELEGATOR_LOCK_ID,
};
use frame_support::assert_ok;
use frame_support::traits::{LockableCurrency, WithdrawReasons};

// Helper function to create a collator account with old-style locks
fn setup_collator_with_lock(account: AccountId, bond: u128) {
	// Set the lock directly using the old system
	Balances::set_lock(COLLATOR_LOCK_ID, &account, bond, WithdrawReasons::all());

	// Manually insert candidate info (simulating pre-migration state)
	let candidate = crate::types::CandidateMetadata::new(bond);
	CandidateInfo::<Test>::insert(&account, candidate);

	// Add to candidate pool
	let mut pool = crate::CandidatePool::<Test>::get();
	let _ = pool.try_insert(crate::Bond {
		owner: account,
		amount: bond,
	});
	crate::CandidatePool::<Test>::put(pool);
}

// Helper function to create a delegator account with old-style locks
fn setup_delegator_with_lock(account: AccountId, collator: AccountId, amount: u128) {
	// Set the lock directly using the old system
	Balances::set_lock(DELEGATOR_LOCK_ID, &account, amount, WithdrawReasons::all());

	// Set up delegator state for migration to work
	let delegator = crate::Delegator {
		id: account,
		delegations: OrderedSet::from(vec![crate::Bond {
			owner: collator,
			amount,
		}]),
		total: amount,
		less_total: 0,
		status: crate::DelegatorStatus::Active,
	};
	crate::DelegatorState::<Test>::insert(&account, delegator);
}

// Helper function to verify an account has NOT been migrated
fn assert_not_migrated(account: AccountId, is_collator: bool) {
	if is_collator {
		assert!(!MigratedCandidates::<Test>::contains_key(&account));
	} else {
		assert!(!MigratedDelegators::<Test>::contains_key(&account));
	}
}

// Helper function to verify an account HAS been migrated
fn assert_migrated(account: AccountId, is_collator: bool) {
	if is_collator {
		assert!(MigratedCandidates::<Test>::contains_key(&account));
	} else {
		assert!(MigratedDelegators::<Test>::contains_key(&account));
	}
}

// Helper function to get the appropriate freeze reason
fn get_freeze_reason(is_collator: bool) -> crate::mock::RuntimeFreezeReason {
	if is_collator {
		FreezeReason::StakingCollator.into()
	} else {
		FreezeReason::StakingDelegator.into()
	}
}

// Helper function to verify freeze amount and ensure no corresponding lock exists
fn assert_freeze_amount_and_no_lock(account: AccountId, expected_amount: u128, is_collator: bool) {
	let freeze_reason = get_freeze_reason(is_collator);
	assert_eq!(
		query_freeze_amount(account, &freeze_reason),
		expected_amount
	);

	// Verify no corresponding lock remains
	let lock_id = if is_collator {
		COLLATOR_LOCK_ID
	} else {
		DELEGATOR_LOCK_ID
	};
	assert!(!Balances::locks(&account)
		.iter()
		.any(|lock| lock.id == lock_id));
}

#[test]
fn collator_bond_more_triggers_migration() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.build()
		.execute_with(|| {
			let initial_bond = 500;

			// Setup collator with old-style lock
			setup_collator_with_lock(1, initial_bond);

			// Verify initial state - not migrated, has lock
			assert_not_migrated(1, true);

			// Call candidate_bond_more which should trigger migration via bond_more
			assert_ok!(ParachainStaking::candidate_bond_more(
				RuntimeOrigin::signed(1),
				100
			));

			// Should be migrated now
			assert_migrated(1, true);

			// Verify freeze amount is updated to new total and no lock remains
			assert_freeze_amount_and_no_lock(1, 600, true); // 500 + 100
		});
}

#[test]
fn delegator_operations_trigger_migration() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000), (2, 1000)])
		.with_candidates(vec![(1, 500)])
		.build()
		.execute_with(|| {
			// Setup a delegator with an old lock
			setup_delegator_with_lock(2, 1, 200);

			// The batch migration should work
			assert_ok!(ParachainStaking::migrate_locks_to_freezes_batch(
				RuntimeOrigin::signed(1),
				vec![(2, false)].try_into().unwrap(),
			));

			// Should be migrated
			assert_migrated(2, false);

			// Verify freeze amount and no lock remains
			assert_freeze_amount_and_no_lock(2, 200, false);
		});
}

#[test]
fn migrate_locks_to_freezes_batch_basic() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000), (2, 1000), (3, 1000)])
		.build()
		.execute_with(|| {
			// Setup multiple collators with old-style locks
			setup_collator_with_lock(1, 500);
			setup_collator_with_lock(2, 400);
			setup_collator_with_lock(3, 300);

			// Verify none are migrated initially
			assert_not_migrated(1, true);
			assert_not_migrated(2, true);
			assert_not_migrated(3, true);

			// Batch migrate
			assert_ok!(ParachainStaking::migrate_locks_to_freezes_batch(
				RuntimeOrigin::signed(1),
				vec![(1, true), (2, true), (3, true)].try_into().unwrap(),
			));

			// Verify all are migrated
			assert_migrated(1, true);
			assert_migrated(2, true);
			assert_migrated(3, true);

			// Verify freeze amounts and no locks remain
			assert_freeze_amount_and_no_lock(1, 500, true);
			assert_freeze_amount_and_no_lock(2, 400, true);
			assert_freeze_amount_and_no_lock(3, 300, true);
		});
}

#[test]
fn migrate_locks_to_freezes_batch_partial_already_migrated() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000), (2, 1000), (3, 1000)])
		.build()
		.execute_with(|| {
			// Setup collators with old-style locks
			setup_collator_with_lock(1, 500);
			setup_collator_with_lock(2, 400);
			setup_collator_with_lock(3, 300);

			// Migrate account 2 individually first via batch call
			assert_ok!(ParachainStaking::migrate_locks_to_freezes_batch(
				RuntimeOrigin::signed(1),
				vec![(2, true)].try_into().unwrap(),
			));
			assert_migrated(2, true);

			// Now batch migrate all three (including already migrated account 2)
			assert_ok!(ParachainStaking::migrate_locks_to_freezes_batch(
				RuntimeOrigin::signed(1),
				vec![(1, true), (2, true), (3, true)].try_into().unwrap(),
			));

			// All should be migrated
			assert_migrated(1, true);
			assert_migrated(2, true);
			assert_migrated(3, true);

			// Verify freeze amounts are correct and no locks remain
			assert_freeze_amount_and_no_lock(1, 500, true);
			assert_freeze_amount_and_no_lock(2, 400, true);
			assert_freeze_amount_and_no_lock(3, 300, true);
		});
}

#[test]
fn execute_leave_candidates_removes_lock() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.build()
		.execute_with(|| {
			let bond_amount = 500;

			// Setup collator with old-style lock
			setup_collator_with_lock(1, bond_amount);
			assert_not_migrated(1, true);

			// Add required empty delegations for execute_leave_candidates
			let empty_delegations: crate::types::Delegations<AccountId, u128> = Default::default();
			crate::TopDelegations::<Test>::insert(&1, empty_delegations.clone());
			crate::BottomDelegations::<Test>::insert(&1, empty_delegations);

			// Schedule leave first
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				RuntimeOrigin::signed(1),
				1 // candidate_count
			));

			// Fast forward to when we can execute
			crate::mock::roll_to(10);

			// Before executing, verify we have the old lock
			assert!(Balances::locks(&1)
				.iter()
				.any(|lock| lock.id == COLLATOR_LOCK_ID));

			// Execute leave should remove both lock and freeze via thaw_extended
			assert_ok!(ParachainStaking::execute_leave_candidates(
				RuntimeOrigin::signed(1),
				1, // candidate account
				0  // delegation_count
			));

			// After leaving, both lock and freeze should be removed
			assert_freeze_amount_and_no_lock(1, 0, true);

			// The account is now completely unstaked
			assert!(!CandidateInfo::<Test>::contains_key(&1));
		});
}

#[test]
fn get_collator_stakable_free_balance_triggers_migration() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.build()
		.execute_with(|| {
			let bond_amount = 500;

			// Setup collator with old-style lock
			setup_collator_with_lock(1, bond_amount);
			assert_not_migrated(1, true);

			// Query stakable balance should trigger migration
			let stakable = ParachainStaking::get_collator_stakable_free_balance(&1);

			// Should be migrated now
			assert_migrated(1, true);

			// Should return correct stakable amount (total - frozen)
			assert_eq!(stakable, 500); // 1000 - 500

			// Verify the freeze was set and no lock remains
			assert_freeze_amount_and_no_lock(1, bond_amount, true);
		});
}

#[test]
fn schedule_candidate_bond_less_does_not_trigger_migration() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.build()
		.execute_with(|| {
			let bond_amount = 500;

			// Setup collator with old-style lock
			setup_collator_with_lock(1, bond_amount);

			// Add required empty delegations
			let empty_delegations: crate::types::Delegations<AccountId, u128> = Default::default();
			crate::TopDelegations::<Test>::insert(&1, empty_delegations.clone());
			crate::BottomDelegations::<Test>::insert(&1, empty_delegations);

			assert_not_migrated(1, true);

			// Schedule bond less should work with unmigrated account
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				RuntimeOrigin::signed(1),
				100
			));

			// Should NOT be migrated after just scheduling
			assert_not_migrated(1, true);

			// Fast forward to execute delay - need to wait 2 rounds
			// Use the round helper to properly advance rounds
			crate::mock::roll_to_round_begin(3);

			// Execute should trigger migration
			assert_ok!(ParachainStaking::execute_candidate_bond_less(
				RuntimeOrigin::signed(1),
				1
			));

			// Now should be migrated
			assert_migrated(1, true);

			// Freeze should be reduced after execution and no lock remains
			assert_freeze_amount_and_no_lock(1, bond_amount - 100, true);
		});
}

#[test]
fn mixed_migrated_and_unmigrated_accounts() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000), (2, 1000), (3, 1000)])
		.build()
		.execute_with(|| {
			// Setup two collators with old locks
			setup_collator_with_lock(1, 500);
			setup_collator_with_lock(2, 400);

			// Migrate only account 1
			assert_ok!(ParachainStaking::migrate_locks_to_freezes_batch(
				RuntimeOrigin::signed(1),
				vec![(1, true)].try_into().unwrap(),
			));

			// Account 1 should be migrated, 2 should not
			assert_migrated(1, true);
			assert_not_migrated(2, true);

			// Both should have correct balances
			assert_freeze_amount_and_no_lock(1, 500, true);

			// Account 2 interacting should trigger its own migration
			assert_ok!(ParachainStaking::candidate_bond_more(
				RuntimeOrigin::signed(2),
				50
			));

			// Now both should be migrated
			assert_migrated(2, true);
			assert_freeze_amount_and_no_lock(2, 450, true); // 400 + 50
		});
}

#[test]
fn zero_balance_migration() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.build()
		.execute_with(|| {
			// Create a candidate with zero bond (edge case)
			let candidate = crate::types::CandidateMetadata::new(0);
			CandidateInfo::<Test>::insert(&1, candidate);

			// No lock needed for zero amount

			// Batch migrate
			assert_ok!(ParachainStaking::migrate_locks_to_freezes_batch(
				RuntimeOrigin::signed(1),
				vec![(1, true)].try_into().unwrap(),
			));

			// Should be marked as migrated
			assert_migrated(1, true);

			// Should have no freeze and no lock
			assert_freeze_amount_and_no_lock(1, 0, true);
		});
}

#[test]
fn migration_preserves_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.build()
		.execute_with(|| {
			let bond_amount = 500;

			// Setup collator with specific state
			setup_collator_with_lock(1, bond_amount);

			// Add some metadata to ensure it's preserved
			let mut candidate = CandidateInfo::<Test>::get(&1).unwrap();
			candidate.status = crate::CollatorStatus::Leaving(5);
			CandidateInfo::<Test>::insert(&1, candidate);

			// Migrate
			assert_ok!(ParachainStaking::migrate_locks_to_freezes_batch(
				RuntimeOrigin::signed(1),
				vec![(1, true)].try_into().unwrap(),
			));

			// Verify candidate state is preserved
			let migrated_candidate = CandidateInfo::<Test>::get(&1).unwrap();
			assert_eq!(migrated_candidate.status, crate::CollatorStatus::Leaving(5));
			assert_eq!(migrated_candidate.bond, bond_amount);

			// Verify freeze was set correctly and no lock remains
			assert_freeze_amount_and_no_lock(1, bond_amount, true);
		});
}

#[test]
fn migrate_locks_to_freezes_batch_mixed_collators_and_delegators() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000), (2, 1000), (3, 1000), (4, 1000)])
		.build()
		.execute_with(|| {
			// Setup mixed accounts: 2 collators and 2 delegators
			setup_collator_with_lock(1, 500);
			setup_collator_with_lock(2, 400);
			setup_delegator_with_lock(3, 1, 300);
			setup_delegator_with_lock(4, 2, 200);

			// Verify none are migrated initially
			assert_not_migrated(1, true);
			assert_not_migrated(2, true);
			assert_not_migrated(3, false);
			assert_not_migrated(4, false);

			// Batch migrate mixed accounts
			assert_ok!(ParachainStaking::migrate_locks_to_freezes_batch(
				RuntimeOrigin::signed(1),
				vec![(1, true), (2, true), (3, false), (4, false),]
					.try_into()
					.unwrap(),
			));

			// Verify all are migrated
			assert_migrated(1, true);
			assert_migrated(2, true);
			assert_migrated(3, false);
			assert_migrated(4, false);

			// Verify freeze amounts and no locks remain
			assert_freeze_amount_and_no_lock(1, 500, true);
			assert_freeze_amount_and_no_lock(2, 400, true);
			assert_freeze_amount_and_no_lock(3, 300, false);
			assert_freeze_amount_and_no_lock(4, 200, false);
		});
}
