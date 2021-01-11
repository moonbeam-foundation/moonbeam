use crate::GetT;
use ethereum::{Transaction as EthereumTransaction, TransactionAction};
use ethereum_types::{H160, H256, U256};
use serde::{Serialize, Serializer};

#[derive(Clone)]
pub struct Summary {
	pub to: Option<H160>,
	pub value: U256,
	pub gas: U256,
	pub gas_price: U256,
}

impl Serialize for Summary {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let to: String = match self.to {
			Some(t) => t.to_string(),
			_ => "null".to_string(),
		};
		let res = format!(
			"{}: {} wei + {} gas x {} wei",
			to, self.value, self.gas, self.gas_price
		);
		serializer.serialize_str(&res)
	}
}

impl GetT for Summary {
	fn get(_hash: H256, _from_address: H160, txn: &EthereumTransaction) -> Self {
		Self {
			to: match txn.action {
				TransactionAction::Call(to) => Some(to),
				_ => None,
			},
			value: txn.value,
			gas_price: txn.gas_price,
			gas: txn.gas_limit,
		}
	}
}
