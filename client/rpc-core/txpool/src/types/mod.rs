mod content;
mod inspect;

use ethereum_types::{H160, U256};
use serde::Serialize;
use std::collections::HashMap;

pub use self::content::Transaction;
pub use self::inspect::Summary;

pub type TransactionMap<T> = HashMap<H160, HashMap<U256, T>>;

#[derive(Debug, Serialize)]
pub struct ResponseData<T: Serialize> {
	pub pending: T,
	pub queued: T,
}
