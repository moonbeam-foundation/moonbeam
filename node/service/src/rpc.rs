// Copyright 2019-2025 PureStake Inc.
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

use crate::client::RuntimeApiCollection;
use cumulus_primitives_core::{ParaId, PersistedValidationData};
use cumulus_primitives_parachain_inherent::ParachainInherentData;
use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
use fc_mapping_sync::{kv::MappingSyncWorker, SyncStrategy};
use fc_rpc::{pending::ConsensusDataProvider, EthBlockDataCacheTask, EthTask, StorageOverride};
use fc_rpc_core::types::{FeeHistoryCache, FilterPool, TransactionRequest};
use futures::StreamExt;
use jsonrpsee::RpcModule;
use moonbeam_cli_opt::EthApi as EthApiCmd;
use moonbeam_core_primitives::{Block, Hash};
use sc_client_api::{
	backend::{AuxStore, Backend, StateBackend, StorageProvider},
	client::BlockchainEvents,
	BlockOf,
};
use sc_consensus_manual_seal::rpc::{EngineCommand, ManualSeal, ManualSealApiServer};
use sc_network::service::traits::NetworkService;
use sc_network_sync::SyncingService;
use sc_rpc::SubscriptionTaskExecutor;
use sc_service::TaskManager;
use sc_transaction_pool::{ChainApi, Pool};
use sc_transaction_pool_api::TransactionPool;
use sp_api::{CallApiAt, ProvideRuntimeApi};
use sp_blockchain::{
	Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, Header as HeaderT};
use std::collections::BTreeMap;

pub struct MoonbeamEGA;

impl fc_rpc::EstimateGasAdapter for MoonbeamEGA {
	fn adapt_request(mut request: TransactionRequest) -> TransactionRequest {
		// Redirect any call to batch precompile:
		// force usage of batchAll method for estimation
		use sp_core::H160;
		const BATCH_PRECOMPILE_ADDRESS: H160 = H160(hex_literal::hex!(
			"0000000000000000000000000000000000000808"
		));
		const BATCH_PRECOMPILE_BATCH_ALL_SELECTOR: [u8; 4] = hex_literal::hex!("96e292b8");
		if request.to == Some(BATCH_PRECOMPILE_ADDRESS) {
			match (&mut request.data.input, &mut request.data.data) {
				(Some(ref mut input), _) => {
					if input.0.len() >= 4 {
						input.0[..4].copy_from_slice(&BATCH_PRECOMPILE_BATCH_ALL_SELECTOR);
					}
				}
				(None, Some(ref mut data)) => {
					if data.0.len() >= 4 {
						data.0[..4].copy_from_slice(&BATCH_PRECOMPILE_BATCH_ALL_SELECTOR);
					}
				}
				(_, _) => {}
			};
		}
		request
	}
}

pub struct MoonbeamEthConfig<C, BE>(std::marker::PhantomData<(C, BE)>);

impl<C, BE> fc_rpc::EthConfig<Block, C> for MoonbeamEthConfig<C, BE>
where
	C: sc_client_api::StorageProvider<Block, BE> + Sync + Send + 'static,
	BE: Backend<Block> + 'static,
{
	type EstimateGasAdapter = MoonbeamEGA;
	type RuntimeStorageOverride =
		fc_rpc::frontier_backend_client::SystemAccountId20StorageOverride<Block, C, BE>;
}

/// Full client dependencies.
pub struct FullDeps<C, P, A: ChainApi, BE> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Graph pool instance.
	pub graph: Arc<Pool<A>>,
	/// The Node authority flag
	pub is_authority: bool,
	/// Network service
	pub network: Arc<dyn NetworkService>,
	/// Chain syncing service
	pub sync: Arc<SyncingService<Block>>,
	/// EthFilterApi pool.
	pub filter_pool: Option<FilterPool>,
	/// The list of optional RPC extensions.
	pub ethapi_cmd: Vec<EthApiCmd>,
	/// Frontier Backend.
	pub frontier_backend: Arc<dyn fc_api::Backend<Block>>,
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
	/// Channels for manual xcm messages (downward, hrmp)
	pub dev_rpc_data: Option<(
		flume::Sender<Vec<u8>>,
		flume::Sender<(ParaId, Vec<u8>)>,
		Arc<std::sync::atomic::AtomicU32>,
	)>,
	/// Ethereum data access overrides.
	pub overrides: Arc<dyn StorageOverride<Block>>,
	/// Cache for Ethereum block data.
	pub block_data_cache: Arc<EthBlockDataCacheTask<Block>>,
	/// Mandated parent hashes for a given block hash.
	pub forced_parent_hashes: Option<BTreeMap<H256, H256>>,
}

pub struct TracingConfig {
	pub tracing_requesters: crate::rpc::tracing::RpcRequesters,
	pub trace_filter_max_count: u32,
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, BE, A>(
	deps: FullDeps<C, P, A, BE>,
	subscription_task_executor: SubscriptionTaskExecutor,
	maybe_tracing_config: Option<TracingConfig>,
	pubsub_notification_sinks: Arc<
		fc_mapping_sync::EthereumBlockNotificationSinks<
			fc_mapping_sync::EthereumBlockNotification<Block>,
		>,
	>,
	pending_consenus_data_provider: Box<dyn ConsensusDataProvider<Block>>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	BE::Blockchain: BlockchainBackend<Block>,
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
	C: BlockchainEvents<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: CallApiAt<Block>,
	C: Send + Sync + 'static,
	A: ChainApi<Block = Block> + 'static,
	C::Api: RuntimeApiCollection,
	P: TransactionPool<Block = Block> + 'static,
{
	use fc_rpc::{
		Eth, EthApiServer, EthFilter, EthFilterApiServer, EthPubSub, EthPubSubApiServer, Net,
		NetApiServer, TxPool, TxPoolApiServer, Web3, Web3ApiServer,
	};
	use moonbeam_dev_rpc::{DevApiServer, DevRpc};
	use moonbeam_finality_rpc::{MoonbeamFinality, MoonbeamFinalityApiServer};
	use moonbeam_rpc_debug::{Debug, DebugServer};
	use moonbeam_rpc_trace::{Trace, TraceServer};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};

	let mut io = RpcModule::new(());
	let FullDeps {
		client,
		pool,
		graph,
		is_authority,
		network,
		sync,
		filter_pool,
		ethapi_cmd,
		command_sink,
		frontier_backend,
		backend: _,
		max_past_logs,
		fee_history_limit,
		fee_history_cache,
		dev_rpc_data,
		overrides,
		block_data_cache,
		forced_parent_hashes,
	} = deps;

	io.merge(System::new(Arc::clone(&client), Arc::clone(&pool)).into_rpc())?;
	io.merge(TransactionPayment::new(Arc::clone(&client)).into_rpc())?;

	// TODO: are we supporting signing?
	let signers = Vec::new();

	enum Never {}
	impl<T> fp_rpc::ConvertTransaction<T> for Never {
		fn convert_transaction(&self, _transaction: pallet_ethereum::Transaction) -> T {
			// The Never type is not instantiable, but this method requires the type to be
			// instantiated to be called (`&self` parameter), so if the code compiles we have the
			// guarantee that this function will never be called.
			unreachable!()
		}
	}
	let convert_transaction: Option<Never> = None;

	let pending_create_inherent_data_providers = move |_, _| async move {
		let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
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
		Ok((timestamp, parachain_inherent_data))
	};

	io.merge(
		Eth::<_, _, _, _, _, _, _, MoonbeamEthConfig<_, _>>::new(
			Arc::clone(&client),
			Arc::clone(&pool),
			graph.clone(),
			convert_transaction,
			Arc::clone(&sync),
			signers,
			Arc::clone(&overrides),
			Arc::clone(&frontier_backend),
			is_authority,
			Arc::clone(&block_data_cache),
			fee_history_cache,
			fee_history_limit,
			10,
			forced_parent_hashes,
			pending_create_inherent_data_providers,
			Some(pending_consenus_data_provider),
		)
		.replace_config::<MoonbeamEthConfig<C, BE>>()
		.into_rpc(),
	)?;

	if let Some(filter_pool) = filter_pool {
		io.merge(
			EthFilter::new(
				client.clone(),
				frontier_backend.clone(),
				graph.clone(),
				filter_pool,
				500_usize, // max stored filters
				max_past_logs,
				1024,
				block_data_cache,
			)
			.into_rpc(),
		)?;
	}

	io.merge(
		Net::new(
			Arc::clone(&client),
			network.clone(),
			// Whether to format the `peer_count` response as Hex (default) or not.
			true,
		)
		.into_rpc(),
	)?;

	io.merge(Web3::new(Arc::clone(&client)).into_rpc())?;
	io.merge(
		EthPubSub::new(
			pool,
			Arc::clone(&client),
			sync.clone(),
			subscription_task_executor,
			overrides,
			pubsub_notification_sinks.clone(),
		)
		.into_rpc(),
	)?;

	if ethapi_cmd.contains(&EthApiCmd::Txpool) {
		io.merge(TxPool::new(Arc::clone(&client), graph).into_rpc())?;
	}

	io.merge(MoonbeamFinality::new(client.clone(), frontier_backend.clone()).into_rpc())?;

	if let Some(command_sink) = command_sink {
		io.merge(
			// We provide the rpc handler with the sending end of the channel to allow the rpc
			// send EngineCommands to the background block authorship task.
			ManualSeal::new(command_sink).into_rpc(),
		)?;
	};

	if let Some((downward_message_channel, hrmp_message_channel, additional_relay_offset)) =
		dev_rpc_data
	{
		io.merge(
			DevRpc {
				downward_message_channel,
				hrmp_message_channel,
				additional_relay_offset,
			}
			.into_rpc(),
		)?;
	}

	if let Some(tracing_config) = maybe_tracing_config {
		if let Some(trace_filter_requester) = tracing_config.tracing_requesters.trace {
			io.merge(
				Trace::new(
					client,
					trace_filter_requester,
					tracing_config.trace_filter_max_count,
				)
				.into_rpc(),
			)?;
		}

		if let Some(debug_requester) = tracing_config.tracing_requesters.debug {
			io.merge(Debug::new(debug_requester).into_rpc())?;
		}
	}

	Ok(io)
}

pub struct SpawnTasksParams<'a, B: BlockT, C, BE> {
	pub task_manager: &'a TaskManager,
	pub client: Arc<C>,
	pub substrate_backend: Arc<BE>,
	pub frontier_backend: Arc<fc_db::Backend<B, C>>,
	pub filter_pool: Option<FilterPool>,
	pub overrides: Arc<dyn StorageOverride<B>>,
	pub fee_history_limit: u64,
	pub fee_history_cache: FeeHistoryCache,
}

/// Spawn the tasks that are required to run Moonbeam.
pub fn spawn_essential_tasks<B, C, BE>(
	params: SpawnTasksParams<B, C, BE>,
	sync: Arc<SyncingService<B>>,
	pubsub_notification_sinks: Arc<
		fc_mapping_sync::EthereumBlockNotificationSinks<
			fc_mapping_sync::EthereumBlockNotification<B>,
		>,
	>,
) where
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
	match *params.frontier_backend {
		fc_db::Backend::KeyValue(ref b) => {
			params.task_manager.spawn_essential_handle().spawn(
				"frontier-mapping-sync-worker",
				Some("frontier"),
				MappingSyncWorker::new(
					params.client.import_notification_stream(),
					Duration::new(6, 0),
					params.client.clone(),
					params.substrate_backend.clone(),
					params.overrides.clone(),
					b.clone(),
					3,
					0,
					SyncStrategy::Parachain,
					sync.clone(),
					pubsub_notification_sinks.clone(),
				)
				.for_each(|()| futures::future::ready(())),
			);
		}
		fc_db::Backend::Sql(ref b) => {
			params.task_manager.spawn_essential_handle().spawn_blocking(
				"frontier-mapping-sync-worker",
				Some("frontier"),
				fc_mapping_sync::sql::SyncWorker::run(
					params.client.clone(),
					params.substrate_backend.clone(),
					b.clone(),
					params.client.import_notification_stream(),
					fc_mapping_sync::sql::SyncWorkerConfig {
						read_notification_timeout: Duration::from_secs(10),
						check_indexed_blocks_interval: Duration::from_secs(60),
					},
					fc_mapping_sync::SyncStrategy::Parachain,
					sync.clone(),
					pubsub_notification_sinks.clone(),
				),
			);
		}
	}

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
