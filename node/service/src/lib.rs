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

//! This module assembles the Moonbeam service components, executes them, and manages communication
//! between them. This is the backbone of the client-side node implementation.
//!
//! This module can assemble:
//! PartialComponents: For maintence tasks without a complete node (eg import/export blocks, purge)
//! Full Service: A complete parachain node including the pool, rpc, network, embedded relay chain
//! Dev Service: A leaner service without the relay chain backing.

use cli_opt::{EthApi as EthApiCmd, RpcConfig};
use fc_consensus::FrontierBlockImport;
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use futures::StreamExt;
#[cfg(feature = "moonbase-native")]
pub use moonbase_runtime;
#[cfg(feature = "moonbeam-native")]
pub use moonbeam_runtime;
#[cfg(feature = "moonriver-native")]
pub use moonriver_runtime;
use std::{collections::BTreeMap, sync::Mutex, time::Duration};
pub mod rpc;
use cumulus_client_consensus_common::ParachainConsensus;
use cumulus_client_network::BlockAnnounceValidator;
use cumulus_client_service::{
	prepare_node_config, start_collator, start_full_node, StartCollatorParams, StartFullNodeParams,
};
use cumulus_primitives_core::ParaId;
use cumulus_primitives_parachain_inherent::{
	MockValidationDataInherentDataProvider, MockXcmConfig,
};
use cumulus_relay_chain_interface::RelayChainInterface;
use cumulus_relay_chain_local::build_relay_chain_interface;
use nimbus_consensus::NimbusManualSealConsensusDataProvider;
use nimbus_consensus::{BuildNimbusConsensusParams, NimbusConsensus};
use nimbus_primitives::NimbusId;
use sc_executor::{NativeElseWasmExecutor, NativeExecutionDispatch};
use sc_network::NetworkService;
use sc_service::{
	error::Error as ServiceError, BasePath, ChainSpec, Configuration, PartialComponents, Role,
	TFullBackend, TFullClient, TaskManager,
};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sp_api::ConstructRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_keystore::SyncCryptoStorePtr;
use std::sync::Arc;
use substrate_prometheus_endpoint::Registry;

pub use client::*;
pub mod chain_spec;
mod client;

type FullClient<RuntimeApi, Executor> =
	TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>;
type FullBackend = TFullBackend<Block>;
type MaybeSelectChain = Option<sc_consensus::LongestChain<FullBackend, Block>>;

pub type HostFunctions = (
	frame_benchmarking::benchmarking::HostFunctions,
	moonbeam_primitives_ext::moonbeam_ext::HostFunctions,
);

#[cfg(feature = "moonbeam-native")]
pub struct MoonbeamExecutor;

#[cfg(feature = "moonbeam-native")]
impl sc_executor::NativeExecutionDispatch for MoonbeamExecutor {
	type ExtendHostFunctions = HostFunctions;

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		moonbeam_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		moonbeam_runtime::native_version()
	}
}

#[cfg(feature = "moonriver-native")]
pub struct MoonriverExecutor;

#[cfg(feature = "moonriver-native")]
impl sc_executor::NativeExecutionDispatch for MoonriverExecutor {
	type ExtendHostFunctions = HostFunctions;

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		moonriver_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		moonriver_runtime::native_version()
	}
}

#[cfg(feature = "moonbase-native")]
pub struct MoonbaseExecutor;

#[cfg(feature = "moonbase-native")]
impl sc_executor::NativeExecutionDispatch for MoonbaseExecutor {
	type ExtendHostFunctions = HostFunctions;

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		moonbase_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		moonbase_runtime::native_version()
	}
}

/// Trivial enum representing runtime variant
#[derive(Clone)]
pub enum RuntimeVariant {
	#[cfg(feature = "moonbeam-native")]
	Moonbeam,
	#[cfg(feature = "moonriver-native")]
	Moonriver,
	#[cfg(feature = "moonbase-native")]
	Moonbase,
	Unrecognized,
}

impl RuntimeVariant {
	pub fn from_chain_spec(chain_spec: &Box<dyn ChainSpec>) -> Self {
		match chain_spec {
			#[cfg(feature = "moonbeam-native")]
			spec if spec.is_moonbeam() => Self::Moonbeam,
			#[cfg(feature = "moonriver-native")]
			spec if spec.is_moonriver() => Self::Moonriver,
			#[cfg(feature = "moonbase-native")]
			spec if spec.is_moonbase() => Self::Moonbase,
			_ => Self::Unrecognized,
		}
	}
}

/// Can be called for a `Configuration` to check if it is a configuration for
/// the `Moonbeam` network.
pub trait IdentifyVariant {
	/// Returns `true` if this is a configuration for the `Moonbase` network.
	fn is_moonbase(&self) -> bool;

	/// Returns `true` if this is a configuration for the `Moonbeam` network.
	fn is_moonbeam(&self) -> bool;

	/// Returns `true` if this is a configuration for the `Moonriver` network.
	fn is_moonriver(&self) -> bool;

	/// Returns `true` if this is a configuration for a dev network.
	fn is_dev(&self) -> bool;
}

impl IdentifyVariant for Box<dyn ChainSpec> {
	fn is_moonbase(&self) -> bool {
		self.id().starts_with("moonbase")
	}

	fn is_moonbeam(&self) -> bool {
		self.id().starts_with("moonbeam")
	}

	fn is_moonriver(&self) -> bool {
		self.id().starts_with("moonriver")
	}

	fn is_dev(&self) -> bool {
		self.id().ends_with("dev")
	}
}

pub fn frontier_database_dir(config: &Configuration) -> std::path::PathBuf {
	let config_dir = config
		.base_path
		.as_ref()
		.map(|base_path| base_path.config_dir(config.chain_spec.id()))
		.unwrap_or_else(|| {
			BasePath::from_project("", "", "moonbeam").config_dir(config.chain_spec.id())
		});
	config_dir.join("frontier").join("db")
}

// TODO This is copied from frontier. It should be imported instead after
// https://github.com/paritytech/frontier/issues/333 is solved
pub fn open_frontier_backend(config: &Configuration) -> Result<Arc<fc_db::Backend<Block>>, String> {
	Ok(Arc::new(fc_db::Backend::<Block>::new(
		&fc_db::DatabaseSettings {
			source: fc_db::DatabaseSettingsSrc::RocksDb {
				path: frontier_database_dir(&config),
				cache_size: 0,
			},
		},
	)?))
}

use sp_runtime::{traits::BlakeTwo256, Percent};
use sp_trie::PrefixedMemoryDB;

pub const SOFT_DEADLINE_PERCENT: Percent = Percent::from_percent(100);

/// Builds a new object suitable for chain operations.
#[allow(clippy::type_complexity)]
pub fn new_chain_ops(
	config: &mut Configuration,
) -> Result<
	(
		Arc<Client>,
		Arc<FullBackend>,
		sc_consensus::BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
		TaskManager,
	),
	ServiceError,
> {
	match &config.chain_spec {
		#[cfg(feature = "moonriver-native")]
		spec if spec.is_moonriver() => {
			new_chain_ops_inner::<moonriver_runtime::RuntimeApi, MoonriverExecutor>(config)
		}
		#[cfg(feature = "moonbeam-native")]
		spec if spec.is_moonbeam() => {
			new_chain_ops_inner::<moonbeam_runtime::RuntimeApi, MoonbeamExecutor>(config)
		}
		#[cfg(feature = "moonbase-native")]
		_ => new_chain_ops_inner::<moonbase_runtime::RuntimeApi, MoonbaseExecutor>(config),
		#[cfg(not(feature = "moonbase-native"))]
		_ => panic!("invalid chain spec"),
	}
}

#[allow(clippy::type_complexity)]
fn new_chain_ops_inner<RuntimeApi, Executor>(
	mut config: &mut Configuration,
) -> Result<
	(
		Arc<Client>,
		Arc<FullBackend>,
		sc_consensus::BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
		TaskManager,
	),
	ServiceError,
>
where
	Client: From<Arc<crate::FullClient<RuntimeApi, Executor>>>,
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	config.keystore = sc_service::config::KeystoreConfig::InMemory;
	let PartialComponents {
		client,
		backend,
		import_queue,
		task_manager,
		..
	} = new_partial::<RuntimeApi, Executor>(config, config.chain_spec.is_dev())?;
	Ok((
		Arc::new(Client::from(client)),
		backend,
		import_queue,
		task_manager,
	))
}

/// Builds the PartialComponents for a parachain or development service
///
/// Use this function if you don't actually need the full service, but just the partial in order to
/// be able to perform chain operations.
#[allow(clippy::type_complexity)]
pub fn new_partial<RuntimeApi, Executor>(
	config: &Configuration,
	dev_service: bool,
) -> Result<
	PartialComponents<
		FullClient<RuntimeApi, Executor>,
		FullBackend,
		MaybeSelectChain,
		sc_consensus::DefaultImportQueue<Block, FullClient<RuntimeApi, Executor>>,
		sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>,
		(
			FrontierBlockImport<
				Block,
				Arc<FullClient<RuntimeApi, Executor>>,
				FullClient<RuntimeApi, Executor>,
			>,
			Option<FilterPool>,
			Option<Telemetry>,
			Option<TelemetryWorkerHandle>,
			Arc<fc_db::Backend<Block>>,
			FeeHistoryCache,
		),
	>,
	ServiceError,
>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let executor = NativeElseWasmExecutor::<Executor>::new(
		config.wasm_method,
		config.default_heap_pages,
		config.max_runtime_instances,
		config.runtime_cache_size,
	);

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;

	let client = Arc::new(client);

	let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager
			.spawn_handle()
			.spawn("telemetry", None, worker.run());
		telemetry
	});

	let maybe_select_chain = if dev_service {
		Some(sc_consensus::LongestChain::new(backend.clone()))
	} else {
		None
	};

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	let filter_pool: Option<FilterPool> = Some(Arc::new(Mutex::new(BTreeMap::new())));
	let fee_history_cache: FeeHistoryCache = Arc::new(Mutex::new(BTreeMap::new()));

	let frontier_backend = open_frontier_backend(config)?;

	let frontier_block_import =
		FrontierBlockImport::new(client.clone(), client.clone(), frontier_backend.clone());

	// Depending whether we are
	let import_queue = nimbus_consensus::import_queue(
		client.clone(),
		frontier_block_import.clone(),
		move |_, _| async move {
			let time = sp_timestamp::InherentDataProvider::from_system_time();

			Ok((time,))
		},
		&task_manager.spawn_essential_handle(),
		config.prometheus_registry(),
		!dev_service,
	)?;

	Ok(PartialComponents {
		backend,
		client,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		select_chain: maybe_select_chain,
		other: (
			frontier_block_import,
			filter_pool,
			telemetry,
			telemetry_worker_handle,
			frontier_backend,
			fee_history_cache,
		),
	})
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("🌗")]
async fn start_node_impl<RuntimeApi, Executor, BIC>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	id: polkadot_primitives::v0::Id,
	rpc_config: RpcConfig,
	build_consensus: BIC,
) -> sc_service::error::Result<(TaskManager, Arc<FullClient<RuntimeApi, Executor>>)>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
	BIC: FnOnce(
		Arc<TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>>,
		Option<&Registry>,
		Option<TelemetryHandle>,
		&TaskManager,
		Arc<dyn RelayChainInterface>,
		Arc<
			sc_transaction_pool::FullPool<
				Block,
				TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>,
			>,
		>,
		Arc<NetworkService<Block, Hash>>,
		SyncCryptoStorePtr,
		bool,
	) -> Result<Box<dyn ParachainConsensus<Block>>, sc_service::Error>,
{
	if matches!(parachain_config.role, Role::Light) {
		return Err("Light client not supported!".into());
	}

	let parachain_config = prepare_node_config(parachain_config);

	let params = new_partial(&parachain_config, false)?;
	let (
		_block_import,
		filter_pool,
		mut telemetry,
		telemetry_worker_handle,
		frontier_backend,
		fee_history_cache,
	) = params.other;

	let client = params.client.clone();
	let backend = params.backend.clone();
	let mut task_manager = params.task_manager;

	let (relay_chain_interface, collator_key) =
		build_relay_chain_interface(polkadot_config, telemetry_worker_handle, &mut task_manager)
			.map_err(|e| match e {
				polkadot_service::Error::Sub(x) => x,
				s => format!("{}", s).into(),
			})?;

	let block_announce_validator = BlockAnnounceValidator::new(relay_chain_interface.clone(), id);

	let force_authoring = parachain_config.force_authoring;
	let collator = parachain_config.role.is_authority();
	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let transaction_pool = params.transaction_pool.clone();
	let import_queue = cumulus_client_service::SharedImportQueue::new(params.import_queue);
	let (network, system_rpc_tx, start_network) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &parachain_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue: import_queue.clone(),
			block_announce_validator_builder: Some(Box::new(|_| {
				Box::new(block_announce_validator)
			})),
			warp_sync: None,
		})?;

	let subscription_task_executor =
		sc_rpc::SubscriptionTaskExecutor::new(task_manager.spawn_handle());
	let overrides = crate::rpc::overrides_handle(client.clone());
	let fee_history_limit = rpc_config.fee_history_limit;

	rpc::spawn_essential_tasks(rpc::SpawnTasksParams {
		task_manager: &task_manager,
		client: client.clone(),
		substrate_backend: backend.clone(),
		frontier_backend: frontier_backend.clone(),
		filter_pool: filter_pool.clone(),
		overrides: overrides.clone(),
		fee_history_limit,
		fee_history_cache: fee_history_cache.clone(),
	});

	let ethapi_cmd = rpc_config.ethapi.clone();
	let tracing_requesters =
		if ethapi_cmd.contains(&EthApiCmd::Debug) || ethapi_cmd.contains(&EthApiCmd::Trace) {
			rpc::tracing::spawn_tracing_tasks(
				&rpc_config,
				rpc::SpawnTasksParams {
					task_manager: &task_manager,
					client: client.clone(),
					substrate_backend: backend.clone(),
					frontier_backend: frontier_backend.clone(),
					filter_pool: filter_pool.clone(),
					overrides: overrides.clone(),
					fee_history_limit,
					fee_history_cache: fee_history_cache.clone(),
				},
			)
		} else {
			rpc::tracing::RpcRequesters {
				debug: None,
				trace: None,
			}
		};

	let block_data_cache = Arc::new(fc_rpc::EthBlockDataCache::new(
		task_manager.spawn_handle(),
		overrides.clone(),
		rpc_config.eth_log_block_cache,
		rpc_config.eth_log_block_cache,
	));

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let network = network.clone();
		let filter_pool = filter_pool.clone();
		let frontier_backend = frontier_backend.clone();
		let backend = backend.clone();
		let ethapi_cmd = ethapi_cmd.clone();
		let max_past_logs = rpc_config.max_past_logs;
		let overrides = overrides.clone();
		let fee_history_cache = fee_history_cache.clone();
		let block_data_cache = block_data_cache.clone();

		Box::new(move |deny_unsafe, _| {
			let deps = rpc::FullDeps {
				backend: backend.clone(),
				client: client.clone(),
				command_sink: None,
				deny_unsafe,
				ethapi_cmd: ethapi_cmd.clone(),
				filter_pool: filter_pool.clone(),
				frontier_backend: frontier_backend.clone(),
				graph: pool.pool().clone(),
				pool: pool.clone(),
				is_authority: collator,
				max_past_logs,
				fee_history_limit,
				fee_history_cache: fee_history_cache.clone(),
				network: network.clone(),
				xcm_senders: None,
				block_data_cache: block_data_cache.clone(),
				overrides: overrides.clone(),
			};
			#[allow(unused_mut)]
			let mut io = rpc::create_full(deps, subscription_task_executor.clone());
			if ethapi_cmd.contains(&EthApiCmd::Debug) || ethapi_cmd.contains(&EthApiCmd::Trace) {
				rpc::tracing::extend_with_tracing(
					client.clone(),
					tracing_requesters.clone(),
					rpc_config.ethapi_trace_max_count,
					&mut io,
				);
			}
			Ok(io)
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		rpc_extensions_builder,
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: params.keystore_container.sync_keystore(),
		backend: backend.clone(),
		network: network.clone(),
		system_rpc_tx,
		telemetry: telemetry.as_mut(),
	})?;

	let announce_block = {
		let network = network.clone();
		Arc::new(move |hash, data| network.announce_block(hash, data))
	};

	let relay_chain_slot_duration = Duration::from_secs(6);

	if collator {
		let parachain_consensus = build_consensus(
			client.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|t| t.handle()),
			&task_manager,
			relay_chain_interface.clone(),
			transaction_pool,
			network,
			params.keystore_container.sync_keystore(),
			force_authoring,
		)?;

		let spawner = task_manager.spawn_handle();

		let params = StartCollatorParams {
			para_id: id,
			block_status: client.clone(),
			announce_block,
			client: client.clone(),
			task_manager: &mut task_manager,
			relay_chain_interface,
			spawner,
			parachain_consensus,
			import_queue,
			collator_key,
			relay_chain_slot_duration,
		};

		start_collator(params).await?;
	} else {
		let params = StartFullNodeParams {
			client: client.clone(),
			announce_block,
			task_manager: &mut task_manager,
			para_id: id,
			relay_chain_interface,
			relay_chain_slot_duration,
			import_queue,
		};

		start_full_node(params)?;
	}

	start_network.start_network();

	Ok((task_manager, client))
}

/// Start a normal parachain node.
// Rustfmt wants to format the closure with space identation.
#[rustfmt::skip]
pub async fn start_node<RuntimeApi, Executor>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	id: polkadot_primitives::v0::Id,
	rpc_config: RpcConfig,
) -> sc_service::error::Result<(TaskManager, Arc<FullClient<RuntimeApi, Executor>>)>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	start_node_impl(
		parachain_config,
		polkadot_config,
		id,
		rpc_config,
		|
			client,
			prometheus_registry,
			telemetry,
			task_manager,
			relay_chain_interface,
			transaction_pool,
			_sync_oracle,
			keystore,
			force_authoring
		| {
			let mut proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
				task_manager.spawn_handle(),
				client.clone(),
				transaction_pool,
				prometheus_registry,
				telemetry.clone(),
			);
			proposer_factory.set_soft_deadline(SOFT_DEADLINE_PERCENT);

			let provider = move |_, (relay_parent, validation_data, author_id)| {
				let relay_chain_interface = relay_chain_interface.clone();
				async move {
					let parachain_inherent =
						cumulus_primitives_parachain_inherent::ParachainInherentData::create_at(
							relay_parent,
							&relay_chain_interface,
							&validation_data,
							id,
						)
						.await;

					let time = sp_timestamp::InherentDataProvider::from_system_time();

					let parachain_inherent = parachain_inherent.ok_or_else(|| {
						Box::<dyn std::error::Error + Send + Sync>::from(
							"Failed to create parachain inherent",
						)
					})?;

					let author = nimbus_primitives::InherentDataProvider::<NimbusId>(author_id);

					Ok((time, parachain_inherent, author))
				}
			};

			Ok(NimbusConsensus::build(BuildNimbusConsensusParams {
				para_id: id,
				proposer_factory,
				block_import: client.clone(),
				parachain_client: client.clone(),
				keystore,
				skip_prediction: force_authoring,
				create_inherent_data_providers: provider,
			}))
		},
	)
	.await
}

/// Builds a new development service. This service uses manual seal, and mocks
/// the parachain inherent.
pub fn new_dev<RuntimeApi, Executor>(
	config: Configuration,
	_author_id: Option<nimbus_primitives::NimbusId>,
	sealing: cli_opt::Sealing,
	rpc_config: RpcConfig,
) -> Result<TaskManager, ServiceError>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	use async_io::Timer;
	use futures::Stream;
	use sc_consensus_manual_seal::{run_manual_seal, EngineCommand, ManualSealParams};
	use sp_core::H256;

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
	} = new_partial::<RuntimeApi, Executor>(&config, true)?;

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

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let prometheus_registry = config.prometheus_registry().cloned();
	let subscription_task_executor =
		sc_rpc::SubscriptionTaskExecutor::new(task_manager.spawn_handle());
	let overrides = crate::rpc::overrides_handle(client.clone());
	let fee_history_limit = rpc_config.fee_history_limit;
	let mut command_sink = None;
	let mut xcm_senders = None;
	let collator = config.role.is_authority();

	if collator {
		let mut env = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);
		env.set_soft_deadline(SOFT_DEADLINE_PERCENT);
		let commands_stream: Box<dyn Stream<Item = EngineCommand<H256>> + Send + Sync + Unpin> =
			match sealing {
				cli_opt::Sealing::Instant => {
					Box::new(
						// This bit cribbed from the implementation of instant seal.
						transaction_pool
							.pool()
							.validated_pool()
							.import_notification_stream()
							.map(|_| EngineCommand::SealNewBlock {
								create_empty: false,
								finalize: false,
								parent_hash: None,
								sender: None,
							}),
					)
				}
				cli_opt::Sealing::Manual => {
					let (sink, stream) = futures::channel::mpsc::channel(1000);
					// Keep a reference to the other end of the channel. It goes to the RPC.
					command_sink = Some(sink);
					Box::new(stream)
				}
				cli_opt::Sealing::Interval(millis) => Box::new(StreamExt::map(
					Timer::interval(Duration::from_millis(millis)),
					|_| EngineCommand::SealNewBlock {
						create_empty: true,
						finalize: false,
						parent_hash: None,
						sender: None,
					},
				)),
			};

		let select_chain = maybe_select_chain.expect(
			"`new_partial` builds a `LongestChainRule` when building dev service.\
				We specified the dev service when calling `new_partial`.\
				Therefore, a `LongestChainRule` is present. qed.",
		);

		let client_set_aside_for_cidp = client.clone();

		// Create channels for mocked XCM messages.
		let (downward_xcm_sender, downward_xcm_receiver) = flume::bounded::<Vec<u8>>(100);
		let (hrmp_xcm_sender, hrmp_xcm_receiver) = flume::bounded::<(ParaId, Vec<u8>)>(100);
		xcm_senders = Some((downward_xcm_sender, hrmp_xcm_sender));

		task_manager.spawn_essential_handle().spawn_blocking(
			"authorship_task",
			Some("block-authoring"),
			run_manual_seal(ManualSealParams {
				block_import,
				env,
				client: client.clone(),
				pool: transaction_pool.clone(),
				commands_stream,
				select_chain,
				consensus_data_provider: Some(Box::new(NimbusManualSealConsensusDataProvider {
					keystore: keystore_container.sync_keystore(),
					client: client.clone(),
				})),
				create_inherent_data_providers: move |block: H256, ()| {
					let current_para_block = client_set_aside_for_cidp
						.number(block)
						.expect("Header lookup should succeed")
						.expect("Header passed in as parent should be present in backend.");

					let downward_xcm_receiver = downward_xcm_receiver.clone();
					let hrmp_xcm_receiver = hrmp_xcm_receiver.clone();

					let client_for_xcm = client_set_aside_for_cidp.clone();
					async move {
						let time = sp_timestamp::InherentDataProvider::from_system_time();

						let mocked_parachain = MockValidationDataInherentDataProvider {
							current_para_block,
							relay_offset: 1000,
							relay_blocks_per_para_block: 2,
							xcm_config: MockXcmConfig::new(
								&*client_for_xcm,
								block,
								Default::default(),
								Default::default(),
							),
							raw_downward_messages: downward_xcm_receiver.drain().collect(),
							raw_horizontal_messages: hrmp_xcm_receiver.drain().collect(),
						};

						Ok((time, mocked_parachain))
					}
				},
			}),
		);
	}
	rpc::spawn_essential_tasks(rpc::SpawnTasksParams {
		task_manager: &task_manager,
		client: client.clone(),
		substrate_backend: backend.clone(),
		frontier_backend: frontier_backend.clone(),
		filter_pool: filter_pool.clone(),
		overrides: overrides.clone(),
		fee_history_limit,
		fee_history_cache: fee_history_cache.clone(),
	});
	let ethapi_cmd = rpc_config.ethapi.clone();
	let tracing_requesters =
		if ethapi_cmd.contains(&EthApiCmd::Debug) || ethapi_cmd.contains(&EthApiCmd::Trace) {
			rpc::tracing::spawn_tracing_tasks(
				&rpc_config,
				rpc::SpawnTasksParams {
					task_manager: &task_manager,
					client: client.clone(),
					substrate_backend: backend.clone(),
					frontier_backend: frontier_backend.clone(),
					filter_pool: filter_pool.clone(),
					overrides: overrides.clone(),
					fee_history_limit,
					fee_history_cache: fee_history_cache.clone(),
				},
			)
		} else {
			rpc::tracing::RpcRequesters {
				debug: None,
				trace: None,
			}
		};

	let block_data_cache = Arc::new(fc_rpc::EthBlockDataCache::new(
		task_manager.spawn_handle(),
		overrides.clone(),
		rpc_config.eth_log_block_cache,
		rpc_config.eth_log_block_cache,
	));

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let backend = backend.clone();
		let network = network.clone();
		let ethapi_cmd = ethapi_cmd.clone();
		let max_past_logs = rpc_config.max_past_logs;
		let overrides = overrides.clone();
		let fee_history_cache = fee_history_cache.clone();
		let block_data_cache = block_data_cache.clone();

		Box::new(move |deny_unsafe, _| {
			let deps = rpc::FullDeps {
				backend: backend.clone(),
				client: client.clone(),
				command_sink: command_sink.clone(),
				deny_unsafe,
				ethapi_cmd: ethapi_cmd.clone(),
				filter_pool: filter_pool.clone(),
				frontier_backend: frontier_backend.clone(),
				graph: pool.pool().clone(),
				pool: pool.clone(),
				is_authority: collator,
				max_past_logs,
				fee_history_limit,
				fee_history_cache: fee_history_cache.clone(),
				network: network.clone(),
				xcm_senders: xcm_senders.clone(),
				overrides: overrides.clone(),
				block_data_cache: block_data_cache.clone(),
			};
			#[allow(unused_mut)]
			let mut io = rpc::create_full(deps, subscription_task_executor.clone());
			if ethapi_cmd.contains(&EthApiCmd::Debug) || ethapi_cmd.contains(&EthApiCmd::Trace) {
				rpc::tracing::extend_with_tracing(
					client.clone(),
					tracing_requesters.clone(),
					rpc_config.ethapi_trace_max_count,
					&mut io,
				);
			}
			Ok(io)
		})
	};

	let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network,
		client,
		keystore: keystore_container.sync_keystore(),
		task_manager: &mut task_manager,
		transaction_pool,
		rpc_extensions_builder,
		backend,
		system_rpc_tx,
		config,
		telemetry: None,
	})?;

	log::info!("Development Service Ready");

	network_starter.start_network();
	Ok(task_manager)
}
