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
use parity_scale_codec::Encode;
use sp_std::vec::Vec;
use xcm::latest::{Location, SendError, SendResult, SendXcm, Xcm, XcmHash};
use xcm::{GetVersion, IntoVersion, VersionedLocation, VersionedXcm};
use xcm_builder::InspectMessageQueues;

/// The target that will be used when publishing logs related to this pallet.
pub const LOG_TARGET: &str = "xcm::bridge-router";

/// Maximal size of the XCM message that may be sent over bridge.
///
/// This should be less than the maximal size, allowed by the messages pallet, because
/// the message itself is wrapped in other structs and is double encoded.
pub const HARD_MESSAGE_SIZE_LIMIT: u32 = 32 * 1024;

pub struct BridgeXcmRouter<MessageExporter, DestinationVersion>(
	PhantomData<(MessageExporter, DestinationVersion)>,
);

// This struct acts as the `SendXcm` to the sibling/child bridge hub instead of regular
// XCMP/DMP transport. This allows injecting dynamic message fees into XCM programs that
// are going to the bridged network.
impl<MessageExporter: SendXcm, DestinationVersion: GetVersion> SendXcm
	for BridgeXcmRouter<MessageExporter, DestinationVersion>
{
	type Ticket = (u32, Location, <MessageExporter>::Ticket);

	fn validate(
		dest: &mut Option<Location>,
		xcm: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		log::trace!(target: LOG_TARGET, "validate - msg: {xcm:?}, destination: {dest:?}");

		// In case of success, the `T::MessageExporter` can modify XCM instructions and consume
		// `dest` / `xcm`, so we retain the clone of original message and the destination for later
		// `DestinationVersion` validation.
		let xcm_to_dest_clone = xcm.clone();
		let dest_clone = dest.clone();

		// First, use the inner exporter to validate the destination to determine if it is even
		// routable. If it is not, return an error. If it is, then the XCM is extended with
		// instructions to pay the message fee at the sibling/child bridge hub. The cost will
		// include both the cost of (1) delivery to the sibling bridge hub (returned by
		// `Config::MessageExporter`) and (2) delivery to the bridged bridge hub (returned by
		// `Self::exporter_for`).
		match MessageExporter::validate(dest, xcm) {
			Ok((ticket, cost)) => {
				// If the ticket is ok, it means we are routing with this router, so we need to
				// apply more validations to the cloned `dest` and `xcm`, which are required here.
				let xcm_to_dest_clone = xcm_to_dest_clone.ok_or(SendError::MissingArgument)?;
				let dest_clone = dest_clone.ok_or(SendError::MissingArgument)?;

				// We won't have access to `dest` and `xcm` in the `deliver` method, so we need to
				// precompute everything required here. However, `dest` and `xcm` were consumed by
				// `T::MessageExporter`, so we need to use their clones.
				let message_size = xcm_to_dest_clone.encoded_size() as _;

				// The bridge doesn't support oversized or overweight messages. Therefore, it's
				// better to drop such messages here rather than at the bridge hub. Let's check the
				// message size.
				if message_size > HARD_MESSAGE_SIZE_LIMIT {
					return Err(SendError::ExceedsMaxMessageSize);
				}

				// We need to ensure that the known `dest`'s XCM version can comprehend the current
				// `xcm` program. This may seem like an additional, unnecessary check, but it is
				// not. A similar check is probably performed by the `T::MessageExporter`, which
				// attempts to send a versioned message to the sibling bridge hub. However, the
				// local bridge hub may have a higher XCM version than the remote `dest`. Once
				// again, it is better to discard such messages here than at the bridge hub (e.g.,
				// to avoid losing funds).
				let destination_version = DestinationVersion::get_version_for(&dest_clone)
					.ok_or(SendError::DestinationUnsupported)?;
				let _ = VersionedXcm::from(xcm_to_dest_clone)
					.into_version(destination_version)
					.map_err(|()| SendError::DestinationUnsupported)?;

				log::info!(
					target: LOG_TARGET,
					"Going to send message to {dest_clone:?} ({message_size:?} bytes) with actual cost: {cost:?}"
				);

				Ok(((message_size, dest_clone, ticket), cost))
			}
			Err(e) => {
				log::trace!(target: LOG_TARGET, "`T::MessageExporter` validates for dest: {dest_clone:?} with error: {e:?}");
				Err(e)
			}
		}
	}

	fn deliver(ticket: Self::Ticket) -> Result<XcmHash, SendError> {
		// Use router to enqueue message to the sibling/child bridge hub. This also should handle
		// payment for passing through this queue.
		let (message_size, dest, ticket) = ticket;
		let xcm_hash = MessageExporter::deliver(ticket)?;

		log::trace!(
			target: LOG_TARGET,
			"deliver - message (size: {message_size:?}) sent to the dest: {dest:?}, xcm_hash: {xcm_hash:?}"
		);

		// increase delivery fee factor (if required)
		//Self::on_message_sent_to(message_size, dest);

		Ok(xcm_hash)
	}
}

impl<MessageExporter, DestinationVersion> InspectMessageQueues
	for BridgeXcmRouter<MessageExporter, DestinationVersion>
{
	fn clear_messages() {}

	/// This router needs to implement `InspectMessageQueues` but doesn't have to
	/// return any messages, since it just reuses the `XcmpQueue` router.
	fn get_messages() -> Vec<(VersionedLocation, Vec<VersionedXcm<()>>)> {
		Vec::new()
	}
}
