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

//! This module assembles the Moonbeam service components, executes them, and manages communication
//! between them. This is the backbone of the client-side node implementation.
//!
//! This module can assemble:
//! PartialComponents: For maintence tasks without a complete node (eg import/export blocks, purge)
//! Full Service: A complete parachain node including the pool, rpc, network, embedded relay chain
//! Dev Service: A leaner service without the relay chain backing.

use cli_opt::RpcConfig;
use fc_consensus::FrontierBlockImport;
use fc_rpc_core::types::{FilterPool, PendingTransactions};
use futures::StreamExt;
pub use moonbase_runtime;
pub use moonbeam_runtime;
pub use moonriver_runtime;
use sc_service::BasePath;
use std::{
	collections::{BTreeMap, HashMap},
	sync::Mutex,
	time::Duration,
};
mod inherents;
mod rpc;
use cumulus_client_network::build_block_announce_validator;
use cumulus_client_service::{
	prepare_node_config, start_collator, start_full_node, StartCollatorParams, StartFullNodeParams,
};
use nimbus_consensus::{build_nimbus_consensus, BuildNimbusConsensusParams};
use nimbus_primitives::NimbusId;

pub use sc_executor::NativeExecutor;
use sc_executor::{native_executor_instance, NativeExecutionDispatch};
use sc_service::{
	error::Error as ServiceError, ChainSpec, Configuration, PartialComponents, Role, TFullBackend,
	TFullClient, TaskManager,
};
use sp_api::ConstructRuntimeApi;
use sp_blockchain::HeaderBackend;
use std::sync::Arc;

pub use client::*;
pub mod chain_spec;
mod client;

use sc_telemetry::{Telemetry, TelemetryWorker, TelemetryWorkerHandle};

type FullClient<RuntimeApi, Executor> = TFullClient<Block, RuntimeApi, Executor>;
type FullBackend = TFullBackend<Block>;
type MaybeSelectChain = Option<sc_consensus::LongestChain<FullBackend, Block>>;

native_executor_instance!(
	pub MoonbeamExecutor,
	moonbeam_runtime::api::dispatch,
	moonbeam_runtime::native_version,
	(
		frame_benchmarking::benchmarking::HostFunctions,
		moonbeam_primitives_ext::moonbeam_ext::HostFunctions
	),
);

native_executor_instance!(
	pub MoonriverExecutor,
	moonriver_runtime::api::dispatch,
	moonriver_runtime::native_version,
	(
		frame_benchmarking::benchmarking::HostFunctions,
		moonbeam_primitives_ext::moonbeam_ext::HostFunctions
	),
);

native_executor_instance!(
	pub MoonbaseExecutor,
	moonbase_runtime::api::dispatch,
	moonbase_runtime::native_version,
	(
		frame_benchmarking::benchmarking::HostFunctions,
		moonbeam_primitives_ext::moonbeam_ext::HostFunctions
	),
);

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

use sp_runtime::traits::BlakeTwo256;
use sp_trie::PrefixedMemoryDB;

/// Builds a new object suitable for chain operations.
#[allow(clippy::type_complexity)]
pub fn new_chain_ops(
	mut config: &mut Configuration,
) -> Result<
	(
		Arc<Client>,
		Arc<FullBackend>,
		sp_consensus::import_queue::BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
		TaskManager,
	),
	ServiceError,
> {
	config.keystore = sc_service::config::KeystoreConfig::InMemory;
	if config.chain_spec.is_moonbase() {
		let PartialComponents {
			client,
			backend,
			import_queue,
			task_manager,
			..
		} = new_partial::<moonbase_runtime::RuntimeApi, MoonbaseExecutor>(
			config,
			config.chain_spec.is_dev(),
		)?;
		Ok((
			Arc::new(Client::Moonbase(client)),
			backend,
			import_queue,
			task_manager,
		))
	} else if config.chain_spec.is_moonriver() {
		let PartialComponents {
			client,
			backend,
			import_queue,
			task_manager,
			..
		} = new_partial::<moonriver_runtime::RuntimeApi, MoonriverExecutor>(
			config,
			config.chain_spec.is_dev(),
		)?;
		Ok((
			Arc::new(Client::Moonriver(client)),
			backend,
			import_queue,
			task_manager,
		))
	} else {
		let PartialComponents {
			client,
			backend,
			import_queue,
			task_manager,
			..
		} = new_partial::<moonbeam_runtime::RuntimeApi, MoonbeamExecutor>(
			config,
			config.chain_spec.is_dev(),
		)?;
		Ok((
			Arc::new(Client::Moonbeam(client)),
			backend,
			import_queue,
			task_manager,
		))
	}
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
		TFullClient<Block, RuntimeApi, Executor>,
		FullBackend,
		MaybeSelectChain,
		sp_consensus::DefaultImportQueue<Block, TFullClient<Block, RuntimeApi, Executor>>,
		sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>,
		(
			FrontierBlockImport<
				Block,
				Arc<TFullClient<Block, RuntimeApi, Executor>>,
				TFullClient<Block, RuntimeApi, Executor>,
			>,
			PendingTransactions,
			Option<FilterPool>,
			Option<Telemetry>,
			Option<TelemetryWorkerHandle>,
			Arc<fc_db::Backend<Block>>,
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

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, Executor>(
			&config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
		)?;

	let client = Arc::new(client);

	let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", worker.run());
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

	let pending_transactions: PendingTransactions = Some(Arc::new(Mutex::new(HashMap::new())));

	let filter_pool: Option<FilterPool> = Some(Arc::new(Mutex::new(BTreeMap::new())));

	let frontier_backend = open_frontier_backend(config)?;

	let frontier_block_import =
		FrontierBlockImport::new(client.clone(), client.clone(), frontier_backend.clone());

	// Depending whether we are
	let import_queue = if dev_service {
		// There is a bug in this import queue where it doesn't properly check inherents:
		// https://github.com/paritytech/substrate/issues/8164
		sc_consensus_manual_seal::import_queue(
			Box::new(frontier_block_import.clone()),
			&task_manager.spawn_essential_handle(),
			config.prometheus_registry(),
		)
	} else {
		nimbus_consensus::import_queue(
			client.clone(),
			frontier_block_import.clone(),
			move |_, _| async move {
				let time = sp_timestamp::InherentDataProvider::from_system_time();

				Ok((time,))
			},
			&task_manager.spawn_essential_handle(),
			config.prometheus_registry(),
		)?
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
			frontier_block_import,
			pending_transactions,
			filter_pool,
			telemetry,
			telemetry_worker_handle,
			frontier_backend,
		),
	})
}

/// `fp_rpc::ConvertTransaction` is implemented for an arbitrary struct that lives in each runtime.
/// It receives a ethereum::Transaction and returns a pallet-ethereum transact Call wrapped in an
/// UncheckedExtrinsic.
///
/// Although the implementation should be the same in each runtime, this might change at some point.
/// `TransactionConverters` is just a `fp_rpc::ConvertTransaction` implementor that proxies calls to
/// each runtime implementation.
pub enum TransactionConverters {
	Moonbeam(moonbeam_runtime::TransactionConverter),
	Moonbase(moonbase_runtime::TransactionConverter),
	Moonriver(moonriver_runtime::TransactionConverter),
}

impl fp_rpc::ConvertTransaction<moonbeam_core_primitives::UncheckedExtrinsic>
	for TransactionConverters
{
	fn convert_transaction(
		&self,
		transaction: ethereum_primitives::Transaction,
	) -> moonbeam_core_primitives::UncheckedExtrinsic {
		match &self {
			Self::Moonbeam(inner) => inner.convert_transaction(transaction),
			Self::Moonriver(inner) => inner.convert_transaction(transaction),
			Self::Moonbase(inner) => inner.convert_transaction(transaction),
		}
	}
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("🌗")]
async fn start_node_impl<RuntimeApi, Executor>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	id: polkadot_primitives::v0::Id,
	rpc_config: RpcConfig,
) -> sc_service::error::Result<(TaskManager, Arc<TFullClient<Block, RuntimeApi, Executor>>)>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	if matches!(parachain_config.role, Role::Light) {
		return Err("Light client not supported!".into());
	}

	let parachain_config = prepare_node_config(parachain_config);

	let params = new_partial(&parachain_config, false)?;
	let (
		block_import,
		pending_transactions,
		filter_pool,
		mut telemetry,
		telemetry_worker_handle,
		frontier_backend,
	) = params.other;

	let relay_chain_full_node =
		cumulus_client_service::build_polkadot_full_node(polkadot_config, telemetry_worker_handle)
			.map_err(|e| match e {
				polkadot_service::Error::Sub(x) => x,
				s => format!("{}", s).into(),
			})?;

	let client = params.client.clone();
	let backend = params.backend.clone();
	let block_announce_validator = build_block_announce_validator(
		relay_chain_full_node.client.clone(),
		id,
		Box::new(relay_chain_full_node.network.clone()),
		relay_chain_full_node.backend.clone(),
	);

	let collator = parachain_config.role.is_authority();
	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let transaction_pool = params.transaction_pool.clone();
	let mut task_manager = params.task_manager;
	let import_queue = cumulus_client_service::SharedImportQueue::new(params.import_queue);
	let (network, system_rpc_tx, start_network) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &parachain_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue: import_queue.clone(),
			on_demand: None,
			block_announce_validator_builder: Some(Box::new(|_| block_announce_validator)),
		})?;

	let subscription_task_executor =
		sc_rpc::SubscriptionTaskExecutor::new(task_manager.spawn_handle());

	let spawned_requesters = rpc::spawn_tasks(
		&rpc_config,
		rpc::SpawnTasksParams {
			task_manager: &task_manager,
			client: client.clone(),
			substrate_backend: backend.clone(),
			frontier_backend: frontier_backend.clone(),
			pending_transactions: pending_transactions.clone(),
			filter_pool: filter_pool.clone(),
		},
	);

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let network = network.clone();
		let pending = pending_transactions.clone();
		let filter_pool = filter_pool.clone();
		let frontier_backend = frontier_backend.clone();
		let backend = backend.clone();
		let ethapi_cmd = rpc_config.ethapi.clone();
		let max_past_logs = rpc_config.max_past_logs;

		let is_moonbeam = parachain_config.chain_spec.is_moonbeam();
		let is_moonriver = parachain_config.chain_spec.is_moonriver();

		Box::new(move |deny_unsafe, _| {
			let transaction_converter: TransactionConverters = if is_moonbeam {
				TransactionConverters::Moonbeam(moonbeam_runtime::TransactionConverter)
			} else if is_moonriver {
				TransactionConverters::Moonriver(moonriver_runtime::TransactionConverter)
			} else {
				TransactionConverters::Moonbase(moonbase_runtime::TransactionConverter)
			};

			let deps = rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				graph: pool.pool().clone(),
				deny_unsafe,
				is_authority: collator,
				network: network.clone(),
				pending_transactions: pending.clone(),
				filter_pool: filter_pool.clone(),
				ethapi_cmd: ethapi_cmd.clone(),
				command_sink: None,
				frontier_backend: frontier_backend.clone(),
				backend: backend.clone(),
				debug_requester: spawned_requesters.debug.clone(),
				trace_filter_requester: spawned_requesters.trace.clone(),
				trace_filter_max_count: rpc_config.ethapi_trace_max_count,
				max_past_logs,
				transaction_converter,
			};

			rpc::create_full(deps, subscription_task_executor.clone())
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		on_demand: None,
		remote_blockchain: None,
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

	if collator {
		let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|t| t.handle()),
		);

		let relay_chain_backend = relay_chain_full_node.backend.clone();
		let relay_chain_client = relay_chain_full_node.client.clone();

		let parachain_consensus = build_nimbus_consensus(BuildNimbusConsensusParams {
			para_id: id,
			proposer_factory,
			block_import,
			relay_chain_client: relay_chain_full_node.client.clone(),
			relay_chain_backend: relay_chain_full_node.backend.clone(),
			parachain_client: client.clone(),
			keystore: params.keystore_container.sync_keystore(),
			create_inherent_data_providers: move |_, (relay_parent, validation_data, author_id)| {
				let parachain_inherent =
							cumulus_primitives_parachain_inherent::ParachainInherentData::
							create_at_with_client(
								relay_parent,
								&relay_chain_client,
								&*relay_chain_backend,
								&validation_data,
								id,
							);
				async move {
					let time = sp_timestamp::InherentDataProvider::from_system_time();

					let parachain_inherent = parachain_inherent.ok_or_else(|| {
						Box::<dyn std::error::Error + Send + Sync>::from(
							"Failed to create parachain inherent",
						)
					})?;

					let author = nimbus_primitives::InherentDataProvider::<NimbusId>(author_id);

					Ok((time, parachain_inherent, author))
				}
			},
		});

		let spawner = task_manager.spawn_handle();

		let params = StartCollatorParams {
			para_id: id,
			block_status: client.clone(),
			announce_block,
			client: client.clone(),
			task_manager: &mut task_manager,
			spawner,
			relay_chain_full_node,
			parachain_consensus,
			import_queue,
		};

		start_collator(params).await?;
	} else {
		let params = StartFullNodeParams {
			client: client.clone(),
			announce_block,
			task_manager: &mut task_manager,
			para_id: id,
			relay_chain_full_node,
		};

		start_full_node(params)?;
	}

	start_network.start_network();

	Ok((task_manager, client))
}

/// Start a normal parachain node.
pub async fn start_node<RuntimeApi, Executor>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	id: polkadot_primitives::v0::Id,
	rpc_config: RpcConfig,
) -> sc_service::error::Result<(TaskManager, Arc<TFullClient<Block, RuntimeApi, Executor>>)>
where
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	start_node_impl(parachain_config, polkadot_config, id, rpc_config).await
}

/// Builds a new development service. This service uses manual seal, and mocks
/// the parachain inherent.
pub fn new_dev(
	config: Configuration,
	_author_id: Option<nimbus_primitives::NimbusId>,
	sealing: cli_opt::Sealing,
	rpc_config: RpcConfig,
) -> Result<TaskManager, ServiceError> {
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
				pending_transactions,
				filter_pool,
				telemetry,
				_telemetry_worker_handle,
				frontier_backend,
			),
	} = new_partial::<moonbase_runtime::RuntimeApi, MoonbaseExecutor>(&config, true)?;

	let (network, system_rpc_tx, network_starter) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: None,
			block_announce_validator_builder: None,
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
	let mut command_sink = None;
	let collator = config.role.is_authority();

	if collator {
		//TODO For now, all dev service nodes use Alith's nimbus id in their author inherent.
		// This could and perhaps should be made more flexible. Here are some options:
		// 1. a dedicated `--dev-author-id` flag that only works with the dev service
		// 2. restore the old --author-id` and also allow it to force a secific key
		//    in the parachain context
		// 3. check the keystore like we do in nimbus. Actually, maybe the keystore-checking could
		//    be exported as a helper function from nimbus.
		let author_id = chain_spec::get_from_seed::<NimbusId>("Alice");

		let env = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

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

		task_manager.spawn_essential_handle().spawn_blocking(
			"authorship_task",
			run_manual_seal(ManualSealParams {
				block_import,
				env,
				client: client.clone(),
				pool: transaction_pool.pool().clone(),
				commands_stream,
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

						let mocked_parachain = inherents::MockValidationDataInherentDataProvider {
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
	}

	let spawned_requesters = rpc::spawn_tasks(
		&rpc_config,
		rpc::SpawnTasksParams {
			task_manager: &task_manager,
			client: client.clone(),
			substrate_backend: backend.clone(),
			frontier_backend: frontier_backend.clone(),
			pending_transactions: pending_transactions.clone(),
			filter_pool: filter_pool.clone(),
		},
	);

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let backend = backend.clone();
		let network = network.clone();
		let pending = pending_transactions;
		let ethapi_cmd = rpc_config.ethapi.clone();
		let max_past_logs = rpc_config.max_past_logs;

		let is_moonbeam = config.chain_spec.is_moonbeam();
		let is_moonriver = config.chain_spec.is_moonriver();

		Box::new(move |deny_unsafe, _| {
			let transaction_converter: TransactionConverters = if is_moonbeam {
				TransactionConverters::Moonbeam(moonbeam_runtime::TransactionConverter)
			} else if is_moonriver {
				TransactionConverters::Moonriver(moonriver_runtime::TransactionConverter)
			} else {
				TransactionConverters::Moonbase(moonbase_runtime::TransactionConverter)
			};

			let deps = rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				graph: pool.pool().clone(),
				deny_unsafe,
				is_authority: collator,
				network: network.clone(),
				pending_transactions: pending.clone(),
				filter_pool: filter_pool.clone(),
				ethapi_cmd: ethapi_cmd.clone(),
				command_sink: command_sink.clone(),
				frontier_backend: frontier_backend.clone(),
				backend: backend.clone(),
				debug_requester: spawned_requesters.debug.clone(),
				trace_filter_requester: spawned_requesters.trace.clone(),
				trace_filter_max_count: rpc_config.ethapi_trace_max_count,
				max_past_logs,
				transaction_converter,
			};
			rpc::create_full(deps, subscription_task_executor.clone())
		})
	};

	let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network,
		client,
		keystore: keystore_container.sync_keystore(),
		task_manager: &mut task_manager,
		transaction_pool,
		rpc_extensions_builder,
		on_demand: None,
		remote_blockchain: None,
		backend,
		system_rpc_tx,
		config,
		telemetry: None,
	})?;

	log::info!("Development Service Ready");

	network_starter.start_network();
	Ok(task_manager)
}
