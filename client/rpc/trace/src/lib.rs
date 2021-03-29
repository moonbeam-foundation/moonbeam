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

use futures::{
	compat::Compat,
	future::{BoxFuture, TryFutureExt},
	select,
	stream::FuturesUnordered,
	FutureExt, SinkExt, StreamExt,
};
use moonbeam_rpc_debug::Debug;
use std::{
	collections::BTreeMap,
	future::Future,
	marker::PhantomData,
	sync::Arc,
	time::{Duration, Instant},
};
use tokio::{sync::oneshot, time::delay_for};

use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use sc_client_api::{
	backend::{AuxStore, Backend, StateBackend},
	StorageProvider,
};
use sc_network::{ExHashT, NetworkService};
use sc_transaction_graph::{ChainApi, Pool};
use sp_api::{BlockId, HeaderT, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{
	Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
use sp_transaction_pool::{InPoolTransaction, TransactionPool};
use sp_utils::mpsc::TracingUnboundedSender;

use ethereum_types::{H128, H256};
use fc_rpc_core::{
	types::{BlockNumber, BlockTransactions},
	EthApi,
};
use fp_rpc::{ConvertTransaction, EthereumRuntimeRPCApi};

pub use moonbeam_rpc_core_trace::{
	FilterRequest, RequestBlockId, Trace as TraceT, TraceServer, TransactionTrace,
};
use moonbeam_rpc_primitives_debug::{block, single, DebugRuntimeApi};
use tracing::{instrument, Instrument};

pub struct Trace {
	pub requester: TraceFilterCacheRequester,
}

impl Trace {
	pub fn new(requester: TraceFilterCacheRequester) -> Self {
		Self { requester }
	}
}

impl TraceT for Trace {
	fn filter(
		&self,
		filter: FilterRequest,
	) -> Compat<BoxFuture<'static, jsonrpc_core::Result<Vec<TransactionTrace>>>> {
		let mut requester = self.requester.clone();

		async move {
			let (tx, rx) = oneshot::channel();

			requester.send((filter, tx)).await.map_err(|err| {
				internal_err(format!(
					"failed to send request to trace filter service : {:?}",
					err
				))
			})?;

			rx.await.map_err(|err| {
				internal_err(format!(
					"trace filter service dropped the channel : {:?}",
					err
				))
			})?
		}
		.boxed()
		.compat()
	}
}

fn internal_err<T: ToString>(message: T) -> RpcError {
	RpcError {
		code: ErrorCode::InternalError,
		message: message.to_string(),
		data: None,
	}
}

pub type Responder = oneshot::Sender<Result<Vec<TransactionTrace>>>;
pub type TraceFilterCacheRequester = TracingUnboundedSender<(FilterRequest, Responder)>;

pub struct TraceFilterCache<B, C, BE, A>(PhantomData<(B, C, BE, A)>);

impl<B, C, BE, A> TraceFilterCache<B, C, BE, A>
where
	BE: Backend<B> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	BE::Blockchain: BlockchainBackend<B>,
	C: ProvideRuntimeApi<B> + AuxStore,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
	C: StorageProvider<B, BE>,
	C: HeaderMetadata<B, Error = BlockChainError> + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	C::Api: BlockBuilder<B, Error = BlockChainError>,
	C::Api: DebugRuntimeApi<B>,
	C::Api: EthereumRuntimeRPCApi<B>,
	A: EthApi,
{
	/// Create a task responsible to perform tracing on Ethereum blocks and keep these traces in
	/// a cache. This function returns a future containing the task main loop (which must be queued
	/// with tokio to work) and a channel to communicate with the task.
	///
	/// The cache strategy is used mainly to provide pagination, which would otherwise
	/// require to replay many times the same transactions.
	///
	/// The channel accepts a tuple containing a request and a oneshot sender to send back the
	/// response. The request defines a range of blocks to trace, which will be checked against
	/// the cache. If the cache doesn't contains this block, the Runtime API trace_block is
	/// performed, which will return an array of all traces (call/subcalls/create/suicide) without
	/// any filtering. After the cache contains all the requested blocks, an iterator is created
	/// over the range of blocks and filtered to only contains the to/from addresses (if any),
	/// followed by pagination. Using an iterator avoid cloning of the entire cache entries : only
	/// the filtered list of trace is cloned. This filtered list is then sent through the oneshot
	/// channel.
	///
	/// When a block cache entry is used, its expiration date is set to NOW + EXPIRATION_DELAY.
	/// A list of timer futures are responsible to wake up the task and check cache entry expiracy
	/// to free memory.
	pub fn task(
		client: Arc<C>,
		backend: Arc<BE>,
		eth_api: A,
	) -> (impl Future<Output = ()>, TraceFilterCacheRequester) {
		const EXPIRATION_DELAY: Duration = Duration::from_secs(600);

		let (tx, mut rx): (TraceFilterCacheRequester, _) =
			sp_utils::mpsc::tracing_unbounded("trace-filter-cache-requester");

		let fut = async move {
			let mut expiration_futures = FuturesUnordered::new();
			let mut cached_blocks = BTreeMap::<u32, CacheBlock>::new();

			tracing::info!("Begining Trace Filter Cache Task ...");

			'service: loop {
				select! {
					req = rx.next() => {
						if let Some((req, response_tx)) = req {
							let span = tracing::debug_span!("received request", request = ?req);
							let _guard = span.enter();

							tracing::trace!("Begining handling request");

							let from_block = match req.from_block {
								None => 1,
								Some(RequestBlockId::Number(n)) => n,
								_ => todo!("support latest/earliest/pending"),
							};

							let to_block = match req.to_block {
								None => todo!("support latest"),
								Some(RequestBlockId::Number(n)) => n,
								_ => todo!("support latest/earliest/pending"),
							};

							let from_address = req.from_address.unwrap_or_default();
							let to_address = req.to_address.unwrap_or_default();

							let range = from_block ..= to_block;

							let block_heights: Vec<u32> = range.clone().collect();

							// Fill cache if needed.
							for block_height in block_heights.iter() {
								let cached = cached_blocks.contains_key(block_height);
								if !cached {
									tracing::trace!(block_height, "Cache miss, replaying block ...");

									let traces = Self::cache_block(&client, &backend, &eth_api, *block_height);
									let traces = match traces {
										Ok(traces) => traces,
										Err(err) => {
											tracing::error!(block_height, ?err, "Failed to replay block, sending error response ...");

											let _ = response_tx.send(Err(err));
											continue 'service;
										}
									};

									cached_blocks.insert(*block_height, CacheBlock {
										traces,
										expiration: Instant::now() + EXPIRATION_DELAY,
									});
								} else {
									cached_blocks.get_mut(block_height).unwrap().expiration = Instant::now() + EXPIRATION_DELAY;
									tracing::trace!(block_height, "Cache hit, no need to replay block !");
								}
							}

							// Build filtered result.
							let traces = cached_blocks.range(range)
								.map(|(_, v)| &v.traces)
								.flatten()
								.filter(|trace| match trace.action {
									block::TransactionTraceAction::Call {from, to, ..} => {
										(from_address.is_empty() || from_address.contains(&from))
										&& (to_address.is_empty() || to_address.contains(&to))
									},
									block::TransactionTraceAction::Create {from, ..} => {
										(from_address.is_empty() || from_address.contains(&from))
										&& to_address.is_empty()
									},
									block::TransactionTraceAction::Suicide {address, ..} => {
										(from_address.is_empty() || from_address.contains(&address))
										&& to_address.is_empty()
									},
								})
								.skip(req.after.unwrap_or(0) as usize);

							let traces: Vec<_> = if let Some(take) = req.count {
								traces.take(take as usize)
								.cloned()
								.collect()
							} else {
								traces.cloned()
								.collect()
							};

							tracing::trace!(?traces, "Work done, sending response ...");

							// Send response.
							let _ = response_tx.send(Ok(traces));

							// Add expiration wake up.
							expiration_futures.push(async move {
								delay_for(Duration::from_secs(60)).await;
								block_heights
							});
						} else {
							// All Senders are dropped, stopping the service.
							break;
						}
					},
					blocks_to_check = expiration_futures.next() => {
						if let Some(blocks_to_check) = blocks_to_check {
							tracing::trace!(?blocks_to_check, "Waking up for potential cache cleaning ...");

							let now = Instant::now();

							let mut blocks_to_remove = vec![];

							for block in blocks_to_check {
								if let Some(cache) = cached_blocks.get(&block) {
									if cache.expiration <= now {
										tracing::trace!(block, "Removing expired block");

										blocks_to_remove.push(block);
									}
								}
							}
						}
					},
				}
			}
		}
		.instrument(tracing::debug_span!("trace_filter_cache"));

		(fut, tx)
	}

	fn cache_block(
		client: &C,
		backend: &BE,
		eth_api: &A,
		block_height: u32,
	) -> Result<Vec<TransactionTrace>> {
		if block_height == 0 {
			return Err(internal_err("Tracing genesis block is not allowed"));
		}

		// Fetch block data from RPC EthApi. false = only get transactions hashes, which is enough.
		let eth_block = eth_api.block_by_number(BlockNumber::Num(block_height as u64), false)?;
		let eth_block = eth_block.ok_or_else(|| {
			internal_err(format!("Could not find block with height {}", block_height))
		})?;

		let eth_block_hash = eth_block.inner.hash.ok_or_else(|| {
			internal_err(format!(
				"Could not get the hash of block with height {}",
				block_height
			))
		})?;

		tracing::trace!("block hash : {}", eth_block_hash);

		let transactions_hash = match &eth_block.inner.transactions {
			BlockTransactions::Hashes(h) => h,
			_ => {
				return Err(internal_err(
					"EthApi::block_by_number should have returned transaction hashes",
				))
			}
		};

		let substrate_block_id = match Debug::<B, C, BE>::load_hash(client, eth_block_hash)
			.map_err(|err| internal_err(format!("{:?}", err)))?
		{
			Some(hash) => hash,
			_ => return Err(internal_err("Block hash not found".to_string())),
		};

		// This handle allow to keep changes between txs in an internal buffer.
		let api = client.runtime_api();
		let substrate_block_header = client.header(substrate_block_id).unwrap().unwrap();
		// The re-execute the block we start from the parent block final state.
		let substrate_parent_block_id = BlockId::<B>::Hash(*substrate_block_header.parent_hash());
		let extrinsics = backend
			.blockchain()
			.body(substrate_block_id)
			.unwrap()
			.unwrap();

		// Trace the block.
		let mut traces: Vec<_> = api
			.trace_block(&substrate_parent_block_id, extrinsics)
			.map_err(|e| {
				internal_err(format!(
					"Blockchain error when replaying block {} : {:?}",
					block_height, e
				))
			})?
			.map_err(|e| {
				internal_err(format!(
					"Internal runtime error when replaying block {} : {:?}",
					block_height, e
				))
			})?;

		// Fill missing data.
		for trace in traces.iter_mut() {
			trace.block_hash = eth_block_hash;
			trace.block_number = block_height;
			trace.transaction_hash = *transactions_hash
				.get(trace.transaction_position as usize)
				.expect("amount of eth transactions should match");

			// Reformat error messages.
			if let block::TransactionTraceOutput::Error(ref mut error) = trace.output {
				if error.as_slice() == b"execution reverted" {
					*error = b"Reverted".to_vec();
				}
			}
		}

		Ok(traces)
	}
}

struct CacheBlock {
	expiration: Instant,
	traces: Vec<TransactionTrace>,
}
