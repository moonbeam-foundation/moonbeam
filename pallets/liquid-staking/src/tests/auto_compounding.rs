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

use super::*;

#[test]
fn empty_delegation() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			LiquidStaking::stake_auto_compounding(
				Origin::signed(ACCOUNT_DELEGATOR_1),
				ACCOUNT_CANDIDATE_1,
				SharesOrStake::Shares(0)
			),
			Error::<Runtime>::StakeMustBeNonZero
		);
		assert_eq!(balance(&ACCOUNT_DELEGATOR_1), 1_000_000_000_000);
		assert_eq!(balance(&ACCOUNT_STAKING), 0);
	});
}

#[test]
fn single_delegation() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(LiquidStaking::stake_auto_compounding(
			Origin::signed(ACCOUNT_DELEGATOR_1),
			ACCOUNT_CANDIDATE_1,
			SharesOrStake::Shares(1)
		));
		assert_eq!(balance(&ACCOUNT_DELEGATOR_1), 999_000_000_000);
		assert_eq!(balance(&ACCOUNT_STAKING), 1_000_000_000);

		assert_eq_events!(vec![
			Event::StakedAutoCompounding {
				candidate: ACCOUNT_CANDIDATE_1,
				delegator: ACCOUNT_DELEGATOR_1,
				shares: 1,
				stake: 1_000_000_000, // InitialManualClaimShareValue
			},
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 1_000_000_000, // InitialManualClaimShareValue
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 1_000_000_000, // InitialManualClaimShareValue
				self_delegation: 0,
				before: None,
				after: None,
			},
		]);
	});
}

#[test]
fn low_self_delegation() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(LiquidStaking::stake_auto_compounding(
			Origin::signed(ACCOUNT_CANDIDATE_1),
			ACCOUNT_CANDIDATE_1,
			SharesOrStake::Shares(1)
		));
		assert_eq!(balance(&ACCOUNT_CANDIDATE_1), 999_000_000_000);
		assert_eq!(balance(&ACCOUNT_STAKING), 1_000_000_000);

		assert_eq_events!(vec![
			Event::StakedAutoCompounding {
				candidate: ACCOUNT_CANDIDATE_1,
				delegator: ACCOUNT_CANDIDATE_1,
				shares: 1,
				stake: 1_000_000_000, // InitialManualClaimShareValue
			},
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 1_000_000_000, // InitialManualClaimShareValue
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 1_000_000_000, // InitialManualClaimShareValue
				self_delegation: 1_000_000_000,
				before: None,
				after: None,
			},
		]);
	});
}

#[test]
fn sufficient_self_delegation() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(LiquidStaking::stake_auto_compounding(
			Origin::signed(ACCOUNT_CANDIDATE_1),
			ACCOUNT_CANDIDATE_1,
			SharesOrStake::Shares(10)
		));
		assert_eq!(balance(&ACCOUNT_CANDIDATE_1), 990_000_000_000);
		assert_eq!(balance(&ACCOUNT_STAKING), 10_000_000_000);

		assert_eq_events!(vec![
			Event::StakedAutoCompounding {
				candidate: ACCOUNT_CANDIDATE_1,
				delegator: ACCOUNT_CANDIDATE_1,
				shares: 10,
				stake: 10_000_000_000,
			},
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 10_000_000_000,
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 10_000_000_000,
				self_delegation: 10_000_000_000,
				before: None,
				after: Some(0),
			},
		]);
	});
}

#[test]
fn multi_delegations() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(LiquidStaking::stake_auto_compounding(
			Origin::signed(ACCOUNT_DELEGATOR_1),
			ACCOUNT_CANDIDATE_1,
			SharesOrStake::Shares(1)
		));
		assert_eq!(balance(&ACCOUNT_DELEGATOR_1), 999_000_000_000);
		assert_eq!(balance(&ACCOUNT_STAKING), 1_000_000_000);

		assert_ok!(LiquidStaking::stake_auto_compounding(
			Origin::signed(ACCOUNT_CANDIDATE_1),
			ACCOUNT_CANDIDATE_1,
			SharesOrStake::Shares(10)
		));
		assert_eq!(balance(&ACCOUNT_CANDIDATE_1), 990_000_000_000);
		assert_eq!(balance(&ACCOUNT_STAKING), 11_000_000_000);

		assert_ok!(LiquidStaking::stake_auto_compounding(
			Origin::signed(ACCOUNT_DELEGATOR_2),
			ACCOUNT_CANDIDATE_1,
			SharesOrStake::Shares(2)
		));
		assert_eq!(balance(&ACCOUNT_DELEGATOR_2), 998_000_000_000);
		assert_eq!(balance(&ACCOUNT_STAKING), 13_000_000_000);

		assert_eq_events!(vec![
			Event::StakedAutoCompounding {
				candidate: ACCOUNT_CANDIDATE_1,
				delegator: ACCOUNT_DELEGATOR_1,
				shares: 1,
				stake: 1_000_000_000,
			},
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 1_000_000_000,
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 1_000_000_000,
				self_delegation: 0,
				before: None,
				after: None,
			},
			// -----
			Event::StakedAutoCompounding {
				candidate: ACCOUNT_CANDIDATE_1,
				delegator: ACCOUNT_CANDIDATE_1,
				shares: 10,
				stake: 10_000_000_000,
			},
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 10_000_000_000,
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 11_000_000_000,
				self_delegation: 10_000_000_000,
				before: None,
				after: Some(0),
			},
			// -----
			Event::StakedAutoCompounding {
				candidate: ACCOUNT_CANDIDATE_1,
				delegator: ACCOUNT_DELEGATOR_2,
				shares: 2,
				stake: 2_000_000_000,
			},
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 2_000_000_000,
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 13_000_000_000,
				self_delegation: 10_000_000_000,
				before: Some(0),
				after: Some(0),
			},
		]);
	});
}

#[test]
fn stake_amount_too_low() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			LiquidStaking::stake_auto_compounding(
				Origin::signed(ACCOUNT_DELEGATOR_1),
				ACCOUNT_CANDIDATE_1,
				SharesOrStake::Stake(999_999_999), // 1 below minimum
			),
			Error::<Runtime>::StakeMustBeNonZero
		);
		assert_eq!(balance(&ACCOUNT_DELEGATOR_1), 1_000_000_000_000);
		assert_eq!(balance(&ACCOUNT_STAKING), 0);
	});
}

#[test]
fn stake_single_delegation() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(LiquidStaking::stake_auto_compounding(
			Origin::signed(ACCOUNT_DELEGATOR_1),
			ACCOUNT_CANDIDATE_1,
			SharesOrStake::Stake(1_999_999_999), // will be rounded down to 1_000_000_000
		));
		assert_eq!(balance(&ACCOUNT_DELEGATOR_1), 999_000_000_000);
		assert_eq!(balance(&ACCOUNT_STAKING), 1_000_000_000);

		assert_eq_events!(vec![
			Event::StakedAutoCompounding {
				candidate: ACCOUNT_CANDIDATE_1,
				delegator: ACCOUNT_DELEGATOR_1,
				shares: 1,
				stake: 1_000_000_000, // InitialManualClaimShareValue
			},
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 1_000_000_000, // InitialManualClaimShareValue
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 1_000_000_000, // InitialManualClaimShareValue
				self_delegation: 0,
				before: None,
				after: None,
			},
		]);
	});
}
