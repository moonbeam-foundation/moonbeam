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

use crate::{
	tests::{BlockCreationPerfTest, FibonacciPerfTest, StoragePerfTest, TestResults, TestRunner},
	txn_signer::UnsignedTransaction,
	PerfCmd,
};

use cumulus_primitives_parachain_inherent::{
	MockValidationDataInherentDataProvider, MockXcmConfig,
};
use ethereum::TransactionAction;
use fp_rpc::{ConvertTransactionRuntimeApi, EthereumRuntimeRPCApi};
use nimbus_primitives::NimbusId;
use sc_cli::{CliConfiguration, Result as CliResult, SharedParams};
use sc_client_api::HeaderBackend;
use sc_consensus_manual_seal::{run_manual_seal, CreatedBlock, EngineCommand, ManualSealParams};
use sc_executor::{NativeElseWasmExecutor, NativeExecutionDispatch};
use sc_service::{Configuration, TFullBackend, TFullClient, TaskManager, TransactionPool};
use sp_api::{BlockId, ConstructRuntimeApi, ProvideRuntimeApi};
use sp_core::{H160, H256, U256};
use sp_runtime::transaction_validity::TransactionSource;
use std::{fs::File, io::prelude::*, marker::PhantomData, sync::Arc};

use futures::{
	channel::{mpsc, oneshot},
	SinkExt, Stream,
};

use cli_table::{print_stdout, WithTitle};
use serde::Serialize;
use service::{chain_spec, rpc, Block, RuntimeApiCollection};
use sha3::{Digest, Keccak256};

pub type FullClient<RuntimeApi, Executor> =
	TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>;
pub type FullBackend = TFullBackend<Block>;

pub struct TestContext<RuntimeApi, Executor>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	_task_manager: TaskManager,
	client: Arc<FullClient<RuntimeApi, Executor>>,
	manual_seal_command_sink: mpsc::Sender<EngineCommand<H256>>,
	pool: Arc<sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>>,

	_marker1: PhantomData<RuntimeApi>,
	_marker2: PhantomData<Executor>,
}

impl<RuntimeApi, Executor> TestContext<RuntimeApi, Executor>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	pub fn from_cmd(config: Configuration, _cmd: &PerfCmd) -> CliResult<Self> {
		println!("perf-test from_cmd");
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
					filter_pool,
					telemetry,
					_telemetry_worker_handle,
					frontier_backend,
					fee_history_cache,
				),
		} = service::new_partial::<RuntimeApi, Executor>(&config, true)?;

		// TODO: review -- we don't need any actual networking
		let (network, system_rpc_tx, network_starter) =
			sc_service::build_network(sc_service::BuildNetworkParams {
				config: &config,
				client: client.clone(),
				transaction_pool: transaction_pool.clone(),
				spawn_handle: task_manager.spawn_handle(),
				import_queue,
				block_announce_validator_builder: None,
				warp_sync: None,
			})?;

		// TODO: maybe offchain worker needed?

		let author_id = chain_spec::get_from_seed::<NimbusId>("Alice");

		// TODO: no need for prometheus here...
		let prometheus_registry = config.prometheus_registry().cloned();

		let mut env = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);
		env.set_soft_deadline(service::SOFT_DEADLINE_PERCENT);

		let command_sink;
		let command_stream: Box<dyn Stream<Item = EngineCommand<H256>> + Send + Sync + Unpin> = {
			let (sink, stream) = mpsc::channel(1000);
			command_sink = sink;
			Box::new(stream)
		};

		let select_chain = maybe_select_chain.expect(
			"`new_partial` builds a `LongestChainRule` when building dev service.\
				We specified the dev service when calling `new_partial`.\
				Therefore, a `LongestChainRule` is present. qed.",
		);

		let client_set_aside_for_cidp = client.clone();

		log::debug!("spawning authorship task...");
		task_manager.spawn_essential_handle().spawn_blocking(
			"authorship_task",
			Some("block-authoring"),
			run_manual_seal(ManualSealParams {
				block_import,
				env,
				client: client.clone(),
				pool: transaction_pool.clone(),
				commands_stream: command_stream,
				select_chain,
				consensus_data_provider: None,
				create_inherent_data_providers: move |block: H256, ()| {
					let current_para_block = client_set_aside_for_cidp
						.number(block)
						.expect("Header lookup should succeed")
						.expect("Header passed in as parent should be present in backend.");
					let author_id = author_id.clone();

					let client_for_xcm = client_set_aside_for_cidp.clone();

					async move {
						let time = sp_timestamp::InherentDataProvider::from_system_time();

						let mocked_parachain = MockValidationDataInherentDataProvider {
							current_para_block,
							relay_offset: 1000,
							relay_blocks_per_para_block: 2,
							raw_downward_messages: Vec::new(),
							raw_horizontal_messages: Vec::new(),
							xcm_config: MockXcmConfig::new(
								&*client_for_xcm,
								block,
								Default::default(),
								Default::default(),
							),
						};

						let author = nimbus_primitives::InherentDataProvider::<NimbusId>(author_id);

						Ok((time, mocked_parachain, author))
					}
				},
			}),
		);

		let subscription_task_executor =
			sc_rpc::SubscriptionTaskExecutor::new(task_manager.spawn_handle());
		let overrides = rpc::overrides_handle(client.clone());

		let fee_history_limit = 2048;
		service::rpc::spawn_essential_tasks(service::rpc::SpawnTasksParams {
			task_manager: &task_manager,
			client: client.clone(),
			substrate_backend: backend.clone(),
			frontier_backend: frontier_backend.clone(),
			filter_pool: filter_pool.clone(),
			overrides: overrides.clone(),
			fee_history_limit,
			fee_history_cache: fee_history_cache.clone(),
		});

		let command_sink_for_deps = Some(command_sink.clone());

		let block_data_cache = Arc::new(fc_rpc::EthBlockDataCache::new(
			task_manager.spawn_handle(),
			overrides.clone(),
			3000,
			3000,
		));

		let rpc_extensions_builder = {
			let client = client.clone();
			let pool = transaction_pool.clone();
			let backend = backend.clone();
			let network = network.clone();
			let max_past_logs = 1000;
			let fee_history_cache = fee_history_cache.clone();
			let overrides = overrides.clone();
			let block_data_cache = block_data_cache.clone();

			Box::new(move |deny_unsafe, _| {
				let deps = rpc::FullDeps {
					client: client.clone(),
					pool: pool.clone(),
					graph: pool.pool().clone(),
					deny_unsafe,
					is_authority: true,
					network: network.clone(),
					filter_pool: filter_pool.clone(),
					ethapi_cmd: Default::default(),
					command_sink: command_sink_for_deps.clone(),
					frontier_backend: frontier_backend.clone(),
					backend: backend.clone(),
					max_past_logs,
					fee_history_limit,
					fee_history_cache: fee_history_cache.clone(),
					xcm_senders: None,
					overrides: overrides.clone(),
					block_data_cache: block_data_cache.clone(),
				};
				#[allow(unused_mut)]
				let mut io = rpc::create_full(deps, subscription_task_executor.clone());
				Ok(io)
			})
		};

		let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
			network,
			client: client.clone(),
			keystore: keystore_container.sync_keystore(),
			task_manager: &mut task_manager,
			transaction_pool: transaction_pool.clone(),
			rpc_extensions_builder,
			backend,
			system_rpc_tx,
			config,
			telemetry: None,
		})?;

		network_starter.start_network();

		Ok(TestContext {
			_task_manager: task_manager,
			client: client.clone(),
			manual_seal_command_sink: command_sink,
			pool: transaction_pool,
			_marker1: Default::default(),
			_marker2: Default::default(),
		})
	}

	pub fn get_alice_details(&self) -> AccountDetails {
		use std::str::FromStr;

		let alice_address = H160::from_str("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")
			.expect("valid hex provided; qed");

		let block = BlockId::Hash(self.client.info().best_hash);

		let nonce = self
			.client
			.runtime_api()
			.account_basic(&block, alice_address)
			.expect("should be able to get alices' account info")
			.nonce;

		AccountDetails {
			address: alice_address,
			privkey: H256::from_str(
				"5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133",
			)
			.expect("valid hex provided; qed"),
			nonce: nonce,
		}
	}

	pub fn evm_call(
		&self,
		from: H160,
		to: H160,
		data: Vec<u8>,
		value: U256,
		gas_limit: U256,
		max_fee_per_gas: Option<U256>,
		max_priority_fee_per_gas: Option<U256>,
		nonce: Option<U256>,
	) -> Result<fp_evm::CallInfo, sp_runtime::DispatchError> {
		let hash = self.client.info().best_hash;
		log::info!("evm_call best_hash: {:?}", hash);

		let result = self.client.runtime_api().call(
			&BlockId::Hash(hash),
			from,
			to,
			data,
			value,
			gas_limit,
			max_fee_per_gas,
			max_priority_fee_per_gas,
			nonce,
			false,
			None,
		);

		result.expect("why is this a Result<Result<...>>???") // TODO
	}

	pub fn evm_create(
		&self,
		from: H160,
		data: Vec<u8>,
		value: U256,
		gas_limit: U256,
		max_fee_per_gas: Option<U256>,
		max_priority_fee_per_gas: Option<U256>,
		nonce: Option<U256>,
	) -> Result<fp_evm::CreateInfo, sp_runtime::DispatchError> {
		let hash = self.client.info().best_hash;
		log::info!("evm_create best_hash: {:?}", hash);

		let result = self.client.runtime_api().create(
			&BlockId::Hash(hash),
			from,
			data,
			value,
			gas_limit,
			max_fee_per_gas,
			max_priority_fee_per_gas,
			nonce,
			false,
			None,
		);

		result.expect("why is this a Result<Result<...>>???") // TODO
	}

	/// Creates a transaction out of the given call/create arguments, signs it, and sends it
	pub fn eth_sign_and_send_transaction(
		&self,
		signing_key: &H256,
		to: Option<H160>,
		data: Vec<u8>,
		value: U256,
		gas_limit: U256,
		gas_price: U256,
		nonce: U256,
	) -> Result<H256, sp_runtime::DispatchError> {
		const CHAIN_ID: u64 = 1281; // TODO: derive from CLI or from Moonbase

		let action = match to {
			Some(addr) => TransactionAction::Call(addr),
			None => TransactionAction::Create,
		};

		let unsigned = UnsignedTransaction {
			chain_id: CHAIN_ID,
			nonce,
			gas_price,
			gas_limit,
			action,
			value,
			input: data,
		};
		let signed = unsigned.sign(signing_key);

		let transaction_hash =
			H256::from_slice(Keccak256::digest(&rlp::encode(&signed)).as_slice());

		let block_hash = BlockId::hash(self.client.info().best_hash);
		log::debug!("eth_sign_and_send_transaction best_hash: {:?}", block_hash);

		#[allow(deprecated)]
		let extrinsic = self
			.client
			.runtime_api()
			.convert_transaction_before_version_2(&block_hash, signed)
			.map_err(|_| "ConvertTransactionRuntimeApi not found")?;

		let future = self
			.pool
			.submit_one(&block_hash, TransactionSource::Local, extrinsic);

		let _ = futures::executor::block_on(future);

		Ok(transaction_hash)
	}

	/// Author a block through manual sealing
	pub fn create_block(&self, create_empty: bool) -> CreatedBlock<H256> {
		log::trace!("Issuing seal command...");
		let hash = self.client.info().best_hash;

		let mut sink = self.manual_seal_command_sink.clone();
		let future = async move {
			// TODO: why use oneshot here? is it impacting txn pool?
			let (sender, receiver) = oneshot::channel();
			let command = EngineCommand::SealNewBlock {
				create_empty,
				finalize: true,
				parent_hash: Some(hash),
				sender: Some(sender),
			};
			let _ = sink.send(command).await;
			receiver.await
		};

		log::trace!("waiting for SealNewBlock command to resolve...");
		futures::executor::block_on(future)
			.expect("block_on failed")
			.expect("Failed to receive SealNewBlock response")
	}
}

/// Struct representing account details, including private key
#[derive(Debug)]
pub struct AccountDetails {
	pub address: H160,
	pub privkey: H256,
	pub nonce: U256,
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
	pub fn run<RuntimeApi, Executor>(&self, cmd: &PerfCmd, config: Configuration) -> CliResult<()>
	where
		RuntimeApi:
			ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
		RuntimeApi::RuntimeApi:
			RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
		Executor: NativeExecutionDispatch + 'static,
	{
		// TODO: Joshy suggested looking at the substrate browser "test":
		// <substrate_repo>/bin/node/browser-testing/src/lib.rs
		let runner = TestContext::<RuntimeApi, Executor>::from_cmd(config, &self)?;

		// create an empty block to warm the runtime cache...
		runner.create_block(true);

		let tests: Vec<Box<dyn TestRunner<RuntimeApi, Executor>>> = vec![
			Box::new(FibonacciPerfTest::new()),
			Box::new(BlockCreationPerfTest::new()),
			Box::new(StoragePerfTest::new()),
		];

		let mut all_test_results: Vec<TestResults> = Default::default();

		let (have_filter, enabled_tests) = if let Some(filter) = &cmd.tests {
			let enabled_tests: Vec<&str> = filter.split(',').collect();
			(true, enabled_tests)
		} else {
			(false, Default::default())
		};

		for mut test in tests {
			if have_filter {
				if !enabled_tests.contains(&test.name().as_str()) {
					continue;
				}
			}
			let mut results: Vec<TestResults> = (*test.run(&runner)?).to_vec();
			all_test_results.append(&mut results);
		}

		#[derive(Serialize)]
		struct AllResults {
			test_results: Vec<TestResults>,
		}

		let all_results = AllResults {
			test_results: all_test_results.clone(),
		};
		let results_str =
			serde_json::to_string_pretty(&all_results).expect("fail to serialize AllResults");

		if let Some(target) = &cmd.output_file {
			let mut file = File::create(target)?;
			file.write_all(&results_str.into_bytes())?;
			println!("Results written to {:?}", target);
		} else {
			println!("System Information:\n{}", results_str);
		}

		let table = all_test_results.with_title();
		print_stdout(table).expect("failed to print results");

		Ok(())
	}
}
