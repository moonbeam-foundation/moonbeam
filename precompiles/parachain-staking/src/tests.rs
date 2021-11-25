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

use crate::mock::{
	events, evm_test_context, precompile_address, roll_to, set_points, Call, ExtBuilder, Origin,
	ParachainStaking, Precompiles, Runtime, TestAccount,
};
use crate::PrecompileOutput;
use frame_support::{assert_ok, dispatch::Dispatchable};
use pallet_evm::Call as EvmCall;
use pallet_evm::{ExitSucceed, PrecompileSet};
use parachain_staking::Event as StakingEvent;
use precompile_utils::{error, EvmDataWriter};
use sha3::{Digest, Keccak256};
use sp_core::U256;

fn evm_call(source: TestAccount, input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: source.to_h160(),
		target: precompile_address(),
		input,
		value: U256::zero(), // No value sent in EVM
		gas_limit: u64::max_value(),
		gas_price: 0.into(),
		nonce: None, // Use the next nonce
	}
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
		let bogus_selector = vec![1u8, 2u8, 3u8];

		// Expected result is an error stating there are too few bytes
		let expected_result = Some(Err(error("tried to parse selector out of bounds")));

		assert_eq!(
			Precompiles::execute(
				precompile_address(),
				&bogus_selector,
				None,
				&evm_test_context(),
			),
			expected_result
		);
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		let bogus_selector = vec![1u8, 2u8, 3u8, 4u8];

		// Expected result is an error stating there are such a selector does not exist
		let expected_result = Some(Err(error("unknown selector")));

		assert_eq!(
			Precompiles::execute(
				precompile_address(),
				&bogus_selector,
				None,
				&evm_test_context(),
			),
			expected_result
		);
	});
}

// DEPRECATED
#[test]
fn min_nomination_works() {
	ExtBuilder::default().build().execute_with(|| {
		let selector = &Keccak256::digest(b"min_nomination()")[0..4];

		// Construct data to read minimum nomination constant
		let mut input_data = Vec::<u8>::from([0u8; 4]);
		input_data[0..4].copy_from_slice(&selector);

		// Expected result is 3
		let expected_result = Some(Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: EvmDataWriter::new().write(3u32).build(),
			cost: Default::default(),
			logs: Default::default(),
		}));

		assert_eq!(
			Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
			expected_result
		);
	});
}

#[test]
fn min_delegation_works() {
	ExtBuilder::default().build().execute_with(|| {
		let selector = &Keccak256::digest(b"min_delegation()")[0..4];

		// Construct data to read minimum nomination constant
		let mut input_data = Vec::<u8>::from([0u8; 4]);
		input_data[0..4].copy_from_slice(&selector);

		// Expected result is 3
		let expected_result = Some(Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: EvmDataWriter::new().write(3u32).build(),
			cost: Default::default(),
			logs: Default::default(),
		}));

		assert_eq!(
			Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
			expected_result
		);
	});
}

#[test]
fn points_zero() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"points(uint256)")[0..4];

			// Construct data to read points for round 1
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			U256::one().to_big_endian(&mut input_data[4..36]);

			// Expected result is 0 points
			let expected_one_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(0u32).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that there are total 0 points in round 1
			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_one_result
			);
		});
}

#[test]
fn points_non_zero() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"points(uint256)")[0..4];

			// Construct data to read points for round 1
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			U256::one().to_big_endian(&mut input_data[4..36]);

			// Expected result is 100 points
			let expected_one_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(100u32).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			set_points(1u32, TestAccount::Alice, 100);

			// Assert that there are total 100 points in round 1
			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_one_result
			);
		});
}

// DEPRECATED
#[test]
fn collator_nomination_count_works() {
	ExtBuilder::default()
		.with_balances(vec![
			(TestAccount::Alice, 1_000),
			(TestAccount::Bob, 50),
			(TestAccount::Charlie, 50),
			(TestAccount::Bogus, 50),
		])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![
			(TestAccount::Bob, TestAccount::Alice, 50),
			(TestAccount::Charlie, TestAccount::Alice, 50),
			(TestAccount::Bogus, TestAccount::Alice, 50),
		])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"collator_nomination_count(address)")[0..4];

			// Construct data to read collator nomination count
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);

			// Expected result 3
			let expected_one_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(3u32).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that there 3 nominations for Alice
			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_one_result
			);
		});
}

#[test]
fn candidate_delegation_count_works() {
	ExtBuilder::default()
		.with_balances(vec![
			(TestAccount::Alice, 1_000),
			(TestAccount::Bob, 50),
			(TestAccount::Charlie, 50),
			(TestAccount::Bogus, 50),
		])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![
			(TestAccount::Bob, TestAccount::Alice, 50),
			(TestAccount::Charlie, TestAccount::Alice, 50),
			(TestAccount::Bogus, TestAccount::Alice, 50),
		])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"candidate_delegation_count(address)")[0..4];

			// Construct data to read candidate delegation count
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);

			// Expected result 3
			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(3u32).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that there 3 delegations to Alice
			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_result
			);
		});
}

// DEPRECATED
#[test]
fn nominator_nomination_count_works() {
	ExtBuilder::default()
		.with_balances(vec![
			(TestAccount::Alice, 1_000),
			(TestAccount::Bob, 1_000),
			(TestAccount::Charlie, 200),
		])
		.with_candidates(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_delegations(vec![
			(TestAccount::Charlie, TestAccount::Alice, 100),
			(TestAccount::Charlie, TestAccount::Bob, 100),
		])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"nominator_nomination_count(address)")[0..4];

			// Construct data to read delegator delegation count
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Charlie.to_h160().0);

			// Expected result is 2
			let expected_one_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(2u32).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that Charlie has 2 outstanding delegations
			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_one_result
			);
		});
}

#[test]
fn delegator_delegation_count_works() {
	ExtBuilder::default()
		.with_balances(vec![
			(TestAccount::Alice, 1_000),
			(TestAccount::Bob, 1_000),
			(TestAccount::Charlie, 200),
		])
		.with_candidates(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_delegations(vec![
			(TestAccount::Charlie, TestAccount::Alice, 100),
			(TestAccount::Charlie, TestAccount::Bob, 100),
		])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"delegator_delegation_count(address)")[0..4];

			// Construct data to read delegator delegation count
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Charlie.to_h160().0);

			// Expected result is 2
			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(2u32).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that Charlie has 2 outstanding nominations
			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_result
			);
		});
}

// DEPRECATED
#[test]
fn is_nominator_true_false() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 50)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 50)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"is_nominator(address)")[0..4];

			// Construct data to read is_nominator for Charlie
			let mut charlie_input_data = Vec::<u8>::from([0u8; 36]);
			charlie_input_data[0..4].copy_from_slice(&selector);
			charlie_input_data[16..36].copy_from_slice(&TestAccount::Charlie.to_h160().0);

			// Expected result is false
			let expected_false_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(false).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that Charlie is not a delegator
			assert_eq!(
				Precompiles::execute(
					precompile_address(),
					&charlie_input_data,
					None,
					&evm_test_context()
				),
				expected_false_result
			);

			// Construct data to read is_nominator for Bob
			let mut bob_input_data = Vec::<u8>::from([0u8; 36]);
			bob_input_data[0..4].copy_from_slice(&selector);
			bob_input_data[16..36].copy_from_slice(&TestAccount::Bob.to_h160().0);

			// Expected result is true
			let expected_true_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(true).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that Bob is a delegator
			assert_eq!(
				Precompiles::execute(
					precompile_address(),
					&bob_input_data,
					None,
					&evm_test_context()
				),
				expected_true_result
			);
		});
}

#[test]
fn is_delegator_false() {
	ExtBuilder::default().build().execute_with(|| {
		let selector = &Keccak256::digest(b"is_delegator(address)")[0..4];

		// Construct data to read is_delegator
		let mut input_data = Vec::<u8>::from([0u8; 36]);
		input_data[0..4].copy_from_slice(&selector);
		input_data[16..36].copy_from_slice(&TestAccount::Charlie.to_h160().0);

		// Expected result is false
		let expected_one_result = Some(Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: EvmDataWriter::new().write(false).build(),
			cost: Default::default(),
			logs: Default::default(),
		}));

		// Assert that Charlie is not a delegator
		assert_eq!(
			Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
			expected_one_result
		);
	});
}

#[test]
fn is_delegator_true() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 50)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 50)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"is_delegator(address)")[0..4];

			// Construct data to read is_delegator
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Bob.to_h160().0);

			// Expected result is true
			let expected_one_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(true).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that Bob is a delegator
			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_one_result
			);
		});
}

#[test]
fn is_candidate_false() {
	ExtBuilder::default().build().execute_with(|| {
		let selector = &Keccak256::digest(b"is_candidate(address)")[0..4];

		// Construct data to read is_candidate
		let mut input_data = Vec::<u8>::from([0u8; 36]);
		input_data[0..4].copy_from_slice(&selector);
		input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);

		// Expected result is false
		let expected_one_result = Some(Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: EvmDataWriter::new().write(false).build(),
			cost: Default::default(),
			logs: Default::default(),
		}));

		// Assert that Alice is not a candidate
		assert_eq!(
			Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
			expected_one_result
		);
	});
}

#[test]
fn is_candidate_true() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"is_candidate(address)")[0..4];

			// Construct data to read is_candidate
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);

			// Expected result is true
			let expected_one_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(true).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that Alice is a candidate
			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_one_result
			);
		});
}

#[test]
fn is_selected_candidate_false() {
	ExtBuilder::default().build().execute_with(|| {
		let selector = &Keccak256::digest(b"is_selected_candidate(address)")[0..4];

		// Construct data to read is_selected_candidate
		let mut input_data = Vec::<u8>::from([0u8; 36]);
		input_data[0..4].copy_from_slice(&selector);
		input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);

		// Expected result is false
		let expected_one_result = Some(Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: EvmDataWriter::new().write(false).build(),
			cost: Default::default(),
			logs: Default::default(),
		}));

		// Assert that Alice is not a selected candidate
		assert_eq!(
			Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
			expected_one_result
		);
	});
}

#[test]
fn is_selected_candidate_true() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"is_selected_candidate(address)")[0..4];

			// Construct data to read is_selected_candidate
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);

			// Expected result is true
			let expected_one_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(true).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that Alice is a selected candidate
			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_one_result
			);
		});
}

#[test]
fn join_candidates_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"join_candidates(uint256,uint256)")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 68]);
			input_data[0..4].copy_from_slice(&selector);
			let amount: U256 = 1000.into();
			amount.to_big_endian(&mut input_data[4..36]);
			let candidate_count = U256::zero();
			candidate_count.to_big_endian(&mut input_data[36..]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::JoinedCollatorCandidates(TestAccount::Alice, 1000, 1000).into();
			// Assert that the events vector contains the one expected
			println!("{:?}", events());
			assert!(events().contains(&expected));
		});
}

// DEPRECATED
#[test]
fn leave_candidates_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"leave_candidates(uint256)")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			let candidate_count = U256::one();
			candidate_count.to_big_endian(&mut input_data[4..]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CandidateScheduledExit(1, TestAccount::Alice, 3).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn schedule_leave_candidates_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"schedule_leave_candidates(uint256)")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			let candidate_count = U256::one();
			candidate_count.to_big_endian(&mut input_data[4..]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CandidateScheduledExit(1, TestAccount::Alice, 3).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn execute_leave_candidates_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(TestAccount::Alice),
				1
			));
			roll_to(10);
			let selector = &Keccak256::digest(b"execute_leave_candidates(address)")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CandidateLeft(TestAccount::Alice, 1_000, 0).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn cancel_leave_candidates_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(TestAccount::Alice),
				1
			));
			let selector = &Keccak256::digest(b"cancel_leave_candidates(uint256)")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			let candidate_count = U256::zero();
			candidate_count.to_big_endian(&mut input_data[4..]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CancelledCandidateExit(TestAccount::Alice).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn go_online_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::go_offline(Origin::signed(
				TestAccount::Alice
			)));
			let selector = &Keccak256::digest(b"go_online()")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 4]);
			input_data[0..4].copy_from_slice(&selector);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CandidateBackOnline(1, TestAccount::Alice).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn go_offline_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"go_offline()")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 4]);
			input_data[0..4].copy_from_slice(&selector);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CandidateWentOffline(1, TestAccount::Alice).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

// DEPRECATED
#[test]
fn candidate_bond_more_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_500)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"candidate_bond_more(uint256)")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			let bond_more_amount: U256 = 500.into();
			bond_more_amount.to_big_endian(&mut input_data[4..36]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			// scheduled event now
			let expected: crate::mock::Event =
				StakingEvent::CandidateBondMoreRequested(TestAccount::Alice, 500, 3).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn schedule_candidate_bond_more_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_500)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"schedule_candidate_bond_more(uint256)")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			let bond_more_amount: U256 = 500.into();
			bond_more_amount.to_big_endian(&mut input_data[4..36]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			// scheduled event now
			let expected: crate::mock::Event =
				StakingEvent::CandidateBondMoreRequested(TestAccount::Alice, 500, 3).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

// DEPRECATED
#[test]
fn candidate_bond_less_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"candidate_bond_less(uint256)")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			let bond_less_amount: U256 = 500.into();
			bond_less_amount.to_big_endian(&mut input_data[4..36]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CandidateBondLessRequested(TestAccount::Alice, 500, 3).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn schedule_candidate_bond_less_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"schedule_candidate_bond_less(uint256)")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			let bond_less_amount: U256 = 500.into();
			bond_less_amount.to_big_endian(&mut input_data[4..36]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CandidateBondLessRequested(TestAccount::Alice, 500, 3).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn execute_candidate_bond_more_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_500)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_candidate_bond_more(
				Origin::signed(TestAccount::Alice),
				500
			));
			roll_to(10);
			let selector = &Keccak256::digest(b"execute_candidate_bond_request(address)")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CandidateBondedMore(TestAccount::Alice, 500, 1500).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn execute_candidate_bond_less_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_500)])
		.with_candidates(vec![(TestAccount::Alice, 1_500)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"execute_candidate_bond_request(address)")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);

			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				Origin::signed(TestAccount::Alice),
				500
			));
			roll_to(10);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CandidateBondedLess(TestAccount::Alice, 500, 1000).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn cancel_candidate_bond_more_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_700)])
		.with_candidates(vec![(TestAccount::Alice, 1_200)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_candidate_bond_more(
				Origin::signed(TestAccount::Alice),
				500
			));
			let selector = &Keccak256::digest(b"cancel_candidate_bond_request()")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::CancelledCandidateBondChange(
				TestAccount::Alice,
				parachain_staking::CandidateBondRequest {
					amount: 500,
					change: parachain_staking::CandidateBondChange::Increase,
					when_executable: 3,
				},
			)
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn cancel_candidate_bond_less_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_200)])
		.with_candidates(vec![(TestAccount::Alice, 1_200)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"cancel_candidate_bond_request()")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);

			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				Origin::signed(TestAccount::Alice),
				200
			));

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::CancelledCandidateBondChange(
				TestAccount::Alice,
				parachain_staking::CandidateBondRequest {
					amount: 200,
					change: parachain_staking::CandidateBondChange::Decrease,
					when_executable: 3,
				},
			)
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

// DEPRECATED
#[test]
fn nominate_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"nominate(address,uint256,uint256,uint256)")[0..4];

			// Construct selector for nominate
			let mut input_data = Vec::<u8>::from([0u8; 132]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);
			let delegation_amount: U256 = 1_000.into();
			delegation_amount.to_big_endian(&mut input_data[36..68]);
			let collator_delegation_count = U256::zero();
			collator_delegation_count.to_big_endian(&mut input_data[68..100]);
			let delegation_count = U256::zero();
			delegation_count.to_big_endian(&mut input_data[100..]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Bob, input_data)).dispatch(Origin::root()));

			assert!(ParachainStaking::is_delegator(&TestAccount::Bob));

			let expected: crate::mock::Event = StakingEvent::Delegation(
				TestAccount::Bob,
				1_000,
				TestAccount::Alice,
				parachain_staking::DelegatorAdded::AddedToTop { new_total: 2_000 },
			)
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn delegate_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"delegate(address,uint256,uint256,uint256)")[0..4];

			// Construct selector for nominate
			let mut input_data = Vec::<u8>::from([0u8; 132]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);
			let delegation_amount: U256 = 1_000.into();
			delegation_amount.to_big_endian(&mut input_data[36..68]);
			let collator_delegation_count = U256::zero();
			collator_delegation_count.to_big_endian(&mut input_data[68..100]);
			let delegation_count = U256::zero();
			delegation_count.to_big_endian(&mut input_data[100..]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Bob, input_data)).dispatch(Origin::root()));

			assert!(ParachainStaking::is_delegator(&TestAccount::Bob));

			let expected: crate::mock::Event = StakingEvent::Delegation(
				TestAccount::Bob,
				1_000,
				TestAccount::Alice,
				parachain_staking::DelegatorAdded::AddedToTop { new_total: 2_000 },
			)
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

// DEPRECATED
#[test]
fn leave_nominators_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"leave_nominators(uint256)")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			let delegation_count = U256::one();
			delegation_count.to_big_endian(&mut input_data[4..]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Bob, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::DelegatorExitScheduled(1, TestAccount::Bob, 3).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn schedule_leave_delegators_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"schedule_leave_delegators()")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 4]);
			input_data[0..4].copy_from_slice(&selector);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Bob, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::DelegatorExitScheduled(1, TestAccount::Bob, 3).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn execute_leave_delegators_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 500)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 500)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				TestAccount::Bob
			)));
			roll_to(10);
			let selector = &Keccak256::digest(b"execute_leave_delegators(address,uint256)")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 68]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Bob.to_h160().0);
			let delegation_count = U256::one();
			delegation_count.to_big_endian(&mut input_data[36..]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::DelegatorLeft(TestAccount::Bob, 500).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn cancel_leave_delegators_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 500)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 500)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_delegators(Origin::signed(
				TestAccount::Bob
			)));
			let selector = &Keccak256::digest(b"cancel_leave_delegators()")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 4]);
			input_data[0..4].copy_from_slice(&selector);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Bob, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::DelegatorExitCancelled(TestAccount::Bob).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

// DEPRECATED
#[test]
fn revoke_nomination_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"revoke_nomination(address)")[0..4];

			// Construct selector for revoke_nomination
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Bob, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::DelegationRevocationScheduled(
				1,
				TestAccount::Bob,
				TestAccount::Alice,
				3,
			)
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn schedule_revoke_delegation_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"schedule_revoke_delegation(address)")[0..4];

			// Construct selector for schedule_revoke_delegation
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Bob, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::DelegationRevocationScheduled(
				1,
				TestAccount::Bob,
				TestAccount::Alice,
				3,
			)
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

// DEPRECATED
#[test]
fn nominator_bond_more_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_500)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 500)])
		.build()
		.execute_with(|| {
			// Construct the delegator_bond_more call
			let mut input_data = Vec::<u8>::from([0u8; 68]);
			input_data[0..4]
				.copy_from_slice(&Keccak256::digest(b"nominator_bond_more(address,uint256)")[0..4]);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);
			let bond_more_amount: U256 = 500.into();
			bond_more_amount.to_big_endian(&mut input_data[36..68]);

			assert_ok!(Call::Evm(evm_call(TestAccount::Bob, input_data)).dispatch(Origin::root()));

			// Check for the right events.
			let expected_event: crate::mock::Event = StakingEvent::DelegationIncreaseScheduled(
				TestAccount::Bob,
				TestAccount::Alice,
				500,
				3,
			)
			.into();

			assert!(events().contains(&expected_event));
		});
}

#[test]
fn schedule_delegator_bond_more_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_500)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 500)])
		.build()
		.execute_with(|| {
			// Construct the delegator_bond_more call
			let mut input_data = Vec::<u8>::from([0u8; 68]);
			input_data[0..4].copy_from_slice(
				&Keccak256::digest(b"schedule_delegator_bond_more(address,uint256)")[0..4],
			);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);
			let bond_more_amount: U256 = 500.into();
			bond_more_amount.to_big_endian(&mut input_data[36..68]);

			assert_ok!(Call::Evm(evm_call(TestAccount::Bob, input_data)).dispatch(Origin::root()));

			// Check for the right events.
			let expected_event: crate::mock::Event = StakingEvent::DelegationIncreaseScheduled(
				TestAccount::Bob,
				TestAccount::Alice,
				500,
				3,
			)
			.into();

			assert!(events().contains(&expected_event));
		});
}

// DEPRECATED
#[test]
fn nominator_bond_less_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_500)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 1_500)])
		.build()
		.execute_with(|| {
			// Construct the delegator_bond_less call
			let mut input_data = Vec::<u8>::from([0u8; 68]);
			input_data[0..4]
				.copy_from_slice(&Keccak256::digest(b"nominator_bond_less(address,uint256)")[0..4]);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);
			let bond_less_amount: U256 = 500.into();
			bond_less_amount.to_big_endian(&mut input_data[36..68]);

			assert_ok!(Call::Evm(evm_call(TestAccount::Bob, input_data)).dispatch(Origin::root()));

			// Check for the right events.
			let expected_event: crate::mock::Event = StakingEvent::DelegationDecreaseScheduled(
				TestAccount::Bob,
				TestAccount::Alice,
				500,
				3,
			)
			.into();

			assert!(events().contains(&expected_event));
		});
}

#[test]
fn schedule_delegator_bond_less_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_500)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 1_500)])
		.build()
		.execute_with(|| {
			// Construct the delegator_bond_less call
			let mut input_data = Vec::<u8>::from([0u8; 68]);
			input_data[0..4].copy_from_slice(
				&Keccak256::digest(b"schedule_delegator_bond_less(address,uint256)")[0..4],
			);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);
			let bond_less_amount: U256 = 500.into();
			bond_less_amount.to_big_endian(&mut input_data[36..68]);

			assert_ok!(Call::Evm(evm_call(TestAccount::Bob, input_data)).dispatch(Origin::root()));

			// Check for the right events.
			let expected_event: crate::mock::Event = StakingEvent::DelegationDecreaseScheduled(
				TestAccount::Bob,
				TestAccount::Alice,
				500,
				3,
			)
			.into();

			assert!(events().contains(&expected_event));
		});
}

#[test]
fn execute_revoke_delegation_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(TestAccount::Bob),
				TestAccount::Alice
			));
			roll_to(10);
			let selector = &Keccak256::digest(b"execute_delegation_request(address,address)")[0..4];

			// Construct selector
			let mut input_data = Vec::<u8>::from([0u8; 68]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Bob.to_h160().0);
			input_data[48..].copy_from_slice(&TestAccount::Alice.to_h160().0);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::DelegationRevoked(TestAccount::Bob, TestAccount::Alice, 1_000).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn execute_delegator_bond_more_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 500)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_delegator_bond_more(
				Origin::signed(TestAccount::Bob),
				TestAccount::Alice,
				500
			));
			roll_to(10);
			let selector = &Keccak256::digest(b"execute_delegation_request(address,address)")[0..4];

			// Construct selector
			let mut input_data = Vec::<u8>::from([0u8; 68]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Bob.to_h160().0);
			input_data[48..].copy_from_slice(&TestAccount::Alice.to_h160().0);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::DelegationIncreased(TestAccount::Bob, TestAccount::Alice, 500, true)
					.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn execute_delegator_bond_less_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(TestAccount::Bob),
				TestAccount::Alice,
				500
			));
			roll_to(10);
			let selector = &Keccak256::digest(b"execute_delegation_request(address,address)")[0..4];

			// Construct selector
			let mut input_data = Vec::<u8>::from([0u8; 68]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Bob.to_h160().0);
			input_data[48..].copy_from_slice(&TestAccount::Alice.to_h160().0);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::DelegationDecreased(TestAccount::Bob, TestAccount::Alice, 500, true)
					.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn cancel_revoke_delegation_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_revoke_delegation(
				Origin::signed(TestAccount::Bob),
				TestAccount::Alice
			));
			let selector = &Keccak256::digest(b"cancel_delegation_request(address)")[0..4];

			// Construct selector
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..].copy_from_slice(&TestAccount::Alice.to_h160().0);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Bob, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::CancelledDelegationRequest(
				TestAccount::Bob,
				parachain_staking::DelegationRequest {
					collator: TestAccount::Alice,
					amount: 1_000,
					when_executable: 3,
					action: parachain_staking::DelegationChange::Revoke,
				},
			)
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn cancel_delegator_bonded_more_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 500)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_delegator_bond_more(
				Origin::signed(TestAccount::Bob),
				TestAccount::Alice,
				500
			));
			let selector = &Keccak256::digest(b"cancel_delegation_request(address)")[0..4];

			// Construct selector
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..].copy_from_slice(&TestAccount::Alice.to_h160().0);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Bob, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::CancelledDelegationRequest(
				TestAccount::Bob,
				parachain_staking::DelegationRequest {
					collator: TestAccount::Alice,
					amount: 500,
					when_executable: 3,
					action: parachain_staking::DelegationChange::Increase,
				},
			)
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn cancel_delegator_bonded_less_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_delegator_bond_less(
				Origin::signed(TestAccount::Bob),
				TestAccount::Alice,
				500
			));
			let selector = &Keccak256::digest(b"cancel_delegation_request(address)")[0..4];

			// Construct selector
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..].copy_from_slice(&TestAccount::Alice.to_h160().0);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Bob, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::CancelledDelegationRequest(
				TestAccount::Bob,
				parachain_staking::DelegationRequest {
					collator: TestAccount::Alice,
					amount: 500,
					when_executable: 3,
					action: parachain_staking::DelegationChange::Decrease,
				},
			)
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}
