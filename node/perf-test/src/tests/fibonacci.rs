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
	command::{TestContext, FullClient, FullBackend},
	tests::{TestRunner, TestResults},
};

use sp_runtime::transaction_validity::TransactionSource;
use sc_service::{
	Configuration, NativeExecutionDispatch, TFullClient, TFullBackend, TaskManager, TransactionPool,
};
use sc_cli::{
	CliConfiguration, Result as CliResult, SharedParams,
};
use sp_core::{H160, H256, U256};
use sc_client_api::HeaderBackend;
use sp_api::{ConstructRuntimeApi, ProvideRuntimeApi, BlockId};
use std::{
	sync::Arc,
	marker::PhantomData,
	time::Instant,
};
use fp_rpc::{EthereumRuntimeRPCApi, ConvertTransaction};
use nimbus_primitives::NimbusId;
use cumulus_primitives_parachain_inherent::MockValidationDataInherentDataProvider;
use sc_consensus_manual_seal::{run_manual_seal, EngineCommand, ManualSealParams, CreatedBlock};
use ethereum::TransactionAction;

use futures::{
	Stream, SinkExt,
	channel::{
		oneshot,
		mpsc,
	},
};

use service::{chain_spec, RuntimeApiCollection, Block};
use sha3::{Digest, Keccak256};

const EXTRINSIC_GAS_LIMIT: u64 = 12_995_000;
const MIN_GAS_PRICE: u64 = 1_000_000_000;

pub struct FibonacciPerfTest { }

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

	// taking a different approach and starting a full dev service
	fn run(&mut self, context: &TestContext<RuntimeApi, Executor>) -> Result<Vec<TestResults>, String>
	{
		let mut results: Vec<TestResults> = Default::default();

		let alice = context.get_alice_details();
		let mut alice_nonce: U256 = 0.into();

		// Fibonacci contract:
		/*
		pragma solidity>= 0.8.0;
		contract Fibonacci {
			function fib2(uint n) public returns(uint b) {
				if (n == 0) {
					return 0;
				}
				uint a = 1;
				b = 1;
				for (uint i = 2; i < n; i++) {
					uint c = a + b;
					a = b;
					b = c;
				}
				return b;
			}
		}
		*/

		// start by deploying a contract...
		let fibonacci_hex =
			"608060405234801561001057600080fd5b5061024b806100206000396000f3fe608060405234801561001057600080fd5b506004361061002b5760003560e01c80633a9bbfcd14610030575b600080fd5b61004a600480360381019061004591906100d3565b610060565b604051610057919061010b565b60405180910390f35b60008082141561007357600090506100b9565b600060019050600191506000600290505b838110156100b6576000838361009a9190610126565b90508392508093505080806100ae90610186565b915050610084565b50505b919050565b6000813590506100cd816101fe565b92915050565b6000602082840312156100e557600080fd5b60006100f3848285016100be565b91505092915050565b6101058161017c565b82525050565b600060208201905061012060008301846100fc565b92915050565b60006101318261017c565b915061013c8361017c565b9250827fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff03821115610171576101706101cf565b5b828201905092915050565b6000819050919050565b60006101918261017c565b91507fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8214156101c4576101c36101cf565b5b600182019050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b6102078161017c565b811461021257600080fd5b5056fea264697066735822122046f6b28b441f2f3caa263c58109a043ec2c03d4823bf1d25b1937cc1c684efab64736f6c63430008040033";
		let fibonacci_bytecode = hex::decode(fibonacci_hex)
			.expect("fibonacci_hex is valid hex; qed");

		// do a create() call (which doesn't persist) to see what our expected contract address
		// will be. afterward we create a txn and produce a block so it will persist.
		// TODO: better way to calculate new contract address
		let now = Instant::now();
		let create_info = context.evm_create(
			alice.address,
			fibonacci_bytecode.clone(),
			0.into(),
			EXTRINSIC_GAS_LIMIT.into(),
			Some(MIN_GAS_PRICE.into()),
			Some(alice_nonce),
			false
		).expect("EVM create failed while estimating contract address");
		results.push(TestResults::new("evm_create", now.elapsed()));

		let fibonacci_address = create_info.value;
		log::debug!("Fibonacci fibonacci_address expected to be {:?}", fibonacci_address);

		log::trace!("Issuing EVM create txn...");
		let txn_hash = context.eth_sign_and_send_transaction(
			&alice.privkey,
			None,
			fibonacci_bytecode,
			0.into(),
			EXTRINSIC_GAS_LIMIT.into(),
			MIN_GAS_PRICE.into(),
			alice_nonce,
		).expect("EVM create failed while trying to deploy Fibonacci contract");

		let now = Instant::now();
		context.create_block(true);
		results.push(TestResults::new("create_fibonacci", now.elapsed()));

		// TODO: verify txn results

		alice_nonce = alice_nonce.saturating_add(1.into());
		let calldata_hex = "3a9bbfcd0000000000000000000000000000000000000000000000000000000000000172";
		let calldata = hex::decode(calldata_hex)
			.expect("calldata is valid hex; qed");

		println!("Calling fib[370] 4096 times...");
		let now = Instant::now();
		for i in 0..4096 {
			let call_results = context.evm_call(
				alice.address,
				fibonacci_address,
				calldata.clone(),
				0.into(),
				EXTRINSIC_GAS_LIMIT.into(),
				Some(MIN_GAS_PRICE.into()),
				Some(alice_nonce),
				false
			).expect("EVM call failed while trying to invoke Fibonacci contract");

			log::debug!("EVM call returned {:?}", call_results);
		}
		results.push(TestResults::new("fibonacci_calls", now.elapsed()));

		Ok(results)
	}
}

