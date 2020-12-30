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

#[test]
fn online_offline_behaves() {
	genesis().execute_with(|| {
		roll_to(4);
		assert_noop!(
			Stake::go_offline(Origin::signed(3)),
			Error::<Test>::CandidateDNE
		);
		roll_to(11);
		assert_noop!(
			Stake::go_online(Origin::signed(3)),
			Error::<Test>::CandidateDNE
		);
		assert_noop!(
			Stake::go_online(Origin::signed(2)),
			Error::<Test>::AlreadyActive
		);
		assert_ok!(Stake::go_offline(Origin::signed(2)));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::ValidatorOffline(3, 2))
		);
		roll_to(21);
		let events = Sys::events()
			.into_iter()
			.map(|r| r.event)
			.filter_map(|e| {
				if let MetaEvent::stake(inner) = e {
					Some(inner)
				} else {
					None
				}
			})
			.collect::<Vec<_>>();
		let mut expected = vec![
			RawEvent::ValidatorChosen(2, 1, 700),
			RawEvent::ValidatorChosen(2, 2, 400),
			RawEvent::NewRound(5, 2, 2, 1100),
			RawEvent::ValidatorChosen(3, 1, 700),
			RawEvent::ValidatorChosen(3, 2, 400),
			RawEvent::NewRound(10, 3, 2, 1100),
			RawEvent::ValidatorOffline(3, 2),
			RawEvent::ValidatorChosen(4, 1, 700),
			RawEvent::NewRound(15, 4, 1, 700),
			RawEvent::ValidatorChosen(5, 1, 700),
			RawEvent::NewRound(20, 5, 1, 700),
		];
		assert_eq!(events, expected);
		assert_noop!(
			Stake::go_offline(Origin::signed(2)),
			Error::<Test>::AlreadyOffline
		);
		assert_ok!(Stake::go_online(Origin::signed(2)));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::ValidatorActivated(5, 2))
		);
		expected.push(RawEvent::ValidatorActivated(5, 2));
		roll_to(26);
		expected.push(RawEvent::ValidatorChosen(6, 1, 700));
		expected.push(RawEvent::ValidatorChosen(6, 2, 400));
		expected.push(RawEvent::NewRound(25, 6, 2, 1100));
		let events = Sys::events()
			.into_iter()
			.map(|r| r.event)
			.filter_map(|e| {
				if let MetaEvent::stake(inner) = e {
					Some(inner)
				} else {
					None
				}
			})
			.collect::<Vec<_>>();
		assert_eq!(events, expected);
	});
}

#[test]
fn validator_exit_enforces_slash_window_delay() {
	genesis().execute_with(|| {
		roll_to(4);
		assert_noop!(
			Stake::request_exit(Origin::signed(3)),
			Error::<Test>::CandidateDNE
		);
		roll_to(8);
		assert_ok!(Stake::request_exit(Origin::signed(2)));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::ValidatorScheduledExit(2, 18, 400))
		);
		let info = Stake::candidates(&2).unwrap();
		assert_eq!(info.state, ValStatus::Leaving(18));
		roll_to(21);
		let events = Sys::events()
			.into_iter()
			.map(|r| r.event)
			.filter_map(|e| {
				if let MetaEvent::stake(inner) = e {
					Some(inner)
				} else {
					None
				}
			})
			.collect::<Vec<_>>();
		// we must exclude leaving validators from rewards while
		// holding them retroactively accountable for previous faults
		// (within the last T::SlashingWindow blocks)
		let expected = vec![
			RawEvent::ValidatorChosen(2, 1, 700),
			RawEvent::ValidatorChosen(2, 2, 400),
			RawEvent::NewRound(5, 2, 2, 1100),
			RawEvent::ValidatorScheduledExit(2, 18, 400),
			RawEvent::ValidatorChosen(3, 1, 700),
			RawEvent::NewRound(10, 3, 1, 700),
			RawEvent::ValidatorChosen(4, 1, 700),
			RawEvent::NewRound(15, 4, 1, 700),
			RawEvent::ValidatorChosen(5, 1, 700),
			RawEvent::ValidatorLeft(2, 400, 0),
			RawEvent::NewRound(20, 5, 1, 700),
		];
		assert_eq!(events, expected);
	});
}
