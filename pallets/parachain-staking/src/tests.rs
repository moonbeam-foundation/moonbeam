// Copyright 2019-2025 PureStake Inc.
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

use crate::auto_compound::{AutoCompoundConfig, AutoCompoundDelegations};
use crate::delegation_requests::{CancelledScheduledRequest, DelegationAction, ScheduledRequest};
use crate::mock::{
	inflation_configs, query_freeze_amount, roll_blocks, roll_to, roll_to_round_begin,
	roll_to_round_end, set_author, set_block_author, AccountId, Balances, BlockNumber, ExtBuilder,
	ParachainStaking, RuntimeEvent, RuntimeOrigin, Test, POINTS_PER_BLOCK, POINTS_PER_ROUND,
};
use crate::{
	assert_events_emitted, assert_events_emitted_match, assert_events_eq, assert_no_events,
	AtStake, Bond, CollatorStatus, DelegationScheduledRequests, DelegatorAdded,
	EnableMarkingOffline, Error, Event, FreezeReason, InflationDistributionInfo, Range,
	WasInactive,
};
use frame_support::traits::{Currency, ExistenceRequirement, WithdrawReasons};
use frame_support::{assert_noop, assert_ok, BoundedVec};
use pallet_balances::{Event as BalancesEvent, PositiveImbalance};
use sp_runtime::{traits::Zero, DispatchError, ModuleError, Perbill, Percent};

mod test_lazy_migration;

#[test]
fn invalid_root_origin_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_total_selected(RuntimeOrigin::signed(45), 6u32),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_noop!(
			ParachainStaking::set_collator_commission(
				RuntimeOrigin::signed(45),
				Perbill::from_percent(5)
			),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_noop!(
			ParachainStaking::set_blocks_per_round(RuntimeOrigin::signed(45), 3u32),
			sp_runtime::DispatchError::BadOrigin
		);
	});
}

// SET TOTAL SELECTED

#[test]
fn set_total_selected_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		// before we can bump total_selected we must bump the blocks per round
		assert_ok!(ParachainStaking::set_blocks_per_round(
			RuntimeOrigin::root(),
			7u32
		));
		roll_blocks(1);
		assert_ok!(ParachainStaking::set_total_selected(
			RuntimeOrigin::root(),
			6u32
		));
		assert_events_eq!(Event::TotalSelectedSet {
			old: 5u32,
			new: 6u32
		});
	});
}

#[test]
fn set_total_selected_fails_if_above_blocks_per_round() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(ParachainStaking::round().length, 5); // test relies on this
		assert_noop!(
			ParachainStaking::set_total_selected(RuntimeOrigin::root(), 6u32),
			Error::<Test>::RoundLengthMustBeGreaterThanTotalSelectedCollators,
		);
	});
}

#[test]
fn set_total_selected_fails_if_above_max_candidates() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(<Test as crate::Config>::MaxCandidates::get(), 200); // test relies on this
		assert_noop!(
			ParachainStaking::set_total_selected(RuntimeOrigin::root(), 201u32),
			Error::<Test>::CannotSetAboveMaxCandidates,
		);
	});
}

#[test]
fn set_total_selected_fails_if_equal_to_blocks_per_round() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_blocks_per_round(
			RuntimeOrigin::root(),
			10u32
		));
		assert_noop!(
			ParachainStaking::set_total_selected(RuntimeOrigin::root(), 10u32),
			Error::<Test>::RoundLengthMustBeGreaterThanTotalSelectedCollators,
		);
	});
}

#[test]
fn set_total_selected_passes_if_below_blocks_per_round() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_blocks_per_round(
			RuntimeOrigin::root(),
			10u32
		));
		assert_ok!(ParachainStaking::set_total_selected(
			RuntimeOrigin::root(),
			9u32
		));
	});
}

#[test]
fn set_blocks_per_round_fails_if_below_total_selected() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_blocks_per_round(
			RuntimeOrigin::root(),
			20u32
		));
		assert_ok!(ParachainStaking::set_total_selected(
			RuntimeOrigin::root(),
			10u32
		));
		assert_noop!(
			ParachainStaking::set_blocks_per_round(RuntimeOrigin::root(), 9u32),
			Error::<Test>::RoundLengthMustBeGreaterThanTotalSelectedCollators,
		);
	});
}

#[test]
fn set_blocks_per_round_fails_if_equal_to_total_selected() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_blocks_per_round(
			RuntimeOrigin::root(),
			10u32
		));
		assert_ok!(ParachainStaking::set_total_selected(
			RuntimeOrigin::root(),
			9u32
		));
		assert_noop!(
			ParachainStaking::set_blocks_per_round(RuntimeOrigin::root(), 9u32),
			Error::<Test>::RoundLengthMustBeGreaterThanTotalSelectedCollators,
		);
	});
}

#[test]
fn set_blocks_per_round_passes_if_above_total_selected() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(ParachainStaking::round().length, 5); // test relies on this
		assert_ok!(ParachainStaking::set_blocks_per_round(
			RuntimeOrigin::root(),
			6u32
		));
	});
}

#[test]
fn set_total_selected_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		// round length must be >= total_selected, so update that first
		assert_ok!(ParachainStaking::set_blocks_per_round(
			RuntimeOrigin::root(),
			10u32
		));

		assert_eq!(ParachainStaking::total_selected(), 5u32);
		assert_ok!(ParachainStaking::set_total_selected(
			RuntimeOrigin::root(),
			6u32
		));
		assert_eq!(ParachainStaking::total_selected(), 6u32);
	});
}

#[test]
fn cannot_set_total_selected_to_current_total_selected() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_total_selected(RuntimeOrigin::root(), 5u32),
			Error::<Test>::NoWritingSameValue
		);
	});
}

#[test]
fn cannot_set_total_selected_below_module_min() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_total_selected(RuntimeOrigin::root(), 4u32),
			Error::<Test>::CannotSetBelowMin
		);
	});
}

// SET COLLATOR COMMISSION

#[test]
fn set_collator_commission_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_collator_commission(
			RuntimeOrigin::root(),
			Perbill::from_percent(5)
		));
		assert_events_eq!(Event::CollatorCommissionSet {
			old: Perbill::from_percent(20),
			new: Perbill::from_percent(5),
		});
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
			RuntimeOrigin::root(),
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
			ParachainStaking::set_collator_commission(
				RuntimeOrigin::root(),
				Perbill::from_percent(20)
			),
			Error::<Test>::NoWritingSameValue
		);
	});
}

// SET BLOCKS PER ROUND

#[test]
fn set_blocks_per_round_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_blocks_per_round(
			RuntimeOrigin::root(),
			6u32
		));
		assert_events_eq!(Event::BlocksPerRoundSet {
			current_round: 1,
			first_block: 0,
			old: 5,
			new: 6,
			new_per_round_inflation_min: Perbill::from_parts(463),
			new_per_round_inflation_ideal: Perbill::from_parts(463),
			new_per_round_inflation_max: Perbill::from_parts(463),
		});
	});
}

#[test]
fn set_blocks_per_round_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(ParachainStaking::round().length, 5);
		assert_ok!(ParachainStaking::set_blocks_per_round(
			RuntimeOrigin::root(),
			6u32
		));
		assert_eq!(ParachainStaking::round().length, 6);
	});
}

#[test]
fn cannot_set_blocks_per_round_below_module_min() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_blocks_per_round(RuntimeOrigin::root(), 2u32),
			Error::<Test>::CannotSetBelowMin
		);
	});
}

#[test]
fn cannot_set_blocks_per_round_to_current_blocks_per_round() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_blocks_per_round(RuntimeOrigin::root(), 5u32),
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
				RuntimeOrigin::root(),
				10u32
			));

			roll_to(10);
			assert_events_emitted!(Event::NewRound {
				starting_block: 10,
				round: 2,
				selected_collators_number: 1,
				total_balance: 20
			},);
			roll_to(17);
			assert_ok!(ParachainStaking::set_blocks_per_round(
				RuntimeOrigin::root(),
				6u32
			));
			roll_to(18);
			assert_events_emitted!(Event::NewRound {
				starting_block: 18,
				round: 3,
				selected_collators_number: 1,
				total_balance: 20
			});
		});
}

// ~~ MONETARY GOVERNANCE ~~

#[test]
fn invalid_monetary_origin_fails() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_staking_expectations(
				RuntimeOrigin::signed(45),
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
				RuntimeOrigin::signed(45),
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
				RuntimeOrigin::signed(45),
				Range {
					min: Perbill::from_percent(3),
					ideal: Perbill::from_percent(4),
					max: Perbill::from_percent(5)
				}
			),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_noop!(
			ParachainStaking::set_parachain_bond_account(RuntimeOrigin::signed(45), 11),
			sp_runtime::DispatchError::BadOrigin
		);
		assert_noop!(
			ParachainStaking::set_parachain_bond_reserve_percent(
				RuntimeOrigin::signed(45),
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
			RuntimeOrigin::root(),
			Range {
				min: 3u128,
				ideal: 4u128,
				max: 5u128,
			}
		));
		assert_events_eq!(Event::StakeExpectationsSet {
			expect_min: 3u128,
			expect_ideal: 4u128,
			expect_max: 5u128,
		});
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
			RuntimeOrigin::root(),
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
				RuntimeOrigin::root(),
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
			RuntimeOrigin::root(),
			Range {
				min: 3u128,
				ideal: 4u128,
				max: 5u128
			}
		));
		assert_noop!(
			ParachainStaking::set_staking_expectations(
				RuntimeOrigin::root(),
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
			RuntimeOrigin::root(),
			Range { min, ideal, max }
		));
		assert_events_eq!(Event::InflationSet {
			annual_min: min,
			annual_ideal: ideal,
			annual_max: max,
			round_min: Perbill::from_parts(29),
			round_ideal: Perbill::from_parts(38),
			round_max: Perbill::from_parts(47),
		});
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
			RuntimeOrigin::root(),
			Range { min, ideal, max }
		),);
		assert_eq!(
			ParachainStaking::inflation_config().annual,
			Range { min, ideal, max }
		);
		assert_eq!(
			ParachainStaking::inflation_config().round,
			Range {
				min: Perbill::from_parts(29),
				ideal: Perbill::from_parts(38),
				max: Perbill::from_parts(47)
			}
		);
	});
}

#[test]
fn cannot_set_invalid_inflation() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_inflation(
				RuntimeOrigin::root(),
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
			RuntimeOrigin::root(),
			Range { min, ideal, max }
		),);
		assert_noop!(
			ParachainStaking::set_inflation(RuntimeOrigin::root(), Range { min, ideal, max }),
			Error::<Test>::NoWritingSameValue
		);
	});
}

// SET PARACHAIN BOND ACCOUNT

#[test]
fn set_parachain_bond_account_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_parachain_bond_account(
			RuntimeOrigin::root(),
			11
		));
		assert_events_eq!(Event::InflationDistributionConfigUpdated {
			old: inflation_configs(0, 30, 0, 0),
			new: inflation_configs(11, 30, 0, 0),
		});
	});
}

#[test]
fn set_parachain_bond_account_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			ParachainStaking::inflation_distribution_info().0[0].account,
			0
		);
		assert_ok!(ParachainStaking::set_parachain_bond_account(
			RuntimeOrigin::root(),
			11
		));
		assert_eq!(
			ParachainStaking::inflation_distribution_info().0[0].account,
			11
		);
	});
}

// SET PARACHAIN BOND RESERVE PERCENT

#[test]
fn set_parachain_bond_reserve_percent_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_parachain_bond_reserve_percent(
			RuntimeOrigin::root(),
			Percent::from_percent(50)
		));
		assert_events_eq!(Event::InflationDistributionConfigUpdated {
			old: inflation_configs(0, 30, 0, 0),
			new: inflation_configs(0, 50, 0, 0),
		});
	});
}

#[test]
fn set_parachain_bond_reserve_percent_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			ParachainStaking::inflation_distribution_info().0[0].percent,
			Percent::from_percent(30)
		);
		assert_ok!(ParachainStaking::set_parachain_bond_reserve_percent(
			RuntimeOrigin::root(),
			Percent::from_percent(50)
		));
		assert_eq!(
			ParachainStaking::inflation_distribution_info().0[0].percent,
			Percent::from_percent(50)
		);
	});
}

#[test]
fn cannot_set_same_parachain_bond_reserve_percent() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_parachain_bond_reserve_percent(
				RuntimeOrigin::root(),
				Percent::from_percent(30)
			),
			Error::<Test>::NoWritingSameValue
		);
	});
}

// Set Inflation Distribution Config

#[test]
fn set_inflation_distribution_config_fails_with_normal_origin() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::set_inflation_distribution_config(
				RuntimeOrigin::signed(45),
				inflation_configs(1, 30, 2, 20)
			),
			sp_runtime::DispatchError::BadOrigin,
		);
	});
}

#[test]
fn set_inflation_distribution_config_event_emits_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_inflation_distribution_config(
			RuntimeOrigin::root(),
			inflation_configs(1, 30, 2, 20),
		));
		assert_events_eq!(Event::InflationDistributionConfigUpdated {
			old: inflation_configs(0, 30, 0, 0),
			new: inflation_configs(1, 30, 2, 20),
		});
		roll_blocks(1);
		assert_ok!(ParachainStaking::set_inflation_distribution_config(
			RuntimeOrigin::root(),
			inflation_configs(5, 10, 6, 5),
		));
		assert_events_eq!(Event::InflationDistributionConfigUpdated {
			old: inflation_configs(1, 30, 2, 20),
			new: inflation_configs(5, 10, 6, 5),
		});
	});
}

#[test]
fn set_inflation_distribution_config_storage_updates_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			InflationDistributionInfo::<Test>::get(),
			inflation_configs(0, 30, 0, 0),
		);
		assert_ok!(ParachainStaking::set_inflation_distribution_config(
			RuntimeOrigin::root(),
			inflation_configs(5, 10, 6, 5),
		));
		assert_eq!(
			InflationDistributionInfo::<Test>::get(),
			inflation_configs(5, 10, 6, 5),
		);
		assert_ok!(ParachainStaking::set_inflation_distribution_config(
			RuntimeOrigin::root(),
			inflation_configs(1, 30, 2, 20),
		));
		assert_eq!(
			InflationDistributionInfo::<Test>::get(),
			inflation_configs(1, 30, 2, 20),
		);
	});
}

#[test]
fn cannot_set_same_inflation_distribution_config() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ParachainStaking::set_inflation_distribution_config(
			RuntimeOrigin::root(),
			inflation_configs(1, 30, 2, 20),
		));
		assert_noop!(
			ParachainStaking::set_inflation_distribution_config(
				RuntimeOrigin::root(),
				inflation_configs(1, 30, 2, 20)
			),
			Error::<Test>::NoWritingSameValue,
		);
	});
}

#[test]
fn sum_of_inflation_distribution_config_percentages_must_lte_100() {
	ExtBuilder::default().build().execute_with(|| {
		let invalid_values: Vec<(u8, u8)> = vec![
			(20, 90),
			(90, 20),
			(50, 51),
			(100, 1),
			(1, 100),
			(55, 55),
			(2, 99),
			(100, 100),
		];

		for (pbr_percentage, treasury_percentage) in invalid_values {
			assert_noop!(
				ParachainStaking::set_inflation_distribution_config(
					RuntimeOrigin::root(),
					inflation_configs(1, pbr_percentage, 2, treasury_percentage),
				),
				Error::<Test>::TotalInflationDistributionPercentExceeds100,
			);
		}

		let valid_values: Vec<(u8, u8)> = vec![
			(0, 100),
			(100, 0),
			(0, 0),
			(100, 0),
			(0, 100),
			(50, 50),
			(1, 99),
			(99, 1),
			(1, 1),
			(10, 20),
			(34, 32),
			(15, 10),
		];

		for (pbr_percentage, treasury_percentage) in valid_values {
			assert_ok!(ParachainStaking::set_inflation_distribution_config(
				RuntimeOrigin::root(),
				inflation_configs(1, pbr_percentage, 2, treasury_percentage),
			));
		}
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
				RuntimeOrigin::signed(1),
				10u128,
				0u32
			));
			assert_events_eq!(Event::JoinedCollatorCandidates {
				account: 1,
				amount_locked: 10u128,
				new_total_amt_locked: 10u128,
			});
		});
}

#[test]
fn join_candidates_reserves_balance() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_eq!(ParachainStaking::get_collator_stakable_free_balance(&1), 10);
			assert_ok!(ParachainStaking::join_candidates(
				RuntimeOrigin::signed(1),
				10u128,
				0u32
			));
			assert_eq!(ParachainStaking::get_collator_stakable_free_balance(&1), 0);
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
				RuntimeOrigin::signed(1),
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
				RuntimeOrigin::signed(1),
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
				RuntimeOrigin::signed(1),
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
				ParachainStaking::join_candidates(RuntimeOrigin::signed(1), 11u128, 100u32),
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
				ParachainStaking::join_candidates(RuntimeOrigin::signed(2), 10u128, 1u32),
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
				ParachainStaking::join_candidates(RuntimeOrigin::signed(1), 9u128, 100u32),
				Error::<Test>::CandidateBondBelowMin
			);
		});
}

#[test]
fn can_force_join_candidates_without_min_bond() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::force_join_candidates(
				RuntimeOrigin::root(),
				1,
				9,
				100u32
			));
			assert_events_eq!(Event::JoinedCollatorCandidates {
				account: 1,
				amount_locked: 9u128,
				new_total_amt_locked: 9u128,
			});
		});
}

#[test]
fn cannot_join_candidates_with_more_than_available_balance() {
	ExtBuilder::default()
		.with_balances(vec![(1, 500)])
		.build()
		.execute_with(|| {
			assert_noop!(
				ParachainStaking::join_candidates(RuntimeOrigin::signed(1), 501u128, 100u32),
				DispatchError::Module(ModuleError {
					index: 2,
					error: [8, 0, 0, 0],
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
					ParachainStaking::join_candidates(RuntimeOrigin::signed(6), 20, i),
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
					RuntimeOrigin::signed(i),
					20,
					count
				));
				count += 1u32;
			}
		});
}

#[test]
fn join_candidates_fails_if_above_max_candidate_count() {
	let mut candidates = vec![];
	for i in 1..=crate::mock::MaxCandidates::get() {
		candidates.push((i as u64, 80));
	}

	let new_candidate = crate::mock::MaxCandidates::get() as u64 + 1;
	let mut balances = candidates.clone();
	balances.push((new_candidate, 100));

	ExtBuilder::default()
		.with_balances(balances)
		.with_candidates(candidates)
		.build()
		.execute_with(|| {
			assert_noop!(
				ParachainStaking::join_candidates(
					RuntimeOrigin::signed(new_candidate),
					80,
					crate::mock::MaxCandidates::get(),
				),
				Error::<Test>::CandidateLimitReached,
			);
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
				RuntimeOrigin::signed(1),
				1u32
			));
			assert_events_eq!(Event::CandidateScheduledExit {
				exit_allowed_round: 1,
				candidate: 1,
				scheduled_exit: 3
			});
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
				RuntimeOrigin::signed(1),
				1u32
			));
			assert!(ParachainStaking::candidate_pool().0.is_empty());
		});
}

#[test]
fn cannot_leave_candidates_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::schedule_leave_candidates(RuntimeOrigin::signed(1), 1u32),
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
				RuntimeOrigin::signed(1),
				1u32
			));
			assert_noop!(
				ParachainStaking::schedule_leave_candidates(RuntimeOrigin::signed(1), 1u32),
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
					ParachainStaking::schedule_leave_candidates(RuntimeOrigin::signed(i), 4u32),
					Error::<Test>::TooLowCandidateCountToLeaveCandidates
				);
			}
		});
}

#[test]
fn enable_marking_offline_works() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::enable_marking_offline(
				RuntimeOrigin::root(),
				true
			));
			assert!(ParachainStaking::marking_offline());

			// Set to false now
			assert_ok!(ParachainStaking::enable_marking_offline(
				RuntimeOrigin::root(),
				false
			));
			assert!(!ParachainStaking::marking_offline());
		});
}

#[test]
fn enable_marking_offline_fails_bad_origin() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_noop!(
				ParachainStaking::enable_marking_offline(RuntimeOrigin::signed(1), true),
				sp_runtime::DispatchError::BadOrigin
			);
		});
}

#[test]
fn was_inactive_is_cleaned_up_after_max_offline_rounds() {
	const ACTIVE_COLLATOR: AccountId = 1;
	const INACTIVE_COLLATOR: AccountId = 2;

	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20)])
		.with_candidates(vec![(1, 20), (2, 20)])
		.build()
		.execute_with(|| {
			assert_eq!(<Test as crate::Config>::MaxOfflineRounds::get(), 2);
			assert_eq!(<Test as crate::Config>::RewardPaymentDelay::get(), 2);

			// ACTIVE_COLLATOR authors all the blocks
			set_block_author(ACTIVE_COLLATOR);

			// Round 2
			roll_to_round_begin(2);

			assert!(<AtStake<Test>>::contains_key(1, ACTIVE_COLLATOR));
			assert!(!<WasInactive<Test>>::contains_key(1, ACTIVE_COLLATOR));

			assert!(<AtStake<Test>>::contains_key(1, INACTIVE_COLLATOR));
			assert!(<WasInactive<Test>>::contains_key(1, INACTIVE_COLLATOR));

			// Round 3
			roll_to_round_begin(3);

			assert!(<AtStake<Test>>::contains_key(2, ACTIVE_COLLATOR));
			assert!(!<WasInactive<Test>>::contains_key(2, ACTIVE_COLLATOR));

			assert!(<AtStake<Test>>::contains_key(2, INACTIVE_COLLATOR));
			assert!(<WasInactive<Test>>::contains_key(2, INACTIVE_COLLATOR));

			// End of round 3
			roll_to_round_end(3);

			assert!(
				!<AtStake<Test>>::contains_key(1, ACTIVE_COLLATOR),
				"Active collator should have no stake in round 1 due to the distribution of rewards"
			);
			assert!(
				!<AtStake<Test>>::contains_key(1, INACTIVE_COLLATOR),
				"Inactive collator should have no stake in round 1 due to the distribution of rewards"
			);

			assert!(
				!<WasInactive<Test>>::contains_key(1, ACTIVE_COLLATOR),
				"Active collator should not be in WasInactive for round 1"
			);
			assert!(
				<WasInactive<Test>>::contains_key(1, INACTIVE_COLLATOR),
				"Inactive collator should still be in WasInactive for round 1"
			);

			// Round 4
			roll_to_round_end(4);

			assert!(
				!<WasInactive<Test>>::contains_key(1, INACTIVE_COLLATOR),
				"Round 1 WasInactive should be cleaned up after MaxOfflineRounds"
			);
			assert!(<WasInactive<Test>>::contains_key(2, INACTIVE_COLLATOR));
			assert!(<WasInactive<Test>>::contains_key(3, INACTIVE_COLLATOR));
		});
}

#[test]
fn notify_inactive_collator_works() {
	const INACTIVE_COLLATOR: AccountId = 1;
	const ACTIVE_COLLATOR: AccountId = 2;

	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.build()
		.execute_with(|| {
			// Enable killswitch
			<EnableMarkingOffline<Test>>::set(true);

			assert_eq!(<Test as crate::Config>::MaxOfflineRounds::get(), 2);
			assert_eq!(<Test as crate::Config>::RewardPaymentDelay::get(), 2);

			// Round 2 - INACTIVE_COLLATOR authors blocks
			set_block_author(INACTIVE_COLLATOR);
			roll_to_round_begin(2);

			// Change block author
			set_block_author(ACTIVE_COLLATOR);

			// INACTIVE_COLLATOR does not produce blocks on round 2 and 3
			roll_to_round_begin(4);
			roll_blocks(1);

			// On round 4 notify inactive collator
			assert_ok!(ParachainStaking::notify_inactive_collator(
				RuntimeOrigin::signed(1),
				INACTIVE_COLLATOR
			));

			// Check the collator was marked as offline as it hasn't produced blocks
			assert_events_eq!(Event::CandidateWentOffline {
				candidate: INACTIVE_COLLATOR
			},);
		});
}

#[test]
fn notify_inactive_collator_succeeds_even_after_rewards_are_distributed() {
	const INACTIVE_COLLATOR: AccountId = 1;
	const ACTIVE_COLLATOR: AccountId = 2;

	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.build()
		.execute_with(|| {
			// Enable killswitch
			<EnableMarkingOffline<Test>>::set(true);

			// We need (strictly) more blocks per round than collators so rewards
			// can be distributed before the end of a round
			assert_ok!(ParachainStaking::set_blocks_per_round(
				RuntimeOrigin::root(),
				6u32
			));

			// ACTIVE_COLLATOR authors all the blocks while INACTIVE_COLLATOR stays inactive
			set_block_author(ACTIVE_COLLATOR);

			// Round 2
			roll_to_round_begin(2);
			roll_blocks(1);

			// INACTIVE_COLLATOR has a stake in round 1
			assert!(<AtStake<Test>>::contains_key(1, INACTIVE_COLLATOR));

			// Round 3
			roll_to_round_begin(3);
			roll_blocks(1);

			// INACTIVE_COLLATOR has a stake in round 2
			assert!(<AtStake<Test>>::contains_key(2, INACTIVE_COLLATOR));

			// End of round 3
			roll_to_round_end(3);

			// INACTIVE_COLLATOR has a no stake in round 1 anymore due to the distribution of rewards
			assert!(!<AtStake<Test>>::contains_key(1, INACTIVE_COLLATOR));

			// Call 'notify_inactive_collator' extrinsic on INACTIVE_COLLATOR
			assert_ok!(ParachainStaking::notify_inactive_collator(
				RuntimeOrigin::signed(1),
				INACTIVE_COLLATOR
			));

			assert_events_eq!(
				Event::Rewarded {
					account: 2,
					rewards: 0,
				},
				Event::CandidateWentOffline {
					candidate: INACTIVE_COLLATOR
				},
			);
		});
}

#[test]
fn notify_inactive_collator_fails_too_low_collator_count() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20), (3, 20)])
		.with_candidates(vec![(1, 20), (2, 20), (3, 20)])
		.build()
		.execute_with(|| {
			// Enable killswitch
			<EnableMarkingOffline<Test>>::set(true);

			// Round 4
			roll_to_round_begin(4);
			roll_blocks(1);

			// Call 'notify_inactive_collator' extrinsic
			assert_noop!(
				ParachainStaking::notify_inactive_collator(RuntimeOrigin::signed(1), 1),
				Error::<Test>::TooLowCollatorCountToNotifyAsInactive
			);
		});
}

#[test]
fn notify_inactive_collator_fails_candidate_is_not_collator() {
	ExtBuilder::default()
		.with_balances(vec![(1, 80), (2, 80), (3, 80), (4, 80), (5, 80), (6, 20)])
		.with_candidates(vec![(1, 80), (2, 80), (3, 80), (4, 80), (5, 80)])
		.build()
		.execute_with(|| {
			// Enable killswitch
			<EnableMarkingOffline<Test>>::set(true);

			set_block_author(1);

			roll_to_round_begin(2);
			assert_events_eq!(
				Event::CollatorChosen {
					round: 2,
					collator_account: 1,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 2,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 3,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 4,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 2,
					collator_account: 5,
					total_exposed_amount: 80,
				},
				Event::NewRound {
					starting_block: 5,
					round: 2,
					selected_collators_number: 5,
					total_balance: 400,
				},
			);
			roll_blocks(1);

			assert_ok!(ParachainStaking::join_candidates(
				RuntimeOrigin::signed(6),
				10,
				100
			));

			// Round 6
			roll_to_round_begin(6);
			assert_events_eq!(
				Event::CollatorChosen {
					round: 6,
					collator_account: 1,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 2,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 3,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 4,
					total_exposed_amount: 80,
				},
				Event::CollatorChosen {
					round: 6,
					collator_account: 5,
					total_exposed_amount: 80,
				},
				Event::NewRound {
					starting_block: 25,
					round: 6,
					selected_collators_number: 5,
					total_balance: 400,
				},
			);
			roll_blocks(1);

			// A candidate cannot be notified as inactive if it hasn't been selected
			// to produce blocks
			assert_noop!(
				ParachainStaking::notify_inactive_collator(RuntimeOrigin::signed(1), 6),
				Error::<Test>::CannotBeNotifiedAsInactive
			);
		});
}

#[test]
fn notify_inactive_collator_fails_cannot_be_notified_as_inactive() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.build()
		.execute_with(|| {
			// Enable killswitch
			<EnableMarkingOffline<Test>>::set(true);

			// Round 2
			roll_to_round_begin(2);

			// Change block author
			set_block_author(1);

			// Round 3
			roll_to_round_begin(3);
			roll_blocks(1);

			// Round 4
			roll_to_round_begin(4);
			roll_blocks(1);

			// Call 'notify_inactive_collator' extrinsic
			assert_noop!(
				ParachainStaking::notify_inactive_collator(RuntimeOrigin::signed(1), 1),
				Error::<Test>::CannotBeNotifiedAsInactive
			);
		});
}

#[test]
fn notify_inactive_collator_fails_round_too_low() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 20)])
		.build()
		.execute_with(|| {
			// Enable killswitch
			<EnableMarkingOffline<Test>>::set(true);

			// Round 1
			roll_to_round_begin(1);
			roll_blocks(1);

			// Call 'notify_inactive_collator' extrinsic
			assert_noop!(
				ParachainStaking::notify_inactive_collator(RuntimeOrigin::signed(1), 1),
				Error::<Test>::CurrentRoundTooLow
			);
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
					RuntimeOrigin::signed(i),
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
				RuntimeOrigin::signed(1),
				1u32
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				RuntimeOrigin::signed(1),
				1,
				0
			));
			assert_events_emitted!(Event::CandidateLeft {
				ex_candidate: 1,
				unlocked_amount: 10,
				new_total_amt_locked: 0
			});
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
				RuntimeOrigin::signed(1),
				1u32
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(1),
				1u32
			));
			roll_to(10);
			for i in 0..3 {
				assert_noop!(
					ParachainStaking::execute_leave_candidates(RuntimeOrigin::signed(1), 1, i),
					Error::<Test>::TooLowCandidateDelegationCountToLeaveCandidates
				);
			}
			assert_ok!(ParachainStaking::execute_leave_candidates(
				RuntimeOrigin::signed(2),
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
			assert_eq!(ParachainStaking::get_collator_stakable_free_balance(&1), 0);
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				RuntimeOrigin::signed(1),
				1u32
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				RuntimeOrigin::signed(1),
				1,
				0
			));
			assert_eq!(ParachainStaking::get_collator_stakable_free_balance(&1), 10);
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
				RuntimeOrigin::signed(1),
				1u32
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				RuntimeOrigin::signed(1),
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
				RuntimeOrigin::signed(1),
				1u32
			));
			// candidate state is not immediately removed
			let candidate_state =
				ParachainStaking::candidate_info(1).expect("just left => still exists");
			assert_eq!(candidate_state.bond, 10u128);
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				RuntimeOrigin::signed(1),
				1,
				0
			));
			assert!(ParachainStaking::candidate_info(1).is_none());
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
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(1),
				1u32
			));
			// candidate state is not immediately removed
			let candidate_state =
				ParachainStaking::candidate_info(1).expect("just left => still exists");
			assert_eq!(candidate_state.bond, 10u128);
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				RuntimeOrigin::signed(1),
				1,
				1
			));
			assert!(ParachainStaking::candidate_info(1).is_none());
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
				RuntimeOrigin::signed(1),
				1u32
			));
			assert_noop!(
				ParachainStaking::execute_leave_candidates(RuntimeOrigin::signed(3), 1, 0)
					.map_err(|err| err.error),
				Error::<Test>::CandidateCannotLeaveYet
			);
			roll_to(9);
			assert_noop!(
				ParachainStaking::execute_leave_candidates(RuntimeOrigin::signed(3), 1, 0)
					.map_err(|err| err.error),
				Error::<Test>::CandidateCannotLeaveYet
			);
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				RuntimeOrigin::signed(3),
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
				RuntimeOrigin::signed(1),
				1u32
			));
			assert_ok!(ParachainStaking::cancel_leave_candidates(
				RuntimeOrigin::signed(1),
				1
			));
			assert_events_emitted!(Event::CancelledCandidateExit { candidate: 1 });
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
				RuntimeOrigin::signed(1),
				1u32
			));
			assert_ok!(ParachainStaking::cancel_leave_candidates(
				RuntimeOrigin::signed(1),
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
				RuntimeOrigin::signed(1),
				1u32
			));
			assert_ok!(ParachainStaking::cancel_leave_candidates(
				RuntimeOrigin::signed(1),
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
			assert_ok!(ParachainStaking::go_offline(RuntimeOrigin::signed(1)));
			assert_events_eq!(Event::CandidateWentOffline { candidate: 1 });
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
			assert_ok!(ParachainStaking::go_offline(RuntimeOrigin::signed(1)));
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
			assert_ok!(ParachainStaking::go_offline(RuntimeOrigin::signed(1)));
			let candidate_state =
				ParachainStaking::candidate_info(1).expect("is candidate, just offline");
			assert_eq!(candidate_state.status, CollatorStatus::Idle);
		});
}

#[test]
fn cannot_go_offline_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::go_offline(RuntimeOrigin::signed(3)).map_err(|err| err.error),
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
			assert_ok!(ParachainStaking::go_offline(RuntimeOrigin::signed(1)));
			assert_noop!(
				ParachainStaking::go_offline(RuntimeOrigin::signed(1)).map_err(|err| err.error),
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
			assert_ok!(ParachainStaking::go_offline(RuntimeOrigin::signed(1)));
			roll_blocks(1);
			assert_ok!(ParachainStaking::go_online(RuntimeOrigin::signed(1)));
			assert_events_eq!(Event::CandidateBackOnline { candidate: 1 });
		});
}

#[test]
fn go_online_adds_to_candidate_pool() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::go_offline(RuntimeOrigin::signed(1)));
			assert!(ParachainStaking::candidate_pool().0.is_empty());
			assert_ok!(ParachainStaking::go_online(RuntimeOrigin::signed(1)));
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
			assert_ok!(ParachainStaking::go_offline(RuntimeOrigin::signed(1)));
			let candidate_state =
				ParachainStaking::candidate_info(1).expect("offline still exists");
			assert_eq!(candidate_state.status, CollatorStatus::Idle);
			assert_ok!(ParachainStaking::go_online(RuntimeOrigin::signed(1)));
			let candidate_state = ParachainStaking::candidate_info(1).expect("online so exists");
			assert_eq!(candidate_state.status, CollatorStatus::Active);
		});
}

#[test]
fn cannot_go_online_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::go_online(RuntimeOrigin::signed(3)),
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
				ParachainStaking::go_online(RuntimeOrigin::signed(1)).map_err(|err| err.error),
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
				RuntimeOrigin::signed(1),
				1
			));
			assert_noop!(
				ParachainStaking::go_online(RuntimeOrigin::signed(1)).map_err(|err| err.error),
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
			assert_ok!(ParachainStaking::candidate_bond_more(
				RuntimeOrigin::signed(1),
				30
			));
			assert_events_eq!(Event::CandidateBondedMore {
				candidate: 1,
				amount: 30,
				new_total_bond: 50
			});
		});
}

#[test]
fn candidate_bond_more_reserves_balance() {
	ExtBuilder::default()
		.with_balances(vec![(1, 50)])
		.with_candidates(vec![(1, 20)])
		.build()
		.execute_with(|| {
			assert_eq!(ParachainStaking::get_collator_stakable_free_balance(&1), 30);
			assert_ok!(ParachainStaking::candidate_bond_more(
				RuntimeOrigin::signed(1),
				30
			));
			assert_eq!(ParachainStaking::get_collator_stakable_free_balance(&1), 0);
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
			assert_ok!(ParachainStaking::candidate_bond_more(
				RuntimeOrigin::signed(1),
				30
			));
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
			assert_ok!(ParachainStaking::candidate_bond_more(
				RuntimeOrigin::signed(1),
				30
			));
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
			assert_ok!(ParachainStaking::candidate_bond_more(
				RuntimeOrigin::signed(1),
				30
			));
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
				RuntimeOrigin::signed(1),
				10
			));
			assert_events_eq!(Event::CandidateBondLessRequested {
				candidate: 1,
				amount_to_decrease: 10,
				execute_round: 3,
			});
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
				RuntimeOrigin::signed(1),
				5
			));
			assert_noop!(
				ParachainStaking::schedule_candidate_bond_less(RuntimeOrigin::signed(1), 5),
				Error::<Test>::PendingCandidateRequestAlreadyExists
			);
		});
}

#[test]
fn cannot_schedule_candidate_bond_less_if_not_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::schedule_candidate_bond_less(RuntimeOrigin::signed(6), 50),
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
				ParachainStaking::schedule_candidate_bond_less(RuntimeOrigin::signed(1), 21),
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
				RuntimeOrigin::signed(1),
				1
			));
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				RuntimeOrigin::signed(1),
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
				RuntimeOrigin::signed(1),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				RuntimeOrigin::signed(1),
				1,
				0
			));
			assert_noop!(
				ParachainStaking::schedule_candidate_bond_less(RuntimeOrigin::signed(1), 10),
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
				RuntimeOrigin::signed(1),
				30
			));
			roll_to(10);
			roll_blocks(1);
			assert_ok!(ParachainStaking::execute_candidate_bond_less(
				RuntimeOrigin::signed(1),
				1
			));
			assert_events_eq!(Event::CandidateBondedLess {
				candidate: 1,
				amount: 30,
				new_bond: 20
			});
		});
}

#[test]
fn execute_candidate_bond_less_unreserves_balance() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_eq!(ParachainStaking::get_collator_stakable_free_balance(&1), 0);
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				RuntimeOrigin::signed(1),
				10
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_candidate_bond_less(
				RuntimeOrigin::signed(1),
				1
			));
			assert_eq!(ParachainStaking::get_collator_stakable_free_balance(&1), 10);
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
				RuntimeOrigin::signed(1),
				10
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_candidate_bond_less(
				RuntimeOrigin::signed(1),
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
				RuntimeOrigin::signed(1),
				10
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_candidate_bond_less(
				RuntimeOrigin::signed(1),
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
				RuntimeOrigin::signed(1),
				10
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_candidate_bond_less(
				RuntimeOrigin::signed(1),
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
				RuntimeOrigin::signed(1),
				10
			));
			assert_ok!(ParachainStaking::cancel_candidate_bond_less(
				RuntimeOrigin::signed(1)
			));
			assert_events_emitted!(Event::CancelledCandidateBondLess {
				candidate: 1,
				amount: 10,
				execute_round: 3,
			});
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
				RuntimeOrigin::signed(1),
				10
			));
			assert_ok!(ParachainStaking::cancel_candidate_bond_less(
				RuntimeOrigin::signed(1)
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
				RuntimeOrigin::signed(1),
				10
			));
			assert_noop!(
				ParachainStaking::cancel_candidate_bond_less(RuntimeOrigin::signed(2)),
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
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				10,
				Percent::zero(),
				0,
				0,
				0
			));
			assert_events_eq!(Event::Delegation {
				delegator: 2,
				locked_amount: 10,
				candidate: 1,
				delegator_position: DelegatorAdded::AddedToTop { new_total: 40 },
				auto_compound: Percent::zero(),
			});
		});
}

#[test]
fn delegate_reserves_balance() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_eq!(ParachainStaking::get_delegator_stakable_balance(&2), 10);
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				10,
				Percent::zero(),
				0,
				0,
				0
			));
			assert_eq!(ParachainStaking::get_delegator_stakable_balance(&2), 0);
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
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				10,
				Percent::zero(),
				0,
				0,
				0
			));
			let delegator_state =
				ParachainStaking::delegator_state(2).expect("just delegated => exists");
			assert_eq!(delegator_state.total(), 10);
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
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				10,
				Percent::zero(),
				0,
				0,
				0
			));
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
			assert_ok!(ParachainStaking::join_candidates(
				RuntimeOrigin::signed(1),
				20,
				0
			));
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				20,
				Percent::zero(),
				0,
				0,
				0
			));
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
				RuntimeOrigin::signed(2),
				1
			));
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				4,
				10,
				Percent::zero(),
				0,
				0,
				2
			));
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
				ParachainStaking::delegate_with_auto_compound(
					RuntimeOrigin::signed(11),
					1,
					10,
					Percent::zero(),
					8,
					0,
					0
				),
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
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(11),
				1,
				11,
				Percent::zero(),
				8,
				0,
				0
			));
			assert_events_emitted!(Event::DelegationKicked {
				delegator: 10,
				candidate: 1,
				unstaked_amount: 10
			});
			assert_events_emitted!(Event::DelegatorLeft {
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
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				1,
			));
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				3,
				10,
				Percent::zero(),
				0,
				0,
				1
			));
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
				ParachainStaking::delegate_with_auto_compound(
					RuntimeOrigin::signed(2),
					1,
					10,
					Percent::zero(),
					0,
					0,
					0
				),
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
				ParachainStaking::delegate_with_auto_compound(
					RuntimeOrigin::signed(2),
					1,
					10,
					Percent::zero(),
					1,
					0,
					1
				),
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
				ParachainStaking::delegate_with_auto_compound(
					RuntimeOrigin::signed(2),
					6,
					10,
					Percent::zero(),
					0,
					0,
					4,
				),
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
				assert_ok!(ParachainStaking::delegate_with_auto_compound(
					RuntimeOrigin::signed(i),
					1,
					10,
					Percent::zero(),
					count,
					0,
					0
				));
				count += 1u32;
			}
			let mut count = 0u32;
			for i in 3..11 {
				assert_ok!(ParachainStaking::delegate_with_auto_compound(
					RuntimeOrigin::signed(i),
					2,
					10,
					Percent::zero(),
					count,
					0,
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
					ParachainStaking::delegate_with_auto_compound(
						RuntimeOrigin::signed(i),
						1,
						10,
						Percent::zero(),
						count,
						0,
						0
					),
					Error::<Test>::TooLowCandidateDelegationCountToDelegate
				);
			}
			// to set up for next error test
			count = 4u32;
			for i in 7..11 {
				assert_ok!(ParachainStaking::delegate_with_auto_compound(
					RuntimeOrigin::signed(i),
					1,
					10,
					Percent::zero(),
					count,
					0,
					0
				));
				count += 1u32;
			}
			count = 0u32;
			for i in 3..11 {
				assert_noop!(
					ParachainStaking::delegate_with_auto_compound(
						RuntimeOrigin::signed(i),
						2,
						10,
						Percent::zero(),
						count,
						0,
						0
					),
					Error::<Test>::TooLowDelegationCountToDelegate
				);
				count += 1u32;
			}
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
				RuntimeOrigin::signed(2),
				1
			));
			assert_events_eq!(Event::DelegationRevocationScheduled {
				round: 1,
				delegator: 2,
				candidate: 1,
				scheduled_exit: 3,
			});
			roll_to_round_begin(3);
			roll_blocks(1);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
				2,
				1
			));
			assert_events_eq!(
				Event::DelegatorLeftCandidate {
					delegator: 2,
					candidate: 1,
					unstaked_amount: 10,
					total_candidate_staked: 30
				},
				Event::DelegationRevoked {
					delegator: 2,
					candidate: 1,
					unstaked_amount: 10,
				},
			);
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
				RuntimeOrigin::signed(2),
				1
			));
			// this is an exit implicitly because last delegation revoked
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				3
			));
		});
}

#[test]
fn cannot_revoke_delegation_if_not_delegator() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::schedule_revoke_delegation(RuntimeOrigin::signed(2), 1),
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
				ParachainStaking::schedule_revoke_delegation(RuntimeOrigin::signed(2), 3),
				Error::<Test>::DelegationDNE
			);
		});
}

#[test]
fn can_schedule_revoke_delegation_below_min_delegator_stake() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 8), (3, 20)])
		.with_candidates(vec![(1, 20), (3, 20)])
		.with_delegations(vec![(2, 1, 5), (2, 3, 3)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
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
			assert_eq!(ParachainStaking::get_delegator_stakable_balance(&2), 5);
			assert_ok!(ParachainStaking::delegator_bond_more(
				RuntimeOrigin::signed(2),
				1,
				5
			));
			assert_eq!(ParachainStaking::get_delegator_stakable_balance(&2), 0);
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
				RuntimeOrigin::signed(2),
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
				ParachainStaking::delegator_state(2)
					.expect("exists")
					.total(),
				10
			);
			assert_ok!(ParachainStaking::delegator_bond_more(
				RuntimeOrigin::signed(2),
				1,
				5
			));
			assert_eq!(
				ParachainStaking::delegator_state(2)
					.expect("exists")
					.total(),
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
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
				1,
				5
			));
			assert_events_eq!(Event::DelegationIncreased {
				delegator: 2,
				candidate: 1,
				amount: 5,
				in_top: false
			});
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
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(1),
				1
			));
			assert_ok!(ParachainStaking::delegator_bond_more(
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
				1
			));
			assert_noop!(
				ParachainStaking::delegator_bond_more(RuntimeOrigin::signed(2), 1, 5),
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
				RuntimeOrigin::signed(2),
				1,
				5,
			));
			assert_ok!(ParachainStaking::delegator_bond_more(
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
				1,
				5
			));
			assert_events_eq!(Event::DelegationDecreaseScheduled {
				delegator: 2,
				candidate: 1,
				amount_to_decrease: 5,
				execute_round: 3,
			});
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
				RuntimeOrigin::signed(2),
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
fn cannot_delegator_bond_less_if_revoking() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25), (3, 20)])
		.with_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				1
			));
			assert_noop!(
				ParachainStaking::schedule_delegator_bond_less(RuntimeOrigin::signed(2), 1, 1)
					.map_err(|err| err.error),
				Error::<Test>::PendingDelegationRequestAlreadyExists
			);
		});
}

#[test]
fn cannot_delegator_bond_less_if_not_delegator() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ParachainStaking::schedule_delegator_bond_less(RuntimeOrigin::signed(2), 1, 5)
				.map_err(|err| err.error),
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
				ParachainStaking::schedule_delegator_bond_less(RuntimeOrigin::signed(2), 3, 5)
					.map_err(|err| err.error),
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
				ParachainStaking::schedule_delegator_bond_less(RuntimeOrigin::signed(2), 3, 5)
					.map_err(|err| err.error),
				Error::<Test>::DelegationDNE
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
				ParachainStaking::schedule_delegator_bond_less(RuntimeOrigin::signed(2), 1, 11)
					.map_err(|err| err.error),
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
				ParachainStaking::schedule_delegator_bond_less(RuntimeOrigin::signed(2), 1, 8)
					.map_err(|err| err.error),
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
				RuntimeOrigin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
				2,
				1
			));
			assert_events_emitted!(Event::DelegatorLeftCandidate {
				delegator: 2,
				candidate: 1,
				unstaked_amount: 10,
				total_candidate_staked: 30
			});
			assert_events_emitted!(Event::DelegatorLeft {
				delegator: 2,
				unstaked_amount: 10
			});
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
				RuntimeOrigin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
				2,
				1
			));
			assert_events_emitted!(Event::DelegatorLeftCandidate {
				delegator: 2,
				candidate: 1,
				unstaked_amount: 10,
				total_candidate_staked: 30
			});
			assert_events_emitted!(Event::DelegatorLeft {
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
				RuntimeOrigin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
				2,
				1
			));
			assert_events_emitted!(Event::DelegatorLeftCandidate {
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
			assert_eq!(ParachainStaking::get_delegator_stakable_balance(&2), 0);
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
				2,
				1
			));
			assert_eq!(ParachainStaking::get_delegator_stakable_balance(&2), 10);
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
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
				1
			));
			roll_to(10);
			// this will be confusing for people
			// if status is leaving, then execute_delegation_request works if last delegation
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(1),
				1
			));
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				1
			));
			roll_to(10);
			// can execute delegation request for leaving candidate
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(1),
				1
			));
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				1
			));
			roll_to(10);
			// revocation executes during execute leave candidates (callable by anyone)
			assert_ok!(ParachainStaking::execute_leave_candidates(
				RuntimeOrigin::signed(1),
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
				RuntimeOrigin::signed(2),
				1
			));
			assert_ok!(ParachainStaking::delegator_bond_more(
				RuntimeOrigin::signed(2),
				3,
				10
			));
			roll_to(100);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
				2,
				1
			));
			assert!(ParachainStaking::is_delegator(&2));
			assert_eq!(ParachainStaking::get_delegator_stakable_balance(&2), 10);
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
				RuntimeOrigin::signed(2),
				1
			));
			assert_events_eq!(Event::DelegationRevocationScheduled {
				round: 1,
				delegator: 2,
				candidate: 1,
				scheduled_exit: 3,
			});
			assert_noop!(
				ParachainStaking::schedule_delegator_bond_less(RuntimeOrigin::signed(2), 1, 2)
					.map_err(|err| err.error),
				Error::<Test>::PendingDelegationRequestAlreadyExists
			);
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				RuntimeOrigin::signed(2),
				3,
				2
			));
			roll_to(10);
			roll_blocks(1);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
				2,
				1
			));
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
				2,
				3
			));
			assert_events_eq!(
				Event::DelegatorLeftCandidate {
					delegator: 2,
					candidate: 1,
					unstaked_amount: 10,
					total_candidate_staked: 30,
				},
				Event::DelegationRevoked {
					delegator: 2,
					candidate: 1,
					unstaked_amount: 10,
				},
				Event::DelegationDecreased {
					delegator: 2,
					candidate: 3,
					amount: 2,
					in_top: true
				},
			);
			assert!(ParachainStaking::is_delegator(&2));
			assert_eq!(ParachainStaking::get_delegator_stakable_balance(&2), 22);
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
			assert_eq!(ParachainStaking::get_delegator_stakable_balance(&2), 0);
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				RuntimeOrigin::signed(2),
				1,
				5
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
				2,
				1
			));
			assert_eq!(ParachainStaking::get_delegator_stakable_balance(&2), 5);
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
				RuntimeOrigin::signed(2),
				1,
				5
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
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
				ParachainStaking::delegator_state(2)
					.expect("exists")
					.total(),
				10
			);
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				RuntimeOrigin::signed(2),
				1,
				5
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
				2,
				1
			));
			assert_eq!(
				ParachainStaking::delegator_state(2)
					.expect("exists")
					.total(),
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
				RuntimeOrigin::signed(2),
				1,
				5
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
				1,
				5
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
				1,
				2
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(6),
				1,
				4
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(6),
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
				RuntimeOrigin::signed(1),
				1
			));
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				RuntimeOrigin::signed(2),
				1,
				5
			));
			roll_to(10);
			// can execute bond more delegation request for leaving candidate
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
				1
			));
			assert_ok!(ParachainStaking::cancel_delegation_request(
				RuntimeOrigin::signed(2),
				1
			));
			assert_events_emitted!(Event::CancelledDelegationRequest {
				delegator: 2,
				collator: 1,
				cancelled_request: CancelledScheduledRequest {
					when_executable: 3,
					action: DelegationAction::Revoke(10),
				},
			});
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
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
				1,
				5
			));
			assert_ok!(ParachainStaking::cancel_delegation_request(
				RuntimeOrigin::signed(2),
				1
			));
			assert_events_emitted!(Event::CancelledDelegationRequest {
				delegator: 2,
				collator: 1,
				cancelled_request: CancelledScheduledRequest {
					when_executable: 3,
					action: DelegationAction::Decrease(5),
				},
			});
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
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
				2,
				1
			));
			assert_eq!(
				ParachainStaking::delegator_state(&2)
					.map(|x| x.less_total)
					.expect("delegator state must exist"),
				0
			);
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				5,
				10,
				Percent::zero(),
				0,
				0,
				2
			));
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				3
			));
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
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
				RuntimeOrigin::signed(2),
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

#[ignore]
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
				RuntimeOrigin::root(),
				11
			));
			assert_events_eq!(Event::InflationDistributionConfigUpdated {
				old: inflation_configs(0, 30, 0, 0),
				new: inflation_configs(11, 30, 0, 0),
			});
			roll_to_round_begin(2);
			// chooses top TotalSelectedCandidates (5), in order
			assert_events_eq!(
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
			);
			assert_eq!(Balances::free_balance(&11), 1);
			// ~ set block author as 1 for all blocks this round
			set_author(2, 1, POINTS_PER_ROUND);
			roll_to_round_begin(4);
			// distribute total issuance to collator 1 and its delegators 6, 7, 19
			assert_eq!(Balances::free_balance(&11), 16);
			// ~ set block author as 1 for all blocks in rounds 3, 4, and 5
			set_author(3, 1, POINTS_PER_ROUND);
			set_author(4, 1, POINTS_PER_ROUND);
			set_author(5, 1, POINTS_PER_ROUND);
			// 1. ensure delegators are paid for 2 rounds after they leave
			assert_noop!(
				ParachainStaking::schedule_revoke_delegation(RuntimeOrigin::signed(66), 1),
				Error::<Test>::DelegatorDNE
			);
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(6),
				1,
			));
			assert_events_eq!(
				Event::InflationDistributed {
					index: 0,
					account: 11, // PBR
					value: 15,
				},
				Event::InflationDistributed {
					index: 1,
					account: 0, // Treasury
					value: 0,
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
				Event::DelegatorExitScheduled {
					round: 4,
					delegator: 6,
					scheduled_exit: 6,
				},
			);
			roll_blocks(3);
			assert_events_eq!(
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
			);
			// fast forward to block in which delegator 6 exit executes
			roll_to_round_begin(5);
			assert_events_eq!(
				Event::InflationDistributed {
					index: 0,
					account: 11, // PBR
					value: 16,
				},
				Event::InflationDistributed {
					index: 1,
					account: 0, // Treasury
					value: 0,
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
			);
			roll_blocks(3);
			assert_events_eq!(
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
			);
			roll_to_round_begin(6);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(6),
				6,
				10
			));
			assert_events_eq!(
				Event::InflationDistributed {
					index: 0,
					account: 11, // PBR
					value: 16,
				},
				Event::InflationDistributed {
					index: 1,
					account: 0, // Treasury
					value: 0,
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
			);
			roll_blocks(3);
			assert_events_eq!(
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
			);
			roll_to_round_begin(7);
			assert_events_eq!(
				Event::InflationDistributed {
					index: 0,
					account: 11, // PBR
					value: 17,
				},
				Event::InflationDistributed {
					index: 1,
					account: 0, // Treasury
					value: 0,
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
			);
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 26,
				},
				Event::Rewarded {
					account: 7,
					rewards: 7,
				},
				Event::Rewarded {
					account: 10,
					rewards: 7,
				},
			);
			assert_eq!(Balances::free_balance(&11), 65);
			roll_blocks(1);
			assert_ok!(ParachainStaking::set_parachain_bond_reserve_percent(
				RuntimeOrigin::root(),
				Percent::from_percent(50)
			));
			assert_events_eq!(Event::InflationDistributionConfigUpdated {
				old: inflation_configs(11, 30, 0, 0),
				new: inflation_configs(11, 50, 0, 0),
			});
			// 6 won't be paid for this round because they left already
			set_author(6, 1, POINTS_PER_ROUND);
			roll_to_round_begin(8);
			// keep paying 6
			assert_events_eq!(
				Event::InflationDistributed {
					index: 0,
					account: 11, // PBR
					value: 30,
				},
				Event::InflationDistributed {
					index: 1,
					account: 0, // Treasury
					value: 0,
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
			);
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 21,
				},
				Event::Rewarded {
					account: 7,
					rewards: 5,
				},
				Event::Rewarded {
					account: 10,
					rewards: 5,
				},
			);
			assert_eq!(Balances::free_balance(&11), 95);
			set_author(7, 1, POINTS_PER_ROUND);
			roll_to_round_begin(9);
			// no more paying 6
			assert_events_eq!(
				Event::InflationDistributed {
					index: 0,
					account: 11, // PBR
					value: 32,
				},
				Event::InflationDistributed {
					index: 1,
					account: 0, // Treasury
					value: 0,
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
			);
			roll_blocks(3);
			assert_events_eq!(
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
			);
			assert_eq!(Balances::free_balance(&11), 127);
			set_author(8, 1, POINTS_PER_ROUND);
			roll_blocks(1);
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(8),
				1,
				10,
				Percent::zero(),
				10,
				0,
				10
			));
			assert_events_eq!(Event::Delegation {
				delegator: 8,
				locked_amount: 10,
				candidate: 1,
				delegator_position: DelegatorAdded::AddedToTop { new_total: 50 },
				auto_compound: Percent::zero(),
			});
			roll_to_round_begin(10);
			// new delegation is not rewarded yet
			assert_events_eq!(
				Event::InflationDistributed {
					index: 0,
					account: 11, // PBR
					value: 33,
				},
				Event::InflationDistributed {
					index: 1,
					account: 0, // Treasury
					value: 0,
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
			);
			roll_blocks(3);
			assert_events_eq!(
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
			);
			assert_eq!(Balances::free_balance(&11), 160);
			set_author(9, 1, POINTS_PER_ROUND);
			set_author(10, 1, POINTS_PER_ROUND);
			roll_to_round_begin(11);
			// new delegation is still not rewarded yet
			assert_events_eq!(
				Event::InflationDistributed {
					index: 0,
					account: 11, // PBR
					value: 35,
				},
				Event::InflationDistributed {
					index: 1,
					account: 0, // Treasury
					value: 0,
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
			);
			roll_blocks(3);
			assert_events_eq!(
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
			);
			assert_eq!(Balances::free_balance(&11), 195);
			roll_to_round_begin(12);
			// new delegation is rewarded, 2 rounds after joining (`RewardPaymentDelay` is 2)
			assert_events_eq!(
				Event::InflationDistributed {
					index: 0,
					account: 11, // PBR
					value: 37,
				},
				Event::InflationDistributed {
					index: 1,
					account: 0, // Treasury
					value: 0,
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
			);
			roll_blocks(3);
			assert_events_eq!(
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
			);
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
			roll_to_round_begin(2);
			assert_ok!(ParachainStaking::join_candidates(
				RuntimeOrigin::signed(4),
				20u128,
				100u32
			));
			assert_events_eq!(
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
				Event::JoinedCollatorCandidates {
					account: 4,
					amount_locked: 20,
					new_total_amt_locked: 60,
				},
			);

			roll_blocks(1);
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(5),
				4,
				10,
				Percent::zero(),
				10,
				0,
				10
			));
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(6),
				4,
				10,
				Percent::zero(),
				10,
				0,
				10
			));
			assert_events_eq!(
				Event::Delegation {
					delegator: 5,
					locked_amount: 10,
					candidate: 4,
					delegator_position: DelegatorAdded::AddedToTop { new_total: 30 },
					auto_compound: Percent::zero(),
				},
				Event::Delegation {
					delegator: 6,
					locked_amount: 10,
					candidate: 4,
					delegator_position: DelegatorAdded::AddedToTop { new_total: 40 },
					auto_compound: Percent::zero(),
				},
			);

			roll_to_round_begin(3);
			assert_events_eq!(
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
			);
			// only reward author with id 4
			set_author(3, 4, POINTS_PER_ROUND);
			roll_to_round_begin(5);
			// 20% of 10 is commission + due_portion (0) = 2 + 4 = 6
			// all delegator payouts are 10-2 = 8 * stake_pct
			assert_events_eq!(
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
			);

			roll_blocks(1);
			assert_events_eq!(
				Event::Rewarded {
					account: 4,
					rewards: 9,
				},
				Event::Rewarded {
					account: 5,
					rewards: 3,
				},
				Event::Rewarded {
					account: 6,
					rewards: 3,
				},
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
		.with_candidates(vec![(1, 500), (2, 200)])
		.with_delegations(vec![(3, 1, 100), (4, 1, 100), (5, 2, 100), (6, 2, 100)])
		.build()
		.execute_with(|| {
			roll_to(11);
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				RuntimeOrigin::signed(2),
				2
			));
			assert_events_eq!(Event::CandidateScheduledExit {
				exit_allowed_round: 3,
				candidate: 2,
				scheduled_exit: 5,
			});
			let info = ParachainStaking::candidate_info(&2).unwrap();
			assert_eq!(info.status, CollatorStatus::Leaving(5));
			roll_to(21);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				RuntimeOrigin::signed(2),
				2,
				2
			));
			// we must exclude leaving collators from rewards while
			// holding them retroactively accountable for previous faults
			// (within the last T::SlashingWindow blocks)
			assert_events_eq!(Event::CandidateLeft {
				ex_candidate: 2,
				unlocked_amount: 400,
				new_total_amt_locked: 700,
			},);
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
			roll_to_round_begin(2);
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				RuntimeOrigin::signed(6),
				6
			));
			// should choose top TotalSelectedCandidates (5), in order
			assert_events_eq!(
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
					scheduled_exit: 4
				},
			);
			roll_to_round_begin(4);
			roll_blocks(1);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				RuntimeOrigin::signed(6),
				6,
				0
			));
			assert_ok!(ParachainStaking::join_candidates(
				RuntimeOrigin::signed(6),
				69u128,
				100u32
			));
			assert_events_eq!(
				Event::CandidateLeft {
					ex_candidate: 6,
					unlocked_amount: 50,
					new_total_amt_locked: 400,
				},
				Event::JoinedCollatorCandidates {
					account: 6,
					amount_locked: 69u128,
					new_total_amt_locked: 469u128,
				},
			);
			roll_to_round_begin(6);
			// should choose top TotalSelectedCandidates (5), in order
			assert_events_eq!(
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
			);
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
			(7, 33),
			(8, 33),
			(9, 33),
		])
		.with_candidates(vec![(1, 100), (2, 90), (3, 80), (4, 70)])
		.build()
		.execute_with(|| {
			roll_to_round_begin(2);
			// should choose top TotalCandidatesSelected (5), in order
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 5,
					round: 2,
					selected_collators_number: 4,
					total_balance: 340,
				},
			);
			// ~ set block author as 1 for all blocks this round
			set_author(2, 1, POINTS_PER_ROUND);
			roll_to_round_begin(4);
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 15,
					round: 4,
					selected_collators_number: 4,
					total_balance: 340,
				},
			);
			// pay total issuance to 1 at 2nd block
			roll_blocks(3);
			assert_events_eq!(Event::Rewarded {
				account: 1,
				rewards: 102,
			});
			// ~ set block author as 1 for 3 blocks this round
			set_author(4, 1, POINTS_PER_BLOCK * 3);
			// ~ set block author as 2 for 2 blocks this round
			set_author(4, 2, POINTS_PER_BLOCK * 2);
			roll_to_round_begin(6);
			// pay 60% total issuance to 1 and 40% total issuance to 2
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 25,
					round: 6,
					selected_collators_number: 4,
					total_balance: 340,
				},
			);
			roll_blocks(3);
			assert_events_eq!(Event::Rewarded {
				account: 1,
				rewards: 63,
			});
			roll_blocks(1);
			assert_events_eq!(Event::Rewarded {
				account: 2,
				rewards: 42,
			},);
			// ~ each collator produces at least 1 block this round
			set_author(6, 1, POINTS_PER_BLOCK * 2);
			set_author(6, 2, POINTS_PER_BLOCK);
			set_author(6, 3, POINTS_PER_BLOCK);
			set_author(6, 4, POINTS_PER_BLOCK);
			roll_to_round_begin(8);
			// pay 20% issuance for all collators
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 35,
					round: 8,
					selected_collators_number: 4,
					total_balance: 340,
				},
			);
			roll_blocks(1);
			assert_events_eq!(Event::Rewarded {
				account: 3,
				rewards: 21,
			});
			roll_blocks(1);
			assert_events_eq!(Event::Rewarded {
				account: 4,
				rewards: 21,
			});
			roll_blocks(1);
			assert_events_eq!(Event::Rewarded {
				account: 1,
				rewards: 43,
			});
			roll_blocks(1);
			assert_events_eq!(Event::Rewarded {
				account: 2,
				rewards: 21,
			});
			// check that distributing rewards clears awarded pts
			assert!(ParachainStaking::awarded_pts(1, 1).is_zero());
			assert!(ParachainStaking::awarded_pts(4, 1).is_zero());
			assert!(ParachainStaking::awarded_pts(4, 2).is_zero());
			assert!(ParachainStaking::awarded_pts(6, 1).is_zero());
			assert!(ParachainStaking::awarded_pts(6, 2).is_zero());
			assert!(ParachainStaking::awarded_pts(6, 3).is_zero());
			assert!(ParachainStaking::awarded_pts(6, 4).is_zero());
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
			roll_to_round_begin(2);
			// chooses top TotalSelectedCandidates (5), in order
			assert_events_eq!(
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
			);
			roll_blocks(1);
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(6),
				2,
				10,
				Percent::zero(),
				10,
				0,
				10
			));
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(6),
				3,
				10,
				Percent::zero(),
				10,
				0,
				10
			));
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(6),
				4,
				10,
				Percent::zero(),
				10,
				0,
				10
			));
			assert_events_eq!(
				Event::Delegation {
					delegator: 6,
					locked_amount: 10,
					candidate: 2,
					delegator_position: DelegatorAdded::AddedToTop { new_total: 50 },
					auto_compound: Percent::zero(),
				},
				Event::Delegation {
					delegator: 6,
					locked_amount: 10,
					candidate: 3,
					delegator_position: DelegatorAdded::AddedToTop { new_total: 30 },
					auto_compound: Percent::zero(),
				},
				Event::Delegation {
					delegator: 6,
					locked_amount: 10,
					candidate: 4,
					delegator_position: DelegatorAdded::AddedToTop { new_total: 30 },
					auto_compound: Percent::zero(),
				},
			);
			roll_to_round_begin(6);
			roll_blocks(1);
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(7),
				2,
				80,
				Percent::zero(),
				10,
				0,
				10
			));
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(10),
				2,
				10,
				Percent::zero(),
				10,
				0,
				10
			));
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				RuntimeOrigin::signed(2),
				5
			));
			assert_events_eq!(
				Event::Delegation {
					delegator: 7,
					locked_amount: 80,
					candidate: 2,
					delegator_position: DelegatorAdded::AddedToTop { new_total: 130 },
					auto_compound: Percent::zero(),
				},
				Event::Delegation {
					delegator: 10,
					locked_amount: 10,
					candidate: 2,
					delegator_position: DelegatorAdded::AddedToBottom,
					auto_compound: Percent::zero(),
				},
				Event::CandidateScheduledExit {
					exit_allowed_round: 6,
					candidate: 2,
					scheduled_exit: 8
				},
			);
			roll_to_round_begin(7);
			assert_events_eq!(
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
			);
			// verify that delegations are removed after collator leaves, not before
			assert_eq!(ParachainStaking::delegator_state(7).unwrap().total(), 90);
			assert_eq!(
				ParachainStaking::delegator_state(7)
					.unwrap()
					.delegations
					.0
					.len(),
				2usize
			);
			assert_eq!(ParachainStaking::delegator_state(6).unwrap().total(), 40);
			assert_eq!(
				ParachainStaking::delegator_state(6)
					.unwrap()
					.delegations
					.0
					.len(),
				4usize
			);
			assert_eq!(
				query_freeze_amount(6, &FreezeReason::StakingDelegator.into()),
				40
			);
			assert_eq!(
				query_freeze_amount(7, &FreezeReason::StakingDelegator.into()),
				90
			);
			assert_eq!(ParachainStaking::get_delegator_stakable_balance(&6), 60);
			assert_eq!(ParachainStaking::get_delegator_stakable_balance(&7), 10);
			roll_to_round_begin(8);
			roll_blocks(1);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				RuntimeOrigin::signed(2),
				2,
				5
			));
			assert_events_eq!(Event::CandidateLeft {
				ex_candidate: 2,
				unlocked_amount: 140,
				new_total_amt_locked: 120,
			});
			assert_eq!(ParachainStaking::delegator_state(7).unwrap().total(), 10);
			assert_eq!(ParachainStaking::delegator_state(6).unwrap().total(), 30);
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
			assert_eq!(ParachainStaking::get_delegator_stakable_balance(&6), 70);
			assert_eq!(ParachainStaking::get_delegator_stakable_balance(&7), 90);
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
				RuntimeOrigin::signed(2),
				2
			));
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(3),
				2
			));
			// Verifies the revocation request is present
			assert!(ParachainStaking::delegation_scheduled_requests(&2)
				.iter()
				.any(|x| x.delegator == 3));

			roll_to(16);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				RuntimeOrigin::signed(2),
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
			(6, 100),
			(7, 100),
			(8, 100),
			(9, 100),
			(10, 100),
		])
		.with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20)])
		.with_delegations(vec![
			(6, 1, 10),
			(7, 1, 10),
			(8, 2, 10),
			(9, 2, 10),
			(10, 1, 10),
		])
		.build()
		.execute_with(|| {
			// ~ set block author as 1 for all blocks this round
			set_author(1, 1, POINTS_PER_ROUND);
			set_author(2, 1, POINTS_PER_ROUND);
			roll_to_round_begin(2);
			// chooses top TotalSelectedCandidates (5), in order
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 5,
					round: 2,
					selected_collators_number: 4,
					total_balance: 130,
				},
			);

			set_author(3, 1, POINTS_PER_ROUND);
			set_author(4, 1, POINTS_PER_ROUND);

			roll_to_round_begin(4);
			// distribute total issuance to collator 1 and its delegators 6, 7, 19
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 15,
					round: 4,
					selected_collators_number: 4,
					total_balance: 130,
				},
			);
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 11,
				},
				Event::Rewarded {
					account: 6,
					rewards: 4,
				},
				Event::Rewarded {
					account: 7,
					rewards: 4,
				},
				Event::Rewarded {
					account: 10,
					rewards: 4,
				},
			);
			// ~ set block author as 1 for all blocks this round
			set_author(5, 1, POINTS_PER_ROUND);

			roll_blocks(1);
			// 1. ensure delegators are paid for 2 rounds after they leave
			assert_noop!(
				ParachainStaking::schedule_revoke_delegation(RuntimeOrigin::signed(66), 1),
				Error::<Test>::DelegatorDNE
			);
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(6),
				1,
			));
			assert_events_eq!(Event::DelegationRevocationScheduled {
				round: 4,
				delegator: 6,
				candidate: 1,
				scheduled_exit: 6,
			});
			// fast forward to block in which delegator 6 exit executes
			roll_to_round_begin(5);
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 20,
					round: 5,
					selected_collators_number: 4,
					total_balance: 130,
				},
			);
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 12,
				},
				Event::Rewarded {
					account: 6,
					rewards: 4,
				},
				Event::Rewarded {
					account: 7,
					rewards: 4,
				},
				Event::Rewarded {
					account: 10,
					rewards: 4,
				},
			);

			set_author(6, 1, POINTS_PER_ROUND);
			// keep paying 6 (note: inflation is in terms of total issuance so that's why 1 is 21)
			roll_to_round_begin(6);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(6),
				6,
				1,
			));
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 25,
					round: 6,
					selected_collators_number: 4,
					total_balance: 130,
				},
				Event::DelegatorLeftCandidate {
					delegator: 6,
					candidate: 1,
					unstaked_amount: 10,
					total_candidate_staked: 40,
				},
				Event::DelegationRevoked {
					delegator: 6,
					candidate: 1,
					unstaked_amount: 10,
				},
				Event::DelegatorLeft {
					delegator: 6,
					unstaked_amount: 10,
				},
			);
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 12,
				},
				Event::Rewarded {
					account: 6,
					rewards: 4,
				},
				Event::Rewarded {
					account: 7,
					rewards: 4,
				},
				Event::Rewarded {
					account: 10,
					rewards: 4,
				},
			);
			// 6 won't be paid for this round because they left already
			set_author(7, 1, POINTS_PER_ROUND);
			roll_to_round_begin(7);
			// keep paying 6
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 30,
					round: 7,
					selected_collators_number: 4,
					total_balance: 120,
				},
			);
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 14,
				},
				Event::Rewarded {
					account: 7,
					rewards: 5,
				},
				Event::Rewarded {
					account: 10,
					rewards: 5,
				},
			);
			set_author(8, 1, POINTS_PER_ROUND);
			roll_to_round_begin(8);
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 35,
					round: 8,
					selected_collators_number: 4,
					total_balance: 120,
				},
			);
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 15,
				},
				Event::Rewarded {
					account: 7,
					rewards: 5,
				},
				Event::Rewarded {
					account: 10,
					rewards: 5,
				},
			);
			set_author(9, 1, POINTS_PER_ROUND);
			roll_to_round_begin(9);
			// no more paying 6
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 40,
					round: 9,
					selected_collators_number: 4,
					total_balance: 120,
				},
			);
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 15,
				},
				Event::Rewarded {
					account: 7,
					rewards: 5,
				},
				Event::Rewarded {
					account: 10,
					rewards: 5,
				},
			);
			roll_blocks(1);
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(8),
				1,
				10,
				Percent::zero(),
				10,
				0,
				10
			));
			assert_events_eq!(Event::Delegation {
				delegator: 8,
				locked_amount: 10,
				candidate: 1,
				delegator_position: DelegatorAdded::AddedToTop { new_total: 50 },
				auto_compound: Percent::zero(),
			});

			set_author(10, 1, POINTS_PER_ROUND);
			roll_to_round_begin(10);
			// new delegation is not rewarded yet
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 45,
					round: 10,
					selected_collators_number: 4,
					total_balance: 130,
				},
			);
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 15,
				},
				Event::Rewarded {
					account: 7,
					rewards: 5,
				},
				Event::Rewarded {
					account: 10,
					rewards: 5,
				},
			);
			roll_to_round_begin(11);
			// new delegation not rewarded yet
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 50,
					round: 11,
					selected_collators_number: 4,
					total_balance: 130,
				},
			);
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 15,
				},
				Event::Rewarded {
					account: 7,
					rewards: 5,
				},
				Event::Rewarded {
					account: 10,
					rewards: 5,
				},
			);
			roll_to_round_begin(12);
			// new delegation is rewarded for first time
			// 2 rounds after joining (`RewardPaymentDelay` = 2)
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 55,
					round: 12,
					selected_collators_number: 4,
					total_balance: 130,
				},
			);
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 14,
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
			);
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
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				10,
				Percent::zero(),
				10,
				0,
				10
			));
			let top_delegations = ParachainStaking::top_delegations(1).unwrap();
			let bottom_delegations = ParachainStaking::bottom_delegations(1).unwrap();
			assert_eq!(top_delegations.delegations.len(), 1usize);
			assert!(bottom_delegations.delegations.is_empty());
			// 2 delegators => 2 top delegators, 0 bottom delegators
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(3),
				1,
				10,
				Percent::zero(),
				10,
				0,
				10
			));
			let top_delegations = ParachainStaking::top_delegations(1).unwrap();
			let bottom_delegations = ParachainStaking::bottom_delegations(1).unwrap();
			assert_eq!(top_delegations.delegations.len(), 2usize);
			assert!(bottom_delegations.delegations.is_empty());
			// 3 delegators => 3 top delegators, 0 bottom delegators
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(4),
				1,
				10,
				Percent::zero(),
				10,
				0,
				10
			));
			let top_delegations = ParachainStaking::top_delegations(1).unwrap();
			let bottom_delegations = ParachainStaking::bottom_delegations(1).unwrap();
			assert_eq!(top_delegations.delegations.len(), 3usize);
			assert!(bottom_delegations.delegations.is_empty());
			// 4 delegators => 4 top delegators, 0 bottom delegators
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(5),
				1,
				10,
				Percent::zero(),
				10,
				0,
				10
			));
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
				RuntimeOrigin::signed(3),
				1,
				8
			));
			// 3: 11 -> 19 => 3 is in top, bumps out 7
			// 16 + 17 + 18 + 19 + 20 = 90 (top 4 + self bond)
			is_candidate_pool_bond(1, 90);
			assert_ok!(ParachainStaking::delegator_bond_more(
				RuntimeOrigin::signed(4),
				1,
				8
			));
			// 4: 12 -> 20 => 4 is in top, bumps out 8
			// 17 + 18 + 19 + 20 + 20 = 94 (top 4 + self bond)
			is_candidate_pool_bond(1, 94);
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				RuntimeOrigin::signed(10),
				1,
				3
			));
			roll_to(30);
			// 10: 18 -> 15 => 10 bumped to bottom, 8 bumped to top (- 18 + 16 = -2 for count)
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(10),
				10,
				1
			));
			// 16 + 17 + 19 + 20 + 20 = 92 (top 4 + self bond)
			is_candidate_pool_bond(1, 92);
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				RuntimeOrigin::signed(9),
				1,
				4
			));
			roll_to(40);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(9),
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
				RuntimeOrigin::signed(3),
				1,
				8
			));
			assert_events_emitted!(Event::DelegationIncreased {
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
				RuntimeOrigin::signed(4),
				1,
				8
			));
			assert_events_emitted!(Event::DelegationIncreased {
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
				RuntimeOrigin::signed(5),
				1,
				8
			));
			assert_events_emitted!(Event::DelegationIncreased {
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
				RuntimeOrigin::signed(6),
				1,
				8
			));
			assert_events_emitted!(Event::DelegationIncreased {
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
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(7),
				1,
				15,
				Percent::zero(),
				10,
				0,
				10
			));
			assert_events_emitted!(Event::Delegation {
				delegator: 7,
				locked_amount: 15,
				candidate: 1,
				delegator_position: DelegatorAdded::AddedToTop { new_total: 74 },
				auto_compound: Percent::zero(),
			});
			let collator1_state = ParachainStaking::candidate_info(1).unwrap();
			// 12 + 13 + 14 + 15 + 20 = 70 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 74);
			// New delegation is added to the bottom
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(8),
				1,
				10,
				Percent::zero(),
				10,
				0,
				10
			));
			assert_events_emitted!(Event::Delegation {
				delegator: 8,
				locked_amount: 10,
				candidate: 1,
				delegator_position: DelegatorAdded::AddedToBottom,
				auto_compound: Percent::zero(),
			});
			let collator1_state = ParachainStaking::candidate_info(1).unwrap();
			// 12 + 13 + 14 + 15 + 20 = 70 (top 4 + self bond)
			assert_eq!(collator1_state.total_counted, 74);
			// 8 increases delegation to the top
			assert_ok!(ParachainStaking::delegator_bond_more(
				RuntimeOrigin::signed(8),
				1,
				3
			));
			assert_events_emitted!(Event::DelegationIncreased {
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
				RuntimeOrigin::signed(3),
				1,
				1
			));
			assert_events_emitted!(Event::DelegationIncreased {
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
				RuntimeOrigin::signed(6),
				1,
				2
			));
			assert_events_emitted!(Event::DelegationDecreaseScheduled {
				delegator: 6,
				candidate: 1,
				amount_to_decrease: 2,
				execute_round: 3,
			});
			roll_to(30);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(6),
				6,
				1
			));
			assert_events_emitted!(Event::DelegationDecreased {
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
				RuntimeOrigin::signed(6),
				1,
				1
			));
			assert_events_emitted!(Event::DelegationDecreaseScheduled {
				delegator: 6,
				candidate: 1,
				amount_to_decrease: 1,
				execute_round: 9,
			});
			roll_to(40);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(6),
				6,
				1
			));
			assert_events_emitted!(Event::DelegationDecreased {
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
		.with_balances(vec![(1, 20), (2, 20), (3, 20)])
		.with_candidates(vec![(1, 20), (2, 20), (3, 20)])
		.build()
		.execute_with(|| {
			// payouts for round 1
			set_author(1, 1, POINTS_PER_BLOCK);
			set_author(1, 2, POINTS_PER_BLOCK * 2);
			set_author(1, 3, POINTS_PER_BLOCK * 2);

			roll_to_round_begin(2);
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 5,
					round: 2,
					selected_collators_number: 3,
					total_balance: 60,
				},
			);

			roll_to_round_begin(3);
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 10,
					round: 3,
					selected_collators_number: 3,
					total_balance: 60,
				},
			);

			roll_blocks(1);
			assert_events_eq!(Event::Rewarded {
				account: 3,
				rewards: 1,
			});

			roll_blocks(1);
			assert_events_eq!(Event::Rewarded {
				account: 1,
				rewards: 0,
			});

			roll_blocks(1);
			assert_events_eq!(Event::Rewarded {
				account: 2,
				rewards: 1,
			});

			// there should be no more payments in this round...
			let num_blocks_rolled = roll_to_round_end(3);
			assert_no_events!();
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
			set_author(1, 1, POINTS_PER_BLOCK * 3);
			set_author(1, 2, POINTS_PER_BLOCK * 2);

			// reflects genesis?
			assert!(<AtStake<Test>>::contains_key(1, 1));
			assert!(<AtStake<Test>>::contains_key(1, 2));

			roll_to_round_begin(2);
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 5,
					round: 2,
					selected_collators_number: 2,
					total_balance: 40,
				},
			);

			// we should have AtStake snapshots as soon as we start a round...
			assert!(<AtStake<Test>>::contains_key(2, 1));
			assert!(<AtStake<Test>>::contains_key(2, 2));
			// ...and it should persist until the round is fully paid out
			assert!(<AtStake<Test>>::contains_key(1, 1));
			assert!(<AtStake<Test>>::contains_key(1, 2));

			assert!(
				<Points<Test>>::contains_key(1),
				"Points should be populated during current round"
			);

			assert!(
				!<Points<Test>>::contains_key(2),
				"Points should not be populated until author noted"
			);

			// first payout occurs in round 3
			roll_to_round_begin(3);
			assert_events_eq!(
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
				Event::NewRound {
					starting_block: 10,
					round: 3,
					selected_collators_number: 2,
					total_balance: 40,
				},
			);

			roll_blocks(1);
			assert_events_eq!(Event::Rewarded {
				account: 1,
				rewards: 1,
			},);

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
			assert!(<DelayedPayouts<Test>>::contains_key(2));
			assert!(
				<Points<Test>>::contains_key(2),
				"We awarded points for round 2"
			);

			assert!(!<DelayedPayouts<Test>>::contains_key(3));
			assert!(
				<Points<Test>>::contains_key(3),
				"We awarded points for round 3"
			);

			// collator 1 has been paid in this last block and associated storage cleaned up
			assert!(!<AtStake<Test>>::contains_key(1, 1));
			assert!(!<AwardedPts<Test>>::contains_key(1, 1));

			// but collator 2 hasn't been paid
			assert!(<AtStake<Test>>::contains_key(1, 2));
			assert!(<AwardedPts<Test>>::contains_key(1, 2));

			// second payout occurs in next block
			roll_blocks(1);
			assert_events_eq!(Event::Rewarded {
				account: 2,
				rewards: 0,
			},);

			roll_to_round_begin(4);
			assert_events_eq!(
				Event::CollatorChosen {
					round: 4,
					collator_account: 1,
					total_exposed_amount: 20,
				},
				Event::CollatorChosen {
					round: 4,
					collator_account: 2,
					total_exposed_amount: 20,
				},
				Event::NewRound {
					starting_block: 15,
					round: 4,
					selected_collators_number: 2,
					total_balance: 40,
				},
			);

			// collators have both been paid and storage fully cleaned up for round 1
			assert!(!<AtStake<Test>>::contains_key(1, 2));
			assert!(!<AwardedPts<Test>>::contains_key(1, 2));
			assert!(!<Points<Test>>::contains_key(1)); // points should be cleaned up
			assert!(!<DelayedPayouts<Test>>::contains_key(1));

			roll_to_round_end(4);

			// no more events expected
			assert_no_events!();
		});
}

#[test]
fn deferred_payment_and_at_stake_storage_items_cleaned_up_for_candidates_not_producing_blocks() {
	use crate::*;

	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20), (3, 20)])
		.with_candidates(vec![(1, 20), (2, 20), (3, 20)])
		.build()
		.execute_with(|| {
			// candidate 3 will not produce blocks
			set_author(1, 1, POINTS_PER_BLOCK * 3);
			set_author(1, 2, POINTS_PER_BLOCK * 2);

			// reflects genesis?
			assert!(<AtStake<Test>>::contains_key(1, 1));
			assert!(<AtStake<Test>>::contains_key(1, 2));

			roll_to_round_begin(2);
			assert!(<AtStake<Test>>::contains_key(1, 1));
			assert!(<AtStake<Test>>::contains_key(1, 2));
			assert!(<AtStake<Test>>::contains_key(1, 3));
			assert!(<AwardedPts<Test>>::contains_key(1, 1));
			assert!(<AwardedPts<Test>>::contains_key(1, 2));
			assert!(!<AwardedPts<Test>>::contains_key(1, 3));
			assert!(<Points<Test>>::contains_key(1));
			roll_to_round_begin(3);
			assert!(<DelayedPayouts<Test>>::contains_key(1));

			// all storage items must be cleaned up
			roll_to_round_begin(4);
			assert!(!<AtStake<Test>>::contains_key(1, 1));
			assert!(!<AtStake<Test>>::contains_key(1, 2));
			assert!(!<AtStake<Test>>::contains_key(1, 3));
			assert!(!<AwardedPts<Test>>::contains_key(1, 1));
			assert!(!<AwardedPts<Test>>::contains_key(1, 2));
			assert!(!<AwardedPts<Test>>::contains_key(1, 3));
			assert!(!<Points<Test>>::contains_key(1));
			assert!(!<DelayedPayouts<Test>>::contains_key(1));
		});
}

#[test]
fn deferred_payment_steady_state_event_flow() {
	// this test "flows" through a number of rounds, asserting that certain things do/don't happen
	// once the staking pallet is in a "steady state" (specifically, once we are past the first few
	// rounds to clear RewardPaymentDelay)
	use crate::mock::System;

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
			let set_round_points = |round: BlockNumber| {
				set_author(round as BlockNumber, 1, 2 * POINTS_PER_ROUND);
				set_author(round as BlockNumber, 2, POINTS_PER_ROUND);
				set_author(round as BlockNumber, 3, POINTS_PER_ROUND);
				set_author(round as BlockNumber, 4, POINTS_PER_ROUND);
			};

			// grab initial issuance -- we will reset it before round issuance is calculated so that
			// it is consistent every round
			let account: AccountId = 111;
			let initial_issuance = Balances::total_issuance();
			let reset_issuance = || {
				let new_issuance = Balances::total_issuance();
				let amount_to_burn = new_issuance - initial_issuance;
				let _ = Balances::burn(Some(account).into(), amount_to_burn, false);
				System::assert_last_event(RuntimeEvent::Balances(BalancesEvent::Burned {
					who: account,
					amount: amount_to_burn,
				}));
				Balances::settle(
					&account,
					PositiveImbalance::new(amount_to_burn),
					WithdrawReasons::FEE,
					ExistenceRequirement::AllowDeath,
				)
				.expect("Account can absorb burn");
			};

			// fn to roll through the first RewardPaymentDelay rounds. returns new round index
			let roll_through_initial_rounds = |mut round: BlockNumber| -> BlockNumber {
				while round < crate::mock::RewardPaymentDelay::get() + 1 {
					set_round_points(round);

					roll_to_round_end(round);
					round += 1;
				}
				reset_issuance();

				round
			};

			// roll through a "steady state" round and make all of our assertions
			// returns new round index
			let roll_through_steady_state_round = |round: BlockNumber| -> BlockNumber {
				let num_rounds_rolled = roll_to_round_begin(round);
				assert!(
					num_rounds_rolled <= 1,
					"expected to be at round begin already"
				);

				assert_events_eq!(
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
						starting_block: (round as u32 - 1) * 5,
						round: round as u32,
						selected_collators_number: 4,
						total_balance: 1600,
					},
				);

				set_round_points(round);

				roll_blocks(1);
				assert_events_eq!(
					Event::Rewarded {
						account: 3,
						rewards: 13,
					},
					Event::Rewarded {
						account: 22,
						rewards: 4,
					},
					Event::Rewarded {
						account: 33,
						rewards: 4,
					},
				);

				roll_blocks(1);
				assert_events_eq!(
					Event::Rewarded {
						account: 4,
						rewards: 13,
					},
					Event::Rewarded {
						account: 33,
						rewards: 4,
					},
					Event::Rewarded {
						account: 44,
						rewards: 4,
					},
				);

				roll_blocks(1);
				assert_events_eq!(
					Event::Rewarded {
						account: 1,
						rewards: 27,
					},
					Event::Rewarded {
						account: 11,
						rewards: 9,
					},
					Event::Rewarded {
						account: 44,
						rewards: 9,
					},
				);

				roll_blocks(1);
				assert_events_eq!(
					Event::Rewarded {
						account: 2,
						rewards: 13,
					},
					Event::Rewarded {
						account: 11,
						rewards: 4,
					},
					Event::Rewarded {
						account: 22,
						rewards: 4,
					},
				);

				roll_blocks(1);
				// Since we defer first deferred staking payout, this test have the maximum amout of
				// supported collators. This eman that the next round is trigerred one block after
				// the last reward.
				//assert_no_events!();

				let num_rounds_rolled = roll_to_round_end(round);
				assert_eq!(num_rounds_rolled, 0, "expected to be at round end already");

				reset_issuance();

				round + 1
			};

			let mut round = 1;
			round = roll_through_initial_rounds(round); // we should be at RewardPaymentDelay
			for _ in 1..2 {
				round = roll_through_steady_state_round(round);
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
				RuntimeOrigin::signed(2),
				1
			));
			// 10 delegates to full 1 => kicks lowest delegation (2, 19)
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(10),
				1,
				20,
				Percent::zero(),
				8,
				0,
				0,
			));
			// check the event
			assert_events_emitted!(Event::DelegationKicked {
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
					RuntimeOrigin::signed(i),
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
					RuntimeOrigin::signed(i),
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
			(1..=3).for_each(|round| set_author(round, 1, POINTS_PER_ROUND));

			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				1
			));
			assert_events_eq!(Event::DelegationRevocationScheduled {
				round: 1,
				delegator: 2,
				candidate: 1,
				scheduled_exit: 3,
			});
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
			assert_events_emitted_match!(Event::NewRound { round: 3, .. });
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 2,
				},
				Event::Rewarded {
					account: 2,
					rewards: 1,
				},
			);

			roll_to_round_begin(4);
			assert_events_emitted_match!(Event::NewRound { round: 4, .. });
			roll_blocks(3);
			assert_events_eq!(Event::Rewarded {
				account: 1,
				rewards: 2,
			},);
			let collator_snapshot =
				ParachainStaking::at_stake(ParachainStaking::round().current, 1)
					.unwrap_or_default();
			assert_eq!(
				1,
				collator_snapshot.delegations.len(),
				"collator snapshot's delegator count was reduced unexpectedly"
			);
			assert_eq!(
				20, collator_snapshot.total,
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
			(2..=4).for_each(|round| set_author(round, 1, POINTS_PER_ROUND));

			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				1
			));
			assert_events_eq!(Event::DelegationRevocationScheduled {
				round: 1,
				delegator: 2,
				candidate: 1,
				scheduled_exit: 3,
			});
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
				RuntimeOrigin::signed(2),
				1
			));

			roll_to_round_begin(4);
			assert_events_emitted_match!(Event::NewRound { round: 4, .. });
			roll_blocks(3);
			assert_events_eq!(Event::Rewarded {
				account: 1,
				rewards: 2,
			},);
			let collator_snapshot =
				ParachainStaking::at_stake(ParachainStaking::round().current, 1)
					.unwrap_or_default();
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
			assert_events_emitted_match!(Event::NewRound { round: 5, .. });
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 1,
				},
				Event::Rewarded {
					account: 2,
					rewards: 1,
				},
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
			(1..=3).for_each(|round| set_author(round, 1, POINTS_PER_ROUND));

			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				RuntimeOrigin::signed(2),
				1,
				10,
			));
			assert_events_eq!(Event::DelegationDecreaseScheduled {
				execute_round: 3,
				delegator: 2,
				candidate: 1,
				amount_to_decrease: 10,
			});
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
			assert_events_emitted_match!(Event::NewRound { round: 3, .. });
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 2,
				},
				Event::Rewarded {
					account: 2,
					rewards: 1,
				},
			);

			roll_to_round_begin(4);
			assert_events_emitted_match!(Event::NewRound { round: 4, .. });
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 1,
				},
				Event::Rewarded {
					account: 2,
					rewards: 1,
				},
			);
			let collator_snapshot =
				ParachainStaking::at_stake(ParachainStaking::round().current, 1)
					.unwrap_or_default();
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
fn test_delegator_scheduled_for_bond_decrease_is_rewarded_when_request_cancelled() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 40), (3, 20), (4, 20)])
		.with_candidates(vec![(1, 20), (3, 20), (4, 20)])
		.with_delegations(vec![(2, 1, 20), (2, 3, 10)])
		.build()
		.execute_with(|| {
			// preset rewards for rounds 2, 3 and 4
			(2..=4).for_each(|round| set_author(round, 1, POINTS_PER_ROUND));

			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				RuntimeOrigin::signed(2),
				1,
				10,
			));
			assert_events_eq!(Event::DelegationDecreaseScheduled {
				execute_round: 3,
				delegator: 2,
				candidate: 1,
				amount_to_decrease: 10,
			});
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
				RuntimeOrigin::signed(2),
				1
			));

			roll_to_round_begin(4);
			assert_events_emitted_match!(Event::NewRound { round: 4, .. });
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 1,
				},
				Event::Rewarded {
					account: 2,
					rewards: 1,
				},
			);
			let collator_snapshot =
				ParachainStaking::at_stake(ParachainStaking::round().current, 1)
					.unwrap_or_default();
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
			assert_events_emitted_match!(Event::NewRound { round: 5, .. });
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 1,
				},
				Event::Rewarded {
					account: 2,
					rewards: 1,
				},
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
			(1..=3).for_each(|round| set_author(round, 1, POINTS_PER_ROUND));

			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				1,
			));
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				3,
			));
			assert_events_eq!(
				Event::DelegationRevocationScheduled {
					round: 1,
					delegator: 2,
					candidate: 1,
					scheduled_exit: 3,
				},
				Event::DelegationRevocationScheduled {
					round: 1,
					delegator: 2,
					candidate: 3,
					scheduled_exit: 3,
				},
			);
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
			assert_events_emitted_match!(Event::NewRound { round: 3, .. });
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 2,
				},
				Event::Rewarded {
					account: 2,
					rewards: 1,
				},
			);

			roll_to_round_begin(4);
			assert_events_emitted_match!(Event::NewRound { round: 4, .. });
			roll_blocks(3);
			assert_events_eq!(Event::Rewarded {
				account: 1,
				rewards: 2,
			},);
			let collator_snapshot =
				ParachainStaking::at_stake(ParachainStaking::round().current, 1)
					.unwrap_or_default();
			assert_eq!(
				1,
				collator_snapshot.delegations.len(),
				"collator snapshot's delegator count was reduced unexpectedly"
			);
			assert_eq!(
				20, collator_snapshot.total,
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
			(2..=4).for_each(|round| set_author(round, 1, POINTS_PER_ROUND));

			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				1,
			));
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				3,
			));
			assert_events_eq!(
				Event::DelegationRevocationScheduled {
					round: 1,
					delegator: 2,
					candidate: 1,
					scheduled_exit: 3,
				},
				Event::DelegationRevocationScheduled {
					round: 1,
					delegator: 2,
					candidate: 3,
					scheduled_exit: 3,
				},
			);
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
				RuntimeOrigin::signed(2),
				1,
			));
			assert_ok!(ParachainStaking::cancel_delegation_request(
				RuntimeOrigin::signed(2),
				3,
			));

			roll_to_round_begin(4);
			assert_events_emitted_match!(Event::NewRound { round: 4, .. });
			roll_blocks(3);
			assert_events_eq!(Event::Rewarded {
				account: 1,
				rewards: 2,
			},);
			let collator_snapshot =
				ParachainStaking::at_stake(ParachainStaking::round().current, 1)
					.unwrap_or_default();
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
			assert_events_emitted_match!(Event::NewRound { round: 5, .. });
			roll_blocks(3);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 1,
				},
				Event::Rewarded {
					account: 2,
					rewards: 1,
				},
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
				BoundedVec::try_from(vec![ScheduledRequest {
					delegator: 2,
					when_executable: 3,
					action: DelegationAction::Decrease(5),
				}])
				.expect("must succeed"),
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
				BoundedVec::try_from(vec![ScheduledRequest {
					delegator: 2,
					when_executable: 3,
					action: DelegationAction::Revoke(5),
				}])
				.expect("must succeed"),
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
				BoundedVec::try_from(vec![ScheduledRequest {
					delegator: 2,
					when_executable: 3,
					action: DelegationAction::Decrease(5),
				}])
				.expect("must succeed"),
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
				BoundedVec::try_from(vec![ScheduledRequest {
					delegator: 2,
					when_executable: 3,
					action: DelegationAction::Revoke(5),
				}])
				.expect("must succeed"),
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
			<DelegationScheduledRequests<Test>>::insert(2, BoundedVec::default());
			<DelegationScheduledRequests<Test>>::insert(3, BoundedVec::default());
			assert_ok!(
				ParachainStaking::hotfix_remove_delegation_requests_exited_candidates(
					RuntimeOrigin::signed(1),
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
			<DelegationScheduledRequests<Test>>::insert(2, BoundedVec::default());
			<DelegationScheduledRequests<Test>>::insert(3, BoundedVec::default());
			assert_ok!(
				ParachainStaking::hotfix_remove_delegation_requests_exited_candidates(
					RuntimeOrigin::signed(1),
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
			<DelegationScheduledRequests<Test>>::insert(2, BoundedVec::default());
			<DelegationScheduledRequests<Test>>::insert(
				3,
				BoundedVec::try_from(vec![ScheduledRequest {
					delegator: 10,
					when_executable: 1,
					action: DelegationAction::Revoke(10),
				}])
				.expect("must succeed"),
			);

			assert_noop!(
				ParachainStaking::hotfix_remove_delegation_requests_exited_candidates(
					RuntimeOrigin::signed(1),
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
			<DelegationScheduledRequests<Test>>::insert(1, BoundedVec::default());
			assert_noop!(
				ParachainStaking::hotfix_remove_delegation_requests_exited_candidates(
					RuntimeOrigin::signed(1),
					vec![1]
				),
				<Error<Test>>::CandidateNotLeaving,
			);
		});
}

#[test]
fn freezing_zero_amount_thaws_freeze() {
	use crate::mock::query_freeze_amount;
	use frame_support::traits::fungible::MutateFreeze;

	// this test demonstrates the behavior of fungible's freeze mechanism

	ExtBuilder::default()
		.with_balances(vec![(1, 100)])
		.build()
		.execute_with(|| {
			let reason = &crate::pallet::FreezeReason::StakingDelegator.into();
			assert_eq!(query_freeze_amount(1, reason), 0);

			// Freeze 1 unit
			assert_ok!(Balances::set_freeze(reason, &1, 1));
			assert_eq!(query_freeze_amount(1, reason), 1);

			// Thaw the freeze
			assert_ok!(Balances::thaw(reason, &1));
			assert_eq!(query_freeze_amount(1, reason), 0);
		});
}

#[test]
fn revoke_last_removes_freeze() {
	use crate::mock::query_freeze_amount;

	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 100), (3, 100)])
		.with_candidates(vec![(1, 25), (2, 25)])
		.with_delegations(vec![(3, 1, 30), (3, 2, 25)])
		.build()
		.execute_with(|| {
			let reason = &crate::pallet::FreezeReason::StakingDelegator.into();
			assert_eq!(query_freeze_amount(3, reason), 55);

			// schedule and remove one...
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(3),
				1
			));
			roll_to_round_begin(3);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(3),
				3,
				1
			));
			assert_eq!(query_freeze_amount(3, reason), 25);

			// schedule and remove the other...
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(3),
				2
			));
			roll_to_round_begin(5);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(3),
				3,
				2
			));
			assert_eq!(query_freeze_amount(3, reason), 0);
		});
}

#[test]
fn test_set_auto_compound_fails_if_invalid_delegation_hint() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			let candidate_auto_compounding_delegation_count_hint = 0;
			let delegation_hint = 0; // is however, 1

			assert_noop!(
				ParachainStaking::set_auto_compound(
					RuntimeOrigin::signed(2),
					1,
					Percent::from_percent(50),
					candidate_auto_compounding_delegation_count_hint,
					delegation_hint,
				),
				<Error<Test>>::TooLowDelegationCountToAutoCompound,
			);
		});
}

#[test]
fn test_set_auto_compound_fails_if_invalid_candidate_auto_compounding_hint() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			<AutoCompoundDelegations<Test>>::new(
				vec![AutoCompoundConfig {
					delegator: 2,
					value: Percent::from_percent(10),
				}]
				.try_into()
				.expect("must succeed"),
			)
			.set_storage(&1);
			let candidate_auto_compounding_delegation_count_hint = 0; // is however, 1
			let delegation_hint = 1;

			assert_noop!(
				ParachainStaking::set_auto_compound(
					RuntimeOrigin::signed(2),
					1,
					Percent::from_percent(50),
					candidate_auto_compounding_delegation_count_hint,
					delegation_hint,
				),
				<Error<Test>>::TooLowCandidateAutoCompoundingDelegationCountToAutoCompound,
			);
		});
}

#[test]
fn test_set_auto_compound_inserts_if_not_exists() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::set_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				Percent::from_percent(50),
				0,
				1,
			));
			assert_events_emitted!(Event::AutoCompoundSet {
				candidate: 1,
				delegator: 2,
				value: Percent::from_percent(50),
			});
			assert_eq!(
				vec![AutoCompoundConfig {
					delegator: 2,
					value: Percent::from_percent(50),
				}],
				ParachainStaking::auto_compounding_delegations(&1).into_inner(),
			);
		});
}

#[test]
fn test_set_auto_compound_updates_if_existing() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			<AutoCompoundDelegations<Test>>::new(
				vec![AutoCompoundConfig {
					delegator: 2,
					value: Percent::from_percent(10),
				}]
				.try_into()
				.expect("must succeed"),
			)
			.set_storage(&1);

			assert_ok!(ParachainStaking::set_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				Percent::from_percent(50),
				1,
				1,
			));
			assert_events_emitted!(Event::AutoCompoundSet {
				candidate: 1,
				delegator: 2,
				value: Percent::from_percent(50),
			});
			assert_eq!(
				vec![AutoCompoundConfig {
					delegator: 2,
					value: Percent::from_percent(50),
				}],
				ParachainStaking::auto_compounding_delegations(&1).into_inner(),
			);
		});
}

#[test]
fn test_set_auto_compound_removes_if_auto_compound_zero_percent() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			<AutoCompoundDelegations<Test>>::new(
				vec![AutoCompoundConfig {
					delegator: 2,
					value: Percent::from_percent(10),
				}]
				.try_into()
				.expect("must succeed"),
			)
			.set_storage(&1);

			assert_ok!(ParachainStaking::set_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				Percent::zero(),
				1,
				1,
			));
			assert_events_emitted!(Event::AutoCompoundSet {
				candidate: 1,
				delegator: 2,
				value: Percent::zero(),
			});
			assert_eq!(0, ParachainStaking::auto_compounding_delegations(&1).len(),);
		});
}

#[test]
fn test_execute_revoke_delegation_removes_auto_compounding_from_state_for_delegation_revoke() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 30), (3, 20)])
		.with_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::set_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				Percent::from_percent(50),
				0,
				2,
			));
			assert_ok!(ParachainStaking::set_auto_compound(
				RuntimeOrigin::signed(2),
				3,
				Percent::from_percent(50),
				0,
				2,
			));
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				1
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
				2,
				1
			));
			assert!(
				!ParachainStaking::auto_compounding_delegations(&1)
					.iter()
					.any(|x| x.delegator == 2),
				"delegation auto-compound config was not removed"
			);
			assert!(
				ParachainStaking::auto_compounding_delegations(&3)
					.iter()
					.any(|x| x.delegator == 2),
				"delegation auto-compound config was erroneously removed"
			);
		});
}

#[test]
fn test_execute_leave_delegators_removes_auto_compounding_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 20), (3, 20)])
		.with_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::set_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				Percent::from_percent(50),
				0,
				2,
			));
			assert_ok!(ParachainStaking::set_auto_compound(
				RuntimeOrigin::signed(2),
				3,
				Percent::from_percent(50),
				0,
				2,
			));

			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				1,
			));
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				3,
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
				2,
				1,
			));
			assert_ok!(ParachainStaking::execute_delegation_request(
				RuntimeOrigin::signed(2),
				2,
				3,
			));

			assert!(
				!ParachainStaking::auto_compounding_delegations(&1)
					.iter()
					.any(|x| x.delegator == 2),
				"delegation auto-compound config was not removed"
			);
			assert!(
				!ParachainStaking::auto_compounding_delegations(&3)
					.iter()
					.any(|x| x.delegator == 2),
				"delegation auto-compound config was not removed"
			);
		});
}

#[test]
fn test_execute_leave_candidates_removes_auto_compounding_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 20), (3, 20)])
		.with_candidates(vec![(1, 30), (3, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::set_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				Percent::from_percent(50),
				0,
				2,
			));
			assert_ok!(ParachainStaking::set_auto_compound(
				RuntimeOrigin::signed(2),
				3,
				Percent::from_percent(50),
				0,
				2,
			));

			assert_ok!(ParachainStaking::schedule_leave_candidates(
				RuntimeOrigin::signed(1),
				2
			));
			roll_to(10);
			assert_ok!(ParachainStaking::execute_leave_candidates(
				RuntimeOrigin::signed(1),
				1,
				1,
			));

			assert!(
				!ParachainStaking::auto_compounding_delegations(&1)
					.iter()
					.any(|x| x.delegator == 2),
				"delegation auto-compound config was not removed"
			);
			assert!(
				ParachainStaking::auto_compounding_delegations(&3)
					.iter()
					.any(|x| x.delegator == 2),
				"delegation auto-compound config was erroneously removed"
			);
		});
}

#[test]
fn test_delegation_kicked_from_bottom_delegation_removes_auto_compounding_state() {
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
			(2, 11, 10), // extra delegation to avoid leaving the delegator set
			(2, 1, 19),
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
			assert_ok!(ParachainStaking::set_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				Percent::from_percent(50),
				0,
				2,
			));

			// kicks lowest delegation (2, 19)
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(10),
				1,
				20,
				Percent::zero(),
				8,
				0,
				0,
			));

			assert!(
				!ParachainStaking::auto_compounding_delegations(&1)
					.iter()
					.any(|x| x.delegator == 2),
				"delegation auto-compound config was not removed"
			);
		});
}

#[test]
fn test_rewards_do_not_auto_compound_on_payment_if_delegation_scheduled_revoke_exists() {
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 200), (3, 200)])
		.with_candidates(vec![(1, 100)])
		.with_delegations(vec![(2, 1, 200), (3, 1, 200)])
		.build()
		.execute_with(|| {
			(2..=5).for_each(|round| set_author(round, 1, POINTS_PER_ROUND));
			assert_ok!(ParachainStaking::set_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				Percent::from_percent(50),
				0,
				1,
			));
			assert_ok!(ParachainStaking::set_auto_compound(
				RuntimeOrigin::signed(3),
				1,
				Percent::from_percent(50),
				1,
				1,
			));
			roll_to_round_begin(3);

			// schedule revoke for delegator 2; no rewards should be compounded
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				1
			));
			roll_to_round_begin(4);

			assert_events_eq!(
				Event::CollatorChosen {
					round: 4,
					collator_account: 1,
					total_exposed_amount: 500,
				},
				Event::NewRound {
					starting_block: 15,
					round: 4,
					selected_collators_number: 1,
					total_balance: 500,
				},
			);

			roll_blocks(1);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 4,
				},
				// no compound since revoke request exists
				Event::Rewarded {
					account: 2,
					rewards: 4,
				},
				// 50%
				Event::Rewarded {
					account: 3,
					rewards: 4,
				},
				Event::Compounded {
					candidate: 1,
					delegator: 3,
					amount: 2,
				},
			);
		});
}

#[test]
fn test_rewards_auto_compound_on_payment_as_per_auto_compound_config() {
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 200), (3, 200), (4, 200), (5, 200)])
		.with_candidates(vec![(1, 100)])
		.with_delegations(vec![(2, 1, 200), (3, 1, 200), (4, 1, 200), (5, 1, 200)])
		.build()
		.execute_with(|| {
			(2..=6).for_each(|round| set_author(round, 1, POINTS_PER_ROUND));
			assert_ok!(ParachainStaking::set_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				Percent::from_percent(0),
				0,
				1,
			));
			assert_ok!(ParachainStaking::set_auto_compound(
				RuntimeOrigin::signed(3),
				1,
				Percent::from_percent(50),
				1,
				1,
			));
			assert_ok!(ParachainStaking::set_auto_compound(
				RuntimeOrigin::signed(4),
				1,
				Percent::from_percent(100),
				2,
				1,
			));
			roll_to_round_begin(4);

			assert_events_eq!(
				Event::CollatorChosen {
					round: 4,
					collator_account: 1,
					total_exposed_amount: 900,
				},
				Event::NewRound {
					starting_block: 15,
					round: 4,
					selected_collators_number: 1,
					total_balance: 900,
				},
			);

			roll_blocks(1);
			assert_events_eq!(
				Event::Rewarded {
					account: 1,
					rewards: 6,
				},
				// 0%
				Event::Rewarded {
					account: 2,
					rewards: 4,
				},
				// 50%
				Event::Rewarded {
					account: 3,
					rewards: 4,
				},
				Event::Compounded {
					candidate: 1,
					delegator: 3,
					amount: 2,
				},
				// 100%
				Event::Rewarded {
					account: 4,
					rewards: 4,
				},
				Event::Compounded {
					candidate: 1,
					delegator: 4,
					amount: 4,
				},
				// no-config
				Event::Rewarded {
					account: 5,
					rewards: 4,
				},
			);
		});
}

#[test]
fn test_delegate_with_auto_compound_fails_if_invalid_delegation_hint() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25), (3, 30)])
		.with_candidates(vec![(1, 30), (3, 30)])
		.with_delegations(vec![(2, 3, 10)])
		.build()
		.execute_with(|| {
			let candidate_delegation_count_hint = 0;
			let candidate_auto_compounding_delegation_count_hint = 0;
			let delegation_hint = 0; // is however, 1

			assert_noop!(
				ParachainStaking::delegate_with_auto_compound(
					RuntimeOrigin::signed(2),
					1,
					10,
					Percent::from_percent(50),
					candidate_delegation_count_hint,
					candidate_auto_compounding_delegation_count_hint,
					delegation_hint,
				),
				<Error<Test>>::TooLowDelegationCountToDelegate,
			);
		});
}

#[test]
fn test_delegate_with_auto_compound_fails_if_invalid_candidate_delegation_count_hint() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25), (3, 30)])
		.with_candidates(vec![(1, 30)])
		.with_delegations(vec![(3, 1, 10)])
		.build()
		.execute_with(|| {
			let candidate_delegation_count_hint = 0; // is however, 1
			let candidate_auto_compounding_delegation_count_hint = 0;
			let delegation_hint = 0;

			assert_noop!(
				ParachainStaking::delegate_with_auto_compound(
					RuntimeOrigin::signed(2),
					1,
					10,
					Percent::from_percent(50),
					candidate_delegation_count_hint,
					candidate_auto_compounding_delegation_count_hint,
					delegation_hint,
				),
				<Error<Test>>::TooLowCandidateDelegationCountToDelegate,
			);
		});
}

#[test]
fn test_delegate_with_auto_compound_fails_if_invalid_candidate_auto_compounding_delegations_hint() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25), (3, 30)])
		.with_candidates(vec![(1, 30)])
		.with_auto_compounding_delegations(vec![(3, 1, 10, Percent::from_percent(10))])
		.build()
		.execute_with(|| {
			let candidate_delegation_count_hint = 1;
			let candidate_auto_compounding_delegation_count_hint = 0; // is however, 1
			let delegation_hint = 0;

			assert_noop!(
				ParachainStaking::delegate_with_auto_compound(
					RuntimeOrigin::signed(2),
					1,
					10,
					Percent::from_percent(50),
					candidate_delegation_count_hint,
					candidate_auto_compounding_delegation_count_hint,
					delegation_hint,
				),
				<Error<Test>>::TooLowCandidateAutoCompoundingDelegationCountToDelegate,
			);
		});
}

#[test]
fn test_delegate_with_auto_compound_sets_auto_compound_config() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 25)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				10,
				Percent::from_percent(50),
				0,
				0,
				0,
			));
			assert_events_emitted!(Event::Delegation {
				delegator: 2,
				locked_amount: 10,
				candidate: 1,
				delegator_position: DelegatorAdded::AddedToTop { new_total: 40 },
				auto_compound: Percent::from_percent(50),
			});
			assert_eq!(
				vec![AutoCompoundConfig {
					delegator: 2,
					value: Percent::from_percent(50),
				}],
				ParachainStaking::auto_compounding_delegations(&1).into_inner(),
			);
		});
}

#[test]
fn test_delegate_with_auto_compound_skips_storage_but_emits_event_for_zero_auto_compound() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				10,
				Percent::zero(),
				0,
				0,
				0,
			));
			assert_eq!(0, ParachainStaking::auto_compounding_delegations(&1).len(),);
			assert_events_eq!(Event::Delegation {
				delegator: 2,
				locked_amount: 10,
				candidate: 1,
				delegator_position: DelegatorAdded::AddedToTop { new_total: 40 },
				auto_compound: Percent::zero(),
			});
		});
}

#[test]
fn test_delegate_with_auto_compound_reserves_balance() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert_eq!(ParachainStaking::get_delegator_stakable_balance(&2), 10);
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				10,
				Percent::from_percent(50),
				0,
				0,
				0,
			));
			assert_eq!(ParachainStaking::get_delegator_stakable_balance(&2), 0);
		});
}

#[test]
fn test_delegate_with_auto_compound_updates_delegator_state() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 10)])
		.with_candidates(vec![(1, 30)])
		.build()
		.execute_with(|| {
			assert!(ParachainStaking::delegator_state(2).is_none());
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				10,
				Percent::from_percent(50),
				0,
				0,
				0
			));
			let delegator_state =
				ParachainStaking::delegator_state(2).expect("just delegated => exists");
			assert_eq!(delegator_state.total(), 10);
			assert_eq!(delegator_state.delegations.0[0].owner, 1);
			assert_eq!(delegator_state.delegations.0[0].amount, 10);
		});
}

#[test]
fn test_delegate_with_auto_compound_updates_collator_state() {
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
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				10,
				Percent::from_percent(50),
				0,
				0,
				0
			));
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
fn test_delegate_with_auto_compound_can_delegate_immediately_after_other_join_candidates() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::join_candidates(
				RuntimeOrigin::signed(1),
				20,
				0
			));
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				20,
				Percent::from_percent(50),
				0,
				0,
				0
			));
		});
}

#[test]
fn test_delegate_with_auto_compound_can_delegate_to_other_if_revoking() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 30), (3, 20), (4, 20)])
		.with_candidates(vec![(1, 20), (3, 20), (4, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				1
			));
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				4,
				10,
				Percent::from_percent(50),
				0,
				0,
				2
			));
		});
}

#[test]
fn test_delegate_with_auto_compound_cannot_delegate_if_less_than_or_equal_lowest_bottom() {
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
				ParachainStaking::delegate_with_auto_compound(
					RuntimeOrigin::signed(11),
					1,
					10,
					Percent::from_percent(50),
					8,
					0,
					0
				),
				Error::<Test>::CannotDelegateLessThanOrEqualToLowestBottomWhenFull
			);
		});
}

#[test]
fn test_delegate_with_auto_compound_can_delegate_if_greater_than_lowest_bottom() {
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
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(11),
				1,
				11,
				Percent::from_percent(50),
				8,
				0,
				0
			));
			assert_events_emitted!(Event::DelegationKicked {
				delegator: 10,
				candidate: 1,
				unstaked_amount: 10
			});
			assert_events_emitted!(Event::DelegatorLeft {
				delegator: 10,
				unstaked_amount: 10
			});
		});
}

#[test]
fn test_delegate_with_auto_compound_can_still_delegate_to_other_if_leaving() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 20), (3, 20)])
		.with_candidates(vec![(1, 20), (3, 20)])
		.with_delegations(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(2),
				1,
			));
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				3,
				10,
				Percent::from_percent(50),
				0,
				0,
				1
			),);
		});
}

#[test]
fn test_delegate_with_auto_compound_cannot_delegate_if_candidate() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 30)])
		.with_candidates(vec![(1, 20), (2, 20)])
		.build()
		.execute_with(|| {
			assert_noop!(
				ParachainStaking::delegate_with_auto_compound(
					RuntimeOrigin::signed(2),
					1,
					10,
					Percent::from_percent(50),
					0,
					0,
					0
				),
				Error::<Test>::CandidateExists
			);
		});
}

#[test]
fn test_delegate_with_auto_compound_cannot_delegate_if_already_delegated() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 30)])
		.with_candidates(vec![(1, 20)])
		.with_delegations(vec![(2, 1, 20)])
		.build()
		.execute_with(|| {
			assert_noop!(
				ParachainStaking::delegate_with_auto_compound(
					RuntimeOrigin::signed(2),
					1,
					10,
					Percent::from_percent(50),
					0,
					1,
					1
				),
				Error::<Test>::AlreadyDelegatedCandidate
			);
		});
}

#[test]
fn test_delegate_with_auto_compound_cannot_delegate_more_than_max_delegations() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20), (2, 50), (3, 20), (4, 20), (5, 20), (6, 20)])
		.with_candidates(vec![(1, 20), (3, 20), (4, 20), (5, 20), (6, 20)])
		.with_delegations(vec![(2, 1, 10), (2, 3, 10), (2, 4, 10), (2, 5, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				ParachainStaking::delegate_with_auto_compound(
					RuntimeOrigin::signed(2),
					6,
					10,
					Percent::from_percent(50),
					0,
					0,
					4
				),
				Error::<Test>::ExceedMaxDelegationsPerDelegator,
			);
		});
}

#[test]
fn test_delegate_skips_auto_compound_storage_but_emits_event_for_zero_auto_compound() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 20), (3, 30)])
		.with_candidates(vec![(1, 30)])
		.with_auto_compounding_delegations(vec![(3, 1, 10, Percent::from_percent(50))])
		.build()
		.execute_with(|| {
			// We already have an auto-compounding delegation from 3 -> 1, so the hint validation
			// would cause a failure if the auto-compounding isn't skipped properly.
			assert_ok!(ParachainStaking::delegate_with_auto_compound(
				RuntimeOrigin::signed(2),
				1,
				10,
				Percent::zero(),
				1,
				0,
				0,
			));
			assert_eq!(1, ParachainStaking::auto_compounding_delegations(&1).len(),);
			assert_events_eq!(Event::Delegation {
				delegator: 2,
				locked_amount: 10,
				candidate: 1,
				delegator_position: DelegatorAdded::AddedToTop { new_total: 50 },
				auto_compound: Percent::zero(),
			});
		});
}

#[test]
fn test_on_initialize_weights() {
	use crate::mock::System;
	use crate::weights::{SubstrateWeight as PalletWeights, WeightInfo};
	use crate::*;
	use frame_support::{pallet_prelude::*, weights::constants::RocksDbWeight};

	// generate balance, candidate, and delegation vecs to "fill" out delegations
	let mut balances = Vec::new();
	let mut candidates = Vec::new();
	let mut delegations = Vec::new();

	for collator in 1..30 {
		balances.push((collator, 100));
		candidates.push((collator, 10));
		let starting_delegator = collator * 1000;
		for delegator in starting_delegator..starting_delegator + 300 {
			balances.push((delegator, 100));
			delegations.push((delegator, collator, 10));
		}
	}

	ExtBuilder::default()
		.with_balances(balances)
		.with_candidates(candidates)
		.with_delegations(delegations)
		.build()
		.execute_with(|| {
			let weight = ParachainStaking::on_initialize(1);

			// TODO: build this with proper db reads/writes
			assert_eq!(Weight::from_parts(401000000, 0), weight);

			// roll to the end of the round, then run on_init again, we should see round change...
			set_author(3, 1, POINTS_PER_ROUND); // must set some points for prepare_staking_payouts
			roll_to_round_end(3);
			let block = System::block_number() + 1;
			let weight = ParachainStaking::on_initialize(block);

			// the total on_init weight during our round change. this number is taken from running
			// the fn with a given weights.rs benchmark, so will need to be updated as benchmarks
			// change.
			//
			// following this assertion, we add individual weights together to show that we can
			// derive this number independently.
			let expected_on_init = 3018132161;
			assert_eq!(Weight::from_parts(expected_on_init, 51554), weight);

			// assemble weight manually to ensure it is well understood
			let mut expected_weight = 0u64;
			expected_weight += PalletWeights::<Test>::base_on_initialize().ref_time();
			expected_weight += PalletWeights::<Test>::prepare_staking_payouts().ref_time();
			expected_weight += PalletWeights::<Test>::mark_collators_as_inactive(5).ref_time();

			// TODO: this should be the same as <TotalSelected<Test>>. I believe this relates to
			// genesis building
			let num_avg_delegations = 8;
			expected_weight += PalletWeights::<Test>::select_top_candidates(
				<TotalSelected<Test>>::get(),
				num_avg_delegations,
			)
			.ref_time();
			// SlotProvider read
			expected_weight += RocksDbWeight::get().reads_writes(1, 0).ref_time();
			// Round write, done in on-round-change code block inside on_initialize()
			expected_weight += RocksDbWeight::get().reads_writes(0, 1).ref_time();
			// more reads/writes manually accounted for for on_finalize
			expected_weight += RocksDbWeight::get().reads_writes(4, 3).ref_time();

			assert_eq!(Weight::from_parts(expected_weight, 51554), weight);
			assert_eq!(expected_on_init, expected_weight); // magic number == independent accounting
		});
}

#[test]
fn test_compute_top_candidates_is_stable() {
	ExtBuilder::default()
		.with_balances(vec![(1, 30), (2, 30), (3, 30), (4, 30), (5, 30), (6, 30)])
		.with_candidates(vec![(1, 30), (2, 30), (3, 30), (4, 30), (5, 30), (6, 30)])
		.build()
		.execute_with(|| {
			// There are 6 candidates with equal amount, but only 5 can be selected
			assert_eq!(ParachainStaking::candidate_pool().0.len(), 6);
			assert_eq!(ParachainStaking::total_selected(), 5);
			// Returns the 5 candidates with greater AccountId, because they are iterated in reverse
			assert_eq!(
				ParachainStaking::compute_top_candidates(),
				vec![2, 3, 4, 5, 6]
			);
		});
}
