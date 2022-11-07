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

//! Unit testing
use crate::mock::{
	last_event, AuthorMapping, Balances, DepositAmount, ExtBuilder, Runtime,
	RuntimeEvent as MetaEvent, RuntimeOrigin, System, TestAuthor,
};
use crate::{keys_size, keys_wrapper, Error, Event, MappingWithDeposit, RegistrationInfo};
use frame_support::{
	assert_noop, assert_ok,
	traits::{OnRuntimeUpgrade, ReservableCurrency},
};
use nimbus_primitives::NimbusId;

#[test]
fn check_key_size() {
	// NimbusId (32) + NimbusId (32)
	assert_eq!(keys_size::<Runtime>(), 64usize);
}

#[test]
fn genesis_builder_works() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.with_mappings(vec![(TestAuthor::Alice.into(), 1)])
		.build()
		.execute_with(|| {
			assert!(System::events().is_empty());
			assert_eq!(Balances::free_balance(&1), 900);
			assert_eq!(Balances::reserved_balance(&1), 100);
			assert_eq!(
				AuthorMapping::account_id_of(&TestAuthor::Alice.into()),
				Some(1)
			);
			assert_eq!(AuthorMapping::account_id_of(&TestAuthor::Bob.into()), None);
		})
}

#[test]
fn eligible_account_can_register() {
	ExtBuilder::default()
		.with_balances(vec![(2, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(AuthorMapping::add_association(
				RuntimeOrigin::signed(2),
				TestAuthor::Bob.into()
			));

			assert_eq!(Balances::free_balance(&2), 900);
			assert_eq!(Balances::reserved_balance(&2), 100);
			assert_eq!(
				AuthorMapping::account_id_of(&TestAuthor::Bob.into()),
				Some(2)
			);

			assert_eq!(
				last_event(),
				MetaEvent::AuthorMapping(Event::KeysRegistered {
					nimbus_id: TestAuthor::Bob.into(),
					account_id: 2,
					keys: TestAuthor::Bob.into(),
				})
			);
		})
}

#[test]
fn cannot_add_association_without_deposit() {
	ExtBuilder::default()
		.with_balances(vec![(2, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				AuthorMapping::add_association(RuntimeOrigin::signed(2), TestAuthor::Alice.into()),
				Error::<Runtime>::CannotAffordSecurityDeposit
			);

			assert_eq!(Balances::free_balance(&2), 10);
			assert_eq!(
				AuthorMapping::account_id_of(&TestAuthor::Alice.into()),
				None
			);
		})
}

#[test]
fn double_registration_costs_twice_as_much() {
	ExtBuilder::default()
		.with_balances(vec![(2, 1000)])
		.build()
		.execute_with(|| {
			// Register once as Bob
			assert_ok!(AuthorMapping::add_association(
				RuntimeOrigin::signed(2),
				TestAuthor::Bob.into()
			));

			assert_eq!(Balances::free_balance(&2), 900);
			assert_eq!(Balances::reserved_balance(&2), 100);
			assert_eq!(
				AuthorMapping::account_id_of(&TestAuthor::Bob.into()),
				Some(2)
			);

			assert_eq!(
				last_event(),
				MetaEvent::AuthorMapping(Event::KeysRegistered {
					nimbus_id: TestAuthor::Bob.into(),
					account_id: 2,
					keys: TestAuthor::Bob.into(),
				})
			);

			// Register again as Alice
			assert_ok!(AuthorMapping::add_association(
				RuntimeOrigin::signed(2),
				TestAuthor::Alice.into()
			));

			assert_eq!(Balances::free_balance(&2), 800);
			assert_eq!(Balances::reserved_balance(&2), 200);
			assert_eq!(
				AuthorMapping::account_id_of(&TestAuthor::Alice.into()),
				Some(2)
			);

			assert_eq!(
				last_event(),
				MetaEvent::AuthorMapping(Event::KeysRegistered {
					nimbus_id: TestAuthor::Alice.into(),
					account_id: 2,
					keys: TestAuthor::Alice.into(),
				})
			);

			// Should still be registered as Bob as well
			assert_eq!(
				AuthorMapping::account_id_of(&TestAuthor::Bob.into()),
				Some(2)
			);
		})
}

#[test]
fn registered_account_can_clear() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.with_mappings(vec![(TestAuthor::Alice.into(), 1)])
		.build()
		.execute_with(|| {
			assert_ok!(AuthorMapping::remove_keys(RuntimeOrigin::signed(1)));

			assert_eq!(Balances::free_balance(&1), 1000);
			assert_eq!(Balances::reserved_balance(&1), 0);
			assert_eq!(
				AuthorMapping::account_id_of(&TestAuthor::Alice.into()),
				None
			);

			assert_eq!(
				last_event(),
				MetaEvent::AuthorMapping(Event::KeysRemoved {
					nimbus_id: TestAuthor::Alice.into(),
					account_id: 1,
					keys: TestAuthor::Alice.into(),
				})
			);
		})
}

#[test]
fn unregistered_author_cannot_be_cleared() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			AuthorMapping::remove_keys(RuntimeOrigin::signed(1)),
			Error::<Runtime>::OldAuthorIdNotFound
		);
	})
}

#[test]
fn registered_author_cannot_be_cleared_by_non_owner() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.with_mappings(vec![(TestAuthor::Alice.into(), 1)])
		.build()
		.execute_with(|| {
			assert_noop!(
				AuthorMapping::clear_association(
					RuntimeOrigin::signed(2),
					TestAuthor::Alice.into()
				),
				Error::<Runtime>::NotYourAssociation
			);
		})
}

#[test]
fn registered_author_cannot_be_overwritten() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.with_mappings(vec![(TestAuthor::Alice.into(), 1)])
		.build()
		.execute_with(|| {
			assert_noop!(
				AuthorMapping::add_association(RuntimeOrigin::signed(2), TestAuthor::Alice.into()),
				Error::<Runtime>::AlreadyAssociated
			);
		})
}

#[test]
fn registered_can_rotate() {
	ExtBuilder::default()
		.with_balances(vec![(2, 1000)])
		.with_mappings(vec![(TestAuthor::Bob.into(), 2)])
		.build()
		.execute_with(|| {
			assert_ok!(AuthorMapping::update_association(
				RuntimeOrigin::signed(2),
				TestAuthor::Bob.into(),
				TestAuthor::Charlie.into()
			));

			assert_eq!(AuthorMapping::account_id_of(&TestAuthor::Bob.into()), None);
			assert_eq!(
				AuthorMapping::account_id_of(&TestAuthor::Charlie.into()),
				Some(2)
			);

			// Should still only have paid a single security deposit
			assert_eq!(Balances::free_balance(&2), 900);
			assert_eq!(Balances::reserved_balance(&2), 100);
		})
}

#[test]
fn unregistered_author_cannot_be_rotated() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			AuthorMapping::update_association(
				RuntimeOrigin::signed(2),
				TestAuthor::Alice.into(),
				TestAuthor::Bob.into()
			),
			Error::<Runtime>::AssociationNotFound
		);
	})
}

#[test]
fn registered_author_cannot_be_rotated_by_non_owner() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.with_mappings(vec![(TestAuthor::Alice.into(), 1)])
		.build()
		.execute_with(|| {
			assert_noop!(
				AuthorMapping::update_association(
					RuntimeOrigin::signed(2),
					TestAuthor::Alice.into(),
					TestAuthor::Bob.into()
				),
				Error::<Runtime>::NotYourAssociation
			);
		})
}

#[test]
fn rotating_to_the_same_nimbus_id_leaves_registration_in_tact() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.with_mappings(vec![(TestAuthor::Alice.into(), 1)])
		.build()
		.execute_with(|| {
			assert_ok!(AuthorMapping::update_association(
				RuntimeOrigin::signed(1),
				TestAuthor::Alice.into(),
				TestAuthor::Alice.into()
			));
		})
}

#[test]
fn eligible_account_can_full_register() {
	ExtBuilder::default()
		.with_balances(vec![(2, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(AuthorMapping::set_keys(
				RuntimeOrigin::signed(2),
				keys_wrapper::<Runtime>(TestAuthor::Bob.into(), TestAuthor::Alice.into()),
			));

			assert_eq!(Balances::free_balance(&2), 900);
			assert_eq!(Balances::reserved_balance(&2), 100);
			assert_eq!(
				AuthorMapping::account_id_of(&TestAuthor::Bob.into()),
				Some(2)
			);

			assert_eq!(
				last_event(),
				MetaEvent::AuthorMapping(Event::KeysRegistered {
					nimbus_id: TestAuthor::Bob.into(),
					account_id: 2,
					keys: TestAuthor::Alice.into(),
				})
			);
		})
}

#[test]
fn cannot_set_keys_without_deposit() {
	ExtBuilder::default()
		.with_balances(vec![(2, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				AuthorMapping::set_keys(
					RuntimeOrigin::signed(2),
					keys_wrapper::<Runtime>(TestAuthor::Alice.into(), TestAuthor::Bob.into()),
				),
				Error::<Runtime>::CannotAffordSecurityDeposit
			);

			assert_eq!(Balances::free_balance(&2), 10);
			assert_eq!(AuthorMapping::keys_of(&TestAuthor::Alice.into()), None);
		})
}

#[test]
fn full_registered_author_cannot_be_overwritten() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.with_mappings(vec![(TestAuthor::Alice.into(), 1)])
		.build()
		.execute_with(|| {
			assert_noop!(
				AuthorMapping::set_keys(
					RuntimeOrigin::signed(2),
					keys_wrapper::<Runtime>(TestAuthor::Alice.into(), TestAuthor::Bob.into()),
				),
				Error::<Runtime>::AlreadyAssociated
			);
		})
}

#[test]
fn registered_can_full_rotate() {
	ExtBuilder::default()
		.with_balances(vec![(2, 1000)])
		.with_mappings(vec![(TestAuthor::Bob.into(), 2)])
		.build()
		.execute_with(|| {
			assert_ok!(AuthorMapping::set_keys(
				RuntimeOrigin::signed(2),
				keys_wrapper::<Runtime>(TestAuthor::Charlie.into(), TestAuthor::Charlie.into())
			));

			assert_eq!(AuthorMapping::account_id_of(&TestAuthor::Bob.into()), None);
			assert_eq!(
				AuthorMapping::account_id_of(&TestAuthor::Charlie.into()),
				Some(2)
			);
			assert_eq!(
				AuthorMapping::keys_of(&TestAuthor::Charlie.into()),
				Some(TestAuthor::Charlie.into())
			);

			// Should still only have paid a single security deposit
			assert_eq!(Balances::free_balance(&2), 900);
			assert_eq!(Balances::reserved_balance(&2), 100);
		})
}

#[test]
fn unregistered_author_can_be_full_rotated() {
	ExtBuilder::default()
		.with_balances(vec![(2, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(AuthorMapping::set_keys(
				RuntimeOrigin::signed(2),
				keys_wrapper::<Runtime>(TestAuthor::Bob.into(), TestAuthor::Bob.into()),
			));
		})
}

#[test]
fn registered_author_cannot_be_full_rotated_by_non_owner() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.with_mappings(vec![(TestAuthor::Alice.into(), 1)])
		.build()
		.execute_with(|| {
			assert_noop!(
				AuthorMapping::set_keys(
					RuntimeOrigin::signed(2),
					keys_wrapper::<Runtime>(TestAuthor::Alice.into(), TestAuthor::Bob.into())
				),
				Error::<Runtime>::AlreadyAssociated
			);
		})
}

#[test]
fn full_rotating_to_the_same_nimbus_id_leaves_registration_in_tact() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.with_mappings(vec![(TestAuthor::Alice.into(), 1)])
		.build()
		.execute_with(|| {
			assert_ok!(AuthorMapping::set_keys(
				RuntimeOrigin::signed(1),
				keys_wrapper::<Runtime>(TestAuthor::Alice.into(), TestAuthor::Alice.into())
			));
		})
}

#[test]
fn add_reverse_mapping_migration_works() {
	ExtBuilder::default()
		.with_balances(vec![(1, 300)])
		.build()
		.execute_with(|| {
			// register 3 NimbusId owned by 1 account
			let alice_as_nimbus: NimbusId = TestAuthor::Alice.into();
			let bob_as_nimbus: NimbusId = TestAuthor::Bob.into();
			let charlie_as_nimbus: NimbusId = TestAuthor::Charlie.into();
			MappingWithDeposit::<Runtime>::insert(
				alice_as_nimbus.clone(),
				RegistrationInfo {
					account: 1,
					deposit: DepositAmount::get(),
					keys: alice_as_nimbus.clone(),
				},
			);
			MappingWithDeposit::<Runtime>::insert(
				bob_as_nimbus.clone(),
				RegistrationInfo {
					account: 1,
					deposit: DepositAmount::get(),
					keys: bob_as_nimbus.clone(),
				},
			);
			MappingWithDeposit::<Runtime>::insert(
				charlie_as_nimbus.clone(),
				RegistrationInfo {
					account: 1,
					deposit: DepositAmount::get(),
					keys: charlie_as_nimbus.clone(),
				},
			);
			assert_ok!(Balances::reserve(&1, DepositAmount::get() * 3));
			// run migration
			crate::migrations::AddAccountIdToNimbusLookup::<Runtime>::on_runtime_upgrade();
			// ensure last 2 mappings revoked => 200 unreserved but still 100 reserved
			assert_eq!(Balances::free_balance(&1), DepositAmount::get() * 2);
			assert_eq!(Balances::reserved_balance(&1), DepositAmount::get() * 1);
			assert!(MappingWithDeposit::<Runtime>::get(bob_as_nimbus).is_some());
			assert!(MappingWithDeposit::<Runtime>::get(alice_as_nimbus).is_none());
			assert!(MappingWithDeposit::<Runtime>::get(charlie_as_nimbus).is_none());
		})
}
