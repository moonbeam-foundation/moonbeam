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
use frame_support::{traits::ConstU32, BoundedVec};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

// polkadot/blob/19f6665a6162e68cd2651f5fe3615d6676821f90/xcm/src/v3/mod.rs#L1193
// Defensively we increase this value to allow UMP fragments through xcm-transactor to prepare our
// runtime for a relay upgrade where the xcm instruction weights are not ZERO hardcoded. If that
// happens stuff will break in our side.
// Rationale behind the value: e.g. staking unbond will go above 64kb and thus
// required_weight_at_most must be below overall weight but still above whatever value we decide to
// set. For this reason we set here a value that makes sense for the overall weight.
pub const DEFAULT_PROOF_SIZE: u64 = 256 * 1024;

/// Max. allowed size of 65_536 bytes.
pub const MAX_ETHEREUM_XCM_INPUT_SIZE: u32 = 2u32.pow(16);

/// Ensure that a proxy between `delegator` and `delegatee` exists in order to deny or grant
/// permission to do xcm-transact to `transact_through_proxy`.
pub trait EnsureProxy<AccountId> {
	fn ensure_ok(delegator: AccountId, delegatee: AccountId) -> Result<(), &'static str>;
}

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
	V2(EthereumXcmTransactionV2),
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
	pub input: BoundedVec<u8, ConstU32<MAX_ETHEREUM_XCM_INPUT_SIZE>>,
	/// Map of addresses to be pre-paid to warm storage.
	pub access_list: Option<Vec<(H160, Vec<H256>)>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub struct EthereumXcmTransactionV2 {
	/// Gas limit to be consumed by EVM execution.
	pub gas_limit: U256,
	/// Either a Call (the callee, account or contract address) or Create (currently unsupported).
	pub action: TransactionAction,
	/// Value to be transfered.
	pub value: U256,
	/// Input data for a contract call. Max. size 65_536 bytes.
	pub input: BoundedVec<u8, ConstU32<MAX_ETHEREUM_XCM_INPUT_SIZE>>,
	/// Map of addresses to be pre-paid to warm storage.
	pub access_list: Option<Vec<(H160, Vec<H256>)>>,
}

pub trait XcmToEthereum {
	fn into_transaction_v2(&self, nonce: U256, chain_id: u64) -> Option<TransactionV2>;
}

impl XcmToEthereum for EthereumXcmTransaction {
	fn into_transaction_v2(&self, nonce: U256, chain_id: u64) -> Option<TransactionV2> {
		match self {
			EthereumXcmTransaction::V1(v1_tx) => v1_tx.into_transaction_v2(nonce, chain_id),
			EthereumXcmTransaction::V2(v2_tx) => v2_tx.into_transaction_v2(nonce, chain_id),
		}
	}
}

impl XcmToEthereum for EthereumXcmTransactionV1 {
	fn into_transaction_v2(&self, nonce: U256, chain_id: u64) -> Option<TransactionV2> {
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
			EthereumXcmFee::Auto => (None, Some(U256::zero())),
		};
		match (gas_price, max_fee) {
			(Some(gas_price), None) => {
				// Legacy or Eip-2930
				if let Some(ref access_list) = self.access_list {
					// Eip-2930
					Some(TransactionV2::EIP2930(EIP2930Transaction {
						chain_id,
						nonce,
						gas_price,
						gas_limit: self.gas_limit,
						action: self.action,
						value: self.value,
						input: self.input.to_vec(),
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
						input: self.input.to_vec(),
						signature: TransactionSignature::new(42, rs_id(), rs_id())?,
					}))
				}
			}
			(None, Some(max_fee)) => {
				// Eip-1559
				Some(TransactionV2::EIP1559(EIP1559Transaction {
					chain_id,
					nonce,
					max_fee_per_gas: max_fee,
					max_priority_fee_per_gas: U256::zero(),
					gas_limit: self.gas_limit,
					action: self.action,
					value: self.value,
					input: self.input.to_vec(),
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

impl XcmToEthereum for EthereumXcmTransactionV2 {
	fn into_transaction_v2(&self, nonce: U256, chain_id: u64) -> Option<TransactionV2> {
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
		// Eip-1559
		Some(TransactionV2::EIP1559(EIP1559Transaction {
			chain_id,
			nonce,
			max_fee_per_gas: U256::zero(),
			max_priority_fee_per_gas: U256::zero(),
			gas_limit: self.gas_limit,
			action: self.action,
			value: self.value,
			input: self.input.to_vec(),
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
}

/// The EthereumXcmTracingStatus storage key.
pub const ETHEREUM_XCM_TRACING_STORAGE_KEY: &[u8] = b":ethereum_xcm_tracing";

/// The current EthereumXcmTransaction trace status.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub enum EthereumXcmTracingStatus {
	/// A full block trace.
	Block,
	/// A single transaction.
	Transaction(H256),
	/// Exit signal.
	TransactionExited,
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_into_ethereum_tx_with_auto_fee_v1() {
		let xcm_transaction = EthereumXcmTransactionV1 {
			gas_limit: U256::one(),
			fee_payment: EthereumXcmFee::Auto,
			action: TransactionAction::Call(H160::default()),
			value: U256::zero(),
			input: BoundedVec::<u8, ConstU32<MAX_ETHEREUM_XCM_INPUT_SIZE>>::try_from(vec![1u8])
				.unwrap(),
			access_list: None,
		};
		let nonce = U256::zero();
		let expected_tx = Some(TransactionV2::EIP1559(EIP1559Transaction {
			chain_id: 111,
			nonce,
			max_fee_per_gas: U256::zero(),
			max_priority_fee_per_gas: U256::zero(),
			gas_limit: U256::one(),
			action: TransactionAction::Call(H160::default()),
			value: U256::zero(),
			input: vec![1u8],
			access_list: vec![],
			odd_y_parity: true,
			r: H256::from_low_u64_be(1u64),
			s: H256::from_low_u64_be(1u64),
		}));

		assert_eq!(xcm_transaction.into_transaction_v2(nonce, 111), expected_tx);
	}

	#[test]
	fn test_legacy_v1() {
		let xcm_transaction = EthereumXcmTransactionV1 {
			gas_limit: U256::one(),
			fee_payment: EthereumXcmFee::Manual(ManualEthereumXcmFee {
				gas_price: Some(U256::zero()),
				max_fee_per_gas: None,
			}),
			action: TransactionAction::Call(H160::default()),
			value: U256::zero(),
			input: BoundedVec::<u8, ConstU32<MAX_ETHEREUM_XCM_INPUT_SIZE>>::try_from(vec![1u8])
				.unwrap(),
			access_list: None,
		};
		let nonce = U256::zero();
		let expected_tx = Some(TransactionV2::Legacy(LegacyTransaction {
			nonce,
			gas_price: U256::zero(),
			gas_limit: U256::one(),
			action: TransactionAction::Call(H160::default()),
			value: U256::zero(),
			input: vec![1u8],
			signature: TransactionSignature::new(42, rs_id(), rs_id()).unwrap(),
		}));

		assert_eq!(xcm_transaction.into_transaction_v2(nonce, 111), expected_tx);
	}
	#[test]
	fn test_eip_2930_v1() {
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
			gas_limit: U256::one(),
			fee_payment: EthereumXcmFee::Manual(ManualEthereumXcmFee {
				gas_price: Some(U256::zero()),
				max_fee_per_gas: None,
			}),
			action: TransactionAction::Call(H160::default()),
			value: U256::zero(),
			input: BoundedVec::<u8, ConstU32<MAX_ETHEREUM_XCM_INPUT_SIZE>>::try_from(vec![1u8])
				.unwrap(),
			access_list: access_list.clone(),
		};

		let nonce = U256::zero();
		let expected_tx = Some(TransactionV2::EIP2930(EIP2930Transaction {
			chain_id: 111,
			nonce,
			gas_price: U256::zero(),
			gas_limit: U256::one(),
			action: TransactionAction::Call(H160::default()),
			value: U256::zero(),
			input: vec![1u8],
			access_list: from_tuple_to_access_list(&access_list.unwrap()),
			odd_y_parity: true,
			r: H256::from_low_u64_be(1u64),
			s: H256::from_low_u64_be(1u64),
		}));

		assert_eq!(xcm_transaction.into_transaction_v2(nonce, 111), expected_tx);
	}

	#[test]
	fn test_eip1559_v2() {
		let xcm_transaction = EthereumXcmTransactionV2 {
			gas_limit: U256::one(),
			action: TransactionAction::Call(H160::default()),
			value: U256::zero(),
			input: BoundedVec::<u8, ConstU32<MAX_ETHEREUM_XCM_INPUT_SIZE>>::try_from(vec![1u8])
				.unwrap(),
			access_list: None,
		};
		let nonce = U256::zero();
		let expected_tx = Some(TransactionV2::EIP1559(EIP1559Transaction {
			chain_id: 111,
			nonce,
			max_fee_per_gas: U256::zero(),
			max_priority_fee_per_gas: U256::zero(),
			gas_limit: U256::one(),
			action: TransactionAction::Call(H160::default()),
			value: U256::zero(),
			input: vec![1u8],
			access_list: vec![],
			odd_y_parity: true,
			r: H256::from_low_u64_be(1u64),
			s: H256::from_low_u64_be(1u64),
		}));

		assert_eq!(xcm_transaction.into_transaction_v2(nonce, 111), expected_tx);
	}
}
