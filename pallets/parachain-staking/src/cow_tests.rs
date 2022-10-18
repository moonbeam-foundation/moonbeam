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

//! # Staking Pallet Unit Tests
//! The unit tests are organized by the call they test. The order matches the order
//! of the calls in the `lib.rs`.
//! 1. Root
//! 2. Monetary Governance
//! 3. Public (Collator, Nominator)
//! 4. Miscellaneous Property-Based Tests
use crate::delegation_requests::{CancelledScheduledRequest, DelegationAction, ScheduledRequest};
use crate::mock::{
	roll_one_block, roll_to, roll_to_round_begin, roll_to_round_end, set_author, Balances,
	BlockNumber, Event as MetaEvent, ExtBuilder, Origin, ParachainStaking, Test,
};
use crate::{
	assert_eq_events, assert_eq_last_events, assert_event_emitted, assert_last_event,
	assert_tail_eq, set::OrderedSet, AtStake, Bond, BottomDelegations, CandidateInfo,
	CandidateMetadata, CandidatePool, CapacityStatus, CollatorStatus, DelegationScheduledRequests,
	Delegations, DelegatorAdded, DelegatorState, DelegatorStatus, Error, Event, Range,
	TopDelegations, DELEGATOR_LOCK_ID,
};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{traits::Zero, DispatchError, ModuleError, Perbill, Percent};

#[test]
fn test_cow_no_changes_works() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20)])
		.with_candidates(vec![(1, 20)])
		.with_delegations(vec![(2, 1, 20)])
		.with_collator_commission(Some(Perbill::zero()))
		.build()
		.execute_with(|| {
			// Make no changes and ensure that rewards are properly paid for several blocks.
			set_author(1, 1, 100);
			roll_to_round_end(2);
			assert_eq_last_events!(
				vec![
					Event::<Test>::CollatorChosen {
						round: 2,
						collator_account: 1,
						total_exposed_amount: 40,
					},
					Event::<Test>::NewRound {
						starting_block: 5,
						round: 2,
						selected_collators_number: 1,
						total_balance: 40,
					},
				],
				"Collator selection and/or round start did not occur properly"
			);

			set_author(2, 1, 100);
			roll_to_round_end(3);
			assert_eq_last_events!(
				vec![
					Event::<Test>::CollatorChosen {
						round: 3,
						collator_account: 1,
						total_exposed_amount: 40,
					},
					Event::<Test>::NewRound {
						starting_block: 10,
						round: 3,
						selected_collators_number: 1,
						total_balance: 40,
					},
					Event::<Test>::Rewarded {
						account: 1,
						rewards: 1,
					},
					Event::<Test>::Rewarded {
						account: 2,
						rewards: 1,
					},
				],
				"Collator selection and/or round start and/or rewards did not occur properly"
			);

			// repeat for following rounds...
			for round in 3..5 {

				set_author(round, 1, 100);
				roll_to_round_end(round + 1);
				assert_eq_last_events!(
					vec![
						Event::<Test>::CollatorChosen {
							round: round + 1,
							collator_account: 1,
							total_exposed_amount: 40,
						},
						Event::<Test>::NewRound {
							starting_block: round * 5,
							round: round + 1,
							selected_collators_number: 1,
							total_balance: 40,
						},
						Event::<Test>::Rewarded {
							account: 1,
							rewards: 1,
						},
						Event::<Test>::Rewarded {
							account: 2,
							rewards: 1,
						},
					],
					"Collator selection and/or round start and/or rewards did not occur properly"
				);
			}
		});
}

#[test]
fn test_cow_after_bond_less_and_execute() {
	ExtBuilder::default()
		.with_balances(vec![(1, 2000), (2, 2000)])
		.with_candidates(vec![(1, 2000)])
		.with_delegations(vec![(2, 1, 2000)])
		.with_collator_commission(Some(Perbill::zero()))
		.build()
		.execute_with(|| {
			// delegator 2 immediately requests bond less, this is executed in round 1
			assert_eq!(1, ParachainStaking::round().current);
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				1000
			));

			// roll through round 2 and look at round change events, they should not be affected by
			// the request
			set_author(1, 1, 100);
			roll_to_round_end(2);
			assert_eq_last_events!(
				vec![
					Event::<Test>::DelegationDecreaseScheduled {
						delegator: 2,
						candidate: 1,
						amount_to_decrease: 1000,
						execute_round: 3,
					},

					// bond still counts towards totals
					Event::<Test>::CollatorChosen {
						round: 2,
						collator_account: 1,
						total_exposed_amount: 4000,
					},
					Event::<Test>::NewRound {
						starting_block: 5,
						round: 2,
						selected_collators_number: 1,
						total_balance: 4000,
					},
				],
				"Round change events incorrect"
			);

			// roll through round 3 and look at payouts
			set_author(2, 1, 100);
			roll_to_round_end(3);
			assert_eq_last_events!(
				vec![
					// bond less still should not impact totals (it won't until executed)
					Event::<Test>::CollatorChosen {
						round: 3,
						collator_account: 1,
						total_exposed_amount: 4000,
					},
					Event::<Test>::NewRound {
						starting_block: 10,
						round: 3,
						selected_collators_number: 1,
						total_balance: 4000,
					},

					// round 1 payouts should also be unaffected by bond less
					// (this is because the bond change request occurred in that round)
					Event::<Test>::Rewarded {
						account: 1,
						rewards: 100,
					},
					Event::<Test>::Rewarded {
						account: 2,
						rewards: 100,
					},
				],
				"Collator selection and/or round start did not occur properly"
			);

			// roll through round 4 and look for payouts of round 2, which should include the
			// effects of the bond request for payout but not for total collator backing
			set_author(3, 1, 100);
			roll_to_round_end(4);
			assert_eq_last_events!(
				vec![
					// bond less still should not impact totals (it never will)
					Event::<Test>::CollatorChosen {
						round: 4,
						collator_account: 1,
						total_exposed_amount: 4000,
					},
					Event::<Test>::NewRound {
						starting_block: 15,
						round: 4,
						selected_collators_number: 1,
						total_balance: 4000,
					},

					// round 2 payouts should be affected by bond less request
					Event::<Test>::Rewarded {
						account: 1,
						rewards: 140,
					},
					Event::<Test>::Rewarded {
						account: 2,
						rewards: 70,
					},
				],
				"Collator selection and/or round start did not occur properly"
			);

			// now execute the bond request, it should change:
			// payouts in the upcoming round (paid several blocks later)
			// total collator backing, observed in the round change following execution
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));

			set_author(4, 1, 100);
			roll_to_round_end(5);
			assert_eq_last_events!(
				vec![
					// the executed bond less should take effect now
					Event::<Test>::CollatorChosen {
						round: 5,
						collator_account: 1,
						total_exposed_amount: 3000,
					},
					Event::<Test>::NewRound {
						starting_block: 20,
						round: 5,
						selected_collators_number: 1,
						total_balance: 3000,
					},

					// round 3 payouts should be affected by bond less execute
					Event::<Test>::Rewarded { account: 1, rewards: 147, },
					Event::<Test>::Rewarded { account: 2, rewards: 73, },
				],
				"Collator selection and/or round start did not occur properly"
			);

			// round 6 and 7 should look similar: delegations unchanged from execute
			set_author(5, 1, 100);
			set_author(6, 1, 100);
			roll_to_round_end(7);
			assert_eq_last_events!(
				vec![
					Event::<Test>::CollatorChosen {
						round: 6,
						collator_account: 1,
						total_exposed_amount: 3000,
					},
					Event::<Test>::NewRound {
						starting_block: 25,
						round: 6,
						selected_collators_number: 1,
						total_balance: 3000,
					},

					Event::<Test>::Rewarded { account: 1, rewards: 154, },
					Event::<Test>::Rewarded { account: 2, rewards: 77, },

					Event::<Test>::CollatorChosen {
						round: 7,
						collator_account: 1,
						total_exposed_amount: 3000,
					},
					Event::<Test>::NewRound {
						starting_block: 30,
						round: 7,
						selected_collators_number: 1,
						total_balance: 3000,
					},

					Event::<Test>::Rewarded { account: 1, rewards: 162, },
					Event::<Test>::Rewarded { account: 2, rewards: 81, },
				],
				"Collator selection and/or round start did not occur properly"
			);
		});
}

#[test]
fn test_cow_after_bond_less_and_cancel() {
	ExtBuilder::default()
		.with_balances(vec![(1, 2000), (2, 2000)])
		.with_candidates(vec![(1, 2000)])
		.with_delegations(vec![(2, 1, 2000)])
		.with_collator_commission(Some(Perbill::zero()))
		.build()
		.execute_with(|| {
			// delegator 2 immediately requests bond less, this is executed in round 1
			assert_eq!(1, ParachainStaking::round().current);
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(Origin::signed(2), 1, 1000));

			// in round 2, delegator cancels
			set_author(1, 1, 100);
			roll_to_round_begin(2);
			assert_ok!(ParachainStaking::cancel_delegation_request(Origin::signed(2), 1));

			// roll to round 3, we should get round 1 payouts now, which should include the
			// candidate's full bond
			set_author(2, 1, 100);
			roll_to_round_end(3);
			assert_eq_last_events!(
				vec![
					// the delegation cancel from round 2 comes before round change events
					Event::<Test>::CancelledDelegationRequest {
						delegator: 2,
						collator: 1,
						cancelled_request: CancelledScheduledRequest {
							when_executable: 3,
							action: DelegationAction::Decrease(1000),
						}
					},
					Event::<Test>::CollatorChosen {
						round: 3,
						collator_account: 1,
						total_exposed_amount: 4000,
					},
					Event::<Test>::NewRound {
						starting_block: 10,
						round: 3,
						selected_collators_number: 1,
						total_balance: 4000,
					},

					Event::<Test>::Rewarded { account: 1, rewards: 100 },
					Event::<Test>::Rewarded { account: 2, rewards: 100 },
				],
				"Collator selection and/or round start did not occur properly"
			);

			// round 4 should include reduced payouts for delegator @ round 2
			set_author(3, 1, 100);
			roll_to_round_end(4);
			assert_eq_last_events!(
				vec![
					Event::<Test>::CollatorChosen {
						round: 4,
						collator_account: 1,
						total_exposed_amount: 4000,
					},
					Event::<Test>::NewRound {
						starting_block: 15,
						round: 4,
						selected_collators_number: 1,
						total_balance: 4000,
					},

					Event::<Test>::Rewarded { account: 1, rewards: 140 },
					Event::<Test>::Rewarded { account: 2, rewards: 70 },
				],
				"Collator selection and/or round start did not occur properly"
			);

			// round 5 should include payouts for round 3 which respect the cancel (back to larger
			// rewards)
			set_author(4, 1, 100);
			roll_to_round_end(5);
			assert_eq_last_events!(
				vec![
					Event::<Test>::CollatorChosen {
						round: 5,
						collator_account: 1,
						total_exposed_amount: 4000,
					},
					Event::<Test>::NewRound {
						starting_block: 20,
						round: 5,
						selected_collators_number: 1,
						total_balance: 4000,
					},

					Event::<Test>::Rewarded { account: 1, rewards: 110 },
					Event::<Test>::Rewarded { account: 2, rewards: 110 },
				],
				"Collator selection and/or round start did not occur properly"
			);
		});
}
