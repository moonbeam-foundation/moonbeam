// This file is part of Frontier.

// Copyright (C) 2019-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A collection of node-specific RPC methods.

use std::{sync::Arc, fmt};

use sc_consensus_manual_seal::rpc::{ManualSeal, ManualSealApi};
use moonbase_runtime::{Hash, AccountId, Index, opaque::Block, Balance};
use sp_api::ProvideRuntimeApi;
use sp_transaction_pool::TransactionPool;
use sc_transaction_graph::{Pool, ChainApi};
use sp_blockchain::{Error as BlockChainError, HeaderMetadata, HeaderBackend};
use sc_rpc_api::DenyUnsafe;
use sc_client_api::{
	backend::{StorageProvider, Backend, StateBackend, AuxStore},
	client::BlockchainEvents
};
use sc_rpc::SubscriptionTaskExecutor;
use sp_runtime::traits::BlakeTwo256;
use sp_block_builder::BlockBuilder;
use sc_network::NetworkService;
use jsonrpc_pubsub::manager::SubscriptionManager;

/// Full client dependencies.
pub struct FullDeps<C, P, A: ChainApi> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Validated pool access.
	pub graph_pool: Arc<Pool<A>>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// The Node authority flag
	pub is_authority: bool,
	/// Number of past blocks allowed for querying ethereum events.
	pub eth_block_limit: Option<u32>,
	/// Number of logs allowed for querying ethereum events.
	pub eth_log_limit: Option<u32>,
	/// Network service
	pub network: Arc<NetworkService<Block, Hash>>,
	/// Manual seal command sink
	pub command_sink: Option<futures::channel::mpsc::Sender<sc_consensus_manual_seal::rpc::EngineCommand<Hash>>>,
}

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, BE, A>(
	deps: FullDeps<C, P, A>,
	subscription_task_executor: SubscriptionTaskExecutor
) -> jsonrpc_core::IoHandler<sc_rpc::Metadata> where
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
	C: BlockchainEvents<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error=BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
	C::Api: BlockBuilder<Block>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: frontier_rpc_primitives::EthereumRuntimeRPCApi<Block>,
	<C::Api as sp_api::ApiErrorExt>::Error: fmt::Debug,
	P: TransactionPool<Block=Block> + 'static,
	A: ChainApi<Block=Block> + 'static,
{
	use substrate_frame_rpc_system::{FullSystem, SystemApi};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
	use frontier_rpc::{EthApi, EthApiServer, NetApi, NetApiServer, EthPubSubApi, EthPubSubApiServer};

	let mut io = jsonrpc_core::IoHandler::default();
	let FullDeps {
		client,
		pool,
		graph_pool,
		deny_unsafe,
		is_authority,
		eth_block_limit,
		eth_log_limit,
		network,
		command_sink
	} = deps;

	io.extend_with(
		SystemApi::to_delegate(FullSystem::new(client.clone(), pool.clone(), deny_unsafe))
	);
	io.extend_with(
		TransactionPaymentApi::to_delegate(TransactionPayment::new(client.clone()))
	);
	io.extend_with(
		EthApiServer::to_delegate(EthApi::new(
			client.clone(),
			graph_pool.clone(),
			pool.clone(),
			moonbase_runtime::TransactionConverter,
			is_authority,
			eth_block_limit,
			eth_log_limit,
		))
	);
	io.extend_with(
		NetApiServer::to_delegate(NetApi::new(
			client.clone(),
		))
	);
	io.extend_with(
		EthPubSubApiServer::to_delegate(EthPubSubApi::new(
			pool.clone(),
			client.clone(),
			network.clone(),
			SubscriptionManager::new(Arc::new(subscription_task_executor)),
		))
	);

	match command_sink {
		Some(command_sink) => {
			io.extend_with(
				// We provide the rpc handler with the sending end of the channel to allow the rpc
				// send EngineCommands to the background block authorship task.
				ManualSealApi::to_delegate(ManualSeal::new(command_sink)),
			);
		}
		_ => {}
	}

	io
}
