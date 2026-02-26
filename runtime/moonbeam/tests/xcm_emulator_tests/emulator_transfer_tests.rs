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

//! Transfer tests using the xcm-emulator harness.
//!
//! Uses the real `westend_runtime` as relay and `moonbeam_runtime` as parachain.
//! Includes full end-to-end DMP transfer (relay → Moonbeam) with DOT registered
//! as a foreign ERC20 asset.
//!
//! Key workarounds (see ADR-001 for details):
//! - [`moonbeam_execute_with`] automatically patches mandatory inherent storage.
//! - `NotFirstBlock` is cleared after network init to skip VRF verification.
//! - Dummy `HeadData` is inserted in relay genesis for DMP routing.
//! - `transfer_assets_using_type_and_then` bypasses the AHM guard.

use crate::emulator_network::*;
use frame_support::assert_ok;
use xcm::latest::prelude::*;
use xcm_emulator::{RelayChain, TestExt};

const ONE_DOT: u128 = 10_000_000_000;

/// Ensure the emulator network initialises (triggers `Parachain::init` which
/// creates one block on Moonbeam, and also initialises the relay).
///
/// After init, we clear Moonbeam's `NotFirstBlock` storage so subsequent
/// blocks skip VRF verification.
fn init_and_prepare() {
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {});
	MoonbeamPara::<PolkadotMoonbeamNet>::ext_wrapper(|| {
		frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
			b"Randomness",
			b"NotFirstBlock",
		));
	});
}

/// Smoke test: relay and Moonbeam initialise and can execute closures.
#[test]
fn emulator_network_initializes() {
	init_and_prepare();

	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		let block = frame_system::Pallet::<westend_runtime::Runtime>::block_number();
		assert!(block >= 1, "Relay at block {block}");
	});

	moonbeam_execute_with(|| {
		let block = frame_system::Pallet::<moonbeam_runtime::Runtime>::block_number();
		assert!(block >= 1, "Moonbeam at block {block}");
	});
}

/// Verify sovereign accounts are correctly computed and funded.
#[test]
fn moonbeam_sovereign_is_funded_on_relay() {
	init_and_prepare();

	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		use frame_support::traits::fungible::Inspect;

		let sov = WestendRelay::<PolkadotMoonbeamNet>::sovereign_account_id_of_child_para(
			MOONBEAM_PARA_ID.into(),
		);
		let balance = <westend_runtime::Balances as Inspect<_>>::balance(&sov);
		assert!(
			balance > 0,
			"Moonbeam sovereign should be funded, got: {balance}"
		);
	});
}

/// Verify DOT transfer type is LocalReserve (relay IS the reserve for its
/// native token). This confirms XCM asset-transfer classification works.
#[test]
fn relay_native_token_is_local_reserve() {
	init_and_prepare();

	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		use xcm_executor::traits::XcmAssetTransfers;

		let dot = Asset {
			id: AssetId(Location::here()),
			fun: Fungible(ONE_DOT),
		};
		let dest = Location::new(0, [Parachain(MOONBEAM_PARA_ID)]);
		let transfer_type =
			xcm_executor::XcmExecutor::<westend_runtime::xcm_config::XcmConfig>::determine_for(
				&dot, &dest,
			);

		assert_eq!(
			transfer_type,
			Ok(xcm_executor::traits::TransferType::LocalReserve),
			"DOT transferred from relay to parachain should be LocalReserve"
		);
	});
}

/// End-to-end: send DOT from relay to Moonbeam and verify it arrives.
///
/// Flow:
/// 1. Register DOT as foreign asset on Moonbeam
/// 2. On relay: `transfer_assets_using_type_and_then` (bypasses AHM guard)
/// 3. Emulator routes DMP to Moonbeam
/// 4. On Moonbeam: verify beneficiary received the DOT
#[test]
fn transfer_dot_from_relay_to_moonbeam() {
	init_and_prepare();

	let beneficiary_key: [u8; 20] = [0x01; 20];
	let sender = sp_runtime::AccountId32::new([1u8; 32]);
	let dot_location = Location::parent();
	let dot_asset_id: u128 = 1;

	// Step 1: Register DOT as a foreign asset on Moonbeam.
	moonbeam_execute_with(|| {
		assert_ok!(moonbeam_runtime::EvmForeignAssets::create_foreign_asset(
			moonbeam_runtime::RuntimeOrigin::root(),
			dot_asset_id,
			dot_location.clone(),
			10, // decimals
			b"DOT".to_vec().try_into().unwrap(),
			b"Polkadot".to_vec().try_into().unwrap(),
		));

		// Register DOT in the XcmWeightTrader so fees can be paid.
		//
		// relative_price formula: dot_fee = native_fee_in_glmr * 10^18 / relative_price
		// GLMR has 18 decimals, DOT has 10. To make 10 DOT (= 10^11 units)
		// cover the XCM execution fee (~32 GLMR = ~32 * 10^18 units), we need:
		//   relative_price >= 32 * 10^18 * 10^18 / 10^11 ≈ 3.2 * 10^26
		// We use 10^28 to give comfortable headroom.
		assert_ok!(moonbeam_runtime::XcmWeightTrader::add_asset(
			moonbeam_runtime::RuntimeOrigin::root(),
			dot_location.clone(),
			10_000_000_000_000_000_000_000_000_000u128, // 10^28
		));
	});

	// Step 2: Send DOT from relay to Moonbeam.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		use frame_support::traits::fungible::Inspect;
		use xcm_executor::traits::TransferType;

		let balance_before = <westend_runtime::Balances as Inspect<_>>::balance(&sender);

		assert_ok!(
			westend_runtime::XcmPallet::transfer_assets_using_type_and_then(
				westend_runtime::RuntimeOrigin::signed(sender.clone()),
				Box::new(xcm::VersionedLocation::from(Location::new(
					0,
					[Parachain(MOONBEAM_PARA_ID)]
				))),
				Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
					id: AssetId(Location::here()),
					fun: Fungible(ONE_DOT * 10),
				}]))),
				Box::new(TransferType::LocalReserve),
				Box::new(xcm::VersionedAssetId::from(AssetId(Location::here()))),
				Box::new(TransferType::LocalReserve),
				Box::new(xcm::VersionedXcm::from(Xcm::<()>(vec![DepositAsset {
					assets: Wild(All),
					beneficiary: Location::new(
						0,
						[AccountKey20 {
							network: None,
							key: beneficiary_key,
						}],
					),
				},]))),
				WeightLimit::Unlimited,
			)
		);

		let balance_after = <westend_runtime::Balances as Inspect<_>>::balance(&sender);

		assert!(
			balance_after < balance_before,
			"Sender balance should decrease: before={balance_before}, after={balance_after}"
		);
	});

	// Step 3: Check DOT arrived on Moonbeam.
	moonbeam_execute_with(|| {
		let beneficiary = moonbeam_runtime::AccountId::from(beneficiary_key);
		let balance = moonbeam_runtime::EvmForeignAssets::balance(dot_asset_id, beneficiary)
			.expect("balance query should succeed");

		assert!(
			balance > sp_core::U256::zero(),
			"Beneficiary should have received DOT on Moonbeam, got balance: {balance}"
		);
	});
}
