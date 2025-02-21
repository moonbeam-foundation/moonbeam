// Copyright 2025 Moonbeam Foundation.
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

use crate::*;
use mock::*;

use frame_support::traits::Currency;
use frame_support::{assert_noop, assert_ok};
use precompile_utils::testing::Bob;
use xcm::latest::prelude::*;

fn encode_ticker(str_: &str) -> BoundedVec<u8, ConstU32<256>> {
	BoundedVec::try_from(str_.as_bytes().to_vec()).expect("too long")
}

fn encode_token_name(str_: &str) -> BoundedVec<u8, ConstU32<256>> {
	BoundedVec::try_from(str_.as_bytes().to_vec()).expect("too long")
}

#[test]
fn create_foreign_and_freeze_unfreeze_using_xcm() {
	ExtBuilder::default().build().execute_with(|| {
		let deposit = ForeignAssetCreationDeposit::get();

		Balances::make_free_balance_be(&PARA_A, deposit);

		let asset_location: Location = (Parent, Parachain(1), PalletInstance(13)).into();

		// create foreign asset
		assert_ok!(EvmForeignAssets::create_foreign_asset(
			RuntimeOrigin::signed(PARA_A),
			1,
			asset_location.clone(),
			18,
			encode_ticker("MTT"),
			encode_token_name("Mytoken"),
		));

		assert_eq!(
			EvmForeignAssets::assets_by_id(1),
			Some(asset_location.clone())
		);
		assert_eq!(
			EvmForeignAssets::assets_by_location(asset_location.clone()),
			Some((1, AssetStatus::Active)),
		);
		expect_events(vec![Event::ForeignAssetCreated {
			contract_address: H160([
				255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
			]),
			asset_id: 1,
			xcm_location: asset_location.clone(),
			deposit: Some(deposit),
		}]);

		let (xcm_location, asset_id): (Location, u128) = get_asset_created_hook_invocation()
			.expect("Decoding of invocation data should not fail");
		assert_eq!(xcm_location, asset_location.clone());
		assert_eq!(asset_id, 1u128);

		// Check storage
		assert_eq!(
			EvmForeignAssets::assets_by_id(&1),
			Some(asset_location.clone())
		);
		assert_eq!(
			EvmForeignAssets::assets_by_location(&asset_location),
			Some((1, AssetStatus::Active))
		);

		// Unfreeze should return AssetNotFrozen error
		assert_noop!(
			EvmForeignAssets::unfreeze_foreign_asset(RuntimeOrigin::signed(PARA_A), 1),
			Error::<Test>::AssetNotFrozen
		);

		// Freeze should work
		assert_ok!(EvmForeignAssets::freeze_foreign_asset(
			RuntimeOrigin::signed(PARA_A),
			1,
			true
		),);
		assert_eq!(
			EvmForeignAssets::assets_by_location(&asset_location),
			Some((1, AssetStatus::FrozenXcmDepositAllowed))
		);

		// Should not be able to freeze an asset already frozen
		assert_noop!(
			EvmForeignAssets::freeze_foreign_asset(RuntimeOrigin::signed(PARA_A), 1, true),
			Error::<Test>::AssetAlreadyFrozen
		);

		// Unfreeze should work
		assert_ok!(EvmForeignAssets::unfreeze_foreign_asset(
			RuntimeOrigin::signed(PARA_A),
			1
		),);
		assert_eq!(
			EvmForeignAssets::assets_by_location(&asset_location),
			Some((1, AssetStatus::Active))
		);
	});
}

#[test]
fn create_foreign_and_freeze_unfreeze_using_root() {
	ExtBuilder::default().build().execute_with(|| {
		// create foreign asset
		assert_ok!(EvmForeignAssets::create_foreign_asset(
			RuntimeOrigin::root(),
			1,
			Location::parent(),
			18,
			encode_ticker("MTT"),
			encode_token_name("Mytoken"),
		));

		assert_eq!(EvmForeignAssets::assets_by_id(1), Some(Location::parent()));
		assert_eq!(
			EvmForeignAssets::assets_by_location(Location::parent()),
			Some((1, AssetStatus::Active)),
		);
		expect_events(vec![crate::Event::ForeignAssetCreated {
			contract_address: H160([
				255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
			]),
			asset_id: 1,
			xcm_location: Location::parent(),
			deposit: None,
		}]);

		let (xcm_location, asset_id): (Location, u128) = get_asset_created_hook_invocation()
			.expect("Decoding of invocation data should not fail");
		assert_eq!(xcm_location, Location::parent());
		assert_eq!(asset_id, 1u128);

		// Check storage
		assert_eq!(EvmForeignAssets::assets_by_id(&1), Some(Location::parent()));
		assert_eq!(
			EvmForeignAssets::assets_by_location(&Location::parent()),
			Some((1, AssetStatus::Active))
		);

		// Unfreeze should return AssetNotFrozen error
		assert_noop!(
			EvmForeignAssets::unfreeze_foreign_asset(RuntimeOrigin::root(), 1),
			Error::<Test>::AssetNotFrozen
		);

		// Freeze should work
		assert_ok!(EvmForeignAssets::freeze_foreign_asset(
			RuntimeOrigin::root(),
			1,
			true
		),);
		assert_eq!(
			EvmForeignAssets::assets_by_location(&Location::parent()),
			Some((1, AssetStatus::FrozenXcmDepositAllowed))
		);

		// Should not be able to freeze an asset already frozen
		assert_noop!(
			EvmForeignAssets::freeze_foreign_asset(RuntimeOrigin::root(), 1, true),
			Error::<Test>::AssetAlreadyFrozen
		);

		// Unfreeze should work
		assert_ok!(EvmForeignAssets::unfreeze_foreign_asset(
			RuntimeOrigin::root(),
			1
		),);
		assert_eq!(
			EvmForeignAssets::assets_by_location(&Location::parent()),
			Some((1, AssetStatus::Active))
		);
	});
}

#[test]
fn test_asset_exists_error() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(EvmForeignAssets::create_foreign_asset(
			RuntimeOrigin::root(),
			1,
			Location::parent(),
			18,
			encode_ticker("MTT"),
			encode_token_name("Mytoken"),
		));
		assert_eq!(
			EvmForeignAssets::assets_by_id(1).unwrap(),
			Location::parent()
		);
		assert_noop!(
			EvmForeignAssets::create_foreign_asset(
				RuntimeOrigin::root(),
				1,
				Location::parent(),
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
				1,
				Location::parent(),
				18,
				encode_ticker("MTT"),
				encode_token_name("Mytoken"),
			),
			sp_runtime::DispatchError::BadOrigin
		);

		assert_noop!(
			EvmForeignAssets::change_xcm_location(
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
			1,
			Location::parent(),
			18,
			encode_ticker("MTT"),
			encode_token_name("Mytoken"),
		));

		assert_ok!(EvmForeignAssets::change_xcm_location(
			RuntimeOrigin::root(),
			1,
			Location::here()
		));

		// New associations are stablished
		assert_eq!(EvmForeignAssets::assets_by_id(1).unwrap(), Location::here());
		assert_eq!(
			EvmForeignAssets::assets_by_location(Location::here()).unwrap(),
			(1, AssetStatus::Active),
		);

		// Old ones are deleted
		assert!(EvmForeignAssets::assets_by_location(Location::parent()).is_none());

		expect_events(vec![
			crate::Event::ForeignAssetCreated {
				contract_address: H160([
					255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
				]),
				asset_id: 1,
				xcm_location: Location::parent(),
				deposit: None,
			},
			crate::Event::ForeignAssetXcmLocationChanged {
				asset_id: 1,
				previous_xcm_location: Location::parent(),
				new_xcm_location: Location::here(),
			},
		])
	});
}

#[test]
fn test_asset_id_non_existent_error() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			EvmForeignAssets::change_xcm_location(RuntimeOrigin::root(), 1, Location::parent()),
			Error::<Test>::AssetDoesNotExist
		);
	});
}

#[test]
fn test_location_already_exist_error() {
	ExtBuilder::default().build().execute_with(|| {
		// Setup: create a first foreign asset taht we will try to override
		assert_ok!(EvmForeignAssets::create_foreign_asset(
			RuntimeOrigin::root(),
			1,
			Location::parent(),
			18,
			encode_ticker("MTT"),
			encode_token_name("Mytoken"),
		));

		assert_noop!(
			EvmForeignAssets::create_foreign_asset(
				RuntimeOrigin::root(),
				2,
				Location::parent(),
				18,
				encode_ticker("MTT"),
				encode_token_name("Mytoken"),
			),
			Error::<Test>::LocationAlreadyExists
		);

		// Setup: create a second foreign asset that will try to override the first one
		assert_ok!(EvmForeignAssets::create_foreign_asset(
			RuntimeOrigin::root(),
			2,
			Location::new(2, *&[]),
			18,
			encode_ticker("MTT"),
			encode_token_name("Mytoken"),
		));

		assert_noop!(
			EvmForeignAssets::change_xcm_location(RuntimeOrigin::root(), 2, Location::parent()),
			Error::<Test>::LocationAlreadyExists
		);
	});
}

#[test]
fn test_governance_can_change_any_asset_location() {
	ExtBuilder::default().build().execute_with(|| {
		let deposit = ForeignAssetCreationDeposit::get();

		Balances::make_free_balance_be(&PARA_C, deposit + 10);

		let asset_location: Location = (Parent, Parachain(3), PalletInstance(22)).into();
		let asset_id = 5;

		// create foreign asset using para c
		assert_ok!(EvmForeignAssets::create_foreign_asset(
			RuntimeOrigin::signed(PARA_C),
			asset_id,
			asset_location.clone(),
			10,
			encode_ticker("PARC"),
			encode_token_name("Parachain C Token"),
		));

		assert_eq!(Balances::free_balance(&PARA_C), 10);

		assert_eq!(
			EvmForeignAssets::assets_by_id(asset_id),
			Some(asset_location.clone())
		);
		assert_eq!(
			EvmForeignAssets::assets_by_location(asset_location),
			Some((asset_id, AssetStatus::Active)),
		);

		// This asset doesn't belong to PARA A, so it should not be able to change the location
		assert_noop!(
			EvmForeignAssets::freeze_foreign_asset(RuntimeOrigin::signed(PARA_A), asset_id, true),
			Error::<Test>::LocationOutsideOfOrigin,
		);

		let new_asset_location: Location = (Parent, Parachain(1), PalletInstance(1)).into();

		// Also PARA A cannot change the location
		assert_noop!(
			EvmForeignAssets::change_xcm_location(
				RuntimeOrigin::signed(PARA_A),
				asset_id,
				new_asset_location.clone(),
			),
			Error::<Test>::LocationOutsideOfOrigin,
		);

		// Change location using root, now PARA A can control this asset
		assert_ok!(EvmForeignAssets::change_xcm_location(
			RuntimeOrigin::root(),
			asset_id,
			new_asset_location.clone(),
		));

		assert_eq!(
			EvmForeignAssets::assets_by_id(asset_id),
			Some(new_asset_location.clone())
		);
		assert_eq!(
			EvmForeignAssets::assets_by_location(new_asset_location),
			Some((asset_id, AssetStatus::Active)),
		);

		// Freeze will not work since this asset has been moved from PARA C to PARA A
		assert_noop!(
			EvmForeignAssets::freeze_foreign_asset(RuntimeOrigin::signed(PARA_C), asset_id, true),
			Error::<Test>::LocationOutsideOfOrigin,
		);

		// But if we try using PARA A, it should work
		assert_ok!(EvmForeignAssets::freeze_foreign_asset(
			RuntimeOrigin::signed(PARA_A),
			asset_id,
			true
		));
	});
}
