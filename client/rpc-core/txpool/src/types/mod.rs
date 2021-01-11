mod content;
mod inspect;

use ethereum::Transaction as EthereumTransaction;
use ethereum_types::{H160, H256, U256};
use serde::Serialize;
use std::collections::HashMap;

pub use self::content::Transaction;
pub use self::inspect::Summary;

pub type TransactionMap<T> = HashMap<H160, HashMap<U256, T>>;

#[derive(Debug, Serialize)]
pub struct TxPoolResult<T: Serialize> {
	pub pending: T,
	pub queued: T,
}

pub trait Get {
	fn get(hash: H256, from_address: H160, txn: &EthereumTransaction) -> Self;
}
