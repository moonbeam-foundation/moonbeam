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

use precompile_utils::{prelude::*, EvmDataReader};
use sp_core::H256;
use sp_std::vec::Vec;
use xcm::latest::MultiLocation;

// TODO: design this with the following criteria in mind:
//       * friendly for other chains (performance, ease, security)
//       * future proof (bare minimum: version this)
//       * easy to parse
//       * flexible -- need to support "MVP" level of XCM functionality
pub struct XcmUserAction {
	pub destination: MultiLocation,
	pub destination_account: MultiLocation,
}

pub enum VersionedUserAction {
	V1(XcmUserAction),
}

/// Parse a user action from some bytes
pub fn parse_user_action(input: &Vec<u8>) -> Result<VersionedUserAction, &'static str> {
	// TODO: actually parse :)
	// more importantly, define a structure (see criteria above)

	Ok(VersionedUserAction::V1(XcmUserAction {
		destination: MultiLocation::parent(),
		destination_account: MultiLocation::parent(),
	}))
}

// Struct representing a Wormhole VM
// see https://github.com/wormhole-foundation/wormhole/blob/main/ethereum/contracts/bridge/BridgeStructs.sol
#[derive(Default)]
pub struct WormholeVM {
	pub version: u8,
	pub timestamp: u32,
	pub nonce: u32,
	pub emitterChainId: u16,
	pub emitterAddress: H256,
	pub sequence: u64,
	pub consistencyLevel: u8,
	pub payload: Vec<u8>,

	pub guardianSetIndex: u32,
	// Signature[] signatures; // Signature: bytes32 r; bytes32 s; uint8 v; uint8 guardianIndex;
	pub hash: H256,
}

impl WormholeVM {
	/// Parse the output from the EVM when calling Wormhole's contracts,
	/// e.g. the Solidity return type "returns (Structs.VM memory vm)"
	pub fn new_from_encoded(encoded: &[u8]) -> MayRevert<Self> {
		let mut reader = EvmDataReader::new(encoded);

		let mut vm: WormholeVM = Default::default();
		vm.version = reader.read()?;
		vm.guardianSetIndex = reader.read()?;
		let signersLen = reader.read()?;

		// TODO: can't use EvmDataReader here, it will read U256 for all int
		// types. The data is packed tightly using the BytesLib library:
		// https://github.com/GNSPS/solidity-bytes-utils

		Ok(vm)
	}
}
