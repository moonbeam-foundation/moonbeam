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
use crate::{mock::*, SELECTOR_LOG_SUBMITTED_AT};
use precompile_utils::{prelude::*, testing::*, EvmDataWriter};

use frame_support::{assert_ok, dispatch::Dispatchable};
use pallet_evm::{Call as EvmCall, Event as EvmEvent};

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
			let proposal = vec![1,2,3];
			let expected_hash = sp_runtime::traits::BlakeTwo256::hash(&proposal);

			let input = PCall::submit_at {
				track_id: 0u16,
				proposal: proposal.into(),
				block_number: 0u32,
			}
			.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			assert!(events().contains(
				&EvmEvent::Log {
					log: log2(
						Precompile1,
						SELECTOR_LOG_SUBMITTED_AT,
						H256::from_low_u64_be(0u64),
						EvmDataWriter::new()
							.write::<u32>(0u32)
							.write::<H256>(expected_hash.into())
							.build(),
					),
				}
				.into()
			));
		});
}
