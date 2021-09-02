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

use frame_support::{
	assert_noop, assert_ok, dispatch::DispatchError, parameter_types, traits::Filter, RuntimeDebug,
};

#[test]
fn registering_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(AssetManager::xcm_asset_register(
			Origin::root(),
			MockAssetType::MockAsset(1),
			1u32.into()
		));

		assert_eq!(
			AssetManager::asset_id_to_type(1).unwrap(),
			MockAssetType::MockAsset(1)
		);
	});
}

#[test]
fn unregistering_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(AssetManager::xcm_asset_register(
			Origin::root(),
			MockAssetType::MockAsset(1),
			1u32.into()
		));

		assert_eq!(
			AssetManager::asset_id_to_type(1).unwrap(),
			MockAssetType::MockAsset(1)
		);

		assert_ok!(AssetManager::xcm_asset_destroy(Origin::root(), 1));
		assert!(AssetManager::asset_id_to_type(1).is_none());
	});
}

#[test]
fn test_asset_exists_error() {
	new_test_ext().execute_with(|| {
		assert_ok!(AssetManager::xcm_asset_register(
			Origin::root(),
			MockAssetType::MockAsset(1),
			1u32.into()
		));

		assert_eq!(
			AssetManager::asset_id_to_type(1).unwrap(),
			MockAssetType::MockAsset(1)
		);
		assert_noop!(
			AssetManager::xcm_asset_register(
				Origin::root(),
				MockAssetType::MockAsset(1),
				1u32.into()
			),
			Error::<Test>::AssetAlreadyExists
		);
	});
}

#[test]
fn test_asset_does_not_exist_error() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			AssetManager::xcm_asset_destroy(Origin::root(), 1),
			Error::<Test>::AssetDoestNotExist
		);
	});
}
