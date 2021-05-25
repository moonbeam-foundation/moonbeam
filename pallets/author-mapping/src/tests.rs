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