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

//! Randomness precompile unit tests
use crate::mock::*;
use pallet_randomness::{Event as RandomnessEvent, RandomnessResults, RequestType};
use precompile_utils::{assert_event_emitted, testing::*, EvmDataWriter};
use sp_core::{H160, H256, U256};

#[test]
fn test_selector_less_than_four_bytes_reverts() {
	ExtBuilder::default().build().execute_with(|| {
		PrecompilesValue::get()
			.prepare_test(Alice, Precompile1, vec![1u8, 2, 3])
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn test_unimplemented_selector_reverts() {
	ExtBuilder::default().build().execute_with(|| {
		PrecompilesValue::get()
			.prepare_test(Alice, Precompile1, vec![1u8, 2, 3, 4])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn selectors() {
	assert!(PCall::relay_epoch_index_selectors().contains(&0x81797566));
	assert!(PCall::required_deposit_selectors().contains(&0xfb7cfdd7));
	assert!(PCall::get_request_status_selectors().contains(&0xd8a4676f));
	assert!(PCall::get_request_selectors().contains(&0xc58343ef));
	assert!(PCall::request_local_randomness_selectors().contains(&0x9478430c));
	assert!(PCall::request_babe_randomness_selectors().contains(&0x33c14a63));
	assert!(PCall::fulfill_request_selectors().contains(&0x9a91eb0d));
	assert!(PCall::increase_request_fee_selectors().contains(&0xd0408a7f));
	assert!(PCall::purge_expired_request_selectors().contains(&0x1d26cbab));
}

#[test]
fn modifiers() {
	ExtBuilder::default().build().execute_with(|| {
		let mut tester =
			PrecompilesModifierTester::new(PrecompilesValue::get(), Alice, Precompile1);

		tester.test_view_modifier(PCall::relay_epoch_index_selectors());
		tester.test_view_modifier(PCall::required_deposit_selectors());
		tester.test_view_modifier(PCall::get_request_status_selectors());
		tester.test_view_modifier(PCall::get_request_selectors());
		tester.test_default_modifier(PCall::request_local_randomness_selectors());
		tester.test_default_modifier(PCall::request_babe_randomness_selectors());
		tester.test_default_modifier(PCall::fulfill_request_selectors());
		tester.test_default_modifier(PCall::purge_expired_request_selectors());
	});
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	for file in ["Randomness.sol"] {
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
fn relay_epoch_index_works() {
	ExtBuilder::default().build().execute_with(|| {
		pallet_evm::AccountCodes::<Runtime>::insert(H160::from(Alice), vec![10u8]);

		PrecompilesValue::get()
			.prepare_test(Alice, Precompile1, PCall::relay_epoch_index {})
			.execute_returns_encoded(1u64);
	})
}

#[test]
fn required_deposit_works() {
	ExtBuilder::default().build().execute_with(|| {
		pallet_evm::AccountCodes::<Runtime>::insert(H160::from(Alice), vec![10u8]);

		PrecompilesValue::get()
			.prepare_test(Alice, Precompile1, PCall::required_deposit {})
			.execute_returns_encoded(U256::from(10));
	})
}

#[test]
fn get_dne_request_status() {
	ExtBuilder::default().build().execute_with(|| {
		pallet_evm::AccountCodes::<Runtime>::insert(H160::from(Alice), vec![10u8]);

		PrecompilesValue::get()
			.prepare_test(
				Alice,
				Precompile1,
				PCall::get_request_status {
					request_id: 1.into(),
				},
			)
			.execute_returns_encoded(0u8);
	})
}

#[test]
fn get_pending_request_status() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			pallet_evm::AccountCodes::<Runtime>::insert(H160::from(Alice), vec![10u8]);

			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::request_babe_randomness {
						refund_address: precompile_utils::data::Address(H160::from(Bob)),
						fee: U256::one(),
						gas_limit: 100u64,
						salt: H256::default(),
						num_words: 1u8,
					},
				)
				.execute_returns([0u8; 32].into());

			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::get_request_status {
						request_id: 0.into(),
					},
				)
				.execute_returns_encoded(1u8);
		})
}

#[test]
fn get_ready_request_status() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			pallet_evm::AccountCodes::<Runtime>::insert(H160::from(Alice), vec![10u8]);
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::request_local_randomness {
						refund_address: precompile_utils::data::Address(H160::from(Bob)),
						fee: U256::one(),
						gas_limit: crate::fulfillment_overhead_gas_cost::<Runtime>(10u8) + 10u64,
						salt: H256::default(),
						num_words: 1u8,
						delay: 2.into(),
					},
				)
				.execute_returns([0u8; 32].into());
			// run to ready block
			System::set_block_number(3);
			// ready status
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::get_request_status {
						request_id: 0.into(),
					},
				)
				.execute_returns_encoded(2u8);
		})
}

#[test]
fn get_expired_request_status() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			pallet_evm::AccountCodes::<Runtime>::insert(H160::from(Alice), vec![10u8]);
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::request_local_randomness {
						refund_address: precompile_utils::data::Address(H160::from(Bob)),
						fee: U256::one(),
						gas_limit: crate::fulfillment_overhead_gas_cost::<Runtime>(10u8) + 10u64,
						salt: H256::default(),
						num_words: 1u8,
						delay: 2.into(),
					},
				)
				.execute_returns([0u8; 32].into());
			// run to expired block
			System::set_block_number(21);
			// ready status
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::get_request_status {
						request_id: 0.into(),
					},
				)
				.execute_returns_encoded(3u8);
		})
}

#[test]
fn get_request_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			pallet_evm::AccountCodes::<Runtime>::insert(H160::from(Alice), vec![10u8]);
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::request_local_randomness {
						refund_address: precompile_utils::data::Address(H160::from(Bob)),
						fee: U256::one(),
						gas_limit: 100u64,
						salt: H256::default(),
						num_words: 1u8,
						delay: 2.into(),
					},
				)
				.execute_returns([0u8; 32].into());
			// run to expired block
			System::set_block_number(21);
			// ready status
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::get_request {
						request_id: 0.into(),
					},
				)
				.execute_returns(
					EvmDataWriter::new()
						.write(U256::zero())
						.write(precompile_utils::data::Address(H160::from(Bob)))
						.write(precompile_utils::data::Address(H160::from(Alice)))
						.write(U256::one())
						.write(U256::from(100))
						.write(H256::default())
						.write(1u8)
						.write(0u8)
						.write(3u32)
						.write(0u64)
						.write(21u32)
						.write(0u64)
						.write(3u8)
						.build(),
				);
		})
}

#[test]
fn request_babe_randomness_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			pallet_evm::AccountCodes::<Runtime>::insert(H160::from(Alice), vec![10u8]);

			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::request_babe_randomness {
						refund_address: precompile_utils::data::Address(H160::from(Bob)),
						fee: U256::one(),
						gas_limit: 100u64,
						salt: H256::default(),
						num_words: 1u8,
					},
				)
				.execute_returns([0u8; 32].into());
			assert_event_emitted!(RuntimeEvent::Randomness(
				RandomnessEvent::RandomnessRequestedBabeEpoch {
					id: 0,
					refund_address: H160::from(Bob),
					contract_address: H160::from(Alice),
					fee: 1,
					gas_limit: 100u64,
					num_words: 1u8,
					salt: H256::default(),
					earliest_epoch: 3,
				}
			));
		})
}

#[test]
fn request_local_randomness_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			pallet_evm::AccountCodes::<Runtime>::insert(H160::from(Alice), vec![10u8]);

			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::request_local_randomness {
						refund_address: precompile_utils::data::Address(H160::from(Bob)),
						fee: U256::one(),
						gas_limit: 100u64,
						salt: H256::default(),
						num_words: 1u8,
						delay: 2.into(),
					},
				)
				.execute_returns([0u8; 32].into());
			assert_event_emitted!(RuntimeEvent::Randomness(
				RandomnessEvent::RandomnessRequestedLocal {
					id: 0,
					refund_address: H160::from(Bob),
					contract_address: H160::from(Alice),
					fee: 1,
					gas_limit: 100u64,
					num_words: 1u8,
					salt: H256::default(),
					earliest_block: 3,
				}
			));
		});
}

#[test]
fn fulfill_request_fails_when_gas_limit_below_call_overhead_cost() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			pallet_evm::AccountCodes::<Runtime>::insert(H160::from(Alice), vec![10u8]);
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::request_local_randomness {
						refund_address: precompile_utils::data::Address(H160::from(Bob)),
						fee: U256::one(),
						gas_limit: 100u64,
						salt: H256::default(),
						num_words: 1u8,
						delay: 2.into(),
					},
				)
				.execute_returns([0u8; 32].into());
			// run to ready block
			System::set_block_number(3);
			// fill randomness results
			let mut filled_results =
				RandomnessResults::<Runtime>::get(RequestType::Local(3)).unwrap();
			filled_results.randomness = Some(H256::default());
			RandomnessResults::<Runtime>::insert(RequestType::Local(3), filled_results);
			// fulfill request
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::fulfill_request {
						request_id: 0.into(),
					},
				)
				.expect_log(crate::log_fulfillment_failed(Alice));
		})
}

#[test]
fn fulfill_request_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			pallet_evm::AccountCodes::<Runtime>::insert(H160::from(Alice), vec![10u8]);
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::request_local_randomness {
						refund_address: precompile_utils::data::Address(H160::from(Bob)),
						fee: U256::one(),
						gas_limit: crate::fulfillment_overhead_gas_cost::<Runtime>(10u8) + 10u64,
						salt: H256::default(),
						num_words: 1u8,
						delay: 2.into(),
					},
				)
				.execute_returns([0u8; 32].into());
			// run to ready block
			System::set_block_number(3);
			// fill randomness results
			let mut filled_results =
				RandomnessResults::<Runtime>::get(RequestType::Local(3)).unwrap();
			filled_results.randomness = Some(H256::default());
			RandomnessResults::<Runtime>::insert(RequestType::Local(3), filled_results);
			// fulfill request
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::fulfill_request {
						request_id: 0.into(),
					},
				)
				.expect_log(crate::log_fulfillment_succeeded(Alice));
		})
}

#[test]
fn increase_request_fee_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			pallet_evm::AccountCodes::<Runtime>::insert(H160::from(Alice), vec![10u8]);
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::request_local_randomness {
						refund_address: precompile_utils::data::Address(H160::from(Bob)),
						fee: U256::one(),
						gas_limit: 100u64,
						salt: H256::default(),
						num_words: 1u8,
						delay: 2.into(),
					},
				)
				.execute_returns([0u8; 32].into());
			// increase request fee
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::increase_request_fee {
						request_id: 0.into(),
						fee_increase: 10.into(),
					},
				)
				.execute_returns(vec![]);
			assert_event_emitted!(RuntimeEvent::Randomness(
				RandomnessEvent::RequestFeeIncreased { id: 0, new_fee: 11 }
			));
		})
}

#[test]
fn purge_expired_request_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			pallet_evm::AccountCodes::<Runtime>::insert(H160::from(Alice), vec![10u8]);
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::request_local_randomness {
						refund_address: precompile_utils::data::Address(H160::from(Bob)),
						fee: U256::one(),
						gas_limit: 100u64,
						salt: H256::default(),
						num_words: 1u8,
						delay: 2.into(),
					},
				)
				.execute_returns([0u8; 32].into());
			System::set_block_number(21);
			// purge expired request
			PrecompilesValue::get()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::purge_expired_request {
						request_id: 0.into(),
					},
				)
				.execute_returns(vec![]);
			assert_event_emitted!(RuntimeEvent::Randomness(
				RandomnessEvent::RequestExpirationExecuted { id: 0 }
			));
		})
}
