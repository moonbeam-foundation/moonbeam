// Copyright Moonsong Labs
// This file is part of Moonkit.

// Moonkit is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonkit is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonkit.  If not, see <http://www.gnu.org/licenses/>.
use crate::*;
use mock::*;

use frame_support::{assert_noop, assert_ok};
use xcm::latest::prelude::*;

#[test]
fn creating_foreign_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ForeignAssetCreator::create_foreign_asset(
			RuntimeOrigin::root(),
			Location::parent(),
			1u32.into(),
			1u32.into(),
			true,
			1u64,
		));

		assert_eq!(
			ForeignAssetCreator::foreign_asset_for_id(1).unwrap(),
			Location::parent()
		);
		assert_eq!(
			ForeignAssetCreator::asset_id_for_foreign(Location::parent()).unwrap(),
			1
		);
		expect_events(vec![crate::Event::ForeignAssetCreated {
			asset_id: 1,
			foreign_asset: Location::parent(),
		}]);

		let (foreign_asset, asset_id, min_balance): (Location, u32, u64) =
			get_asset_created_hook_invocation()
				.expect("Decoding of invocation data should not fail");
		assert_eq!(foreign_asset, Location::parent());
		assert_eq!(asset_id, 1u32);
		assert_eq!(min_balance, 1u64);
	});
}

#[test]
fn test_asset_exists_error() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ForeignAssetCreator::create_foreign_asset(
			RuntimeOrigin::root(),
			Location::parent(),
			1u32.into(),
			1u32.into(),
			true,
			1u64,
		));
		assert_eq!(
			ForeignAssetCreator::foreign_asset_for_id(1).unwrap(),
			Location::parent()
		);
		assert_noop!(
			ForeignAssetCreator::create_foreign_asset(
				RuntimeOrigin::root(),
				Location::parent(),
				1u32.into(),
				1u32.into(),
				true,
				1u64,
			),
			Error::<Test>::AssetAlreadyExists
		);
	});
}

#[test]
fn test_regular_user_cannot_call_extrinsics() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ForeignAssetCreator::create_foreign_asset(
				RuntimeOrigin::signed(1),
				Location::parent(),
				1u32.into(),
				1u32.into(),
				true,
				1u64,
			),
			sp_runtime::DispatchError::BadOrigin
		);

		assert_noop!(
			ForeignAssetCreator::change_existing_asset_type(
				RuntimeOrigin::signed(1),
				1,
				Location::parent()
			),
			sp_runtime::DispatchError::BadOrigin
		);
	});
}

#[test]
fn test_root_can_change_foreign_asset_for_asset_id() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ForeignAssetCreator::create_foreign_asset(
			RuntimeOrigin::root(),
			Location::parent(),
			1u32.into(),
			1u32.into(),
			true,
			1u64,
		));

		assert_ok!(ForeignAssetCreator::change_existing_asset_type(
			RuntimeOrigin::root(),
			1,
			Location::here()
		));

		// New associations are stablished
		assert_eq!(
			ForeignAssetCreator::foreign_asset_for_id(1).unwrap(),
			Location::here()
		);
		assert_eq!(
			ForeignAssetCreator::asset_id_for_foreign(Location::here()).unwrap(),
			1
		);

		// Old ones are deleted
		assert!(ForeignAssetCreator::asset_id_for_foreign(Location::parent()).is_none());

		expect_events(vec![
			crate::Event::ForeignAssetCreated {
				asset_id: 1,
				foreign_asset: Location::parent(),
			},
			crate::Event::ForeignAssetTypeChanged {
				asset_id: 1,
				new_foreign_asset: Location::here(),
			},
		])
	});
}

#[test]
fn test_asset_id_non_existent_error() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			ForeignAssetCreator::change_existing_asset_type(
				RuntimeOrigin::root(),
				1,
				Location::parent()
			),
			Error::<Test>::AssetDoesNotExist
		);
	});
}

#[test]
fn test_root_can_remove_asset_association() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ForeignAssetCreator::create_foreign_asset(
			RuntimeOrigin::root(),
			Location::parent(),
			1u32.into(),
			1u32.into(),
			true,
			1u64,
		));

		assert_ok!(ForeignAssetCreator::remove_existing_asset_type(
			RuntimeOrigin::root(),
			1
		));

		// Mappings are deleted
		assert!(ForeignAssetCreator::foreign_asset_for_id(1).is_none());
		assert!(ForeignAssetCreator::asset_id_for_foreign(Location::parent()).is_none());

		expect_events(vec![
			crate::Event::ForeignAssetCreated {
				asset_id: 1,
				foreign_asset: Location::parent(),
			},
			crate::Event::ForeignAssetRemoved {
				asset_id: 1,
				foreign_asset: Location::parent(),
			},
		])
	});
}

#[test]
fn test_destroy_foreign_asset_also_removes_everything() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ForeignAssetCreator::create_foreign_asset(
			RuntimeOrigin::root(),
			Location::parent(),
			1u32.into(),
			1u32.into(),
			true,
			1u64,
		));

		let (foreign_asset, asset_id, min_balance): (Location, u32, u64) =
			get_asset_created_hook_invocation()
				.expect("Decoding of invocation data should not fail");
		assert_eq!(foreign_asset, Location::parent());
		assert_eq!(asset_id, 1u32);
		assert_eq!(min_balance, 1u64);

		assert_ok!(ForeignAssetCreator::destroy_foreign_asset(
			RuntimeOrigin::root(),
			1
		));

		// Mappings are deleted
		assert!(ForeignAssetCreator::asset_id_for_foreign(Location::parent()).is_none());
		assert!(ForeignAssetCreator::foreign_asset_for_id(1).is_none());

		expect_events(vec![
			crate::Event::ForeignAssetCreated {
				asset_id: 1,
				foreign_asset: Location::parent(),
			},
			crate::Event::ForeignAssetDestroyed {
				asset_id: 1,
				foreign_asset: Location::parent(),
			},
		]);

		let (foreign_asset, asset_id): (Location, u32) = get_asset_destroyed_hook_invocation()
			.expect("Decoding of invocation data should not fail");
		assert_eq!(foreign_asset, Location::parent());
		assert_eq!(asset_id, 1u32);
	});
}
