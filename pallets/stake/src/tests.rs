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
fn genesis_works() {
	two_validators_four_nominators().execute_with(|| {
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
	five_validators_five_nominators().execute_with(|| {
		assert!(Sys::events().is_empty());
		// validators
		for x in 1..5 {
			assert!(Stake::is_candidate(&x));
			assert_eq!(Balances::free_balance(&x), 80);
			assert_eq!(Balances::reserved_balance(&x), 20);
		}
		assert!(Stake::is_candidate(&5));
		assert_eq!(Balances::free_balance(&5), 90);
		assert_eq!(Balances::reserved_balance(&5), 10);
		// nominators
		for x in 6..11 {
			assert!(Stake::is_nominator(&x));
			assert_eq!(Balances::free_balance(&x), 90);
			assert_eq!(Balances::reserved_balance(&x), 10);
		}
	});
}

#[test]
fn online_offline_behaves() {
	two_validators_four_nominators().execute_with(|| {
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
			MetaEvent::stake(RawEvent::ValidatorWentOffline(3, 2))
		);
		roll_to(21);
		let mut expected = vec![
			RawEvent::ValidatorChosen(2, 1, 700),
			RawEvent::ValidatorChosen(2, 2, 400),
			RawEvent::NewRound(5, 2, 2, 1100),
			RawEvent::ValidatorChosen(3, 1, 700),
			RawEvent::ValidatorChosen(3, 2, 400),
			RawEvent::NewRound(10, 3, 2, 1100),
			RawEvent::ValidatorWentOffline(3, 2),
			RawEvent::ValidatorChosen(4, 1, 700),
			RawEvent::NewRound(15, 4, 1, 700),
			RawEvent::ValidatorChosen(5, 1, 700),
			RawEvent::NewRound(20, 5, 1, 700),
		];
		assert_eq!(events(), expected);
		assert_noop!(
			Stake::go_offline(Origin::signed(2)),
			Error::<Test>::AlreadyOffline
		);
		assert_ok!(Stake::go_online(Origin::signed(2)));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::ValidatorBackOnline(5, 2))
		);
		expected.push(RawEvent::ValidatorBackOnline(5, 2));
		roll_to(26);
		expected.push(RawEvent::ValidatorChosen(6, 1, 700));
		expected.push(RawEvent::ValidatorChosen(6, 2, 400));
		expected.push(RawEvent::NewRound(25, 6, 2, 1100));
		assert_eq!(events(), expected);
	});
}

#[test]
fn join_validator_candidates_works() {
	two_validators_four_nominators().execute_with(|| {
		assert_noop!(
			Stake::join_candidates(Origin::signed(1), Perbill::from_percent(2), 11u128,),
			Error::<Test>::CandidateExists
		);
		assert_noop!(
			Stake::join_candidates(Origin::signed(3), Perbill::from_percent(2), 11u128,),
			Error::<Test>::NominatorExists
		);
		assert_noop!(
			Stake::join_candidates(Origin::signed(7), Perbill::from_percent(2), 9u128,),
			Error::<Test>::ValBondBelowMin
		);
		assert_noop!(
			Stake::join_candidates(Origin::signed(8), Perbill::from_percent(2), 10u128,),
			DispatchError::Module {
				index: 0,
				error: 3,
				message: Some("InsufficientBalance")
			}
		);
		assert_noop!(
			Stake::join_candidates(Origin::signed(7), Perbill::from_percent(51), 10u128,),
			Error::<Test>::FeeOverMax
		);
		assert!(Sys::events().is_empty());
		assert_ok!(Stake::join_candidates(
			Origin::signed(7),
			Perbill::from_percent(3),
			10u128,
		));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::JoinedValidatorCandidates(7, 10u128, 1110u128))
		);
	});
}

#[test]
fn validator_exit_executes_after_delay() {
	two_validators_four_nominators().execute_with(|| {
		roll_to(4);
		assert_noop!(
			Stake::leave_candidates(Origin::signed(3)),
			Error::<Test>::CandidateDNE
		);
		roll_to(11);
		assert_ok!(Stake::leave_candidates(Origin::signed(2)));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::ValidatorScheduledExit(3, 2, 5))
		);
		let info = <Stake as Store>::Candidates::get(&2).unwrap();
		assert_eq!(info.state, ValidatorStatus::Leaving(5));
		roll_to(21);
		// we must exclude leaving validators from rewards while
		// holding them retroactively accountable for previous faults
		// (within the last T::SlashingWindow blocks)
		let expected = vec![
			RawEvent::ValidatorChosen(2, 1, 700),
			RawEvent::ValidatorChosen(2, 2, 400),
			RawEvent::NewRound(5, 2, 2, 1100),
			RawEvent::ValidatorChosen(3, 1, 700),
			RawEvent::ValidatorChosen(3, 2, 400),
			RawEvent::NewRound(10, 3, 2, 1100),
			RawEvent::ValidatorScheduledExit(3, 2, 5),
			RawEvent::ValidatorChosen(4, 1, 700),
			RawEvent::NewRound(15, 4, 1, 700),
			RawEvent::ValidatorLeft(2, 400, 700),
			RawEvent::ValidatorChosen(5, 1, 700),
			RawEvent::NewRound(20, 5, 1, 700),
		];
		assert_eq!(events(), expected);
	});
}

#[test]
fn validator_selection_chooses_top_candidates() {
	five_validators_no_nominators().execute_with(|| {
		roll_to(4);
		roll_to(8);
		// should choose top MaxValidators (5), in order
		let expected = vec![
			RawEvent::ValidatorChosen(2, 1, 100),
			RawEvent::ValidatorChosen(2, 2, 90),
			RawEvent::ValidatorChosen(2, 3, 80),
			RawEvent::ValidatorChosen(2, 4, 70),
			RawEvent::ValidatorChosen(2, 5, 60),
			RawEvent::NewRound(5, 2, 5, 400),
		];
		assert_eq!(events(), expected);
		assert_ok!(Stake::leave_candidates(Origin::signed(6)));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::ValidatorScheduledExit(2, 6, 4))
		);
		roll_to(21);
		assert_ok!(Stake::join_candidates(
			Origin::signed(6),
			Perbill::from_percent(2),
			69u128
		));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::JoinedValidatorCandidates(6, 69u128, 469u128))
		);
		roll_to(27);
		// should choose top MaxValidators (5), in order
		let expected = vec![
			RawEvent::ValidatorChosen(2, 1, 100),
			RawEvent::ValidatorChosen(2, 2, 90),
			RawEvent::ValidatorChosen(2, 3, 80),
			RawEvent::ValidatorChosen(2, 4, 70),
			RawEvent::ValidatorChosen(2, 5, 60),
			RawEvent::NewRound(5, 2, 5, 400),
			RawEvent::ValidatorScheduledExit(2, 6, 4),
			RawEvent::ValidatorChosen(3, 1, 100),
			RawEvent::ValidatorChosen(3, 2, 90),
			RawEvent::ValidatorChosen(3, 3, 80),
			RawEvent::ValidatorChosen(3, 4, 70),
			RawEvent::ValidatorChosen(3, 5, 60),
			RawEvent::NewRound(10, 3, 5, 400),
			RawEvent::ValidatorLeft(6, 50, 400),
			RawEvent::ValidatorChosen(4, 1, 100),
			RawEvent::ValidatorChosen(4, 2, 90),
			RawEvent::ValidatorChosen(4, 3, 80),
			RawEvent::ValidatorChosen(4, 4, 70),
			RawEvent::ValidatorChosen(4, 5, 60),
			RawEvent::NewRound(15, 4, 5, 400),
			RawEvent::ValidatorChosen(5, 1, 100),
			RawEvent::ValidatorChosen(5, 2, 90),
			RawEvent::ValidatorChosen(5, 3, 80),
			RawEvent::ValidatorChosen(5, 4, 70),
			RawEvent::ValidatorChosen(5, 5, 60),
			RawEvent::NewRound(20, 5, 5, 400),
			RawEvent::JoinedValidatorCandidates(6, 69, 469),
			RawEvent::ValidatorChosen(6, 1, 100),
			RawEvent::ValidatorChosen(6, 2, 90),
			RawEvent::ValidatorChosen(6, 3, 80),
			RawEvent::ValidatorChosen(6, 4, 70),
			RawEvent::ValidatorChosen(6, 6, 69),
			RawEvent::NewRound(25, 6, 5, 409),
		];
		assert_eq!(events(), expected);
	});
}

#[test]
fn exit_queue_works() {
	five_validators_no_nominators().execute_with(|| {
		roll_to(4);
		roll_to(8);
		// should choose top MaxValidators (5), in order
		let mut expected = vec![
			RawEvent::ValidatorChosen(2, 1, 100),
			RawEvent::ValidatorChosen(2, 2, 90),
			RawEvent::ValidatorChosen(2, 3, 80),
			RawEvent::ValidatorChosen(2, 4, 70),
			RawEvent::ValidatorChosen(2, 5, 60),
			RawEvent::NewRound(5, 2, 5, 400),
		];
		assert_eq!(events(), expected);
		assert_ok!(Stake::leave_candidates(Origin::signed(6)));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::ValidatorScheduledExit(2, 6, 4))
		);
		roll_to(11);
		assert_ok!(Stake::leave_candidates(Origin::signed(5)));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::ValidatorScheduledExit(3, 5, 5))
		);
		roll_to(16);
		assert_ok!(Stake::leave_candidates(Origin::signed(4)));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::ValidatorScheduledExit(4, 4, 6))
		);
		assert_noop!(
			Stake::leave_candidates(Origin::signed(4)),
			Error::<Test>::AlreadyLeaving
		);
		roll_to(21);
		let mut new_events = vec![
			RawEvent::ValidatorScheduledExit(2, 6, 4),
			RawEvent::ValidatorChosen(3, 1, 100),
			RawEvent::ValidatorChosen(3, 2, 90),
			RawEvent::ValidatorChosen(3, 3, 80),
			RawEvent::ValidatorChosen(3, 4, 70),
			RawEvent::ValidatorChosen(3, 5, 60),
			RawEvent::NewRound(10, 3, 5, 400),
			RawEvent::ValidatorScheduledExit(3, 5, 5),
			RawEvent::ValidatorLeft(6, 50, 400),
			RawEvent::ValidatorChosen(4, 1, 100),
			RawEvent::ValidatorChosen(4, 2, 90),
			RawEvent::ValidatorChosen(4, 3, 80),
			RawEvent::ValidatorChosen(4, 4, 70),
			RawEvent::NewRound(15, 4, 4, 340),
			RawEvent::ValidatorScheduledExit(4, 4, 6),
			RawEvent::ValidatorLeft(5, 60, 340),
			RawEvent::ValidatorChosen(5, 1, 100),
			RawEvent::ValidatorChosen(5, 2, 90),
			RawEvent::ValidatorChosen(5, 3, 80),
			RawEvent::NewRound(20, 5, 3, 270),
		];
		expected.append(&mut new_events);
		assert_eq!(events(), expected);
	});
}

#[test]
fn payout_distribution_to_solo_validators() {
	five_validators_no_nominators().execute_with(|| {
		roll_to(4);
		roll_to(8);
		// should choose top MaxValidators (5), in order
		let mut expected = vec![
			RawEvent::ValidatorChosen(2, 1, 100),
			RawEvent::ValidatorChosen(2, 2, 90),
			RawEvent::ValidatorChosen(2, 3, 80),
			RawEvent::ValidatorChosen(2, 4, 70),
			RawEvent::ValidatorChosen(2, 5, 60),
			RawEvent::NewRound(5, 2, 5, 400),
		];
		assert_eq!(events(), expected);
		// ~ set block author as 1 for all blocks this round
		set_author(2, 1, 100);
		roll_to(16);
		// pay total issuance (=10) to 1
		let mut new = vec![
			RawEvent::ValidatorChosen(3, 1, 100),
			RawEvent::ValidatorChosen(3, 2, 90),
			RawEvent::ValidatorChosen(3, 3, 80),
			RawEvent::ValidatorChosen(3, 4, 70),
			RawEvent::ValidatorChosen(3, 5, 60),
			RawEvent::NewRound(10, 3, 5, 400),
			RawEvent::Rewarded(1, 305),
			RawEvent::ValidatorChosen(4, 1, 100),
			RawEvent::ValidatorChosen(4, 2, 90),
			RawEvent::ValidatorChosen(4, 3, 80),
			RawEvent::ValidatorChosen(4, 4, 70),
			RawEvent::ValidatorChosen(4, 5, 60),
			RawEvent::NewRound(15, 4, 5, 400),
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
			RawEvent::ValidatorChosen(5, 1, 100),
			RawEvent::ValidatorChosen(5, 2, 90),
			RawEvent::ValidatorChosen(5, 3, 80),
			RawEvent::ValidatorChosen(5, 4, 70),
			RawEvent::ValidatorChosen(5, 5, 60),
			RawEvent::NewRound(20, 5, 5, 400),
			RawEvent::Rewarded(1, 192),
			RawEvent::Rewarded(2, 128),
			RawEvent::ValidatorChosen(6, 1, 100),
			RawEvent::ValidatorChosen(6, 2, 90),
			RawEvent::ValidatorChosen(6, 3, 80),
			RawEvent::ValidatorChosen(6, 4, 70),
			RawEvent::ValidatorChosen(6, 5, 60),
			RawEvent::NewRound(25, 6, 5, 400),
		];
		expected.append(&mut new1);
		assert_eq!(events(), expected);
		// ~ each validator produces 1 block this round
		set_author(6, 1, 20);
		set_author(6, 2, 20);
		set_author(6, 3, 20);
		set_author(6, 4, 20);
		set_author(6, 5, 20);
		roll_to(36);
		// pay 20% issuance for all validators
		let mut new2 = vec![
			RawEvent::ValidatorChosen(7, 1, 100),
			RawEvent::ValidatorChosen(7, 2, 90),
			RawEvent::ValidatorChosen(7, 3, 80),
			RawEvent::ValidatorChosen(7, 4, 70),
			RawEvent::ValidatorChosen(7, 5, 60),
			RawEvent::NewRound(30, 7, 5, 400),
			RawEvent::Rewarded(5, 67),
			RawEvent::Rewarded(3, 67),
			RawEvent::Rewarded(1, 67),
			RawEvent::Rewarded(4, 67),
			RawEvent::Rewarded(2, 67),
			RawEvent::ValidatorChosen(8, 1, 100),
			RawEvent::ValidatorChosen(8, 2, 90),
			RawEvent::ValidatorChosen(8, 3, 80),
			RawEvent::ValidatorChosen(8, 4, 70),
			RawEvent::ValidatorChosen(8, 5, 60),
			RawEvent::NewRound(35, 8, 5, 400),
		];
		expected.append(&mut new2);
		assert_eq!(events(), expected);
		// check that distributing rewards clears awarded pts
		assert!(<Stake as Store>::AwardedPts::get(1, 1).is_zero());
		assert!(<Stake as Store>::AwardedPts::get(4, 1).is_zero());
		assert!(<Stake as Store>::AwardedPts::get(4, 2).is_zero());
		assert!(<Stake as Store>::AwardedPts::get(6, 1).is_zero());
		assert!(<Stake as Store>::AwardedPts::get(6, 2).is_zero());
		assert!(<Stake as Store>::AwardedPts::get(6, 3).is_zero());
		assert!(<Stake as Store>::AwardedPts::get(6, 4).is_zero());
		assert!(<Stake as Store>::AwardedPts::get(6, 5).is_zero());
	});
}

#[test]
fn payout_distribution_to_nominators() {
	five_validators_five_nominators().execute_with(|| {
		roll_to(4);
		roll_to(8);
		// chooses top MaxValidators (5), in order
		let mut expected = vec![
			RawEvent::ValidatorChosen(2, 1, 50),
			RawEvent::ValidatorChosen(2, 2, 40),
			RawEvent::ValidatorChosen(2, 4, 20),
			RawEvent::ValidatorChosen(2, 3, 20),
			RawEvent::ValidatorChosen(2, 5, 10),
			RawEvent::NewRound(5, 2, 5, 140),
		];
		assert_eq!(events(), expected);
		// ~ set block author as 1 for all blocks this round
		set_author(2, 1, 100);
		roll_to(16);
		// distribute total issuance (=10) to validator 1 and its nominators 6, 7, 19
		// -> NOTE that no fee is taken because validators at genesis set default 2% fee
		// and 2% of 10 is ~0 by the Perbill arithmetic
		let mut new = vec![
			RawEvent::ValidatorChosen(3, 1, 50),
			RawEvent::ValidatorChosen(3, 2, 40),
			RawEvent::ValidatorChosen(3, 4, 20),
			RawEvent::ValidatorChosen(3, 3, 20),
			RawEvent::ValidatorChosen(3, 5, 10),
			RawEvent::NewRound(10, 3, 5, 140),
			RawEvent::Rewarded(1, 20),
			RawEvent::Rewarded(6, 10),
			RawEvent::Rewarded(7, 10),
			RawEvent::Rewarded(10, 10),
			RawEvent::ValidatorChosen(4, 1, 50),
			RawEvent::ValidatorChosen(4, 2, 40),
			RawEvent::ValidatorChosen(4, 4, 20),
			RawEvent::ValidatorChosen(4, 3, 20),
			RawEvent::ValidatorChosen(4, 5, 10),
			RawEvent::NewRound(15, 4, 5, 140),
		];
		expected.append(&mut new);
		assert_eq!(events(), expected);
	});
}

#[test]
fn pays_validator_commission() {
	one_validator_two_nominators().execute_with(|| {
		roll_to(4);
		roll_to(8);
		// chooses top MaxValidators (5), in order
		let mut expected = vec![
			RawEvent::ValidatorChosen(2, 1, 40),
			RawEvent::NewRound(5, 2, 1, 40),
		];
		assert_eq!(events(), expected);
		assert_ok!(Stake::join_candidates(
			Origin::signed(4),
			Perbill::from_percent(20),
			20u128
		));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::JoinedValidatorCandidates(4, 20u128, 60u128))
		);
		roll_to(9);
		assert_ok!(Stake::join_nominators(Origin::signed(5), 4, 10));
		assert_ok!(Stake::join_nominators(Origin::signed(6), 4, 10));
		roll_to(11);
		let mut new = vec![
			RawEvent::JoinedValidatorCandidates(4, 20, 60),
			RawEvent::ValidatorNominated(5, 10, 4, 30),
			RawEvent::NominatorJoined(5, 10),
			RawEvent::ValidatorNominated(6, 10, 4, 40),
			RawEvent::NominatorJoined(6, 10),
			RawEvent::ValidatorChosen(3, 4, 40),
			RawEvent::ValidatorChosen(3, 1, 40),
			RawEvent::NewRound(10, 3, 2, 80),
		];
		expected.append(&mut new);
		assert_eq!(events(), expected);
		// only reward author with id 4
		set_author(3, 4, 100);
		roll_to(21);
		// 20% of 10 is commission + due_portion (4) = 2 + 4 = 6
		// all nominator payouts are 10-2 = 8 * stake_pct
		let mut new2 = vec![
			RawEvent::ValidatorChosen(4, 4, 40),
			RawEvent::ValidatorChosen(4, 1, 40),
			RawEvent::NewRound(15, 4, 2, 80),
			RawEvent::Rewarded(4, 18),
			RawEvent::Rewarded(5, 6),
			RawEvent::Rewarded(6, 6),
			RawEvent::ValidatorChosen(5, 4, 40),
			RawEvent::ValidatorChosen(5, 1, 40),
			RawEvent::NewRound(20, 5, 2, 80),
		];
		expected.append(&mut new2);
		assert_eq!(events(), expected);
	});
}

#[test]
fn multiple_nominations() {
	five_validators_five_nominators().execute_with(|| {
		roll_to(4);
		roll_to(8);
		// chooses top MaxValidators (5), in order
		let mut expected = vec![
			RawEvent::ValidatorChosen(2, 1, 50),
			RawEvent::ValidatorChosen(2, 2, 40),
			RawEvent::ValidatorChosen(2, 4, 20),
			RawEvent::ValidatorChosen(2, 3, 20),
			RawEvent::ValidatorChosen(2, 5, 10),
			RawEvent::NewRound(5, 2, 5, 140),
		];
		assert_eq!(events(), expected);
		assert_noop!(
			Stake::nominate_new(Origin::signed(5), 2, 10),
			Error::<Test>::NominatorDNE,
		);
		assert_noop!(
			Stake::nominate_new(Origin::signed(11), 1, 10),
			Error::<Test>::NominatorDNE,
		);
		assert_noop!(
			Stake::nominate_new(Origin::signed(6), 1, 10),
			Error::<Test>::AlreadyNominatedValidator,
		);
		assert_noop!(
			Stake::nominate_new(Origin::signed(6), 2, 2),
			Error::<Test>::NominationBelowMin,
		);
		assert_ok!(Stake::nominate_new(Origin::signed(6), 2, 10));
		assert_ok!(Stake::nominate_new(Origin::signed(6), 3, 10));
		assert_ok!(Stake::nominate_new(Origin::signed(6), 4, 10));
		assert_noop!(
			Stake::nominate_new(Origin::signed(6), 5, 10),
			Error::<Test>::ExceedMaxValidatorsPerNom,
		);
		roll_to(16);
		let mut new = vec![
			RawEvent::ValidatorNominated(6, 10, 2, 50),
			RawEvent::ValidatorNominated(6, 10, 3, 30),
			RawEvent::ValidatorNominated(6, 10, 4, 30),
			RawEvent::ValidatorChosen(3, 2, 50),
			RawEvent::ValidatorChosen(3, 1, 50),
			RawEvent::ValidatorChosen(3, 4, 30),
			RawEvent::ValidatorChosen(3, 3, 30),
			RawEvent::ValidatorChosen(3, 5, 10),
			RawEvent::NewRound(10, 3, 5, 170),
			RawEvent::ValidatorChosen(4, 2, 50),
			RawEvent::ValidatorChosen(4, 1, 50),
			RawEvent::ValidatorChosen(4, 4, 30),
			RawEvent::ValidatorChosen(4, 3, 30),
			RawEvent::ValidatorChosen(4, 5, 10),
			RawEvent::NewRound(15, 4, 5, 170),
		];
		expected.append(&mut new);
		assert_eq!(events(), expected);
		roll_to(21);
		assert_ok!(Stake::nominate_new(Origin::signed(7), 2, 80));
		assert_noop!(
			Stake::nominate_new(Origin::signed(7), 3, 11),
			DispatchError::Module {
				index: 0,
				error: 3,
				message: Some("InsufficientBalance")
			},
		);
		assert_noop!(
			Stake::nominate_new(Origin::signed(10), 2, 10),
			Error::<Test>::TooManyNominators
		);
		roll_to(26);
		let mut new2 = vec![
			RawEvent::ValidatorChosen(5, 2, 50),
			RawEvent::ValidatorChosen(5, 1, 50),
			RawEvent::ValidatorChosen(5, 4, 30),
			RawEvent::ValidatorChosen(5, 3, 30),
			RawEvent::ValidatorChosen(5, 5, 10),
			RawEvent::NewRound(20, 5, 5, 170),
			RawEvent::ValidatorNominated(7, 80, 2, 130),
			RawEvent::ValidatorChosen(6, 2, 130),
			RawEvent::ValidatorChosen(6, 1, 50),
			RawEvent::ValidatorChosen(6, 4, 30),
			RawEvent::ValidatorChosen(6, 3, 30),
			RawEvent::ValidatorChosen(6, 5, 10),
			RawEvent::NewRound(25, 6, 5, 250),
		];
		expected.append(&mut new2);
		assert_eq!(events(), expected);
		assert_ok!(Stake::leave_candidates(Origin::signed(2)));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::ValidatorScheduledExit(6, 2, 8))
		);
		roll_to(31);
		let mut new3 = vec![
			RawEvent::ValidatorScheduledExit(6, 2, 8),
			RawEvent::ValidatorChosen(7, 1, 50),
			RawEvent::ValidatorChosen(7, 4, 30),
			RawEvent::ValidatorChosen(7, 3, 30),
			RawEvent::ValidatorChosen(7, 5, 10),
			RawEvent::NewRound(30, 7, 4, 120),
		];
		expected.append(&mut new3);
		assert_eq!(events(), expected);
		// verify that nominations are removed after validator leaves, not before
		assert_eq!(<Stake as Store>::Nominators::get(7).unwrap().total, 90);
		assert_eq!(
			<Stake as Store>::Nominators::get(7)
				.unwrap()
				.nominations
				.0
				.len(),
			2usize
		);
		assert_eq!(<Stake as Store>::Nominators::get(6).unwrap().total, 40);
		assert_eq!(
			<Stake as Store>::Nominators::get(6)
				.unwrap()
				.nominations
				.0
				.len(),
			4usize
		);
		assert_eq!(Balances::reserved_balance(&6), 40);
		assert_eq!(Balances::reserved_balance(&7), 90);
		assert_eq!(Balances::free_balance(&6), 60);
		assert_eq!(Balances::free_balance(&7), 10);
		roll_to(40);
		assert_eq!(<Stake as Store>::Nominators::get(7).unwrap().total, 10);
		assert_eq!(<Stake as Store>::Nominators::get(6).unwrap().total, 30);
		assert_eq!(
			<Stake as Store>::Nominators::get(7)
				.unwrap()
				.nominations
				.0
				.len(),
			1usize
		);
		assert_eq!(
			<Stake as Store>::Nominators::get(6)
				.unwrap()
				.nominations
				.0
				.len(),
			3usize
		);
		assert_eq!(Balances::reserved_balance(&6), 30);
		assert_eq!(Balances::reserved_balance(&7), 10);
		assert_eq!(Balances::free_balance(&6), 70);
		assert_eq!(Balances::free_balance(&7), 90);
	});
}

#[test]
fn validators_bond_more_less() {
	five_validators_five_nominators().execute_with(|| {
		roll_to(4);
		assert_noop!(
			Stake::candidate_bond_more(Origin::signed(6), 50),
			Error::<Test>::CandidateDNE
		);
		assert_ok!(Stake::candidate_bond_more(Origin::signed(1), 50));
		assert_noop!(
			Stake::candidate_bond_more(Origin::signed(1), 40),
			DispatchError::Module {
				index: 0,
				error: 3,
				message: Some("InsufficientBalance")
			}
		);
		assert_ok!(Stake::leave_candidates(Origin::signed(1)));
		assert_noop!(
			Stake::candidate_bond_more(Origin::signed(1), 30),
			Error::<Test>::CannotActivateIfLeaving
		);
		roll_to(30);
		assert_noop!(
			Stake::candidate_bond_more(Origin::signed(1), 40),
			Error::<Test>::CandidateDNE
		);
		assert_ok!(Stake::candidate_bond_more(Origin::signed(2), 80));
		assert_ok!(Stake::candidate_bond_less(Origin::signed(2), 90));
		assert_ok!(Stake::candidate_bond_less(Origin::signed(3), 10));
		assert_noop!(
			Stake::candidate_bond_less(Origin::signed(2), 11),
			Error::<Test>::Underflow
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
	});
}

#[test]
fn nominators_bond_more_less() {
	five_validators_five_nominators().execute_with(|| {
		roll_to(4);
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
			Error::<Test>::Underflow
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
		assert_noop!(
			Stake::nominator_bond_less(Origin::signed(6), 2, 5),
			Error::<Test>::NominationDNE
		);
		assert_noop!(
			Stake::nominator_bond_more(Origin::signed(6), 1, 81),
			DispatchError::Module {
				index: 0,
				error: 3,
				message: Some("InsufficientBalance")
			}
		);
		roll_to(9);
		assert_eq!(Balances::reserved_balance(&6), 20);
		assert_ok!(Stake::leave_candidates(Origin::signed(1)));
		roll_to(31);
		assert!(!Stake::is_nominator(&6));
		assert_eq!(Balances::reserved_balance(&6), 0);
		assert_eq!(Balances::free_balance(&6), 100);
	});
}

#[test]
fn switch_nomination_works() {
	five_validators_five_nominators().execute_with(|| {
		roll_to(4);
		roll_to(8);
		let mut expected = vec![
			RawEvent::ValidatorChosen(2, 1, 50),
			RawEvent::ValidatorChosen(2, 2, 40),
			RawEvent::ValidatorChosen(2, 4, 20),
			RawEvent::ValidatorChosen(2, 3, 20),
			RawEvent::ValidatorChosen(2, 5, 10),
			RawEvent::NewRound(5, 2, 5, 140),
		];
		assert_eq!(events(), expected);
		assert_noop!(
			Stake::switch_nomination(Origin::signed(1), 1, 2),
			Error::<Test>::NominatorDNE
		);
		assert_noop!(
			Stake::switch_nomination(Origin::signed(6), 1, 7),
			Error::<Test>::CandidateDNE
		);
		assert_noop!(
			Stake::switch_nomination(Origin::signed(6), 2, 1),
			Error::<Test>::NominationDNE
		);
		assert_noop!(
			Stake::switch_nomination(Origin::signed(6), 1, 1),
			Error::<Test>::CannotSwitchToSameNomination
		);
		assert_ok!(Stake::switch_nomination(Origin::signed(6), 1, 2));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::NominationSwapped(6, 10, 1, 2))
		);
		assert_ok!(Stake::switch_nomination(Origin::signed(7), 1, 2));
		assert_ok!(Stake::switch_nomination(Origin::signed(8), 2, 1));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::NominationSwapped(8, 10, 2, 1))
		);
		assert_ok!(Stake::switch_nomination(Origin::signed(9), 2, 1));
		assert_ok!(Stake::switch_nomination(Origin::signed(10), 1, 2));
		assert_eq!(
			last_event(),
			MetaEvent::stake(RawEvent::NominationSwapped(10, 10, 1, 2))
		);
		// verify nothing changed with roles or balances since genesis
		for x in 1..5 {
			assert!(Stake::is_candidate(&x));
			assert_eq!(Balances::free_balance(&x), 80);
			assert_eq!(Balances::reserved_balance(&x), 20);
		}
		assert!(Stake::is_candidate(&5));
		assert_eq!(Balances::free_balance(&5), 90);
		assert_eq!(Balances::reserved_balance(&5), 10);
		for x in 6..11 {
			assert!(Stake::is_nominator(&x));
			assert_eq!(Balances::free_balance(&x), 90);
			assert_eq!(Balances::reserved_balance(&x), 10);
		}
		roll_to(10);
		roll_to(16);
		let mut new = vec![
			RawEvent::NominationSwapped(6, 10, 1, 2),
			RawEvent::NominationSwapped(7, 10, 1, 2),
			RawEvent::NominationSwapped(8, 10, 2, 1),
			RawEvent::NominationSwapped(9, 10, 2, 1),
			RawEvent::NominationSwapped(10, 10, 1, 2),
			RawEvent::ValidatorChosen(3, 2, 50),
			RawEvent::ValidatorChosen(3, 1, 40),
			RawEvent::ValidatorChosen(3, 4, 20),
			RawEvent::ValidatorChosen(3, 3, 20),
			RawEvent::ValidatorChosen(3, 5, 10),
			RawEvent::NewRound(10, 3, 5, 140),
			RawEvent::ValidatorChosen(4, 2, 50),
			RawEvent::ValidatorChosen(4, 1, 40),
			RawEvent::ValidatorChosen(4, 4, 20),
			RawEvent::ValidatorChosen(4, 3, 20),
			RawEvent::ValidatorChosen(4, 5, 10),
			RawEvent::NewRound(15, 4, 5, 140),
		];
		expected.append(&mut new);
		assert_eq!(events(), expected);
	});
}
