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
use pallet_evm_precompile_relay_verifier::{
	proof::{ReadProof, StorageProofChecker},
	GetCallDataLimit, GetKeyLengthLimit,
};
use sp_core::H256;

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_relay_storage_roots::Config {}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		BenchmarkError,
	}

	impl<T: Config> Pallet<T> {
		pub fn verify_relay_entry(
			relay_block_number: RelayBlockNumber,
			key: BoundedVec<u8, GetKeyLengthLimit>,
			proof: BoundedVec<u8, GetCallDataLimit>,
		) -> Result<Vec<u8>, Error<T>> {
			let storage_root = Self::get_storage_root(relay_block_number)?;

			// Decode the proof of type `ReadProof` (The proof is expected to be
			// a SCALE encoded `ReadProof` that is returned by the `state_getProof` RPC call).
			let proof =
				ReadProof::decode(&mut proof.as_slice()).map_err(|_| Error::<T>::BenchmarkError)?;

			let proof_checker = StorageProofChecker::new(storage_root, proof.proof)
				.map_err(|_| Error::<T>::BenchmarkError)?;

			let value: Vec<u8> = proof_checker
				.read_entry(key.as_slice(), None)
				.map_err(|_| Error::<T>::BenchmarkError)?;

			Ok(value)
		}

		pub fn verify_relay_entries(
			relay_block_number: RelayBlockNumber,
			keys: Vec<Vec<u8>>,
			proof: Vec<u8>,
		) -> Result<Vec<Vec<u8>>, Error<T>> {
			let storage_root = Self::get_storage_root(relay_block_number)?;

			let proof =
				ReadProof::decode(&mut proof.as_slice()).map_err(|_| Error::<T>::BenchmarkError)?;

			let proof_checker = StorageProofChecker::new(storage_root, proof.proof)
				.map_err(|_| Error::<T>::BenchmarkError)?;

			let mut values = Vec::new();
			for key in Vec::from(keys) {
				let value: Vec<u8> = proof_checker
					.read_entry(key.as_slice(), None)
					.map_err(|_| Error::<T>::BenchmarkError)?;
				values.push(value.into());
			}

			Ok(values.into())
		}

		#[allow(dead_code)]
		fn latest_relay_block() -> Result<RelayBlockNumber, Error<T>> {
			pallet_relay_storage_roots::RelayStorageRootKeys::<T>::get()
				.last()
				.cloned()
				.ok_or(Error::<T>::BenchmarkError)
		}

		/// Returns the storage root at the given relay block number stored on-chain. Use the pallet
		/// `pallet_relay_storage_roots` to store the storage roots on-chain.
		fn get_storage_root(relay_block_number: RelayBlockNumber) -> Result<H256, Error<T>> {
			let storage_root =
				pallet_relay_storage_roots::RelayStorageRoot::<T>::get(relay_block_number)
					.ok_or(Error::<T>::BenchmarkError)?;
			Ok(storage_root)
		}
	}
}
