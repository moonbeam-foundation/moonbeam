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

//! XCM Transact integration tests.
//!
//! Tests for remote execution via XCM Transact instruction:
//! - Transact from relay chain
//! - Transact from sibling parachain
//! - Transact with sovereign origin
//! - Transact with XCM origin
//! - Call dispatch filtering
//! - Weight and fee handling for transact

use crate::networks::*;
use moonbeam_runtime::{xcm_config::XcmOriginToTransactDispatchOrigin, RuntimeCall, RuntimeOrigin};
use parity_scale_codec::Encode;
use sp_runtime::traits::Dispatchable;
use sp_weights::Weight;
use xcm::latest::prelude::*;
use xcm_executor::traits::ConvertOrigin;

#[test]
fn transact_origin_converts_relay_to_dispatch_origin() {
	moonbeam_execute_with(|| {
		let relay_origin = Location::parent();

		// XcmOriginToTransactDispatchOrigin should convert relay location
		let converted =
			<XcmOriginToTransactDispatchOrigin as ConvertOrigin<RuntimeOrigin>>::convert_origin(
				relay_origin.clone(),
				OriginKind::SovereignAccount,
			);

		assert!(
			converted.is_ok(),
			"Relay origin should convert to dispatch origin"
		);
	});
}

#[test]
fn transact_origin_converts_sibling_to_dispatch_origin() {
	moonbeam_execute_with(|| {
		let sibling_origin = Location::new(1, [Parachain(2000)]);

		let converted =
			<XcmOriginToTransactDispatchOrigin as ConvertOrigin<RuntimeOrigin>>::convert_origin(
				sibling_origin.clone(),
				OriginKind::SovereignAccount,
			);

		assert!(
			converted.is_ok(),
			"Sibling origin should convert to dispatch origin"
		);
	});
}

#[test]
fn transact_filter_allows_safe_calls() {
	moonbeam_execute_with(|| {
		use frame_support::traits::Contains;
		use moonbeam_runtime::xcm_config::SafeCallFilter;

		// System::remark should be allowed (safe call)
		let safe_call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
		assert!(
			SafeCallFilter::contains(&safe_call),
			"Safe calls should pass filter"
		);
	});
}

#[test]
fn transact_can_dispatch_system_remark() {
	moonbeam_execute_with(|| {
		// Create a simple remark call
		let call = RuntimeCall::System(frame_system::Call::remark {
			remark: b"test".to_vec(),
		});

		// The call should be encodable for transact
		let encoded_call = call.encode();
		assert!(!encoded_call.is_empty(), "Call should encode successfully");

		// Call should be dispatchable
		let origin = RuntimeOrigin::root();
		let result = call.dispatch(origin);
		assert!(result.is_ok(), "Remark should dispatch successfully");
	});
}

#[test]
fn transact_with_xcm_origin_kind() {
	moonbeam_execute_with(|| {
		let xcm_origin = Location::parent();

		// Test XCM origin kind conversion
		let converted =
			<XcmOriginToTransactDispatchOrigin as ConvertOrigin<RuntimeOrigin>>::convert_origin(
				xcm_origin.clone(),
				OriginKind::Xcm,
			);

		// XCM origin kind should convert via pallet_xcm::XcmPassthrough
		assert!(converted.is_ok(), "XCM origin kind should convert");
	});
}

#[test]
fn transact_with_native_origin_from_relay() {
	moonbeam_execute_with(|| {
		let relay_origin = Location::parent();

		// Native origin from relay should convert via RelayChainAsNative
		let converted =
			<XcmOriginToTransactDispatchOrigin as ConvertOrigin<RuntimeOrigin>>::convert_origin(
				relay_origin.clone(),
				OriginKind::Native,
			);

		assert!(converted.is_ok(), "Native relay origin should convert");
	});
}

#[test]
fn transact_weight_configured_correctly() {
	moonbeam_execute_with(|| {
		// Verify that the weigher can weigh transact instructions
		use moonbeam_runtime::xcm_config::XcmWeigher;
		use xcm_executor::traits::WeightBounds;

		let encoded_call =
			RuntimeCall::System(frame_system::Call::remark { remark: vec![] }).encode();
		let transact_message: Xcm<RuntimeCall> = Xcm(vec![Transact {
			origin_kind: OriginKind::SovereignAccount,
			call: encoded_call.into(),
			fallback_max_weight: Some(Weight::from_parts(1_000_000, 1024)),
		}]);

		let weight = XcmWeigher::weight(&mut transact_message.clone(), Weight::MAX);
		assert!(weight.is_ok(), "Should weigh transact instruction");
	});
}
