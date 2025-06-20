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

//! # Moonbase bridge primitives

#![cfg_attr(not(feature = "std"), no_std)]

pub mod betanet;
pub mod stagenet;

pub use bp_bridge_hub_cumulus::{
	BlockLength, BlockWeights, Hasher, Nonce, SignedBlock, AVERAGE_BLOCK_INTERVAL,
	MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX, MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX,
};
use bp_runtime::{decl_bridge_finality_runtime_apis, decl_bridge_messages_runtime_apis};

pub use moonbeam_core_primitives::{AccountId, Balance, BlockNumber, Hash, Header, Signature};

/// Bridge lane identifier.
pub type LaneId = bp_messages::HashedLaneId;

decl_bridge_finality_runtime_apis!(moonbase_westend);
decl_bridge_messages_runtime_apis!(moonbase_westend, LaneId);
