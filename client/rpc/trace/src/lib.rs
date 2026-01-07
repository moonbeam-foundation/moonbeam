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

//! `trace_filter` RPC handler and its associated service task.
//! The RPC handler rely on `CacheTask` which provides a future that must be run inside a tokio
//! executor.
//!
//! The implementation is composed of multiple tasks :
//! - Many calls the RPC handler `Trace::filter`, communicating with the main task.
//! - A main `CacheTask` managing the cache and the communication between tasks.
//! - For each traced block an async task responsible to wait for a permit, spawn a blocking
//!   task and waiting for the result, then send it to the main `CacheTask`.

use futures::{select, FutureExt};
use std::{
	collections::{BTreeMap, HashMap},
	future::Future,
	marker::PhantomData,
	sync::Arc,
	time::{Duration, Instant},
};
use tokio::{
	sync::{mpsc, oneshot, Semaphore},
	time::interval,
};
use tracing::{instrument, Instrument};

use sc_client_api::backend::{Backend, StateBackend, StorageProvider};
use sc_service::SpawnTaskHandle;
use sp_api::{ApiExt, Core, ProvideRuntimeApi};
use sp_block_builder::BlockBuilder;
use sp_blockchain::{
	Backend as BlockchainBackend, Error as BlockChainError, HeaderBackend, HeaderMetadata,
};
use sp_runtime::traits::{BlakeTwo256, Block as BlockT, Header as HeaderT};
use substrate_prometheus_endpoint::Registry as PrometheusRegistry;

use ethereum_types::H256;
use fc_rpc::lru_cache::LRUCacheByteLimited;
use fc_storage::StorageOverride;
use fp_rpc::EthereumRuntimeRPCApi;

use moonbeam_client_evm_tracing::{
	formatters::ResponseFormatter,
	types::block::{self, TransactionTrace},
};
pub use moonbeam_rpc_core_trace::{FilterRequest, TraceServer};
use moonbeam_rpc_core_types::{RequestBlockId, RequestBlockTag};
use moonbeam_rpc_primitives_debug::DebugRuntimeApi;

type TxsTraceRes = Result<Vec<TransactionTrace>, String>;

/// Log target for trace cache operations
const CACHE_LOG_TARGET: &str = "trace-cache";

/// Maximum time allowed for tracing a single block.
/// Prevents indefinite hangs if runtime code has bugs.
const TRACING_TIMEOUT_SECS: u64 = 60;

/// RPC handler. Will communicate with a `CacheTask` through a `CacheRequester`.
pub struct Trace<B, C> {
	_phantom: PhantomData<B>,
	client: Arc<C>,
	requester: CacheRequester,
	max_count: u32,
	max_block_range: u32,
}

impl<B, C> Clone for Trace<B, C> {
	fn clone(&self) -> Self {
		Self {
			_phantom: PhantomData,
			client: Arc::clone(&self.client),
			requester: self.requester.clone(),
			max_count: self.max_count,
			max_block_range: self.max_block_range,
		}
	}
}

impl<B, C> Trace<B, C>
where
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	B::Header: HeaderT<Number = u32>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
{
	/// Create a new RPC handler.
	pub fn new(
		client: Arc<C>,
		requester: CacheRequester,
		max_count: u32,
		max_block_range: u32,
	) -> Self {
		Self {
			client,
			requester,
			max_count,
			max_block_range,
			_phantom: PhantomData,
		}
	}

	/// Convert an optional block ID (number or tag) to a block height.
	fn block_id(&self, id: Option<RequestBlockId>) -> Result<u32, &'static str> {
		match id {
			Some(RequestBlockId::Number(n)) => Ok(n),
			None | Some(RequestBlockId::Tag(RequestBlockTag::Latest)) => {
				Ok(self.client.info().best_number)
			}
			Some(RequestBlockId::Tag(RequestBlockTag::Earliest)) => Ok(0),
			Some(RequestBlockId::Tag(RequestBlockTag::Finalized)) => {
				Ok(self.client.info().finalized_number)
			}
			Some(RequestBlockId::Tag(RequestBlockTag::Pending)) => {
				Err("'pending' is not supported")
			}
			Some(RequestBlockId::Hash(_)) => Err("Block hash not supported"),
		}
	}

	/// `trace_filter` endpoint (wrapped in the trait implementation with futures compatibility)
	async fn filter(self, req: FilterRequest) -> TxsTraceRes {
		let from_block = self.block_id(req.from_block)?;
		let to_block = self.block_id(req.to_block)?;

		// Validate block range to prevent abuse
		let block_range = to_block.saturating_sub(from_block);
		if block_range > self.max_block_range {
			return Err(format!(
				"block range is too wide (maximum {})",
				self.max_block_range
			));
		}

		let block_heights = from_block..=to_block;

		let count = req.count.unwrap_or(self.max_count);
		if count > self.max_count {
			return Err(format!(
				"count ({}) can't be greater than maximum ({})",
				count, self.max_count
			));
		}

		// Build a list of all the Substrate block hashes that need to be traced.
		let mut block_hashes = vec![];
		for block_height in block_heights {
			if block_height == 0 {
				continue; // no traces for genesis block.
			}

			let block_hash = self
				.client
				.hash(block_height)
				.map_err(|e| {
					format!(
						"Error when fetching block {} header : {:?}",
						block_height, e
					)
				})?
				.ok_or_else(|| format!("Block with height {} don't exist", block_height))?;

			block_hashes.push(block_hash);
		}

		// Fetch traces for all blocks
		self.fetch_traces(req, &block_hashes, count as usize).await
	}

	async fn fetch_traces(
		&self,
		req: FilterRequest,
		block_hashes: &[H256],
		count: usize,
	) -> TxsTraceRes {
		let from_address = req.from_address.unwrap_or_default();
		let to_address = req.to_address.unwrap_or_default();

		let mut traces_amount: i64 = -(req.after.unwrap_or(0) as i64);
		let mut traces = vec![];

		for &block_hash in block_hashes {
			// Request the traces of this block to the cache service.
			// This will resolve quickly if the block is already cached, or wait until the block
			// has finished tracing.
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
						return Err(format!(
							"the amount of traces goes over the maximum ({}), please use 'after' \
							and 'count' in your request",
							self.max_count
						));
					}

					traces = traces.into_iter().take(count).collect();
					break;
				}
			}
		}

		Ok(traces)
	}
}

#[jsonrpsee::core::async_trait]
impl<B, C> TraceServer for Trace<B, C>
where
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	B::Header: HeaderT<Number = u32>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
{
	async fn filter(
		&self,
		filter: FilterRequest,
	) -> jsonrpsee::core::RpcResult<Vec<TransactionTrace>> {
		self.clone()
			.filter(filter)
			.await
			.map_err(fc_rpc::internal_err)
	}
}

/// Requests the cache task can accept.
enum CacheRequest {
	/// Fetch the traces for given block hash.
	/// The task will answer only when it has processed this block.
	GetTraces {
		/// Returns the array of traces or an error.
		sender: oneshot::Sender<TxsTraceRes>,
		/// Hash of the block.
		block: H256,
	},
}

/// Allows to interact with the cache task.
#[derive(Clone)]
pub struct CacheRequester(mpsc::Sender<CacheRequest>);

impl CacheRequester {
	/// Fetch the traces for given block hash.
	/// If the block is already cached, returns immediately.
	/// If the block is being traced, waits for the result.
	/// If the block is not cached, triggers tracing and waits for the result.
	#[instrument(skip(self))]
	pub async fn get_traces(&self, block: H256) -> TxsTraceRes {
		let (response_tx, response_rx) = oneshot::channel();
		let sender = self.0.clone();

		sender
			.send(CacheRequest::GetTraces {
				sender: response_tx,
				block,
			})
			.await
			.map_err(|e| format!("Trace cache task is overloaded or closed. Error : {:?}", e))?;

		response_rx
			.await
			.map_err(|e| {
				format!(
					"Trace cache task closed the response channel. Error : {:?}",
					e
				)
			})?
			.map_err(|e| format!("Failed to replay block. Error : {:?}", e))
	}
}

/// Entry in the wait list for a block being traced.
struct WaitListEntry {
	/// Time when this entry was created
	created_at: Instant,
	/// All requests waiting for this block to be traced
	waiters: Vec<oneshot::Sender<TxsTraceRes>>,
}

/// Wait list for requests pending the same block trace.
/// Multiple concurrent requests for the same block will be added to this list
/// and all will receive the result once tracing completes.
type WaitList = HashMap<H256, WaitListEntry>;

/// Message sent from blocking trace tasks back to the main cache task.
enum BlockingTaskMessage {
	/// The tracing is finished and the result is sent to the main task.
	Finished {
		block_hash: H256,
		result: TxsTraceRes,
	},
}

/// Type wrapper for the cache task, generic over the Client, Block and Backend types.
pub struct CacheTask<B, C, BE> {
	client: Arc<C>,
	backend: Arc<BE>,
	blocking_permits: Arc<Semaphore>,
	cache: LRUCacheByteLimited<H256, Vec<TransactionTrace>>,
	wait_list: WaitList,
	_phantom: PhantomData<B>,
}

impl<B, C, BE> CacheTask<B, C, BE>
where
	BE: Backend<B> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
	C: ProvideRuntimeApi<B>,
	C: StorageProvider<B, BE>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	B::Header: HeaderT<Number = u32>,
	C::Api: BlockBuilder<B>,
	C::Api: DebugRuntimeApi<B>,
	C::Api: EthereumRuntimeRPCApi<B>,
	C::Api: ApiExt<B>,
{
	/// Create a new cache task.
	///
	/// Returns a Future that needs to be added to a tokio executor, and a handle allowing to
	/// send requests to the task.
	pub fn create(
		client: Arc<C>,
		backend: Arc<BE>,
		cache_size_bytes: u64,
		blocking_permits: Arc<Semaphore>,
		overrides: Arc<dyn StorageOverride<B>>,
		prometheus: Option<PrometheusRegistry>,
		spawn_handle: SpawnTaskHandle,
	) -> (impl Future<Output = ()>, CacheRequester) {
		// Communication with the outside world - bounded channel to prevent memory exhaustion
		let (requester_tx, mut requester_rx) = mpsc::channel(10_000);

		// Task running in the service.
		let task = async move {
			let (blocking_tx, mut blocking_rx) =
				mpsc::channel(blocking_permits.available_permits() * 2);

			// Periodic cleanup interval for orphaned wait list entries
			let mut cleanup_interval = interval(Duration::from_secs(30));
			cleanup_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

			let mut inner = Self {
				client,
				backend,
				blocking_permits,
				cache: LRUCacheByteLimited::new(
					"trace-filter-blocks-cache",
					cache_size_bytes,
					prometheus,
				),
				wait_list: HashMap::new(),
				_phantom: Default::default(),
			};

			loop {
				select! {
					request = requester_rx.recv().fuse() => {
						match request {
							None => break,
							Some(CacheRequest::GetTraces {sender, block}) =>
								inner.request_get_traces(&blocking_tx, sender, block, overrides.clone(), &spawn_handle),
						}
					},
					message = blocking_rx.recv().fuse() => {
						if let Some(BlockingTaskMessage::Finished { block_hash, result }) = message {
							inner.blocking_finished(block_hash, result);
						}
					},
					_ = cleanup_interval.tick().fuse() => {
						inner.cleanup_wait_list();
					},
				}
			}
		}
		.instrument(tracing::debug_span!("trace_filter_cache"));

		(task, CacheRequester(requester_tx))
	}

	/// Handle a request to get traces for a specific block.
	/// - If cached: respond immediately
	/// - If pending: add to wait list
	/// - If new: spawn trace task and add to wait list
	fn request_get_traces(
		&mut self,
		blocking_tx: &mpsc::Sender<BlockingTaskMessage>,
		sender: oneshot::Sender<TxsTraceRes>,
		block: H256,
		overrides: Arc<dyn StorageOverride<B>>,
		spawn_handle: &SpawnTaskHandle,
	) {
		// Check if block is already cached
		if let Some(cached) = self.cache.get(&block) {
			// Cache hit - respond immediately (LRU handles access tracking)
			// Wrap in Ok since cache only stores successful traces
			let _ = sender.send(Ok(cached.clone()));
			return;
		}

		// Check if block is currently being traced
		if let Some(entry) = self.wait_list.get_mut(&block) {
			entry.waiters.push(sender);
			return;
		}

		// Add sender to wait list for this new block
		self.wait_list.insert(
			block,
			WaitListEntry {
				created_at: Instant::now(),
				waiters: vec![sender],
			},
		);

		// Spawn worker task to trace the block
		let blocking_permits = Arc::clone(&self.blocking_permits);
		let client = Arc::clone(&self.client);
		let backend = Arc::clone(&self.backend);
		let blocking_tx = blocking_tx.clone();

		spawn_handle.spawn(
			"trace-block",
			Some("trace-filter"),
			async move {
				// Wait for permit to limit concurrent tracing operations
				let _permit = blocking_permits.acquire().await;

				// Perform block tracing in blocking task with timeout
				let result = match tokio::time::timeout(
					Duration::from_secs(TRACING_TIMEOUT_SECS),
					tokio::task::spawn_blocking(move || {
						Self::cache_block(client, backend, block, overrides)
					}),
				)
				.await
				{
					// Timeout occurred
					Err(_elapsed) => {
						log::error!(
							target: CACHE_LOG_TARGET,
							"Tracing timeout for block {}",
							block
						);
						Err(format!(
							"Tracing timeout after {} seconds",
							TRACING_TIMEOUT_SECS
						))
					}
					// Task completed
					Ok(join_result) => {
						match join_result {
							// Task panicked
							Err(join_err) => Err(format!("Tracing panicked: {:?}", join_err)),
							// Task succeeded, return its result
							Ok(trace_result) => trace_result,
						}
					}
				};

				// Send result back to main task
				let _ = blocking_tx
					.send(BlockingTaskMessage::Finished {
						block_hash: block,
						result,
					})
					.await;
			}
			.instrument(tracing::trace_span!("trace_block", block = %block)),
		);
	}

	/// Handle completion of a block trace task.
	/// Sends result to all waiting requests and caches it.
	fn blocking_finished(&mut self, block_hash: H256, result: TxsTraceRes) {
		// Get all waiting senders for this block
		if let Some(entry) = self.wait_list.remove(&block_hash) {
			// Send result to all waiting requests
			for sender in entry.waiters {
				let _ = sender.send(result.clone());
			}

			// Only cache successful results to avoid polluting cache with transient errors
			// (network timeouts, temporary DB locks, etc.)
			// Failed blocks can be retried on next request
			if let Ok(traces) = result {
				self.cache.put(block_hash, traces);
			}
		}
	}

	/// Clean up orphaned wait list entries that have been pending too long.
	/// This handles cases where spawned tasks panic or get cancelled.
	fn cleanup_wait_list(&mut self) {
		let timeout = Duration::from_secs(TRACING_TIMEOUT_SECS + 10);
		let now = Instant::now();

		let mut to_remove = Vec::new();

		for (block_hash, entry) in &self.wait_list {
			if now.duration_since(entry.created_at) > timeout {
				log::warn!(
					target: CACHE_LOG_TARGET,
					"Cleaning up orphaned wait list entry for block {}",
					block_hash
				);
				to_remove.push(*block_hash);
			}
		}

		// Remove timed-out entries and notify waiters
		for block_hash in to_remove {
			if let Some(entry) = self.wait_list.remove(&block_hash) {
				for sender in entry.waiters {
					let _ = sender.send(Err(format!(
						"Trace request timeout (task failed or was cancelled)"
					)));
				}
			}
		}
	}

	/// (In blocking task) Use the Runtime API to trace the block.
	#[instrument(skip(client, backend, overrides))]
	fn cache_block(
		client: Arc<C>,
		backend: Arc<BE>,
		substrate_hash: H256,
		overrides: Arc<dyn StorageOverride<B>>,
	) -> TxsTraceRes {
		// Get Substrate block data.
		let api = client.runtime_api();
		let block_header = client
			.header(substrate_hash)
			.map_err(|e| {
				format!(
					"Error when fetching substrate block {} header : {:?}",
					substrate_hash, e
				)
			})?
			.ok_or_else(|| format!("Substrate block {} don't exist", substrate_hash))?;

		let height = *block_header.number();
		let substrate_parent_hash = *block_header.parent_hash();

		// Get Ethereum block data.
		let (eth_block, eth_transactions) = match (
			overrides.current_block(substrate_hash),
			overrides.current_transaction_statuses(substrate_hash),
		) {
			(Some(a), Some(b)) => (a, b),
			_ => {
				return Err(format!(
					"Failed to get Ethereum block data for Substrate block {}",
					substrate_hash
				))
			}
		};

		let eth_block_hash = eth_block.header.hash();
		let eth_tx_hashes = eth_transactions
			.iter()
			.map(|t| t.transaction_hash)
			.collect();

		// Get extrinsics (containing Ethereum ones)
		let extrinsics = backend
			.blockchain()
			.body(substrate_hash)
			.map_err(|e| {
				format!(
					"Blockchain error when fetching extrinsics of block {} : {:?}",
					height, e
				)
			})?
			.ok_or_else(|| format!("Could not find block {} when fetching extrinsics.", height))?;

		// Get DebugRuntimeApi version
		let trace_api_version = if let Ok(Some(api_version)) =
			api.api_version::<dyn DebugRuntimeApi<B>>(substrate_parent_hash)
		{
			api_version
		} else {
			return Err("Runtime api version call failed (trace)".to_string());
		};

		// Trace the block.
		let f = || -> Result<_, String> {
			let result = if trace_api_version >= 5 {
				api.trace_block(
					substrate_parent_hash,
					extrinsics,
					eth_tx_hashes,
					&block_header,
				)
			} else {
				// Get core runtime api version
				let core_api_version = if let Ok(Some(api_version)) =
					api.api_version::<dyn Core<B>>(substrate_parent_hash)
				{
					api_version
				} else {
					return Err("Runtime api version call failed (core)".to_string());
				};

				// Initialize block: calls the "on_initialize" hook on every pallet
				// in AllPalletsWithSystem
				// This was fine before pallet-message-queue because the XCM messages
				// were processed by the "setValidationData" inherent call and not on an
				// "on_initialize" hook, which runs before enabling XCM tracing
				if core_api_version >= 5 {
					api.initialize_block(substrate_parent_hash, &block_header)
						.map_err(|e| format!("Runtime api access error: {:?}", e))?;
				} else {
					#[allow(deprecated)]
					api.initialize_block_before_version_5(substrate_parent_hash, &block_header)
						.map_err(|e| format!("Runtime api access error: {:?}", e))?;
				}

				#[allow(deprecated)]
				api.trace_block_before_version_5(substrate_parent_hash, extrinsics, eth_tx_hashes)
			};

			result
				.map_err(|e| format!("Blockchain error when replaying block {} : {:?}", height, e))?
				.map_err(|e| {
					tracing::warn!(
						target: "tracing",
						"Internal runtime error when replaying block {} : {:?}",
						height,
						e
					);
					format!(
						"Internal runtime error when replaying block {} : {:?}",
						height, e
					)
				})?;

			Ok(moonbeam_rpc_primitives_debug::Response::Block)
		};

		let eth_transactions_by_index: BTreeMap<u32, H256> = eth_transactions
			.iter()
			.map(|t| (t.transaction_index, t.transaction_hash))
			.collect();

		let mut proxy = moonbeam_client_evm_tracing::listeners::CallList::default();
		proxy.using(f)?;

		let traces: Vec<TransactionTrace> =
			moonbeam_client_evm_tracing::formatters::TraceFilter::format(proxy)
				.ok_or("Fail to format proxy")?
				.into_iter()
				.filter_map(|mut trace| {
					match eth_transactions_by_index.get(&trace.transaction_position) {
						Some(transaction_hash) => {
							trace.block_hash = eth_block_hash;
							trace.block_number = height;
							trace.transaction_hash = *transaction_hash;

							// Reformat error messages.
							if let block::TransactionTraceOutput::Error(ref mut error) =
								trace.output
							{
								if error.as_slice() == b"execution reverted" {
									*error = b"Reverted".to_vec();
								}
							}

							Some(trace)
						}
						None => {
							log::warn!(
								target: "tracing",
								"A trace in block {} does not map to any known ethereum transaction. Trace: {:?}",
								height,
								trace,
							);
							None
						}
					}
				})
				.collect();

		Ok(traces)
	}
}
