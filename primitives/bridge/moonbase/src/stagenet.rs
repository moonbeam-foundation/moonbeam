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

//! # Stagenet bridge primitives

use bp_bridge_hub_cumulus::{
	BlockLength, BlockWeights, Hasher, Nonce, MAX_BRIDGE_HUB_HEADER_SIZE,
	MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX, MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX,
};
use bp_messages::{ChainWithMessages, MessageNonce, Weight};
use bp_runtime::{Chain, ChainId, Parachain};
use frame_support::__private::StateVersion;
use frame_support::dispatch::DispatchClass;

pub use moonbeam_core_primitives::{AccountId, Balance, BlockNumber, Hash, Header, Signature};

pub const PARACHAIN_ID: u32 = 1000;
/// Name of the messages pallet instance that is deployed at bridged chains.
pub const WITH_BRIDGE_MESSAGES_PALLET_NAME: &str = "BridgeMessages";

/// Stagenet parachain.
pub struct Stagenet;

impl Chain for Stagenet {
	const ID: ChainId = *b"stgn";

	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hasher = Hasher;
	type Header = Header;

	type AccountId = AccountId;
	type Balance = Balance;
	type Nonce = Nonce;
	type Signature = Signature;

	const STATE_VERSION: StateVersion = StateVersion::V1;

	fn max_extrinsic_size() -> u32 {
		*BlockLength::get().max.get(DispatchClass::Normal)
	}

	fn max_extrinsic_weight() -> Weight {
		BlockWeights::get()
			.get(DispatchClass::Normal)
			.max_extrinsic
			.unwrap_or(Weight::MAX)
	}
}

impl Parachain for Stagenet {
	const PARACHAIN_ID: u32 = PARACHAIN_ID;
	const MAX_HEADER_SIZE: u32 = MAX_BRIDGE_HUB_HEADER_SIZE;
}

impl ChainWithMessages for Stagenet {
	const WITH_CHAIN_MESSAGES_PALLET_NAME: &'static str = WITH_BRIDGE_MESSAGES_PALLET_NAME;

	const MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX: MessageNonce =
		MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	const MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX: MessageNonce =
		MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
}
