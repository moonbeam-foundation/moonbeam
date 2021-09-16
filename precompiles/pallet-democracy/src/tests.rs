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

use crate::{
	mock::{
		events, evm_test_context, precompile_address, roll_to, Balances, Call, Democracy,
		ExtBuilder, Origin, Precompiles, Test,
		TestAccount::{self, Alice, Bob},
	},
	Action,
};
use evm::executor::PrecompileOutput;
use frame_support::{assert_ok, dispatch::Dispatchable, traits::Currency};
use pallet_balances::Event as BalancesEvent;
use pallet_democracy::{
	AccountVote, Call as DemocracyCall, Config as DemocracyConfig, Event as DemocracyEvent, Vote,
	VoteThreshold, Voting,
};
use pallet_evm::{Call as EvmCall, Event as EvmEvent, ExitError, ExitSucceed, PrecompileSet};
use precompile_utils::{error, Address, EvmDataWriter};
use sp_core::{H160, U256};
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
		let bogus_selector = vec![1u8, 2u8, 3u8, 4u8];

		// Expected result is an error stating there are too few bytes
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
fn selectors() {
	assert_eq!(Action::Delegate as u32, 0x0185921e);
	assert_eq!(Action::DepositOf as u32, 0xa30305e9);
	assert_eq!(Action::FinishedReferendumInfo as u32, 0xb1fd383f);
	assert_eq!(Action::LowestUnbaked as u32, 0x0388f282);
	assert_eq!(Action::OngoingReferendumInfo as u32, 0x8b93d11a);
	assert_eq!(Action::Propose as u32, 0x7824e7d1);
	assert_eq!(Action::PublicPropCount as u32, 0x56fdf547);
	assert_eq!(Action::RemoveVote as u32, 0x2042f50b);
	assert_eq!(Action::Second as u32, 0xc7a76601);
	assert_eq!(Action::StandardVote as u32, 0x3f3c21cc);
	assert_eq!(Action::UnDelegate as u32, 0xcb37b8ea);
	assert_eq!(Action::Unlock as u32, 0x2f6c493c);

	//TODO also test logs once we have them
}

#[test]
fn prop_count_zero() {
	ExtBuilder::default().build().execute_with(|| {
		// Construct data to read prop count
		let input = EvmDataWriter::new()
			.write_selector(Action::PublicPropCount)
			.build();

		// Expected result is zero. because no props are open yet.
		let expected_zero_result = Some(Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: Vec::from([0u8; 32]),
			cost: Default::default(),
			logs: Default::default(),
		}));

		// Assert that no props have been opened.
		assert_eq!(
			Precompiles::execute(precompile_address(), &input, None, &evm_test_context()),
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
			// There is no interesting genesis config for pallet democracy so we make the proposal here
			assert_ok!(
				Call::Democracy(DemocracyCall::propose(Default::default(), 1000u128))
					.dispatch(Origin::signed(Alice))
			);

			// Construct data to read prop count
			let input = EvmDataWriter::new()
				.write_selector(Action::PublicPropCount)
				.build();

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

// It is impossible to have a proposal with zero deposits. When the proposal is made, the proposer
// makes a deposit. So there is always at least one depositor.
#[test]
fn deposit_of_non_zero() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// There is no interesting genesis config for pallet democracy so we make the proposal here
			assert_ok!(
				Call::Democracy(DemocracyCall::propose(Default::default(), 1000u128))
					.dispatch(Origin::signed(Alice))
			);

			// Construct data to read prop count
			let input = EvmDataWriter::new()
				.write_selector(Action::DepositOf)
				.write(0u32)
				.build();

			// Expected result is Alice's deposit of 1000
			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(1000u32).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				Precompiles::execute(precompile_address(), &input, None, &evm_test_context()),
				expected_result
			)
		});
}

#[test]
fn deposit_of_bad_index() {
	ExtBuilder::default().build().execute_with(|| {
		// Construct data to read prop count
		let input = EvmDataWriter::new()
			.write_selector(Action::DepositOf)
			.write(10u32)
			.build();

		// Expected result is an error stating there is no such proposal in the underlying pallet
		let expected_result = Some(Err(error("No such proposal in pallet democracy")));

		assert_eq!(
			Precompiles::execute(precompile_address(), &input, None, &evm_test_context(),),
			expected_result
		);
	});
}

#[test]
fn lowest_unbaked_zero() {
	ExtBuilder::default().build().execute_with(|| {
		// Construct data to read lowest unbaked referendum index
		let input = EvmDataWriter::new()
			.write_selector(Action::LowestUnbaked)
			.build();

		// Expected result is zero
		let expected_zero_result = Some(Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: EvmDataWriter::new().write(0u32).build(),
			cost: Default::default(),
			logs: Default::default(),
		}));

		assert_eq!(
			Precompiles::execute(precompile_address(), &input, None, &evm_test_context()),
			expected_zero_result
		)
	});
}

// This test is currently failing. I believe it is caused by a bug in the underlying pallet. I've
// asked about it in https://github.com/paritytech/substrate/issues/9739
#[ignore]
#[test]
fn lowest_unbaked_non_zero() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000_000)])
		.with_referenda(vec![
			(Default::default(), VoteThreshold::SimpleMajority, 10),
			(Default::default(), VoteThreshold::SimpleMajority, 10),
		])
		.build()
		.execute_with(|| {
			// To ensure the referendum passes, we need an Aye vote on it
			assert_ok!(Call::Democracy(DemocracyCall::vote(
				0, // referendum 0
				AccountVote::Standard {
					vote: Vote {
						aye: true,
						conviction: 0u8.try_into().unwrap()
					},
					balance: 100_000,
				}
			))
			.dispatch(Origin::signed(Alice)));

			// Assert that the vote was recorded in storage
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

			// Run it through until it is baked
			roll_to(
				<Test as DemocracyConfig>::VotingPeriod::get()
					+ <Test as DemocracyConfig>::LaunchPeriod::get()
					+ 1000,
			);

			// Construct data to read lowest unbaked referendum index
			let input = EvmDataWriter::new()
				.write_selector(Action::LowestUnbaked)
				.build();

			// Expected result is one
			let expected_one_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(1u32).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				Precompiles::execute(precompile_address(), &input, None, &evm_test_context()),
				expected_one_result
			)
		});
}

// waiting on https://github.com/paritytech/substrate/pull/9565
#[ignore]
#[test]
fn ongoing_ref_info_works() {
	todo!()
}

// waiting on https://github.com/paritytech/substrate/pull/9565
#[ignore]
#[test]
fn ongoing_ref_info_bad_index() {
	todo!()
}

// waiting on https://github.com/paritytech/substrate/pull/9565
#[ignore]
#[test]
fn ongoing_ref_info_is_not_ongoing() {
	todo!()
}

// waiting on https://github.com/paritytech/substrate/pull/9565
#[ignore]
#[test]
fn finished_ref_info_works() {
	todo!()
}

// waiting on https://github.com/paritytech/substrate/pull/9565
#[ignore]
#[test]
fn finished_ref_info_bad_index() {
	todo!()
}

// waiting on https://github.com/paritytech/substrate/pull/9565
#[ignore]
#[test]
fn finished_ref_info_is_not_finished() {
	todo!()
}

#[test]
fn propose_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// Construct data to propose empty hash with value 100
			let input = EvmDataWriter::new()
				.write_selector(Action::Propose)
				.write(sp_core::H256::zero())
				.write(100u64)
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
					BalancesEvent::Reserved(Alice, 100).into(),
					DemocracyEvent::Proposed(0, 100).into(),
					EvmEvent::Executed(precompile_address()).into(),
				]
			);
		})
}

// Potential additional `propose` error cases:
// * propose_bad_length
// * proposing fails when you don't have enough funds to cover the deposit

#[test]
fn second_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// Before we can second anything, we have to have a proposal there to second.
			assert_ok!(Call::Democracy(DemocracyCall::propose(
				Default::default(), // Propose the default hash
				100u128,            // bond of 100 tokens
			))
			.dispatch(Origin::signed(Alice)));

			// Construct the call to second via a precompile
			let input = EvmDataWriter::new()
				.write_selector(Action::Second)
				.write(0u64) //prop index
				.write(100u64) // seconds upper bound
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

// Potential additional `second` error cases:
// * proposal doesn't exist
// * you can't afford it

#[test]
fn standard_vote_aye_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000_000)])
		.with_referenda(vec![(
			Default::default(),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			// Construct input data to vote aye
			let input = EvmDataWriter::new()
				.write_selector(Action::StandardVote)
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
					DemocracyEvent::Started(0, pallet_democracy::VoteThreshold::SimpleMajority)
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
		.with_referenda(vec![(
			Default::default(),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			// Construct input data to vote aye
			let input = EvmDataWriter::new()
				.write_selector(Action::StandardVote)
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
					DemocracyEvent::Started(0, pallet_democracy::VoteThreshold::SimpleMajority)
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

//Potential additional `standard_vote` error cases:
// * can't afford it
// * invalid conviction
// * referendum doesn't exist

#[test]
fn remove_vote_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.with_referenda(vec![(
			Default::default(),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			// Vote on it
			assert_ok!(Democracy::vote(
				Origin::signed(Alice),
				0, // Propose the default hash
				AccountVote::Standard {
					vote: Vote {
						aye: true,
						conviction: 0u8.try_into().unwrap()
					},
					balance: 100,
				},
			));

			// Construct input data to remove the vote
			let input = EvmDataWriter::new()
				.write_selector(Action::RemoveVote)
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
					DemocracyEvent::Started(0, pallet_democracy::VoteThreshold::SimpleMajority)
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

			// Wait until it becomes a referendum
			roll_to(<Test as DemocracyConfig>::LaunchPeriod::get());

			// Construct input data to remove a non-existant vote
			let input = EvmDataWriter::new()
				.write_selector(Action::RemoveVote)
				.write(0u32) // Referendum index 0
				.build();

			// Expected result is an error from the pallet
			if let Some(Err(ExitError::Other(e))) =
				Precompiles::execute(precompile_address(), &input, None, &evm_test_context())
			{
				assert!(e.contains("NotVoter"));
			} else {
				panic!("Expected an ExitError, but didn't get one.")
			}
		})
}

#[test]
fn delegate_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// Construct input data to delegate Alice -> Bob
			let input = EvmDataWriter::new()
				.write_selector(Action::Delegate)
				.write::<Address>(H160::from(Bob).into()) // Delegate to
				.write(2u8) // 2X conviction
				.write(100u128) // 100 tokens
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
					DemocracyEvent::Delegated(Alice, Bob).into(),
					EvmEvent::Executed(precompile_address()).into(),
				]
			);

			// Check that storage is correct
			assert_eq!(
				pallet_democracy::VotingOf::<Test>::get(Alice),
				Voting::Delegating {
					balance: 100,
					target: Bob,
					conviction: 2u8.try_into().unwrap(),
					delegations: Default::default(),
					prior: Default::default(),
				}
			);
			// Would be nice to check that it shows up for Bob too, but  can't because of
			// private fields. At elast I can see it works manually when uncommenting this.
			// assert_eq!(
			// 	pallet_democracy::VotingOf::<Test>::get(Bob),
			// 	Voting::Direct {
			// 		votes: Default::default(),
			// 		delegations: pallet_democracy::Delegations {
			// 			votes: 200, //because of 2x conviction
			// 			capital: 100,
			// 		},
			// 		prior: Default::default(),
			// 	}
			// );
		})
}

#[test]
fn undelegate_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// Before we can undelegate there has to be a delegation.
			// There's no a genesis config or helper function available, so I'll make one here.
			assert_ok!(Democracy::delegate(
				Origin::signed(Alice),
				Bob,
				1u8.try_into().unwrap(),
				100
			));

			// Construct input data to un-delegate Alice
			let input = EvmDataWriter::new()
				.write_selector(Action::UnDelegate)
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
					DemocracyEvent::Delegated(Alice, Bob).into(),
					DemocracyEvent::Undelegated(Alice).into(),
					EvmEvent::Executed(precompile_address()).into(),
				]
			);

			// Would be nice to check storage too, but I can't express PriorLock because
			// it is private.
			// assert_eq!(
			// 	pallet_democracy::VotingOf::<Test>::get(Alice),
			// 	Voting::Direct{
			// 		votes: Default::default(),
			// 		delegations: Default::default(),
			// 		prior: pallet_democracy::vote::PriorLock(11, 100),
			// 	}
			// );
		})
}

#[test]
fn undelegate_dne() {
	ExtBuilder::default().build().execute_with(|| {
		// Construct input data to un-delegate Alice
		let input = EvmDataWriter::new()
			.write_selector(Action::UnDelegate)
			.build();

		// Expected result is an error from the pallet
		if let Some(Err(ExitError::Other(e))) =
			Precompiles::execute(precompile_address(), &input, None, &evm_test_context())
		{
			assert!(e.contains("NotDelegating"));
		} else {
			panic!("Expected an ExitError, but didn't get one.")
		}
	})
}

#[test]
#[ignore]
fn unlock_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.with_referenda(vec![(
			Default::default(),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			// Alice votes to get some tokens locked
			assert_ok!(Democracy::vote(
				Origin::signed(Alice),
				0,
				AccountVote::Standard {
					vote: Vote {
						aye: true,
						conviction: 1u8.try_into().unwrap()
					},
					balance: 100,
				}
			));

			// Tokens are locked in `try_vote` when a vote is cast. Why is that not
			// reflected here?
			// https://github.com/paritytech/substrate/blob/master/frame/democracy/src/lib.rs#L1405
			// One possible way to look further: I just noticed there is a `Locks` storage item in
			// the pallet.
			// And also, maybe write a test in the pallet to ensure the locks work as expected.
			assert_eq!(
				<Balances as Currency<TestAccount>>::free_balance(&Alice),
				900
			);

			// Let time elapse until she wins the vote and gets her tokens locked
			roll_to(11);

			// Let time elapse until her tokens no longer need to be locked
			// NOTE: This is  bogus hash with no preimage, so no actual outcome
			// will be successfully dispatched. Nonetheless, she should still have her
			// tokens locked.
			roll_to(21);

			// Construct input data to un-lock tokens for Alice
			let input = EvmDataWriter::new()
				.write_selector(Action::Unlock)
				.write::<Address>(H160::from(Alice).into())
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
					DemocracyEvent::Started(0, pallet_democracy::VoteThreshold::SimpleMajority)
						.into(),
					DemocracyEvent::Passed(0).into(),
					EvmEvent::Executed(precompile_address()).into(),
				]
			);
		})
}

#[test]
fn unlock_with_nothing_locked() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)]) // So that she already has an account existing.
		.build()
		.execute_with(|| {
			// Construct input data to un-lock tokens for Alice
			let input = EvmDataWriter::new()
				.write_selector(Action::Unlock)
				.write::<Address>(H160::from(Alice).into())
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
				vec![EvmEvent::Executed(precompile_address()).into(),]
			);
		})
}
