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
	crate::{Error, Trader, XcmPaymentApiError},
	frame_support::pallet_prelude::Weight,
	frame_support::{assert_noop, assert_ok},
	sp_runtime::DispatchError,
	xcm::v4::{
		Asset, AssetId as XcmAssetId, Error as XcmError, Fungibility, Location, XcmContext, XcmHash,
	},
	xcm::{IntoVersion, VersionedAssetId},
	xcm_executor::traits::WeightTrader,
};

fn xcm_fees_account() -> <Test as frame_system::Config>::AccountId {
	<Test as crate::Config>::XcmFeesAccount::get()
}

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
			XcmWeightTrader::get_asset_relative_price(&Location::parent()),
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
			XcmWeightTrader::get_asset_relative_price(&Location::parent()),
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
			XcmWeightTrader::get_asset_relative_price(&Location::parent()),
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
			XcmWeightTrader::get_asset_relative_price(&Location::parent()),
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
			XcmWeightTrader::get_asset_relative_price(&Location::parent()),
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
			XcmWeightTrader::get_asset_relative_price(&Location::parent()),
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
			XcmWeightTrader::get_asset_relative_price(&Location::parent()),
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
			XcmWeightTrader::get_asset_relative_price(&Location::parent()),
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

#[test]
fn test_trader_native_asset() {
	new_test_ext().execute_with(|| {
		let weight_to_buy = Weight::from_parts(10_000, 0);
		let dummy_xcm_context = XcmContext::with_message_id(XcmHash::default());

		// Should not be able to buy weight with too low asset balance
		assert_eq!(
			Trader::<Test>::new().buy_weight(
				weight_to_buy,
				Asset {
					fun: Fungibility::Fungible(9_999),
					id: XcmAssetId(Location::here()),
				}
				.into(),
				&dummy_xcm_context
			),
			Err(XcmError::TooExpensive)
		);

		// Should not be able to buy weight with unsupported asset
		assert_eq!(
			Trader::<Test>::new().buy_weight(
				weight_to_buy,
				Asset {
					fun: Fungibility::Fungible(10_000),
					id: XcmAssetId(Location::parent()),
				}
				.into(),
				&dummy_xcm_context
			),
			Err(XcmError::AssetNotFound)
		);

		// Should not be able to buy weight without asset
		assert_eq!(
			Trader::<Test>::new().buy_weight(weight_to_buy, Default::default(), &dummy_xcm_context),
			Err(XcmError::AssetNotFound)
		);

		// Should be able to buy weight with just enough native asset
		let mut trader = Trader::<Test>::new();
		assert_eq!(
			trader.buy_weight(
				weight_to_buy,
				Asset {
					fun: Fungibility::Fungible(10_000),
					id: XcmAssetId(Location::here()),
				}
				.into(),
				&dummy_xcm_context
			),
			Ok(Default::default())
		);

		// Should not refund any funds
		let actual_weight = weight_to_buy;
		assert_eq!(
			trader.refund_weight(actual_weight, &dummy_xcm_context),
			None
		);

		// Should not be able to buy weight again with the same trader
		assert_eq!(
			trader.buy_weight(
				weight_to_buy,
				Asset {
					fun: Fungibility::Fungible(10_000),
					id: XcmAssetId(Location::here()),
				}
				.into(),
				&dummy_xcm_context
			),
			Err(XcmError::NotWithdrawable)
		);

		// Fees asset should be deposited into XcmFeesAccount
		drop(trader);
		assert_eq!(Balances::free_balance(&xcm_fees_account()), 10_000);

		// Should be able to buy weight with more native asset (and get back unused amount)
		let mut trader = Trader::<Test>::new();
		assert_eq!(
			trader.buy_weight(
				weight_to_buy,
				Asset {
					fun: Fungibility::Fungible(11_000),
					id: XcmAssetId(Location::here()),
				}
				.into(),
				&dummy_xcm_context
			),
			Ok(Asset {
				fun: Fungibility::Fungible(1_000),
				id: XcmAssetId(Location::here()),
			}
			.into())
		);

		// Should be able to refund unused weights
		let actual_weight = weight_to_buy.saturating_sub(Weight::from_parts(2_000, 0));
		assert_eq!(
			trader.refund_weight(actual_weight, &dummy_xcm_context),
			Some(Asset {
				fun: Fungibility::Fungible(2_000),
				id: XcmAssetId(Location::here()),
			})
		);

		// Fees asset should be deposited again into XcmFeesAccount (2 times cost minus one refund)
		drop(trader);
		assert_eq!(
			Balances::free_balance(&xcm_fees_account()),
			(2 * 10_000) - 2_000
		);
	})
}

#[test]
fn test_trader_parent_asset() {
	new_test_ext().execute_with(|| {
		let weight_to_buy = Weight::from_parts(10_000, 0);
		let dummy_xcm_context = XcmContext::with_message_id(XcmHash::default());

		// Setup (add a supported asset)
		assert_ok!(XcmWeightTrader::add_asset(
			RuntimeOrigin::signed(AddAccount::get()),
			Location::parent(),
			500_000_000,
		));
		assert_eq!(
			XcmWeightTrader::get_asset_relative_price(&Location::parent()),
			Some(500_000_000),
		);

		// Should be able to pay fees with registered asset
		let mut trader = Trader::<Test>::new();
		assert_eq!(
			trader.buy_weight(
				weight_to_buy,
				Asset {
					fun: Fungibility::Fungible(22_000),
					id: XcmAssetId(Location::parent()),
				}
				.into(),
				&dummy_xcm_context
			),
			Ok(Asset {
				fun: Fungibility::Fungible(2_000),
				id: XcmAssetId(Location::parent()),
			}
			.into())
		);

		// Should be able to refund unused weights
		let actual_weight = weight_to_buy.saturating_sub(Weight::from_parts(2_000, 0));
		assert_eq!(
			trader.refund_weight(actual_weight, &dummy_xcm_context),
			Some(Asset {
				fun: Fungibility::Fungible(4_000),
				id: XcmAssetId(Location::parent()),
			})
		);

		// Fees asset should be deposited into XcmFeesAccount
		drop(trader);
		assert_eq!(
			get_parent_asset_deposited(),
			Some((xcm_fees_account(), 20_000 - 4_000))
		);

		// Should not be able to buy weight if the asset is not a first position
		assert_eq!(
			Trader::<Test>::new().buy_weight(
				weight_to_buy,
				vec![
					Asset {
						fun: Fungibility::Fungible(10),
						id: XcmAssetId(Location::here()),
					},
					Asset {
						fun: Fungibility::Fungible(30_000),
						id: XcmAssetId(Location::parent()),
					}
				]
				.into(),
				&dummy_xcm_context
			),
			Err(XcmError::TooExpensive)
		);
	})
}

#[test]
fn test_query_acceptable_payment_assets() {
	new_test_ext().execute_with(|| {
		// By default, only native asset should be supported
		assert_eq!(
			XcmWeightTrader::query_acceptable_payment_assets(4),
			Ok(vec![VersionedAssetId::V4(XcmAssetId(
				<Test as crate::Config>::NativeLocation::get()
			))])
		);

		// We should support XCMv3
		assert_eq!(
			XcmWeightTrader::query_acceptable_payment_assets(3),
			Ok(vec![VersionedAssetId::V4(XcmAssetId(
				<Test as crate::Config>::NativeLocation::get()
			))
			.into_version(3)
			.expect("native location should be convertible to v3")])
		);

		// We should not support XCMv2
		assert_eq!(
			XcmWeightTrader::query_acceptable_payment_assets(2),
			Err(XcmPaymentApiError::UnhandledXcmVersion)
		);

		// Setup (add a supported asset)
		assert_ok!(XcmWeightTrader::add_asset(
			RuntimeOrigin::signed(AddAccount::get()),
			Location::parent(),
			500_000_000,
		));
		assert_eq!(
			XcmWeightTrader::get_asset_relative_price(&Location::parent()),
			Some(500_000_000),
		);

		// We should support parent asset now
		assert_eq!(
			XcmWeightTrader::query_acceptable_payment_assets(4),
			Ok(vec![
				VersionedAssetId::V4(XcmAssetId(<Test as crate::Config>::NativeLocation::get())),
				VersionedAssetId::V4(XcmAssetId(Location::parent()))
			])
		);

		// Setup: pause parent asset
		assert_ok!(XcmWeightTrader::pause_asset_support(
			RuntimeOrigin::signed(PauseAccount::get()),
			Location::parent(),
		));
		assert_eq!(
			XcmWeightTrader::get_asset_relative_price(&Location::parent()),
			None
		);

		// We should not support paused assets
		assert_eq!(
			XcmWeightTrader::query_acceptable_payment_assets(4),
			Ok(vec![VersionedAssetId::V4(XcmAssetId(
				<Test as crate::Config>::NativeLocation::get()
			)),])
		);
	})
}

#[test]
fn test_query_weight_to_asset_fee() {
	new_test_ext().execute_with(|| {
		let native_asset =
			VersionedAssetId::V4(XcmAssetId(<Test as crate::Config>::NativeLocation::get()));
		let parent_asset = VersionedAssetId::V4(XcmAssetId(Location::parent()));
		let weight_to_buy = Weight::from_parts(10_000, 0);

		// Native asset price should be 1:1
		assert_eq!(
			XcmWeightTrader::query_weight_to_asset_fee(weight_to_buy, native_asset.clone()),
			Ok(10_000)
		);

		// Should not be able to query fees for an unsupported asset
		assert_eq!(
			XcmWeightTrader::query_weight_to_asset_fee(weight_to_buy, parent_asset.clone()),
			Err(XcmPaymentApiError::AssetNotFound)
		);

		// Setup (add a supported asset)
		assert_ok!(XcmWeightTrader::add_asset(
			RuntimeOrigin::signed(AddAccount::get()),
			Location::parent(),
			500_000_000,
		));
		assert_eq!(
			XcmWeightTrader::get_asset_relative_price(&Location::parent()),
			Some(500_000_000),
		);

		// Parent asset price should be 0.5
		assert_eq!(
			XcmWeightTrader::query_weight_to_asset_fee(weight_to_buy, parent_asset.clone()),
			Ok(2 * 10_000)
		);

		// Setup: pause parent asset
		assert_ok!(XcmWeightTrader::pause_asset_support(
			RuntimeOrigin::signed(PauseAccount::get()),
			Location::parent(),
		));
		assert_eq!(
			XcmWeightTrader::get_asset_relative_price(&Location::parent()),
			None
		);

		// We should not support paused assets
		assert_eq!(
			XcmWeightTrader::query_weight_to_asset_fee(weight_to_buy, parent_asset.clone()),
			Err(XcmPaymentApiError::AssetNotFound)
		);

		// Setup: unpause parent asset and edit price
		assert_ok!(XcmWeightTrader::resume_asset_support(
			RuntimeOrigin::signed(ResumeAccount::get()),
			Location::parent(),
		));
		assert_ok!(XcmWeightTrader::edit_asset(
			RuntimeOrigin::signed(EditAccount::get()),
			Location::parent(),
			2_000_000_000,
		));
		assert_eq!(
			XcmWeightTrader::get_asset_relative_price(&Location::parent()),
			Some(2_000_000_000),
		);

		// We should support unpaused asset with new price
		assert_eq!(
			XcmWeightTrader::query_weight_to_asset_fee(weight_to_buy, parent_asset),
			Ok(10_000 / 2)
		);
	})
}
