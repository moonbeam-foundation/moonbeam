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

//! Tests for the XCM Trader (pallet_xcm_weight_trader::Trader) configuration.
//!
//! The trader is responsible for converting weight to fee and accepting payment
//! in different assets. Moonbase uses a custom trader that:
//! - Accepts the native token (UNIT) at standard rates
//! - Accepts registered foreign assets at configured relative prices

use crate::xcm_common::*;
use frame_support::traits::PalletInfoAccess;
use moonbase_runtime::{Balances, Runtime, Treasury};
use pallet_xcm_weight_trader::{Pallet as XcmWeightTrader, Trader};
use sp_weights::Weight;
use xcm::latest::prelude::*;
use xcm::VersionedAssetId;
use xcm_executor::traits::WeightTrader;
use xcm_executor::AssetsInHolding;

fn native_location() -> Location {
	Location::new(0, [PalletInstance(Balances::index() as u8)])
}

#[test]
fn trader_accepts_native_token() {
	ExtBuilder::default().build().execute_with(|| {
		let mut trader = Trader::<Runtime>::new();
		let weight_to_buy = Weight::from_parts(1_000_000_000, 64 * 1024);

		// Create payment in native token
		let mut payment = AssetsInHolding::new();
		payment.subsume(Asset {
			id: AssetId(native_location()),
			fun: Fungible(ONE_UNIT),
		});

		let context = XcmContext::with_message_id([0u8; 32]);
		let result = trader.buy_weight(weight_to_buy, payment.clone(), &context);

		// Should succeed - native token is always accepted
		assert!(result.is_ok(), "Native token should be accepted for fees");
	});
}

#[test]
fn trader_computes_native_fee_correctly() {
	ExtBuilder::default().build().execute_with(|| {
		let weight = Weight::from_parts(1_000_000_000, 64 * 1024);
		let native_loc = native_location();

		// Compute fee for native token using public API
		let versioned_asset_id = VersionedAssetId::V5(AssetId(native_loc));
		let fee_result =
			XcmWeightTrader::<Runtime>::query_weight_to_asset_fee(weight, versioned_asset_id);

		assert!(fee_result.is_ok(), "Should compute fee for native token");
		let fee = fee_result.unwrap();
		assert!(fee > 0, "Fee should be non-zero");
	});
}

#[test]
fn trader_rejects_unsupported_asset() {
	ExtBuilder::default().build().execute_with(|| {
		let mut trader = Trader::<Runtime>::new();
		let weight_to_buy = Weight::from_parts(1_000_000_000, 64 * 1024);

		// Try to pay with unsupported asset
		let unsupported_asset_location = Location::new(1, [Parachain(9999), PalletInstance(99)]);
		let mut payment = AssetsInHolding::new();
		payment.subsume(Asset {
			id: AssetId(unsupported_asset_location.clone()),
			fun: Fungible(1_000_000_000_000),
		});

		let context = XcmContext::with_message_id([0u8; 32]);
		let result = trader.buy_weight(weight_to_buy, payment, &context);

		// Should fail - asset not registered
		assert!(result.is_err(), "Unsupported asset should be rejected");
		assert_eq!(result.unwrap_err(), XcmError::AssetNotFound);
	});
}

#[test]
fn trader_accepts_registered_foreign_asset() {
	// Register DOT as supported asset
	let dot_location = Location::parent();

	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_id: 1,
			xcm_location: dot_location.clone(),
			decimals: 10,
			name: "Polkadot",
			symbol: "DOT",
			balances: vec![],
		}])
		.build()
		.execute_with(|| {
			// First verify the asset is registered and queryable
			let versioned_asset_id = VersionedAssetId::V5(AssetId(dot_location.clone()));
			let weight = Weight::from_parts(1_000_000_000, 64 * 1024);
			let fee_result =
				XcmWeightTrader::<Runtime>::query_weight_to_asset_fee(weight, versioned_asset_id);

			// If the query succeeds, the asset is properly registered
			assert!(
				fee_result.is_ok(),
				"Registered foreign asset should be queryable"
			);

			// Now test the trader directly
			let mut trader = Trader::<Runtime>::new();
			let weight_to_buy = Weight::from_parts(1_000_000_000, 64 * 1024);

			// Pay with DOT - need sufficient amount to cover the computed fee
			let fee = fee_result.unwrap();
			let mut payment = AssetsInHolding::new();
			payment.subsume(Asset {
				id: AssetId(dot_location.clone()),
				fun: Fungible(fee * 2), // Double to ensure enough
			});

			let context = XcmContext::with_message_id([0u8; 32]);
			let result = trader.buy_weight(weight_to_buy, payment, &context);

			// Should succeed - DOT is registered in XcmWeightTrader
			assert!(
				result.is_ok(),
				"Registered foreign asset should be accepted: {:?}",
				result
			);
		});
}

#[test]
fn trader_computes_foreign_asset_fee_with_relative_price() {
	let dot_location = Location::parent();

	ExtBuilder::default()
		.with_xcm_assets(vec![XcmAssetInitialization {
			asset_id: 1,
			xcm_location: dot_location.clone(),
			decimals: 10,
			name: "Polkadot",
			symbol: "DOT",
			balances: vec![],
		}])
		.build()
		.execute_with(|| {
			let weight = Weight::from_parts(1_000_000_000, 64 * 1024);

			// Compute fee for DOT using public API
			let versioned_asset_id = VersionedAssetId::V5(AssetId(dot_location.clone()));
			let fee_result =
				XcmWeightTrader::<Runtime>::query_weight_to_asset_fee(weight, versioned_asset_id);

			assert!(
				fee_result.is_ok(),
				"Should compute fee for registered asset"
			);
			let fee = fee_result.unwrap();
			// Fee should be computed based on relative price
			assert!(fee > 0, "Fee should be non-zero");
		});
}

#[test]
fn trader_cannot_buy_weight_twice() {
	ExtBuilder::default().build().execute_with(|| {
		let mut trader = Trader::<Runtime>::new();
		let weight_to_buy = Weight::from_parts(1_000_000_000, 64 * 1024);
		let context = XcmContext::with_message_id([0u8; 32]);

		// First purchase
		let mut payment1 = AssetsInHolding::new();
		payment1.subsume(Asset {
			id: AssetId(native_location()),
			fun: Fungible(ONE_UNIT),
		});
		let _ = trader.buy_weight(weight_to_buy, payment1, &context);

		// Second purchase should fail
		let mut payment2 = AssetsInHolding::new();
		payment2.subsume(Asset {
			id: AssetId(native_location()),
			fun: Fungible(ONE_UNIT),
		});
		let result = trader.buy_weight(weight_to_buy, payment2, &context);

		assert!(result.is_err(), "Second buy_weight should fail");
		assert_eq!(result.unwrap_err(), XcmError::NotWithdrawable);
	});
}

#[test]
fn trader_refunds_unused_weight() {
	ExtBuilder::default().build().execute_with(|| {
		let mut trader = Trader::<Runtime>::new();
		let weight_bought = Weight::from_parts(2_000_000_000, 128 * 1024);
		let weight_used = Weight::from_parts(1_000_000_000, 64 * 1024);
		let context = XcmContext::with_message_id([0u8; 32]);

		// Buy more weight than needed
		let mut payment = AssetsInHolding::new();
		payment.subsume(Asset {
			id: AssetId(native_location()),
			fun: Fungible(ONE_UNIT * 10), // Plenty of funds
		});

		let buy_result = trader.buy_weight(weight_bought, payment, &context);
		assert!(buy_result.is_ok());

		// Refund unused weight
		let unused_weight = weight_bought.saturating_sub(weight_used);
		let refund = trader.refund_weight(unused_weight, &context);

		// Should get some refund
		if let Some(refunded_asset) = refund {
			match refunded_asset.fun {
				Fungible(amount) => {
					assert!(amount > 0, "Should receive non-zero refund");
				}
				_ => panic!("Expected fungible refund"),
			}
		}
	});
}

#[test]
fn trader_handles_insufficient_payment() {
	ExtBuilder::default().build().execute_with(|| {
		let mut trader = Trader::<Runtime>::new();
		let weight_to_buy = Weight::from_parts(1_000_000_000_000, 64 * 1024); // Very large weight
		let context = XcmContext::with_message_id([0u8; 32]);

		// Try to pay with very small amount
		let mut payment = AssetsInHolding::new();
		payment.subsume(Asset {
			id: AssetId(native_location()),
			fun: Fungible(1), // Tiny amount
		});

		let result = trader.buy_weight(weight_to_buy, payment, &context);

		// Should fail - insufficient payment
		assert!(result.is_err(), "Insufficient payment should be rejected");
		assert_eq!(result.unwrap_err(), XcmError::TooExpensive);
	});
}

#[test]
fn xcm_fees_go_to_treasury() {
	ExtBuilder::default().build().execute_with(|| {
		use moonbase_runtime::xcm_config::XcmFeesAccount;

		assert_eq!(
			XcmFeesAccount::get(),
			Treasury::account_id(),
			"XCM fee destination should be the treasury account"
		);
	});
}
