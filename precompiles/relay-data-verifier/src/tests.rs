// Copyright 2025 Moonbeam Foundation.
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
use crate::{mock::*, GetArrayLimit};
use parity_scale_codec::Encode;

use precompile_utils::{
	solidity::codec::{BoundedBytes, BoundedVec, UnboundedBytes},
	testing::*,
};
use sp_core::H256;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	check_precompile_implements_solidity_interfaces(
		&["RelayDataVerifier.sol"],
		PCall::supports_selector,
	)
}

#[test]
fn selectors() {
	assert!(PCall::verify_entry_selectors().contains(&0x27001faa));
	assert!(PCall::verify_entries_selectors().contains(&0x2da33a45));
	assert!(PCall::latest_relay_block_selectors().contains(&0xaed36869));
}

#[test]
fn modifiers() {
	ExtBuilder::default().build().execute_with(|| {
		let mut tester =
			PrecompilesModifierTester::new(PrecompilesValue::get(), Alice, Precompile1);

		tester.test_view_modifier(PCall::latest_relay_block_selectors());
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

// Test that the latest relay block number is returned correctly
#[test]
fn test_last_relay_block_retrieval() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			fill_relay_storage_roots::<Runtime>();
			set_current_relay_chain_state(250, H256::default());

			precompiles()
				.prepare_test(Alice, Precompile1, PCall::latest_relay_block {})
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(250u32);
		});
}

// Test that the latest_relay_block fails when no relay block is stored on chain yet
#[test]
fn test_last_relay_block_not_stored() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(Alice, Precompile1, PCall::latest_relay_block {})
				.expect_cost(0)
				.expect_no_logs()
				.execute_reverts(|output| output == b"No relay block found");
		});
}

// Test that verify_entry and verify_entries functions fail when the relay block number is not found
#[test]
fn test_block_not_found() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			fill_relay_storage_roots::<Runtime>();

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::verify_entry {
						relay_block_number: 4,
						proof: mocked_read_proof(),
						key: BoundedBytes::from(vec![0u8; 32]),
					},
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_reverts(|output| output == b"Block number not present");

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::verify_entries {
						relay_block_number: 4,
						proof: mocked_read_proof(),
						keys: BoundedVec::from(vec![BoundedBytes::from(vec![0u8; 32])]),
					},
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_reverts(|output| output == b"Block number not present");
		});
}

// Test that verify_entry and verify_entries functions fail when the root does not match
#[test]
fn test_root_mismatch() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			fill_relay_storage_roots::<Runtime>();
			let relay_block_number = 250;
			let state_root = H256::from_slice(
				&hex::decode("767caa877bcea0d34dd515a202b75efa41bffbc9f814ab59e2c1c96716d4c65e")
					.unwrap(),
			);
			set_current_relay_chain_state(relay_block_number, state_root);

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::verify_entry {
						relay_block_number,
						proof: mocked_read_proof(),
						key: BoundedBytes::from(vec![0u8; 32]),
					},
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_reverts(|output| output == b"Root Mismatch");

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::verify_entries {
						relay_block_number,
						proof: mocked_read_proof(),
						keys: BoundedVec::from(vec![BoundedBytes::from(vec![0u8; 32])]),
					},
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_reverts(|output| output == b"Root Mismatch");
		});
}

// Test that verify_entry and verify_entries functions fail when the entry is not found
#[test]
fn test_entry_not_found() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			fill_relay_storage_roots::<Runtime>();
			let relay_block_number = 250;
			set_current_relay_chain_state(relay_block_number, H256::from_slice(STORAGE_ROOT));

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::verify_entry {
						relay_block_number,
						proof: mocked_read_proof(),
						key: BoundedBytes::from(
							hex::decode(
								"89d139e01a5eb2256f222e5fc5dbe6b33c9c1284130706f5aea0c8b3d4c54d2c",
							)
							.unwrap(),
						),
					},
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_reverts(|output| output == b"Value is not present");

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::verify_entries {
						relay_block_number,
						proof: mocked_read_proof(),
						keys: BoundedVec::from(vec![
							BoundedBytes::from(TIMESTAMP_KEY),
							BoundedBytes::from(TOTAL_ISSUANCE_KEY),
							BoundedBytes::from(TREASURY_APPROVALS_KEY),
							// This key is not present in the proof
							BoundedBytes::from(hex::decode(
								"89d139e01a5eb2256f222e5fc5dbe6b33c9c1284130706f5aea0c8b3d4c54d89ec",
							).unwrap()),
						]),
					},
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_reverts(|output| output == b"Value is not present");
		});
}

// Test that verify_entry returns the correct value
#[test]
fn test_verify_entry() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			fill_relay_storage_roots::<Runtime>();
			let relay_block_number = 250;
			set_current_relay_chain_state(relay_block_number, H256::from_slice(STORAGE_ROOT));

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::verify_entry {
						relay_block_number,
						proof: mocked_read_proof(),
						key: BoundedBytes::from(TIMESTAMP_KEY),
					},
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(UnboundedBytes::from(1_708_190_328_000u64.encode()));
		});
}

// Test that verify_entries fails with an empty keys array
#[test]
fn test_verify_entries_empty_keys() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			fill_relay_storage_roots::<Runtime>();
			let relay_block_number = 250;
			set_current_relay_chain_state(relay_block_number, H256::from_slice(STORAGE_ROOT));

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::verify_entries {
						relay_block_number,
						proof: mocked_read_proof(),
						keys: BoundedVec::from(vec![]),
					},
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_reverts(|output| output == b"Keys must not be empty");
		});
}

// Test that verify_entries returns the correct values
#[test]
fn test_verify_entries() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			fill_relay_storage_roots::<Runtime>();
			let relay_block_number = 250;
			set_current_relay_chain_state(relay_block_number, H256::from_slice(STORAGE_ROOT));

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::verify_entries {
						relay_block_number,
						proof: mocked_read_proof(),
						keys: BoundedVec::from(vec![
							BoundedBytes::from(TIMESTAMP_KEY),
							BoundedBytes::from(TOTAL_ISSUANCE_KEY),
							BoundedBytes::from(TREASURY_APPROVALS_KEY),
						]),
					},
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(BoundedVec::<UnboundedBytes, GetArrayLimit>::from(vec![
					UnboundedBytes::from(1_708_190_328_000u64.encode()),
					UnboundedBytes::from(14_123_366_426_803_276_130u128.encode()),
					UnboundedBytes::from(
						vec![
							607, 608, 609, 610, 611, 612, 613, 614, 615, 616, 617, 618, 619, 620,
							621, 622, 623,
						]
						.encode(),
					),
				]));
		});
}
