// Copyright 2024 Moonbeam Foundation.
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

//! This pallet is designed for benchmarking precompile functions. It should not be used in
//! production.

#![cfg_attr(not(feature = "std"), no_std)]

use cumulus_primitives_core::relay_chain::BlockNumber as RelayBlockNumber;
use frame_support::pallet;
pub use pallet::*;
use storage_proof_primitives::verify_relay_entry;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;

pub mod weights;
pub use crate::weights::WeightInfo;

#[pallet]
pub mod pallet {
	use storage_proof_primitives::RawStorageProof;

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_relay_storage_roots::Config {
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		BenchmarkError,
	}

	impl<T: Config> Pallet<T> {
		pub fn verify(
			relay_block_number: RelayBlockNumber,
			proof: RawStorageProof,
			key: &[u8],
		) -> Result<(), Error<T>> {
			verify_relay_entry::<T>(relay_block_number, proof, key)
				.map_err(|_| Error::<T>::BenchmarkError)?;
			Ok(())
		}

		#[allow(dead_code)]
		pub fn latest_relay_block() -> Result<RelayBlockNumber, Error<T>> {
			pallet_relay_storage_roots::RelayStorageRootKeys::<T>::get()
				.last()
				.cloned()
				.ok_or(Error::<T>::BenchmarkError)
		}
	}
}
