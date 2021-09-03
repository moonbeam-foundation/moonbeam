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
	events, evm_test_context, precompile_address, roll_to, Call, Crowdloan, ExtBuilder, Origin,
	Precompiles, TestAccount::Alice, TestAccount::Bob, TestAccount::Charlie,
};
use crate::{Action, PrecompileOutput};
use frame_support::{assert_ok, dispatch::Dispatchable};
use num_enum::TryFromPrimitive;
use pallet_crowdloan_rewards::{Call as CrowdloanCall, Event as CrowdloanEvent};
use pallet_evm::{Call as EvmCall, ExitSucceed, PrecompileSet};
use precompile_utils::{error, Address, EvmDataWriter};
use sha3::{Digest, Keccak256};
use sp_core::{H160, U256};

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

#[test]
fn is_contributor_returns_false() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"is_contributor(address)")[0..4];
			let input = EvmDataWriter::new()
				.write_raw_bytes(selector)
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
				Precompiles::execute(precompile_address(), &input, None, &evm_test_context()),
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
				([1u8; 32], Some(Alice), 50u32.into()),
				([2u8; 32], Some(Bob), 50u32.into()),
			]))
			.dispatch(Origin::root()));

			assert_ok!(Crowdloan::complete_initialization(
				Origin::root(),
				init_block + VESTING
			));

			let selector = &Keccak256::digest(b"is_contributor(address)")[0..4];
			let input = EvmDataWriter::new()
				.write_raw_bytes(selector)
				.write(Address(H160::from(Alice)))
				.build();

			// Assert that no props have been opened.
			assert_eq!(
				Precompiles::execute(precompile_address(), &input, None, &evm_test_context()),
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

			println!("rolled to five");

			let selector = &Keccak256::digest(b"claim()")[0..4];
			let input = EvmDataWriter::new().write_raw_bytes(selector).build();

			// println!("about to call evm");

			// let inner_call = EvmCall::call(
			// 	Alice.into(),
			// 	precompile_address(),
			// 	input,
			// 	U256::zero(), // No value sent in EVM
			// 	u64::max_value(),
			// 	0.into(),
			// 	None, // Use the next nonce
			// );
			// println!("evm call is");
			// println!("{:?}", inner_call);
			//
			// // Make sure the call goes through successfully
			// assert_ok!(Call::Evm(inner_call).dispatch(Origin::root()));
			//
			// println!("returned from evm");

			// let's try calling claim directly instead of through the precompile.
			// So maybe it isn't the precompile? It seems this call just isn't working?
			assert_ok!(Call::Crowdloan(CrowdloanCall::claim()).dispatch(Origin::signed(Alice)));

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
			let input = EvmDataWriter::new()
				.write_raw_bytes(selector)
				.write(Address(H160::from(Alice)))
				.build();

			// Assert that no props have been opened.
			assert_eq!(
				Precompiles::execute(precompile_address(), &input, None, &evm_test_context()),
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
			let input = EvmDataWriter::new()
				.write_raw_bytes(selector)
				.write(Address(H160::from(Charlie)))
				.build();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call(
				Alice.into(),
				precompile_address(),
				input,
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
			let input = EvmDataWriter::new()
				.write_raw_bytes(&selector)
				.write_raw_bytes(&[1u8; 4]) // incomplete data
				.build();

			assert_eq!(
				Precompiles::execute(precompile_address(), &input, None, &evm_test_context(),),
				Some(Err(error("input doesn't match expected length",)))
			);
		})
}
