// Copyright 2019-2021 PureStake Inc.
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
			1,
			200u128.into()
		));

		assert_eq!(AssetManager::asset_id_units_per_second(1).unwrap(), 200);
		assert!(AssetManager::supported_fee_payment_assets().contains(&1));

		expect_events(vec![
			crate::Event::AssetRegistered(1, MockAssetType::MockAsset(1), 0),
			crate::Event::UnitsPerSecondChanged(1, 200),
		])
	});
}

#[test]
fn test_change_units_per_second_after_setting_it_once() {
	new_test_ext().execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			Origin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			true,
		));

		assert_ok!(AssetManager::set_asset_units_per_second(
			Origin::root(),
			1,
			200u128.into()
		));

		assert_eq!(AssetManager::asset_id_units_per_second(1).unwrap(), 200);
		assert!(AssetManager::supported_fee_payment_assets().contains(&1));

		assert_ok!(AssetManager::set_asset_units_per_second(
			Origin::root(),
			1,
			100u128.into()
		));

		assert_eq!(AssetManager::asset_id_units_per_second(1).unwrap(), 100);
		assert!(AssetManager::supported_fee_payment_assets().contains(&1));

		expect_events(vec![
			crate::Event::AssetRegistered(1, MockAssetType::MockAsset(1), 0),
			crate::Event::UnitsPerSecondChanged(1, 200),
			crate::Event::UnitsPerSecondChanged(1, 100),
		]);
	});
}

#[test]
fn test_root_can_change_units_per_second_and_then_remove() {
	new_test_ext().execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			Origin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			true,
		));

		assert_ok!(AssetManager::set_asset_units_per_second(
			Origin::root(),
			1,
			200u128.into()
		));

		assert_eq!(AssetManager::asset_id_units_per_second(1).unwrap(), 200);
		assert!(AssetManager::supported_fee_payment_assets().contains(&1));

		assert_ok!(AssetManager::remove_supported_asset(Origin::root(), 1,));

		expect_events(vec![
			crate::Event::AssetRegistered(1, MockAssetType::MockAsset(1), 0),
			crate::Event::UnitsPerSecondChanged(1, 200),
			crate::Event::SupportedAssetRemoved(1),
		]);
	});
}

#[test]
fn test_asset_id_non_existent_error() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			AssetManager::set_asset_units_per_second(Origin::root(), 1, 200u128.into()),
			Error::<Test>::AssetDoesNotExist
		);

		assert_noop!(
			AssetManager::remove_supported_asset(Origin::root(), 1),
			Error::<Test>::AssetDoesNotExist
		);
	});
}

#[test]
fn test_populate_supported_fee_payment_assets_works() {
	new_test_ext().execute_with(|| {
		use frame_support::StorageHasher;
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdUnitsPerSecond";
		use frame_support::traits::OnRuntimeUpgrade;
		use parity_scale_codec::Encode;

		put_storage_value(
			pallet_prefix,
			storage_item_prefix,
			&Blake2_128Concat::hash(&1u32.encode()),
			10u128,
		);

		assert_noop!(
			AssetManager::set_asset_units_per_second(Origin::root(), 1, 200u128.into()),
			Error::<Test>::AssetDoesNotExist
		);

		assert!(AssetManager::supported_fee_payment_assets().len() == 0);

		// We run the migration
		crate::migrations::PopulateSupportedFeePaymentAssets::<Test>::on_runtime_upgrade();

		assert!(AssetManager::supported_fee_payment_assets().len() == 1);
		assert!(AssetManager::supported_fee_payment_assets().contains(&1));
	});
}
