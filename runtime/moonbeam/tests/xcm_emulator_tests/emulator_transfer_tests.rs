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

//! Transfer tests using xcm-emulator with the **real** Moonbeam runtime.
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

/// Full network init: register DOT on Moonbeam, configure weight trader.
fn setup_relay_to_moonbeam() {
	init_network();
	moonbeam_execute_with(|| {
		register_dot_asset(DOT_ASSET_ID);
	});
}

/// Full network init with sibling: register DOT on both paras, open HRMP channels.
fn setup_with_sibling() {
	init_network();

	moonbeam_execute_with(|| {
		register_dot_asset(DOT_ASSET_ID);
	});
	sibling_execute_with(|| {
		register_dot_asset(DOT_ASSET_ID);
	});

	// Open bi-directional HRMP channels between Moonbeam (2004) and Sibling (2005).
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		open_hrmp_channels(MOONBEAM_PARA_ID, SIBLING_PARA_ID);
	});
}

// ===========================================================================
// Transfer: Relay → Moonbeam (DMP)
// ===========================================================================

#[test]
fn transfer_dot_from_relay_to_moonbeam() {
	setup_relay_to_moonbeam();

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

	moonbeam_execute_with(|| {
		let beneficiary = moonbeam_runtime::AccountId::from(beneficiary_key);
		let balance = moonbeam_runtime::EvmForeignAssets::balance(DOT_ASSET_ID, beneficiary)
			.expect("balance query should succeed");
		assert!(
			balance > U256::zero(),
			"Beneficiary should have DOT on Moonbeam, got {balance}"
		);
	});
}

// ===========================================================================
// Transfer: Moonbeam → Relay (UMP)
// ===========================================================================

#[test]
fn transfer_dot_from_moonbeam_to_relay() {
	setup_relay_to_moonbeam();

	// First: send DOT from relay to Moonbeam so ALITH has some DOT.
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

	let alith_dot_before = moonbeam_execute_with(|| {
		moonbeam_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonbeam_runtime::AccountId::from(ALITH),
		)
		.unwrap()
	});
	assert!(alith_dot_before > U256::zero(), "ALITH should have DOT");

	// Record relay-side balance of a relay account before the return transfer.
	let relay_bob = sp_runtime::AccountId32::new([2u8; 32]);
	let relay_bob_before = WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		<westend_runtime::Balances as Inspect<_>>::balance(&relay_bob)
	});

	// Now send DOT back from Moonbeam to relay via PolkadotXcm.
	// DOT's reserve is the relay, so we use DestinationReserve transfer type.
	moonbeam_execute_with(|| {
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
			moonbeam_runtime::PolkadotXcm::transfer_assets_using_type_and_then(
				moonbeam_runtime::RuntimeOrigin::signed(moonbeam_runtime::AccountId::from(ALITH)),
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
	setup_relay_to_moonbeam();

	// Send a tiny amount (1 unit) from relay — should fail to pay Moonbeam execution fees.
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
	moonbeam_execute_with(|| {
		let balance = moonbeam_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonbeam_runtime::AccountId::from(ALITH),
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
	setup_relay_to_moonbeam();

	let treasury_dot_before = moonbeam_execute_with(|| {
		let treasury = moonbeam_runtime::Treasury::account_id();
		moonbeam_runtime::EvmForeignAssets::balance(DOT_ASSET_ID, treasury).unwrap_or(U256::zero())
	});

	// Send DOT from relay to Moonbeam (fees will be charged).
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

	moonbeam_execute_with(|| {
		let treasury = moonbeam_runtime::Treasury::account_id();
		let treasury_dot_after =
			moonbeam_runtime::EvmForeignAssets::balance(DOT_ASSET_ID, treasury)
				.unwrap_or(U256::zero());
		assert!(
			treasury_dot_after > treasury_dot_before,
			"Treasury should collect fees: before={treasury_dot_before}, after={treasury_dot_after}"
		);

		// And beneficiary should have gotten the rest (not the full amount).
		let beneficiary_balance = moonbeam_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonbeam_runtime::AccountId::from(BALTATHAR),
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
	setup_relay_to_moonbeam();

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

	moonbeam_execute_with(|| {
		let balance = moonbeam_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonbeam_runtime::AccountId::from(fresh_account),
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
fn transfer_dot_from_moonbeam_to_sibling() {
	setup_with_sibling();

	// First fund Moonbeam ALITH with DOT from relay.
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

	// Verify ALITH got DOT on Moonbeam.
	let alith_dot = moonbeam_execute_with(|| {
		moonbeam_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonbeam_runtime::AccountId::from(ALITH),
		)
		.unwrap()
	});
	assert!(alith_dot > U256::zero(), "ALITH should have DOT");

	// Now send DOT from Moonbeam to Sibling via reserve transfer through relay.
	// DOT's reserve is the relay (parent), so we use RemoteReserve.
	// The custom_xcm_on_dest must include BuyExecution since the sibling's
	// barrier requires paid execution.
	moonbeam_execute_with(|| {
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
			moonbeam_runtime::PolkadotXcm::transfer_assets_using_type_and_then(
				moonbeam_runtime::RuntimeOrigin::signed(moonbeam_runtime::AccountId::from(ALITH)),
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
		let balance = moonbeam_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonbeam_runtime::AccountId::from(BALTATHAR),
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
	setup_relay_to_moonbeam();

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

	moonbeam_execute_with(|| {
		// ALITH should have both native GLMR and foreign DOT.
		let glmr = <moonbeam_runtime::Balances as Inspect<_>>::balance(
			&moonbeam_runtime::AccountId::from(ALITH),
		);
		assert!(glmr > 0, "ALITH should still have GLMR");

		let dot = moonbeam_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonbeam_runtime::AccountId::from(ALITH),
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
	setup_relay_to_moonbeam();

	let test_account: [u8; 20] = [77u8; 20];

	// Give the test account some GLMR.
	moonbeam_execute_with(|| {
		<moonbeam_runtime::Balances as Mutate<_>>::mint_into(
			&moonbeam_runtime::AccountId::from(test_account),
			moonbeam_runtime::currency::GLMR,
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
	moonbeam_execute_with(|| {
		let balance = <moonbeam_runtime::Balances as Inspect<_>>::balance(
			&moonbeam_runtime::AccountId::from(test_account),
		);
		let _ = <moonbeam_runtime::Balances as Mutate<_>>::burn_from(
			&moonbeam_runtime::AccountId::from(test_account),
			balance,
			frame_support::traits::tokens::Preservation::Expendable,
			frame_support::traits::tokens::Precision::BestEffort,
			frame_support::traits::tokens::Fortitude::Force,
		);

		// Foreign asset balance should still be accessible.
		let dot = moonbeam_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonbeam_runtime::AccountId::from(test_account),
		)
		.unwrap();
		assert!(
			dot > U256::zero(),
			"Foreign asset should survive native balance drain"
		);
	});
}
