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
	Account::{self, Alice, Bob, Precompile},
	Balances, Call, Democracy, ExtBuilder, Origin, PCall, Precompiles, PrecompilesValue, Runtime,
};
use frame_support::{assert_ok, dispatch::Dispatchable, traits::Currency};
use pallet_balances::Event as BalancesEvent;
use pallet_democracy::{
	AccountVote, Call as DemocracyCall, Config as DemocracyConfig, Event as DemocracyEvent,
	PreimageStatus, Vote, VoteThreshold, Voting,
};
use pallet_evm::{Call as EvmCall, Event as EvmEvent};
use precompile_utils::{solidity, testing::*};
use sp_core::{H160, H256, U256};
use std::{convert::TryInto, str::from_utf8};

fn precompiles() -> Precompiles<Runtime> {
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
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
		precompiles()
			.prepare_test(Alice, Precompile, vec![1u8, 2u8, 3u8])
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile, vec![1u8, 2u8, 3u8, 4u8])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn selectors() {
	assert!(PCall::delegate_selectors().contains(&0x0185921e));
	assert!(PCall::deposit_of_selectors().contains(&0x4767142d));
	assert!(PCall::finished_referendum_info_selectors().contains(&0xc75abcce));
	assert!(PCall::lowest_unbaked_selectors().contains(&0xd49dccf0));
	assert!(PCall::ongoing_referendum_info_selectors().contains(&0xf033b7cd));
	assert!(PCall::propose_selectors().contains(&0x7824e7d1));
	assert!(PCall::public_prop_count_selectors().contains(&0x31305462));
	assert!(PCall::remove_vote_selectors().contains(&0x3f68fde4));
	assert!(PCall::second_selectors().contains(&0xc7a76601));
	assert!(PCall::standard_vote_selectors().contains(&0x6cd18b0d));
	assert!(PCall::un_delegate_selectors().contains(&0x1eef225c));
	assert!(PCall::unlock_selectors().contains(&0x2f6c493c));

	// TODO also test logs once we have them
}

#[test]
fn prop_count_zero() {
	ExtBuilder::default().build().execute_with(|| {
		// Assert that no props have been opened.
		precompiles()
			.prepare_test(Alice, Precompile, PCall::public_prop_count {})
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns([0u8; 32].into())
	});
}

#[test]
fn prop_count_non_zero() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// There is no interesting genesis config for pallet democracy so we make the proposal here
			assert_ok!(Call::Democracy(DemocracyCall::propose {
				proposal_hash: Default::default(),
				value: 1000u128
			})
			.dispatch(Origin::signed(Alice)));

			precompiles()
				.prepare_test(Alice, Precompile, PCall::public_prop_count {})
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(1u32);
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
			assert_ok!(Call::Democracy(DemocracyCall::propose {
				proposal_hash: Default::default(),
				value: 1000u128
			})
			.dispatch(Origin::signed(Alice)));

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::deposit_of {
						prop_index: 0.into(),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(1000u32);
		});
}

#[test]
fn deposit_of_bad_index() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				PCall::deposit_of {
					prop_index: 10.into(),
				},
			)
			.execute_reverts(|output| output == b"No such proposal in pallet democracy");
	});
}

#[test]
fn lowest_unbaked_zero() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile, PCall::lowest_unbaked {})
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns_encoded(0u32);
	});
}

// This test is currently failing. I believe it is caused by a bug in the underlying pallet. I've
// asked about it in https://github.com/paritytech/substrate/issues/9739
#[ignore]
#[test]
fn lowest_unbaked_non_zero() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1_000_000)])
		.with_referenda(vec![
			(Default::default(), VoteThreshold::SimpleMajority, 10),
			(Default::default(), VoteThreshold::SimpleMajority, 10),
		])
		.build()
		.execute_with(|| {
			// To ensure the referendum passes, we need an Aye vote on it
			assert_ok!(Call::Democracy(DemocracyCall::vote {
				ref_index: 0, // referendum 0
				vote: AccountVote::Standard {
					vote: Vote {
						aye: true,
						conviction: 0u8.try_into().unwrap()
					},
					balance: 100_000,
				}
			})
			.dispatch(Origin::signed(Alice)));

			// Assert that the vote was recorded in storage
			assert_eq!(
				pallet_democracy::VotingOf::<Runtime>::get(Alice),
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
				<Runtime as DemocracyConfig>::VotingPeriod::get()
					+ <Runtime as DemocracyConfig>::LaunchPeriod::get()
					+ 1000,
			);

			precompiles()
				.prepare_test(Alice, Precompile, PCall::lowest_unbaked {})
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(1u32);
		});
}

#[test]
fn ongoing_ref_info_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000_000)])
		.with_referenda(vec![(
			Default::default(),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Democracy(DemocracyCall::vote {
				ref_index: 0, // referendum 0
				vote: AccountVote::Standard {
					vote: Vote {
						aye: true,
						conviction: 0u8.try_into().unwrap()
					},
					balance: 100_000,
				}
			})
			.dispatch(Origin::signed(Alice)));

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::ongoing_referendum_info { ref_index: 0 },
				)
				.expect_no_logs()
				.execute_returns_encoded((
					U256::from(11),      // end
					H256::zero(),        // hash
					2u8,                 // threshold type
					U256::from(10),      // delay
					U256::from(10_000),  // tally ayes
					U256::zero(),        // tally nays
					U256::from(100_000), // turnout
				));
		})
}

#[test]
fn ongoing_ref_info_bad_index() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000_000)])
		.with_referenda(vec![(
			Default::default(),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::ongoing_referendum_info { ref_index: 1 },
				)
				.expect_no_logs()
				.execute_reverts(|output| output == b"Unknown referendum");
		})
}

#[test]
fn ongoing_ref_info_is_not_ongoing() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000_000)])
		.with_referenda(vec![(
			Default::default(),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Democracy(DemocracyCall::vote {
				ref_index: 0, // referendum 0
				vote: AccountVote::Standard {
					vote: Vote {
						aye: true,
						conviction: 0u8.try_into().unwrap()
					},
					balance: 100_000,
				}
			})
			.dispatch(Origin::signed(Alice)));

			roll_to(12);

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::ongoing_referendum_info { ref_index: 0 },
				)
				.expect_no_logs()
				.execute_reverts(|output| output == b"Referendum is finished");
		})
}

#[test]
fn finished_ref_info_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000_000)])
		.with_referenda(vec![(
			Default::default(),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Democracy(DemocracyCall::vote {
				ref_index: 0, // referendum 0
				vote: AccountVote::Standard {
					vote: Vote {
						aye: true,
						conviction: 0u8.try_into().unwrap()
					},
					balance: 100_000,
				}
			})
			.dispatch(Origin::signed(Alice)));

			roll_to(12);

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::finished_referendum_info { ref_index: 0 },
				)
				.expect_no_logs()
				.execute_returns_encoded((true, U256::from(11)));
		})
}

#[test]
fn finished_ref_info_bad_index() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000_000)])
		.with_referenda(vec![(
			Default::default(),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::finished_referendum_info { ref_index: 1 },
				)
				.expect_no_logs()
				.execute_reverts(|output| output == b"Unknown referendum");
		})
}

#[test]
fn finished_ref_info_is_not_finished() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000_000)])
		.with_referenda(vec![(
			Default::default(),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::finished_referendum_info { ref_index: 0 },
				)
				.expect_no_logs()
				.execute_reverts(|output| output == b"Referendum is ongoing");
		})
}

#[test]
fn propose_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// Construct data to propose empty hash with value 100
			let input = PCall::propose {
				proposal_hash: H256::zero(),
				value: 100.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(input)).dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					BalancesEvent::Reserved {
						who: Alice,
						amount: 100
					}
					.into(),
					DemocracyEvent::Proposed {
						proposal_index: 0,
						deposit: 100
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile.into()
					}
					.into(),
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
			assert_ok!(Call::Democracy(DemocracyCall::propose {
				proposal_hash: Default::default(), // Propose the default hash
				value: 100u128,                    // bond of 100 tokens
			})
			.dispatch(Origin::signed(Alice)));

			// Construct the call to second via a precompile
			let input = PCall::second {
				prop_index: 0.into(),
				seconds_upper_bound: 100.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(input)).dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					BalancesEvent::Reserved {
						who: Alice,
						amount: 100
					}
					.into(),
					DemocracyEvent::Proposed {
						proposal_index: 0,
						deposit: 100
					}
					.into(),
					// This 100 is reserved for the second.
					// Pallet democracy does not have an event for seconding
					BalancesEvent::Reserved {
						who: Alice,
						amount: 100
					}
					.into(),
					DemocracyEvent::Seconded {
						seconder: Alice,
						prop_index: 0
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile.into()
					}
					.into(),
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
			let input = PCall::standard_vote {
				ref_index: 0.into(),
				aye: true,
				vote_amount: 100_000.into(),
				conviction: 0.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(input)).dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					DemocracyEvent::Started {
						ref_index: 0,
						threshold: pallet_democracy::VoteThreshold::SimpleMajority
					}
					.into(),
					DemocracyEvent::Voted {
						voter: Alice,
						ref_index: 0,
						vote: AccountVote::Standard {
							vote: Vote {
								aye: true,
								conviction: 0u8.try_into().unwrap()
							},
							balance: 100000
						}
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile.into()
					}
					.into(),
				]
			);

			// Assert that the vote was recorded in storage
			// Should check ReferendumInfoOf too, but can't because of private fields etc
			assert_eq!(
				pallet_democracy::VotingOf::<Runtime>::get(Alice),
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
			let input = PCall::standard_vote {
				ref_index: 0.into(),
				aye: false,
				vote_amount: 100_000.into(),
				conviction: 3.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(input)).dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					DemocracyEvent::Started {
						ref_index: 0,
						threshold: pallet_democracy::VoteThreshold::SimpleMajority
					}
					.into(),
					DemocracyEvent::Voted {
						voter: Alice,
						ref_index: 0,
						vote: AccountVote::Standard {
							vote: Vote {
								aye: false,
								conviction: 3u8.try_into().unwrap()
							},
							balance: 100000
						}
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile.into()
					}
					.into(),
				]
			);

			// Assert that the vote was recorded in storage
			// Should check ReferendumInfoOf too, but can't because of private fields etc
			assert_eq!(
				pallet_democracy::VotingOf::<Runtime>::get(Alice),
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

			let input = PCall::remove_vote {
				ref_index: 0.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(input)).dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					DemocracyEvent::Started {
						ref_index: 0,
						threshold: pallet_democracy::VoteThreshold::SimpleMajority
					}
					.into(),
					DemocracyEvent::Voted {
						voter: Alice,
						ref_index: 0,
						vote: AccountVote::Standard {
							vote: Vote {
								aye: true,
								conviction: 0u8.try_into().unwrap()
							},
							balance: 100
						}
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile.into()
					}
					.into(),
				]
			);

			// Assert that the vote was recorded in storage
			// Should check ReferendumInfoOf too, but can't because of private fields etc
			assert_eq!(
				pallet_democracy::VotingOf::<Runtime>::get(Alice),
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
			assert_ok!(Call::Democracy(DemocracyCall::propose {
				proposal_hash: Default::default(), // Propose the default hash
				value: 100u128,                    // bond of 100 tokens
			})
			.dispatch(Origin::signed(Alice)));

			// Wait until it becomes a referendum
			roll_to(<Runtime as DemocracyConfig>::LaunchPeriod::get());

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					PCall::remove_vote {
						ref_index: 0.into(),
					},
				)
				.execute_reverts(|output| from_utf8(&output).unwrap().contains("NotVoter"));
		})
}

#[test]
fn delegate_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// Construct input data to delegate Alice -> Bob
			let input = PCall::delegate {
				representative: H160::from(Bob).into(),
				conviction: 2.into(),
				amount: 100.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(input)).dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					DemocracyEvent::Delegated {
						who: Alice,
						target: Bob
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile.into()
					}
					.into(),
				]
			);

			// Check that storage is correct
			assert_eq!(
				pallet_democracy::VotingOf::<Runtime>::get(Alice),
				Voting::Delegating {
					balance: 100,
					target: Bob,
					conviction: 2u8.try_into().unwrap(),
					delegations: Default::default(),
					prior: Default::default(),
				}
			);

			assert_eq!(
				pallet_democracy::VotingOf::<Runtime>::get(Bob),
				Voting::Direct {
					votes: Default::default(),
					delegations: pallet_democracy::Delegations {
						votes: 200, //because of 2x conviction
						capital: 100,
					},
					prior: Default::default(),
				}
			);
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
			let input = PCall::un_delegate {}.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(input)).dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					DemocracyEvent::Delegated {
						who: Alice,
						target: Bob
					}
					.into(),
					DemocracyEvent::Undelegated { account: Alice }.into(),
					EvmEvent::Executed {
						address: Precompile.into()
					}
					.into(),
				]
			);

			// Would be nice to check storage too, but I can't express PriorLock because
			// it is private.
			// assert_eq!(
			// 	pallet_democracy::VotingOf::<Runtime>::get(Alice),
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
		precompiles()
			.prepare_test(Alice, Precompile, PCall::un_delegate {})
			.execute_reverts(|output| from_utf8(&output).unwrap().contains("NotDelegating"));
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
			assert_eq!(<Balances as Currency<Account>>::free_balance(&Alice), 900);

			// Let time elapse until she wins the vote and gets her tokens locked
			roll_to(11);

			// Let time elapse until her tokens no longer need to be locked
			// NOTE: This is  bogus hash with no preimage, so no actual outcome
			// will be successfully dispatched. Nonetheless, she should still have her
			// tokens locked.
			roll_to(21);

			// Construct input data to un-lock tokens for Alice
			let input = PCall::unlock {
				target: H160::from(Alice).into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(input)).dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					DemocracyEvent::Started {
						ref_index: 0,
						threshold: pallet_democracy::VoteThreshold::SimpleMajority
					}
					.into(),
					DemocracyEvent::Passed { ref_index: 0 }.into(),
					EvmEvent::Executed {
						address: Precompile.into()
					}
					.into(),
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
			let input = PCall::unlock {
				target: H160::from(Alice).into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(evm_call(input)).dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![EvmEvent::Executed {
					address: Precompile.into()
				}
				.into(),]
			);
		})
}

#[test]
fn note_preimage_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)]) // So she can afford the deposit
		.build()
		.execute_with(|| {
			// Construct our dummy proposal and associated data
			let dummy_preimage: Vec<u8> = vec![1, 2, 3, 4];
			let dummy_bytes = dummy_preimage.clone();
			let proposal_hash =
				<<Runtime as frame_system::Config>::Hashing as sp_runtime::traits::Hash>::hash(
					&dummy_preimage[..],
				);
			let expected_deposit =
				crate::mock::PreimageByteDeposit::get() * (dummy_preimage.len() as u128);

			// Construct input data to note preimage
			let input = PCall::note_preimage {
				encoded_proposal: dummy_bytes.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call {
				source: Alice.into(),
				target: Precompile.into(),
				input,
				value: U256::zero(), // No value sent in EVM
				gas_limit: u64::max_value(),
				max_fee_per_gas: U256::zero(),
				max_priority_fee_per_gas: Some(U256::zero()),
				nonce: None, // Use the next nonce
				access_list: Vec::new(),
			})
			.dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					BalancesEvent::Reserved {
						who: Alice,
						amount: expected_deposit
					}
					.into(),
					DemocracyEvent::PreimageNoted {
						proposal_hash,
						who: Alice,
						deposit: expected_deposit
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile.into()
					}
					.into(),
				]
			);

			// Check storage to make sure the data is actually stored there.
			// There is no `Eq` implementation, so we check the data individually
			if let PreimageStatus::Available {
				data,
				provider,
				deposit,
				expiry,
				..
			} = pallet_democracy::Preimages::<Runtime>::get(proposal_hash).unwrap()
			{
				assert_eq!(data, dummy_preimage);
				assert_eq!(provider, Alice);
				assert_eq!(deposit, 40u128);
				assert_eq!(expiry, None);
			} else {
				panic!("Expected preimage status to be available");
			}
		})
}

#[test]
fn note_preimage_works_with_real_data() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)]) // So she can afford the deposit
		.build()
		.execute_with(|| {
			// Construct our dummy proposal and associated data
			let dummy_preimage: Vec<u8> =
				hex_literal::hex!("0c026be02d1d3665660d22ff9624b7be0551ee1ac91b").to_vec();
			let dummy_bytes = dummy_preimage.clone();
			let proposal_hash =
				<<Runtime as frame_system::Config>::Hashing as sp_runtime::traits::Hash>::hash(
					&dummy_preimage[..],
				);
			let expected_deposit =
				crate::mock::PreimageByteDeposit::get() * (dummy_preimage.len() as u128);

			// Assert that the hash is as expected from TS tests
			assert_eq!(
				proposal_hash,
				sp_core::H256::from(hex_literal::hex!(
					"e435886138904e20b9d834d5c30b51693e5e53cc97f6d6da5908f1e41468bebf"
				))
			);

			// Construct input data to note preimage
			let input = PCall::note_preimage {
				encoded_proposal: dummy_bytes.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call {
				source: Alice.into(),
				target: Precompile.into(),
				input,
				value: U256::zero(), // No value sent in EVM
				gas_limit: u64::max_value(),
				max_fee_per_gas: U256::zero(),
				max_priority_fee_per_gas: Some(U256::zero()),
				nonce: None, // Use the next nonce
				access_list: Vec::new(),
			})
			.dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					BalancesEvent::Reserved {
						who: Alice,
						amount: expected_deposit
					}
					.into(),
					DemocracyEvent::PreimageNoted {
						proposal_hash,
						who: Alice,
						deposit: expected_deposit
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile.into()
					}
					.into(),
				]
			);

			// Check storage to make sure the data is actually stored there.
			// There is no `Eq` implementation, so we check the data individually
			if let PreimageStatus::Available {
				data,
				provider,
				deposit,
				expiry,
				..
			} = pallet_democracy::Preimages::<Runtime>::get(proposal_hash).unwrap()
			{
				assert_eq!(data, dummy_preimage);
				assert_eq!(provider, Alice);
				assert_eq!(deposit, (10 * dummy_preimage.len()) as u128);
				assert_eq!(expiry, None);
			} else {
				panic!("Expected preimage status to be available");
			}
		})
}

#[test]
fn cannot_note_duplicate_preimage() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)]) // So she can afford the deposit
		.build()
		.execute_with(|| {
			// Construct our dummy proposal and associated data
			let dummy_preimage: Vec<u8> = vec![1, 2, 3, 4];
			let dummy_bytes = dummy_preimage.clone();
			let proposal_hash =
				<<Runtime as frame_system::Config>::Hashing as sp_runtime::traits::Hash>::hash(
					&dummy_preimage[..],
				);
			let expected_deposit =
				crate::mock::PreimageByteDeposit::get() * (dummy_preimage.len() as u128);

			// Construct input data to note preimage
			let input: Vec<_> = PCall::note_preimage {
				encoded_proposal: dummy_bytes.into(),
			}
			.into();

			// First call should go successfully
			assert_ok!(Call::Evm(EvmCall::call {
				source: Alice.into(),
				target: Precompile.into(),
				input: input.clone(),
				value: U256::zero(), // No value sent in EVM
				gas_limit: u64::max_value(),
				max_fee_per_gas: U256::zero(),
				max_priority_fee_per_gas: Some(U256::zero()),
				nonce: None, // Use the next nonce
				access_list: Vec::new(),
			})
			.dispatch(Origin::root()));

			// Second call should fail because that preimage is already noted
			assert_ok!(Call::Evm(EvmCall::call {
				source: Alice.into(),
				target: Precompile.into(),
				input,
				value: U256::zero(), // No value sent in EVM
				gas_limit: u64::max_value(),
				max_fee_per_gas: U256::zero(),
				max_priority_fee_per_gas: Some(U256::zero()),
				nonce: None, // Use the next nonce
				access_list: Vec::new(),
			})
			.dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					BalancesEvent::Reserved {
						who: Alice,
						amount: expected_deposit
					}
					.into(),
					DemocracyEvent::PreimageNoted {
						proposal_hash,
						who: Alice,
						deposit: expected_deposit
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile.into()
					}
					.into(),
					EvmEvent::ExecutedFailed {
						address: Precompile.into()
					}
					.into(),
				]
			);
		})
}

#[test]
fn cannot_note_imminent_preimage_before_it_is_actually_imminent() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// Construct our dummy proposal and associated data
			let dummy_preimage: Vec<u8> = vec![1, 2, 3, 4];
			let dummy_bytes = dummy_preimage.clone();

			// Construct input data to note preimage
			let input = PCall::note_imminent_preimage {
				encoded_proposal: dummy_bytes.into(),
			}
			.into();

			// This call should not succeed because
			assert_ok!(Call::Evm(EvmCall::call {
				source: Alice.into(),
				target: Precompile.into(),
				input,
				value: U256::zero(), // No value sent in EVM
				gas_limit: u64::max_value(),
				max_fee_per_gas: 0.into(),
				max_priority_fee_per_gas: Some(U256::zero()),
				nonce: None, // Use the next nonce
				access_list: Vec::new(),
			})
			.dispatch(Origin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![EvmEvent::ExecutedFailed {
					address: Precompile.into()
				}
				.into()]
			);
		})
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	for file in ["DemocracyInterface.sol"] {
		for solidity_fn in solidity::get_selectors(file) {
			assert_eq!(
				solidity_fn.compute_selector_hex(),
				solidity_fn.docs_selector,
				"documented selector for '{}' did not match for file '{}'",
				solidity_fn.signature(),
				file,
			);

			let selector = solidity_fn.compute_selector();
			if !PCall::supports_selector(selector) {
				panic!(
					"failed decoding selector 0x{:x} => '{}' as Action for file '{}'",
					selector,
					solidity_fn.signature(),
					file,
				)
			}
		}
	}
}

#[test]
fn test_deprecated_solidity_selectors_are_supported() {
	for deprecated_function in [
		"public_prop_count()",
		"deposit_of(uint256)",
		"lowest_unbaked()",
		"standard_vote(uint256,bool,uint256,uint256)",
		"remove_vote(uint256)",
		"un_delegate()",
		"note_preimage(bytes)",
		"note_imminent_preimage(bytes)",
	] {
		let selector = solidity::compute_selector(deprecated_function);
		if !PCall::supports_selector(selector) {
			panic!(
				"failed decoding selector 0x{:x} => '{}' as Action",
				selector, deprecated_function,
			)
		}
	}
}
