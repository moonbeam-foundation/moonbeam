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
		Error, ReadWriteOps, StateMigrationStatus, StateMigrationStatusValue, MAX_ITEM_PROOF_SIZE,
		PROOF_SIZE_BUFFER,
	},
	frame_support::{assert_noop, fail, traits::Hooks, weights::Weight},
	rlp::RlpStream,
	sp_core::{hexdisplay, H160, H256},
	sp_io::hashing::keccak_256,
	sp_runtime::{print, traits::Bounded, AccountId32},
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

fn count_keys_and_data_without_code() -> (u64, u64) {
	let mut keys: u64 = 0;
	let mut data: u64 = 0;

	let mut current_key: Option<Vec<u8>> = Some(Default::default());
	while let Some(key) = current_key {
		if key.as_slice() == sp_core::storage::well_known_keys::CODE {
			current_key = sp_io::storage::next_key(&key);
			continue;
		}
		print!("Key: {} ", hexdisplay::ascii_format(&key));
		keys += 1;
		if let Some(_) = sp_io::storage::get(&key) {
			print!("HAS DATA");
			data += 1;
		}
		println!();
		current_key = sp_io::storage::next_key(&key);
	}

	(keys, data)
}

fn weight_for(read: u64, write: u64) -> Weight {
	<Test as frame_system::Config>::DbWeight::get().reads_writes(read, write)
}

fn base_line_weight_with(reads: u64, writes: u64) -> Weight {
	Weight::from_parts(0, 0).saturating_add(
		<Test as frame_system::Config>::DbWeight::get().reads_writes(reads + 10, writes + 9),
	)
}

fn rem_weight_for_entries(num_entries: u64) -> Weight {
	let proof = PROOF_SIZE_BUFFER + num_entries * MAX_ITEM_PROOF_SIZE;
	Weight::from_parts(u64::max_value(), proof)
}

#[test]
fn test_state_migration_baseline() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			StateMigrationStatusValue::<Test>::get(),
			StateMigrationStatus::NotStarted
		);

		let (keys, data) = count_keys_and_data_without_code();
		println!("Keys: {}, Data: {}", keys, data);

		let weight = LazyMigrations::on_idle(0, Weight::max_value());

		// READS: 2 * keys + 2 (skipped and status)
		// Next key requests = keys (we have first key as default which is not counted, and extra
		// next_key request to check if we are done)
		//
		// 1 next key request for the skipped key ":code"
		// Read requests = keys (we read each key once)
		// 1 Read request for the StateMigrationStatusValue

		// WRITES: data + 1 (status)
		// Write requests = data (we write each data once)
		// 1 Write request for the StateMigrationStatusValue
		assert_eq!(weight, weight_for(2 * keys + 2, data + 1));

		assert_eq!(
			StateMigrationStatusValue::<Test>::get(),
			StateMigrationStatus::Complete
		);
	})
}

#[test]
fn test_state_migration_cannot_fit_any_item() {
	ExtBuilder::default().build().execute_with(|| {
		StateMigrationStatusValue::<Test>::put(StateMigrationStatus::Complete);

		let weight = LazyMigrations::on_idle(0, rem_weight_for_entries(0));

		assert_eq!(weight, weight_for(0, 0));
	})
}

#[test]
fn test_state_migration_when_complete() {
	ExtBuilder::default().build().execute_with(|| {
		StateMigrationStatusValue::<Test>::put(StateMigrationStatus::Complete);

		let weight = LazyMigrations::on_idle(0, Weight::max_value());

		// just reading the status of the migration
		assert_eq!(weight, weight_for(1, 0));
	})
}

#[test]
fn test_state_migration_when_errored() {
	ExtBuilder::default().build().execute_with(|| {
		StateMigrationStatusValue::<Test>::put(StateMigrationStatus::Error(
			"Error".as_bytes().to_vec().try_into().unwrap_or_default(),
		));

		let weight = LazyMigrations::on_idle(0, Weight::max_value());

		// just reading the status of the migration
		assert_eq!(weight, weight_for(1, 0));
	})
}

#[test]
fn test_state_migration_can_only_fit_one_item() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			StateMigrationStatusValue::<Test>::get(),
			StateMigrationStatus::NotStarted
		);

		let data = sp_io::storage::get(Default::default());
		let weight = LazyMigrations::on_idle(0, rem_weight_for_entries(1));

		let reads = 2; // key read + status read
		let writes = 1 + data.map(|_| 1).unwrap_or(0);
		assert_eq!(weight, weight_for(reads, writes));

		assert!(matches!(
			StateMigrationStatusValue::<Test>::get(),
			StateMigrationStatus::Started(_)
		));

		let weight = LazyMigrations::on_idle(0, rem_weight_for_entries(3));
		let reads = 3 + 3 + 1; // next key + key read + status
		let writes = 1 + 3; // status write + key write
		assert_eq!(weight, weight_for(reads, writes));
	})
}

#[test]
fn test_state_migration_can_only_fit_three_item() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			StateMigrationStatusValue::<Test>::get(),
			StateMigrationStatus::NotStarted
		);

		let weight = LazyMigrations::on_idle(0, rem_weight_for_entries(3));

		// 2 next key requests (default key dons't need a next key request) + 1 status read
		// 3 key reads.
		// 1 status write + 2 key writes (default key doesn't have any data)
		let reads = 6;
		let writes = 3;
		assert_eq!(weight, weight_for(reads, writes));

		assert!(matches!(
			StateMigrationStatusValue::<Test>::get(),
			StateMigrationStatus::Started(_)
		));
	})
}

#[test]
fn test_state_migration_can_fit_exactly_all_item() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			StateMigrationStatusValue::<Test>::get(),
			StateMigrationStatus::NotStarted
		);

		let (keys, data) = count_keys_and_data_without_code();
		let weight = LazyMigrations::on_idle(0, rem_weight_for_entries(keys));

		// we deduct the extra next_key request to check if we are done.
		// will know if we are done on the next call to on_idle
		assert_eq!(weight, weight_for(2 * keys + 1, data + 1));

		assert!(matches!(
			StateMigrationStatusValue::<Test>::get(),
			StateMigrationStatus::Started(_),
		));

		// after calling on_idle status is added to the storage so we need to account for that
		let (new_keys, new_data) = count_keys_and_data_without_code();
		let (diff_keys, diff_data) = (new_keys - keys, new_data - data);

		let weight = LazyMigrations::on_idle(0, rem_weight_for_entries(1 + diff_keys));
		// (next_key + read) for each new key + status + next_key to check if we are done
		let reads = diff_keys * 2 + 2;
		let writes = 1 + diff_data; // status
		assert_eq!(weight, weight_for(reads, writes));

		assert!(matches!(
			StateMigrationStatusValue::<Test>::get(),
			StateMigrationStatus::Complete,
		));
	})
}
