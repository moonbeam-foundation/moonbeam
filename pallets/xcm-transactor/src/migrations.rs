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
/// 1. Migrates existing RelayIndices to ChainIndicesMap under the Relay transactor key
/// 2. Initializes AssetHub indices with network-specific values
/// 3. Keeps the old RelayIndices storage for backwards compatibility (deprecated)
///
/// Note: For fresh chains, ChainIndicesMap should be initialized via genesis config,
/// not via this migration.
///
/// # Type Parameters
/// - `RelayIndicesValue`: A `Get<RelayChainIndices>` that provides the network-specific
///   Relay chain indices (e.g., Polkadot, Kusama, or Westend relay)
/// - `AssetHubIndicesValue`: A `Get<AssetHubIndices>` that provides the network-specific
///   AssetHub indices (e.g., Polkadot, Kusama, or Westend AssetHub)
pub mod v1 {
	use super::*;
	use crate::chain_indices::RelayChainIndices;
	#[cfg(feature = "try-runtime")]
	use sp_std::vec::Vec;

	pub struct MigrateToChainIndicesMap<
		T,
		RelayTransactor,
		AssetHubTransactor,
		RelayIndicesValue,
		AssetHubIndicesValue,
	>(
		PhantomData<(
			T,
			RelayTransactor,
			AssetHubTransactor,
			RelayIndicesValue,
			AssetHubIndicesValue,
		)>,
	);

	impl<T, RelayTransactor, AssetHubTransactor, RelayIndicesValue, AssetHubIndicesValue>
		OnRuntimeUpgrade
		for MigrateToChainIndicesMap<
			T,
			RelayTransactor,
			AssetHubTransactor,
			RelayIndicesValue,
			AssetHubIndicesValue,
		>
	where
		T: Config,
		RelayTransactor: Get<T::Transactor>,
		AssetHubTransactor: Get<T::Transactor>,
		RelayIndicesValue: Get<RelayChainIndices>,
		AssetHubIndicesValue: Get<AssetHubIndices>,
	{
		fn on_runtime_upgrade() -> Weight {
			let mut weight = T::DbWeight::get().reads(1);

			// Check if migration is needed by seeing if ChainIndicesMap is empty
			let relay_key = RelayTransactor::get();
			if ChainIndicesMap::<T>::contains_key(&relay_key) {
				// Migration already ran or genesis initialized ChainIndicesMap
				// Ensure RelayIndices is also populated for backwards compatibility
				if let Some(ChainIndices::Relay(relay_indices)) =
					ChainIndicesMap::<T>::get(&relay_key)
				{
					RelayIndices::<T>::put(relay_indices);
					weight = weight.saturating_add(T::DbWeight::get().writes(1));
				}
				return weight;
			}

			// Step 1: Migrate existing RelayIndices to ChainIndicesMap
			let old_relay_indices = RelayIndices::<T>::get();

			// If RelayIndices is default, this is likely a fresh chain that should have
			// been initialized via genesis config. Use the network-specific hardcoded values
			// as a fallback to ensure the chain can function.
			let relay_indices_to_use = if old_relay_indices == Default::default() {
				RelayIndicesValue::get()
			} else {
				old_relay_indices
			};

			// Populate both new and old storage
			ChainIndicesMap::<T>::insert(&relay_key, ChainIndices::Relay(relay_indices_to_use));
			RelayIndices::<T>::put(relay_indices_to_use);
			weight = weight.saturating_add(T::DbWeight::get().writes(2));

			// Step 2: Initialize AssetHub indices
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

			// Encode state for post-upgrade verification
			Ok(old_indices.encode())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
			use crate::chain_indices::RelayChainIndices;
			use parity_scale_codec::Decode;

			// Decode pre-upgrade state
			let old_relay_indices: RelayChainIndices = Decode::decode(&mut &state[..])
				.map_err(|_| "Failed to decode pre-upgrade state")?;

			let relay_key = RelayTransactor::get();
			let assethub_key = AssetHubTransactor::get();

			// Verify Relay indices were migrated correctly (always present now)
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
