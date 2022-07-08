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
use crate::mock::DefaultBlocksPerRound;
use crate::mock::{
	roll_one_block, roll_to, roll_to_round_begin, roll_to_round_end, set_author, Balances,
	Event as MetaEvent, ExtBuilder, Origin, ParachainStaking, Test,
};
use crate::{
	assert_eq_events, assert_eq_last_events, assert_event_emitted, assert_event_not_emitted,
	assert_last_event, assert_tail_eq, set::OrderedSet, AtStake, Bond, BottomDelegations,
	CandidateInfo, CandidateMetadata, CandidatePool, CandidateState, CapacityStatus,
	CollatorCandidate, CollatorStatus, Config, DelegationScheduledRequests, Delegations, Delegator,
	DelegatorAdded, DelegatorState, DelegatorStatus, Error, Event, Range, TopDelegations, Total,
};
use frame_support::traits::EstimateNextSessionRotation;
use frame_support::{assert_noop, assert_ok, traits::ReservableCurrency};
use pallet_session::{SessionManager, ShouldEndSession};
use sp_runtime::{traits::Zero, DispatchError, ModuleError, Perbill, Percent};

// ~~ ROOT ~~

#[test]
fn invalid_root_origin_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_total_selected(Origin::signed(45), 6u32),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_noop!(
			ParachainStaking::set_collator_commission(Origin::signed(45), Perbill::from_percent(5)),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_noop!(
			ParachainStaking::set_blocks_per_round(Origin::signed(45), 3u32),
			sp_runtime::DispatchError::BadOrigin
		);
	});
}

// SET TOTAL SELECTED

#[test]
fn set_total_selected_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		// before we can bump total_selected we must bump the blocks per round
		assert_ok!(ParachainStaking::set_blocks_per_round(Origin::root(), 6u32));
		assert_ok!(ParachainStaking::set_total_selected(Origin::root(), 6u32));
		assert_last_event!(MetaEvent::ParachainStaking(Event::TotalSelectedSet {
			old: 5u32,
			new: 6u32
		}));
	});
}

#[test]
fn set_total_selected_fails_if_above_blocks_per_round() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(ParachainStaking::round().length, 5); // test relies on this
		assert_noop!(
			ParachainStaking::set_total_selected(Origin::root(), 6u32),
			Error::<Test>::RoundLengthMustBeAtLeastTotalSelectedCollators,
		);
	});
}

#[test]
fn set_total_selected_passes_if_equal_to_blocks_per_round() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_blocks_per_round(
			Origin::root(),
			10u32
		));
		assert_ok!(ParachainStaking::set_total_selected(Origin::root(), 10u32));
	});
}

#[test]
fn set_total_selected_passes_if_below_blocks_per_round() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_blocks_per_round(
			Origin::root(),
			10u32
		));
		assert_ok!(ParachainStaking::set_total_selected(Origin::root(), 9u32));
	});
}

#[test]
fn set_blocks_per_round_fails_if_below_total_selected() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_blocks_per_round(
			Origin::root(),
			20u32
		));
		assert_ok!(ParachainStaking::set_total_selected(Origin::root(), 15u32));
		assert_noop!(
			ParachainStaking::set_blocks_per_round(Origin::root(), 14u32),
			Error::<Test>::RoundLengthMustBeAtLeastTotalSelectedCollators,
		);
	});
}

#[test]
fn set_blocks_per_round_passes_if_equal_to_total_selected() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_blocks_per_round(
			Origin::root(),
			10u32
		));
		assert_ok!(ParachainStaking::set_total_selected(Origin::root(), 9u32));
		assert_ok!(ParachainStaking::set_blocks_per_round(Origin::root(), 9u32));
	});
}

#[test]
fn set_blocks_per_round_passes_if_above_total_selected() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(ParachainStaking::round().length, 5); // test relies on this
		assert_ok!(ParachainStaking::set_blocks_per_round(Origin::root(), 6u32));
	});
}

#[test]
fn set_total_selected_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		// round length must be >= total_selected, so update that first
		assert_ok!(ParachainStaking::set_blocks_per_round(
			Origin::root(),
			10u32
		));

		assert_eq!(ParachainStaking::total_selected(), 5u32);
		assert_ok!(ParachainStaking::set_total_selected(Origin::root(), 6u32));
		assert_eq!(ParachainStaking::total_selected(), 6u32);
	});
}

#[test]
fn cannot_set_total_selected_to_current_total_selected() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_total_selected(Origin::root(), 5u32),
			Error::<Test>::NoWritingSameValue
		);
	});
}

#[test]
fn cannot_set_total_selected_below_module_min() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_total_selected(Origin::root(), 4u32),
			Error::<Test>::CannotSetBelowMin
		);
	});
}

// SET COLLATOR COMMISSION

#[test]
fn set_collator_commission_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_collator_commission(
			Origin::root(),
			Perbill::from_percent(5)
		));
		assert_last_event!(MetaEvent::ParachainStaking(Event::CollatorCommissionSet {
			old: Perbill::from_percent(20),
			new: Perbill::from_percent(5),
		}));
	});
}

#[test]
fn set_collator_commission_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			ParachainStaking::collator_commission(),
			Perbill::from_percent(20)
		);
		assert_ok!(ParachainStaking::set_collator_commission(
			Origin::root(),
			Perbill::from_percent(5)
		));
		assert_eq!(
			ParachainStaking::collator_commission(),
			Perbill::from_percent(5)
		);
	});
}

#[test]
fn cannot_set_collator_commission_to_current_collator_commission() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_collator_commission(Origin::root(), Perbill::from_percent(20)),
			Error::<Test>::NoWritingSameValue
		);
	});
}

// SET BLOCKS PER ROUND

#[test]
fn set_blocks_per_round_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_blocks_per_round(Origin::root(), 6u32));
		assert_last_event!(MetaEvent::ParachainStaking(Event::BlocksPerRoundSet {
			current_round: 1,
			first_block: 0,
			old: 5,
			new: 6,
			new_per_round_inflation_min: Perbill::from_parts(926),
			new_per_round_inflation_ideal: Perbill::from_parts(926),
			new_per_round_inflation_max: Perbill::from_parts(926),
		}));
	});
}

#[test]
fn set_blocks_per_round_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(ParachainStaking::round().length, 5);
		assert_ok!(ParachainStaking::set_blocks_per_round(Origin::root(), 6u32));
		assert_eq!(ParachainStaking::round().length, 6);
	});
}

#[test]
fn cannot_set_blocks_per_round_below_module_min() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_blocks_per_round(Origin::root(), 2u32),
			Error::<Test>::CannotSetBelowMin
		);
	});
}

#[test]
fn cannot_set_blocks_per_round_to_current_blocks_per_round() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_blocks_per_round(Origin::root(), 5u32),
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
			assert_ok!(ParachainStaking::set_blocks_per_round(
				Origin::root(),
				10u32
			));

			roll_to(17);
			assert_last_event!(MetaEvent::ParachainStaking(Event::NewRound {
				starting_block: 10,
				round: 2,
				selected_collators_number: 1,
				total_balance: 20
			}));
			assert_ok!(ParachainStaking::set_blocks_per_round(Origin::root(), 5u32));
			roll_to(18);
			assert_last_event!(MetaEvent::ParachainStaking(Event::NewRound {
				starting_block: 18,
				round: 3,
				selected_collators_number: 1,
				total_balance: 20
			}));
		});
}

// ~~ MONETARY GOVERNANCE ~~

#[test]
fn invalid_monetary_origin_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_staking_expectations(
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
			ParachainStaking::set_inflation(
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
			ParachainStaking::set_inflation(
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
			ParachainStaking::set_parachain_bond_account(Origin::signed(45), 11),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_noop!(
			ParachainStaking::set_parachain_bond_reserve_percent(
				Origin::signed(45),
				Percent::from_percent(2)
			),
			sp_runtime::DispatchError::BadOrigin
		);
	});
}

// SET STAKING EXPECTATIONS

#[test]
fn set_staking_event_emits_event_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		// valid call succeeds
		assert_ok!(ParachainStaking::set_staking_expectations(
			Origin::root(),
			Range {
				min: 3u128,
				ideal: 4u128,
				max: 5u128,
			}
		));
		assert_last_event!(MetaEvent::ParachainStaking(Event::StakeExpectationsSet {
			expect_min: 3u128,
			expect_ideal: 4u128,
			expect_max: 5u128,
		}));
	});
}

#[test]
fn set_staking_updates_storage_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			ParachainStaking::inflation_config().expect,
			Range {
				min: 700,
				ideal: 700,
				max: 700
			}
		);
		assert_ok!(ParachainStaking::set_staking_expectations(
			Origin::root(),
			Range {
				min: 3u128,
				ideal: 4u128,
				max: 5u128,
			}
		));
		assert_eq!(
			ParachainStaking::inflation_config().expect,
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
			ParachainStaking::set_staking_expectations(
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
		assert_ok!(ParachainStaking::set_staking_expectations(
			Origin::root(),
			Range {
				min: 3u128,
				ideal: 4u128,
				max: 5u128
			}
		));
		assert_noop!(
			ParachainStaking::set_staking_expectations(
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
		assert_ok!(ParachainStaking::set_inflation(
			Origin::root(),
			Range { min, ideal, max }
		));
		assert_last_event!(MetaEvent::ParachainStaking(Event::InflationSet {
			annual_min: min,
			annual_ideal: ideal,
			annual_max: max,
			round_min: Perbill::from_parts(57),
			round_ideal: Perbill::from_parts(75),
			round_max: Perbill::from_parts(93),
		}));
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
			ParachainStaking::inflation_config().annual,
			Range {
				min: Perbill::from_percent(50),
				ideal: Perbill::from_percent(50),
				max: Perbill::from_percent(50)
			}
		);
		assert_eq!(
			ParachainStaking::inflation_config().round,
			Range {
				min: Perbill::from_percent(5),
				ideal: Perbill::from_percent(5),
				max: Perbill::from_percent(5)
			}
		);
		assert_ok!(ParachainStaking::set_inflation(
			Origin::root(),
			Range { min, ideal, max }
		),);
		assert_eq!(
			ParachainStaking::inflation_config().annual,
			Range { min, ideal, max }
		);
		assert_eq!(
			ParachainStaking::inflation_config().round,
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
			ParachainStaking::set_inflation(
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
		assert_ok!(ParachainStaking::set_inflation(
			Origin::root(),
			Range { min, ideal, max }
		),);
		assert_noop!(
			ParachainStaking::set_inflation(Origin::root(), Range { min, ideal, max }),
			Error::<Test>::NoWritingSameValue
		);
	});
}

// SET PARACHAIN BOND ACCOUNT

#[test]
fn set_parachain_bond_account_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_parachain_bond_account(
			Origin::root(),
			11
		));
		assert_last_event!(MetaEvent::ParachainStaking(
			Event::ParachainBondAccountSet { old: 0, new: 11 }
		));
	});
}

#[test]
fn set_parachain_bond_account_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(ParachainStaking::parachain_bond_info().account, 0);
		assert_ok!(ParachainStaking::set_parachain_bond_account(
			Origin::root(),
			11
		));
		assert_eq!(ParachainStaking::parachain_bond_info().account, 11);
	});
}

// SET PARACHAIN BOND RESERVE PERCENT

#[test]
fn set_parachain_bond_reserve_percent_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_parachain_bond_reserve_percent(
			Origin::root(),
			Percent::from_percent(50)
		));
		assert_last_event!(MetaEvent::ParachainStaking(
			Event::ParachainBondReservePercentSet {
				old: Percent::from_percent(30),
				new: Percent::from_percent(50),
			}
		));
	});
}

#[test]
fn set_parachain_bond_reserve_percent_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			ParachainStaking::parachain_bond_info().percent,
			Percent::from_percent(30)
		);
		assert_ok!(ParachainStaking::set_parachain_bond_reserve_percent(
			Origin::root(),
			Percent::from_percent(50)
		));
		assert_eq!(
			ParachainStaking::parachain_bond_info().percent,
			Percent::from_percent(50)
		);
	});
}

#[test]
fn cannot_set_same_parachain_bond_reserve_percent() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_parachain_bond_reserve_percent(
				Origin::root(),
				Percent::from_percent(30)
			),
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
			assert_ok!(ParachainStaking::join_candidates(
				Origin::signed(1),
				10u128,
				0u32
			));
			assert_last_event!(MetaEvent::ParachainStaking(
				Event::JoinedCollatorCandidates {
					account: 1,
					amount_locked: 10u128,
					new_total_amt_locked: 10u128,
				}
			));
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
			assert_ok!(ParachainStaking::join_candidates(
				Origin::signed(1),
				10u128,
				0u32
			));
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
			assert_eq!(ParachainStaking::total(), 0);
			assert_ok!(ParachainStaking::join_candidates(
				Origin::signed(1),
				10u128,
				0u32
			));
			assert_eq!(ParachainStaking::total(), 10);
		});
}

#[test]
fn join_candidates_creates_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert!(ParachainStaking::candidate_info(1).is_none());
			assert_ok!(ParachainStaking::join_candidates(
				Origin::signed(1),
				10u128,
				0u32
			));
			let candidate_state =
				ParachainStaking::candidate_info(1).expect("just joined => exists");
			assert_eq!(candidate_state.bond, 10u128);
		});
}

#[test]
fn join_candidates_adds_to_candidate_pool() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert!(ParachainStaking::candidate_pool().0.is_empty());
			assert_ok!(ParachainStaking::join_candidates(
				Origin::signed(1),
				10u128,
				0u32
			));
			let candidate_pool = ParachainStaking::candidate_pool();
			assert_eq!(candidate_pool.0[0].owner, 1);
			assert_eq!(candidate_pool.0[0].amount, 10);
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
				ParachainStaking::join_candidates(Origin::signed(1), 11u128, 100u32),
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
				ParachainStaking::join_candidates(Origin::signed(2), 10u128, 1u32),
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
				ParachainStaking::join_candidates(Origin::signed(1), 9u128, 100u32),
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
				ParachainStaking::join_candidates(Origin::signed(1), 501u128, 100u32),
				DispatchError::Module(ModuleError {
					index: 1,
					error: [2, 0, 0, 0],
					message: Some("InsufficientBalance")
				})
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
					ParachainStaking::join_candidates(Origin::signed(6), 20, i),
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
				assert_ok!(ParachainStaking::join_candidates(
					Origin::signed(i),
					20,
					count
				));
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
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1u32
			));
			assert_last_event!(MetaEvent::ParachainStaking(Event::CandidateScheduledExit {
				exit_allowed_round: 1,
				candidate: 1,
				scheduled_exit: 3
			}));
		});
}

#[test]
fn leave_candidates_removes_candidate_from_candidate_pool() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(ParachainStaking::candidate_pool().0.len(), 1);
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1u32
			));
			assert!(ParachainStaking::candidate_pool().0.is_empty());
		});
}

#[test]
fn cannot_leave_candidates_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::schedule_leave_candidates(Origin::signed(1), 1u32),
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
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1u32
			));
			assert_noop!(
				ParachainStaking::schedule_leave_candidates(Origin::signed(1), 1u32),
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
					ParachainStaking::schedule_leave_candidates(Origin::signed(i), 4u32),
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
				assert_ok!(ParachainStaking::schedule_leave_candidates(
					Origin::signed(i),
					count
				));
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
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1u32
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				Origin::signed(1),
				1,
				0
			));
			assert_last_event!(MetaEvent::ParachainStaking(Event::CandidateLeft {
				ex_candidate: 1,
				unlocked_amount: 10,
				new_total_amt_locked: 0
			}));
		});
}

#[test]
fn execute_leave_candidates_callable_by_any_signed() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1u32
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				Origin::signed(2),
				1,
				0
			));
		});
}

#[test]
fn execute_leave_candidates_requires_correct_weight_hint() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10), (2, 10), (3, 10), (4, 10)])
		.with_candidates(vec![(1, 10)])
		.with_delegations(vec![(2, 1, 10), (3, 1, 10), (4, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1u32
			));
			roll_to(10);
			for i in 0..3 {
				assert_noop!(
					ParachainStaking::execute_leave_candidates(Origin::signed(1), 1, i),
					Error::<Test>::TooLowCandidateDelegationCountToLeaveCandidates
				);
			}
			assert_ok!(ParachainStaking::execute_leave_candidates(
				Origin::signed(2),
				1,
				3
			));
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
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1u32
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				Origin::signed(1),
				1,
				0
			));
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
			assert_eq!(ParachainStaking::total(), 10);
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1u32
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				Origin::signed(1),
				1,
				0
			));
			assert_eq!(ParachainStaking::total(), 0);
		});
}

#[test]
fn execute_leave_candidates_removes_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1u32
			));
			// candidate state is not immediately removed
			let candidate_state =
				ParachainStaking::candidate_info(1).expect("just left => still exists");
			assert_eq!(candidate_state.bond, 10u128);
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				Origin::signed(1),
				1,
				0
			));
			assert!(ParachainStaking::candidate_state(1).is_none());
		});
}

#[test]
fn execute_leave_candidates_removes_pending_delegation_requests() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10), (2, 15)])
		.with_candidates(vec![(1, 10)])
		.with_delegations(vec![(2, 1, 15)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				5
			));
			let state = ParachainStaking::delegation_scheduled_requests(&1);
			assert_eq!(
				state,
				vec![ScheduledRequest {
					delegator: 2,
					when_executable: 3,
					action: DelegationAction::Decrease(5),
				}],
			);
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1u32
			));
			// candidate state is not immediately removed
			let candidate_state =
				ParachainStaking::candidate_info(1).expect("just left => still exists");
			assert_eq!(candidate_state.bond, 10u128);
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				Origin::signed(1),
				1,
				1
			));
			assert!(ParachainStaking::candidate_state(1).is_none());
			assert!(
				!ParachainStaking::delegation_scheduled_requests(&1)
					.iter()
					.any(|x| x.delegator == 2),
				"delegation request not removed"
			);
			assert!(
				!<DelegationScheduledRequests<Test>>::contains_key(&1),
				"the key was not removed from storage"
			);
		});
}

#[test]
fn cannot_execute_leave_candidates_before_delay() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1u32
			));
			assert_noop!(
				ParachainStaking::execute_leave_candidates(Origin::signed(3), 1, 0),
				Error::<Test>::CandidateCannotLeaveYet
			);
			roll_to(9);
			assert_noop!(
				ParachainStaking::execute_leave_candidates(Origin::signed(3), 1, 0),
				Error::<Test>::CandidateCannotLeaveYet
			);
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				Origin::signed(3),
				1,
				0
			));
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
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1u32
			));
			assert_ok!(ParachainStaking::cancel_leave_candidates(
				Origin::signed(1),
				1
			));
			assert_last_event!(MetaEvent::ParachainStaking(Event::CancelledCandidateExit {
				candidate: 1
			}));
		});
}

#[test]
fn cancel_leave_candidates_updates_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_candidates(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1u32
			));
			assert_ok!(ParachainStaking::cancel_leave_candidates(
				Origin::signed(1),
				1
			));
			let candidate =
				ParachainStaking::candidate_info(&1).expect("just cancelled leave so exists");
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
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1u32
			));
			assert_ok!(ParachainStaking::cancel_leave_candidates(
				Origin::signed(1),
				1
			));
			assert_eq!(ParachainStaking::candidate_pool().0[0].owner, 1);
			assert_eq!(ParachainStaking::candidate_pool().0[0].amount, 10);
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
			assert_ok!(ParachainStaking::go_offline(Origin::signed(1)));
			assert_last_event!(MetaEvent::ParachainStaking(Event::CandidateWentOffline {
				candidate: 1
			}));
		});
}

#[test]
fn go_offline_removes_candidate_from_candidate_pool() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_eq!(ParachainStaking::candidate_pool().0.len(), 1);
			assert_ok!(ParachainStaking::go_offline(Origin::signed(1)));
			assert!(ParachainStaking::candidate_pool().0.is_empty());
		});
}

#[test]
fn go_offline_updates_candidate_state_to_idle() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			let candidate_state = ParachainStaking::candidate_info(1).expect("is active candidate");
			assert_eq!(candidate_state.status, CollatorStatus::Active);
			assert_ok!(ParachainStaking::go_offline(Origin::signed(1)));
			let candidate_state =
				ParachainStaking::candidate_info(1).expect("is candidate, just offline");
			assert_eq!(candidate_state.status, CollatorStatus::Idle);
		});
}

#[test]
fn cannot_go_offline_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::go_offline(Origin::signed(3)),
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
			assert_ok!(ParachainStaking::go_offline(Origin::signed(1)));
			assert_noop!(
				ParachainStaking::go_offline(Origin::signed(1)),
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
			assert_ok!(ParachainStaking::go_offline(Origin::signed(1)));
			assert_ok!(ParachainStaking::go_online(Origin::signed(1)));
			assert_last_event!(MetaEvent::ParachainStaking(Event::CandidateBackOnline {
				candidate: 1
			}));
		});
}

#[test]
fn go_online_adds_to_candidate_pool() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::go_offline(Origin::signed(1)));
			assert!(ParachainStaking::candidate_pool().0.is_empty());
			assert_ok!(ParachainStaking::go_online(Origin::signed(1)));
			assert_eq!(ParachainStaking::candidate_pool().0[0].owner, 1);
			assert_eq!(ParachainStaking::candidate_pool().0[0].amount, 20);
		});
}

#[test]
fn go_online_storage_updates_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::go_offline(Origin::signed(1)));
			let candidate_state =
				ParachainStaking::candidate_info(1).expect("offline still exists");
			assert_eq!(candidate_state.status, CollatorStatus::Idle);
			assert_ok!(ParachainStaking::go_online(Origin::signed(1)));
			let candidate_state = ParachainStaking::candidate_info(1).expect("online so exists");
			assert_eq!(candidate_state.status, CollatorStatus::Active);
		});
}

#[test]
fn cannot_go_online_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::go_online(Origin::signed(3)),
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
				ParachainStaking::go_online(Origin::signed(1)),
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
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1
			));
			assert_noop!(
				ParachainStaking::go_online(Origin::signed(1)),
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
			assert_ok!(ParachainStaking::candidate_bond_more(Origin::signed(1), 30));
			assert_last_event!(MetaEvent::ParachainStaking(Event::CandidateBondedMore {
				candidate: 1,
				amount: 30,
				new_total_bond: 50
			}));
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
			assert_ok!(ParachainStaking::candidate_bond_more(Origin::signed(1), 30));
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
			let mut total = ParachainStaking::total();
			assert_ok!(ParachainStaking::candidate_bond_more(Origin::signed(1), 30));
			total += 30;
			assert_eq!(ParachainStaking::total(), total);
		});
}

#[test]
fn candidate_bond_more_updates_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 50)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			let candidate_state = ParachainStaking::candidate_info(1).expect("updated => exists");
			assert_eq!(candidate_state.bond, 20);
			assert_ok!(ParachainStaking::candidate_bond_more(Origin::signed(1), 30));
			let candidate_state = ParachainStaking::candidate_info(1).expect("updated => exists");
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
			assert_eq!(ParachainStaking::candidate_pool().0[0].owner, 1);
			assert_eq!(ParachainStaking::candidate_pool().0[0].amount, 20);
			assert_ok!(ParachainStaking::candidate_bond_more(Origin::signed(1), 30));
			assert_eq!(ParachainStaking::candidate_pool().0[0].owner, 1);
			assert_eq!(ParachainStaking::candidate_pool().0[0].amount, 50);
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
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				Origin::signed(1),
				10
			));
			assert_last_event!(MetaEvent::ParachainStaking(
				Event::CandidateBondLessRequested {
					candidate: 1,
					amount_to_decrease: 10,
					execute_round: 3,
				}
			));
		});
}

#[test]
fn cannot_schedule_candidate_bond_less_if_request_exists() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				Origin::signed(1),
				5
			));
			assert_noop!(
				ParachainStaking::schedule_candidate_bond_less(Origin::signed(1), 5),
				Error::<Test>::PendingCandidateRequestAlreadyExists
			);
		});
}

#[test]
fn cannot_schedule_candidate_bond_less_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::schedule_candidate_bond_less(Origin::signed(6), 50),
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
				ParachainStaking::schedule_candidate_bond_less(Origin::signed(1), 21),
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
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1
			));
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				Origin::signed(1),
				10
			));
		});
}

#[test]
fn cannot_schedule_candidate_bond_less_if_exited_candidates() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				Origin::signed(1),
				1,
				0
			));
			assert_noop!(
				ParachainStaking::schedule_candidate_bond_less(Origin::signed(1), 10),
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
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				Origin::signed(1),
				30
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_candidate_bond_less(
				Origin::signed(1),
				1
			));
			assert_last_event!(MetaEvent::ParachainStaking(Event::CandidateBondedLess {
				candidate: 1,
				amount: 30,
				new_bond: 20
			}));
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
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				Origin::signed(1),
				10
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_candidate_bond_less(
				Origin::signed(1),
				1
			));
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
			let mut total = ParachainStaking::total();
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				Origin::signed(1),
				10
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_candidate_bond_less(
				Origin::signed(1),
				1
			));
			total -= 10;
			assert_eq!(ParachainStaking::total(), total);
		});
}

#[test]
fn execute_candidate_bond_less_updates_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			let candidate_state = ParachainStaking::candidate_info(1).expect("updated => exists");
			assert_eq!(candidate_state.bond, 30);
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				Origin::signed(1),
				10
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_candidate_bond_less(
				Origin::signed(1),
				1
			));
			let candidate_state = ParachainStaking::candidate_info(1).expect("updated => exists");
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
			assert_eq!(ParachainStaking::candidate_pool().0[0].owner, 1);
			assert_eq!(ParachainStaking::candidate_pool().0[0].amount, 30);
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				Origin::signed(1),
				10
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_candidate_bond_less(
				Origin::signed(1),
				1
			));
			assert_eq!(ParachainStaking::candidate_pool().0[0].owner, 1);
			assert_eq!(ParachainStaking::candidate_pool().0[0].amount, 20);
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
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				Origin::signed(1),
				10
			));
			assert_ok!(ParachainStaking::cancel_candidate_bond_less(
				Origin::signed(1)
			));
			assert_last_event!(MetaEvent::ParachainStaking(
				Event::CancelledCandidateBondLess {
					candidate: 1,
					amount: 10,
					execute_round: 3,
				}
			));
		});
}

#[test]
fn cancel_candidate_bond_less_updates_candidate_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				Origin::signed(1),
				10
			));
			assert_ok!(ParachainStaking::cancel_candidate_bond_less(
				Origin::signed(1)
			));
			assert!(ParachainStaking::candidate_info(&1)
				.unwrap()
				.request
				.is_none());
		});
}

#[test]
fn only_candidate_can_cancel_candidate_bond_less_request() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				Origin::signed(1),
				10
			));
			assert_noop!(
				ParachainStaking::cancel_candidate_bond_less(Origin::signed(2)),
				Error::<Test>::CandidateDNE
			);
		});
}

// DELEGATE

#[test]
fn delegate_event_emits_correctly() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::delegate(Origin::signed(2), 1, 10, 0, 0));
			assert_last_event!(MetaEvent::ParachainStaking(Event::Delegation {
				delegator: 2,
				locked_amount: 10,
				candidate: 1,
				delegator_position: DelegatorAdded::AddedToTop { new_total: 40 },
			}));
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
			assert_ok!(ParachainStaking::delegate(Origin::signed(2), 1, 10, 0, 0));
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
			assert!(ParachainStaking::delegator_state(2).is_none());
			assert_ok!(ParachainStaking::delegate(Origin::signed(2), 1, 10, 0, 0));
			let delegator_state =
				ParachainStaking::delegator_state(2).expect("just delegated => exists");
			assert_eq!(delegator_state.total, 10);
			assert_eq!(delegator_state.delegations.0[0].owner, 1);
			assert_eq!(delegator_state.delegations.0[0].amount, 10);
		});
}

#[test]
fn delegate_updates_collator_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			let candidate_state =
				ParachainStaking::candidate_info(1).expect("registered in genesis");
			assert_eq!(candidate_state.total_counted, 30);
			let top_delegations =
				ParachainStaking::top_delegations(1).expect("registered in genesis");
			assert!(top_delegations.delegations.is_empty());
			assert!(top_delegations.total.is_zero());
			assert_ok!(ParachainStaking::delegate(Origin::signed(2), 1, 10, 0, 0));
			let candidate_state =
				ParachainStaking::candidate_info(1).expect("just delegated => exists");
			assert_eq!(candidate_state.total_counted, 40);
			let top_delegations =
				ParachainStaking::top_delegations(1).expect("just delegated => exists");
			assert_eq!(top_delegations.delegations[0].owner, 2);
			assert_eq!(top_delegations.delegations[0].amount, 10);
			assert_eq!(top_delegations.total, 10);
		});
}

#[test]
fn can_delegate_immediately_after_other_join_candidates() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::join_candidates(Origin::signed(1), 20, 0));
			assert_ok!(ParachainStaking::delegate(Origin::signed(2), 1, 20, 0, 0));
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			assert_ok!(ParachainStaking::delegate(Origin::signed(2), 4, 10, 0, 2));
		});
}

#[test]
fn cannot_delegate_if_full_and_new_delegation_less_than_or_equal_lowest_bottom() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 20),
			(2, 10),
			(3, 10),
			(4, 10),
			(5, 10),
			(6, 10),
			(7, 10),
			(8, 10),
			(9, 10),
			(10, 10),
			(11, 10),
		])
		.with_candidates(vec![(1, 20)])
		.with_delegations(vec![
			(2, 1, 10),
			(3, 1, 10),
			(4, 1, 10),
			(5, 1, 10),
			(6, 1, 10),
			(8, 1, 10),
			(9, 1, 10),
			(10, 1, 10),
		])
		.build()
		.execute_with(|| {
			assert_noop!(
				ParachainStaking::delegate(Origin::signed(11), 1, 10, 8, 0),
				Error::<Test>::CannotDelegateLessThanOrEqualToLowestBottomWhenFull
			);
		});
}

#[test]
fn can_delegate_if_full_and_new_delegation_greater_than_lowest_bottom() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 20),
			(2, 10),
			(3, 10),
			(4, 10),
			(5, 10),
			(6, 10),
			(7, 10),
			(8, 10),
			(9, 10),
			(10, 10),
			(11, 11),
		])
		.with_candidates(vec![(1, 20)])
		.with_delegations(vec![
			(2, 1, 10),
			(3, 1, 10),
			(4, 1, 10),
			(5, 1, 10),
			(6, 1, 10),
			(8, 1, 10),
			(9, 1, 10),
			(10, 1, 10),
		])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::delegate(Origin::signed(11), 1, 11, 8, 0));
			assert_event_emitted!(Event::DelegationKicked {
				delegator: 10,
				candidate: 1,
				unstaked_amount: 10
			});
			assert_event_emitted!(Event::DelegatorLeft {
				delegator: 10,
				unstaked_amount: 10
			});
		});
}

#[test]
fn can_still_delegate_if_leaving() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20), (3, 20)])
		.with_candidates(vec![(1, 20), (3, 20)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				2
			)));
			assert_ok!(ParachainStaking::delegate(Origin::signed(2), 3, 10, 0, 1),);
		});
}

#[test]
fn cannot_delegate_if_candidate() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 30)])
		.with_candidates(vec![(1, 20), (2, 20)])
		.build()
		.execute_with(|| {
			assert_noop!(
				ParachainStaking::delegate(Origin::signed(2), 1, 10, 0, 0),
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
				ParachainStaking::delegate(Origin::signed(2), 1, 10, 1, 1),
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
				ParachainStaking::delegate(Origin::signed(2), 6, 10, 0, 4),
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
				assert_ok!(ParachainStaking::delegate(
					Origin::signed(i),
					1,
					10,
					count,
					0u32
				));
				count += 1u32;
			}
			let mut count = 0u32;
			for i in 3..11 {
				assert_ok!(ParachainStaking::delegate(
					Origin::signed(i),
					2,
					10,
					count,
					1u32
				));
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
					ParachainStaking::delegate(Origin::signed(i), 1, 10, count, 0u32),
					Error::<Test>::TooLowCandidateDelegationCountToDelegate
				);
			}
			// to set up for next error test
			count = 4u32;
			for i in 7..11 {
				assert_ok!(ParachainStaking::delegate(
					Origin::signed(i),
					1,
					10,
					count,
					0u32
				));
				count += 1u32;
			}
			count = 0u32;
			for i in 3..11 {
				assert_noop!(
					ParachainStaking::delegate(Origin::signed(i), 2, 10, count, 0u32),
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
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				2
			)));
			assert_last_event!(MetaEvent::ParachainStaking(Event::DelegatorExitScheduled {
				round: 1,
				delegator: 2,
				scheduled_exit: 3
			}));
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
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				2
			)));
			assert_noop!(
				ParachainStaking::schedule_leave_delegators(Origin::signed(2)),
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
				ParachainStaking::schedule_leave_delegators(Origin::signed(2)),
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
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				2
			)));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_delegators(
				Origin::signed(2),
				2,
				1
			));
			assert_event_emitted!(Event::DelegatorLeft {
				delegator: 2,
				unstaked_amount: 10
			});
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
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				2
			)));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_delegators(
				Origin::signed(2),
				2,
				1
			));
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
			assert_eq!(ParachainStaking::total(), 40);
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				2
			)));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_delegators(
				Origin::signed(2),
				2,
				1
			));
			assert_eq!(ParachainStaking::total(), 30);
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
			assert!(ParachainStaking::delegator_state(2).is_some());
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				2
			)));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_delegators(
				Origin::signed(2),
				2,
				1
			));
			assert!(ParachainStaking::delegator_state(2).is_none());
		});
}

#[test]
fn execute_leave_delegators_removes_pending_delegation_requests() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10), (2, 15)])
		.with_candidates(vec![(1, 10)])
		.with_delegations(vec![(2, 1, 15)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				5
			));
			let state = ParachainStaking::delegation_scheduled_requests(&1);
			assert_eq!(
				state,
				vec![ScheduledRequest {
					delegator: 2,
					when_executable: 3,
					action: DelegationAction::Decrease(5),
				}],
			);
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				2
			)));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_delegators(
				Origin::signed(2),
				2,
				1
			));
			assert!(ParachainStaking::delegator_state(2).is_none());
			assert!(
				!ParachainStaking::delegation_scheduled_requests(&1)
					.iter()
					.any(|x| x.delegator == 2),
				"delegation request not removed"
			)
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
					ParachainStaking::candidate_info(i).expect("initialized in ext builder");
				assert_eq!(candidate_state.total_counted, 30);
				let top_delegations =
					ParachainStaking::top_delegations(i).expect("initialized in ext builder");
				assert_eq!(top_delegations.delegations[0].owner, 1);
				assert_eq!(top_delegations.delegations[0].amount, 10);
				assert_eq!(top_delegations.total, 10);
			}
			assert_eq!(
				ParachainStaking::delegator_state(1)
					.unwrap()
					.delegations
					.0
					.len(),
				4usize
			);
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				1
			)));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_delegators(
				Origin::signed(1),
				1,
				10
			));
			for i in 2..6 {
				let candidate_state =
					ParachainStaking::candidate_info(i).expect("initialized in ext builder");
				assert_eq!(candidate_state.total_counted, 20);
				let top_delegations =
					ParachainStaking::top_delegations(i).expect("initialized in ext builder");
				assert!(top_delegations.delegations.is_empty());
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
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				2
			)));
			assert_noop!(
				ParachainStaking::execute_leave_delegators(Origin::signed(2), 2, 1),
				Error::<Test>::DelegatorCannotLeaveYet
			);
			// can execute after delay
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_delegators(
				Origin::signed(2),
				2,
				1
			));
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
				assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
					i
				)));
			}
			roll_to(10);
			for i in 3..7 {
				assert_noop!(
					ParachainStaking::execute_leave_delegators(Origin::signed(i), i, 0),
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
				assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
					i
				)));
			}
			roll_to(10);
			for i in 3..7 {
				assert_ok!(ParachainStaking::execute_leave_delegators(
					Origin::signed(i),
					i,
					1
				));
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
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				2
			)));
			assert_ok!(ParachainStaking::cancel_leave_delegators(Origin::signed(2)));
			assert_last_event!(MetaEvent::ParachainStaking(Event::DelegatorExitCancelled {
				delegator: 2
			}));
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
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				2
			)));
			assert_ok!(ParachainStaking::cancel_leave_delegators(Origin::signed(2)));
			let delegator =
				ParachainStaking::delegator_state(&2).expect("just cancelled exit so exists");
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			assert_last_event!(MetaEvent::ParachainStaking(
				Event::DelegationRevocationScheduled {
					round: 1,
					delegator: 2,
					candidate: 1,
					scheduled_exit: 3,
				}
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			assert_event_emitted!(Event::DelegatorLeftCandidate {
				delegator: 2,
				candidate: 1,
				unstaked_amount: 10,
				total_candidate_staked: 30
			});
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			// this is an exit implicitly because last delegation revoked
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				3
			));
		});
}

#[test]
fn delegator_not_allowed_revoke_if_already_leaving() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 20), (3, 20)])
		.with_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				2
			)));
			assert_noop!(
				ParachainStaking::schedule_revoke_delegation(Origin::signed(2), 3),
				<Error<Test>>::PendingDelegationRequestAlreadyExists,
			);
		});
}

#[test]
fn cannot_revoke_delegation_if_not_delegator() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::schedule_revoke_delegation(Origin::signed(2), 1),
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
				ParachainStaking::schedule_revoke_delegation(Origin::signed(2), 3),
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
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
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(2),
				1,
				5
			));
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
			assert_eq!(ParachainStaking::total(), 40);
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(2),
				1,
				5
			));
			assert_eq!(ParachainStaking::total(), 45);
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
			assert_eq!(
				ParachainStaking::delegator_state(2).expect("exists").total,
				10
			);
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(2),
				1,
				5
			));
			assert_eq!(
				ParachainStaking::delegator_state(2).expect("exists").total,
				15
			);
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
				ParachainStaking::top_delegations(1).unwrap().delegations[0].owner,
				2
			);
			assert_eq!(
				ParachainStaking::top_delegations(1).unwrap().delegations[0].amount,
				10
			);
			assert_eq!(ParachainStaking::top_delegations(1).unwrap().total, 10);
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(2),
				1,
				5
			));
			assert_eq!(
				ParachainStaking::top_delegations(1).unwrap().delegations[0].owner,
				2
			);
			assert_eq!(
				ParachainStaking::top_delegations(1).unwrap().delegations[0].amount,
				15
			);
			assert_eq!(ParachainStaking::top_delegations(1).unwrap().total, 15);
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
				ParachainStaking::bottom_delegations(1)
					.expect("exists")
					.delegations[0]
					.owner,
				2
			);
			assert_eq!(
				ParachainStaking::bottom_delegations(1)
					.expect("exists")
					.delegations[0]
					.amount,
				10
			);
			assert_eq!(ParachainStaking::bottom_delegations(1).unwrap().total, 10);
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(2),
				1,
				5
			));
			assert_last_event!(MetaEvent::ParachainStaking(Event::DelegationIncreased {
				delegator: 2,
				candidate: 1,
				amount: 5,
				in_top: false
			}));
			assert_eq!(
				ParachainStaking::bottom_delegations(1)
					.expect("exists")
					.delegations[0]
					.owner,
				2
			);
			assert_eq!(
				ParachainStaking::bottom_delegations(1)
					.expect("exists")
					.delegations[0]
					.amount,
				15
			);
			assert_eq!(ParachainStaking::bottom_delegations(1).unwrap().total, 15);
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
			assert_eq!(ParachainStaking::total(), 40);
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(2),
				1,
				5
			));
			assert_eq!(ParachainStaking::total(), 45);
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
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1
			));
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(2),
				1,
				5
			));
		});
}

#[test]
fn delegator_bond_more_disallowed_when_revoke_scheduled() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			assert_noop!(
				ParachainStaking::delegator_bond_more(Origin::signed(2), 1, 5),
				<Error<Test>>::PendingDelegationRevoke
			);
		});
}

#[test]
fn delegator_bond_more_allowed_when_bond_decrease_scheduled() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 15)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				5,
			));
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(2),
				1,
				5
			));
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
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				5
			));
			assert_last_event!(MetaEvent::ParachainStaking(
				Event::DelegationDecreaseScheduled {
					delegator: 2,
					candidate: 1,
					amount_to_decrease: 5,
					execute_round: 3,
				}
			));
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
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				5
			));
			let state = ParachainStaking::delegation_scheduled_requests(&1);
			assert_eq!(
				state,
				vec![ScheduledRequest {
					delegator: 2,
					when_executable: 3,
					action: DelegationAction::Decrease(5),
				}],
			);
		});
}

#[test]
fn delegator_not_allowed_bond_less_if_leaving() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 15)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				2
			)));
			assert_noop!(
				ParachainStaking::schedule_delegator_bond_less(Origin::signed(2), 1, 1),
				<Error<Test>>::PendingDelegationRequestAlreadyExists,
			);
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			assert_noop!(
				ParachainStaking::schedule_delegator_bond_less(Origin::signed(2), 1, 1),
				Error::<Test>::PendingDelegationRequestAlreadyExists
			);
		});
}

#[test]
fn cannot_delegator_bond_less_if_not_delegator() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::schedule_delegator_bond_less(Origin::signed(2), 1, 5),
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
				ParachainStaking::schedule_delegator_bond_less(Origin::signed(2), 3, 5),
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
				ParachainStaking::schedule_delegator_bond_less(Origin::signed(2), 3, 5),
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
				ParachainStaking::schedule_delegator_bond_less(Origin::signed(2), 1, 6),
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
				ParachainStaking::schedule_delegator_bond_less(Origin::signed(2), 1, 11),
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
				ParachainStaking::schedule_delegator_bond_less(Origin::signed(2), 1, 8),
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			assert_event_emitted!(Event::DelegatorLeftCandidate {
				delegator: 2,
				candidate: 1,
				unstaked_amount: 10,
				total_candidate_staked: 30
			});
			assert_event_emitted!(Event::DelegatorLeft {
				delegator: 2,
				unstaked_amount: 10
			});
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			roll_to(10);
			assert_noop!(
				ParachainStaking::execute_delegation_request(Origin::signed(2), 2, 1),
				Error::<Test>::DelegatorBondBelowMin
			);
			// but delegator can cancel the request and request to leave instead:
			assert_ok!(ParachainStaking::cancel_delegation_request(
				Origin::signed(2),
				1
			));
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				2
			)));
			roll_to(20);
			assert_ok!(ParachainStaking::execute_leave_delegators(
				Origin::signed(2),
				2,
				2
			));
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			assert_event_emitted!(Event::DelegatorLeftCandidate {
				delegator: 2,
				candidate: 1,
				unstaked_amount: 10,
				total_candidate_staked: 30
			});
			assert_event_emitted!(Event::DelegatorLeft {
				delegator: 2,
				unstaked_amount: 10
			});
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			assert_event_emitted!(Event::DelegatorLeftCandidate {
				delegator: 2,
				candidate: 1,
				unstaked_amount: 10,
				total_candidate_staked: 30
			});
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
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
			assert!(!ParachainStaking::delegation_scheduled_requests(&1)
				.iter()
				.any(|x| x.delegator == 2));
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			assert!(ParachainStaking::delegation_scheduled_requests(&1)
				.iter()
				.any(|x| x.delegator == 2));
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			assert!(!ParachainStaking::delegation_scheduled_requests(&1)
				.iter()
				.any(|x| x.delegator == 2));
		});
}

#[test]
fn execute_revoke_delegation_removes_revocation_from_state_for_single_delegation_leave() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 20), (3, 20)])
		.with_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			assert!(
				!ParachainStaking::delegation_scheduled_requests(&1)
					.iter()
					.any(|x| x.delegator == 2),
				"delegation was not removed"
			);
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
			assert_eq!(ParachainStaking::total(), 40);
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			assert_eq!(ParachainStaking::total(), 30);
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
			assert!(ParachainStaking::delegator_state(2).is_some());
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			roll_to(10);
			// this will be confusing for people
			// if status is leaving, then execute_delegation_request works if last delegation
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			assert!(ParachainStaking::delegator_state(2).is_none());
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
				ParachainStaking::candidate_info(1)
					.expect("exists")
					.delegation_count,
				1u32
			);
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			assert!(ParachainStaking::candidate_info(1)
				.expect("exists")
				.delegation_count
				.is_zero());
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
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1
			));
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			roll_to(10);
			// can execute delegation request for leaving candidate
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
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
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1
			));
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			roll_to(10);
			// revocation executes during execute leave candidates (callable by anyone)
			assert_ok!(ParachainStaking::execute_leave_candidates(
				Origin::signed(1),
				1,
				1
			));
			assert!(!ParachainStaking::is_delegator(&2));
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(2),
				3,
				10
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			assert!(ParachainStaking::is_delegator(&2));
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			assert_last_event!(MetaEvent::ParachainStaking(
				Event::DelegationRevocationScheduled {
					round: 1,
					delegator: 2,
					candidate: 1,
					scheduled_exit: 3,
				}
			));
			assert_noop!(
				ParachainStaking::schedule_delegator_bond_less(Origin::signed(2), 1, 2),
				Error::<Test>::PendingDelegationRequestAlreadyExists
			);
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				3,
				2
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				3
			));
			assert_last_event!(MetaEvent::ParachainStaking(Event::DelegationDecreased {
				delegator: 2,
				candidate: 3,
				amount: 2,
				in_top: true
			}));
			assert!(ParachainStaking::is_delegator(&2));
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
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				5
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
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
			assert_eq!(ParachainStaking::total(), 40);
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				5
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			assert_eq!(ParachainStaking::total(), 35);
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
			assert_eq!(
				ParachainStaking::delegator_state(2).expect("exists").total,
				10
			);
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				5
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			assert_eq!(
				ParachainStaking::delegator_state(2).expect("exists").total,
				5
			);
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
				ParachainStaking::top_delegations(1).unwrap().delegations[0].owner,
				2
			);
			assert_eq!(
				ParachainStaking::top_delegations(1).unwrap().delegations[0].amount,
				10
			);
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				5
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			assert_eq!(
				ParachainStaking::top_delegations(1).unwrap().delegations[0].owner,
				2
			);
			assert_eq!(
				ParachainStaking::top_delegations(1).unwrap().delegations[0].amount,
				5
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
			assert_eq!(ParachainStaking::total(), 40);
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				5
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			assert_eq!(ParachainStaking::total(), 35);
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
			let pre_call_candidate_info =
				ParachainStaking::candidate_info(&1).expect("delegated by all so exists");
			let pre_call_top_delegations =
				ParachainStaking::top_delegations(&1).expect("delegated by all so exists");
			let pre_call_bottom_delegations =
				ParachainStaking::bottom_delegations(&1).expect("delegated by all so exists");
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				2
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			let post_call_candidate_info =
				ParachainStaking::candidate_info(&1).expect("delegated by all so exists");
			let post_call_top_delegations =
				ParachainStaking::top_delegations(&1).expect("delegated by all so exists");
			let post_call_bottom_delegations =
				ParachainStaking::bottom_delegations(&1).expect("delegated by all so exists");
			let mut not_equal = false;
			for Bond { owner, amount } in pre_call_bottom_delegations.delegations {
				for Bond {
					owner: post_owner,
					amount: post_amount,
				} in &post_call_bottom_delegations.delegations
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
			for Bond { owner, amount } in pre_call_top_delegations.delegations {
				for Bond {
					owner: post_owner,
					amount: post_amount,
				} in &post_call_top_delegations.delegations
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
				pre_call_candidate_info.total_counted,
				post_call_candidate_info.total_counted
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
			let pre_call_candidate_info =
				ParachainStaking::candidate_info(&1).expect("delegated by all so exists");
			let pre_call_top_delegations =
				ParachainStaking::top_delegations(&1).expect("delegated by all so exists");
			let pre_call_bottom_delegations =
				ParachainStaking::bottom_delegations(&1).expect("delegated by all so exists");
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(6),
				1,
				4
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(6),
				6,
				1
			));
			let post_call_candidate_info =
				ParachainStaking::candidate_info(&1).expect("delegated by all so exists");
			let post_call_top_delegations =
				ParachainStaking::top_delegations(&1).expect("delegated by all so exists");
			let post_call_bottom_delegations =
				ParachainStaking::bottom_delegations(&1).expect("delegated by all so exists");
			let mut equal = true;
			for Bond { owner, amount } in pre_call_bottom_delegations.delegations {
				for Bond {
					owner: post_owner,
					amount: post_amount,
				} in &post_call_bottom_delegations.delegations
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
			for Bond { owner, amount } in pre_call_top_delegations.delegations {
				for Bond {
					owner: post_owner,
					amount: post_amount,
				} in &post_call_top_delegations.delegations
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
				pre_call_candidate_info.total_counted - 4,
				post_call_candidate_info.total_counted
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
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(1),
				1
			));
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				5
			));
			roll_to(10);
			// can execute bond more delegation request for leaving candidate
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			assert_ok!(ParachainStaking::cancel_delegation_request(
				Origin::signed(2),
				1
			));
			assert_last_event!(MetaEvent::ParachainStaking(
				Event::CancelledDelegationRequest {
					delegator: 2,
					collator: 1,
					cancelled_request: CancelledScheduledRequest {
						when_executable: 3,
						action: DelegationAction::Revoke(10),
					},
				}
			));
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			let state = ParachainStaking::delegation_scheduled_requests(&1);
			assert_eq!(
				state,
				vec![ScheduledRequest {
					delegator: 2,
					when_executable: 3,
					action: DelegationAction::Revoke(10),
				}],
			);
			assert_eq!(
				ParachainStaking::delegator_state(&2)
					.map(|x| x.less_total)
					.expect("delegator state must exist"),
				10
			);
			assert_ok!(ParachainStaking::cancel_delegation_request(
				Origin::signed(2),
				1
			));
			assert!(!ParachainStaking::delegation_scheduled_requests(&1)
				.iter()
				.any(|x| x.delegator == 2));
			assert_eq!(
				ParachainStaking::delegator_state(&2)
					.map(|x| x.less_total)
					.expect("delegator state must exist"),
				0
			);
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
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				5
			));
			assert_ok!(ParachainStaking::cancel_delegation_request(
				Origin::signed(2),
				1
			));
			assert_last_event!(MetaEvent::ParachainStaking(
				Event::CancelledDelegationRequest {
					delegator: 2,
					collator: 1,
					cancelled_request: CancelledScheduledRequest {
						when_executable: 3,
						action: DelegationAction::Decrease(5),
					},
				}
			));
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
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				5
			));
			let state = ParachainStaking::delegation_scheduled_requests(&1);
			assert_eq!(
				state,
				vec![ScheduledRequest {
					delegator: 2,
					when_executable: 3,
					action: DelegationAction::Decrease(5),
				}],
			);
			assert_eq!(
				ParachainStaking::delegator_state(&2)
					.map(|x| x.less_total)
					.expect("delegator state must exist"),
				5
			);
			assert_ok!(ParachainStaking::cancel_delegation_request(
				Origin::signed(2),
				1
			));
			assert!(!ParachainStaking::delegation_scheduled_requests(&1)
				.iter()
				.any(|x| x.delegator == 2));
			assert_eq!(
				ParachainStaking::delegator_state(&2)
					.map(|x| x.less_total)
					.expect("delegator state must exist"),
				0
			);
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			assert_eq!(
				ParachainStaking::delegator_state(&2)
					.map(|x| x.less_total)
					.expect("delegator state must exist"),
				10
			);
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				1
			));
			assert_eq!(
				ParachainStaking::delegator_state(&2)
					.map(|x| x.less_total)
					.expect("delegator state must exist"),
				0
			);
			assert_ok!(ParachainStaking::delegate(Origin::signed(2), 5, 10, 0, 2));
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				3
			));
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				4
			));
			assert_eq!(
				ParachainStaking::delegator_state(&2)
					.map(|x| x.less_total)
					.expect("delegator state must exist"),
				20,
			);
			roll_to(20);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				3
			));
			assert_eq!(
				ParachainStaking::delegator_state(&2)
					.map(|x| x.less_total)
					.expect("delegator state must exist"),
				10,
			);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(2),
				2,
				4
			));
			assert_eq!(
				ParachainStaking::delegator_state(&2)
					.map(|x| x.less_total)
					.expect("delegator state must exist"),
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
			assert_ok!(ParachainStaking::set_parachain_bond_account(
				Origin::root(),
				11
			));
			roll_to(8);
			// chooses top TotalSelectedCandidates (5), in order
			let mut expected = vec![
				Event::ParachainBondAccountSet { old: 0, new: 11 },
				Event::CollatorChosen {
					round: 2,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 5,
					round: 2,
					selected_collators_number: 5,
					total_balance: 140,
				},
			];
			assert_eq_events!(expected.clone());
			assert_eq!(Balances::free_balance(&11), 1);
			// ~ set block author as 1 for all blocks this round
			set_author(2, 1, 100);
			roll_to(16);
			// distribute total issuance to collator 1 and its delegators 6, 7, 19
			let mut new = vec![
				Event::CollatorChosen {
					round: 3,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 10,
					round: 3,
					selected_collators_number: 5,
					total_balance: 140,
				},
				Event::ReservedForParachainBond {
					account: 11,
					value: 15,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 15,
					round: 4,
					selected_collators_number: 5,
					total_balance: 140,
				},
				Event::Rewarded {
					account: 1,
					rewards: 20,
				},
				Event::Rewarded {
					account: 6,
					rewards: 5,
				},
				Event::Rewarded {
					account: 7,
					rewards: 5,
				},
				Event::Rewarded {
					account: 10,
					rewards: 5,
				},
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
				ParachainStaking::schedule_leave_delegators(Origin::signed(66)),
				Error::<Test>::DelegatorDNE
			);
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				6
			)));
			// fast forward to block in which delegator 6 exit executes
			roll_to(25);
			assert_ok!(ParachainStaking::execute_leave_delegators(
				Origin::signed(6),
				6,
				10
			));
			roll_to(30);
			let mut new2 = vec![
				Event::DelegatorExitScheduled {
					round: 4,
					delegator: 6,
					scheduled_exit: 6,
				},
				Event::ReservedForParachainBond {
					account: 11,
					value: 16,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 20,
					round: 5,
					selected_collators_number: 5,
					total_balance: 140,
				},
				Event::Rewarded {
					account: 1,
					rewards: 21,
				},
				Event::Rewarded {
					account: 6,
					rewards: 5,
				},
				Event::Rewarded {
					account: 7,
					rewards: 5,
				},
				Event::Rewarded {
					account: 10,
					rewards: 5,
				},
				Event::ReservedForParachainBond {
					account: 11,
					value: 16,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 25,
					round: 6,
					selected_collators_number: 5,
					total_balance: 140,
				},
				Event::Rewarded {
					account: 1,
					rewards: 22,
				},
				Event::Rewarded {
					account: 6,
					rewards: 6,
				},
				Event::Rewarded {
					account: 7,
					rewards: 6,
				},
				Event::Rewarded {
					account: 10,
					rewards: 6,
				},
				Event::DelegatorLeftCandidate {
					delegator: 6,
					candidate: 1,
					unstaked_amount: 10,
					total_candidate_staked: 40,
				},
				Event::DelegatorLeft {
					delegator: 6,
					unstaked_amount: 10,
				},
				Event::ReservedForParachainBond {
					account: 11,
					value: 17,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 1,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 30,
					round: 7,
					selected_collators_number: 5,
					total_balance: 130,
				},
				Event::Rewarded {
					account: 1,
					rewards: 24,
				},
				Event::Rewarded {
					account: 7,
					rewards: 6,
				},
				Event::Rewarded {
					account: 10,
					rewards: 6,
				},
			];
			expected.append(&mut new2);
			assert_eq_events!(expected.clone());
			assert_eq!(Balances::free_balance(&11), 65);
			assert_ok!(ParachainStaking::set_parachain_bond_reserve_percent(
				Origin::root(),
				Percent::from_percent(50)
			));
			// 6 won't be paid for this round because they left already
			set_author(6, 1, 100);
			roll_to(35);
			// keep paying 6
			let mut new3 = vec![
				Event::ParachainBondReservePercentSet {
					old: Percent::from_percent(30),
					new: Percent::from_percent(50),
				},
				Event::ReservedForParachainBond {
					account: 11,
					value: 30,
				},
				Event::CollatorChosen {
					round: 8,
					collator_account: 1,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 8,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 8,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 8,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 8,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 35,
					round: 8,
					selected_collators_number: 5,
					total_balance: 130,
				},
				Event::Rewarded {
					account: 1,
					rewards: 20,
				},
				Event::Rewarded {
					account: 7,
					rewards: 4,
				},
				Event::Rewarded {
					account: 10,
					rewards: 4,
				},
			];
			expected.append(&mut new3);
			assert_eq_events!(expected.clone());
			assert_eq!(Balances::free_balance(&11), 95);
			set_author(7, 1, 100);
			roll_to(40);
			// no more paying 6
			let mut new4 = vec![
				Event::ReservedForParachainBond {
					account: 11,
					value: 31,
				},
				Event::CollatorChosen {
					round: 9,
					collator_account: 1,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 9,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 9,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 9,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 9,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 40,
					round: 9,
					selected_collators_number: 5,
					total_balance: 130,
				},
				Event::Rewarded {
					account: 1,
					rewards: 22,
				},
				Event::Rewarded {
					account: 7,
					rewards: 5,
				},
				Event::Rewarded {
					account: 10,
					rewards: 5,
				},
			];
			expected.append(&mut new4);
			assert_eq_events!(expected.clone());
			assert_eq!(Balances::free_balance(&11), 126);
			set_author(8, 1, 100);
			assert_ok!(ParachainStaking::delegate(Origin::signed(8), 1, 10, 10, 10));
			roll_to(45);
			// new delegation is not rewarded yet
			let mut new5 = vec![
				Event::Delegation {
					delegator: 8,
					locked_amount: 10,
					candidate: 1,
					delegator_position: DelegatorAdded::AddedToTop { new_total: 50 },
				},
				Event::ReservedForParachainBond {
					account: 11,
					value: 33,
				},
				Event::CollatorChosen {
					round: 10,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 10,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 10,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 10,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 10,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 45,
					round: 10,
					selected_collators_number: 5,
					total_balance: 140,
				},
				Event::Rewarded {
					account: 1,
					rewards: 23,
				},
				Event::Rewarded {
					account: 7,
					rewards: 5,
				},
				Event::Rewarded {
					account: 10,
					rewards: 5,
				},
			];
			expected.append(&mut new5);
			assert_eq_events!(expected.clone());
			assert_eq!(Balances::free_balance(&11), 159);
			set_author(9, 1, 100);
			set_author(10, 1, 100);
			roll_to(50);
			// new delegation is still not rewarded yet
			let mut new6 = vec![
				Event::ReservedForParachainBond {
					account: 11,
					value: 35,
				},
				Event::CollatorChosen {
					round: 11,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 11,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 11,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 11,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 11,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 50,
					round: 11,
					selected_collators_number: 5,
					total_balance: 140,
				},
				Event::Rewarded {
					account: 1,
					rewards: 24,
				},
				Event::Rewarded {
					account: 7,
					rewards: 5,
				},
				Event::Rewarded {
					account: 10,
					rewards: 5,
				},
			];
			expected.append(&mut new6);
			assert_eq_events!(expected.clone());
			assert_eq!(Balances::free_balance(&11), 194);
			roll_to(55);
			// new delegation is rewarded, 2 rounds after joining (`RewardPaymentDelay` is 2)
			let mut new7 = vec![
				Event::ReservedForParachainBond {
					account: 11,
					value: 36,
				},
				Event::CollatorChosen {
					round: 12,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 12,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 12,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 12,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 12,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 55,
					round: 12,
					selected_collators_number: 5,
					total_balance: 140,
				},
				Event::Rewarded {
					account: 1,
					rewards: 24,
				},
				Event::Rewarded {
					account: 7,
					rewards: 4,
				},
				Event::Rewarded {
					account: 10,
					rewards: 4,
				},
				Event::Rewarded {
					account: 8,
					rewards: 4,
				},
			];
			expected.append(&mut new7);
			assert_eq_events!(expected);
			assert_eq!(Balances::free_balance(&11), 230);
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
				Event::CollatorChosen {
					round: 2,
					collator_account: 1,
					total_exposed_amount: 40,
				},
				Event::NewRound {
					starting_block: 5,
					round: 2,
					selected_collators_number: 1,
					total_balance: 40,
				},
			];
			assert_eq_events!(expected.clone());
			assert_ok!(ParachainStaking::join_candidates(
				Origin::signed(4),
				20u128,
				100u32
			));
			assert_last_event!(MetaEvent::ParachainStaking(
				Event::JoinedCollatorCandidates {
					account: 4,
					amount_locked: 20u128,
					new_total_amt_locked: 60u128,
				}
			));
			roll_to(9);
			assert_ok!(ParachainStaking::delegate(Origin::signed(5), 4, 10, 10, 10));
			assert_ok!(ParachainStaking::delegate(Origin::signed(6), 4, 10, 10, 10));
			roll_to(11);
			let mut new = vec![
				Event::JoinedCollatorCandidates {
					account: 4,
					amount_locked: 20,
					new_total_amt_locked: 60,
				},
				Event::Delegation {
					delegator: 5,
					locked_amount: 10,
					candidate: 4,
					delegator_position: DelegatorAdded::AddedToTop { new_total: 30 },
				},
				Event::Delegation {
					delegator: 6,
					locked_amount: 10,
					candidate: 4,
					delegator_position: DelegatorAdded::AddedToTop { new_total: 40 },
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 1,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 4,
					total_exposed_amount: 40,
				},
				Event::NewRound {
					starting_block: 10,
					round: 3,
					selected_collators_number: 2,
					total_balance: 80,
				},
			];
			expected.append(&mut new);
			assert_eq_events!(expected.clone());
			// only reward author with id 4
			set_author(3, 4, 100);
			roll_to(21);
			// 20% of 10 is commission + due_portion (0) = 2 + 4 = 6
			// all delegator payouts are 10-2 = 8 * stake_pct
			let mut new2 = vec![
				Event::CollatorChosen {
					round: 4,
					collator_account: 1,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 4,
					total_exposed_amount: 40,
				},
				Event::NewRound {
					starting_block: 15,
					round: 4,
					selected_collators_number: 2,
					total_balance: 80,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 1,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 4,
					total_exposed_amount: 40,
				},
				Event::NewRound {
					starting_block: 20,
					round: 5,
					selected_collators_number: 2,
					total_balance: 80,
				},
				Event::Rewarded {
					account: 4,
					rewards: 18,
				},
				Event::Rewarded {
					account: 5,
					rewards: 6,
				},
				Event::Rewarded {
					account: 6,
					rewards: 6,
				},
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
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(2),
				2
			));
			let info = ParachainStaking::candidate_info(&2).unwrap();
			assert_eq!(info.status, CollatorStatus::Leaving(5));
			roll_to(21);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				Origin::signed(2),
				2,
				2
			));
			// we must exclude leaving collators from rewards while
			// holding them retroactively accountable for previous faults
			// (within the last T::SlashingWindow blocks)
			let expected = vec![
				Event::CollatorChosen {
					round: 2,
					collator_account: 1,
					total_exposed_amount: 700,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 2,
					total_exposed_amount: 400,
				},
				Event::NewRound {
					starting_block: 5,
					round: 2,
					selected_collators_number: 2,
					total_balance: 1100,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 1,
					total_exposed_amount: 700,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 2,
					total_exposed_amount: 400,
				},
				Event::NewRound {
					starting_block: 10,
					round: 3,
					selected_collators_number: 2,
					total_balance: 1100,
				},
				Event::CandidateScheduledExit {
					exit_allowed_round: 3,
					candidate: 2,
					scheduled_exit: 5,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 1,
					total_exposed_amount: 700,
				},
				Event::NewRound {
					starting_block: 15,
					round: 4,
					selected_collators_number: 1,
					total_balance: 700,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 1,
					total_exposed_amount: 700,
				},
				Event::NewRound {
					starting_block: 20,
					round: 5,
					selected_collators_number: 1,
					total_balance: 700,
				},
				Event::CandidateLeft {
					ex_candidate: 2,
					unlocked_amount: 400,
					new_total_amt_locked: 700,
				},
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
				Event::CollatorChosen {
					round: 2,
					collator_account: 1,
					total_exposed_amount: 100,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 2,
					total_exposed_amount: 90,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 3,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 4,
					total_exposed_amount: 70,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 5,
					total_exposed_amount: 60,
				},
				Event::NewRound {
					starting_block: 5,
					round: 2,
					selected_collators_number: 5,
					total_balance: 400,
				},
			];
			assert_eq_events!(expected.clone());
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(6),
				6
			));
			assert_last_event!(MetaEvent::ParachainStaking(Event::CandidateScheduledExit {
				exit_allowed_round: 2,
				candidate: 6,
				scheduled_exit: 4
			}));
			roll_to(21);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				Origin::signed(6),
				6,
				0
			));
			assert_ok!(ParachainStaking::join_candidates(
				Origin::signed(6),
				69u128,
				100u32
			));
			assert_last_event!(MetaEvent::ParachainStaking(
				Event::JoinedCollatorCandidates {
					account: 6,
					amount_locked: 69u128,
					new_total_amt_locked: 469u128,
				}
			));
			roll_to(27);
			// should choose top TotalSelectedCandidates (5), in order
			let expected = vec![
				Event::CollatorChosen {
					round: 2,
					collator_account: 1,
					total_exposed_amount: 100,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 2,
					total_exposed_amount: 90,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 3,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 4,
					total_exposed_amount: 70,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 5,
					total_exposed_amount: 60,
				},
				Event::NewRound {
					starting_block: 5,
					round: 2,
					selected_collators_number: 5,
					total_balance: 400,
				},
				Event::CandidateScheduledExit {
					exit_allowed_round: 2,
					candidate: 6,
					scheduled_exit: 4,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 1,
					total_exposed_amount: 100,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 2,
					total_exposed_amount: 90,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 3,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 4,
					total_exposed_amount: 70,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 5,
					total_exposed_amount: 60,
				},
				Event::NewRound {
					starting_block: 10,
					round: 3,
					selected_collators_number: 5,
					total_balance: 400,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 1,
					total_exposed_amount: 100,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 2,
					total_exposed_amount: 90,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 3,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 4,
					total_exposed_amount: 70,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 5,
					total_exposed_amount: 60,
				},
				Event::NewRound {
					starting_block: 15,
					round: 4,
					selected_collators_number: 5,
					total_balance: 400,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 1,
					total_exposed_amount: 100,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 2,
					total_exposed_amount: 90,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 3,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 4,
					total_exposed_amount: 70,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 5,
					total_exposed_amount: 60,
				},
				Event::NewRound {
					starting_block: 20,
					round: 5,
					selected_collators_number: 5,
					total_balance: 400,
				},
				Event::CandidateLeft {
					ex_candidate: 6,
					unlocked_amount: 50,
					new_total_amt_locked: 400,
				},
				Event::JoinedCollatorCandidates {
					account: 6,
					amount_locked: 69,
					new_total_amt_locked: 469,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 1,
					total_exposed_amount: 100,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 2,
					total_exposed_amount: 90,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 3,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 4,
					total_exposed_amount: 70,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 6,
					total_exposed_amount: 69,
				},
				Event::NewRound {
					starting_block: 25,
					round: 6,
					selected_collators_number: 5,
					total_balance: 409,
				},
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
				Event::CollatorChosen {
					round: 2,
					collator_account: 1,
					total_exposed_amount: 100,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 2,
					total_exposed_amount: 90,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 3,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 4,
					total_exposed_amount: 70,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 5,
					total_exposed_amount: 60,
				},
				Event::NewRound {
					starting_block: 5,
					round: 2,
					selected_collators_number: 5,
					total_balance: 400,
				},
			];
			assert_eq_events!(expected.clone());
			// ~ set block author as 1 for all blocks this round
			set_author(2, 1, 100);
			roll_to(16);
			// pay total issuance to 1
			let mut new = vec![
				Event::CollatorChosen {
					round: 3,
					collator_account: 1,
					total_exposed_amount: 100,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 2,
					total_exposed_amount: 90,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 3,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 4,
					total_exposed_amount: 70,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 5,
					total_exposed_amount: 60,
				},
				Event::NewRound {
					starting_block: 10,
					round: 3,
					selected_collators_number: 5,
					total_balance: 400,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 1,
					total_exposed_amount: 100,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 2,
					total_exposed_amount: 90,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 3,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 4,
					total_exposed_amount: 70,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 5,
					total_exposed_amount: 60,
				},
				Event::NewRound {
					starting_block: 15,
					round: 4,
					selected_collators_number: 5,
					total_balance: 400,
				},
				Event::Rewarded {
					account: 1,
					rewards: 305,
				},
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
				Event::CollatorChosen {
					round: 5,
					collator_account: 1,
					total_exposed_amount: 100,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 2,
					total_exposed_amount: 90,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 3,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 4,
					total_exposed_amount: 70,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 5,
					total_exposed_amount: 60,
				},
				Event::NewRound {
					starting_block: 20,
					round: 5,
					selected_collators_number: 5,
					total_balance: 400,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 1,
					total_exposed_amount: 100,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 2,
					total_exposed_amount: 90,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 3,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 4,
					total_exposed_amount: 70,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 5,
					total_exposed_amount: 60,
				},
				Event::NewRound {
					starting_block: 25,
					round: 6,
					selected_collators_number: 5,
					total_balance: 400,
				},
				Event::Rewarded {
					account: 1,
					rewards: 192,
				},
				Event::Rewarded {
					account: 2,
					rewards: 128,
				},
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
				Event::CollatorChosen {
					round: 7,
					collator_account: 1,
					total_exposed_amount: 100,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 2,
					total_exposed_amount: 90,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 3,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 4,
					total_exposed_amount: 70,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 5,
					total_exposed_amount: 60,
				},
				Event::NewRound {
					starting_block: 30,
					round: 7,
					selected_collators_number: 5,
					total_balance: 400,
				},
				Event::CollatorChosen {
					round: 8,
					collator_account: 1,
					total_exposed_amount: 100,
				},
				Event::CollatorChosen {
					round: 8,
					collator_account: 2,
					total_exposed_amount: 90,
				},
				Event::CollatorChosen {
					round: 8,
					collator_account: 3,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 8,
					collator_account: 4,
					total_exposed_amount: 70,
				},
				Event::CollatorChosen {
					round: 8,
					collator_account: 5,
					total_exposed_amount: 60,
				},
				Event::NewRound {
					starting_block: 35,
					round: 8,
					selected_collators_number: 5,
					total_balance: 400,
				},
				Event::Rewarded {
					account: 5,
					rewards: 67,
				},
				Event::Rewarded {
					account: 3,
					rewards: 67,
				},
				Event::Rewarded {
					account: 4,
					rewards: 67,
				},
				Event::Rewarded {
					account: 1,
					rewards: 67,
				},
				Event::Rewarded {
					account: 2,
					rewards: 67,
				},
			];
			expected.append(&mut new2);
			assert_eq_events!(expected);
			// check that distributing rewards clears awarded pts
			assert!(ParachainStaking::awarded_pts(1, 1).is_zero());
			assert!(ParachainStaking::awarded_pts(4, 1).is_zero());
			assert!(ParachainStaking::awarded_pts(4, 2).is_zero());
			assert!(ParachainStaking::awarded_pts(6, 1).is_zero());
			assert!(ParachainStaking::awarded_pts(6, 2).is_zero());
			assert!(ParachainStaking::awarded_pts(6, 3).is_zero());
			assert!(ParachainStaking::awarded_pts(6, 4).is_zero());
			assert!(ParachainStaking::awarded_pts(6, 5).is_zero());
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
				Event::CollatorChosen {
					round: 2,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 5,
					round: 2,
					selected_collators_number: 5,
					total_balance: 140,
				},
			];
			assert_eq_events!(expected.clone());
			assert_ok!(ParachainStaking::delegate(Origin::signed(6), 2, 10, 10, 10));
			assert_ok!(ParachainStaking::delegate(Origin::signed(6), 3, 10, 10, 10));
			assert_ok!(ParachainStaking::delegate(Origin::signed(6), 4, 10, 10, 10));
			roll_to(16);
			let mut new = vec![
				Event::Delegation {
					delegator: 6,
					locked_amount: 10,
					candidate: 2,
					delegator_position: DelegatorAdded::AddedToTop { new_total: 50 },
				},
				Event::Delegation {
					delegator: 6,
					locked_amount: 10,
					candidate: 3,
					delegator_position: DelegatorAdded::AddedToTop { new_total: 30 },
				},
				Event::Delegation {
					delegator: 6,
					locked_amount: 10,
					candidate: 4,
					delegator_position: DelegatorAdded::AddedToTop { new_total: 30 },
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 2,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 3,
					total_exposed_amount: 30,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 4,
					total_exposed_amount: 30,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 10,
					round: 3,
					selected_collators_number: 5,
					total_balance: 170,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 2,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 3,
					total_exposed_amount: 30,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 4,
					total_exposed_amount: 30,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 15,
					round: 4,
					selected_collators_number: 5,
					total_balance: 170,
				},
			];
			expected.append(&mut new);
			assert_eq_events!(expected.clone());
			roll_to(21);
			assert_ok!(ParachainStaking::delegate(Origin::signed(7), 2, 80, 10, 10));
			assert_ok!(ParachainStaking::delegate(
				Origin::signed(10),
				2,
				10,
				10,
				10
			),);
			roll_to(26);
			let mut new2 = vec![
				Event::CollatorChosen {
					round: 5,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 2,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 3,
					total_exposed_amount: 30,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 4,
					total_exposed_amount: 30,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 20,
					round: 5,
					selected_collators_number: 5,
					total_balance: 170,
				},
				Event::Delegation {
					delegator: 7,
					locked_amount: 80,
					candidate: 2,
					delegator_position: DelegatorAdded::AddedToTop { new_total: 130 },
				},
				Event::Delegation {
					delegator: 10,
					locked_amount: 10,
					candidate: 2,
					delegator_position: DelegatorAdded::AddedToBottom,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 2,
					total_exposed_amount: 130,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 3,
					total_exposed_amount: 30,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 4,
					total_exposed_amount: 30,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 25,
					round: 6,
					selected_collators_number: 5,
					total_balance: 250,
				},
			];
			expected.append(&mut new2);
			assert_eq_events!(expected.clone());
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(2),
				5
			));
			assert_last_event!(MetaEvent::ParachainStaking(Event::CandidateScheduledExit {
				exit_allowed_round: 6,
				candidate: 2,
				scheduled_exit: 8
			}));
			roll_to(31);
			let mut new3 = vec![
				Event::CandidateScheduledExit {
					exit_allowed_round: 6,
					candidate: 2,
					scheduled_exit: 8,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 3,
					total_exposed_amount: 30,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 4,
					total_exposed_amount: 30,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 30,
					round: 7,
					selected_collators_number: 4,
					total_balance: 120,
				},
			];
			expected.append(&mut new3);
			assert_eq_events!(expected);
			// verify that delegations are removed after collator leaves, not before
			assert_eq!(ParachainStaking::delegator_state(7).unwrap().total, 90);
			assert_eq!(
				ParachainStaking::delegator_state(7)
					.unwrap()
					.delegations
					.0
					.len(),
				2usize
			);
			assert_eq!(ParachainStaking::delegator_state(6).unwrap().total, 40);
			assert_eq!(
				ParachainStaking::delegator_state(6)
					.unwrap()
					.delegations
					.0
					.len(),
				4usize
			);
			assert_eq!(Balances::reserved_balance(&6), 40);
			assert_eq!(Balances::reserved_balance(&7), 90);
			assert_eq!(Balances::free_balance(&6), 60);
			assert_eq!(Balances::free_balance(&7), 10);
			roll_to(40);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				Origin::signed(2),
				2,
				5
			));
			assert_eq!(ParachainStaking::delegator_state(7).unwrap().total, 10);
			assert_eq!(ParachainStaking::delegator_state(6).unwrap().total, 30);
			assert_eq!(
				ParachainStaking::delegator_state(7)
					.unwrap()
					.delegations
					.0
					.len(),
				1usize
			);
			assert_eq!(
				ParachainStaking::delegator_state(6)
					.unwrap()
					.delegations
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
// The test verifies that the pending revoke request is removed by 2's exit so there is no dangling
// revoke request after 2 exits
fn execute_leave_candidate_removes_delegations() {
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 100), (3, 100), (4, 100)])
		.with_candidates(vec![(1, 20), (2, 20)])
		.with_delegations(vec![(3, 1, 10), (3, 2, 10), (4, 1, 10), (4, 2, 10)])
		.build()
		.execute_with(|| {
			// Verifies the revocation request is initially empty
			assert!(!ParachainStaking::delegation_scheduled_requests(&2)
				.iter()
				.any(|x| x.delegator == 3));

			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(2),
				2
			));
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(3),
				2
			));
			// Verifies the revocation request is present
			assert!(ParachainStaking::delegation_scheduled_requests(&2)
				.iter()
				.any(|x| x.delegator == 3));

			roll_to(16);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				Origin::signed(2),
				2,
				2
			));
			// Verifies the revocation request is again empty
			assert!(!ParachainStaking::delegation_scheduled_requests(&2)
				.iter()
				.any(|x| x.delegator == 3));
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
				Event::CollatorChosen {
					round: 2,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 5,
					round: 2,
					selected_collators_number: 5,
					total_balance: 140,
				},
			];
			assert_eq_events!(expected.clone());
			// ~ set block author as 1 for all blocks this round
			set_author(2, 1, 100);
			roll_to(16);
			// distribute total issuance to collator 1 and its delegators 6, 7, 19
			let mut new = vec![
				Event::CollatorChosen {
					round: 3,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 10,
					round: 3,
					selected_collators_number: 5,
					total_balance: 140,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 15,
					round: 4,
					selected_collators_number: 5,
					total_balance: 140,
				},
				Event::Rewarded {
					account: 1,
					rewards: 26,
				},
				Event::Rewarded {
					account: 6,
					rewards: 8,
				},
				Event::Rewarded {
					account: 7,
					rewards: 8,
				},
				Event::Rewarded {
					account: 10,
					rewards: 8,
				},
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
				ParachainStaking::schedule_leave_delegators(Origin::signed(66)),
				Error::<Test>::DelegatorDNE
			);
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				6
			)));
			// fast forward to block in which delegator 6 exit executes
			roll_to(25);
			assert_ok!(ParachainStaking::execute_leave_delegators(
				Origin::signed(6),
				6,
				10
			));
			// keep paying 6 (note: inflation is in terms of total issuance so that's why 1 is 21)
			let mut new2 = vec![
				Event::DelegatorExitScheduled {
					round: 4,
					delegator: 6,
					scheduled_exit: 6,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 5,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 20,
					round: 5,
					selected_collators_number: 5,
					total_balance: 140,
				},
				Event::Rewarded {
					account: 1,
					rewards: 27,
				},
				Event::Rewarded {
					account: 6,
					rewards: 8,
				},
				Event::Rewarded {
					account: 7,
					rewards: 8,
				},
				Event::Rewarded {
					account: 10,
					rewards: 8,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 25,
					round: 6,
					selected_collators_number: 5,
					total_balance: 140,
				},
				Event::Rewarded {
					account: 1,
					rewards: 29,
				},
				Event::Rewarded {
					account: 6,
					rewards: 9,
				},
				Event::Rewarded {
					account: 7,
					rewards: 9,
				},
				Event::Rewarded {
					account: 10,
					rewards: 9,
				},
				Event::DelegatorLeftCandidate {
					delegator: 6,
					candidate: 1,
					unstaked_amount: 10,
					total_candidate_staked: 40,
				},
				Event::DelegatorLeft {
					delegator: 6,
					unstaked_amount: 10,
				},
			];
			expected.append(&mut new2);
			assert_eq_events!(expected.clone());
			// 6 won't be paid for this round because they left already
			set_author(7, 1, 100);
			roll_to(35);
			// keep paying 6
			let mut new3 = vec![
				Event::CollatorChosen {
					round: 7,
					collator_account: 1,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 7,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 30,
					round: 7,
					selected_collators_number: 5,
					total_balance: 130,
				},
				Event::Rewarded {
					account: 1,
					rewards: 30,
				},
				Event::Rewarded {
					account: 7,
					rewards: 9,
				},
				Event::Rewarded {
					account: 10,
					rewards: 9,
				},
				Event::CollatorChosen {
					round: 8,
					collator_account: 1,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 8,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 8,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 8,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 8,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 35,
					round: 8,
					selected_collators_number: 5,
					total_balance: 130,
				},
				Event::Rewarded {
					account: 1,
					rewards: 31,
				},
				Event::Rewarded {
					account: 7,
					rewards: 10,
				},
				Event::Rewarded {
					account: 10,
					rewards: 10,
				},
			];
			expected.append(&mut new3);
			assert_eq_events!(expected.clone());
			set_author(8, 1, 100);
			roll_to(40);
			// no more paying 6
			let mut new4 = vec![
				Event::CollatorChosen {
					round: 9,
					collator_account: 1,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 9,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 9,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 9,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 9,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 40,
					round: 9,
					selected_collators_number: 5,
					total_balance: 130,
				},
				Event::Rewarded {
					account: 1,
					rewards: 38,
				},
				Event::Rewarded {
					account: 7,
					rewards: 12,
				},
				Event::Rewarded {
					account: 10,
					rewards: 12,
				},
			];
			expected.append(&mut new4);
			assert_eq_events!(expected.clone());
			set_author(9, 1, 100);
			assert_ok!(ParachainStaking::delegate(Origin::signed(8), 1, 10, 10, 10));
			roll_to(45);
			// new delegation is not rewarded yet
			let mut new5 = vec![
				Event::Delegation {
					delegator: 8,
					locked_amount: 10,
					candidate: 1,
					delegator_position: DelegatorAdded::AddedToTop { new_total: 50 },
				},
				Event::CollatorChosen {
					round: 10,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 10,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 10,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 10,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 10,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 45,
					round: 10,
					selected_collators_number: 5,
					total_balance: 140,
				},
				Event::Rewarded {
					account: 1,
					rewards: 39,
				},
				Event::Rewarded {
					account: 7,
					rewards: 13,
				},
				Event::Rewarded {
					account: 10,
					rewards: 13,
				},
			];
			expected.append(&mut new5);
			assert_eq_events!(expected.clone());
			set_author(10, 1, 100);
			roll_to(50);
			// new delegation not rewarded yet
			let mut new6 = vec![
				Event::CollatorChosen {
					round: 11,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 11,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 11,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 11,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 11,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 50,
					round: 11,
					selected_collators_number: 5,
					total_balance: 140,
				},
				Event::Rewarded {
					account: 1,
					rewards: 41,
				},
				Event::Rewarded {
					account: 7,
					rewards: 14,
				},
				Event::Rewarded {
					account: 10,
					rewards: 14,
				},
			];
			expected.append(&mut new6);
			assert_eq_events!(expected.clone());
			roll_to(55);
			// new delegation is rewarded for first time
			// 2 rounds after joining (`RewardPaymentDelay` = 2)
			let mut new7 = vec![
				Event::CollatorChosen {
					round: 12,
					collator_account: 1,
					total_exposed_amount: 50,
				},
				Event::CollatorChosen {
					round: 12,
					collator_account: 2,
					total_exposed_amount: 40,
				},
				Event::CollatorChosen {
					round: 12,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 12,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 12,
					collator_account: 5,
					total_exposed_amount: 10,
				},
				Event::NewRound {
					starting_block: 55,
					round: 12,
					selected_collators_number: 5,
					total_balance: 140,
				},
				Event::Rewarded {
					account: 1,
					rewards: 38,
				},
				Event::Rewarded {
					account: 7,
					rewards: 12,
				},
				Event::Rewarded {
					account: 10,
					rewards: 12,
				},
				Event::Rewarded {
					account: 8,
					rewards: 12,
				},
			];
			expected.append(&mut new7);
			assert_eq_events!(expected);
		});
}

#[test]
fn bottom_delegations_are_empty_when_top_delegations_not_full() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 10), (3, 10), (4, 10), (5, 10)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			// no top delegators => no bottom delegators
			let top_delegations = ParachainStaking::top_delegations(1).unwrap();
			let bottom_delegations = ParachainStaking::bottom_delegations(1).unwrap();
			assert!(top_delegations.delegations.is_empty());
			assert!(bottom_delegations.delegations.is_empty());
			// 1 delegator => 1 top delegator, 0 bottom delegators
			assert_ok!(ParachainStaking::delegate(Origin::signed(2), 1, 10, 10, 10));
			let top_delegations = ParachainStaking::top_delegations(1).unwrap();
			let bottom_delegations = ParachainStaking::bottom_delegations(1).unwrap();
			assert_eq!(top_delegations.delegations.len(), 1usize);
			assert!(bottom_delegations.delegations.is_empty());
			// 2 delegators => 2 top delegators, 0 bottom delegators
			assert_ok!(ParachainStaking::delegate(Origin::signed(3), 1, 10, 10, 10));
			let top_delegations = ParachainStaking::top_delegations(1).unwrap();
			let bottom_delegations = ParachainStaking::bottom_delegations(1).unwrap();
			assert_eq!(top_delegations.delegations.len(), 2usize);
			assert!(bottom_delegations.delegations.is_empty());
			// 3 delegators => 3 top delegators, 0 bottom delegators
			assert_ok!(ParachainStaking::delegate(Origin::signed(4), 1, 10, 10, 10));
			let top_delegations = ParachainStaking::top_delegations(1).unwrap();
			let bottom_delegations = ParachainStaking::bottom_delegations(1).unwrap();
			assert_eq!(top_delegations.delegations.len(), 3usize);
			assert!(bottom_delegations.delegations.is_empty());
			// 4 delegators => 4 top delegators, 0 bottom delegators
			assert_ok!(ParachainStaking::delegate(Origin::signed(5), 1, 10, 10, 10));
			let top_delegations = ParachainStaking::top_delegations(1).unwrap();
			let bottom_delegations = ParachainStaking::bottom_delegations(1).unwrap();
			assert_eq!(top_delegations.delegations.len(), 4usize);
			assert!(bottom_delegations.delegations.is_empty());
		});
}

#[test]
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
				let pool = ParachainStaking::candidate_pool();
				for candidate in pool.0 {
					if candidate.owner == account {
						assert_eq!(
							candidate.amount, bond,
							"Candidate Bond {:?} is Not Equal to Expected: {:?}",
							candidate.amount, bond
						);
					}
				}
			}
			// 15 + 16 + 17 + 18 + 20 = 86 (top 4 + self bond)
			is_candidate_pool_bond(1, 86);
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(3),
				1,
				8
			));
			// 3: 11 -> 19 => 3 is in top, bumps out 7
			// 16 + 17 + 18 + 19 + 20 = 90 (top 4 + self bond)
			is_candidate_pool_bond(1, 90);
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(4),
				1,
				8
			));
			// 4: 12 -> 20 => 4 is in top, bumps out 8
			// 17 + 18 + 19 + 20 + 20 = 94 (top 4 + self bond)
			is_candidate_pool_bond(1, 94);
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(10),
				1,
				3
			));
			roll_to(30);
			// 10: 18 -> 15 => 10 bumped to bottom, 8 bumped to top (- 18 + 16 = -2 for count)
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(10),
				10,
				1
			));
			// 16 + 17 + 19 + 20 + 20 = 92 (top 4 + self bond)
			is_candidate_pool_bond(1, 92);
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(9),
				1,
				4
			));
			roll_to(40);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(9),
				9,
				1
			));
			// 15 + 16 + 19 + 20 + 20 = 90 (top 4 + self bond)
			is_candidate_pool_bond(1, 90);
		});
}

#[test]
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
				assert!(ParachainStaking::is_delegator(&i));
			}
			let collator_state = ParachainStaking::candidate_info(1).unwrap();
			// 15 + 16 + 17 + 18 + 20 = 86 (top 4 + self bond)
			assert_eq!(collator_state.total_counted, 86);
			// bump bottom to the top
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(3),
				1,
				8
			));
			assert_event_emitted!(Event::DelegationIncreased {
				delegator: 3,
				candidate: 1,
				amount: 8,
				in_top: true,
			});
			let collator_state = ParachainStaking::candidate_info(1).unwrap();
			// 16 + 17 + 18 + 19 + 20 = 90 (top 4 + self bond)
			assert_eq!(collator_state.total_counted, 90);
			// bump bottom to the top
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(4),
				1,
				8
			));
			assert_event_emitted!(Event::DelegationIncreased {
				delegator: 4,
				candidate: 1,
				amount: 8,
				in_top: true,
			});
			let collator_state = ParachainStaking::candidate_info(1).unwrap();
			// 17 + 18 + 19 + 20 + 20 = 94 (top 4 + self bond)
			assert_eq!(collator_state.total_counted, 94);
			// bump bottom to the top
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(5),
				1,
				8
			));
			assert_event_emitted!(Event::DelegationIncreased {
				delegator: 5,
				candidate: 1,
				amount: 8,
				in_top: true,
			});
			let collator_state = ParachainStaking::candidate_info(1).unwrap();
			// 18 + 19 + 20 + 21 + 20 = 98 (top 4 + self bond)
			assert_eq!(collator_state.total_counted, 98);
			// bump bottom to the top
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(6),
				1,
				8
			));
			assert_event_emitted!(Event::DelegationIncreased {
				delegator: 6,
				candidate: 1,
				amount: 8,
				in_top: true,
			});
			let collator_state = ParachainStaking::candidate_info(1).unwrap();
			// 19 + 20 + 21 + 22 + 20 = 102 (top 4 + self bond)
			assert_eq!(collator_state.total_counted, 102);
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
			let collator1_state = ParachainStaking::candidate_info(1).unwrap();
			// 11 + 12 + 13 + 14 + 20 = 70 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 70);
			// Top delegations are full, new highest delegation is made
			assert_ok!(ParachainStaking::delegate(Origin::signed(7), 1, 15, 10, 10));
			assert_event_emitted!(Event::Delegation {
				delegator: 7,
				locked_amount: 15,
				candidate: 1,
				delegator_position: DelegatorAdded::AddedToTop { new_total: 74 },
			});
			let collator1_state = ParachainStaking::candidate_info(1).unwrap();
			// 12 + 13 + 14 + 15 + 20 = 70 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 74);
			// New delegation is added to the bottom
			assert_ok!(ParachainStaking::delegate(Origin::signed(8), 1, 10, 10, 10));
			assert_event_emitted!(Event::Delegation {
				delegator: 8,
				locked_amount: 10,
				candidate: 1,
				delegator_position: DelegatorAdded::AddedToBottom,
			});
			let collator1_state = ParachainStaking::candidate_info(1).unwrap();
			// 12 + 13 + 14 + 15 + 20 = 70 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 74);
			// 8 increases delegation to the top
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(8),
				1,
				3
			));
			assert_event_emitted!(Event::DelegationIncreased {
				delegator: 8,
				candidate: 1,
				amount: 3,
				in_top: true,
			});
			let collator1_state = ParachainStaking::candidate_info(1).unwrap();
			// 13 + 13 + 14 + 15 + 20 = 75 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 75);
			// 3 increases delegation but stays in bottom
			assert_ok!(ParachainStaking::delegator_bond_more(
				Origin::signed(3),
				1,
				1
			));
			assert_event_emitted!(Event::DelegationIncreased {
				delegator: 3,
				candidate: 1,
				amount: 1,
				in_top: false,
			});
			let collator1_state = ParachainStaking::candidate_info(1).unwrap();
			// 13 + 13 + 14 + 15 + 20 = 75 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 75);
			// 6 decreases delegation but stays in top
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(6),
				1,
				2
			));
			assert_event_emitted!(Event::DelegationDecreaseScheduled {
				delegator: 6,
				candidate: 1,
				amount_to_decrease: 2,
				execute_round: 3,
			});
			roll_to(30);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(6),
				6,
				1
			));
			assert_event_emitted!(Event::DelegationDecreased {
				delegator: 6,
				candidate: 1,
				amount: 2,
				in_top: true,
			});
			let collator1_state = ParachainStaking::candidate_info(1).unwrap();
			// 12 + 13 + 13 + 15 + 20 = 73 (top 4 + self bond)ƒ
			assert_eq!(collator1_state.total_counted, 73);
			// 6 decreases delegation and is bumped to bottom
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(6),
				1,
				1
			));
			assert_event_emitted!(Event::DelegationDecreaseScheduled {
				delegator: 6,
				candidate: 1,
				amount_to_decrease: 1,
				execute_round: 9,
			});
			roll_to(40);
			assert_ok!(ParachainStaking::execute_delegation_request(
				Origin::signed(6),
				6,
				1
			));
			assert_event_emitted!(Event::DelegationDecreased {
				delegator: 6,
				candidate: 1,
				amount: 1,
				in_top: false,
			});
			let collator1_state = ParachainStaking::candidate_info(1).unwrap();
			// 12 + 13 + 13 + 15 + 20 = 73 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 73);
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
				Event::CollatorChosen {
					round: 2,
					collator_account: 1,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 2,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::NewRound {
					starting_block: 5,
					round: 2,
					selected_collators_number: 4,
					total_balance: 80,
				},
			];
			assert_eq_events!(expected);

			roll_to_round_begin(3);
			expected.append(&mut vec![
				Event::CollatorChosen {
					round: 3,
					collator_account: 1,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 2,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 3,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 3,
					collator_account: 4,
					total_exposed_amount: 20,
				},
				Event::NewRound {
					starting_block: 10,
					round: 3,
					selected_collators_number: 4,
					total_balance: 80,
				},
				// rewards will begin immediately following a NewRound
				Event::Rewarded {
					account: 3,
					rewards: 1,
				},
			]);
			assert_eq_events!(expected);

			// roll to the next block where we start round 3; we should have round change and first
			// payout made.
			roll_one_block();
			expected.push(Event::Rewarded {
				account: 4,
				rewards: 2,
			});
			assert_eq_events!(expected);

			roll_one_block();
			expected.push(Event::Rewarded {
				account: 1,
				rewards: 1,
			});
			assert_eq_events!(expected);

			roll_one_block();
			expected.push(Event::Rewarded {
				account: 2,
				rewards: 1,
			});
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
				Event::CollatorChosen {
					round: round,
					collator_account: 1,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: round,
					collator_account: 2,
					total_exposed_amount: 20,
				},
				Event::NewRound {
					starting_block: 5,
					round: round,
					selected_collators_number: 2,
					total_balance: 40,
				},
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
				Event::CollatorChosen {
					round: round,
					collator_account: 1,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: round,
					collator_account: 2,
					total_exposed_amount: 20,
				},
				Event::NewRound {
					starting_block: 10,
					round: round,
					selected_collators_number: 2,
					total_balance: 40,
				},
				Event::Rewarded {
					account: 1,
					rewards: 1,
				},
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
				Event::Rewarded {
					account: 2,
					rewards: 1,
				}, // from previous round
				Event::CollatorChosen {
					round: round,
					collator_account: 1,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: round,
					collator_account: 2,
					total_exposed_amount: 20,
				},
				Event::NewRound {
					starting_block: 15,
					round: round,
					selected_collators_number: 2,
					total_balance: 40,
				},
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
					Event::CollatorChosen {
						round: round as u32,
						collator_account: 1,
						total_exposed_amount: 400,
					},
					Event::CollatorChosen {
						round: round as u32,
						collator_account: 2,
						total_exposed_amount: 400,
					},
					Event::CollatorChosen {
						round: round as u32,
						collator_account: 3,
						total_exposed_amount: 400,
					},
					Event::CollatorChosen {
						round: round as u32,
						collator_account: 4,
						total_exposed_amount: 400,
					},
					Event::NewRound {
						starting_block: (round - 1) * 5,
						round: round as u32,
						selected_collators_number: 4,
						total_balance: 1600,
					},
					// first payout should occur on round change
					Event::Rewarded {
						account: 3,
						rewards: 19,
					},
					Event::Rewarded {
						account: 22,
						rewards: 6,
					},
					Event::Rewarded {
						account: 33,
						rewards: 6,
					},
				];
				assert_eq_last_events!(expected);

				set_round_points(round);

				roll_one_block();
				let expected = vec![
					Event::Rewarded {
						account: 4,
						rewards: 19,
					},
					Event::Rewarded {
						account: 33,
						rewards: 6,
					},
					Event::Rewarded {
						account: 44,
						rewards: 6,
					},
				];
				assert_eq_last_events!(expected);

				roll_one_block();
				let expected = vec![
					Event::Rewarded {
						account: 1,
						rewards: 19,
					},
					Event::Rewarded {
						account: 11,
						rewards: 6,
					},
					Event::Rewarded {
						account: 44,
						rewards: 6,
					},
				];
				assert_eq_last_events!(expected);

				roll_one_block();
				let expected = vec![
					Event::Rewarded {
						account: 2,
						rewards: 19,
					},
					Event::Rewarded {
						account: 11,
						rewards: 6,
					},
					Event::Rewarded {
						account: 22,
						rewards: 6,
					},
				];
				assert_eq_last_events!(expected);

				roll_one_block();
				let expected = vec![
					// we paid everyone out by now, should repeat last event
					Event::Rewarded {
						account: 22,
						rewards: 6,
					},
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
fn patch_incorrect_delegations_sums() {
	ExtBuilder::default()
		.with_balances(vec![(11, 22), (12, 20)])
		.build()
		.execute_with(|| {
			// corrupt top and bottom delegations so totals are incorrect
			let old_top_delegations = Delegations {
				delegations: vec![
					Bond {
						owner: 2,
						amount: 103,
					},
					Bond {
						owner: 3,
						amount: 102,
					},
					Bond {
						owner: 4,
						amount: 101,
					},
					Bond {
						owner: 5,
						amount: 100,
					},
				],
				// should be 406
				total: 453,
			};
			<TopDelegations<Test>>::insert(&1, old_top_delegations);
			let old_bottom_delegations = Delegations {
				delegations: vec![
					Bond {
						owner: 6,
						amount: 25,
					},
					Bond {
						owner: 7,
						amount: 24,
					},
					Bond {
						owner: 8,
						amount: 23,
					},
					Bond {
						owner: 9,
						amount: 22,
					},
				],
				// should be 94
				total: 222,
			};
			<BottomDelegations<Test>>::insert(&1, old_bottom_delegations);
			<CandidateInfo<Test>>::insert(
				&1,
				CandidateMetadata {
					bond: 25,
					delegation_count: 8,
					// 25 + 453 (incorrect), should be 25 + 406 after upgrade
					total_counted: 478,
					lowest_top_delegation_amount: 100,
					highest_bottom_delegation_amount: 25,
					lowest_bottom_delegation_amount: 22,
					top_capacity: CapacityStatus::Full,
					bottom_capacity: CapacityStatus::Full,
					request: None,
					status: CollatorStatus::Active,
				},
			);
			<CandidatePool<Test>>::put(OrderedSet::from(vec![Bond {
				owner: 1,
				amount: 478,
			}]));
			crate::migrations::PatchIncorrectDelegationSums::<Test>::on_runtime_upgrade();
			let top = <TopDelegations<Test>>::get(&1).expect("just updated so exists");
			assert_eq!(top.total, 406);
			let bottom = <BottomDelegations<Test>>::get(&1).expect("just updated so exists");
			assert_eq!(bottom.total, 94);
			let info = <CandidateInfo<Test>>::get(&1).expect("just updated so exists");
			assert_eq!(info.total_counted, 431);
			let only_bond = <CandidatePool<Test>>::get().0[0].clone();
			assert_eq!(only_bond.owner, 1);
			assert_eq!(only_bond.amount, 431);
		});
}

#[test]
/// Kicks extra bottom delegations to force leave delegators if last delegation
fn split_candidate_state_kicks_extra_bottom_delegators_to_exit() {
	#[allow(deprecated)]
	ExtBuilder::default()
		.with_balances(vec![(11, 22), (12, 20)])
		.build()
		.execute_with(|| {
			for i in 11..13 {
				let old_delegator_state = Delegator {
					id: i,
					delegations: OrderedSet::from(vec![
						Bond {
							owner: 1,
							amount: 10,
						},
						Bond {
							owner: 2,
							amount: 10,
						},
					]),
					total: 20,
					less_total: 0,
					status: DelegatorStatus::Active,
				};
				<DelegatorState<Test>>::insert(&i, old_delegator_state);
			}
			assert_ok!(<Test as Config>::Currency::reserve(&11, 22));
			assert_ok!(<Test as Config>::Currency::reserve(&12, 20));
			assert_eq!(Balances::reserved_balance(&11), 22);
			assert_eq!(Balances::reserved_balance(&12), 20);
			for i in 1..3 {
				let old_candidate_state = CollatorCandidate {
					id: i,
					bond: 20,
					delegators: OrderedSet::from(vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
					top_delegations: vec![
						Bond {
							owner: 3,
							amount: 19,
						},
						Bond {
							owner: 4,
							amount: 18,
						},
						Bond {
							owner: 5,
							amount: 17,
						},
						Bond {
							owner: 6,
							amount: 16,
						},
					],
					bottom_delegations: vec![
						Bond {
							owner: 12,
							amount: 10,
						},
						Bond {
							owner: 11,
							amount: 11,
						},
						Bond {
							owner: 10,
							amount: 12,
						},
						Bond {
							owner: 9,
							amount: 13,
						},
						Bond {
							owner: 8,
							amount: 14,
						},
						Bond {
							owner: 7,
							amount: 15,
						},
					],
					total_counted: 90,
					total_backing: 165,
					request: None,
					state: CollatorStatus::Active,
				};
				<CandidateState<Test>>::insert(&i, old_candidate_state);
			}
			// total is 165 * 2 = 330
			<Total<Test>>::put(330);
			assert!(ParachainStaking::is_delegator(&11));
			assert!(ParachainStaking::is_delegator(&12));
			crate::migrations::SplitCandidateStateToDecreasePoV::<Test>::on_runtime_upgrade();
			assert_event_emitted!(Event::DelegationKicked {
				delegator: 11,
				candidate: 1,
				unstaked_amount: 11
			});
			assert_event_emitted!(Event::DelegationKicked {
				delegator: 11,
				candidate: 2,
				unstaked_amount: 11
			});
			assert_event_emitted!(Event::DelegationKicked {
				delegator: 12,
				candidate: 1,
				unstaked_amount: 10
			});
			assert_event_emitted!(Event::DelegationKicked {
				delegator: 12,
				candidate: 2,
				unstaked_amount: 10
			});
			assert_event_emitted!(Event::DelegatorLeft {
				delegator: 12,
				unstaked_amount: 10
			});
			assert_event_emitted!(Event::DelegatorLeft {
				delegator: 11,
				unstaked_amount: 11
			});
			// kicked 11 and 12 and revoked them
			assert_eq!(Balances::free_balance(&11), 22);
			assert_eq!(Balances::free_balance(&12), 20);
			assert!(!ParachainStaking::is_delegator(&11));
			assert!(!ParachainStaking::is_delegator(&12));
			for i in 1..3 {
				let top_delegations = <TopDelegations<Test>>::get(&i).unwrap();
				assert_eq!(top_delegations.total, 70);
				assert_eq!(
					top_delegations.delegations,
					vec![
						Bond {
							owner: 3,
							amount: 19
						},
						Bond {
							owner: 4,
							amount: 18
						},
						Bond {
							owner: 5,
							amount: 17
						},
						Bond {
							owner: 6,
							amount: 16
						}
					]
				);
				let bottom_delegations = <BottomDelegations<Test>>::get(&i).unwrap();
				assert_eq!(bottom_delegations.total, 54);
				assert_eq!(
					bottom_delegations.delegations,
					vec![
						Bond {
							owner: 7,
							amount: 15
						},
						Bond {
							owner: 8,
							amount: 14
						},
						Bond {
							owner: 9,
							amount: 13
						},
						Bond {
							owner: 10,
							amount: 12
						}
					]
				);
				let candidate_metadata = <CandidateInfo<Test>>::get(&i).unwrap();
				assert_eq!(candidate_metadata.top_capacity, CapacityStatus::Full);
				assert_eq!(candidate_metadata.bottom_capacity, CapacityStatus::Full);
				assert_eq!(candidate_metadata.lowest_top_delegation_amount, 16);
				assert_eq!(candidate_metadata.highest_bottom_delegation_amount, 15);
				assert_eq!(candidate_metadata.lowest_bottom_delegation_amount, 12);
			}
		});
}

#[test]
/// Force revokes candidate state
fn split_candidate_state_kicks_extra_bottom_delegations_without_exit() {
	#[allow(deprecated)]
	ExtBuilder::default()
		.with_balances(vec![(11, 32), (12, 30)])
		.build()
		.execute_with(|| {
			for i in 11..13 {
				let old_delegator_state = Delegator {
					id: i,
					delegations: OrderedSet::from(vec![
						Bond {
							owner: 1,
							amount: 10,
						},
						Bond {
							owner: 2,
							amount: 10,
						},
						Bond {
							owner: 3,
							amount: 10,
						},
					]),
					total: 30,
					less_total: 0,
					status: DelegatorStatus::Active,
				};
				<DelegatorState<Test>>::insert(&i, old_delegator_state);
			}
			assert_ok!(<Test as Config>::Currency::reserve(&11, 32));
			assert_ok!(<Test as Config>::Currency::reserve(&12, 30));
			assert_eq!(Balances::reserved_balance(&11), 32);
			assert_eq!(Balances::reserved_balance(&12), 30);
			for i in 1..3 {
				let old_candidate_state = CollatorCandidate {
					id: i,
					bond: 20,
					delegators: OrderedSet::from(vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
					top_delegations: vec![
						Bond {
							owner: 3,
							amount: 19,
						},
						Bond {
							owner: 4,
							amount: 18,
						},
						Bond {
							owner: 5,
							amount: 17,
						},
						Bond {
							owner: 6,
							amount: 16,
						},
					],
					bottom_delegations: vec![
						Bond {
							owner: 12,
							amount: 10,
						},
						Bond {
							owner: 11,
							amount: 11,
						},
						Bond {
							owner: 10,
							amount: 12,
						},
						Bond {
							owner: 9,
							amount: 13,
						},
						Bond {
							owner: 8,
							amount: 14,
						},
						Bond {
							owner: 7,
							amount: 15,
						},
					],
					total_counted: 90,
					total_backing: 165,
					request: None,
					state: CollatorStatus::Active,
				};
				<CandidateState<Test>>::insert(&i, old_candidate_state);
			}
			// total is 165 * 2 + 20 = 330
			<Total<Test>>::put(350);
			assert!(ParachainStaking::is_delegator(&11));
			assert!(ParachainStaking::is_delegator(&12));
			crate::migrations::SplitCandidateStateToDecreasePoV::<Test>::on_runtime_upgrade();
			assert_event_emitted!(Event::DelegationKicked {
				delegator: 11,
				candidate: 1,
				unstaked_amount: 11,
			});
			assert_event_emitted!(Event::DelegationKicked {
				delegator: 11,
				candidate: 2,
				unstaked_amount: 11,
			});
			assert_event_emitted!(Event::DelegationKicked {
				delegator: 12,
				candidate: 1,
				unstaked_amount: 10,
			});
			assert_event_emitted!(Event::DelegationKicked {
				delegator: 12,
				candidate: 2,
				unstaked_amount: 10,
			});
			assert_event_not_emitted!(Event::DelegatorLeft {
				delegator: 12,
				unstaked_amount: 10,
			});
			assert_event_not_emitted!(Event::DelegatorLeft {
				delegator: 11,
				unstaked_amount: 10,
			});
			// kicked 11 and 12 and revoked them
			assert_eq!(Balances::free_balance(&11), 22);
			assert_eq!(Balances::free_balance(&12), 20);
			assert_eq!(Balances::reserved_balance(&11), 10);
			assert_eq!(Balances::reserved_balance(&12), 10);
			assert!(ParachainStaking::is_delegator(&11));
			assert!(ParachainStaking::is_delegator(&12));
			for i in 1..3 {
				let top_delegations = <TopDelegations<Test>>::get(&i).unwrap();
				assert_eq!(top_delegations.total, 70);
				assert_eq!(
					top_delegations.delegations,
					vec![
						Bond {
							owner: 3,
							amount: 19
						},
						Bond {
							owner: 4,
							amount: 18
						},
						Bond {
							owner: 5,
							amount: 17
						},
						Bond {
							owner: 6,
							amount: 16
						}
					]
				);
				let bottom_delegations = <BottomDelegations<Test>>::get(&i).unwrap();
				assert_eq!(bottom_delegations.total, 54);
				assert_eq!(
					bottom_delegations.delegations,
					vec![
						Bond {
							owner: 7,
							amount: 15
						},
						Bond {
							owner: 8,
							amount: 14
						},
						Bond {
							owner: 9,
							amount: 13
						},
						Bond {
							owner: 10,
							amount: 12
						}
					]
				);
				let candidate_metadata = <CandidateInfo<Test>>::get(&i).unwrap();
				assert_eq!(candidate_metadata.top_capacity, CapacityStatus::Full);
				assert_eq!(candidate_metadata.bottom_capacity, CapacityStatus::Full);
				assert_eq!(candidate_metadata.lowest_top_delegation_amount, 16);
				assert_eq!(candidate_metadata.highest_bottom_delegation_amount, 15);
				assert_eq!(candidate_metadata.lowest_bottom_delegation_amount, 12);
			}
		});
}

#[test]
fn split_candidate_state_migrates_empty_delegations_correctly() {
	ExtBuilder::default()
		// .with_balances(vec![(1, 20), (2, 20), (3, 20), (4, 20)])
		// .with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20)])
		.build()
		.execute_with(|| {
			// set candidate state as per commented out lines above
			for i in 1..5 {
				let old_candidate_state = CollatorCandidate {
					id: i,
					bond: 20,
					delegators: OrderedSet::new(),
					top_delegations: Vec::new(),
					bottom_delegations: Vec::new(),
					total_counted: 20,
					total_backing: 20,
					request: None,
					state: CollatorStatus::Active,
				};
				<CandidateState<Test>>::insert(&i, old_candidate_state);
			}
			crate::migrations::SplitCandidateStateToDecreasePoV::<Test>::on_runtime_upgrade();
			for i in 1..5 {
				let top_delegations = <TopDelegations<Test>>::get(&i).unwrap();
				assert_eq!(top_delegations.total, 0);
				assert!(top_delegations.delegations.is_empty());
				let bottom_delegations = <BottomDelegations<Test>>::get(&i).unwrap();
				assert_eq!(bottom_delegations.total, 0);
				assert!(bottom_delegations.delegations.is_empty());
				let candidate_metadata = <CandidateInfo<Test>>::get(&i).unwrap();
				assert_eq!(candidate_metadata.top_capacity, CapacityStatus::Empty);
				assert_eq!(candidate_metadata.bottom_capacity, CapacityStatus::Empty);
				assert_eq!(candidate_metadata.lowest_top_delegation_amount, 0);
				assert_eq!(candidate_metadata.highest_bottom_delegation_amount, 0);
				assert_eq!(candidate_metadata.lowest_bottom_delegation_amount, 0);
			}
		});
}

#[test]
fn split_candidate_state_migrates_partial_top_delegations_correctly() {
	ExtBuilder::default()
		// .with_balances(vec![(1, 20), (2, 20), (3, 20), (4, 20)])
		// .with_candidates(vec![(1, 20), (2, 20)])
		// .with_delegations(vec![(3, 1, 10), (4, 1, 10), (3, 2, 10), (4, 2, 10)])
		.build()
		.execute_with(|| {
			// set up candidate state as per commented out lines above
			for i in 1..3 {
				let old_candidate_state = CollatorCandidate {
					id: i,
					bond: 20,
					delegators: OrderedSet::from(vec![3, 4]),
					top_delegations: vec![
						Bond {
							owner: 3,
							amount: 10,
						},
						Bond {
							owner: 4,
							amount: 10,
						},
					],
					bottom_delegations: Vec::new(),
					total_counted: 40,
					total_backing: 40,
					request: None,
					state: CollatorStatus::Active,
				};
				<CandidateState<Test>>::insert(&i, old_candidate_state);
			}
			crate::migrations::SplitCandidateStateToDecreasePoV::<Test>::on_runtime_upgrade();
			for i in 1..3 {
				let top_delegations = <TopDelegations<Test>>::get(&i).unwrap();
				assert_eq!(top_delegations.total, 20);
				assert_eq!(
					top_delegations.delegations,
					vec![
						Bond {
							owner: 3,
							amount: 10
						},
						Bond {
							owner: 4,
							amount: 10
						}
					]
				);
				let bottom_delegations = <BottomDelegations<Test>>::get(&i).unwrap();
				assert_eq!(bottom_delegations.total, 0);
				assert!(bottom_delegations.delegations.is_empty());
				let candidate_metadata = <CandidateInfo<Test>>::get(&i).unwrap();
				assert_eq!(candidate_metadata.top_capacity, CapacityStatus::Partial);
				assert_eq!(candidate_metadata.bottom_capacity, CapacityStatus::Empty);
				assert_eq!(candidate_metadata.lowest_top_delegation_amount, 10);
				assert_eq!(candidate_metadata.highest_bottom_delegation_amount, 0);
				assert_eq!(candidate_metadata.lowest_bottom_delegation_amount, 0);
			}
		});
}

#[test]
fn split_candidate_state_migrates_full_top_delegations_correctly() {
	ExtBuilder::default()
		// .with_balances(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20), (6, 20)])
		// .with_candidates(vec![(1, 20), (2, 20)])
		// .with_delegations(vec![
		// 	(3, 1, 10),
		// 	(4, 1, 10),
		// 	(5, 1, 10),
		// 	(6, 1, 10),
		// 	(3, 2, 10),
		// 	(4, 2, 10),
		// 	(5, 2, 10),
		// 	(6, 2, 10),
		// ])
		.build()
		.execute_with(|| {
			// set up candidate state as per commented out lines
			for i in 1..3 {
				let old_candidate_state = CollatorCandidate {
					id: i,
					bond: 20,
					delegators: OrderedSet::from(vec![3, 4, 5, 6]),
					top_delegations: vec![
						Bond {
							owner: 3,
							amount: 10,
						},
						Bond {
							owner: 4,
							amount: 10,
						},
						Bond {
							owner: 5,
							amount: 10,
						},
						Bond {
							owner: 6,
							amount: 10,
						},
					],
					bottom_delegations: Vec::new(),
					total_counted: 60,
					total_backing: 60,
					request: None,
					state: CollatorStatus::Active,
				};
				<CandidateState<Test>>::insert(&i, old_candidate_state);
			}
			crate::migrations::SplitCandidateStateToDecreasePoV::<Test>::on_runtime_upgrade();
			for i in 1..3 {
				let top_delegations = <TopDelegations<Test>>::get(&i).unwrap();
				assert_eq!(top_delegations.total, 40);
				assert_eq!(
					top_delegations.delegations,
					vec![
						Bond {
							owner: 3,
							amount: 10
						},
						Bond {
							owner: 4,
							amount: 10
						},
						Bond {
							owner: 5,
							amount: 10
						},
						Bond {
							owner: 6,
							amount: 10
						}
					]
				);
				let bottom_delegations = <BottomDelegations<Test>>::get(&i).unwrap();
				assert_eq!(bottom_delegations.total, 0);
				assert!(bottom_delegations.delegations.is_empty());
				let candidate_metadata = <CandidateInfo<Test>>::get(&i).unwrap();
				assert_eq!(candidate_metadata.top_capacity, CapacityStatus::Full);
				assert_eq!(candidate_metadata.bottom_capacity, CapacityStatus::Empty);
				assert_eq!(candidate_metadata.lowest_top_delegation_amount, 10);
				assert_eq!(candidate_metadata.highest_bottom_delegation_amount, 0);
				assert_eq!(candidate_metadata.lowest_bottom_delegation_amount, 0);
			}
		});
}

#[test]
fn split_candidate_state_migrates_full_top_partial_bottom_delegations_correctly() {
	ExtBuilder::default()
		// .with_balances(vec![
		// 	(1, 20),
		// 	(2, 20),
		// 	(3, 38),
		// 	(4, 36),
		// 	(5, 34),
		// 	(6, 32),
		// 	(7, 30),
		// 	(8, 28),
		// ])
		// .with_candidates(vec![(1, 20), (2, 20)])
		// .with_delegations(vec![
		// 	(3, 1, 19),
		// 	(4, 1, 18),
		// 	(5, 1, 17),
		// 	(6, 1, 16),
		// 	(7, 1, 15),
		// 	(8, 1, 14),
		// 	(3, 2, 19),
		// 	(4, 2, 18),
		// 	(5, 2, 17),
		// 	(6, 2, 16),
		// 	(7, 2, 15),
		// 	(8, 2, 14),
		// ])
		.build()
		.execute_with(|| {
			// set up candidate state as per commented out lines
			for i in 1..3 {
				let old_candidate_state = CollatorCandidate {
					id: i,
					bond: 20,
					delegators: OrderedSet::from(vec![3, 4, 5, 6, 7, 8]),
					top_delegations: vec![
						Bond {
							owner: 3,
							amount: 19,
						},
						Bond {
							owner: 4,
							amount: 18,
						},
						Bond {
							owner: 5,
							amount: 17,
						},
						Bond {
							owner: 6,
							amount: 16,
						},
					],
					bottom_delegations: vec![
						Bond {
							owner: 8,
							amount: 14,
						},
						Bond {
							owner: 7,
							amount: 15,
						},
					],
					total_counted: 90,
					total_backing: 119,
					request: None,
					state: CollatorStatus::Active,
				};
				<CandidateState<Test>>::insert(&i, old_candidate_state);
			}
			crate::migrations::SplitCandidateStateToDecreasePoV::<Test>::on_runtime_upgrade();
			for i in 1..3 {
				let top_delegations = <TopDelegations<Test>>::get(&i).unwrap();
				assert_eq!(top_delegations.total, 70);
				assert_eq!(
					top_delegations.delegations,
					vec![
						Bond {
							owner: 3,
							amount: 19
						},
						Bond {
							owner: 4,
							amount: 18
						},
						Bond {
							owner: 5,
							amount: 17
						},
						Bond {
							owner: 6,
							amount: 16
						}
					]
				);
				let bottom_delegations = <BottomDelegations<Test>>::get(&i).unwrap();
				assert_eq!(bottom_delegations.total, 29);
				assert_eq!(
					bottom_delegations.delegations,
					vec![
						Bond {
							owner: 7,
							amount: 15
						},
						Bond {
							owner: 8,
							amount: 14
						}
					]
				);
				let candidate_metadata = <CandidateInfo<Test>>::get(&i).unwrap();
				assert_eq!(candidate_metadata.top_capacity, CapacityStatus::Full);
				assert_eq!(candidate_metadata.bottom_capacity, CapacityStatus::Partial);
				assert_eq!(candidate_metadata.lowest_top_delegation_amount, 16);
				assert_eq!(candidate_metadata.highest_bottom_delegation_amount, 15);
				assert_eq!(candidate_metadata.lowest_bottom_delegation_amount, 14);
			}
		});
}

#[test]
fn split_candidate_state_migrates_full_top_and_bottom_delegations_correctly() {
	ExtBuilder::default()
		// .with_balances(vec![
		// 	(1, 20),
		// 	(2, 20),
		// 	(3, 38),
		// 	(4, 36),
		// 	(5, 34),
		// 	(6, 32),
		// 	(7, 30),
		// 	(8, 28),
		// 	(9, 26),
		// 	(10, 24),
		// ])
		// .with_candidates(vec![(1, 20), (2, 20)])
		// .with_delegations(vec![
		// 	(3, 1, 19),
		// 	(4, 1, 18),
		// 	(5, 1, 17),
		// 	(6, 1, 16),
		// 	(7, 1, 15),
		// 	(8, 1, 14),
		// 	(9, 1, 13),
		// 	(10, 1, 12),
		// 	(3, 2, 19),
		// 	(4, 2, 18),
		// 	(5, 2, 17),
		// 	(6, 2, 16),
		// 	(7, 2, 15),
		// 	(8, 2, 14),
		// 	(9, 2, 13),
		// 	(10, 2, 12),
		// ])
		.build()
		.execute_with(|| {
			// set up candidate state as per commented out lines
			for i in 1..3 {
				let old_candidate_state = CollatorCandidate {
					id: i,
					bond: 20,
					delegators: OrderedSet::from(vec![3, 4, 5, 6, 7, 8, 9, 10]),
					top_delegations: vec![
						Bond {
							owner: 3,
							amount: 19,
						},
						Bond {
							owner: 4,
							amount: 18,
						},
						Bond {
							owner: 5,
							amount: 17,
						},
						Bond {
							owner: 6,
							amount: 16,
						},
					],
					bottom_delegations: vec![
						Bond {
							owner: 10,
							amount: 12,
						},
						Bond {
							owner: 9,
							amount: 13,
						},
						Bond {
							owner: 8,
							amount: 14,
						},
						Bond {
							owner: 7,
							amount: 15,
						},
					],
					total_counted: 90,
					total_backing: 144,
					request: None,
					state: CollatorStatus::Active,
				};
				<CandidateState<Test>>::insert(&i, old_candidate_state);
			}
			crate::migrations::SplitCandidateStateToDecreasePoV::<Test>::on_runtime_upgrade();
			for i in 1..3 {
				let top_delegations = <TopDelegations<Test>>::get(&i).unwrap();
				assert_eq!(top_delegations.total, 70);
				assert_eq!(
					top_delegations.delegations,
					vec![
						Bond {
							owner: 3,
							amount: 19
						},
						Bond {
							owner: 4,
							amount: 18
						},
						Bond {
							owner: 5,
							amount: 17
						},
						Bond {
							owner: 6,
							amount: 16
						}
					]
				);
				let bottom_delegations = <BottomDelegations<Test>>::get(&i).unwrap();
				assert_eq!(bottom_delegations.total, 54);
				assert_eq!(
					bottom_delegations.delegations,
					vec![
						Bond {
							owner: 7,
							amount: 15
						},
						Bond {
							owner: 8,
							amount: 14
						},
						Bond {
							owner: 9,
							amount: 13
						},
						Bond {
							owner: 10,
							amount: 12
						}
					]
				);
				let candidate_metadata = <CandidateInfo<Test>>::get(&i).unwrap();
				assert_eq!(candidate_metadata.top_capacity, CapacityStatus::Full);
				assert_eq!(candidate_metadata.bottom_capacity, CapacityStatus::Full);
				assert_eq!(candidate_metadata.lowest_top_delegation_amount, 16);
				assert_eq!(candidate_metadata.highest_bottom_delegation_amount, 15);
				assert_eq!(candidate_metadata.lowest_bottom_delegation_amount, 12);
			}
		});
}

// #[test]
// fn remove_exit_queue_migration_migrates_leaving_candidates() {
// 	use crate::pallet::ExitQueue2;
// 	use crate::set::*;
// 	use crate::*;
// 	ExtBuilder::default()
// 		.with_balances(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
// 		.with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
// 		.build()
// 		.execute_with(|| {
// 			// prepare leaving state for all 5 candidates before the migration
// 			for i in 1..6 {
// 				// manually change the CollatorState2 status
// 				<CollatorState2<Test>>::insert(
// 					i,
// 					Collator2 {
// 						id: i,
// 						bond: 20,
// 						nominators: OrderedSet::new(),
// 						top_nominators: Vec::new(),
// 						bottom_nominators: Vec::new(),
// 						total_counted: 20,
// 						total_backing: 20,
// 						// set to leaving
// 						state: CollatorStatus::Leaving(3),
// 					},
// 				);
// 			}
// 			<ExitQueue2<Test>>::put(ExitQ {
// 				candidates: OrderedSet(vec![1, 2, 3, 4, 5]),
// 				candidate_schedule: vec![(1, 3), (2, 3), (3, 3), (4, 3), (5, 3)],
// 				..Default::default()
// 			});
// 			// execute migration
// 			migrations::RemoveExitQueue::<Test>::on_runtime_upgrade();
// 			// check expected candidate state reflects previous state
// 			for i in 1..6 {
// 				assert!(<CollatorState2<Test>>::get(i).is_none());
// 				assert_eq!(
// 					<CandidateState<Test>>::get(i).unwrap().state,
// 					CollatorStatus::Leaving(3)
// 				);
// 			}
// 			// exit queue should be empty
// 			assert_eq!(<ExitQueue2<Test>>::get(), ExitQ::default());
// 		});
// }

// #[test]
// fn remove_exit_queue_migration_migrates_leaving_delegators() {
// 	use crate::pallet::ExitQueue2;
// 	use crate::set::*;
// 	use crate::*;
// 	ExtBuilder::default()
// 		.with_balances(vec![(2, 100), (3, 100), (4, 100), (5, 100), (6, 100)])
// 		.with_candidates(vec![(2, 20)])
// 		.with_delegations(vec![(3, 1, 10), (4, 1, 10), (5, 1, 10), (6, 1, 10)])
// 		.build()
// 		.execute_with(|| {
// 			// prepare leaving state for all 4 delegators before the migration
// 			for i in 3..7 {
// 				<NominatorState2<Test>>::insert(
// 					i,
// 					Nominator2 {
// 						delegations: OrderedSet(vec![Bond {
// 							owner: 1,
// 							amount: 10,
// 						}]),
// 						revocations: OrderedSet::new(),
// 						total: 10,
// 						scheduled_revocations_count: 0u32,
// 						scheduled_revocations_total: 0u32.into(),
// 						status: DelegatorStatus::Leaving(3),
// 					},
// 				);
// 			}
// 			<ExitQueue2<Test>>::put(ExitQ {
// 				nominators_leaving: OrderedSet(vec![3, 4, 5, 6]),
// 				nominator_schedule: vec![(3, None, 3), (4, None, 3), (5, None, 3), (6, None, 3)],
// 				..Default::default()
// 			});
// 			// execute migration
// 			migrations::RemoveExitQueue::<Test>::on_runtime_upgrade();
// 			// check expected delegator state reflects previous state
// 			for i in 3..7 {
// 				assert!(<NominatorState2<Test>>::get(i).is_none());
// 				assert_eq!(
// 					ParachainStaking::delegator_state(i).unwrap().status,
// 					DelegatorStatus::Leaving(3)
// 				);
// 			}
// 			// exit queue should be empty
// 			assert_eq!(<ExitQueue2<Test>>::get(), ExitQ::default());
// 		});
// }

// #[test]
// fn remove_exit_queue_migration_migrates_delegator_revocations() {
// 	use crate::pallet::ExitQueue2;
// 	use crate::set::*;
// 	use crate::*;
// 	ExtBuilder::default()
// 		.with_balances(vec![(2, 100), (3, 100), (4, 100), (5, 100), (6, 100)])
// 		.with_candidates(vec![(2, 20)])
// 		.with_delegations(vec![(3, 1, 10), (4, 1, 10), (5, 1, 10), (6, 1, 10)])
// 		.build()
// 		.execute_with(|| {
// 			// prepare leaving state for all 4 delegators before the migration
// 			for i in 3..7 {
// 				<NominatorState2<Test>>::insert(
// 					i,
// 					Nominator2 {
// 						delegations: OrderedSet(vec![Bond {
// 							owner: 1,
// 							amount: 10,
// 						}]),
// 						revocations: OrderedSet(vec![1]),
// 						total: 10,
// 						scheduled_revocations_count: 1u32,
// 						scheduled_revocations_total: 10u32.into(),
// 						status: DelegatorStatus::Active,
// 					},
// 				);
// 			}
// 			<ExitQueue2<Test>>::put(ExitQ {
// 				nominator_schedule: vec![
// 					(3, Some(1), 3),
// 					(4, Some(1), 3),
// 					(5, Some(1), 3),
// 					(6, Some(1), 3),
// 				],
// 				..Default::default()
// 			});
// 			// execute migration
// 			migrations::RemoveExitQueue::<Test>::on_runtime_upgrade();
// 			// check expected delegator state reflects previous state
// 			for i in 3..7 {
// 				assert!(<NominatorState2<Test>>::get(i).is_none());
// 				assert_eq!(
// 					ParachainStaking::delegator_state(i)
// 						.unwrap()
// 						.requests
// 						.requests
// 						.get(&1),
// 					Some(&DelegationRequest {
// 						collator: 1,
// 						amount: 10,
// 						when_executable: 3,
// 						action: DelegationChange::Revoke
// 					})
// 				);
// 			}
// 			// exit queue should be empty
// 			assert_eq!(<ExitQueue2<Test>>::get(), ExitQ::default());
// 		});
// }

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

#[test]
fn delegation_kicked_from_bottom_removes_pending_request() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 30),
			(2, 29),
			(3, 20),
			(4, 20),
			(5, 20),
			(6, 20),
			(7, 20),
			(8, 20),
			(9, 20),
			(10, 20),
			(11, 30),
		])
		.with_candidates(vec![(1, 30), (11, 30)])
		.with_delegations(vec![
			(2, 1, 19),
			(2, 11, 10), // second delegation so not left after first is kicked
			(3, 1, 20),
			(4, 1, 20),
			(5, 1, 20),
			(6, 1, 20),
			(7, 1, 20),
			(8, 1, 20),
			(9, 1, 20),
		])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			// 10 delegates to full 1 => kicks lowest delegation (2, 19)
			assert_ok!(ParachainStaking::delegate(Origin::signed(10), 1, 20, 8, 0));
			// check the event
			assert_event_emitted!(Event::DelegationKicked {
				delegator: 2,
				candidate: 1,
				unstaked_amount: 19,
			});
			// ensure request DNE
			assert!(!ParachainStaking::delegation_scheduled_requests(&1)
				.iter()
				.any(|x| x.delegator == 2));
		});
}

#[test]
fn no_selected_candidates_defaults_to_last_round_collators() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 30), (3, 30), (4, 30), (5, 30)])
		.with_candidates(vec![(1, 30), (2, 30), (3, 30), (4, 30), (5, 30)])
		.build()
		.execute_with(|| {
			roll_to_round_begin(1);
			// schedule to leave
			for i in 1..6 {
				assert_ok!(ParachainStaking::schedule_leave_candidates(
					Origin::signed(i),
					5
				));
			}
			let old_round = ParachainStaking::round().current;
			let old_selected_candidates = ParachainStaking::selected_candidates();
			let mut old_at_stake_snapshots = Vec::new();
			for account in old_selected_candidates.clone() {
				old_at_stake_snapshots.push(<AtStake<Test>>::get(old_round, account));
			}
			roll_to_round_begin(3);
			// execute leave
			for i in 1..6 {
				assert_ok!(ParachainStaking::execute_leave_candidates(
					Origin::signed(i),
					i,
					0,
				));
			}
			// next round
			roll_to_round_begin(4);
			let new_round = ParachainStaking::round().current;
			// check AtStake matches previous
			let new_selected_candidates = ParachainStaking::selected_candidates();
			assert_eq!(old_selected_candidates, new_selected_candidates);
			let mut index = 0usize;
			for account in new_selected_candidates {
				assert_eq!(
					old_at_stake_snapshots[index],
					<AtStake<Test>>::get(new_round, account)
				);
				index += 1usize;
			}
		});
}

#[test]
fn test_delegator_scheduled_for_revoke_is_rewarded_for_previous_rounds_but_not_for_future() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 40), (3, 20), (4, 20)])
		.with_candidates(vec![(1, 20), (3, 20), (4, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			// preset rewards for rounds 1, 2 and 3
			(1..=3).for_each(|round| set_author(round, 1, 1));

			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			assert_last_event!(MetaEvent::ParachainStaking(
				Event::DelegationRevocationScheduled {
					round: 1,
					delegator: 2,
					candidate: 1,
					scheduled_exit: 3,
				}
			));
			let collator = ParachainStaking::candidate_info(1).expect("candidate must exist");
			assert_eq!(
				1, collator.delegation_count,
				"collator's delegator count was reduced unexpectedly"
			);
			assert_eq!(
				30, collator.total_counted,
				"collator's total was reduced unexpectedly"
			);

			roll_to_round_begin(3);
			assert_eq_last_events!(
				vec![
					Event::<Test>::Rewarded {
						account: 1,
						rewards: 4,
					},
					Event::<Test>::Rewarded {
						account: 2,
						rewards: 1,
					},
				],
				"delegator was not rewarded as intended"
			);

			roll_to_round_begin(4);
			assert_eq_last_events!(
				vec![Event::<Test>::Rewarded {
					account: 1,
					rewards: 4,
				}],
				"delegator was rewarded unexpectedly"
			);
			let collator_snapshot =
				ParachainStaking::at_stake(ParachainStaking::round().current, 1);
			assert_eq!(
				1,
				collator_snapshot.delegations.len(),
				"collator snapshot's delegator count was reduced unexpectedly"
			);
			assert_eq!(
				30, collator_snapshot.total,
				"collator snapshot's total was reduced unexpectedly",
			);
		});
}

#[test]
fn test_delegator_scheduled_for_revoke_is_rewarded_when_request_cancelled() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 40), (3, 20), (4, 20)])
		.with_candidates(vec![(1, 20), (3, 20), (4, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			// preset rewards for rounds 2, 3 and 4
			(2..=4).for_each(|round| set_author(round, 1, 1));

			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(2),
				1
			));
			assert_last_event!(MetaEvent::ParachainStaking(
				Event::DelegationRevocationScheduled {
					round: 1,
					delegator: 2,
					candidate: 1,
					scheduled_exit: 3,
				}
			));
			let collator = ParachainStaking::candidate_info(1).expect("candidate must exist");
			assert_eq!(
				1, collator.delegation_count,
				"collator's delegator count was reduced unexpectedly"
			);
			assert_eq!(
				30, collator.total_counted,
				"collator's total was reduced unexpectedly"
			);

			roll_to_round_begin(2);
			assert_ok!(ParachainStaking::cancel_delegation_request(
				Origin::signed(2),
				1
			));

			roll_to_round_begin(4);
			assert_eq_last_events!(
				vec![Event::<Test>::Rewarded {
					account: 1,
					rewards: 4,
				}],
				"delegator was rewarded unexpectedly",
			);
			let collator_snapshot =
				ParachainStaking::at_stake(ParachainStaking::round().current, 1);
			assert_eq!(
				1,
				collator_snapshot.delegations.len(),
				"collator snapshot's delegator count was reduced unexpectedly"
			);
			assert_eq!(
				30, collator_snapshot.total,
				"collator snapshot's total was reduced unexpectedly",
			);

			roll_to_round_begin(5);
			assert_eq_last_events!(
				vec![
					Event::<Test>::Rewarded {
						account: 1,
						rewards: 4,
					},
					Event::<Test>::Rewarded {
						account: 2,
						rewards: 1,
					},
				],
				"delegator was not rewarded as intended",
			);
		});
}

#[test]
fn test_delegator_scheduled_for_bond_decrease_is_rewarded_for_previous_rounds_but_less_for_future()
{
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 40), (3, 20), (4, 20)])
		.with_candidates(vec![(1, 20), (3, 20), (4, 20)])
		.with_delegations(vec![(2, 1, 20), (2, 3, 10)])
		.build()
		.execute_with(|| {
			// preset rewards for rounds 1, 2 and 3
			(1..=3).for_each(|round| set_author(round, 1, 1));

			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				10,
			));
			assert_last_event!(MetaEvent::ParachainStaking(
				Event::DelegationDecreaseScheduled {
					execute_round: 3,
					delegator: 2,
					candidate: 1,
					amount_to_decrease: 10,
				}
			));
			let collator = ParachainStaking::candidate_info(1).expect("candidate must exist");
			assert_eq!(
				1, collator.delegation_count,
				"collator's delegator count was reduced unexpectedly"
			);
			assert_eq!(
				40, collator.total_counted,
				"collator's total was reduced unexpectedly"
			);

			roll_to_round_begin(3);
			assert_eq_last_events!(
				vec![
					Event::<Test>::Rewarded {
						account: 1,
						rewards: 3,
					},
					Event::<Test>::Rewarded {
						account: 2,
						rewards: 2,
					},
				],
				"delegator was not rewarded as intended"
			);

			roll_to_round_begin(4);
			assert_eq_last_events!(
				vec![
					Event::<Test>::Rewarded {
						account: 1,
						rewards: 3,
					},
					Event::<Test>::Rewarded {
						account: 2,
						rewards: 1,
					},
				],
				"delegator was rewarded unexpectedly"
			);
			let collator_snapshot =
				ParachainStaking::at_stake(ParachainStaking::round().current, 1);
			assert_eq!(
				1,
				collator_snapshot.delegations.len(),
				"collator snapshot's delegator count was reduced unexpectedly"
			);
			assert_eq!(
				40, collator_snapshot.total,
				"collator snapshot's total was reduced unexpectedly",
			);
		});
}

#[test]
fn test_delegator_scheduled_for_bond_decrease_is_rewarded_when_request_cancelled() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 40), (3, 20), (4, 20)])
		.with_candidates(vec![(1, 20), (3, 20), (4, 20)])
		.with_delegations(vec![(2, 1, 20), (2, 3, 10)])
		.build()
		.execute_with(|| {
			// preset rewards for rounds 2, 3 and 4
			(2..=4).for_each(|round| set_author(round, 1, 1));

			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(2),
				1,
				10,
			));
			assert_last_event!(MetaEvent::ParachainStaking(
				Event::DelegationDecreaseScheduled {
					execute_round: 3,
					delegator: 2,
					candidate: 1,
					amount_to_decrease: 10,
				}
			));
			let collator = ParachainStaking::candidate_info(1).expect("candidate must exist");
			assert_eq!(
				1, collator.delegation_count,
				"collator's delegator count was reduced unexpectedly"
			);
			assert_eq!(
				40, collator.total_counted,
				"collator's total was reduced unexpectedly"
			);

			roll_to_round_begin(2);
			assert_ok!(ParachainStaking::cancel_delegation_request(
				Origin::signed(2),
				1
			));

			roll_to_round_begin(4);
			assert_eq_last_events!(
				vec![
					Event::<Test>::Rewarded {
						account: 1,
						rewards: 3,
					},
					Event::<Test>::Rewarded {
						account: 2,
						rewards: 1,
					},
				],
				"delegator was rewarded unexpectedly",
			);
			let collator_snapshot =
				ParachainStaking::at_stake(ParachainStaking::round().current, 1);
			assert_eq!(
				1,
				collator_snapshot.delegations.len(),
				"collator snapshot's delegator count was reduced unexpectedly"
			);
			assert_eq!(
				40, collator_snapshot.total,
				"collator snapshot's total was reduced unexpectedly",
			);

			roll_to_round_begin(5);
			assert_eq_last_events!(
				vec![
					Event::<Test>::Rewarded {
						account: 1,
						rewards: 3,
					},
					Event::<Test>::Rewarded {
						account: 2,
						rewards: 2,
					},
				],
				"delegator was not rewarded as intended",
			);
		});
}

#[test]
fn test_delegator_scheduled_for_leave_is_rewarded_for_previous_rounds_but_not_for_future() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 40), (3, 20), (4, 20)])
		.with_candidates(vec![(1, 20), (3, 20), (4, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			// preset rewards for rounds 1, 2 and 3
			(1..=3).for_each(|round| set_author(round, 1, 1));

			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				2
			),));
			assert_last_event!(MetaEvent::ParachainStaking(Event::DelegatorExitScheduled {
				round: 1,
				delegator: 2,
				scheduled_exit: 3,
			}));
			let collator = ParachainStaking::candidate_info(1).expect("candidate must exist");
			assert_eq!(
				1, collator.delegation_count,
				"collator's delegator count was reduced unexpectedly"
			);
			assert_eq!(
				30, collator.total_counted,
				"collator's total was reduced unexpectedly"
			);

			roll_to_round_begin(3);
			assert_eq_last_events!(
				vec![
					Event::<Test>::Rewarded {
						account: 1,
						rewards: 4,
					},
					Event::<Test>::Rewarded {
						account: 2,
						rewards: 1,
					},
				],
				"delegator was not rewarded as intended"
			);

			roll_to_round_begin(4);
			assert_eq_last_events!(
				vec![Event::<Test>::Rewarded {
					account: 1,
					rewards: 4,
				},],
				"delegator was rewarded unexpectedly"
			);
			let collator_snapshot =
				ParachainStaking::at_stake(ParachainStaking::round().current, 1);
			assert_eq!(
				1,
				collator_snapshot.delegations.len(),
				"collator snapshot's delegator count was reduced unexpectedly"
			);
			assert_eq!(
				30, collator_snapshot.total,
				"collator snapshot's total was reduced unexpectedly",
			);
		});
}

#[test]
fn test_delegator_scheduled_for_leave_is_rewarded_when_request_cancelled() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 40), (3, 20), (4, 20)])
		.with_candidates(vec![(1, 20), (3, 20), (4, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			// preset rewards for rounds 2, 3 and 4
			(2..=4).for_each(|round| set_author(round, 1, 1));

			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				2
			)));
			assert_last_event!(MetaEvent::ParachainStaking(Event::DelegatorExitScheduled {
				round: 1,
				delegator: 2,
				scheduled_exit: 3,
			}));
			let collator = ParachainStaking::candidate_info(1).expect("candidate must exist");
			assert_eq!(
				1, collator.delegation_count,
				"collator's delegator count was reduced unexpectedly"
			);
			assert_eq!(
				30, collator.total_counted,
				"collator's total was reduced unexpectedly"
			);

			roll_to_round_begin(2);
			assert_ok!(ParachainStaking::cancel_leave_delegators(Origin::signed(2)));

			roll_to_round_begin(4);
			assert_eq_last_events!(
				vec![Event::<Test>::Rewarded {
					account: 1,
					rewards: 4,
				},],
				"delegator was rewarded unexpectedly",
			);
			let collator_snapshot =
				ParachainStaking::at_stake(ParachainStaking::round().current, 1);
			assert_eq!(
				1,
				collator_snapshot.delegations.len(),
				"collator snapshot's delegator count was reduced unexpectedly"
			);
			assert_eq!(
				30, collator_snapshot.total,
				"collator snapshot's total was reduced unexpectedly",
			);

			roll_to_round_begin(5);
			assert_eq_last_events!(
				vec![
					Event::<Test>::Rewarded {
						account: 1,
						rewards: 4,
					},
					Event::<Test>::Rewarded {
						account: 2,
						rewards: 1,
					},
				],
				"delegator was not rewarded as intended",
			);
		});
}

#[test]
fn test_delegation_request_exists_returns_false_when_nothing_exists() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert!(!ParachainStaking::delegation_request_exists(&1, &2));
		});
}

#[test]
fn test_delegation_request_exists_returns_true_when_decrease_exists() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			<DelegationScheduledRequests<Test>>::insert(
				1,
				vec![ScheduledRequest {
					delegator: 2,
					when_executable: 3,
					action: DelegationAction::Decrease(5),
				}],
			);
			assert!(ParachainStaking::delegation_request_exists(&1, &2));
		});
}

#[test]
fn test_delegation_request_exists_returns_true_when_revoke_exists() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			<DelegationScheduledRequests<Test>>::insert(
				1,
				vec![ScheduledRequest {
					delegator: 2,
					when_executable: 3,
					action: DelegationAction::Revoke(5),
				}],
			);
			assert!(ParachainStaking::delegation_request_exists(&1, &2));
		});
}

#[test]
fn test_delegation_request_revoke_exists_returns_false_when_nothing_exists() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert!(!ParachainStaking::delegation_request_revoke_exists(&1, &2));
		});
}

#[test]
fn test_delegation_request_revoke_exists_returns_false_when_decrease_exists() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			<DelegationScheduledRequests<Test>>::insert(
				1,
				vec![ScheduledRequest {
					delegator: 2,
					when_executable: 3,
					action: DelegationAction::Decrease(5),
				}],
			);
			assert!(!ParachainStaking::delegation_request_revoke_exists(&1, &2));
		});
}

#[test]
fn test_delegation_request_revoke_exists_returns_true_when_revoke_exists() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			<DelegationScheduledRequests<Test>>::insert(
				1,
				vec![ScheduledRequest {
					delegator: 2,
					when_executable: 3,
					action: DelegationAction::Revoke(5),
				}],
			);
			assert!(ParachainStaking::delegation_request_revoke_exists(&1, &2));
		});
}

#[test]
fn test_hotfix_remove_delegation_requests_exited_candidates_cleans_up() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			// invalid state
			<DelegationScheduledRequests<Test>>::insert(
				2,
				Vec::<ScheduledRequest<u64, u128>>::new(),
			);
			<DelegationScheduledRequests<Test>>::insert(
				3,
				Vec::<ScheduledRequest<u64, u128>>::new(),
			);
			assert_ok!(
				ParachainStaking::hotfix_remove_delegation_requests_exited_candidates(
					Origin::signed(1),
					vec![2, 3, 4] // 4 does not exist, but is OK for idempotency
				)
			);

			assert!(!<DelegationScheduledRequests<Test>>::contains_key(2));
			assert!(!<DelegationScheduledRequests<Test>>::contains_key(3));
		});
}

#[test]
fn test_hotfix_remove_delegation_requests_exited_candidates_cleans_up_only_specified_keys() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			// invalid state
			<DelegationScheduledRequests<Test>>::insert(
				2,
				Vec::<ScheduledRequest<u64, u128>>::new(),
			);
			<DelegationScheduledRequests<Test>>::insert(
				3,
				Vec::<ScheduledRequest<u64, u128>>::new(),
			);
			assert_ok!(
				ParachainStaking::hotfix_remove_delegation_requests_exited_candidates(
					Origin::signed(1),
					vec![2]
				)
			);

			assert!(!<DelegationScheduledRequests<Test>>::contains_key(2));
			assert!(<DelegationScheduledRequests<Test>>::contains_key(3));
		});
}

#[test]
fn test_hotfix_remove_delegation_requests_exited_candidates_errors_when_requests_not_empty() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			// invalid state
			<DelegationScheduledRequests<Test>>::insert(
				2,
				Vec::<ScheduledRequest<u64, u128>>::new(),
			);
			<DelegationScheduledRequests<Test>>::insert(
				3,
				vec![ScheduledRequest {
					delegator: 10,
					when_executable: 1,
					action: DelegationAction::Revoke(10),
				}],
			);

			assert_noop!(
				ParachainStaking::hotfix_remove_delegation_requests_exited_candidates(
					Origin::signed(1),
					vec![2, 3]
				),
				<Error<Test>>::CandidateNotLeaving,
			);
		});
}

#[test]
fn test_hotfix_remove_delegation_requests_exited_candidates_errors_when_candidate_not_exited() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			// invalid state
			<DelegationScheduledRequests<Test>>::insert(
				1,
				Vec::<ScheduledRequest<u64, u128>>::new(),
			);
			assert_noop!(
				ParachainStaking::hotfix_remove_delegation_requests_exited_candidates(
					Origin::signed(1),
					vec![1]
				),
				<Error<Test>>::CandidateNotLeaving,
			);
		});
}

// EventHandler

#[test]
fn note_author_updates_points() {
	use crate::{AwardedPts, Points};
	use pallet_authorship::EventHandler;

	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 30)])
		.with_candidates(vec![(1, 30), (2, 30)])
		.build()
		.execute_with(|| {
			roll_to_round_begin(1);
			ParachainStaking::note_author(1);
			ParachainStaking::note_author(1);
			ParachainStaking::note_author(2);
			let col1_points = <AwardedPts<Test>>::get(1, 1);
			let col2_points = <AwardedPts<Test>>::get(1, 2);
			let total_points = <Points<Test>>::get(1);
			assert_eq!(40, col1_points);
			assert_eq!(20, col2_points);
			assert_eq!(60, total_points);
		});
}

// SessionManager

#[test]
fn new_session_returns_selected_collators() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 30), (3, 30)])
		.with_candidates(vec![(1, 30), (2, 30), (3, 30)])
		.build()
		.execute_with(|| {
			assert_eq!(Some(vec![1, 2, 3]), ParachainStaking::new_session(1));
		});
}

#[test]
fn new_session_returns_none_if_no_selected_collators() {
	ExtBuilder::default()
		.with_balances(vec![])
		.with_candidates(vec![])
		.build()
		.execute_with(|| {
			roll_to_round_begin(2);
			assert_eq!(None, ParachainStaking::new_session(1));
		});
}

// ShouldEndSession

#[test]
fn should_end_session_ties_sessions_to_rounds() {
	ExtBuilder::default()
		.with_balances(vec![])
		.with_candidates(vec![])
		.build()
		.execute_with(|| {
			let mut block = 1;
			assert!(!ParachainStaking::should_end_session(block));
			block = DefaultBlocksPerRound::get() as u64;
			assert!(ParachainStaking::should_end_session(block));
		});
}

// EstimateNextSessionRotation

#[test]
fn average_session_length_is_round_length() {
	ExtBuilder::default()
		.with_balances(vec![])
		.with_candidates(vec![])
		.build()
		.execute_with(|| {
			let round_length = DefaultBlocksPerRound::get() as u64;
			assert_eq!(round_length, ParachainStaking::average_session_length());
		});
}

#[test]
fn estimates_current_session_progress() {
	ExtBuilder::default()
		.with_balances(vec![])
		.with_candidates(vec![])
		.build()
		.execute_with(|| {
			let round_length = DefaultBlocksPerRound::get() as u64;
			let block = 2;
			roll_to(2);
			assert_eq!(
				sp_runtime::Permill::from_rational(block, round_length),
				ParachainStaking::estimate_current_session_progress(block)
					.0
					.unwrap()
			);
		});
}

#[test]
fn estimates_next_session_rotation() {
	ExtBuilder::default()
		.with_balances(vec![])
		.with_candidates(vec![])
		.build()
		.execute_with(|| {
			roll_to_round_begin(1);
			let round_length = DefaultBlocksPerRound::get() as u64;
			assert_eq!(
				round_length,
				ParachainStaking::estimate_next_session_rotation(1)
					.0
					.unwrap()
			);
		});
}
