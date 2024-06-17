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

//! Moonriver Runtime Api Integration Tests

mod common;
use common::*;

use fp_evm::{FeeCalculator, GenesisAccount};
use frame_support::assert_ok;
use nimbus_primitives::NimbusId;
use pallet_evm::{Account as EVMAccount, AddressMapping};
use sp_core::{ByteArray, H160, H256, U256};

use fp_rpc::runtime_decl_for_ethereum_runtime_rpc_api::EthereumRuntimeRPCApi;
use moonbeam_core_primitives::Header;
use moonbeam_rpc_primitives_txpool::runtime_decl_for_tx_pool_runtime_api::TxPoolRuntimeApi;
use moonriver_runtime::{Executive, TransactionPaymentAsGasPrice};
use nimbus_primitives::runtime_decl_for_nimbus_api::NimbusApi;
use std::{collections::BTreeMap, str::FromStr};

#[test]
fn ethereum_runtime_rpc_api_chain_id() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Runtime::chain_id(), CHAIN_ID);
	});
}

#[test]
fn ethereum_runtime_rpc_api_account_basic() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 2_000 * MOVR)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Runtime::account_basic(H160::from(ALICE)),
				EVMAccount {
					balance: U256::from(2_000 * MOVR),
					nonce: U256::zero()
				}
			);
		});
}

#[test]
fn ethereum_runtime_rpc_api_gas_price() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			Runtime::gas_price(),
			TransactionPaymentAsGasPrice::min_gas_price().0
		);
	});
}

#[test]
fn ethereum_runtime_rpc_api_account_code_at() {
	let address = H160::from(EVM_CONTRACT);
	let code: Vec<u8> = vec![1, 2, 3, 4, 5];
	ExtBuilder::default()
		.with_evm_accounts({
			let mut map = BTreeMap::new();
			map.insert(
				address,
				GenesisAccount {
					balance: U256::zero(),
					code: code.clone(),
					nonce: Default::default(),
					storage: Default::default(),
				},
			);
			map
		})
		.build()
		.execute_with(|| {
			assert_eq!(Runtime::account_code_at(address), code);
		});
}

#[test]
fn ethereum_runtime_rpc_api_author() {
	ExtBuilder::default()
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * MOVR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_delegations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * MOVR,
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			run_to_block(2, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));
			assert_eq!(Runtime::author(), H160::from(ALICE));
		});
}

#[test]
fn ethereum_runtime_rpc_api_storage_at() {
	let address = H160::from(EVM_CONTRACT);
	let mut key = [0u8; 32];
	key[31..32].copy_from_slice(&[6u8][..]);
	let mut value = [0u8; 32];
	value[31..32].copy_from_slice(&[7u8][..]);
	let item = H256::from_slice(&key[..]);
	let mut storage: BTreeMap<H256, H256> = BTreeMap::new();
	storage.insert(H256::from_slice(&key[..]), item);
	ExtBuilder::default()
		.with_evm_accounts({
			let mut map = BTreeMap::new();
			map.insert(
				address,
				GenesisAccount {
					balance: U256::zero(),
					code: Vec::new(),
					nonce: Default::default(),
					storage: storage.clone(),
				},
			);
			map
		})
		.build()
		.execute_with(|| {
			assert_eq!(Runtime::storage_at(address, U256::from(6)), item);
		});
}

#[test]
fn ethereum_runtime_rpc_api_call() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 2_000 * MOVR),
		])
		.build()
		.execute_with(|| {
			let execution_result = Runtime::call(
				H160::from(ALICE),     // from
				H160::from(BOB),       // to
				Vec::new(),            // data
				U256::from(1000u64),   // value
				U256::from(100000u64), // gas_limit
				None,                  // max_fee_per_gas
				None,                  // max_priority_fee_per_gas
				None,                  // nonce
				false,                 // estimate
				None,                  // access_list
			);
			assert!(execution_result.is_ok());
		});
}

#[test]
fn ethereum_runtime_rpc_api_create() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 2_000 * MOVR)])
		.build()
		.execute_with(|| {
			let execution_result = Runtime::create(
				H160::from(ALICE),     // from
				vec![0, 1, 1, 0],      // data
				U256::zero(),          // value
				U256::from(100000u64), // gas_limit
				None,                  // max_fee_per_gas
				None,                  // max_priority_fee_per_gas
				None,                  // nonce
				false,                 // estimate
				None,                  // access_list
			);
			assert!(execution_result.is_ok());
		});
}

#[test]
fn ethereum_runtime_rpc_api_current_transaction_statuses() {
	let alith = <Runtime as pallet_evm::Config>::AddressMapping::into_account_id(
		H160::from_str("f24ff3a9cf04c71dbc94d0b566f7a27b94566cac")
			.expect("internal H160 is valid; qed"),
	);
	ExtBuilder::default()
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * MOVR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(alith, 2_000 * MOVR),
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_delegations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * MOVR,
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			// set_author(NimbusId::from_slice(&ALICE_NIMBUS));
			let result =
				Executive::apply_extrinsic(unchecked_eth_tx(VALID_ETH_TX)).expect("Apply result.");
			assert_eq!(result, Ok(()));
			rpc_run_to_block(2);
			let statuses =
				Runtime::current_transaction_statuses().expect("Transaction statuses result.");
			assert_eq!(statuses.len(), 1);
		});
}

#[test]
fn ethereum_runtime_rpc_api_current_block() {
	ExtBuilder::default()
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * MOVR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_delegations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * MOVR,
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			// set_author(NimbusId::from_slice(&ALICE_NIMBUS));
			rpc_run_to_block(2);
			let block = Runtime::current_block().expect("Block result.");
			assert_eq!(block.header.number, U256::from(1u8));
		});
}

#[test]
fn ethereum_runtime_rpc_api_current_receipts() {
	let alith = <Runtime as pallet_evm::Config>::AddressMapping::into_account_id(
		H160::from_str("f24ff3a9cf04c71dbc94d0b566f7a27b94566cac")
			.expect("internal H160 is valid; qed"),
	);
	ExtBuilder::default()
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * MOVR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(alith, 2_000 * MOVR),
			(AccountId::from(ALICE), 2_000 * MOVR),
			(AccountId::from(BOB), 1_000 * MOVR),
		])
		.with_delegations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * MOVR,
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			// set_author(NimbusId::from_slice(&ALICE_NIMBUS));
			let result =
				Executive::apply_extrinsic(unchecked_eth_tx(VALID_ETH_TX)).expect("Apply result.");
			assert_eq!(result, Ok(()));
			rpc_run_to_block(2);
			let receipts = Runtime::current_receipts().expect("Receipts result.");
			assert_eq!(receipts.len(), 1);
		});
}

#[test]
fn txpool_runtime_api_extrinsic_filter() {
	ExtBuilder::default().build().execute_with(|| {
		let non_eth_uxt = UncheckedExtrinsic::new_unsigned(
			pallet_balances::Call::<Runtime>::transfer_allow_death {
				dest: AccountId::from(BOB),
				value: 1 * MOVR,
			}
			.into(),
		);
		let eth_uxt = unchecked_eth_tx(VALID_ETH_TX);
		let txpool = <Runtime as TxPoolRuntimeApi<moonriver_runtime::Block>>::extrinsic_filter(
			vec![eth_uxt.clone(), non_eth_uxt.clone()],
			vec![unchecked_eth_tx(VALID_ETH_TX), non_eth_uxt],
		);
		assert_eq!(txpool.ready.len(), 1);
		assert_eq!(txpool.future.len(), 1);
	});
}

#[test]
fn can_author_when_selected_is_empty() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 20_000_000 * MOVR),
			(AccountId::from(BOB), 10_000_000 * MOVR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 2_000_000 * MOVR)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
			AccountId::from(ALICE),
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			run_to_block(2, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));

			assert_eq!(ParachainStaking::candidate_pool().0.len(), 1);

			let slot_number = 0;
			let parent = Header {
				digest: Default::default(),
				extrinsics_root: Default::default(),
				number: Default::default(),
				parent_hash: Default::default(),
				state_root: Default::default(),
			};

			// Base case: ALICE can author blocks when she is the only candidate
			let can_author_block = Runtime::can_author(
				NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
				slot_number,
				&parent,
			);

			assert!(can_author_block);

			// Remove ALICE from candidate pool, leaving the candidate_pool empty
			assert_ok!(ParachainStaking::go_offline(origin_of(AccountId::from(
				ALICE
			))));

			// Need to fast forward to right before the next session, which is when selected candidates
			// will be updated. We want to test the creation of the first block of the next session.
			run_to_block(1799, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));

			assert_eq!(ParachainStaking::candidate_pool().0.len(), 0);

			let slot_number = 0;
			let parent = Header {
				digest: Default::default(),
				extrinsics_root: Default::default(),
				number: 1799,
				parent_hash: Default::default(),
				state_root: Default::default(),
			};

			let can_author_block = Runtime::can_author(
				NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
				slot_number,
				&parent,
			);

			assert!(can_author_block);

			// Check that it works as expected after session update
			run_to_block(1800, Some(NimbusId::from_slice(&ALICE_NIMBUS).unwrap()));

			assert_eq!(ParachainStaking::candidate_pool().0.len(), 0);

			let slot_number = 0;
			let parent = Header {
				digest: Default::default(),
				extrinsics_root: Default::default(),
				number: 1800,
				parent_hash: Default::default(),
				state_root: Default::default(),
			};

			let can_author_block = Runtime::can_author(
				NimbusId::from_slice(&ALICE_NIMBUS).unwrap(),
				slot_number,
				&parent,
			);

			assert!(can_author_block);
		});
}
