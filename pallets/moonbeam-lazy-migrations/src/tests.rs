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
		Error, StateMigrationStatus, StateMigrationStatusValue, MAX_ITEM_PROOF_SIZE,
		PROOF_SIZE_BUFFER,
	},
	frame_support::{assert_noop, assert_ok, traits::Hooks, weights::Weight},
	rlp::RlpStream,
	sp_core::{H160, H256},
	sp_io::hashing::keccak_256,
	sp_runtime::{traits::Bounded, AccountId32},
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

fn create_dummy_contract_without_metadata(seed: u8) -> H160 {
	let address = address_build(seed);
	let dummy_code = vec![1, 2, 3];
	pallet_evm::AccountCodes::<Test>::insert(address, dummy_code);
	address
}

#[test]
fn test_create_contract_metadata_contract_not_exist() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			LazyMigrations::create_contract_metadata(
				RuntimeOrigin::signed(AccountId32::from([45; 32])),
				address_build(1),
			),
			Error::<Test>::ContractNotExist
		);
	});
}

#[test]
fn test_create_contract_metadata_success_path() {
	ExtBuilder::default().build().execute_with(|| {
		// Setup: create a dummy contract
		let address = create_dummy_contract_without_metadata(1);

		assert_ok!(LazyMigrations::create_contract_metadata(
			RuntimeOrigin::signed(AccountId32::from([45; 32])),
			address,
		));

		assert!(pallet_evm::AccountCodesMetadata::<Test>::get(address).is_some());

		// Should not be able to set metadata again
		assert_noop!(
			LazyMigrations::create_contract_metadata(
				RuntimeOrigin::signed(AccountId32::from([45; 32])),
				address,
			),
			Error::<Test>::ContractMetadataAlreadySet
		);
	});
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
		keys += 1;
		if let Some(_) = sp_io::storage::get(&key) {
			data += 1;
		}
		current_key = sp_io::storage::next_key(&key);
	}

	(keys, data)
}

fn weight_for(read: u64, write: u64) -> Weight {
	<Test as frame_system::Config>::DbWeight::get().reads_writes(read, write)
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
			(StateMigrationStatus::NotStarted, 0)
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
			(StateMigrationStatus::Complete, keys)
		);
	})
}

#[test]
fn test_state_migration_cannot_fit_any_item() {
	ExtBuilder::default().build().execute_with(|| {
		StateMigrationStatusValue::<Test>::put((StateMigrationStatus::NotStarted, 0));

		let weight = LazyMigrations::on_idle(0, rem_weight_for_entries(0));

		assert_eq!(weight, weight_for(0, 0));
	})
}

#[test]
fn test_state_migration_when_complete() {
	ExtBuilder::default().build().execute_with(|| {
		StateMigrationStatusValue::<Test>::put((StateMigrationStatus::Complete, 0));

		let weight = LazyMigrations::on_idle(0, Weight::max_value());

		// just reading the status of the migration
		assert_eq!(weight, weight_for(1, 0));
	})
}

#[test]
fn test_state_migration_when_errored() {
	ExtBuilder::default().build().execute_with(|| {
		StateMigrationStatusValue::<Test>::put((
			StateMigrationStatus::Error("Error".as_bytes().to_vec().try_into().unwrap_or_default()),
			1,
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
			(StateMigrationStatus::NotStarted, 0)
		);

		let data = sp_io::storage::get(Default::default());
		let weight = LazyMigrations::on_idle(0, rem_weight_for_entries(1));

		let reads = 2; // key read + status read
		let writes = 1 + data.map(|_| 1).unwrap_or(0);
		assert_eq!(weight, weight_for(reads, writes));

		assert!(matches!(
			StateMigrationStatusValue::<Test>::get(),
			(StateMigrationStatus::Started(_), 1)
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
			(StateMigrationStatus::NotStarted, 0)
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
			(StateMigrationStatus::Started(_), 3)
		));
	})
}

#[test]
fn test_state_migration_can_fit_exactly_all_item() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			StateMigrationStatusValue::<Test>::get(),
			(StateMigrationStatus::NotStarted, 0)
		);

		let (keys, data) = count_keys_and_data_without_code();
		let weight = LazyMigrations::on_idle(0, rem_weight_for_entries(keys));

		// we deduct the extra next_key request to check if we are done.
		// will know if we are done on the next call to on_idle
		assert_eq!(weight, weight_for(2 * keys + 1, data + 1));

		assert!(matches!(
			StateMigrationStatusValue::<Test>::get(),
			(StateMigrationStatus::Started(_), n) if n == keys,
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
			(StateMigrationStatus::Complete, n) if n == new_keys,
		));
	})
}

#[test]
fn test_state_migration_will_migrate_10_000_items() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(
			StateMigrationStatusValue::<Test>::get(),
			(StateMigrationStatus::NotStarted, 0)
		);

		for i in 0..100 {
			mock_contract_with_entries(i as u8, i as u64, 100);
		}

		StateMigrationStatusValue::<Test>::put((StateMigrationStatus::NotStarted, 0));

		let (keys, data) = count_keys_and_data_without_code();

		// assuming we can only fit 100 items at a time

		let mut total_weight: Weight = Weight::zero();
		let num_of_on_idle_calls = 200;
		let entries_per_on_idle = 100;
		let needed_on_idle_calls = (keys as f64 / entries_per_on_idle as f64).ceil() as u64;

		// Reads:
		// Read status => num_of_on_idle_calls
		// Read keys   => keys
		// Next keys   => keys - 1  + 1 skip + 1 done check
		//
		// Writes:
		// Write status => needed_on_idle_calls
		// Write keys   => data
		let expected_reads = (keys - 1 + 2) + keys + num_of_on_idle_calls;
		let expected_writes = data + needed_on_idle_calls;

		println!("Keys: {}, Data: {}", keys, data);
		println!("entries_per_on_idle: {}", entries_per_on_idle);
		println!("num_of_on_idle_calls: {}", num_of_on_idle_calls);
		println!("needed_on_idle_calls: {}", needed_on_idle_calls);
		println!(
			"Expected Reads: {}, Expected Writes: {}",
			expected_reads, expected_writes
		);

		for i in 1..=num_of_on_idle_calls {
			let weight = LazyMigrations::on_idle(i, rem_weight_for_entries(entries_per_on_idle));
			total_weight = total_weight.saturating_add(weight);

			let status = StateMigrationStatusValue::<Test>::get();
			if i < needed_on_idle_calls {
				let migrated_so_far = i * entries_per_on_idle;
				assert!(
					matches!(status, (StateMigrationStatus::Started(_), n) if n == migrated_so_far),
					"Status: {:?} at call: #{} doesn't match Started",
					status,
					i,
				);
				assert!(weight.all_gte(weight_for(1, 0)));
			} else {
				assert!(
					matches!(status, (StateMigrationStatus::Complete, n) if n == keys),
					"Status: {:?} at call: {} doesn't match Complete",
					status,
					i,
				);
				if i == needed_on_idle_calls {
					// last call to on_idle
					assert!(weight.all_gte(weight_for(1, 0)));
				} else {
					// extra calls to on_idle, just status update check
					assert_eq!(weight, weight_for(1, 0));
				}
			}
		}

		assert_eq!(total_weight, weight_for(expected_reads, expected_writes));
	})
}
