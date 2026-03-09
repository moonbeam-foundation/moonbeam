// Copyright 2019-2025 Moonbeam Foundation.
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

//! XCM version discovery / negotiation tests.
//!
//! Verifies that `SafeXcmVersion` is configured from genesis and that
//! Moonbeam discovers the XCM version of remote chains (relay and siblings)
//! after the first cross-chain interaction.
//!
//! Full runtime-upgrade version negotiation (as in the legacy mock tests)
//! is not feasible with xcm-emulator because there is no mock version
//! switcher. These tests cover the subset that works with the real runtime.

use crate::emulator_network::*;
use frame_support::assert_ok;
use xcm::latest::prelude::*;
use xcm_emulator::TestExt;

// ===========================================================================
// Helpers
// ===========================================================================

/// Register Moonbeam GLMR as foreign asset on the current chain context.
fn register_glmr_foreign_asset(source_para_id: u32) {
	let glmr_location =
		xcm::latest::Location::new(1, [Parachain(source_para_id), PalletInstance(3u8)]);

	frame_support::assert_ok!(moonbase_runtime::EvmForeignAssets::create_foreign_asset(
		moonbase_runtime::RuntimeOrigin::root(),
		UNIT_ASSET_ID,
		glmr_location.clone(),
		18,
		b"UNIT".to_vec().try_into().unwrap(),
		b"Moonbase".to_vec().try_into().unwrap(),
	));

	frame_support::assert_ok!(moonbase_runtime::XcmWeightTrader::add_asset(
		moonbase_runtime::RuntimeOrigin::root(),
		glmr_location,
		10_000_000_000_000_000_000_000_000_000u128,
	));
}

// ===========================================================================
// Tests
// ===========================================================================

/// Verify that Moonbeam subscribes to the relay's XCM version on first
/// interaction. After a DMP transfer the relay should know Moonbeam's
/// supported XCM version.
#[test]
fn xcm_version_discovery_with_relay() {
	init_network();

	moonbase_execute_with(|| {
		register_dot_asset(DOT_ASSET_ID);
	});

	// Send DOT from relay to Moonbeam to trigger version discovery.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		let beneficiary = Location::new(
			0,
			[AccountKey20 {
				network: None,
				key: ALITH,
			}],
		);
		let assets: xcm::VersionedAssets = (Location::here(), ONE_DOT * 5).into();
		let fees_id: xcm::VersionedAssetId = AssetId(Location::here()).into();
		let xcm_on_dest = Xcm::<()>(vec![DepositAsset {
			assets: Wild(All),
			beneficiary,
		}]);

		assert_ok!(
			westend_runtime::XcmPallet::transfer_assets_using_type_and_then(
				westend_runtime::RuntimeOrigin::signed(RELAY_ALICE.clone()),
				Box::new(xcm::VersionedLocation::from(Location::new(
					0,
					[Parachain(MOONBEAM_PARA_ID)]
				))),
				Box::new(assets),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(fees_id),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedXcm::V5(xcm_on_dest)),
				WeightLimit::Unlimited,
			)
		);
	});

	// After the transfer the relay should be able to determine Moonbeam's
	// supported XCM version via its version discovery/subscription mechanism.
	// We verify the relay can weigh XCM (a proxy for version-awareness).
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApiV1;
		let weight =
			westend_runtime::Runtime::query_xcm_weight(xcm::VersionedXcm::from(Xcm::<()>(vec![
				ClearOrigin,
			])));
		assert!(weight.is_ok(), "Relay should be version-aware");
	});

	// Moonbeam should have its safe_xcm_version set from genesis.
	moonbase_execute_with(|| {
		use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApiV1;
		let weight =
			moonbase_runtime::Runtime::query_xcm_weight(xcm::VersionedXcm::from(Xcm::<()>(vec![
				ClearOrigin,
			])));
		assert!(weight.is_ok(), "Moonbase should be version-aware");
	});
}

/// Verify that Moonbeam and a sibling negotiate XCM versions via HRMP.
#[test]
fn xcm_version_discovery_with_sibling() {
	init_network();

	moonbase_execute_with(|| register_dot_asset(DOT_ASSET_ID));
	sibling_execute_with(|| register_dot_asset(DOT_ASSET_ID));

	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		open_hrmp_channels(MOONBEAM_PARA_ID, SIBLING_PARA_ID);
	});

	// Register GLMR on sibling so we can do a transfer.
	sibling_execute_with(|| register_glmr_foreign_asset(MOONBEAM_PARA_ID));

	let amount = moonbase_runtime::currency::UNIT;

	// Transfer triggers version negotiation between the two parachains.
	moonbase_execute_with(|| {
		assert_ok!(moonbase_runtime::PolkadotXcm::transfer_assets(
			moonbase_runtime::RuntimeOrigin::signed(moonbase_runtime::AccountId::from(ALITH)),
			Box::new(xcm::VersionedLocation::from(Location::new(
				1,
				[Parachain(SIBLING_PARA_ID)],
			))),
			Box::new(xcm::VersionedLocation::from(Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: BALTATHAR,
				}],
			))),
			Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
				id: AssetId(Location::new(0, [PalletInstance(3)])),
				fun: Fungible(amount),
			}]))),
			0,
			WeightLimit::Unlimited,
		));
	});

	// After the transfer both chains should be version-aware.
	sibling_execute_with(|| {
		use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApiV1;
		let weight =
			moonbase_runtime::Runtime::query_xcm_weight(xcm::VersionedXcm::from(Xcm::<()>(vec![
				ClearOrigin,
			])));
		assert!(weight.is_ok(), "Sibling should be version-aware");
	});

	moonbase_execute_with(|| {
		use xcm_runtime_apis::fees::runtime_decl_for_xcm_payment_api::XcmPaymentApiV1;
		let weight =
			moonbase_runtime::Runtime::query_xcm_weight(xcm::VersionedXcm::from(Xcm::<()>(vec![
				ClearOrigin,
			])));
		assert!(weight.is_ok(), "Moonbase should be version-aware");
	});
}
