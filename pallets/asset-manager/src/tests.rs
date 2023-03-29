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
	assert_noop, assert_ok,
	storage::migration::{put_storage_value, storage_key_iter},
	Blake2_128Concat,
};
use xcm::latest::prelude::*;

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
fn registering_local_works() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.build()
		.execute_with(|| {
			let asset_id = MockLocalAssetIdCreator::create_asset_id_from_metadata(0);

			assert_ok!(AssetManager::register_local_asset(
				RuntimeOrigin::root(),
				1u64,
				1u64,
				true,
				0u32.into(),
			));

			assert_eq!(AssetManager::local_asset_counter(), 1);
			assert_eq!(
				AssetManager::local_asset_deposit(asset_id),
				Some(AssetInfo {
					creator: 1,
					deposit: 1
				})
			);

			expect_events(vec![crate::Event::LocalAssetRegistered {
				asset_id,
				creator: 1,
				owner: 1,
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
fn test_root_can_change_units_per_second() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			RuntimeOrigin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			true
		));

		assert_ok!(AssetManager::set_asset_units_per_second(
			RuntimeOrigin::root(),
			MockAssetType::MockAsset(1),
			200u128.into(),
			0
		));

		assert_eq!(
			AssetManager::asset_type_units_per_second(MockAssetType::MockAsset(1)).unwrap(),
			200
		);
		assert!(AssetManager::supported_fee_payment_assets().contains(&MockAssetType::MockAsset(1)));

		expect_events(vec![
			crate::Event::ForeignAssetRegistered {
				asset_id: 1,
				asset: MockAssetType::MockAsset(1),
				metadata: 0,
			},
			crate::Event::UnitsPerSecondChanged {
				asset_type: MockAssetType::MockAsset(1),
				units_per_second: 200,
			},
		])
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
			AssetManager::set_asset_units_per_second(
				RuntimeOrigin::signed(1),
				MockAssetType::MockAsset(1),
				200u128.into(),
				0
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

		assert_ok!(AssetManager::set_asset_units_per_second(
			RuntimeOrigin::root(),
			MockAssetType::MockAsset(1),
			200u128.into(),
			0
		));

		assert_ok!(AssetManager::change_existing_asset_type(
			RuntimeOrigin::root(),
			1,
			MockAssetType::MockAsset(2),
			1
		));

		// New one contains the new asset type units per second
		assert_eq!(
			AssetManager::asset_type_units_per_second(MockAssetType::MockAsset(2)).unwrap(),
			200
		);

		// Old one does not contain units per second
		assert!(AssetManager::asset_type_units_per_second(MockAssetType::MockAsset(1)).is_none());

		// New associations are stablished
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
			crate::Event::UnitsPerSecondChanged {
				asset_type: MockAssetType::MockAsset(1),
				units_per_second: 200,
			},
			crate::Event::ForeignAssetTypeChanged {
				asset_id: 1,
				new_asset_type: MockAssetType::MockAsset(2),
			},
		])
	});
}

#[test]
fn test_change_units_per_second_after_setting_it_once() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			RuntimeOrigin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			true,
		));

		assert_ok!(AssetManager::set_asset_units_per_second(
			RuntimeOrigin::root(),
			MockAssetType::MockAsset(1),
			200u128.into(),
			0
		));

		assert_eq!(
			AssetManager::asset_type_units_per_second(MockAssetType::MockAsset(1)).unwrap(),
			200
		);
		assert!(AssetManager::supported_fee_payment_assets().contains(&MockAssetType::MockAsset(1)));

		assert_ok!(AssetManager::set_asset_units_per_second(
			RuntimeOrigin::root(),
			MockAssetType::MockAsset(1),
			100u128.into(),
			1
		));

		assert_eq!(
			AssetManager::asset_type_units_per_second(MockAssetType::MockAsset(1)).unwrap(),
			100
		);
		assert!(AssetManager::supported_fee_payment_assets().contains(&MockAssetType::MockAsset(1)));

		expect_events(vec![
			crate::Event::ForeignAssetRegistered {
				asset_id: 1,
				asset: MockAssetType::MockAsset(1),
				metadata: 0,
			},
			crate::Event::UnitsPerSecondChanged {
				asset_type: MockAssetType::MockAsset(1),
				units_per_second: 200,
			},
			crate::Event::UnitsPerSecondChanged {
				asset_type: MockAssetType::MockAsset(1),
				units_per_second: 100,
			},
		]);
	});
}

#[test]
fn test_root_can_change_units_per_second_and_then_remove() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			RuntimeOrigin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			true,
		));

		assert_ok!(AssetManager::set_asset_units_per_second(
			RuntimeOrigin::root(),
			MockAssetType::MockAsset(1),
			200u128.into(),
			0
		));

		assert_eq!(
			AssetManager::asset_type_units_per_second(MockAssetType::MockAsset(1)).unwrap(),
			200
		);
		assert!(AssetManager::supported_fee_payment_assets().contains(&MockAssetType::MockAsset(1)));

		assert_ok!(AssetManager::remove_supported_asset(
			RuntimeOrigin::root(),
			MockAssetType::MockAsset(1),
			1,
		));

		assert!(
			!AssetManager::supported_fee_payment_assets().contains(&MockAssetType::MockAsset(1))
		);

		expect_events(vec![
			crate::Event::ForeignAssetRegistered {
				asset_id: 1,
				asset: MockAssetType::MockAsset(1),
				metadata: 0,
			},
			crate::Event::UnitsPerSecondChanged {
				asset_type: MockAssetType::MockAsset(1),
				units_per_second: 200,
			},
			crate::Event::SupportedAssetRemoved {
				asset_type: MockAssetType::MockAsset(1),
			},
		]);
	});
}

#[test]
fn test_weight_hint_error() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(AssetManager::register_foreign_asset(
			RuntimeOrigin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			true,
		));

		assert_ok!(AssetManager::set_asset_units_per_second(
			RuntimeOrigin::root(),
			MockAssetType::MockAsset(1),
			200u128.into(),
			0
		));

		assert_noop!(
			AssetManager::remove_supported_asset(
				RuntimeOrigin::root(),
				MockAssetType::MockAsset(1),
				0
			),
			Error::<Test>::TooLowNumAssetsWeightHint
		);
	});
}

#[test]
fn test_asset_id_non_existent_error() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			AssetManager::set_asset_units_per_second(
				RuntimeOrigin::root(),
				MockAssetType::MockAsset(1),
				200u128.into(),
				0
			),
			Error::<Test>::AssetDoesNotExist
		);
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
fn test_populate_supported_fee_payment_assets_works() {
	ExtBuilder::default().build().execute_with(|| {
		use frame_support::StorageHasher;
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetTypeUnitsPerSecond";
		use frame_support::traits::OnRuntimeUpgrade;
		use parity_scale_codec::Encode;

		put_storage_value(
			pallet_prefix,
			storage_item_prefix,
			&Blake2_128Concat::hash(&MockAssetType::MockAsset(1).encode()),
			10u128,
		);

		assert_noop!(
			AssetManager::set_asset_units_per_second(
				RuntimeOrigin::root(),
				MockAssetType::MockAsset(1),
				200u128.into(),
				0
			),
			Error::<Test>::AssetDoesNotExist
		);

		assert!(AssetManager::supported_fee_payment_assets().len() == 0);

		// We run the migration
		crate::migrations::PopulateSupportedFeePaymentAssets::<Test>::on_runtime_upgrade();

		assert!(AssetManager::supported_fee_payment_assets().len() == 1);
		assert!(AssetManager::supported_fee_payment_assets().contains(&MockAssetType::MockAsset(1)));
	});
}

#[test]
fn test_asset_manager_units_with_asset_type_migration_works() {
	ExtBuilder::default().build().execute_with(|| {
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdUnitsPerSecond";
		use frame_support::traits::OnRuntimeUpgrade;
		use frame_support::StorageHasher;
		use parity_scale_codec::Encode;

		assert_ok!(AssetManager::register_foreign_asset(
			RuntimeOrigin::root(),
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
		crate::migrations::UnitsWithAssetType::<Test>::on_runtime_upgrade();

		// After migration, units per second should be indexed by AssetType
		assert_eq!(
			AssetManager::asset_type_units_per_second(MockAssetType::MockAsset(1)).unwrap(),
			200
		);

		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdType";

		// Assert that old storage is empty
		assert!(storage_key_iter::<mock::AssetId, u128, Blake2_128Concat>(
			pallet_prefix,
			storage_item_prefix
		)
		.next()
		.is_none());
	});
}

#[test]
fn test_asset_manager_populate_asset_type_id_storage_migration_works() {
	ExtBuilder::default().build().execute_with(|| {
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
		crate::migrations::PopulateAssetTypeIdStorage::<Test>::on_runtime_upgrade();

		// After migration, the new storage item should be populated
		assert_eq!(
			AssetManager::asset_type_id(MockAssetType::MockAsset(1)).unwrap(),
			1
		);
	});
}

#[test]
fn test_asset_manager_change_statemine_prefixes() {
	ExtBuilder::default().build().execute_with(|| {
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdType";
		use frame_support::traits::OnRuntimeUpgrade;
		use frame_support::StorageHasher;
		use parity_scale_codec::Encode;

		let statemine_para_id = mock::StatemineParaIdInfo::get();
		let statemine_assets_pallet = mock::StatemineAssetsInstanceInfo::get();

		let statemine_multilocation = MockAssetType::Xcm(MultiLocation {
			parents: 1,
			interior: X2(Parachain(statemine_para_id), GeneralIndex(1)),
		});

		let statemine_multilocation_2 = MockAssetType::Xcm(MultiLocation {
			parents: 1,
			interior: X2(Parachain(statemine_para_id), GeneralIndex(2)),
		});

		let statemine_multilocation_3 = MockAssetType::Xcm(MultiLocation {
			parents: 1,
			interior: X2(Parachain(statemine_para_id), GeneralIndex(3)),
		});

		let asset_id: mock::AssetId = statemine_multilocation.clone().into();

		// We are gonna test three cases:
		// Case 1: AssetManagerPopulateAssetTypeIdStorage has not executed yet
		// (only AssetIdType is populated)
		// Case 2: AssetManagerPopulateAssetTypeIdStorage has already executed
		// Case 3: AssetManagerUnitsWithAssetType has already executed

		// To mimic case 1, we populate AssetIdType manually but not AssetTypeId
		put_storage_value(
			pallet_prefix,
			storage_item_prefix,
			&Blake2_128Concat::hash(&asset_id.encode()),
			statemine_multilocation.clone(),
		);

		// Assert the storage item is well populated
		assert_eq!(
			AssetManager::asset_id_type(asset_id).unwrap(),
			statemine_multilocation
		);

		// To mimic case 2, we can simply register the asset through the extrinsic
		assert_ok!(AssetManager::register_foreign_asset(
			RuntimeOrigin::root(),
			statemine_multilocation_2.clone(),
			0u32.into(),
			1u32.into(),
			true
		));

		// To mimic case 3, we can simply register the asset through the extrinsic
		// But we also need to set units per second
		assert_ok!(AssetManager::register_foreign_asset(
			RuntimeOrigin::root(),
			statemine_multilocation_3.clone(),
			0u32.into(),
			1u32.into(),
			true
		));

		assert_ok!(AssetManager::set_asset_units_per_second(
			RuntimeOrigin::root(),
			statemine_multilocation_3.clone(),
			1u128,
			0
		));

		// We run the migration
		crate::migrations::ChangeStateminePrefixes::<
			Test,
			mock::StatemineParaIdInfo,
			mock::StatemineAssetsInstanceInfo,
		>::on_runtime_upgrade();

		// Check case 1
		let expected_statemine_multilocation = MockAssetType::Xcm(MultiLocation {
			parents: 1,
			interior: X3(
				Parachain(statemine_para_id),
				PalletInstance(statemine_assets_pallet),
				GeneralIndex(1),
			),
		});

		// After migration, the storage item should have been upgraded
		assert_eq!(
			AssetManager::asset_id_type(asset_id).unwrap(),
			expected_statemine_multilocation
		);

		// Check case 2
		let expected_statemine_multilocation_2 = MockAssetType::Xcm(MultiLocation {
			parents: 1,
			interior: X3(
				Parachain(statemine_para_id),
				PalletInstance(statemine_assets_pallet),
				GeneralIndex(2),
			),
		});

		let asset_id_2: mock::AssetId = statemine_multilocation_2.clone().into();

		// After migration, both storage items should have been upgraded
		assert_eq!(
			AssetManager::asset_id_type(asset_id_2).unwrap(),
			expected_statemine_multilocation_2
		);

		assert_eq!(
			AssetManager::asset_type_id(expected_statemine_multilocation_2).unwrap(),
			asset_id_2
		);

		// And the previous one should be cleaned
		assert!(AssetManager::asset_type_id(&statemine_multilocation_2).is_none());

		// Check case 3
		let expected_statemine_multilocation_3 = MockAssetType::Xcm(MultiLocation {
			parents: 1,
			interior: X3(
				Parachain(statemine_para_id),
				PalletInstance(statemine_assets_pallet),
				GeneralIndex(3),
			),
		});

		let asset_id_3: mock::AssetId = statemine_multilocation_3.clone().into();

		// After migration, both storage items should have been upgraded
		assert_eq!(
			AssetManager::asset_id_type(asset_id_3).unwrap(),
			expected_statemine_multilocation_3
		);

		assert_eq!(
			AssetManager::asset_type_id(&expected_statemine_multilocation_3).unwrap(),
			asset_id_3
		);

		// The previous one should be cleaned
		assert!(AssetManager::asset_type_id(&statemine_multilocation_3).is_none());

		// Units per second updated
		assert_eq!(
			AssetManager::asset_type_units_per_second(&expected_statemine_multilocation_3).unwrap(),
			1
		);
		assert!(AssetManager::asset_type_units_per_second(&statemine_multilocation_3).is_none());
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

		assert_ok!(AssetManager::set_asset_units_per_second(
			RuntimeOrigin::root(),
			MockAssetType::MockAsset(1),
			200u128.into(),
			0
		));

		assert_ok!(AssetManager::remove_existing_asset_type(
			RuntimeOrigin::root(),
			1,
			1
		));

		// Mappings are deleted
		assert!(AssetManager::asset_type_id(MockAssetType::MockAsset(1)).is_none());
		assert!(AssetManager::asset_id_type(1).is_none());

		// Units per second removed
		assert!(AssetManager::asset_type_units_per_second(MockAssetType::MockAsset(1)).is_none());

		expect_events(vec![
			crate::Event::ForeignAssetRegistered {
				asset_id: 1,
				asset: MockAssetType::MockAsset(1),
				metadata: 0,
			},
			crate::Event::UnitsPerSecondChanged {
				asset_type: MockAssetType::MockAsset(1),
				units_per_second: 200,
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

		// Units per second removed
		assert!(AssetManager::asset_type_units_per_second(MockAssetType::MockAsset(1)).is_none());

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

		// Units per second removed
		assert!(AssetManager::asset_type_units_per_second(MockAssetType::MockAsset(1)).is_none());

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

#[test]
fn test_destroy_local_asset_works() {
	ExtBuilder::default()
		.with_balances(vec![(1, 20)])
		.build()
		.execute_with(|| {
			let asset_id = MockLocalAssetIdCreator::create_asset_id_from_metadata(0);

			assert_ok!(AssetManager::register_local_asset(
				RuntimeOrigin::root(),
				1u64,
				1u64,
				true,
				0u32.into(),
			));
			assert_eq!(
				AssetManager::local_asset_deposit(asset_id),
				Some(AssetInfo {
					creator: 1,
					deposit: 1
				})
			);

			assert_ok!(AssetManager::destroy_local_asset(RuntimeOrigin::root(), 0,));

			assert_eq!(AssetManager::local_asset_counter(), 1);
			assert_eq!(AssetManager::local_asset_deposit(asset_id), None);
			expect_events(vec![
				crate::Event::LocalAssetRegistered {
					asset_id,
					creator: 1,
					owner: 1,
				},
				crate::Event::LocalAssetDestroyed { asset_id },
			]);
		});
}

#[test]
fn xcm_v2_to_v3_asset_id_type_storage_migration_works() {
	ExtBuilder::default().build().execute_with(|| {
		use frame_support::StorageHasher;
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdType";
		use frame_support::traits::OnRuntimeUpgrade;
		use parity_scale_codec::Encode;

		let statemine_para_id = mock::StatemineParaIdInfo::get();

		let old_multilocation = xcm::v2::MultiLocation {
			parents: 1,
			interior: xcm::v2::Junctions::X2(
				xcm::v2::Junction::Parachain(statemine_para_id),
				xcm::v2::Junction::GeneralIndex(1),
			),
		};
		let old_asset_type = crate::migrations::OldAssetType::Xcm(old_multilocation.clone());

		put_storage_value(
			pallet_prefix,
			storage_item_prefix,
			&Blake2_128Concat::hash(&1u32.encode()),
			old_asset_type,
		);

		crate::migrations::XcmV2ToV3AssetManager::<Test>::on_runtime_upgrade();

		let new_expected_multilocation: MultiLocation = old_multilocation
			.try_into()
			.expect("convert xcm v2 into v3");
		let new_actual_multilocation: MockAssetType =
			AssetManager::asset_id_type(1).expect("migrated to xcm v3");
		assert!(new_actual_multilocation == MockAssetType::Xcm(new_expected_multilocation));
	});
}

#[test]
fn xcm_v2_to_v3_asset_type_id_storage_migration_works() {
	ExtBuilder::default().build().execute_with(|| {
		use frame_support::StorageHasher;
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetTypeId";
		use frame_support::traits::OnRuntimeUpgrade;
		use parity_scale_codec::Encode;

		let statemine_para_id = mock::StatemineParaIdInfo::get();

		let old_multilocation = xcm::v2::MultiLocation {
			parents: 1,
			interior: xcm::v2::Junctions::X2(
				xcm::v2::Junction::Parachain(statemine_para_id),
				xcm::v2::Junction::GeneralIndex(1),
			),
		};
		let old_asset_type = crate::migrations::OldAssetType::Xcm(old_multilocation.clone());

		put_storage_value(
			pallet_prefix,
			storage_item_prefix,
			&Blake2_128Concat::hash(&old_asset_type.encode()),
			1u32,
		);

		crate::migrations::XcmV2ToV3AssetManager::<Test>::on_runtime_upgrade();

		let new_expected_multilocation: MultiLocation = old_multilocation
			.try_into()
			.expect("convert xcm v2 into v3");
		let new_value: u32 =
			AssetManager::asset_type_id(MockAssetType::Xcm(new_expected_multilocation))
				.expect("migrated to xcm v3");
		assert!(new_value == 1);
	});
}

#[test]
fn xcm_v2_to_v3_units_per_second_storage_migration_works() {
	ExtBuilder::default().build().execute_with(|| {
		use frame_support::StorageHasher;
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetTypeUnitsPerSecond";
		use frame_support::traits::OnRuntimeUpgrade;
		use parity_scale_codec::Encode;

		let statemine_para_id = mock::StatemineParaIdInfo::get();

		let old_multilocation = xcm::v2::MultiLocation {
			parents: 1,
			interior: xcm::v2::Junctions::X2(
				xcm::v2::Junction::Parachain(statemine_para_id),
				xcm::v2::Junction::GeneralIndex(1),
			),
		};
		let old_asset_type = crate::migrations::OldAssetType::Xcm(old_multilocation.clone());

		put_storage_value(
			pallet_prefix,
			storage_item_prefix,
			&Blake2_128Concat::hash(&old_asset_type.encode()),
			1u128,
		);

		crate::migrations::XcmV2ToV3AssetManager::<Test>::on_runtime_upgrade();

		let new_expected_multilocation: MultiLocation = old_multilocation
			.try_into()
			.expect("convert xcm v2 into v3");
		let new_value: u128 = AssetManager::asset_type_units_per_second(MockAssetType::Xcm(
			new_expected_multilocation,
		))
		.expect("migrated to xcm v3");
		assert!(new_value == 1);
	});
}

#[test]
fn xcm_v2_to_v3_supported_assets_storage_migration_works() {
	ExtBuilder::default().build().execute_with(|| {
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"SupportedFeePaymentAssets";
		use frame_support::traits::OnRuntimeUpgrade;

		let statemine_para_id = mock::StatemineParaIdInfo::get();

		let old_multilocation = xcm::v2::MultiLocation {
			parents: 1,
			interior: xcm::v2::Junctions::X2(
				xcm::v2::Junction::Parachain(statemine_para_id),
				xcm::v2::Junction::GeneralIndex(1),
			),
		};
		let old_asset_types = vec![crate::migrations::OldAssetType::Xcm(
			old_multilocation.clone(),
		)];

		put_storage_value(pallet_prefix, storage_item_prefix, &[], old_asset_types);
		crate::migrations::XcmV2ToV3AssetManager::<Test>::on_runtime_upgrade();

		let new_expected_value: MultiLocation = old_multilocation
			.try_into()
			.expect("convert xcm v2 into v3");
		let new_value = AssetManager::supported_fee_payment_assets();
		assert!(new_value == vec![MockAssetType::Xcm(new_expected_value)]);
	});
}
