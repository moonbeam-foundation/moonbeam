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
	events, roll_to, AccountId, Crowdloan, ExtBuilder, PCall, Precompiles, PrecompilesValue,
	Runtime, RuntimeCall, RuntimeOrigin,
};
use frame_support::assert_ok;
use pallet_crowdloan_rewards::{Call as CrowdloanCall, Event as CrowdloanEvent};
use pallet_evm::Call as EvmCall;
use precompile_utils::{prelude::*, testing::*};
use sha3::{Digest, Keccak256};
use sp_core::U256;
use sp_runtime::traits::Dispatchable;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

fn evm_call(input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: Alice.into(),
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
	assert!(PCall::is_contributor_selectors().contains(&0x1d0d35f5));
	assert!(PCall::reward_info_selectors().contains(&0xcbecf6b5));
	assert!(PCall::claim_selectors().contains(&0x4e71d92d));
	assert!(PCall::update_reward_address_selectors().contains(&0x944dd5a2));
}

#[test]
fn modifiers() {
	ExtBuilder::default().build().execute_with(|| {
		let mut tester = PrecompilesModifierTester::new(precompiles(), Alice, Precompile1);

		tester.test_view_modifier(PCall::is_contributor_selectors());
		tester.test_view_modifier(PCall::reward_info_selectors());
		tester.test_default_modifier(PCall::claim_selectors());
		tester.test_default_modifier(PCall::update_reward_address_selectors());
	});
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
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
fn is_contributor_returns_false() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::is_contributor {
						contributor: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(false);
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
			assert_ok!(
				RuntimeCall::Crowdloan(CrowdloanCall::initialize_reward_vec {
					rewards: vec![
						([1u8; 32], Some(Alice.into()), 50u32.into()),
						([2u8; 32], Some(Bob.into()), 50u32.into()),
					]
				})
				.dispatch(RuntimeOrigin::root())
			);

			assert_ok!(Crowdloan::complete_initialization(
				RuntimeOrigin::root(),
				init_block + VESTING
			));

			// Assert that no props have been opened.
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::is_contributor {
						contributor: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(true);
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
			assert_ok!(
				RuntimeCall::Crowdloan(CrowdloanCall::initialize_reward_vec {
					rewards: vec![
						([1u8; 32].into(), Some(Alice.into()), 50u32.into()),
						([2u8; 32].into(), Some(Bob.into()), 50u32.into()),
					]
				})
				.dispatch(RuntimeOrigin::root())
			);

			assert_ok!(Crowdloan::complete_initialization(
				RuntimeOrigin::root(),
				init_block + VESTING
			));

			roll_to(5);

			let input = PCall::claim {}.into();

			// Make sure the call goes through successfully
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			let expected: crate::mock::RuntimeEvent =
				CrowdloanEvent::RewardsPaid(Alice.into(), 25).into();
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
			assert_ok!(
				RuntimeCall::Crowdloan(CrowdloanCall::initialize_reward_vec {
					rewards: vec![
						([1u8; 32].into(), Some(Alice.into()), 50u32.into()),
						([2u8; 32].into(), Some(Bob.into()), 50u32.into()),
					]
				})
				.dispatch(RuntimeOrigin::root())
			);

			assert_ok!(Crowdloan::complete_initialization(
				RuntimeOrigin::root(),
				init_block + VESTING
			));

			roll_to(5);

			// Assert that no props have been opened.
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::reward_info {
						contributor: Address(Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns((U256::from(50u64), U256::from(10u64)));
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
			assert_ok!(
				RuntimeCall::Crowdloan(CrowdloanCall::initialize_reward_vec {
					rewards: vec![
						([1u8; 32].into(), Some(Alice.into()), 50u32.into()),
						([2u8; 32].into(), Some(Bob.into()), 50u32.into()),
					]
				})
				.dispatch(RuntimeOrigin::root())
			);

			assert_ok!(Crowdloan::complete_initialization(
				RuntimeOrigin::root(),
				init_block + VESTING
			));

			roll_to(5);

			let input = PCall::update_reward_address {
				new_address: Address(Charlie.into()),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			let expected: crate::mock::RuntimeEvent =
				CrowdloanEvent::RewardAddressUpdated(Alice.into(), Charlie.into()).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
			// Assert storage is correctly moved
			assert!(Crowdloan::accounts_payable(AccountId::from(Alice)).is_none());
			assert!(Crowdloan::accounts_payable(AccountId::from(Charlie)).is_some());
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
				.prepare_test(Alice, Precompile1, input)
				.execute_reverts(|output| output == b"Expected at least 1 arguments")
		})
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	check_precompile_implements_solidity_interfaces(
		&["CrowdloanInterface.sol"],
		PCall::supports_selector,
	)
}

#[test]
fn test_deprecated_solidity_selectors_are_supported() {
	for deprecated_function in [
		"is_contributor(address)",
		"reward_info(address)",
		"update_reward_address(address)",
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
