// Copyright 2019-2025 PureStake Inc.
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
/// Values are based on polkadot-fellows/runtimes AssetHub (Polkadot) runtime v2.0.2
///
/// WARNING: These indices MUST be verified against the actual AssetHub runtime
/// before deployment. Use `subxt metadata` to extract correct indices.
#[derive(
	Clone, Copy, Debug, Default, Deserialize, Serialize, Encode, Decode, TypeInfo, PartialEq, Eq,
)]
pub struct AssetHubIndices {
	// Pallet indices (from AssetHub Polkadot runtime)
	pub utility: u8,           // 40
	pub proxy: u8,             // 42
	pub staking: u8,           // 89 (pallet_staking for delegated staking)
	pub nomination_pools: u8,  // ~80 (TBD - verify with metadata)
	pub delegated_staking: u8, // ~88 (TBD - verify with metadata)
	pub assets: u8,            // 50 (for future asset operations)
	pub nfts: u8,              // 52 (for future NFT operations)

	// Utility call indices (standard across Substrate)
	pub as_derivative: u8, // 1
	pub batch: u8,         // 0
	pub batch_all: u8,     // 2

	// Proxy call indices (standard)
	pub proxy_call: u8,   // 0
	pub add_proxy: u8,    // 1
	pub remove_proxy: u8, // 2

	// Staking call indices (must be verified)
	// These may differ from Relay Chain indices
	pub bond: u8,              // TBD
	pub bond_extra: u8,        // TBD
	pub unbond: u8,            // TBD
	pub withdraw_unbonded: u8, // TBD
	pub validate: u8,          // TBD (may not be supported)
	pub nominate: u8,          // TBD
	pub chill: u8,             // TBD
	pub set_payee: u8,         // TBD
	pub set_controller: u8,    // TBD (deprecated)
	pub rebond: u8,            // TBD
}

impl AssetHubIndices {
	/// Create default AssetHub indices for Polkadot AssetHub
	///
	/// These values are estimates and MUST be verified before production use
	pub fn polkadot_default() -> Self {
		Self {
			// Verified pallet indices from polkadot-fellows/runtimes
			utility: 40,
			proxy: 42,
			staking: 89,
			nomination_pools: 80,  // Estimate - VERIFY
			delegated_staking: 88, // Estimate - VERIFY
			assets: 50,
			nfts: 52,

			// Standard utility indices
			as_derivative: 1,
			batch: 0,
			batch_all: 2,

			// Standard proxy indices
			proxy_call: 0,
			add_proxy: 1,
			remove_proxy: 2,

			// Staking indices - MUST BE VERIFIED
			// These are placeholders and likely INCORRECT
			bond: 0,
			bond_extra: 1,
			unbond: 2,
			withdraw_unbonded: 3,
			validate: 4,
			nominate: 5,
			chill: 6,
			set_payee: 7,
			set_controller: 8,
			rebond: 19,
		}
	}
}

/// Unified chain indices enum
///
/// Wraps either RelayChainIndices or AssetHubIndices depending on the target chain
#[derive(Clone, Debug, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub enum ChainIndices {
	Relay(RelayChainIndices),
	AssetHub(AssetHubIndices),
}

impl Default for ChainIndices {
	fn default() -> Self {
		ChainIndices::Relay(RelayChainIndices::default())
	}
}
