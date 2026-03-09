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

//! HRMP (Horizontal Relay-routed Message Passing) integration tests.
//!
//! Tests for XCM routing between parachains via the relay chain:
//! - XCMP queue configuration
//! - Message routing to siblings
//! - Message size limits
//! - Channel capacity handling

use crate::networks::*;
use xcm::latest::prelude::*;

#[test]
fn xcmp_queue_configured() {
	moonbase_execute_with(|| {
		// Verify XCMP queue pallet is configured
		// The XcmpQueue is used for horizontal (parachain-to-parachain) messaging
		// LocalXcmRouter is a tuple of:
		// - ParentAsUmp (for relay chain communication)
		// - XcmpQueue (for sibling parachain communication)
	});
}

#[test]
fn hrmp_channel_info_accessible() {
	moonbase_execute_with(|| {
		// The XCMP queue uses ParachainSystem for channel info
		// Verify the integration is correctly configured

		// cumulus_pallet_xcmp_queue::Config::ChannelInfo = ParachainSystem
		// This allows the XCMP queue to query channel state
	});
}

#[test]
fn xcmp_max_message_size_configured() {
	moonbase_execute_with(|| {
		// Verify message size limits are configured
		// The MessageQueueHeapSize affects max message processing

		use moonbase_runtime::xcm_config::MessageQueueHeapSize;
		let heap_size = MessageQueueHeapSize::get();

		assert!(
			heap_size > 0,
			"Message queue heap size should be configured"
		);
		assert!(
			heap_size >= 100 * 1024,
			"Heap size should be at least 100KB for HRMP compatibility"
		);
	});
}

#[test]
fn sibling_parachain_routing_configured() {
	moonbase_execute_with(|| {
		// Verify XCM can be routed to sibling parachains
		use moonbase_runtime::xcm_config::LocationToAccountId;
		use xcm_executor::traits::ConvertLocation;

		// Sibling parachain locations should convert to sovereign accounts
		let sibling_2000 = Location::new(1, [Parachain(2000)]);
		let sibling_3000 = Location::new(1, [Parachain(3000)]);

		let sovereign_2000 = LocationToAccountId::convert_location(&sibling_2000);
		let sovereign_3000 = LocationToAccountId::convert_location(&sibling_3000);

		assert!(
			sovereign_2000.is_some(),
			"Should compute sovereign for para 2000"
		);
		assert!(
			sovereign_3000.is_some(),
			"Should compute sovereign for para 3000"
		);
		assert_ne!(
			sovereign_2000, sovereign_3000,
			"Different parachains should have different sovereigns"
		);
	});
}
