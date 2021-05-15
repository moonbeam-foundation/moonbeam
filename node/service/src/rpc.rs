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

//! A collection of node-specific RPC methods.

use std::collections::BTreeMap;
use std::sync::Arc;

use cli_opt::EthApi as EthApiCmd;
use ethereum::EthereumStorageSchema;
use fc_rpc::{OverrideHandle, RuntimeApiStorageOverride, SchemaV1Override, StorageOverride};
use fc_rpc_core::types::{FilterPool, PendingTransactions};
use jsonrpc_pubsub::manager::SubscriptionManager;
use moonbeam_rpc_debug::DebugRequester;
use moonbeam_rpc_trace::CacheRequester as TraceFilterCacheRequester;
#[cfg(feature = "with-moonbeam-runtime")]
use moonbeam_runtime::{opaque::Block, AccountId, Balance, Hash, Index};
#[cfg(feature = "with-moonriver-runtime")]
use moonriver_runtime::{opaque::Block, AccountId, Balance, Hash, Index};
#[cfg(feature = "with-moonbase-runtime")]
use moonbase_runtime::{opaque::Block, AccountId, Balance, Hash, Index};
use sc_client_api::{
	backend::{AuxStore, Backend, StateBackend, StorageProvider},
	client::BlockchainEvents,
};
use sc_consensus_manual_seal::rpc::{EngineCommand, ManualSeal, ManualSealApi};
use sc_network::NetworkService;
use sc_rpc::SubscriptionTaskExecutor;
use sc_rpc_api::DenyUnsafe;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{
	Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_runtime::traits::BlakeTwo256;
use sp_transaction_pool::TransactionPool;
use crate::client::{RuntimeApiCollection, EthereumRuntimeApiCollection};

// TODO-multiples-runtimes one rpc definition per network

/// Full client dependencies.
#[cfg(not(feature = "with-moonriver-runtime"))]
pub struct FullDeps<C, P, BE> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// The Node authority flag
	pub is_authority: bool,
	/// Network service
	pub network: Arc<NetworkService<Block, Hash>>,
	/// Ethereum pending transactions.
	pub pending_transactions: PendingTransactions,
	/// EthFilterApi pool.
	pub filter_pool: Option<FilterPool>,
	/// The list of optional RPC extensions.
	pub ethapi_cmd: Vec<EthApiCmd>,
	/// Frontier Backend.
	pub frontier_backend: Arc<fc_db::Backend<Block>>,
	/// Backend.
	pub backend: Arc<BE>,
	/// Manual seal command sink
	pub command_sink: Option<futures::channel::mpsc::Sender<EngineCommand<Hash>>>,
	/// Debug server requester.
	pub debug_requester: Option<DebugRequester>,
	/// Trace filter cache server requester.
	pub trace_filter_requester: Option<TraceFilterCacheRequester>,
	/// Trace filter max count.
	pub trace_filter_max_count: u32,
	/// Maximum number of logs in a query.
	pub max_past_logs: u32,
}
/// Instantiate all Full RPC extensions.
#[cfg(not(feature = "with-moonriver-runtime"))]
pub fn create_full<C, P, BE>(
	deps: FullDeps<C, P, BE>,
	subscription_task_executor: SubscriptionTaskExecutor,
) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
where
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	BE::Blockchain: BlockchainBackend<Block>,
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
	C: BlockchainEvents<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: RuntimeApiCollection<StateBackend = BE::State> + EthereumRuntimeApiCollection,
	P: TransactionPool<Block = Block> + 'static,
{
	use fc_rpc::{
		EthApi, EthApiServer, EthFilterApi, EthFilterApiServer, EthPubSubApi, EthPubSubApiServer,
		HexEncodedIdProvider, NetApi, NetApiServer, Web3Api, Web3ApiServer,
	};
	use moonbeam_rpc_debug::{Debug, DebugServer};
	use moonbeam_rpc_trace::{Trace, TraceServer};
	use moonbeam_rpc_txpool::{TxPool, TxPoolServer};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
	use substrate_frame_rpc_system::{FullSystem, SystemApi};

	let mut io = jsonrpc_core::IoHandler::default();
	let FullDeps {
		client,
		pool,
		deny_unsafe,
		is_authority,
		network,
		pending_transactions,
		filter_pool,
		ethapi_cmd,
		command_sink,
		frontier_backend,
		backend: _,
		debug_requester,
		trace_filter_requester,
		trace_filter_max_count,
		max_past_logs,
	} = deps;

	io.extend_with(SystemApi::to_delegate(FullSystem::new(
		client.clone(),
		pool.clone(),
		deny_unsafe,
	)));
	io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
		client.clone(),
	)));

	#[cfg(not(feature = "with-moonriver-runtime"))]
	{
		// TODO: are we supporting signing?
		let signers = Vec::new();
	
		let mut overrides_map = BTreeMap::new();
		overrides_map.insert(
			EthereumStorageSchema::V1,
			Box::new(SchemaV1Override::new(client.clone()))
				as Box<dyn StorageOverride<_> + Send + Sync>,
		);
	
		let overrides = Arc::new(OverrideHandle {
			schemas: overrides_map,
			fallback: Box::new(RuntimeApiStorageOverride::new(client.clone())),
		});
		#[cfg(feature = "with-moonbeam-runtime")]
		let tx_converter: moonbeam_runtime::TransactionConverter =
			moonbeam_runtime::TransactionConverter;
		#[cfg(feature = "with-moonbase-runtime")]
		let tx_converter: moonbase_runtime::TransactionConverter =
			moonbase_runtime::TransactionConverter;

		io.extend_with(EthApiServer::to_delegate(EthApi::new(
			client.clone(),
			pool.clone(),
			tx_converter,
			network.clone(),
			pending_transactions,
			signers,
			overrides.clone(),
			frontier_backend.clone(),
			is_authority,
			max_past_logs,
		)));
	
		if let Some(filter_pool) = filter_pool {
			io.extend_with(EthFilterApiServer::to_delegate(EthFilterApi::new(
				client.clone(),
				filter_pool.clone(),
				500 as usize, // max stored filters
				overrides.clone(),
				max_past_logs,
			)));
		}
	
		io.extend_with(NetApiServer::to_delegate(NetApi::new(
			client.clone(),
			network.clone(),
		)));
		io.extend_with(Web3ApiServer::to_delegate(Web3Api::new(client.clone())));
		io.extend_with(EthPubSubApiServer::to_delegate(EthPubSubApi::new(
			pool.clone(),
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
				pool,
			)));
		}
	
		if let Some(command_sink) = command_sink {
			io.extend_with(
				// We provide the rpc handler with the sending end of the channel to allow the rpc
				// send EngineCommands to the background block authorship task.
				ManualSealApi::to_delegate(ManualSeal::new(command_sink)),
			);
		};
	
		if let Some(trace_filter_requester) = trace_filter_requester {
			io.extend_with(TraceServer::to_delegate(Trace::new(
				client,
				trace_filter_requester,
				trace_filter_max_count,
			)));
		}
	
		if let Some(debug_requester) = debug_requester {
			io.extend_with(DebugServer::to_delegate(Debug::new(debug_requester)));
		}
	}

	io
}

/// Full client dependencies.
#[cfg(feature = "with-moonriver-runtime")]
pub struct FullDepsMoonriver<C, P, BE> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// The Node authority flag
	pub is_authority: bool,
	/// Network service
	pub network: Arc<NetworkService<Block, Hash>>,
	/// Backend.
	pub backend: Arc<BE>,
	/// Manual seal command sink
	pub command_sink: Option<futures::channel::mpsc::Sender<EngineCommand<Hash>>>,
}
/// Instantiate all Full RPC extensions.
#[cfg(feature = "with-moonriver-runtime")]
pub fn create_full_moonriver<C, P, BE>(
	deps: FullDepsMoonriver<C, P, BE>,
	subscription_task_executor: SubscriptionTaskExecutor,
) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
where
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	BE::Blockchain: BlockchainBackend<Block>,
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
	C: BlockchainEvents<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: RuntimeApiCollection<StateBackend = BE::State> + EthereumRuntimeApiCollection,
	P: TransactionPool<Block = Block> + 'static,
{
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
	use substrate_frame_rpc_system::{FullSystem, SystemApi};

	let mut io = jsonrpc_core::IoHandler::default();
	let FullDepsMoonriver {
		client,
		pool,
		deny_unsafe,
		is_authority,
		network,
		command_sink,
		backend: _,
	} = deps;

	io.extend_with(SystemApi::to_delegate(FullSystem::new(
		client.clone(),
		pool.clone(),
		deny_unsafe,
	)));
	io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
		client.clone(),
	)));
	io
}
