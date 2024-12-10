// Copyright 2024 Moonbeam foundation
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
	lazy_loading, open_frontier_backend, rpc, set_prometheus_registry, BlockImportPipeline,
	ClientCustomizations, FrontierBlockImport, HostFunctions, PartialComponentsResult,
	PendingConsensusDataProvider, RuntimeApiCollection, SOFT_DEADLINE_PERCENT,
};
use cumulus_client_parachain_inherent::{MockValidationDataInherentDataProvider, MockXcmConfig};
use cumulus_primitives_core::{relay_chain, BlockT, ParaId};
use cumulus_primitives_parachain_inherent::ParachainInherentData;
use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
use fc_rpc::StorageOverrideHandler;
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use futures::{FutureExt, StreamExt};
use moonbeam_cli_opt::{EthApi as EthApiCmd, LazyLoadingConfig, RpcConfig};
use moonbeam_core_primitives::{Block, Hash};
use nimbus_consensus::NimbusManualSealConsensusDataProvider;
use nimbus_primitives::NimbusId;
use parity_scale_codec::Encode;
use polkadot_primitives::{
	AbridgedHostConfiguration, AsyncBackingParams, PersistedValidationData, Slot, UpgradeGoAhead,
};
use sc_chain_spec::{get_extension, BuildGenesisBlock, GenesisBlockBuilder};
use sc_client_api::{Backend, BadBlocks, ExecutorProvider, ForkBlocks, StorageProvider};
use sc_executor::{HeapAllocStrategy, RuntimeVersionOf, WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY};
use sc_network::config::FullNetworkConfiguration;
use sc_network::NetworkBackend;
use sc_network_common::sync::SyncMode;
use sc_service::{
	error::Error as ServiceError, ClientConfig, Configuration, Error, KeystoreContainer,
	LocalCallExecutor, PartialComponents, TaskManager,
};
use sc_telemetry::{TelemetryHandle, TelemetryWorker};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_api::ConstructRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::traits::CodeExecutor;
use sp_core::{twox_128, H256};
use sp_runtime::traits::NumberFor;
use sp_storage::StorageKey;
use std::collections::BTreeMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub mod backend;
pub mod call_executor;
mod client;
mod helpers;
mod lock;
mod manual_sealing;
mod state_overrides;

pub const LAZY_LOADING_LOG_TARGET: &'static str = "lazy-loading";

/// Lazy loading client type.
pub type TLazyLoadingClient<TBl, TRtApi, TExec> = sc_service::client::Client<
	TLazyLoadingBackend<TBl>,
	TLazyLoadingCallExecutor<TBl, TExec>,
	TBl,
	TRtApi,
>;

/// Lazy loading client backend type.
pub type TLazyLoadingBackend<TBl> = backend::Backend<TBl>;

/// Lazy loading client call executor type.
pub type TLazyLoadingCallExecutor<TBl, TExec> = call_executor::LazyLoadingCallExecutor<
	TBl,
	LocalCallExecutor<TBl, TLazyLoadingBackend<TBl>, TExec>,
>;

/// Lazy loading parts type.
pub type TLazyLoadingParts<TBl, TRtApi, TExec> = (
	TLazyLoadingClient<TBl, TRtApi, TExec>,
	Arc<TLazyLoadingBackend<TBl>>,
	KeystoreContainer,
	TaskManager,
);

type LazyLoadingClient<RuntimeApi> =
	TLazyLoadingClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>;
type LazyLoadingBackend = TLazyLoadingBackend<Block>;

/// Create the initial parts of a lazy loading node.
pub fn new_lazy_loading_parts<TBl, TRtApi, TExec>(
	config: &mut Configuration,
	lazy_loading_config: &LazyLoadingConfig,
	telemetry: Option<TelemetryHandle>,
	executor: TExec,
) -> Result<TLazyLoadingParts<TBl, TRtApi, TExec>, Error>
where
	TBl: BlockT + sp_runtime::DeserializeOwned,
	TBl::Hash: From<H256>,
	TExec: CodeExecutor + RuntimeVersionOf + Clone,
{
	let backend = backend::new_lazy_loading_backend(config, &lazy_loading_config)?;

	let genesis_block_builder = GenesisBlockBuilder::new(
		config.chain_spec.as_storage_builder(),
		!config.no_genesis(),
		backend.clone(),
		executor.clone(),
	)?;

	new_lazy_loading_parts_with_genesis_builder(
		config,
		telemetry,
		executor,
		backend,
		genesis_block_builder,
	)
}

/// Create the initial parts of a lazy loading node.
pub fn new_lazy_loading_parts_with_genesis_builder<TBl, TRtApi, TExec, TBuildGenesisBlock>(
	config: &Configuration,
	telemetry: Option<TelemetryHandle>,
	executor: TExec,
	backend: Arc<TLazyLoadingBackend<TBl>>,
	genesis_block_builder: TBuildGenesisBlock,
) -> Result<TLazyLoadingParts<TBl, TRtApi, TExec>, Error>
where
	TBl: BlockT + sp_runtime::DeserializeOwned,
	TBl::Hash: From<H256>,
	TExec: CodeExecutor + RuntimeVersionOf + Clone,
	TBuildGenesisBlock:
		BuildGenesisBlock<
			TBl,
			BlockImportOperation = <TLazyLoadingBackend<TBl> as sc_client_api::backend::Backend<
				TBl,
			>>::BlockImportOperation,
		>,
{
	let keystore_container = KeystoreContainer::new(&config.keystore)?;

	let task_manager = {
		let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
		TaskManager::new(config.tokio_handle.clone(), registry)?
	};

	let chain_spec = &config.chain_spec;
	let fork_blocks = get_extension::<ForkBlocks<TBl>>(chain_spec.extensions())
		.cloned()
		.unwrap_or_default();

	let bad_blocks = get_extension::<BadBlocks<TBl>>(chain_spec.extensions())
		.cloned()
		.unwrap_or_default();

	let client = {
		let extensions = sc_client_api::execution_extensions::ExecutionExtensions::new(
			None,
			Arc::new(executor.clone()),
		);

		let wasm_runtime_substitutes = config
			.chain_spec
			.code_substitutes()
			.into_iter()
			.map(|(n, c)| {
				let number = NumberFor::<TBl>::from_str(&n).map_err(|_| {
					Error::Application(Box::from(format!(
						"Failed to parse `{}` as block number for code substitutes. \
						 In an old version the key for code substitute was a block hash. \
						 Please update the chain spec to a version that is compatible with your node.",
						n
					)))
				})?;
				Ok((number, c))
			})
			.collect::<Result<std::collections::HashMap<_, _>, Error>>()?;

		let client = client::new_client(
			backend.clone(),
			executor,
			genesis_block_builder,
			fork_blocks,
			bad_blocks,
			extensions,
			Box::new(task_manager.spawn_handle()),
			config
				.prometheus_config
				.as_ref()
				.map(|config| config.registry.clone()),
			telemetry,
			ClientConfig {
				offchain_worker_enabled: config.offchain_worker.enabled,
				offchain_indexing_api: config.offchain_worker.indexing_enabled,
				wasmtime_precompiled: config.wasmtime_precompiled.clone(),
				wasm_runtime_overrides: config.wasm_runtime_overrides.clone(),
				no_genesis: matches!(
					config.network.sync_mode,
					SyncMode::LightState { .. } | SyncMode::Warp { .. }
				),
				wasm_runtime_substitutes,
				enable_import_proof_recording: false,
			},
		)?;

		client
	};

	Ok((client, backend, keystore_container, task_manager))
}

/// Builds the PartialComponents for a lazy loading node.
#[allow(clippy::type_complexity)]
pub fn new_lazy_loading_partial<RuntimeApi, Customizations>(
	config: &mut Configuration,
	rpc_config: &RpcConfig,
	lazy_loading_config: &LazyLoadingConfig,
) -> PartialComponentsResult<LazyLoadingClient<RuntimeApi>, LazyLoadingBackend>
where
	RuntimeApi: ConstructRuntimeApi<Block, LazyLoadingClient<RuntimeApi>> + Send + Sync + 'static,
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
		new_lazy_loading_parts::<Block, RuntimeApi, _>(
			config,
			lazy_loading_config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
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

	let maybe_select_chain = Some(sc_consensus::LongestChain::new(backend.clone()));

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
		// Create a dummy parachain inherent data provider which is required to pass
		// the checks by the para chain system. We use dummy values because in the 'pending context'
		// neither do we have access to the real values nor do we need them.
		let (relay_parent_storage_root, relay_chain_state) =
			RelayStateSproofBuilder::default().into_state_root_and_proof();
		let vfp = PersistedValidationData {
			// This is a hack to make `cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases`
			// happy. Relay parent number can't be bigger than u32::MAX.
			relay_parent_number: u32::MAX,
			relay_parent_storage_root,
			..Default::default()
		};
		let parachain_inherent_data = ParachainInherentData {
			validation_data: vfp,
			relay_chain_state,
			downward_messages: Default::default(),
			horizontal_messages: Default::default(),
		};
		Ok((time, parachain_inherent_data))
	};

	let import_queue = nimbus_consensus::import_queue(
		client.clone(),
		frontier_block_import.clone(),
		create_inherent_data_providers,
		&task_manager.spawn_essential_handle(),
		config.prometheus_registry(),
		false,
	)?;
	let block_import = BlockImportPipeline::Dev(frontier_block_import);

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

/// Builds a new lazy loading service. This service uses manual seal, and mocks
/// the parachain inherent.
#[sc_tracing::logging::prefix_logs_with("Lazy loading 🌗")]
pub async fn new_lazy_loading_service<RuntimeApi, Customizations, Net>(
	mut config: Configuration,
	_author_id: Option<NimbusId>,
	sealing: moonbeam_cli_opt::Sealing,
	rpc_config: RpcConfig,
	lazy_loading_config: LazyLoadingConfig,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> Result<TaskManager, ServiceError>
where
	RuntimeApi: ConstructRuntimeApi<Block, LazyLoadingClient<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	Customizations: ClientCustomizations + 'static,
	Net: NetworkBackend<Block, Hash>,
{
	use async_io::Timer;
	use futures::Stream;
	use sc_consensus_manual_seal::{EngineCommand, ManualSealParams};

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
	} = lazy_loading::new_lazy_loading_partial::<RuntimeApi, Customizations>(
		&mut config,
		&rpc_config,
		&lazy_loading_config,
	)?;

	let start_delay = 10;
	let lazy_loading_startup_disclaimer = format!(
		r#"

		You are now running the Moonbeam client in lazy loading mode, where data is retrieved
		from a live RPC node on demand.

		Using remote state from: {rpc}
		Forking from block: {fork_block}

		To ensure the client works properly, please note the following:

		    1. *Avoid Throttling*: Ensure that the backing RPC node is not limiting the number of
		    requests, as this can prevent the lazy loading client from functioning correctly;

		    2. *Be Patient*: As the client may take approximately 20 times longer than normal to
		    retrieve and process the necessary data for the requested operation.


		The service will start in {start_delay} seconds...

		"#,
		rpc = lazy_loading_config.state_rpc,
		fork_block = backend.fork_checkpoint.number
	);

	log::warn!(
		"{}",
		ansi_term::Colour::Yellow.paint(lazy_loading_startup_disclaimer)
	);
	tokio::time::sleep(Duration::from_secs(start_delay)).await;

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
			"`new_lazy_loading_partial` builds a `LongestChainRule` when building dev service.\
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
			additional_relay_offset,
		));

		let client_clone = client.clone();
		let keystore_clone = keystore_container.keystore().clone();
		let maybe_provide_vrf_digest =
			move |nimbus_id: NimbusId, parent: Hash| -> Option<sp_runtime::generic::DigestItem> {
				moonbeam_vrf::vrf_pre_digest::<Block, LazyLoadingClient<RuntimeApi>>(
					&client_clone,
					&keystore_clone,
					nimbus_id,
					parent,
				)
			};

		let parachain_id = helpers::get_parachain_id(backend.rpc_client.clone())
			.unwrap_or_else(|| panic!("Could not get parachain identifier for lazy loading mode."));

		task_manager.spawn_essential_handle().spawn_blocking(
			"authorship_task",
			Some("block-authoring"),
			manual_sealing::run_manual_seal(ManualSealParams {
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

					let client_for_cidp = client_set_aside_for_cidp.clone();
					async move {
						let time = sp_timestamp::InherentDataProvider::from_system_time();

						let current_para_block = maybe_current_para_block?
							.ok_or(sp_blockchain::Error::UnknownBlock(block.to_string()))?;

						let current_para_block_head = Some(polkadot_primitives::HeadData(
							maybe_current_para_head?.encode(),
						));

						let mut additional_key_values = vec![
							(
								moonbeam_core_primitives::well_known_relay_keys::TIMESTAMP_NOW
									.to_vec(),
								sp_timestamp::Timestamp::current().encode(),
							),
							(
								relay_chain::well_known_keys::ACTIVE_CONFIG.to_vec(),
								AbridgedHostConfiguration {
									max_code_size: 3_145_728,
									max_head_data_size: 20_480,
									max_upward_queue_count: 174_762,
									max_upward_queue_size: 1_048_576,
									max_upward_message_size: 65_531,
									max_upward_message_num_per_candidate: 16,
									hrmp_max_message_num_per_candidate: 10,
									validation_upgrade_cooldown: 14_400,
									validation_upgrade_delay: 600,
									async_backing_params: AsyncBackingParams {
										max_candidate_depth: 3,
										allowed_ancestry_len: 2,
									},
								}
								.encode(),
							),
							// Override current slot number
							(
								relay_chain::well_known_keys::CURRENT_SLOT.to_vec(),
								Slot::from(u64::from(current_para_block)).encode(),
							),
						];

						// If there is a pending upgrade, lets mimic a GoAhead
						// signal from the relay

						let storage_key = [
							twox_128(b"ParachainSystem"),
							twox_128(b"PendingValidationCode"),
						]
						.concat();
						let has_pending_upgrade = client_for_cidp
							.storage(block, &StorageKey(storage_key))
							.map_or(false, |ok| ok.map_or(false, |some| !some.0.is_empty()));
						if has_pending_upgrade {
							additional_key_values.push((
								relay_chain::well_known_keys::upgrade_go_ahead_signal(ParaId::new(
									parachain_id,
								)),
								Some(UpgradeGoAhead::GoAhead).encode(),
							));
						}

						let mocked_parachain = MockValidationDataInherentDataProvider {
							current_para_block,
							para_id: ParaId::new(parachain_id),
							current_para_block_head,
							relay_offset: 1000,
							relay_blocks_per_para_block: 2,
							// TODO: Recheck
							para_blocks_per_relay_epoch: 10,
							relay_randomness_config: (),
							xcm_config: MockXcmConfig::new(
								&*client_for_cidp,
								block,
								Default::default(),
							),
							raw_downward_messages: downward_xcm_receiver.drain().collect(),
							raw_horizontal_messages: hrmp_xcm_receiver.drain().collect(),
							additional_key_values: Some(additional_key_values),
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
		let command_sink_for_task = command_sink.clone();
		move |deny_unsafe, subscription_task_executor| {
			let deps = rpc::FullDeps {
				backend: backend.clone(),
				client: client.clone(),
				command_sink: command_sink_for_task.clone(),
				deny_unsafe,
				ethapi_cmd: ethapi_cmd.clone(),
				filter_pool: filter_pool.clone(),
				frontier_backend: match *frontier_backend {
					fc_db::Backend::KeyValue(ref b) => b.clone(),
					fc_db::Backend::Sql(ref b) => b.clone(),
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

	network_starter.start_network();

	log::info!("Service Ready");

	Ok(task_manager)
}
