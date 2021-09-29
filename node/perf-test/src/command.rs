// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::PerfCmd;

use sp_runtime::traits::{Block as BlockT, Header as HeaderT, NumberFor};
use sc_service::{Configuration, NativeExecutionDispatch, TFullClient, TFullBackend, TaskManager};
use sc_cli::{
	CliConfiguration, Result as CliResult, SharedParams,
};
use sp_core::{
	H160, H256, U256,
	Encode,
	offchain:: {
		testing::{TestOffchainExt, TestTransactionPoolExt},
		OffchainDbExt, OffchainWorkerExt, TransactionPoolExt,
	}
};
use sc_cli::{ExecutionStrategy, WasmExecutionMethod};
use sc_client_db::BenchmarkingState;
use sc_client_api::HeaderBackend;
use sc_executor::NativeExecutor;
use sp_externalities::Extensions;
use sp_keystore::{testing::KeyStore, KeystoreExt, SyncCryptoStorePtr};
use sp_api::{ConstructRuntimeApi, ProvideRuntimeApi, BlockId};
use std::{fmt::Debug, sync::Arc, marker::PhantomData, time};
use sp_state_machine::StateMachine;
use cli_opt::RpcConfig;
use fp_rpc::EthereumRuntimeRPCApi;
use nimbus_primitives::NimbusId;
use cumulus_primitives_parachain_inherent::{
	MockValidationDataInherentDataProvider, ParachainInherentData,
};
use sc_consensus_manual_seal::{run_manual_seal, EngineCommand, ManualSealParams};

use async_io::Timer;
use futures::{Stream, Sink, SinkExt, channel::mpsc::Sender};

use service::{chain_spec, RuntimeApiCollection, Block};
type FullClient<RuntimeApi, Executor> = TFullClient<Block, RuntimeApi, Executor>;
type FullBackend = TFullBackend<Block>;

struct PerfTestRunner<RuntimeApi, Executor>
	where
		RuntimeApi:
			ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
		RuntimeApi::RuntimeApi:
			RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
		Executor: NativeExecutionDispatch + 'static,
{
	task_manager: TaskManager,
	client: Arc<TFullClient<Block, RuntimeApi, Executor>>,
	manual_seal_command_sink: futures::channel::mpsc::Sender<EngineCommand<H256>>,

	_marker1: PhantomData<RuntimeApi>,
	_marker2: PhantomData<Executor>,
}

// TODO: am I abusing the name "runner"?
impl<RuntimeApi, Executor> PerfTestRunner<RuntimeApi, Executor>
	where
		RuntimeApi:
			ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
		RuntimeApi::RuntimeApi:
			RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
		Executor: NativeExecutionDispatch + 'static,
{
	fn from_cmd(config: &Configuration, cmd: &PerfCmd) -> CliResult<Self> {

		// TODO: can we explicitly disable everything RPC-related? or anything network related, for
		//       that matter?
		//       Note: new_dev() calls `network_starter.start_network()` at the very end
		let rpc_config = RpcConfig {
			ethapi: Default::default(),
			ethapi_max_permits: 0,
			ethapi_trace_max_count: 0,
			ethapi_trace_cache_duration: 0,
			max_past_logs: 0,
		};

		// TODO: allow CLI to configure or at least make sure this doesn't conflict with available
		//       CLI options
		let sealing = cli_opt::Sealing::Manual;

		let sc_service::PartialComponents {
			client,
			backend,
			mut task_manager,
			import_queue,
			keystore_container,
			select_chain: maybe_select_chain,
			transaction_pool,
			other:
				(
					block_import,
					pending_transactions,
					filter_pool,
					telemetry,
					_telemetry_worker_handle,
					frontier_backend,
				),
		} = service::new_partial::<RuntimeApi, Executor>(&config, true)?;
		let author_id = chain_spec::get_from_seed::<NimbusId>("Alice");

		// TODO: no need for prometheus here...
		let prometheus_registry = config.prometheus_registry().cloned();
		// let mut command_sink: Option<Box<Sink<EngineCommand<H256>>>> = None;
		let mut command_sink: Option<Box<dyn Sink<EngineCommand<H256>, Error = ()>>> = None;

		let env = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		log::warn!("opening channel...");
		let (command_sink, commands_stream) = futures::channel::mpsc::channel(1000);

		let select_chain = maybe_select_chain.expect(
			"`new_partial` builds a `LongestChainRule` when building dev service.\
				We specified the dev service when calling `new_partial`.\
				Therefore, a `LongestChainRule` is present. qed.",
		);

		let client_set_aside_for_cidp = client.clone();

		log::warn!("spawning authorship task...");
		task_manager.spawn_essential_handle().spawn_blocking(
			"authorship_task",
			run_manual_seal(ManualSealParams {
				block_import,
				env,
				client: client.clone(),
				pool: transaction_pool.clone(),
				commands_stream: Box::new(commands_stream),
				select_chain,
				consensus_data_provider: None,
				create_inherent_data_providers: move |block: H256, ()| {
					let current_para_block = client_set_aside_for_cidp
						.number(block)
						.expect("Header lookup should succeed")
						.expect("Header passed in as parent should be present in backend.");
					let author_id = author_id.clone();

					async move {
						let time = sp_timestamp::InherentDataProvider::from_system_time();

						let mocked_parachain = MockValidationDataInherentDataProvider {
							current_para_block,
							relay_offset: 1000,
							relay_blocks_per_para_block: 2,
						};

						let author = nimbus_primitives::InherentDataProvider::<NimbusId>(author_id);

						Ok((time, mocked_parachain, author))
					}
				},
			}),
		);

		log::warn!("returning new PerfTestRunner");
		Ok(PerfTestRunner {
			task_manager,
			client: client.clone(),
			manual_seal_command_sink: command_sink,
			_marker1: Default::default(),
			_marker2: Default::default(),
		})
	}

	fn evm_call(
		&mut self,
		from: H160,
		to: H160,
		data: Vec<u8>,
		value: U256,
		gas_limit: U256,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		estimate: bool,
	) -> Result<fp_evm::CallInfo, sp_runtime::DispatchError> {
		let hash = self.client.info().best_hash;

		let result = self.client.runtime_api().call(
			&BlockId::Hash(hash),
			from,
			to,
			data,
			value,
			gas_limit,
			gas_price,
			nonce,
			false,
		);

		result.expect("why is this a Result<Result<...>>???") // TODO
	}

	fn evm_create(
		&mut self,
		from: H160,
		data: Vec<u8>,
		value: U256,
		gas_limit: U256,
		gas_price: Option<U256>,
		nonce: Option<U256>,
		estimate: bool,
	) -> Result<fp_evm::CreateInfo, sp_runtime::DispatchError> {
		let hash = self.client.info().best_hash;

		let result = self.client.runtime_api().create(
			&BlockId::Hash(hash),
			from,
			data,
			value,
			gas_limit,
			gas_price,
			nonce,
			false,
		);

		result.expect("why is this a Result<Result<...>>???") // TODO
	}

	/// Author a block through manual sealing
	fn create_block(&mut self) {
		log::warn!("Issuing seal command...");
		let result = self.manual_seal_command_sink.send(
			EngineCommand::SealNewBlock {
				create_empty: false,
				finalize: false,
				parent_hash: None,
				sender: None,
			});

		// log::warn!("SealNewBlock send result: {:?}", result);
	}
}

impl CliConfiguration for PerfCmd {
	fn shared_params(&self) -> &SharedParams {
		&self.shared_params
	}

	// copied from BenchmarkCmd, might be useful
	/*
	fn chain_id(&self, _is_dev: bool) -> Result<String> {
		Ok(match self.shared_params.chain {
			Some(ref chain) => chain.clone(),
			None => "dev".into(),
		})
	}
	*/
}

impl PerfCmd {

	// taking a different approach and starting a full dev service
	pub fn run<RuntimeApi, Executor>(&self, config: Configuration, ) -> CliResult<()>
	where
		RuntimeApi:
			ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
		RuntimeApi::RuntimeApi:
			RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
		Executor: NativeExecutionDispatch + 'static,
	{
		log::warn!("PerfCmd::run()");

		let alice_hex = "f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
		let alice_bytes = hex::decode(alice_hex)
			.expect("alice_hex is valid hex; qed");
		let alice = H160::from_slice(&alice_bytes[..]);

		log::warn!("alice: {:?}", alice);

		let mut runner = PerfTestRunner::<RuntimeApi, Executor>::from_cmd(&config, &self)?;
		log::warn!("Created runner");

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

		// TODO: send txn rather than call create
		log::warn!("Issuing EVM create call...");
		let create_results = runner.evm_create(
			alice,
			fibonacci_bytecode,
			0.into(),
			1_000_000.into(),
			Some(1_000_000_000.into()),
			Some(alice_nonce),
			false,
		).expect("EVM create failed while trying to deploy Fibonacci contract");
		let fibonacci_address: H160 = create_results.value;
		log::debug!("fibonacci_address: {:?}", fibonacci_address);

		runner.create_block();

		alice_nonce = alice_nonce.saturating_add(1.into());
		let calldata_hex = "3a9bbfcd0000000000000000000000000000000000000000000000000000000000000400";
		let calldata = hex::decode(calldata_hex)
			.expect("calldata is valid hex; qed");

		/*
		let call_results = runner.evm_call(
			alice,
			fibonacci_address,
			calldata,
			0.into(),
			1_000_000.into(),
			Some(1_000_000_000.into()),
			Some(alice_nonce),
			false
		).expect("EVM call failed while trying to invoke Fibonacci contract");

		log::warn!("EVM call returned {:?}", call_results);
		*/

		log::warn!("sleeping for a bit...");
		std::thread::sleep(std::time::Duration::from_millis(5000));

		Ok(())
	}
}

