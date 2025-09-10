// Copyright 2024 Moonbeam foundation
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

//! # Lazy Migration Pallet
//!
//! An empty pallet reserved for future migrations.

#![allow(non_camel_case_types)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
pub mod mock;

#[cfg(test)]
mod tests;

// #[cfg(any(test, feature = "runtime-benchmarks"))]
// mod benchmarks;

pub mod weights;
pub use weights::WeightInfo;

use frame_support::pallet;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
pub use pallet::*;

const MAX_CONTRACT_CODE_SIZE: u64 = 25 * 1024;

#[pallet]
pub mod pallet {

	use super::*;
	use sp_core::H160;

	/// Pallet for multi block migrations
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The contract already have metadata
		ContractMetadataAlreadySet,
		/// Contract not exist
		ContractNotExist,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T as frame_system::Config>::AccountId: Into<H160> + From<H160>,
	{
		#[pallet::call_index(2)]
		#[pallet::weight(Pallet::<T>::create_contract_metadata_weight(MAX_CONTRACT_CODE_SIZE))]
		pub fn create_contract_metadata(
			origin: OriginFor<T>,
			address: H160,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			ensure!(
				pallet_evm::AccountCodesMetadata::<T>::get(address).is_none(),
				Error::<T>::ContractMetadataAlreadySet
			);

			// Ensure contract exist
			let code = pallet_evm::AccountCodes::<T>::get(address);
			ensure!(!code.is_empty(), Error::<T>::ContractNotExist);

			// Construct metadata
			let code_size = code.len() as u64;
			let code_hash = sp_core::H256::from(sp_io::hashing::keccak_256(&code));
			let meta = pallet_evm::CodeMetadata {
				size: code_size,
				hash: code_hash,
			};

			// Set metadata
			pallet_evm::AccountCodesMetadata::<T>::insert(address, meta);

			Ok((
				Some(Self::create_contract_metadata_weight(code_size)),
				Pays::No,
			)
				.into())
		}
	}

	impl<T: Config> Pallet<T> {
		fn create_contract_metadata_weight(code_size: u64) -> Weight {
			// max entry size of AccountCodesMetadata (full key + value)
			const PROOF_SIZE_CODE_METADATA: u64 = 100;
			// intermediates nodes might be up to 3Kb
			const PROOF_SIZE_INTERMEDIATES_NODES: u64 = 3 * 1024;

			// Account for 2 reads, 1 write
			<T as frame_system::Config>::DbWeight::get()
				.reads_writes(2, 1)
				.set_proof_size(
					code_size + (PROOF_SIZE_INTERMEDIATES_NODES * 2) + PROOF_SIZE_CODE_METADATA,
				)
		}
	}
}
