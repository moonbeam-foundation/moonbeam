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
	events, last_event, roll_to, set_author, Balances, Event as MetaEvent, ExtBuilder, Origin,
	Stake, System, Test,
};
use crate::{CollatorStatus, Error, Event, Range};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{traits::Zero, DispatchError, Perbill, Percent};

// ~~ PUBLIC DISPATCHABLES ~~

#[test]
fn online_offline_works() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 1000),
			(2, 300),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 9),
			(9, 4),
		])
		.with_collators(vec![(1, 500), (2, 200)])
		.with_nominations(vec![(3, 1, 100), (4, 1, 100), (5, 2, 100), (6, 2, 100)])
		.build()
		.execute_with(|| {
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
				MetaEvent::stake(Event::CollatorWentOffline(3, 2))
			);
			roll_to(21);
			let mut expected = vec![
				Event::CollatorChosen(2, 1, 700),
				Event::CollatorChosen(2, 2, 400),
				Event::NewRound(5, 2, 2, 1100),
				Event::CollatorChosen(3, 1, 700),
				Event::CollatorChosen(3, 2, 400),
				Event::NewRound(10, 3, 2, 1100),
				Event::CollatorWentOffline(3, 2),
				Event::CollatorChosen(4, 1, 700),
				Event::NewRound(15, 4, 1, 700),
				Event::CollatorChosen(5, 1, 700),
				Event::NewRound(20, 5, 1, 700),
			];
			assert_eq!(events(), expected);
			assert_noop!(
				Stake::go_offline(Origin::signed(2)),
				Error::<Test>::AlreadyOffline
			);
			assert_ok!(Stake::go_online(Origin::signed(2)));
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::CollatorBackOnline(5, 2))
			);
			expected.push(Event::CollatorBackOnline(5, 2));
			roll_to(26);
			expected.push(Event::CollatorChosen(6, 1, 700));
			expected.push(Event::CollatorChosen(6, 2, 400));
			expected.push(Event::NewRound(25, 6, 2, 1100));
			assert_eq!(events(), expected);
		});
}

#[test]
fn join_collator_candidates() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 1000),
			(2, 300),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 9),
			(9, 4),
		])
		.with_collators(vec![(1, 500), (2, 200)])
		.with_nominations(vec![(3, 1, 100), (4, 1, 100), (5, 2, 100), (6, 2, 100)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::join_candidates(Origin::signed(1), 11u128,),
				Error::<Test>::CandidateExists
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(3), 11u128,),
				Error::<Test>::NominatorExists
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(7), 9u128,),
				Error::<Test>::ValBondBelowMin
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(8), 10u128,),
				DispatchError::Module {
					index: 1,

					error: 2,
					message: Some("InsufficientBalance")
				}
			);
			assert!(System::events().is_empty());
			assert_ok!(Stake::join_candidates(Origin::signed(7), 10u128,));
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::JoinedCollatorCandidates(7, 10u128, 1110u128))
			);
		});
}

#[test]
fn collator_exit_executes_after_delay() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 1000),
			(2, 300),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 9),
			(9, 4),
		])
		.with_collators(vec![(1, 500), (2, 200)])
		.with_nominations(vec![(3, 1, 100), (4, 1, 100), (5, 2, 100), (6, 2, 100)])
		.build()
		.execute_with(|| {
			roll_to(4);
			assert_noop!(
				Stake::leave_candidates(Origin::signed(3)),
				Error::<Test>::CandidateDNE
			);
			roll_to(11);
			assert_ok!(Stake::leave_candidates(Origin::signed(2)));
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::CollatorScheduledExit(3, 2, 5))
			);
			let info = Stake::collator_state(&2).unwrap();
			assert_eq!(info.state, CollatorStatus::Leaving(5));
			roll_to(21);
			// we must exclude leaving collators from rewards while
			// holding them retroactively accountable for previous faults
			// (within the last T::SlashingWindow blocks)
			let expected = vec![
				Event::CollatorChosen(2, 1, 700),
				Event::CollatorChosen(2, 2, 400),
				Event::NewRound(5, 2, 2, 1100),
				Event::CollatorChosen(3, 1, 700),
				Event::CollatorChosen(3, 2, 400),
				Event::NewRound(10, 3, 2, 1100),
				Event::CollatorScheduledExit(3, 2, 5),
				Event::CollatorChosen(4, 1, 700),
				Event::NewRound(15, 4, 1, 700),
				Event::CollatorLeft(2, 400, 700),
				Event::CollatorChosen(5, 1, 700),
				Event::NewRound(20, 5, 1, 700),
			];
			assert_eq!(events(), expected);
		});
}

#[test]
fn collator_selection_chooses_top_candidates() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 1000),
			(2, 1000),
			(3, 1000),
			(4, 1000),
			(5, 1000),
			(6, 1000),
			(7, 33),
			(8, 33),
			(9, 33),
		])
		.with_collators(vec![(1, 100), (2, 90), (3, 80), (4, 70), (5, 60), (6, 50)])
		.build()
		.execute_with(|| {
			roll_to(8);
			// should choose top TotalSelectedCandidates (5), in order
			let expected = vec![
				Event::CollatorChosen(2, 1, 100),
				Event::CollatorChosen(2, 2, 90),
				Event::CollatorChosen(2, 3, 80),
				Event::CollatorChosen(2, 4, 70),
				Event::CollatorChosen(2, 5, 60),
				Event::NewRound(5, 2, 5, 400),
			];
			assert_eq!(events(), expected);
			assert_ok!(Stake::leave_candidates(Origin::signed(6)));
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::CollatorScheduledExit(2, 6, 4))
			);
			roll_to(21);
			assert_ok!(Stake::join_candidates(Origin::signed(6), 69u128));
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::JoinedCollatorCandidates(6, 69u128, 469u128))
			);
			roll_to(27);
			// should choose top TotalSelectedCandidates (5), in order
			let expected = vec![
				Event::CollatorChosen(2, 1, 100),
				Event::CollatorChosen(2, 2, 90),
				Event::CollatorChosen(2, 3, 80),
				Event::CollatorChosen(2, 4, 70),
				Event::CollatorChosen(2, 5, 60),
				Event::NewRound(5, 2, 5, 400),
				Event::CollatorScheduledExit(2, 6, 4),
				Event::CollatorChosen(3, 1, 100),
				Event::CollatorChosen(3, 2, 90),
				Event::CollatorChosen(3, 3, 80),
				Event::CollatorChosen(3, 4, 70),
				Event::CollatorChosen(3, 5, 60),
				Event::NewRound(10, 3, 5, 400),
				Event::CollatorLeft(6, 50, 400),
				Event::CollatorChosen(4, 1, 100),
				Event::CollatorChosen(4, 2, 90),
				Event::CollatorChosen(4, 3, 80),
				Event::CollatorChosen(4, 4, 70),
				Event::CollatorChosen(4, 5, 60),
				Event::NewRound(15, 4, 5, 400),
				Event::CollatorChosen(5, 1, 100),
				Event::CollatorChosen(5, 2, 90),
				Event::CollatorChosen(5, 3, 80),
				Event::CollatorChosen(5, 4, 70),
				Event::CollatorChosen(5, 5, 60),
				Event::NewRound(20, 5, 5, 400),
				Event::JoinedCollatorCandidates(6, 69, 469),
				Event::CollatorChosen(6, 1, 100),
				Event::CollatorChosen(6, 2, 90),
				Event::CollatorChosen(6, 3, 80),
				Event::CollatorChosen(6, 4, 70),
				Event::CollatorChosen(6, 6, 69),
				Event::NewRound(25, 6, 5, 409),
			];
			assert_eq!(events(), expected);
		});
}

#[test]
fn exit_queue() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 1000),
			(2, 1000),
			(3, 1000),
			(4, 1000),
			(5, 1000),
			(6, 1000),
			(7, 33),
			(8, 33),
			(9, 33),
		])
		.with_collators(vec![(1, 100), (2, 90), (3, 80), (4, 70), (5, 60), (6, 50)])
		.build()
		.execute_with(|| {
			roll_to(8);
			// should choose top TotalSelectedCandidates (5), in order
			let mut expected = vec![
				Event::CollatorChosen(2, 1, 100),
				Event::CollatorChosen(2, 2, 90),
				Event::CollatorChosen(2, 3, 80),
				Event::CollatorChosen(2, 4, 70),
				Event::CollatorChosen(2, 5, 60),
				Event::NewRound(5, 2, 5, 400),
			];
			assert_eq!(events(), expected);
			assert_ok!(Stake::leave_candidates(Origin::signed(6)));
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::CollatorScheduledExit(2, 6, 4))
			);
			roll_to(11);
			assert_ok!(Stake::leave_candidates(Origin::signed(5)));
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::CollatorScheduledExit(3, 5, 5))
			);
			roll_to(16);
			assert_ok!(Stake::leave_candidates(Origin::signed(4)));
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::CollatorScheduledExit(4, 4, 6))
			);
			assert_noop!(
				Stake::leave_candidates(Origin::signed(4)),
				Error::<Test>::AlreadyLeaving
			);
			roll_to(21);
			let mut new_events = vec![
				Event::CollatorScheduledExit(2, 6, 4),
				Event::CollatorChosen(3, 1, 100),
				Event::CollatorChosen(3, 2, 90),
				Event::CollatorChosen(3, 3, 80),
				Event::CollatorChosen(3, 4, 70),
				Event::CollatorChosen(3, 5, 60),
				Event::NewRound(10, 3, 5, 400),
				Event::CollatorScheduledExit(3, 5, 5),
				Event::CollatorLeft(6, 50, 400),
				Event::CollatorChosen(4, 1, 100),
				Event::CollatorChosen(4, 2, 90),
				Event::CollatorChosen(4, 3, 80),
				Event::CollatorChosen(4, 4, 70),
				Event::NewRound(15, 4, 4, 340),
				Event::CollatorScheduledExit(4, 4, 6),
				Event::CollatorLeft(5, 60, 340),
				Event::CollatorChosen(5, 1, 100),
				Event::CollatorChosen(5, 2, 90),
				Event::CollatorChosen(5, 3, 80),
				Event::NewRound(20, 5, 3, 270),
			];
			expected.append(&mut new_events);
			assert_eq!(events(), expected);
		});
}

#[test]
fn payout_distribution_to_solo_collators() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 1000),
			(2, 1000),
			(3, 1000),
			(4, 1000),
			(5, 1000),
			(6, 1000),
			(7, 33),
			(8, 33),
			(9, 33),
		])
		.with_collators(vec![(1, 100), (2, 90), (3, 80), (4, 70), (5, 60), (6, 50)])
		.build()
		.execute_with(|| {
			roll_to(8);
			// should choose top TotalCandidatesSelected (5), in order
			let mut expected = vec![
				Event::CollatorChosen(2, 1, 100),
				Event::CollatorChosen(2, 2, 90),
				Event::CollatorChosen(2, 3, 80),
				Event::CollatorChosen(2, 4, 70),
				Event::CollatorChosen(2, 5, 60),
				Event::NewRound(5, 2, 5, 400),
			];
			assert_eq!(events(), expected);
			// ~ set block author as 1 for all blocks this round
			set_author(2, 1, 100);
			roll_to(16);
			// pay total issuance to 1
			let mut new = vec![
				Event::CollatorChosen(3, 1, 100),
				Event::CollatorChosen(3, 2, 90),
				Event::CollatorChosen(3, 3, 80),
				Event::CollatorChosen(3, 4, 70),
				Event::CollatorChosen(3, 5, 60),
				Event::NewRound(10, 3, 5, 400),
				Event::Rewarded(1, 305),
				Event::CollatorChosen(4, 1, 100),
				Event::CollatorChosen(4, 2, 90),
				Event::CollatorChosen(4, 3, 80),
				Event::CollatorChosen(4, 4, 70),
				Event::CollatorChosen(4, 5, 60),
				Event::NewRound(15, 4, 5, 400),
			];
			expected.append(&mut new);
			assert_eq!(events(), expected);
			// ~ set block author as 1 for 3 blocks this round
			set_author(4, 1, 60);
			// ~ set block author as 2 for 2 blocks this round
			set_author(4, 2, 40);
			roll_to(26);
			// pay 60% total issuance to 1 and 40% total issuance to 2
			let mut new1 = vec![
				Event::CollatorChosen(5, 1, 100),
				Event::CollatorChosen(5, 2, 90),
				Event::CollatorChosen(5, 3, 80),
				Event::CollatorChosen(5, 4, 70),
				Event::CollatorChosen(5, 5, 60),
				Event::NewRound(20, 5, 5, 400),
				Event::Rewarded(1, 192),
				Event::Rewarded(2, 128),
				Event::CollatorChosen(6, 1, 100),
				Event::CollatorChosen(6, 2, 90),
				Event::CollatorChosen(6, 3, 80),
				Event::CollatorChosen(6, 4, 70),
				Event::CollatorChosen(6, 5, 60),
				Event::NewRound(25, 6, 5, 400),
			];
			expected.append(&mut new1);
			assert_eq!(events(), expected);
			// ~ each collator produces 1 block this round
			set_author(6, 1, 20);
			set_author(6, 2, 20);
			set_author(6, 3, 20);
			set_author(6, 4, 20);
			set_author(6, 5, 20);
			roll_to(36);
			// pay 20% issuance for all collators
			let mut new2 = vec![
				Event::CollatorChosen(7, 1, 100),
				Event::CollatorChosen(7, 2, 90),
				Event::CollatorChosen(7, 3, 80),
				Event::CollatorChosen(7, 4, 70),
				Event::CollatorChosen(7, 5, 60),
				Event::NewRound(30, 7, 5, 400),
				Event::Rewarded(5, 67),
				Event::Rewarded(3, 67),
				Event::Rewarded(4, 67),
				Event::Rewarded(1, 67),
				Event::Rewarded(2, 67),
				Event::CollatorChosen(8, 1, 100),
				Event::CollatorChosen(8, 2, 90),
				Event::CollatorChosen(8, 3, 80),
				Event::CollatorChosen(8, 4, 70),
				Event::CollatorChosen(8, 5, 60),
				Event::NewRound(35, 8, 5, 400),
			];
			expected.append(&mut new2);
			assert_eq!(events(), expected);
			// check that distributing rewards clears awarded pts
			assert!(Stake::awarded_pts(1, 1).is_zero());
			assert!(Stake::awarded_pts(4, 1).is_zero());
			assert!(Stake::awarded_pts(4, 2).is_zero());
			assert!(Stake::awarded_pts(6, 1).is_zero());
			assert!(Stake::awarded_pts(6, 2).is_zero());
			assert!(Stake::awarded_pts(6, 3).is_zero());
			assert!(Stake::awarded_pts(6, 4).is_zero());
			assert!(Stake::awarded_pts(6, 5).is_zero());
		});
}

#[test]
fn collator_commission() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
		])
		.with_collators(vec![(1, 20)])
		.with_nominations(vec![(2, 1, 10), (3, 1, 10)])
		.build()
		.execute_with(|| {
			roll_to(8);
			// chooses top TotalSelectedCandidates (5), in order
			let mut expected = vec![
				Event::CollatorChosen(2, 1, 40),
				Event::NewRound(5, 2, 1, 40),
			];
			assert_eq!(events(), expected);
			assert_ok!(Stake::join_candidates(Origin::signed(4), 20u128));
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::JoinedCollatorCandidates(4, 20u128, 60u128))
			);
			roll_to(9);
			assert_ok!(Stake::nominate(Origin::signed(5), 4, 10));
			assert_ok!(Stake::nominate(Origin::signed(6), 4, 10));
			roll_to(11);
			let mut new = vec![
				Event::JoinedCollatorCandidates(4, 20, 60),
				Event::Nomination(5, 10, 4, true, 30),
				Event::Nomination(6, 10, 4, true, 40),
				Event::CollatorChosen(3, 4, 40),
				Event::CollatorChosen(3, 1, 40),
				Event::NewRound(10, 3, 2, 80),
			];
			expected.append(&mut new);
			assert_eq!(events(), expected);
			// only reward author with id 4
			set_author(3, 4, 100);
			roll_to(21);
			// 20% of 10 is commission + due_portion (4) = 2 + 4 = 6
			// all nominator payouts are 10-2 = 8 * stake_pct
			let mut new2 = vec![
				Event::CollatorChosen(4, 4, 40),
				Event::CollatorChosen(4, 1, 40),
				Event::NewRound(15, 4, 2, 80),
				Event::Rewarded(4, 18),
				Event::Rewarded(6, 6),
				Event::Rewarded(5, 6),
				Event::CollatorChosen(5, 4, 40),
				Event::CollatorChosen(5, 1, 40),
				Event::NewRound(20, 5, 2, 80),
			];
			expected.append(&mut new2);
			assert_eq!(events(), expected);
		});
}

#[test]
fn multiple_nominations() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 100),
			(9, 100),
			(10, 100),
		])
		.with_collators(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_nominations(vec![
			(6, 1, 10),
			(7, 1, 10),
			(8, 2, 10),
			(9, 2, 10),
			(10, 1, 10),
		])
		.build()
		.execute_with(|| {
			roll_to(8);
			// chooses top TotalSelectedCandidates (5), in order
			let mut expected = vec![
				Event::CollatorChosen(2, 1, 50),
				Event::CollatorChosen(2, 2, 40),
				Event::CollatorChosen(2, 4, 20),
				Event::CollatorChosen(2, 3, 20),
				Event::CollatorChosen(2, 5, 10),
				Event::NewRound(5, 2, 5, 140),
			];
			assert_eq!(events(), expected);
			assert_noop!(
				Stake::nominate(Origin::signed(6), 1, 10),
				Error::<Test>::AlreadyNominatedCollator,
			);
			assert_noop!(
				Stake::nominate(Origin::signed(6), 2, 2),
				Error::<Test>::NominationBelowMin,
			);
			assert_ok!(Stake::nominate(Origin::signed(6), 2, 10));
			assert_ok!(Stake::nominate(Origin::signed(6), 3, 10));
			assert_ok!(Stake::nominate(Origin::signed(6), 4, 10));
			assert_noop!(
				Stake::nominate(Origin::signed(6), 5, 10),
				Error::<Test>::ExceedMaxCollatorsPerNom,
			);
			roll_to(16);
			let mut new = vec![
				Event::Nomination(6, 10, 2, true, 50),
				Event::Nomination(6, 10, 3, true, 30),
				Event::Nomination(6, 10, 4, true, 30),
				Event::CollatorChosen(3, 2, 50),
				Event::CollatorChosen(3, 1, 50),
				Event::CollatorChosen(3, 4, 30),
				Event::CollatorChosen(3, 3, 30),
				Event::CollatorChosen(3, 5, 10),
				Event::NewRound(10, 3, 5, 170),
				Event::CollatorChosen(4, 2, 50),
				Event::CollatorChosen(4, 1, 50),
				Event::CollatorChosen(4, 4, 30),
				Event::CollatorChosen(4, 3, 30),
				Event::CollatorChosen(4, 5, 10),
				Event::NewRound(15, 4, 5, 170),
			];
			expected.append(&mut new);
			assert_eq!(events(), expected);
			roll_to(21);
			assert_ok!(Stake::nominate(Origin::signed(7), 2, 80));
			assert_noop!(
				Stake::nominate(Origin::signed(7), 3, 11),
				DispatchError::Module {
					index: 1,

					error: 2,
					message: Some("InsufficientBalance")
				},
			);
			assert_ok!(Stake::nominate(Origin::signed(10), 2, 10),);
			roll_to(26);
			let mut new2 = vec![
				Event::CollatorChosen(5, 2, 50),
				Event::CollatorChosen(5, 1, 50),
				Event::CollatorChosen(5, 4, 30),
				Event::CollatorChosen(5, 3, 30),
				Event::CollatorChosen(5, 5, 10),
				Event::NewRound(20, 5, 5, 170),
				Event::Nomination(7, 80, 2, true, 130),
				Event::Nomination(10, 10, 2, false, 130),
				Event::CollatorChosen(6, 2, 130),
				Event::CollatorChosen(6, 1, 50),
				Event::CollatorChosen(6, 4, 30),
				Event::CollatorChosen(6, 3, 30),
				Event::CollatorChosen(6, 5, 10),
				Event::NewRound(25, 6, 5, 250),
			];
			expected.append(&mut new2);
			assert_eq!(events(), expected);
			assert_ok!(Stake::leave_candidates(Origin::signed(2)));
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::CollatorScheduledExit(6, 2, 8))
			);
			roll_to(31);
			let mut new3 = vec![
				Event::CollatorScheduledExit(6, 2, 8),
				Event::CollatorChosen(7, 1, 50),
				Event::CollatorChosen(7, 4, 30),
				Event::CollatorChosen(7, 3, 30),
				Event::CollatorChosen(7, 5, 10),
				Event::NewRound(30, 7, 4, 120),
			];
			expected.append(&mut new3);
			assert_eq!(events(), expected);
			// verify that nominations are removed after collator leaves, not before
			assert_eq!(Stake::nominator_state(7).unwrap().total, 90);
			assert_eq!(
				Stake::nominator_state(7).unwrap().nominations.0.len(),
				2usize
			);
			assert_eq!(Stake::nominator_state(6).unwrap().total, 40);
			assert_eq!(
				Stake::nominator_state(6).unwrap().nominations.0.len(),
				4usize
			);
			assert_eq!(Balances::reserved_balance(&6), 40);
			assert_eq!(Balances::reserved_balance(&7), 90);
			assert_eq!(Balances::free_balance(&6), 60);
			assert_eq!(Balances::free_balance(&7), 10);
			roll_to(40);
			assert_eq!(Stake::nominator_state(7).unwrap().total, 10);
			assert_eq!(Stake::nominator_state(6).unwrap().total, 30);
			assert_eq!(
				Stake::nominator_state(7).unwrap().nominations.0.len(),
				1usize
			);
			assert_eq!(
				Stake::nominator_state(6).unwrap().nominations.0.len(),
				3usize
			);
			assert_eq!(Balances::reserved_balance(&6), 30);
			assert_eq!(Balances::reserved_balance(&7), 10);
			assert_eq!(Balances::free_balance(&6), 70);
			assert_eq!(Balances::free_balance(&7), 90);
		});
}

#[test]
fn collators_bond() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 100),
			(9, 100),
			(10, 100),
		])
		.with_collators(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_nominations(vec![
			(6, 1, 10),
			(7, 1, 10),
			(8, 2, 10),
			(9, 2, 10),
			(10, 1, 10),
		])
		.build()
		.execute_with(|| {
			roll_to(4);
			assert_noop!(
				Stake::candidate_bond_more(Origin::signed(6), 50),
				Error::<Test>::CandidateDNE
			);
			let mut total = Stake::total();
			assert_ok!(Stake::candidate_bond_more(Origin::signed(1), 50));
			total += 50;
			assert_eq!(Stake::total(), total);
			assert_noop!(
				Stake::candidate_bond_more(Origin::signed(1), 40),
				DispatchError::Module {
					index: 1,

					error: 2,
					message: Some("InsufficientBalance")
				}
			);
			assert_ok!(Stake::leave_candidates(Origin::signed(1)));
			assert_noop!(
				Stake::candidate_bond_more(Origin::signed(1), 30),
				Error::<Test>::CannotActivateIfLeaving
			);
			roll_to(30);
			total -= 100;
			assert_eq!(Stake::total(), total);
			assert_noop!(
				Stake::candidate_bond_more(Origin::signed(1), 40),
				Error::<Test>::CandidateDNE
			);
			assert_ok!(Stake::candidate_bond_more(Origin::signed(2), 80));
			total += 80;
			assert_eq!(Stake::total(), total);
			assert_ok!(Stake::candidate_bond_less(Origin::signed(2), 90));
			total -= 90;
			assert_eq!(Stake::total(), total);
			assert_ok!(Stake::candidate_bond_less(Origin::signed(3), 10));
			total -= 10;
			assert_eq!(Stake::total(), total);
			assert_noop!(
				Stake::candidate_bond_less(Origin::signed(2), 11),
				Error::<Test>::CannotBondLessGEQTotalBond
			);
			assert_noop!(
				Stake::candidate_bond_less(Origin::signed(2), 1),
				Error::<Test>::ValBondBelowMin
			);
			assert_noop!(
				Stake::candidate_bond_less(Origin::signed(3), 1),
				Error::<Test>::ValBondBelowMin
			);
			assert_noop!(
				Stake::candidate_bond_less(Origin::signed(4), 11),
				Error::<Test>::ValBondBelowMin
			);
			assert_ok!(Stake::candidate_bond_less(Origin::signed(4), 10));
			total -= 10;
			assert_eq!(Stake::total(), total);
		});
}

#[test]
fn nominators_bond() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 100),
			(9, 100),
			(10, 100),
		])
		.with_collators(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_nominations(vec![
			(6, 1, 10),
			(7, 1, 10),
			(8, 2, 10),
			(9, 2, 10),
			(10, 1, 10),
		])
		.build()
		.execute_with(|| {
			roll_to(4);
			let mut total = Stake::total();
			assert_noop!(
				Stake::nominator_bond_more(Origin::signed(1), 2, 50),
				Error::<Test>::NominatorDNE
			);
			assert_noop!(
				Stake::nominator_bond_more(Origin::signed(6), 2, 50),
				Error::<Test>::NominationDNE
			);
			assert_noop!(
				Stake::nominator_bond_more(Origin::signed(7), 6, 50),
				Error::<Test>::CandidateDNE
			);
			assert_noop!(
				Stake::nominator_bond_less(Origin::signed(6), 1, 11),
				Error::<Test>::CannotBondLessGEQTotalBond
			);
			assert_noop!(
				Stake::nominator_bond_less(Origin::signed(6), 1, 8),
				Error::<Test>::NominationBelowMin
			);
			assert_noop!(
				Stake::nominator_bond_less(Origin::signed(6), 1, 6),
				Error::<Test>::NomBondBelowMin
			);
			assert_ok!(Stake::nominator_bond_more(Origin::signed(6), 1, 10));
			total += 10;
			assert_eq!(Stake::total(), total);
			assert_noop!(
				Stake::nominator_bond_less(Origin::signed(6), 2, 5),
				Error::<Test>::NominationDNE
			);
			assert_noop!(
				Stake::nominator_bond_more(Origin::signed(6), 1, 81),
				DispatchError::Module {
					index: 1,

					error: 2,
					message: Some("InsufficientBalance")
				}
			);
			roll_to(9);
			assert_eq!(Balances::reserved_balance(&6), 20);
			assert_ok!(Stake::leave_candidates(Origin::signed(1)));
			assert_eq!(Stake::total(), total);
			roll_to(31);
			total -= 60;
			assert_eq!(Stake::total(), total);
			assert!(!Stake::is_nominator(&6));
			assert_eq!(Balances::reserved_balance(&6), 0);
			assert_eq!(Balances::free_balance(&6), 100);
		});
}

#[test]
fn revoke_nomination_or_leave_nominators() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 100),
			(9, 100),
			(10, 100),
		])
		.with_collators(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_nominations(vec![
			(6, 1, 10),
			(7, 1, 10),
			(8, 2, 10),
			(9, 2, 10),
			(10, 1, 10),
		])
		.build()
		.execute_with(|| {
			roll_to(4);
			assert_noop!(
				Stake::revoke_nomination(Origin::signed(1), 2),
				Error::<Test>::NominatorDNE
			);
			assert_noop!(
				Stake::revoke_nomination(Origin::signed(6), 2),
				Error::<Test>::NominationDNE
			);
			assert_noop!(
				Stake::leave_nominators(Origin::signed(1)),
				Error::<Test>::NominatorDNE
			);
			assert_ok!(Stake::nominate(Origin::signed(6), 2, 3));
			assert_ok!(Stake::nominate(Origin::signed(6), 3, 3));
			assert_ok!(Stake::revoke_nomination(Origin::signed(6), 1));
			// cannot revoke nomination because would leave remaining total below MinNominatorStk
			assert_noop!(
				Stake::revoke_nomination(Origin::signed(6), 2),
				Error::<Test>::NomBondBelowMin
			);
			assert_noop!(
				Stake::revoke_nomination(Origin::signed(6), 3),
				Error::<Test>::NomBondBelowMin
			);
			// can revoke both remaining by calling leave nominators
			assert_ok!(Stake::leave_nominators(Origin::signed(6)));
			// this leads to 8 leaving set of nominators
			assert_ok!(Stake::revoke_nomination(Origin::signed(8), 2));
		});
}

#[test]
fn payouts_follow_nomination_changes() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 100),
			(9, 100),
			(10, 100),
		])
		.with_collators(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_nominations(vec![
			(6, 1, 10),
			(7, 1, 10),
			(8, 2, 10),
			(9, 2, 10),
			(10, 1, 10),
		])
		.build()
		.execute_with(|| {
			roll_to(8);
			// chooses top TotalSelectedCandidates (5), in order
			let mut expected = vec![
				Event::CollatorChosen(2, 1, 50),
				Event::CollatorChosen(2, 2, 40),
				Event::CollatorChosen(2, 4, 20),
				Event::CollatorChosen(2, 3, 20),
				Event::CollatorChosen(2, 5, 10),
				Event::NewRound(5, 2, 5, 140),
			];
			assert_eq!(events(), expected);
			// ~ set block author as 1 for all blocks this round
			set_author(2, 1, 100);
			roll_to(16);
			// distribute total issuance to collator 1 and its nominators 6, 7, 19
			let mut new = vec![
				Event::CollatorChosen(3, 1, 50),
				Event::CollatorChosen(3, 2, 40),
				Event::CollatorChosen(3, 4, 20),
				Event::CollatorChosen(3, 3, 20),
				Event::CollatorChosen(3, 5, 10),
				Event::NewRound(10, 3, 5, 140),
				Event::Rewarded(1, 26),
				Event::Rewarded(7, 8),
				Event::Rewarded(10, 8),
				Event::Rewarded(6, 8),
				Event::CollatorChosen(4, 1, 50),
				Event::CollatorChosen(4, 2, 40),
				Event::CollatorChosen(4, 4, 20),
				Event::CollatorChosen(4, 3, 20),
				Event::CollatorChosen(4, 5, 10),
				Event::NewRound(15, 4, 5, 140),
			];
			expected.append(&mut new);
			assert_eq!(events(), expected);
			// ~ set block author as 1 for all blocks this round
			set_author(3, 1, 100);
			set_author(4, 1, 100);
			// 1. ensure nominators are paid for 2 rounds after they leave
			assert_noop!(
				Stake::leave_nominators(Origin::signed(66)),
				Error::<Test>::NominatorDNE
			);
			assert_ok!(Stake::leave_nominators(Origin::signed(6)));
			roll_to(21);
			// keep paying 6 (note: inflation is in terms of total issuance so that's why 1 is 21)
			let mut new2 = vec![
				Event::NominatorLeftCollator(6, 1, 10, 40),
				Event::NominatorLeft(6, 10),
				Event::Rewarded(1, 27),
				Event::Rewarded(7, 8),
				Event::Rewarded(10, 8),
				Event::Rewarded(6, 8),
				Event::CollatorChosen(5, 2, 40),
				Event::CollatorChosen(5, 1, 40),
				Event::CollatorChosen(5, 4, 20),
				Event::CollatorChosen(5, 3, 20),
				Event::CollatorChosen(5, 5, 10),
				Event::NewRound(20, 5, 5, 130),
			];
			expected.append(&mut new2);
			assert_eq!(events(), expected);
			// 6 won't be paid for this round because they left already
			set_author(5, 1, 100);
			roll_to(26);
			// keep paying 6
			let mut new3 = vec![
				Event::Rewarded(1, 29),
				Event::Rewarded(7, 9),
				Event::Rewarded(10, 9),
				Event::Rewarded(6, 9),
				Event::CollatorChosen(6, 2, 40),
				Event::CollatorChosen(6, 1, 40),
				Event::CollatorChosen(6, 4, 20),
				Event::CollatorChosen(6, 3, 20),
				Event::CollatorChosen(6, 5, 10),
				Event::NewRound(25, 6, 5, 130),
			];
			expected.append(&mut new3);
			assert_eq!(events(), expected);
			set_author(6, 1, 100);
			roll_to(31);
			// no more paying 6
			let mut new4 = vec![
				Event::Rewarded(1, 35),
				Event::Rewarded(7, 11),
				Event::Rewarded(10, 11),
				Event::CollatorChosen(7, 2, 40),
				Event::CollatorChosen(7, 1, 40),
				Event::CollatorChosen(7, 4, 20),
				Event::CollatorChosen(7, 3, 20),
				Event::CollatorChosen(7, 5, 10),
				Event::NewRound(30, 7, 5, 130),
			];
			expected.append(&mut new4);
			assert_eq!(events(), expected);
			set_author(7, 1, 100);
			assert_ok!(Stake::nominate(Origin::signed(8), 1, 10));
			roll_to(36);
			// new nomination is not rewarded yet
			let mut new5 = vec![
				Event::Nomination(8, 10, 1, true, 50),
				Event::Rewarded(1, 36),
				Event::Rewarded(7, 12),
				Event::Rewarded(10, 12),
				Event::CollatorChosen(8, 1, 50),
				Event::CollatorChosen(8, 2, 40),
				Event::CollatorChosen(8, 4, 20),
				Event::CollatorChosen(8, 3, 20),
				Event::CollatorChosen(8, 5, 10),
				Event::NewRound(35, 8, 5, 140),
			];
			expected.append(&mut new5);
			assert_eq!(events(), expected);
			set_author(8, 1, 100);
			roll_to(41);
			// new nomination is still not rewarded yet
			let mut new6 = vec![
				Event::Rewarded(1, 38),
				Event::Rewarded(7, 13),
				Event::Rewarded(10, 13),
				Event::CollatorChosen(9, 1, 50),
				Event::CollatorChosen(9, 2, 40),
				Event::CollatorChosen(9, 4, 20),
				Event::CollatorChosen(9, 3, 20),
				Event::CollatorChosen(9, 5, 10),
				Event::NewRound(40, 9, 5, 140),
			];
			expected.append(&mut new6);
			assert_eq!(events(), expected);
			roll_to(46);
			// new nomination is rewarded for first time, 2 rounds after joining (`BondDuration` = 2)
			let mut new7 = vec![
				Event::Rewarded(1, 35),
				Event::Rewarded(7, 11),
				Event::Rewarded(8, 11),
				Event::Rewarded(10, 11),
				Event::CollatorChosen(10, 1, 50),
				Event::CollatorChosen(10, 2, 40),
				Event::CollatorChosen(10, 4, 20),
				Event::CollatorChosen(10, 3, 20),
				Event::CollatorChosen(10, 5, 10),
				Event::NewRound(45, 10, 5, 140),
			];
			expected.append(&mut new7);
			assert_eq!(events(), expected);
		});
}

#[test]
fn parachain_bond_reserve_works() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 100),
			(9, 100),
			(10, 100),
			(11, 1),
		])
		.with_collators(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_nominations(vec![
			(6, 1, 10),
			(7, 1, 10),
			(8, 2, 10),
			(9, 2, 10),
			(10, 1, 10),
		])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::free_balance(&11), 1);
			// set parachain bond account so DefaultParachainBondReservePercent = 30% of inflation
			// is allocated to this account hereafter
			assert_ok!(Stake::set_parachain_bond_account(Origin::root(), 11));
			roll_to(8);
			// chooses top TotalSelectedCandidates (5), in order
			let mut expected = vec![
				Event::ParachainBondAccountSet(0, 11),
				Event::CollatorChosen(2, 1, 50),
				Event::CollatorChosen(2, 2, 40),
				Event::CollatorChosen(2, 4, 20),
				Event::CollatorChosen(2, 3, 20),
				Event::CollatorChosen(2, 5, 10),
				Event::NewRound(5, 2, 5, 140),
			];
			assert_eq!(events(), expected);
			assert_eq!(Balances::free_balance(&11), 1);
			// ~ set block author as 1 for all blocks this round
			set_author(2, 1, 100);
			roll_to(16);
			// distribute total issuance to collator 1 and its nominators 6, 7, 19
			let mut new = vec![
				Event::CollatorChosen(3, 1, 50),
				Event::CollatorChosen(3, 2, 40),
				Event::CollatorChosen(3, 4, 20),
				Event::CollatorChosen(3, 3, 20),
				Event::CollatorChosen(3, 5, 10),
				Event::NewRound(10, 3, 5, 140),
				Event::ReservedForParachainBond(11, 15),
				Event::Rewarded(1, 18),
				Event::Rewarded(7, 6),
				Event::Rewarded(10, 6),
				Event::Rewarded(6, 6),
				Event::CollatorChosen(4, 1, 50),
				Event::CollatorChosen(4, 2, 40),
				Event::CollatorChosen(4, 4, 20),
				Event::CollatorChosen(4, 3, 20),
				Event::CollatorChosen(4, 5, 10),
				Event::NewRound(15, 4, 5, 140),
			];
			expected.append(&mut new);
			assert_eq!(events(), expected);
			assert_eq!(Balances::free_balance(&11), 16);
			// ~ set block author as 1 for all blocks this round
			set_author(3, 1, 100);
			set_author(4, 1, 100);
			// 1. ensure nominators are paid for 2 rounds after they leave
			assert_noop!(
				Stake::leave_nominators(Origin::signed(66)),
				Error::<Test>::NominatorDNE
			);
			assert_ok!(Stake::leave_nominators(Origin::signed(6)));
			roll_to(21);
			// keep paying 6 (note: inflation is in terms of total issuance so that's why 1 is 21)
			let mut new2 = vec![
				Event::NominatorLeftCollator(6, 1, 10, 40),
				Event::NominatorLeft(6, 10),
				Event::ReservedForParachainBond(11, 16),
				Event::Rewarded(1, 19),
				Event::Rewarded(7, 6),
				Event::Rewarded(10, 6),
				Event::Rewarded(6, 6),
				Event::CollatorChosen(5, 2, 40),
				Event::CollatorChosen(5, 1, 40),
				Event::CollatorChosen(5, 4, 20),
				Event::CollatorChosen(5, 3, 20),
				Event::CollatorChosen(5, 5, 10),
				Event::NewRound(20, 5, 5, 130),
			];
			expected.append(&mut new2);
			assert_eq!(events(), expected);
			assert_eq!(Balances::free_balance(&11), 32);
			assert_ok!(Stake::set_parachain_bond_reserve_percent(
				Origin::root(),
				Percent::from_percent(50)
			));
			// 6 won't be paid for this round because they left already
			set_author(5, 1, 100);
			roll_to(26);
			// keep paying 6
			let mut new3 = vec![
				Event::ParachainBondReservePercentSet(
					Percent::from_percent(30),
					Percent::from_percent(50),
				),
				Event::ReservedForParachainBond(11, 27),
				Event::Rewarded(1, 15),
				Event::Rewarded(7, 4),
				Event::Rewarded(10, 4),
				Event::Rewarded(6, 4),
				Event::CollatorChosen(6, 2, 40),
				Event::CollatorChosen(6, 1, 40),
				Event::CollatorChosen(6, 4, 20),
				Event::CollatorChosen(6, 3, 20),
				Event::CollatorChosen(6, 5, 10),
				Event::NewRound(25, 6, 5, 130),
			];
			expected.append(&mut new3);
			assert_eq!(events(), expected);
			assert_eq!(Balances::free_balance(&11), 59);
			set_author(6, 1, 100);
			roll_to(31);
			// no more paying 6
			let mut new4 = vec![
				Event::ReservedForParachainBond(11, 29),
				Event::Rewarded(1, 17),
				Event::Rewarded(7, 6),
				Event::Rewarded(10, 6),
				Event::CollatorChosen(7, 2, 40),
				Event::CollatorChosen(7, 1, 40),
				Event::CollatorChosen(7, 4, 20),
				Event::CollatorChosen(7, 3, 20),
				Event::CollatorChosen(7, 5, 10),
				Event::NewRound(30, 7, 5, 130),
			];
			expected.append(&mut new4);
			assert_eq!(events(), expected);
			assert_eq!(Balances::free_balance(&11), 88);
			set_author(7, 1, 100);
			assert_ok!(Stake::nominate(Origin::signed(8), 1, 10));
			roll_to(36);
			// new nomination is not rewarded yet
			let mut new5 = vec![
				Event::Nomination(8, 10, 1, true, 50),
				Event::ReservedForParachainBond(11, 30),
				Event::Rewarded(1, 18),
				Event::Rewarded(7, 6),
				Event::Rewarded(10, 6),
				Event::CollatorChosen(8, 1, 50),
				Event::CollatorChosen(8, 2, 40),
				Event::CollatorChosen(8, 4, 20),
				Event::CollatorChosen(8, 3, 20),
				Event::CollatorChosen(8, 5, 10),
				Event::NewRound(35, 8, 5, 140),
			];
			expected.append(&mut new5);
			assert_eq!(events(), expected);
			assert_eq!(Balances::free_balance(&11), 118);
			set_author(8, 1, 100);
			roll_to(41);
			// new nomination is still not rewarded yet
			let mut new6 = vec![
				Event::ReservedForParachainBond(11, 32),
				Event::Rewarded(1, 19),
				Event::Rewarded(7, 6),
				Event::Rewarded(10, 6),
				Event::CollatorChosen(9, 1, 50),
				Event::CollatorChosen(9, 2, 40),
				Event::CollatorChosen(9, 4, 20),
				Event::CollatorChosen(9, 3, 20),
				Event::CollatorChosen(9, 5, 10),
				Event::NewRound(40, 9, 5, 140),
			];
			expected.append(&mut new6);
			assert_eq!(events(), expected);
			assert_eq!(Balances::free_balance(&11), 150);
			roll_to(46);
			// new nomination is rewarded for first time, 2 rounds after joining (`BondDuration` = 2)
			let mut new7 = vec![
				Event::ReservedForParachainBond(11, 33),
				Event::Rewarded(1, 18),
				Event::Rewarded(7, 5),
				Event::Rewarded(8, 5),
				Event::Rewarded(10, 5),
				Event::CollatorChosen(10, 1, 50),
				Event::CollatorChosen(10, 2, 40),
				Event::CollatorChosen(10, 4, 20),
				Event::CollatorChosen(10, 3, 20),
				Event::CollatorChosen(10, 5, 10),
				Event::NewRound(45, 10, 5, 140),
			];
			expected.append(&mut new7);
			assert_eq!(events(), expected);
			assert_eq!(Balances::free_balance(&11), 183);
		});
}

// ~~ ROOT DISPATCHABLES ~~

#[test]
fn set_staking_expectations_works() {
	ExtBuilder::default().build().execute_with(|| {
		// invalid call fails
		assert_noop!(
			Stake::set_staking_expectations(
				Origin::root(),
				Range {
					min: 5u32.into(),
					ideal: 4u32.into(),
					max: 3u32.into()
				}
			),
			Error::<Test>::InvalidSchedule
		);
		let (min, ideal, max): (u128, u128, u128) = (3u32.into(), 4u32.into(), 5u32.into());
		// valid call succeeds
		assert_ok!(Stake::set_staking_expectations(
			Origin::root(),
			Range { min, ideal, max }
		),);
		// verify event emission
		assert_eq!(
			last_event(),
			MetaEvent::stake(Event::StakeExpectationsSet(min, ideal, max))
		);
		// verify storage change
		let config = Stake::inflation_config();
		assert_eq!(config.expect, Range { min, ideal, max });
	});
}

#[test]
fn set_inflation_works() {
	ExtBuilder::default().build().execute_with(|| {
		// invalid call fails
		assert_noop!(
			Stake::set_inflation(
				Origin::root(),
				Range {
					min: Perbill::from_percent(5),
					ideal: Perbill::from_percent(4),
					max: Perbill::from_percent(3)
				}
			),
			Error::<Test>::InvalidSchedule
		);
		let (min, ideal, max): (Perbill, Perbill, Perbill) = (
			Perbill::from_percent(3),
			Perbill::from_percent(4),
			Perbill::from_percent(5),
		);
		// valid call succeeds
		assert_ok!(Stake::set_inflation(
			Origin::root(),
			Range { min, ideal, max }
		),);
		// verify event emission
		assert_eq!(
			last_event(),
			MetaEvent::stake(Event::InflationSet(
				Perbill::from_parts(30000000),
				Perbill::from_parts(40000000),
				Perbill::from_parts(50000000),
				Perbill::from_parts(57),
				Perbill::from_parts(75),
				Perbill::from_parts(93)
			))
		);
		// verify storage change
		let config = Stake::inflation_config();
		assert_eq!(config.annual, Range { min, ideal, max });
		assert_eq!(
			config.round,
			Range {
				min: Perbill::from_parts(57),
				ideal: Perbill::from_parts(75),
				max: Perbill::from_parts(93)
			}
		);
		// invalid call fails
		assert_noop!(
			Stake::set_inflation(Origin::root(), Range { min, ideal, max }),
			Error::<Test>::NoWritingSameValue
		);
	});
}

#[test]
fn set_total_selected_works() {
	ExtBuilder::default().build().execute_with(|| {
		// invalid call fails
		assert_noop!(
			Stake::set_total_selected(Origin::root(), 4u32),
			Error::<Test>::CannotSetBelowMin
		);
		// valid call succeeds
		assert_ok!(Stake::set_total_selected(Origin::root(), 6u32));
		// verify event emission
		assert_eq!(
			last_event(),
			MetaEvent::stake(Event::TotalSelectedSet(5u32, 6u32,))
		);
		// verify storage change
		assert_eq!(Stake::total_selected(), 6u32);
		// invalid call fails
		assert_noop!(
			Stake::set_total_selected(Origin::root(), 6u32),
			Error::<Test>::NoWritingSameValue
		);
	});
}

#[test]
fn set_collator_commission_works() {
	ExtBuilder::default().build().execute_with(|| {
		// valid call succeeds
		assert_ok!(Stake::set_collator_commission(
			Origin::root(),
			Perbill::from_percent(5)
		));
		// verify event emission
		assert_eq!(
			last_event(),
			MetaEvent::stake(Event::CollatorCommissionSet(
				Perbill::from_percent(20),
				Perbill::from_percent(5),
			))
		);
		// verify storage change
		assert_eq!(Stake::collator_commission(), Perbill::from_percent(5));
		// invalid call fails
		assert_noop!(
			Stake::set_collator_commission(Origin::root(), Perbill::from_percent(5)),
			Error::<Test>::NoWritingSameValue
		);
	});
}

#[test]
fn mutable_blocks_per_round() {
	// round_immediately_jumps_if_current_duration_exceeds_new_blocks_per_round
	ExtBuilder::default()
		.with_balances(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
		])
		.with_collators(vec![(1, 20)])
		.with_nominations(vec![(2, 1, 10), (3, 1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::set_blocks_per_round(Origin::root(), 2u32),
				Error::<Test>::CannotSetBelowMin
			);
			assert_noop!(
				Stake::set_blocks_per_round(Origin::root(), 5u32),
				Error::<Test>::NoWritingSameValue
			);
			// Default round every 5 blocks, but MinBlocksPerRound is 3 and we set it to min 3 blocks
			roll_to(8);
			// chooses top TotalSelectedCandidates (5), in order
			let init = vec![
				Event::CollatorChosen(2, 1, 40),
				Event::NewRound(5, 2, 1, 40),
			];
			assert_eq!(events(), init);
			assert_ok!(Stake::set_blocks_per_round(Origin::root(), 3u32));
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::BlocksPerRoundSet(
					2,
					5,
					5,
					3,
					Perbill::from_parts(463),
					Perbill::from_parts(463),
					Perbill::from_parts(463)
				))
			);
			roll_to(12);
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::NewRound(11, 4, 1, 40))
			);
		});
	// round_immediately_jumps_if_current_duration_exceeds_new_blocks_per_round
	ExtBuilder::default()
		.with_balances(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
		])
		.with_collators(vec![(1, 20)])
		.with_nominations(vec![(2, 1, 10), (3, 1, 10)])
		.build()
		.execute_with(|| {
			roll_to(9);
			let init = vec![
				Event::CollatorChosen(2, 1, 40),
				Event::NewRound(5, 2, 1, 40),
			];
			assert_eq!(events(), init);
			assert_ok!(Stake::set_blocks_per_round(Origin::root(), 3u32));
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::BlocksPerRoundSet(
					2,
					5,
					5,
					3,
					Perbill::from_parts(463),
					Perbill::from_parts(463),
					Perbill::from_parts(463)
				))
			);
			roll_to(13);
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::NewRound(12, 4, 1, 40))
			);
		});
	// if current duration less than new blocks per round (bpr), round waits until new bpr passes
	ExtBuilder::default()
		.with_balances(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
		])
		.with_collators(vec![(1, 20)])
		.with_nominations(vec![(2, 1, 10), (3, 1, 10)])
		.build()
		.execute_with(|| {
			// Default round every 5 blocks, but MinBlocksPerRound is 3 and we set it to min 3 blocks
			roll_to(6);
			// chooses top TotalSelectedCandidates (5), in order
			let init = vec![
				Event::CollatorChosen(2, 1, 40),
				Event::NewRound(5, 2, 1, 40),
			];
			assert_eq!(events(), init);
			assert_ok!(Stake::set_blocks_per_round(Origin::root(), 3u32));
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::BlocksPerRoundSet(
					2,
					5,
					5,
					3,
					Perbill::from_parts(463),
					Perbill::from_parts(463),
					Perbill::from_parts(463)
				))
			);
			roll_to(9);
			assert_eq!(last_event(), MetaEvent::stake(Event::NewRound(8, 3, 1, 40)));
		});
}
