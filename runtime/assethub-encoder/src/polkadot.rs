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

//! Polkadot AssetHub pallet and call indices
//!
//! These indices have been verified against the actual Polkadot AssetHub runtime metadata.
//!
//! ## Verification
//!
//! Verified using:
//! ```bash
//! subxt metadata --url wss://polkadot-asset-hub-rpc.polkadot.io:443 --format json
//! ```
//!
//! ## Sources
//! - Runtime: Polkadot AssetHub (asset-hub-polkadot)
//! - Metadata version: V16
//! - Last verified: 2025-11-14

use pallet_xcm_transactor::chain_indices::AssetHubIndices;

/// Polkadot AssetHub pallet and extrinsic indices
///
/// All indices have been verified against live Polkadot AssetHub metadata.
pub const POLKADOT_ASSETHUB_INDICES: AssetHubIndices = AssetHubIndices {
	// Pallet indices (from AssetHub Polkadot runtime metadata)
	utility: 40,
	proxy: 42,
	staking: 89,
	nomination_pools: 80,
	delegated_staking: 83,
	assets: 50,
	nfts: 52,

	// Utility call indices
	as_derivative: 1,
	batch: 0,
	batch_all: 2,

	// Proxy call indices
	proxy_call: 0,
	add_proxy: 1,
	remove_proxy: 2,

	// Staking call indices
	bond: 0,
	bond_extra: 1,
	unbond: 2,
	withdraw_unbonded: 3,
	validate: 4,
	nominate: 5,
	chill: 6,
	set_payee: 7,
	set_controller: 8, // Deprecated but present
	rebond: 19,
};
