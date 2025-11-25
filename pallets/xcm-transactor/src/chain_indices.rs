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

//! Chain-specific pallet and call indices for XCM Transactor
//!
//! This module defines indices structures for different chains (Relay, AssetHub)
//! to enable proper SCALE encoding of remote calls.

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

/// Relay Chain pallet and call indices
///
/// These indices are used to encode calls for the Relay Chain (Polkadot/Kusama).
#[derive(
	Clone, Copy, Debug, Default, Deserialize, Serialize, Encode, Decode, TypeInfo, PartialEq, Eq,
)]
pub struct RelayChainIndices {
	// Pallet indices
	pub staking: u8,
	pub utility: u8,
	pub hrmp: u8,
	// Staking indices
	pub bond: u8,
	pub bond_extra: u8,
	pub unbond: u8,
	pub withdraw_unbonded: u8,
	pub validate: u8,
	pub nominate: u8,
	pub chill: u8,
	pub set_payee: u8,
	pub set_controller: u8,
	pub rebond: u8,
	// Utility indices
	pub as_derivative: u8,
	// Hrmp indices
	pub init_open_channel: u8,
	pub accept_open_channel: u8,
	pub close_channel: u8,
	pub cancel_open_request: u8,
}

/// AssetHub pallet and call indices
///
/// These indices are used to encode calls for AssetHub system parachain.
/// Network-specific values are defined in the `moonbeam-assethub-encoder` crate.
#[derive(
	Clone, Copy, Debug, Default, Deserialize, Serialize, Encode, Decode, TypeInfo, PartialEq, Eq,
)]
pub struct AssetHubIndices {
	// Pallet indices
	pub utility: u8,
	pub proxy: u8,
	pub staking: u8,
	pub nomination_pools: u8,
	pub delegated_staking: u8,
	pub assets: u8,
	pub nfts: u8,

	// Utility call indices
	pub as_derivative: u8,
	pub batch: u8,
	pub batch_all: u8,

	// Proxy call indices
	pub proxy_call: u8,
	pub add_proxy: u8,
	pub remove_proxy: u8,

	// Staking call indices
	pub bond: u8,
	pub bond_extra: u8,
	pub unbond: u8,
	pub withdraw_unbonded: u8,
	pub validate: u8,
	pub nominate: u8,
	pub chill: u8,
	pub set_payee: u8,
	pub set_controller: u8,
	pub rebond: u8,
}

/// Unified chain indices enum
///
/// Wraps either RelayChainIndices or AssetHubIndices depending on the target chain
#[derive(Clone, Debug, Deserialize, Serialize, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub enum ChainIndices {
	Relay(RelayChainIndices),
	AssetHub(AssetHubIndices),
}

impl Default for ChainIndices {
	fn default() -> Self {
		ChainIndices::Relay(RelayChainIndices::default())
	}
}
