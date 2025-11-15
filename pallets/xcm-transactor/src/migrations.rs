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

//! XCM Transactor pallet migrations

use crate::{
	chain_indices::{AssetHubIndices, ChainIndices},
	pallet::Config,
	ChainIndicesMap, RelayIndices,
};
use frame_support::{
	pallet_prelude::*,
	traits::{OnRuntimeUpgrade, StorageVersion},
	weights::Weight,
};
use sp_std::marker::PhantomData;

/// The current storage version
pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

/// Migration from RelayIndices (single StorageValue) to ChainIndicesMap (StorageMap)
///
/// This migration:
/// 1. Reads the old RelayIndices storage
/// 2. Inserts it into ChainIndicesMap under the Relay transactor key
/// 3. Initializes AssetHub indices with network-specific values
/// 4. Keeps the old RelayIndices storage for backwards compatibility (deprecated)
///
/// # Type Parameters
/// - `AssetHubIndicesValue`: A `Get<AssetHubIndices>` that provides the network-specific
///   AssetHub indices (e.g., Polkadot, Kusama, or Westend AssetHub)
pub mod v1 {
	use super::*;

	pub struct MigrateToChainIndicesMap<T, RelayTransactor, AssetHubTransactor, AssetHubIndicesValue>(
		PhantomData<(T, RelayTransactor, AssetHubTransactor, AssetHubIndicesValue)>,
	);

	impl<T, RelayTransactor, AssetHubTransactor, AssetHubIndicesValue> OnRuntimeUpgrade
		for MigrateToChainIndicesMap<T, RelayTransactor, AssetHubTransactor, AssetHubIndicesValue>
	where
		T: Config,
		RelayTransactor: Get<T::Transactor>,
		AssetHubTransactor: Get<T::Transactor>,
		AssetHubIndicesValue: Get<AssetHubIndices>,
	{
		fn on_runtime_upgrade() -> Weight {
			let mut weight = T::DbWeight::get().reads(1);

			// Check if migration is needed by seeing if ChainIndicesMap is empty
			let relay_key = RelayTransactor::get();
			if ChainIndicesMap::<T>::contains_key(&relay_key) {
				return weight;
			}

			// Step 1: Migrate old RelayIndices to ChainIndicesMap
			let old_relay_indices = RelayIndices::<T>::get();

			// Only migrate if there's data (non-default)
			if old_relay_indices != Default::default() {
				ChainIndicesMap::<T>::insert(&relay_key, ChainIndices::Relay(old_relay_indices));
				weight = weight.saturating_add(T::DbWeight::get().writes(1));
			}

			// Step 2: Initialize AssetHub indices with network-specific values
			let assethub_key = AssetHubTransactor::get();
			let assethub_indices = AssetHubIndicesValue::get();
			ChainIndicesMap::<T>::insert(&assethub_key, ChainIndices::AssetHub(assethub_indices));
			weight = weight.saturating_add(T::DbWeight::get().writes(1));

			// Note: We keep RelayIndices storage for backwards compatibility
			// It will be removed in a future version

			weight
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
			use parity_scale_codec::Encode;

			// Store the current RelayIndices for verification
			let old_indices = RelayIndices::<T>::get();
			let has_relay_data = old_indices != Default::default();

			// Encode state for post-upgrade verification
			Ok((has_relay_data, old_indices).encode())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
			use crate::chain_indices::RelayChainIndices;
			use parity_scale_codec::Decode;

			// Decode pre-upgrade state
			let (had_relay_data, old_relay_indices): (bool, RelayChainIndices) =
				Decode::decode(&mut &state[..])
					.map_err(|_| "Failed to decode pre-upgrade state")?;

			let relay_key = RelayTransactor::get();
			let assethub_key = AssetHubTransactor::get();

			// Verify Relay indices were migrated correctly
			if had_relay_data {
				let migrated_relay = ChainIndicesMap::<T>::get(relay_key)
					.ok_or("Relay indices not found in ChainIndicesMap")?;

				match migrated_relay {
					ChainIndices::Relay(indices) => {
						if indices != old_relay_indices {
							return Err("Migrated Relay indices don't match original".into());
						}
					}
					_ => {
						return Err("Expected ChainIndices::Relay variant".into());
					}
				}

				// Verify old storage still exists (backwards compat)
				let current_relay_indices = RelayIndices::<T>::get();
				if current_relay_indices != old_relay_indices {
					return Err("RelayIndices storage should remain unchanged".into());
				}
			}

			// Verify AssetHub indices were initialized
			let assethub_indices = ChainIndicesMap::<T>::get(assethub_key)
				.ok_or("AssetHub indices not found in ChainIndicesMap")?;

			match assethub_indices {
				ChainIndices::AssetHub(indices) => {
					// Verify key indices are set (not all zeros)
					if indices.utility == 0 && indices.proxy == 0 && indices.staking == 0 {
						return Err("AssetHub indices are all zero".into());
					}
				}
				_ => {
					return Err("Expected ChainIndices::AssetHub variant".into());
				}
			}

			Ok(())
		}
	}
}
