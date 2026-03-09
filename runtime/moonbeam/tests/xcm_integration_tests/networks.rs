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

//! Network composition for XCM integration tests.
//!
//! This module provides xcm-simulator infrastructure for executing cross-chain tests
//! using the chain definitions from the chains module.

use crate::chains::{
	asset_hub_mock::{asset_hub_ext, ASSET_HUB_PARA_ID},
	moonbeam::{moonbeam_ext, MOONBEAM_PARA_ID},
	relay_mock::relay_ext,
};
use moonbeam_runtime::currency::GLMR;
use parity_scale_codec::{Decode, Encode};
use sp_io::TestExternalities;
use sp_runtime::traits::AccountIdConversion;
use sp_weights::Weight;
use std::cell::RefCell;
use std::collections::VecDeque;
use xcm::latest::prelude::*;
use xcm_executor::XcmExecutor;
use xcm_simulator::{DmpMessageHandlerT, ParaId, XcmpMessageHandlerT};

pub const ONE_DOT: u128 = 10_000_000_000; // DOT has 10 decimals
#[allow(dead_code)]
pub const ONE_GLMR: u128 = GLMR;

// ============================================================================
// Message Buses
// ============================================================================

// Messages from relay chain to parachains (DMP - Downward Message Passing)
thread_local! {
	pub static DMP_QUEUE: RefCell<VecDeque<(ParaId, Vec<u8>)>> = RefCell::new(VecDeque::new());
}

// Messages from parachains to relay chain (UMP - Upward Message Passing)
thread_local! {
	pub static UMP_QUEUE: RefCell<VecDeque<(ParaId, Vec<u8>)>> = RefCell::new(VecDeque::new());
}

// Messages between parachains (XCMP - Cross-chain Message Passing)
thread_local! {
	pub static XCMP_QUEUE: RefCell<VecDeque<(ParaId, ParaId, Vec<u8>)>> = RefCell::new(VecDeque::new());
}

// ============================================================================
// Chain Externalities Storage
// ============================================================================

thread_local! {
	pub static RELAY_EXT: RefCell<TestExternalities> = RefCell::new(relay_ext());
	pub static MOONBEAM_EXT: RefCell<TestExternalities> = RefCell::new(moonbeam_ext());
	pub static ASSET_HUB_EXT: RefCell<TestExternalities> = RefCell::new(asset_hub_ext());
}

// ============================================================================
// XCM Routers
// ============================================================================

/// XCM Router for the relay chain - sends DMP to parachains
pub struct RelayChainXcmRouter;

impl SendXcm for RelayChainXcmRouter {
	type Ticket = (ParaId, Xcm<()>);

	fn validate(
		dest: &mut Option<Location>,
		msg: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		let dest = dest.take().ok_or(SendError::MissingArgument)?;
		let msg = msg.take().ok_or(SendError::MissingArgument)?;

		// Check if destination is a parachain
		match dest.unpack() {
			(0, [Parachain(id)]) => Ok((((*id).into(), msg), Assets::new())),
			_ => Err(SendError::NotApplicable),
		}
	}

	fn deliver(ticket: Self::Ticket) -> Result<XcmHash, SendError> {
		let (para_id, msg) = ticket;
		let encoded = xcm::VersionedXcm::<()>::from(msg).encode();

		DMP_QUEUE.with(|q| q.borrow_mut().push_back((para_id, encoded)));

		Ok([0u8; 32])
	}
}

/// XCM Router for parachains - sends UMP to relay or XCMP to siblings
pub struct ParachainXcmRouter<T>(core::marker::PhantomData<T>);

impl<T: xcm_simulator::mock_message_queue::Config> SendXcm for ParachainXcmRouter<T> {
	type Ticket = (Location, Xcm<()>);

	fn validate(
		dest: &mut Option<Location>,
		msg: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		let dest = dest.take().ok_or(SendError::MissingArgument)?;
		let msg = msg.take().ok_or(SendError::MissingArgument)?;

		Ok(((dest, msg), Assets::new()))
	}

	fn deliver(ticket: Self::Ticket) -> Result<XcmHash, SendError> {
		let (dest, msg) = ticket;
		let encoded = xcm::VersionedXcm::<()>::from(msg).encode();

		match dest.unpack() {
			// UMP - message to relay chain
			(1, []) => {
				let para_id = xcm_simulator::mock_message_queue::ParachainId::<T>::get();
				UMP_QUEUE.with(|q| q.borrow_mut().push_back((para_id.into(), encoded)));
			}
			// XCMP - message to sibling parachain
			(1, [Parachain(sibling_id)]) => {
				let para_id = xcm_simulator::mock_message_queue::ParachainId::<T>::get();
				XCMP_QUEUE.with(|q| {
					q.borrow_mut()
						.push_back((para_id.into(), (*sibling_id).into(), encoded))
				});
			}
			_ => return Err(SendError::NotApplicable),
		}

		Ok([0u8; 32])
	}
}

// ============================================================================
// Test Network
// ============================================================================

/// Reset all chain states and message queues
pub fn reset_networks() {
	RELAY_EXT.with(|ext| *ext.borrow_mut() = relay_ext());
	MOONBEAM_EXT.with(|ext| *ext.borrow_mut() = moonbeam_ext());
	ASSET_HUB_EXT.with(|ext| *ext.borrow_mut() = asset_hub_ext());
	DMP_QUEUE.with(|q| q.borrow_mut().clear());
	UMP_QUEUE.with(|q| q.borrow_mut().clear());
	XCMP_QUEUE.with(|q| q.borrow_mut().clear());
}

/// Execute a closure within the Relay chain context
pub fn relay_execute_with<R>(f: impl FnOnce() -> R) -> R {
	RELAY_EXT.with(|ext| ext.borrow_mut().execute_with(f))
}

/// Execute a closure within the Moonbeam context
pub fn moonbeam_execute_with<R>(f: impl FnOnce() -> R) -> R {
	MOONBEAM_EXT.with(|ext| ext.borrow_mut().execute_with(f))
}

/// Execute a closure within the Asset Hub context
pub fn asset_hub_execute_with<R>(f: impl FnOnce() -> R) -> R {
	ASSET_HUB_EXT.with(|ext| ext.borrow_mut().execute_with(f))
}

/// Process all pending DMP messages (relay -> parachains)
pub fn process_dmp_messages() {
	while let Some((para_id, msg)) = DMP_QUEUE.with(|q| q.borrow_mut().pop_front()) {
		let para_id_u32: u32 = para_id.into();

		match para_id_u32 {
			id if id == MOONBEAM_PARA_ID => {
				moonbeam_execute_with(|| {
					// Decode the versioned XCM and execute directly
					if let Ok(versioned) =
						xcm::VersionedXcm::<moonbeam_runtime::RuntimeCall>::decode(&mut &msg[..])
					{
						if let Ok(xcm_msg) = versioned.try_into() {
							let origin = Location::parent();
							let mut hash = sp_io::hashing::blake2_256(&msg);
							let _ = XcmExecutor::<moonbeam_runtime::xcm_config::XcmExecutorConfig>::prepare_and_execute(
                                origin,
                                xcm_msg,
                                &mut hash,
                                Weight::MAX,
                                Weight::zero(),
                            );
						}
					}
				});
			}
			id if id == ASSET_HUB_PARA_ID => {
				asset_hub_execute_with(|| {
					use crate::chains::asset_hub_mock::MsgQueue;
					let _ = <MsgQueue as DmpMessageHandlerT>::handle_dmp_messages(
						vec![(0, msg)].into_iter(),
						Weight::MAX,
					);
				});
			}
			_ => panic!("Unknown parachain: {}", para_id_u32),
		}
	}
}

/// Process all pending UMP messages (parachains -> relay)
pub fn process_ump_messages() {
	while let Some((para_id, msg)) = UMP_QUEUE.with(|q| q.borrow_mut().pop_front()) {
		relay_execute_with(|| {
			// Decode the versioned XCM and execute directly on relay
			if let Ok(versioned) =
				xcm::VersionedXcm::<crate::chains::relay_mock::RuntimeCall>::decode(&mut &msg[..])
			{
				if let Ok(xcm_msg) = versioned.try_into() {
					// Origin is the parachain that sent the UMP
					let origin = Location::new(0, [Parachain(para_id.into())]);
					let mut hash = sp_io::hashing::blake2_256(&msg);
					let _ =
						XcmExecutor::<crate::chains::relay_mock::XcmConfig>::prepare_and_execute(
							origin,
							xcm_msg,
							&mut hash,
							Weight::MAX,
							Weight::zero(),
						);
				}
			}
		});
	}
}

/// Process all pending XCMP messages (parachain -> parachain)
pub fn process_xcmp_messages() {
	while let Some((from, to, msg)) = XCMP_QUEUE.with(|q| q.borrow_mut().pop_front()) {
		let to_u32: u32 = to.into();
		let from_u32: u32 = from.into();

		match to_u32 {
			id if id == MOONBEAM_PARA_ID => {
				moonbeam_execute_with(|| {
					// Decode and execute XCM directly
					if let Ok(versioned) =
						xcm::VersionedXcm::<moonbeam_runtime::RuntimeCall>::decode(&mut &msg[..])
					{
						if let Ok(xcm_msg) = versioned.try_into() {
							// Origin is sibling parachain
							let origin = Location::new(1, [Parachain(from_u32)]);
							let mut hash = sp_io::hashing::blake2_256(&msg);
							let _ = XcmExecutor::<moonbeam_runtime::xcm_config::XcmExecutorConfig>::prepare_and_execute(
                                origin,
                                xcm_msg,
                                &mut hash,
                                Weight::MAX,
                                Weight::zero(),
                            );
						}
					}
				});
			}
			id if id == ASSET_HUB_PARA_ID => {
				asset_hub_execute_with(|| {
					use crate::chains::asset_hub_mock::MsgQueue;
					let _ = <MsgQueue as XcmpMessageHandlerT>::handle_xcmp_messages(
						vec![(from_u32.into(), 0, &msg[..])].into_iter(),
						Weight::MAX,
					);
				});
			}
			_ => panic!("Unknown destination parachain: {}", to_u32),
		}
	}
}

/// Process all pending XCM messages across all chains
pub fn dispatch_xcm_buses() {
	// Keep processing until all queues are empty
	loop {
		let has_dmp = DMP_QUEUE.with(|q| !q.borrow().is_empty());
		let has_ump = UMP_QUEUE.with(|q| !q.borrow().is_empty());
		let has_xcmp = XCMP_QUEUE.with(|q| !q.borrow().is_empty());

		if !has_dmp && !has_ump && !has_xcmp {
			break;
		}

		process_dmp_messages();
		process_ump_messages();
		process_xcmp_messages();
	}
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the sovereign account of a parachain on the relay chain
pub fn parachain_sovereign_account(para_id: u32) -> sp_runtime::AccountId32 {
	ParaId::from(para_id).into_account_truncating()
}

/// Get Moonbeam's sovereign account on the relay chain
pub fn moonbeam_sovereign_account() -> sp_runtime::AccountId32 {
	parachain_sovereign_account(MOONBEAM_PARA_ID)
}

/// Get Asset Hub's sovereign account on the relay chain
pub fn asset_hub_sovereign_account() -> sp_runtime::AccountId32 {
	parachain_sovereign_account(ASSET_HUB_PARA_ID)
}

/// Helper to get parachain IDs
pub mod para_ids {
	pub const MOONBEAM: u32 = super::MOONBEAM_PARA_ID;
}
