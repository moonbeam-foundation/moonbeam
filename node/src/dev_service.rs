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

use fc_consensus::FrontierBlockImport;
use fc_rpc_core::types::PendingTransactions;
use moonbeam_runtime::{self, opaque::Block, RuntimeApi};
use sc_client_api::BlockchainEvents;
use sc_consensus_manual_seal::{self as manual_seal};
use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sp_core::H160;
use std::{
	collections::HashMap,
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

pub fn new_partial(
	config: &Configuration,
	// manual_seal: bool, // For now only manual seal. Maybe bring this back to support instant later.
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
		other: (frontier_block_import, pending_transactions),
	})
}

/// Builds a new service for a full client.
pub fn new_full(
	config: Configuration,
	// manual_seal: bool,
	author_id: Option<H160>,
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
		other: (block_import, pending_transactions),
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

	// Channel for the rpc handler to communicate with the authorship task.
	let (command_sink, commands_stream) = futures::channel::mpsc::channel(1000);

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
	let telemetry_connection_sinks = sc_service::TelemetryConnectionSinks::default();
	let is_authority = role.is_authority();
	let subscription_task_executor =
		sc_rpc::SubscriptionTaskExecutor::new(task_manager.spawn_handle());

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let network = network.clone();
		let pending = pending_transactions.clone();
		Box::new(move |deny_unsafe, _| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				graph: pool.pool().clone(),
				deny_unsafe,
				is_authority,
				network: network.clone(),
				pending_transactions: pending.clone(),
				command_sink: Some(command_sink.clone()),
			};
			crate::rpc::create_full(deps, subscription_task_executor.clone())
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		network: network.clone(),
		client: client.clone(),
		keystore: keystore_container.sync_keystore(),
		task_manager: &mut task_manager,
		transaction_pool: transaction_pool.clone(),
		telemetry_connection_sinks,
		rpc_extensions_builder,
		on_demand: None,
		remote_blockchain: None,
		backend,
		network_status_sinks,
		system_rpc_tx,
		config,
	})?;

	// Spawn Frontier pending transactions maintenance task (as essential, otherwise we leak).
	if pending_transactions.is_some() {
		use fp_consensus::{ConsensusLog, FRONTIER_ENGINE_ID};
		use futures::StreamExt;
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
						for log in notification.header.digest.logs {
							let log = log.try_to::<ConsensusLog>(OpaqueDigestItemId::Consensus(
								&FRONTIER_ENGINE_ID,
							));
							if let Some(log) = log {
								frontier_log = Some(log);
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

	if role.is_authority() {
		let env = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
		);

		// Background authorship future
		let authorship_future = manual_seal::run_manual_seal(manual_seal::ManualSealParams {
			block_import,
			env,
			client,
			pool: transaction_pool.pool().clone(),
			commands_stream,
			select_chain,
			consensus_data_provider: None,
			inherent_data_providers,
		});

		// we spawn the future on a background thread managed by service.
		task_manager
			.spawn_essential_handle()
			.spawn_blocking("manual-seal", authorship_future);
	}

	log::info!("Development Service Ready");

	network_starter.start_network();
	Ok(task_manager)
}
