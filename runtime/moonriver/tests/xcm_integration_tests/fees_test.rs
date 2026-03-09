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

//! XCM fee integration tests.
//!
//! Tests for fee calculation and payment in XCM:
//! - Fee destination configuration
//! - Trader fee calculation
//! - Multi-asset fee support

use crate::networks::*;
use moonriver_runtime::{Runtime, Treasury};
use sp_weights::Weight;
use xcm::latest::prelude::*;

#[test]
fn xcm_fees_go_to_treasury() {
	moonriver_execute_with(|| {
		// Verify XcmFeesAccount points to Treasury
		use moonriver_runtime::xcm_config::XcmFeesAccount;

		let fee_account = XcmFeesAccount::get();
		let treasury_account = Treasury::account_id();

		assert_eq!(
			fee_account, treasury_account,
			"XCM fees should go to treasury"
		);
	});
}

#[test]
fn trader_computes_fees_for_weight() {
	moonriver_execute_with(|| {
		use frame_support::traits::PalletInfoAccess;
		use moonriver_runtime::Balances;
		use pallet_xcm_weight_trader::Pallet as XcmWeightTrader;
		use xcm::VersionedAssetId;

		// Native token location
		let native_location = Location::new(0, [PalletInstance(Balances::index() as u8)]);

		// Compute fee for some weight using the public query API
		let weight = Weight::from_parts(1_000_000_000, 64 * 1024);
		let versioned_asset_id = VersionedAssetId::V5(AssetId(native_location));
		let fee = XcmWeightTrader::<Runtime>::query_weight_to_asset_fee(weight, versioned_asset_id);

		assert!(fee.is_ok(), "Should compute fee for native token");
		let fee_amount = fee.unwrap();
		assert!(fee_amount > 0, "Fee should be non-zero for non-zero weight");
	});
}
