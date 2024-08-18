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
use {
	crate::mock::*,
	crate::Error,
	frame_support::{assert_noop, assert_ok},
	sp_runtime::DispatchError,
	xcm::latest::{Asset, Error as XcmError, Junction, Location, Result as XcmResult},
};

#[test]
fn test_add_supported_asset() {
	new_test_ext().execute_with(|| {
		// Call with bad origin
		assert_noop!(
			XcmWeightTrader::add_asset(
				RuntimeOrigin::signed(EditAccount::get()),
				Location::parent(),
				1_000,
			),
			DispatchError::BadOrigin
		);

		// Call with invalid location
		assert_noop!(
			XcmWeightTrader::add_asset(
				RuntimeOrigin::signed(AddAccount::get()),
				Location::new(2, []),
				1_000,
			),
			Error::<Test>::XcmLocationFiltered
		);

		// Call with invalid price
		assert_noop!(
			XcmWeightTrader::add_asset(
				RuntimeOrigin::signed(AddAccount::get()),
				Location::parent(),
				0,
			),
			Error::<Test>::UnitsCannotBeZero
		);

		// Call with the right origin
		assert_ok!(XcmWeightTrader::add_asset(
			RuntimeOrigin::signed(AddAccount::get()),
			Location::parent(),
			1_000,
		));

		// The account should be supported
		assert_eq!(
			XcmWeightTrader::get_asset_units_for_one_billion_native(&Location::parent()),
			Some(1_000),
		);

		// Check storage
		assert_eq!(
			crate::pallet::SupportedAssets::<Test>::get(&Location::parent()),
			Some((true, 1_000))
		);

		// Try to add the same asset twice (should fail)
		assert_noop!(
			XcmWeightTrader::add_asset(
				RuntimeOrigin::signed(AddAccount::get()),
				Location::parent(),
				1_000,
			),
			Error::<Test>::AssetAlreadyAdded
		);
	})
}

#[test]
fn test_edit_supported_asset() {
	new_test_ext().execute_with(|| {
		// Should not be able to edit an asset not added yet
		assert_noop!(
			XcmWeightTrader::edit_asset(
				RuntimeOrigin::signed(EditAccount::get()),
				Location::parent(),
				2_000,
			),
			Error::<Test>::AssetNotFound
		);

		// Setup (add a supported asset)
		assert_ok!(XcmWeightTrader::add_asset(
			RuntimeOrigin::signed(AddAccount::get()),
			Location::parent(),
			1_000,
		));
		assert_eq!(
			XcmWeightTrader::get_asset_units_for_one_billion_native(&Location::parent()),
			Some(1_000),
		);

		// Call with bad origin
		assert_noop!(
			XcmWeightTrader::edit_asset(
				RuntimeOrigin::signed(AddAccount::get()),
				Location::parent(),
				2_000,
			),
			DispatchError::BadOrigin
		);

		// Call with invalid price
		assert_noop!(
			XcmWeightTrader::edit_asset(
				RuntimeOrigin::signed(EditAccount::get()),
				Location::parent(),
				0,
			),
			Error::<Test>::UnitsCannotBeZero
		);

		// Call with right origin and valid params
		assert_ok!(XcmWeightTrader::edit_asset(
			RuntimeOrigin::signed(EditAccount::get()),
			Location::parent(),
			2_000,
		),);

		// The account should be supported
		assert_eq!(
			XcmWeightTrader::get_asset_units_for_one_billion_native(&Location::parent()),
			Some(2_000),
		);

		// Check storage
		assert_eq!(
			crate::pallet::SupportedAssets::<Test>::get(&Location::parent()),
			Some((true, 2_000))
		);
	})
}

#[test]
fn test_pause_asset_support() {
	new_test_ext().execute_with(|| {
		// Should not be able to pause an asset not added yet
		assert_noop!(
			XcmWeightTrader::pause_asset_support(
				RuntimeOrigin::signed(PauseAccount::get()),
				Location::parent(),
			),
			Error::<Test>::AssetNotFound
		);

		// Setup (add a supported asset)
		assert_ok!(XcmWeightTrader::add_asset(
			RuntimeOrigin::signed(AddAccount::get()),
			Location::parent(),
			1_000,
		));
		assert_eq!(
			XcmWeightTrader::get_asset_units_for_one_billion_native(&Location::parent()),
			Some(1_000),
		);

		// Call with bad origin
		assert_noop!(
			XcmWeightTrader::pause_asset_support(
				RuntimeOrigin::signed(AddAccount::get()),
				Location::parent(),
			),
			DispatchError::BadOrigin
		);

		// Call with right origin and valid params
		assert_ok!(XcmWeightTrader::pause_asset_support(
			RuntimeOrigin::signed(PauseAccount::get()),
			Location::parent(),
		));

		// The account should be paused
		assert_eq!(
			XcmWeightTrader::get_asset_units_for_one_billion_native(&Location::parent()),
			None,
		);

		// Check storage
		assert_eq!(
			crate::pallet::SupportedAssets::<Test>::get(&Location::parent()),
			Some((false, 1_000))
		);

		// Should not be able to pause an asset already paused
		assert_noop!(
			XcmWeightTrader::pause_asset_support(
				RuntimeOrigin::signed(PauseAccount::get()),
				Location::parent(),
			),
			Error::<Test>::AssetAlreadyPaused
		);
	})
}

#[test]
fn test_resume_asset_support() {
	new_test_ext().execute_with(|| {
		// Setup (add a supported asset and pause it)
		assert_ok!(XcmWeightTrader::add_asset(
			RuntimeOrigin::signed(AddAccount::get()),
			Location::parent(),
			1_000,
		));
		assert_ok!(XcmWeightTrader::pause_asset_support(
			RuntimeOrigin::signed(PauseAccount::get()),
			Location::parent(),
		));
		assert_eq!(
			crate::pallet::SupportedAssets::<Test>::get(&Location::parent()),
			Some((false, 1_000))
		);

		// Call with bad origin
		assert_noop!(
			XcmWeightTrader::resume_asset_support(
				RuntimeOrigin::signed(AddAccount::get()),
				Location::parent(),
			),
			DispatchError::BadOrigin
		);

		// Call with invalid location
		assert_noop!(
			XcmWeightTrader::resume_asset_support(
				RuntimeOrigin::signed(ResumeAccount::get()),
				Location::new(2, []),
			),
			Error::<Test>::AssetNotFound
		);

		// Call with right origin and valid params
		assert_ok!(XcmWeightTrader::resume_asset_support(
			RuntimeOrigin::signed(ResumeAccount::get()),
			Location::parent(),
		));

		// The asset should be supported again
		assert_eq!(
			XcmWeightTrader::get_asset_units_for_one_billion_native(&Location::parent()),
			Some(1_000),
		);

		// Check storage
		assert_eq!(
			crate::pallet::SupportedAssets::<Test>::get(&Location::parent()),
			Some((true, 1_000))
		);

		// Should not be able to resume an asset already active
		assert_noop!(
			XcmWeightTrader::resume_asset_support(
				RuntimeOrigin::signed(ResumeAccount::get()),
				Location::parent(),
			),
			Error::<Test>::AssetNotPaused
		);
	})
}

#[test]
fn test_remove_asset_support() {
	new_test_ext().execute_with(|| {
		// Should not be able to remove an asset not added yet
		assert_noop!(
			XcmWeightTrader::remove_asset(
				RuntimeOrigin::signed(RemoveAccount::get()),
				Location::parent(),
			),
			Error::<Test>::AssetNotFound
		);

		// Setup (add a supported asset)
		assert_ok!(XcmWeightTrader::add_asset(
			RuntimeOrigin::signed(AddAccount::get()),
			Location::parent(),
			1_000,
		));
		assert_eq!(
			XcmWeightTrader::get_asset_units_for_one_billion_native(&Location::parent()),
			Some(1_000),
		);

		// Call with bad origin
		assert_noop!(
			XcmWeightTrader::remove_asset(
				RuntimeOrigin::signed(AddAccount::get()),
				Location::parent(),
			),
			DispatchError::BadOrigin
		);

		// Call with right origin and valid params
		assert_ok!(XcmWeightTrader::remove_asset(
			RuntimeOrigin::signed(RemoveAccount::get()),
			Location::parent(),
		));

		// The account should be removed
		assert_eq!(
			XcmWeightTrader::get_asset_units_for_one_billion_native(&Location::parent()),
			None,
		);

		// Check storage
		assert_eq!(
			crate::pallet::SupportedAssets::<Test>::get(&Location::parent()),
			None
		);

		// Should not be able to pause an asset already removed
		assert_noop!(
			XcmWeightTrader::remove_asset(
				RuntimeOrigin::signed(RemoveAccount::get()),
				Location::parent(),
			),
			Error::<Test>::AssetNotFound
		);
	})
}
