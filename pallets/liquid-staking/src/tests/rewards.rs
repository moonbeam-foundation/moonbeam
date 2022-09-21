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

use {super::*, crate::CandidateExt};

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
				ACCOUNT_CANDIDATE_1.with_gen(0),
				SharesOrStake::Shares(10_000)
			));
			assert_eq!(balance(&ACCOUNT_CANDIDATE_1), 1 * PETA - 10 * MEGA);
			assert_eq!(balance(&ACCOUNT_STAKING), 10 * MEGA);

			let rewards = 1 * MEGA;
			let rewards_delegator = rewards * 5 / 10; // 50%;

			assert_ok!(crate::rewards::distribute_rewards::<Runtime>(
				ACCOUNT_CANDIDATE_1.with_gen(0),
				rewards
			));

			// Distributing delegators AC rewards change the value of an AC share.
			// Collator AC rewards are distributed after delegator AC rewards to not give the collator
			// more shares which would give them a bigger part.
			let new_ac_share_value = crate::pools::auto_compounding::shares_to_stake::<Runtime>(
				&ACCOUNT_CANDIDATE_1.with_gen(0),
				1,
			)
			.unwrap();
			// Distributing AC rewards have rounding.
			let rewards_collator = rewards * 2 / 10; // 20%
			let rewards_collator_ac_in_shares = rewards_collator / new_ac_share_value;
			let rewards_collator_ac = rewards_collator_ac_in_shares * new_ac_share_value;
			let rewards_collator_mc = rewards_collator - rewards_collator_ac;

			assert_eq!(balance(&ACCOUNT_RESERVE), rewards * 3 / 10);
			assert_eq!(
				balance(&ACCOUNT_CANDIDATE_1),
				1 * PETA - 10 * MEGA + rewards_collator_mc
			);
			assert_eq!(
				balance(&ACCOUNT_STAKING),
				10 * MEGA + rewards_collator_ac + rewards_delegator
			);

			assert_eq_events!(vec![
				Event::StakedAutoCompounding {
					candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
					delegator: ACCOUNT_CANDIDATE_1,
					shares: 10_000,
					stake: 10 * MEGA,
				},
				Event::IncreasedStake {
					candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
					stake: 10 * MEGA,
				},
				Event::UpdatedCandidatePosition {
					candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
					stake: 10 * MEGA,
					self_delegation: 10 * MEGA,
					before: None,
					after: Some(0)
				},
				// Colator rewards
				Event::StakedAutoCompounding {
					candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
					delegator: ACCOUNT_CANDIDATE_1,
					shares: rewards_collator_ac_in_shares,
					stake: rewards_collator_ac,
				},
				// Update total stake following AC reward distribution.
				Event::IncreasedStake {
					candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
					stake: rewards_delegator + rewards_collator_ac,
				},
				Event::UpdatedCandidatePosition {
					candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
					stake: 10 * MEGA + rewards_delegator + rewards_collator_ac,
					self_delegation: 10 * MEGA + rewards_delegator + rewards_collator_ac,
					before: Some(0),
					after: Some(0),
				},
				// Final events
				Event::RewardedCollator {
					collator: ACCOUNT_CANDIDATE_1.with_gen(0),
					auto_compounding_rewards: rewards_collator_ac,
					manual_claim_rewards: rewards_collator_mc,
				},
				Event::RewardedDelegators {
					collator: ACCOUNT_CANDIDATE_1.with_gen(0),
					auto_compounding_rewards: rewards_delegator,
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
			ACCOUNT_CANDIDATE_1.with_gen(0),
			SharesOrStake::Shares(10_000)
		));
		assert_eq!(balance(&ACCOUNT_CANDIDATE_1), 1 * PETA - 10 * MEGA);
		assert_eq!(balance(&ACCOUNT_STAKING), 10 * MEGA);

		let rewards = 1 * MEGA;
		assert_ok!(crate::rewards::distribute_rewards::<Runtime>(
			ACCOUNT_CANDIDATE_1.with_gen(0),
			rewards
		));

		assert_eq!(balance(&ACCOUNT_RESERVE), rewards * 3 / 10);
		assert_eq!(
			balance(&ACCOUNT_CANDIDATE_1),
			1 * PETA - 10 * MEGA + (rewards * 2 / 10)
		);
		assert_eq!(balance(&ACCOUNT_STAKING), 10 * MEGA + (rewards * 5 / 10));

		assert_eq_events!(vec![
			Event::StakedManualClaim {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				delegator: ACCOUNT_CANDIDATE_1,
				shares: 10_000,
				stake: 10 * MEGA,
			},
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				stake: 10 * MEGA,
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				stake: 10 * MEGA,
				self_delegation: 10 * MEGA,
				before: None,
				after: Some(0)
			},
			// ------
			Event::RewardedCollator {
				collator: ACCOUNT_CANDIDATE_1.with_gen(0),
				auto_compounding_rewards: 0,
				manual_claim_rewards: rewards * 2 / 10,
			},
			Event::RewardedDelegators {
				collator: ACCOUNT_CANDIDATE_1.with_gen(0),
				auto_compounding_rewards: 0,
				manual_claim_rewards: rewards * 5 / 10,
			},
		]);
	});
}

#[test]
fn mixed_candidate_only() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(LiquidStaking::stake_manual_claim(
			Origin::signed(ACCOUNT_CANDIDATE_1),
			ACCOUNT_CANDIDATE_1.with_gen(0),
			SharesOrStake::Shares(10_000)
		));
		assert_ok!(LiquidStaking::stake_auto_compounding(
			Origin::signed(ACCOUNT_CANDIDATE_1),
			ACCOUNT_CANDIDATE_1.with_gen(0),
			SharesOrStake::Shares(30_000)
		));

		assert_eq!(balance(&ACCOUNT_CANDIDATE_1), 1 * PETA - 40 * MEGA);
		assert_eq!(balance(&ACCOUNT_STAKING), 40 * MEGA);

		let rewards = 1 * MEGA;
		let rewards_reserve = rewards * 3 / 10;

		let shared_rewards = rewards - rewards_reserve;
		let rewards_delegator = rewards * 5 / 10; // 50%

		// Distributing MC rewards have rounding.
		let rewards_delegator_mc = round_down(rewards_delegator * 1 / 4, 10_000); // 25% MC
		let rewards_delegator_ac = rewards_delegator - rewards_delegator_mc;

		assert_ok!(crate::rewards::distribute_rewards::<Runtime>(
			ACCOUNT_CANDIDATE_1.with_gen(0),
			rewards
		));

		// Distributing delegators AC rewards change the value of an AC share.
		// Collator AC rewards are distributed after delegator AC rewards to not give the collator
		// more shares which would give them a bigger part.

		// Distributing AC rewards have rounding.
		let rewards_collator = shared_rewards - rewards_delegator; // 20%
		let rewards_collator_ac_in_shares =
			crate::pools::auto_compounding::stake_to_shares::<Runtime>(
				&ACCOUNT_CANDIDATE_1.with_gen(0),
				rewards_collator * 3 / 4,
			)
			.unwrap(); // 75% AC
		let rewards_collator_ac = crate::pools::auto_compounding::shares_to_stake::<Runtime>(
			&ACCOUNT_CANDIDATE_1.with_gen(0),
			rewards_collator_ac_in_shares,
		)
		.unwrap();
		let rewards_collator_mc = rewards_collator - rewards_collator_ac;

		assert_eq!(balance(&ACCOUNT_RESERVE), rewards * 3 / 10);
		assert_eq!(
			balance(&ACCOUNT_CANDIDATE_1),
			1 * PETA - 40 * MEGA + rewards_collator_mc
		);
		assert_eq!(
			balance(&ACCOUNT_STAKING),
			40 * MEGA + rewards_collator_ac + rewards_delegator
		);

		assert_eq_events!(vec![
			Event::StakedManualClaim {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				delegator: ACCOUNT_CANDIDATE_1,
				shares: 10_000,
				stake: 10 * MEGA,
			},
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				stake: 10 * MEGA,
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				stake: 10 * MEGA,
				self_delegation: 10 * MEGA,
				before: None,
				after: Some(0)
			},
			// ------
			Event::StakedAutoCompounding {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				delegator: ACCOUNT_CANDIDATE_1,
				shares: 30_000,
				stake: 30 * MEGA,
			},
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				stake: 30 * MEGA,
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				stake: 40 * MEGA,
				self_delegation: 40 * MEGA,
				before: Some(0),
				after: Some(0)
			},
			// ------
			// Colator rewards
			Event::StakedAutoCompounding {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				delegator: ACCOUNT_CANDIDATE_1,
				shares: rewards_collator_ac_in_shares,
				stake: rewards_collator_ac,
			},
			// Update total stake following AC reward distribution.
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				stake: rewards_collator_ac + rewards_delegator_ac,
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				stake: 40 * MEGA + rewards_collator_ac + rewards_delegator_ac,
				self_delegation: 40 * MEGA + rewards_collator_ac + rewards_delegator_ac,
				before: Some(0),
				after: Some(0),
			},
			// Final events
			Event::RewardedCollator {
				collator: ACCOUNT_CANDIDATE_1.with_gen(0),
				auto_compounding_rewards: rewards_collator_ac,
				manual_claim_rewards: rewards_collator_mc,
			},
			Event::RewardedDelegators {
				collator: ACCOUNT_CANDIDATE_1.with_gen(0),
				auto_compounding_rewards: rewards_delegator_ac,
				manual_claim_rewards: rewards_delegator_mc,
			},
		]);
	});
}

// To ensure the computations don't have issues we'll test with only someone delegating to
// a collator, but the collator not staking. It means that the collator will not have enough
// self-delegation to be in the ellected set. However since in tests we trigger the reward manually
// it will still distribute the rewards.
#[test]
fn mixed_delegator_only() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(LiquidStaking::stake_manual_claim(
			Origin::signed(ACCOUNT_DELEGATOR_1),
			ACCOUNT_CANDIDATE_1.with_gen(0),
			SharesOrStake::Shares(10_000)
		));
		assert_ok!(LiquidStaking::stake_auto_compounding(
			Origin::signed(ACCOUNT_DELEGATOR_1),
			ACCOUNT_CANDIDATE_1.with_gen(0),
			SharesOrStake::Shares(30_000)
		));

		assert_eq!(balance(&ACCOUNT_CANDIDATE_1), 1 * PETA);
		assert_eq!(balance(&ACCOUNT_DELEGATOR_1), 1 * PETA - 40 * MEGA);
		assert_eq!(balance(&ACCOUNT_STAKING), 40 * MEGA);

		let rewards = 1 * MEGA;
		let rewards_reserve = rewards * 3 / 10;

		let shared_rewards = rewards - rewards_reserve;
		let rewards_delegator = rewards * 5 / 10; // 50%

		// Distributing MC rewards have rounding.
		let rewards_delegator_mc = round_down(rewards_delegator * 1 / 4, 10_000); // 25% MC
		let rewards_delegator_ac = rewards_delegator - rewards_delegator_mc;

		assert_ok!(crate::rewards::distribute_rewards::<Runtime>(
			ACCOUNT_CANDIDATE_1.with_gen(0),
			rewards
		));

		// Distributing delegators AC rewards change the value of an AC share.
		// Collator AC rewards are distributed after delegator AC rewards to not give the collator
		// more shares which would give them a bigger part.

		// Distributing AC rewards have rounding.
		// Here collator exceptionaly don't have shares, so all the rewards go throught the manual
		// claim path and thus be transfered.
		let rewards_collator = shared_rewards - rewards_delegator; // 20%

		assert_eq!(balance(&ACCOUNT_RESERVE), rewards * 3 / 10);
		assert_eq!(balance(&ACCOUNT_CANDIDATE_1), 1 * PETA + rewards_collator);
		assert_eq!(balance(&ACCOUNT_DELEGATOR_1), 1 * PETA - 40 * MEGA);
		assert_eq!(balance(&ACCOUNT_STAKING), 40 * MEGA + rewards_delegator);

		assert_eq_events!(vec![
			Event::StakedManualClaim {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				delegator: ACCOUNT_DELEGATOR_1,
				shares: 10_000,
				stake: 10 * MEGA,
			},
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				stake: 10 * MEGA,
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				stake: 10 * MEGA,
				self_delegation: 0,
				before: None,
				after: None,
			},
			// ------
			Event::StakedAutoCompounding {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				delegator: ACCOUNT_DELEGATOR_1,
				shares: 30_000,
				stake: 30 * MEGA,
			},
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				stake: 30 * MEGA,
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				stake: 40 * MEGA,
				self_delegation: 0,
				before: None,
				after: None,
			},
			// ------
			// Update total stake following AC reward distribution.
			Event::IncreasedStake {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				stake: rewards_delegator_ac,
			},
			Event::UpdatedCandidatePosition {
				candidate: ACCOUNT_CANDIDATE_1.with_gen(0),
				stake: 40 * MEGA + rewards_delegator_ac,
				self_delegation: 0,
				before: None,
				after: None,
			},
			// Final events
			Event::RewardedCollator {
				collator: ACCOUNT_CANDIDATE_1.with_gen(0),
				auto_compounding_rewards: 0,
				manual_claim_rewards: rewards_collator,
			},
			Event::RewardedDelegators {
				collator: ACCOUNT_CANDIDATE_1.with_gen(0),
				auto_compounding_rewards: rewards_delegator_ac,
				manual_claim_rewards: rewards_delegator_mc,
			},
		]);
	});
}
