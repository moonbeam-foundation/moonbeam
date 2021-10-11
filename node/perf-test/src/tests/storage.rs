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
	command::{FullBackend, FullClient, TestContext},
	tests::{TestResults, TestRunner},
};

use sc_service::NativeExecutionDispatch;
use sp_api::ConstructRuntimeApi;
use sp_core::U256;
use std::time::Instant;

use service::{Block, RuntimeApiCollection};
use sha3::{Digest, Keccak256};

const EXTRINSIC_GAS_LIMIT: u64 = 12_995_000;
const MIN_GAS_PRICE: u64 = 1_000_000_000;

pub struct StoragePerfTest {}

impl StoragePerfTest {
	pub fn new() -> Self {
		StoragePerfTest {}
	}
}

impl<RuntimeApi, Executor> TestRunner<RuntimeApi, Executor> for StoragePerfTest
where
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	fn run(
		&mut self,
		context: &TestContext<RuntimeApi, Executor>,
	) -> Result<Vec<TestResults>, String> {
		let mut results: Vec<TestResults> = Default::default();

		let alice = context.get_alice_details();
		let mut alice_nonce = alice.nonce;

		/*
		pragma solidity >=0.8.0;
		contract StorageBloater {
		  mapping(uint => uint) public bloat;
		  uint256 sum = 0;

		  function bloat_storage(uint start, uint num_items, uint seed) public {
			for (uint i=0; i<num_items; i++) {
			  bloat[start + i] = start + i * seed;
			}
		  }

		  function calculate_sum(uint num_items) public {
			uint256 tmp = 0;
			for (uint i=0; i<num_items; i++) {
			  tmp += bloat[i];
			}
			sum = tmp;
		  }
		}
		*/

		let calculate_bloat_storage_calldata = |start: u64, num_items: u64, seed: u64| {
			let start = U256::from(start);
			let num_items = U256::from(num_items);
			let seed = U256::from(seed);

			let mut bytes = Vec::<u8>::with_capacity(4 + 32 + 32 + 32);

			bytes.extend_from_slice(
				&Keccak256::digest(b"bloat_storage(uint256,uint256,uint256)")[0..4],
			);
			bytes.resize(4 + 32 + 32 + 32, 0);

			U256::from(start).to_big_endian(&mut bytes[4..36]);
			U256::from(num_items).to_big_endian(&mut bytes[36..68]);
			U256::from(seed).to_big_endian(&mut bytes[68..100]);

			bytes
		};

		let storage_bloater_hex =
			"6080604052600060015534801561001557600080fd5b506103b0806100256000396000f3fe608060405234801561001057600080fd5b50600436106100415760003560e01c806356f510a914610046578063b5ee966014610062578063fc757f9614610092575b600080fd5b610060600480360381019061005b91906101af565b6100ae565b005b61007c60048036038101906100779190610182565b61010a565b6040516100899190610211565b60405180910390f35b6100ac60048036038101906100a79190610182565b610122565b005b60005b828110156101045781816100c59190610282565b846100d0919061022c565b60008083876100df919061022c565b81526020019081526020016000208190555080806100fc906102e6565b9150506100b1565b50505050565b60006020528060005260406000206000915090505481565b6000805b8281101561016157600080828152602001908152602001600020548261014c919061022c565b91508080610159906102e6565b915050610126565b50806001819055505050565b60008135905061017c81610363565b92915050565b6000602082840312156101985761019761035e565b5b60006101a68482850161016d565b91505092915050565b6000806000606084860312156101c8576101c761035e565b5b60006101d68682870161016d565b93505060206101e78682870161016d565b92505060406101f88682870161016d565b9150509250925092565b61020b816102dc565b82525050565b60006020820190506102266000830184610202565b92915050565b6000610237826102dc565b9150610242836102dc565b9250827fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff038211156102775761027661032f565b5b828201905092915050565b600061028d826102dc565b9150610298836102dc565b9250817fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff04831182151516156102d1576102d061032f565b5b828202905092915050565b6000819050919050565b60006102f1826102dc565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8214156103245761032361032f565b5b600182019050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b600080fd5b61036c816102dc565b811461037757600080fd5b5056fea264697066735822122079d68b115cdfb621806dd5223907c9a47cff16ce5fd06dd2d561361179a25f2a64736f6c63430008070033";
		let storage_bloater_bytecode =
			hex::decode(storage_bloater_hex).expect("storage_bloater_hex is valid hex; qed");

		// precalculate contract address.
		// TODO: need so much DRY....
		let create_info = context
			.evm_create(
				alice.address,
				storage_bloater_bytecode.clone(),
				0.into(),
				EXTRINSIC_GAS_LIMIT.into(),
				Some(MIN_GAS_PRICE.into()),
				Some(alice_nonce),
			)
			.expect("EVM create failed while estimating contract address");
		let storage_bloater_address = create_info.value;
		log::debug!(
			"storage_bloater address expected to be {:?}",
			storage_bloater_address
		);

		log::trace!("Issuing EVM create txn for stoarge_bloater...");
		let _txn_hash = context
			.eth_sign_and_send_transaction(
				&alice.privkey,
				None,
				storage_bloater_bytecode,
				0.into(),
				EXTRINSIC_GAS_LIMIT.into(),
				MIN_GAS_PRICE.into(),
				alice_nonce,
			)
			.expect("EVM create failed while trying to deploy storage_bloater contract");

		alice_nonce = alice_nonce.saturating_add(1.into());

		let now = Instant::now();
		context.create_block(true);
		results.push(TestResults::new(
			"storage bloat contract creation",
			now.elapsed(),
		));

		// fill our storage contract with bloat
		let now = Instant::now();
		for i in 0..100 {
			// create calldata
			let calldata = calculate_bloat_storage_calldata(i * 500, 500, 0);

			let _txn_hash = context
				.eth_sign_and_send_transaction(
					&alice.privkey,
					Some(storage_bloater_address),
					calldata.clone(),
					0.into(),
					EXTRINSIC_GAS_LIMIT.into(),
					MIN_GAS_PRICE.into(),
					alice_nonce,
				)
				.expect("EVM create failed while trying to deploy storage_bloater contract");

			log::warn!(target: "storage_bloat", "creating block...");
			context.create_block(true);
			log::warn!(target: "storage_bloat", "done creating block...");
			alice_nonce = alice_nonce.saturating_add(1.into());
		}

		results.push(TestResults::new("storage bloating", now.elapsed()));

		// TODO: read storage

		Ok(results)
	}
}
