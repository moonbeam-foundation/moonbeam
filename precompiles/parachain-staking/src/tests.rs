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
	Call, ExtBuilder, Origin, ParachainStaking, PrecompilesValue, Runtime, TestPrecompiles,
};
use crate::Action;
use frame_support::{assert_ok, dispatch::Dispatchable};
use pallet_evm::Call as EvmCall;
use pallet_parachain_staking::Event as StakingEvent;
use precompile_utils::{prelude::*, testing::*};
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
	assert_eq!(Action::IsDelegator as u32, 0x1f030587);
	assert_eq!(Action::IsCandidate as u32, 0x8545c833);
	assert_eq!(Action::IsSelectedCandidate as u32, 0x8f6d27c7);
	assert_eq!(Action::Points as u32, 0x9799b4e7);
	assert_eq!(Action::MinDelegation as u32, 0x72ce8933);
	assert_eq!(Action::CandidateCount as u32, 0x4b1c4c29);
	assert_eq!(Action::Round as u32, 0x146ca531);
	assert_eq!(Action::CandidateDelegationCount as u32, 0x815b796c);
	assert_eq!(Action::DelegatorDelegationCount as u32, 0xfbc51bca);
	assert_eq!(Action::SelectedCandidates as u32, 0x89f47a21);
	assert_eq!(Action::DelegationRequestIsPending as u32, 0x192e1db3);
	assert_eq!(Action::CandidateExitIsPending as u32, 0xeb613b8a);
	assert_eq!(Action::CandidateRequestIsPending as u32, 0x26ab05fb);
	assert_eq!(Action::JoinCandidates as u32, 0x0a1bff60);
	assert_eq!(Action::ScheduleLeaveCandidates as u32, 0x60afbac6);
	assert_eq!(Action::ExecuteLeaveCandidates as u32, 0x3fdc4c30);
	assert_eq!(Action::CancelLeaveCandidates as u32, 0x0880b3e2);
	assert_eq!(Action::GoOffline as u32, 0x767e0450);
	assert_eq!(Action::GoOnline as u32, 0xd2f73ceb);
	assert_eq!(Action::CandidateBondMore as u32, 0xc57bd3a8);
	assert_eq!(Action::ScheduleCandidateBondLess as u32, 0x034c47bc);
	assert_eq!(Action::ExecuteCandidateBondLess as u32, 0xa9a2b8b7);
	assert_eq!(Action::CancelCandidateBondLess as u32, 0x583d0fdc);
	assert_eq!(Action::Delegate as u32, 0x829f5ee3);
	assert_eq!(Action::ScheduleLeaveDelegators as u32, 0x65a5bbd0);
	assert_eq!(Action::ExecuteLeaveDelegators as u32, 0xa84a7468);
	assert_eq!(Action::CancelLeaveDelegators as u32, 0x2a987643);
	assert_eq!(Action::ScheduleRevokeDelegation as u32, 0x22266e75);
	assert_eq!(Action::ExecuteLeaveDelegators as u32, 0xa84a7468);
	assert_eq!(Action::CancelLeaveDelegators as u32, 0x2a987643);
	assert_eq!(Action::ScheduleRevokeDelegation as u32, 0x22266e75);
	assert_eq!(Action::DelegatorBondMore as u32, 0xf8331108);
	assert_eq!(Action::ScheduleDelegatorBondLess as u32, 0x00043acf);
	assert_eq!(Action::ExecuteDelegationRequest as u32, 0xe42366a6);
	assert_eq!(Action::CancelDelegationRequest as u32, 0x7284cf50);
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile, vec![1u8, 2u8, 3u8])
			.execute_reverts(|output| output == b"tried to parse selector out of bounds");
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile, vec![1u8, 2u8, 3u8, 4u8])
			.execute_reverts(|output| output == b"unknown selector");
	});
}

#[test]
fn min_delegation_works() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				EvmDataWriter::new_with_selector(Action::MinDelegation).build(),
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(EvmDataWriter::new().write(3u32).build())
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
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::Points)
						.write(U256::one())
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(0u32).build());
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
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::Points)
						.write(U256::one())
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(100u32).build());
		});
}

#[test]
fn round_works() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Round).build(),
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(EvmDataWriter::new().write(1u32).build());

		// test next `ROUNDS_TO_TEST` rounds
		const ROUNDS_TO_TEST: u64 = 10;
		let mut current_round: u64 = 1;
		while current_round < ROUNDS_TO_TEST {
			current_round += 1;
			roll_to_round_begin(current_round);

			// Assert that round is equal to expectation
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::Round).build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(current_round).build());
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
					EvmDataWriter::new_with_selector(Action::CandidateDelegationCount)
						.write(Address(Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(3u32).build());
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
					EvmDataWriter::new_with_selector(Action::DelegatorDelegationCount)
						.write(Address(Charlie.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(2u32).build());
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
				EvmDataWriter::new_with_selector(Action::IsDelegator)
					.write(Address(Charlie.into()))
					.build(),
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(EvmDataWriter::new().write(false).build());
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
					EvmDataWriter::new_with_selector(Action::IsDelegator)
						.write(Address(Bob.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());
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
				EvmDataWriter::new_with_selector(Action::IsCandidate)
					.write(Address(Alice.into()))
					.build(),
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(EvmDataWriter::new().write(false).build());
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
					EvmDataWriter::new_with_selector(Action::IsCandidate)
						.write(Address(Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());
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
				EvmDataWriter::new_with_selector(Action::IsSelectedCandidate)
					.write(Address(Alice.into()))
					.build(),
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(EvmDataWriter::new().write(false).build());
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
					EvmDataWriter::new_with_selector(Action::IsSelectedCandidate)
						.write(Address(Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());
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
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::SelectedCandidates).build(),
				)
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
					EvmDataWriter::new_with_selector(Action::DelegationRequestIsPending)
						.write(Address(Charlie.into()))
						.write(Address(Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(false).build());

			// Schedule Revoke request
			precompiles()
				.prepare_test(
					Charlie,
					Precompile,
					EvmDataWriter::new_with_selector(Action::ScheduleRevokeDelegation)
						.write(Address(Alice.into()))
						.build(),
				)
				.expect_cost(281793000)
				.expect_no_logs()
				.execute_returns(vec![]);

			// Assert that we have pending requests
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::DelegationRequestIsPending)
						.write(Address(Charlie.into()))
						.write(Address(Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());
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
				EvmDataWriter::new_with_selector(Action::DelegationRequestIsPending)
					.write(Address(Bob.into()))
					.write(Address(Alice.into()))
					.build(),
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(EvmDataWriter::new().write(false).build());
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
					EvmDataWriter::new_with_selector(Action::CandidateExitIsPending)
						.write(Address(Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(false).build());

			// Schedule exit request
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::ScheduleLeaveCandidates)
						.write(U256::one())
						.build(),
				)
				.expect_cost(303417000)
				.expect_no_logs()
				.execute_returns(vec![]);

			// Assert that we have pending exit
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::CandidateExitIsPending)
						.write(Address(Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());
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
				EvmDataWriter::new_with_selector(Action::CandidateExitIsPending)
					.write(Address(Bob.into()))
					.build(),
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(EvmDataWriter::new().write(false).build());
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
					EvmDataWriter::new_with_selector(Action::CandidateRequestIsPending)
						.write(Address(Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(false).build());

			// Schedule bond less request
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::ScheduleCandidateBondLess)
						.write(U256::zero())
						.build(),
				)
				.expect_cost(151710000)
				.expect_no_logs()
				.execute_returns(vec![]);

			// Assert that we have pending requests
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::CandidateRequestIsPending)
						.write(Address(Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());
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
				EvmDataWriter::new_with_selector(Action::CandidateRequestIsPending)
					.write(Address(Bob.into()))
					.build(),
			)
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns(EvmDataWriter::new().write(false).build());
	})
}

#[test]
fn join_candidates_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000)])
		.build()
		.execute_with(|| {
			let input_data = EvmDataWriter::new_with_selector(Action::JoinCandidates)
				.write(U256::from(1000u32))
				.write(U256::zero())
				.build();

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
			let input_data = EvmDataWriter::new_with_selector(Action::ScheduleLeaveCandidates)
				.write(U256::one())
				.build();

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

			let input_data = EvmDataWriter::new_with_selector(Action::ExecuteLeaveCandidates)
				.write(Address(Alice.into()))
				.write(U256::zero())
				.build();

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

			let input_data = EvmDataWriter::new_with_selector(Action::CancelLeaveCandidates)
				.write(U256::zero())
				.build();

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

			let input_data = EvmDataWriter::new_with_selector(Action::GoOnline).build();

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
			let input_data = EvmDataWriter::new_with_selector(Action::GoOffline).build();
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
			let input_data = EvmDataWriter::new_with_selector(Action::CandidateBondMore)
				.write(U256::from(500))
				.build();

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
			let input_data = EvmDataWriter::new_with_selector(Action::ScheduleCandidateBondLess)
				.write(U256::from(500))
				.build();

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
			let input_data = EvmDataWriter::new_with_selector(Action::ExecuteCandidateBondLess)
				.write(Address(Alice.into()))
				.build();

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

			let input_data =
				EvmDataWriter::new_with_selector(Action::CancelCandidateBondLess).build();

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
			let input_data = EvmDataWriter::new_with_selector(Action::Delegate)
				.write(Address(Alice.into()))
				.write(U256::from(1_000))
				.write(U256::zero())
				.write(U256::zero())
				.build();

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
			let input_data =
				EvmDataWriter::new_with_selector(Action::ScheduleLeaveDelegators).build();

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

			let input_data = EvmDataWriter::new_with_selector(Action::ExecuteLeaveDelegators)
				.write(Address(Bob.into()))
				.write(U256::one())
				.build();

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

			let input_data =
				EvmDataWriter::new_with_selector(Action::CancelLeaveDelegators).build();

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
			let input_data = EvmDataWriter::new_with_selector(Action::ScheduleRevokeDelegation)
				.write(Address(Alice.into()))
				.build();

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
			let input_data = EvmDataWriter::new_with_selector(Action::DelegatorBondMore)
				.write(Address(Alice.into()))
				.write(U256::from(500))
				.build();

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
			let input_data = EvmDataWriter::new_with_selector(Action::ScheduleDelegatorBondLess)
				.write(Address(Alice.into()))
				.write(U256::from(500))
				.build();

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

			let input_data = EvmDataWriter::new_with_selector(Action::ExecuteDelegationRequest)
				.write(Address(Bob.into()))
				.write(Address(Alice.into()))
				.build();

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

			let input_data = EvmDataWriter::new_with_selector(Action::ExecuteDelegationRequest)
				.write(Address(Bob.into()))
				.write(Address(Alice.into()))
				.build();

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

			let input_data = EvmDataWriter::new_with_selector(Action::CancelDelegationRequest)
				.write(Address(Alice.into()))
				.build();

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

			let input_data = EvmDataWriter::new_with_selector(Action::CancelDelegationRequest)
				.write(Address(Alice.into()))
				.build();

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
