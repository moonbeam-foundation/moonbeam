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
	mock::*, SELECTOR_LOG_DECISION_DEPOSIT_PLACED, SELECTOR_LOG_DECISION_DEPOSIT_REFUNDED,
	SELECTOR_LOG_SUBMISSION_DEPOSIT_REFUNDED, SELECTOR_LOG_SUBMITTED_AFTER,
	SELECTOR_LOG_SUBMITTED_AT,
};
use precompile_utils::{prelude::*, testing::*, EvmDataWriter};

use frame_support::{assert_ok, dispatch::Dispatchable};
use pallet_evm::{Call as EvmCall, Event as EvmEvent};
use pallet_referenda::Call as ReferendaCall;

use sp_core::{Hasher, H256, U256};

fn evm_call(input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: Alice.into(),
		target: Precompile1.into(),
		input,
		value: U256::zero(),
		gas_limit: u64::max_value(),
		max_fee_per_gas: 0.into(),
		max_priority_fee_per_gas: Some(U256::zero()),
		nonce: None,
		access_list: Vec::new(),
	}
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	for file in ["Referenda.sol"] {
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
fn submitted_at_logs_work() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			let proposal = vec![1, 2, 3];
			let proposal_hash = sp_runtime::traits::BlakeTwo256::hash(&proposal);

			// Submit referendum at index 0
			let input = PCall::submit_at {
				track_id: 0u16,
				proposal_hash: proposal_hash,
				proposal_len: proposal.len() as u32,
				block_number: 0u32,
			}
			.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Submit referendum at index 1
			let input = PCall::submit_at {
				track_id: 0u16,
				proposal_hash: proposal_hash,
				proposal_len: proposal.len() as u32,
				block_number: 0u32,
			}
			.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			assert!(vec![
				EvmEvent::Log {
					log: log2(
						Precompile1,
						SELECTOR_LOG_SUBMITTED_AT,
						H256::from_low_u64_be(0u64),
						EvmDataWriter::new()
							// Referendum index 0
							.write::<u32>(0u32)
							.write::<H256>(proposal_hash)
							.build(),
					),
				}
				.into(),
				EvmEvent::Log {
					log: log2(
						Precompile1,
						SELECTOR_LOG_SUBMITTED_AT,
						H256::from_low_u64_be(0u64),
						EvmDataWriter::new()
							// Referendum index 1
							.write::<u32>(1u32)
							.write::<H256>(proposal_hash)
							.build(),
					),
				}
				.into()
			]
			.iter()
			.all(|log| events().contains(log)));
		});
}

#[test]
fn submitted_after_logs_work() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			let proposal = vec![1, 2, 3];
			let proposal_hash = sp_runtime::traits::BlakeTwo256::hash(&proposal);

			// Submit referendum at index 0
			let input = PCall::submit_after {
				track_id: 0u16,
				proposal_hash: proposal_hash,
				proposal_len: proposal.len() as u32,
				block_number: 0u32,
			}
			.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Submit referendum at index 1
			let input = PCall::submit_after {
				track_id: 0u16,
				proposal_hash: proposal_hash,
				proposal_len: proposal.len() as u32,
				block_number: 0u32,
			}
			.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			assert!(vec![
				EvmEvent::Log {
					log: log2(
						Precompile1,
						SELECTOR_LOG_SUBMITTED_AFTER,
						H256::from_low_u64_be(0u64),
						EvmDataWriter::new()
							// Referendum index 0
							.write::<u32>(0u32)
							.write::<H256>(proposal_hash)
							.build(),
					),
				}
				.into(),
				EvmEvent::Log {
					log: log2(
						Precompile1,
						SELECTOR_LOG_SUBMITTED_AFTER,
						H256::from_low_u64_be(0u64),
						EvmDataWriter::new()
							// Referendum index 1
							.write::<u32>(1u32)
							.write::<H256>(proposal_hash)
							.build(),
					),
				}
				.into()
			]
			.iter()
			.all(|log| events().contains(log)));
		});
}

#[test]
fn place_and_refund_decision_deposit_logs_work() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			let proposal = vec![1, 2, 3];
			let proposal_hash = sp_runtime::traits::BlakeTwo256::hash(&proposal);
			let referendum_index = 0u32;

			// Create referendum
			let input = PCall::submit_at {
				track_id: 0u16,
				proposal_hash: proposal_hash,
				proposal_len: proposal.len() as u32,
				block_number: 0u32,
			}
			.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Place referendum decision deposit
			let input = PCall::place_decision_deposit {
				index: referendum_index,
			}
			.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Assert all place events are emitted
			assert!(vec![
				RuntimeEvent::Referenda(pallet_referenda::pallet::Event::DecisionDepositPlaced {
					index: referendum_index,
					who: Alice.into(),
					amount: 10
				}),
				EvmEvent::Log {
					log: log1(
						Precompile1,
						SELECTOR_LOG_DECISION_DEPOSIT_PLACED,
						EvmDataWriter::new()
							.write::<u32>(referendum_index)
							.write::<Address>(Address(Alice.into()))
							// Decision deposit
							.write::<U256>(U256::from(10))
							.build(),
					)
				}
				.into()
			]
			.iter()
			.all(|log| events().contains(log)));

			// Cancel referendum so we can refund
			assert_ok!(RuntimeCall::Referenda(ReferendaCall::cancel {
				index: referendum_index,
			})
			.dispatch(RuntimeOrigin::signed(Alice.into())));

			// Refund referendum decision deposit
			let input = PCall::refund_decision_deposit {
				index: referendum_index,
			}
			.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Refund referendum submission deposit.
			// Eligible because we cancelled the referendum.
			let input = PCall::refund_submission_deposit {
				index: referendum_index,
			}
			.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Assert all refund events are emitted
			assert!(vec![
				RuntimeEvent::Referenda(pallet_referenda::pallet::Event::DecisionDepositRefunded {
					index: referendum_index,
					who: Alice.into(),
					amount: 10
				}),
				RuntimeEvent::Referenda(
					pallet_referenda::pallet::Event::SubmissionDepositRefunded {
						index: referendum_index,
						who: Alice.into(),
						amount: 15
					}
				),
				EvmEvent::Log {
					log: log1(
						Precompile1,
						SELECTOR_LOG_DECISION_DEPOSIT_REFUNDED,
						EvmDataWriter::new()
							.write::<u32>(referendum_index)
							.write::<Address>(Address(Alice.into()))
							// Decision deposit
							.write::<U256>(U256::from(10))
							.build(),
					)
				}
				.into(),
				EvmEvent::Log {
					log: log1(
						Precompile1,
						SELECTOR_LOG_SUBMISSION_DEPOSIT_REFUNDED,
						EvmDataWriter::new()
							.write::<u32>(referendum_index)
							.write::<Address>(Address(Alice.into()))
							// Submission deposit
							.write::<U256>(U256::from(15))
							.build(),
					)
				}
				.into()
			]
			.iter()
			.all(|log| events().contains(log)));
		});
}
