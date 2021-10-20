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

// source code for this contract can [currently] be found in:
// tests/contracts/sources.ts under "StorageBloater"
const STORAGE_BLOATER_HEX: &str = concat!(
	"6080604052600060015534801561001557600080fd5b5061040e806100256000396000f3fe6080",
	"60405234801561001057600080fd5b50600436106100415760003560e01c806356f510a914610046",
	"578063b5ee966014610062578063d9a902b414610092575b600080fd5b6100606004803603810190",
	"61005b919061020d565b6100ae565b005b61007c600480360381019061007791906101a0565b6101",
	"0a565b604051610089919061026f565b60405180910390f35b6100ac60048036038101906100a791",
	"906101cd565b610122565b005b60005b828110156101045781816100c591906102e0565b846100d0",
	"919061028a565b60008083876100df919061028a565b815260200190815260200160002081905550",
	"80806100fc90610344565b9150506100b1565b50505050565b600060205280600052604060002060",
	"00915090505481565b6000805b8281101561016c57600080828661013d919061028a565b81526020",
	"019081526020016000205482610157919061028a565b9150808061016490610344565b9150506101",
	"26565b50806001600082825461017f919061028a565b92505081905550505050565b600081359050",
	"61019a816103c1565b92915050565b6000602082840312156101b6576101b56103bc565b5b600061",
	"01c48482850161018b565b91505092915050565b600080604083850312156101e4576101e36103bc",
	"565b5b60006101f28582860161018b565b92505060206102038582860161018b565b915050925092",
	"9050565b600080600060608486031215610226576102256103bc565b5b6000610234868287016101",
	"8b565b93505060206102458682870161018b565b92505060406102568682870161018b565b915050",
	"9250925092565b6102698161033a565b82525050565b600060208201905061028460008301846102",
	"60565b92915050565b60006102958261033a565b91506102a08361033a565b9250827fffffffffff",
	"ffffffffffffffffffffffffffffffffffffffffffffffffffffff038211156102d5576102d46103",
	"8d565b5b828201905092915050565b60006102eb8261033a565b91506102f68361033a565b925081",
	"7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff04831182151516",
	"1561032f5761032e61038d565b5b828202905092915050565b6000819050919050565b600061034f",
	"8261033a565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
	"ff8214156103825761038161038d565b5b600182019050919050565b7f4e487b7100000000000000",
	"000000000000000000000000000000000000000000600052601160045260246000fd5b600080fd5b",
	"6103ca8161033a565b81146103d557600080fd5b5056fea2646970667358221220abec915f4f93ce",
	"2b1d8fce7509c4a05914401e30a8ad5f58d6aef8baf7fc905864736f6c63430008060033"
);

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

		let calculate_calculate_sum_calldata = |start: u64, num_items: u64| {
			let start = U256::from(start);
			let num_items = U256::from(num_items);

			let mut bytes = Vec::<u8>::with_capacity(4 + 32 + 32);

			bytes.extend_from_slice(&Keccak256::digest(b"calculate_sum(uint256,uint256)")[0..4]);
			bytes.resize(4 + 32 + 32, 0);

			U256::from(start).to_big_endian(&mut bytes[4..36]);
			U256::from(num_items).to_big_endian(&mut bytes[36..68]);

			bytes
		};

		let storage_bloater_bytecode =
			hex::decode(STORAGE_BLOATER_HEX).expect("STORAGE_BLOATER_HEX is valid hex; qed");

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

		const NUM_STORAGE_ITEMS: u64 = 50000;
		const NUM_STORAGE_ITEMS_PER_BLOCK: u64 = 500;
		const NUM_BLOCKS: u64 = NUM_STORAGE_ITEMS / NUM_STORAGE_ITEMS_PER_BLOCK;

		assert!(NUM_BLOCKS == 100);

		// fill our storage contract with bloat
		let now = Instant::now();
		for i in 0..NUM_BLOCKS {
			// create calldata
			let calldata = calculate_bloat_storage_calldata(
				i * NUM_STORAGE_ITEMS_PER_BLOCK,
				NUM_STORAGE_ITEMS_PER_BLOCK,
				0,
			);

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
				.expect("EVM call failed while trying to call bloat_storage()");

			context.create_block(true);
			alice_nonce = alice_nonce.saturating_add(1.into());
		}

		results.push(TestResults::new(
			"storage bloating",
			now.elapsed().as_micros(),
			43335945,
		));

		const NUM_READS_PER_BLOCK: u64 = 5000;
		const NUM_READ_BLOCKS: u64 = NUM_STORAGE_ITEMS / NUM_READS_PER_BLOCK;
		assert!(NUM_READ_BLOCKS == 10);

		let now = Instant::now();
		for i in 0..NUM_READ_BLOCKS {
			// now call calculate_sum to force a read of these items which were written
			let calldata =
				calculate_calculate_sum_calldata(i * NUM_READS_PER_BLOCK, NUM_READS_PER_BLOCK);

			let _txn_hash = context
				.evm_call(
					alice.address,
					storage_bloater_address,
					calldata.clone(),
					0.into(),
					EXTRINSIC_GAS_LIMIT.into(),
					Some(MIN_GAS_PRICE.into()),
					Some(alice_nonce),
				)
				.expect("EVM call failed while trying to call calculate_sum()");

			// context.create_block(true);
		}

		results.push(TestResults::new(
			"storage bloat read",
			now.elapsed().as_micros(),
			43335945,
		));

		Ok(results)
	}
}
