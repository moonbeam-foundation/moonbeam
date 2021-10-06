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

//! Moonbase Runtime Api Integration Tests

mod common;
use common::*;

use nimbus_primitives::NimbusId;
use pallet_evm::{Account as EVMAccount, AddressMapping, FeeCalculator, GenesisAccount};
use sp_core::{Public, H160, H256, U256};

use fp_rpc::runtime_decl_for_EthereumRuntimeRPCApi::EthereumRuntimeRPCApi;
use moonbeam_rpc_primitives_txpool::runtime_decl_for_TxPoolRuntimeApi::TxPoolRuntimeApi;
use std::collections::BTreeMap;
use std::str::FromStr;

#[test]
fn ethereum_runtime_rpc_api_chain_id() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Runtime::chain_id(), CHAIN_ID);
	});
}

#[test]
fn ethereum_runtime_rpc_api_account_basic() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 2_000 * UNIT)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Runtime::account_basic(H160::from(ALICE)),
				EVMAccount {
					balance: U256::from(2_000 * UNIT),
					nonce: U256::zero()
				}
			);
		});
}

#[test]
fn ethereum_runtime_rpc_api_gas_price() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Runtime::gas_price(), FixedGasPrice::min_gas_price());
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
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS),
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_nominations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * UNIT,
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			set_author(NimbusId::from_slice(&ALICE_NIMBUS));
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
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 2_000 * UNIT),
		])
		.build()
		.execute_with(|| {
			let execution_result = Runtime::call(
				H160::from(ALICE),  // from
				H160::from(BOB),    // to
				Vec::new(),         // data
				U256::from(1000),   // value
				U256::from(100000), // gas_limit
				None,               // gas_price
				None,               // nonce
				false,              // estimate
			);
			assert!(execution_result.is_ok());
		});
}

#[test]
fn ethereum_runtime_rpc_api_create() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 2_000 * UNIT)])
		.build()
		.execute_with(|| {
			let execution_result = Runtime::create(
				H160::from(ALICE),  // from
				vec![0, 1, 1, 0],   // data
				U256::zero(),       // value
				U256::from(100000), // gas_limit
				None,               // gas_price
				None,               // nonce
				false,              // estimate
			);
			assert!(execution_result.is_ok());
		});
}

#[test]
fn ethereum_runtime_rpc_api_current_transaction_statuses() {
	let alith = <Runtime as pallet_evm::Config>::AddressMapping::into_account_id(
		H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b")
			.expect("internal H160 is valid; qed"),
	);
	ExtBuilder::default()
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS),
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(alith, 2_000 * UNIT),
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_nominations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * UNIT,
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			set_author(NimbusId::from_slice(&ALICE_NIMBUS));
			let _result = Executive::apply_extrinsic(unchecked_eth_tx(VALID_ETH_TX));
			run_to_block(2);
			let statuses =
				Runtime::current_transaction_statuses().expect("Transaction statuses result.");
			assert_eq!(statuses.len(), 1);
		});
}

#[test]
fn ethereum_runtime_rpc_api_current_block() {
	ExtBuilder::default()
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS),
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_nominations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * UNIT,
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			set_author(NimbusId::from_slice(&ALICE_NIMBUS));
			run_to_block(2);
			let block = Runtime::current_block().expect("Block result.");
			assert_eq!(block.header.number, U256::from(1));
		});
}

#[test]
fn ethereum_runtime_rpc_api_current_receipts() {
	let alith = <Runtime as pallet_evm::Config>::AddressMapping::into_account_id(
		H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b")
			.expect("internal H160 is valid; qed"),
	);
	ExtBuilder::default()
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * UNIT)])
		.with_mappings(vec![(
			NimbusId::from_slice(&ALICE_NIMBUS),
			AccountId::from(ALICE),
		)])
		.with_balances(vec![
			(alith, 2_000 * UNIT),
			(AccountId::from(ALICE), 2_000 * UNIT),
			(AccountId::from(BOB), 1_000 * UNIT),
		])
		.with_nominations(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * UNIT,
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			set_author(NimbusId::from_slice(&ALICE_NIMBUS));
			let _result = Executive::apply_extrinsic(unchecked_eth_tx(VALID_ETH_TX));
			run_to_block(2);
			let receipts = Runtime::current_receipts().expect("Receipts result.");
			assert_eq!(receipts.len(), 1);
		});
}

#[test]
fn txpool_runtime_api_extrinsic_filter() {
	ExtBuilder::default().build().execute_with(|| {
		let non_eth_uxt = UncheckedExtrinsic::new_unsigned(
			pallet_balances::Call::<Runtime>::transfer(AccountId::from(BOB), 1 * UNIT).into(),
		);
		let eth_uxt = unchecked_eth_tx(VALID_ETH_TX);
		let txpool = <Runtime as TxPoolRuntimeApi<moonbase_runtime::Block>>::extrinsic_filter(
			vec![eth_uxt.clone(), non_eth_uxt.clone()],
			vec![unchecked_eth_tx(VALID_ETH_TX), non_eth_uxt],
		);
		assert_eq!(txpool.ready.len(), 1);
		assert_eq!(txpool.future.len(), 1);
	});
}

// Some Priority-related test ideas
// 1. Eth balance transfer with various gas prices. Priority == gas price
// 2. Eth contract call with various gas prices. Priority == gas price
// 3. System remark with no tip -> calculate expected priority from gas weight mapping
// 4. System remark with tip.
// 5. Operational dispatch has higher priority than normal for otherwise same transactions
