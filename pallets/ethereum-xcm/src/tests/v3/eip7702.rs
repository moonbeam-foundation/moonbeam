// Copyright 2019-2025 PureStake Inc.
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
use crate::{mock::*, Error, RawOrigin};
use ethereum_types::{H160, U256};
use frame_support::{
	assert_noop, assert_ok,
	dispatch::{Pays, PostDispatchInfo},
	traits::{ConstU32, Get},
	weights::Weight,
	BoundedVec,
};
use sp_runtime::{DispatchError, DispatchErrorWithPostInfo};
use xcm_primitives::{EthereumXcmTransaction, EthereumXcmTransactionV3};

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

fn xcm_evm_transfer_eip_7702_transaction(destination: H160, value: U256) -> EthereumXcmTransaction {
	EthereumXcmTransaction::V3(EthereumXcmTransactionV3 {
		gas_limit: U256::from(0x5208),
		action: ethereum::TransactionAction::Call(destination),
		value,
		input:
			BoundedVec::<u8, ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>>::try_from(
				vec![],
			)
			.unwrap(),
		access_list: None,
		authorization_list: None,
	})
}

fn xcm_evm_call_eip_7702_transaction(destination: H160, input: Vec<u8>) -> EthereumXcmTransaction {
	EthereumXcmTransaction::V3(EthereumXcmTransactionV3 {
		gas_limit: U256::from(0x100000),
		action: ethereum::TransactionAction::Call(destination),
		value: U256::zero(),
		input:
			BoundedVec::<u8, ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>>::try_from(
				input,
			)
			.unwrap(),
		access_list: None,
		authorization_list: None,
	})
}

fn xcm_erc20_creation_eip_7702_transaction() -> EthereumXcmTransaction {
	EthereumXcmTransaction::V3(EthereumXcmTransactionV3 {
		gas_limit: U256::from(0x100000),
		action: ethereum::TransactionAction::Create,
		value: U256::zero(),
		input:
			BoundedVec::<u8, ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>>::try_from(
				hex::decode(CONTRACT).unwrap(),
			)
			.unwrap(),
		access_list: None,
		authorization_list: None,
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
			xcm_evm_transfer_eip_7702_transaction(bob.address, U256::from(100)),
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
				xcm_erc20_creation_eip_7702_transaction()
			),
			DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(Weight::zero()),
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
		let t = EIP7702UnsignedTransaction {
			nonce: U256::zero(),
			max_priority_fee_per_gas: U256::one(),
			max_fee_per_gas: U256::one(),
			gas_limit: U256::from(0x100000),
			destination: ethereum::TransactionAction::Create,
			value: U256::zero(),
			data: hex::decode(CONTRACT).unwrap(),
		}
		.sign(&alice.private_key, None, None);
		assert_ok!(Ethereum::execute(alice.address, &t, None, None));

		let contract_address = hex::decode("32dcab0ef3fb2de2fce1d2e0799d36239671f04a").unwrap();
		let foo = hex::decode("c2985578").unwrap();
		let bar = hex::decode("febb0f7e").unwrap();

		let _ = EthereumXcm::transact(
			RawOrigin::XcmEthereumTransaction(bob.address).into(),
			xcm_evm_call_eip_7702_transaction(H160::from_slice(&contract_address), foo),
		)
		.expect("Failed to call `foo`");

		// Evm call failing still succesfully dispatched
		let _ = EthereumXcm::transact(
			RawOrigin::XcmEthereumTransaction(bob.address).into(),
			xcm_evm_call_eip_7702_transaction(H160::from_slice(&contract_address), bar),
		)
		.expect("Failed to call `bar`");

		assert!(pallet_ethereum::Pending::<Test>::count() == 2);

		// Transaction is in Pending storage, with nonce 0 and status 1 (evm succeed).
		let (transaction_0, _, receipt_0) = &pallet_ethereum::Pending::<Test>::get(0).unwrap();
		match (transaction_0, receipt_0) {
			(&crate::Transaction::EIP7702(ref t), &crate::Receipt::EIP7702(ref r)) => {
				assert!(t.nonce == U256::from(0u8));
				assert!(r.status_code == 1u8);
			}
			_ => unreachable!(),
		}

		// Transaction is in Pending storage, with nonce 1 and status 0 (evm failed).
		let (transaction_1, _, receipt_1) = &pallet_ethereum::Pending::<Test>::get(1).unwrap();
		match (transaction_1, receipt_1) {
			(&crate::Transaction::EIP7702(ref t), &crate::Receipt::EIP7702(ref r)) => {
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
		// Not enough gas limit to cover the transaction cost.
		assert_noop!(
			EthereumXcm::transact(
				RawOrigin::XcmEthereumTransaction(alice.address).into(),
				EthereumXcmTransaction::V3(EthereumXcmTransactionV3 {
					gas_limit: U256::from(0x5207),
					action: ethereum::TransactionAction::Call(bob.address),
					value: U256::one(),
					input: BoundedVec::<
						u8,
						ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>,
					>::try_from(vec![])
					.unwrap(),
					access_list: None,
					authorization_list: None,
				}),
			),
			DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: Some(Weight::zero()),
					pays_fee: Pays::Yes,
				},
				error: DispatchError::Other("Failed to validate ethereum transaction"),
			}
		);
	});
}

#[test]
fn test_ensure_transact_xcm_trough_no_proxy_error() {
	let (pairs, mut ext) = new_test_ext(2);
	let alice = &pairs[0];
	let bob = &pairs[1];

	ext.execute_with(|| {
		let r = EthereumXcm::transact_through_proxy(
			RawOrigin::XcmEthereumTransaction(alice.address).into(),
			bob.address,
			xcm_evm_transfer_eip_7702_transaction(bob.address, U256::from(100)),
		);
		assert!(r.is_err());
		assert_eq!(
			r.unwrap_err().error,
			sp_runtime::DispatchError::Other("proxy error: expected `ProxyType::Any`"),
		);
	});
}

#[test]
fn test_ensure_transact_xcm_trough_proxy_error() {
	let (pairs, mut ext) = new_test_ext(2);
	let alice = &pairs[0];
	let bob = &pairs[1];

	ext.execute_with(|| {
		let _ = Proxy::add_proxy_delegate(
			&bob.account_id,
			alice.account_id.clone(),
			ProxyType::NotAllowed,
			0,
		);
		let r = EthereumXcm::transact_through_proxy(
			RawOrigin::XcmEthereumTransaction(alice.address).into(),
			bob.address,
			xcm_evm_transfer_eip_7702_transaction(bob.address, U256::from(100)),
		);
		assert!(r.is_err());
		assert_eq!(
			r.unwrap_err().error,
			sp_runtime::DispatchError::Other("proxy error: expected `ProxyType::Any`"),
		);
	});
}

#[test]
fn test_ensure_transact_xcm_trough_proxy_ok() {
	let (pairs, mut ext) = new_test_ext(3);
	let alice = &pairs[0];
	let bob = &pairs[1];
	let charlie = &pairs[2];

	let allowed_proxies = vec![ProxyType::Any];

	for proxy in allowed_proxies.into_iter() {
		ext.execute_with(|| {
			let _ = Proxy::add_proxy_delegate(&bob.account_id, alice.account_id.clone(), proxy, 0);
			let alice_before = System::account(&alice.account_id);
			let bob_before = System::account(&bob.account_id);
			let charlie_before = System::account(&charlie.account_id);

			let r = EthereumXcm::transact_through_proxy(
				RawOrigin::XcmEthereumTransaction(alice.address).into(),
				bob.address,
				xcm_evm_transfer_eip_7702_transaction(charlie.address, U256::from(100)),
			);
			// Transact succeeded
			assert!(r.is_ok());

			let alice_after = System::account(&alice.account_id);
			let bob_after = System::account(&bob.account_id);
			let charlie_after = System::account(&charlie.account_id);

			// Alice remains unchanged
			assert_eq!(alice_before, alice_after);

			// Bob nonce was increased
			assert_eq!(bob_after.nonce, bob_before.nonce + 1);

			// Bob sent some funds without paying any fees
			assert_eq!(bob_after.data.free, bob_before.data.free - 100);

			// Charlie receive some funds
			assert_eq!(charlie_after.data.free, charlie_before.data.free + 100);

			// Clear proxy
			let _ =
				Proxy::remove_proxy_delegate(&bob.account_id, alice.account_id.clone(), proxy, 0);
		});
	}
}

#[test]
fn test_global_nonce_incr() {
	let (pairs, mut ext) = new_test_ext(3);
	let alice = &pairs[0];
	let bob = &pairs[1];
	let charlie = &pairs[2];

	ext.execute_with(|| {
		assert_eq!(EthereumXcm::nonce(), U256::zero());

		EthereumXcm::transact(
			RawOrigin::XcmEthereumTransaction(alice.address).into(),
			xcm_evm_transfer_eip_7702_transaction(charlie.address, U256::one()),
		)
		.expect("Failed to execute transaction from Alice to Charlie");

		assert_eq!(EthereumXcm::nonce(), U256::one());

		EthereumXcm::transact(
			RawOrigin::XcmEthereumTransaction(bob.address).into(),
			xcm_evm_transfer_eip_7702_transaction(charlie.address, U256::one()),
		)
		.expect("Failed to execute transaction from Bob to Charlie");

		assert_eq!(EthereumXcm::nonce(), U256::from(2));
	});
}

#[test]
fn test_global_nonce_not_incr() {
	let (pairs, mut ext) = new_test_ext(2);
	let alice = &pairs[0];
	let bob = &pairs[1];

	ext.execute_with(|| {
		assert_eq!(EthereumXcm::nonce(), U256::zero());

		let invalid_transaction_cost =
			EthereumXcmTransaction::V3(
				EthereumXcmTransactionV3 {
					gas_limit: U256::one(),
					action: ethereum::TransactionAction::Call(bob.address),
					value: U256::one(),
					input: BoundedVec::<
						u8,
						ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>,
					>::try_from(vec![])
					.unwrap(),
					access_list: None,
					authorization_list: None,
				},
			);

		EthereumXcm::transact(
			RawOrigin::XcmEthereumTransaction(alice.address).into(),
			invalid_transaction_cost,
		)
		.expect_err("Failed to execute transaction from Alice to Bob");

		assert_eq!(EthereumXcm::nonce(), U256::zero());
	});
}

#[test]
fn test_transaction_hash_collision() {
	let (pairs, mut ext) = new_test_ext(3);
	let alice = &pairs[0];
	let bob = &pairs[1];
	let charlie = &pairs[2];

	ext.execute_with(|| {
		EthereumXcm::transact(
			RawOrigin::XcmEthereumTransaction(alice.address).into(),
			xcm_evm_transfer_eip_7702_transaction(charlie.address, U256::one()),
		)
		.expect("Failed to execute transaction from Alice to Charlie");

		EthereumXcm::transact(
			RawOrigin::XcmEthereumTransaction(bob.address).into(),
			xcm_evm_transfer_eip_7702_transaction(charlie.address, U256::one()),
		)
		.expect("Failed to execute transaction from Bob to Charlie");

		let mut hashes = pallet_ethereum::Pending::<Test>::iter_values()
			.map(|(tx, _, _)| tx.hash())
			.collect::<Vec<ethereum_types::H256>>();

		// Holds two transactions hashes
		assert_eq!(hashes.len(), 2);

		hashes.dedup();

		// Still holds two transactions hashes after removing potential consecutive repeated values.
		assert_eq!(hashes.len(), 2);
	});
}

#[test]
fn check_suspend_ethereum_to_xcm_works() {
	let (pairs, mut ext) = new_test_ext(2);
	let alice = &pairs[0];
	let bob = &pairs[1];

	let db_weights: frame_support::weights::RuntimeDbWeight =
		<Test as frame_system::Config>::DbWeight::get();

	ext.execute_with(|| {
		assert_ok!(EthereumXcm::suspend_ethereum_xcm_execution(
			RuntimeOrigin::root(),
		));
		assert_noop!(
			EthereumXcm::transact(
				RawOrigin::XcmEthereumTransaction(alice.address).into(),
				xcm_evm_transfer_eip_7702_transaction(bob.address, U256::from(100)),
			),
			DispatchErrorWithPostInfo {
				error: Error::<Test>::EthereumXcmExecutionSuspended.into(),
				post_info: PostDispatchInfo {
					actual_weight: Some(db_weights.reads(1)),
					pays_fee: Pays::Yes
				}
			}
		);

		assert_noop!(
			EthereumXcm::transact_through_proxy(
				RawOrigin::XcmEthereumTransaction(alice.address).into(),
				bob.address,
				xcm_evm_transfer_eip_7702_transaction(bob.address, U256::from(100)),
			),
			DispatchErrorWithPostInfo {
				error: Error::<Test>::EthereumXcmExecutionSuspended.into(),
				post_info: PostDispatchInfo {
					actual_weight: Some(db_weights.reads(1)),
					pays_fee: Pays::Yes
				}
			}
		);
	});
}

#[test]
fn transact_after_resume_ethereum_to_xcm_works() {
	let (pairs, mut ext) = new_test_ext(2);
	let alice = &pairs[0];
	let bob = &pairs[1];

	ext.execute_with(|| {
		let bob_before = System::account(&bob.account_id);

		assert_ok!(EthereumXcm::suspend_ethereum_xcm_execution(
			RuntimeOrigin::root()
		));

		assert_ok!(EthereumXcm::resume_ethereum_xcm_execution(
			RuntimeOrigin::root()
		));
		assert_ok!(EthereumXcm::transact(
			RawOrigin::XcmEthereumTransaction(alice.address).into(),
			xcm_evm_transfer_eip_7702_transaction(bob.address, U256::from(100)),
		));
		let bob_after = System::account(&bob.account_id);

		// Bob sent some funds without paying any fees
		assert_eq!(bob_after.data.free, bob_before.data.free + 100);
	});
}

#[test]
fn transact_through_proxy_after_resume_ethereum_to_xcm_works() {
	let (pairs, mut ext) = new_test_ext(3);
	let alice = &pairs[0];
	let bob = &pairs[1];
	let charlie = &pairs[2];

	ext.execute_with(|| {
		let _ =
			Proxy::add_proxy_delegate(&bob.account_id, alice.account_id.clone(), ProxyType::Any, 0);
		let alice_before = System::account(&alice.account_id);
		let bob_before = System::account(&bob.account_id);
		let charlie_before = System::account(&charlie.account_id);

		assert_ok!(EthereumXcm::suspend_ethereum_xcm_execution(
			RuntimeOrigin::root()
		));

		assert_ok!(EthereumXcm::resume_ethereum_xcm_execution(
			RuntimeOrigin::root()
		));
		assert_ok!(EthereumXcm::transact_through_proxy(
			RawOrigin::XcmEthereumTransaction(alice.address).into(),
			bob.address,
			xcm_evm_transfer_eip_7702_transaction(charlie.address, U256::from(100)),
		));

		let alice_after = System::account(&alice.account_id);
		let bob_after = System::account(&bob.account_id);
		let charlie_after = System::account(&charlie.account_id);

		// Alice remains unchanged
		assert_eq!(alice_before, alice_after);

		// Bob nonce was increased
		assert_eq!(bob_after.nonce, bob_before.nonce + 1);

		// Bob sent some funds without paying any fees
		assert_eq!(bob_after.data.free, bob_before.data.free - 100);

		// Charlie receive some funds
		assert_eq!(charlie_after.data.free, charlie_before.data.free + 100);
	});
}
