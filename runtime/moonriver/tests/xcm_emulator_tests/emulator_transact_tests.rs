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

//! XcmTransactor tests using the **real** Moonriver runtime against Westend relay.
//!
//! Covers:
//! - transact_through_sovereign (relay) — basic, fee_payer=None, custom fee/weight, refund
//! - transact_through_derivative (relay) — basic, custom fee/weight, refund
//! - transact_through_signed (relay) — basic, custom fee/weight, refund
//! - HRMP channel management (init/accept/close)

use crate::emulator_network::*;
use frame_support::{
	assert_ok,
	traits::fungible::{Inspect, Mutate},
};
use pallet_xcm_transactor::{Currency, CurrencyPayment, HrmpOperation, TransactWeights};
use parity_scale_codec::Encode;
use sp_core::U256;
use xcm::latest::prelude::*;
use xcm_emulator::{RelayChain, TestExt};
use xcm_executor::traits::ConvertLocation;

const DOT_ASSET_ID: u128 = 1;

// ===========================================================================
// Setup
// ===========================================================================

fn setup_transactor() {
	init_network();

	moonriver_execute_with(|| {
		register_dot_asset(DOT_ASSET_ID);
		set_westend_relay_indices();

		// Configure transact info for the relay destination.
		assert_ok!(moonriver_runtime::XcmTransactor::set_transact_info(
			moonriver_runtime::RuntimeOrigin::root(),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			3_000u64.into(),          // extra_weight (relay charges per instruction)
			20_000_000_000u64.into(), // max_weight
			// 4 instructions in transact_through_signed
			Some(4_000u64.into()),
		));
	});

	// Fund Moonriver's sovereign on relay so it can pay fees for UMP transacts.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		// The sovereign is already funded via relay genesis (endowment).
	});
}

/// Encode a `system::remark_with_event` call for the Westend relay.
fn relay_remark_call() -> Vec<u8> {
	westend_runtime::RuntimeCall::System(
		frame_system::Call::<westend_runtime::Runtime>::remark_with_event {
			remark: b"hello from Moonriver".to_vec(),
		},
	)
	.encode()
}

/// Derive the relay account for a signed XCM origin from a parachain user.
/// The XCM `DescendOrigin(AccountKey20(key))` shifts the origin to
/// `Parachain(para_id)/AccountKey20(key)`, which the relay's `LocationConverter`
/// hashes into a 32-byte account.
fn relay_derived_account(para_id: u32, key: [u8; 20]) -> sp_runtime::AccountId32 {
	let location = Location::new(0, [Parachain(para_id), AccountKey20 { network: None, key }]);
	westend_runtime::xcm_config::LocationConverter::convert_location(&location)
		.expect("Should derive relay account from parachain signed origin")
}

/// Assert that the relay processed a UMP message and emitted a Remarked event.
fn assert_relay_remark_executed() {
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
				westend_runtime::RuntimeEvent::System(frame_system::Event::Remarked { .. })
			)
		});
		assert!(has_remark, "Relay should have emitted a Remarked event");
	});
}

/// Send DOT from relay to Moonriver ALITH.
fn fund_moonriver_alith_with_dot(amount: u128) {
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
	fund_moonriver_alith_with_dot(ONE_DOT * 1000);

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

	moonriver_execute_with(|| {
		assert_ok!(
			moonriver_runtime::XcmTransactor::transact_through_sovereign(
				moonriver_runtime::RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::from(Location::parent())),
				Some(moonriver_runtime::AccountId::from(ALITH)),
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
						Location::parent()
					))),
					fee_amount: Some(ONE_DOT), // explicit fee
				},
				relay_remark_call(),
				OriginKind::SovereignAccount,
				TransactWeights {
					transact_required_weight_at_most: 1_000_000_000u64.into(),
					overall_weight: Some(Limited(2_000_000_000u64.into())),
				},
				false,
			)
		);
	});

	assert_relay_remark_executed();

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

	moonriver_execute_with(|| {
		register_dot_asset(DOT_ASSET_ID);
		set_westend_relay_indices();
	});
	sibling_execute_with(|| {
		register_dot_asset(DOT_ASSET_ID);
		set_westend_relay_indices();
	});

	use pallet_xcm_transactor::{HrmpInitParams, HrmpOperation};

	// Step 1: Moonriver requests to open channel to sibling.
	moonriver_execute_with(|| {
		assert_ok!(moonriver_runtime::XcmTransactor::hrmp_manage(
			moonriver_runtime::RuntimeOrigin::root(),
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
		assert_ok!(moonriver_runtime::XcmTransactor::hrmp_manage(
			moonriver_runtime::RuntimeOrigin::root(),
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
		assert!(has_accept, "Relay should have emitted OpenChannelAccepted");
	});

	// Step 3: Process the pending open requests and verify the channel is established.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(westend_runtime::Hrmp::force_process_hrmp_open(
			westend_runtime::RuntimeOrigin::root(),
			1,
		));

		use polkadot_runtime_parachains::hrmp;
		let channel =
			hrmp::HrmpChannels::<westend_runtime::Runtime>::get(xcm_emulator::HrmpChannelId {
				sender: MOONBEAM_PARA_ID.into(),
				recipient: SIBLING_PARA_ID.into(),
			});
		assert!(
			channel.is_some(),
			"HRMP channel Moonriver → Sibling should be established"
		);
	});
}

// ===========================================================================
// HRMP: close channel via XcmTransactor
// ===========================================================================

#[test]
fn hrmp_close_via_xcm_transactor() {
	init_network();

	moonriver_execute_with(|| {
		register_dot_asset(DOT_ASSET_ID);
		set_westend_relay_indices();
	});

	// Force-open a channel so we can close it.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		open_hrmp_channels(MOONBEAM_PARA_ID, SIBLING_PARA_ID);
	});

	// Close the channel from Moonriver side via XcmTransactor.
	moonriver_execute_with(|| {
		assert_ok!(moonriver_runtime::XcmTransactor::hrmp_manage(
			moonriver_runtime::RuntimeOrigin::root(),
			HrmpOperation::Close(xcm_emulator::HrmpChannelId {
				sender: MOONBEAM_PARA_ID.into(),
				recipient: SIBLING_PARA_ID.into(),
			}),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent(),
				))),
				fee_amount: Some(ONE_DOT * 100),
			},
			TransactWeights {
				transact_required_weight_at_most: 5_000_000_000u64.into(),
				overall_weight: Some(Limited(10_000_000_000u64.into())),
			},
		));
	});

	// Verify the close event on relay.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		let events = westend_runtime::System::events();
		let has_close = events.iter().any(|e| {
			matches!(
				&e.event,
				westend_runtime::RuntimeEvent::Hrmp(
					polkadot_runtime_parachains::hrmp::Event::ChannelClosed { .. }
				)
			)
		});
		assert!(has_close, "Relay should have emitted ChannelClosed");
	});
}

// ===========================================================================
// Transact through sovereign: fee_payer = None
// ===========================================================================

#[test]
fn transact_through_sovereign_fee_payer_none() {
	setup_transactor();

	// With fee_payer = None, no local withdraw happens — only the sovereign on
	// relay pays. The sovereign must be funded from genesis.
	moonriver_execute_with(|| {
		assert_ok!(
			moonriver_runtime::XcmTransactor::transact_through_sovereign(
				moonriver_runtime::RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::from(Location::parent())),
				None, // no fee payer — no local withdraw
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
						Location::parent(),
					))),
					fee_amount: Some(ONE_DOT),
				},
				relay_remark_call(),
				OriginKind::SovereignAccount,
				TransactWeights {
					transact_required_weight_at_most: 1_000_000_000u64.into(),
					overall_weight: Some(Limited(2_000_000_000u64.into())),
				},
				false,
			)
		);
	});

	assert_relay_remark_executed();
}

// ===========================================================================
// Transact through sovereign: custom fee & weight (no refund)
// ===========================================================================

#[test]
fn transact_through_sovereign_custom_fee_weight() {
	setup_transactor();
	fund_moonriver_alith_with_dot(ONE_DOT * 1000);

	moonriver_execute_with(|| {
		assert_ok!(
			moonriver_runtime::XcmTransactor::transact_through_sovereign(
				moonriver_runtime::RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::from(Location::parent())),
				Some(moonriver_runtime::AccountId::from(ALITH)),
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
						Location::parent(),
					))),
					fee_amount: Some(ONE_DOT * 5), // explicit larger fee
				},
				relay_remark_call(),
				OriginKind::SovereignAccount,
				TransactWeights {
					transact_required_weight_at_most: 2_000_000_000u64.into(),
					overall_weight: Some(Limited(4_000_000_000u64.into())),
				},
				false,
			)
		);
	});

	assert_relay_remark_executed();
}

// ===========================================================================
// Transact through sovereign: custom fee, weight & refund
// ===========================================================================

#[test]
fn transact_through_sovereign_custom_fee_weight_refund() {
	setup_transactor();
	fund_moonriver_alith_with_dot(ONE_DOT * 1000);

	let sovereign_before = WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		let sovereign = WestendRelay::<PolkadotMoonbeamNet>::sovereign_account_id_of(
			Location::new(0, [Parachain(MOONBEAM_PARA_ID)]),
		);
		<westend_runtime::Balances as Inspect<_>>::balance(&sovereign)
	});

	moonriver_execute_with(|| {
		assert_ok!(
			moonriver_runtime::XcmTransactor::transact_through_sovereign(
				moonriver_runtime::RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::from(Location::parent())),
				Some(moonriver_runtime::AccountId::from(ALITH)),
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
						Location::parent(),
					))),
					fee_amount: Some(ONE_DOT * 10), // overpay to test refund
				},
				relay_remark_call(),
				OriginKind::SovereignAccount,
				TransactWeights {
					transact_required_weight_at_most: 2_000_000_000u64.into(),
					overall_weight: Some(Limited(4_000_000_000u64.into())),
				},
				true, // refund = true
			)
		);
	});

	assert_relay_remark_executed();

	// With refund=true, leftover fees are deposited back to the sovereign.
	// The sovereign should have lost less than the full 10 DOT fee.
	let sovereign_after = WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		let sovereign = WestendRelay::<PolkadotMoonbeamNet>::sovereign_account_id_of(
			Location::new(0, [Parachain(MOONBEAM_PARA_ID)]),
		);
		<westend_runtime::Balances as Inspect<_>>::balance(&sovereign)
	});
	let fee_spent = sovereign_before.saturating_sub(sovereign_after);
	assert!(
		fee_spent < ONE_DOT * 10,
		"With refund, sovereign should spend less than the full fee: spent={fee_spent}"
	);
}

// ===========================================================================
// Transact through signed (para → relay)
// ===========================================================================

#[test]
fn transact_through_signed_to_relay() {
	setup_transactor();
	fund_moonriver_alith_with_dot(ONE_DOT * 1000);

	let derived_account = relay_derived_account(MOONBEAM_PARA_ID, ALITH);

	// Fund the derived account on relay so it can pay XCM fees.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(westend_runtime::Balances::transfer_allow_death(
			westend_runtime::RuntimeOrigin::signed(RELAY_ALICE),
			derived_account.clone().into(),
			ONE_DOT * 100,
		));
	});

	moonriver_execute_with(|| {
		assert_ok!(moonriver_runtime::XcmTransactor::transact_through_signed(
			moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent(),
				))),
				fee_amount: Some(ONE_DOT * 10),
			},
			relay_remark_call(),
			TransactWeights {
				transact_required_weight_at_most: 1_000_000_000u64.into(),
				overall_weight: Some(Limited(4_000_000_000u64.into())),
			},
			false,
		));
	});

	assert_relay_remark_executed();
}

// ===========================================================================
// Transact through signed: custom fee & weight
// ===========================================================================

#[test]
fn transact_through_signed_custom_fee_weight() {
	setup_transactor();
	fund_moonriver_alith_with_dot(ONE_DOT * 1000);

	let derived_account = relay_derived_account(MOONBEAM_PARA_ID, ALITH);

	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(westend_runtime::Balances::transfer_allow_death(
			westend_runtime::RuntimeOrigin::signed(RELAY_ALICE),
			derived_account.clone().into(),
			ONE_DOT * 100,
		));
	});

	moonriver_execute_with(|| {
		assert_ok!(moonriver_runtime::XcmTransactor::transact_through_signed(
			moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent(),
				))),
				fee_amount: Some(ONE_DOT * 5),
			},
			relay_remark_call(),
			TransactWeights {
				transact_required_weight_at_most: 2_000_000_000u64.into(),
				overall_weight: Some(Limited(6_000_000_000u64.into())),
			},
			false,
		));
	});

	assert_relay_remark_executed();
}

// ===========================================================================
// Transact through signed: custom fee, weight & refund
// ===========================================================================

#[test]
fn transact_through_signed_custom_fee_weight_refund() {
	setup_transactor();
	fund_moonriver_alith_with_dot(ONE_DOT * 1000);

	let derived_account = relay_derived_account(MOONBEAM_PARA_ID, ALITH);

	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(westend_runtime::Balances::transfer_allow_death(
			westend_runtime::RuntimeOrigin::signed(RELAY_ALICE),
			derived_account.clone().into(),
			ONE_DOT * 100,
		));
	});

	let derived_before = WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		<westend_runtime::Balances as Inspect<_>>::balance(&derived_account)
	});

	moonriver_execute_with(|| {
		assert_ok!(moonriver_runtime::XcmTransactor::transact_through_signed(
			moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
			Box::new(xcm::VersionedLocation::from(Location::parent())),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent(),
				))),
				fee_amount: Some(ONE_DOT * 20), // overpay
			},
			relay_remark_call(),
			TransactWeights {
				transact_required_weight_at_most: 2_000_000_000u64.into(),
				overall_weight: Some(Limited(6_000_000_000u64.into())),
			},
			true, // refund = true
		));
	});

	assert_relay_remark_executed();

	// With refund, the derived account should get surplus back.
	let derived_after = WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		<westend_runtime::Balances as Inspect<_>>::balance(&derived_account)
	});
	let fee_spent = derived_before.saturating_sub(derived_after);
	assert!(
		fee_spent < ONE_DOT * 20,
		"With refund, derived account should spend less than the full fee: spent={fee_spent}"
	);
}

// ===========================================================================
// Transact through derivative
// ===========================================================================

/// Setup for derivative transact tests.
/// Registers ALITH as the owner of derivative index 0 and funds the derivative
/// sub-account on the relay.
fn setup_derivative() {
	setup_transactor();
	fund_moonriver_alith_with_dot(ONE_DOT * 1000);

	let derivative_index: u16 = 0;

	// Register ALITH as the owner of index 0.
	moonriver_execute_with(|| {
		assert_ok!(moonriver_runtime::XcmTransactor::register(
			moonriver_runtime::RuntimeOrigin::root(),
			moonriver_runtime::AccountId::from(ALITH),
			derivative_index,
		));
	});

	// Fund the derivative account on relay.
	// The derivative is computed from the sovereign account of Moonriver parachain.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		let sovereign = WestendRelay::<PolkadotMoonbeamNet>::sovereign_account_id_of(
			Location::new(0, [Parachain(MOONBEAM_PARA_ID)]),
		);
		let derivative = pallet_utility::Pallet::<westend_runtime::Runtime>::derivative_account_id(
			sovereign,
			derivative_index,
		);
		assert_ok!(westend_runtime::Balances::transfer_allow_death(
			westend_runtime::RuntimeOrigin::signed(RELAY_ALICE),
			derivative.into(),
			ONE_DOT * 100,
		));
	});
}

#[test]
fn transact_through_derivative_to_relay() {
	setup_derivative();

	moonriver_execute_with(|| {
		assert_ok!(
			moonriver_runtime::XcmTransactor::transact_through_derivative(
				moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH),),
				moonriver_runtime::xcm_config::Transactors::Relay,
				0u16, // derivative index
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
						Location::parent()
					),)),
					fee_amount: Some(ONE_DOT * 10),
				},
				// Inner call (unwrapped — the pallet wraps it in as_derivative).
				relay_remark_call(),
				TransactWeights {
					transact_required_weight_at_most: 2_000_000_000u64.into(),
					overall_weight: Some(Limited(4_000_000_000u64.into())),
				},
				false,
			)
		);
	});

	assert_relay_remark_executed();
}

#[test]
fn transact_through_derivative_custom_fee_weight() {
	setup_derivative();

	moonriver_execute_with(|| {
		assert_ok!(
			moonriver_runtime::XcmTransactor::transact_through_derivative(
				moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH),),
				moonriver_runtime::xcm_config::Transactors::Relay,
				0u16,
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
						Location::parent()
					),)),
					fee_amount: Some(ONE_DOT * 5),
				},
				relay_remark_call(),
				TransactWeights {
					transact_required_weight_at_most: 3_000_000_000u64.into(),
					overall_weight: Some(Limited(6_000_000_000u64.into())),
				},
				false,
			)
		);
	});

	assert_relay_remark_executed();
}

#[test]
fn transact_through_derivative_custom_fee_weight_refund() {
	setup_derivative();

	let sovereign_before = WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		let sovereign = WestendRelay::<PolkadotMoonbeamNet>::sovereign_account_id_of(
			Location::new(0, [Parachain(MOONBEAM_PARA_ID)]),
		);
		<westend_runtime::Balances as Inspect<_>>::balance(&sovereign)
	});

	moonriver_execute_with(|| {
		assert_ok!(
			moonriver_runtime::XcmTransactor::transact_through_derivative(
				moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH),),
				moonriver_runtime::xcm_config::Transactors::Relay,
				0u16,
				CurrencyPayment {
					currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
						Location::parent()
					),)),
					fee_amount: Some(ONE_DOT * 20), // overpay
				},
				relay_remark_call(),
				TransactWeights {
					transact_required_weight_at_most: 2_000_000_000u64.into(),
					overall_weight: Some(Limited(4_000_000_000u64.into())),
				},
				true, // refund
			)
		);
	});

	assert_relay_remark_executed();

	// With refund, surplus should be deposited back to the sovereign (SelfLocation).
	let sovereign_after = WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		let sovereign = WestendRelay::<PolkadotMoonbeamNet>::sovereign_account_id_of(
			Location::new(0, [Parachain(MOONBEAM_PARA_ID)]),
		);
		<westend_runtime::Balances as Inspect<_>>::balance(&sovereign)
	});
	let fee_spent = sovereign_before.saturating_sub(sovereign_after);
	assert!(
		fee_spent < ONE_DOT * 20,
		"With refund, sovereign should spend less than the full fee: spent={fee_spent}"
	);
}

// ===========================================================================
// Transact through signed: para → para
// ===========================================================================

/// Setup for para-to-para transact tests via signed origin.
/// Opens HRMP channels between Moonriver and Sibling, registers DOT on both,
/// and funds the derived account on the sibling.
fn setup_para_to_para_signed() -> moonriver_runtime::AccountId {
	init_network();

	// Register DOT + relay indices on Moonriver.
	moonriver_execute_with(|| {
		register_dot_asset(DOT_ASSET_ID);
		set_westend_relay_indices();
	});

	// Open HRMP channels.
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		open_hrmp_channels(MOONBEAM_PARA_ID, SIBLING_PARA_ID);
	});

	// Register DOT on sibling so it can accept DOT as XCM fee.
	sibling_execute_with(|| {
		register_dot_asset(DOT_ASSET_ID);
	});

	// Compute the derived account on the sibling for ALITH's signed origin from Moonriver.
	// After DescendOrigin(AccountKey20(ALITH)), the sibling sees origin
	// Location::new(1, [Parachain(2004), AccountKey20(ALITH)]).
	let derived_on_sibling: moonriver_runtime::AccountId = sibling_execute_with(|| {
		<moonriver_runtime::xcm_config::LocationToAccountId as ConvertLocation<
			moonriver_runtime::AccountId,
		>>::convert_location(&Location::new(
			1,
			[
				Parachain(MOONBEAM_PARA_ID),
				AccountKey20 {
					network: None,
					key: ALITH,
				},
			],
		))
		.expect("Should derive sibling account for Moonriver ALITH")
	});

	// Fund the derived account on sibling with DOT (relay → sibling DMP).
	WestendRelay::<PolkadotMoonbeamNet>::execute_with(|| {
		assert_ok!(
			westend_runtime::XcmPallet::transfer_assets_using_type_and_then(
				westend_runtime::RuntimeOrigin::signed(RELAY_ALICE),
				Box::new(xcm::VersionedLocation::from(Location::new(
					0,
					[Parachain(SIBLING_PARA_ID)],
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
							key: derived_on_sibling.into(),
						}],
					),
				}]))),
				WeightLimit::Unlimited,
			)
		);
	});

	// Verify the derived account received DOT.
	sibling_execute_with(|| {
		let balance =
			moonriver_runtime::EvmForeignAssets::balance(DOT_ASSET_ID, derived_on_sibling).unwrap();
		assert!(
			balance > sp_core::U256::zero(),
			"Derived account on sibling should have DOT"
		);
	});

	derived_on_sibling
}

/// Encode a `system::remark_with_event` call for the sibling (Moonriver runtime).
fn sibling_remark_call() -> Vec<u8> {
	moonriver_runtime::RuntimeCall::System(
		frame_system::Call::<moonriver_runtime::Runtime>::remark_with_event {
			remark: b"hello from Moonriver to sibling".to_vec(),
		},
	)
	.encode()
}

/// Assert that the sibling processed the HRMP transact and emitted a Remarked event.
fn assert_sibling_remark_executed() {
	sibling_execute_with(|| {
		let events = moonriver_runtime::System::events();

		let has_remark = events.iter().any(|e| {
			matches!(
				&e.event,
				moonriver_runtime::RuntimeEvent::System(frame_system::Event::Remarked { .. })
			)
		});
		assert!(
			has_remark,
			"Sibling should have emitted Remarked event from transact"
		);
	});
}

#[test]
fn transact_through_signed_para_to_para() {
	setup_para_to_para_signed();

	moonriver_execute_with(|| {
		assert_ok!(moonriver_runtime::XcmTransactor::transact_through_signed(
			moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
			Box::new(xcm::VersionedLocation::from(Location::new(
				1,
				[Parachain(SIBLING_PARA_ID)],
			))),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent(),
				))),
				fee_amount: Some(ONE_DOT * 10),
			},
			sibling_remark_call(),
			TransactWeights {
				transact_required_weight_at_most: 1_000_000_000u64.into(),
				overall_weight: Some(Limited(4_000_000_000u64.into())),
			},
			false,
		));
	});

	assert_sibling_remark_executed();
}

#[test]
fn transact_through_signed_para_to_para_refund() {
	let derived_on_sibling = setup_para_to_para_signed();

	let dot_before = sibling_execute_with(|| {
		moonriver_runtime::EvmForeignAssets::balance(DOT_ASSET_ID, derived_on_sibling).unwrap()
	});

	moonriver_execute_with(|| {
		assert_ok!(moonriver_runtime::XcmTransactor::transact_through_signed(
			moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
			Box::new(xcm::VersionedLocation::from(Location::new(
				1,
				[Parachain(SIBLING_PARA_ID)],
			))),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent(),
				))),
				fee_amount: Some(ONE_DOT * 20), // overpay
			},
			sibling_remark_call(),
			TransactWeights {
				transact_required_weight_at_most: 1_000_000_000u64.into(),
				// Refund appendix (RefundSurplus + DepositAsset) needs extra weight.
				overall_weight: Some(Limited(8_000_000_000u64.into())),
			},
			true, // refund = true
		));
	});

	assert_sibling_remark_executed();

	// With refund, the derived account should get surplus back.
	let dot_after = sibling_execute_with(|| {
		moonriver_runtime::EvmForeignAssets::balance(DOT_ASSET_ID, derived_on_sibling).unwrap()
	});
	let spent = dot_before.saturating_sub(dot_after);
	assert!(
		spent < sp_core::U256::from(ONE_DOT * 20),
		"With refund, derived account should spend less than the full 20 DOT fee: spent={spent}"
	);
}

// ===========================================================================
// Transact through signed: para → para (EthereumXcm)
// ===========================================================================

/// Common setup for Ethereum XCM transact tests.
/// Returns the derived account on the sibling.
fn setup_para_to_para_ethereum() -> moonriver_runtime::AccountId {
	let derived_on_sibling = setup_para_to_para_signed();

	// The derived account needs GLMR on the sibling for EVM value transfers.
	sibling_execute_with(|| {
		<moonriver_runtime::Balances as Mutate<_>>::mint_into(
			&derived_on_sibling,
			moonriver_runtime::currency::MOVR * 10,
		)
		.expect("Should mint GLMR for derived account on sibling");
	});

	derived_on_sibling
}

/// Encode an `EthereumXcm::transact` call that does an EVM value transfer.
fn ethereum_xcm_transfer_call(recipient: sp_core::H160, value: u128) -> Vec<u8> {
	use sp_runtime::BoundedVec;

	let eth_tx =
		xcm_primitives::EthereumXcmTransaction::V1(xcm_primitives::EthereumXcmTransactionV1 {
			gas_limit: U256::from(21000),
			fee_payment: xcm_primitives::EthereumXcmFee::Auto,
			action: pallet_ethereum::TransactionAction::Call(recipient),
			value: U256::from(value),
			input: BoundedVec::<
				u8,
				sp_core::ConstU32<{ xcm_primitives::MAX_ETHEREUM_XCM_INPUT_SIZE }>,
			>::try_from(vec![])
			.unwrap(),
			access_list: None,
		});

	moonriver_runtime::RuntimeCall::EthereumXcm(pallet_ethereum_xcm::Call::<
		moonriver_runtime::Runtime,
	>::transact {
		xcm_transaction: eth_tx,
	})
	.encode()
}

/// EVM transfer to ALITH on sibling via EthereumXcm::transact.
#[test]
fn transact_through_signed_para_to_para_ethereum() {
	let _derived = setup_para_to_para_ethereum();

	let transfer_value = 100u128;
	let alith_h160 = sp_core::H160::from(ALITH);

	let alith_balance_before = sibling_execute_with(|| {
		<moonriver_runtime::Balances as Inspect<_>>::balance(&moonriver_runtime::AccountId::from(
			ALITH,
		))
	});

	moonriver_execute_with(|| {
		assert_ok!(moonriver_runtime::XcmTransactor::transact_through_signed(
			moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
			Box::new(xcm::VersionedLocation::from(Location::new(
				1,
				[Parachain(SIBLING_PARA_ID)],
			))),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent(),
				))),
				fee_amount: Some(ONE_DOT * 10),
			},
			ethereum_xcm_transfer_call(alith_h160, transfer_value),
			TransactWeights {
				transact_required_weight_at_most: 4_000_000_000u64.into(),
				overall_weight: Some(Limited(8_000_000_000u64.into())),
			},
			false,
		));
	});

	let alith_balance_after = sibling_execute_with(|| {
		<moonriver_runtime::Balances as Inspect<_>>::balance(&moonriver_runtime::AccountId::from(
			ALITH,
		))
	});
	assert_eq!(
		alith_balance_after - alith_balance_before,
		transfer_value,
		"ALITH should receive {transfer_value} WEI on sibling via EthereumXcm transact"
	);
}

/// EthereumXcm::transact_through_proxy fails without a proxy set up.
#[test]
fn transact_through_signed_para_to_para_ethereum_no_proxy_fails() {
	let _derived = setup_para_to_para_ethereum();

	let alith_h160 = sp_core::H160::from(ALITH);
	let transfer_value = 100u128;

	// Encode a transact_through_proxy call without any proxy being set.
	let eth_tx =
		xcm_primitives::EthereumXcmTransaction::V2(xcm_primitives::EthereumXcmTransactionV2 {
			gas_limit: U256::from(21000),
			action: pallet_ethereum::TransactionAction::Call(alith_h160),
			value: U256::from(transfer_value),
			input: sp_runtime::BoundedVec::try_from(vec![]).unwrap(),
			access_list: None,
		});

	let proxy_call = moonriver_runtime::RuntimeCall::EthereumXcm(pallet_ethereum_xcm::Call::<
		moonriver_runtime::Runtime,
	>::transact_through_proxy {
		transact_as: alith_h160,
		xcm_transaction: eth_tx,
	})
	.encode();

	let alith_balance_before = sibling_execute_with(|| {
		<moonriver_runtime::Balances as Inspect<_>>::balance(&moonriver_runtime::AccountId::from(
			ALITH,
		))
	});

	moonriver_execute_with(|| {
		assert_ok!(moonriver_runtime::XcmTransactor::transact_through_signed(
			moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
			Box::new(xcm::VersionedLocation::from(Location::new(
				1,
				[Parachain(SIBLING_PARA_ID)],
			))),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent(),
				))),
				fee_amount: Some(ONE_DOT * 10),
			},
			proxy_call,
			TransactWeights {
				transact_required_weight_at_most: 4_000_000_000u64.into(),
				overall_weight: Some(Limited(8_000_000_000u64.into())),
			},
			false,
		));
	});

	// The EVM transfer should NOT have happened (proxy not set).
	let alith_balance_after = sibling_execute_with(|| {
		<moonriver_runtime::Balances as Inspect<_>>::balance(&moonriver_runtime::AccountId::from(
			ALITH,
		))
	});
	assert_eq!(
		alith_balance_after, alith_balance_before,
		"ALITH balance should be unchanged — transact_through_proxy should fail without proxy"
	);
}

/// EthereumXcm::transact_through_proxy succeeds with a proxy set up.
#[test]
fn transact_through_signed_para_to_para_ethereum_proxy_succeeds() {
	let derived = setup_para_to_para_ethereum();

	let recipient: [u8; 20] = [42u8; 20];
	let transfer_value = 100u128;

	// Set up proxy: ALITH delegates to the derived account on the sibling.
	sibling_execute_with(|| {
		assert_ok!(moonriver_runtime::Proxy::add_proxy(
			moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
			derived,
			moonriver_runtime::ProxyType::Any,
			0,
		));
	});

	let recipient_balance_before = sibling_execute_with(|| {
		<moonriver_runtime::Balances as Inspect<_>>::balance(&moonriver_runtime::AccountId::from(
			recipient,
		))
	});

	// Encode a transact_through_proxy call targeting ALITH as proxy principal,
	// EVM transfer to `recipient`.
	let eth_tx =
		xcm_primitives::EthereumXcmTransaction::V2(xcm_primitives::EthereumXcmTransactionV2 {
			gas_limit: U256::from(21000),
			action: pallet_ethereum::TransactionAction::Call(sp_core::H160::from(recipient)),
			value: U256::from(transfer_value),
			input: sp_runtime::BoundedVec::try_from(vec![]).unwrap(),
			access_list: None,
		});

	let proxy_call = moonriver_runtime::RuntimeCall::EthereumXcm(pallet_ethereum_xcm::Call::<
		moonriver_runtime::Runtime,
	>::transact_through_proxy {
		transact_as: sp_core::H160::from(ALITH),
		xcm_transaction: eth_tx,
	})
	.encode();

	moonriver_execute_with(|| {
		assert_ok!(moonriver_runtime::XcmTransactor::transact_through_signed(
			moonriver_runtime::RuntimeOrigin::signed(moonriver_runtime::AccountId::from(ALITH)),
			Box::new(xcm::VersionedLocation::from(Location::new(
				1,
				[Parachain(SIBLING_PARA_ID)],
			))),
			CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::from(
					Location::parent(),
				))),
				fee_amount: Some(ONE_DOT * 10),
			},
			proxy_call,
			TransactWeights {
				transact_required_weight_at_most: 4_000_000_000u64.into(),
				overall_weight: Some(Limited(8_000_000_000u64.into())),
			},
			false,
		));
	});

	let recipient_balance_after = sibling_execute_with(|| {
		<moonriver_runtime::Balances as Inspect<_>>::balance(&moonriver_runtime::AccountId::from(
			recipient,
		))
	});
	assert_eq!(
		recipient_balance_after - recipient_balance_before,
		transfer_value,
		"Recipient should receive {transfer_value} WEI via EthereumXcm proxy transact"
	);
}
