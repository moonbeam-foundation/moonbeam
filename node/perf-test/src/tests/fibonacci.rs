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

// source code for this contract can [currently] be found in:
// tests/contracts/sources.ts under "Fibonacci"
const FIBONACCI_HEX: &str = concat!(
	"608060405234801561001057600080fd5b5061024b806100206000396000f3fe6080604052348015",
	"61001057600080fd5b506004361061002b5760003560e01c80633a9bbfcd14610030575b600080fd",
	"5b61004a600480360381019061004591906100d3565b610060565b604051610057919061010b565b",
	"60405180910390f35b60008082141561007357600090506100b9565b600060019050600191506000",
	"600290505b838110156100b6576000838361009a9190610126565b90508392508093505080806100",
	"ae90610186565b915050610084565b50505b919050565b6000813590506100cd816101fe565b9291",
	"5050565b6000602082840312156100e557600080fd5b60006100f3848285016100be565b91505092",
	"915050565b6101058161017c565b82525050565b600060208201905061012060008301846100fc56",
	"5b92915050565b60006101318261017c565b915061013c8361017c565b9250827fffffffffffffff",
	"ffffffffffffffffffffffffffffffffffffffffffffffffff03821115610171576101706101cf56",
	"5b5b828201905092915050565b6000819050919050565b60006101918261017c565b91507fffffff",
	"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8214156101c4576101c361",
	"01cf565b5b600182019050919050565b7f4e487b7100000000000000000000000000000000000000",
	"000000000000000000600052601160045260246000fd5b6102078161017c565b8114610212576000",
	"80fd5b5056fea264697066735822122046f6b28b441f2f3caa263c58109a043ec2c03d4823bf1d25",
	"b1937cc1c684efab64736f6c63430008040033"
);

pub struct FibonacciPerfTest {}

impl FibonacciPerfTest {
	pub fn new() -> Self {
		FibonacciPerfTest {}
	}
}

impl<RuntimeApi, Executor> TestRunner<RuntimeApi, Executor> for FibonacciPerfTest
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

		// start by deploying a contract...
		let fibonacci_bytecode =
			hex::decode(FIBONACCI_HEX).expect("FIBONACCI_HEX is valid hex; qed");

		// do a create() call (which doesn't persist) to see what our expected contract address
		// will be. afterward we create a txn and produce a block so it will persist.
		// TODO: better way to calculate new contract address
		let now = Instant::now();
		let create_info = context
			.evm_create(
				alice.address,
				fibonacci_bytecode.clone(),
				0.into(),
				EXTRINSIC_GAS_LIMIT.into(),
				Some(MIN_GAS_PRICE.into()),
				Some(alice_nonce),
			)
			.expect("EVM create failed while estimating contract address");
		results.push(TestResults::new("evm_create", now.elapsed()));

		let fibonacci_address = create_info.value;
		log::debug!(
			"Fibonacci fibonacci_address expected to be {:?}",
			fibonacci_address
		);

		log::trace!("Issuing EVM create txn...");
		let _txn_hash = context
			.eth_sign_and_send_transaction(
				&alice.privkey,
				None,
				fibonacci_bytecode,
				0.into(),
				EXTRINSIC_GAS_LIMIT.into(),
				MIN_GAS_PRICE.into(),
				alice_nonce,
			)
			.expect("EVM create failed while trying to deploy Fibonacci contract");

		let now = Instant::now();
		context.create_block(true);
		results.push(TestResults::new("create_fibonacci", now.elapsed()));

		// TODO: verify txn results

		alice_nonce = alice_nonce.saturating_add(1.into());
		let calldata_hex =
			"3a9bbfcd0000000000000000000000000000000000000000000000000000000000000172";
		let calldata = hex::decode(calldata_hex).expect("calldata is valid hex; qed");

		const NUM_FIB_370_CALLS: u32 = 4096;
		println!("Calling fib[370] {} times...", NUM_FIB_370_CALLS);
		let now = Instant::now();
		for _ in 0..NUM_FIB_370_CALLS {
			let call_results = context
				.evm_call(
					alice.address,
					fibonacci_address,
					calldata.clone(),
					0.into(),
					EXTRINSIC_GAS_LIMIT.into(),
					Some(MIN_GAS_PRICE.into()),
					Some(alice_nonce),
				)
				.expect("EVM call failed while trying to invoke Fibonacci contract");

			log::debug!("EVM call returned {:?}", call_results);
		}
		results.push(TestResults::new("fibonacci_calls", now.elapsed()));

		Ok(results)
	}
}
