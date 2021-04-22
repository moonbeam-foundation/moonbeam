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
use std::{
	collections::{btree_map::Entry, BTreeMap, VecDeque},
	future::Future,
	marker::PhantomData,
	sync::Arc,
	time::{Duration, Instant},
};
use tokio::{
	sync::{mpsc, oneshot, watch, Semaphore},
	time::delay_for,
};

use jsonrpc_core::Result;
use sc_client_api::backend::Backend;
use sp_api::{ApiRef, BlockId, HeaderT, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{
	Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_runtime::traits::Block as BlockT;
use sp_utils::mpsc::TracingUnboundedSender;

use ethereum_types::H256;
use fc_rpc::internal_err;
use fp_rpc::EthereumRuntimeRPCApi;

pub use moonbeam_rpc_core_trace::{
	FilterRequest, RequestBlockId, RequestBlockTag, Trace as TraceT, TraceServer, TransactionTrace,
};
use moonbeam_rpc_primitives_debug::{block, DebugRuntimeApi};
use tracing::{instrument, Instrument};

pub struct Trace2<B, C> {
	_phantom: PhantomData<B>,
	client: Arc<C>,
	requester: CacheRequester,
	max_count: u32,
}

impl<B, C> Clone for Trace2<B, C> {
	fn clone(&self) -> Self {
		Self {
			_phantom: PhantomData::default(),
			client: Arc::clone(&self.client),
			requester: self.requester.clone(),
			max_count: self.max_count,
		}
	}
}

impl<B, C> Trace2<B, C>
where
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	B::Header: HeaderT<Number = u32>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
{
	pub fn new(client: Arc<C>, requester: CacheRequester, max_count: u32) -> Self {
		Self {
			client,
			requester,
			max_count,
			_phantom: PhantomData::default(),
		}
	}

	fn block_id(&self, id: Option<RequestBlockId>) -> Result<u32> {
		match id {
			Some(RequestBlockId::Number(n)) => Ok(n),
			None | Some(RequestBlockId::Tag(RequestBlockTag::Latest)) => {
				Ok(self.client.info().best_number)
			}
			Some(RequestBlockId::Tag(RequestBlockTag::Earliest)) => Ok(0),
			Some(RequestBlockId::Tag(RequestBlockTag::Pending)) => {
				Err(internal_err("'pending' is not supported"))
			}
		}
	}

	async fn filter(self, req: FilterRequest) -> Result<Vec<TransactionTrace>> {
		let from_block = self.block_id(req.from_block)?;
		let to_block = self.block_id(req.to_block)?;
		let block_heights = from_block..=to_block;

		let count = req.count.unwrap_or(self.max_count);
		if count > self.max_count {
			return Err(internal_err(format!(
				"count ({}) can't be greater than maximum ({})",
				count, self.max_count
			)));
		}

		let mut block_hashes = vec![];

		for block_height in block_heights {
			if block_height == 0 {
				continue; // no traces for genesis block.
			}

			let block_id = BlockId::<B>::Number(block_height);
			let block_header = self
				.client
				.header(block_id)
				.map_err(|e| {
					internal_err(format!(
						"Error when fetching block {} header : {:?}",
						block_height, e
					))
				})?
				.ok_or_else(|| {
					internal_err(format!("Block with height {} don't exist", block_height))
				})?;

			let block_hash = block_header.hash();

			block_hashes.push(block_hash);
		}

		let batch_id = self.requester.start_batch(block_hashes.clone()).await?;
		let res = self.fetch_traces(req, &block_hashes, count as usize).await;
		self.requester.stop_batch(batch_id).await;

		res
	}

	async fn fetch_traces(
		&self,
		req: FilterRequest,
		block_hashes: &[H256],
		count: usize,
	) -> Result<Vec<TransactionTrace>> {
		let from_address = req.from_address.unwrap_or_default();
		let to_address = req.to_address.unwrap_or_default();

		let mut traces_amount: i64 = -(req.after.unwrap_or(0) as i64);
		let mut traces = vec![];

		for &block_hash in block_hashes {
			let block_traces = self.requester.get_traces(block_hash).await?;

			// Filter addresses.
			let mut block_traces: Vec<_> = block_traces
				.iter()
				.filter(|trace| match trace.action {
					block::TransactionTraceAction::Call { from, to, .. } => {
						(from_address.is_empty() || from_address.contains(&from))
							&& (to_address.is_empty() || to_address.contains(&to))
					}
					block::TransactionTraceAction::Create { from, .. } => {
						(from_address.is_empty() || from_address.contains(&from))
							&& to_address.is_empty()
					}
					block::TransactionTraceAction::Suicide { address, .. } => {
						(from_address.is_empty() || from_address.contains(&address))
							&& to_address.is_empty()
					}
				})
				.cloned()
				.collect();

			// Don't insert anything if we're still before "after"
			traces_amount += block_traces.len() as i64;
			if traces_amount > 0 {
				let traces_amount = traces_amount as usize;
				// If the current Vec of traces is across the "after" marker,
				// we skip some elements of it.
				if traces_amount < block_traces.len() {
					let skip = block_traces.len() - traces_amount;
					block_traces = block_traces.into_iter().skip(skip).collect();
				}

				traces.append(&mut block_traces);

				// If we go over "count" (the limit), we trim and exit the loop,
				// unless we used the default maximum, in which case we return an error.
				if traces_amount >= count {
					if req.count.is_none() {
						return Err(internal_err(format!(
							"the amount of traces goes over the maximum ({}), please use 'after' \
							and 'count' in your request",
							self.max_count
						)));
					}

					traces = traces.into_iter().take(count).collect();
					break;
				}
			}
		}

		Ok(traces)
	}
}

impl<B, C> TraceT for Trace2<B, C>
where
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	B::Header: HeaderT<Number = u32>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
{
	fn filter(
		&self,
		filter: FilterRequest,
	) -> Compat<BoxFuture<'static, jsonrpc_core::Result<Vec<TransactionTrace>>>> {
		self.clone().filter(filter).boxed().compat()
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CacheBatchId(u64);

/// Enumeration of all kinds of requests the cache task can accept.
enum CacheRequest {
	/// Request to start caching the provided range of blocks.
	/// The task will add to blocks to its pool and immediately return the batch ID.
	StartBatch {
		/// Returns the ID of the batch for cancellation.
		sender: oneshot::Sender<CacheBatchId>,
		/// List of block hash to trace.
		blocks: Vec<H256>,
	},
	/// Fetch the traces for given block hash.
	/// The task will answer only when it has processed this block.
	GetTraces {
		/// Returns the array of traces or an error.
		sender: oneshot::Sender<Result<Vec<TransactionTrace>>>,
		/// Hash of the block.
		block: H256,
	},
	/// Notify the cache that it can stop the batch with that ID. Any block contained only in
	/// this batch and still in the waiting pool will be discarded.
	StopBatch { batch_id: CacheBatchId },
}

/// Allows to interact with the cache task.
#[derive(Clone)]
pub struct CacheRequester(TracingUnboundedSender<CacheRequest>);

impl CacheRequester {
	/// Request to start caching the provided range of blocks.
	/// The task will add to blocks to its pool and immediately return the batch ID.
	#[instrument(skip(self))]
	pub async fn start_batch(&self, blocks: Vec<H256>) -> Result<CacheBatchId> {
		let (response_tx, response_rx) = oneshot::channel();
		let mut sender = self.0.clone();

		sender
			.send(CacheRequest::StartBatch {
				sender: response_tx,
				blocks,
			})
			.await
			.map_err(|e| {
				internal_err(format!(
					"Failed to send request to the trace cache task. Error : {:?}",
					e
				))
			})?;

		response_rx.await.map_err(|e| {
			internal_err(format!(
				"Trace cache task closed the response channel. Error : {:?}",
				e
			))
		})
	}

	/// Fetch the traces for given block hash.
	/// The task will answer only when it has processed this block.
	/// The block should be part of a batch first. If no batch has requested the block it will not
	/// be processed.
	#[instrument(skip(self))]
	pub async fn get_traces(&self, block: H256) -> Result<Vec<TransactionTrace>> {
		let (response_tx, response_rx) = oneshot::channel();
		let mut sender = self.0.clone();

		sender
			.send(CacheRequest::GetTraces {
				sender: response_tx,
				block,
			})
			.await
			.map_err(|e| {
				internal_err(format!(
					"Failed to send request to the trace cache task. Error : {:?}",
					e
				))
			})?;

		response_rx
			.await
			.map_err(|e| {
				internal_err(format!(
					"Trace cache task closed the response channel. Error : {:?}",
					e
				))
			})?
			.map_err(|e| internal_err(format!("Failed to replay block. Error : {:?}", e)))
	}

	/// Notify the cache that it can stop the batch with that ID. Any block contained only in
	/// this batch and still in the waiting pool will be discarded.
	#[instrument(skip(self))]
	pub async fn stop_batch(&self, batch_id: CacheBatchId) {
		let mut sender = self.0.clone();

		// Here we don't care if the request has been accepted or refused, the caller can't
		// do anything with it.
		let _ = sender
			.send(CacheRequest::StopBatch { batch_id })
			.await
			.map_err(|e| {
				internal_err(format!(
					"Failed to send request to the trace cache task. Error : {:?}",
					e
				))
			});
	}
}

struct CacheBlock2 {
	active_batch_count: usize,
	state: CacheBlockState,
}

enum CacheBlockState {
	/// Block has been added to the pool blocks to be replayed.
	/// It may be currently waiting to be replayed or being replayed.
	Pooled {
		started: bool,
		waiting_requests: Vec<oneshot::Sender<Result<Vec<TransactionTrace>>>>,
		unqueue_sender: oneshot::Sender<()>,
	},
	Cached {
		traces: Result<Vec<TransactionTrace>>,
	},
}

enum BlockingTaskMessage {
	Started {
		block_hash: H256,
	},
	Finished {
		block_hash: H256,
		result: Result<Vec<TransactionTrace>>,
	},
}

/// Type wrapper for the cache task, generic over the Client, Block and Backend types.
pub struct CacheTask<B, C, BE>(PhantomData<(B, C, BE)>);

impl<B, C, BE> CacheTask<B, C, BE>
where
	BE: Backend<B> + 'static,
	C: ProvideRuntimeApi<B>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	B::Header: HeaderT<Number = u32>,
	C::Api: BlockBuilder<B>,
	C::Api: DebugRuntimeApi<B>,
	C::Api: EthereumRuntimeRPCApi<B>,
{
	/// Create a new cache task.
	///
	/// Returns a Future that needs to be added to a tokio executor, and an handle allowing to
	/// send requests to the task.
	pub fn create(
		client: Arc<C>,
		backend: Arc<BE>,
		cache_duration: Duration,
		blocking_permits: Arc<Semaphore>,
	) -> (impl Future<Output = ()>, CacheRequester) {
		let (requester_tx, mut requester_rx) =
			sp_utils::mpsc::tracing_unbounded("trace-filter-cache");

		let task = async move {
			let mut batch_expirations = FuturesUnordered::new();
			let mut cached_blocks = BTreeMap::<H256, CacheBlock2>::new();
			let mut batches = BTreeMap::<u64, Vec<H256>>::new();
			let mut next_batch_id = 0u64;

			let (blocking_tx, blocking_rx) =
				mpsc::channel(blocking_permits.available_permits() * 2);
			let mut blocking_rx = blocking_rx.fuse();

			loop {
				select! {
					request = requester_rx.next() => {
						match request {
							None => break,
							Some(CacheRequest::StartBatch {sender, blocks}) => {
								tracing::trace!("Starting batch {}", next_batch_id);
								batches.insert(next_batch_id, blocks.clone());

								for block in blocks {
									// The block is already in the cache, awesome !
									if let Some(block_cache) = cached_blocks.get_mut(&block) {
										block_cache.active_batch_count += 1;
										tracing::trace!("Cache hit for block {}, now used by {} batches.", block, block_cache.active_batch_count);
									}
									// Otherwise we need to queue this block for tracing.
									else {
										tracing::trace!("Cache miss for block {}, pooling it for tracing.", block);

										let blocking_permits = Arc::clone(&blocking_permits);
										let (unqueue_sender, unqueue_receiver) = oneshot::channel();
										let client = Arc::clone(&client);
										let backend = Arc::clone(&backend);
										let mut blocking_tx = blocking_tx.clone();

										// Spawn all block caching asynchronously.
										// It will wait to obtain a permit, then spawn a blocking task.
										// When the blocking task returns its result, it is send
										// thought a channel to the main task loop.
										tokio::spawn(async move {
											tracing::trace!("Waiting for blocking permit or task cancellation");
											let _permit = select!(
												_ = unqueue_receiver.fuse() => {
												tracing::trace!("Tracing of the block has been cancelled.");
													return;
												},
												permit = blocking_permits.acquire().fuse() => permit,
											);

											// Warn the main task that block tracing as started, and
											// this block cache entry should not be removed.
											let _ = blocking_tx.send(BlockingTaskMessage::Started {
												block_hash: block,
											}).await;

											tracing::trace!("Start block tracing in a blocking task.");

											// Perform block tracing in a tokio blocking task.
											let result = async {
												tokio::task::spawn_blocking(move || {
													Self::cache_block(client, backend, block)
												}).await
													.map_err(|e| internal_err(format!("Tracing Substrate block {} panicked : {:?}", block, e)))?
											}.await;

											tracing::trace!("Block tracing finished, sending result to main task.");

											// Send response to main task.
											let _ = blocking_tx.send(BlockingTaskMessage::Finished {
												block_hash: block,
												result,
											}).await;
										}.instrument(tracing::trace_span!("Block tracing", block = %block)));

										cached_blocks.insert(block, CacheBlock2 {
											active_batch_count: 1,
											state: CacheBlockState::Pooled {
												started: false,
												waiting_requests: vec![],
												unqueue_sender,
											}
										});
									}
								}

								let _ = sender.send(CacheBatchId(next_batch_id));

								next_batch_id = next_batch_id.overflowing_add(1).0;
							},
							Some(CacheRequest::GetTraces {sender, block}) => {
								if let Some(block_cache) = cached_blocks.get_mut(&block) {
									match &mut block_cache.state {
										CacheBlockState::Pooled { ref mut waiting_requests, ..} => {
											tracing::warn!("A request asked a pooled block ({}), adding it to the list of waiting requests.", block);
											waiting_requests.push(sender)},
										CacheBlockState::Cached { ref traces, .. } => {
											tracing::warn!("A request asked a cache block ({}), sending the traces directly.", block);
											let _ = sender.send(traces.clone());
										}
									}
								} else {
									tracing::warn!("An RPC request asked to get a block ({}) which was not batched.", block);
									let _ = sender.send(Err(internal_err(format!("RPC request asked a block ({}) that was not batched", block))));
								}
							},
							Some(CacheRequest::StopBatch {batch_id}) => {
								tracing::trace!("Stopping batch {}", batch_id.0);

								batch_expirations.push(async move {
									delay_for(cache_duration).await;
									batch_id
								});

								if let Some(blocks) = batches.get(&batch_id.0) {
									for block in blocks {
										let mut remove = false;

										// We remove early the block cache if this batch is the last
										// pooling this block.
										if let Some(block_cache) = cached_blocks.get_mut(block) {
											if block_cache.active_batch_count == 1 && matches!(block_cache.state, CacheBlockState::Pooled {started: false ,..}) {
												remove = true;
											}
										}

										if remove {
											tracing::trace!("Pooled block {} is no longer requested.", block);
											// Remove block from the cache. Drops the value,
											// closing all the channels contained in it.
											let _ = cached_blocks.remove(&block);
										}
									}
								}

							},
						}
					},
					message = blocking_rx.next() => {
						match message {
							None => (),
							Some(BlockingTaskMessage::Started { block_hash }) => {
								if let Some(block_cache) = cached_blocks.get_mut(&block_hash) {
									if let CacheBlockState::Pooled {ref mut started, ..} = block_cache.state {
										*started = true;
									}
								}
							},
							Some(BlockingTaskMessage::Finished { block_hash, result }) => {
								// In some cases it might be possible to receive traces of a block
								// that has no entry in the cache because it was removed of the pool
								// and received a permit concurrently. We just ignore it.
								//
								// TODO : Should we add it back ? Should it have an active_batch_count
								// of 1 then ?
								if let Some(block_cache) = cached_blocks.get_mut(&block_hash) {
									if let CacheBlockState::Pooled {ref mut waiting_requests, ..} = block_cache.state {
										// Send result in waiting channels
										while let Some(channel) = waiting_requests.pop() {
											let _ = channel.send(result.clone());
										}

										// Update cache entry
										block_cache.state = CacheBlockState::Cached {
											traces: result,
										};
									}
								}
							},
						}
					},
					batch_id = batch_expirations.next() => {
						match batch_id {
							None => (),
							Some(batch_id) => {
								if let Some(batch) = batches.remove(&batch_id.0) {
									for block in batch {
										// For each block of the batch, we remove it if it was the
										// last batch containing it.
										let mut remove = false;
										if let Some(block_cache) = cached_blocks.get_mut(&block) {
											block_cache.active_batch_count -= 1;

											if block_cache.active_batch_count == 0 {
												remove = true;
											}
										}

										if remove {
											let _ = cached_blocks.remove(&block);
										}
									}
								}
							}
						}
					}
				}
			}
		}
		.instrument(tracing::debug_span!("trace_filter_cache"));

		(task, CacheRequester(requester_tx))
	}

	fn cache_block(
		client: Arc<C>,
		backend: Arc<BE>,
		substrate_hash: H256,
	) -> Result<Vec<TransactionTrace>> {
		let substrate_block_id = BlockId::Hash(substrate_hash);

		let api = client.runtime_api();
		let block_header = client
			.header(substrate_block_id)
			.map_err(|e| {
				internal_err(format!(
					"Error when fetching substrate block {} header : {:?}",
					substrate_hash, e
				))
			})?
			.ok_or_else(|| {
				internal_err(format!("Subtrate block {} don't exist", substrate_block_id))
			})?;

		let height = *block_header.number();
		let substrate_parent_id = BlockId::<B>::Hash(*block_header.parent_hash());

		let (eth_block, _, eth_transactions) = api
			.current_all(&BlockId::Hash(substrate_hash))
			.map_err(|e| {
				internal_err(format!(
					"Failed to get Ethereum block data for Substrate block {} : {:?}",
					substrate_hash, e
				))
			})?;

		let (eth_block, eth_transactions) = match (eth_block, eth_transactions) {
			(Some(a), Some(b)) => (a, b),
			_ => {
				return Err(internal_err(format!(
					"Failed to get Ethereum block data for Substrate block {}",
					substrate_hash
				)))
			}
		};

		let eth_block_hash = eth_block.header.hash();

		let extrinsics = backend
			.blockchain()
			.body(substrate_block_id)
			.unwrap()
			.unwrap();

		// Trace the block.
		let mut traces: Vec<_> = api
			.trace_block(&substrate_parent_id, extrinsics)
			.map_err(|e| {
				internal_err(format!(
					"Blockchain error when replaying block {} : {:?}",
					height, e
				))
			})?
			.map_err(|e| {
				internal_err(format!(
					"Internal runtime error when replaying block {} : {:?}",
					height, e
				))
			})?;

		// Fill missing data.
		for trace in traces.iter_mut() {
			trace.block_hash = eth_block_hash;
			trace.block_number = height;
			trace.transaction_hash = eth_transactions
				.get(trace.transaction_position as usize)
				.expect("amount of eth transactions should match")
				.transaction_hash;

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

// OLD IMPLEMENTATION FOR REFERENCE :

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

pub type Responder = oneshot::Sender<Result<Vec<TransactionTrace>>>;
pub type TraceFilterCacheRequester = TracingUnboundedSender<(FilterRequest, Responder)>;

pub struct TraceFilterCache<B, C, BE>(PhantomData<(B, C, BE)>);

impl<B, C, BE> TraceFilterCache<B, C, BE>
where
	BE: Backend<B> + 'static,
	C: ProvideRuntimeApi<B>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	B::Header: HeaderT<Number = u32>,
	C::Api: BlockBuilder<B>,
	C::Api: DebugRuntimeApi<B>,
	C::Api: EthereumRuntimeRPCApi<B>,
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
		max_count: u32,
		cache_duration: u32,
	) -> (impl Future<Output = ()>, TraceFilterCacheRequester) {
		let cache_duration = Duration::from_secs(cache_duration as u64);

		let (tx, mut rx): (TraceFilterCacheRequester, _) =
			sp_utils::mpsc::tracing_unbounded("trace-filter-cache-requester");

		let fut = async move {
			let mut expiration_futures = FuturesUnordered::new();
			// Substrate block hash => Block cache
			let mut cached_blocks = BTreeMap::<H256, CacheBlock>::new();

			tracing::info!("Begining Trace Filter Cache Task ...");

			loop {
				select! {
					req = rx.next() => {
						if let Some((req, response_tx)) = req {
							let span = tracing::debug_span!("received request", request = ?req);
							let _guard = span.enter();

							tracing::trace!("Begining handling request");

							let mut touched_blocks = vec![];
							let res = Self::handle_request(
								&client,
								&backend,
								&mut cached_blocks,
								&mut touched_blocks,
								req,
								max_count,
								cache_duration,
							);

							expiration_futures.push(async move {
								delay_for(cache_duration).await;
								touched_blocks
							});

							tracing::trace!(?res, "Work done, sending response ...");

							// Send response.
							let _ = response_tx.send(res);
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
										tracing::trace!(%block, "Removing expired block");

										blocks_to_remove.push(block);
									}
								}
							}

							for block in blocks_to_remove {
								let _ = cached_blocks.remove(&block);
							}
						}
					},
				}
			}
		}
		.instrument(tracing::debug_span!("trace_filter_cache"));

		(fut, tx)
	}

	fn block_id(client: &C, id: Option<RequestBlockId>) -> Result<u32> {
		match id {
			Some(RequestBlockId::Number(n)) => Ok(n),
			None | Some(RequestBlockId::Tag(RequestBlockTag::Latest)) => {
				Ok(client.info().best_number)
			}
			Some(RequestBlockId::Tag(RequestBlockTag::Earliest)) => Ok(0),
			Some(RequestBlockId::Tag(RequestBlockTag::Pending)) => {
				Err(internal_err("'pending' is not supported"))
			}
		}
	}

	fn handle_request(
		client: &C,
		backend: &BE,
		cached_blocks: &mut BTreeMap<H256, CacheBlock>,
		touched_blocks: &mut Vec<H256>,
		req: FilterRequest,
		max_count: u32,
		cache_duration: Duration,
	) -> Result<Vec<TransactionTrace>> {
		let from_block = Self::block_id(client, req.from_block)?;
		let to_block = Self::block_id(client, req.to_block)?;

		let count = req.count.unwrap_or(max_count);

		if count > max_count {
			return Err(internal_err(format!(
				"count ({}) can't be greater than maximum ({})",
				count, max_count
			)));
		}

		let count = count as usize;

		let block_heights = from_block..=to_block;

		let from_address = req.from_address.unwrap_or_default();
		let to_address = req.to_address.unwrap_or_default();

		let mut traces_amount: i64 = -(req.after.unwrap_or(0) as i64);
		let mut traces = vec![];
		for block_height in block_heights {
			if block_height == 0 {
				continue; // no traces for genesis block.
			}

			let api = client.runtime_api();
			let block_id = BlockId::<B>::Number(block_height);
			let block_header = client
				.header(block_id)
				.map_err(|e| {
					internal_err(format!(
						"Error when fetching block {} header : {:?}",
						block_height, e
					))
				})?
				.ok_or_else(|| {
					internal_err(format!("Block with height {} don't exist", block_height))
				})?;

			let block_hash = block_header.hash();

			let cache_block = match cached_blocks.entry(block_hash) {
				Entry::Occupied(entry) => {
					tracing::trace!(block_height, %block_hash, "Cache hit, no need to replay block !");

					let cache_block = entry.into_mut();
					cache_block.expiration = Instant::now() + cache_duration;

					cache_block
				}
				Entry::Vacant(entry) => {
					tracing::trace!(block_height, %block_hash, "Cache miss, replaying block ...");

					let traces = Self::cache_block(&backend, api, &block_header)?;

					entry.insert(CacheBlock {
						traces,
						expiration: Instant::now() + cache_duration,
					})
				}
			};

			touched_blocks.push(block_hash);

			// Filter addresses.
			let mut block_traces: Vec<_> = cache_block
				.traces
				.iter()
				.filter(|trace| match trace.action {
					block::TransactionTraceAction::Call { from, to, .. } => {
						(from_address.is_empty() || from_address.contains(&from))
							&& (to_address.is_empty() || to_address.contains(&to))
					}
					block::TransactionTraceAction::Create { from, .. } => {
						(from_address.is_empty() || from_address.contains(&from))
							&& to_address.is_empty()
					}
					block::TransactionTraceAction::Suicide { address, .. } => {
						(from_address.is_empty() || from_address.contains(&address))
							&& to_address.is_empty()
					}
				})
				.cloned()
				.collect();

			// Don't insert anything if we're still before "after"
			traces_amount += block_traces.len() as i64;
			if traces_amount > 0 {
				let traces_amount = traces_amount as usize;
				// If the current Vec of traces is across the "after" marker,
				// we skip some elements of it.
				if traces_amount < block_traces.len() {
					let skip = block_traces.len() - traces_amount;
					block_traces = block_traces.into_iter().skip(skip).collect();
				}

				traces.append(&mut block_traces);

				// If we go over "count" (the limit), we trim and exit the loop,
				// unless we used the default maximum, in which case we return an error.
				if traces_amount >= count {
					if req.count.is_none() {
						return Err(internal_err(format!(
							"the amount of traces goes over the maximum ({}), please use 'after' \
							and 'count' in your request",
							max_count
						)));
					}

					traces = traces.into_iter().take(count).collect();
					break;
				}
			}
		}

		Ok(traces)
	}

	fn cache_block(
		backend: &BE,
		api: ApiRef<C::Api>,
		block_header: &B::Header,
	) -> Result<Vec<TransactionTrace>> {
		let height = *block_header.number();
		let substrate_hash = block_header.hash();
		let substrate_block_id = BlockId::Hash(substrate_hash);
		let substrate_parent_id = BlockId::<B>::Hash(*block_header.parent_hash());

		let (eth_block, _, eth_transactions) = api
			.current_all(&BlockId::Hash(substrate_hash))
			.map_err(|e| {
				internal_err(format!(
					"Failed to get Ethereum block data for Substrate block {} : {:?}",
					substrate_hash, e
				))
			})?;

		let (eth_block, eth_transactions) = match (eth_block, eth_transactions) {
			(Some(a), Some(b)) => (a, b),
			_ => {
				return Err(internal_err(format!(
					"Failed to get Ethereum block data for Substrate block {}",
					substrate_hash
				)))
			}
		};

		let eth_block_hash = eth_block.header.hash();

		let extrinsics = backend
			.blockchain()
			.body(substrate_block_id)
			.unwrap()
			.unwrap();

		// Trace the block.
		let mut traces: Vec<_> = api
			.trace_block(&substrate_parent_id, extrinsics)
			.map_err(|e| {
				internal_err(format!(
					"Blockchain error when replaying block {} : {:?}",
					height, e
				))
			})?
			.map_err(|e| {
				internal_err(format!(
					"Internal runtime error when replaying block {} : {:?}",
					height, e
				))
			})?;

		// Fill missing data.
		for trace in traces.iter_mut() {
			trace.block_hash = eth_block_hash;
			trace.block_number = height;
			trace.transaction_hash = eth_transactions
				.get(trace.transaction_position as usize)
				.expect("amount of eth transactions should match")
				.transaction_hash;

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
