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
	collections::{btree_map::Entry, BTreeMap},
	future::Future,
	marker::PhantomData,
	sync::Arc,
	time::{Duration, Instant},
};
use tokio::{sync::oneshot, time::delay_for};

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
use tracing::Instrument;

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
const EXPIRATION_DELAY: Duration = Duration::from_secs(600);

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
	) -> (impl Future<Output = ()>, TraceFilterCacheRequester) {
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
								req,
								&mut touched_blocks
							);

							expiration_futures.push(async move {
								delay_for(Duration::from_secs(60)).await;
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
		req: FilterRequest,
		touched_blocks: &mut Vec<H256>,
	) -> Result<Vec<TransactionTrace>> {
		let from_block = Self::block_id(client, req.from_block)?;
		let to_block = Self::block_id(client, req.to_block)?;

		if to_block < from_block {
			return Err(internal_err(format!(
				"fromBlock ({}) must be greater or equal than ({})",
				from_block, to_block
			)));
		}

		let block_heights = from_block..=to_block;

		let from_address = req.from_address.unwrap_or_default();
		let to_address = req.to_address.unwrap_or_default();

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
					cache_block.expiration = Instant::now() + EXPIRATION_DELAY;

					cache_block
				}
				Entry::Vacant(entry) => {
					tracing::trace!(block_height, %block_hash, "Cache miss, replaying block ...");

					let traces = Self::cache_block(&backend, api, &block_header)?;

					entry.insert(CacheBlock {
						traces,
						expiration: Instant::now() + EXPIRATION_DELAY,
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

			traces.append(&mut block_traces);
		}

		// Paginations.
		let traces = traces.into_iter().skip(req.after.unwrap_or(0) as usize);
		let traces: Vec<_> = if let Some(take) = req.count {
			traces.take(take as usize).collect()
		} else {
			traces.collect()
		};

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
