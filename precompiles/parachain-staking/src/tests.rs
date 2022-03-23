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
	events, evm_test_context, precompile_address, roll_to, set_points, Call, ExtBuilder, Origin,
	ParachainStaking, PrecompilesValue, Runtime, TestAccount, TestPrecompiles,
};
use crate::{Action, PrecompileOutput};
use fp_evm::PrecompileFailure;
use frame_support::{assert_ok, dispatch::Dispatchable};
use pallet_evm::{Call as EvmCall, ExitSucceed, PrecompileSet};
use parachain_staking::Event as StakingEvent;
use precompile_utils::EvmDataWriter;
use sha3::{Digest, Keccak256};
use sp_core::U256;
use std::assert_matches::assert_matches;

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

fn evm_call(source: TestAccount, input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: source.to_h160(),
		target: precompile_address(),
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
	// DEPRECATED
	assert_eq!(Action::IsNominator as u32, 0x8e5080e7);
	assert_eq!(Action::IsDelegator as u32, 0x1f030587);
	assert_eq!(Action::IsCandidate as u32, 0x8545c833);
	assert_eq!(Action::IsSelectedCandidate as u32, 0x8f6d27c7);
	assert_eq!(Action::Points as u32, 0x9799b4e7);
	// DEPRECATED
	assert_eq!(Action::MinNomination as u32, 0xc9f593b2);
	assert_eq!(Action::MinDelegation as u32, 0x72ce8933);
	assert_eq!(Action::CandidateCount as u32, 0x4b1c4c29);
	assert_eq!(Action::CollatorNominationCount as u32, 0x0ad6a7be);
	assert_eq!(Action::CandidateDelegationCount as u32, 0x815b796c);
	assert_eq!(Action::NominatorNominationCount as u32, 0xdae5659b);
	assert_eq!(Action::DelegatorDelegationCount as u32, 0xfbc51bca);
	assert_eq!(Action::SelectedCandidates as u32, 0x89f47a21);
	assert_eq!(Action::JoinCandidates as u32, 0x0a1bff60);
	// DEPRECATED
	assert_eq!(Action::LeaveCandidates as u32, 0x72b02a31);
	assert_eq!(Action::ScheduleLeaveCandidates as u32, 0x60afbac6);
	assert_eq!(Action::ExecuteLeaveCandidates as u32, 0x3fdc4c30);
	assert_eq!(Action::CancelLeaveCandidates as u32, 0x0880b3e2);
	assert_eq!(Action::GoOffline as u32, 0x767e0450);
	assert_eq!(Action::GoOnline as u32, 0xd2f73ceb);
	assert_eq!(Action::CandidateBondMore as u32, 0xc57bd3a8);
	// DEPRECATED
	assert_eq!(Action::CandidateBondLess as u32, 0x289b6ba7);
	assert_eq!(Action::ScheduleCandidateBondLess as u32, 0x034c47bc);
	assert_eq!(Action::ExecuteCandidateBondLess as u32, 0xa9a2b8b7);
	assert_eq!(Action::CancelCandidateBondLess as u32, 0x583d0fdc);
	// DEPRECATED
	assert_eq!(Action::Nominate as u32, 0x49df6eb3);
	assert_eq!(Action::Delegate as u32, 0x829f5ee3);
	// DEPRECATED
	assert_eq!(Action::LeaveNominators as u32, 0xb71d2153);
	assert_eq!(Action::ScheduleLeaveDelegators as u32, 0x65a5bbd0);
	assert_eq!(Action::ExecuteLeaveDelegators as u32, 0xa84a7468);
	assert_eq!(Action::CancelLeaveDelegators as u32, 0x2a987643);
	// DEPRECATED
	assert_eq!(Action::RevokeNomination as u32, 0x4b65c34b);
	assert_eq!(Action::ScheduleRevokeDelegation as u32, 0x22266e75);
	assert_eq!(Action::ExecuteLeaveDelegators as u32, 0xa84a7468);
	assert_eq!(Action::CancelLeaveDelegators as u32, 0x2a987643);
	// DEPRECATED
	assert_eq!(Action::RevokeNomination as u32, 0x4b65c34b);
	assert_eq!(Action::ScheduleRevokeDelegation as u32, 0x22266e75);
	// DEPRECATED
	assert_eq!(Action::NominatorBondMore as u32, 0x971d44c8);
	assert_eq!(Action::DelegatorBondMore as u32, 0xf8331108);
	// DEPRECATED
	assert_eq!(Action::NominatorBondLess as u32, 0xf6a52569);
	assert_eq!(Action::ScheduleDelegatorBondLess as u32, 0x00043acf);
	assert_eq!(Action::ExecuteDelegationRequest as u32, 0xe42366a6);
	assert_eq!(Action::CancelDelegationRequest as u32, 0x7284cf50);
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
		let bogus_selector = vec![1u8, 2u8, 3u8];

		assert_matches!(
			precompiles().execute(
				precompile_address(),
				&bogus_selector,
				None,
				&evm_test_context(),
				false,
			),
			Some(Err(PrecompileFailure::Revert { output, ..}))
			if &output == b"tried to parse selector out of bounds"
		);
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		let bogus_selector = vec![1u8, 2u8, 3u8, 4u8];

		assert_matches!(
			precompiles().execute(
				precompile_address(),
				&bogus_selector,
				None,
				&evm_test_context(),
				false,
			),
			Some(Err(PrecompileFailure::Revert { output, ..}))
			if &output == b"unknown selector"
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
			precompiles().execute(
				precompile_address(),
				&input_data,
				None,
				&evm_test_context(),
				false
			),
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
			precompiles().execute(
				precompile_address(),
				&input_data,
				None,
				&evm_test_context(),
				false
			),
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
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
				precompiles().execute(
					precompile_address(),
					&charlie_input_data,
					None,
					&evm_test_context(),
					false,
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
				precompiles().execute(
					precompile_address(),
					&bob_input_data,
					None,
					&evm_test_context(),
					false
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
			precompiles().execute(
				precompile_address(),
				&input_data,
				None,
				&evm_test_context(),
				false
			),
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
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
			precompiles().execute(
				precompile_address(),
				&input_data,
				None,
				&evm_test_context(),
				false
			),
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
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
			precompiles().execute(
				precompile_address(),
				&input_data,
				None,
				&evm_test_context(),
				false
			),
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
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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

			let expected: crate::mock::Event = StakingEvent::JoinedCollatorCandidates {
				account: TestAccount::Alice,
				amount_locked: 1000,
				new_total_amt_locked: 1000,
			}
			.into();
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

			let expected: crate::mock::Event = StakingEvent::CandidateScheduledExit {
				exit_allowed_round: 1,
				candidate: TestAccount::Alice,
				scheduled_exit: 3,
			}
			.into();
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

			let expected: crate::mock::Event = StakingEvent::CandidateScheduledExit {
				exit_allowed_round: 1,
				candidate: TestAccount::Alice,
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
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			assert_ok!(ParachainStaking::schedule_leave_candidates(
				Origin::signed(TestAccount::Alice),
				1
			));
			roll_to(10);
			let selector = &Keccak256::digest(b"execute_leave_candidates(address,uint256)")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 68]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);
			let candidate_delegation_count = U256::zero();
			candidate_delegation_count.to_big_endian(&mut input_data[36..]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::CandidateLeft {
				ex_candidate: TestAccount::Alice,
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

			let expected: crate::mock::Event = StakingEvent::CancelledCandidateExit {
				candidate: TestAccount::Alice,
			}
			.into();
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

			let expected: crate::mock::Event = StakingEvent::CandidateBackOnline {
				candidate: TestAccount::Alice,
			}
			.into();
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

			let expected: crate::mock::Event = StakingEvent::CandidateWentOffline {
				candidate: TestAccount::Alice,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

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

			let expected: crate::mock::Event = StakingEvent::CandidateBondedMore {
				candidate: TestAccount::Alice,
				amount: 500,
				new_total_bond: 1500,
			}
			.into();
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

			let expected: crate::mock::Event = StakingEvent::CandidateBondLessRequested {
				candidate: TestAccount::Alice,
				amount_to_decrease: 500,
				execute_round: 3,
			}
			.into();
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

			let expected: crate::mock::Event = StakingEvent::CandidateBondLessRequested {
				candidate: TestAccount::Alice,
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
		.with_balances(vec![(TestAccount::Alice, 1_500)])
		.with_candidates(vec![(TestAccount::Alice, 1_500)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"execute_candidate_bond_less(address)")[0..4];

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

			let expected: crate::mock::Event = StakingEvent::CandidateBondedLess {
				candidate: TestAccount::Alice,
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
		.with_balances(vec![(TestAccount::Alice, 1_200)])
		.with_candidates(vec![(TestAccount::Alice, 1_200)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"cancel_candidate_bond_less()")[0..4];

			// Construct data
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);

			assert_ok!(ParachainStaking::schedule_candidate_bond_less(
				Origin::signed(TestAccount::Alice),
				200
			));

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(TestAccount::Alice, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::CancelledCandidateBondLess {
				candidate: TestAccount::Alice,
				amount: 200,
				execute_round: 3,
			}
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

			let expected: crate::mock::Event = StakingEvent::Delegation {
				delegator: TestAccount::Bob,
				locked_amount: 1_000,
				candidate: TestAccount::Alice,
				delegator_position: parachain_staking::DelegatorAdded::AddedToTop {
					new_total: 2_000,
				},
			}
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

			let expected: crate::mock::Event = StakingEvent::Delegation {
				delegator: TestAccount::Bob,
				locked_amount: 1_000,
				candidate: TestAccount::Alice,
				delegator_position: parachain_staking::DelegatorAdded::AddedToTop {
					new_total: 2_000,
				},
			}
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

			let expected: crate::mock::Event = StakingEvent::DelegatorExitScheduled {
				round: 1,
				delegator: TestAccount::Bob,
				scheduled_exit: 3,
			}
			.into();
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

			let expected: crate::mock::Event = StakingEvent::DelegatorExitScheduled {
				round: 1,
				delegator: TestAccount::Bob,
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

			let expected: crate::mock::Event = StakingEvent::DelegatorLeft {
				delegator: TestAccount::Bob,
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

			let expected: crate::mock::Event = StakingEvent::DelegatorExitCancelled {
				delegator: TestAccount::Bob,
			}
			.into();
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

			let expected: crate::mock::Event = StakingEvent::DelegationRevocationScheduled {
				round: 1,
				delegator: TestAccount::Bob,
				candidate: TestAccount::Alice,
				scheduled_exit: 3,
			}
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

			let expected: crate::mock::Event = StakingEvent::DelegationRevocationScheduled {
				round: 1,
				delegator: TestAccount::Bob,
				candidate: TestAccount::Alice,
				scheduled_exit: 3,
			}
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

			let expected: crate::mock::Event = StakingEvent::DelegationIncreased {
				delegator: TestAccount::Bob,
				candidate: TestAccount::Alice,
				amount: 500,
				in_top: true,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn delegator_bond_more_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_500)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_delegations(vec![(TestAccount::Bob, TestAccount::Alice, 500)])
		.build()
		.execute_with(|| {
			// Construct the delegator_bond_more call
			let mut input_data = Vec::<u8>::from([0u8; 68]);
			input_data[0..4]
				.copy_from_slice(&Keccak256::digest(b"delegator_bond_more(address,uint256)")[0..4]);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);
			let bond_more_amount: U256 = 500.into();
			bond_more_amount.to_big_endian(&mut input_data[36..68]);

			assert_ok!(Call::Evm(evm_call(TestAccount::Bob, input_data)).dispatch(Origin::root()));

			let expected: crate::mock::Event = StakingEvent::DelegationIncreased {
				delegator: TestAccount::Bob,
				candidate: TestAccount::Alice,
				amount: 500,
				in_top: true,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
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
			let expected_event: crate::mock::Event = StakingEvent::DelegationDecreaseScheduled {
				delegator: TestAccount::Bob,
				candidate: TestAccount::Alice,
				amount_to_decrease: 500,
				execute_round: 3,
			}
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
			let expected_event: crate::mock::Event = StakingEvent::DelegationDecreaseScheduled {
				delegator: TestAccount::Bob,
				candidate: TestAccount::Alice,
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

			let expected: crate::mock::Event = StakingEvent::DelegationRevoked {
				delegator: TestAccount::Bob,
				candidate: TestAccount::Alice,
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

			let expected: crate::mock::Event = StakingEvent::DelegationDecreased {
				delegator: TestAccount::Bob,
				candidate: TestAccount::Alice,
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

			let expected: crate::mock::Event = StakingEvent::CancelledDelegationRequest {
				delegator: TestAccount::Bob,
				cancelled_request: parachain_staking::DelegationRequest {
					collator: TestAccount::Alice,
					amount: 1_000,
					when_executable: 3,
					action: parachain_staking::DelegationChange::Revoke,
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

			let expected: crate::mock::Event = StakingEvent::CancelledDelegationRequest {
				delegator: TestAccount::Bob,
				cancelled_request: parachain_staking::DelegationRequest {
					collator: TestAccount::Alice,
					amount: 500,
					when_executable: 3,
					action: parachain_staking::DelegationChange::Decrease,
				},
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}
