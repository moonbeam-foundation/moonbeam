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

use crate::{cli::Sealing, inherents::build_inherent_data_providers};
use async_io::Timer;
use cumulus_network::build_block_announce_validator;
use cumulus_service::{
	prepare_node_config, start_collator, start_full_node, StartCollatorParams, StartFullNodeParams,
};
use fc_consensus::FrontierBlockImport;
use fc_rpc_core::types::{FilterPool, PendingTransactions};
use futures::{Stream, StreamExt};
use moonbeam_runtime::{opaque::Block, RuntimeApi};
use polkadot_primitives::v0::CollatorPair;
use sc_client_api::BlockchainEvents;
use sc_consensus_manual_seal::{run_manual_seal, EngineCommand, ManualSealParams};
use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;
use sc_service::{
	error::Error as ServiceError, Configuration, PartialComponents, Role, TFullBackend,
	TFullClient, TaskManager,
};
use sp_core::{Pair, H160, H256};
use sp_runtime::traits::BlakeTwo256;
use sp_trie::PrefixedMemoryDB;
use std::{
	collections::{BTreeMap, HashMap},
	sync::{Arc, Mutex},
	time::Duration,
};

// Our native executor instance.
native_executor_instance!(
	pub Executor,
	moonbeam_runtime::api::dispatch,
	moonbeam_runtime::native_version,
);

type FullClient = TFullClient<Block, RuntimeApi, Executor>;
type FullBackend = TFullBackend<Block>;

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
#[allow(clippy::type_complexity)]
pub fn new_partial(
	config: &Configuration,
	author: Option<H160>,
	mock_inherents: bool,
) -> Result<
	PartialComponents<
		FullClient,
		FullBackend,
		(),
		sp_consensus::import_queue::BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
		sc_transaction_pool::FullPool<Block, FullClient>,
		(
			FrontierBlockImport<Block, Arc<FullClient>, FullClient>,
			PendingTransactions,
			Option<FilterPool>,
		),
	>,
	ServiceError,
> {
	let inherent_data_providers = build_inherent_data_providers(author, mock_inherents)?;

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, Executor>(&config)?;
	let client = Arc::new(client);

	let registry = config.prometheus_registry();

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.prometheus_registry(),
		task_manager.spawn_handle(),
		client.clone(),
	);

	let pending_transactions: PendingTransactions = Some(Arc::new(Mutex::new(HashMap::new())));

	let filter_pool: Option<FilterPool> = Some(Arc::new(Mutex::new(BTreeMap::new())));

	let frontier_block_import = FrontierBlockImport::new(client.clone(), client.clone(), true);

	// We build the cumulus import queue here regardless of whether we're running a parachain or
	// the dev service. Either one will be fine when only partial components are necessary.
	// When running the dev service, an alternate import queue will be built below.
	let import_queue = cumulus_consensus::import_queue::import_queue(
		client.clone(),
		frontier_block_import.clone(),
		inherent_data_providers.clone(),
		&task_manager.spawn_handle(),
		registry,
	)?;

	Ok(PartialComponents {
		backend,
		client,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		inherent_data_providers,
		select_chain: (),
		other: (frontier_block_import, pending_transactions, filter_pool),
	})
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
async fn start_node_impl<RB>(
	parachain_config: Configuration,
	collator_key: CollatorPair,
	author_id: Option<H160>,
	polkadot_config: Configuration,
	id: polkadot_primitives::v0::Id,
	collator: bool,
	_rpc_ext_builder: RB,
) -> sc_service::error::Result<(TaskManager, Arc<FullClient>)>
where
	RB: Fn(
			Arc<TFullClient<Block, RuntimeApi, Executor>>,
		) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
		+ Send
		+ 'static,
{
	if matches!(parachain_config.role, Role::Light) {
		return Err("Light client not supported!".into());
	}

	let parachain_config = prepare_node_config(parachain_config);

	let polkadot_full_node =
		cumulus_service::build_polkadot_full_node(polkadot_config, collator_key.public()).map_err(
			|e| match e {
				polkadot_service::Error::Sub(x) => x,
				s => format!("{}", s).into(),
			},
		)?;

	let params = new_partial(&parachain_config, author_id, false)?;

	let client = params.client.clone();
	let backend = params.backend.clone();
	let block_announce_validator = build_block_announce_validator(
		polkadot_full_node.client.clone(),
		id,
		Box::new(polkadot_full_node.network.clone()),
		polkadot_full_node.backend.clone(),
	);

	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let transaction_pool = params.transaction_pool.clone();
	let mut task_manager = params.task_manager;
	let import_queue = params.import_queue;
	let (block_import, pending_transactions, filter_pool) = params.other;
	let (network, network_status_sinks, system_rpc_tx, start_network) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &parachain_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			on_demand: None,
			block_announce_validator_builder: Some(Box::new(|_| block_announce_validator)),
		})?;

	let subscription_task_executor =
		sc_rpc::SubscriptionTaskExecutor::new(task_manager.spawn_handle());

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let network = network.clone();
		let pending = pending_transactions.clone();
		let filter_pool = filter_pool.clone();
		Box::new(move |deny_unsafe, _| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				graph: pool.pool().clone(),
				deny_unsafe,
				is_authority: collator,
				network: network.clone(),
				pending_transactions: pending.clone(),
				filter_pool: filter_pool.clone(),
				command_sink: None,
			};

			crate::rpc::create_full(deps, subscription_task_executor.clone())
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
		network_status_sinks,
		system_rpc_tx,
	})?;

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

	let announce_block = {
		let network = network.clone();
		Arc::new(move |hash, data| network.announce_block(hash, Some(data)))
	};

	if collator {
		let proposer_factory = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool,
			prometheus_registry.as_ref(),
		);
		let spawner = task_manager.spawn_handle();

		let polkadot_backend = polkadot_full_node.backend.clone();

		let params = StartCollatorParams {
			para_id: id,
			block_import,
			proposer_factory,
			inherent_data_providers: params.inherent_data_providers,
			block_status: client.clone(),
			announce_block,
			client: client.clone(),
			task_manager: &mut task_manager,
			collator_key,
			polkadot_full_node,
			spawner,
			backend,
			polkadot_backend,
		};

		start_collator(params).await?;
	} else {
		let params = StartFullNodeParams {
			client: client.clone(),
			announce_block,
			task_manager: &mut task_manager,
			para_id: id,
			polkadot_full_node,
		};

		start_full_node(params)?;
	}

	start_network.start_network();

	Ok((task_manager, client))
}

/// Start a normal parachain node.
pub async fn start_node(
	parachain_config: Configuration,
	collator_key: CollatorPair,
	author_id: Option<H160>,
	polkadot_config: Configuration,
	id: polkadot_primitives::v0::Id,
	collator: bool,
) -> sc_service::error::Result<(TaskManager, Arc<FullClient>)> {
	start_node_impl(
		parachain_config,
		collator_key,
		author_id,
		polkadot_config,
		id,
		collator,
		|_| Default::default(),
	)
	.await
}

/// Builds a new development service. This service uses manual seal, and mocks
/// the parachain inherent.
pub fn new_dev(
	config: Configuration,
	sealing: Sealing,
	author_id: Option<H160>,
	collator: bool,
) -> Result<TaskManager, ServiceError> {
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue: _,
		keystore_container,
		select_chain: _,
		transaction_pool,
		inherent_data_providers,
		other: (block_import, pending_transactions, filter_pool),
	} = new_partial(&config, author_id, true)?;

	// When running the dev service we build a manual seal import queue so that we can properly
	// follow the longest chain rule. However, there is another bug in this import queue where
	// it doesn't properly check inherents:
	// https://github.com/paritytech/substrate/issues/8164
	let dev_import_queue = sc_consensus_manual_seal::import_queue(
		Box::new(block_import.clone()),
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
	);

	let (network, network_status_sinks, system_rpc_tx, network_starter) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue: dev_import_queue,
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

	let prometheus_registry = config.prometheus_registry().cloned();
	let subscription_task_executor =
		sc_rpc::SubscriptionTaskExecutor::new(task_manager.spawn_handle());
	let mut command_sink = None;

	if collator {
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

		// Typically the longest chain rule is constructed in `new_partial`, but the full service,
		// does not use one. So instead we construct our longest chain rule here.
		let select_chain = sc_consensus::LongestChain::new(backend.clone());

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

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let network = network.clone();
		let pending = pending_transactions.clone();
		let filter_pool = filter_pool.clone();
		Box::new(move |deny_unsafe, _| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				graph: pool.pool().clone(),
				deny_unsafe,
				is_authority: collator,
				network: network.clone(),
				pending_transactions: pending.clone(),
				filter_pool: filter_pool.clone(),
				command_sink: command_sink.clone(),
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
