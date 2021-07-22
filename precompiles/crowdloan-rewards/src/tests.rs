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

use crate::mock::Crowdloan;
use crate::mock::{
	events, evm_test_context, precompile_address, roll_to, Call, ExtBuilder, Origin, Precompiles,
	TestAccount::Alice, TestAccount::Bob, TestAccount::Charlie,
};
use crate::PrecompileOutput;
use frame_support::{assert_ok, dispatch::Dispatchable};
use pallet_crowdloan_rewards::{Call as CrowdloanCall, Event as CrowdloanEvent};
use pallet_evm::Call as EvmCall;
use pallet_evm::{ExitError, ExitSucceed, PrecompileSet};
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
			"No crowdloan rewards wrapper method at given selector".into(),
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
fn is_contributor_returns_false() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"is_contributor(address)")[0..4];

			// Construct data to read prop count
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&Alice.to_h160().0);

			// Expected result is one
			let expected_one_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: OutputBuilder::new().write_bool(false).build(),
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
fn is_contributor_returns_true() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.with_crowdloan_pot(100u32.into())
		.build()
		.execute_with(|| {
			pub const VESTING: u32 = 8;
			// The init relay block gets inserted
			roll_to(2);

			let init_block = Crowdloan::init_relay_block();
			assert_ok!(Call::Crowdloan(CrowdloanCall::initialize_reward_vec(vec![
				([1u8; 32].into(), Some(Alice), 50u32.into()),
				([2u8; 32].into(), Some(Bob), 50u32.into()),
			]))
			.dispatch(Origin::root()));

			assert_ok!(Crowdloan::complete_initialization(
				Origin::root(),
				init_block + VESTING
			));
			let selector = &Keccak256::digest(b"is_contributor(address)")[0..4];

			// Construct data to read prop count
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&Alice.to_h160().0);

			// Expected result is one
			let expected_one_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: OutputBuilder::new().write_bool(true).build(),
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
fn claim_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.with_crowdloan_pot(100u32.into())
		.build()
		.execute_with(|| {
			pub const VESTING: u32 = 8;
			// The init relay block gets inserted
			roll_to(2);

			let init_block = Crowdloan::init_relay_block();
			assert_ok!(Call::Crowdloan(CrowdloanCall::initialize_reward_vec(vec![
				([1u8; 32].into(), Some(Alice), 50u32.into()),
				([2u8; 32].into(), Some(Bob), 50u32.into()),
			]))
			.dispatch(Origin::root()));

			assert_ok!(Crowdloan::complete_initialization(
				Origin::root(),
				init_block + VESTING
			));

			roll_to(5);

			let selector = &Keccak256::digest(b"claim()")[0..4];

			// Construct data to read prop count
			let mut input_data = Vec::<u8>::from([0u8; 4]);
			input_data[0..4].copy_from_slice(&selector);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call(
				Alice.to_h160(),
				precompile_address(),
				input_data,
				U256::zero(), // No value sent in EVM
				u64::max_value(),
				0.into(),
				None, // Use the next nonce
			))
			.dispatch(Origin::root()));

			let expected: crate::mock::Event = CrowdloanEvent::RewardsPaid(Alice, 25).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn reward_info_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.with_crowdloan_pot(100u32.into())
		.build()
		.execute_with(|| {
			pub const VESTING: u32 = 8;
			// The init relay block gets inserted
			roll_to(2);

			let init_block = Crowdloan::init_relay_block();
			assert_ok!(Call::Crowdloan(CrowdloanCall::initialize_reward_vec(vec![
				([1u8; 32].into(), Some(Alice), 50u32.into()),
				([2u8; 32].into(), Some(Bob), 50u32.into()),
			]))
			.dispatch(Origin::root()));

			assert_ok!(Crowdloan::complete_initialization(
				Origin::root(),
				init_block + VESTING
			));

			roll_to(5);

			let selector = &Keccak256::digest(b"reward_info(address)")[0..4];

			// Construct data to read prop count
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&Alice.to_h160().0);

			let mut output = OutputBuilder::new().write_u256(50u64).build();
			output.extend(OutputBuilder::new().write_u256(10u64).build()); // Expected result
			let expected_buffer_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: output,
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that no props have been opened.
			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_buffer_result
			);
		});
}

#[test]
fn update_reward_address_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.with_crowdloan_pot(100u32.into())
		.build()
		.execute_with(|| {
			pub const VESTING: u32 = 8;
			// The init relay block gets inserted
			roll_to(2);

			let init_block = Crowdloan::init_relay_block();
			assert_ok!(Call::Crowdloan(CrowdloanCall::initialize_reward_vec(vec![
				([1u8; 32].into(), Some(Alice), 50u32.into()),
				([2u8; 32].into(), Some(Bob), 50u32.into()),
			]))
			.dispatch(Origin::root()));

			assert_ok!(Crowdloan::complete_initialization(
				Origin::root(),
				init_block + VESTING
			));

			roll_to(5);

			let selector = &Keccak256::digest(b"update_reward_address(address)")[0..4];

			// Construct data to read prop count
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..36].copy_from_slice(&Charlie.to_h160().0);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call(
				Alice.to_h160(),
				precompile_address(),
				input_data,
				U256::zero(), // No value sent in EVM
				u64::max_value(),
				0.into(),
				None, // Use the next nonce
			))
			.dispatch(Origin::root()));

			let expected: crate::mock::Event =
				CrowdloanEvent::RewardAddressUpdated(Alice, Charlie).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
			// Assert storage is correctly moved
			assert!(Crowdloan::accounts_payable(Alice).is_none());
			assert!(Crowdloan::accounts_payable(Charlie).is_some());
		});
}

#[test]
fn test_bound_checks_for_address_parsing() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.with_crowdloan_pot(100u32.into())
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"update_reward_address(address)")[0..4];

			// Construct data to read prop count
			let mut input_data = Vec::<u8>::from([0u8; 20]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[16..20].copy_from_slice(&[1u8; 4]);

			// Expected result is an error stating there are too few bytes
			let expected_result = Some(Err(ExitError::Other(
				"input doesn't match expected length".into(),
			)));

			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context(),),
				expected_result
			);
		})
}
