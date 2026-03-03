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

//! XcmTransactor tests using the **real** Moonbeam runtime against Westend relay.
//!
//! Covers: transact_through_sovereign (relay), HRMP channel management.

use crate::emulator_network::*;
use frame_support::{
	assert_ok,
	traits::fungible::Inspect,
};
use pallet_xcm_transactor::{Currency, CurrencyPayment, TransactWeights};
use parity_scale_codec::Encode;
use xcm::latest::prelude::*;
use xcm_emulator::{RelayChain, TestExt};

const DOT_ASSET_ID: u128 = 1;

// ===========================================================================
// Setup
// ===========================================================================

fn setup_transactor() {
	init_network();

	moonbeam_execute_with(|| {
		register_dot_asset(DOT_ASSET_ID);
		set_westend_relay_indices();

		// Configure transact info for the relay destination.
		assert_ok!(moonbeam_runtime::XcmTransactor::set_transact_info(
			moonbeam_runtime::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			3_000u64.into(),          // extra_weight (relay charges per instruction)
			20_000_000_000u64.into(), // max_weight
			None,
		));
	});

	// Fund Moonbeam's sovereign on relay so it can pay fees for UMP transacts.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		// The sovereign is already funded via relay genesis (endowment).
	});
}

/// Send DOT from relay to Moonbeam ALITH.
fn fund_moonbeam_alith_with_dot(amount: u128) {
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
							key: ALITH
						}],
					),
				}]))),
				WeightLimit::Unlimited,
			)
		);
	});
}

// ===========================================================================
// Transact through sovereign (para → relay)
// ===========================================================================

#[test]
fn transact_through_sovereign_to_relay() {
	setup_transactor();
	fund_moonbeam_alith_with_dot(ONE_DOT * 1000);

	// Check the sovereign account balance on relay before transact.
	let sovereign = WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		WestendRelay::<PolkadotMoonbeamNet>::sovereign_account_id_of(Location::new(
			0,
			[Parachain(MOONBEAM_PARA_ID)],
		))
	});
	let sovereign_before = WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		<westend_runtime::Balances as Inspect<_>>::balance(&sovereign)
	});
	assert!(
		sovereign_before > 0,
		"Sovereign should be funded from genesis"
	);

	// Encode a system::remark_with_event call for the relay.
	let encoded = westend_runtime::RuntimeCall::System(
		frame_system::Call::<westend_runtime::Runtime>::remark_with_event {
			remark: b"hello from Moonbeam".to_vec(),
		},
	)
	.encode();

	moonbeam_execute_with(|| {
		assert_ok!(moonbeam_runtime::XcmTransactor::transact_through_sovereign(
			moonbeam_runtime::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			Some(moonbeam_runtime::AccountId::from(ALITH)),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: Some(ONE_DOT), // explicit fee
			},
			encoded,
			OriginKind::SovereignAccount,
			TransactWeights {
				transact_required_weight_at_most: 1_000_000_000u64.into(),
				overall_weight: Some(Limited(2_000_000_000u64.into())),
			},
			false,
		));
	});

	// Verify the remark was executed on the relay.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		let events = westend_runtime::System::events();

		let was_processed = events.iter().any(|e| {
			matches!(
				&e.event,
				westend_runtime::RuntimeEvent::MessageQueue(
					pallet_message_queue::Event::Processed { success: true, .. }
				)
			)
		});
		assert!(
			was_processed,
			"Relay should have successfully processed the UMP transact"
		);

		let has_remark = events.iter().any(|e| {
			matches!(
				&e.event,
				westend_runtime::RuntimeEvent::System(
					frame_system::Event::Remarked { .. }
				)
			)
		});
		assert!(has_remark, "Relay should have emitted a Remarked event");
	});

	// Verify the sovereign paid fees for the XCM execution.
	let sovereign_after = WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		<westend_runtime::Balances as Inspect<_>>::balance(&sovereign)
	});
	assert!(
		sovereign_after <= sovereign_before,
		"Sovereign should have spent DOT: before={sovereign_before}, after={sovereign_after}"
	);
}

// ===========================================================================
// HRMP: open and close channels via XcmTransactor
// ===========================================================================

#[test]
fn hrmp_init_accept_close_via_xcm_transactor() {
	init_network();

	moonbeam_execute_with(|| {
		register_dot_asset(DOT_ASSET_ID);
		set_westend_relay_indices();
	});
	sibling_execute_with(|| {
		register_dot_asset(DOT_ASSET_ID);
		set_westend_relay_indices();
	});

	use pallet_xcm_transactor::{HrmpInitParams, HrmpOperation};

	// Step 1: Moonbeam requests to open channel to sibling.
	moonbeam_execute_with(|| {
		assert_ok!(moonbeam_runtime::XcmTransactor::hrmp_manage(
			moonbeam_runtime::RuntimeOrigin::root(),
			HrmpOperation::InitOpen(HrmpInitParams {
				para_id: SIBLING_PARA_ID.into(),
				proposed_max_capacity: 8,
				proposed_max_message_size: 1024,
			}),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: Some(ONE_DOT * 100),
			},
			TransactWeights {
				transact_required_weight_at_most: 5_000_000_000u64.into(),
				overall_weight: Some(Limited(10_000_000_000u64.into())),
			},
		));
	});

	// Verify the open-channel request arrived on relay.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		let events = westend_runtime::System::events();
		let has_open_request = events.iter().any(|e| {
			matches!(
				&e.event,
				westend_runtime::RuntimeEvent::Hrmp(
					polkadot_runtime_parachains::hrmp::Event::OpenChannelRequested { .. }
				)
			)
		});
		assert!(
			has_open_request,
			"Relay should have emitted OpenChannelRequested"
		);
	});

	// Step 2: Sibling accepts the channel.
	sibling_execute_with(|| {
		assert_ok!(moonbeam_runtime::XcmTransactor::hrmp_manage(
			moonbeam_runtime::RuntimeOrigin::root(),
			HrmpOperation::Accept {
				para_id: MOONBEAM_PARA_ID.into(),
			},
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent()
				))),
				fee_amount: Some(ONE_DOT * 100),
			},
			TransactWeights {
				transact_required_weight_at_most: 5_000_000_000u64.into(),
				overall_weight: Some(Limited(10_000_000_000u64.into())),
			},
		));
	});

	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		let events = westend_runtime::System::events();
		let has_accept = events.iter().any(|e| {
			matches!(
				&e.event,
				westend_runtime::RuntimeEvent::Hrmp(
					polkadot_runtime_parachains::hrmp::Event::OpenChannelAccepted { .. }
				)
			)
		});
		assert!(
			has_accept,
			"Relay should have emitted OpenChannelAccepted"
		);
	});

	// Step 3: Process the pending open requests and verify the channel is established.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(westend_runtime::Hrmp::force_process_hrmp_open(
			westend_runtime::RuntimeOrigin::root(),
			1,
		));

		use polkadot_runtime_parachains::hrmp;
		let channel = hrmp::HrmpChannels::<westend_runtime::Runtime>::get(
			xcm_emulator::HrmpChannelId {
				sender: MOONBEAM_PARA_ID.into(),
				recipient: SIBLING_PARA_ID.into(),
			},
		);
		assert!(
			channel.is_some(),
			"HRMP channel Moonbeam → Sibling should be established"
		);
	});
}
