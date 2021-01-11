use ethereum_types::{H256, U256};
use jsonrpc_core::Result;
use jsonrpc_core::{Error, ErrorCode};
pub use moonbeam_rpc_core_txpool::{
	Summary, Transaction, TransactionMap, TxPool as TxPoolT, TxPoolResult, TxPoolServer,
};
use sp_api::{BlockId, ProvideRuntimeApi};
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_runtime::traits::Block as BlockT;
use sp_transaction_pool::{InPoolTransaction, TransactionPool};
use std::{marker::PhantomData, sync::Arc};

use moonbeam_rpc_primitives_txpool::TxPoolRuntimeApi;

pub fn internal_err<T: ToString>(message: T) -> Error {
	Error {
		code: ErrorCode::InternalError,
		message: message.to_string(),
		data: None,
	}
}

pub struct TxPool<B: BlockT, C, P> {
	client: Arc<C>,
	pool: Arc<P>,
	_marker: PhantomData<B>,
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

/// TODO:
/// Future pool not yet supported. We need to
/// 	- Use InpoolTransaction::requires() to get the TransactionTag bytes.
/// 	- Somehow decode and identify the tag to either add it to the future or pending pool.

impl<B, C, P> TxPoolT for TxPool<B, C, P>
where
	C: ProvideRuntimeApi<B>,
	C: HeaderMetadata<B, Error = BlockChainError> + HeaderBackend<B> + 'static,
	C: Send + Sync + 'static,
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	P: TransactionPool<Block = B> + Send + Sync + 'static,
	C::Api: TxPoolRuntimeApi<B>,
{
	fn content(&self) -> Result<TxPoolResult<TransactionMap<Transaction>>> {
		let txs: Vec<<B as BlockT>::Extrinsic> = self
			.pool
			.ready()
			.map(|in_pool_tx| in_pool_tx.data().clone())
			.collect();

		let best_block: BlockId<B> = BlockId::Hash(self.client.info().best_hash);
		let _ethereum_txns = self.client.runtime_api().extrinsic_filter(&best_block, txs);
		// TODO continue here with building the TxnPoolResult
		unimplemented!();
	}

	fn inspect(&self) -> Result<TxPoolResult<TransactionMap<Summary>>> {
		unimplemented!();
	}

	fn status(&self) -> Result<TxPoolResult<U256>> {
		let status = self.pool.status();
		Ok(TxPoolResult {
			pending: U256::from(status.ready),
			queued: U256::from(status.future),
		})
	}
}
