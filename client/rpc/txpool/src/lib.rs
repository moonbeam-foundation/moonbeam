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

use ethereum_types::{H160, H256, U256};
use fc_rpc::{internal_err, public_key};
use jsonrpc_core::Result as RpcResult;
pub use moonbeam_rpc_core_txpool::{
	GetT, Summary, Transaction, TransactionMap, TxPool as TxPoolT, TxPoolResult, TxPoolServer,
};
use sc_transaction_graph::{ChainApi, Pool};
use serde::Serialize;
use sha3::{Digest, Keccak256};
use sp_api::{BlockId, ProvideRuntimeApi};
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_runtime::traits::Block as BlockT;
use sp_transaction_pool::InPoolTransaction;
use std::collections::HashMap;
use std::{marker::PhantomData, sync::Arc};

use moonbeam_rpc_primitives_txpool::{TxPoolResponse, TxPoolRuntimeApi};

pub struct TxPool<B: BlockT, C, A: ChainApi> {
	client: Arc<C>,
	graph: Arc<Pool<A>>,
	_marker: PhantomData<B>,
}

impl<B, C, A> TxPool<B, C, A>
where
	C: ProvideRuntimeApi<B>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B> + 'static,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	A: ChainApi<Block = B> + 'static,
	C::Api: TxPoolRuntimeApi<B>,
{
	/// Use the transaction graph interface to get the extrinsics currently in the ready and future
	/// queues.
	fn map_build<T>(&self) -> RpcResult<TxPoolResult<TransactionMap<T>>>
	where
		T: GetT + Serialize,
	{
		// Collect transactions in the ready validated pool.
		let txs_ready = self
			.graph
			.validated_pool()
			.ready()
			.map(|in_pool_tx| in_pool_tx.data().clone())
			.collect();

		// Collect transactions in the future validated pool.
		let txs_future = self
			.graph
			.validated_pool()
			.futures()
			.iter()
			.map(|(_hash, extrinsic)| extrinsic.clone())
			.collect();

		// Use the runtime to match the (here) opaque extrinsics against ethereum transactions.
		let best_block: BlockId<B> = BlockId::Hash(self.client.info().best_hash);
		let ethereum_txns: TxPoolResponse = self
			.client
			.runtime_api()
			.extrinsic_filter(&best_block, txs_ready, txs_future)
			.map_err(|err| {
				internal_err(format!("fetch runtime extrinsic filter failed: {:?}", err))
			})?;
		// Build the T response.
		let mut pending = TransactionMap::<T>::new();
		for txn in ethereum_txns.ready.iter() {
			let hash = H256::from_slice(Keccak256::digest(&rlp::encode(txn)).as_slice());
			let from_address = match public_key(txn) {
				Ok(pk) => H160::from(H256::from_slice(Keccak256::digest(&pk).as_slice())),
				Err(_e) => H160::default(),
			};
			pending
				.entry(from_address)
				.or_insert_with(HashMap::new)
				.insert(txn.nonce, T::get(hash, from_address, txn));
		}
		let mut queued = TransactionMap::<T>::new();
		for txn in ethereum_txns.future.iter() {
			let hash = H256::from_slice(Keccak256::digest(&rlp::encode(txn)).as_slice());
			let from_address = match public_key(txn) {
				Ok(pk) => H160::from(H256::from_slice(Keccak256::digest(&pk).as_slice())),
				Err(_e) => H160::default(),
			};
			queued
				.entry(from_address)
				.or_insert_with(HashMap::new)
				.insert(txn.nonce, T::get(hash, from_address, txn));
		}
		Ok(TxPoolResult { pending, queued })
	}
}

impl<B: BlockT, C, A: ChainApi> TxPool<B, C, A> {
	pub fn new(client: Arc<C>, graph: Arc<Pool<A>>) -> Self {
		Self {
			client,
			graph,
			_marker: PhantomData,
		}
	}
}

impl<B, C, A> TxPoolT for TxPool<B, C, A>
where
	C: ProvideRuntimeApi<B>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	A: ChainApi<Block = B> + 'static,
	C::Api: TxPoolRuntimeApi<B>,
{
	fn content(&self) -> RpcResult<TxPoolResult<TransactionMap<Transaction>>> {
		self.map_build::<Transaction>()
	}

	fn inspect(&self) -> RpcResult<TxPoolResult<TransactionMap<Summary>>> {
		self.map_build::<Summary>()
	}

	fn status(&self) -> RpcResult<TxPoolResult<U256>> {
		let status = self.graph.validated_pool().status();
		Ok(TxPoolResult {
			pending: U256::from(status.ready),
			queued: U256::from(status.future),
		})
	}
}
