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
use crate::{mock::*, SELECTOR_LOG_VOTE};
use precompile_utils::{prelude::*, testing::*, EvmDataWriter};

use frame_support::{assert_ok, dispatch::Dispatchable};
use pallet_evm::{Call as EvmCall, Event as EvmEvent};
use sp_core::{H160, H256, U256};

const ONGOING_POLL_INDEX: u32 = 3;

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
	for file in ["ConvictionVoting.sol"] {
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
fn vote_events_are_emited_on_success() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			// Vote Yes.
			let input = PCall::vote_yes {
				poll_index: ONGOING_POLL_INDEX,
				vote_amount: 100_000.into(),
				conviction: 0.into(),
			}
			.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Vote No.
			let input = PCall::vote_no {
				poll_index: ONGOING_POLL_INDEX,
				vote_amount: 99_000.into(),
				conviction: 1.into(),
			}
			.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Assert events are emitted.
			assert_eq!(
				events(),
				vec![
					// Vote yes event
					EvmEvent::Log {
						log: log2(
							Precompile1,
							SELECTOR_LOG_VOTE,
							H256::from_low_u64_be(ONGOING_POLL_INDEX as u64),
							EvmDataWriter::new()
								.write::<Address>(H160::from(Alice).into()) // caller
								.write::<bool>(true) // vote
								.write::<U256>(100_000.into()) // amount
								.write::<u8>(0) // conviction
								.build(),
						),
					}
					.into(),
					EvmEvent::Executed {
						address: Precompile1.into()
					}
					.into(),
					// Vote no event
					EvmEvent::Log {
						log: log2(
							Precompile1,
							SELECTOR_LOG_VOTE,
							H256::from_low_u64_be(ONGOING_POLL_INDEX as u64),
							EvmDataWriter::new()
								.write::<Address>(H160::from(Alice).into()) // caller
								.write::<bool>(false) // vote
								.write::<U256>(99_000.into()) // amount
								.write::<u8>(1) // conviction
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
		})
}
