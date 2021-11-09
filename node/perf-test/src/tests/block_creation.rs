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
use std::time::Instant;

use service::{Block, RuntimeApiCollection};

const EXTRINSIC_GAS_LIMIT: u64 = 12_995_000;
const MIN_GAS_PRICE: u64 = 1_000_000_000;

pub struct BlockCreationPerfTest {}

impl BlockCreationPerfTest {
	pub fn new() -> Self {
		BlockCreationPerfTest {}
	}
}

impl<RuntimeApi, Executor> TestRunner<RuntimeApi, Executor> for BlockCreationPerfTest
where
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	fn name(&self) -> String {
		"block_creation".into()
	}

	fn run(
		&mut self,
		context: &TestContext<RuntimeApi, Executor>,
	) -> Result<Vec<TestResults>, String> {
		let mut results: Vec<TestResults> = Default::default();

		let alice = context.get_alice_details();
		let mut alice_nonce = alice.nonce;

		const NUM_EMPTY_BLOCKS: u64 = 2048;
		println!("Creating {} empty blocks...", NUM_EMPTY_BLOCKS);
		let now = Instant::now();
		for _i in 1..NUM_EMPTY_BLOCKS {
			context.create_block(true);
		}
		results.push(TestResults::new(
			"empty blocks",
			now.elapsed().as_micros(),
			17962999,
		));

		println!("Creating blocks with increasing nonce-dependent txns...");
		let now = Instant::now();
		for i in 1..67 {
			for _ in 1..i {
				let _txn_hash = context
					.eth_sign_and_send_transaction(
						&alice.privkey,
						Some(alice.address),
						Default::default(),
						1.into(),
						EXTRINSIC_GAS_LIMIT.into(),
						MIN_GAS_PRICE.into(),
						alice_nonce,
					)
					.expect("Failed to send funds in nonce-dependent test");

				alice_nonce = alice_nonce.saturating_add(1.into());
			}

			context.create_block(true);
		}
		results.push(TestResults::new(
			"nonce-dependent blocks",
			now.elapsed().as_micros(),
			23069066,
		));

		Ok(results)
	}
}
