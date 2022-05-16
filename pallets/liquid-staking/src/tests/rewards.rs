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
fn auto_compound_candidate_only() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_CANDIDATE_1, 1_000_000_000_000_000)])
		.build()
		.execute_with(|| {
			// we need to test with high amount of shares
			// such that rewards can give non-zero integer amount of
			// AC shares.
			assert_ok!(LiquidStaking::stake_auto_compounding(
				Origin::signed(ACCOUNT_CANDIDATE_1),
				ACCOUNT_CANDIDATE_1,
				SharesOrStake::Shares(10_000)
			));
			assert_eq!(balance(&ACCOUNT_CANDIDATE_1), 1 * PETA - 10 * MEGA);
			assert_eq!(balance(&ACCOUNT_STAKING), 10 * MEGA);

			let rewards = 1 * MEGA;
			assert_ok!(crate::rewards::distribute_rewards::<Runtime>(
				ACCOUNT_CANDIDATE_1,
				rewards
			));
			// assert_eq!(balance(&ACCOUNT_CANDIDATE_1), 990_000_000_000);
			// assert_eq!(balance(&ACCOUNT_STAKING), 10_000_000_000);

			assert_eq_events!(vec![
				Event::StakedAutoCompounding {
					candidate: ACCOUNT_CANDIDATE_1,
					delegator: ACCOUNT_CANDIDATE_1,
					shares: 10_000,
					stake: 10 * MEGA,
				},
				Event::IncreasedStake {
					candidate: ACCOUNT_CANDIDATE_1,
					stake: 10 * MEGA,
				},
				Event::UpdatedCandidatePosition {
					candidate: ACCOUNT_CANDIDATE_1,
					stake: 10 * MEGA,
					self_delegation: 10 * MEGA,
					before: None,
					after: Some(0)
				},
				// Colator rewards
				Event::StakedAutoCompounding {
					candidate: ACCOUNT_CANDIDATE_1,
					delegator: ACCOUNT_CANDIDATE_1,
					shares: rewards * 2 / 10_000, // stake/share ratio
					stake: rewards * 2 / 10,
				},
				// Delegators rewards
				Event::IncreasedStake {
					candidate: ACCOUNT_CANDIDATE_1,
					stake: rewards * 7 / 10,
				},
				Event::UpdatedCandidatePosition {
					candidate: ACCOUNT_CANDIDATE_1,
					stake: 10 * MEGA + rewards * 7 / 10,
					self_delegation: 10 * MEGA + rewards * 7 / 10,
					before: Some(0),
					after: Some(0),
				},
				// Final events
				Event::RewardedCollator {
					collator: ACCOUNT_CANDIDATE_1,
					auto_compounding_rewards: rewards * 2 / 10,
					manual_claim_rewards: 0,
				},
				Event::RewardedDelegators {
					collator: ACCOUNT_CANDIDATE_1,
					auto_compounding_rewards: rewards * 5 / 10,
					manual_claim_rewards: 0,
				},
			]);
		});
}

#[test]
fn manual_claim_candidate_only() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(LiquidStaking::stake_manual_claim(
			Origin::signed(ACCOUNT_CANDIDATE_1),
			ACCOUNT_CANDIDATE_1,
			SharesOrStake::Shares(10_000)
		));
		assert_eq!(balance(&ACCOUNT_CANDIDATE_1), 1 * PETA - 10 * MEGA);
		assert_eq!(balance(&ACCOUNT_STAKING), 10 * MEGA);

		let rewards = 1 * MEGA;
		assert_ok!(crate::rewards::distribute_rewards::<Runtime>(
			ACCOUNT_CANDIDATE_1,
			rewards
		));
		// assert_eq!(balance(&ACCOUNT_CANDIDATE_1), 990_000_000_000);
		// assert_eq!(balance(&ACCOUNT_STAKING), 10_000_000_000);

		assert_eq_events!(vec![
			Event::StakedManualClaim {
				candidate: ACCOUNT_CANDIDATE_1,
				delegator: ACCOUNT_CANDIDATE_1,
				shares: 10_000,
				stake: 10 * MEGA,
			},
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 10 * MEGA,
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1,
				stake: 10 * MEGA,
				self_delegation: 10 * MEGA,
				before: None,
				after: Some(0)
			},
			// ------
			Event::RewardedCollator {
				collator: ACCOUNT_CANDIDATE_1,
				auto_compounding_rewards: 0,
				manual_claim_rewards: rewards * 2 / 10,
			},
			Event::RewardedDelegators {
				collator: ACCOUNT_CANDIDATE_1,
				auto_compounding_rewards: 0,
				manual_claim_rewards: rewards * 5 / 10,
			},
		]);
	});
}
