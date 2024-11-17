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
use crate::{
	foreign_asset::ForeignAssetMigrationStatus,
	mock::{AssetId, Balances},
};
use pallet_assets::Asset;
use pallet_evm::AddressMapping;
use sp_runtime::DispatchError;
use xcm::latest::Location;
use {
	crate::{
		mock::{
			AccountId, AssetManager, Assets, ExtBuilder, LazyMigrations, MockAssetType,
			RuntimeOrigin, Test, ALITH, BOB,
		},
		Error, StateMigrationStatus, StateMigrationStatusValue, MAX_ITEM_PROOF_SIZE,
		PROOF_SIZE_BUFFER,
	},
	frame_support::{assert_noop, assert_ok, traits::Hooks, weights::Weight},
	rlp::RlpStream,
	sp_core::{H160, H256},
	sp_io::hashing::keccak_256,
	sp_runtime::traits::Bounded,
};

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
			RuntimeOrigin::signed(AccountId::from([45; 20])),
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
				RuntimeOrigin::signed(AccountId::from([45; 20])),
				vec![contract1_address].try_into().unwrap(),
				1000
			),
			Error::<Test>::ContractNotCorrupted
		);

		assert_noop!(
			LazyMigrations::clear_suicided_storage(
				RuntimeOrigin::signed(AccountId::from([45; 20])),
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
			RuntimeOrigin::signed(AccountId::from([45; 20])),
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
			RuntimeOrigin::signed(AccountId::from([45; 20])),
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
			RuntimeOrigin::signed(AccountId::from([45; 20])),
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
				RuntimeOrigin::signed(AccountId::from([45; 20])),
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

#[test]
fn test_create_contract_metadata_contract_not_exist() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			LazyMigrations::create_contract_metadata(
				RuntimeOrigin::signed(AccountId::from([45; 20])),
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
			RuntimeOrigin::signed(AccountId::from([45; 20])),
			address,
		));

		assert!(pallet_evm::AccountCodesMetadata::<Test>::get(address).is_some());

		// Should not be able to set metadata again
		assert_noop!(
			LazyMigrations::create_contract_metadata(
				RuntimeOrigin::signed(AccountId::from([45; 20])),
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

// Helper function to create a foreign asset with basic metadata
fn create_old_foreign_asset(location: Location) -> AssetId {
	let asset = MockAssetType::Xcm(location.clone().into());
	let asset_id: AssetId = asset.clone().into();
	// First register asset in asset manager with a Location
	assert_ok!(AssetManager::register_foreign_asset(
		RuntimeOrigin::root(),
		asset,
		1,
		1,
		true,
	));

	// Set metadata for the asset
	assert_ok!(Assets::set_metadata(
		RuntimeOrigin::signed(AssetManager::account_id()),
		asset_id,
		b"Test".to_vec(),
		b"TEST".to_vec(),
		12,
	));

	asset_id
}

#[test]
fn test_start_foreign_asset_migration_success() {
	ExtBuilder::default().build().execute_with(|| {
		let location = xcm::latest::Location::new(1, [xcm::latest::Junction::Parachain(1000)]);
		let asset_id = create_old_foreign_asset(location);

		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(AssetManager::account_id()),
			asset_id.into(),
			ALITH.into(),
			100,
		));

		// Verify asset is live by calling transfer in pallet assets
		assert_ok!(Assets::transfer(
			RuntimeOrigin::signed(ALITH),
			asset_id.into(),
			BOB.into(),
			100,
		));

		// Try to migrate the asset
		assert_ok!(LazyMigrations::start_foreign_assets_migration(
			RuntimeOrigin::root(),
			asset_id,
		));

		// Verify asset is frozen by calling transfer in pallet assets
		assert_noop!(
			Assets::transfer(
				RuntimeOrigin::signed(ALITH),
				asset_id.into(),
				BOB.into(),
				100,
			),
			pallet_assets::Error::<Test>::AssetNotLive
		);

		// Verify migration status
		match crate::pallet::ForeignAssetMigrationStatusValue::<Test>::get() {
			ForeignAssetMigrationStatus::Migrating(info) => {
				assert_eq!(info.asset_id, asset_id);
				assert_eq!(info.remaining_balances, 1);
				assert_eq!(info.remaining_approvals, 0);
			}
			_ => panic!("Expected migration status to be Migrating"),
		}
	});
}

#[test]
fn test_start_foreign_asset_migration_already_migrating() {
	ExtBuilder::default().build().execute_with(|| {
		let location = xcm::latest::Location::new(1, [xcm::latest::Junction::Parachain(1000)]);
		let asset_id = create_old_foreign_asset(location);

		// Start first migrationJunction::Parachain(1000)
		assert_ok!(LazyMigrations::start_foreign_assets_migration(
			RuntimeOrigin::root(),
			asset_id,
		));

		// Try to start another migration while one is in progress
		assert_noop!(
			LazyMigrations::start_foreign_assets_migration(RuntimeOrigin::root(), 2u128),
			Error::<Test>::MigrationNotFinished
		);
	});
}

#[test]
fn test_start_foreign_asset_migration_asset_not_found() {
	ExtBuilder::default().build().execute_with(|| {
		// Try to migrate non-existent asset
		assert_noop!(
			LazyMigrations::start_foreign_assets_migration(RuntimeOrigin::root(), 1u128),
			Error::<Test>::AssetNotFound
		);
	});
}

#[test]
fn test_start_foreign_asset_migration_asset_type_not_found() {
	ExtBuilder::default().build().execute_with(|| {
		let asset_id = 1u128;

		// Create asset without registering in asset manager
		assert_ok!(Assets::create(
			RuntimeOrigin::signed(ALITH),
			asset_id.into(),
			ALITH.into(),
			1,
		));

		assert_noop!(
			LazyMigrations::start_foreign_assets_migration(RuntimeOrigin::root(), asset_id),
			Error::<Test>::AssetTypeNotFound
		);
	});
}

#[test]
fn test_start_foreign_asset_migration_unauthorized() {
	ExtBuilder::default().build().execute_with(|| {
		let location = xcm::latest::Location::new(1, [xcm::latest::Junction::Parachain(1000)]);
		let asset_id = create_old_foreign_asset(location);

		// Try to migrate with non-root origin
		assert_noop!(
			LazyMigrations::start_foreign_assets_migration(RuntimeOrigin::signed(ALITH), asset_id),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn test_start_foreign_asset_migration_with_balances_and_approvals() {
	ExtBuilder::default().build().execute_with(|| {
		let location = xcm::latest::Location::new(1, [xcm::latest::Junction::Parachain(1000)]);
		let asset_id = create_old_foreign_asset(location);

		// Add some balances
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(AssetManager::account_id()),
			asset_id.into(),
			BOB.into(),
			100,
		));

		// Add some approvals
		assert_ok!(Assets::approve_transfer(
			RuntimeOrigin::signed(BOB),
			asset_id.into(),
			AccountId::from([3; 20]).into(),
			50,
		));

		// Start migration
		assert_ok!(LazyMigrations::start_foreign_assets_migration(
			RuntimeOrigin::root(),
			asset_id,
		));

		// Verify migration status includes the balances and approvals
		match crate::pallet::ForeignAssetMigrationStatusValue::<Test>::get() {
			ForeignAssetMigrationStatus::Migrating(info) => {
				assert_eq!(info.asset_id, asset_id);
				assert_eq!(info.remaining_balances, 1);
				assert_eq!(info.remaining_approvals, 1);
			}
			_ => panic!("Expected migration status to be Migrating"),
		}
	});
}

#[test]
fn test_migrate_foreign_asset_balances_without_migration_started() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			LazyMigrations::migrate_foreign_asset_balances(RuntimeOrigin::signed(ALITH), 100),
			Error::<Test>::NoMigrationInProgress
		);
	});
}

#[test]
fn test_migrate_foreign_asset_balances_zero_limit() {
	ExtBuilder::default().build().execute_with(|| {
		let location = xcm::latest::Location::new(1, [xcm::latest::Junction::Parachain(1000)]);
		let asset_id = create_old_foreign_asset(location);

		// Start migration
		assert_ok!(LazyMigrations::start_foreign_assets_migration(
			RuntimeOrigin::root(),
			asset_id,
		));

		assert_noop!(
			LazyMigrations::migrate_foreign_asset_balances(RuntimeOrigin::signed(ALITH), 0),
			Error::<Test>::LimitCannotBeZero
		);
	});
}

#[test]
fn test_migrate_foreign_asset_balances_single_account() {
	ExtBuilder::default().build().execute_with(|| {
		let location = xcm::latest::Location::new(1, [xcm::latest::Junction::Parachain(1000)]);
		let asset_id = create_old_foreign_asset(location);

		// Mint some tokens to an account
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(AssetManager::account_id()),
			asset_id.into(),
			BOB.into(),
			100,
		));

		// Start migration
		assert_ok!(LazyMigrations::start_foreign_assets_migration(
			RuntimeOrigin::root(),
			asset_id,
		));

		// Migrate balances
		assert_ok!(LazyMigrations::migrate_foreign_asset_balances(
			RuntimeOrigin::signed(ALITH),
			10
		));

		// Verify migration status
		match crate::pallet::ForeignAssetMigrationStatusValue::<Test>::get() {
			ForeignAssetMigrationStatus::Migrating(info) => {
				assert_eq!(info.asset_id, asset_id);
				assert_eq!(info.remaining_balances, 0);
				assert_eq!(info.remaining_approvals, 0);
			}
			_ => panic!("Expected migration status to be Migrating"),
		}

		// Verify the balance was migrated by
		assert_eq!(Assets::balance(asset_id, BOB), 0);
	});
}

#[test]
fn test_migrate_foreign_asset_balances_multiple_accounts() {
	ExtBuilder::default().build().execute_with(|| {
		let location = xcm::latest::Location::new(1, [xcm::latest::Junction::Parachain(1000)]);
		let asset_id = create_old_foreign_asset(location);

		// Mint tokens to multiple accounts
		for i in 1..=5 {
			assert_ok!(Assets::mint(
				RuntimeOrigin::signed(AssetManager::account_id()),
				asset_id.into(),
				AccountId::from([i as u8; 20]).into(),
				100 * i as u128,
			));
		}

		// Start migration
		assert_ok!(LazyMigrations::start_foreign_assets_migration(
			RuntimeOrigin::root(),
			asset_id,
		));

		// Migrate balances in batches
		assert_ok!(LazyMigrations::migrate_foreign_asset_balances(
			RuntimeOrigin::signed(ALITH),
			3
		));

		// Check intermediate state
		match crate::pallet::ForeignAssetMigrationStatusValue::<Test>::get() {
			ForeignAssetMigrationStatus::Migrating(info) => {
				assert_eq!(info.asset_id, asset_id);
				assert_eq!(info.remaining_balances, 2);
				assert_eq!(info.remaining_approvals, 0);
			}
			_ => panic!("Expected migration status to be Migrating"),
		}

		// Migrate remaining balances
		assert_ok!(LazyMigrations::migrate_foreign_asset_balances(
			RuntimeOrigin::signed(ALITH),
			2
		));

		// Verify final state
		match crate::pallet::ForeignAssetMigrationStatusValue::<Test>::get() {
			ForeignAssetMigrationStatus::Migrating(info) => {
				assert_eq!(info.asset_id, asset_id);
				assert_eq!(info.remaining_balances, 0);
				assert_eq!(info.remaining_approvals, 0);
			}
			_ => panic!("Expected migration status to be Migrating"),
		}

		// Verify all balances were migrated correctly
		for i in 1..=5 {
			assert_eq!(Assets::balance(asset_id, AccountId::from([i as u8; 20])), 0);
		}
	});
}

#[test]
fn test_migrate_foreign_asset_balances_with_reserved_deposits() {
	ExtBuilder::default().build().execute_with(|| {
		let location = xcm::latest::Location::new(1, [xcm::latest::Junction::Parachain(1000)]);
		let asset_id = create_old_foreign_asset(location);

		// Create account with reserved deposit
		let account = BOB;
		assert_ok!(Assets::touch(
			RuntimeOrigin::signed(account.clone()),
			asset_id.into(),
		));

		// Mint some tokens
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(AssetManager::account_id()),
			asset_id.into(),
			account.clone().into(),
			100,
		));

		// Check initial reserved balance
		let initial_reserved = Balances::reserved_balance(&account);
		assert!(initial_reserved > 0);

		// Start migration
		assert_ok!(LazyMigrations::start_foreign_assets_migration(
			RuntimeOrigin::root(),
			asset_id,
		));

		// Migrate balances
		assert_ok!(LazyMigrations::migrate_foreign_asset_balances(
			RuntimeOrigin::signed(ALITH),
			10
		));

		// Verify the balance was migrated
		assert_eq!(Assets::balance(asset_id, account.clone()), 0);

		// Verify the deposit was unreserved
		assert_eq!(Balances::reserved_balance(&account), 0);
	});
}

#[test]
fn test_migrate_foreign_asset_approvals_without_migration_started() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			LazyMigrations::migrate_foreign_asset_approvals(
				RuntimeOrigin::signed(AccountId::from([45; 20])),
				100
			),
			Error::<Test>::NoMigrationInProgress
		);
	});
}

#[test]
fn test_migrate_foreign_asset_approvals_zero_limit() {
	ExtBuilder::default().build().execute_with(|| {
		// Create and start migration for an asset
		let location = xcm::latest::Location::new(1, [xcm::latest::Junction::Parachain(1000)]);
		let asset_id = create_old_foreign_asset(location);
		assert_ok!(LazyMigrations::start_foreign_assets_migration(
			RuntimeOrigin::root(),
			asset_id,
		));

		assert_noop!(
			LazyMigrations::migrate_foreign_asset_approvals(
				RuntimeOrigin::signed(AccountId::from([45; 20])),
				0
			),
			Error::<Test>::LimitCannotBeZero
		);
	});
}

#[test]
fn test_migrate_foreign_asset_approvals_single_approval() {
	ExtBuilder::default().build().execute_with(|| {
		let location = xcm::latest::Location::new(1, [xcm::latest::Junction::Parachain(1000)]);
		let asset_id = create_old_foreign_asset(location);

		// Create an approval
		assert_ok!(Assets::approve_transfer(
			RuntimeOrigin::signed(BOB),
			asset_id.into(),
			AccountId::from([3; 20]).into(),
			50,
		));

		// Start migration
		assert_ok!(LazyMigrations::start_foreign_assets_migration(
			RuntimeOrigin::root(),
			asset_id,
		));

		// Check initial migration status
		match crate::pallet::ForeignAssetMigrationStatusValue::<Test>::get() {
			ForeignAssetMigrationStatus::Migrating(info) => {
				assert_eq!(info.remaining_approvals, 1);
			}
			_ => panic!("Expected migration status to be Migrating"),
		}

		// Initial reserved balance for approval
		let initial_reserved = Balances::reserved_balance(&BOB);
		assert!(initial_reserved > 0);

		// Migrate approvals
		assert_ok!(LazyMigrations::migrate_foreign_asset_approvals(
			RuntimeOrigin::signed(ALITH),
			10
		));

		// Check final migration status
		match crate::pallet::ForeignAssetMigrationStatusValue::<Test>::get() {
			ForeignAssetMigrationStatus::Migrating(info) => {
				assert_eq!(info.remaining_approvals, 0);
			}
			_ => panic!("Expected migration status to be Migrating"),
		}

		// Verify the deposit was unreserved
		assert_eq!(Balances::reserved_balance(&BOB), 0);
	});
}

#[test]
fn test_migrate_foreign_asset_approvals_multiple_approvals() {
	ExtBuilder::default().build().execute_with(|| {
		let location = xcm::latest::Location::new(1, [xcm::latest::Junction::Parachain(1000)]);
		let asset_id = create_old_foreign_asset(location);

		// Create multiple approvals from different accounts
		for i in 1..=5 {
			let owner = AccountId::from([i as u8; 20]);
			// Add balance for each account
			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				owner.clone(),
				100 * i as u128,
			));

			let spender = AccountId::from([i as u8 + 1; 20]);
			assert_ok!(Assets::approve_transfer(
				RuntimeOrigin::signed(owner.clone()),
				asset_id.into(),
				spender.into(),
				50 * i as u128,
			));
		}

		// Start migration
		assert_ok!(LazyMigrations::start_foreign_assets_migration(
			RuntimeOrigin::root(),
			asset_id,
		));

		// Check initial migration status
		match crate::pallet::ForeignAssetMigrationStatusValue::<Test>::get() {
			ForeignAssetMigrationStatus::Migrating(info) => {
				assert_eq!(info.remaining_approvals, 5);
			}
			_ => panic!("Expected migration status to be Migrating"),
		}

		// Migrate approvals in batches
		assert_ok!(LazyMigrations::migrate_foreign_asset_approvals(
			RuntimeOrigin::signed(ALITH),
			3
		));

		// Check intermediate state
		match crate::pallet::ForeignAssetMigrationStatusValue::<Test>::get() {
			ForeignAssetMigrationStatus::Migrating(info) => {
				assert_eq!(info.remaining_approvals, 2);
			}
			_ => panic!("Expected migration status to be Migrating"),
		}

		// Migrate remaining approvals
		assert_ok!(LazyMigrations::migrate_foreign_asset_approvals(
			RuntimeOrigin::signed(ALITH),
			2
		));

		// Check final migration status
		match crate::pallet::ForeignAssetMigrationStatusValue::<Test>::get() {
			ForeignAssetMigrationStatus::Migrating(info) => {
				assert_eq!(info.remaining_approvals, 0);
			}
			_ => panic!("Expected migration status to be Migrating"),
		}

		// Verify all deposits were unreserved
		for i in 1..=5 {
			let owner = AccountId::from([i as u8; 20]);
			assert_eq!(Balances::reserved_balance(&owner), 0);
		}
	});
}

#[test]
fn test_migrate_foreign_asset_approvals_with_balances() {
	ExtBuilder::default().build().execute_with(|| {
		let location = xcm::latest::Location::new(1, [xcm::latest::Junction::Parachain(1000)]);
		let asset_id = create_old_foreign_asset(location);

		// Create balance and approval for account
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(AssetManager::account_id()),
			asset_id.into(),
			BOB.into(),
			100,
		));

		assert_ok!(Assets::approve_transfer(
			RuntimeOrigin::signed(BOB),
			asset_id.into(),
			AccountId::from([3; 20]).into(),
			50,
		));

		// Start migration
		assert_ok!(LazyMigrations::start_foreign_assets_migration(
			RuntimeOrigin::root(),
			asset_id,
		));

		// Migrate approvals first
		assert_ok!(LazyMigrations::migrate_foreign_asset_approvals(
			RuntimeOrigin::signed(ALITH),
			10
		));

		// Check that approvals were migrated but balances remain
		match crate::pallet::ForeignAssetMigrationStatusValue::<Test>::get() {
			ForeignAssetMigrationStatus::Migrating(info) => {
				assert_eq!(info.remaining_approvals, 0);
				assert_eq!(info.remaining_balances, 1);
			}
			_ => panic!("Expected migration status to be Migrating"),
		}

		// Migrate balances
		assert_ok!(LazyMigrations::migrate_foreign_asset_balances(
			RuntimeOrigin::signed(ALITH),
			10
		));

		// Verify final state
		match crate::pallet::ForeignAssetMigrationStatusValue::<Test>::get() {
			ForeignAssetMigrationStatus::Migrating(info) => {
				assert_eq!(info.remaining_approvals, 0);
				assert_eq!(info.remaining_balances, 0);
			}
			_ => panic!("Expected migration status to be Migrating"),
		}
	});
}

#[test]
fn test_migrate_foreign_asset_approvals_exceed_limit() {
	ExtBuilder::default().build().execute_with(|| {
		let location = xcm::latest::Location::new(1, [xcm::latest::Junction::Parachain(1000)]);
		let asset_id = create_old_foreign_asset(location);

		for i in 1..=10 {
			let owner = AccountId::from([i as u8; 20]);
			// Add balance and approval for each account
			assert_ok!(Balances::force_set_balance(
				RuntimeOrigin::root(),
				owner.clone(),
				100 * i as u128,
			));

			let spender = AccountId::from([i as u8 + 1; 20]);
			assert_ok!(Assets::approve_transfer(
				RuntimeOrigin::signed(owner.clone()),
				asset_id.into(),
				spender.into(),
				50 * i as u128,
			));
		}

		// Start migration
		assert_ok!(LazyMigrations::start_foreign_assets_migration(
			RuntimeOrigin::root(),
			asset_id,
		));

		// Migrate with limit less than total approvals
		assert_ok!(LazyMigrations::migrate_foreign_asset_approvals(
			RuntimeOrigin::signed(ALITH),
			5
		));

		// Verify partial migration
		match crate::pallet::ForeignAssetMigrationStatusValue::<Test>::get() {
			ForeignAssetMigrationStatus::Migrating(info) => {
				assert_eq!(info.remaining_approvals, 5);
			}
			_ => panic!("Expected migration status to be Migrating"),
		}
	});
}
