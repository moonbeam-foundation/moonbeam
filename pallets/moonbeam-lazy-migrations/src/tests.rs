// Copyright 2024 Moonbeam foundation
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

//! Unit testing
use {
	crate::{
		mock::{ExtBuilder, LazyMigrations, RuntimeOrigin, Test},
		Error,
	},
	frame_support::assert_noop,
	rlp::RlpStream,
	sp_core::{H160, H256},
	sp_io::hashing::keccak_256,
	sp_runtime::AccountId32,
};

use pallet_evm::AddressMapping;

// Helper function that calculates the contract address
pub fn contract_address(sender: H160, nonce: u64) -> H160 {
	let mut rlp = RlpStream::new_list(2);
	rlp.append(&sender);
	rlp.append(&nonce);

	H160::from_slice(&keccak_256(&rlp.out())[12..])
}

fn address_build(seed: u8) -> H160 {
	let address = H160::from(H256::from(keccak_256(&[seed; 32])));
	address
}

// Helper function that creates a `num_entries` storage entries for a contract
fn mock_contract_with_entries(seed: u8, nonce: u64, num_entries: u32) -> H160 {
	let address = address_build(seed);

	let contract_address = contract_address(address, nonce);
	let account_id =
		<Test as pallet_evm::Config>::AddressMapping::into_account_id(contract_address);
	let _ = frame_system::Pallet::<Test>::inc_sufficients(&account_id);

	// Add num_entries storage entries to the suicided contract
	for i in 0..num_entries {
		pallet_evm::AccountStorages::<Test>::insert(
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

		// No addresses have been migrated yet
		assert_eq!(crate::pallet::SuicidedContractsRemoved::<Test>::get(), 0);

		// The account has some storage entries
		assert_eq!(
			pallet_evm::AccountStorages::<Test>::iter_prefix(contract_address).count(),
			10
		);

		// Call the extrinsic to delete the storage entries
		let _ = LazyMigrations::clear_suicided_storage(
			RuntimeOrigin::signed(AccountId32::from([45; 32])),
			vec![contract_address].try_into().unwrap(),
			1000,
		);

		// One address has been migrated
		assert_eq!(crate::pallet::SuicidedContractsRemoved::<Test>::get(), 1);
		// All the account storage should have been removed
		assert_eq!(
			pallet_evm::AccountStorages::<Test>::iter_prefix(contract_address).count(),
			0
		);
	})
}

// Test that the extrinsic fails if the contract is not suicided
#[test]
fn test_clear_suicided_contract_failed() {
	ExtBuilder::default().build().execute_with(|| {
		let contract1_address = mock_contract_with_entries(1, 1, 10);
		let contract2_address = mock_contract_with_entries(2, 1, 10);

		// The contracts have not been self-destructed.
		pallet_evm::AccountCodes::<Test>::insert(contract1_address, vec![1, 2, 3]);
		pallet_evm::Suicided::<Test>::insert(contract2_address, ());

		assert_noop!(
			LazyMigrations::clear_suicided_storage(
				RuntimeOrigin::signed(AccountId32::from([45; 32])),
				vec![contract1_address].try_into().unwrap(),
				1000
			),
			Error::<Test>::ContractNotCorrupted
		);

		assert_noop!(
			LazyMigrations::clear_suicided_storage(
				RuntimeOrigin::signed(AccountId32::from([45; 32])),
				vec![contract2_address].try_into().unwrap(),
				1000
			),
			Error::<Test>::ContractNotCorrupted
		);

		// Check that no storage has been removed

		assert_eq!(
			pallet_evm::AccountStorages::<Test>::iter_prefix(contract1_address).count(),
			10
		);
		assert_eq!(
			pallet_evm::AccountStorages::<Test>::iter_prefix(contract2_address).count(),
			10
		);
	})
}

// Test that the extrinsic can handle an empty input
#[test]
fn test_clear_suicided_empty_input() {
	ExtBuilder::default().build().execute_with(|| {
		let contract_address = mock_contract_with_entries(1, 1, 10);

		let _ = LazyMigrations::clear_suicided_storage(
			RuntimeOrigin::signed(AccountId32::from([45; 32])),
			vec![].try_into().unwrap(),
			1000,
		);

		assert_eq!(
			pallet_evm::AccountStorages::<Test>::iter_prefix(contract_address).count(),
			10
		);
	})
}

// Test with multiple deleted contracts ensuring that the extrinsic can handle
// multiple addresses at once.
#[test]
fn test_clear_suicided_contract_multiple_addresses() {
	ExtBuilder::default().build().execute_with(|| {
		let contract_address1 = mock_contract_with_entries(1, 1, 10);
		let contract_address2 = mock_contract_with_entries(2, 1, 20);
		let contract_address3 = mock_contract_with_entries(3, 1, 30);

		// Call the extrinsic to delete the storage entries
		let _ = LazyMigrations::clear_suicided_storage(
			RuntimeOrigin::signed(AccountId32::from([45; 32])),
			vec![contract_address1, contract_address2, contract_address3]
				.try_into()
				.unwrap(),
			1000,
		)
		.unwrap();

		assert_eq!(
			pallet_evm::AccountStorages::<Test>::iter_prefix(contract_address1).count(),
			0
		);
		assert_eq!(
			pallet_evm::AccountStorages::<Test>::iter_prefix(contract_address2).count(),
			0
		);
		assert_eq!(
			pallet_evm::AccountStorages::<Test>::iter_prefix(contract_address3).count(),
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

		let _ = LazyMigrations::clear_suicided_storage(
			RuntimeOrigin::signed(AccountId32::from([45; 32])),
			vec![contract_address1, contract_address2]
				.try_into()
				.unwrap(),
			1000,
		)
		.unwrap();
		assert_eq!(
			pallet_evm::AccountStorages::<Test>::iter_prefix(contract_address1).count(),
			1000
		);

		assert_eq!(
			pallet_evm::AccountStorages::<Test>::iter_prefix(contract_address2).count(),
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
		pallet_evm::AccountCodes::<Test>::insert(contract_address3, vec![1, 2, 3]);

		assert_noop!(
			LazyMigrations::clear_suicided_storage(
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
			Error::<Test>::ContractNotCorrupted
		);

		assert_eq!(
			pallet_evm::AccountStorages::<Test>::iter_prefix(contract_address1).count(),
			10
		);
		assert_eq!(
			pallet_evm::AccountStorages::<Test>::iter_prefix(contract_address2).count(),
			10
		);
		assert_eq!(
			pallet_evm::AccountStorages::<Test>::iter_prefix(contract_address3).count(),
			10
		);
		assert_eq!(
			pallet_evm::AccountStorages::<Test>::iter_prefix(contract_address4).count(),
			10
		);
	})
}
