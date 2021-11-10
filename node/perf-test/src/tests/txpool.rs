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
	command::{FullBackend, FullClient, TestContext, AccountDetails},
	tests::{TestResults, TestRunner},
};

use sc_service::NativeExecutionDispatch;
use sp_api::ConstructRuntimeApi;
use sp_core::{U256, H256};
use std::time::Instant;

use service::{Block, RuntimeApiCollection};
use sha3::{Digest, Keccak256};

const EXTRINSIC_GAS_LIMIT: u64 = 12_995_000;
const MIN_GAS_PRICE: u64 = 1_000_000_000;

pub struct TxPoolPerfTest {}

impl TxPoolPerfTest {
	pub fn new() -> Self {
		TxPoolPerfTest {}
	}
}

impl<RuntimeApi, Executor> TestRunner<RuntimeApi, Executor> for TxPoolPerfTest
where
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	fn name(&self) -> String {
		"txpool".into()
	}

	fn run(
		&mut self,
		context: &TestContext<RuntimeApi, Executor>,
	) -> Result<Vec<TestResults>, String> {
		let mut results: Vec<TestResults> = Default::default();

		let alice = context.get_alice_details();
		let mut alice_nonce = alice.nonce;

		// TODO:
		// 1. initialize many accounts with some balances to transfer to each other
		// 2. create many transactions from these accounts with no nonce-dependencies
		// 3. create blocks until the transactions are entirely drained
		//    TODO: it would be nice to show txns per block stats here

		const NUM_ACCOUNTS: usize = 8000; // note that default txpool size is 8192
		let mut accounts: Vec<AccountDetails> = Vec::with_capacity(NUM_ACCOUNTS);

		log::debug!(target: "perf-test", "txpool test creating {} accounts...", NUM_ACCOUNTS);
		for i in 0..NUM_ACCOUNTS {
			accounts.push(context.random_account());
		}

		log::debug!(target: "perf-test", "txpool test sending {} funding transactions...", NUM_ACCOUNTS);
		for i in 0..NUM_ACCOUNTS {
			let account = &accounts[i];
			let txn_hash = context
				.eth_sign_and_send_transaction(
					&alice.privkey,
					Some(account.address),
					Default::default(),
					100_000_000_000_000_000_000_u128.into(),
					EXTRINSIC_GAS_LIMIT.into(),
					MIN_GAS_PRICE.into(),
					alice_nonce,
				)
				.expect("failed to submit funding txn in txpool perf-test");

			let status = context.pool_status();
			// println!("txpool backlog @ {}: {}/{}", i, status.ready, status.future);

			alice_nonce = alice_nonce.saturating_add(1.into());
		}

		// create blocks until we drain the pool
		log::debug!(target: "perf-test", "txpool test draining pool...");
		let now = Instant::now();
		loop {
			// println!("calling create_block...");
			context.create_block(true);
			// println!("create_block returned");

			let status = context.pool_status();
			let backlog = status.ready + status.future;
			// println!("current txpool backlog: {}", backlog);
			if backlog == 0 {
				break;
			}
		}

		results.push(TestResults::new(
			"txpool funding",
			now.elapsed().as_micros(),
			1,
		));

		log::debug!(target: "perf-test", "txpool test sending balance transfers from different accounts...");
		for i in 0..NUM_ACCOUNTS-1 {
			let next_account = accounts[i+1].clone();
			let mut account = &mut accounts[i];

			log::debug!(target: "perf-test", "sending from [{}]({}) to [{}]({})",
				i,
				account.address, 
				i+1,
				next_account.address);

			// TODO: "BalanceLow" error after usually 200 or so of these...
			context.evm_call(
				account.address,
				next_account.address,
				Default::default(),
				1.into(),
				EXTRINSIC_GAS_LIMIT.into(),
				Some(MIN_GAS_PRICE.into()),
				Some(account.nonce)).expect("call failed");
			/*
			let _txn_hash = context
				.eth_sign_and_send_transaction(
					&account.privkey,
					Some(next_account.address),
					Default::default(),
					1.into(),
					EXTRINSIC_GAS_LIMIT.into(),
					MIN_GAS_PRICE.into(),
					account.nonce,
				)
				.expect("failed to submit balance swap txn in txpool perf-test");
				*/

			let status = context.pool_status();
			println!("txpool backlog @ {}: {}/{}", i, status.ready, status.future);

			account.nonce = account.nonce.saturating_add(1.into());
		}

		// create blocks until we drain the pool
		log::debug!(target: "perf-test", "txpool test draining balance swap txns...");
		let now = Instant::now();
		loop {
			println!("calling create_block...");
			context.create_block(true);
			println!("create_block returned");

			let status = context.pool_status();
			let backlog = status.ready + status.future;
			println!("current txpool backlog: {}", backlog);
			if backlog == 0 {
				break;
			}
		}

		results.push(TestResults::new(
			"balance swaps",
			now.elapsed().as_micros(),
			1,
		));

		Ok(results)
	}
}
