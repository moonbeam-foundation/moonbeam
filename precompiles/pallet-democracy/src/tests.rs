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
	events, evm_test_context, precompile_address, roll_to, Call, ExtBuilder, Origin,
	Precompiles, Test, TestAccount::Alice,
};
//TODO Can PrecompileOutput come from somewhere better?
use crate::PrecompileOutput;
use frame_support::{assert_ok, dispatch::Dispatchable};
use pallet_balances::Event as BalancesEvent;
use pallet_democracy::{AccountVote, Call as DemocracyCall, Event as DemocracyEvent, Vote, Voting};
use pallet_evm::{Call as EvmCall, Event as EvmEvent};
use pallet_evm::{ExitError, ExitSucceed, PrecompileSet};
use precompile_utils::{error, EvmDataWriter};
use sha3::{Digest, Keccak256};
use sp_core::U256;
use std::convert::TryInto;

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
		// This selector is only three bytes long when four are required.
		let bogus_selector = vec![1u8, 2u8, 3u8, 4u8];

		// Expected result is an error stating there are too few bytes
		let expected_result = Some(Err(ExitError::Other(
			"No democracy wrapper method at given selector".into(),
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
fn prop_count_zero() {
	ExtBuilder::default().build().execute_with(|| {
		let selector = hex_literal::hex!("56fdf547");

		// Construct data to read prop count
		let mut input_data = Vec::<u8>::from([0u8; 4]);
		input_data[0..4].copy_from_slice(&selector);

		// Expected result is zero. because no props are open yet.
		let expected_zero_result = Some(Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: Vec::from([0u8; 32]),
			cost: Default::default(),
			logs: Default::default(),
		}));

		// Assert that no props have been opened.
		assert_eq!(
			Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
			expected_zero_result
		);
	});
}

#[test]
fn prop_count_non_zero() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"public_prop_count()")[0..4];

			// There is no interesting genesis config for pallet democracy so we make the proposal here

			//TODO is this comment about not compiling still relevant?
			// This line doesn't compile becuase it says `propose` is a private function.
			// Why is this a private function? It is defined as `pub(crate) fn propose`
			// https://github.com/paritytech/substrate/blob/polkadot-v0.9.4/frame/democracy/src/lib.rs#L637
			assert_ok!(
				Call::Democracy(DemocracyCall::propose(Default::default(), 1000u128))
					.dispatch(Origin::signed(Alice))
			);

			// Construct data to read prop count
			let input = EvmDataWriter::new().write_raw_bytes(selector).build();

			// Expected result is one
			let expected_one_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(1u32).build(),
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
fn propose_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"propose(bytes32,uint256)")[0..4];

			// Construct data to propose empty hash with value 100
			let mut input_data = Vec::<u8>::from([0u8; 68]);
			input_data[0..4].copy_from_slice(&selector);
			// Leave the hash (input_data[4..36]) empty
			let amount: U256 = 100.into();
			amount.to_big_endian(&mut input_data[36..68]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call(
				Alice.into(),
				precompile_address(),
				input_data,
				U256::zero(), // No value sent in EVM
				u64::max_value(),
				0.into(),
				None, // Use the next nonce
			))
			.dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					BalancesEvent::Reserved(Alice, 100).into(),
					DemocracyEvent::Proposed(0, 100).into(),
					EvmEvent::Executed(precompile_address()).into(),
				]
			);
		})
}

// TODO propose error cases
// propose_bad_length
// proposing fails when you don't have enough funds to cover the deposit

#[test]
fn second_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"second(uint256,uint256)")[0..4];

			// Before we can second anything, we have to have a proposal there to second.
			assert_ok!(Call::Democracy(DemocracyCall::propose(
				Default::default(), // Propose the default hash
				100u128,            // bond of 100 tokens
			))
			.dispatch(Origin::signed(Alice)));

			// Construct the call to second via a precompile
			let mut input_data = Vec::<u8>::from([0u8; 68]);
			input_data[0..4].copy_from_slice(&selector);
			let index = U256::zero();
			index.to_big_endian(&mut input_data[4..36]);
			let amount: U256 = 100.into();
			amount.to_big_endian(&mut input_data[36..68]);

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call(
				Alice.into(),
				precompile_address(),
				input_data,
				U256::zero(), // No value sent in EVM
				u64::max_value(),
				0.into(),
				None, // Use the next nonce
			))
			.dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					BalancesEvent::Reserved(Alice, 100).into(),
					DemocracyEvent::Proposed(0, 100).into(),
					// This 100 is reserved for the second.
					// Pallet democracy does not have an event for seconding
					BalancesEvent::Reserved(Alice, 100).into(),
					EvmEvent::Executed(precompile_address()).into(),
				]
			);
		})
}

// TODO Second error cases
// proposal doesn't exist
// you can't afford it

#[test]
fn standard_vote_aye_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000_000)])
		.build()
		.execute_with(|| {
			// Before we can vote on anything, we have to have a referendum there to vote on.
			// This will be nicer after https://github.com/paritytech/substrate/pull/9484
			// Make a proposal
			assert_ok!(Call::Democracy(DemocracyCall::propose(
				Default::default(), // Propose the default hash
				100u128,            // bond of 100 tokens
			))
			.dispatch(Origin::signed(Alice)));

			// Wait until it becomes a referendum (10 block launch period)
			roll_to(11);

			// Construct input data to vote aye
			let selector = &Keccak256::digest(b"stardard_vote(uint256,bool,uint256,uint256)")[0..4];
			let input = EvmDataWriter::new()
				.write_raw_bytes(selector)
				.write(0u32) // Referendum index 0
				.write(true) // Aye
				.write(100_000u128) // 100_000 tokens
				.write(0u8) // No conviction
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

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					// Making proposal
					BalancesEvent::Reserved(Alice, 100).into(),
					DemocracyEvent::Proposed(0, 100).into(),
					// Proposal -> Referendum
					BalancesEvent::Unreserved(Alice, 100).into(),
					DemocracyEvent::Tabled(0, 100, vec![Alice]).into(),
					DemocracyEvent::Started(
						0,
						pallet_democracy::VoteThreshold::SuperMajorityApprove
					)
					.into(),
					EvmEvent::Executed(precompile_address()).into(),
				]
			);

			// Assert that the vote was recorded in storage
			// Should check ReferendumInfoOf too, but can't because of private fields etc
			assert_eq!(
				pallet_democracy::VotingOf::<Test>::get(Alice),
				Voting::Direct {
					votes: vec![(
						0,
						AccountVote::Standard {
							vote: Vote {
								aye: true,
								conviction: 0u8.try_into().unwrap()
							},
							balance: 100_000,
						}
					)],
					delegations: Default::default(),
					prior: Default::default(),
				},
			);
		})
}

#[test]
fn standard_vote_nay_conviction_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000_000)])
		.build()
		.execute_with(|| {
			// Before we can vote on anything, we have to have a referendum there to vote on.
			// This will be nicer after https://github.com/paritytech/substrate/pull/9484
			// Make a proposal
			assert_ok!(Call::Democracy(DemocracyCall::propose(
				Default::default(), // Propose the default hash
				100u128,            // bond of 100 tokens
			))
			.dispatch(Origin::signed(Alice)));

			// Wait until it becomes a referendum (10 block launch period)
			roll_to(11);

			// Construct input data to vote aye
			let selector = &Keccak256::digest(b"stardard_vote(uint256,bool,uint256,uint256)")[0..4];
			let input = EvmDataWriter::new()
				.write_raw_bytes(selector)
				.write(0u32) // Referendum index 0
				.write(false) // Nay
				.write(100_000u128) // 100_000 tokens
				.write(3u8) // 3X conviction
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

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					// Making proposal
					BalancesEvent::Reserved(Alice, 100).into(),
					DemocracyEvent::Proposed(0, 100).into(),
					// Proposal -> Referendum
					BalancesEvent::Unreserved(Alice, 100).into(),
					DemocracyEvent::Tabled(0, 100, vec![Alice]).into(),
					DemocracyEvent::Started(
						0,
						pallet_democracy::VoteThreshold::SuperMajorityApprove
					)
					.into(),
					EvmEvent::Executed(precompile_address()).into(),
				]
			);

			// Assert that the vote was recorded in storage
			// Should check ReferendumInfoOf too, but can't because of private fields etc
			assert_eq!(
				pallet_democracy::VotingOf::<Test>::get(Alice),
				Voting::Direct {
					votes: vec![(
						0,
						AccountVote::Standard {
							vote: Vote {
								aye: false,
								conviction: 3u8.try_into().unwrap()
							},
							balance: 100_000,
						}
					)],
					delegations: Default::default(),
					prior: Default::default(),
				},
			);
		})
}

//TODO Standard vote error cases
// can't afford it
// invalid conviction
// referendum doesn't exist

#[test]
fn remove_vote_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// Before we can vote on anything, we have to have a referendum there to vote on.
			// This will be nicer after https://github.com/paritytech/substrate/pull/9484
			// Make a proposal
			assert_ok!(Call::Democracy(DemocracyCall::propose(
				Default::default(), // Propose the default hash
				100u128,            // bond of 100 tokens
			))
			.dispatch(Origin::signed(Alice)));

			// Wait until it becomes a referendum (10 block launch period)
			roll_to(11);

			// Vote on it
			//TODO Can I call this directly now? I thought they are all public?
			assert_ok!(Call::Democracy(DemocracyCall::vote(
				0, // Propose the default hash
				AccountVote::Standard {
					vote: Vote {
						aye: true,
						conviction: 0u8.try_into().unwrap()
					},
					balance: 100,
				},
			))
			.dispatch(Origin::signed(Alice)));

			// Construct input data to remove the vote
			let selector = &Keccak256::digest(b"remove_vote(uint256)")[0..4];
			let input = EvmDataWriter::new()
				.write_raw_bytes(selector)
				.write(0u32) // Referendum index 0
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

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					// Making proposal
					BalancesEvent::Reserved(Alice, 100).into(),
					DemocracyEvent::Proposed(0, 100).into(),
					// Proposal -> Referendum
					BalancesEvent::Unreserved(Alice, 100).into(),
					DemocracyEvent::Tabled(0, 100, vec![Alice]).into(),
					DemocracyEvent::Started(
						0,
						pallet_democracy::VoteThreshold::SuperMajorityApprove
					)
					.into(),
					EvmEvent::Executed(precompile_address()).into(),
				]
			);

			// Assert that the vote was recorded in storage
			// Should check ReferendumInfoOf too, but can't because of private fields etc
			assert_eq!(
				pallet_democracy::VotingOf::<Test>::get(Alice),
				Voting::Direct {
					votes: vec![],
					delegations: Default::default(),
					prior: Default::default(),
				},
			);
		})
}

#[test]
fn remove_vote_dne() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// Before we can vote on anything, we have to have a referendum there to vote on.
			// This will be nicer after https://github.com/paritytech/substrate/pull/9484
			// Make a proposal
			assert_ok!(Call::Democracy(DemocracyCall::propose(
				Default::default(), // Propose the default hash
				100u128,            // bond of 100 tokens
			))
			.dispatch(Origin::signed(Alice)));

			// Wait until it becomes a referendum (10 block launch period)
			roll_to(11);

			// Construct input data to remove a non-existant vote
			let selector = &Keccak256::digest(b"remove_vote(uint256)")[0..4];
			let input = EvmDataWriter::new()
				.write_raw_bytes(selector)
				.write(0u32) // Referendum index 0
				.build();

			// TODO one weakness of try_dispatch is that it doesn't propogate the error
			// I can't assert that this actually failed for the reason I expected.
			// Expected result is an error stating there are too few bytes
			let expected_result = Some(Err(error("dispatched call failed")));

			assert_eq!(
				Precompiles::execute(precompile_address(), &input, None, &evm_test_context(),),
				expected_result
			);
		})
}

#[test]
fn delegate_works() {
	todo!()
}

//TODO Delecate error cases
// invalid conviction

#[test]
fn undelegate_works() {
	todo!()
}

#[test]
fn undelegate_dne() {
	todo!()
}

#[test]
fn unlock_with_nothing_locked() {
	todo!()
}

#[test]
fn unlock_works() {
	// This one will be hard because we have to get some tokens locked first.
	todo!()
}
