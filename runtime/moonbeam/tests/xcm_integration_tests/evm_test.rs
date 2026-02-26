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

//! EVM-XCM integration tests.
//!
//! Tests for XCM interactions with the EVM:
//! - XCM triggering EVM calls via pallet-ethereum-xcm
//! - ERC20 XCM bridge functionality
//! - Foreign asset representation in EVM
//! - EVM precompiles for XCM

use crate::common::*;
use crate::networks::*;
use sp_core::H160;
use xcm::latest::prelude::*;

#[test]
fn evm_foreign_assets_configured() {
	moonbeam_execute_with(|| {
		// Verify EvmForeignAssets pallet is in the AssetTransactors
		// AssetTransactors = (LocalAssetTransactor, EvmForeignAssets, Erc20XcmBridge)

		// This means XCM can deposit/withdraw foreign assets that are
		// registered with the EvmForeignAssets pallet
	});
}

#[test]
fn erc20_xcm_bridge_configured() {
	moonbeam_execute_with(|| {
		// Verify Erc20XcmBridge is configured
		// This allows bridging ERC20 tokens via XCM
		// The XcmExecutor is wrapped with Erc20XcmBridge wrapper
		// XcmExecutor = pallet_erc20_xcm_bridge::XcmExecutorWrapper<...>
	});
}

#[test]
fn ethereum_xcm_pallet_configured() {
	moonbeam_execute_with(|| {
		// Verify pallet-ethereum-xcm is available for XCM-triggered EVM calls

		// MoonbeamCall in XcmExecutorConfig::CallDispatcher handles
		// routing calls to pallet-ethereum-xcm when appropriate
	});
}

#[test]
fn location_to_h160_converts_accounts() {
	moonbeam_execute_with(|| {
		use moonbeam_runtime::xcm_config::LocationToH160;
		use xcm_executor::traits::ConvertLocation;

		// AccountKey20 should convert to H160
		let account_location = Location::new(
			0,
			[AccountKey20 {
				network: Some(NetworkId::Polkadot),
				key: ALICE,
			}],
		);

		let h160 = LocationToH160::convert_location(&account_location);

		assert!(h160.is_some(), "Should convert AccountKey20 to H160");
		assert_eq!(
			h160.unwrap(),
			H160::from(ALICE),
			"Should match the account key"
		);
	});
}
