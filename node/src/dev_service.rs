// Copyright 2019-2020 PureStake Inc.
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

//! Service and ServiceFactory implementation. Specialized wrapper over Substrate service.
//! This one is used specifically for the --dev service.

use crate::cli::{EthApi as EthApiCmd, Sealing};
use async_io::Timer;
use fc_consensus::FrontierBlockImport;
use fc_rpc::EthApi;
use fc_rpc_core::types::{FilterPool, PendingTransactions};
use futures::Stream;
use futures::StreamExt;
use moonbeam_rpc_trace::TraceFilterCache;
use moonbeam_runtime::{self, opaque::Block, RuntimeApi};
use sc_client_api::BlockchainEvents;
use sc_consensus_manual_seal::{run_manual_seal, EngineCommand, ManualSealParams};
use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sp_core::H160;
use sp_core::H256;
use std::time::Duration;
use std::{
	collections::{BTreeMap, HashMap},
	sync::{Arc, Mutex},
};

// Our native executor instance.
native_executor_instance!(
	pub Executor,
	moonbeam_runtime::api::dispatch,
	moonbeam_runtime::native_version,
);

type FullClient = sc_service::TFullClient<Block, RuntimeApi, Executor>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

#[allow(clippy::type_complexity)]
pub fn new_partial(
	config: &Configuration,
	author: Option<H160>,
) -> Result<
	sc_service::PartialComponents<
		FullClient,
		FullBackend,
		FullSelectChain,
		sp_consensus::import_queue::BasicQueue<Block, sp_api::TransactionFor<FullClient, Block>>,
		sc_transaction_pool::FullPool<Block, FullClient>,
		(
			FrontierBlockImport<Block, Arc<FullClient>, FullClient>,
			PendingTransactions,
			Option<FilterPool>,
		),
	>,
	ServiceError,
> {
	let inherent_data_providers = crate::inherents::build_inherent_data_providers(author, true)?;

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, Executor>(&config)?;
	let client = Arc::new(client);

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
	);

	let pending_transactions: PendingTransactions = Some(Arc::new(Mutex::new(HashMap::new())));

	let filter_pool: Option<FilterPool> = Some(Arc::new(Mutex::new(BTreeMap::new())));

	let frontier_block_import = FrontierBlockImport::new(client.clone(), client.clone(), true);

	let import_queue = sc_consensus_manual_seal::import_queue(
		Box::new(frontier_block_import.clone()),
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
	);

	Ok(sc_service::PartialComponents {
		client,
		backend,
		task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		inherent_data_providers,
		other: (frontier_block_import, pending_transactions, filter_pool),
	})
}

/// Builds a new service for a full client.
pub fn new_full(
	config: Configuration,
	sealing: Sealing,
	author_id: Option<H160>,
	ethapi_cmd: Vec<EthApiCmd>,
) -> Result<TaskManager, ServiceError> {
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		inherent_data_providers,
		other: (block_import, pending_transactions, filter_pool),
	} = new_partial(&config, author_id)?;

	let (network, network_status_sinks, system_rpc_tx, network_starter) =
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
			backend.clone(),
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let role = config.role.clone();
	let prometheus_registry = config.prometheus_registry().cloned();
	let is_authority = role.is_authority();
	let subscription_task_executor =
		sc_rpc::SubscriptionTaskExecutor::new(task_manager.spawn_handle());
	let mut command_sink = None;

	if role.is_authority() {
		let env = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
		);

		let commands_stream: Box<dyn Stream<Item = EngineCommand<H256>> + Send + Sync + Unpin> =
			match sealing {
				Sealing::Instant => {
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
				Sealing::Manual => {
					let (sink, stream) = futures::channel::mpsc::channel(1000);
					// Keep a reference to the other end of the channel. It goes to the RPC.
					command_sink = Some(sink);
					Box::new(stream)
				}
				Sealing::Interval(millis) => Box::new(StreamExt::map(
					Timer::interval(Duration::from_millis(millis)),
					|_| EngineCommand::SealNewBlock {
						create_empty: true,
						finalize: false,
						parent_hash: None,
						sender: None,
					},
				)),
			};

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
				inherent_data_providers,
			}),
		);
	}

	let (trace_filter_task, trace_filter_requester) = if ethapi_cmd.contains(&EthApiCmd::Trace) {
		// WARNING : We create a second one in "rpc.rs" for the Trace RPC Api.
		// Is this okay to have it 2 times ? What happens if they have different parameters (signers ?) ?
		let eth_api = EthApi::new(
			client.clone(),
			transaction_pool.clone(),
			transaction_pool.pool().clone(),
			moonbeam_runtime::TransactionConverter,
			network.clone(),
			pending_transactions.clone(),
			vec![],
			is_authority,
		);

		let (trace_filter_task, trace_filter_requester) =
			TraceFilterCache::task(Arc::clone(&client), Arc::clone(&backend), eth_api);
		(Some(trace_filter_task), Some(trace_filter_requester))
	} else {
		(None, None)
	};

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let network = network.clone();
		let pending = pending_transactions.clone();
		let filter_pool = filter_pool.clone();
		let backend = backend.clone();
		let ethapi_cmd = ethapi_cmd.clone();
		Box::new(move |deny_unsafe, _| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				graph: pool.pool().clone(),
				deny_unsafe,
				is_authority,
				network: network.clone(),
				pending_transactions: pending.clone(),
				backend: backend.clone(),
				filter_pool: filter_pool.clone(),
				ethapi_cmd: ethapi_cmd.clone(),
				command_sink: command_sink.clone(),
				trace_filter_requester: trace_filter_requester.clone(),
			};
			crate::rpc::create_full(deps, subscription_task_executor.clone())
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network,
		client: client.clone(),
		keystore: keystore_container.sync_keystore(),
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		rpc_extensions_builder,
		on_demand: None,
		remote_blockchain: None,
		backend,
		network_status_sinks,
		system_rpc_tx,
		config,
	})?;

	// Spawn trace_filter cache task if enabled.
	if let Some(trace_filter_task) = trace_filter_task {
		task_manager
			.spawn_essential_handle()
			.spawn("trace-filter-cache", trace_filter_task);
	}

	// Spawn Frontier EthFilterApi maintenance task.
	if filter_pool.is_some() {
		// Each filter is allowed to stay in the pool for 100 blocks.
		const FILTER_RETAIN_THRESHOLD: u64 = 100;
		task_manager.spawn_essential_handle().spawn(
			"frontier-filter-pool",
			client
				.import_notification_stream()
				.for_each(move |notification| {
					if let Ok(locked) = &mut filter_pool.clone().unwrap().lock() {
						let imported_number: u64 = notification.header.number as u64;
						for (k, v) in locked.clone().iter() {
							let lifespan_limit = v.at_block + FILTER_RETAIN_THRESHOLD;
							if lifespan_limit <= imported_number {
								locked.remove(&k);
							}
						}
					}
					futures::future::ready(())
				}),
		);
	}

	// Spawn Frontier pending transactions maintenance task (as essential, otherwise we leak).
	if pending_transactions.is_some() {
		use fp_consensus::{ConsensusLog, FRONTIER_ENGINE_ID};
		use sp_runtime::generic::OpaqueDigestItemId;

		const TRANSACTION_RETAIN_THRESHOLD: u64 = 5;
		task_manager.spawn_essential_handle().spawn(
			"frontier-pending-transactions",
			client
				.import_notification_stream()
				.for_each(move |notification| {
					if let Ok(locked) = &mut pending_transactions.clone().unwrap().lock() {
						// As pending transactions have a finite lifespan anyway
						// we can ignore MultiplePostRuntimeLogs error checks.
						let mut frontier_log: Option<_> = None;
						for log in notification.header.digest.logs.iter().rev() {
							let log = log.try_to::<ConsensusLog>(OpaqueDigestItemId::Consensus(
								&FRONTIER_ENGINE_ID,
							));
							if log.is_some() {
								frontier_log = log;
								break;
							}
						}

						let imported_number: u64 = notification.header.number as u64;

						if let Some(ConsensusLog::EndBlock {
							block_hash: _,
							transaction_hashes,
						}) = frontier_log
						{
							// Retain all pending transactions that were not
							// processed in the current block.
							locked.retain(|&k, _| !transaction_hashes.contains(&k));
						}
						locked.retain(|_, v| {
							// Drop all the transactions that exceeded the given lifespan.
							let lifespan_limit = v.at_block + TRANSACTION_RETAIN_THRESHOLD;
							lifespan_limit > imported_number
						});
					}
					futures::future::ready(())
				}),
		);
	}

	log::info!("Development Service Ready");

	network_starter.start_network();
	Ok(task_manager)
}
