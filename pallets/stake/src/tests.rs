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
use crate::*;
use mock::*;

#[test]
fn genesis_config_works() {
	new_test_ext().execute_with(|| {
		assert!(Sys::events().is_empty());
		// validators
		assert_eq!(Balances::reserved_balance(&1), 500);
		assert_eq!(Balances::free_balance(&1), 500);
		assert_eq!(Balances::reserved_balance(&2), 200);
		assert_eq!(Balances::free_balance(&2), 100);
		// nominators
		for x in 3..7 {
			assert_eq!(Balances::free_balance(&x), 0);
			assert_eq!(Balances::reserved_balance(&x), 100);
		}
	});
}
