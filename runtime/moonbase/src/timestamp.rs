// Copyright 2019-2022 PureStake Inc.
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

//! A way to get a relyable timestamp

use crate::AsyncBacking;
use frame_support::storage::types::{StorageValue, ValueQuery};
use frame_support::traits::{StorageInstance, Time};
pub use moonbeam_core_primitives::RELAY_CHAIN_SLOT_DURATION_MILLIS;

/// Compute the current timestamp from the last relay slot
pub struct TimestampFromRelaySlot;

impl Time for TimestampFromRelaySlot {
	type Moment = u64;

	fn now() -> Self::Moment {
		if let Some((last_slot, _)) = AsyncBacking::slot_info() {
			RelayGenesisTime::get()
				+ u64::from(RELAY_CHAIN_SLOT_DURATION_MILLIS).saturating_mul((*last_slot).into())
		} else {
			RelayGenesisTime::get()
		}
	}
}

// Prefix for storage value RelayGenesisTime
pub struct RelayGenesisTimePrefix;
impl StorageInstance for RelayGenesisTimePrefix {
	const STORAGE_PREFIX: &'static str = "RelayGenesisTime";

	fn pallet_prefix() -> &'static str {
		"runtime"
	}
}

// Storage type used to store relay genesis time
type RelayGenesisTime = StorageValue<RelayGenesisTimePrefix, u64, ValueQuery>;
