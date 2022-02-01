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

use std::assert_matches::assert_matches;

use crate::mock::{
	events, evm_test_context, precompile_address, roll_to, Call, Crowdloan, ExtBuilder, Origin,
	PrecompilesValue, Runtime, TestAccount::Alice, TestAccount::Bob, TestAccount::Charlie,
	TestPrecompiles,
};
use crate::{Action, PrecompileOutput};
use fp_evm::PrecompileFailure;
use frame_support::{assert_ok, dispatch::Dispatchable};
use num_enum::TryFromPrimitive;
use pallet_crowdloan_rewards::{Call as CrowdloanCall, Event as CrowdloanEvent};
use pallet_evm::{Call as EvmCall, ExitSucceed, PrecompileSet};
use precompile_utils::{Address, EvmDataWriter};
use sha3::{Digest, Keccak256};
use sp_core::{H160, U256};

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

fn evm_call(input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: Alice.into(),
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
fn test_selector_enum() {
	let mut buffer = [0u8; 4];
	buffer.copy_from_slice(&Keccak256::digest(b"is_contributor(address)")[0..4]);
	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::IsContributor,
	);
	buffer.copy_from_slice(&Keccak256::digest(b"claim()")[0..4]);
	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::Claim,
	);
	buffer.copy_from_slice(&Keccak256::digest(b"reward_info(address)")[0..4]);
	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::RewardInfo,
	);
	buffer.copy_from_slice(&Keccak256::digest(b"update_reward_address(address)")[0..4]);
	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::UpdateRewardAddress,
	);
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
				if output == b"tried to parse selector out of bounds"
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
				if output == b"unknown selector",
		);
	});
}

#[test]
fn is_contributor_returns_false() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let input = EvmDataWriter::new_with_selector(Action::IsContributor)
				.write(Address(H160::from(Alice)))
				.build();

			// Expected result is one
			let expected_one_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(false).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that no props have been opened.
			assert_eq!(
				precompiles().execute(
					precompile_address(),
					&input,
					None,
					&evm_test_context(),
					false
				),
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

			let init_block = Crowdloan::init_vesting_block();
			assert_ok!(Call::Crowdloan(CrowdloanCall::initialize_reward_vec {
				rewards: vec![
					([1u8; 32], Some(Alice), 50u32.into()),
					([2u8; 32], Some(Bob), 50u32.into()),
				]
			})
			.dispatch(Origin::root()));

			assert_ok!(Crowdloan::complete_initialization(
				Origin::root(),
				init_block + VESTING
			));

			let input = EvmDataWriter::new_with_selector(Action::IsContributor)
				.write(Address(H160::from(Alice)))
				.build();

			// Assert that no props have been opened.
			assert_eq!(
				precompiles().execute(
					precompile_address(),
					&input,
					None,
					&evm_test_context(),
					false
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
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

			let init_block = Crowdloan::init_vesting_block();
			assert_ok!(Call::Crowdloan(CrowdloanCall::initialize_reward_vec {
				rewards: vec![
					([1u8; 32].into(), Some(Alice), 50u32.into()),
					([2u8; 32].into(), Some(Bob), 50u32.into()),
				]
			})
			.dispatch(Origin::root()));

			assert_ok!(Crowdloan::complete_initialization(
				Origin::root(),
				init_block + VESTING
			));

			roll_to(5);

			let input = EvmDataWriter::new_with_selector(Action::Claim).build();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(input)).dispatch(Origin::root()));

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

			let init_block = Crowdloan::init_vesting_block();
			assert_ok!(Call::Crowdloan(CrowdloanCall::initialize_reward_vec {
				rewards: vec![
					([1u8; 32].into(), Some(Alice), 50u32.into()),
					([2u8; 32].into(), Some(Bob), 50u32.into()),
				]
			})
			.dispatch(Origin::root()));

			assert_ok!(Crowdloan::complete_initialization(
				Origin::root(),
				init_block + VESTING
			));

			roll_to(5);

			let input = EvmDataWriter::new_with_selector(Action::RewardInfo)
				.write(Address(H160::from(Alice)))
				.build();

			// Assert that no props have been opened.
			assert_eq!(
				precompiles().execute(
					precompile_address(),
					&input,
					None,
					&evm_test_context(),
					false
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new()
						.write(U256::from(50u64))
						.write(U256::from(10u64))
						.build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
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

			let init_block = Crowdloan::init_vesting_block();
			assert_ok!(Call::Crowdloan(CrowdloanCall::initialize_reward_vec {
				rewards: vec![
					([1u8; 32].into(), Some(Alice), 50u32.into()),
					([2u8; 32].into(), Some(Bob), 50u32.into()),
				]
			})
			.dispatch(Origin::root()));

			assert_ok!(Crowdloan::complete_initialization(
				Origin::root(),
				init_block + VESTING
			));

			roll_to(5);

			let input = EvmDataWriter::new_with_selector(Action::UpdateRewardAddress)
				.write(Address(H160::from(Charlie)))
				.build();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(input)).dispatch(Origin::root()));

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
			let mut input = Keccak256::digest(b"update_reward_address(address)")[0..4].to_vec();
			input.extend_from_slice(&[1u8; 4]); // incomplete data

			assert_matches!(
				precompiles().execute(
					precompile_address(),
					&input,
					None,
					&evm_test_context(),
					false
				),
				Some(Err(PrecompileFailure::Revert { output, ..}))
				if output == b"input doesn't match expected length",
			);
		})
}
