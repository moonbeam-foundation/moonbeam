// Copyright 2019-2021 PureStake Inc.
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
use crate::mock::{
	roll_one_block, roll_to, roll_to_round_begin, roll_to_round_end, set_author, Balances,
	Event as MetaEvent, ExtBuilder, Origin, Stake, Test,
};
use crate::{
	assert_eq_events, assert_eq_last_events, assert_event_emitted, assert_last_event,
	assert_tail_eq, Bond, CandidateState, CollatorStatus, DelegationChange, DelegationRequest,
	DelegatorAdded, Error, Event, Range,
};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{traits::Zero, DispatchError, Perbill, Percent};

// ~~ ROOT ~~

#[test]
fn invalid_root_origin_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::set_total_selected(Origin::signed(45), 6u32),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_noop!(
			Stake::set_collator_commission(Origin::signed(45), Perbill::from_percent(5)),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_noop!(
			Stake::set_blocks_per_round(Origin::signed(45), 3u32),
			sp_runtime::DispatchError::BadOrigin
		);
	});
}

// SET TOTAL SELECTED

#[test]
fn set_total_selected_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		// before we can bump total_selected we must bump the blocks per round
		assert_ok!(Stake::set_blocks_per_round(Origin::root(), 6u32));
		assert_ok!(Stake::set_total_selected(Origin::root(), 6u32));
		assert_last_event!(MetaEvent::Stake(Event::TotalSelectedSet(5u32, 6u32)));
	});
}

#[test]
fn set_total_selected_fails_if_above_blocks_per_round() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Stake::round().length, 5); // test relies on this
		assert_noop!(
			Stake::set_total_selected(Origin::root(), 6u32),
			Error::<Test>::RoundLengthMustBeAtLeastTotalSelectedCollators,
		);
	});
}

#[test]
fn set_total_selected_passes_if_equal_to_blocks_per_round() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Stake::set_blocks_per_round(Origin::root(), 10u32));
		assert_ok!(Stake::set_total_selected(Origin::root(), 10u32));
	});
}

#[test]
fn set_total_selected_passes_if_below_blocks_per_round() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Stake::set_blocks_per_round(Origin::root(), 10u32));
		assert_ok!(Stake::set_total_selected(Origin::root(), 9u32));
	});
}

#[test]
fn set_blocks_per_round_fails_if_below_total_selected() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Stake::set_blocks_per_round(Origin::root(), 20u32));
		assert_ok!(Stake::set_total_selected(Origin::root(), 15u32));
		assert_noop!(
			Stake::set_blocks_per_round(Origin::root(), 14u32),
			Error::<Test>::RoundLengthMustBeAtLeastTotalSelectedCollators,
		);
	});
}

#[test]
fn set_blocks_per_round_passes_if_equal_to_total_selected() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Stake::set_blocks_per_round(Origin::root(), 10u32));
		assert_ok!(Stake::set_total_selected(Origin::root(), 9u32));
		assert_ok!(Stake::set_blocks_per_round(Origin::root(), 9u32));
	});
}

#[test]
fn set_blocks_per_round_passes_if_above_total_selected() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Stake::round().length, 5); // test relies on this
		assert_ok!(Stake::set_blocks_per_round(Origin::root(), 6u32));
	});
}

#[test]
fn set_total_selected_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		// round length must be >= total_selected, so update that first
		assert_ok!(Stake::set_blocks_per_round(Origin::root(), 10u32));

		assert_eq!(Stake::total_selected(), 5u32);
		assert_ok!(Stake::set_total_selected(Origin::root(), 6u32));
		assert_eq!(Stake::total_selected(), 6u32);
	});
}

#[test]
fn cannot_set_total_selected_to_current_total_selected() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::set_total_selected(Origin::root(), 5u32),
			Error::<Test>::NoWritingSameValue
		);
	});
}

#[test]
fn cannot_set_total_selected_below_module_min() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::set_total_selected(Origin::root(), 4u32),
			Error::<Test>::CannotSetBelowMin
		);
	});
}

// SET COLLATOR COMMISSION

#[test]
fn set_collator_commission_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Stake::set_collator_commission(
			Origin::root(),
			Perbill::from_percent(5)
		));
		assert_last_event!(MetaEvent::Stake(Event::CollatorCommissionSet(
			Perbill::from_percent(20),
			Perbill::from_percent(5),
		)));
	});
}

#[test]
fn set_collator_commission_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Stake::collator_commission(), Perbill::from_percent(20));
		assert_ok!(Stake::set_collator_commission(
			Origin::root(),
			Perbill::from_percent(5)
		));
		assert_eq!(Stake::collator_commission(), Perbill::from_percent(5));
	});
}

#[test]
fn cannot_set_collator_commission_to_current_collator_commission() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::set_collator_commission(Origin::root(), Perbill::from_percent(20)),
			Error::<Test>::NoWritingSameValue
		);
	});
}

// SET BLOCKS PER ROUND

#[test]
fn set_blocks_per_round_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Stake::set_blocks_per_round(Origin::root(), 6u32));
		assert_last_event!(MetaEvent::Stake(Event::BlocksPerRoundSet(
			1,
			0,
			5,
			6,
			Perbill::from_parts(926),
			Perbill::from_parts(926),
			Perbill::from_parts(926),
		)));
	});
}

#[test]
fn set_blocks_per_round_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Stake::round().length, 5);
		assert_ok!(Stake::set_blocks_per_round(Origin::root(), 6u32));
		assert_eq!(Stake::round().length, 6);
	});
}

#[test]
fn cannot_set_blocks_per_round_below_module_min() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::set_blocks_per_round(Origin::root(), 2u32),
			Error::<Test>::CannotSetBelowMin
		);
	});
}

#[test]
fn cannot_set_blocks_per_round_to_current_blocks_per_round() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::set_blocks_per_round(Origin::root(), 5u32),
			Error::<Test>::NoWritingSameValue
		);
	});
}

#[test]
fn round_immediately_jumps_if_current_duration_exceeds_new_blocks_per_round() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			// we can't lower the blocks per round because it must be above the number of collators,
			// and we can't lower the number of collators because it must be above
			// MinSelectedCandidates. so we first raise blocks per round, then lower it.
			assert_ok!(Stake::set_blocks_per_round(Origin::root(), 10u32));

			roll_to(17);
			assert_last_event!(MetaEvent::Stake(Event::NewRound(10, 2, 1, 20)));
			assert_ok!(Stake::set_blocks_per_round(Origin::root(), 5u32));
			roll_to(18);
			assert_last_event!(MetaEvent::Stake(Event::NewRound(18, 3, 1, 20)));
		});
}

// ~~ MONETARY GOVERNANCE ~~

#[test]
fn invalid_monetary_origin_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::set_staking_expectations(
				Origin::signed(45),
				Range {
					min: 3u32.into(),
					ideal: 4u32.into(),
					max: 5u32.into()
				}
			),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_noop!(
			Stake::set_inflation(
				Origin::signed(45),
				Range {
					min: Perbill::from_percent(3),
					ideal: Perbill::from_percent(4),
					max: Perbill::from_percent(5)
				}
			),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_noop!(
			Stake::set_inflation(
				Origin::signed(45),
				Range {
					min: Perbill::from_percent(3),
					ideal: Perbill::from_percent(4),
					max: Perbill::from_percent(5)
				}
			),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_noop!(
			Stake::set_parachain_bond_account(Origin::signed(45), 11),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_noop!(
			Stake::set_parachain_bond_reserve_percent(Origin::signed(45), Percent::from_percent(2)),
			sp_runtime::DispatchError::BadOrigin
		);
	});
}

// SET STAKING EXPECTATIONS

#[test]
fn set_staking_event_emits_event_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		// valid call succeeds
		assert_ok!(Stake::set_staking_expectations(
			Origin::root(),
			Range {
				min: 3u128,
				ideal: 4u128,
				max: 5u128,
			}
		));
		assert_last_event!(MetaEvent::Stake(Event::StakeExpectationsSet(
			3u128, 4u128, 5u128,
		)));
	});
}

#[test]
fn set_staking_updates_storage_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			Stake::inflation_config().expect,
			Range {
				min: 700,
				ideal: 700,
				max: 700
			}
		);
		assert_ok!(Stake::set_staking_expectations(
			Origin::root(),
			Range {
				min: 3u128,
				ideal: 4u128,
				max: 5u128,
			}
		));
		assert_eq!(
			Stake::inflation_config().expect,
			Range {
				min: 3u128,
				ideal: 4u128,
				max: 5u128
			}
		);
	});
}

#[test]
fn cannot_set_invalid_staking_expectations() {
	ExtBuilder::default().build().execute_with(|| {
		// invalid call fails
		assert_noop!(
			Stake::set_staking_expectations(
				Origin::root(),
				Range {
					min: 5u128,
					ideal: 4u128,
					max: 3u128
				}
			),
			Error::<Test>::InvalidSchedule
		);
	});
}

#[test]
fn cannot_set_same_staking_expectations() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Stake::set_staking_expectations(
			Origin::root(),
			Range {
				min: 3u128,
				ideal: 4u128,
				max: 5u128
			}
		));
		assert_noop!(
			Stake::set_staking_expectations(
				Origin::root(),
				Range {
					min: 3u128,
					ideal: 4u128,
					max: 5u128
				}
			),
			Error::<Test>::NoWritingSameValue
		);
	});
}

// SET INFLATION

#[test]
fn set_inflation_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		let (min, ideal, max): (Perbill, Perbill, Perbill) = (
			Perbill::from_percent(3),
			Perbill::from_percent(4),
			Perbill::from_percent(5),
		);
		assert_ok!(Stake::set_inflation(
			Origin::root(),
			Range { min, ideal, max }
		));
		assert_last_event!(MetaEvent::Stake(Event::InflationSet(
			min,
			ideal,
			max,
			Perbill::from_parts(57),
			Perbill::from_parts(75),
			Perbill::from_parts(93),
		)));
	});
}

#[test]
fn set_inflation_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		let (min, ideal, max): (Perbill, Perbill, Perbill) = (
			Perbill::from_percent(3),
			Perbill::from_percent(4),
			Perbill::from_percent(5),
		);
		assert_eq!(
			Stake::inflation_config().annual,
			Range {
				min: Perbill::from_percent(50),
				ideal: Perbill::from_percent(50),
				max: Perbill::from_percent(50)
			}
		);
		assert_eq!(
			Stake::inflation_config().round,
			Range {
				min: Perbill::from_percent(5),
				ideal: Perbill::from_percent(5),
				max: Perbill::from_percent(5)
			}
		);
		assert_ok!(Stake::set_inflation(
			Origin::root(),
			Range { min, ideal, max }
		),);
		assert_eq!(Stake::inflation_config().annual, Range { min, ideal, max });
		assert_eq!(
			Stake::inflation_config().round,
			Range {
				min: Perbill::from_parts(57),
				ideal: Perbill::from_parts(75),
				max: Perbill::from_parts(93)
			}
		);
	});
}

#[test]
fn cannot_set_invalid_inflation() {
	ExtBuilder::default().build().execute_with(|| {
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
	});
}

#[test]
fn cannot_set_same_inflation() {
	ExtBuilder::default().build().execute_with(|| {
		let (min, ideal, max): (Perbill, Perbill, Perbill) = (
			Perbill::from_percent(3),
			Perbill::from_percent(4),
			Perbill::from_percent(5),
		);
		assert_ok!(Stake::set_inflation(
			Origin::root(),
			Range { min, ideal, max }
		),);
		assert_noop!(
			Stake::set_inflation(Origin::root(), Range { min, ideal, max }),
			Error::<Test>::NoWritingSameValue
		);
	});
}

// SET PARACHAIN BOND ACCOUNT

#[test]
fn set_parachain_bond_account_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Stake::set_parachain_bond_account(Origin::root(), 11));
		assert_last_event!(MetaEvent::Stake(Event::ParachainBondAccountSet(0, 11)));
	});
}

#[test]
fn set_parachain_bond_account_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Stake::parachain_bond_info().account, 0);
		assert_ok!(Stake::set_parachain_bond_account(Origin::root(), 11));
		assert_eq!(Stake::parachain_bond_info().account, 11);
	});
}

// SET PARACHAIN BOND RESERVE PERCENT

#[test]
fn set_parachain_bond_reserve_percent_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Stake::set_parachain_bond_reserve_percent(
			Origin::root(),
			Percent::from_percent(50)
		));
		assert_last_event!(MetaEvent::Stake(Event::ParachainBondReservePercentSet(
			Percent::from_percent(30),
			Percent::from_percent(50),
		)));
	});
}

#[test]
fn set_parachain_bond_reserve_percent_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			Stake::parachain_bond_info().percent,
			Percent::from_percent(30)
		);
		assert_ok!(Stake::set_parachain_bond_reserve_percent(
			Origin::root(),
			Percent::from_percent(50)
		));
		assert_eq!(
			Stake::parachain_bond_info().percent,
			Percent::from_percent(50)
		);
	});
}

#[test]
fn cannot_set_same_parachain_bond_reserve_percent() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::set_parachain_bond_reserve_percent(Origin::root(), Percent::from_percent(30)),
			Error::<Test>::NoWritingSameValue
		);
	});
}

// ~~ PUBLIC ~~

// JOIN CANDIDATES

#[test]
fn join_candidates_event_emits_correctly() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::join_candidates(Origin::signed(1), 10u128, 0u32));
			assert_last_event!(MetaEvent::Stake(Event::JoinedCollatorCandidates(
				1, 10u128, 10u128,
			)));
		});
}

#[test]
fn join_candidates_reserves_balance() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::reserved_balance(&1), 0);
			assert_eq!(Balances::free_balance(&1), 10);
			assert_ok!(Stake::join_candidates(Origin::signed(1), 10u128, 0u32));
			assert_eq!(Balances::reserved_balance(&1), 10);
			assert_eq!(Balances::free_balance(&1), 0);
		});
}

#[test]
fn join_candidates_increases_total_staked() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::total(), 0);
			assert_ok!(Stake::join_candidates(Origin::signed(1), 10u128, 0u32));
			assert_eq!(Stake::total(), 10);
		});
}

#[test]
fn join_candidates_creates_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert!(Stake::candidate_state(1).is_none());
			assert_ok!(Stake::join_candidates(Origin::signed(1), 10u128, 0u32));
			let candidate_state = Stake::candidate_state(1).expect("just joined => exists");
			assert_eq!(candidate_state.bond, 10u128);
		});
}

#[test]
fn join_candidates_adds_to_candidate_pool() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert!(Stake::candidate_pool().0.is_empty());
			assert_ok!(Stake::join_candidates(Origin::signed(1), 10u128, 0u32));
			let candidate_pool = Stake::candidate_pool();
			assert_eq!(
				candidate_pool.0[0],
				Bond {
					owner: 1,
					amount: 10u128
				}
			);
		});
}

#[test]
fn cannot_join_candidates_if_candidate() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.with_candidates(vec![(1, 500)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::join_candidates(Origin::signed(1), 11u128, 100u32),
				Error::<Test>::CandidateExists
			);
		});
}

#[test]
fn cannot_join_candidates_if_delegator() {
	ExtBuilder::default()
		.with_balances(vec![(1, 50), (2, 20)])
		.with_candidates(vec![(1, 50)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::join_candidates(Origin::signed(2), 10u128, 1u32),
				Error::<Test>::DelegatorExists
			);
		});
}

#[test]
fn cannot_join_candidates_without_min_bond() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::join_candidates(Origin::signed(1), 9u128, 100u32),
				Error::<Test>::CandidateBondBelowMin
			);
		});
}

#[test]
fn cannot_join_candidates_with_more_than_available_balance() {
	ExtBuilder::default()
		.with_balances(vec![(1, 500)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::join_candidates(Origin::signed(1), 501u128, 100u32),
				DispatchError::Module {
					index: 1,
					error: 2,
					message: Some("InsufficientBalance")
				}
			);
		});
}

#[test]
fn insufficient_join_candidates_weight_hint_fails() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20), (6, 20)])
		.with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.build()
		.execute_with(|| {
			for i in 0..5 {
				assert_noop!(
					Stake::join_candidates(Origin::signed(6), 20, i),
					Error::<Test>::TooLowCandidateCountWeightHintJoinCandidates
				);
			}
		});
}

#[test]
fn sufficient_join_candidates_weight_hint_succeeds() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 20),
			(2, 20),
			(3, 20),
			(4, 20),
			(5, 20),
			(6, 20),
			(7, 20),
			(8, 20),
			(9, 20),
		])
		.with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.build()
		.execute_with(|| {
			let mut count = 5u32;
			for i in 6..10 {
				assert_ok!(Stake::join_candidates(Origin::signed(i), 20, count));
				count += 1u32;
			}
		});
}

// SCHEDULE LEAVE CANDIDATES

#[test]
fn leave_candidates_event_emits_correctly() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			assert_last_event!(MetaEvent::Stake(Event::CandidateScheduledExit(1, 1, 3)));
		});
}

#[test]
fn leave_candidates_removes_candidate_from_candidate_pool() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::candidate_pool().0.len(), 1);
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			assert!(Stake::candidate_pool().0.is_empty());
		});
}

#[test]
fn cannot_leave_candidates_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::schedule_leave_candidates(Origin::signed(1), 1u32),
			Error::<Test>::CandidateDNE
		);
	});
}

#[test]
fn cannot_leave_candidates_if_already_leaving_candidates() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			assert_noop!(
				Stake::schedule_leave_candidates(Origin::signed(1), 1u32),
				Error::<Test>::CandidateAlreadyLeaving
			);
		});
}

#[test]
fn insufficient_leave_candidates_weight_hint_fails() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.build()
		.execute_with(|| {
			for i in 1..6 {
				assert_noop!(
					Stake::schedule_leave_candidates(Origin::signed(i), 4u32),
					Error::<Test>::TooLowCandidateCountToLeaveCandidates
				);
			}
		});
}

#[test]
fn sufficient_leave_candidates_weight_hint_succeeds() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.build()
		.execute_with(|| {
			let mut count = 5u32;
			for i in 1..6 {
				assert_ok!(Stake::schedule_leave_candidates(Origin::signed(i), count));
				count -= 1u32;
			}
		});
}

// EXECUTE LEAVE CANDIDATES

#[test]
fn execute_leave_candidates_emits_event() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			roll_to(10);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(1), 1));
			assert_last_event!(MetaEvent::Stake(Event::CandidateLeft(1, 10, 0)));
		});
}

#[test]
fn execute_leave_candidates_callable_by_any_signed() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			roll_to(10);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(2), 1));
		});
}

#[test]
fn execute_leave_candidates_unreserves_balance() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::reserved_balance(&1), 10);
			assert_eq!(Balances::free_balance(&1), 0);
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			roll_to(10);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(1), 1));
			assert_eq!(Balances::reserved_balance(&1), 0);
			assert_eq!(Balances::free_balance(&1), 10);
		});
}

#[test]
fn execute_leave_candidates_decreases_total_staked() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::total(), 10);
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			roll_to(10);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(1), 1));
			assert_eq!(Stake::total(), 0);
		});
}

#[test]
fn execute_leave_candidates_removes_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			// candidate state is not immediately removed
			let candidate_state = Stake::candidate_state(1).expect("just left => still exists");
			assert_eq!(candidate_state.bond, 10u128);
			roll_to(10);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(1), 1));
			assert!(Stake::candidate_state(1).is_none());
		});
}

#[test]
fn cannot_execute_leave_candidates_before_delay() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			assert_noop!(
				Stake::execute_leave_candidates(Origin::signed(3), 1),
				Error::<Test>::CandidateCannotLeaveYet
			);
			roll_to(9);
			assert_noop!(
				Stake::execute_leave_candidates(Origin::signed(3), 1),
				Error::<Test>::CandidateCannotLeaveYet
			);
			roll_to(10);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(3), 1));
		});
}

// CANCEL LEAVE CANDIDATES

#[test]
fn cancel_leave_candidates_emits_event() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			assert_ok!(Stake::cancel_leave_candidates(Origin::signed(1), 1));
			assert_last_event!(MetaEvent::Stake(Event::CancelledCandidateExit(1)));
		});
}

#[test]
fn cancel_leave_candidates_updates_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			assert_ok!(Stake::cancel_leave_candidates(Origin::signed(1), 1));
			let candidate = Stake::candidate_state(&1).expect("just cancelled leave so exists");
			assert!(candidate.is_active());
		});
}

#[test]
fn cancel_leave_candidates_adds_to_candidate_pool() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1u32));
			assert_ok!(Stake::cancel_leave_candidates(Origin::signed(1), 1));
			assert_eq!(
				Stake::candidate_pool().0[0],
				Bond {
					owner: 1,
					amount: 10
				}
			);
		});
}

// GO OFFLINE

#[test]
fn go_offline_event_emits_correctly() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::go_offline(Origin::signed(1)));
			assert_last_event!(MetaEvent::Stake(Event::CandidateWentOffline(1, 1)));
		});
}

#[test]
fn go_offline_removes_candidate_from_candidate_pool() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::candidate_pool().0.len(), 1);
			assert_ok!(Stake::go_offline(Origin::signed(1)));
			assert!(Stake::candidate_pool().0.is_empty());
		});
}

#[test]
fn go_offline_updates_candidate_state_to_idle() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			let candidate_state = Stake::candidate_state(1).expect("is active candidate");
			assert_eq!(candidate_state.state, CollatorStatus::Active);
			assert_ok!(Stake::go_offline(Origin::signed(1)));
			let candidate_state = Stake::candidate_state(1).expect("is candidate, just offline");
			assert_eq!(candidate_state.state, CollatorStatus::Idle);
		});
}

#[test]
fn cannot_go_offline_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::go_offline(Origin::signed(3)),
			Error::<Test>::CandidateDNE
		);
	});
}

#[test]
fn cannot_go_offline_if_already_offline() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::go_offline(Origin::signed(1)));
			assert_noop!(
				Stake::go_offline(Origin::signed(1)),
				Error::<Test>::AlreadyOffline
			);
		});
}

// GO ONLINE

#[test]
fn go_online_event_emits_correctly() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::go_offline(Origin::signed(1)));
			assert_ok!(Stake::go_online(Origin::signed(1)));
			assert_last_event!(MetaEvent::Stake(Event::CandidateBackOnline(1, 1)));
		});
}

#[test]
fn go_online_adds_to_candidate_pool() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::go_offline(Origin::signed(1)));
			assert!(Stake::candidate_pool().0.is_empty());
			assert_ok!(Stake::go_online(Origin::signed(1)));
			assert_eq!(
				Stake::candidate_pool().0[0],
				Bond {
					owner: 1,
					amount: 20
				}
			);
		});
}

#[test]
fn go_online_storage_updates_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::go_offline(Origin::signed(1)));
			let candidate_state = Stake::candidate_state(1).expect("offline still exists");
			assert_eq!(candidate_state.state, CollatorStatus::Idle);
			assert_ok!(Stake::go_online(Origin::signed(1)));
			let candidate_state = Stake::candidate_state(1).expect("online so exists");
			assert_eq!(candidate_state.state, CollatorStatus::Active);
		});
}

#[test]
fn cannot_go_online_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::go_online(Origin::signed(3)),
			Error::<Test>::CandidateDNE
		);
	});
}

#[test]
fn cannot_go_online_if_already_online() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::go_online(Origin::signed(1)),
				Error::<Test>::AlreadyActive
			);
		});
}

#[test]
fn cannot_go_online_if_leaving() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1));
			assert_noop!(
				Stake::go_online(Origin::signed(1)),
				Error::<Test>::CannotGoOnlineIfLeaving
			);
		});
}

// CANDIDATE BOND MORE

#[test]
fn candidate_bond_more_emits_correct_event() {
	ExtBuilder::default()
		.with_balances(vec![(1, 50)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::candidate_bond_more(Origin::signed(1), 30));
			assert_last_event!(MetaEvent::Stake(Event::CandidateBondedMore(1, 30, 50)));
		});
}

#[test]
fn candidate_bond_more_reserves_balance() {
	ExtBuilder::default()
		.with_balances(vec![(1, 50)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::reserved_balance(&1), 20);
			assert_eq!(Balances::free_balance(&1), 30);
			assert_ok!(Stake::candidate_bond_more(Origin::signed(1), 30));
			assert_eq!(Balances::reserved_balance(&1), 50);
			assert_eq!(Balances::free_balance(&1), 0);
		});
}

#[test]
fn candidate_bond_more_increases_total() {
	ExtBuilder::default()
		.with_balances(vec![(1, 50)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			let mut total = Stake::total();
			assert_ok!(Stake::candidate_bond_more(Origin::signed(1), 30));
			total += 30;
			assert_eq!(Stake::total(), total);
		});
}

#[test]
fn candidate_bond_more_updates_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 50)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			let candidate_state = Stake::candidate_state(1).expect("updated => exists");
			assert_eq!(candidate_state.bond, 20);
			assert_ok!(Stake::candidate_bond_more(Origin::signed(1), 30));
			let candidate_state = Stake::candidate_state(1).expect("updated => exists");
			assert_eq!(candidate_state.bond, 50);
		});
}

#[test]
fn candidate_bond_more_updates_candidate_pool() {
	ExtBuilder::default()
		.with_balances(vec![(1, 50)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Stake::candidate_pool().0[0],
				Bond {
					owner: 1,
					amount: 20
				}
			);
			assert_ok!(Stake::candidate_bond_more(Origin::signed(1), 30));
			assert_eq!(
				Stake::candidate_pool().0[0],
				Bond {
					owner: 1,
					amount: 50
				}
			);
		});
}

// SCHEDULE CANDIDATE BOND LESS

#[test]
fn schedule_candidate_bond_less_event_emits_correctly() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
			assert_last_event!(MetaEvent::Stake(Event::CandidateBondLessRequested(
				1, 10, 3,
			)));
		});
}

#[test]
fn cannot_schedule_candidate_bond_less_if_request_exists() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 5));
			assert_noop!(
				Stake::schedule_candidate_bond_less(Origin::signed(1), 5),
				Error::<Test>::PendingCandidateRequestAlreadyExists
			);
		});
}

#[test]
fn cannot_schedule_candidate_bond_less_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::schedule_candidate_bond_less(Origin::signed(6), 50),
			Error::<Test>::CandidateDNE
		);
	});
}

#[test]
fn cannot_schedule_candidate_bond_less_if_new_total_below_min_candidate_stk() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_candidate_bond_less(Origin::signed(1), 21),
				Error::<Test>::CandidateBondBelowMin
			);
		});
}

#[test]
fn can_schedule_candidate_bond_less_if_leaving_candidates() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1));
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
		});
}

#[test]
fn cannot_schedule_candidate_bond_less_if_exited_candidates() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1));
			roll_to(10);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(1), 1));
			assert_noop!(
				Stake::schedule_candidate_bond_less(Origin::signed(1), 10),
				Error::<Test>::CandidateDNE
			);
		});
}

// 2. EXECUTE BOND LESS REQUEST

#[test]
fn execute_candidate_bond_less_emits_correct_event() {
	ExtBuilder::default()
		.with_balances(vec![(1, 50)])
		.with_candidates(vec![(1, 50)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 30));
			roll_to(10);
			assert_ok!(Stake::execute_candidate_bond_less(Origin::signed(1), 1));
			assert_last_event!(MetaEvent::Stake(Event::CandidateBondedLess(1, 30, 20)));
		});
}

#[test]
fn execute_candidate_bond_less_unreserves_balance() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::reserved_balance(&1), 30);
			assert_eq!(Balances::free_balance(&1), 0);
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
			roll_to(10);
			assert_ok!(Stake::execute_candidate_bond_less(Origin::signed(1), 1));
			assert_eq!(Balances::reserved_balance(&1), 20);
			assert_eq!(Balances::free_balance(&1), 10);
		});
}

#[test]
fn execute_candidate_bond_less_decreases_total() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			let mut total = Stake::total();
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
			roll_to(10);
			assert_ok!(Stake::execute_candidate_bond_less(Origin::signed(1), 1));
			total -= 10;
			assert_eq!(Stake::total(), total);
		});
}

#[test]
fn execute_candidate_bond_less_updates_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			let candidate_state = Stake::candidate_state(1).expect("updated => exists");
			assert_eq!(candidate_state.bond, 30);
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
			roll_to(10);
			assert_ok!(Stake::execute_candidate_bond_less(Origin::signed(1), 1));
			let candidate_state = Stake::candidate_state(1).expect("updated => exists");
			assert_eq!(candidate_state.bond, 20);
		});
}

#[test]
fn execute_candidate_bond_less_updates_candidate_pool() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Stake::candidate_pool().0[0],
				Bond {
					owner: 1,
					amount: 30
				}
			);
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
			roll_to(10);
			assert_ok!(Stake::execute_candidate_bond_less(Origin::signed(1), 1));
			assert_eq!(
				Stake::candidate_pool().0[0],
				Bond {
					owner: 1,
					amount: 20
				}
			);
		});
}

// CANCEL CANDIDATE BOND LESS REQUEST

#[test]
fn cancel_candidate_bond_less_emits_event() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
			assert_ok!(Stake::cancel_candidate_bond_less(Origin::signed(1)));
			assert_last_event!(MetaEvent::Stake(Event::CancelledCandidateBondLess(
				1, 10, 3,
			)));
		});
}

#[test]
fn cancel_candidate_bond_less_updates_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
			assert_ok!(Stake::cancel_candidate_bond_less(Origin::signed(1)));
			assert!(Stake::candidate_state(&1).unwrap().request.is_none());
		});
}

#[test]
fn only_candidate_can_cancel_candidate_bond_less_request() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_candidate_bond_less(Origin::signed(1), 10));
			assert_noop!(
				Stake::cancel_candidate_bond_less(Origin::signed(2)),
				Error::<Test>::CandidateDNE
			);
		});
}

// NOMINATE

#[test]
fn delegate_event_emits_correctly() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::delegate(Origin::signed(2), 1, 10, 0, 0));
			assert_last_event!(MetaEvent::Stake(Event::Delegation(
				2,
				10,
				1,
				DelegatorAdded::AddedToTop { new_total: 40 },
			)));
		});
}

#[test]
fn delegate_reserves_balance() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::reserved_balance(&2), 0);
			assert_eq!(Balances::free_balance(&2), 10);
			assert_ok!(Stake::delegate(Origin::signed(2), 1, 10, 0, 0));
			assert_eq!(Balances::reserved_balance(&2), 10);
			assert_eq!(Balances::free_balance(&2), 0);
		});
}

#[test]
fn delegate_updates_delegator_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert!(Stake::delegator_state(2).is_none());
			assert_ok!(Stake::delegate(Origin::signed(2), 1, 10, 0, 0));
			let delegator_state = Stake::delegator_state(2).expect("just delegated => exists");
			assert_eq!(delegator_state.total, 10);
			assert_eq!(
				delegator_state.delegations.0[0],
				Bond {
					owner: 1,
					amount: 10
				}
			);
		});
}

#[test]
fn delegate_updates_collator_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			let candidate_state = Stake::candidate_state(1).expect("registered in genesis");
			assert_eq!(candidate_state.total_backing, 30);
			assert_eq!(candidate_state.total_counted, 30);
			assert!(candidate_state.top_delegations.is_empty());
			assert_ok!(Stake::delegate(Origin::signed(2), 1, 10, 0, 0));
			let candidate_state = Stake::candidate_state(1).expect("just delegated => exists");
			assert_eq!(candidate_state.total_backing, 40);
			assert_eq!(candidate_state.total_counted, 40);
			assert_eq!(
				candidate_state.top_delegations[0],
				Bond {
					owner: 2,
					amount: 10
				}
			);
		});
}

#[test]
fn can_delegate_immediately_after_other_join_candidates() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::join_candidates(Origin::signed(1), 20, 0));
			assert_ok!(Stake::delegate(Origin::signed(2), 1, 20, 0, 0));
		});
}

#[test]
fn can_delegate_if_revoking() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 30), (3, 20), (4, 20)])
		.with_candidates(vec![(1, 20), (3, 20), (4, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			assert_ok!(Stake::delegate(Origin::signed(2), 4, 10, 0, 2));
		});
}

#[test]
fn cannot_delegate_if_leaving() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20), (3, 20)])
		.with_candidates(vec![(1, 20), (3, 20)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_noop!(
				Stake::delegate(Origin::signed(2), 3, 10, 0, 1),
				Error::<Test>::CannotDelegateIfLeaving
			);
		});
}

#[test]
fn cannot_delegate_if_candidate() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20)])
		.with_candidates(vec![(1, 20), (2, 20)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::delegate(Origin::signed(2), 1, 10, 0, 0),
				Error::<Test>::CandidateExists
			);
		});
}

#[test]
fn cannot_delegate_if_already_delegated() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 30)])
		.with_candidates(vec![(1, 20)])
		.with_delegations(vec![(2, 1, 20)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::delegate(Origin::signed(2), 1, 10, 1, 1),
				Error::<Test>::AlreadyDelegatedCandidate
			);
		});
}

#[test]
fn cannot_delegate_more_than_max_delegations() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 50), (3, 20), (4, 20), (5, 20), (6, 20)])
		.with_candidates(vec![(1, 20), (3, 20), (4, 20), (5, 20), (6, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10), (2, 4, 10), (2, 5, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::delegate(Origin::signed(2), 6, 10, 0, 4),
				Error::<Test>::ExceedMaxDelegationsPerDelegator,
			);
		});
}

#[test]
fn sufficient_delegate_weight_hint_succeeds() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 20),
			(2, 20),
			(3, 20),
			(4, 20),
			(5, 20),
			(6, 20),
			(7, 20),
			(8, 20),
			(9, 20),
			(10, 20),
		])
		.with_candidates(vec![(1, 20), (2, 20)])
		.with_delegations(vec![(3, 1, 10), (4, 1, 10), (5, 1, 10), (6, 1, 10)])
		.build()
		.execute_with(|| {
			let mut count = 4u32;
			for i in 7..11 {
				assert_ok!(Stake::delegate(Origin::signed(i), 1, 10, count, 0u32));
				count += 1u32;
			}
			let mut count = 0u32;
			for i in 3..11 {
				assert_ok!(Stake::delegate(Origin::signed(i), 2, 10, count, 1u32));
				count += 1u32;
			}
		});
}

#[test]
fn insufficient_delegate_weight_hint_fails() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 20),
			(2, 20),
			(3, 20),
			(4, 20),
			(5, 20),
			(6, 20),
			(7, 20),
			(8, 20),
			(9, 20),
			(10, 20),
		])
		.with_candidates(vec![(1, 20), (2, 20)])
		.with_delegations(vec![(3, 1, 10), (4, 1, 10), (5, 1, 10), (6, 1, 10)])
		.build()
		.execute_with(|| {
			let mut count = 3u32;
			for i in 7..11 {
				assert_noop!(
					Stake::delegate(Origin::signed(i), 1, 10, count, 0u32),
					Error::<Test>::TooLowCandidateDelegationCountToDelegate
				);
			}
			// to set up for next error test
			count = 4u32;
			for i in 7..11 {
				assert_ok!(Stake::delegate(Origin::signed(i), 1, 10, count, 0u32));
				count += 1u32;
			}
			count = 0u32;
			for i in 3..11 {
				assert_noop!(
					Stake::delegate(Origin::signed(i), 2, 10, count, 0u32),
					Error::<Test>::TooLowDelegationCountToDelegate
				);
				count += 1u32;
			}
		});
}

// SCHEDULE LEAVE DELEGATORS

#[test]
fn schedule_leave_delegators_event_emits_correctly() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_last_event!(MetaEvent::Stake(Event::DelegatorExitScheduled(1, 2, 3)));
		});
}

#[test]
fn cannot_schedule_leave_delegators_if_already_leaving() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_noop!(
				Stake::schedule_leave_delegators(Origin::signed(2)),
				Error::<Test>::DelegatorAlreadyLeaving
			);
		});
}

#[test]
fn cannot_schedule_leave_delegators_if_not_delegator() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_leave_delegators(Origin::signed(2)),
				Error::<Test>::DelegatorDNE
			);
		});
}

// EXECUTE LEAVE DELEGATORS

#[test]
fn execute_leave_delegators_event_emits_correctly() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			roll_to(10);
			assert_ok!(Stake::execute_leave_delegators(Origin::signed(2), 2, 1));
			assert_event_emitted!(Event::DelegatorLeft(2, 10));
		});
}

#[test]
fn execute_leave_delegators_unreserves_balance() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::reserved_balance(&2), 10);
			assert_eq!(Balances::free_balance(&2), 0);
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			roll_to(10);
			assert_ok!(Stake::execute_leave_delegators(Origin::signed(2), 2, 1));
			assert_eq!(Balances::reserved_balance(&2), 0);
			assert_eq!(Balances::free_balance(&2), 10);
		});
}

#[test]
fn execute_leave_delegators_decreases_total_staked() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::total(), 40);
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			roll_to(10);
			assert_ok!(Stake::execute_leave_delegators(Origin::signed(2), 2, 1));
			assert_eq!(Stake::total(), 30);
		});
}

#[test]
fn execute_leave_delegators_removes_delegator_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert!(Stake::delegator_state(2).is_some());
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			roll_to(10);
			assert_ok!(Stake::execute_leave_delegators(Origin::signed(2), 2, 1));
			assert!(Stake::delegator_state(2).is_none());
		});
}

#[test]
fn execute_leave_delegators_removes_delegations_from_collator_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 20), (3, 20), (4, 20), (5, 20)])
		.with_candidates(vec![(2, 20), (3, 20), (4, 20), (5, 20)])
		.with_delegations(vec![(1, 2, 10), (1, 3, 10), (1, 4, 10), (1, 5, 10)])
		.build()
		.execute_with(|| {
			for i in 2..6 {
				let candidate_state =
					Stake::candidate_state(i).expect("initialized in ext builder");
				assert_eq!(
					candidate_state.top_delegations[0],
					Bond {
						owner: 1,
						amount: 10
					}
				);
				assert_eq!(candidate_state.delegators.0[0], 1);
				assert_eq!(candidate_state.total_backing, 30);
			}
			assert_eq!(
				Stake::delegator_state(1).unwrap().delegations.0.len(),
				4usize
			);
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(1)));
			roll_to(10);
			assert_ok!(Stake::execute_leave_delegators(Origin::signed(1), 1, 10));
			for i in 2..6 {
				let candidate_state =
					Stake::candidate_state(i).expect("initialized in ext builder");
				assert!(candidate_state.top_delegations.is_empty());
				assert!(candidate_state.delegators.0.is_empty());
				assert_eq!(candidate_state.total_backing, 20);
			}
		});
}

#[test]
fn cannot_execute_leave_delegators_before_delay() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_noop!(
				Stake::execute_leave_delegators(Origin::signed(2), 2, 1),
				Error::<Test>::DelegatorCannotLeaveYet
			);
			// can execute after delay
			roll_to(10);
			assert_ok!(Stake::execute_leave_delegators(Origin::signed(2), 2, 1));
		});
}

#[test]
fn insufficient_execute_leave_delegators_weight_hint_fails() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20), (6, 20)])
		.with_candidates(vec![(1, 20)])
		.with_delegations(vec![(3, 1, 10), (4, 1, 10), (5, 1, 10), (6, 1, 10)])
		.build()
		.execute_with(|| {
			for i in 3..7 {
				assert_ok!(Stake::schedule_leave_delegators(Origin::signed(i)));
			}
			roll_to(10);
			for i in 3..7 {
				assert_noop!(
					Stake::execute_leave_delegators(Origin::signed(i), i, 0),
					Error::<Test>::TooLowDelegationCountToLeaveDelegators
				);
			}
		});
}

#[test]
fn sufficient_execute_leave_delegators_weight_hint_succeeds() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20), (6, 20)])
		.with_candidates(vec![(1, 20)])
		.with_delegations(vec![(3, 1, 10), (4, 1, 10), (5, 1, 10), (6, 1, 10)])
		.build()
		.execute_with(|| {
			for i in 3..7 {
				assert_ok!(Stake::schedule_leave_delegators(Origin::signed(i)));
			}
			roll_to(10);
			for i in 3..7 {
				assert_ok!(Stake::execute_leave_delegators(Origin::signed(i), i, 1));
			}
		});
}

// CANCEL LEAVE DELEGATORS

#[test]
fn cancel_leave_delegators_emits_correct_event() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_ok!(Stake::cancel_leave_delegators(Origin::signed(2)));
			assert_last_event!(MetaEvent::Stake(Event::DelegatorExitCancelled(2)));
		});
}

#[test]
fn cancel_leave_delegators_updates_delegator_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_ok!(Stake::cancel_leave_delegators(Origin::signed(2)));
			let delegator = Stake::delegator_state(&2).expect("just cancelled exit so exists");
			assert!(delegator.is_active());
		});
}

// SCHEDULE REVOKE DELEGATION

#[test]
fn revoke_delegation_event_emits_correctly() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 20), (3, 30)])
		.with_candidates(vec![(1, 30), (3, 30)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			assert_last_event!(MetaEvent::Stake(Event::DelegationRevocationScheduled(
				1, 2, 1, 3,
			)));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert_event_emitted!(Event::DelegatorLeftCandidate(2, 1, 10, 30));
		});
}

#[test]
fn can_revoke_delegation_if_revoking_another_delegation() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 20), (3, 20)])
		.with_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			// this is an exit implicitly because last delegation revoked
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 3));
		});
}

#[test]
fn can_revoke_if_leaving() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 20), (3, 20)])
		.with_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 3));
		});
}

#[test]
fn cannot_revoke_delegation_if_not_delegator() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::schedule_revoke_delegation(Origin::signed(2), 1),
			Error::<Test>::DelegatorDNE
		);
	});
}

#[test]
fn cannot_revoke_delegation_that_dne() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_revoke_delegation(Origin::signed(2), 3),
				Error::<Test>::DelegationDNE
			);
		});
}

#[test]
// See `cannot_execute_revoke_delegation_below_min_delegator_stake` for where the "must be above
// MinDelegatorStk" rule is now enforced.
fn can_schedule_revoke_delegation_below_min_delegator_stake() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 8), (3, 20)])
		.with_candidates(vec![(1, 20), (3, 20)])
		.with_delegations(vec![(2, 1, 5), (2, 3, 3)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
		});
}

// DELEGATOR BOND MORE

#[test]
fn delegator_bond_more_reserves_balance() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 15)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::reserved_balance(&2), 10);
			assert_eq!(Balances::free_balance(&2), 5);
			assert_ok!(Stake::delegator_bond_more(Origin::signed(2), 1, 5));
			assert_eq!(Balances::reserved_balance(&2), 15);
			assert_eq!(Balances::free_balance(&2), 0);
		});
}

#[test]
fn delegator_bond_more_increases_total_staked() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 15)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::total(), 40);
			assert_ok!(Stake::delegator_bond_more(Origin::signed(2), 1, 5));
			assert_eq!(Stake::total(), 45);
		});
}

#[test]
fn delegator_bond_more_updates_delegator_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 15)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::delegator_state(2).expect("exists").total, 10);
			assert_ok!(Stake::delegator_bond_more(Origin::signed(2), 1, 5));
			assert_eq!(Stake::delegator_state(2).expect("exists").total, 15);
		});
}

#[test]
fn delegator_bond_more_updates_candidate_state_top_delegations() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 15)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Stake::candidate_state(1).expect("exists").top_delegations[0],
				Bond {
					owner: 2,
					amount: 10
				}
			);
			assert_ok!(Stake::delegator_bond_more(Origin::signed(2), 1, 5));
			assert_eq!(
				Stake::candidate_state(1).expect("exists").top_delegations[0],
				Bond {
					owner: 2,
					amount: 15
				}
			);
		});
}

#[test]
fn delegator_bond_more_updates_candidate_state_bottom_delegations() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 20), (3, 20), (4, 20), (5, 20), (6, 20)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![
			(2, 1, 10),
			(3, 1, 20),
			(4, 1, 20),
			(5, 1, 20),
			(6, 1, 20),
		])
		.build()
		.execute_with(|| {
			assert_eq!(
				Stake::candidate_state(1)
					.expect("exists")
					.bottom_delegations[0],
				Bond {
					owner: 2,
					amount: 10
				}
			);
			assert_ok!(Stake::delegator_bond_more(Origin::signed(2), 1, 5));
			assert_last_event!(MetaEvent::Stake(Event::DelegationIncreased(2, 1, 5, false)));
			assert_eq!(
				Stake::candidate_state(1)
					.expect("exists")
					.bottom_delegations[0],
				Bond {
					owner: 2,
					amount: 15
				}
			);
		});
}

#[test]
fn delegator_bond_more_increases_total() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 15)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::total(), 40);
			assert_ok!(Stake::delegator_bond_more(Origin::signed(2), 1, 5));
			assert_eq!(Stake::total(), 45);
		});
}

#[test]
fn can_delegator_bond_more_for_leaving_candidate() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 15)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1));
			assert_ok!(Stake::delegator_bond_more(Origin::signed(2), 1, 5));
		});
}

// DELEGATOR BOND LESS

#[test]
fn delegator_bond_less_event_emits_correctly() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			assert_last_event!(MetaEvent::Stake(Event::DelegationDecreaseScheduled(
				2, 1, 5, 3,
			)));
		});
}

#[test]
fn delegator_bond_less_updates_delegator_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			let state = Stake::delegator_state(&2).expect("just request bonded less so exists");
			assert_eq!(
				state.requests().get(&1),
				Some(&DelegationRequest {
					collator: 1,
					amount: 5,
					when_executable: 3,
					action: DelegationChange::Decrease
				})
			);
		});
}

#[test]
fn can_delegator_bond_less_if_leaving() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 15)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 1));
		});
}

#[test]
fn cannot_delegator_bond_less_if_revoking() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25), (3, 20)])
		.with_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			assert_noop!(
				Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 1),
				Error::<Test>::PendingDelegationRequestAlreadyExists
			);
		});
}

#[test]
fn cannot_delegator_bond_less_if_not_delegator() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5),
			Error::<Test>::DelegatorDNE
		);
	});
}

#[test]
fn cannot_delegator_bond_less_if_candidate_dne() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_delegator_bond_less(Origin::signed(2), 3, 5),
				Error::<Test>::DelegationDNE
			);
		});
}

#[test]
fn cannot_delegator_bond_less_if_delegation_dne() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10), (3, 30)])
		.with_candidates(vec![(1, 30), (3, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_delegator_bond_less(Origin::signed(2), 3, 5),
				Error::<Test>::DelegationDNE
			);
		});
}

#[test]
fn cannot_delegator_bond_less_below_min_collator_stk() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 6),
				Error::<Test>::DelegatorBondBelowMin
			);
		});
}

#[test]
fn cannot_delegator_bond_less_more_than_total_delegation() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 11),
				Error::<Test>::DelegatorBondBelowMin
			);
		});
}

#[test]
fn cannot_delegator_bond_less_below_min_delegation() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 20), (3, 30)])
		.with_candidates(vec![(1, 30), (3, 30)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 8),
				Error::<Test>::DelegationBelowMin
			);
		});
}

// EXECUTE PENDING DELEGATION REQUEST

// 1. REVOKE DELEGATION

#[test]
fn execute_revoke_delegation_emits_exit_event_if_exit_happens() {
	// last delegation is revocation
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert_event_emitted!(Event::DelegatorLeftCandidate(2, 1, 10, 30));
			assert_event_emitted!(Event::DelegatorLeft(2, 10));
		});
}

#[test]
fn cannot_execute_revoke_delegation_below_min_delegator_stake() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 8), (3, 20)])
		.with_candidates(vec![(1, 20), (3, 20)])
		.with_delegations(vec![(2, 1, 5), (2, 3, 3)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			assert_noop!(
				Stake::execute_delegation_request(Origin::signed(2), 2, 1),
				Error::<Test>::DelegatorBondBelowMin
			);
			// but delegator can cancel the request and request to leave instead:
			assert_ok!(Stake::cancel_delegation_request(Origin::signed(2), 1));
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(2)));
			roll_to(20);
			assert_ok!(Stake::execute_leave_delegators(Origin::signed(2), 2, 2));
		});
}

#[test]
fn revoke_delegation_executes_exit_if_last_delegation() {
	// last delegation is revocation
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert_event_emitted!(Event::DelegatorLeftCandidate(2, 1, 10, 30));
			assert_event_emitted!(Event::DelegatorLeft(2, 10));
		});
}

#[test]
fn execute_revoke_delegation_emits_correct_event() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 20), (3, 30)])
		.with_candidates(vec![(1, 30), (3, 30)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert_event_emitted!(Event::DelegatorLeftCandidate(2, 1, 10, 30));
		});
}

#[test]
fn execute_revoke_delegation_unreserves_balance() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::reserved_balance(&2), 10);
			assert_eq!(Balances::free_balance(&2), 0);
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert_eq!(Balances::reserved_balance(&2), 0);
			assert_eq!(Balances::free_balance(&2), 10);
		});
}

#[test]
fn execute_revoke_delegation_adds_revocation_to_delegator_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 20), (3, 20)])
		.with_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert!(Stake::delegator_state(2)
				.expect("exists")
				.requests
				.requests
				.get(&1)
				.is_none());
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			assert!(Stake::delegator_state(2)
				.expect("exists")
				.requests
				.requests
				.get(&1)
				.is_some());
		});
}

#[test]
fn execute_revoke_delegation_removes_revocation_from_delegator_state_upon_execution() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 20), (3, 20)])
		.with_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert!(Stake::delegator_state(2)
				.expect("exists")
				.requests
				.requests
				.get(&1)
				.is_none());
		});
}

#[test]
fn execute_revoke_delegation_decreases_total_staked() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::total(), 40);
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert_eq!(Stake::total(), 30);
		});
}

#[test]
fn execute_revoke_delegation_for_last_delegation_removes_delegator_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert!(Stake::delegator_state(2).is_some());
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			// this will be confusing for people
			// if status is leaving, then execute_delegation_request works if last delegation
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert!(Stake::delegator_state(2).is_none());
		});
}

#[test]
fn execute_revoke_delegation_removes_delegation_from_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Stake::candidate_state(1)
					.expect("exists")
					.delegators
					.0
					.len(),
				1usize
			);
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert!(Stake::candidate_state(1)
				.expect("exists")
				.delegators
				.0
				.is_empty());
		});
}

#[test]
fn can_execute_revoke_delegation_for_leaving_candidate() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1));
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			// can execute delegation request for leaving candidate
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
		});
}

#[test]
fn can_execute_leave_candidates_if_revoking_candidate() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1));
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			roll_to(10);
			// revocation executes during execute leave candidates (callable by anyone)
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(1), 1));
			assert!(!Stake::is_delegator(&2));
			assert_eq!(Balances::reserved_balance(&2), 0);
			assert_eq!(Balances::free_balance(&2), 10);
		});
}

#[test]
fn delegator_bond_more_after_revoke_delegation_does_not_effect_exit() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 30), (3, 30)])
		.with_candidates(vec![(1, 30), (3, 30)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			assert_ok!(Stake::delegator_bond_more(Origin::signed(2), 3, 10));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert!(Stake::is_delegator(&2));
			assert_eq!(Balances::reserved_balance(&2), 20);
			assert_eq!(Balances::free_balance(&2), 10);
		});
}

#[test]
fn delegator_bond_less_after_revoke_delegation_does_not_effect_exit() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 30), (3, 30)])
		.with_candidates(vec![(1, 30), (3, 30)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			assert_last_event!(MetaEvent::Stake(Event::DelegationRevocationScheduled(
				1, 2, 1, 3,
			)));
			assert_noop!(
				Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 2),
				Error::<Test>::PendingDelegationRequestAlreadyExists
			);
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 3, 2));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 3));
			assert_last_event!(MetaEvent::Stake(Event::DelegationDecreased(2, 3, 2, true)));
			assert!(Stake::is_delegator(&2));
			assert_eq!(Balances::reserved_balance(&2), 8);
			assert_eq!(Balances::free_balance(&2), 22);
		});
}

// 2. EXECUTE BOND LESS

#[test]
fn execute_delegator_bond_less_unreserves_balance() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::reserved_balance(&2), 10);
			assert_eq!(Balances::free_balance(&2), 0);
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert_eq!(Balances::reserved_balance(&2), 5);
			assert_eq!(Balances::free_balance(&2), 5);
		});
}

#[test]
fn execute_delegator_bond_less_decreases_total_staked() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::total(), 40);
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert_eq!(Stake::total(), 35);
		});
}

#[test]
fn execute_delegator_bond_less_updates_delegator_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 15)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::delegator_state(2).expect("exists").total, 10);
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert_eq!(Stake::delegator_state(2).expect("exists").total, 5);
		});
}

#[test]
fn execute_delegator_bond_less_updates_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 15)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Stake::candidate_state(1).expect("exists").top_delegations[0],
				Bond {
					owner: 2,
					amount: 10
				}
			);
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert_eq!(
				Stake::candidate_state(1).expect("exists").top_delegations[0],
				Bond {
					owner: 2,
					amount: 5
				}
			);
		});
}

#[test]
fn execute_delegator_bond_less_decreases_total() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 15)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(Stake::total(), 40);
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert_eq!(Stake::total(), 35);
		});
}

#[test]
fn execute_delegator_bond_less_updates_just_bottom_delegations() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 10), (3, 11), (4, 12), (5, 14), (6, 15)])
		.with_candidates(vec![(1, 20)])
		.with_delegations(vec![
			(2, 1, 10),
			(3, 1, 11),
			(4, 1, 12),
			(5, 1, 14),
			(6, 1, 15),
		])
		.build()
		.execute_with(|| {
			let pre_call_collator_state =
				Stake::candidate_state(&1).expect("delegated by all so exists");
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 2));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			let post_call_collator_state =
				Stake::candidate_state(&1).expect("delegated by all so exists");
			let mut not_equal = false;
			for Bond { owner, amount } in pre_call_collator_state.bottom_delegations {
				for Bond {
					owner: post_owner,
					amount: post_amount,
				} in &post_call_collator_state.bottom_delegations
				{
					if &owner == post_owner {
						if &amount != post_amount {
							not_equal = true;
							break;
						}
					}
				}
			}
			assert!(not_equal);
			let mut equal = true;
			for Bond { owner, amount } in pre_call_collator_state.top_delegations {
				for Bond {
					owner: post_owner,
					amount: post_amount,
				} in &post_call_collator_state.top_delegations
				{
					if &owner == post_owner {
						if &amount != post_amount {
							equal = false;
							break;
						}
					}
				}
			}
			assert!(equal);
			assert_eq!(
				pre_call_collator_state.total_backing - 2,
				post_call_collator_state.total_backing
			);
			assert_eq!(
				pre_call_collator_state.total_counted,
				post_call_collator_state.total_counted
			);
		});
}

#[test]
fn execute_delegator_bond_less_does_not_delete_bottom_delegations() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 10), (3, 11), (4, 12), (5, 14), (6, 15)])
		.with_candidates(vec![(1, 20)])
		.with_delegations(vec![
			(2, 1, 10),
			(3, 1, 11),
			(4, 1, 12),
			(5, 1, 14),
			(6, 1, 15),
		])
		.build()
		.execute_with(|| {
			let pre_call_collator_state =
				Stake::candidate_state(&1).expect("delegated by all so exists");
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(6), 1, 4));
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(6), 6, 1));
			let post_call_collator_state =
				Stake::candidate_state(&1).expect("delegated by all so exists");
			let mut equal = true;
			for Bond { owner, amount } in pre_call_collator_state.bottom_delegations {
				for Bond {
					owner: post_owner,
					amount: post_amount,
				} in &post_call_collator_state.bottom_delegations
				{
					if &owner == post_owner {
						if &amount != post_amount {
							equal = false;
							break;
						}
					}
				}
			}
			assert!(equal);
			let mut not_equal = false;
			for Bond { owner, amount } in pre_call_collator_state.top_delegations {
				for Bond {
					owner: post_owner,
					amount: post_amount,
				} in &post_call_collator_state.top_delegations
				{
					if &owner == post_owner {
						if &amount != post_amount {
							not_equal = true;
							break;
						}
					}
				}
			}
			assert!(not_equal);
			assert_eq!(
				pre_call_collator_state.total_backing - 4,
				post_call_collator_state.total_backing
			);
			assert_eq!(
				pre_call_collator_state.total_counted - 4,
				post_call_collator_state.total_counted
			);
		});
}

#[test]
fn can_execute_delegator_bond_less_for_leaving_candidate() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 15)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 15)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(1), 1));
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			roll_to(10);
			// can execute bond more delegation request for leaving candidate
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
		});
}

// CANCEL PENDING DELEGATION REQUEST
// 1. CANCEL REVOKE DELEGATION

#[test]
fn cancel_revoke_delegation_emits_correct_event() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			assert_ok!(Stake::cancel_delegation_request(Origin::signed(2), 1));
			assert_last_event!(MetaEvent::Stake(Event::CancelledDelegationRequest(
				2,
				DelegationRequest {
					collator: 1,
					amount: 10,
					when_executable: 3,
					action: DelegationChange::Revoke,
				},
			)));
		});
}

#[test]
fn cancel_revoke_delegation_updates_delegator_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			let state = Stake::delegator_state(&2).unwrap();
			assert_eq!(
				state.requests().get(&1),
				Some(&DelegationRequest {
					collator: 1,
					amount: 10,
					when_executable: 3,
					action: DelegationChange::Revoke,
				})
			);
			assert_eq!(state.requests.less_total, 10);
			assert_ok!(Stake::cancel_delegation_request(Origin::signed(2), 1));
			let state = Stake::delegator_state(&2).unwrap();
			assert!(state.requests().get(&1).is_none());
			assert_eq!(state.requests.less_total, 0);
		});
}

// 2. CANCEL DELEGATOR BOND LESS

#[test]
fn cancel_delegator_bond_less_correct_event() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 15)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 15)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			assert_ok!(Stake::cancel_delegation_request(Origin::signed(2), 1));
			assert_last_event!(MetaEvent::Stake(Event::CancelledDelegationRequest(
				2,
				DelegationRequest {
					collator: 1,
					amount: 5,
					when_executable: 3,
					action: DelegationChange::Decrease,
				},
			)));
		});
}

#[test]
fn cancel_delegator_bond_less_updates_delegator_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 15)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 15)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(2), 1, 5));
			let state = Stake::delegator_state(&2).unwrap();
			assert_eq!(
				state.requests().get(&1),
				Some(&DelegationRequest {
					collator: 1,
					amount: 5,
					when_executable: 3,
					action: DelegationChange::Decrease,
				})
			);
			assert_eq!(state.requests.less_total, 5);
			assert_ok!(Stake::cancel_delegation_request(Origin::signed(2), 1));
			let state = Stake::delegator_state(&2).unwrap();
			assert!(state.requests().get(&1).is_none());
			assert_eq!(state.requests.less_total, 0);
		});
}

// ~~ PROPERTY-BASED TESTS ~~

#[test]
fn delegator_schedule_revocation_total() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 40), (3, 20), (4, 20), (5, 20)])
		.with_candidates(vec![(1, 20), (3, 20), (4, 20), (5, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10), (2, 4, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 1));
			assert_eq!(
				Stake::delegator_state(2)
					.expect("exists")
					.requests
					.less_total,
				10
			);
			roll_to(10);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 1));
			assert_eq!(
				Stake::delegator_state(2)
					.expect("exists")
					.requests
					.less_total,
				0
			);
			assert_ok!(Stake::delegate(Origin::signed(2), 5, 10, 0, 2));
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 3));
			assert_ok!(Stake::schedule_revoke_delegation(Origin::signed(2), 4));
			assert_eq!(
				Stake::delegator_state(2)
					.expect("exists")
					.requests
					.less_total,
				20
			);
			roll_to(20);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 3));
			assert_eq!(
				Stake::delegator_state(2)
					.expect("exists")
					.requests
					.less_total,
				10
			);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(2), 2, 4));
			assert_eq!(
				Stake::delegator_state(2)
					.expect("exists")
					.requests
					.less_total,
				0
			);
		});
}

#[test]
fn parachain_bond_inflation_reserve_matches_config() {
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
		.with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_delegations(vec![
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
				Event::CollatorChosen(2, 3, 20),
				Event::CollatorChosen(2, 4, 20),
				Event::CollatorChosen(2, 5, 10),
				Event::NewRound(5, 2, 5, 140),
			];
			assert_eq_events!(expected.clone());
			assert_eq!(Balances::free_balance(&11), 1);
			// ~ set block author as 1 for all blocks this round
			set_author(2, 1, 100);
			roll_to(16);
			// distribute total issuance to collator 1 and its delegators 6, 7, 19
			let mut new = vec![
				Event::CollatorChosen(3, 1, 50),
				Event::CollatorChosen(3, 2, 40),
				Event::CollatorChosen(3, 3, 20),
				Event::CollatorChosen(3, 4, 20),
				Event::CollatorChosen(3, 5, 10),
				Event::NewRound(10, 3, 5, 140),
				Event::ReservedForParachainBond(11, 15),
				Event::CollatorChosen(4, 1, 50),
				Event::CollatorChosen(4, 2, 40),
				Event::CollatorChosen(4, 3, 20),
				Event::CollatorChosen(4, 4, 20),
				Event::CollatorChosen(4, 5, 10),
				Event::NewRound(15, 4, 5, 140),
				Event::Rewarded(1, 20),
				Event::Rewarded(7, 5),
				Event::Rewarded(10, 5),
				Event::Rewarded(6, 5),
			];
			expected.append(&mut new);
			assert_eq_events!(expected.clone());
			assert_eq!(Balances::free_balance(&11), 16);
			// ~ set block author as 1 for all blocks this round
			set_author(3, 1, 100);
			set_author(4, 1, 100);
			set_author(5, 1, 100);
			// 1. ensure delegators are paid for 2 rounds after they leave
			assert_noop!(
				Stake::schedule_leave_delegators(Origin::signed(66)),
				Error::<Test>::DelegatorDNE
			);
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(6)));
			// fast forward to block in which delegator 6 exit executes
			roll_to(25);
			assert_ok!(Stake::execute_leave_delegators(Origin::signed(6), 6, 10));
			roll_to(30);
			let mut new2 = vec![
				Event::DelegatorExitScheduled(4, 6, 6),
				Event::ReservedForParachainBond(11, 16),
				Event::CollatorChosen(5, 1, 50),
				Event::CollatorChosen(5, 2, 40),
				Event::CollatorChosen(5, 3, 20),
				Event::CollatorChosen(5, 4, 20),
				Event::CollatorChosen(5, 5, 10),
				Event::NewRound(20, 5, 5, 140),
				Event::Rewarded(1, 21),
				Event::Rewarded(7, 5),
				Event::Rewarded(10, 5),
				Event::Rewarded(6, 5),
				Event::ReservedForParachainBond(11, 16),
				Event::CollatorChosen(6, 1, 50),
				Event::CollatorChosen(6, 2, 40),
				Event::CollatorChosen(6, 3, 20),
				Event::CollatorChosen(6, 4, 20),
				Event::CollatorChosen(6, 5, 10),
				Event::NewRound(25, 6, 5, 140),
				Event::Rewarded(1, 22),
				Event::Rewarded(7, 6),
				Event::Rewarded(10, 6),
				Event::Rewarded(6, 6),
				Event::DelegatorLeftCandidate(6, 1, 10, 40),
				Event::DelegatorLeft(6, 10),
				Event::ReservedForParachainBond(11, 17),
				Event::CollatorChosen(7, 1, 40),
				Event::CollatorChosen(7, 2, 40),
				Event::CollatorChosen(7, 3, 20),
				Event::CollatorChosen(7, 4, 20),
				Event::CollatorChosen(7, 5, 10),
				Event::NewRound(30, 7, 5, 130),
				Event::Rewarded(1, 24),
				Event::Rewarded(7, 6),
				Event::Rewarded(10, 6),
				Event::Rewarded(6, 6),
			];
			expected.append(&mut new2);
			assert_eq_events!(expected.clone());
			assert_eq!(Balances::free_balance(&11), 65);
			assert_ok!(Stake::set_parachain_bond_reserve_percent(
				Origin::root(),
				Percent::from_percent(50)
			));
			// 6 won't be paid for this round because they left already
			set_author(6, 1, 100);
			roll_to(35);
			// keep paying 6
			let mut new3 = vec![
				Event::ParachainBondReservePercentSet(
					Percent::from_percent(30),
					Percent::from_percent(50),
				),
				Event::ReservedForParachainBond(11, 30),
				Event::CollatorChosen(8, 1, 40),
				Event::CollatorChosen(8, 2, 40),
				Event::CollatorChosen(8, 3, 20),
				Event::CollatorChosen(8, 4, 20),
				Event::CollatorChosen(8, 5, 10),
				Event::NewRound(35, 8, 5, 130),
				Event::Rewarded(1, 20),
				Event::Rewarded(7, 4),
				Event::Rewarded(10, 4),
				Event::Rewarded(6, 4),
			];
			expected.append(&mut new3);
			assert_eq_events!(expected.clone());
			assert_eq!(Balances::free_balance(&11), 95);
			set_author(7, 1, 100);
			roll_to(40);
			// no more paying 6
			let mut new4 = vec![
				Event::ReservedForParachainBond(11, 32),
				Event::CollatorChosen(9, 1, 40),
				Event::CollatorChosen(9, 2, 40),
				Event::CollatorChosen(9, 3, 20),
				Event::CollatorChosen(9, 4, 20),
				Event::CollatorChosen(9, 5, 10),
				Event::NewRound(40, 9, 5, 130),
				Event::Rewarded(1, 22),
				Event::Rewarded(7, 5),
				Event::Rewarded(10, 5),
			];
			expected.append(&mut new4);
			assert_eq_events!(expected.clone());
			assert_eq!(Balances::free_balance(&11), 127);
			set_author(8, 1, 100);
			assert_ok!(Stake::delegate(Origin::signed(8), 1, 10, 10, 10));
			roll_to(45);
			// new delegation is not rewarded yet
			let mut new5 = vec![
				Event::Delegation(8, 10, 1, DelegatorAdded::AddedToTop { new_total: 50 }),
				Event::ReservedForParachainBond(11, 33),
				Event::CollatorChosen(10, 1, 50),
				Event::CollatorChosen(10, 2, 40),
				Event::CollatorChosen(10, 3, 20),
				Event::CollatorChosen(10, 4, 20),
				Event::CollatorChosen(10, 5, 10),
				Event::NewRound(45, 10, 5, 140),
				Event::Rewarded(1, 23),
				Event::Rewarded(7, 5),
				Event::Rewarded(10, 5),
			];
			expected.append(&mut new5);
			assert_eq_events!(expected.clone());
			assert_eq!(Balances::free_balance(&11), 160);
			set_author(9, 1, 100);
			set_author(10, 1, 100);
			roll_to(50);
			// new delegation is still not rewarded yet
			let mut new6 = vec![
				Event::ReservedForParachainBond(11, 35),
				Event::CollatorChosen(11, 1, 50),
				Event::CollatorChosen(11, 2, 40),
				Event::CollatorChosen(11, 3, 20),
				Event::CollatorChosen(11, 4, 20),
				Event::CollatorChosen(11, 5, 10),
				Event::NewRound(50, 11, 5, 140),
				Event::Rewarded(1, 24),
				Event::Rewarded(7, 5),
				Event::Rewarded(10, 5),
			];
			expected.append(&mut new6);
			assert_eq_events!(expected.clone());
			assert_eq!(Balances::free_balance(&11), 195);
			roll_to(55);
			// new delegation is rewarded, 2 rounds after joining (`RewardPaymentDelay` is 2)
			let mut new7 = vec![
				Event::ReservedForParachainBond(11, 37),
				Event::CollatorChosen(12, 1, 50),
				Event::CollatorChosen(12, 2, 40),
				Event::CollatorChosen(12, 3, 20),
				Event::CollatorChosen(12, 4, 20),
				Event::CollatorChosen(12, 5, 10),
				Event::NewRound(55, 12, 5, 140),
				Event::Rewarded(1, 24),
				Event::Rewarded(7, 4),
				Event::Rewarded(8, 4),
				Event::Rewarded(10, 4),
			];
			expected.append(&mut new7);
			assert_eq_events!(expected);
			assert_eq!(Balances::free_balance(&11), 232);
		});
}

#[test]
fn paid_collator_commission_matches_config() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
		])
		.with_candidates(vec![(1, 20)])
		.with_delegations(vec![(2, 1, 10), (3, 1, 10)])
		.build()
		.execute_with(|| {
			roll_to(8);
			// chooses top TotalSelectedCandidates (5), in order
			let mut expected = vec![
				Event::CollatorChosen(2, 1, 40),
				Event::NewRound(5, 2, 1, 40),
			];
			assert_eq_events!(expected.clone());
			assert_ok!(Stake::join_candidates(Origin::signed(4), 20u128, 100u32));
			assert_last_event!(MetaEvent::Stake(Event::JoinedCollatorCandidates(
				4, 20u128, 60u128,
			)));
			roll_to(9);
			assert_ok!(Stake::delegate(Origin::signed(5), 4, 10, 10, 10));
			assert_ok!(Stake::delegate(Origin::signed(6), 4, 10, 10, 10));
			roll_to(11);
			let mut new = vec![
				Event::JoinedCollatorCandidates(4, 20, 60),
				Event::Delegation(5, 10, 4, DelegatorAdded::AddedToTop { new_total: 30 }),
				Event::Delegation(6, 10, 4, DelegatorAdded::AddedToTop { new_total: 40 }),
				Event::CollatorChosen(3, 1, 40),
				Event::CollatorChosen(3, 4, 40),
				Event::NewRound(10, 3, 2, 80),
			];
			expected.append(&mut new);
			assert_eq_events!(expected.clone());
			// only reward author with id 4
			set_author(3, 4, 100);
			roll_to(21);
			// 20% of 10 is commission + due_portion (0) = 2 + 4 = 6
			// all delegator payouts are 10-2 = 8 * stake_pct
			let mut new2 = vec![
				Event::CollatorChosen(4, 1, 40),
				Event::CollatorChosen(4, 4, 40),
				Event::NewRound(15, 4, 2, 80),
				Event::CollatorChosen(5, 1, 40),
				Event::CollatorChosen(5, 4, 40),
				Event::NewRound(20, 5, 2, 80),
				Event::Rewarded(4, 18),
				Event::Rewarded(6, 6),
				Event::Rewarded(5, 6),
			];
			expected.append(&mut new2);
			assert_eq_events!(expected);
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
		.with_candidates(vec![(1, 500), (2, 200)])
		.with_delegations(vec![(3, 1, 100), (4, 1, 100), (5, 2, 100), (6, 2, 100)])
		.build()
		.execute_with(|| {
			roll_to(11);
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(2), 2));
			let info = Stake::candidate_state(&2).unwrap();
			assert_eq!(info.state, CollatorStatus::Leaving(5));
			roll_to(21);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(2), 2));
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
				Event::CandidateScheduledExit(3, 2, 5),
				Event::CollatorChosen(4, 1, 700),
				Event::NewRound(15, 4, 1, 700),
				Event::CollatorChosen(5, 1, 700),
				Event::NewRound(20, 5, 1, 700),
				Event::CandidateLeft(2, 400, 700),
			];
			assert_eq_events!(expected);
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
		.with_candidates(vec![(1, 100), (2, 90), (3, 80), (4, 70), (5, 60), (6, 50)])
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
			assert_eq_events!(expected.clone());
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(6), 6));
			assert_last_event!(MetaEvent::Stake(Event::CandidateScheduledExit(2, 6, 4)));
			roll_to(21);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(6), 6));
			assert_ok!(Stake::join_candidates(Origin::signed(6), 69u128, 100u32));
			assert_last_event!(MetaEvent::Stake(Event::JoinedCollatorCandidates(
				6, 69u128, 469u128,
			)));
			roll_to(27);
			// should choose top TotalSelectedCandidates (5), in order
			let expected = vec![
				Event::CollatorChosen(2, 1, 100),
				Event::CollatorChosen(2, 2, 90),
				Event::CollatorChosen(2, 3, 80),
				Event::CollatorChosen(2, 4, 70),
				Event::CollatorChosen(2, 5, 60),
				Event::NewRound(5, 2, 5, 400),
				Event::CandidateScheduledExit(2, 6, 4),
				Event::CollatorChosen(3, 1, 100),
				Event::CollatorChosen(3, 2, 90),
				Event::CollatorChosen(3, 3, 80),
				Event::CollatorChosen(3, 4, 70),
				Event::CollatorChosen(3, 5, 60),
				Event::NewRound(10, 3, 5, 400),
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
				Event::CandidateLeft(6, 50, 400),
				Event::JoinedCollatorCandidates(6, 69, 469),
				Event::CollatorChosen(6, 1, 100),
				Event::CollatorChosen(6, 2, 90),
				Event::CollatorChosen(6, 3, 80),
				Event::CollatorChosen(6, 4, 70),
				Event::CollatorChosen(6, 6, 69),
				Event::NewRound(25, 6, 5, 409),
			];
			assert_eq_events!(expected);
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
		.with_candidates(vec![(1, 100), (2, 90), (3, 80), (4, 70), (5, 60), (6, 50)])
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
			assert_eq_events!(expected.clone());
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
				Event::CollatorChosen(4, 1, 100),
				Event::CollatorChosen(4, 2, 90),
				Event::CollatorChosen(4, 3, 80),
				Event::CollatorChosen(4, 4, 70),
				Event::CollatorChosen(4, 5, 60),
				Event::NewRound(15, 4, 5, 400),
				Event::Rewarded(1, 305),
			];
			expected.append(&mut new);
			assert_eq_events!(expected.clone());
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
				Event::CollatorChosen(6, 1, 100),
				Event::CollatorChosen(6, 2, 90),
				Event::CollatorChosen(6, 3, 80),
				Event::CollatorChosen(6, 4, 70),
				Event::CollatorChosen(6, 5, 60),
				Event::NewRound(25, 6, 5, 400),
				Event::Rewarded(1, 192),
				Event::Rewarded(2, 128),
			];
			expected.append(&mut new1);
			assert_eq_events!(expected.clone());
			// ~ each collator produces 1 block this round
			set_author(6, 1, 20);
			set_author(6, 2, 20);
			set_author(6, 3, 20);
			set_author(6, 4, 20);
			set_author(6, 5, 20);
			roll_to(39);
			// pay 20% issuance for all collators
			let mut new2 = vec![
				Event::CollatorChosen(7, 1, 100),
				Event::CollatorChosen(7, 2, 90),
				Event::CollatorChosen(7, 3, 80),
				Event::CollatorChosen(7, 4, 70),
				Event::CollatorChosen(7, 5, 60),
				Event::NewRound(30, 7, 5, 400),
				Event::CollatorChosen(8, 1, 100),
				Event::CollatorChosen(8, 2, 90),
				Event::CollatorChosen(8, 3, 80),
				Event::CollatorChosen(8, 4, 70),
				Event::CollatorChosen(8, 5, 60),
				Event::NewRound(35, 8, 5, 400),
				Event::Rewarded(5, 67),
				Event::Rewarded(3, 67),
				Event::Rewarded(4, 67),
				Event::Rewarded(1, 67),
				Event::Rewarded(2, 67),
			];
			expected.append(&mut new2);
			assert_eq_events!(expected);
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
fn multiple_delegations() {
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
		.with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_delegations(vec![
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
				Event::CollatorChosen(2, 3, 20),
				Event::CollatorChosen(2, 4, 20),
				Event::CollatorChosen(2, 5, 10),
				Event::NewRound(5, 2, 5, 140),
			];
			assert_eq_events!(expected.clone());
			assert_ok!(Stake::delegate(Origin::signed(6), 2, 10, 10, 10));
			assert_ok!(Stake::delegate(Origin::signed(6), 3, 10, 10, 10));
			assert_ok!(Stake::delegate(Origin::signed(6), 4, 10, 10, 10));
			roll_to(16);
			let mut new = vec![
				Event::Delegation(6, 10, 2, DelegatorAdded::AddedToTop { new_total: 50 }),
				Event::Delegation(6, 10, 3, DelegatorAdded::AddedToTop { new_total: 30 }),
				Event::Delegation(6, 10, 4, DelegatorAdded::AddedToTop { new_total: 30 }),
				Event::CollatorChosen(3, 1, 50),
				Event::CollatorChosen(3, 2, 50),
				Event::CollatorChosen(3, 3, 30),
				Event::CollatorChosen(3, 4, 30),
				Event::CollatorChosen(3, 5, 10),
				Event::NewRound(10, 3, 5, 170),
				Event::CollatorChosen(4, 1, 50),
				Event::CollatorChosen(4, 2, 50),
				Event::CollatorChosen(4, 3, 30),
				Event::CollatorChosen(4, 4, 30),
				Event::CollatorChosen(4, 5, 10),
				Event::NewRound(15, 4, 5, 170),
			];
			expected.append(&mut new);
			assert_eq_events!(expected.clone());
			roll_to(21);
			assert_ok!(Stake::delegate(Origin::signed(7), 2, 80, 10, 10));
			assert_ok!(Stake::delegate(Origin::signed(10), 2, 10, 10, 10),);
			roll_to(26);
			let mut new2 = vec![
				Event::CollatorChosen(5, 1, 50),
				Event::CollatorChosen(5, 2, 50),
				Event::CollatorChosen(5, 3, 30),
				Event::CollatorChosen(5, 4, 30),
				Event::CollatorChosen(5, 5, 10),
				Event::NewRound(20, 5, 5, 170),
				Event::Delegation(7, 80, 2, DelegatorAdded::AddedToTop { new_total: 130 }),
				Event::Delegation(10, 10, 2, DelegatorAdded::AddedToBottom),
				Event::CollatorChosen(6, 1, 50),
				Event::CollatorChosen(6, 2, 130),
				Event::CollatorChosen(6, 3, 30),
				Event::CollatorChosen(6, 4, 30),
				Event::CollatorChosen(6, 5, 10),
				Event::NewRound(25, 6, 5, 250),
			];
			expected.append(&mut new2);
			assert_eq_events!(expected.clone());
			assert_ok!(Stake::schedule_leave_candidates(Origin::signed(2), 5));
			assert_last_event!(MetaEvent::Stake(Event::CandidateScheduledExit(6, 2, 8)));
			roll_to(31);
			let mut new3 = vec![
				Event::CandidateScheduledExit(6, 2, 8),
				Event::CollatorChosen(7, 1, 50),
				Event::CollatorChosen(7, 3, 30),
				Event::CollatorChosen(7, 4, 30),
				Event::CollatorChosen(7, 5, 10),
				Event::NewRound(30, 7, 4, 120),
			];
			expected.append(&mut new3);
			assert_eq_events!(expected);
			// verify that delegations are removed after collator leaves, not before
			assert_eq!(Stake::delegator_state(7).unwrap().total, 90);
			assert_eq!(
				Stake::delegator_state(7).unwrap().delegations.0.len(),
				2usize
			);
			assert_eq!(Stake::delegator_state(6).unwrap().total, 40);
			assert_eq!(
				Stake::delegator_state(6).unwrap().delegations.0.len(),
				4usize
			);
			assert_eq!(Balances::reserved_balance(&6), 40);
			assert_eq!(Balances::reserved_balance(&7), 90);
			assert_eq!(Balances::free_balance(&6), 60);
			assert_eq!(Balances::free_balance(&7), 10);
			roll_to(40);
			assert_ok!(Stake::execute_leave_candidates(Origin::signed(2), 2));
			assert_eq!(Stake::delegator_state(7).unwrap().total, 10);
			assert_eq!(Stake::delegator_state(6).unwrap().total, 30);
			assert_eq!(
				Stake::delegator_state(7).unwrap().delegations.0.len(),
				1usize
			);
			assert_eq!(
				Stake::delegator_state(6).unwrap().delegations.0.len(),
				3usize
			);
			assert_eq!(Balances::reserved_balance(&6), 30);
			assert_eq!(Balances::reserved_balance(&7), 10);
			assert_eq!(Balances::free_balance(&6), 70);
			assert_eq!(Balances::free_balance(&7), 90);
		});
}

#[test]
fn payouts_follow_delegation_changes() {
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
		.with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_delegations(vec![
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
				Event::CollatorChosen(2, 3, 20),
				Event::CollatorChosen(2, 4, 20),
				Event::CollatorChosen(2, 5, 10),
				Event::NewRound(5, 2, 5, 140),
			];
			assert_eq_events!(expected.clone());
			// ~ set block author as 1 for all blocks this round
			set_author(2, 1, 100);
			roll_to(16);
			// distribute total issuance to collator 1 and its delegators 6, 7, 19
			let mut new = vec![
				Event::CollatorChosen(3, 1, 50),
				Event::CollatorChosen(3, 2, 40),
				Event::CollatorChosen(3, 3, 20),
				Event::CollatorChosen(3, 4, 20),
				Event::CollatorChosen(3, 5, 10),
				Event::NewRound(10, 3, 5, 140),
				Event::CollatorChosen(4, 1, 50),
				Event::CollatorChosen(4, 2, 40),
				Event::CollatorChosen(4, 3, 20),
				Event::CollatorChosen(4, 4, 20),
				Event::CollatorChosen(4, 5, 10),
				Event::NewRound(15, 4, 5, 140),
				Event::Rewarded(1, 26),
				Event::Rewarded(7, 8),
				Event::Rewarded(10, 8),
				Event::Rewarded(6, 8),
			];
			expected.append(&mut new);
			assert_eq_events!(expected.clone());
			// ~ set block author as 1 for all blocks this round
			set_author(3, 1, 100);
			set_author(4, 1, 100);
			set_author(5, 1, 100);
			set_author(6, 1, 100);
			// 1. ensure delegators are paid for 2 rounds after they leave
			assert_noop!(
				Stake::schedule_leave_delegators(Origin::signed(66)),
				Error::<Test>::DelegatorDNE
			);
			assert_ok!(Stake::schedule_leave_delegators(Origin::signed(6)));
			// fast forward to block in which delegator 6 exit executes
			roll_to(25);
			assert_ok!(Stake::execute_leave_delegators(Origin::signed(6), 6, 10));
			// keep paying 6 (note: inflation is in terms of total issuance so that's why 1 is 21)
			let mut new2 = vec![
				Event::DelegatorExitScheduled(4, 6, 6),
				Event::CollatorChosen(5, 1, 50),
				Event::CollatorChosen(5, 2, 40),
				Event::CollatorChosen(5, 3, 20),
				Event::CollatorChosen(5, 4, 20),
				Event::CollatorChosen(5, 5, 10),
				Event::NewRound(20, 5, 5, 140),
				Event::Rewarded(1, 27),
				Event::Rewarded(7, 8),
				Event::Rewarded(10, 8),
				Event::Rewarded(6, 8),
				Event::CollatorChosen(6, 1, 50),
				Event::CollatorChosen(6, 2, 40),
				Event::CollatorChosen(6, 3, 20),
				Event::CollatorChosen(6, 4, 20),
				Event::CollatorChosen(6, 5, 10),
				Event::NewRound(25, 6, 5, 140),
				Event::Rewarded(1, 29),
				Event::Rewarded(7, 9),
				Event::Rewarded(10, 9),
				Event::Rewarded(6, 9),
				Event::DelegatorLeftCandidate(6, 1, 10, 40),
				Event::DelegatorLeft(6, 10),
			];
			expected.append(&mut new2);
			assert_eq_events!(expected.clone());
			// 6 won't be paid for this round because they left already
			set_author(7, 1, 100);
			roll_to(35);
			// keep paying 6
			let mut new3 = vec![
				Event::CollatorChosen(7, 1, 40),
				Event::CollatorChosen(7, 2, 40),
				Event::CollatorChosen(7, 3, 20),
				Event::CollatorChosen(7, 4, 20),
				Event::CollatorChosen(7, 5, 10),
				Event::NewRound(30, 7, 5, 130),
				Event::Rewarded(1, 30),
				Event::Rewarded(7, 9),
				Event::Rewarded(10, 9),
				Event::Rewarded(6, 9),
				Event::CollatorChosen(8, 1, 40),
				Event::CollatorChosen(8, 2, 40),
				Event::CollatorChosen(8, 3, 20),
				Event::CollatorChosen(8, 4, 20),
				Event::CollatorChosen(8, 5, 10),
				Event::NewRound(35, 8, 5, 130),
				Event::Rewarded(1, 32),
				Event::Rewarded(7, 10),
				Event::Rewarded(10, 10),
				Event::Rewarded(6, 10),
			];
			expected.append(&mut new3);
			assert_eq_events!(expected.clone());
			set_author(8, 1, 100);
			roll_to(40);
			// no more paying 6
			let mut new4 = vec![
				Event::CollatorChosen(9, 1, 40),
				Event::CollatorChosen(9, 2, 40),
				Event::CollatorChosen(9, 3, 20),
				Event::CollatorChosen(9, 4, 20),
				Event::CollatorChosen(9, 5, 10),
				Event::NewRound(40, 9, 5, 130),
				Event::Rewarded(1, 38),
				Event::Rewarded(7, 13),
				Event::Rewarded(10, 13),
			];
			expected.append(&mut new4);
			assert_eq_events!(expected.clone());
			set_author(9, 1, 100);
			assert_ok!(Stake::delegate(Origin::signed(8), 1, 10, 10, 10));
			roll_to(45);
			// new delegation is not rewarded yet
			let mut new5 = vec![
				Event::Delegation(8, 10, 1, DelegatorAdded::AddedToTop { new_total: 50 }),
				Event::CollatorChosen(10, 1, 50),
				Event::CollatorChosen(10, 2, 40),
				Event::CollatorChosen(10, 3, 20),
				Event::CollatorChosen(10, 4, 20),
				Event::CollatorChosen(10, 5, 10),
				Event::NewRound(45, 10, 5, 140),
				Event::Rewarded(1, 40),
				Event::Rewarded(7, 13),
				Event::Rewarded(10, 13),
			];
			expected.append(&mut new5);
			assert_eq_events!(expected.clone());
			set_author(10, 1, 100);
			roll_to(50);
			// new delegation not rewarded yet
			let mut new6 = vec![
				Event::CollatorChosen(11, 1, 50),
				Event::CollatorChosen(11, 2, 40),
				Event::CollatorChosen(11, 3, 20),
				Event::CollatorChosen(11, 4, 20),
				Event::CollatorChosen(11, 5, 10),
				Event::NewRound(50, 11, 5, 140),
				Event::Rewarded(1, 42),
				Event::Rewarded(7, 14),
				Event::Rewarded(10, 14),
			];
			expected.append(&mut new6);
			assert_eq_events!(expected.clone());
			roll_to(55);
			// new delegation is rewarded for first time
			// 2 rounds after joining (`RewardPaymentDelay` = 2)
			let mut new7 = vec![
				Event::CollatorChosen(12, 1, 50),
				Event::CollatorChosen(12, 2, 40),
				Event::CollatorChosen(12, 3, 20),
				Event::CollatorChosen(12, 4, 20),
				Event::CollatorChosen(12, 5, 10),
				Event::NewRound(55, 12, 5, 140),
				Event::Rewarded(1, 39),
				Event::Rewarded(7, 12),
				Event::Rewarded(8, 12),
				Event::Rewarded(10, 12),
			];
			expected.append(&mut new7);
			assert_eq_events!(expected);
		});
}

#[test]
// MaxDelegatorsPerCandidate = 4
fn bottom_delegations_are_empty_when_top_delegations_not_full() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 10), (3, 10), (4, 10), (5, 10)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			// no top delegators => no bottom delegators
			let collator_state = Stake::candidate_state(1).unwrap();
			assert!(collator_state.top_delegations.is_empty());
			assert!(collator_state.bottom_delegations.is_empty());
			// 1 delegator => 1 top delegator, 0 bottom delegators
			assert_ok!(Stake::delegate(Origin::signed(2), 1, 10, 10, 10));
			let collator_state = Stake::candidate_state(1).unwrap();
			assert_eq!(collator_state.top_delegations.len(), 1usize);
			assert!(collator_state.bottom_delegations.is_empty());
			// 2 delegators => 2 top delegators, 0 bottom delegators
			assert_ok!(Stake::delegate(Origin::signed(3), 1, 10, 10, 10));
			let collator_state = Stake::candidate_state(1).unwrap();
			assert_eq!(collator_state.top_delegations.len(), 2usize);
			assert!(collator_state.bottom_delegations.is_empty());
			// 3 delegators => 3 top delegators, 0 bottom delegators
			assert_ok!(Stake::delegate(Origin::signed(4), 1, 10, 10, 10));
			let collator_state = Stake::candidate_state(1).unwrap();
			assert_eq!(collator_state.top_delegations.len(), 3usize);
			assert!(collator_state.bottom_delegations.is_empty());
			// 4 delegators => 4 top delegators, 0 bottom delegators
			assert_ok!(Stake::delegate(Origin::signed(5), 1, 10, 10, 10));
			let collator_state = Stake::candidate_state(1).unwrap();
			assert_eq!(collator_state.top_delegations.len(), 4usize);
			assert!(collator_state.bottom_delegations.is_empty());
		});
}

#[test]
// MaxDelegatorsPerCandidate = 4
fn candidate_pool_updates_when_total_counted_changes() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 20),
			(3, 19),
			(4, 20),
			(5, 21),
			(6, 22),
			(7, 15),
			(8, 16),
			(9, 17),
			(10, 18),
		])
		.with_candidates(vec![(1, 20)])
		.with_delegations(vec![
			(3, 1, 11),
			(4, 1, 12),
			(5, 1, 13),
			(6, 1, 14),
			(7, 1, 15),
			(8, 1, 16),
			(9, 1, 17),
			(10, 1, 18),
		])
		.build()
		.execute_with(|| {
			fn is_candidate_pool_bond(account: u64, bond: u128) {
				let pool = Stake::candidate_pool();
				for candidate in pool.0 {
					if candidate.owner == account {
						assert_eq!(candidate.amount, bond);
					}
				}
			}
			// 15 + 16 + 17 + 18 + 20 = 86 (top 4 + self bond)
			is_candidate_pool_bond(1, 86);
			assert_ok!(Stake::delegator_bond_more(Origin::signed(3), 1, 8));
			// 3: 11 -> 19 => 3 is in top, bumps out 7
			// 16 + 17 + 18 + 19 + 20 = 90 (top 4 + self bond)
			is_candidate_pool_bond(1, 90);
			assert_ok!(Stake::delegator_bond_more(Origin::signed(4), 1, 8));
			// 4: 12 -> 20 => 4 is in top, bumps out 8
			// 17 + 18 + 19 + 20 + 20 = 94 (top 4 + self bond)
			is_candidate_pool_bond(1, 94);
			assert_ok!(Stake::schedule_delegator_bond_less(
				Origin::signed(10),
				1,
				3
			));
			roll_to(30);
			// 10: 18 -> 15 => 10 bumped to bottom, 8 bumped to top (- 18 + 16 = -2 for count)
			assert_ok!(Stake::execute_delegation_request(Origin::signed(10), 10, 1));
			// 16 + 17 + 19 + 20 + 20 = 92 (top 4 + self bond)
			is_candidate_pool_bond(1, 92);
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(9), 1, 4));
			roll_to(40);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(9), 9, 1));
			// 15 + 16 + 19 + 20 + 20 = 90 (top 4 + self bond)
			is_candidate_pool_bond(1, 90);
		});
}

#[test]
// MaxDelegatorsPerCandidate = 4
fn only_top_collators_are_counted() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 20),
			(3, 19),
			(4, 20),
			(5, 21),
			(6, 22),
			(7, 15),
			(8, 16),
			(9, 17),
			(10, 18),
		])
		.with_candidates(vec![(1, 20)])
		.with_delegations(vec![
			(3, 1, 11),
			(4, 1, 12),
			(5, 1, 13),
			(6, 1, 14),
			(7, 1, 15),
			(8, 1, 16),
			(9, 1, 17),
			(10, 1, 18),
		])
		.build()
		.execute_with(|| {
			// sanity check that 3-10 are delegators immediately
			for i in 3..11 {
				assert!(Stake::is_delegator(&i));
			}
			let collator_state = Stake::candidate_state(1).unwrap();
			// 15 + 16 + 17 + 18 + 20 = 86 (top 4 + self bond)
			assert_eq!(collator_state.total_counted, 86);
			// 11 + 12 + 13 + 14 = 50
			assert_eq!(
				collator_state.total_counted + 50,
				collator_state.total_backing
			);
			// bump bottom to the top
			assert_ok!(Stake::delegator_bond_more(Origin::signed(3), 1, 8));
			assert_event_emitted!(Event::DelegationIncreased(3, 1, 8, true));
			let collator_state = Stake::candidate_state(1).unwrap();
			// 16 + 17 + 18 + 19 + 20 = 90 (top 4 + self bond)
			assert_eq!(collator_state.total_counted, 90);
			// 12 + 13 + 14 + 15 = 54
			assert_eq!(
				collator_state.total_counted + 54,
				collator_state.total_backing
			);
			// bump bottom to the top
			assert_ok!(Stake::delegator_bond_more(Origin::signed(4), 1, 8));
			assert_event_emitted!(Event::DelegationIncreased(4, 1, 8, true));
			let collator_state = Stake::candidate_state(1).unwrap();
			// 17 + 18 + 19 + 20 + 20 = 94 (top 4 + self bond)
			assert_eq!(collator_state.total_counted, 94);
			// 13 + 14 + 15 + 16 = 58
			assert_eq!(
				collator_state.total_counted + 58,
				collator_state.total_backing
			);
			// bump bottom to the top
			assert_ok!(Stake::delegator_bond_more(Origin::signed(5), 1, 8));
			assert_event_emitted!(Event::DelegationIncreased(5, 1, 8, true));
			let collator_state = Stake::candidate_state(1).unwrap();
			// 18 + 19 + 20 + 21 + 20 = 98 (top 4 + self bond)
			assert_eq!(collator_state.total_counted, 98);
			// 14 + 15 + 16 + 17 = 62
			assert_eq!(
				collator_state.total_counted + 62,
				collator_state.total_backing
			);
			// bump bottom to the top
			assert_ok!(Stake::delegator_bond_more(Origin::signed(6), 1, 8));
			assert_event_emitted!(Event::DelegationIncreased(6, 1, 8, true));
			let collator_state = Stake::candidate_state(1).unwrap();
			// 19 + 20 + 21 + 22 + 20 = 102 (top 4 + self bond)
			assert_eq!(collator_state.total_counted, 102);
			// 15 + 16 + 17 + 18 = 66
			assert_eq!(
				collator_state.total_counted + 66,
				collator_state.total_backing
			);
		});
}

#[test]
fn delegation_events_convey_correct_position() {
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
		.with_candidates(vec![(1, 20), (2, 20)])
		.with_delegations(vec![(3, 1, 11), (4, 1, 12), (5, 1, 13), (6, 1, 14)])
		.build()
		.execute_with(|| {
			let collator1_state = Stake::candidate_state(1).unwrap();
			// 11 + 12 + 13 + 14 + 20 = 70 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 70);
			assert_eq!(collator1_state.total_counted, collator1_state.total_backing);
			// Top delegations are full, new highest delegation is made
			assert_ok!(Stake::delegate(Origin::signed(7), 1, 15, 10, 10));
			assert_event_emitted!(Event::Delegation(
				7,
				15,
				1,
				DelegatorAdded::AddedToTop { new_total: 74 },
			));
			let collator1_state = Stake::candidate_state(1).unwrap();
			// 12 + 13 + 14 + 15 + 20 = 70 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 74);
			// 11 = 11
			assert_eq!(
				collator1_state.total_counted + 11,
				collator1_state.total_backing
			);
			// New delegation is added to the bottom
			assert_ok!(Stake::delegate(Origin::signed(8), 1, 10, 10, 10));
			assert_event_emitted!(Event::Delegation(8, 10, 1, DelegatorAdded::AddedToBottom));
			let collator1_state = Stake::candidate_state(1).unwrap();
			// 12 + 13 + 14 + 15 + 20 = 70 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 74);
			// 10 + 11 = 21
			assert_eq!(
				collator1_state.total_counted + 21,
				collator1_state.total_backing
			);
			// 8 increases delegation to the top
			assert_ok!(Stake::delegator_bond_more(Origin::signed(8), 1, 3));
			assert_event_emitted!(Event::DelegationIncreased(8, 1, 3, true));
			let collator1_state = Stake::candidate_state(1).unwrap();
			// 13 + 13 + 14 + 15 + 20 = 75 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 75);
			// 11 + 12 = 23
			assert_eq!(
				collator1_state.total_counted + 23,
				collator1_state.total_backing
			);
			// 3 increases delegation but stays in bottom
			assert_ok!(Stake::delegator_bond_more(Origin::signed(3), 1, 1));
			assert_event_emitted!(Event::DelegationIncreased(3, 1, 1, false));
			let collator1_state = Stake::candidate_state(1).unwrap();
			// 13 + 13 + 14 + 15 + 20 = 75 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 75);
			// 12 + 12 = 24
			assert_eq!(
				collator1_state.total_counted + 24,
				collator1_state.total_backing
			);
			// 6 decreases delegation but stays in top
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(6), 1, 2));
			assert_event_emitted!(Event::DelegationDecreaseScheduled(6, 1, 2, 3));
			roll_to(30);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(6), 6, 1));
			assert_event_emitted!(Event::DelegationDecreased(6, 1, 2, true));
			let collator1_state = Stake::candidate_state(1).unwrap();
			// 12 + 13 + 13 + 15 + 20 = 73 (top 4 + self bond)ƒ
			assert_eq!(collator1_state.total_counted, 73);
			// 12 + 12 = 24
			assert_eq!(
				collator1_state.total_counted + 24,
				collator1_state.total_backing
			);
			// 6 decreases delegation and is bumped to bottom
			assert_ok!(Stake::schedule_delegator_bond_less(Origin::signed(6), 1, 1));
			assert_event_emitted!(Event::DelegationDecreaseScheduled(6, 1, 1, 9));
			roll_to(40);
			assert_ok!(Stake::execute_delegation_request(Origin::signed(6), 6, 1));
			assert_event_emitted!(Event::DelegationDecreased(6, 1, 1, false));
			let collator1_state = Stake::candidate_state(1).unwrap();
			// 12 + 13 + 13 + 15 + 20 = 73 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 73);
			// 11 + 12 = 23
			assert_eq!(
				collator1_state.total_counted + 23,
				collator1_state.total_backing
			);
		});
}

#[test]
fn no_rewards_paid_until_after_reward_payment_delay() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20), (3, 20), (4, 20)])
		.with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20)])
		.build()
		.execute_with(|| {
			roll_to_round_begin(2);
			// payouts for round 1
			set_author(1, 1, 1);
			set_author(1, 2, 1);
			set_author(1, 3, 1);
			set_author(1, 4, 1);
			set_author(1, 4, 1);
			let mut expected = vec![
				Event::CollatorChosen(2, 1, 20),
				Event::CollatorChosen(2, 2, 20),
				Event::CollatorChosen(2, 3, 20),
				Event::CollatorChosen(2, 4, 20),
				Event::NewRound(5, 2, 4, 80),
			];
			assert_eq_events!(expected);

			roll_to_round_begin(3);
			expected.append(&mut vec![
				Event::CollatorChosen(3, 1, 20),
				Event::CollatorChosen(3, 2, 20),
				Event::CollatorChosen(3, 3, 20),
				Event::CollatorChosen(3, 4, 20),
				Event::NewRound(10, 3, 4, 80),
				// rewards will begin immediately following a NewRound
				Event::Rewarded(3, 1),
			]);
			assert_eq_events!(expected);

			// roll to the next block where we start round 3; we should have round change and first
			// payout made.
			roll_one_block();
			expected.push(Event::Rewarded(4, 2));
			assert_eq_events!(expected);

			roll_one_block();
			expected.push(Event::Rewarded(1, 1));
			assert_eq_events!(expected);

			roll_one_block();
			expected.push(Event::Rewarded(2, 1));
			assert_eq_events!(expected);

			// there should be no more payments in this round...
			let num_blocks_rolled = roll_to_round_end(3);
			assert_eq_events!(expected);
			assert_eq!(num_blocks_rolled, 1);
		});
}

#[test]
fn deferred_payment_storage_items_are_cleaned_up() {
	use crate::*;

	// this test sets up two collators, gives them points in round one, and focuses on the
	// storage over the next several blocks to show that it is properly cleaned up

	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20)])
		.with_candidates(vec![(1, 20), (2, 20)])
		.build()
		.execute_with(|| {
			let mut round: u32 = 1;
			set_author(round, 1, 1);
			set_author(round, 2, 1);

			// reflects genesis?
			assert!(<AtStake<Test>>::contains_key(round, 1));
			assert!(<AtStake<Test>>::contains_key(round, 2));

			round = 2;
			roll_to_round_begin(round.into());
			let mut expected = vec![
				Event::CollatorChosen(round, 1, 20),
				Event::CollatorChosen(round, 2, 20),
				Event::NewRound(5, round, 2, 40),
			];
			assert_eq_events!(expected);

			// we should have AtStake snapshots as soon as we start a round...
			assert!(<AtStake<Test>>::contains_key(2, 1));
			assert!(<AtStake<Test>>::contains_key(2, 2));
			// ...and it should persist until the round is fully paid out
			assert!(<AtStake<Test>>::contains_key(1, 1));
			assert!(<AtStake<Test>>::contains_key(1, 2));

			assert!(
				!<DelayedPayouts<Test>>::contains_key(1),
				"DelayedPayouts shouldn't be populated until after RewardPaymentDelay"
			);
			assert!(
				<Points<Test>>::contains_key(1),
				"Points should be populated during current round"
			);
			assert!(
				<Staked<Test>>::contains_key(1),
				"Staked should be populated when round changes"
			);

			assert!(
				!<Points<Test>>::contains_key(2),
				"Points should not be populated until author noted"
			);
			assert!(
				<Staked<Test>>::contains_key(2),
				"Staked should be populated when round changes"
			);

			// first payout occurs in round 3
			round = 3;
			roll_to_round_begin(round.into());
			expected.append(&mut vec![
				Event::CollatorChosen(round, 1, 20),
				Event::CollatorChosen(round, 2, 20),
				Event::NewRound(10, round, 2, 40),
				Event::Rewarded(1, 1),
			]);
			assert_eq_events!(expected);

			// payouts should exist for past rounds that haven't been paid out yet..
			assert!(<AtStake<Test>>::contains_key(3, 1));
			assert!(<AtStake<Test>>::contains_key(3, 2));
			assert!(<AtStake<Test>>::contains_key(2, 1));
			assert!(<AtStake<Test>>::contains_key(2, 2));

			assert!(
				<DelayedPayouts<Test>>::contains_key(1),
				"DelayedPayouts should be populated after RewardPaymentDelay"
			);
			assert!(<Points<Test>>::contains_key(1));
			assert!(
				!<Staked<Test>>::contains_key(1),
				"Staked should be cleaned up after round change"
			);

			assert!(!<DelayedPayouts<Test>>::contains_key(2));
			assert!(
				!<Points<Test>>::contains_key(2),
				"We never rewarded points for round 2"
			);
			assert!(<Staked<Test>>::contains_key(2));

			assert!(!<DelayedPayouts<Test>>::contains_key(3));
			assert!(
				!<Points<Test>>::contains_key(3),
				"We never awarded points for round 3"
			);
			assert!(<Staked<Test>>::contains_key(3));

			// collator 1 has been paid in this last block and associated storage cleaned up
			assert!(!<AtStake<Test>>::contains_key(1, 1));
			assert!(!<AwardedPts<Test>>::contains_key(1, 1));

			// but collator 2 hasn't been paid
			assert!(<AtStake<Test>>::contains_key(1, 2));
			assert!(<AwardedPts<Test>>::contains_key(1, 2));

			round = 4;
			roll_to_round_begin(round.into());
			expected.append(&mut vec![
				Event::Rewarded(2, 1), // from previous round
				Event::CollatorChosen(round, 1, 20),
				Event::CollatorChosen(round, 2, 20),
				Event::NewRound(15, round, 2, 40),
			]);
			assert_eq_events!(expected);

			// collators have both been paid and storage fully cleaned up for round 1
			assert!(!<AtStake<Test>>::contains_key(1, 2));
			assert!(!<AwardedPts<Test>>::contains_key(1, 2));
			assert!(!<Staked<Test>>::contains_key(1));
			assert!(!<Points<Test>>::contains_key(1)); // points should be cleaned up
			assert!(!<DelayedPayouts<Test>>::contains_key(1));

			roll_to_round_end(4);

			// no more events expected
			assert_eq_events!(expected);
		});
}

#[test]
fn deferred_payment_steady_state_event_flow() {
	use frame_support::traits::{Currency, ExistenceRequirement, WithdrawReasons};

	// this test "flows" through a number of rounds, asserting that certain things do/don't happen
	// once the staking pallet is in a "steady state" (specifically, once we are past the first few
	// rounds to clear RewardPaymentDelay)

	ExtBuilder::default()
		.with_balances(vec![
			// collators
			(1, 200),
			(2, 200),
			(3, 200),
			(4, 200),
			// delegators
			(11, 200),
			(22, 200),
			(33, 200),
			(44, 200),
			// burn account, see `reset_issuance()`
			(111, 1000),
		])
		.with_candidates(vec![(1, 200), (2, 200), (3, 200), (4, 200)])
		.with_delegations(vec![
			// delegator 11 delegates 100 to 1 and 2
			(11, 1, 100),
			(11, 2, 100),
			// delegator 22 delegates 100 to 2 and 3
			(22, 2, 100),
			(22, 3, 100),
			// delegator 33 delegates 100 to 3 and 4
			(33, 3, 100),
			(33, 4, 100),
			// delegator 44 delegates 100 to 4 and 1
			(44, 4, 100),
			(44, 1, 100),
		])
		.build()
		.execute_with(|| {
			// convenience to set the round points consistently
			let set_round_points = |round: u64| {
				set_author(round as u32, 1, 1);
				set_author(round as u32, 2, 1);
				set_author(round as u32, 3, 1);
				set_author(round as u32, 4, 1);
			};

			// grab initial issuance -- we will reset it before round issuance is calculated so that
			// it is consistent every round
			let initial_issuance = Balances::total_issuance();
			let reset_issuance = || {
				let new_issuance = Balances::total_issuance();
				let diff = new_issuance - initial_issuance;
				let burned = Balances::burn(diff);
				Balances::settle(
					&111,
					burned,
					WithdrawReasons::FEE,
					ExistenceRequirement::AllowDeath,
				)
				.expect("Account can absorb burn");
			};

			// fn to roll through the first RewardPaymentDelay rounds. returns new round index
			let roll_through_initial_rounds = |mut round: u64| -> u64 {
				while round < crate::mock::RewardPaymentDelay::get() as u64 + 1 {
					set_round_points(round);

					roll_to_round_end(round);
					round += 1;
				}

				reset_issuance();

				round
			};

			// roll through a "steady state" round and make all of our assertions
			// returns new round index
			let roll_through_steady_state_round = |round: u64| -> u64 {
				let num_rounds_rolled = roll_to_round_begin(round);
				assert_eq!(
					num_rounds_rolled, 1,
					"expected to be at round begin already"
				);

				let expected = vec![
					Event::CollatorChosen(round as u32, 1, 400),
					Event::CollatorChosen(round as u32, 2, 400),
					Event::CollatorChosen(round as u32, 3, 400),
					Event::CollatorChosen(round as u32, 4, 400),
					Event::NewRound((round - 1) * 5, round as u32, 4, 1600),
					// first payout should occur on round change
					Event::Rewarded(3, 19),
					Event::Rewarded(33, 6),
					Event::Rewarded(22, 6),
				];
				assert_eq_last_events!(expected);

				set_round_points(round);

				roll_one_block();
				let expected = vec![
					Event::Rewarded(4, 19),
					Event::Rewarded(44, 6),
					Event::Rewarded(33, 6),
				];
				assert_eq_last_events!(expected);

				roll_one_block();
				let expected = vec![
					Event::Rewarded(1, 19),
					Event::Rewarded(44, 6),
					Event::Rewarded(11, 6),
				];
				assert_eq_last_events!(expected);

				roll_one_block();
				let expected = vec![
					Event::Rewarded(2, 19),
					Event::Rewarded(22, 6),
					Event::Rewarded(11, 6),
				];
				assert_eq_last_events!(expected);

				roll_one_block();
				let expected = vec![
					// we paid everyone out by now, should repeat last event
					Event::Rewarded(11, 6),
				];
				assert_eq_last_events!(expected);

				let num_rounds_rolled = roll_to_round_end(round);
				assert_eq!(num_rounds_rolled, 0, "expected to be at round end already");

				reset_issuance();

				round + 1
			};

			let mut round = 1;
			round = roll_through_initial_rounds(round); // we should be at RewardPaymentDelay
			for _ in 1..5 {
				round = roll_through_steady_state_round(round);
			}
		});
}

// MIGRATION UNIT TESTS
use frame_support::traits::OnRuntimeUpgrade;

#[test]
fn increase_delegations_per_candidate_migrates_bottom_delegations() {
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
		])
		.with_candidates(vec![(1, 20)])
		.with_delegations(vec![
			(2, 1, 19),
			(3, 1, 20),
			(4, 1, 21),
			(5, 1, 22),
			(6, 1, 23),
			(7, 1, 24),
			(8, 1, 25),
		])
		.build()
		.execute_with(|| {
			// start by corrupting collator state like the bug -- have some in bottom with open
			// slots in the top
			let mut candidate_state =
				<CandidateState<Test>>::get(&1).expect("set up 1 as candidate");
			// corrupt storage via unhandled pop
			candidate_state.top_delegations.pop();
			candidate_state.top_delegations.pop();
			assert_eq!(candidate_state.top_delegations.len(), 2); // < MaxNominatorsPerCollator = 4
			assert_eq!(candidate_state.bottom_delegations.len(), 3);
			<CandidateState<Test>>::insert(&1, candidate_state);
			// full migration, first cleans delegator set and second cleans other items
			crate::migrations::IncreaseMaxDelegationsPerCandidate::<Test>::on_runtime_upgrade();
			let post_candidate_state =
				<CandidateState<Test>>::get(&1).expect("set up 1 as candidate");
			assert_eq!(post_candidate_state.top_delegations.len(), 4);
			assert_eq!(post_candidate_state.bottom_delegations.len(), 1);
		});
}

#[test]
fn remove_exit_queue_migration_migrates_leaving_candidates() {
	use crate::pallet::ExitQueue2;
	use crate::set::*;
	use crate::*;
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.build()
		.execute_with(|| {
			// prepare leaving state for all 5 candidates before the migration
			for i in 1..6 {
				// manually change the CollatorState2 status
				<CollatorState2<Test>>::insert(
					i,
					Collator2 {
						id: i,
						bond: 20,
						nominators: OrderedSet::new(),
						top_nominators: Vec::new(),
						bottom_nominators: Vec::new(),
						total_counted: 20,
						total_backing: 20,
						// set to leaving
						state: CollatorStatus::Leaving(3),
					},
				);
			}
			<ExitQueue2<Test>>::put(ExitQ {
				candidates: OrderedSet(vec![1, 2, 3, 4, 5]),
				candidate_schedule: vec![(1, 3), (2, 3), (3, 3), (4, 3), (5, 3)],
				..Default::default()
			});
			// execute migration
			migrations::RemoveExitQueue::<Test>::on_runtime_upgrade();
			// check expected candidate state reflects previous state
			for i in 1..6 {
				assert!(<CollatorState2<Test>>::get(i).is_none());
				assert_eq!(
					<CandidateState<Test>>::get(i).unwrap().state,
					CollatorStatus::Leaving(3)
				);
			}
			// exit queue should be empty
			assert_eq!(<ExitQueue2<Test>>::get(), ExitQ::default());
		});
}

#[test]
fn remove_exit_queue_migration_migrates_leaving_delegators() {
	use crate::pallet::ExitQueue2;
	use crate::set::*;
	use crate::*;
	ExtBuilder::default()
		.with_balances(vec![(2, 100), (3, 100), (4, 100), (5, 100), (6, 100)])
		.with_candidates(vec![(2, 20)])
		.with_delegations(vec![(3, 1, 10), (4, 1, 10), (5, 1, 10), (6, 1, 10)])
		.build()
		.execute_with(|| {
			// prepare leaving state for all 4 delegators before the migration
			for i in 3..7 {
				<NominatorState2<Test>>::insert(
					i,
					Nominator2 {
						delegations: OrderedSet(vec![Bond {
							owner: 1,
							amount: 10,
						}]),
						revocations: OrderedSet::new(),
						total: 10,
						scheduled_revocations_count: 0u32,
						scheduled_revocations_total: 0u32.into(),
						status: DelegatorStatus::Leaving(3),
					},
				);
			}
			<ExitQueue2<Test>>::put(ExitQ {
				nominators_leaving: OrderedSet(vec![3, 4, 5, 6]),
				nominator_schedule: vec![(3, None, 3), (4, None, 3), (5, None, 3), (6, None, 3)],
				..Default::default()
			});
			// execute migration
			migrations::RemoveExitQueue::<Test>::on_runtime_upgrade();
			// check expected delegator state reflects previous state
			for i in 3..7 {
				assert!(<NominatorState2<Test>>::get(i).is_none());
				assert_eq!(
					<DelegatorState<Test>>::get(i).unwrap().status,
					DelegatorStatus::Leaving(3)
				);
			}
			// exit queue should be empty
			assert_eq!(<ExitQueue2<Test>>::get(), ExitQ::default());
		});
}

#[test]
fn remove_exit_queue_migration_migrates_delegator_revocations() {
	use crate::pallet::ExitQueue2;
	use crate::set::*;
	use crate::*;
	ExtBuilder::default()
		.with_balances(vec![(2, 100), (3, 100), (4, 100), (5, 100), (6, 100)])
		.with_candidates(vec![(2, 20)])
		.with_delegations(vec![(3, 1, 10), (4, 1, 10), (5, 1, 10), (6, 1, 10)])
		.build()
		.execute_with(|| {
			// prepare leaving state for all 4 delegators before the migration
			for i in 3..7 {
				<NominatorState2<Test>>::insert(
					i,
					Nominator2 {
						delegations: OrderedSet(vec![Bond {
							owner: 1,
							amount: 10,
						}]),
						revocations: OrderedSet(vec![1]),
						total: 10,
						scheduled_revocations_count: 1u32,
						scheduled_revocations_total: 10u32.into(),
						status: DelegatorStatus::Active,
					},
				);
			}
			<ExitQueue2<Test>>::put(ExitQ {
				nominator_schedule: vec![
					(3, Some(1), 3),
					(4, Some(1), 3),
					(5, Some(1), 3),
					(6, Some(1), 3),
				],
				..Default::default()
			});
			// execute migration
			migrations::RemoveExitQueue::<Test>::on_runtime_upgrade();
			// check expected delegator state reflects previous state
			for i in 3..7 {
				assert!(<NominatorState2<Test>>::get(i).is_none());
				assert_eq!(
					<DelegatorState<Test>>::get(i)
						.unwrap()
						.requests
						.requests
						.get(&1),
					Some(&DelegationRequest {
						collator: 1,
						amount: 10,
						when_executable: 3,
						action: DelegationChange::Revoke
					})
				);
			}
			// exit queue should be empty
			assert_eq!(<ExitQueue2<Test>>::get(), ExitQ::default());
		});
}

#[test]
fn verify_purge_storage_migration_works() {
	use crate::{Points, Round, RoundInfo, Staked};
	ExtBuilder::default().build().execute_with(|| {
		// mutate storage similar to if 10 rounds had passed
		for i in 1..=10 {
			<Staked<Test>>::insert(i, 100);
			<Points<Test>>::insert(i, 100);
		}
		// set the round information to the 10th round
		// (we do not use roll_to because the payment logic uses `take` in the code)
		<Round<Test>>::put(RoundInfo {
			current: 10,
			first: 45,
			length: 5,
		});
		// execute the migration
		crate::migrations::PurgeStaleStorage::<Test>::on_runtime_upgrade();
		// verify that all inserted items are removed except last 2 rounds
		for i in 1..=8 {
			assert_eq!(<Staked<Test>>::get(i), 0);
			assert_eq!(<Points<Test>>::get(i), 0);
		}
		// last 2 rounds are still stored (necessary for future payouts)
		for i in 9..=10 {
			assert_eq!(<Staked<Test>>::get(i), 100);
			assert_eq!(<Points<Test>>::get(i), 100);
		}
	});
}
