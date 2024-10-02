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

use frame_support::{assert_noop, assert_ok};

#[test]
fn registering_foreign_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			RuntimeOrigin::root(),
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
		expect_events(vec![crate::Event::ForeignAssetRegistered {
			asset_id: 1,
			asset: MockAssetType::MockAsset(1),
			metadata: 0u32,
		}])
	});
}

#[test]
fn test_asset_exists_error() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			RuntimeOrigin::root(),
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
			AssetManager::register_foreign_asset(
				RuntimeOrigin::root(),
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
fn test_regular_user_cannot_call_extrinsics() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			AssetManager::register_foreign_asset(
				RuntimeOrigin::signed(1),
				MockAssetType::MockAsset(1),
				0u32.into(),
				1u32.into(),
				true
			),
			sp_runtime::DispatchError::BadOrigin
		);

		assert_noop!(
			AssetManager::change_existing_asset_type(
				RuntimeOrigin::signed(1),
				1,
				MockAssetType::MockAsset(2),
				1
			),
			sp_runtime::DispatchError::BadOrigin
		);
	});
}

#[test]
fn test_root_can_change_asset_id_type() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			RuntimeOrigin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			true
		));

		assert_ok!(AssetManager::change_existing_asset_type(
			RuntimeOrigin::root(),
			1,
			MockAssetType::MockAsset(2),
			1
		));

		// New associations are established
		assert_eq!(
			AssetManager::asset_id_type(1).unwrap(),
			MockAssetType::MockAsset(2)
		);
		assert_eq!(
			AssetManager::asset_type_id(MockAssetType::MockAsset(2)).unwrap(),
			1
		);

		// Old ones are deleted
		assert!(AssetManager::asset_type_id(MockAssetType::MockAsset(1)).is_none());

		expect_events(vec![
			crate::Event::ForeignAssetRegistered {
				asset_id: 1,
				asset: MockAssetType::MockAsset(1),
				metadata: 0,
			},
			crate::Event::ForeignAssetXcmLocationChanged {
				asset_id: 1,
				new_asset_type: MockAssetType::MockAsset(2),
			},
		])
	});
}

#[test]
fn test_asset_id_non_existent_error() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			AssetManager::change_existing_asset_type(
				RuntimeOrigin::root(),
				1,
				MockAssetType::MockAsset(2),
				1
			),
			Error::<Test>::AssetDoesNotExist
		);
	});
}

#[test]
fn test_root_can_remove_asset_association() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			RuntimeOrigin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			true
		));

		assert_ok!(AssetManager::remove_existing_asset_type(
			RuntimeOrigin::root(),
			1,
			1
		));

		// Mappings are deleted
		assert!(AssetManager::asset_type_id(MockAssetType::MockAsset(1)).is_none());
		assert!(AssetManager::asset_id_type(1).is_none());

		expect_events(vec![
			crate::Event::ForeignAssetRegistered {
				asset_id: 1,
				asset: MockAssetType::MockAsset(1),
				metadata: 0,
			},
			crate::Event::ForeignAssetRemoved {
				asset_id: 1,
				asset_type: MockAssetType::MockAsset(1),
			},
		])
	});
}

#[test]
fn test_removing_without_asset_units_per_second_does_not_panic() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			RuntimeOrigin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			true
		));

		assert_ok!(AssetManager::remove_existing_asset_type(
			RuntimeOrigin::root(),
			1,
			0
		));

		// Mappings are deleted
		assert!(AssetManager::asset_type_id(MockAssetType::MockAsset(1)).is_none());
		assert!(AssetManager::asset_id_type(1).is_none());

		expect_events(vec![
			crate::Event::ForeignAssetRegistered {
				asset_id: 1,
				asset: MockAssetType::MockAsset(1),
				metadata: 0,
			},
			crate::Event::ForeignAssetRemoved {
				asset_id: 1,
				asset_type: MockAssetType::MockAsset(1),
			},
		])
	});
}

#[test]
fn test_destroy_foreign_asset_also_removes_everything() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			RuntimeOrigin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			true
		));

		assert_ok!(AssetManager::destroy_foreign_asset(
			RuntimeOrigin::root(),
			1,
			0,
		));

		// Mappings are deleted
		assert!(AssetManager::asset_type_id(MockAssetType::MockAsset(1)).is_none());
		assert!(AssetManager::asset_id_type(1).is_none());

		expect_events(vec![
			crate::Event::ForeignAssetRegistered {
				asset_id: 1,
				asset: MockAssetType::MockAsset(1),
				metadata: 0,
			},
			crate::Event::ForeignAssetDestroyed {
				asset_id: 1,
				asset_type: MockAssetType::MockAsset(1),
			},
		])
	});
}
