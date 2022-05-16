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
fn both_contribute_to_candidate_stake() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(LiquidStaking::stake_manual_claim(
			Origin::signed(ACCOUNT_CANDIDATE_1),
			ACCOUNT_CANDIDATE_1,
			SharesOrStake::Shares(10)
		));
		assert_eq!(balance(&ACCOUNT_CANDIDATE_1), 1 * PETA - 10 * KILO);
		assert_eq!(balance(&ACCOUNT_STAKING), 10 * KILO);

		assert_ok!(LiquidStaking::stake_auto_compounding(
			Origin::signed(ACCOUNT_CANDIDATE_1),
			ACCOUNT_CANDIDATE_1,
			SharesOrStake::Shares(5)
		));
		assert_eq!(balance(&ACCOUNT_CANDIDATE_1), 1 * PETA - 15 * KILO);
		assert_eq!(balance(&ACCOUNT_STAKING), 15 * KILO);

		assert_eq_events!(vec![
			Event::StakedManualClaim {
				candidate: ACCOUNT_CANDIDATE_1,
				delegator: ACCOUNT_CANDIDATE_1,
				shares: 10,
				stake: 10 * KILO,
			},
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 10 * KILO,
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 10 * KILO,
				self_delegation: 10 * KILO,
				before: None,
				after: Some(0)
			},
			Event::StakedAutoCompounding {
				candidate: ACCOUNT_CANDIDATE_1,
				delegator: ACCOUNT_CANDIDATE_1,
				shares: 5,
				stake: 5 * KILO,
			},
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 5 * KILO,
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 15 * KILO,
				self_delegation: 15 * KILO,
				before: Some(0),
				after: Some(0),
			},
		]);
	});
}
