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
	collections::BTreeMap,
	future::Future,
	marker::PhantomData,
	sync::Arc,
	time::{Duration, Instant},
};
use tokio::{sync::oneshot, time::sleep};

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

use moonbeam_rpc_core_trace::{FilterRequest, Trace as TraceT, TransactionTrace};
use moonbeam_rpc_primitives_debug::{single, DebugRuntimeApi};

pub struct Trace {
	pub requester: TraceFilterCacheRequester,
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

pub type TraceFilterCacheRequester = TracingUnboundedSender<(
	FilterRequest,
	oneshot::Sender<Result<Vec<TransactionTrace>>>,
)>;

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
	// TODO :
	// 1. Handle requests and add traces to the cache :
	//    Cache is a BTreeMap : Block height => Vec of TransactionTrace + expiration time
	//    No filtering is done in the cache, it stores all traces.
	//    Existing block in cache get the expiration time bumped.
	//
	//    Filtering is done on top :
	//    1. Apply the filter and return a list of indices to keep
	//    2. Use the indices to build the filtered vec of traces (with correct pointers)
	//
	// 2. Remove expired cache :
	//    Iterate over each block in the BTreeMap, and remove the entry if expired.
	//    Question : Is the expiration time and delay between checks configurable ?
	//    Other idea : Spawn a timer future when updating the cache, providing the height
	//        of the block. When woken up, check only this block expiration time.
	//        Will create a future for each request, but is more reactive to cleanup.
	//        Which is better ?

	// How to get Ethereum block hash and transactions :
	// EthApi::block_by_number -> Rich<Block>
	// Block.transactions (Full).hash
	// Debug RPC impl shows how to get the substrate equivalents for mapping.

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

			'service: loop {
				select! {
					req = rx.next() => {
						if let Some((req, response_tx)) = req {
							let block_heights: Vec<u32> = (
								req.from_block.unwrap_or(0)
								..= req.to_block.expect("end block range")
							).collect();

							for block_height in block_heights.iter() {
								if !cached_blocks.contains_key(block_height) {
									let traces = Self::cache_block(&eth_api, *block_height);
									let traces = match traces {
										Ok(traces) => traces,
										Err(err) => {
											// Error when creating one block cache, we return an
											// error.
											let _ = response_tx.send(Err(err));
											continue 'service;
										}
									};

									cached_blocks.insert(*block_height, CacheBlock {
										traces,
										expiration: Instant::now() + EXPIRATION_DELAY,
									});
								}

								todo!("Build filtered trace vec");
							}

							expiration_futures.push(async move {
								sleep(Duration::from_secs(60)).await;
								block_heights
							});

							todo!("send response");
						} else {
							// All Senders are dropped, stopping the service.
							break;
						}
					},
					blocks_to_check = expiration_futures.next() => {
						if let Some(blocks_to_check) = blocks_to_check {
							let now = Instant::now();

							let mut blocks_to_remove = vec![];

							for block in blocks_to_check {
								if let Some(cache) = cached_blocks.get(&block) {
									if cache.expiration <= now {
										blocks_to_remove.push(block);
									}
								}
							}
						} else {
							todo!("what to do when this end ?")
						}
					},
				}
			}
		};

		(fut, tx)
	}

	fn cache_block(eth_api: &A, block_height: u32) -> Result<Vec<TransactionTrace>> {
		// Fetch block data from RPC EthApi. false = only get transactions hashes, which is enough.
		let block = eth_api.block_by_number(BlockNumber::Num(block_height as u64), false)?;
		let block = block.ok_or_else(|| {
			internal_err(format!("Could not find block with height {}", block_height))
		})?;

		let block_hash = block.inner.hash.ok_or_else(|| {
			internal_err(format!(
				"Could not get the hash of block with height {}",
				block_height
			))
		})?;

		let transactions_hash = match &block.inner.transactions {
			BlockTransactions::Hashes(h) => h,
			_ => {
				return Err(internal_err(
					"EthApi::block_by_number should have returned transaction hashes",
				))
			}
		};

		todo!("Call Runtime API");
	}
}

struct CacheBlock {
	expiration: Instant,
	traces: Vec<TransactionTrace>,
}
