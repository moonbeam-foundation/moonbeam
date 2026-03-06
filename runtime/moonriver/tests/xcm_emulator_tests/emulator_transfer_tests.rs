// Copyright 2019-2025 Moonriver Foundation.
// This file is part of Moonriver.

// Moonriver is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonriver is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonriver.  If not, see <http://www.gnu.org/licenses/>.

//! Transfer tests using xcm-emulator with the **real** Moonriver runtime.
//!
//! Covers: relay→para, para→relay, para→para transfers, fee behaviour,
//! account sufficiency, and error cases.

use crate::emulator_network::*;
use frame_support::{
	assert_ok,
	traits::{fungible::Inspect, tokens::fungible::Mutate},
};
use sp_core::U256;
use xcm::latest::prelude::*;
use xcm_emulator::TestExt;

const DOT_ASSET_ID: u128 = 1;

// ===========================================================================
// Setup helper
// ===========================================================================

/// Full network init: register DOT on Moonriver, configure weight trader.
fn setup_relay_to_moonriver() {
	init_network();
	moonriver_execute_with(|| {
		register_dot_asset(DOT_ASSET_ID);
	});
}

/// Full network init with sibling: register DOT on both paras, open HRMP channels.
fn setup_with_sibling() {
	init_network();

	moonriver_execute_with(|| {
		register_dot_asset(DOT_ASSET_ID);
	});
	sibling_execute_with(|| {
		register_dot_asset(DOT_ASSET_ID);
	});

	// Open bi-directional HRMP channels between Moonriver (2004) and Sibling (2005).
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		open_hrmp_channels(MOONBEAM_PARA_ID, SIBLING_PARA_ID);
	});
}

// ===========================================================================
// Transfer: Relay → Moonriver (DMP)
// ===========================================================================

#[test]
fn transfer_dot_from_relay_to_moonriver() {
	setup_relay_to_moonriver();

	let sender = RELAY_ALICE;
	let beneficiary_key = ALITH;

	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
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
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedAssetId::from(AssetId(Location::here()))),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedXcm::from(Xcm::<()>(vec![DepositAsset {
					assets: Wild(All),
					beneficiary: Location::new(
						0,
						[AccountKey20 {
							network: None,
							key: beneficiary_key,
						}],
					),
				}]))),
				WeightLimit::Unlimited,
			)
		);

		let balance_after = <westend_runtime::Balances as Inspect<_>>::balance(&sender);
		assert!(
			balance_after < balance_before,
			"Sender balance should decrease"
		);
	});

	moonriver_execute_with(|| {
		let beneficiary = moonriver_runtime::AccountId::from(beneficiary_key);
		let balance = moonriver_runtime::EvmForeignAssets::balance(DOT_ASSET_ID, beneficiary)
			.expect("balance query should succeed");
		assert!(
			balance > U256::zero(),
			"Beneficiary should have DOT on Moonriver, got {balance}"
		);
	});
}

// ===========================================================================
// Transfer: Moonriver → Relay (UMP)
// ===========================================================================

#[test]
fn transfer_dot_from_moonriver_to_relay() {
	setup_relay_to_moonriver();

	// First: send DOT from relay to Moonriver so ALITH has some DOT.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(
			westend_runtime::XcmPallet::transfer_assets_using_type_and_then(
				westend_runtime::RuntimeOrigin::signed(RELAY_ALICE),
				Box::new(xcm::VersionedLocation::from(Location::new(
					0,
					[Parachain(MOONBEAM_PARA_ID)]
				))),
				Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
					id: AssetId(Location::here()),
					fun: Fungible(ONE_DOT * 100),
				}]))),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedAssetId::from(AssetId(Location::here()))),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedXcm::from(Xcm::<()>(vec![DepositAsset {
					assets: Wild(All),
					beneficiary: Location::new(
						0,
						[AccountKey20 {
							network: None,
							key: ALITH
						}],
					),
				}]))),
				WeightLimit::Unlimited,
			)
		);
	});

	let alith_dot_before = moonriver_execute_with(|| {
		moonriver_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonriver_runtime::AccountId::from(ALITH),
		)
		.unwrap()
	});
	assert!(alith_dot_before > U256::zero(), "ALITH should have DOT");

	// Record relay-side balance of a relay account before the return transfer.
	let relay_bob = sp_runtime::AccountId32::new([2u8; 32]);
	let relay_bob_before = WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		<westend_runtime::Balances as Inspect<_>>::balance(&relay_bob)
	});

	// Now send DOT back from Moonriver to relay via PolkadotXcm.
	// DOT's reserve is the relay, so we use DestinationReserve transfer type.
	moonriver_execute_with(|| {
		let dot_location = Location::parent();
		let dest = Location::parent();
		let beneficiary = Location::new(
			0,
			[AccountId32 {
				network: None,
				id: relay_bob.clone().into(),
			}],
		);
		let amount = ONE_DOT * 5;

		assert_ok!(
			moonriver_runtime::PolkadotXcm::transfer_assets_using_type_and_then(
				moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
				Box::new(xcm::VersionedLocation::from(dest)),
				Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
					id: AssetId(dot_location.clone()),
					fun: Fungible(amount),
				}]))),
				Box::new(xcm_executor::traits::TransferType::DestinationReserve),
				Box::new(xcm::VersionedAssetId::from(AssetId(dot_location))),
				Box::new(xcm_executor::traits::TransferType::DestinationReserve),
				Box::new(xcm::VersionedXcm::from(Xcm::<()>(vec![DepositAsset {
					assets: Wild(All),
					beneficiary,
				}]))),
				WeightLimit::Unlimited,
			)
		);
	});

	// Verify relay account received DOT (minus fees).
	let relay_bob_after = WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		<westend_runtime::Balances as Inspect<_>>::balance(&relay_bob)
	});
	assert!(
		relay_bob_after > relay_bob_before,
		"Relay Bob should have more DOT: before={relay_bob_before}, after={relay_bob_after}"
	);
}

// ===========================================================================
// Fee behaviour: insufficient fees
// ===========================================================================

#[test]
fn error_when_not_paying_enough_fees() {
	setup_relay_to_moonriver();

	// Send a tiny amount (1 unit) from relay — should fail to pay Moonriver execution fees.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(
			westend_runtime::XcmPallet::transfer_assets_using_type_and_then(
				westend_runtime::RuntimeOrigin::signed(RELAY_ALICE),
				Box::new(xcm::VersionedLocation::from(Location::new(
					0,
					[Parachain(MOONBEAM_PARA_ID)]
				))),
				Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
					id: AssetId(Location::here()),
					fun: Fungible(1), // way too little for fees
				}]))),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedAssetId::from(AssetId(Location::here()))),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedXcm::from(Xcm::<()>(vec![DepositAsset {
					assets: Wild(All),
					beneficiary: Location::new(
						0,
						[AccountKey20 {
							network: None,
							key: ALITH
						}],
					),
				}]))),
				WeightLimit::Unlimited,
			)
		);
	});

	// ALITH should NOT have received the token (execution failed).
	moonriver_execute_with(|| {
		let balance = moonriver_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonriver_runtime::AccountId::from(ALITH),
		)
		.unwrap();
		assert_eq!(
			balance,
			U256::zero(),
			"Should not receive DOT when fees are insufficient"
		);
	});
}

// ===========================================================================
// Fee behaviour: fees go to treasury
// ===========================================================================

#[test]
fn fees_collected_by_treasury() {
	setup_relay_to_moonriver();

	let treasury_dot_before = moonriver_execute_with(|| {
		let treasury = moonriver_runtime::Treasury::account_id();
		moonriver_runtime::EvmForeignAssets::balance(DOT_ASSET_ID, treasury).unwrap_or(U256::zero())
	});

	// Send DOT from relay to Moonriver (fees will be charged).
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(
			westend_runtime::XcmPallet::transfer_assets_using_type_and_then(
				westend_runtime::RuntimeOrigin::signed(RELAY_ALICE),
				Box::new(xcm::VersionedLocation::from(Location::new(
					0,
					[Parachain(MOONBEAM_PARA_ID)]
				))),
				Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
					id: AssetId(Location::here()),
					fun: Fungible(ONE_DOT * 10),
				}]))),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedAssetId::from(AssetId(Location::here()))),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedXcm::from(Xcm::<()>(vec![DepositAsset {
					assets: Wild(All),
					beneficiary: Location::new(
						0,
						[AccountKey20 {
							network: None,
							key: BALTATHAR
						}],
					),
				}]))),
				WeightLimit::Unlimited,
			)
		);
	});

	moonriver_execute_with(|| {
		let treasury = moonriver_runtime::Treasury::account_id();
		let treasury_dot_after =
			moonriver_runtime::EvmForeignAssets::balance(DOT_ASSET_ID, treasury)
				.unwrap_or(U256::zero());
		assert!(
			treasury_dot_after > treasury_dot_before,
			"Treasury should collect fees: before={treasury_dot_before}, after={treasury_dot_after}"
		);

		// And beneficiary should have gotten the rest (not the full amount).
		let beneficiary_balance = moonriver_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonriver_runtime::AccountId::from(BALTATHAR),
		)
		.unwrap();
		assert!(
			beneficiary_balance > U256::zero(),
			"Beneficiary received DOT"
		);
		assert!(
			beneficiary_balance < U256::from(ONE_DOT * 10),
			"Beneficiary received less than sent (fees deducted)"
		);
	});
}

// ===========================================================================
// Account sufficiency: non-existent account receives foreign asset
// ===========================================================================

#[test]
fn receive_asset_for_non_existent_account() {
	setup_relay_to_moonriver();

	let fresh_account: [u8; 20] = [42u8; 20];

	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(
			westend_runtime::XcmPallet::transfer_assets_using_type_and_then(
				westend_runtime::RuntimeOrigin::signed(RELAY_ALICE),
				Box::new(xcm::VersionedLocation::from(Location::new(
					0,
					[Parachain(MOONBEAM_PARA_ID)]
				))),
				Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
					id: AssetId(Location::here()),
					fun: Fungible(ONE_DOT * 10),
				}]))),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedAssetId::from(AssetId(Location::here()))),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedXcm::from(Xcm::<()>(vec![DepositAsset {
					assets: Wild(All),
					beneficiary: Location::new(
						0,
						[AccountKey20 {
							network: None,
							key: fresh_account,
						}],
					),
				}]))),
				WeightLimit::Unlimited,
			)
		);
	});

	moonriver_execute_with(|| {
		let balance = moonriver_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonriver_runtime::AccountId::from(fresh_account),
		)
		.unwrap();
		assert!(
			balance > U256::zero(),
			"Fresh (non-existent) account should receive DOT via XCM"
		);
	});
}

// ===========================================================================
// Transfer: Para → Para via relay (XCMP/HRMP)
// ===========================================================================

#[test]
fn transfer_dot_from_moonriver_to_sibling() {
	setup_with_sibling();

	// First fund Moonriver ALITH with DOT from relay.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(
			westend_runtime::XcmPallet::transfer_assets_using_type_and_then(
				westend_runtime::RuntimeOrigin::signed(RELAY_ALICE),
				Box::new(xcm::VersionedLocation::from(Location::new(
					0,
					[Parachain(MOONBEAM_PARA_ID)]
				))),
				Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
					id: AssetId(Location::here()),
					fun: Fungible(ONE_DOT * 100),
				}]))),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedAssetId::from(AssetId(Location::here()))),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedXcm::from(Xcm::<()>(vec![DepositAsset {
					assets: Wild(All),
					beneficiary: Location::new(
						0,
						[AccountKey20 {
							network: None,
							key: ALITH
						}],
					),
				}]))),
				WeightLimit::Unlimited,
			)
		);
	});

	// Verify ALITH got DOT on Moonriver.
	let alith_dot = moonriver_execute_with(|| {
		moonriver_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonriver_runtime::AccountId::from(ALITH),
		)
		.unwrap()
	});
	assert!(alith_dot > U256::zero(), "ALITH should have DOT");

	// Now send DOT from Moonriver to Sibling via reserve transfer through relay.
	// DOT's reserve is the relay (parent), so we use RemoteReserve.
	// The custom_xcm_on_dest must include BuyExecution since the sibling's
	// barrier requires paid execution.
	moonriver_execute_with(|| {
		let dest = Location::new(1, [Parachain(SIBLING_PARA_ID)]);
		let beneficiary = Location::new(
			0,
			[AccountKey20 {
				network: None,
				key: BALTATHAR,
			}],
		);
		let dot_location = Location::parent();
		// Send a large amount so enough survives relay fees for the sibling.
		let amount = ONE_DOT * 50;

		assert_ok!(
			moonriver_runtime::PolkadotXcm::transfer_assets_using_type_and_then(
				moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
				Box::new(xcm::VersionedLocation::from(dest)),
				Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
					id: AssetId(dot_location.clone()),
					fun: Fungible(amount),
				}]))),
				Box::new(xcm_executor::traits::TransferType::RemoteReserve(
					xcm::VersionedLocation::from(Location::parent())
				)),
				Box::new(xcm::VersionedAssetId::from(AssetId(dot_location.clone()))),
				Box::new(xcm_executor::traits::TransferType::RemoteReserve(
					xcm::VersionedLocation::from(Location::parent())
				)),
				Box::new(xcm::VersionedXcm::from(Xcm::<()>(vec![
					BuyExecution {
						// Use a small fee amount that will definitely be in holding
						// after the relay takes its share.
						fees: Asset {
							id: AssetId(dot_location),
							fun: Fungible(ONE_DOT / 10),
						},
						weight_limit: WeightLimit::Unlimited,
					},
					DepositAsset {
						assets: Wild(All),
						beneficiary,
					},
				]))),
				WeightLimit::Unlimited,
			)
		);
	});

	// Trigger message routing on the relay so the DMP is delivered to sibling.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {});

	// Verify BALTATHAR received DOT on the sibling.
	sibling_execute_with(|| {
		let balance = moonriver_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonriver_runtime::AccountId::from(BALTATHAR),
		)
		.unwrap();
		assert!(
			balance > U256::zero(),
			"BALTATHAR should have DOT on sibling, got {balance}"
		);
	});
}

// ===========================================================================
// EVM account with native balance receives foreign assets
// ===========================================================================

#[test]
fn evm_account_receives_foreign_asset() {
	setup_relay_to_moonriver();

	// ALITH has GLMR from genesis. Send DOT and verify both balances coexist.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(
			westend_runtime::XcmPallet::transfer_assets_using_type_and_then(
				westend_runtime::RuntimeOrigin::signed(RELAY_ALICE),
				Box::new(xcm::VersionedLocation::from(Location::new(
					0,
					[Parachain(MOONBEAM_PARA_ID)],
				))),
				Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
					id: AssetId(Location::here()),
					fun: Fungible(ONE_DOT * 10),
				}]))),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedAssetId::from(AssetId(Location::here()))),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedXcm::from(Xcm::<()>(vec![DepositAsset {
					assets: Wild(All),
					beneficiary: Location::new(
						0,
						[AccountKey20 { network: None, key: ALITH }],
					),
				}]))),
				WeightLimit::Unlimited,
			)
		);
	});

	moonriver_execute_with(|| {
		// ALITH should have both native GLMR and foreign DOT.
		let glmr = <moonriver_runtime::Balances as Inspect<_>>::balance(
			&moonriver_runtime::AccountId::from(ALITH),
		);
		assert!(glmr > 0, "ALITH should still have GLMR");

		let dot = moonriver_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonriver_runtime::AccountId::from(ALITH),
		)
		.unwrap();
		assert!(dot > U256::zero(), "ALITH should also have DOT");
	});
}

// ===========================================================================
// Foreign assets survive native balance drainage
// ===========================================================================

#[test]
fn foreign_assets_survive_native_balance_drain() {
	setup_relay_to_moonriver();

	let test_account: [u8; 20] = [77u8; 20];

	// Give the test account some GLMR.
	moonriver_execute_with(|| {
		<moonriver_runtime::Balances as Mutate<_>>::mint_into(
			&moonriver_runtime::AccountId::from(test_account),
			moonriver_runtime::currency::MOVR,
		)
		.expect("Should mint GLMR");
	});

	// Send DOT to the test account.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(
			westend_runtime::XcmPallet::transfer_assets_using_type_and_then(
				westend_runtime::RuntimeOrigin::signed(RELAY_ALICE),
				Box::new(xcm::VersionedLocation::from(Location::new(
					0,
					[Parachain(MOONBEAM_PARA_ID)],
				))),
				Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
					id: AssetId(Location::here()),
					fun: Fungible(ONE_DOT * 10),
				}]))),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedAssetId::from(AssetId(Location::here()))),
				Box::new(xcm_executor::traits::TransferType::LocalReserve),
				Box::new(xcm::VersionedXcm::from(Xcm::<()>(vec![DepositAsset {
					assets: Wild(All),
					beneficiary: Location::new(
						0,
						[AccountKey20 { network: None, key: test_account }],
					),
				}]))),
				WeightLimit::Unlimited,
			)
		);
	});

	// Drain all GLMR, then verify foreign asset is still accessible.
	moonriver_execute_with(|| {
		let balance = <moonriver_runtime::Balances as Inspect<_>>::balance(
			&moonriver_runtime::AccountId::from(test_account),
		);
		let _ = <moonriver_runtime::Balances as Mutate<_>>::burn_from(
			&moonriver_runtime::AccountId::from(test_account),
			balance,
			frame_support::traits::tokens::Preservation::Expendable,
			frame_support::traits::tokens::Precision::BestEffort,
			frame_support::traits::tokens::Fortitude::Force,
		);

		// Foreign asset balance should still be accessible.
		let dot = moonriver_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonriver_runtime::AccountId::from(test_account),
		)
		.unwrap();
		assert!(
			dot > U256::zero(),
			"Foreign asset should survive native balance drain"
		);
	});
}

// ===========================================================================
// Native asset (GLMR) para → para transfers
// ===========================================================================

const MOVR_ASSET_ID: u128 = 2;

/// Register Moonriver's native GLMR as a foreign asset on the sibling and
/// configure the XCM weight trader price.
fn register_movr_on_sibling() {
	sibling_execute_with(|| {
		// From the sibling's perspective, Moonriver's native token lives at:
		// ../Parachain(2004)/PalletInstance(10)  (pallet_balances = index 10)
		let glmr_location = xcm::latest::Location::new(
			1,
			[
				Parachain(MOONBEAM_PARA_ID),
				PalletInstance(10u8),
			],
		);

		frame_support::assert_ok!(
			moonriver_runtime::EvmForeignAssets::create_foreign_asset(
				moonriver_runtime::RuntimeOrigin::root(),
				MOVR_ASSET_ID,
				glmr_location.clone(),
				18, // GLMR has 18 decimals
				b"MOVR".to_vec().try_into().unwrap(),
				b"Moonriver".to_vec().try_into().unwrap(),
			)
		);

		frame_support::assert_ok!(moonriver_runtime::XcmWeightTrader::add_asset(
			moonriver_runtime::RuntimeOrigin::root(),
			glmr_location,
			10_000_000_000_000_000_000_000_000_000u128, // 10^28 (generous relative price)
		));
	});
}

/// Setup for GLMR para→para transfers: open HRMP, register DOT on Moonriver,
/// register GLMR on sibling.
fn setup_movr_para_to_para() {
	setup_with_sibling();
	register_movr_on_sibling();
}

/// Transfer GLMR from Moonriver to Sibling (reserve-backed).
#[test]
fn transfer_movr_from_moonriver_to_sibling() {
	setup_movr_para_to_para();

	let alith_before = moonriver_execute_with(|| {
		<moonriver_runtime::Balances as Inspect<_>>::balance(
			&moonriver_runtime::AccountId::from(ALITH),
		)
	});

	let amount = moonriver_runtime::currency::MOVR; // 1 GLMR

	moonriver_execute_with(|| {
		assert_ok!(moonriver_runtime::PolkadotXcm::transfer_assets(
			moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
			Box::new(xcm::VersionedLocation::from(Location::new(
				1,
				[Parachain(SIBLING_PARA_ID)],
			))),
			Box::new(xcm::VersionedLocation::from(Location::new(
				0,
				[AccountKey20 { network: None, key: BALTATHAR }],
			))),
			Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
				id: AssetId(Location::new(0, [PalletInstance(10)])),
				fun: Fungible(amount),
			}]))),
			0,
			WeightLimit::Unlimited,
		));
	});

	// ALITH should have less GLMR after the transfer.
	let alith_after = moonriver_execute_with(|| {
		<moonriver_runtime::Balances as Inspect<_>>::balance(
			&moonriver_runtime::AccountId::from(ALITH),
		)
	});
	assert!(
		alith_after < alith_before,
		"ALITH should have less GLMR after transfer"
	);
	assert!(
		alith_before - alith_after >= amount,
		"ALITH should have spent at least {amount}"
	);

	// BALTATHAR should have GLMR on sibling (as foreign asset).
	sibling_execute_with(|| {
		let balance = moonriver_runtime::EvmForeignAssets::balance(
			MOVR_ASSET_ID,
			moonriver_runtime::AccountId::from(BALTATHAR),
		)
		.unwrap();
		assert!(
			balance > U256::zero(),
			"BALTATHAR should have GLMR on sibling"
		);
	});
}

/// Roundtrip: GLMR from Moonriver → Sibling → back to Moonriver.
#[test]
fn transfer_movr_roundtrip_moonriver_sibling() {
	setup_movr_para_to_para();

	let alith_initial = moonriver_execute_with(|| {
		<moonriver_runtime::Balances as Inspect<_>>::balance(
			&moonriver_runtime::AccountId::from(ALITH),
		)
	});

	let amount = moonriver_runtime::currency::MOVR; // 1 GLMR

	// Step 1: Send GLMR from Moonriver to Sibling (BALTATHAR).
	moonriver_execute_with(|| {
		assert_ok!(moonriver_runtime::PolkadotXcm::transfer_assets(
			moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
			Box::new(xcm::VersionedLocation::from(Location::new(
				1,
				[Parachain(SIBLING_PARA_ID)],
			))),
			Box::new(xcm::VersionedLocation::from(Location::new(
				0,
				[AccountKey20 { network: None, key: BALTATHAR }],
			))),
			Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
				id: AssetId(Location::new(0, [PalletInstance(10)])),
				fun: Fungible(amount),
			}]))),
			0,
			WeightLimit::Unlimited,
		));
	});

	// Verify BALTATHAR got GLMR on sibling.
	let glmr_on_sibling = sibling_execute_with(|| {
		moonriver_runtime::EvmForeignAssets::balance(
			MOVR_ASSET_ID,
			moonriver_runtime::AccountId::from(BALTATHAR),
		)
		.unwrap()
	});
	assert!(
		glmr_on_sibling > U256::zero(),
		"BALTATHAR should have GLMR on sibling: {glmr_on_sibling}"
	);

	// Step 2: Send GLMR back from Sibling to Moonriver (ALITH).
	// From the sibling's perspective, GLMR is at ../Parachain(2004)/PalletInstance(10).
	sibling_execute_with(|| {
		let glmr_location = Location::new(
			1,
			[Parachain(MOONBEAM_PARA_ID), PalletInstance(10)],
		);

		assert_ok!(moonriver_runtime::PolkadotXcm::transfer_assets(
			moonriver_runtime::RuntimeOrigin::signed(
				moonriver_runtime::AccountId::from(BALTATHAR),
			),
			Box::new(xcm::VersionedLocation::from(Location::new(
				1,
				[Parachain(MOONBEAM_PARA_ID)],
			))),
			Box::new(xcm::VersionedLocation::from(Location::new(
				0,
				[AccountKey20 { network: None, key: ALITH }],
			))),
			Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
				id: AssetId(glmr_location),
				fun: Fungible(glmr_on_sibling.as_u128()),
			}]))),
			0,
			WeightLimit::Unlimited,
		));
	});

	// ALITH should have recovered most of the GLMR (minus fees on both hops).
	let alith_final = moonriver_execute_with(|| {
		<moonriver_runtime::Balances as Inspect<_>>::balance(
			&moonriver_runtime::AccountId::from(ALITH),
		)
	});
	// After roundtrip, ALITH loses some to fees but should still have most.
	let total_lost = alith_initial.saturating_sub(alith_final);
	assert!(
		total_lost < amount,
		"Roundtrip should only lose fees, not the full amount: lost={total_lost}, sent={amount}"
	);
}

/// GLMR transfer with trader: fees are deducted from GLMR on the sibling.
#[test]
fn transfer_movr_to_sibling_with_trader_fees() {
	setup_movr_para_to_para();

	let amount = moonriver_runtime::currency::MOVR * 100; // 100 GLMR

	moonriver_execute_with(|| {
		assert_ok!(moonriver_runtime::PolkadotXcm::transfer_assets(
			moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
			Box::new(xcm::VersionedLocation::from(Location::new(
				1,
				[Parachain(SIBLING_PARA_ID)],
			))),
			Box::new(xcm::VersionedLocation::from(Location::new(
				0,
				[AccountKey20 { network: None, key: BALTATHAR }],
			))),
			Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
				id: AssetId(Location::new(0, [PalletInstance(10)])),
				fun: Fungible(amount),
			}]))),
			0,
			WeightLimit::Unlimited,
		));
	});

	sibling_execute_with(|| {
		let received = moonriver_runtime::EvmForeignAssets::balance(
			MOVR_ASSET_ID,
			moonriver_runtime::AccountId::from(BALTATHAR),
		)
		.unwrap();

		// BALTATHAR should receive less than the full amount (fees deducted).
		assert!(
			received > U256::zero() && received < U256::from(amount),
			"Should receive less than full amount due to fees: received={received}, sent={amount}"
		);

		// Treasury should have received some GLMR as fees.
		let treasury = moonriver_runtime::Treasury::account_id();
		let treasury_fee = moonriver_runtime::EvmForeignAssets::balance(
			MOVR_ASSET_ID,
			treasury,
		)
		.unwrap();
		assert!(
			treasury_fee > U256::zero(),
			"Treasury should have collected GLMR fees"
		);
	});
}

// ===========================================================================
// DOT transfers via RemoteReserve (relay as reserve)
// ===========================================================================

/// Fund ALITH with DOT via relay DMP.
fn fund_moonriver_alith_with_dot(amount: u128) {
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		let beneficiary = Location::new(
			0,
			[AccountKey20 {
				network: None,
				key: ALITH,
			}],
		);
		let assets: xcm::VersionedAssets = (Location::here(), amount).into();
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
}

/// Send DOT from Moonriver to a sibling using `RemoteReserve` through the
/// relay. DOT's reserve is the relay (parent), so a direct
/// `DestinationReserve` is invalid — the relay must mediate.
#[test]
fn transfer_dot_to_sibling_via_remote_reserve() {
	setup_with_sibling();

	let send_amount = ONE_DOT * 100;
	fund_moonriver_alith_with_dot(send_amount);

	let alith_dot_before = moonriver_execute_with(|| {
		moonriver_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonriver_runtime::AccountId::from(ALITH),
		)
		.unwrap_or_default()
	});
	assert!(
		alith_dot_before > U256::zero(),
		"ALITH should have DOT before transfer"
	);

	let transfer = ONE_DOT * 50;

	moonriver_execute_with(|| {
		let dot_location = Location::parent();

		assert_ok!(
			moonriver_runtime::PolkadotXcm::transfer_assets_using_type_and_then(
				moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
				Box::new(xcm::VersionedLocation::from(Location::new(
					1,
					[Parachain(SIBLING_PARA_ID)],
				))),
				Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
					id: AssetId(dot_location.clone()),
					fun: Fungible(transfer),
				}]))),
				Box::new(xcm_executor::traits::TransferType::RemoteReserve(
					xcm::VersionedLocation::from(Location::parent()),
				)),
				Box::new(xcm::VersionedAssetId::from(AssetId(dot_location.clone()))),
				Box::new(xcm_executor::traits::TransferType::RemoteReserve(
					xcm::VersionedLocation::from(Location::parent()),
				)),
				Box::new(xcm::VersionedXcm::from(Xcm::<()>(vec![
					BuyExecution {
						fees: Asset {
							id: AssetId(dot_location),
							fun: Fungible(ONE_DOT / 10),
						},
						weight_limit: WeightLimit::Unlimited,
					},
					DepositAsset {
						assets: Wild(All),
						beneficiary: Location::new(
							0,
							[AccountKey20 {
								network: None,
								key: BALTATHAR,
							}],
						),
					},
				]))),
				WeightLimit::Unlimited,
			)
		);
	});

	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {});

	let alith_dot_after = moonriver_execute_with(|| {
		moonriver_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonriver_runtime::AccountId::from(ALITH),
		)
		.unwrap_or_default()
	});
	assert!(
		alith_dot_after < alith_dot_before,
		"ALITH DOT should decrease after transfer"
	);

	let baltathar_dot = sibling_execute_with(|| {
		moonriver_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonriver_runtime::AccountId::from(BALTATHAR),
		)
		.unwrap_or_default()
	});
	assert!(
		baltathar_dot > U256::zero(),
		"BALTATHAR should have DOT on sibling (got {baltathar_dot})"
	);
}

/// Roundtrip: DOT from Moonriver → Sibling → back to Moonriver, both legs
/// using RemoteReserve through the relay.
#[test]
fn transfer_dot_roundtrip_via_remote_reserve() {
	setup_with_sibling();

	let send_amount = ONE_DOT * 100;
	fund_moonriver_alith_with_dot(send_amount);

	let outbound = ONE_DOT * 50;
	let dot_location = Location::parent();

	// ── Moonriver → Sibling ────────────────────────────────────────────────
	moonriver_execute_with(|| {
		assert_ok!(
			moonriver_runtime::PolkadotXcm::transfer_assets_using_type_and_then(
				moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
				Box::new(xcm::VersionedLocation::from(Location::new(
					1,
					[Parachain(SIBLING_PARA_ID)],
				))),
				Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
					id: AssetId(dot_location.clone()),
					fun: Fungible(outbound),
				}]))),
				Box::new(xcm_executor::traits::TransferType::RemoteReserve(
					xcm::VersionedLocation::from(Location::parent()),
				)),
				Box::new(xcm::VersionedAssetId::from(AssetId(dot_location.clone()))),
				Box::new(xcm_executor::traits::TransferType::RemoteReserve(
					xcm::VersionedLocation::from(Location::parent()),
				)),
				Box::new(xcm::VersionedXcm::from(Xcm::<()>(vec![
					BuyExecution {
						fees: Asset {
							id: AssetId(dot_location.clone()),
							fun: Fungible(ONE_DOT / 10),
						},
						weight_limit: WeightLimit::Unlimited,
					},
					DepositAsset {
						assets: Wild(All),
						beneficiary: Location::new(
							0,
							[AccountKey20 {
								network: None,
								key: BALTATHAR,
							}],
						),
					},
				]))),
				WeightLimit::Unlimited,
			)
		);
	});

	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {});

	let baltathar_dot = sibling_execute_with(|| {
		moonriver_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonriver_runtime::AccountId::from(BALTATHAR),
		)
		.unwrap_or_default()
	});
	assert!(baltathar_dot > U256::zero(), "Sibling should have DOT");

	// ── Sibling → Moonriver ────────────────────────────────────────────────
	let return_amount_raw: u128 = baltathar_dot.try_into().unwrap();
	let return_half = return_amount_raw / 2;

	sibling_execute_with(|| {
		assert_ok!(
			moonriver_runtime::PolkadotXcm::transfer_assets_using_type_and_then(
				moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(
					BALTATHAR,
				)),
				Box::new(xcm::VersionedLocation::from(Location::new(
					1,
					[Parachain(MOONBEAM_PARA_ID)],
				))),
				Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
					id: AssetId(dot_location.clone()),
					fun: Fungible(return_half),
				}]))),
				Box::new(xcm_executor::traits::TransferType::RemoteReserve(
					xcm::VersionedLocation::from(Location::parent()),
				)),
				Box::new(xcm::VersionedAssetId::from(AssetId(dot_location.clone()))),
				Box::new(xcm_executor::traits::TransferType::RemoteReserve(
					xcm::VersionedLocation::from(Location::parent()),
				)),
				Box::new(xcm::VersionedXcm::from(Xcm::<()>(vec![
					BuyExecution {
						fees: Asset {
							id: AssetId(dot_location),
							fun: Fungible(ONE_DOT / 10),
						},
						weight_limit: WeightLimit::Unlimited,
					},
					DepositAsset {
						assets: Wild(All),
						beneficiary: Location::new(
							0,
							[AccountKey20 {
								network: None,
								key: ALITH,
							}],
						),
					},
				]))),
				WeightLimit::Unlimited,
			)
		);
	});

	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {});

	let alith_dot_final = moonriver_execute_with(|| {
		moonriver_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonriver_runtime::AccountId::from(ALITH),
		)
		.unwrap_or_default()
	});
	assert!(
		alith_dot_final > U256::from(send_amount - outbound),
		"ALITH should have more DOT than after the outbound leg (got {alith_dot_final})"
	);
}

/// Transfer GLMR to a sibling as a self-reserve asset (GLMR pays its own
/// fees). Exercises `transfer_assets` with a single asset where the fee
/// asset and the transfer asset are the same.
#[test]
fn transfer_movr_self_reserve_to_sibling() {
	setup_with_sibling();
	register_movr_on_sibling();

	let glmr_amount = moonriver_runtime::currency::MOVR;

	moonriver_execute_with(|| {
		assert_ok!(moonriver_runtime::PolkadotXcm::transfer_assets(
			moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
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
				id: AssetId(Location::new(0, [PalletInstance(10)])),
				fun: Fungible(glmr_amount),
			}]))),
			0,
			WeightLimit::Unlimited,
		));
	});

	let bal_glmr = sibling_execute_with(|| {
		moonriver_runtime::EvmForeignAssets::balance(
			MOVR_ASSET_ID,
			moonriver_runtime::AccountId::from(BALTATHAR),
		)
		.unwrap_or_default()
	});
	assert!(
		bal_glmr > U256::zero(),
		"BALTATHAR should have received GLMR on sibling (got {bal_glmr})"
	);
}

/// Receive a sibling-native foreign asset on Moonriver.
/// A sibling sends its own native token (another Moonriver instance's GLMR)
/// to Moonriver, which receives it as an EVM foreign asset.
#[test]
fn receive_sibling_native_asset() {
	setup_with_sibling();

	// On Moonriver, register the sibling's GLMR (PalletInstance(10) on para 2005)
	// as a foreign asset with id=3.
	const SIBLING_MOVR_ASSET_ID: u128 = 3;
	moonriver_execute_with(|| {
		let sibling_glmr_location = xcm::latest::Location::new(
			1,
			[Parachain(SIBLING_PARA_ID), PalletInstance(10u8)],
		);

		frame_support::assert_ok!(
			moonriver_runtime::EvmForeignAssets::create_foreign_asset(
				moonriver_runtime::RuntimeOrigin::root(),
				SIBLING_MOVR_ASSET_ID,
				sibling_glmr_location.clone(),
				18,
				b"sGLMR".to_vec().try_into().unwrap(),
				b"Sibling Glimmer".to_vec().try_into().unwrap(),
			)
		);

		frame_support::assert_ok!(moonriver_runtime::XcmWeightTrader::add_asset(
			moonriver_runtime::RuntimeOrigin::root(),
			sibling_glmr_location,
			10_000_000_000_000_000_000_000_000_000u128,
		));
	});

	let amount = moonriver_runtime::currency::MOVR;

	sibling_execute_with(|| {
		assert_ok!(moonriver_runtime::PolkadotXcm::transfer_assets(
			moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
			Box::new(xcm::VersionedLocation::from(Location::new(
				1,
				[Parachain(MOONBEAM_PARA_ID)],
			))),
			Box::new(xcm::VersionedLocation::from(Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: BALTATHAR,
				}],
			))),
			Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
				id: AssetId(Location::new(0, [PalletInstance(10)])),
				fun: Fungible(amount),
			}]))),
			0,
			WeightLimit::Unlimited,
		));
	});

	let bal = moonriver_execute_with(|| {
		moonriver_runtime::EvmForeignAssets::balance(
			SIBLING_MOVR_ASSET_ID,
			moonriver_runtime::AccountId::from(BALTATHAR),
		)
		.unwrap_or_default()
	});
	assert!(
		bal > U256::zero(),
		"BALTATHAR should have sibling GLMR on Moonriver (got {bal})"
	);
}
