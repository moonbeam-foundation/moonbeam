// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
// This file is part of Frontier.
//
// Copyright (c) 2020 Parity Technologies (UK) Ltd.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use log::warn;
use sc_client_api::{
	backend::{AuxStore, Backend, StateBackend, StorageProvider},
	client::BlockchainEvents,
};
use sc_rpc::Metadata;
use sp_api::{BlockId, ProvideRuntimeApi};
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_io::hashing::twox_128;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, UniqueSaturatedInto};
use sp_storage::{StorageData, StorageKey};
use sp_transaction_pool::TransactionPool;
use std::collections::BTreeMap;
use std::{marker::PhantomData, sync::Arc};

use parity_scale_codec::Decode;
use ethereum_types::{H256, U256};
use fc_rpc_core::types::{
	pubsub::{Kind, Params, PubSubSyncStatus, Result as PubSubResult},
	Bytes, FilteredParams, Header, Log, Rich,
};
use fc_rpc_core::EthPubSubApi::{self as EthPubSubApiT};
use jsonrpc_pubsub::{manager::SubscriptionManager, typed::Subscriber, SubscriptionId};
use sha3::{Digest, Keccak256};

pub use fc_rpc_core::EthPubSubApiServer;
use futures::{StreamExt as _, TryStreamExt as _};

use fp_rpc::{EthereumRuntimeRPCApi, TransactionStatus};
use jsonrpc_core::{
	futures::{Future, Sink},
	Result as JsonRpcResult,
};

use fc_rpc::HexEncodedIdProvider;
use sc_network::{ExHashT, NetworkService};

pub struct EthPubSubApi<B: BlockT, P, C, BE, H: ExHashT> {
	_pool: Arc<P>,
	client: Arc<C>,
	network: Arc<NetworkService<B, H>>,
	subscriptions: SubscriptionManager<HexEncodedIdProvider>,
	_marker: PhantomData<(B, BE)>,
}

impl<B: BlockT, P, C, BE, H: ExHashT> EthPubSubApi<B, P, C, BE, H> {
	pub fn new(
		_pool: Arc<P>,
		client: Arc<C>,
		network: Arc<NetworkService<B, H>>,
		subscriptions: SubscriptionManager<HexEncodedIdProvider>,
	) -> Self {
		Self {
			_pool,
			client,
			network,
			subscriptions,
			_marker: PhantomData,
		}
	}
}

struct SubscriptionResult {}
#[allow(clippy::all)]
impl SubscriptionResult {
	pub fn new() -> Self {
		SubscriptionResult {}
	}
	pub fn new_heads(&self, block: ethereum::Block) -> PubSubResult {
		PubSubResult::Header(Box::new(Rich {
			inner: Header {
				hash: Some(H256::from_slice(
					Keccak256::digest(&rlp::encode(&block.header)).as_slice(),
				)),
				parent_hash: block.header.parent_hash,
				uncles_hash: block.header.ommers_hash,
				author: block.header.beneficiary,
				miner: block.header.beneficiary,
				state_root: block.header.state_root,
				transactions_root: block.header.transactions_root,
				receipts_root: block.header.receipts_root,
				number: Some(block.header.number),
				gas_used: block.header.gas_used,
				gas_limit: block.header.gas_limit,
				extra_data: Bytes(block.header.extra_data.clone()),
				logs_bloom: block.header.logs_bloom,
				timestamp: U256::from(block.header.timestamp),
				difficulty: block.header.difficulty,
				seal_fields: vec![
					Bytes(block.header.mix_hash.as_bytes().to_vec()),
					Bytes(block.header.nonce.as_bytes().to_vec()),
				],
				size: Some(U256::from(rlp::encode(&block).len() as u32)),
			},
			extra_info: BTreeMap::new(),
		}))
	}
	pub fn logs(
		&self,
		block_input: Option<ethereum::Block>,
		receipts: Vec<ethereum::Receipt>,
		params: &FilteredParams,
	) -> Vec<Log> {
		if block_input.is_none() {
			return Vec::new();
		}
		let block = block_input.unwrap();
		let block_hash = Some(H256::from_slice(
			Keccak256::digest(&rlp::encode(&block.header)).as_slice(),
		));
		let mut logs: Vec<Log> = vec![];
		let mut log_index: u32 = 0;
		for (receipt_index, receipt) in receipts.into_iter().enumerate() {
			let mut transaction_log_index: u32 = 0;
			let transaction_hash: Option<H256> = if receipt.logs.len() > 0 {
				Some(H256::from_slice(
					Keccak256::digest(&rlp::encode(&block.transactions[receipt_index as usize]))
						.as_slice(),
				))
			} else {
				None
			};
			for log in receipt.logs {
				if self.add_log(block_hash.unwrap(), &log, &block, params) {
					logs.push(Log {
						address: log.address,
						topics: log.topics,
						data: Bytes(log.data),
						block_hash: block_hash,
						block_number: Some(block.header.number),
						transaction_hash: transaction_hash,
						transaction_index: Some(U256::from(log_index)),
						log_index: Some(U256::from(log_index)),
						transaction_log_index: Some(U256::from(transaction_log_index)),
						removed: false,
					});
				}
				log_index += 1;
				transaction_log_index += 1;
			}
		}
		logs
	}
	fn add_log(
		&self,
		block_hash: H256,
		ethereum_log: &ethereum::Log,
		block: &ethereum::Block,
		params: &FilteredParams,
	) -> bool {
		let log = Log {
			address: ethereum_log.address.clone(),
			topics: ethereum_log.topics.clone(),
			data: Bytes(ethereum_log.data.clone()),
			block_hash: None,
			block_number: None,
			transaction_hash: None,
			transaction_index: None,
			log_index: None,
			transaction_log_index: None,
			removed: false,
		};
		if let Some(_) = params.filter {
			let block_number =
				UniqueSaturatedInto::<u64>::unique_saturated_into(block.header.number);
			if !params.filter_block_range(block_number)
				|| !params.filter_block_hash(block_hash)
				|| !params.filter_address(&log)
				|| !params.filter_topics(&log)
			{
				return false;
			}
		}
		true
	}
}

fn storage_prefix_build(module: &[u8], storage: &[u8]) -> Vec<u8> {
	[twox_128(module), twox_128(storage)].concat().to_vec()
}

macro_rules! stream_build {
	($context:expr => $module:expr, $storage:expr) => {{
		let key: StorageKey = StorageKey(storage_prefix_build($module, $storage));
		match $context
			.client
			.storage_changes_notification_stream(Some(&[key]), None)
			{
			Ok(stream) => Some(stream),
			Err(_err) => None,
			}
		}};
}

#[allow(clippy::all)]
impl<B: BlockT, P, C, BE, H: ExHashT> EthPubSubApiT for EthPubSubApi<B, P, C, BE, H>
where
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	P: TransactionPool<Block = B> + Send + Sync + 'static,
	C: ProvideRuntimeApi<B> + StorageProvider<B, BE> + BlockchainEvents<B> + AuxStore,
	C: HeaderBackend<B> + HeaderMetadata<B, Error = BlockChainError> + 'static,
	C: Send + Sync + 'static,
	C::Api: EthereumRuntimeRPCApi<B>,
	BE: Backend<B> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
{
	type Metadata = Metadata;
	fn subscribe(
		&self,
		_metadata: Self::Metadata,
		subscriber: Subscriber<PubSubResult>,
		kind: Kind,
		params: Option<Params>,
	) {
		let filtered_params = match params {
			Some(Params::Logs(filter)) => FilteredParams::new(Some(filter)),
			_ => FilteredParams::default(),
		};

		let client = self.client.clone();
		let network = self.network.clone();
		match kind {
			Kind::Logs => {
				if let Some(stream) = stream_build!(
					self => b"Ethereum", b"CurrentReceipts"
				) {
					self.subscriptions.add(subscriber, |sink| {
						let stream = stream
							.flat_map(move |(block_hash, changes)| {
								let id = BlockId::Hash(block_hash);
								let data = changes.iter().last().unwrap().2.unwrap();
								let receipts: Vec<ethereum::Receipt> =
									Decode::decode(&mut &data.0[..]).unwrap();
								let block: Option<ethereum::Block> = if let Ok(Some(data)) = client
									.storage(
										&id,
										&StorageKey(storage_prefix_build(
											b"Ethereum",
											b"CurrentBlock",
										)),
									) {
									if let Ok(result) = Decode::decode(&mut &data.0[..]) {
										Some(result)
									} else {
										None
									}
								} else {
									None
								};
								futures::stream::iter(SubscriptionResult::new().logs(
									block,
									receipts,
									&filtered_params,
								))
							})
							.map(|x| {
								return Ok::<
									Result<PubSubResult, jsonrpc_core::types::error::Error>,
									(),
								>(Ok(PubSubResult::Log(Box::new(x))));
							})
							.compat();

						sink.sink_map_err(|e| warn!("Error sending notifications: {:?}", e))
							.send_all(stream)
							.map(|_| ())
					});
				}
			}
			Kind::NewHeads => {
				if let Some(stream) = stream_build!(
					self => b"Ethereum", b"CurrentBlock"
				) {
					self.subscriptions.add(subscriber, |sink| {
						let stream = stream
							.map(|(_block, changes)| {
								let data = changes.iter().last().unwrap().2.unwrap();
								let block: ethereum::Block =
									Decode::decode(&mut &data.0[..]).unwrap();
								return Ok::<_, ()>(Ok(SubscriptionResult::new().new_heads(block)));
							})
							.compat();

						sink.sink_map_err(|e| warn!("Error sending notifications: {:?}", e))
							.send_all(stream)
							.map(|_| ())
					});
				}
			}
			Kind::NewPendingTransactions => {
				if let Some(stream) = stream_build!(
					self => b"Ethereum", b"Pending"
				) {
					self.subscriptions.add(subscriber, |sink| {
						let stream = stream
							.flat_map(|(_block, changes)| {
								let mut transactions: Vec<ethereum::Transaction> = vec![];
								let storage: Vec<Option<StorageData>> = changes
									.iter()
									.filter_map(|(o_sk, _k, v)| {
										if o_sk.is_none() {
											Some(v.cloned())
										} else {
											None
										}
									})
									.collect();
								for change in storage {
									if let Some(data) = change {
										let storage: Vec<(
											ethereum::Transaction,
											TransactionStatus,
											ethereum::Receipt,
										)> = Decode::decode(&mut &data.0[..]).unwrap();
										let tmp: Vec<ethereum::Transaction> =
											storage.iter().map(|x| x.0.clone()).collect();
										transactions.extend(tmp);
									}
								}
								futures::stream::iter(transactions)
							})
							.map(|transaction| {
								return Ok::<
									Result<PubSubResult, jsonrpc_core::types::error::Error>,
									(),
								>(Ok(PubSubResult::TransactionHash(H256::from_slice(
									Keccak256::digest(&rlp::encode(&transaction)).as_slice(),
								))));
							})
							.compat();

						sink.sink_map_err(|e| warn!("Error sending notifications: {:?}", e))
							.send_all(stream)
							.map(|_| ())
					});
				}
			}
			Kind::Syncing => {
				if let Some(stream) = stream_build!(
					self => b"Ethereum", b"CurrentBlock"
				) {
					self.subscriptions.add(subscriber, |sink| {
						let mut previous_syncing = network.is_major_syncing();
						let stream = stream
							.filter_map(move |(_, _)| {
								let syncing = network.is_major_syncing();
								if previous_syncing != syncing {
									previous_syncing = syncing;
									futures::future::ready(Some(syncing))
								} else {
									futures::future::ready(None)
								}
							})
							.map(|syncing| {
								return Ok::<
									Result<PubSubResult, jsonrpc_core::types::error::Error>,
									(),
								>(Ok(PubSubResult::SyncState(PubSubSyncStatus {
									syncing: syncing,
								})));
							})
							.compat();
						sink.sink_map_err(|e| warn!("Error sending notifications: {:?}", e))
							.send_all(stream)
							.map(|_| ())
					});
				}
			}
		}
	}

	fn unsubscribe(
		&self,
		_metadata: Option<Self::Metadata>,
		subscription_id: SubscriptionId,
	) -> JsonRpcResult<bool> {
		Ok(self.subscriptions.cancel(subscription_id))
	}
}
