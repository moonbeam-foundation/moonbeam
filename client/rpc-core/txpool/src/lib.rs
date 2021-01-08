use ethereum_types::U256;
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;

mod types;

pub use crate::types::{ResponseData as TxnPoolResult, Summary, Transaction};

pub use rpc_impl_TxPool::gen_server::TxPool as TxPoolServer;

#[rpc(server)]
pub trait TxPool {
	#[rpc(name = "txpool_content")]
	fn content(&self) -> Result<TxnPoolResult<Transaction>>;

	#[rpc(name = "txpool_inspect")]
	fn inspect(&self) -> Result<TxnPoolResult<Summary>>;

	#[rpc(name = "txpool_status")]
	fn status(&self) -> Result<TxnPoolResult<U256>>;
}
