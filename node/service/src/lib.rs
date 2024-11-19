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

pub mod rpc;

use cumulus_client_cli::CollatorOptions;
use cumulus_client_collator::service::CollatorService;
use cumulus_client_consensus_common::ParachainBlockImport as TParachainBlockImport;
use cumulus_client_consensus_proposer::Proposer;
use cumulus_client_parachain_inherent::{MockValidationDataInherentDataProvider, MockXcmConfig};
use cumulus_client_service::{
	prepare_node_config, start_relay_chain_tasks, CollatorSybilResistance, DARecoveryProfile,
	ParachainHostFunctions, StartRelayChainTasksParams,
};
use cumulus_primitives_core::{
	relay_chain::{well_known_keys, CollatorPair},
	ParaId,
};
use cumulus_relay_chain_inprocess_interface::build_inprocess_relay_chain;
use cumulus_relay_chain_interface::{OverseerHandle, RelayChainInterface, RelayChainResult};
use cumulus_relay_chain_minimal_node::build_minimal_relay_chain_node_with_rpc;
use fc_consensus::FrontierBlockImport as TFrontierBlockImport;
use fc_db::DatabaseSource;
use fc_rpc::StorageOverrideHandler;
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use futures::{FutureExt, StreamExt};
use maplit::hashmap;
#[cfg(feature = "moonbase-native")]
pub use moonbase_runtime;
use moonbeam_cli_opt::{EthApi as EthApiCmd, FrontierBackendConfig, RpcConfig};
#[cfg(feature = "moonbeam-native")]
pub use moonbeam_runtime;
use moonbeam_vrf::VrfDigestsProvider;
#[cfg(feature = "moonriver-native")]
pub use moonriver_runtime;
use nimbus_consensus::NimbusManualSealConsensusDataProvider;
use nimbus_primitives::{DigestsProvider, NimbusId};
use polkadot_primitives::Slot;
use sc_client_api::{
	backend::{AuxStore, Backend, StateBackend, StorageProvider},
	ExecutorProvider,
};
use sc_consensus::ImportQueue;
use sc_executor::{HeapAllocStrategy, WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY};
use sc_network::{config::FullNetworkConfiguration, NetworkBackend, NetworkBlock};
use sc_service::config::PrometheusConfig;
use sc_service::{
	error::Error as ServiceError, ChainSpec, Configuration, PartialComponents, TFullBackend,
	TFullClient, TaskManager,
};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use session_keys_primitives::VrfApi;
use sp_api::{ConstructRuntimeApi, ProvideRuntimeApi};
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus::SyncOracle;
use sp_core::{ByteArray, Encode, H256};
use sp_keystore::{Keystore, KeystorePtr};
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::{collections::BTreeMap, path::Path, sync::Mutex, time::Duration};
use substrate_prometheus_endpoint::Registry;

pub use client::*;
pub mod chain_spec;
mod client;
#[cfg(feature = "lazy-loading")]
pub mod lazy_loading;

type FullClient<RuntimeApi> = TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>;
type FullBackend = TFullBackend<Block>;

type MaybeSelectChain<Backend> = Option<sc_consensus::LongestChain<Backend, Block>>;
type FrontierBlockImport<Client> = TFrontierBlockImport<Block, Arc<Client>, Client>;
type ParachainBlockImport<Client, Backend> =
	TParachainBlockImport<Block, FrontierBlockImport<Client>, Backend>;
type PartialComponentsResult<Client, Backend> = Result<
	PartialComponents<
		Client,
		Backend,
		MaybeSelectChain<Backend>,
		sc_consensus::DefaultImportQueue<Block>,
		sc_transaction_pool::FullPool<Block, Client>,
		(
			BlockImportPipeline<FrontierBlockImport<Client>, ParachainBlockImport<Client, Backend>>,
			Option<FilterPool>,
			Option<Telemetry>,
			Option<TelemetryWorkerHandle>,
			Arc<fc_db::Backend<Block, Client>>,
			FeeHistoryCache,
		),
	>,
	ServiceError,
>;

const RELAY_CHAIN_SLOT_DURATION_MILLIS: u64 = 6_000;

static TIMESTAMP: AtomicU64 = AtomicU64::new(0);

/// Provide a mock duration starting at 0 in millisecond for timestamp inherent.
/// Each call will increment timestamp by slot_duration making Aura think time has passed.
struct MockTimestampInherentDataProvider;
#[async_trait::async_trait]
impl sp_inherents::InherentDataProvider for MockTimestampInherentDataProvider {
	async fn provide_inherent_data(
		&self,
		inherent_data: &mut sp_inherents::InherentData,
	) -> Result<(), sp_inherents::Error> {
		TIMESTAMP.fetch_add(RELAY_CHAIN_SLOT_DURATION_MILLIS, Ordering::SeqCst);
		inherent_data.put_data(
			sp_timestamp::INHERENT_IDENTIFIER,
			&TIMESTAMP.load(Ordering::SeqCst),
		)
	}

	async fn try_handle_error(
		&self,
		_identifier: &sp_inherents::InherentIdentifier,
		_error: &[u8],
	) -> Option<Result<(), sp_inherents::Error>> {
		// The pallet never reports error.
		None
	}
}

#[cfg(feature = "runtime-benchmarks")]
pub type HostFunctions = (
	frame_benchmarking::benchmarking::HostFunctions,
	ParachainHostFunctions,
	moonbeam_primitives_ext::moonbeam_ext::HostFunctions,
);
#[cfg(not(feature = "runtime-benchmarks"))]
pub type HostFunctions = (
	ParachainHostFunctions,
	moonbeam_primitives_ext::moonbeam_ext::HostFunctions,
);

/// Block Import Pipeline used.
pub enum BlockImportPipeline<T, E> {
	/// Used in dev mode to import new blocks as best blocks.
	Dev(T),
	/// Used in parachain mode.
	Parachain(E),
}

/// A trait that must be implemented by all moon* runtimes executors.
///
/// This feature allows, for instance, to customize the client extensions according to the type
/// of network.
/// For the moment, this feature is only used to specify the first block compatible with
/// ed25519-zebra, but it could be used for other things in the future.
pub trait ClientCustomizations {
	/// The host function ed25519_verify has changed its behavior in the substrate history,
	/// because of the change from lib ed25519-dalek to lib ed25519-zebra.
	/// Some networks may have old blocks that are not compatible with ed25519-zebra,
	/// for these networks this function should return the 1st block compatible with the new lib.
	/// If this function returns None (default behavior), it implies that all blocks are compatible
	/// with the new lib (ed25519-zebra).
	fn first_block_number_compatible_with_ed25519_zebra() -> Option<u32> {
		None
	}
}

#[cfg(feature = "moonbeam-native")]
pub struct MoonbeamCustomizations;
#[cfg(feature = "moonbeam-native")]
impl ClientCustomizations for MoonbeamCustomizations {
	fn first_block_number_compatible_with_ed25519_zebra() -> Option<u32> {
		Some(2_000_000)
	}
}

#[cfg(feature = "moonriver-native")]
pub struct MoonriverCustomizations;
#[cfg(feature = "moonriver-native")]
impl ClientCustomizations for MoonriverCustomizations {
	fn first_block_number_compatible_with_ed25519_zebra() -> Option<u32> {
		Some(3_000_000)
	}
}

#[cfg(feature = "moonbase-native")]
pub struct MoonbaseCustomizations;
#[cfg(feature = "moonbase-native")]
impl ClientCustomizations for MoonbaseCustomizations {
	fn first_block_number_compatible_with_ed25519_zebra() -> Option<u32> {
		Some(3_000_000)
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
		self.chain_type() == sc_chain_spec::ChainType::Development
	}
}

pub fn frontier_database_dir(config: &Configuration, path: &str) -> std::path::PathBuf {
	config
		.base_path
		.config_dir(config.chain_spec.id())
		.join("frontier")
		.join(path)
}

// TODO This is copied from frontier. It should be imported instead after
// https://github.com/paritytech/frontier/issues/333 is solved
pub fn open_frontier_backend<C, BE>(
	client: Arc<C>,
	config: &Configuration,
	rpc_config: &RpcConfig,
) -> Result<fc_db::Backend<Block, C>, String>
where
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
	C: Send + Sync + 'static,
	C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
{
	let frontier_backend = match rpc_config.frontier_backend_config {
		FrontierBackendConfig::KeyValue => {
			fc_db::Backend::KeyValue(Arc::new(fc_db::kv::Backend::<Block, C>::new(
				client,
				&fc_db::kv::DatabaseSettings {
					source: match config.database {
						DatabaseSource::RocksDb { .. } => DatabaseSource::RocksDb {
							path: frontier_database_dir(config, "db"),
							cache_size: 0,
						},
						DatabaseSource::ParityDb { .. } => DatabaseSource::ParityDb {
							path: frontier_database_dir(config, "paritydb"),
						},
						DatabaseSource::Auto { .. } => DatabaseSource::Auto {
							rocksdb_path: frontier_database_dir(config, "db"),
							paritydb_path: frontier_database_dir(config, "paritydb"),
							cache_size: 0,
						},
						_ => {
							return Err(
								"Supported db sources: `rocksdb` | `paritydb` | `auto`".to_string()
							)
						}
					},
				},
			)?))
		}
		FrontierBackendConfig::Sql {
			pool_size,
			num_ops_timeout,
			thread_count,
			cache_size,
		} => {
			let overrides = Arc::new(StorageOverrideHandler::new(client.clone()));
			let sqlite_db_path = frontier_database_dir(config, "sql");
			std::fs::create_dir_all(&sqlite_db_path).expect("failed creating sql db directory");
			let backend = futures::executor::block_on(fc_db::sql::Backend::new(
				fc_db::sql::BackendConfig::Sqlite(fc_db::sql::SqliteBackendConfig {
					path: Path::new("sqlite:///")
						.join(sqlite_db_path)
						.join("frontier.db3")
						.to_str()
						.expect("frontier sql path error"),
					create_if_missing: true,
					thread_count: thread_count,
					cache_size: cache_size,
				}),
				pool_size,
				std::num::NonZeroU32::new(num_ops_timeout),
				overrides.clone(),
			))
			.unwrap_or_else(|err| panic!("failed creating sql backend: {:?}", err));
			fc_db::Backend::Sql(Arc::new(backend))
		}
	};

	Ok(frontier_backend)
}

use sp_runtime::{traits::BlakeTwo256, DigestItem, Percent};

pub const SOFT_DEADLINE_PERCENT: Percent = Percent::from_percent(100);

/// Builds a new object suitable for chain operations.
#[allow(clippy::type_complexity)]
pub fn new_chain_ops(
	config: &mut Configuration,
	rpc_config: &RpcConfig,
) -> Result<
	(
		Arc<Client>,
		Arc<FullBackend>,
		sc_consensus::BasicQueue<Block>,
		TaskManager,
	),
	ServiceError,
> {
	match &config.chain_spec {
		#[cfg(feature = "moonriver-native")]
		spec if spec.is_moonriver() => new_chain_ops_inner::<
			moonriver_runtime::RuntimeApi,
			MoonriverCustomizations,
		>(config, rpc_config),
		#[cfg(feature = "moonbeam-native")]
		spec if spec.is_moonbeam() => new_chain_ops_inner::<
			moonbeam_runtime::RuntimeApi,
			MoonbeamCustomizations,
		>(config, rpc_config),
		#[cfg(feature = "moonbase-native")]
		_ => new_chain_ops_inner::<moonbase_runtime::RuntimeApi, MoonbaseCustomizations>(
			config, rpc_config,
		),
		#[cfg(not(feature = "moonbase-native"))]
		_ => panic!("invalid chain spec"),
	}
}

#[allow(clippy::type_complexity)]
fn new_chain_ops_inner<RuntimeApi, Customizations>(
	config: &mut Configuration,
	rpc_config: &RpcConfig,
) -> Result<
	(
		Arc<Client>,
		Arc<FullBackend>,
		sc_consensus::BasicQueue<Block>,
		TaskManager,
	),
	ServiceError,
>
where
	Client: From<Arc<crate::FullClient<RuntimeApi>>>,
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	Customizations: ClientCustomizations + 'static,
{
	config.keystore = sc_service::config::KeystoreConfig::InMemory;
	let PartialComponents {
		client,
		backend,
		import_queue,
		task_manager,
		..
	} = new_partial::<RuntimeApi, Customizations>(config, rpc_config, config.chain_spec.is_dev())?;
	Ok((
		Arc::new(Client::from(client)),
		backend,
		import_queue,
		task_manager,
	))
}

// If we're using prometheus, use a registry with a prefix of `moonbeam`.
fn set_prometheus_registry(
	config: &mut Configuration,
	skip_prefix: bool,
) -> Result<(), ServiceError> {
	if let Some(PrometheusConfig { registry, .. }) = config.prometheus_config.as_mut() {
		let labels = hashmap! {
			"chain".into() => config.chain_spec.id().into(),
		};
		let prefix = if skip_prefix {
			None
		} else {
			Some("moonbeam".into())
		};

		*registry = Registry::new_custom(prefix, Some(labels))?;
	}

	Ok(())
}

/// Builds the PartialComponents for a parachain or development service
///
/// Use this function if you don't actually need the full service, but just the partial in order to
/// be able to perform chain operations.
#[allow(clippy::type_complexity)]
pub fn new_partial<RuntimeApi, Customizations>(
	config: &mut Configuration,
	rpc_config: &RpcConfig,
	dev_service: bool,
) -> PartialComponentsResult<FullClient<RuntimeApi>, FullBackend>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	Customizations: ClientCustomizations + 'static,
{
	set_prometheus_registry(config, rpc_config.no_prometheus_prefix)?;

	// Use ethereum style for subscription ids
	config.rpc_id_provider = Some(Box::new(fc_rpc::EthereumSubIdProvider));

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

	let heap_pages = config
		.default_heap_pages
		.map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| HeapAllocStrategy::Static {
			extra_pages: h as _,
		});
	let mut wasm_builder = WasmExecutor::builder()
		.with_execution_method(config.wasm_method)
		.with_onchain_heap_alloc_strategy(heap_pages)
		.with_offchain_heap_alloc_strategy(heap_pages)
		.with_ignore_onchain_heap_pages(true)
		.with_max_runtime_instances(config.max_runtime_instances)
		.with_runtime_cache_size(config.runtime_cache_size);

	if let Some(ref wasmtime_precompiled_path) = config.wasmtime_precompiled {
		wasm_builder = wasm_builder.with_wasmtime_precompiled_path(wasmtime_precompiled_path);
	}

	let executor = wasm_builder.build();

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts_record_import::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
			true,
		)?;

	if let Some(block_number) = Customizations::first_block_number_compatible_with_ed25519_zebra() {
		client
			.execution_extensions()
			.set_extensions_factory(sc_client_api::execution_extensions::ExtensionBeforeBlock::<
			Block,
			sp_io::UseDalekExt,
		>::new(block_number));
	}

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

	let frontier_backend = Arc::new(open_frontier_backend(client.clone(), config, rpc_config)?);
	let frontier_block_import = FrontierBlockImport::new(client.clone(), client.clone());

	let create_inherent_data_providers = move |_, _| async move {
		let time = sp_timestamp::InherentDataProvider::from_system_time();
		Ok((time,))
	};

	let (import_queue, block_import) = if dev_service {
		(
			nimbus_consensus::import_queue(
				client.clone(),
				frontier_block_import.clone(),
				create_inherent_data_providers,
				&task_manager.spawn_essential_handle(),
				config.prometheus_registry(),
			)?,
			BlockImportPipeline::Dev(frontier_block_import),
		)
	} else {
		let parachain_block_import = ParachainBlockImport::new_with_delayed_best_block(
			frontier_block_import,
			backend.clone(),
		);
		(
			nimbus_consensus::import_queue(
				client.clone(),
				parachain_block_import.clone(),
				create_inherent_data_providers,
				&task_manager.spawn_essential_handle(),
				config.prometheus_registry(),
			)?,
			BlockImportPipeline::Parachain(parachain_block_import),
		)
	};

	Ok(PartialComponents {
		backend,
		client,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		select_chain: maybe_select_chain,
		other: (
			block_import,
			filter_pool,
			telemetry,
			telemetry_worker_handle,
			frontier_backend,
			fee_history_cache,
		),
	})
}

async fn build_relay_chain_interface(
	polkadot_config: Configuration,
	parachain_config: &Configuration,
	telemetry_worker_handle: Option<TelemetryWorkerHandle>,
	task_manager: &mut TaskManager,
	collator_options: CollatorOptions,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> RelayChainResult<(
	Arc<(dyn RelayChainInterface + 'static)>,
	Option<CollatorPair>,
)> {
	if let cumulus_client_cli::RelayChainMode::ExternalRpc(rpc_target_urls) =
		collator_options.relay_chain_mode
	{
		build_minimal_relay_chain_node_with_rpc(polkadot_config, task_manager, rpc_target_urls)
			.await
	} else {
		build_inprocess_relay_chain(
			polkadot_config,
			parachain_config,
			telemetry_worker_handle,
			task_manager,
			hwbench,
		)
	}
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("🌗")]
async fn start_node_impl<RuntimeApi, Customizations, Net>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	para_id: ParaId,
	rpc_config: RpcConfig,
	async_backing: bool,
	block_authoring_duration: Duration,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<FullClient<RuntimeApi>>)>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	Customizations: ClientCustomizations + 'static,
	Net: NetworkBackend<Block, Hash>,
{
	let mut parachain_config = prepare_node_config(parachain_config);

	let params =
		new_partial::<RuntimeApi, Customizations>(&mut parachain_config, &rpc_config, false)?;
	let (
		block_import,
		filter_pool,
		mut telemetry,
		telemetry_worker_handle,
		frontier_backend,
		fee_history_cache,
	) = params.other;

	let client = params.client.clone();
	let backend = params.backend.clone();
	let mut task_manager = params.task_manager;

	let (relay_chain_interface, collator_key) = build_relay_chain_interface(
		polkadot_config,
		&parachain_config,
		telemetry_worker_handle,
		&mut task_manager,
		collator_options.clone(),
		hwbench.clone(),
	)
	.await
	.map_err(|e| sc_service::Error::Application(Box::new(e) as Box<_>))?;

	let force_authoring = parachain_config.force_authoring;
	let collator = parachain_config.role.is_authority();
	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let transaction_pool = params.transaction_pool.clone();
	let import_queue_service = params.import_queue.service();
	let net_config = FullNetworkConfiguration::<_, _, Net>::new(&parachain_config.network);

	let (network, system_rpc_tx, tx_handler_controller, start_network, sync_service) =
		cumulus_client_service::build_network(cumulus_client_service::BuildNetworkParams {
			parachain_config: &parachain_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue: params.import_queue,
			para_id,
			relay_chain_interface: relay_chain_interface.clone(),
			net_config,
			sybil_resistance_level: CollatorSybilResistance::Resistant,
		})
		.await?;

	let overrides = Arc::new(StorageOverrideHandler::new(client.clone()));
	let fee_history_limit = rpc_config.fee_history_limit;

	// Sinks for pubsub notifications.
	// Everytime a new subscription is created, a new mpsc channel is added to the sink pool.
	// The MappingSyncWorker sends through the channel on block import and the subscription emits a
	// notification to the subscriber on receiving a message through this channel.
	// This way we avoid race conditions when using native substrate block import notification
	// stream.
	let pubsub_notification_sinks: fc_mapping_sync::EthereumBlockNotificationSinks<
		fc_mapping_sync::EthereumBlockNotification<Block>,
	> = Default::default();
	let pubsub_notification_sinks = Arc::new(pubsub_notification_sinks);

	rpc::spawn_essential_tasks(
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
		sync_service.clone(),
		pubsub_notification_sinks.clone(),
	);

	let ethapi_cmd = rpc_config.ethapi.clone();
	let tracing_requesters =
		if ethapi_cmd.contains(&EthApiCmd::Debug) || ethapi_cmd.contains(&EthApiCmd::Trace) {
			rpc::tracing::spawn_tracing_tasks(
				&rpc_config,
				prometheus_registry.clone(),
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

	let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
		task_manager.spawn_handle(),
		overrides.clone(),
		rpc_config.eth_log_block_cache,
		rpc_config.eth_statuses_cache,
		prometheus_registry.clone(),
	));

	let rpc_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let network = network.clone();
		let sync = sync_service.clone();
		let filter_pool = filter_pool.clone();
		let frontier_backend = frontier_backend.clone();
		let backend = backend.clone();
		let ethapi_cmd = ethapi_cmd.clone();
		let max_past_logs = rpc_config.max_past_logs;
		let overrides = overrides.clone();
		let fee_history_cache = fee_history_cache.clone();
		let block_data_cache = block_data_cache.clone();
		let pubsub_notification_sinks = pubsub_notification_sinks.clone();

		let keystore = params.keystore_container.keystore();
		move |deny_unsafe, subscription_task_executor| {
			#[cfg(feature = "moonbase-native")]
			let forced_parent_hashes = {
				let mut forced_parent_hashes = BTreeMap::new();
				// Fixes for https://github.com/paritytech/frontier/pull/570
				// #1648995
				forced_parent_hashes.insert(
					H256::from_str(
						"0xa352fee3eef9c554a31ec0612af887796a920613358abf3353727760ea14207b",
					)
					.expect("must be valid hash"),
					H256::from_str(
						"0x0d0fd88778aec08b3a83ce36387dbf130f6f304fc91e9a44c9605eaf8a80ce5d",
					)
					.expect("must be valid hash"),
				);
				Some(forced_parent_hashes)
			};
			#[cfg(not(feature = "moonbase-native"))]
			let forced_parent_hashes = None;

			let deps = rpc::FullDeps {
				backend: backend.clone(),
				client: client.clone(),
				command_sink: None,
				deny_unsafe,
				ethapi_cmd: ethapi_cmd.clone(),
				filter_pool: filter_pool.clone(),
				frontier_backend: match &*frontier_backend {
					fc_db::Backend::KeyValue(b) => b.clone(),
					fc_db::Backend::Sql(b) => b.clone(),
				},
				graph: pool.pool().clone(),
				pool: pool.clone(),
				is_authority: collator,
				max_past_logs,
				fee_history_limit,
				fee_history_cache: fee_history_cache.clone(),
				network: network.clone(),
				sync: sync.clone(),
				dev_rpc_data: None,
				block_data_cache: block_data_cache.clone(),
				overrides: overrides.clone(),
				forced_parent_hashes,
			};
			let pending_consensus_data_provider = Box::new(PendingConsensusDataProvider::new(
				client.clone(),
				keystore.clone(),
			));
			if ethapi_cmd.contains(&EthApiCmd::Debug) || ethapi_cmd.contains(&EthApiCmd::Trace) {
				rpc::create_full(
					deps,
					subscription_task_executor,
					Some(crate::rpc::TracingConfig {
						tracing_requesters: tracing_requesters.clone(),
						trace_filter_max_count: rpc_config.ethapi_trace_max_count,
					}),
					pubsub_notification_sinks.clone(),
					pending_consensus_data_provider,
				)
				.map_err(Into::into)
			} else {
				rpc::create_full(
					deps,
					subscription_task_executor,
					None,
					pubsub_notification_sinks.clone(),
					pending_consensus_data_provider,
				)
				.map_err(Into::into)
			}
		}
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		rpc_builder: Box::new(rpc_builder),
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: params.keystore_container.keystore(),
		backend: backend.clone(),
		network: network.clone(),
		sync_service: sync_service.clone(),
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	if let Some(hwbench) = hwbench {
		sc_sysinfo::print_hwbench(&hwbench);

		if let Some(ref mut telemetry) = telemetry {
			let telemetry_handle = telemetry.handle();
			task_manager.spawn_handle().spawn(
				"telemetry_hwbench",
				None,
				sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
			);
		}
	}

	let announce_block = {
		let sync_service = sync_service.clone();
		Arc::new(move |hash, data| sync_service.announce_block(hash, data))
	};

	let relay_chain_slot_duration = Duration::from_secs(6);
	let overseer_handle = relay_chain_interface
		.overseer_handle()
		.map_err(|e| sc_service::Error::Application(Box::new(e)))?;

	start_relay_chain_tasks(StartRelayChainTasksParams {
		client: client.clone(),
		announce_block: announce_block.clone(),
		para_id,
		relay_chain_interface: relay_chain_interface.clone(),
		task_manager: &mut task_manager,
		da_recovery_profile: if collator {
			DARecoveryProfile::Collator
		} else {
			DARecoveryProfile::FullNode
		},
		import_queue: import_queue_service,
		relay_chain_slot_duration,
		recovery_handle: Box::new(overseer_handle.clone()),
		sync_service: sync_service.clone(),
	})?;

	let BlockImportPipeline::Parachain(block_import) = block_import else {
		return Err(sc_service::Error::Other(
			"Block import pipeline is not for parachain".into(),
		));
	};

	if collator {
		start_consensus::<RuntimeApi, _>(
			async_backing,
			backend.clone(),
			client.clone(),
			block_import,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|t| t.handle()),
			&task_manager,
			relay_chain_interface.clone(),
			transaction_pool,
			params.keystore_container.keystore(),
			para_id,
			collator_key.expect("Command line arguments do not allow this. qed"),
			overseer_handle,
			announce_block,
			force_authoring,
			relay_chain_slot_duration,
			block_authoring_duration,
			sync_service.clone(),
		)?;
		/*let parachain_consensus = build_consensus(
			client.clone(),
			backend,
			block_import,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|t| t.handle()),
			&task_manager,
			relay_chain_interface.clone(),
			transaction_pool,
			sync_service.clone(),
			params.keystore_container.keystore(),
			force_authoring,
		)?;

		let spawner = task_manager.spawn_handle();

		let params = StartCollatorParams {
			para_id,
			block_status: client.clone(),
			announce_block,
			client: client.clone(),
			task_manager: &mut task_manager,
			relay_chain_interface,
			spawner,
			parachain_consensus,
			import_queue: import_queue_service,
			recovery_handle: Box::new(overseer_handle),
			collator_key: collator_key.ok_or(sc_service::error::Error::Other(
				"Collator Key is None".to_string(),
			))?,
			relay_chain_slot_duration,
			sync_service,
		};

		#[allow(deprecated)]
		start_collator(params).await?;*/
	}

	start_network.start_network();

	Ok((task_manager, client))
}

fn start_consensus<RuntimeApi, SO>(
	async_backing: bool,
	backend: Arc<FullBackend>,
	client: Arc<FullClient<RuntimeApi>>,
	block_import: ParachainBlockImport<FullClient<RuntimeApi>, FullBackend>,
	prometheus_registry: Option<&Registry>,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
	relay_chain_interface: Arc<dyn RelayChainInterface>,
	transaction_pool: Arc<sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi>>>,
	keystore: KeystorePtr,
	para_id: ParaId,
	collator_key: CollatorPair,
	overseer_handle: OverseerHandle,
	announce_block: Arc<dyn Fn(Hash, Option<Vec<u8>>) + Send + Sync>,
	force_authoring: bool,
	relay_chain_slot_duration: Duration,
	block_authoring_duration: Duration,
	sync_oracle: SO,
) -> Result<(), sc_service::Error>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	sc_client_api::StateBackendFor<FullBackend, Block>: sc_client_api::StateBackend<BlakeTwo256>,
	SO: SyncOracle + Send + Sync + Clone + 'static,
{
	let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
		task_manager.spawn_handle(),
		client.clone(),
		transaction_pool,
		prometheus_registry,
		telemetry.clone(),
	);

	let proposer = Proposer::new(proposer_factory);

	let collator_service = CollatorService::new(
		client.clone(),
		Arc::new(task_manager.spawn_handle()),
		announce_block,
		client.clone(),
	);

	let create_inherent_data_providers = |_, _| async move {
		let time = sp_timestamp::InherentDataProvider::from_system_time();

		let author = nimbus_primitives::InherentDataProvider;

		let randomness = session_keys_primitives::InherentDataProvider;

		Ok((time, author, randomness))
	};

	let client_clone = client.clone();
	let keystore_clone = keystore.clone();
	let maybe_provide_vrf_digest =
		move |nimbus_id: NimbusId, parent: Hash| -> Option<sp_runtime::generic::DigestItem> {
			moonbeam_vrf::vrf_pre_digest::<Block, FullClient<RuntimeApi>>(
				&client_clone,
				&keystore_clone,
				nimbus_id,
				parent,
			)
		};

	if async_backing {
		log::info!("Collator started with asynchronous backing.");
		let client_clone = client.clone();
		let code_hash_provider = move |block_hash| {
			client_clone
				.code_at(block_hash)
				.ok()
				.map(polkadot_primitives::ValidationCode)
				.map(|c| c.hash())
		};
		task_manager.spawn_essential_handle().spawn(
			"nimbus",
			None,
			nimbus_consensus::collators::lookahead::run::<
				Block,
				_,
				_,
				_,
				FullBackend,
				_,
				_,
				_,
				_,
				_,
				_,
			>(nimbus_consensus::collators::lookahead::Params {
				additional_digests_provider: maybe_provide_vrf_digest,
				additional_relay_keys: vec![
					moonbeam_core_primitives::well_known_relay_keys::TIMESTAMP_NOW.to_vec(),
				],
				authoring_duration: block_authoring_duration,
				block_import,
				code_hash_provider,
				collator_key,
				collator_service,
				create_inherent_data_providers,
				force_authoring,
				keystore,
				overseer_handle,
				para_backend: backend,
				para_client: client,
				para_id,
				proposer,
				relay_chain_slot_duration,
				relay_client: relay_chain_interface,
				slot_duration: None,
				sync_oracle,
				reinitialize: false,
			}),
		);
	} else {
		log::info!("Collator started without asynchronous backing.");
		task_manager.spawn_essential_handle().spawn(
			"nimbus",
			None,
			nimbus_consensus::collators::basic::run::<Block, _, _, FullBackend, _, _, _, _, _>(
				nimbus_consensus::collators::basic::Params {
					additional_digests_provider: maybe_provide_vrf_digest,
					additional_relay_keys: vec![
						moonbeam_core_primitives::well_known_relay_keys::TIMESTAMP_NOW.to_vec(),
					],
					//authoring_duration: Duration::from_millis(500),
					block_import,
					collator_key,
					collator_service,
					create_inherent_data_providers,
					force_authoring,
					keystore,
					overseer_handle,
					para_id,
					para_client: client,
					proposer,
					relay_client: relay_chain_interface,
				},
			),
		);
	};

	Ok(())
}

/// Start a normal parachain node.
// Rustfmt wants to format the closure with space identation.
#[rustfmt::skip]
pub async fn start_node<RuntimeApi, Customizations>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	para_id: ParaId,
	rpc_config: RpcConfig,
	async_backing: bool,
	block_authoring_duration: Duration,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<FullClient<RuntimeApi>>)>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection,
	Customizations: ClientCustomizations + 'static,
{
	start_node_impl::<RuntimeApi, Customizations, sc_network::NetworkWorker<_, _>>(
		parachain_config,
		polkadot_config,
		collator_options,
		para_id,
		rpc_config,
		async_backing,
		block_authoring_duration,
		hwbench,
	)
	.await
}

/// Builds a new development service. This service uses manual seal, and mocks
/// the parachain inherent.
pub async fn new_dev<RuntimeApi, Customizations, Net>(
	mut config: Configuration,
	para_id: Option<u32>,
	_author_id: Option<NimbusId>,
	sealing: moonbeam_cli_opt::Sealing,
	rpc_config: RpcConfig,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> Result<TaskManager, ServiceError>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	Customizations: ClientCustomizations + 'static,
	Net: NetworkBackend<Block, Hash>,
{
	use async_io::Timer;
	use futures::Stream;
	use sc_consensus_manual_seal::{run_manual_seal, EngineCommand, ManualSealParams};

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
				block_import_pipeline,
				filter_pool,
				mut telemetry,
				_telemetry_worker_handle,
				frontier_backend,
				fee_history_cache,
			),
	} = new_partial::<RuntimeApi, Customizations>(&mut config, &rpc_config, true)?;

	let block_import = if let BlockImportPipeline::Dev(block_import) = block_import_pipeline {
		block_import
	} else {
		return Err(ServiceError::Other(
			"Block import pipeline is not dev".to_string(),
		));
	};

	let net_config = FullNetworkConfiguration::<_, _, Net>::new(&config.network);

	let metrics = Net::register_notification_metrics(
		config.prometheus_config.as_ref().map(|cfg| &cfg.registry),
	);

	let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_params: None,
			net_config,
			block_relay: None,
			metrics,
		})?;

	if config.offchain_worker.enabled {
		task_manager.spawn_handle().spawn(
			"offchain-workers-runner",
			"offchain-work",
			sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
				runtime_api_provider: client.clone(),
				keystore: Some(keystore_container.keystore()),
				offchain_db: backend.offchain_storage(),
				transaction_pool: Some(OffchainTransactionPoolFactory::new(
					transaction_pool.clone(),
				)),
				network_provider: Arc::new(network.clone()),
				is_validator: config.role.is_authority(),
				enable_http_requests: true,
				custom_extensions: move |_| vec![],
			})
			.run(client.clone(), task_manager.spawn_handle())
			.boxed(),
		);
	}

	let prometheus_registry = config.prometheus_registry().cloned();
	let overrides = Arc::new(StorageOverrideHandler::new(client.clone()));
	let fee_history_limit = rpc_config.fee_history_limit;
	let mut command_sink = None;
	let mut dev_rpc_data = None;
	let collator = config.role.is_authority();

	if collator {
		let mut env = sc_basic_authorship::ProposerFactory::with_proof_recording(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);
		env.set_soft_deadline(SOFT_DEADLINE_PERCENT);
		// TODO: Need to cherry-pick
		//
		// https://github.com/moonbeam-foundation/substrate/commit/
		// d59476b362e38071d44d32c98c32fb35fd280930#diff-a1c022c97c7f9200cab161864c
		// 06d204f0c8b689955e42177731e232115e9a6f
		//
		// env.enable_ensure_proof_size_limit_after_each_extrinsic();

		let commands_stream: Box<dyn Stream<Item = EngineCommand<H256>> + Send + Sync + Unpin> =
			match sealing {
				moonbeam_cli_opt::Sealing::Instant => {
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
				moonbeam_cli_opt::Sealing::Manual => {
					let (sink, stream) = futures::channel::mpsc::channel(1000);
					// Keep a reference to the other end of the channel. It goes to the RPC.
					command_sink = Some(sink);
					Box::new(stream)
				}
				moonbeam_cli_opt::Sealing::Interval(millis) => Box::new(StreamExt::map(
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
		let additional_relay_offset = Arc::new(std::sync::atomic::AtomicU32::new(0));
		dev_rpc_data = Some((
			downward_xcm_sender,
			hrmp_xcm_sender,
			additional_relay_offset.clone(),
		));

		let client_clone = client.clone();
		let keystore_clone = keystore_container.keystore().clone();
		let maybe_provide_vrf_digest =
			move |nimbus_id: NimbusId, parent: Hash| -> Option<sp_runtime::generic::DigestItem> {
				moonbeam_vrf::vrf_pre_digest::<Block, FullClient<RuntimeApi>>(
					&client_clone,
					&keystore_clone,
					nimbus_id,
					parent,
				)
			};

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
					keystore: keystore_container.keystore(),
					client: client.clone(),
					additional_digests_provider: maybe_provide_vrf_digest,
					_phantom: Default::default(),
				})),
				create_inherent_data_providers: move |block: H256, ()| {
					let maybe_current_para_block = client_set_aside_for_cidp.number(block);
					let maybe_current_para_head = client_set_aside_for_cidp.expect_header(block);
					let downward_xcm_receiver = downward_xcm_receiver.clone();
					let hrmp_xcm_receiver = hrmp_xcm_receiver.clone();
					let additional_relay_offset = additional_relay_offset.clone();
					let relay_slot_key = well_known_keys::CURRENT_SLOT.to_vec();

					let client_for_xcm = client_set_aside_for_cidp.clone();
					async move {
						let time = MockTimestampInherentDataProvider;

						let current_para_block = maybe_current_para_block?
							.ok_or(sp_blockchain::Error::UnknownBlock(block.to_string()))?;

						let current_para_block_head = Some(polkadot_primitives::HeadData(
							maybe_current_para_head?.encode(),
						));

						// Get the mocked timestamp
						let timestamp = TIMESTAMP.load(Ordering::SeqCst);
						// Calculate mocked slot number (should be consecutively 1, 2, ...)
						let slot = timestamp.saturating_div(RELAY_CHAIN_SLOT_DURATION_MILLIS);

						let additional_key_values = Some(vec![
							(
								moonbeam_core_primitives::well_known_relay_keys::TIMESTAMP_NOW
									.to_vec(),
								sp_timestamp::Timestamp::current().encode(),
							),
							(relay_slot_key, Slot::from(slot).encode()),
						]);

						let mocked_parachain = MockValidationDataInherentDataProvider {
							current_para_block,
							para_id: para_id.unwrap().into(),
							current_para_block_head,
							relay_offset: 1000
								+ additional_relay_offset.load(std::sync::atomic::Ordering::SeqCst),
							relay_blocks_per_para_block: 2,
							// TODO: Recheck
							para_blocks_per_relay_epoch: 10,
							relay_randomness_config: (),
							xcm_config: MockXcmConfig::new(
								&*client_for_xcm,
								block,
								Default::default(),
							),
							raw_downward_messages: downward_xcm_receiver.drain().collect(),
							raw_horizontal_messages: hrmp_xcm_receiver.drain().collect(),
							additional_key_values,
						};

						let randomness = session_keys_primitives::InherentDataProvider;

						Ok((time, mocked_parachain, randomness))
					}
				},
			}),
		);
	}

	// Sinks for pubsub notifications.
	// Everytime a new subscription is created, a new mpsc channel is added to the sink pool.
	// The MappingSyncWorker sends through the channel on block import and the subscription emits a
	// notification to the subscriber on receiving a message through this channel.
	// This way we avoid race conditions when using native substrate block import notification
	// stream.
	let pubsub_notification_sinks: fc_mapping_sync::EthereumBlockNotificationSinks<
		fc_mapping_sync::EthereumBlockNotification<Block>,
	> = Default::default();
	let pubsub_notification_sinks = Arc::new(pubsub_notification_sinks);

	rpc::spawn_essential_tasks(
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
		sync_service.clone(),
		pubsub_notification_sinks.clone(),
	);
	let ethapi_cmd = rpc_config.ethapi.clone();
	let tracing_requesters =
		if ethapi_cmd.contains(&EthApiCmd::Debug) || ethapi_cmd.contains(&EthApiCmd::Trace) {
			rpc::tracing::spawn_tracing_tasks(
				&rpc_config,
				prometheus_registry.clone(),
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

	let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
		task_manager.spawn_handle(),
		overrides.clone(),
		rpc_config.eth_log_block_cache,
		rpc_config.eth_statuses_cache,
		prometheus_registry,
	));

	let rpc_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let backend = backend.clone();
		let network = network.clone();
		let sync = sync_service.clone();
		let ethapi_cmd = ethapi_cmd.clone();
		let max_past_logs = rpc_config.max_past_logs;
		let overrides = overrides.clone();
		let fee_history_cache = fee_history_cache.clone();
		let block_data_cache = block_data_cache.clone();
		let pubsub_notification_sinks = pubsub_notification_sinks.clone();

		let keystore = keystore_container.keystore();
		move |deny_unsafe, subscription_task_executor| {
			let deps = rpc::FullDeps {
				backend: backend.clone(),
				client: client.clone(),
				command_sink: command_sink.clone(),
				deny_unsafe,
				ethapi_cmd: ethapi_cmd.clone(),
				filter_pool: filter_pool.clone(),
				frontier_backend: match &*frontier_backend {
					fc_db::Backend::KeyValue(b) => b.clone(),
					fc_db::Backend::Sql(b) => b.clone(),
				},
				graph: pool.pool().clone(),
				pool: pool.clone(),
				is_authority: collator,
				max_past_logs,
				fee_history_limit,
				fee_history_cache: fee_history_cache.clone(),
				network: network.clone(),
				sync: sync.clone(),
				dev_rpc_data: dev_rpc_data.clone(),
				overrides: overrides.clone(),
				block_data_cache: block_data_cache.clone(),
				forced_parent_hashes: None,
			};

			let pending_consensus_data_provider = Box::new(PendingConsensusDataProvider::new(
				client.clone(),
				keystore.clone(),
			));
			if ethapi_cmd.contains(&EthApiCmd::Debug) || ethapi_cmd.contains(&EthApiCmd::Trace) {
				rpc::create_full(
					deps,
					subscription_task_executor,
					Some(crate::rpc::TracingConfig {
						tracing_requesters: tracing_requesters.clone(),
						trace_filter_max_count: rpc_config.ethapi_trace_max_count,
					}),
					pubsub_notification_sinks.clone(),
					pending_consensus_data_provider,
				)
				.map_err(Into::into)
			} else {
				rpc::create_full(
					deps,
					subscription_task_executor,
					None,
					pubsub_notification_sinks.clone(),
					pending_consensus_data_provider,
				)
				.map_err(Into::into)
			}
		}
	};

	let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network,
		client,
		keystore: keystore_container.keystore(),
		task_manager: &mut task_manager,
		transaction_pool,
		rpc_builder: Box::new(rpc_builder),
		backend,
		system_rpc_tx,
		sync_service: sync_service.clone(),
		config,
		tx_handler_controller,
		telemetry: None,
	})?;

	if let Some(hwbench) = hwbench {
		sc_sysinfo::print_hwbench(&hwbench);

		if let Some(ref mut telemetry) = telemetry {
			let telemetry_handle = telemetry.handle();
			task_manager.spawn_handle().spawn(
				"telemetry_hwbench",
				None,
				sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
			);
		}
	}

	log::info!("Development Service Ready");

	network_starter.start_network();
	Ok(task_manager)
}

#[cfg(test)]
mod tests {
	use jsonrpsee::server::BatchRequestConfig;
	use moonbase_runtime::{currency::UNIT, AccountId};
	use prometheus::{proto::LabelPair, Counter};
	use sc_network::config::NetworkConfiguration;
	use sc_service::ChainType;
	use sc_service::{
		config::{BasePath, DatabaseSource, KeystoreConfig},
		Configuration, Role,
	};
	use std::path::Path;
	use std::str::FromStr;

	use crate::chain_spec::moonbase::{testnet_genesis, ChainSpec};
	use crate::chain_spec::Extensions;

	use super::*;

	#[test]
	fn test_set_prometheus_registry_uses_moonbeam_prefix() {
		let counter_name = "my_counter";
		let expected_metric_name = "moonbeam_my_counter";
		let counter = Box::new(Counter::new(counter_name, "foobar").unwrap());
		let mut config = Configuration {
			prometheus_config: Some(PrometheusConfig::new_with_default_registry(
				"0.0.0.0:8080".parse().unwrap(),
				"".into(),
			)),
			..test_config("test")
		};

		set_prometheus_registry(&mut config, false).unwrap();
		// generate metric
		let reg = config.prometheus_registry().unwrap();
		reg.register(counter.clone()).unwrap();
		counter.inc();

		let actual_metric_name = reg.gather().first().unwrap().get_name().to_string();
		assert_eq!(actual_metric_name.as_str(), expected_metric_name);
	}

	#[test]
	fn test_set_prometheus_registry_skips_moonbeam_prefix() {
		let counter_name = "my_counter";
		let counter = Box::new(Counter::new(counter_name, "foobar").unwrap());
		let mut config = Configuration {
			prometheus_config: Some(PrometheusConfig::new_with_default_registry(
				"0.0.0.0:8080".parse().unwrap(),
				"".into(),
			)),
			..test_config("test")
		};

		set_prometheus_registry(&mut config, true).unwrap();
		// generate metric
		let reg = config.prometheus_registry().unwrap();
		reg.register(counter.clone()).unwrap();
		counter.inc();

		let actual_metric_name = reg.gather().first().unwrap().get_name().to_string();
		assert_eq!(actual_metric_name.as_str(), counter_name);
	}

	#[test]
	fn test_set_prometheus_registry_adds_chain_id_as_label() {
		let input_chain_id = "moonriver";

		let mut expected_label = LabelPair::default();
		expected_label.set_name("chain".to_owned());
		expected_label.set_value("moonriver".to_owned());
		let expected_chain_label = Some(expected_label);

		let counter = Box::new(Counter::new("foo", "foobar").unwrap());
		let mut config = Configuration {
			prometheus_config: Some(PrometheusConfig::new_with_default_registry(
				"0.0.0.0:8080".parse().unwrap(),
				"".into(),
			)),
			..test_config(input_chain_id)
		};

		set_prometheus_registry(&mut config, false).unwrap();
		// generate metric
		let reg = config.prometheus_registry().unwrap();
		reg.register(counter.clone()).unwrap();
		counter.inc();

		let actual_chain_label = reg
			.gather()
			.first()
			.unwrap()
			.get_metric()
			.first()
			.unwrap()
			.get_label()
			.into_iter()
			.find(|x| x.get_name() == "chain")
			.cloned();

		assert_eq!(actual_chain_label, expected_chain_label);
	}

	#[test]
	fn dalek_does_not_panic() {
		use futures::executor::block_on;
		use sc_block_builder::BlockBuilderBuilder;
		use sc_client_db::{Backend, BlocksPruning, DatabaseSettings, DatabaseSource, PruningMode};
		use sp_api::ProvideRuntimeApi;
		use sp_consensus::BlockOrigin;
		use substrate_test_runtime::TestAPI;
		use substrate_test_runtime_client::runtime::Block;
		use substrate_test_runtime_client::{
			ClientBlockImportExt, TestClientBuilder, TestClientBuilderExt,
		};

		fn zero_ed_pub() -> sp_core::ed25519::Public {
			sp_core::ed25519::Public::default()
		}

		// This is an invalid signature
		// this breaks after ed25519 1.3. It makes the signature panic at creation
		// This test ensures we should never panic
		fn invalid_sig() -> sp_core::ed25519::Signature {
			let signature = hex_literal::hex!(
				"a25b94f9c64270fdfffa673f11cfe961633e3e4972e6940a3cf
		7351dd90b71447041a83583a52cee1cf21b36ba7fd1d0211dca58b48d997fc78d9bc82ab7a38e"
			);
			sp_core::ed25519::Signature::from_raw(signature[0..64].try_into().unwrap())
		}

		let tmp = tempfile::tempdir().unwrap();
		let backend = Arc::new(
			Backend::new(
				DatabaseSettings {
					trie_cache_maximum_size: Some(1 << 20),
					state_pruning: Some(PruningMode::ArchiveAll),
					blocks_pruning: BlocksPruning::KeepAll,
					source: DatabaseSource::RocksDb {
						path: tmp.path().into(),
						cache_size: 1024,
					},
				},
				u64::MAX,
			)
			.unwrap(),
		);
		let mut client = TestClientBuilder::with_backend(backend).build();

		client
			.execution_extensions()
			.set_extensions_factory(sc_client_api::execution_extensions::ExtensionBeforeBlock::<
			Block,
			sp_io::UseDalekExt,
		>::new(1));

		let a1 = BlockBuilderBuilder::new(&client)
			.on_parent_block(client.chain_info().genesis_hash)
			.with_parent_block_number(0)
			// Enable proof recording if required. This call is optional.
			.enable_proof_recording()
			.build()
			.unwrap()
			.build()
			.unwrap()
			.block;

		block_on(client.import(BlockOrigin::NetworkInitialSync, a1.clone())).unwrap();

		// On block zero it will use dalek
		// shouldnt panic on importing invalid sig
		assert!(!client
			.runtime_api()
			.verify_ed25519(
				client.chain_info().genesis_hash,
				invalid_sig(),
				zero_ed_pub(),
				vec![]
			)
			.unwrap());
	}

	fn test_config(chain_id: &str) -> Configuration {
		let network_config = NetworkConfiguration::new("", "", Default::default(), None);
		let runtime = tokio::runtime::Runtime::new().expect("failed creating tokio runtime");
		let spec = ChainSpec::builder(&[0u8], Extensions::default())
			.with_name("test")
			.with_id(chain_id)
			.with_chain_type(ChainType::Local)
			.with_genesis_config(testnet_genesis(
				AccountId::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap(),
				vec![],
				vec![],
				vec![],
				vec![],
				vec![],
				1000 * UNIT,
				ParaId::new(0),
				0,
			))
			.build();

		Configuration {
			impl_name: String::from("test-impl"),
			impl_version: String::from("0.1"),
			role: Role::Full,
			tokio_handle: runtime.handle().clone(),
			transaction_pool: Default::default(),
			network: network_config,
			keystore: KeystoreConfig::Path {
				path: "key".into(),
				password: None,
			},
			database: DatabaseSource::RocksDb {
				path: "db".into(),
				cache_size: 128,
			},
			trie_cache_maximum_size: Some(16777216),
			state_pruning: Default::default(),
			blocks_pruning: sc_service::BlocksPruning::KeepAll,
			chain_spec: Box::new(spec),
			wasm_method: Default::default(),
			wasm_runtime_overrides: Default::default(),
			rpc_id_provider: None,
			rpc_max_connections: Default::default(),
			rpc_cors: None,
			rpc_methods: Default::default(),
			rpc_max_request_size: Default::default(),
			rpc_max_response_size: Default::default(),
			rpc_max_subs_per_conn: Default::default(),
			rpc_addr: None,
			rpc_port: Default::default(),
			rpc_message_buffer_capacity: Default::default(),
			data_path: Default::default(),
			prometheus_config: None,
			telemetry_endpoints: None,
			default_heap_pages: None,
			offchain_worker: Default::default(),
			force_authoring: false,
			disable_grandpa: false,
			dev_key_seed: None,
			tracing_targets: None,
			tracing_receiver: Default::default(),
			max_runtime_instances: 8,
			announce_block: true,
			base_path: BasePath::new(Path::new("")),
			informant_output_format: Default::default(),
			wasmtime_precompiled: None,
			runtime_cache_size: 2,
			rpc_rate_limit: Default::default(),
			rpc_rate_limit_whitelisted_ips: vec![],
			rpc_batch_config: BatchRequestConfig::Unlimited,
			rpc_rate_limit_trust_proxy_headers: false,
		}
	}
}

struct PendingConsensusDataProvider<Client>
where
	Client: HeaderBackend<Block> + sp_api::ProvideRuntimeApi<Block> + Send + Sync,
	Client::Api: VrfApi<Block>,
{
	client: Arc<Client>,
	keystore: Arc<dyn Keystore>,
}

impl<Client> PendingConsensusDataProvider<Client>
where
	Client: HeaderBackend<Block> + sp_api::ProvideRuntimeApi<Block> + Send + Sync,
	Client::Api: VrfApi<Block>,
{
	pub fn new(client: Arc<Client>, keystore: Arc<dyn Keystore>) -> Self {
		Self { client, keystore }
	}
}

impl<Client> fc_rpc::pending::ConsensusDataProvider<Block> for PendingConsensusDataProvider<Client>
where
	Client: HeaderBackend<Block> + sp_api::ProvideRuntimeApi<Block> + Send + Sync,
	Client::Api: VrfApi<Block>,
{
	fn create_digest(
		&self,
		parent: &Header,
		_data: &sp_inherents::InherentData,
	) -> Result<sp_runtime::Digest, sp_inherents::Error> {
		let hash = parent.hash();
		// Get the digest from the best block header.
		let mut digest = self
			.client
			.header(hash)
			.map_err(|e| sp_inherents::Error::Application(Box::new(e)))?
			.ok_or(sp_inherents::Error::Application(
				"Best block header should be present".into(),
			))?
			.digest;
		// Get the nimbus id from the digest.
		let nimbus_id = digest
			.logs
			.iter()
			.find_map(|x| {
				if let DigestItem::PreRuntime(nimbus_primitives::NIMBUS_ENGINE_ID, nimbus_id) = x {
					Some(NimbusId::from_slice(nimbus_id.as_slice()).map_err(|_| {
						sp_inherents::Error::Application(
							"Nimbus pre-runtime digest should be valid".into(),
						)
					}))
				} else {
					None
				}
			})
			.ok_or(sp_inherents::Error::Application(
				"Nimbus pre-runtime digest should be present".into(),
			))??;
		// Remove the old VRF digest.
		let pos = digest.logs.iter().position(|x| {
			matches!(
				x,
				DigestItem::PreRuntime(session_keys_primitives::VRF_ENGINE_ID, _)
			)
		});
		if let Some(pos) = pos {
			digest.logs.remove(pos);
		}
		// Create the VRF digest.
		let vrf_digest = VrfDigestsProvider::new(self.client.clone(), self.keystore.clone())
			.provide_digests(nimbus_id, hash);
		// Append the VRF digest to the digest.
		digest.logs.extend(vrf_digest);
		Ok(digest)
	}
}
