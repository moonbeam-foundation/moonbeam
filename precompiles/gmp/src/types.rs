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

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode};
use precompile_utils::{
	data::{Address, BoundedBytes, String},
	prelude::*,
	EvmData,
};
use sp_core::{H160, H256, U256};
use sp_std::vec::Vec;
use xcm::latest::MultiLocation;

// TODO: design this with the following criteria in mind:
//       * friendly for other chains (performance, ease, security)
//       * future proof (bare minimum: version this)
//       * easy to parse
//       * flexible -- need to support "MVP" level of XCM functionality
#[derive(Encode, Decode, Debug)]
pub struct XcmRoutingUserAction {
	pub currency_address: H160,
	pub amount: U256,
	pub destination_chain: MultiLocation,
	pub destination_account: MultiLocation,
}

#[derive(Encode, Decode, Debug)]
pub enum VersionedUserAction {
	V1(XcmRoutingUserAction),
}

/// Parse a user action from some bytes
pub fn parse_user_action(input: &Vec<u8>) -> Result<VersionedUserAction, &'static str> {
	// TODO: actually parse :)
	// more importantly, define a structure (see criteria above)

	Ok(VersionedUserAction::V1(XcmRoutingUserAction {
		currency_address: Default::default(),
		amount: 1u32.into(),
		destination_chain: MultiLocation::parent(),
		destination_account: MultiLocation::parent(),
	}))
}

// Struct representing a Wormhole VM
// The main purpose of this struct is to decode the ABI encoded struct returned from certain calls
// in the Wormhole Ethereum contracts.
//
// https://github.com/wormhole-foundation/wormhole/blob/main/ethereum/contracts/Structs.sol
#[derive(Debug, EvmData)]
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
#[derive(Debug, EvmData)]
pub struct WormholeSignature {
	pub r: U256,
	pub s: U256,
	pub v: u8,
	pub guardianIndex: u8,
}
