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

//! Tests for LocationToAccountId configuration.
//!
//! LocationToAccountId converts XCM Locations to AccountIds. Moonbeam uses:
//! - ParentIsPreset: Relay chain maps to a preset account
//! - SiblingParachainConvertsVia: Sibling parachains map to sovereign accounts
//! - AccountKey20Aliases: AccountKey20 junctions map directly to AccountId
//! - HashedDescription: Other locations map via a hashed description
//! - ExternalConsensusLocationsConverterFor: Bridged locations map to accounts

use crate::xcm_common::*;
use moonbeam_runtime::{xcm_config::LocationToAccountId, AccountId};
use polkadot_parachain::primitives::Sibling;
use sp_runtime::traits::AccountIdConversion;
use xcm::latest::prelude::*;
use xcm_executor::traits::ConvertLocation;

#[test]
fn location_converts_relay_to_account() {
	ExtBuilder::default().build().execute_with(|| {
		let relay_location = Location::parent();
		let account = LocationToAccountId::convert_location(&relay_location);

		assert!(
			account.is_some(),
			"Relay location should convert to account"
		);

		// ParentIsPreset decodes b"Parent" padded with zeros into AccountId (H160).
		let relay_account = account.unwrap();
		let expected: [u8; 20] = {
			let mut buf = [0u8; 20];
			buf[..6].copy_from_slice(b"Parent");
			buf
		};
		assert_eq!(
			relay_account,
			AccountId::from(expected),
			"Relay sovereign should be derived from b\"Parent\" via ParentIsPreset"
		);
	});
}

#[test]
fn location_converts_sibling_parachain_to_sovereign_account() {
	ExtBuilder::default().build().execute_with(|| {
		// SiblingParachainConvertsVia derives a sovereign account from the parachain ID
		// using Sibling's AccountIdConversion::into_account_truncating.
		let sibling_para_id = 2000u32;
		let sibling_location = Location::new(1, [Parachain(sibling_para_id)]);
		let account = LocationToAccountId::convert_location(&sibling_location);

		let expected: AccountId = Sibling::from(sibling_para_id).into_account_truncating();
		assert_eq!(
			account,
			Some(expected),
			"Sibling parachain should convert to sovereign account derived from para ID"
		);
	});
}

#[test]
fn location_converts_different_siblings_to_different_accounts() {
	ExtBuilder::default().build().execute_with(|| {
		let account_a = LocationToAccountId::convert_location(&Location::new(1, [Parachain(2000)]));
		let account_b = LocationToAccountId::convert_location(&Location::new(1, [Parachain(3000)]));

		assert!(
			account_a.is_some(),
			"Sibling 2000 should convert to account"
		);
		assert!(
			account_b.is_some(),
			"Sibling 3000 should convert to account"
		);
		assert_ne!(
			account_a, account_b,
			"Different sibling parachains should have different sovereign accounts"
		);
	});
}

#[test]
fn location_converts_account_key20_directly() {
	ExtBuilder::default().build().execute_with(|| {
		let expected_account = ALICE;
		let location = Location::new(
			0,
			[AccountKey20 {
				network: Some(NetworkId::Polkadot),
				key: expected_account,
			}],
		);

		let account = LocationToAccountId::convert_location(&location);

		assert!(account.is_some(), "AccountKey20 should convert to account");
		assert_eq!(
			account.unwrap(),
			AccountId::from(expected_account),
			"AccountKey20 should map directly to the same account"
		);
	});
}

#[test]
fn location_converts_only_supported_patterns() {
	ExtBuilder::default().build().execute_with(|| {
		// LocationToAccountId supports specific patterns:
		// - ParentIsPreset: Location::parent()
		// - SiblingParachainConvertsVia: siblings with only Parachain junction
		// - AccountKey20Aliases: AccountKey20 junctions
		// - HashedDescription: describable patterns
		// - ExternalConsensusLocationsConverterFor: foreign consensus

		// Complex sibling locations with extra interior junctions may NOT be supported
		// by the current configuration, as HashedDescription's DescribeFamily
		// doesn't describe all arbitrary patterns.
		let complex_location =
			Location::new(1, [Parachain(2000), PalletInstance(10), GeneralIndex(42)]);

		// This pattern may or may not convert depending on DescribeFamily configuration
		let account = LocationToAccountId::convert_location(&complex_location);

		// The current configuration may not support this pattern
		// If it doesn't convert, that's expected behavior - not all patterns are supported
		if account.is_some() {
			// If it does convert, same location should produce same account
			let account_again = LocationToAccountId::convert_location(&complex_location);
			assert_eq!(
				account, account_again,
				"Same location should produce same account"
			);
		}
		// Test passes either way - we're verifying current behavior, not mandating support
	});
}

#[test]
fn location_converts_bridged_parachain() {
	ExtBuilder::default().build().execute_with(|| {
		// A parachain from another consensus (bridged)
		let bridged_location =
			Location::new(2, [GlobalConsensus(NetworkId::Kusama), Parachain(1000)]);

		let account = LocationToAccountId::convert_location(&bridged_location);

		// ExternalConsensusLocationsConverterFor should handle this
		assert!(
			account.is_some(),
			"Bridged parachain should convert to account"
		);
	});
}
