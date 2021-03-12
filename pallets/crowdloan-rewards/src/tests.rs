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
use sp_runtime::AccountId32;
use sp_core::Pair;

#[test]
fn geneses() {
	let pairs = get_ed25519_pairs(3);
	two_assigned_three_unassigned().execute_with(|| {
		assert!(Sys::events().is_empty());
		// accounts_payable
		assert!(Crowdloan::accounts_payable(&1).is_some());
		assert!(Crowdloan::accounts_payable(&1).is_some());
		assert!(Crowdloan::accounts_payable(&2).is_some());
		assert!(Crowdloan::accounts_payable(&3).is_none());
		assert!(Crowdloan::accounts_payable(&4).is_none());
		assert!(Crowdloan::accounts_payable(&5).is_none());


		// accounts_mapping
		assert!(Crowdloan::accounts_mapping(&AccountId32::from([1u8; 32])).is_some());
		assert!(Crowdloan::accounts_mapping(&AccountId32::from([2u8; 32])).is_some());
		assert!(Crowdloan::accounts_mapping(&AccountId32::from(pairs[0].public())).is_none());
		assert!(Crowdloan::accounts_mapping(&AccountId32::from(pairs[1].public())).is_none());
		assert!(Crowdloan::accounts_mapping(&AccountId32::from(pairs[2].public())).is_none());

		// unassociated_contributions
		assert!(Crowdloan::unassociated_contributions(&AccountId32::from([1u8; 32])).is_none());
		assert!(Crowdloan::unassociated_contributions(&AccountId32::from([2u8; 32])).is_none());
		assert!(Crowdloan::unassociated_contributions(&AccountId32::from(pairs[0].public())).is_some());
		assert!(Crowdloan::unassociated_contributions(&AccountId32::from(pairs[1].public())).is_some());
		assert!(Crowdloan::unassociated_contributions(&AccountId32::from(pairs[2].public())).is_some());
	});
}