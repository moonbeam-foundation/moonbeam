// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Tests for AssetManager Pallet
use crate::*;
use mock::*;

use frame_support::{assert_noop, assert_ok};

#[test]
fn registering_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(AssetManager::asset_register(
			Origin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			100u128.into()
		));

		assert_eq!(
			AssetManager::asset_id_info(1).unwrap().asset_type,
			MockAssetType::MockAsset(1)
		);
		assert_eq!(
			AssetManager::asset_id_info(1).unwrap().units_per_second,
			100u128
		);
		expect_events(vec![crate::Event::AssetRegistered(
			1,
			MockAssetType::MockAsset(1),
			0u32,
			100,
		)])
	});
}

#[test]
fn test_asset_exists_error() {
	new_test_ext().execute_with(|| {
		assert_ok!(AssetManager::asset_register(
			Origin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			0u128.into()
		));

		assert_eq!(
			AssetManager::asset_id_info(1).unwrap().asset_type,
			MockAssetType::MockAsset(1)
		);
		assert_noop!(
			AssetManager::asset_register(
				Origin::root(),
				MockAssetType::MockAsset(1),
				0u32.into(),
				1u32.into(),
				0u128.into()
			),
			Error::<Test>::AssetAlreadyExists
		);
	});
}

#[test]
fn test_root_can_change_units_per_second() {
	new_test_ext().execute_with(|| {
		assert_ok!(AssetManager::asset_register(
			Origin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
			0u128.into()
		));

		assert_eq!(AssetManager::asset_id_info(1).unwrap().units_per_second, 0);
		assert_ok!(AssetManager::asset_change_units_per_second(
			Origin::root(),
			1,
			200u128.into()
		));

		assert_eq!(
			AssetManager::asset_id_info(1).unwrap().units_per_second,
			200
		);

		expect_events(vec![
			crate::Event::AssetRegistered(1, MockAssetType::MockAsset(1), 0, 0),
			crate::Event::UnitsPerSecondChaned(1, 200),
		])
	});
}

#[test]
fn test_asset_id_non_existent_error() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			AssetManager::asset_change_units_per_second(Origin::root(), 1, 200u128.into()),
			Error::<Test>::AssetDoesNotExist
		);
	});
}
