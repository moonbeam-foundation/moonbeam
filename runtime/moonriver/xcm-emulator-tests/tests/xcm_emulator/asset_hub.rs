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

//! Asset Hub ↔ Moonbeam transfer tests using xcm-emulator.
//!
//! These tests exercise cross-chain transfers between the real
//! `asset-hub-westend-runtime` (para 1000) and the real `moonriver-runtime`
//! (para 2004), with Westend as relay.

use crate::network::*;
use frame_support::{assert_ok, traits::fungible::Inspect};
use sp_core::U256;
use xcm::latest::prelude::*;
use xcm_emulator::TestExt;

// ===========================================================================
// Setup helpers
// ===========================================================================

/// Register DOT on Moonbeam, open HRMP between Asset Hub and Moonbeam.
fn setup_asset_hub_and_moonriver() {
	init_network();

	moonriver_execute_with(|| register_dot_asset(DOT_ASSET_ID));

	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		open_hrmp_channels(ASSET_HUB_PARA_ID, MOONRIVER_PARA_ID);
	});
}

/// Fund ALITH on Moonbeam with DOT from the relay.
fn fund_moonriver_alith_with_dot(amount: u128) {
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(
			westend_runtime::XcmPallet::transfer_assets_using_type_and_then(
				westend_runtime::RuntimeOrigin::signed(RELAY_ALICE.clone()),
				Box::new(xcm::VersionedLocation::from(Location::new(
					0,
					[Parachain(MOONRIVER_PARA_ID)]
				))),
				Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
					id: AssetId(Location::here()),
					fun: Fungible(amount),
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
							key: ALITH,
						}],
					),
				}]))),
				WeightLimit::Unlimited,
			)
		);
	});
}

// ===========================================================================
// Tests
// ===========================================================================

/// Transfer DOT from the relay to Asset Hub, confirming the real
/// asset-hub-westend-runtime processes DMP correctly.
#[test]
fn transfer_dot_from_relay_to_asset_hub() {
	init_network();

	let recipient = sp_runtime::AccountId32::new([2u8; 32]);

	let balance_before = asset_hub_execute_with(|| {
		<asset_hub_westend_runtime::Balances as Inspect<_>>::balance(&recipient)
	});

	// Send DOT from relay to Asset Hub (DOT is the native token on both).
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(westend_runtime::XcmPallet::limited_teleport_assets(
			westend_runtime::RuntimeOrigin::signed(RELAY_ALICE.clone()),
			Box::new(xcm::VersionedLocation::from(Location::new(
				0,
				[Parachain(ASSET_HUB_PARA_ID)],
			))),
			Box::new(xcm::VersionedLocation::from(Location::new(
				0,
				[AccountId32 {
					network: None,
					id: recipient.clone().into(),
				}],
			))),
			Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
				id: AssetId(Location::here()),
				fun: Fungible(ONE_DOT * 10),
			}]))),
			0,
			WeightLimit::Unlimited,
		));
	});

	let balance_after = asset_hub_execute_with(|| {
		<asset_hub_westend_runtime::Balances as Inspect<_>>::balance(&recipient)
	});
	assert!(
		balance_after > balance_before,
		"Asset Hub account should have received DOT: before={balance_before}, after={balance_after}"
	);
}

/// Transfer DOT from the relay to both Asset Hub (teleport) and Moonbeam
/// (reserve), confirming both chains can hold DOT originated from the
/// same relay in the same network.
#[test]
fn relay_funds_both_asset_hub_and_moonriver() {
	setup_asset_hub_and_moonriver();

	let ah_recipient = sp_runtime::AccountId32::new([2u8; 32]);

	// Fund Asset Hub via teleport.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(westend_runtime::XcmPallet::limited_teleport_assets(
			westend_runtime::RuntimeOrigin::signed(RELAY_ALICE.clone()),
			Box::new(xcm::VersionedLocation::from(Location::new(
				0,
				[Parachain(ASSET_HUB_PARA_ID)],
			))),
			Box::new(xcm::VersionedLocation::from(Location::new(
				0,
				[AccountId32 {
					network: None,
					id: ah_recipient.clone().into(),
				}],
			))),
			Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
				id: AssetId(Location::here()),
				fun: Fungible(ONE_DOT * 10),
			}]))),
			0,
			WeightLimit::Unlimited,
		));
	});

	// Fund Moonbeam via reserve.
	fund_moonriver_alith_with_dot(ONE_DOT * 10);

	// Both chains should have DOT.
	let ah_balance = asset_hub_execute_with(|| {
		<asset_hub_westend_runtime::Balances as Inspect<_>>::balance(&ah_recipient)
	});
	assert!(
		ah_balance > 0,
		"Asset Hub should have DOT (got {ah_balance})"
	);

	let moonriver_balance = moonriver_execute_with(|| {
		moonriver_runtime::EvmForeignAssets::balance(
			DOT_ASSET_ID,
			moonriver_runtime::AccountId::from(ALITH),
		)
		.unwrap_or_default()
	});
	assert!(
		moonriver_balance > U256::zero(),
		"Moonriver should have DOT (got {moonriver_balance})"
	);
}

/// Transfer a trust-backed asset (e.g. USDT) from Asset Hub to Moonbeam.
/// Asset Hub is the reserve for trust-backed assets, so this is a
/// reserve-backed transfer.
#[test]
fn transfer_trust_backed_asset_from_asset_hub_to_moonriver() {
	setup_asset_hub_and_moonriver();

	// Create and mint a trust-backed asset (id=1984, "USDT") on Asset Hub.
	let asset_id: u32 = 1984;
	let asset_owner = sp_runtime::AccountId32::new([1u8; 32]);
	let mint_amount: u128 = 1_000_000_000; // 1000 USDT (6 decimals)

	asset_hub_execute_with(|| {
		assert_ok!(asset_hub_westend_runtime::Assets::force_create(
			asset_hub_westend_runtime::RuntimeOrigin::root(),
			asset_id.into(),
			asset_owner.clone().into(),
			true,
			1_000, // min_balance
		));
		assert_ok!(asset_hub_westend_runtime::Assets::mint(
			asset_hub_westend_runtime::RuntimeOrigin::signed(asset_owner.clone()),
			asset_id.into(),
			asset_owner.clone().into(),
			mint_amount,
		));
	});

	// Register this asset on Moonbeam as a foreign asset.
	// From Moonbeam's perspective: ../Parachain(1000)/PalletInstance(50)/GeneralIndex(1984)
	const USDT_FOREIGN_ID: u128 = 10;
	moonriver_execute_with(|| {
		let usdt_location = xcm::latest::Location::new(
			1,
			[
				Parachain(ASSET_HUB_PARA_ID),
				PalletInstance(50u8), // pallet_assets instance 1
				GeneralIndex(asset_id as u128),
			],
		);

		frame_support::assert_ok!(moonriver_runtime::EvmForeignAssets::create_foreign_asset(
			moonriver_runtime::RuntimeOrigin::root(),
			USDT_FOREIGN_ID,
			usdt_location.clone(),
			6, // USDT decimals
			b"USDT".to_vec().try_into().unwrap(),
			b"Tether USD".to_vec().try_into().unwrap(),
		));

		frame_support::assert_ok!(moonriver_runtime::XcmWeightTrader::add_asset(
			moonriver_runtime::RuntimeOrigin::root(),
			usdt_location,
			10_000_000_000_000_000_000_000_000_000u128,
		));
	});

	// Also need DOT on Asset Hub sender for fees. Fund via relay teleport.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(westend_runtime::XcmPallet::limited_teleport_assets(
			westend_runtime::RuntimeOrigin::signed(RELAY_ALICE.clone()),
			Box::new(xcm::VersionedLocation::from(Location::new(
				0,
				[Parachain(ASSET_HUB_PARA_ID)],
			))),
			Box::new(xcm::VersionedLocation::from(Location::new(
				0,
				[AccountId32 {
					network: None,
					id: asset_owner.clone().into(),
				}],
			))),
			Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
				id: AssetId(Location::here()),
				fun: Fungible(ONE_DOT * 100),
			}]))),
			0,
			WeightLimit::Unlimited,
		));
	});

	// Transfer USDT from Asset Hub to Moonbeam.
	// Asset Hub is the reserve for this trust-backed asset.
	let transfer_amount: u128 = 500_000_000; // 500 USDT

	asset_hub_execute_with(|| {
		let usdt_on_ah = Location::new(0, [PalletInstance(50u8), GeneralIndex(asset_id as u128)]);

		assert_ok!(asset_hub_westend_runtime::PolkadotXcm::transfer_assets(
			asset_hub_westend_runtime::RuntimeOrigin::signed(asset_owner.clone()),
			Box::new(xcm::VersionedLocation::from(Location::new(
				1,
				[Parachain(MOONRIVER_PARA_ID)],
			))),
			Box::new(xcm::VersionedLocation::from(Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: ALITH,
				}],
			))),
			Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
				id: AssetId(usdt_on_ah),
				fun: Fungible(transfer_amount),
			}]))),
			0,
			WeightLimit::Unlimited,
		));
	});

	// ALITH on Moonbeam should have received USDT as a foreign asset.
	let alith_usdt = moonriver_execute_with(|| {
		moonriver_runtime::EvmForeignAssets::balance(
			USDT_FOREIGN_ID,
			moonriver_runtime::AccountId::from(ALITH),
		)
		.unwrap_or_default()
	});
	assert!(
		alith_usdt > U256::zero(),
		"ALITH should have received USDT on Moonbeam (got {alith_usdt})"
	);
}

/// Moonriver → Asset Hub return-leg coverage for a trust-backed asset.
///
/// AH is the reserve for trust-backed assets (`pallet-assets`, `PalletInstance(50)`),
/// so from Moonriver this is a `DestinationReserve` transfer. The forward leg
/// (AH → Moonriver) leaves USDT in Moonriver's sovereign account on AH; the
/// return leg withdraws from that sovereign back to the recipient on AH.
///
/// What this test verifies on the Moonriver side:
///   - `transfer_assets_using_type_and_then` with `DestinationReserve` succeeds.
///   - The foreign-asset balance burns by the transfer amount.
///   - The XCM message is delivered to AH and `WithdrawAsset` succeeds against
///     Moonriver's sovereign account on AH (USDT is burned from the sovereign).
///
/// What this test does NOT verify on the AH side: that the recipient ends up
/// with USDT. AH's `SwapFirstAssetTrader` requires either a USDT⇄WND pool with
/// enough depth or DOT in the holding to satisfy `BuyExecution`. Configuring
/// that is out of scope here; the post-fee delivery path is exercised by
/// dedicated AH tests upstream in polkadot-sdk.
#[test]
fn transfer_trust_backed_asset_from_moonriver_to_asset_hub() {
	setup_asset_hub_and_moonriver();

	let asset_id: u32 = 1984;
	let asset_owner = sp_runtime::AccountId32::new([1u8; 32]);
	let mint_amount: u128 = 1_000_000_000_000;

	asset_hub_execute_with(|| {
		assert_ok!(asset_hub_westend_runtime::Assets::force_create(
			asset_hub_westend_runtime::RuntimeOrigin::root(),
			asset_id.into(),
			asset_owner.clone().into(),
			true,
			1_000,
		));
		assert_ok!(asset_hub_westend_runtime::Assets::mint(
			asset_hub_westend_runtime::RuntimeOrigin::signed(asset_owner.clone()),
			asset_id.into(),
			asset_owner.clone().into(),
			mint_amount,
		));
	});

	const USDT_FOREIGN_ID: u128 = 11;
	let usdt_on_moonriver = Location::new(
		1,
		[
			Parachain(ASSET_HUB_PARA_ID),
			PalletInstance(50u8),
			GeneralIndex(asset_id as u128),
		],
	);
	moonriver_execute_with(|| {
		assert_ok!(moonriver_runtime::EvmForeignAssets::create_foreign_asset(
			moonriver_runtime::RuntimeOrigin::root(),
			USDT_FOREIGN_ID,
			usdt_on_moonriver.clone(),
			6,
			b"USDT".to_vec().try_into().unwrap(),
			b"Tether USD".to_vec().try_into().unwrap(),
		));
		assert_ok!(moonriver_runtime::XcmWeightTrader::add_asset(
			moonriver_runtime::RuntimeOrigin::root(),
			usdt_on_moonriver.clone(),
			10_000_000_000_000_000_000_000_000_000u128,
		));
	});

	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(westend_runtime::XcmPallet::limited_teleport_assets(
			westend_runtime::RuntimeOrigin::signed(RELAY_ALICE.clone()),
			Box::new(xcm::VersionedLocation::from(Location::new(
				0,
				[Parachain(ASSET_HUB_PARA_ID)],
			))),
			Box::new(xcm::VersionedLocation::from(Location::new(
				0,
				[AccountId32 {
					network: None,
					id: asset_owner.clone().into(),
				}],
			))),
			Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
				id: AssetId(Location::here()),
				fun: Fungible(ONE_DOT * 100),
			}]))),
			0,
			WeightLimit::Unlimited,
		));
	});

	let forward_amount: u128 = 2_000_000_000;
	asset_hub_execute_with(|| {
		let usdt_on_ah = Location::new(0, [PalletInstance(50u8), GeneralIndex(asset_id as u128)]);

		assert_ok!(asset_hub_westend_runtime::PolkadotXcm::transfer_assets(
			asset_hub_westend_runtime::RuntimeOrigin::signed(asset_owner.clone()),
			Box::new(xcm::VersionedLocation::from(Location::new(
				1,
				[Parachain(MOONRIVER_PARA_ID)],
			))),
			Box::new(xcm::VersionedLocation::from(Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: ALITH,
				}],
			))),
			Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
				id: AssetId(usdt_on_ah),
				fun: Fungible(forward_amount),
			}]))),
			0,
			WeightLimit::Unlimited,
		));
	});

	let alith_usdt_before_return = moonriver_execute_with(|| {
		moonriver_runtime::EvmForeignAssets::balance(
			USDT_FOREIGN_ID,
			moonriver_runtime::AccountId::from(ALITH),
		)
		.unwrap_or_default()
	});
	assert!(
		alith_usdt_before_return > U256::zero(),
		"ALITH should hold USDT before return leg (got {alith_usdt_before_return})"
	);

	let moonriver_sovereign_on_ah: sp_runtime::AccountId32 =
		<asset_hub_westend_runtime::xcm_config::LocationToAccountId as xcm_executor::traits::ConvertLocation<
			sp_runtime::AccountId32,
		>>::convert_location(&Location::new(1, [Parachain(MOONRIVER_PARA_ID)]))
		.expect("sibling sovereign convert");
	let sovereign_usdt_before = asset_hub_execute_with(|| {
		asset_hub_westend_runtime::Assets::balance(asset_id, moonriver_sovereign_on_ah.clone())
	});
	assert_eq!(
		sovereign_usdt_before, forward_amount,
		"Moonriver sovereign on AH should hold the forwarded USDT"
	);

	let return_amount: u128 = 1_000_000_000;
	let return_recipient = sp_runtime::AccountId32::new([3u8; 32]);
	moonriver_execute_with(|| {
		assert_ok!(
			moonriver_runtime::PolkadotXcm::transfer_assets_using_type_and_then(
				moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
				Box::new(xcm::VersionedLocation::from(Location::new(
					1,
					[Parachain(ASSET_HUB_PARA_ID)],
				))),
				Box::new(xcm::VersionedAssets::from(Assets::from(vec![Asset {
					id: AssetId(usdt_on_moonriver.clone()),
					fun: Fungible(return_amount),
				}]))),
				Box::new(xcm_executor::traits::TransferType::DestinationReserve),
				Box::new(xcm::VersionedAssetId::from(AssetId(usdt_on_moonriver))),
				Box::new(xcm_executor::traits::TransferType::DestinationReserve),
				Box::new(xcm::VersionedXcm::from(Xcm::<()>(vec![DepositAsset {
					assets: Wild(All),
					beneficiary: Location::new(
						0,
						[AccountId32 {
							network: None,
							id: return_recipient.into(),
						}],
					),
				}]))),
				WeightLimit::Unlimited,
			)
		);
	});

	let alith_usdt_after_return = moonriver_execute_with(|| {
		moonriver_runtime::EvmForeignAssets::balance(
			USDT_FOREIGN_ID,
			moonriver_runtime::AccountId::from(ALITH),
		)
		.unwrap_or_default()
	});
	assert_eq!(
		alith_usdt_after_return,
		alith_usdt_before_return - U256::from(return_amount),
		"ALITH USDT on Moonriver should drop by {return_amount}"
	);

	let sovereign_usdt_after = asset_hub_execute_with(|| {
		asset_hub_westend_runtime::Assets::balance(asset_id, moonriver_sovereign_on_ah)
	});
	assert_eq!(
		sovereign_usdt_after,
		sovereign_usdt_before - return_amount,
		"Moonriver sovereign on AH should be debited by {return_amount}"
	);
}
