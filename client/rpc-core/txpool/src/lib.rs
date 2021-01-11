use ethereum_types::U256;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

mod types;

pub use crate::types::{Summary, Transaction, TransactionMap, TxPoolResult};

pub use rpc_impl_TxPool::gen_server::TxPool as TxPoolServer;

#[rpc(server)]
pub trait TxPool {
	#[rpc(name = "txpool_content")]
	fn content(&self) -> Result<TxPoolResult<TransactionMap<Transaction>>>;

	#[rpc(name = "txpool_inspect")]
	fn inspect(&self) -> Result<TxPoolResult<TransactionMap<Summary>>>;

	#[rpc(name = "txpool_status")]
	fn status(&self) -> Result<TxPoolResult<U256>>;
}
