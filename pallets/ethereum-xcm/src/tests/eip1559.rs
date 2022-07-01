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

use super::*;
use frame_support::{
	assert_noop,
	weights::{Pays, PostDispatchInfo},
};
use sp_runtime::{DispatchError, DispatchErrorWithPostInfo};
use xcm_primitives::{EthereumXcmFee, EthereumXcmTransaction, EthereumXcmTransactionV1};

// 	pragma solidity ^0.6.6;
// 	contract Test {
// 		function foo() external pure returns (bool) {
// 			return true;
// 		}
// 		function bar() external pure {
// 			require(false, "error_msg");
// 		}
// 	}
const CONTRACT: &str = "608060405234801561001057600080fd5b50610113806100206000396000f3fe6080604052\
						348015600f57600080fd5b506004361060325760003560e01c8063c2985578146037578063\
						febb0f7e146057575b600080fd5b603d605f565b6040518082151515158152602001915050\
						60405180910390f35b605d6068565b005b60006001905090565b600060db576040517f08c3\
						79a00000000000000000000000000000000000000000000000000000000081526004018080\
						602001828103825260098152602001807f6572726f725f6d73670000000000000000000000\
						00000000000000000000000081525060200191505060405180910390fd5b56fea264697066\
						7358221220fde68a3968e0e99b16fabf9b2997a78218b32214031f8e07e2c502daf603a69e\
						64736f6c63430006060033";

fn xcm_evm_transfer_eip_1559_transaction(destination: H160, value: U256) -> EthereumXcmTransaction {
	EthereumXcmTransaction::V1(EthereumXcmTransactionV1 {
		fee_payment: EthereumXcmFee::Auto,
		gas_limit: U256::from(0x100000),
		action: ethereum::TransactionAction::Call(destination),
		value,
		input: vec![],
		access_list: None,
	})
}

fn xcm_evm_call_eip_1559_transaction(destination: H160, input: Vec<u8>) -> EthereumXcmTransaction {
	EthereumXcmTransaction::V1(EthereumXcmTransactionV1 {
		fee_payment: EthereumXcmFee::Auto,
		gas_limit: U256::from(0x100000),
		action: ethereum::TransactionAction::Call(destination),
		value: U256::zero(),
		input,
		access_list: None,
	})
}

fn xcm_erc20_creation_eip_1559_transaction() -> EthereumXcmTransaction {
	EthereumXcmTransaction::V1(EthereumXcmTransactionV1 {
		fee_payment: EthereumXcmFee::Auto,

		gas_limit: U256::from(0x100000),
		action: ethereum::TransactionAction::Create,
		value: U256::zero(),
		input: hex::decode(ERC20_CONTRACT_BYTECODE.trim_end()).unwrap(),
		access_list: None,
	})
}

#[test]
fn test_transact_xcm_evm_transfer() {
	let (pairs, mut ext) = new_test_ext(2);
	let alice = &pairs[0];
	let bob = &pairs[1];

	ext.execute_with(|| {
		let balances_before = System::account(&bob.account_id);
		EthereumXcm::transact(
			RawOrigin::XcmEthereumTransaction(alice.address).into(),
			xcm_evm_transfer_eip_1559_transaction(bob.address, U256::from(100)),
		)
		.expect("Failed to execute transaction");

		assert_eq!(
			System::account(&bob.account_id).data.free,
			balances_before.data.free + 100
		);
	});
}

#[test]
fn test_transact_xcm_create() {
	let (pairs, mut ext) = new_test_ext(1);
	let alice = &pairs[0];

	ext.execute_with(|| {
		assert_noop!(
			EthereumXcm::transact(
				RawOrigin::XcmEthereumTransaction(alice.address).into(),
				xcm_erc20_creation_eip_1559_transaction()
			),
			DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(0),
					pays_fee: Pays::Yes,
				},
				error: DispatchError::Other("Cannot convert xcm payload to known type"),
			}
		);
	});
}

#[test]
fn test_transact_xcm_evm_call_works() {
	let (pairs, mut ext) = new_test_ext(2);
	let alice = &pairs[0];
	let bob = &pairs[1];

	ext.execute_with(|| {
		let t = EIP1559UnsignedTransaction {
			nonce: U256::zero(),
			max_priority_fee_per_gas: U256::from(1),
			max_fee_per_gas: U256::from(1),
			gas_limit: U256::from(0x100000),
			action: ethereum::TransactionAction::Create,
			value: U256::zero(),
			input: hex::decode(CONTRACT).unwrap(),
		}
		.sign(&alice.private_key, None);
		assert_ok!(Ethereum::execute(alice.address, &t, None,));

		let contract_address = hex::decode("32dcab0ef3fb2de2fce1d2e0799d36239671f04a").unwrap();
		let foo = hex::decode("c2985578").unwrap();
		let bar = hex::decode("febb0f7e").unwrap();

		let _ = EthereumXcm::transact(
			RawOrigin::XcmEthereumTransaction(bob.address).into(),
			xcm_evm_call_eip_1559_transaction(H160::from_slice(&contract_address), foo),
		)
		.expect("Failed to call `foo`");

		// Evm call failing still succesfully dispatched
		let _ = EthereumXcm::transact(
			RawOrigin::XcmEthereumTransaction(bob.address).into(),
			xcm_evm_call_eip_1559_transaction(H160::from_slice(&contract_address), bar),
		)
		.expect("Failed to call `bar`");

		let pending = Ethereum::pending();
		assert!(pending.len() == 2);

		// Transaction is in Pending storage, with nonce 0 and status 1 (evm succeed).
		let (transaction_0, _, receipt_0) = &pending[0];
		match (transaction_0, receipt_0) {
			(&crate::Transaction::EIP1559(ref t), &crate::Receipt::EIP1559(ref r)) => {
				assert!(t.nonce == U256::from(0u8));
				assert!(r.status_code == 1u8);
			}
			_ => unreachable!(),
		}

		// Transaction is in Pending storage, with nonce 1 and status 0 (evm failed).
		let (transaction_1, _, receipt_1) = &pending[1];
		match (transaction_1, receipt_1) {
			(&crate::Transaction::EIP1559(ref t), &crate::Receipt::EIP1559(ref r)) => {
				assert!(t.nonce == U256::from(1u8));
				assert!(r.status_code == 0u8);
			}
			_ => unreachable!(),
		}
	});
}

#[test]
fn test_transact_xcm_validation_works() {
	let (pairs, mut ext) = new_test_ext(2);
	let alice = &pairs[0];
	let bob = &pairs[1];

	ext.execute_with(|| {
		// Not enough balance fails to validate.
		assert_noop!(
			EthereumXcm::transact(
				RawOrigin::XcmEthereumTransaction(alice.address).into(),
				xcm_evm_transfer_eip_1559_transaction(bob.address, U256::MAX),
			),
			DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(0),
					pays_fee: Pays::Yes,
				},
				error: DispatchError::Other("Failed to validate ethereum transaction"),
			}
		);
		// Not enough base fee fails to validate.
		assert_noop!(
			EthereumXcm::transact(
				RawOrigin::XcmEthereumTransaction(alice.address).into(),
				EthereumXcmTransaction::V1(EthereumXcmTransactionV1 {
					fee_payment: EthereumXcmFee::Manual(xcm_primitives::ManualEthereumXcmFee {
						gas_price: Some(U256::from(0)),
						max_fee_per_gas: None,
					}),
					gas_limit: U256::from(0x100000),
					action: ethereum::TransactionAction::Call(bob.address),
					value: U256::from(1),
					input: vec![],
					access_list: None,
				}),
			),
			DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(0),
					pays_fee: Pays::Yes,
				},
				error: DispatchError::Other("Failed to validate ethereum transaction"),
			}
		);
	});
}
