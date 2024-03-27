// Copyright 2019-2023 PureStake Inc.
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

//! Precompile to receive GMP callbacks and forward to XCM

use parity_scale_codec::{Decode, Encode};
use precompile_utils::prelude::*;
use sp_core::{H256, U256};
use sp_std::vec::Vec;
use xcm::VersionedLocation;

// A user action which will attempt to route the transferred assets to the account/chain specified
// by the given Location. Recall that a Location can contain both a chain and an account
// on that chain, as this one should.
#[derive(Encode, Decode, Debug)]
pub struct XcmRoutingUserAction {
	pub destination: VersionedLocation,
}

// A user action which is the same as XcmRoutingUserAction but also allows a fee to be paid. The
// fee is paid in the same asset being transferred, and must be <= the amount being sent.
#[derive(Encode, Decode, Debug)]
pub struct XcmRoutingUserActionWithFee {
	pub destination: VersionedLocation,
	pub fee: U256,
}

// A simple versioning wrapper around the initial XcmRoutingUserAction use-case. This should make
// future breaking changes easy to add in a backwards-compatible way.
#[derive(Encode, Decode, Debug)]
#[non_exhaustive]
pub enum VersionedUserAction {
	V1(XcmRoutingUserAction),
	V2(XcmRoutingUserActionWithFee),
}

// Struct representing a Wormhole VM
// The main purpose of this struct is to decode the ABI encoded struct returned from certain calls
// in the Wormhole Ethereum contracts.
//
// https://github.com/wormhole-foundation/wormhole/blob/main/ethereum/contracts/Structs.sol
#[derive(Debug, solidity::Codec)]
pub struct WormholeVM {
	pub version: u8,
	pub timestamp: u32,
	pub nonce: u32,
	pub emitter_chain_id: u16,
	pub emitter_address: H256,
	pub sequence: u64,
	pub consistency_level: u8,
	pub payload: BoundedBytes<crate::GetCallDataLimit>,

	pub guardian_set_index: u32,
	pub signatures: Vec<WormholeSignature>, // TODO: review: can this allow unbounded allocations?
	pub hash: H256,
}

// Struct representing a Wormhole Signature struct
#[derive(Debug, solidity::Codec)]
pub struct WormholeSignature {
	pub r: U256,
	pub s: U256,
	pub v: u8,
	pub guardian_index: u8,
}

// Struct representing a wormhole "BridgeStructs.TransferWithPayload" struct
// As with WormholeVM, the main purpose of this struct is to decode the ABI encoded struct when it
// returned from calls to Wormhole Ethereum contracts.
#[derive(Debug, solidity::Codec)]
pub struct WormholeTransferWithPayloadData {
	pub payload_id: u8,
	pub amount: U256,
	pub token_address: H256,
	pub token_chain: u16,
	pub to: H256,
	pub to_chain: u16,
	pub from_address: H256,
	pub payload: BoundedBytes<crate::GetCallDataLimit>,
}
