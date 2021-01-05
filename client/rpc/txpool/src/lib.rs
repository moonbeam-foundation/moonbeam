use ethereum_types::{H256, U256};
use jsonrpc_core::Result;
pub use moonbeam_rpc_core_txpool::{
	Summary, Transaction, TxPool as TxPoolT, TxPoolServer, TxnPoolResult,
};
use sp_runtime::traits::Block as BlockT;
use sp_transaction_pool::TransactionPool;
use std::{marker::PhantomData, sync::Arc};

pub struct TxPool<B: BlockT, P> {
	_pool: Arc<P>,
	_marker: PhantomData<B>,
}

impl<B: BlockT, P> TxPool<B, P> {
	pub fn new(pool: Arc<P>) -> Self {
		Self {
			_pool: pool,
			_marker: PhantomData,
		}
	}
}

impl<B, P> TxPoolT for TxPool<B, P>
where
	B: BlockT<Hash = H256> + Send + Sync + 'static,
	P: TransactionPool<Block = B> + Send + Sync + 'static,
{
	fn content(&self) -> Result<TxnPoolResult<Transaction>> {
		unimplemented!();
	}

	fn inspect(&self) -> Result<TxnPoolResult<Summary>> {
		unimplemented!();
	}

	fn status(&self) -> Result<TxnPoolResult<U256>> {
		unimplemented!();
	}
}
