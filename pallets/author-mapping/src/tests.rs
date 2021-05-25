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

//! Unit testing
use crate::mock::{
	events, last_event, roll_to, Balances, Event as MetaEvent, ExtBuilder, Origin,
	AuthorMapping, System, Test, TestAuthor,
};
use crate::{Error, Event};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{traits::Zero, DispatchError, Perbill};

#[test]
fn genesis_builder_works() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 1000),
		])
		.with_mappings(vec![(TestAuthor::Alice, 1)])
		.build()
		.execute_with(|| {
			assert!(System::events().is_empty());
			assert_eq!(Balances::free_balance(&1), 1000);
			assert_eq!(AuthorMapping::account_id_of(TestAuthor::Alice), Some(1));
			assert_eq!(AuthorMapping::account_id_of(TestAuthor::Bob), None);
		})
}

#[test]
fn eligible_account_can_register() {
	ExtBuilder::default()
		.with_balances(vec![
			(2, 1000),
		])
		.build()
		.execute_with(|| {
			assert_ok!(AuthorMapping::add_association(Origin::signed(2), TestAuthor::Bob));

			assert_eq!(Balances::free_balance(&2), 900);
			assert_eq!(Balances::reserved_balance(&2), 100);
			assert_eq!(AuthorMapping::account_id_of(TestAuthor::Bob), Some(2));

			assert_eq!(
				last_event(),
				MetaEvent::pallet_author_mapping(Event::AuthorRegistered(TestAuthor::Bob, 2))
			);
		})
}


#[test]
fn ineligible_account_cannot_register() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 1000),
		])
		.build()
		.execute_with(|| {
			assert_noop!(
				AuthorMapping::add_association(Origin::signed(1), TestAuthor::Alice),
				Error::<Test>::CannotSetAuthor
			);

			assert_eq!(Balances::free_balance(&1), 1000);
			assert_eq!(AuthorMapping::account_id_of(TestAuthor::Alice), None);
		})
}

// Staked account can double register
// Registered account can clear
// Unregistered account cannot clear
// Registered account can rotate
// unstaked account can be narced after period
// unstaked account cannot be narced before period
// staked account can be narced after period
// staked account cannot be narced before period
// Account that cannot afford security deposit cannot register