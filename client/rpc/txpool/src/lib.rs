use ethereum_types::U256;
use jsonrpc_core::Result;
pub use moonbeam_rpc_core_txpool::{
	Summary, Transaction, TxPool as TxPoolT, TxPoolServer, TxnPoolResult,
};

pub struct TxPool {}

impl TxPool {
	pub fn new() -> Self {
		Self {}
	}
}

impl TxPoolT for TxPool {
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
