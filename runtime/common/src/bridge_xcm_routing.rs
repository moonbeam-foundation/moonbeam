// Copyright 2025 Moonbeam foundation
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

use core::marker::PhantomData;
use sp_std::vec::Vec;
use xcm::latest::{Location, SendError, SendResult, SendXcm, Xcm, XcmHash};
use xcm::{VersionedLocation, VersionedXcm};
use xcm_builder::InspectMessageQueues;

/// The target that will be used when publishing logs related to this component.
pub const LOG_TARGET: &str = "xcm::bridge-routing";

pub struct BridgeXcmRouter<MessageExporter>(PhantomData<MessageExporter>);

// This struct acts as the `SendXcm` to the local instance of pallet_bridge_messages instead of
// regular XCMP/DMP transport.
impl<MessageExporter: SendXcm> SendXcm for BridgeXcmRouter<MessageExporter> {
	type Ticket = MessageExporter::Ticket;

	fn validate(
		dest: &mut Option<Location>,
		xcm: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		log::trace!(target: LOG_TARGET, "validate - msg: {xcm:?}, destination: {dest:?}");

		MessageExporter::validate(dest, xcm)
	}

	fn deliver(ticket: Self::Ticket) -> Result<XcmHash, SendError> {
		MessageExporter::deliver(ticket)
	}
}

/// This router needs to implement `InspectMessageQueues` but doesn't have to
/// return any messages, since it just reuses the `XcmpQueue` router.
impl<MessageExporter> InspectMessageQueues for BridgeXcmRouter<MessageExporter> {
	fn clear_messages() {}

	fn get_messages() -> Vec<(VersionedLocation, Vec<VersionedXcm<()>>)> {
		Vec::new()
	}
}
