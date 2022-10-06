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

use crate::mock::{
	events, roll_to, roll_to_round_begin, set_points,
	Account::{self, Alice, Bob, Bogus, Charlie, Precompile},
	Call, ExtBuilder, Origin, PCall, ParachainStaking, PrecompilesValue, Runtime, TestPrecompiles,
};
use frame_support::{assert_ok, dispatch::Dispatchable};
use pallet_evm::Call as EvmCall;
use pallet_parachain_staking::Event as StakingEvent;
use precompile_utils::{prelude::*, solidity, testing::*};
use sp_core::U256;

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

fn evm_call(source: Account, input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: source.into(),
		target: Precompile.into(),
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
	assert!(PCall::points_selectors().contains(&0x9799b4e7));
	assert!(PCall::min_delegation_selectors().contains(&0x02985992));
	assert!(PCall::candidate_count_selectors().contains(&0xa9a981a3));
	assert!(PCall::round_selectors().contains(&0x146ca531));
	assert!(PCall::candidate_delegation_count_selectors().contains(&0x2ec087eb));
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
	assert!(PCall::schedule_leave_delegators_selectors().contains(&0xf939dadb));
	assert!(PCall::execute_leave_delegators_selectors().contains(&0xfb1e2bf9));
	assert!(PCall::cancel_leave_delegators_selectors().contains(&0xf7421284));
	assert!(PCall::schedule_revoke_delegation_selectors().contains(&0x1a1c740c));
	assert!(PCall::delegator_bond_more_selectors().contains(&0x0465135b));
	assert!(PCall::schedule_delegator_bond_less_selectors().contains(&0xc172fd2b));
	assert!(PCall::execute_delegation_request_selectors().contains(&0xe98c8abe));
	assert!(PCall::cancel_delegation_request_selectors().contains(&0xc90eee83));
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile, vec![1u8, 2u8, 3u8])
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile, vec![1u8, 2u8, 3u8, 4u8])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn min_delegation_works() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile, PCall::min_delegation {})
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns_encoded(3u32)
	});
}

#[test]
fn points_zero() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.build()
		.execute_with(|| {
			precompiles()
				// Assert that there are total 0 points in round 1
				.prepare_test(Alice, Precompile, PCall::points { round: 1.into() })
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(0u32);
		});
}

#[test]
fn points_non_zero() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.build()
		.execute_with(|| {
			set_points(1u32, Alice, 100);

			// Assert that there are total 100 points in round 1
			precompiles()
				.prepare_test(Alice, Precompile, PCall::points { round: 1.into() })
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(100u32);
		});
}

#[test]
fn round_works() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile, PCall::round {})
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns_encoded(1u32);

		// test next `ROUNDS_TO_TEST` rounds
		const ROUNDS_TO_TEST: u32 = 10;
		let mut current_round = 1;
		while current_round < ROUNDS_TO_TEST {
			current_round += 1;
			roll_to_round_begin(current_round);

			// Assert that round is equal to expectation
			precompiles()
				.prepare_test(Alice, Precompile, PCall::round {})
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(current_round);
		}
	});
}

#[test]
fn candidate_delegation_count_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000), (Bob, 50), (Charlie, 50), (Bogus, 50)])
		.with_candidates(vec![(Alice, 1_000)])
		.with_delegations(vec![
			(Bob, Alice, 50),
			(Charlie, Alice, 50),
			(Bogus, Alice, 50),
		])
		.build()
		.execute_with(|| {
			// Assert that there 3 delegations to Alice
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::candidate_delegation_count {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(3u32);
		});
}

#[test]
fn delegator_delegation_count_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000), (Bob, 1_000), (Charlie, 200)])
		.with_candidates(vec![(Alice, 1_000), (Bob, 1_000)])
		.with_delegations(vec![(Charlie, Alice, 100), (Charlie, Bob, 100)])
		.build()
		.execute_with(|| {
			// Assert that Charlie has 2 outstanding nominations
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::delegator_delegation_count {
						delegator: Address(Charlie.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(2u32);
		});
}

#[test]
fn is_delegator_false() {
	ExtBuilder::default().build().execute_with(|| {
		// Assert that Charlie is not a delegator
		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				PCall::is_delegator {
					delegator: Address(Charlie.into()),
				},
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns_encoded(false);
	});
}

#[test]
fn is_delegator_true() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000), (Bob, 50)])
		.with_candidates(vec![(Alice, 1_000)])
		.with_delegations(vec![(Bob, Alice, 50)])
		.build()
		.execute_with(|| {
			// Assert that Bob is a delegator
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::is_delegator {
						delegator: Address(Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(true);
		});
}

#[test]
fn is_candidate_false() {
	ExtBuilder::default().build().execute_with(|| {
		// Assert that Alice is not a candidate
		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				PCall::is_candidate {
					candidate: Address(Alice.into()),
				},
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns_encoded(false);
	});
}

#[test]
fn is_candidate_true() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.build()
		.execute_with(|| {
			// Assert that Alice is a candidate
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::is_candidate {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(true);
		});
}

#[test]
fn is_selected_candidate_false() {
	ExtBuilder::default().build().execute_with(|| {
		// Assert that Alice is not a selected candidate
		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				PCall::is_selected_candidate {
					candidate: Address(Alice.into()),
				},
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns_encoded(false);
	});
}

#[test]
fn is_selected_candidate_true() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.build()
		.execute_with(|| {
			// Assert that Alice is not a selected candidate
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::is_selected_candidate {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(true);
		});
}

#[test]
fn selected_candidates_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(Alice, Precompile, PCall::selected_candidates {})
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(vec![Address(Alice.into())])
						.build(),
				);
		});
}

#[test]
fn delegation_request_is_pending_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000), (Charlie, 50), (Bogus, 50)])
		.with_candidates(vec![(Alice, 1_000)])
		.with_delegations(vec![(Charlie, Alice, 50)])
		.build()
		.execute_with(|| {
			// Assert that we dont have pending requests
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::delegation_request_is_pending {
						delegator: Address(Charlie.into()),
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(false);

			// Schedule Revoke request
			precompiles()
				.prepare_test(
					Charlie,
					Precompile,
					PCall::schedule_revoke_delegation {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(290930000)
				.expect_no_logs()
				.execute_returns(vec![]);

			// Assert that we have pending requests
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::delegation_request_is_pending {
						delegator: Address(Charlie.into()),
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(true);
		})
}

#[test]
fn delegation_request_is_pending_returns_false_for_non_existing_delegator() {
	ExtBuilder::default().build().execute_with(|| {
		// Expected false because delegator Bob does not exist
		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				PCall::delegation_request_is_pending {
					delegator: Address(Bob.into()),
					candidate: Address(Alice.into()),
				},
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns_encoded(false);
	})
}

#[test]
fn candidate_exit_is_pending_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.build()
		.execute_with(|| {
			// Assert that we don't have pending requests
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::candidate_exit_is_pending {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(false);

			// Schedule exit request
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::schedule_leave_candidates {
						candidate_count: 1.into(),
					},
				)
				.expect_cost(323429000)
				.expect_no_logs()
				.execute_returns(vec![]);

			// Assert that we have pending exit
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::candidate_exit_is_pending {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(true);
		})
}

#[test]
fn candidate_exit_is_pending_returns_false_for_non_existing_delegator() {
	ExtBuilder::default().build().execute_with(|| {
		// Expected false because candidate Bob does not exist
		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				PCall::candidate_exit_is_pending {
					candidate: Address(Bob.into()),
				},
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns_encoded(false);
	})
}

#[test]
fn candidate_request_is_pending_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_050)])
		.with_candidates(vec![(Alice, 1_050)])
		.build()
		.execute_with(|| {
			// Assert that we dont have pending requests
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::candidate_request_is_pending {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(false);

			// Schedule bond less request
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::schedule_candidate_bond_less { less: 0.into() },
				)
				.expect_cost(161834000)
				.expect_no_logs()
				.execute_returns(vec![]);

			// Assert that we have pending requests
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::candidate_request_is_pending {
						candidate: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(true);
		})
}

#[test]
fn candidate_request_is_pending_returns_false_for_non_existing_candidate() {
	ExtBuilder::default().build().execute_with(|| {
		// Expected false because candidate Bob does not exist
		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				PCall::candidate_request_is_pending {
					candidate: Address(Bob.into()),
				},
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns_encoded(false);
	})
}

#[test]
fn join_candidates_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000)])
		.build()
		.execute_with(|| {
			let input_data = PCall::join_candidates {
				amount: 1000.into(),
				candidate_count: 0.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::JoinedCollatorCandidates {
				account: Alice,
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
		.with_balances(vec![(Alice, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.build()
		.execute_with(|| {
			let input_data = PCall::schedule_leave_candidates {
				candidate_count: 1.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::CandidateScheduledExit {
				exit_allowed_round: 1,
				candidate: Alice,
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
		.with_balances(vec![(Alice, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(Alice),
				1
			));
			roll_to(10);

			let input_data = PCall::execute_leave_candidates {
				candidate: Address(Alice.into()),
				candidate_count: 0.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::CandidateLeft {
				ex_candidate: Alice,
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
		.with_balances(vec![(Alice, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(Alice),
				1
			));

			let input_data = PCall::cancel_leave_candidates {
				candidate_count: 0.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CancelledCandidateExit { candidate: Alice }.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn go_online_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::go_offline(Origin::signed(Alice)));

			let input_data = PCall::go_online {}.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CandidateBackOnline { candidate: Alice }.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn go_offline_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.build()
		.execute_with(|| {
			let input_data = PCall::go_offline {}.into();
			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CandidateWentOffline { candidate: Alice }.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn candidate_bond_more_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_500)])
		.with_candidates(vec![(Alice, 1_000)])
		.build()
		.execute_with(|| {
			let input_data = PCall::candidate_bond_more { more: 500.into() }.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::CandidateBondedMore {
				candidate: Alice,
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
		.with_balances(vec![(Alice, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.build()
		.execute_with(|| {
			let input_data = PCall::schedule_candidate_bond_less { less: 500.into() }.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::CandidateBondLessRequested {
				candidate: Alice,
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
		.with_balances(vec![(Alice, 1_500)])
		.with_candidates(vec![(Alice, 1_500)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				Origin::signed(Alice),
				500
			));
			roll_to(10);

			// Make sure the call goes through successfully
			let input_data = PCall::execute_candidate_bond_less {
				candidate: Address(Alice.into()),
			}
			.into();

			assert_ok!(Call::Evm(evm_call(Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::CandidateBondedLess {
				candidate: Alice,
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
		.with_balances(vec![(Alice, 1_200)])
		.with_candidates(vec![(Alice, 1_200)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				Origin::signed(Alice),
				200
			));

			let input_data = PCall::cancel_candidate_bond_less {}.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::CancelledCandidateBondLess {
				candidate: Alice,
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
		.with_balances(vec![(Alice, 1_000), (Bob, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
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
			assert_ok!(Call::Evm(evm_call(Bob, input_data)).dispatch(Origin::root()));

			assert!(ParachainStaking::is_delegator(&Bob));

			let expected: crate::mock::Event = StakingEvent::Delegation {
				delegator: Bob,
				locked_amount: 1_000,
				candidate: Alice,
				delegator_position: pallet_parachain_staking::DelegatorAdded::AddedToTop {
					new_total: 2_000,
				},
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn schedule_leave_delegators_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000), (Bob, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.with_delegations(vec![(Bob, Alice, 1_000)])
		.build()
		.execute_with(|| {
			let input_data = PCall::schedule_leave_delegators {}.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Bob, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::DelegatorExitScheduled {
				round: 1,
				delegator: Bob,
				scheduled_exit: 3,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn execute_leave_delegators_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000), (Bob, 500)])
		.with_candidates(vec![(Alice, 1_000)])
		.with_delegations(vec![(Bob, Alice, 500)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				Bob
			)));
			roll_to(10);

			let input_data = PCall::execute_leave_delegators {
				delegator: Address(Bob.into()),
				delegator_delegation_count: 1.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::DelegatorLeft {
				delegator: Bob,
				unstaked_amount: 500,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn cancel_leave_delegators_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000), (Bob, 500)])
		.with_candidates(vec![(Alice, 1_000)])
		.with_delegations(vec![(Bob, Alice, 500)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				Bob
			)));

			let input_data = PCall::cancel_leave_delegators {}.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Bob, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::DelegatorExitCancelled { delegator: Bob }.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn schedule_revoke_delegation_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000), (Bob, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.with_delegations(vec![(Bob, Alice, 1_000)])
		.build()
		.execute_with(|| {
			let input_data = PCall::schedule_revoke_delegation {
				candidate: Address(Alice.into()),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Bob, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::DelegationRevocationScheduled {
				round: 1,
				delegator: Bob,
				candidate: Alice,
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
		.with_balances(vec![(Alice, 1_000), (Bob, 1_500)])
		.with_candidates(vec![(Alice, 1_000)])
		.with_delegations(vec![(Bob, Alice, 500)])
		.build()
		.execute_with(|| {
			let input_data = PCall::delegator_bond_more {
				candidate: Address(Alice.into()),
				more: 500.into(),
			}
			.into();

			assert_ok!(Call::Evm(evm_call(Bob, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::DelegationIncreased {
				delegator: Bob,
				candidate: Alice,
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
		.with_balances(vec![(Alice, 1_000), (Bob, 1_500)])
		.with_candidates(vec![(Alice, 1_000)])
		.with_delegations(vec![(Bob, Alice, 1_500)])
		.build()
		.execute_with(|| {
			let input_data = PCall::schedule_delegator_bond_less {
				candidate: Address(Alice.into()),
				less: 500.into(),
			}
			.into();

			assert_ok!(Call::Evm(evm_call(Bob, input_data)).dispatch(Origin::root()));

			// Check for the right events.
			let expected_event: crate::mock::Event = StakingEvent::DelegationDecreaseScheduled {
				delegator: Bob,
				candidate: Alice,
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
		.with_balances(vec![(Alice, 1_000), (Bob, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.with_delegations(vec![(Bob, Alice, 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(Bob),
				Alice
			));
			roll_to(10);

			let input_data = PCall::execute_delegation_request {
				delegator: Address(Bob.into()),
				candidate: Address(Alice.into()),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::DelegationRevoked {
				delegator: Bob,
				candidate: Alice,
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
		.with_balances(vec![(Alice, 1_000), (Bob, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.with_delegations(vec![(Bob, Alice, 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(Bob),
				Alice,
				500
			));
			roll_to(10);

			let input_data = PCall::execute_delegation_request {
				delegator: Address(Bob.into()),
				candidate: Address(Alice.into()),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::DelegationDecreased {
				delegator: Bob,
				candidate: Alice,
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
		.with_balances(vec![(Alice, 1_000), (Bob, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.with_delegations(vec![(Bob, Alice, 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(Bob),
				Alice
			));

			let input_data = PCall::cancel_delegation_request {
				candidate: Address(Alice.into()),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Bob, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::CancelledDelegationRequest {
				delegator: Bob,
				collator: Alice,
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
		.with_balances(vec![(Alice, 1_000), (Bob, 1_000)])
		.with_candidates(vec![(Alice, 1_000)])
		.with_delegations(vec![(Bob, Alice, 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(Bob),
				Alice,
				500
			));

			let input_data = PCall::cancel_delegation_request {
				candidate: Address(Alice.into()),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(Bob, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::CancelledDelegationRequest {
				delegator: Bob,
				collator: Alice,
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
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	for file in ["StakingInterface.sol"] {
		for solidity_fn in solidity::get_selectors(file) {
			assert_eq!(
				solidity_fn.compute_selector_hex(),
				solidity_fn.docs_selector,
				"documented selector for '{}' did not match for file '{}'",
				solidity_fn.signature(),
				file,
			);

			let selector = solidity_fn.compute_selector();
			if !PCall::supports_selector(selector) {
				panic!(
					"failed decoding selector 0x{:x} => '{}' as Action for file '{}'",
					selector,
					solidity_fn.signature(),
					file,
				)
			}
		}
	}
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
		"schedule_leave_delegators()",
		"execute_leave_delegators(address,uint256)",
		"cancel_leave_delegators()",
		"schedule_revoke_delegation(address)",
		"schedule_delegator_bond_less(address,uint256)",
		"delegator_bond_more(address,uint256)",
		"execute_delegation_request(address,address)",
		"cancel_delegation_request(address)",
	] {
		let selector = solidity::compute_selector(deprecated_function);
		if !PCall::supports_selector(selector) {
			panic!(
				"failed decoding selector 0x{:x} => '{}' as Action",
				selector, deprecated_function,
			)
		}
	}
}
