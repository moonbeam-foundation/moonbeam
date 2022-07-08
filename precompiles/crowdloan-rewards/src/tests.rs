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
	events, roll_to,
	Account::{Alice, Bob, Charlie, Precompile},
	Call, Crowdloan, ExtBuilder, Origin, PrecompilesValue, Runtime, TestPrecompiles,
};
use crate::Action;
use frame_support::{assert_ok, dispatch::Dispatchable};
use pallet_crowdloan_rewards::{Call as CrowdloanCall, Event as CrowdloanEvent};
use pallet_evm::Call as EvmCall;
use precompile_utils::{prelude::*, testing::*};
use sha3::{Digest, Keccak256};
use sp_core::{H160, U256};

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

fn evm_call(input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: Alice.into(),
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
fn test_selector_enum() {
	assert_eq!(Action::IsContributor as u32, 0x53440c90);
	assert_eq!(Action::RewardInfo as u32, 0x76f70249);
	assert_eq!(Action::Claim as u32, 0x4e71d92d);
	assert_eq!(Action::UpdateRewardAddress as u32, 0xaaac61d6);
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
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
fn is_contributor_returns_false() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::IsContributor)
						.write(Address(H160::from(Alice)))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(false).build());
		});
}

#[test]
fn is_contributor_returns_true() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.with_crowdloan_pot(100u32.into())
		.build()
		.execute_with(|| {
			pub const VESTING: u32 = 8;
			// The init relay block gets inserted
			roll_to(2);

			let init_block = Crowdloan::init_vesting_block();
			assert_ok!(Call::Crowdloan(CrowdloanCall::initialize_reward_vec {
				rewards: vec![
					([1u8; 32], Some(Alice.into()), 50u32.into()),
					([2u8; 32], Some(Bob.into()), 50u32.into()),
				]
			})
			.dispatch(Origin::root()));

			assert_ok!(Crowdloan::complete_initialization(
				Origin::root(),
				init_block + VESTING
			));

			// Assert that no props have been opened.
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::IsContributor)
						.write(Address(H160::from(Alice)))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());
		});
}

#[test]
fn claim_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.with_crowdloan_pot(100u32.into())
		.build()
		.execute_with(|| {
			pub const VESTING: u32 = 8;
			// The init relay block gets inserted
			roll_to(2);

			let init_block = Crowdloan::init_vesting_block();
			assert_ok!(Call::Crowdloan(CrowdloanCall::initialize_reward_vec {
				rewards: vec![
					([1u8; 32].into(), Some(Alice.into()), 50u32.into()),
					([2u8; 32].into(), Some(Bob.into()), 50u32.into()),
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

			let expected: crate::mock::Event = CrowdloanEvent::RewardsPaid(Alice.into(), 25).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn reward_info_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.with_crowdloan_pot(100u32.into())
		.build()
		.execute_with(|| {
			pub const VESTING: u32 = 8;
			// The init relay block gets inserted
			roll_to(2);

			let init_block = Crowdloan::init_vesting_block();
			assert_ok!(Call::Crowdloan(CrowdloanCall::initialize_reward_vec {
				rewards: vec![
					([1u8; 32].into(), Some(Alice.into()), 50u32.into()),
					([2u8; 32].into(), Some(Bob.into()), 50u32.into()),
				]
			})
			.dispatch(Origin::root()));

			assert_ok!(Crowdloan::complete_initialization(
				Origin::root(),
				init_block + VESTING
			));

			roll_to(5);

			// Assert that no props have been opened.
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::RewardInfo)
						.write(Address(H160::from(Alice)))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(U256::from(50u64))
						.write(U256::from(10u64))
						.build(),
				);
		});
}

#[test]
fn update_reward_address_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.with_crowdloan_pot(100u32.into())
		.build()
		.execute_with(|| {
			pub const VESTING: u32 = 8;
			// The init relay block gets inserted
			roll_to(2);

			let init_block = Crowdloan::init_vesting_block();
			assert_ok!(Call::Crowdloan(CrowdloanCall::initialize_reward_vec {
				rewards: vec![
					([1u8; 32].into(), Some(Alice.into()), 50u32.into()),
					([2u8; 32].into(), Some(Bob.into()), 50u32.into()),
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
				CrowdloanEvent::RewardAddressUpdated(Alice.into(), Charlie.into()).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
			// Assert storage is correctly moved
			assert!(Crowdloan::accounts_payable(H160::from(Alice)).is_none());
			assert!(Crowdloan::accounts_payable(H160::from(Charlie)).is_some());
		});
}

#[test]
fn test_bound_checks_for_address_parsing() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.with_crowdloan_pot(100u32.into())
		.build()
		.execute_with(|| {
			let mut input = Keccak256::digest(b"update_reward_address(address)")[0..4].to_vec();
			input.extend_from_slice(&[1u8; 4]); // incomplete data

			precompiles()
				.prepare_test(Alice, Precompile, input)
				.execute_reverts(|output| output == b"input doesn't match expected length")
		})
}
