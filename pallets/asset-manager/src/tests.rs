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

// Tests for AssetManager Pallet
use crate::*;
use mock::*;

use frame_support::{
	assert_noop, assert_ok, storage::migration::put_storage_value, Blake2_128Concat,
};

#[test]
fn registering_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			Origin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			true
		));

		assert_eq!(
			AssetManager::asset_id_type(1).unwrap(),
			MockAssetType::MockAsset(1)
		);
		assert_eq!(
			AssetManager::asset_type_id(MockAssetType::MockAsset(1)).unwrap(),
			1
		);
		expect_events(vec![crate::Event::AssetRegistered(
			1,
			MockAssetType::MockAsset(1),
			0u32,
		)])
	});
}

#[test]
fn test_asset_exists_error() {
	new_test_ext().execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			Origin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			true
		));

		assert_eq!(
			AssetManager::asset_id_type(1).unwrap(),
			MockAssetType::MockAsset(1)
		);
		assert_noop!(
			AssetManager::register_asset(
				Origin::root(),
				MockAssetType::MockAsset(1),
				0u32.into(),
				1u32.into(),
				true
			),
			Error::<Test>::AssetAlreadyExists
		);
	});
}

#[test]
fn test_root_can_change_units_per_second() {
	new_test_ext().execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			Origin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			true
		));

		assert_ok!(AssetManager::set_asset_units_per_second(
			Origin::root(),
			MockAssetType::MockAsset(1),
			200u128.into()
		));

		assert_eq!(
			AssetManager::asset_type_units_per_second(MockAssetType::MockAsset(1)).unwrap(),
			200
		);

		expect_events(vec![
			crate::Event::AssetRegistered(1, MockAssetType::MockAsset(1), 0),
			crate::Event::UnitsPerSecondChanged(MockAssetType::MockAsset(1), 200),
		])
	});
}

#[test]
fn test_root_can_change_asset_id_type() {
	new_test_ext().execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			Origin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			true
		));

		assert_ok!(AssetManager::change_existing_asset_type(
			Origin::root(),
			1,
			MockAssetType::MockAsset(2),
		));

		expect_events(vec![
			crate::Event::AssetRegistered(1, MockAssetType::MockAsset(1), 0),
			crate::Event::AssetTypeChanged(1, MockAssetType::MockAsset(2)),
		])
	});
}

#[test]
fn test_asset_id_non_existent_error() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			AssetManager::set_asset_units_per_second(
				Origin::root(),
				MockAssetType::MockAsset(1),
				200u128.into()
			),
			Error::<Test>::AssetDoesNotExist
		);
		assert_noop!(
			AssetManager::change_existing_asset_type(
				Origin::root(),
				1,
				MockAssetType::MockAsset(2),
			),
			Error::<Test>::AssetDoesNotExist
		);
	});
}

#[test]
fn test_asset_manager_units_with_asset_type_migration_works() {
	new_test_ext().execute_with(|| {
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdUnitsPerSecond";
		use frame_support::traits::OnRuntimeUpgrade;
		use frame_support::StorageHasher;
		use parity_scale_codec::Encode;

		assert_ok!(AssetManager::register_asset(
			Origin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			true
		));

		// We populate the previous storage with assetId as key
		put_storage_value(
			pallet_prefix,
			storage_item_prefix,
			&Blake2_128Concat::hash(&1u32.encode()),
			200u128,
		);

		// We run the migration
		crate::migrations::AssetManagerUnitsWithAssetType::<Test>::on_runtime_upgrade();

		// After migration, units per second should be indexed by AssetType
		assert_eq!(
			AssetManager::asset_type_units_per_second(MockAssetType::MockAsset(1)).unwrap(),
			200
		);
	});
}

#[test]
fn test_asset_manager_populate_asset_type_id_storage_migration_works() {
	new_test_ext().execute_with(|| {
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdType";
		use frame_support::traits::OnRuntimeUpgrade;
		use frame_support::StorageHasher;
		use parity_scale_codec::Encode;

		// We populate AssetIdType manually
		put_storage_value(
			pallet_prefix,
			storage_item_prefix,
			&Blake2_128Concat::hash(&1u32.encode()),
			MockAssetType::MockAsset(1),
		);

		// We run the migration
		crate::migrations::AssetManagerPopulateAssetTypeIdStorage::<Test>::on_runtime_upgrade();

		// After migration, the new storage item should be populated
		assert_eq!(
			AssetManager::asset_type_id(MockAssetType::MockAsset(1)).unwrap(),
			1
		);
	});
}
