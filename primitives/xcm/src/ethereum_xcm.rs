// Copyright 2019-2022 PureStake Inc.
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

use ethereum::{
	AccessList, AccessListItem, EIP1559Transaction, EIP2930Transaction, LegacyTransaction,
	TransactionAction, TransactionSignature, TransactionV2,
};
use ethereum_types::{H160, H256, U256};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, TypeInfo)]
/// Manually sets a gas fee.
pub struct ManualEthereumXcmFee {
	/// Legacy or Eip-2930, all fee will be used.
	pub gas_price: Option<U256>,
	/// Eip-1559, must be at least the on-chain base fee at the time of applying the xcm
	/// and will use up to the defined value.
	pub max_fee_per_gas: Option<U256>,
}

/// Xcm transact's Ethereum transaction configurable fee.
#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub enum EthereumXcmFee {
	/// Manually set gas fee.
	Manual(ManualEthereumXcmFee),
	/// Use the on-chain base fee at the time of processing the xcm.
	Auto,
}

/// Xcm transact's Ethereum transaction.
#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub enum EthereumXcmTransaction {
	V1(EthereumXcmTransactionV1),
}

/// Value for `r` and `s` for the invalid signature included in Xcm transact's Ethereum transaction.
pub fn rs_id() -> H256 {
	H256::from_low_u64_be(1u64)
}

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub struct EthereumXcmTransactionV1 {
	/// Gas limit to be consumed by EVM execution.
	pub gas_limit: U256,
	/// Fee configuration of choice.
	pub fee_payment: EthereumXcmFee,
	/// Either a Call (the callee, account or contract address) or Create (currently unsupported).
	pub action: TransactionAction,
	/// Value to be transfered.
	pub value: U256,
	/// Input data for a contract call.
	pub input: Vec<u8>,
	/// Map of addresses to be pre-paid to warm storage.
	pub access_list: Option<Vec<(H160, Vec<H256>)>>,
}

pub trait XcmToEthereum {
	fn into_transaction_v2(&self, base_fee: U256, nonce: U256) -> Option<TransactionV2>;
}

impl XcmToEthereum for EthereumXcmTransaction {
	fn into_transaction_v2(&self, base_fee: U256, nonce: U256) -> Option<TransactionV2> {
		match self {
			EthereumXcmTransaction::V1(v1_tx) => v1_tx.into_transaction_v2(base_fee, nonce),
		}
	}
}

impl XcmToEthereum for EthereumXcmTransactionV1 {
	fn into_transaction_v2(&self, base_fee: U256, nonce: U256) -> Option<TransactionV2> {
		// We dont support creates for now
		if self.action == TransactionAction::Create {
			return None;
		}
		let from_tuple_to_access_list = |t: &Vec<(H160, Vec<H256>)>| -> AccessList {
			t.iter()
				.map(|item| AccessListItem {
					address: item.0.clone(),
					storage_keys: item.1.clone(),
				})
				.collect::<Vec<AccessListItem>>()
		};

		let (gas_price, max_fee) = match &self.fee_payment {
			EthereumXcmFee::Manual(fee_config) => {
				(fee_config.gas_price, fee_config.max_fee_per_gas)
			}
			EthereumXcmFee::Auto => (None, Some(base_fee)),
		};
		match (gas_price, max_fee) {
			(Some(gas_price), None) => {
				// Legacy or Eip-2930
				if let Some(ref access_list) = self.access_list {
					// Eip-2930
					Some(TransactionV2::EIP2930(EIP2930Transaction {
						chain_id: 0,
						nonce,
						gas_price,
						gas_limit: self.gas_limit,
						action: self.action,
						value: self.value,
						input: self.input.clone(),
						access_list: from_tuple_to_access_list(access_list),
						odd_y_parity: true,
						r: rs_id(),
						s: rs_id(),
					}))
				} else {
					// Legacy
					Some(TransactionV2::Legacy(LegacyTransaction {
						nonce,
						gas_price,
						gas_limit: self.gas_limit,
						action: self.action,
						value: self.value,
						input: self.input.clone(),
						signature: TransactionSignature::new(42, rs_id(), rs_id())?,
					}))
				}
			}
			(None, Some(max_fee)) => {
				// Eip-1559
				Some(TransactionV2::EIP1559(EIP1559Transaction {
					chain_id: 0,
					nonce,
					max_fee_per_gas: max_fee,
					max_priority_fee_per_gas: U256::zero(),
					gas_limit: self.gas_limit,
					action: self.action,
					value: self.value,
					input: self.input.clone(),
					access_list: if let Some(ref access_list) = self.access_list {
						from_tuple_to_access_list(access_list)
					} else {
						Vec::new()
					},
					odd_y_parity: true,
					r: rs_id(),
					s: rs_id(),
				}))
			}
			_ => return None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_into_ethereum_tx_with_auto_fee() {
		let xcm_transaction = EthereumXcmTransactionV1 {
			gas_limit: U256::from(1),
			fee_payment: EthereumXcmFee::Auto,
			action: TransactionAction::Call(H160::default()),
			value: U256::from(0),
			input: vec![1u8],
			access_list: None,
		};
		let nonce = U256::from(0);
		let base_fee = U256::from(1);
		let expected_tx = Some(TransactionV2::EIP1559(EIP1559Transaction {
			chain_id: 0,
			nonce,
			max_fee_per_gas: base_fee,
			max_priority_fee_per_gas: U256::zero(),
			gas_limit: U256::from(1),
			action: TransactionAction::Call(H160::default()),
			value: U256::from(0),
			input: vec![1u8],
			access_list: vec![],
			odd_y_parity: true,
			r: H256::from_low_u64_be(1u64),
			s: H256::from_low_u64_be(1u64),
		}));

		assert_eq!(
			xcm_transaction.into_transaction_v2(base_fee, nonce),
			expected_tx
		);
	}

	#[test]
	fn test_legacy() {
		let xcm_transaction = EthereumXcmTransactionV1 {
			gas_limit: U256::from(1),
			fee_payment: EthereumXcmFee::Manual(ManualEthereumXcmFee {
				gas_price: Some(U256::from(1)),
				max_fee_per_gas: None,
			}),
			action: TransactionAction::Call(H160::default()),
			value: U256::from(0),
			input: vec![1u8],
			access_list: None,
		};
		let nonce = U256::from(0);
		let gas_price = U256::from(1);
		let expected_tx = Some(TransactionV2::Legacy(LegacyTransaction {
			nonce,
			gas_price,
			gas_limit: U256::from(1),
			action: TransactionAction::Call(H160::default()),
			value: U256::from(0),
			input: vec![1u8],
			signature: TransactionSignature::new(42, rs_id(), rs_id()).unwrap(),
		}));

		assert_eq!(
			xcm_transaction.into_transaction_v2(gas_price, nonce),
			expected_tx
		);
	}
	#[test]
	fn test_eip_2930() {
		let access_list = Some(vec![(H160::default(), vec![H256::default()])]);
		let from_tuple_to_access_list = |t: &Vec<(H160, Vec<H256>)>| -> AccessList {
			t.iter()
				.map(|item| AccessListItem {
					address: item.0.clone(),
					storage_keys: item.1.clone(),
				})
				.collect::<Vec<AccessListItem>>()
		};

		let xcm_transaction = EthereumXcmTransactionV1 {
			gas_limit: U256::from(1),
			fee_payment: EthereumXcmFee::Manual(ManualEthereumXcmFee {
				gas_price: Some(U256::from(1)),
				max_fee_per_gas: None,
			}),
			action: TransactionAction::Call(H160::default()),
			value: U256::from(0),
			input: vec![1u8],
			access_list: access_list.clone(),
		};

		let nonce = U256::from(0);
		let gas_price = U256::from(1);
		let expected_tx = Some(TransactionV2::EIP2930(EIP2930Transaction {
			chain_id: 0,
			nonce,
			gas_price,
			gas_limit: U256::from(1),
			action: TransactionAction::Call(H160::default()),
			value: U256::from(0),
			input: vec![1u8],
			access_list: from_tuple_to_access_list(&access_list.unwrap()),
			odd_y_parity: true,
			r: H256::from_low_u64_be(1u64),
			s: H256::from_low_u64_be(1u64),
		}));

		assert_eq!(
			xcm_transaction.into_transaction_v2(gas_price, nonce),
			expected_tx
		);
	}
}
