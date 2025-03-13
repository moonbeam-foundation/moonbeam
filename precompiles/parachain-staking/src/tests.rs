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

use crate::mock::{
	events, roll_to, roll_to_round_begin, set_points, ExtBuilder, PCall, ParachainStaking,
	Precompiles, PrecompilesValue, Runtime, RuntimeCall, RuntimeOrigin,
};
use core::str::from_utf8;
use frame_support::assert_ok;
use frame_support::sp_runtime::Percent;
use pallet_evm::Call as EvmCall;
use pallet_parachain_staking::Event as StakingEvent;
use precompile_utils::{prelude::*, testing::*};
use sp_core::{H160, U256};
use sp_runtime::traits::Dispatchable;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

fn evm_call(source: impl Into<H160>, input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: source.into(),
		target: Precompile1.into(),
		input,
		value: U256::zero(), // No value sent in EVM
		gas_limit: u64::max_value(),
		max_fee_per_gas: 0.into(),
		max_priority_fee_per_gas: Some(U256::zero()),
		nonce: None, // Use the next nonce
		access_list: Vec::new(),
	}
}

#[test]
fn selectors() {
	assert!(PCall::is_delegator_selectors().contains(&0xfd8ab482));
	assert!(PCall::is_candidate_selectors().contains(&0xd51b9e93));
	assert!(PCall::is_selected_candidate_selectors().contains(&0x740d7d2a));
	assert!(PCall::delegation_amount_selectors().contains(&0xa73e51bc));
	assert!(PCall::is_in_top_delegations_selectors().contains(&0x91cc8657));
	assert!(PCall::points_selectors().contains(&0x9799b4e7));
	assert!(PCall::min_delegation_selectors().contains(&0x02985992));
	assert!(PCall::candidate_count_selectors().contains(&0xa9a981a3));
	assert!(PCall::round_selectors().contains(&0x146ca531));
	assert!(PCall::candidate_delegation_count_selectors().contains(&0x2ec087eb));
	assert!(PCall::candidate_auto_compounding_delegation_count_selectors().contains(&0x905f0806));
	assert!(PCall::delegator_delegation_count_selectors().contains(&0x067ec822));
	assert!(PCall::selected_candidates_selectors().contains(&0xbcf868a6));
	assert!(PCall::delegation_request_is_pending_selectors().contains(&0x3b16def8));
	assert!(PCall::candidate_exit_is_pending_selectors().contains(&0x43443682));
	assert!(PCall::candidate_request_is_pending_selectors().contains(&0xd0deec11));
	assert!(PCall::join_candidates_selectors().contains(&0x1f2f83ad));
	assert!(PCall::schedule_leave_candidates_selectors().contains(&0xb1a3c1b7));
	assert!(PCall::execute_leave_candidates_selectors().contains(&0x3867f308));
	assert!(PCall::cancel_leave_candidates_selectors().contains(&0x9c76ebb4));
	assert!(PCall::go_offline_selectors().contains(&0xa6485ccd));
	assert!(PCall::go_online_selectors().contains(&0x6e5b676b));
	assert!(PCall::candidate_bond_more_selectors().contains(&0xa52c8643));
	assert!(PCall::schedule_candidate_bond_less_selectors().contains(&0x60744ae0));
	assert!(PCall::execute_candidate_bond_less_selectors().contains(&0x2e290290));
	assert!(PCall::cancel_candidate_bond_less_selectors().contains(&0xb5ad5f07));
	assert!(PCall::delegate_selectors().contains(&0x829f5ee3));
	assert!(PCall::schedule_revoke_delegation_selectors().contains(&0x1a1c740c));
	assert!(PCall::delegator_bond_more_selectors().contains(&0x0465135b));
	assert!(PCall::schedule_delegator_bond_less_selectors().contains(&0xc172fd2b));
	assert!(PCall::execute_delegation_request_selectors().contains(&0xe98c8abe));
	assert!(PCall::cancel_delegation_request_selectors().contains(&0xc90eee83));
	assert!(PCall::get_delegator_total_staked_selectors().contains(&0xe6861713));
	assert!(PCall::get_candidate_total_counted_selectors().contains(&0xbc5a1043));
}

#[test]
fn modifiers() {
	ExtBuilder::default().build().execute_with(|| {
		let mut tester = PrecompilesModifierTester::new(precompiles(), Alice, Precompile1);

		tester.test_view_modifier(PCall::is_delegator_selectors());
		tester.test_view_modifier(PCall::is_candidate_selectors());
		tester.test_view_modifier(PCall::is_selected_candidate_selectors());
		tester.test_view_modifier(PCall::points_selectors());
		tester.test_view_modifier(PCall::delegation_amount_selectors());
		tester.test_view_modifier(PCall::is_in_top_delegations_selectors());
		tester.test_view_modifier(PCall::min_delegation_selectors());
		tester.test_view_modifier(PCall::candidate_count_selectors());
		tester.test_view_modifier(PCall::round_selectors());
		tester.test_view_modifier(PCall::candidate_delegation_count_selectors());
		tester.test_view_modifier(PCall::delegator_delegation_count_selectors());
		tester.test_view_modifier(PCall::selected_candidates_selectors());
		tester.test_view_modifier(PCall::delegation_request_is_pending_selectors());
		tester.test_view_modifier(PCall::candidate_exit_is_pending_selectors());
		tester.test_view_modifier(PCall::candidate_request_is_pending_selectors());
		tester.test_default_modifier(PCall::join_candidates_selectors());
		tester.test_default_modifier(PCall::schedule_leave_candidates_selectors());
		tester.test_default_modifier(PCall::execute_leave_candidates_selectors());
		tester.test_default_modifier(PCall::cancel_leave_candidates_selectors());
		tester.test_default_modifier(PCall::go_offline_selectors());
		tester.test_default_modifier(PCall::go_online_selectors());
		tester.test_default_modifier(PCall::candidate_bond_more_selectors());
		tester.test_default_modifier(PCall::schedule_candidate_bond_less_selectors());
		tester.test_default_modifier(PCall::execute_candidate_bond_less_selectors());
		tester.test_default_modifier(PCall::cancel_candidate_bond_less_selectors());
		tester.test_default_modifier(PCall::delegate_selectors());
		tester.test_default_modifier(PCall::schedule_revoke_delegation_selectors());
		tester.test_default_modifier(PCall::delegator_bond_more_selectors());
		tester.test_default_modifier(PCall::schedule_delegator_bond_less_selectors());
		tester.test_default_modifier(PCall::execute_delegation_request_selectors());
		tester.test_default_modifier(PCall::cancel_delegation_request_selectors());
		tester.test_view_modifier(PCall::get_delegator_total_staked_selectors());
		tester.test_view_modifier(PCall::get_candidate_total_counted_selectors());
	});
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile1, vec![1u8, 2u8, 3u8])
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile1, vec![1u8, 2u8, 3u8, 4u8])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn min_delegation_works() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile1, PCall::min_delegation {})
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(3u32)
	});
}

#[test]
fn points_zero() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			precompiles()
				// Assert that there are total 0 points in round 1
				.prepare_test(Alice, Precompile1, PCall::points { round: 1.into() })
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(0u32);
		});
}

#[test]
fn points_non_zero() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			set_points(1u32, Alice, 100);

			// Assert that there are total 100 points in round 1
			precompiles()
				.prepare_test(Alice, Precompile1, PCall::points { round: 1.into() })
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(100u32);
		});
}

#[test]
fn awarded_points_zero() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			set_points(1u32, Alice, 100);

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::awarded_points {
						round: 1u32.into(),
						candidate: Address(Bob.into()),
					},
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(0u32);
		});
}

#[test]
fn awarded_points_non_zero() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			set_points(1u32, Alice, 100);
			set_points(1u32, Bob, 10);

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::awarded_points {
						round: 1u32.into(),
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(100u32);
		});
}

#[test]
fn delegation_amount_zero() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::delegation_amount {
						delegator: Address(Alice.into()),
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(0u32);
		});
}

#[test]
fn delegation_amount_nonzero() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000), (Bob.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_delegations(vec![(Bob.into(), Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::delegation_amount {
						delegator: Address(Bob.into()),
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(1000u32);
		});
}

#[test]
fn is_not_in_top_delegations_when_delegation_dne() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::delegation_amount {
						delegator: Address(Alice.into()),
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(false);
		});
}

#[test]
fn is_not_in_top_delegations_because_not_in_top() {
	ExtBuilder::default()
		.with_balances(vec![
			(Alice.into(), 1_000),
			(Bob.into(), 500),
			(Charlie.into(), 501),
			(David.into(), 502),
		])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_delegations(vec![
			(Bob.into(), Alice.into(), 500),
			(Charlie.into(), Alice.into(), 501),
			(David.into(), Alice.into(), 502),
		])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::is_in_top_delegations {
						delegator: Address(Bob.into()),
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(false);
		});
}

#[test]
fn is_in_top_delegations() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000), (Bob.into(), 500)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_delegations(vec![(Bob.into(), Alice.into(), 500)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::is_in_top_delegations {
						delegator: Address(Bob.into()),
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(true);
		});
}

#[test]
fn round_works() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile1, PCall::round {})
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(1u32);

		// test next `ROUNDS_TO_TEST` rounds
		const ROUNDS_TO_TEST: u32 = 10;
		let mut current_round = 1;
		while current_round < ROUNDS_TO_TEST {
			current_round += 1;
			roll_to_round_begin(current_round);

			// Assert that round is equal to expectation
			precompiles()
				.prepare_test(Alice, Precompile1, PCall::round {})
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(current_round);
		}
	});
}

#[test]
fn candidate_delegation_count_works() {
	ExtBuilder::default()
		.with_balances(vec![
			(Alice.into(), 1_000),
			(Bob.into(), 50),
			(Charlie.into(), 50),
			(David.into(), 50),
		])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_delegations(vec![
			(Bob.into(), Alice.into(), 50),
			(Charlie.into(), Alice.into(), 50),
			(David.into(), Alice.into(), 50),
		])
		.build()
		.execute_with(|| {
			// Assert that there 3 delegations to Alice
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::candidate_delegation_count {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(3u32);
		});
}

#[test]
fn candidate_auto_compounding_delegation_count_works() {
	ExtBuilder::default()
		.with_balances(vec![
			(Alice.into(), 1_000),
			(Bob.into(), 50),
			(Charlie.into(), 50),
			(David.into(), 50),
		])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_delegations(vec![
			(Bob.into(), Alice.into(), 50),
			(Charlie.into(), Alice.into(), 50),
			(David.into(), Alice.into(), 50),
		])
		.with_auto_compounding_delegations(vec![(
			Charlie.into(),
			Alice.into(),
			50,
			Percent::from_percent(50),
		)])
		.build()
		.execute_with(|| {
			// Assert that there 1 auto compounding delegations to Alice
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::candidate_auto_compounding_delegation_count {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(1u32);
		});
}

#[test]
fn candidate_auto_compounding_elegation_count_works_with_zero() {
	ExtBuilder::default()
		.with_balances(vec![
			(Alice.into(), 1_000),
			(Bob.into(), 50),
			(Charlie.into(), 50),
			(David.into(), 50),
		])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_delegations(vec![
			(Bob.into(), Alice.into(), 50),
			(Charlie.into(), Alice.into(), 50),
			(David.into(), Alice.into(), 50),
		])
		.build()
		.execute_with(|| {
			// Assert that there 0 auto compounding delegations to Alice
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::candidate_auto_compounding_delegation_count {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(0u32);
		});
}

#[test]
fn delegator_delegation_count_works() {
	ExtBuilder::default()
		.with_balances(vec![
			(Alice.into(), 1_000),
			(Bob.into(), 1_000),
			(Charlie.into(), 200),
		])
		.with_candidates(vec![(Alice.into(), 1_000), (Bob.into(), 1_000)])
		.with_delegations(vec![
			(Charlie.into(), Alice.into(), 100),
			(Charlie.into(), Bob.into(), 100),
		])
		.build()
		.execute_with(|| {
			// Assert that Charlie has 2 outstanding nominations
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::delegator_delegation_count {
						delegator: Address(Charlie.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(2u32);
		});
}

#[test]
fn is_delegator_false() {
	ExtBuilder::default().build().execute_with(|| {
		// Assert that Charlie is not a delegator
		precompiles()
			.prepare_test(
				Alice,
				Precompile1,
				PCall::is_delegator {
					delegator: Address(Charlie.into()),
				},
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(false);
	});
}

#[test]
fn is_delegator_true() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000), (Bob.into(), 50)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_delegations(vec![(Bob.into(), Alice.into(), 50)])
		.build()
		.execute_with(|| {
			// Assert that Bob is a delegator
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::is_delegator {
						delegator: Address(Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(true);
		});
}

#[test]
fn is_candidate_false() {
	ExtBuilder::default().build().execute_with(|| {
		// Assert that Alice is not a candidate
		precompiles()
			.prepare_test(
				Alice,
				Precompile1,
				PCall::is_candidate {
					candidate: Address(Alice.into()),
				},
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(false);
	});
}

#[test]
fn is_candidate_true() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			// Assert that Alice is a candidate
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::is_candidate {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(true);
		});
}

#[test]
fn is_selected_candidate_false() {
	ExtBuilder::default().build().execute_with(|| {
		// Assert that Alice is not a selected candidate
		precompiles()
			.prepare_test(
				Alice,
				Precompile1,
				PCall::is_selected_candidate {
					candidate: Address(Alice.into()),
				},
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(false);
	});
}

#[test]
fn is_selected_candidate_true() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			// Assert that Alice is not a selected candidate
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::is_selected_candidate {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(true);
		});
}

#[test]
fn selected_candidates_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(Alice, Precompile1, PCall::selected_candidates {})
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(vec![Address(Alice.into())]);
		});
}

#[test]
fn delegation_request_is_pending_works() {
	ExtBuilder::default()
		.with_balances(vec![
			(Alice.into(), 1_000),
			(Charlie.into(), 50),
			(David.into(), 50),
		])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_delegations(vec![(Charlie.into(), Alice.into(), 50)])
		.build()
		.execute_with(|| {
			// Assert that we dont have pending requests
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::delegation_request_is_pending {
						delegator: Address(Charlie.into()),
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(false);

			// Schedule Revoke request
			precompiles()
				.prepare_test(
					Charlie,
					Precompile1,
					PCall::schedule_revoke_delegation {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(272791237)
				.expect_no_logs()
				.execute_returns(());

			// Assert that we have pending requests
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::delegation_request_is_pending {
						delegator: Address(Charlie.into()),
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(true);
		})
}

#[test]
fn delegation_request_is_pending_returns_false_for_non_existing_delegator() {
	ExtBuilder::default().build().execute_with(|| {
		// Expected false because delegator Bob does not exist
		precompiles()
			.prepare_test(
				Alice,
				Precompile1,
				PCall::delegation_request_is_pending {
					delegator: Address(Bob.into()),
					candidate: Address(Alice.into()),
				},
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(false);
	})
}

#[test]
fn candidate_exit_is_pending_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			// Assert that we don't have pending requests
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::candidate_exit_is_pending {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(false);

			// Schedule exit request
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::schedule_leave_candidates {
						candidate_count: 1.into(),
					},
				)
				.expect_cost(264694393)
				.expect_no_logs()
				.execute_returns(());

			// Assert that we have pending exit
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::candidate_exit_is_pending {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(true);
		})
}

#[test]
fn candidate_exit_is_pending_returns_false_for_non_existing_delegator() {
	ExtBuilder::default().build().execute_with(|| {
		// Expected false because candidate Bob does not exist
		precompiles()
			.prepare_test(
				Alice,
				Precompile1,
				PCall::candidate_exit_is_pending {
					candidate: Address(Bob.into()),
				},
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(false);
	})
}

#[test]
fn candidate_request_is_pending_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_050)])
		.with_candidates(vec![(Alice.into(), 1_050)])
		.build()
		.execute_with(|| {
			// Assert that we dont have pending requests
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::candidate_request_is_pending {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(false);

			// Schedule bond less request
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::schedule_candidate_bond_less { less: 0.into() },
				)
				.expect_cost(136000000)
				.expect_no_logs()
				.execute_returns(());

			// Assert that we have pending requests
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::candidate_request_is_pending {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(true);
		})
}

#[test]
fn candidate_request_is_pending_returns_false_for_non_existing_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		// Expected false because candidate Bob does not exist
		precompiles()
			.prepare_test(
				Alice,
				Precompile1,
				PCall::candidate_request_is_pending {
					candidate: Address(Bob.into()),
				},
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(false);
	})
}

#[test]
fn delegation_auto_compound_returns_value_if_set() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000), (Charlie.into(), 50)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_auto_compounding_delegations(vec![(
			Charlie.into(),
			Alice.into(),
			50,
			Percent::from_percent(50),
		)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::delegation_auto_compound {
						delegator: Address(Charlie.into()),
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(50u8);
		})
}

#[test]
fn delegation_auto_compound_returns_zero_if_not_set() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000), (Charlie.into(), 50)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_delegations(vec![(Charlie.into(), Alice.into(), 50)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::delegation_auto_compound {
						delegator: Address(Charlie.into()),
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(0u8);
		})
}

#[test]
fn join_candidates_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			let input_data = PCall::join_candidates {
				amount: 1000.into(),
				candidate_count: 0.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(
				RuntimeCall::Evm(evm_call(Alice, input_data)).dispatch(RuntimeOrigin::root())
			);

			let expected: crate::mock::RuntimeEvent = StakingEvent::JoinedCollatorCandidates {
				account: Alice.into(),
				amount_locked: 1000,
				new_total_amt_locked: 1000,
			}
			.into();
			// Assert that the events vector contains the one expected
			println!("{:?}", events());
			assert!(events().contains(&expected));
		});
}

#[test]
fn schedule_leave_candidates_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			let input_data = PCall::schedule_leave_candidates {
				candidate_count: 1.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(
				RuntimeCall::Evm(evm_call(Alice, input_data)).dispatch(RuntimeOrigin::root())
			);

			let expected: crate::mock::RuntimeEvent = StakingEvent::CandidateScheduledExit {
				exit_allowed_round: 1,
				candidate: Alice.into(),
				scheduled_exit: 3,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn execute_leave_candidates_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				RuntimeOrigin::signed(Alice.into()),
				1
			));
			roll_to(10);

			let input_data = PCall::execute_leave_candidates {
				candidate: Address(Alice.into()),
				candidate_count: 0.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(
				RuntimeCall::Evm(evm_call(Alice, input_data)).dispatch(RuntimeOrigin::root())
			);

			let expected: crate::mock::RuntimeEvent = StakingEvent::CandidateLeft {
				ex_candidate: Alice.into(),
				unlocked_amount: 1_000,
				new_total_amt_locked: 0,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn cancel_leave_candidates_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				RuntimeOrigin::signed(Alice.into()),
				1
			));

			let input_data = PCall::cancel_leave_candidates {
				candidate_count: 0.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(
				RuntimeCall::Evm(evm_call(Alice, input_data)).dispatch(RuntimeOrigin::root())
			);

			let expected: crate::mock::RuntimeEvent = StakingEvent::CancelledCandidateExit {
				candidate: Alice.into(),
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn go_online_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::go_offline(RuntimeOrigin::signed(
				Alice.into()
			)));

			let input_data = PCall::go_online {}.into();

			// Make sure the call goes through successfully
			assert_ok!(
				RuntimeCall::Evm(evm_call(Alice, input_data)).dispatch(RuntimeOrigin::root())
			);

			let expected: crate::mock::RuntimeEvent = StakingEvent::CandidateBackOnline {
				candidate: Alice.into(),
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn go_offline_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			let input_data = PCall::go_offline {}.into();
			// Make sure the call goes through successfully
			assert_ok!(
				RuntimeCall::Evm(evm_call(Alice, input_data)).dispatch(RuntimeOrigin::root())
			);

			let expected: crate::mock::RuntimeEvent = StakingEvent::CandidateWentOffline {
				candidate: Alice.into(),
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn candidate_bond_more_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_500)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			let input_data = PCall::candidate_bond_more { more: 500.into() }.into();

			// Make sure the call goes through successfully
			assert_ok!(
				RuntimeCall::Evm(evm_call(Alice, input_data)).dispatch(RuntimeOrigin::root())
			);

			let expected: crate::mock::RuntimeEvent = StakingEvent::CandidateBondedMore {
				candidate: Alice.into(),
				amount: 500,
				new_total_bond: 1500,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn schedule_candidate_bond_less_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			let input_data = PCall::schedule_candidate_bond_less { less: 500.into() }.into();

			// Make sure the call goes through successfully
			assert_ok!(
				RuntimeCall::Evm(evm_call(Alice, input_data)).dispatch(RuntimeOrigin::root())
			);

			let expected: crate::mock::RuntimeEvent = StakingEvent::CandidateBondLessRequested {
				candidate: Alice.into(),
				amount_to_decrease: 500,
				execute_round: 3,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn execute_candidate_bond_less_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_500)])
		.with_candidates(vec![(Alice.into(), 1_500)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				RuntimeOrigin::signed(Alice.into()),
				500
			));
			roll_to(10);

			// Make sure the call goes through successfully
			let input_data = PCall::execute_candidate_bond_less {
				candidate: Address(Alice.into()),
			}
			.into();

			assert_ok!(
				RuntimeCall::Evm(evm_call(Alice, input_data)).dispatch(RuntimeOrigin::root())
			);

			let expected: crate::mock::RuntimeEvent = StakingEvent::CandidateBondedLess {
				candidate: Alice.into(),
				amount: 500,
				new_bond: 1000,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn cancel_candidate_bond_less_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_200)])
		.with_candidates(vec![(Alice.into(), 1_200)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				RuntimeOrigin::signed(Alice.into()),
				200
			));

			let input_data = PCall::cancel_candidate_bond_less {}.into();

			// Make sure the call goes through successfully
			assert_ok!(
				RuntimeCall::Evm(evm_call(Alice, input_data)).dispatch(RuntimeOrigin::root())
			);

			let expected: crate::mock::RuntimeEvent = StakingEvent::CancelledCandidateBondLess {
				candidate: Alice.into(),
				amount: 200,
				execute_round: 3,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn delegate_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000), (Bob.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			let input_data = PCall::delegate {
				candidate: Address(Alice.into()),
				amount: 1_000.into(),
				candidate_delegation_count: 0.into(),
				delegator_delegation_count: 0.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(RuntimeCall::Evm(evm_call(Bob, input_data)).dispatch(RuntimeOrigin::root()));

			assert!(ParachainStaking::is_delegator(&Bob.into()));

			let expected: crate::mock::RuntimeEvent = StakingEvent::Delegation {
				delegator: Bob.into(),
				locked_amount: 1_000,
				candidate: Alice.into(),
				delegator_position: pallet_parachain_staking::DelegatorAdded::AddedToTop {
					new_total: 2_000,
				},
				auto_compound: Percent::zero(),
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn schedule_revoke_delegation_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000), (Bob.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_delegations(vec![(Bob.into(), Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			let input_data = PCall::schedule_revoke_delegation {
				candidate: Address(Alice.into()),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(RuntimeCall::Evm(evm_call(Bob, input_data)).dispatch(RuntimeOrigin::root()));

			let expected: crate::mock::RuntimeEvent = StakingEvent::DelegationRevocationScheduled {
				round: 1,
				delegator: Bob.into(),
				candidate: Alice.into(),
				scheduled_exit: 3,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn delegator_bond_more_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000), (Bob.into(), 1_500)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_delegations(vec![(Bob.into(), Alice.into(), 500)])
		.build()
		.execute_with(|| {
			let input_data = PCall::delegator_bond_more {
				candidate: Address(Alice.into()),
				more: 500.into(),
			}
			.into();

			assert_ok!(RuntimeCall::Evm(evm_call(Bob, input_data)).dispatch(RuntimeOrigin::root()));

			let expected: crate::mock::RuntimeEvent = StakingEvent::DelegationIncreased {
				delegator: Bob.into(),
				candidate: Alice.into(),
				amount: 500,
				in_top: true,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn schedule_delegator_bond_less_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000), (Bob.into(), 1_500)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_delegations(vec![(Bob.into(), Alice.into(), 1_500)])
		.build()
		.execute_with(|| {
			let input_data = PCall::schedule_delegator_bond_less {
				candidate: Address(Alice.into()),
				less: 500.into(),
			}
			.into();

			assert_ok!(RuntimeCall::Evm(evm_call(Bob, input_data)).dispatch(RuntimeOrigin::root()));

			// Check for the right events.
			let expected_event: crate::mock::RuntimeEvent =
				StakingEvent::DelegationDecreaseScheduled {
					delegator: Bob.into(),
					candidate: Alice.into(),
					amount_to_decrease: 500,
					execute_round: 3,
				}
				.into();

			assert!(events().contains(&expected_event));
		});
}

#[test]
fn execute_revoke_delegation_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000), (Bob.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_delegations(vec![(Bob.into(), Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(Bob.into()),
				Alice.into()
			));
			roll_to(10);

			let input_data = PCall::execute_delegation_request {
				delegator: Address(Bob.into()),
				candidate: Address(Alice.into()),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(
				RuntimeCall::Evm(evm_call(Alice, input_data)).dispatch(RuntimeOrigin::root())
			);

			let expected: crate::mock::RuntimeEvent = StakingEvent::DelegationRevoked {
				delegator: Bob.into(),
				candidate: Alice.into(),
				unstaked_amount: 1_000,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn execute_delegator_bond_less_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000), (Bob.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_delegations(vec![(Bob.into(), Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				RuntimeOrigin::signed(Bob.into()),
				Alice.into(),
				500
			));
			roll_to(10);

			let input_data = PCall::execute_delegation_request {
				delegator: Address(Bob.into()),
				candidate: Address(Alice.into()),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(
				RuntimeCall::Evm(evm_call(Alice, input_data)).dispatch(RuntimeOrigin::root())
			);

			let expected: crate::mock::RuntimeEvent = StakingEvent::DelegationDecreased {
				delegator: Bob.into(),
				candidate: Alice.into(),
				amount: 500,
				in_top: true,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn cancel_revoke_delegation_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000), (Bob.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_delegations(vec![(Bob.into(), Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				RuntimeOrigin::signed(Bob.into()),
				Alice.into()
			));

			let input_data = PCall::cancel_delegation_request {
				candidate: Address(Alice.into()),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(RuntimeCall::Evm(evm_call(Bob, input_data)).dispatch(RuntimeOrigin::root()));

			let expected: crate::mock::RuntimeEvent = StakingEvent::CancelledDelegationRequest {
				delegator: Bob.into(),
				collator: Alice.into(),
				cancelled_request: pallet_parachain_staking::CancelledScheduledRequest {
					when_executable: 3,
					action: pallet_parachain_staking::DelegationAction::Revoke(1_000),
				},
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn cancel_delegator_bonded_less_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1_000), (Bob.into(), 1_000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.with_delegations(vec![(Bob.into(), Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				RuntimeOrigin::signed(Bob.into()),
				Alice.into(),
				500
			));

			let input_data = PCall::cancel_delegation_request {
				candidate: Address(Alice.into()),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(RuntimeCall::Evm(evm_call(Bob, input_data)).dispatch(RuntimeOrigin::root()));

			let expected: crate::mock::RuntimeEvent = StakingEvent::CancelledDelegationRequest {
				delegator: Bob.into(),
				collator: Alice.into(),
				cancelled_request: pallet_parachain_staking::CancelledScheduledRequest {
					when_executable: 3,
					action: pallet_parachain_staking::DelegationAction::Decrease(500),
				},
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn delegate_with_auto_compound_works() {
	for auto_compound_percent in [0, 50, 100] {
		ExtBuilder::default()
			.with_balances(vec![(Alice.into(), 1_000), (Bob.into(), 1_000)])
			.with_candidates(vec![(Alice.into(), 1_000)])
			.build()
			.execute_with(|| {
				let input_data = PCall::delegate_with_auto_compound {
					candidate: Address(Alice.into()),
					amount: 1_000.into(),
					auto_compound: auto_compound_percent,
					candidate_delegation_count: 0.into(),
					candidate_auto_compounding_delegation_count: 0.into(),
					delegator_delegation_count: 0.into(),
				}
				.into();

				// Make sure the call goes through successfully
				assert_ok!(
					RuntimeCall::Evm(evm_call(Bob, input_data)).dispatch(RuntimeOrigin::root())
				);

				assert!(ParachainStaking::is_delegator(&Bob.into()));

				let expected: crate::mock::RuntimeEvent = StakingEvent::Delegation {
					delegator: Bob.into(),
					locked_amount: 1_000,
					candidate: Alice.into(),
					delegator_position: pallet_parachain_staking::DelegatorAdded::AddedToTop {
						new_total: 2_000,
					},
					auto_compound: Percent::from_percent(auto_compound_percent),
				}
				.into();
				// Assert that the events vector contains the one expected
				assert!(events().contains(&expected));
			});
	}
}

#[test]
fn delegate_with_auto_compound_returns_error_if_percent_above_hundred() {
	for auto_compound_percent in [101, 255] {
		ExtBuilder::default()
			.with_balances(vec![(Alice.into(), 1_000), (Bob.into(), 1_000)])
			.with_candidates(vec![(Alice.into(), 1_000)])
			.build()
			.execute_with(|| {
				PrecompilesValue::get()
					.prepare_test(
						Bob,
						Precompile1,
						PCall::delegate_with_auto_compound {
							candidate: Address(Alice.into()),
							amount: 1_000.into(),
							auto_compound: auto_compound_percent,
							candidate_delegation_count: 0.into(),
							candidate_auto_compounding_delegation_count: 0.into(),
							delegator_delegation_count: 0.into(),
						},
					)
					.execute_reverts(|output| {
						from_utf8(&output).unwrap().contains(
							"auto_compound: Must be an integer between 0 and 100 included",
						)
					});
			});
	}
}

#[test]
fn set_auto_compound_works_if_delegation() {
	for auto_compound_percent in [0, 50, 100] {
		ExtBuilder::default()
			.with_balances(vec![(Alice.into(), 1_000), (Bob.into(), 1_000)])
			.with_candidates(vec![(Alice.into(), 1_000)])
			.with_delegations(vec![(Bob.into(), Alice.into(), 1_000)])
			.build()
			.execute_with(|| {
				let input_data = PCall::set_auto_compound {
					candidate: Address(Alice.into()),
					value: auto_compound_percent,
					candidate_auto_compounding_delegation_count: 0.into(),
					delegator_delegation_count: 1.into(),
				}
				.into();

				// Make sure the call goes through successfully
				assert_ok!(
					RuntimeCall::Evm(evm_call(Bob, input_data)).dispatch(RuntimeOrigin::root())
				);

				assert_eq!(
					ParachainStaking::delegation_auto_compound(&Alice.into(), &Bob.into()),
					Percent::from_percent(auto_compound_percent)
				);

				let expected: crate::mock::RuntimeEvent = StakingEvent::AutoCompoundSet {
					candidate: Alice.into(),
					delegator: Bob.into(),
					value: Percent::from_percent(auto_compound_percent),
				}
				.into();
				// Assert that the events vector contains the one expected
				assert!(events().contains(&expected));
			});
	}
}

#[test]
fn set_auto_compound_returns_error_if_value_above_hundred_percent() {
	for auto_compound_percent in [101, 255] {
		ExtBuilder::default()
			.with_balances(vec![(Alice.into(), 1_000), (Bob.into(), 1_000)])
			.with_candidates(vec![(Alice.into(), 1_000)])
			.with_delegations(vec![(Bob.into(), Alice.into(), 1_000)])
			.build()
			.execute_with(|| {
				PrecompilesValue::get()
					.prepare_test(
						Bob,
						Precompile1,
						PCall::set_auto_compound {
							candidate: Address(Alice.into()),
							value: auto_compound_percent,
							candidate_auto_compounding_delegation_count: 0.into(),
							delegator_delegation_count: 1.into(),
						},
					)
					.execute_reverts(|output| {
						from_utf8(&output)
							.unwrap()
							.contains("value: Must be an integer between 0 and 100 included")
					});
			});
	}
}

#[test]
fn set_auto_compound_fails_if_not_delegation() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000), (Bob.into(), 1000)])
		.with_candidates(vec![(Alice.into(), 1_000)])
		.build()
		.execute_with(|| {
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::set_auto_compound {
						candidate: Address(Alice.into()),
						value: 50,
						candidate_auto_compounding_delegation_count: 0.into(),
						delegator_delegation_count: 0.into(),
					},
				)
				.execute_reverts(|output| from_utf8(&output).unwrap().contains("DelegatorDNE"));
		});
}

#[test]
fn get_delegator_total_staked_getter() {
	ExtBuilder::default()
		.with_balances(vec![
			(Alice.into(), 1_000),
			(Bob.into(), 1_000),
			(Charlie.into(), 1_500),
		])
		.with_candidates(vec![(Alice.into(), 1_000), (Bob.into(), 1_000)])
		.with_delegations(vec![
			(Charlie.into(), Alice.into(), 1_000),
			(Charlie.into(), Bob.into(), 499),
		])
		.build()
		.execute_with(|| {
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::get_delegator_total_staked {
						delegator: Address(Charlie.into()),
					},
				)
				.execute_returns(U256::from(1_499));
		});
}

#[test]
fn get_delegator_total_staked_getter_unknown() {
	ExtBuilder::default()
		.with_balances(vec![
			(Alice.into(), 1_000),
			(Bob.into(), 1_000),
			(Charlie.into(), 1_500),
		])
		.with_candidates(vec![(Alice.into(), 1_000), (Bob.into(), 1_000)])
		.build()
		.execute_with(|| {
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::get_delegator_total_staked {
						delegator: Address(Charlie.into()),
					},
				)
				.execute_returns(U256::zero());
		});
}

#[test]
fn get_candidate_total_counted_getter() {
	ExtBuilder::default()
		.with_balances(vec![
			(Alice.into(), 1_000),
			(Bob.into(), 1_000),
			(Charlie.into(), 1_500),
		])
		.with_candidates(vec![(Alice.into(), 1_000), (Bob.into(), 1_000)])
		.with_delegations(vec![
			(Charlie.into(), Alice.into(), 1_000),
			(Charlie.into(), Bob.into(), 499),
		])
		.build()
		.execute_with(|| {
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::get_candidate_total_counted {
						candidate: Address(Alice.into()),
					},
				)
				.execute_returns(U256::from(2_000));
		});
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	check_precompile_implements_solidity_interfaces(
		&["StakingInterface.sol"],
		PCall::supports_selector,
	)
}

#[test]
fn test_deprecated_solidity_selectors_are_supported() {
	for deprecated_function in [
		"min_delegation()",
		"candidate_count()",
		"candidate_delegation_count(address)",
		"delegator_delegation_count(address)",
		"selected_candidates()",
		"is_delegator(address)",
		"is_candidate(address)",
		"is_selected_candidate(address)",
		"delegation_request_is_pending(address,address)",
		"candidate_exit_is_pending(address)",
		"candidate_request_is_pending(address)",
		"join_candidates(uint256,uint256)",
		"schedule_leave_candidates(uint256)",
		"execute_leave_candidates(address,uint256)",
		"cancel_leave_candidates(uint256)",
		"go_offline()",
		"go_online()",
		"schedule_candidate_bond_less(uint256)",
		"candidate_bond_more(uint256)",
		"execute_candidate_bond_less(address)",
		"cancel_candidate_bond_less()",
		"schedule_revoke_delegation(address)",
		"schedule_delegator_bond_less(address,uint256)",
		"delegator_bond_more(address,uint256)",
		"execute_delegation_request(address,address)",
		"cancel_delegation_request(address)",
	] {
		let selector = compute_selector(deprecated_function);
		if !PCall::supports_selector(selector) {
			panic!(
				"failed decoding selector 0x{:x} => '{}' as Action",
				selector, deprecated_function,
			)
		}
	}
}
