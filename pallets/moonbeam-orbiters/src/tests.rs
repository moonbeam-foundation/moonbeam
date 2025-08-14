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

//! Unit testing

use crate::mock::{roll_to, ExtBuilder, MoonbeamOrbiters, RuntimeOrigin, System, Test};
use crate::{Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_orbiter_rotation() {
	ExtBuilder::default()
		.with_balances(vec![(2, 20_000), (3, 20_000)])
		.with_min_orbiter_deposit(10_000)
		.build()
		.execute_with(|| {
			// Add a collator to the orbiter program
			assert_ok!(MoonbeamOrbiters::add_collator(RuntimeOrigin::root(), 1),);
			// Register two orbiters
			assert_ok!(MoonbeamOrbiters::orbiter_register(RuntimeOrigin::signed(2)),);
			assert_ok!(MoonbeamOrbiters::collator_add_orbiter(
				RuntimeOrigin::signed(1),
				2
			),);
			assert_ok!(MoonbeamOrbiters::orbiter_register(RuntimeOrigin::signed(3)),);
			assert_ok!(MoonbeamOrbiters::collator_add_orbiter(
				RuntimeOrigin::signed(1),
				3
			),);

			// Roll to second round
			roll_to(4);
			System::assert_last_event(
				Event::<Test>::OrbiterRotation {
					collator: 1,
					old_orbiter: None,
					new_orbiter: Some(2),
				}
				.into(),
			);

			// New orbiter should be active for rounds 2 and 3
			assert_eq!(crate::OrbiterPerRound::<Test>::get(2, 1), Some(2));
			assert_eq!(crate::OrbiterPerRound::<Test>::get(3, 1), Some(2));

			// Roll to fourth round
			roll_to(8);
			System::assert_last_event(
				Event::<Test>::OrbiterRotation {
					collator: 1,
					old_orbiter: Some(2),
					new_orbiter: Some(3),
				}
				.into(),
			);

			// New orbiter should be active for rounds 4 and 5
			assert_eq!(crate::OrbiterPerRound::<Test>::get(4, 1), Some(3));
			assert_eq!(crate::OrbiterPerRound::<Test>::get(5, 1), Some(3));

			// Roll to sixth round, we should come back to the first orbiter
			roll_to(12);
			System::assert_last_event(
				Event::<Test>::OrbiterRotation {
					collator: 1,
					old_orbiter: Some(3),
					new_orbiter: Some(2),
				}
				.into(),
			);
		});
}

#[test]
fn test_collator_add_orbiter() {
	ExtBuilder::default()
		.with_balances(vec![(2, 20_000), (3, 20_000), (4, 20_000)])
		.with_min_orbiter_deposit(10_000)
		.build()
		.execute_with(|| {
			// Add a collator to the orbiter program
			assert_ok!(MoonbeamOrbiters::add_collator(RuntimeOrigin::root(), 1),);
			// Register some orbiters
			assert_ok!(MoonbeamOrbiters::orbiter_register(RuntimeOrigin::signed(2)),);
			assert_ok!(MoonbeamOrbiters::orbiter_register(RuntimeOrigin::signed(3)),);
			assert_ok!(MoonbeamOrbiters::orbiter_register(RuntimeOrigin::signed(4)),);

			// Try to add an orbiter to a collator pool
			// Should fail because collator not exist
			assert_noop!(
				MoonbeamOrbiters::collator_add_orbiter(RuntimeOrigin::signed(99), 2),
				Error::<Test>::CollatorNotFound
			);

			// Try to add an orbiter to a collator pool
			// Should fail because orbiter not exist
			assert_noop!(
				MoonbeamOrbiters::collator_add_orbiter(RuntimeOrigin::signed(1), 99),
				Error::<Test>::OrbiterDepositNotFound
			);

			// Try to add an orbiter to a collator pool, should success
			assert_ok!(MoonbeamOrbiters::collator_add_orbiter(
				RuntimeOrigin::signed(1),
				2
			),);
			System::assert_last_event(
				Event::<Test>::OrbiterJoinCollatorPool {
					collator: 1,
					orbiter: 2,
				}
				.into(),
			);

			// Try to add the same orbiter again, should fail
			assert_noop!(
				MoonbeamOrbiters::collator_add_orbiter(RuntimeOrigin::signed(1), 2),
				Error::<Test>::OrbiterAlreadyInPool
			);

			// Try to add a second orbiter to the collator pool, should success
			assert_ok!(MoonbeamOrbiters::collator_add_orbiter(
				RuntimeOrigin::signed(1),
				3
			),);
			System::assert_last_event(
				Event::<Test>::OrbiterJoinCollatorPool {
					collator: 1,
					orbiter: 3,
				}
				.into(),
			);

			// Try to add a third orbiter to the collator pool, should fail
			assert_noop!(
				MoonbeamOrbiters::collator_add_orbiter(RuntimeOrigin::signed(1), 4),
				Error::<Test>::CollatorPoolTooLarge
			);
		});
}

#[test]
fn test_collator_remove_orbiter() {
	ExtBuilder::default()
		.with_balances(vec![(2, 20_000)])
		.with_min_orbiter_deposit(10_000)
		.build()
		.execute_with(|| {
			// Add a collator to the orbiter program
			assert_ok!(MoonbeamOrbiters::add_collator(RuntimeOrigin::root(), 1),);
			// Register an orbiter
			assert_ok!(MoonbeamOrbiters::orbiter_register(RuntimeOrigin::signed(2)),);
			assert_ok!(MoonbeamOrbiters::collator_add_orbiter(
				RuntimeOrigin::signed(1),
				2
			),);

			// Try to remove an orbiter to a collator pool
			// Should fail because collator not exist
			assert_noop!(
				MoonbeamOrbiters::collator_remove_orbiter(RuntimeOrigin::signed(99), 2),
				Error::<Test>::CollatorNotFound
			);

			// Try to remove an orbiter to a collator pool
			// Should fail because orbiter not exist
			assert_noop!(
				MoonbeamOrbiters::collator_remove_orbiter(RuntimeOrigin::signed(1), 99),
				Error::<Test>::OrbiterNotFound
			);

			// Try to remove an orbiter to a collator pool, should success
			assert_ok!(MoonbeamOrbiters::collator_remove_orbiter(
				RuntimeOrigin::signed(1),
				2
			),);
			System::assert_last_event(
				Event::<Test>::OrbiterLeaveCollatorPool {
					collator: 1,
					orbiter: 2,
				}
				.into(),
			);

			// Try to remove the same orbiter again, should fail
			assert_noop!(
				MoonbeamOrbiters::collator_remove_orbiter(RuntimeOrigin::signed(1), 2),
				Error::<Test>::OrbiterNotFound
			);
		});
}

#[test]
fn test_collator_remove_orbiter_then_add_orbiter() {
	ExtBuilder::default()
		.with_balances(vec![(2, 20_000), (3, 20_000)])
		.with_min_orbiter_deposit(10_000)
		.build()
		.execute_with(|| {
			// Add a collator to the orbiter program
			assert_ok!(MoonbeamOrbiters::add_collator(RuntimeOrigin::root(), 1),);
			// Register an orbiter
			assert_ok!(MoonbeamOrbiters::orbiter_register(RuntimeOrigin::signed(2)),);
			assert_ok!(MoonbeamOrbiters::collator_add_orbiter(
				RuntimeOrigin::signed(1),
				2
			),);

			// Try to remove an orbiter to a collator pool, should success
			assert_ok!(MoonbeamOrbiters::collator_remove_orbiter(
				RuntimeOrigin::signed(1),
				2
			),);
			System::assert_last_event(
				Event::<Test>::OrbiterLeaveCollatorPool {
					collator: 1,
					orbiter: 2,
				}
				.into(),
			);

			// Try to register another orbiter, should success
			assert_ok!(MoonbeamOrbiters::orbiter_register(RuntimeOrigin::signed(3)),);
			assert_ok!(MoonbeamOrbiters::collator_add_orbiter(
				RuntimeOrigin::signed(1),
				3
			),);
			System::assert_last_event(
				Event::<Test>::OrbiterJoinCollatorPool {
					collator: 1,
					orbiter: 3,
				}
				.into(),
			);
		});
}

#[test]
fn test_orbiter_register_fail_if_insufficient_balance() {
	ExtBuilder::default()
		.with_min_orbiter_deposit(10_000)
		.build()
		.execute_with(|| {
			assert_noop!(
				MoonbeamOrbiters::orbiter_register(RuntimeOrigin::signed(1)),
				pallet_balances::Error::<Test>::InsufficientBalance
			);
		});
}

#[test]
fn test_orbiter_register_ok() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20_000)])
		.with_min_orbiter_deposit(10_000)
		.build()
		.execute_with(|| {
			assert!(MoonbeamOrbiters::orbiter(1).is_none());
			assert_ok!(MoonbeamOrbiters::orbiter_register(RuntimeOrigin::signed(1)),);
			assert!(MoonbeamOrbiters::orbiter(1).is_some());
			System::assert_has_event(
				pallet_balances::Event::<Test>::Reserved {
					who: 1,
					amount: 10_000,
				}
				.into(),
			);
			System::assert_last_event(
				Event::<Test>::OrbiterRegistered {
					account: 1,
					deposit: 10_000,
				}
				.into(),
			);
		});
}

#[test]
fn test_add_collator() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(MoonbeamOrbiters::add_collator(RuntimeOrigin::root(), 1),);
		assert_noop!(
			MoonbeamOrbiters::add_collator(RuntimeOrigin::root(), 1),
			Error::<Test>::CollatorAlreadyAdded
		);
	});
}

#[test]
fn test_orbiter_unregister() {
	ExtBuilder::default()
		.with_balances(vec![(2, 20_000)])
		.with_min_orbiter_deposit(10_000)
		.build()
		.execute_with(|| {
			// Add a collator
			assert_ok!(MoonbeamOrbiters::add_collator(RuntimeOrigin::root(), 1),);
			// Register an orbiter
			assert_ok!(MoonbeamOrbiters::orbiter_register(RuntimeOrigin::signed(2)),);

			// Try to unregister an orbiter with wrong hint
			// Should fail because there is 1 collator pool and we provide 0
			assert_noop!(
				MoonbeamOrbiters::orbiter_unregister(RuntimeOrigin::signed(2), 0),
				Error::<Test>::CollatorsPoolCountTooLow
			);

			// Try to unregister an orbiter with right hint, should success
			assert!(MoonbeamOrbiters::orbiter(2).is_some());
			assert_ok!(MoonbeamOrbiters::orbiter_unregister(
				RuntimeOrigin::signed(2),
				1
			),);
			assert!(MoonbeamOrbiters::orbiter(2).is_none());
			System::assert_has_event(
				pallet_balances::Event::<Test>::Unreserved {
					who: 2,
					amount: 10_000,
				}
				.into(),
			);
			System::assert_last_event(Event::<Test>::OrbiterUnregistered { account: 2 }.into());
		});
}
