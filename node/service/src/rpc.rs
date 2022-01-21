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

//! A collection of node-specific RPC extensions and related background tasks.

pub mod tracing;

use std::{sync::Arc, time::Duration};

use fp_rpc::EthereumRuntimeRPCApi;
use sp_block_builder::BlockBuilder;

use crate::{client::RuntimeApiCollection, TransactionConverters};
use cli_opt::EthApi as EthApiCmd;
use cumulus_primitives_core::ParaId;
use fc_mapping_sync::{MappingSyncWorker, SyncStrategy};
use fc_rpc::{
	EthApi, EthApiServer, EthBlockDataCache, EthFilterApi, EthFilterApiServer, EthPubSubApi,
	EthPubSubApiServer, EthTask, HexEncodedIdProvider, NetApi, NetApiServer, OverrideHandle,
	RuntimeApiStorageOverride, SchemaV1Override, SchemaV2Override, SchemaV3Override,
	StorageOverride, Web3Api, Web3ApiServer,
};
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use futures::StreamExt;
use jsonrpc_pubsub::manager::SubscriptionManager;
use manual_xcm_rpc::{ManualXcm, ManualXcmApi};
use moonbeam_core_primitives::{Block, Hash};
use moonbeam_finality_rpc::{MoonbeamFinality, MoonbeamFinalityApi};
use moonbeam_rpc_txpool::{TxPool, TxPoolServer};
use pallet_ethereum::EthereumStorageSchema;
use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
use sc_client_api::{
	backend::{AuxStore, Backend, StateBackend, StorageProvider},
	client::BlockchainEvents,
	BlockOf,
};
use sc_consensus_manual_seal::rpc::{EngineCommand, ManualSeal, ManualSealApi};
use sc_network::NetworkService;
use sc_rpc::SubscriptionTaskExecutor;
use sc_rpc_api::DenyUnsafe;
use sc_service::TaskManager;
use sc_transaction_pool::{ChainApi, Pool};
use sc_transaction_pool_api::TransactionPool;
use sp_api::{HeaderT, ProvideRuntimeApi};
use sp_blockchain::{
	Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
use std::collections::BTreeMap;
use substrate_frame_rpc_system::{FullSystem, SystemApi};

/// Full client dependencies.
pub struct FullDeps<C, P, A: ChainApi, BE> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Graph pool instance.
	pub graph: Arc<Pool<A>>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// The Node authority flag
	pub is_authority: bool,
	/// Network service
	pub network: Arc<NetworkService<Block, Hash>>,
	/// EthFilterApi pool.
	pub filter_pool: Option<FilterPool>,
	/// The list of optional RPC extensions.
	pub ethapi_cmd: Vec<EthApiCmd>,
	/// Size of the LRU cache for block data and their transaction statuses.
	pub eth_log_block_cache: usize,
	/// Frontier Backend.
	pub frontier_backend: Arc<fc_db::Backend<Block>>,
	/// Backend.
	pub backend: Arc<BE>,
	/// Manual seal command sink
	pub command_sink: Option<futures::channel::mpsc::Sender<EngineCommand<Hash>>>,
	/// Maximum number of logs in a query.
	pub max_past_logs: u32,
	/// Maximum fee history cache size.
	pub fee_history_limit: u64,
	/// Fee history cache.
	pub fee_history_cache: FeeHistoryCache,
	/// Ethereum transaction to Extrinsic converter.
	pub transaction_converter: TransactionConverters,
	/// Channels for manual xcm messages (downward, hrmp)
	pub xcm_senders: Option<(flume::Sender<Vec<u8>>, flume::Sender<(ParaId, Vec<u8>)>)>,
}

pub fn overrides_handle<C, BE>(client: Arc<C>) -> Arc<OverrideHandle<Block>>
where
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
	C: Send + Sync + 'static,
	C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
{
	let mut overrides_map = BTreeMap::new();
	overrides_map.insert(
		EthereumStorageSchema::V1,
		Box::new(SchemaV1Override::new(client.clone()))
			as Box<dyn StorageOverride<_> + Send + Sync>,
	);
	overrides_map.insert(
		EthereumStorageSchema::V2,
		Box::new(SchemaV2Override::new(client.clone()))
			as Box<dyn StorageOverride<_> + Send + Sync>,
	);
	overrides_map.insert(
		EthereumStorageSchema::V3,
		Box::new(SchemaV3Override::new(client.clone()))
			as Box<dyn StorageOverride<_> + Send + Sync>,
	);

	Arc::new(OverrideHandle {
		schemas: overrides_map,
		fallback: Box::new(RuntimeApiStorageOverride::new(client.clone())),
	})
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, BE, A>(
	deps: FullDeps<C, P, A, BE>,
	subscription_task_executor: SubscriptionTaskExecutor,
	overrides: Arc<OverrideHandle<Block>>,
) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
where
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	BE::Blockchain: BlockchainBackend<Block>,
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
	C: BlockchainEvents<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	A: ChainApi<Block = Block> + 'static,
	C::Api: RuntimeApiCollection<StateBackend = BE::State>,
	P: TransactionPool<Block = Block> + 'static,
{
	let mut io = jsonrpc_core::IoHandler::default();
	let FullDeps {
		client,
		pool,
		graph,
		deny_unsafe,
		is_authority,
		network,
		filter_pool,
		ethapi_cmd,
		eth_log_block_cache,
		command_sink,
		frontier_backend,
		backend: _,
		max_past_logs,
		fee_history_limit,
		fee_history_cache,
		transaction_converter,
		xcm_senders,
	} = deps;

	io.extend_with(SystemApi::to_delegate(FullSystem::new(
		client.clone(),
		pool.clone(),
		deny_unsafe,
	)));
	io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
		client.clone(),
	)));
	// TODO: are we supporting signing?
	let signers = Vec::new();

	let block_data_cache = Arc::new(EthBlockDataCache::new(
		eth_log_block_cache,
		eth_log_block_cache,
	));

	io.extend_with(EthApiServer::to_delegate(EthApi::new(
		client.clone(),
		pool.clone(),
		graph.clone(),
		transaction_converter,
		network.clone(),
		signers,
		overrides.clone(),
		frontier_backend.clone(),
		is_authority,
		max_past_logs,
		block_data_cache.clone(),
		fc_rpc::format::Geth,
		fee_history_limit,
		fee_history_cache,
	)));

	if let Some(filter_pool) = filter_pool {
		io.extend_with(EthFilterApiServer::to_delegate(EthFilterApi::new(
			client.clone(),
			frontier_backend.clone(),
			filter_pool,
			500_usize, // max stored filters
			overrides.clone(),
			max_past_logs,
			block_data_cache.clone(),
		)));
	}

	io.extend_with(NetApiServer::to_delegate(NetApi::new(
		client.clone(),
		network.clone(),
		true,
	)));
	io.extend_with(Web3ApiServer::to_delegate(Web3Api::new(client.clone())));
	io.extend_with(EthPubSubApiServer::to_delegate(EthPubSubApi::new(
		pool,
		client.clone(),
		network,
		SubscriptionManager::<HexEncodedIdProvider>::with_id_provider(
			HexEncodedIdProvider::default(),
			Arc::new(subscription_task_executor),
		),
		overrides,
	)));
	if ethapi_cmd.contains(&EthApiCmd::Txpool) {
		io.extend_with(TxPoolServer::to_delegate(TxPool::new(
			Arc::clone(&client),
			graph,
		)));
	}

	io.extend_with(MoonbeamFinalityApi::to_delegate(MoonbeamFinality::new(
		client.clone(),
		frontier_backend.clone(),
	)));

	if let Some(command_sink) = command_sink {
		io.extend_with(
			// We provide the rpc handler with the sending end of the channel to allow the rpc
			// send EngineCommands to the background block authorship task.
			ManualSealApi::to_delegate(ManualSeal::new(command_sink)),
		);
	};

	if let Some((downward_message_channel, hrmp_message_channel)) = xcm_senders {
		io.extend_with(ManualXcmApi::to_delegate(ManualXcm {
			downward_message_channel,
			hrmp_message_channel,
		}));
	}

	io
}

pub struct SpawnTasksParams<'a, B: BlockT, C, BE> {
	pub task_manager: &'a TaskManager,
	pub client: Arc<C>,
	pub substrate_backend: Arc<BE>,
	pub frontier_backend: Arc<fc_db::Backend<B>>,
	pub filter_pool: Option<FilterPool>,
	pub overrides: Arc<OverrideHandle<B>>,
	pub fee_history_limit: u64,
	pub fee_history_cache: FeeHistoryCache,
}

/// Spawn the tasks that are required to run Moonbeam.
pub fn spawn_essential_tasks<B, C, BE>(params: SpawnTasksParams<B, C, BE>)
where
	C: ProvideRuntimeApi<B> + BlockOf,
	C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError> + 'static,
	C: BlockchainEvents<B> + StorageProvider<B, BE>,
	C: Send + Sync + 'static,
	C::Api: EthereumRuntimeRPCApi<B>,
	C::Api: BlockBuilder<B>,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	B::Header: HeaderT<Number = u32>,
	BE: Backend<B> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
{
	// Frontier offchain DB task. Essential.
	// Maps emulated ethereum data to substrate native data.
	params.task_manager.spawn_essential_handle().spawn(
		"frontier-mapping-sync-worker",
		Some("frontier"),
		MappingSyncWorker::new(
			params.client.import_notification_stream(),
			Duration::new(6, 0),
			params.client.clone(),
			params.substrate_backend.clone(),
			params.frontier_backend.clone(),
			SyncStrategy::Parachain,
		)
		.for_each(|()| futures::future::ready(())),
	);

	// Frontier `EthFilterApi` maintenance.
	// Manages the pool of user-created Filters.
	if let Some(filter_pool) = params.filter_pool {
		// Each filter is allowed to stay in the pool for 100 blocks.
		const FILTER_RETAIN_THRESHOLD: u64 = 100;
		params.task_manager.spawn_essential_handle().spawn(
			"frontier-filter-pool",
			Some("frontier"),
			EthTask::filter_pool_task(
				Arc::clone(&params.client),
				filter_pool,
				FILTER_RETAIN_THRESHOLD,
			),
		);
	}

	params.task_manager.spawn_essential_handle().spawn(
		"frontier-schema-cache-task",
		Some("frontier"),
		EthTask::ethereum_schema_cache_task(
			Arc::clone(&params.client),
			Arc::clone(&params.frontier_backend),
		),
	);

	// Spawn Frontier FeeHistory cache maintenance task.
	params.task_manager.spawn_essential_handle().spawn(
		"frontier-fee-history",
		Some("frontier"),
		EthTask::fee_history_task(
			Arc::clone(&params.client),
			Arc::clone(&params.overrides),
			params.fee_history_cache,
			params.fee_history_limit,
		),
	);
}
