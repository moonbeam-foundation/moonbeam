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

use crate::{
	mock::{
		events, roll_to, set_balance_proposal, AccountId, Balances, Democracy, ExtBuilder, PCall,
		Precompiles, PrecompilesValue, Preimage, Runtime, RuntimeCall, RuntimeOrigin,
	},
	SELECTOR_LOG_DELEGATED, SELECTOR_LOG_PROPOSED, SELECTOR_LOG_SECONDED,
	SELECTOR_LOG_STANDARD_VOTE, SELECTOR_LOG_UNDELEGATED,
};
use frame_support::{
	assert_ok,
	dispatch::Dispatchable,
	traits::{Currency, PreimageProvider, QueryPreimage, StorePreimage},
};
use pallet_balances::Event as BalancesEvent;
use pallet_preimage::Event as PreimageEvent;

use pallet_democracy::{
	AccountVote, Call as DemocracyCall, Config as DemocracyConfig, Event as DemocracyEvent, Vote,
	VoteThreshold, Voting,
};
use pallet_evm::{Call as EvmCall, Event as EvmEvent};
use precompile_utils::{prelude::*, testing::*};
use sp_core::{H160, H256, U256};
use std::{convert::TryInto, str::from_utf8};

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
fn modifiers() {
	ExtBuilder::default().build().execute_with(|| {
		let mut tester = PrecompilesModifierTester::new(precompiles(), Alice, Precompile1);

		tester.test_default_modifier(PCall::delegate_selectors());
		tester.test_view_modifier(PCall::deposit_of_selectors());
		tester.test_view_modifier(PCall::finished_referendum_info_selectors());
		tester.test_view_modifier(PCall::lowest_unbaked_selectors());
		tester.test_view_modifier(PCall::ongoing_referendum_info_selectors());
		tester.test_default_modifier(PCall::propose_selectors());
		tester.test_view_modifier(PCall::public_prop_count_selectors());
		tester.test_default_modifier(PCall::remove_vote_selectors());
		tester.test_default_modifier(PCall::second_selectors());
		tester.test_default_modifier(PCall::standard_vote_selectors());
		tester.test_default_modifier(PCall::un_delegate_selectors());
		tester.test_default_modifier(PCall::unlock_selectors());
	});
}

#[test]
fn prop_count_zero() {
	ExtBuilder::default().build().execute_with(|| {
		// Assert that no props have been opened.
		precompiles()
			.prepare_test(Alice, Precompile1, PCall::public_prop_count {})
			.expect_cost(0) // TODO: Test db read/write costs
			.expect_no_logs()
			.execute_returns([0u8; 32].into())
	});
}

#[test]
fn prop_count_non_zero() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// There is no interesting genesis config for pallet democracy so we make the proposal here
			assert_ok!(RuntimeCall::Democracy(DemocracyCall::propose {
				proposal: set_balance_proposal(Charlie, 1000u128),
				value: 1000u128
			})
			.dispatch(RuntimeOrigin::signed(Alice.into())));

			precompiles()
				.prepare_test(Alice, Precompile1, PCall::public_prop_count {})
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
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// There is no interesting genesis config for pallet democracy so we make the proposal here
			assert_ok!(RuntimeCall::Democracy(DemocracyCall::propose {
				proposal: set_balance_proposal(Charlie, 1000u128),
				value: 1000u128
			})
			.dispatch(RuntimeOrigin::signed(Alice.into())));

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
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
				Precompile1,
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
			.prepare_test(Alice, Precompile1, PCall::lowest_unbaked {})
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
		.with_balances(vec![(Alice.into(), 1_000_000)])
		.with_referenda(vec![
			(
				set_balance_proposal(Bob, 1000u128),
				VoteThreshold::SimpleMajority,
				10,
			),
			(
				set_balance_proposal(Charlie, 1000u128),
				VoteThreshold::SimpleMajority,
				10,
			),
		])
		.build()
		.execute_with(|| {
			// To ensure the referendum passes, we need an Aye vote on it
			assert_ok!(RuntimeCall::Democracy(DemocracyCall::vote {
				ref_index: 0, // referendum 0
				vote: AccountVote::Standard {
					vote: Vote {
						aye: true,
						conviction: 0u8.try_into().unwrap()
					},
					balance: 100_000,
				}
			})
			.dispatch(RuntimeOrigin::signed(Alice.into())));

			let voting = match pallet_democracy::VotingOf::<Runtime>::get(AccountId::from(Alice)) {
				Voting::Direct {
					votes,
					delegations,
					prior,
				} => (votes.into_inner(), delegations, prior),
				_ => panic!("Votes are not direct"),
			};

			// Assert that the vote was recorded in storage
			assert_eq!(
				voting,
				(
					vec![(
						0,
						AccountVote::Standard {
							vote: Vote {
								aye: true,
								conviction: 0u8.try_into().unwrap()
							},
							balance: 100_000,
						}
					)],
					Default::default(),
					Default::default()
				)
			);

			// Run it through until it is baked
			roll_to(
				<Runtime as DemocracyConfig>::VotingPeriod::get()
					+ <Runtime as DemocracyConfig>::LaunchPeriod::get()
					+ 1000,
			);

			precompiles()
				.prepare_test(Alice, Precompile1, PCall::lowest_unbaked {})
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(1u32);
		});
}

#[test]
fn ongoing_ref_info_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000_000)])
		.with_referenda(vec![(
			set_balance_proposal(Charlie, 1000u128),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			assert_ok!(RuntimeCall::Democracy(DemocracyCall::vote {
				ref_index: 0, // referendum 0
				vote: AccountVote::Standard {
					vote: Vote {
						aye: true,
						conviction: 0u8.try_into().unwrap()
					},
					balance: 100_000,
				}
			})
			.dispatch(RuntimeOrigin::signed(Alice.into())));

			let hash = set_balance_proposal(Charlie, 1000u128).hash();
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::ongoing_referendum_info { ref_index: 0 },
				)
				.expect_no_logs()
				.execute_returns_encoded((
					U256::from(11),      // end
					hash,                // hash
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
		.with_balances(vec![(Alice.into(), 1000_000)])
		.with_referenda(vec![(
			set_balance_proposal(Charlie, 1000u128),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::ongoing_referendum_info { ref_index: 1 },
				)
				.expect_no_logs()
				.execute_reverts(|output| output == b"Unknown referendum");
		})
}

#[test]
fn ongoing_ref_info_is_not_ongoing() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000_000)])
		.with_referenda(vec![(
			set_balance_proposal(Charlie, 1000u128),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			assert_ok!(RuntimeCall::Democracy(DemocracyCall::vote {
				ref_index: 0, // referendum 0
				vote: AccountVote::Standard {
					vote: Vote {
						aye: true,
						conviction: 0u8.try_into().unwrap()
					},
					balance: 100_000,
				}
			})
			.dispatch(RuntimeOrigin::signed(Alice.into())));

			roll_to(12);

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::ongoing_referendum_info { ref_index: 0 },
				)
				.expect_no_logs()
				.execute_reverts(|output| output == b"Referendum is finished");
		})
}

#[test]
fn finished_ref_info_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000_000)])
		.with_referenda(vec![(
			set_balance_proposal(Charlie, 1000u128),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			assert_ok!(RuntimeCall::Democracy(DemocracyCall::vote {
				ref_index: 0, // referendum 0
				vote: AccountVote::Standard {
					vote: Vote {
						aye: true,
						conviction: 0u8.try_into().unwrap()
					},
					balance: 100_000,
				}
			})
			.dispatch(RuntimeOrigin::signed(Alice.into())));

			roll_to(12);

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::finished_referendum_info { ref_index: 0 },
				)
				.expect_no_logs()
				.execute_returns_encoded((true, U256::from(11)));
		})
}

#[test]
fn finished_ref_info_bad_index() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000_000)])
		.with_referenda(vec![(
			set_balance_proposal(Charlie, 1000u128),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::finished_referendum_info { ref_index: 1 },
				)
				.expect_no_logs()
				.execute_reverts(|output| output == b"Unknown referendum");
		})
}

#[test]
fn finished_ref_info_is_not_finished() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000_000)])
		.with_referenda(vec![(
			set_balance_proposal(Charlie, 1000u128),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::finished_referendum_info { ref_index: 0 },
				)
				.expect_no_logs()
				.execute_reverts(|output| output == b"Referendum is ongoing");
		})
}

#[test]
fn propose_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// Note first
			let hash = <<Runtime as pallet_democracy::Config>::Preimages as StorePreimage>::note(
				Default::default(),
			)
			.unwrap();

			// Construct data to propose empty hash with value 100
			let input = PCall::propose {
				proposal_hash: hash,
				value: 100.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					PreimageEvent::Noted { hash }.into(),
					BalancesEvent::Reserved {
						who: Alice.into(),
						amount: 100
					}
					.into(),
					DemocracyEvent::Proposed {
						proposal_index: 0,
						deposit: 100
					}
					.into(),
					EvmEvent::Log {
						log: log2(
							Precompile1,
							SELECTOR_LOG_PROPOSED,
							H256::zero(), // proposal index,
							EvmDataWriter::new().write::<U256>(100.into()).build(),
						)
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile1.into()
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
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// Before we can second anything, we have to have a proposal there to second.
			assert_ok!(RuntimeCall::Democracy(DemocracyCall::propose {
				proposal: set_balance_proposal(Charlie, 1000u128), // Propose the default hash
				value: 100u128,                                    // bond of 100 tokens
			})
			.dispatch(RuntimeOrigin::signed(Alice.into())));

			// Construct the call to second via a precompile
			let input = PCall::second {
				prop_index: 0.into(),
				seconds_upper_bound: 100.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					BalancesEvent::Reserved {
						who: Alice.into(),
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
						who: Alice.into(),
						amount: 100
					}
					.into(),
					DemocracyEvent::Seconded {
						seconder: Alice.into(),
						prop_index: 0
					}
					.into(),
					EvmEvent::Log {
						log: log2(
							Precompile1,
							SELECTOR_LOG_SECONDED,
							H256::zero(), // proposal index,
							EvmDataWriter::new()
								.write::<Address>(H160::from(Alice).into())
								.build(),
						)
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile1.into()
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
		.with_balances(vec![(Alice.into(), 1000_000)])
		.with_referenda(vec![(
			set_balance_proposal(Charlie, 1000u128),
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
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

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
						voter: Alice.into(),
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
					EvmEvent::Log {
						log: log2(
							Precompile1,
							SELECTOR_LOG_STANDARD_VOTE,
							H256::zero(), // referendum index,
							EvmDataWriter::new()
								.write::<Address>(H160::from(Alice).into())
								.write::<bool>(true)
								.write::<U256>(100000.into())
								.write::<u8>(0)
								.build(),
						),
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile1.into()
					}
					.into(),
				]
			);

			let voting = match pallet_democracy::VotingOf::<Runtime>::get(AccountId::from(Alice)) {
				Voting::Direct {
					votes,
					delegations,
					prior,
				} => (votes.into_inner(), delegations, prior),
				_ => panic!("Votes are not direct"),
			};

			// Assert that the vote was recorded in storage
			assert_eq!(
				voting,
				(
					vec![(
						0,
						AccountVote::Standard {
							vote: Vote {
								aye: true,
								conviction: 0u8.try_into().unwrap()
							},
							balance: 100_000,
						}
					)],
					Default::default(),
					Default::default()
				)
			);
		})
}

#[test]
fn standard_vote_nay_conviction_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000_000)])
		.with_referenda(vec![(
			set_balance_proposal(Charlie, 1000u128),
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
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

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
						voter: Alice.into(),
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
					EvmEvent::Log {
						log: log2(
							Precompile1,
							SELECTOR_LOG_STANDARD_VOTE,
							H256::zero(), // referendum index,
							EvmDataWriter::new()
								.write::<Address>(H160::from(Alice).into())
								.write::<bool>(false)
								.write::<U256>(100000.into())
								.write::<u8>(3)
								.build(),
						),
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile1.into()
					}
					.into(),
				]
			);

			let voting = match pallet_democracy::VotingOf::<Runtime>::get(AccountId::from(Alice)) {
				Voting::Direct {
					votes,
					delegations,
					prior,
				} => (votes.into_inner(), delegations, prior),
				_ => panic!("Votes are not direct"),
			};

			// Assert that the vote was recorded in storage
			// Should check ReferendumInfoOf too, but can't because of private fields etc
			assert_eq!(
				voting,
				(
					vec![(
						0,
						AccountVote::Standard {
							vote: Vote {
								aye: false,
								conviction: 3u8.try_into().unwrap()
							},
							balance: 100_000,
						}
					)],
					Default::default(),
					Default::default()
				)
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
		.with_balances(vec![(Alice.into(), 1000)])
		.with_referenda(vec![(
			set_balance_proposal(Charlie, 1000u128),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			// Vote on it
			assert_ok!(Democracy::vote(
				RuntimeOrigin::signed(Alice.into()),
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
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

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
						voter: Alice.into(),
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
						address: Precompile1.into()
					}
					.into(),
				]
			);

			let voting = match pallet_democracy::VotingOf::<Runtime>::get(AccountId::from(Alice)) {
				Voting::Direct {
					votes,
					delegations,
					prior,
				} => (votes.into_inner(), delegations, prior),
				_ => panic!("Votes are not direct"),
			};

			// Assert that the vote was recorded in storage
			// Should check ReferendumInfoOf too, but can't because of private fields etc
			assert_eq!(voting, (vec![], Default::default(), Default::default()));
		})
}

#[test]
fn remove_vote_dne() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// Before we can vote on anything, we have to have a referendum there to vote on.
			// This will be nicer after https://github.com/paritytech/substrate/pull/9484
			// Make a proposal
			assert_ok!(RuntimeCall::Democracy(DemocracyCall::propose {
				proposal: set_balance_proposal(Charlie, 1000u128), // Propose the default hash
				value: 100u128,                                    // bond of 100 tokens
			})
			.dispatch(RuntimeOrigin::signed(Alice.into())));

			// Wait until it becomes a referendum
			roll_to(<Runtime as DemocracyConfig>::LaunchPeriod::get());

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
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
		.with_balances(vec![(Alice.into(), 1000)])
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
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					DemocracyEvent::Delegated {
						who: Alice.into(),
						target: Bob.into()
					}
					.into(),
					EvmEvent::Log {
						log: log2(
							Precompile1,
							SELECTOR_LOG_DELEGATED,
							H160::from(Alice),
							EvmDataWriter::new()
								.write::<Address>(H160::from(Bob).into())
								.build(),
						),
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile1.into()
					}
					.into(),
				]
			);
			let alice_voting =
				match pallet_democracy::VotingOf::<Runtime>::get(AccountId::from(Alice)) {
					Voting::Delegating {
						balance,
						target,
						conviction,
						delegations,
						prior,
					} => (balance, target, conviction, delegations, prior),
					_ => panic!("Votes are not delegating"),
				};

			// Assert that the vote was recorded in storage
			// Should check ReferendumInfoOf too, but can't because of private fields etc
			assert_eq!(
				alice_voting,
				(
					100,
					Bob.into(),
					2u8.try_into().unwrap(),
					Default::default(),
					Default::default(),
				)
			);

			let bob_voting = match pallet_democracy::VotingOf::<Runtime>::get(AccountId::from(Bob))
			{
				Voting::Direct {
					votes,
					delegations,
					prior,
				} => (votes.into_inner(), delegations, prior),
				_ => panic!("Votes are not direct"),
			};

			// Assert that the vote was recorded in storage
			assert_eq!(
				bob_voting,
				(
					Default::default(),
					pallet_democracy::Delegations {
						votes: 200, //because of 2x conviction
						capital: 100,
					},
					Default::default()
				)
			);
		})
}

#[test]
fn undelegate_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// Before we can undelegate there has to be a delegation.
			// There's no a genesis config or helper function available, so I'll make one here.
			assert_ok!(Democracy::delegate(
				RuntimeOrigin::signed(Alice.into()),
				Bob.into(),
				1u8.try_into().unwrap(),
				100
			));

			// Construct input data to un-delegate Alice
			let input = PCall::un_delegate {}.into();

			// Make sure the call goes through successfully
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					DemocracyEvent::Delegated {
						who: Alice.into(),
						target: Bob.into()
					}
					.into(),
					DemocracyEvent::Undelegated {
						account: Alice.into()
					}
					.into(),
					EvmEvent::Log {
						log: log2(Precompile1, SELECTOR_LOG_UNDELEGATED, H160::from(Alice), [],),
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile1.into()
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
			.prepare_test(Alice, Precompile1, PCall::un_delegate {})
			.execute_reverts(|output| from_utf8(&output).unwrap().contains("NotDelegating"));
	})
}

#[test]
#[ignore]
fn unlock_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.with_referenda(vec![(
			set_balance_proposal(Charlie, 1000u128),
			VoteThreshold::SimpleMajority,
			10,
		)])
		.build()
		.execute_with(|| {
			// Alice votes to get some tokens locked
			assert_ok!(Democracy::vote(
				RuntimeOrigin::signed(Alice.into()),
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
				<Balances as Currency<AccountId>>::free_balance(&Alice.into()),
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
			let input = PCall::unlock {
				target: H160::from(Alice).into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

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
						address: Precompile1.into()
					}
					.into(),
				]
			);
		})
}

#[test]
fn unlock_with_nothing_locked() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)]) // So that she already has an account existing.
		.build()
		.execute_with(|| {
			// Construct input data to un-lock tokens for Alice
			let input = PCall::unlock {
				target: H160::from(Alice).into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![EvmEvent::Executed {
					address: Precompile1.into()
				}
				.into(),]
			);
		})
}

#[test]
fn note_preimage_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)]) // So she can afford the deposit
		.build()
		.execute_with(|| {
			// Construct our dummy proposal and associated data
			let dummy_preimage: Vec<u8> = vec![1, 2, 3, 4];
			let dummy_bytes = dummy_preimage.clone();
			let proposal_hash =
				<<Runtime as frame_system::Config>::Hashing as sp_runtime::traits::Hash>::hash(
					&dummy_preimage[..],
				);
			let expected_deposit = (crate::mock::ByteDeposit::get() as u128
				* (dummy_preimage.len() as u128))
				.saturating_add(crate::mock::BaseDeposit::get() as u128);

			// Construct input data to note preimage
			let input = PCall::note_preimage {
				encoded_proposal: dummy_bytes.into(),
			}
			.into();

			// Make sure the call goes through successfully
			assert_ok!(RuntimeCall::Evm(EvmCall::call {
				source: Alice.into(),
				target: Precompile1.into(),
				input,
				value: U256::zero(), // No value sent in EVM
				gas_limit: u64::max_value(),
				max_fee_per_gas: U256::zero(),
				max_priority_fee_per_gas: Some(U256::zero()),
				nonce: None, // Use the next nonce
				access_list: Vec::new(),
			})
			.dispatch(RuntimeOrigin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					BalancesEvent::Reserved {
						who: Alice.into(),
						amount: expected_deposit
					}
					.into(),
					PreimageEvent::Noted {
						hash: proposal_hash
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile1.into()
					}
					.into(),
				]
			);

			// Storage is private so we need to check information through traits
			let len = <<Runtime as pallet_democracy::Config>::Preimages as QueryPreimage>::len(
				&proposal_hash,
			)
			.unwrap();

			assert_eq!(len, dummy_preimage.len() as u32);

			let requested =
				<<Runtime as pallet_democracy::Config>::Preimages as QueryPreimage>::is_requested(
					&proposal_hash,
				);

			// preimage not requested yet
			assert!(!requested);

			let preimage =
				<Preimage as PreimageProvider<H256>>::get_preimage(&proposal_hash).unwrap();

			// preimage bytes are stored correctly
			assert_eq!(preimage, dummy_preimage);
		})
}

#[test]
fn note_preimage_works_with_real_data() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)]) // So she can afford the deposit
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
			let expected_deposit = (crate::mock::ByteDeposit::get() as u128
				* (dummy_preimage.len() as u128))
				.saturating_add(crate::mock::BaseDeposit::get() as u128);

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
			assert_ok!(RuntimeCall::Evm(EvmCall::call {
				source: Alice.into(),
				target: Precompile1.into(),
				input,
				value: U256::zero(), // No value sent in EVM
				gas_limit: u64::max_value(),
				max_fee_per_gas: U256::zero(),
				max_priority_fee_per_gas: Some(U256::zero()),
				nonce: None, // Use the next nonce
				access_list: Vec::new(),
			})
			.dispatch(RuntimeOrigin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					BalancesEvent::Reserved {
						who: Alice.into(),
						amount: expected_deposit
					}
					.into(),
					PreimageEvent::Noted {
						hash: proposal_hash
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile1.into()
					}
					.into(),
				]
			);

			// Storage is private so we need to check information through traits
			let len = <<Runtime as pallet_democracy::Config>::Preimages as QueryPreimage>::len(
				&proposal_hash,
			)
			.unwrap();

			assert_eq!(len, dummy_preimage.len() as u32);

			let requested =
				<<Runtime as pallet_democracy::Config>::Preimages as QueryPreimage>::is_requested(
					&proposal_hash,
				);

			// preimage not requested yet
			assert!(!requested,);

			let preimage =
				<Preimage as PreimageProvider<H256>>::get_preimage(&proposal_hash).unwrap();

			// preimage bytes are stored correctly
			assert_eq!(preimage, dummy_preimage);
		})
}

#[test]
fn cannot_note_duplicate_preimage() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)]) // So she can afford the deposit
		.build()
		.execute_with(|| {
			// Construct our dummy proposal and associated data
			let dummy_preimage: Vec<u8> = vec![1, 2, 3, 4];
			let dummy_bytes = dummy_preimage.clone();
			let proposal_hash =
				<<Runtime as frame_system::Config>::Hashing as sp_runtime::traits::Hash>::hash(
					&dummy_preimage[..],
				);
			let expected_deposit = (crate::mock::ByteDeposit::get() as u128
				* (dummy_preimage.len() as u128))
				.saturating_add(crate::mock::BaseDeposit::get() as u128);

			// Construct input data to note preimage
			let input: Vec<_> = PCall::note_preimage {
				encoded_proposal: dummy_bytes.into(),
			}
			.into();

			// First call should go successfully
			assert_ok!(RuntimeCall::Evm(EvmCall::call {
				source: Alice.into(),
				target: Precompile1.into(),
				input: input.clone(),
				value: U256::zero(), // No value sent in EVM
				gas_limit: u64::max_value(),
				max_fee_per_gas: U256::zero(),
				max_priority_fee_per_gas: Some(U256::zero()),
				nonce: None, // Use the next nonce
				access_list: Vec::new(),
			})
			.dispatch(RuntimeOrigin::root()));

			// Second call should fail because that preimage is already noted
			assert_ok!(RuntimeCall::Evm(EvmCall::call {
				source: Alice.into(),
				target: Precompile1.into(),
				input,
				value: U256::zero(), // No value sent in EVM
				gas_limit: u64::max_value(),
				max_fee_per_gas: U256::zero(),
				max_priority_fee_per_gas: Some(U256::zero()),
				nonce: None, // Use the next nonce
				access_list: Vec::new(),
			})
			.dispatch(RuntimeOrigin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![
					BalancesEvent::Reserved {
						who: Alice.into(),
						amount: expected_deposit
					}
					.into(),
					PreimageEvent::Noted {
						hash: proposal_hash
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile1.into()
					}
					.into(),
					EvmEvent::ExecutedFailed {
						address: Precompile1.into()
					}
					.into(),
				]
			);
		})
}

#[test]
fn cannot_note_imminent_preimage_before_it_is_actually_imminent() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
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
			assert_ok!(RuntimeCall::Evm(EvmCall::call {
				source: Alice.into(),
				target: Precompile1.into(),
				input,
				value: U256::zero(), // No value sent in EVM
				gas_limit: u64::max_value(),
				max_fee_per_gas: 0.into(),
				max_priority_fee_per_gas: Some(U256::zero()),
				nonce: None, // Use the next nonce
				access_list: Vec::new(),
			})
			.dispatch(RuntimeOrigin::root()));

			// Assert that the events are as expected
			assert_eq!(
				events(),
				vec![EvmEvent::ExecutedFailed {
					address: Precompile1.into()
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
