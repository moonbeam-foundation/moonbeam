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
	events, evm_test_context, precompile_address, set_points, Call, ExtBuilder, Origin,
	ParachainStaking, Precompiles, TestAccount,
};
use crate::PrecompileOutput;
use frame_support::{assert_ok, dispatch::Dispatchable};
use pallet_evm::Call as EvmCall;
use pallet_evm::{ExitError, ExitSucceed, PrecompileSet};
use parachain_staking::Event as StakingEvent;
use precompile_utils::OutputBuilder;
use sha3::{Digest, Keccak256};
use sp_core::U256;

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
		let bogus_selector = vec![1u8, 2u8, 3u8];

		// Expected result is an error stating there are too few bytes
		let expected_result = Some(Err(ExitError::Other(
			"input must at least contain a selector".into(),
		)));

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
		let expected_result = Some(Err(ExitError::Other(
			"No parachain-staking wrapper method at given selector".into(),
		)));

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
fn min_nomination_works() {
	ExtBuilder::default().build().execute_with(|| {
		let selector = &Keccak256::digest(b"min_nomination()")[0..4];

		// Construct data to read minimum nomination constant
		let mut input_data = Vec::<u8>::from([0u8; 4]);
		input_data[0..4].copy_from_slice(&selector);

		// Expected result is 3
		let expected_one_result = Some(Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: OutputBuilder::new().write_u256(3u32).build(),
			cost: Default::default(),
			logs: Default::default(),
		}));

		// Assert that no props have been opened.
		assert_eq!(
			Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
			expected_one_result
		);
	});
}

#[test]
fn points_works() {
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
				output: OutputBuilder::new().write_u256(100u32).build(),
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

#[test]
fn candidate_count_works() {
	ExtBuilder::default()
		.with_balances(vec![
			(TestAccount::Alice, 1_000),
			(TestAccount::Bob, 1_000),
			(TestAccount::Charlie, 1_000),
			(TestAccount::Bogus, 1_000),
		])
		.with_candidates(vec![
			(TestAccount::Alice, 1_000),
			(TestAccount::Bob, 1_000),
			(TestAccount::Charlie, 1_000),
			(TestAccount::Bogus, 1_000),
		])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"candidate_count()")[0..4];

			// Construct data to read candidate count
			let mut input_data = Vec::<u8>::from([0u8; 4]);
			input_data[0..4].copy_from_slice(&selector);

			// Expected result is 4
			let expected_one_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: OutputBuilder::new().write_u256(4u32).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that there are 4 candidates
			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_one_result
			);
		});
}

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
		.with_nominations(vec![
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
				output: OutputBuilder::new().write_u256(3u32).build(),
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
fn nominator_nomination_count_works() {
	ExtBuilder::default()
		.with_balances(vec![
			(TestAccount::Alice, 1_000),
			(TestAccount::Bob, 1_000),
			(TestAccount::Charlie, 200),
		])
		.with_candidates(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_nominations(vec![
			(TestAccount::Charlie, TestAccount::Alice, 100),
			(TestAccount::Charlie, TestAccount::Bob, 100),
		])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"nominator_nomination_count(address)")[0..4];

			// Construct data to read nominator nomination count
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Charlie.to_h160().0);

			// Expected result is 2
			let expected_one_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: OutputBuilder::new().write_u256(2u32).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that Charlie has 2 outstanding nominations
			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_one_result
			);
		});
}

#[test]
fn is_nominator_false() {
	ExtBuilder::default().build().execute_with(|| {
		let selector = &Keccak256::digest(b"is_nominator(address)")[0..4];

		// Construct data to read is_nominator
		let mut input_data = Vec::<u8>::from([0u8; 36]);
		input_data[0..4].copy_from_slice(&selector);
		input_data[16..36].copy_from_slice(&TestAccount::Charlie.to_h160().0);

		// Expected result is false
		let expected_one_result = Some(Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: OutputBuilder::new().write_bool(false).build(),
			cost: Default::default(),
			logs: Default::default(),
		}));

		// Assert that Charlie is not a nominator
		assert_eq!(
			Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
			expected_one_result
		);
	});
}

#[test]
fn is_nominator_true() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 50)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_nominations(vec![(TestAccount::Bob, TestAccount::Alice, 50)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"is_nominator(address)")[0..4];

			// Construct data to read is_nominator
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Bob.to_h160().0);

			// Expected result is true
			let expected_one_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: OutputBuilder::new().write_bool(true).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that Bob is a nominator
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
			output: OutputBuilder::new().write_bool(false).build(),
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
				output: OutputBuilder::new().write_bool(true).build(),
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

		// Construct data to read is_candidate
		let mut input_data = Vec::<u8>::from([0u8; 36]);
		input_data[0..4].copy_from_slice(&selector);
		input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);

		// Expected result is false
		let expected_one_result = Some(Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: OutputBuilder::new().write_bool(false).build(),
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
				output: OutputBuilder::new().write_bool(true).build(),
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

			// Construct data to read is_selected_candidate
			let mut input_data = Vec::<u8>::from([0u8; 68]);
			input_data[0..4].copy_from_slice(&selector);
			let amount: U256 = 1000.into();
			amount.to_big_endian(&mut input_data[4..36]);
			let candidate_count = U256::zero();
			candidate_count.to_big_endian(&mut input_data[36..]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call(
				TestAccount::Alice.to_h160(),
				precompile_address(),
				input_data,
				U256::zero(), // No value sent in EVM
				u64::max_value(),
				0.into(),
				None, // Use the next nonce
			))
			.dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::JoinedCollatorCandidates(TestAccount::Alice, 1000, 1000).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn leave_candidates_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"leave_candidates(uint256)")[0..4];

			// Construct data to read is_selected_candidate
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			let candidate_count = U256::one();
			candidate_count.to_big_endian(&mut input_data[4..]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call(
				TestAccount::Alice.to_h160(),
				precompile_address(),
				input_data,
				U256::zero(), // No value sent in EVM
				u64::max_value(),
				0.into(),
				None, // Use the next nonce
			))
			.dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CollatorScheduledExit(1, TestAccount::Alice, 3).into();
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

			// Construct selector for go_offline
			let mut input_data = Vec::<u8>::from([0u8; 4]);
			input_data[0..4].copy_from_slice(&selector);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call(
				TestAccount::Alice.to_h160(),
				precompile_address(),
				input_data,
				U256::zero(), // No value sent in EVM
				u64::max_value(),
				0.into(),
				None, // Use the next nonce
			))
			.dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CollatorBackOnline(1, TestAccount::Alice).into();
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

			// Construct selector for go_offline
			let mut input_data = Vec::<u8>::from([0u8; 4]);
			input_data[0..4].copy_from_slice(&selector);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call(
				TestAccount::Alice.to_h160(),
				precompile_address(),
				input_data,
				U256::zero(), // No value sent in EVM
				u64::max_value(),
				0.into(),
				None, // Use the next nonce
			))
			.dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CollatorWentOffline(1, TestAccount::Alice).into();
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

			// Construct selector for candidate_bond_more
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			let bond_more_amount: U256 = 500.into();
			bond_more_amount.to_big_endian(&mut input_data[4..36]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call(
				TestAccount::Alice.to_h160(),
				precompile_address(),
				input_data,
				U256::zero(), // No value sent in EVM
				u64::max_value(),
				0.into(),
				None, // Use the next nonce
			))
			.dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CollatorBondedMore(TestAccount::Alice, 1_000, 1_500).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn candidate_bond_less_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"candidate_bond_less(uint256)")[0..4];

			// Construct selector for candidate_bond_less
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			let bond_less_amount: U256 = 500.into();
			bond_less_amount.to_big_endian(&mut input_data[4..36]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call(
				TestAccount::Alice.to_h160(),
				precompile_address(),
				input_data,
				U256::zero(), // No value sent in EVM
				u64::max_value(),
				0.into(),
				None, // Use the next nonce
			))
			.dispatch(Origin::root()));

			let expected: crate::mock::Event =
				StakingEvent::CollatorBondedLess(TestAccount::Alice, 1_000, 500).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

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
			let nomination_amount: U256 = 1_000.into();
			nomination_amount.to_big_endian(&mut input_data[36..68]);
			let collator_nominator_count = U256::zero();
			collator_nominator_count.to_big_endian(&mut input_data[68..100]);
			let nomination_count = U256::zero();
			nomination_count.to_big_endian(&mut input_data[100..]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call(
				TestAccount::Bob.to_h160(),
				precompile_address(),
				input_data,
				U256::zero(), // No value sent in EVM
				u64::max_value(),
				0.into(),
				None, // Use the next nonce
			))
			.dispatch(Origin::root()));

			assert!(ParachainStaking::is_nominator(&TestAccount::Bob));

			let expected: crate::mock::Event = StakingEvent::Nomination(
				TestAccount::Bob,
				1_000,
				TestAccount::Alice,
				parachain_staking::NominatorAdded::AddedToTop { new_total: 2_000 },
			)
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn leave_nominators_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_nominations(vec![(TestAccount::Bob, TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"leave_nominators(uint256)")[0..4];

			// Construct selector for leave_nominators
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			let nomination_count = U256::one();
			nomination_count.to_big_endian(&mut input_data[4..]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call(
				TestAccount::Bob.to_h160(),
				precompile_address(),
				input_data,
				U256::zero(), // No value sent in EVM
				u64::max_value(),
				0.into(),
				None, // Use the next nonce
			))
			.dispatch(Origin::root()));

			assert!(!ParachainStaking::is_nominator(&TestAccount::Bob));

			let expected: crate::mock::Event =
				StakingEvent::NominatorLeft(TestAccount::Bob, 1_000).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn revoke_nomination_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_nominations(vec![(TestAccount::Bob, TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"revoke_nomination(address)")[0..4];

			// Construct selector for revoke_nomination
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call(
				TestAccount::Bob.to_h160(),
				precompile_address(),
				input_data,
				U256::zero(), // No value sent in EVM
				u64::max_value(),
				0.into(),
				None, // Use the next nonce
			))
			.dispatch(Origin::root()));

			assert!(!ParachainStaking::is_nominator(&TestAccount::Bob));

			let expected: crate::mock::Event =
				StakingEvent::NominatorLeft(TestAccount::Bob, 1_000).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn nominator_bond_more_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_500)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_nominations(vec![(TestAccount::Bob, TestAccount::Alice, 500)])
		.build()
		.execute_with(|| {
			// Construct the nominator_bond_more call
			let mut bond_more_call_data = Vec::<u8>::from([0u8; 68]);
			bond_more_call_data[0..4]
				.copy_from_slice(&Keccak256::digest(b"nominator_bond_more(address,uint256)")[0..4]);
			bond_more_call_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);
			let bond_more_amount: U256 = 500.into();
			bond_more_amount.to_big_endian(&mut bond_more_call_data[36..68]);

			assert_ok!(Call::Evm(EvmCall::call(
				TestAccount::Bob.to_h160(),
				precompile_address(),
				bond_more_call_data,
				U256::zero(), // No value sent in EVM
				u64::max_value(),
				0.into(),
				None, // Use the next nonce
			))
			.dispatch(Origin::root()));

			// Check for the right events.
			let expected_event: crate::mock::Event = StakingEvent::NominationIncreased(
				TestAccount::Bob,
				TestAccount::Alice,
				1_500,
				true,
				2_000,
			)
			.into();

			assert!(events().contains(&expected_event));
		});
}

#[test]
fn nominator_bond_less_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000), (TestAccount::Bob, 1_500)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.with_nominations(vec![(TestAccount::Bob, TestAccount::Alice, 1_500)])
		.build()
		.execute_with(|| {
			// Construct the nominator_bond_less call
			let mut bond_less_call_data = Vec::<u8>::from([0u8; 68]);
			bond_less_call_data[0..4]
				.copy_from_slice(&Keccak256::digest(b"nominator_bond_less(address,uint256)")[0..4]);
			bond_less_call_data[16..36].copy_from_slice(&TestAccount::Alice.to_h160().0);
			let bond_less_amount: U256 = 500.into();
			bond_less_amount.to_big_endian(&mut bond_less_call_data[36..68]);

			assert_ok!(Call::Evm(EvmCall::call(
				TestAccount::Bob.to_h160(),
				precompile_address(),
				bond_less_call_data,
				U256::zero(), // No value sent in EVM
				u64::max_value(),
				0.into(),
				None, // Use the next nonce
			))
			.dispatch(Origin::root()));

			// Check for the right events.
			let expected_event: crate::mock::Event = StakingEvent::NominationDecreased(
				TestAccount::Bob,
				TestAccount::Alice,
				2_500,
				true,
				2_000,
			)
			.into();

			assert!(events().contains(&expected_event));
		});
}
