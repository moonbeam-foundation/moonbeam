// Copyright 2019-2021 PureStake Inc.
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
	last_event, AuthorMapping, Balances, Event as MetaEvent, ExtBuilder, Origin, Runtime, System,
	TestAuthor,
};
use crate::{Error, Event};
use frame_support::{assert_noop, assert_ok};

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
				Origin::signed(2),
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
				MetaEvent::AuthorMapping(Event::AuthorRegistered(TestAuthor::Bob.into(), 2))
			);
		})
}

#[test]
fn cannot_register_without_deposit() {
	ExtBuilder::default()
		.with_balances(vec![(2, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				AuthorMapping::add_association(Origin::signed(2), TestAuthor::Alice.into()),
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
				Origin::signed(2),
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
				MetaEvent::AuthorMapping(Event::AuthorRegistered(TestAuthor::Bob.into(), 2))
			);

			// Register again as Alice
			assert_ok!(AuthorMapping::add_association(
				Origin::signed(2),
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
				MetaEvent::AuthorMapping(Event::AuthorRegistered(TestAuthor::Alice.into(), 2))
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
			assert_ok!(AuthorMapping::clear_association(
				Origin::signed(1),
				TestAuthor::Alice.into()
			));

			assert_eq!(Balances::free_balance(&1), 1000);
			assert_eq!(Balances::reserved_balance(&1), 0);
			assert_eq!(
				AuthorMapping::account_id_of(&TestAuthor::Alice.into()),
				None
			);

			assert_eq!(
				last_event(),
				MetaEvent::AuthorMapping(Event::AuthorDeRegistered(TestAuthor::Alice.into()))
			);
		})
}

#[test]
fn unregistered_author_cannot_be_cleared() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			AuthorMapping::clear_association(Origin::signed(1), TestAuthor::Alice.into()),
			Error::<Runtime>::AssociationNotFound
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
				AuthorMapping::clear_association(Origin::signed(2), TestAuthor::Alice.into()),
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
				AuthorMapping::add_association(Origin::signed(2), TestAuthor::Alice.into()),
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
				Origin::signed(2),
				TestAuthor::Bob.into(),
				TestAuthor::Charlie.into()
			));

			assert_eq!(AuthorMapping::account_id_of(&TestAuthor::Bob.into()), None);
			assert_eq!(
				AuthorMapping::account_id_of(&TestAuthor::Charlie.into()),
				Some(2)
			);

			// Should still only ahve paid a single security deposit
			assert_eq!(Balances::free_balance(&2), 900);
			assert_eq!(Balances::reserved_balance(&2), 100);
		})
}

#[test]
fn unregistered_author_cannot_be_rotated() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			AuthorMapping::update_association(
				Origin::signed(2),
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
					Origin::signed(2),
					TestAuthor::Alice.into(),
					TestAuthor::Bob.into()
				),
				Error::<Runtime>::NotYourAssociation
			);
		})
}

//TODO Test ideas in case we bring back the narc extrinsic
// unstaked account can be narced after period
// unstaked account cannot be narced before period
// staked account can be narced after period
// staked account cannot be narced before period
