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

use crate::Runtime;
use cumulus_pallet_parachain_system::{
	consensus_hook::UnincludedSegmentCapacity,
	relay_state_snapshot::{self, ReadEntryErr},
	ConsensusHook, RelayChainStateProof,
};
use frame_support::pallet_prelude::*;
use frame_support::storage::types::{StorageValue, ValueQuery};
use frame_support::traits::{StorageInstance, Time};
pub use moonbeam_core_primitives::well_known_relay_keys;

/// Get the relay timestamp.
/// Noe that the relay timestamp is populated at the parachain system inherent.
/// If you fetch the timestamp before, you will get the timestamp of the parent block.
pub struct RelayTimestamp;
impl Time for RelayTimestamp {
	type Moment = u64;

	fn now() -> Self::Moment {
		RelayTimestampNow::get()
	}
}

/// A wrapper around the consensus hook to get the relay timlestmap from the relay storage proof
pub struct ConsensusHookWrapperForRelayTimestamp<Inner>(core::marker::PhantomData<Inner>);
impl<Inner: ConsensusHook> ConsensusHook for ConsensusHookWrapperForRelayTimestamp<Inner> {
	fn on_state_proof(state_proof: &RelayChainStateProof) -> (Weight, UnincludedSegmentCapacity) {
		let relay_timestamp: u64 =
			match state_proof.read_entry(well_known_relay_keys::TIMESTAMP_NOW, None) {
				Ok(relay_timestamp) => relay_timestamp,
				// Log the read entry error
				Err(relay_state_snapshot::Error::ReadEntry(ReadEntryErr::Proof)) => {
					log::error!("Invalid relay storage proof: fail to read key TIMESTAMP_NOW");
					panic!("Invalid realy storage proof: fail to read key TIMESTAMP_NOW");
				}
				Err(relay_state_snapshot::Error::ReadEntry(ReadEntryErr::Decode)) => {
					log::error!("Corrupted relay storage: fail to decode value TIMESTAMP_NOW");
					panic!("Corrupted relay storage: fail to decode value TIMESTAMP_NOW");
				}
				Err(relay_state_snapshot::Error::ReadEntry(ReadEntryErr::Absent)) => {
					log::error!("Corrupted relay storage: value TIMESTAMP_NOW is absent!");
					panic!("Corrupted relay storage: value TIMESTAMP_NOW is absent!");
				}
				// Can't return another kind of error, the blokc is invalid anyway, so we should panic
				_ => unreachable!(),
			};

		let wrapper_weight = <Runtime as frame_system::Config>::DbWeight::get().writes(1);

		RelayTimestampNow::put(relay_timestamp);

		let (weight, capacity) = Inner::on_state_proof(state_proof);

		(weight.saturating_add(wrapper_weight), capacity)
	}
}

// Prefix for storage value RelayTimestampNow
struct RelayTimestampNowPrefix;
impl StorageInstance for RelayTimestampNowPrefix {
	const STORAGE_PREFIX: &'static str = "RelayTimestampNow";

	fn pallet_prefix() -> &'static str {
		"runtime"
	}
}

// Storage type used to store the last current relay timestamp
type RelayTimestampNow = StorageValue<RelayTimestampNowPrefix, u64, ValueQuery>;
