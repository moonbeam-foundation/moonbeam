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

		log::warn!("we have a service!");

		Ok(PerfTestRunner {
			task_manager,
			client: client.clone(),
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

		let mut runner = PerfTestRunner::<RuntimeApi, Executor>::from_cmd(&config, &self)?;
		let info = runner.evm_call(
			Default::default(),
			Default::default(),
			Default::default(),
			Default::default(),
			Default::default(),
			Some(Default::default()),
			Some(Default::default()),
			false,
		).expect("EVM call failed");

		log::warn!("EVM call returned {:?}", info);

		Ok(())
	}
}

