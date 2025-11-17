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

//! A module that is responsible for migration of storage.

use crate::{Config, RelayIndices};
use cumulus_primitives_core::Weight;
use frame_support::traits::{Get, OnRuntimeUpgrade};
use sp_std::marker::PhantomData;

/// Migrates the pallet storage to v1.
pub struct UpdateRelayChainIndices<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for UpdateRelayChainIndices<T> {
	fn on_runtime_upgrade() -> Weight {
		let mut indices = RelayIndices::<T>::get();

		// https://github.com/polkadot-fellows/runtimes/blob/release-v2.0.0/system-parachains/asset-hubs/asset-hub-kusama/src/lib.rs#L1628
		indices.staking = 89;
		// https://github.com/polkadot-fellows/runtimes/blob/release-v2.0.0/system-parachains/asset-hubs/asset-hub-kusama/src/lib.rs#L1596
		indices.utility = 40;

		RelayIndices::<T>::set(indices);

		T::DbWeight::get().reads_writes(1, 1)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, sp_runtime::DispatchError> {
		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		Ok(())
	}
}
