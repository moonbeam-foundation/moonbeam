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
use pallet_evm_precompile_relay_verifier::{RawKey, ReadProof, RelayDataVerifierPrecompile};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;

#[pallet]
pub mod pallet {
	use super::*;
	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_relay_storage_roots::Config + pallet_evm::Config
	{
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		BenchmarkError,
	}

	impl<T: Config> Pallet<T> {
		pub fn verify_entry(
			relay_block_number: RelayBlockNumber,
			proof: ReadProof,
			key: RawKey,
		) -> Result<(), Error<T>> {
			RelayDataVerifierPrecompile::<T>::do_verify_entry(relay_block_number, proof, key)
				.map_err(|_| Error::<T>::BenchmarkError)?;
			Ok(())
		}

		pub fn verify_entries(
			relay_block_number: RelayBlockNumber,
			proof: ReadProof,
			keys: Vec<RawKey>,
		) -> Result<(), Error<T>> {
			RelayDataVerifierPrecompile::<T>::do_verify_entries(
				relay_block_number,
				proof,
				keys.into(),
			)
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
