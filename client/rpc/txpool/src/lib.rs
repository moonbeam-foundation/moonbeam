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

use ethereum::TransactionMessage;
use ethereum_types::{H160, H256, U256};
use fc_rpc::{internal_err, public_key};
use jsonrpc_core::Result as RpcResult;
pub use moonbeam_rpc_core_txpool::{
	GetT, Summary, Transaction, TransactionMap, TxPool as TxPoolT, TxPoolResult, TxPoolServer,
};
use sha3::{Digest, Keccak256};
use sp_api::{BlockId, ProvideRuntimeApi};
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_runtime::traits::Block as BlockT;
use sp_transaction_pool::{InPoolTransaction, TransactionPool};
use std::collections::HashMap;
use std::{marker::PhantomData, sync::Arc};

use moonbeam_rpc_primitives_txpool::TxPoolRuntimeApi;

pub struct TxPool<B: BlockT, C, P> {
	client: Arc<C>,
	pool: Arc<P>,
	_marker: PhantomData<B>,
}

impl<B, C, P> TxPool<B, C, P>
where
	C: ProvideRuntimeApi<B>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B> + 'static,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	P: TransactionPool<Block = B> + Send + Sync + 'static,
	C::Api: TxPoolRuntimeApi<B>,
{
	fn map_build<T>(&self) -> RpcResult<TransactionMap<T>>
	where
		T: GetT,
	{
		let txs: Vec<_> = self
			.pool
			.ready()
			.map(|in_pool_tx| in_pool_tx.data().clone())
			.collect();

		let best_block: BlockId<B> = BlockId::Hash(self.client.info().best_hash);
		let ethereum_txns = self
			.client
			.runtime_api()
			.extrinsic_filter(&best_block, txs)
			.map_err(|err| {
				internal_err(format!("fetch runtime extrinsic filter failed: {:?}", err))
			})?;
		let mut out = TransactionMap::<T>::new();
		for txn in ethereum_txns.iter() {
			let transaction_message = TransactionMessage::from(txn.clone());
			let hash = transaction_message.hash();
			let from_address = match public_key(txn) {
				Ok(pk) => H160::from(H256::from_slice(Keccak256::digest(&pk).as_slice())),
				Err(_e) => H160::default(),
			};
			out.entry(from_address)
				.or_insert_with(HashMap::new)
				.insert(txn.nonce, T::get(hash, from_address, txn));
		}
		Ok(out)
	}
}

impl<B: BlockT, C, P> TxPool<B, C, P> {
	pub fn new(client: Arc<C>, pool: Arc<P>) -> Self {
		Self {
			client,
			pool,
			_marker: PhantomData,
		}
	}
}

impl<B, C, P> TxPoolT for TxPool<B, C, P>
where
	C: ProvideRuntimeApi<B>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B>,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	P: TransactionPool<Block = B> + Send + Sync + 'static,
	C::Api: TxPoolRuntimeApi<B>,
{
	fn content(&self) -> RpcResult<TxPoolResult<TransactionMap<Transaction>>> {
		let pending = self.map_build::<Transaction>()?;
		Ok(TxPoolResult {
			pending,
			// Future queue not yet supported. We need to do something like:
			// - Use InpoolTransaction::requires() to get the TransactionTag bytes.
			// - Somehow decode and identify the tag to either add it to the future or pending pool.
			queued: HashMap::new(),
		})
	}

	fn inspect(&self) -> RpcResult<TxPoolResult<TransactionMap<Summary>>> {
		let pending = self.map_build::<Summary>()?;
		Ok(TxPoolResult {
			pending,
			queued: HashMap::new(),
		})
	}

	fn status(&self) -> RpcResult<TxPoolResult<U256>> {
		let status = self.pool.status();
		Ok(TxPoolResult {
			pending: U256::from(status.ready),
			queued: U256::from(status.future),
		})
	}
}
