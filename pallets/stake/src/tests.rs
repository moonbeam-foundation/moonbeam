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
use frame_support::{assert_noop, assert_ok};
use mock::*;
use sp_runtime::DispatchError;

#[test]
fn genesis_config_works() {
	genesis().execute_with(|| {
		assert!(Sys::events().is_empty());
		// validators
		assert_eq!(Balances::reserved_balance(&1), 500);
		assert_eq!(Balances::free_balance(&1), 500);
		assert!(Stake::is_candidate(&1));
		assert_eq!(Balances::reserved_balance(&2), 200);
		assert_eq!(Balances::free_balance(&2), 100);
		assert!(Stake::is_candidate(&2));
		// nominators
		for x in 3..7 {
			assert!(Stake::is_nominator(&x));
			assert_eq!(Balances::free_balance(&x), 0);
			assert_eq!(Balances::reserved_balance(&x), 100);
		}
		// uninvolved
		for x in 7..10 {
			assert!(!Stake::is_nominator(&x));
		}
		assert_eq!(Balances::free_balance(&7), 100);
		assert_eq!(Balances::reserved_balance(&7), 0);
		assert_eq!(Balances::free_balance(&8), 9);
		assert_eq!(Balances::reserved_balance(&8), 0);
		assert_eq!(Balances::free_balance(&9), 4);
		assert_eq!(Balances::reserved_balance(&9), 0);
	});
}

#[test]
fn join_candidates_works() {
	genesis().execute_with(|| {
		assert_noop!(
			Stake::join_candidates(
				Origin::signed(1),
				11u128,
				Perbill::from_percent(2),
				5u128,
				RewardPolicy::<Test>::default()
			),
			Error::<Test>::ValidatorExists
		);
		assert_noop!(
			Stake::join_candidates(
				Origin::signed(3),
				11u128,
				Perbill::from_percent(2),
				5u128,
				RewardPolicy::<Test>::default()
			),
			Error::<Test>::NominatorExists
		);
		assert_noop!(
			Stake::join_candidates(
				Origin::signed(7),
				9u128,
				Perbill::from_percent(2),
				5u128,
				RewardPolicy::<Test>::default()
			),
			Error::<Test>::CandidateBondBelowMin
		);
		assert_noop!(
			Stake::join_candidates(
				Origin::signed(7),
				10u128,
				Perbill::from_percent(2),
				4u128,
				RewardPolicy::<Test>::default()
			),
			Error::<Test>::NominatorBondBelowMin
		);
		assert_noop!(
			Stake::join_candidates(
				Origin::signed(8),
				10u128,
				Perbill::from_percent(2),
				5u128,
				RewardPolicy::<Test>::default()
			),
			DispatchError::Module {
				index: 0,
				error: 3,
				message: Some("InsufficientBalance")
			}
		);
		assert_noop!(
			Stake::join_candidates(
				Origin::signed(7),
				10u128,
				Perbill::from_percent(51),
				5u128,
				RewardPolicy::<Test>::default()
			),
			Error::<Test>::FeeExceedsMaxValidatorFee
		);
		assert!(Sys::events().is_empty());
		assert_ok!(Stake::join_candidates(
			Origin::signed(7),
			10u128,
			Perbill::from_percent(3),
			5u128,
			RewardPolicy::<Test>::default()
		));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::CandidateJoined(7, 10u128))
		);
	});
}
