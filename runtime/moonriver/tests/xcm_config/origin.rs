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

//! Tests for XcmOriginToTransactDispatchOrigin and SafeCallFilter configuration.
//!
//! XcmOriginToTransactDispatchOrigin converts XCM locations + OriginKind into
//! dispatch origins for Transact. Moonriver uses:
//! - SovereignSignedViaLocation: SovereignAccount kind → Signed origin
//! - RelayChainAsNative: Native kind from relay → relay origin
//! - SiblingParachainAsNative: Native kind from sibling → sibling origin
//! - XcmPassthrough: Xcm kind → pallet_xcm origin
//! - SignedAccountKey20AsNative: Native kind from AccountKey20 → Signed origin
//!
//! SafeCallFilter determines which calls are allowed via XCM Transact.

use crate::xcm_common::*;
use moonriver_runtime::{
	xcm_config::{SafeCallFilter, XcmOriginToTransactDispatchOrigin},
	RuntimeCall, RuntimeOrigin,
};
use xcm::latest::prelude::*;
use xcm_executor::traits::ConvertOrigin;

#[test]
fn origin_converts_relay_with_sovereign_kind() {
	ExtBuilder::default().build().execute_with(|| {
		// SovereignSignedViaLocation converts relay location + SovereignAccount kind
		// into a Signed origin using the relay's sovereign account.
		let relay = Location::parent();

		let result =
			<XcmOriginToTransactDispatchOrigin as ConvertOrigin<RuntimeOrigin>>::convert_origin(
				relay,
				OriginKind::SovereignAccount,
			);

		assert!(
			result.is_ok(),
			"Relay + SovereignAccount should convert to dispatch origin"
		);
	});
}

#[test]
fn origin_converts_sibling_with_sovereign_kind() {
	ExtBuilder::default().build().execute_with(|| {
		let sibling = Location::new(1, [Parachain(2000)]);

		let result =
			<XcmOriginToTransactDispatchOrigin as ConvertOrigin<RuntimeOrigin>>::convert_origin(
				sibling,
				OriginKind::SovereignAccount,
			);

		assert!(
			result.is_ok(),
			"Sibling + SovereignAccount should convert to dispatch origin"
		);
	});
}

#[test]
fn origin_converts_relay_with_native_kind() {
	ExtBuilder::default().build().execute_with(|| {
		// RelayChainAsNative converts relay location + Native kind into the
		// relay chain origin (used for governance-like calls).
		let relay = Location::parent();

		let result =
			<XcmOriginToTransactDispatchOrigin as ConvertOrigin<RuntimeOrigin>>::convert_origin(
				relay,
				OriginKind::Native,
			);

		assert!(
			result.is_ok(),
			"Relay + Native should convert via RelayChainAsNative"
		);
	});
}

#[test]
fn origin_converts_relay_with_xcm_kind() {
	ExtBuilder::default().build().execute_with(|| {
		// XcmPassthrough converts any location + Xcm kind into a pallet_xcm origin.
		let relay = Location::parent();

		let result =
			<XcmOriginToTransactDispatchOrigin as ConvertOrigin<RuntimeOrigin>>::convert_origin(
				relay,
				OriginKind::Xcm,
			);

		assert!(
			result.is_ok(),
			"Relay + Xcm should convert via XcmPassthrough"
		);
	});
}

#[test]
fn origin_converts_account_key20_with_native_kind() {
	ExtBuilder::default().build().execute_with(|| {
		// SignedAccountKey20AsNative converts AccountKey20 + Native kind into a
		// Signed origin with the 20-byte account.
		let account_location = Location::new(
			0,
			[AccountKey20 {
				network: Some(NetworkId::Kusama),
				key: ALICE,
			}],
		);

		let result =
			<XcmOriginToTransactDispatchOrigin as ConvertOrigin<RuntimeOrigin>>::convert_origin(
				account_location,
				OriginKind::Native,
			);

		assert!(
			result.is_ok(),
			"AccountKey20 + Native should convert via SignedAccountKey20AsNative"
		);
	});
}

#[test]
fn origin_rejects_superuser_kind() {
	ExtBuilder::default().build().execute_with(|| {
		// No converter handles Superuser kind, so it should be rejected.
		let relay = Location::parent();

		let result =
			<XcmOriginToTransactDispatchOrigin as ConvertOrigin<RuntimeOrigin>>::convert_origin(
				relay,
				OriginKind::Superuser,
			);

		assert!(result.is_err(), "Superuser kind should not be convertible");
	});
}

#[test]
fn safe_call_filter_allows_all_calls() {
	ExtBuilder::default().build().execute_with(|| {
		use frame_support::traits::Contains;

		// Moonriver's SafeCallFilter currently allows all calls (returns true).
		// This is intentional — filtering is handled at the EVM level.
		// Verify with calls from two distinct pallets to confirm the filter
		// is truly permissive, not a pallet-specific whitelist.
		let remark_call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
		assert!(
			SafeCallFilter::contains(&remark_call),
			"SafeCallFilter should allow System::remark"
		);

		let utility_call = RuntimeCall::Utility(pallet_utility::Call::batch { calls: vec![] });
		assert!(
			SafeCallFilter::contains(&utility_call),
			"SafeCallFilter should allow Utility::batch"
		);
	});
}
