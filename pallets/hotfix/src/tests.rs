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

use crate::*;
use frame_support::assert_noop;
use mock::*;
use rlp::RlpStream;
use sp_core::{keccak_256, H160, H256};
use sp_runtime::AccountId32;

// Helper function that calculates the contract address
pub fn contract_address(sender: H160, nonce: u64) -> H160 {
	let mut rlp = RlpStream::new_list(2);
	rlp.append(&sender);
	rlp.append(&nonce);

	H160::from_slice(&keccak_256(&rlp.out())[12..])
}

fn address_build(seed: u8) -> H160 {
	let private_key = H256::from_slice(&[(seed + 1) as u8; 32]);
	let secret_key = libsecp256k1::SecretKey::parse_slice(&private_key[..]).unwrap();
	let public_key = &libsecp256k1::PublicKey::from_secret_key(&secret_key).serialize()[1..65];
	let address = H160::from(H256::from(keccak_256(public_key)));

	let mut data = [0u8; 32];
	data[0..20].copy_from_slice(&address[..]);

	address
}

// Helper function that creates a `num_entries` storage entries for a contract
fn mock_contract_with_entries(seed: u8, nonce: u64, num_entries: u32) -> H160 {
	let address = address_build(seed);

	let contract_address = contract_address(address, nonce);
	let account_id =
		<Runtime as pallet_evm::Config>::AddressMapping::into_account_id(contract_address);
	let _ = frame_system::Pallet::<Runtime>::inc_sufficients(&account_id);

	// Add num_entries storage entries to the suicided contract
	for i in 0..num_entries {
		pallet_evm::AccountStorages::<Runtime>::insert(
			contract_address,
			H256::from_low_u64_be(i as u64),
			H256::from_low_u64_be(i as u64),
		);
	}

	contract_address
}

#[test]
fn test_clear_suicided_contract_succesfull() {
	ExtBuilder::default().build().execute_with(|| {
		let contract_address = mock_contract_with_entries(1, 1, 10);

		// Call the extrinsic to delete the storage entries
		let _ = Hotfix::clear_suicided_storage(
			RuntimeOrigin::signed(AccountId32::from([45; 32])),
			vec![contract_address].try_into().unwrap(),
			1000,
		);

		assert_eq!(
			pallet_evm::AccountStorages::<Runtime>::iter_prefix(contract_address).count(),
			0
		);
	})
}

// Test that the extrinsic fails if the contract is not suicided
#[test]
fn test_clear_suicided_contract_failed() {
	ExtBuilder::default().build().execute_with(|| {
		let contract_address = mock_contract_with_entries(1, 1, 10);
		// Contract has not been self-destructed.
		pallet_evm::AccountCodes::<Runtime>::insert(contract_address, vec![1, 2, 3]);

		assert_noop!(
			Hotfix::clear_suicided_storage(
				RuntimeOrigin::signed(AccountId32::from([45; 32])),
				vec![contract_address].try_into().unwrap(),
				1000
			),
			Error::<Runtime>::ContractNotSuicided
		);

		assert_eq!(
			pallet_evm::AccountStorages::<Runtime>::iter_prefix(contract_address).count(),
			10
		);
	})
}

// Test that the extrinsic can handle an empty input
#[test]
fn test_clear_suicided_empty_input() {
	ExtBuilder::default().build().execute_with(|| {
		let contract_address = mock_contract_with_entries(1, 1, 10);

		let _ = Hotfix::clear_suicided_storage(
			RuntimeOrigin::signed(AccountId32::from([45; 32])),
			vec![].try_into().unwrap(),
			1000,
		);

		assert_eq!(
			pallet_evm::AccountStorages::<Runtime>::iter_prefix(contract_address).count(),
			10
		);
	})
}

// Test with multiple deleted contracts ensuring that the extrinsic can handle multiple addresses at once.
#[test]
fn test_clear_suicided_contract_multiple_addresses() {
	ExtBuilder::default().build().execute_with(|| {
		let contract_address1 = mock_contract_with_entries(1, 1, 10);
		let contract_address2 = mock_contract_with_entries(2, 1, 20);
		let contract_address3 = mock_contract_with_entries(3, 1, 30);

		// Call the extrinsic to delete the storage entries
		let _ = Hotfix::clear_suicided_storage(
			RuntimeOrigin::signed(AccountId32::from([45; 32])),
			vec![contract_address1, contract_address2, contract_address3]
				.try_into()
				.unwrap(),
			1000,
		)
		.unwrap();

		assert_eq!(
			pallet_evm::AccountStorages::<Runtime>::iter_prefix(contract_address1).count(),
			0
		);
		assert_eq!(
			pallet_evm::AccountStorages::<Runtime>::iter_prefix(contract_address2).count(),
			0
		);
		assert_eq!(
			pallet_evm::AccountStorages::<Runtime>::iter_prefix(contract_address3).count(),
			0
		);
	})
}

// Test that the limit of entries to be deleted is respected
#[test]
fn test_clear_suicided_entry_limit() {
	ExtBuilder::default().build().execute_with(|| {
		let contract_address1 = mock_contract_with_entries(1, 1, 2000);
		let contract_address2 = mock_contract_with_entries(2, 1, 1);

		let _ = Hotfix::clear_suicided_storage(
			RuntimeOrigin::signed(AccountId32::from([45; 32])),
			vec![contract_address1, contract_address2]
				.try_into()
				.unwrap(),
			1000,
		)
		.unwrap();
		assert_eq!(
			pallet_evm::AccountStorages::<Runtime>::iter_prefix(contract_address1).count(),
			1000
		);

		assert_eq!(
			pallet_evm::AccountStorages::<Runtime>::iter_prefix(contract_address2).count(),
			1
		);
	})
}

// Test a combination of Suicided and non-suicided contracts
#[test]
fn test_clear_suicided_mixed_suicided_and_non_suicided() {
	ExtBuilder::default().build().execute_with(|| {
		let contract_address1 = mock_contract_with_entries(1, 1, 10);
		let contract_address2 = mock_contract_with_entries(2, 1, 10);
		let contract_address3 = mock_contract_with_entries(3, 1, 10);
		let contract_address4 = mock_contract_with_entries(4, 1, 10);

		// Contract has not been self-destructed.
		pallet_evm::AccountCodes::<Runtime>::insert(contract_address3, vec![1, 2, 3]);

		assert_noop!(
			Hotfix::clear_suicided_storage(
				RuntimeOrigin::signed(AccountId32::from([45; 32])),
				vec![
					contract_address1,
					contract_address2,
					contract_address3,
					contract_address4
				]
				.try_into()
				.unwrap(),
				1000
			),
			Error::<Runtime>::ContractNotSuicided
		);

		assert_eq!(
			pallet_evm::AccountStorages::<Runtime>::iter_prefix(contract_address1).count(),
			10
		);
		assert_eq!(
			pallet_evm::AccountStorages::<Runtime>::iter_prefix(contract_address2).count(),
			10
		);
		assert_eq!(
			pallet_evm::AccountStorages::<Runtime>::iter_prefix(contract_address3).count(),
			10
		);
		assert_eq!(
			pallet_evm::AccountStorages::<Runtime>::iter_prefix(contract_address4).count(),
			10
		);
	})
}
