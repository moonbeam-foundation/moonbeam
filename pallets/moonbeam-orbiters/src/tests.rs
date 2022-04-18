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

use crate::mock::{last_event, Event, ExtBuilder, MoonbeamOrbiters, Origin, Test};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_orbiter_register_fail_if_insufficient_balance() {
	ExtBuilder::default()
		.with_min_orbiter_deposit(10_000)
		.build()
		.execute_with(|| {
			assert_noop!(
				MoonbeamOrbiters::orbiter_register(Origin::signed(1)),
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
			assert_ok!(MoonbeamOrbiters::orbiter_register(Origin::signed(1)),);
			assert_eq!(
				last_event(),
				Event::Balances(pallet_balances::Event::<Test>::Reserved {
					who: 1,
					amount: 10_000
				})
			)
		});
}
