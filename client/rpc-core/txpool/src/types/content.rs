use crate::GetT;
use ethereum::{Transaction as EthereumTransaction, TransactionAction};
use ethereum_types::{H160, H256, U256};
use fc_rpc_core::types::Bytes;
use serde::Serialize;

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
	/// Hash
	pub hash: H256,
	/// Nonce
	pub nonce: U256,
	/// Block hash
	pub block_hash: Option<H256>,
	/// Block number
	pub block_number: Option<U256>,
	/// Sender
	pub from: H160,
	/// Recipient
	pub to: Option<H160>,
	/// Transfered value
	pub value: U256,
	/// Gas Price
	pub gas_price: U256,
	/// Gas
	pub gas: U256,
	/// Data
	pub input: Bytes,
}

impl GetT for Transaction {
	fn get(hash: H256, from_address: H160, txn: &EthereumTransaction) -> Self {
		Self {
			hash,
			nonce: txn.nonce,
			block_hash: Some(H256::default()), // or None?
			block_number: None,
			from: from_address,
			to: match txn.action {
				TransactionAction::Call(to) => Some(to),
				_ => None,
			},
			value: txn.value,
			gas_price: txn.gas_price,
			gas: txn.gas_limit,
			input: Bytes(txn.input.clone()),
		}
	}
}
