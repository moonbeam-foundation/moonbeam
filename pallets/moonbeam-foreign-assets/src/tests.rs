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
use precompile_utils::testing::{Bob, Charlie, MockAccount};
use xcm::latest::prelude::*;

fn encode_ticker(str_: &str) -> BoundedVec<u8, ConstU32<256>> {
	BoundedVec::try_from(str_.as_bytes().to_vec()).expect("too long")
}

fn encode_token_name(str_: &str) -> BoundedVec<u8, ConstU32<256>> {
	BoundedVec::try_from(str_.as_bytes().to_vec()).expect("too long")
}

#[test]
fn creating_foreign_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EvmForeignAssets::create_foreign_asset(
			RuntimeOrigin::root(),
			Location::parent(),
			1,
			18,
			encode_ticker("MTT"),
			encode_token_name("Mytoken"),
		));

		assert_eq!(
			EvmForeignAssets::foreign_asset_for_id(1).unwrap(),
			Location::parent()
		);
		assert_eq!(
			EvmForeignAssets::asset_id_for_foreign(Location::parent()).unwrap(),
			1
		);
		expect_events(vec![crate::Event::ForeignAssetCreated {
			asset_id: 1,
			foreign_asset: Location::parent(),
		}]);

		let (foreign_asset, asset_id): (Location, u128) = get_asset_created_hook_invocation()
			.expect("Decoding of invocation data should not fail");
		assert_eq!(foreign_asset, Location::parent());
		assert_eq!(asset_id, 1u128);
	});
}

#[test]
fn test_asset_exists_error() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EvmForeignAssets::create_foreign_asset(
			RuntimeOrigin::root(),
			Location::parent(),
			1,
			18,
			encode_ticker("MTT"),
			encode_token_name("Mytoken"),
		));
		assert_eq!(
			EvmForeignAssets::foreign_asset_for_id(1).unwrap(),
			Location::parent()
		);
		assert_noop!(
			EvmForeignAssets::create_foreign_asset(
				RuntimeOrigin::root(),
				Location::parent(),
				1,
				18,
				encode_ticker("MTT"),
				encode_token_name("Mytoken"),
			),
			Error::<Test>::AssetAlreadyExists
		);
	});
}

#[test]
fn test_regular_user_cannot_call_extrinsics() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EvmForeignAssets::create_foreign_asset(
				RuntimeOrigin::signed(Bob.into()),
				Location::parent(),
				1,
				18,
				encode_ticker("MTT"),
				encode_token_name("Mytoken"),
			),
			sp_runtime::DispatchError::BadOrigin
		);

		assert_noop!(
			EvmForeignAssets::change_existing_asset_type(
				RuntimeOrigin::signed(Bob.into()),
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
		assert_ok!(EvmForeignAssets::create_foreign_asset(
			RuntimeOrigin::root(),
			Location::parent(),
			1,
			18,
			encode_ticker("MTT"),
			encode_token_name("Mytoken"),
		));

		assert_ok!(EvmForeignAssets::change_existing_asset_type(
			RuntimeOrigin::root(),
			1,
			Location::here()
		));

		// New associations are stablished
		assert_eq!(
			EvmForeignAssets::foreign_asset_for_id(1).unwrap(),
			Location::here()
		);
		assert_eq!(
			EvmForeignAssets::asset_id_for_foreign(Location::here()).unwrap(),
			1
		);

		// Old ones are deleted
		assert!(EvmForeignAssets::asset_id_for_foreign(Location::parent()).is_none());

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
			EvmForeignAssets::change_existing_asset_type(
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
		assert_ok!(EvmForeignAssets::create_foreign_asset(
			RuntimeOrigin::root(),
			Location::parent(),
			1,
			18,
			encode_ticker("MTT"),
			encode_token_name("Mytoken"),
		));

		assert_ok!(EvmForeignAssets::remove_existing_asset_type(
			RuntimeOrigin::root(),
			1
		));

		// Mappings are deleted
		assert!(EvmForeignAssets::foreign_asset_for_id(1).is_none());
		assert!(EvmForeignAssets::asset_id_for_foreign(Location::parent()).is_none());

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