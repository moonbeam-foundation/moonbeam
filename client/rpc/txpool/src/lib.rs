// Copyright 2019-2022 PureStake Inc.
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
use jsonrpsee::core::RpcResult;
pub use moonbeam_rpc_core_txpool::{
	GetT, Summary, Transaction, TransactionMap, TxPoolResult, TxPoolServer,
};
use sc_transaction_pool::{ChainApi, Pool};
use sc_transaction_pool_api::InPoolTransaction;
use serde::Serialize;
use sha3::{Digest, Keccak256};
use sp_api::{ApiExt, ProvideRuntimeApi};
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_runtime::traits::Block as BlockT;
use std::collections::HashMap;
use std::{marker::PhantomData, sync::Arc};

use moonbeam_rpc_primitives_txpool::{
	Transaction as TransactionV2, TxPoolResponse, TxPoolRuntimeApi,
};

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
		let best_block = self.client.info().best_hash;
		let api = self.client.runtime_api();
		let api_version =
			if let Ok(Some(api_version)) = api.api_version::<dyn TxPoolRuntimeApi<B>>(best_block) {
				api_version
			} else {
				return Err(internal_err(
					"failed to retrieve Runtime Api version".to_string(),
				));
			};
		let ethereum_txns: TxPoolResponse = if api_version == 1 {
			#[allow(deprecated)]
			let res = api.extrinsic_filter_before_version_2(best_block, txs_ready, txs_future)
				.map_err(|err| {
					internal_err(format!("fetch runtime extrinsic filter failed: {:?}", err))
				})?;
			TxPoolResponse {
				ready: res
					.ready
					.iter()
					.map(|t| TransactionV2::Legacy(t.clone()))
					.collect(),
				future: res
					.future
					.iter()
					.map(|t| TransactionV2::Legacy(t.clone()))
					.collect(),
			}
		} else {
			api.extrinsic_filter(best_block, txs_ready, txs_future)
				.map_err(|err| {
					internal_err(format!("fetch runtime extrinsic filter failed: {:?}", err))
				})?
		};
		// Build the T response.
		let mut pending = TransactionMap::<T>::new();
		for txn in ethereum_txns.ready.iter() {
			let hash = txn.hash();
			let nonce = match txn {
				TransactionV2::Legacy(t) => t.nonce,
				TransactionV2::EIP2930(t) => t.nonce,
				TransactionV2::EIP1559(t) => t.nonce,
			};
			let from_address = match public_key(txn) {
				Ok(pk) => H160::from(H256::from_slice(Keccak256::digest(&pk).as_slice())),
				Err(_e) => H160::default(),
			};
			pending
				.entry(from_address)
				.or_insert_with(HashMap::new)
				.insert(nonce, T::get(hash, from_address, txn));
		}
		let mut queued = TransactionMap::<T>::new();
		for txn in ethereum_txns.future.iter() {
			let hash = txn.hash();
			let nonce = match txn {
				TransactionV2::Legacy(t) => t.nonce,
				TransactionV2::EIP2930(t) => t.nonce,
				TransactionV2::EIP1559(t) => t.nonce,
			};
			let from_address = match public_key(txn) {
				Ok(pk) => H160::from(H256::from_slice(Keccak256::digest(&pk).as_slice())),
				Err(_e) => H160::default(),
			};
			queued
				.entry(from_address)
				.or_insert_with(HashMap::new)
				.insert(nonce, T::get(hash, from_address, txn));
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

impl<B, C, A> TxPoolServer for TxPool<B, C, A>
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

impl<B: BlockT, C, A: ChainApi> Clone for TxPool<B, C, A> {
	fn clone(&self) -> Self {
		Self::new(self.client.clone(), self.graph.clone())
	}
}
