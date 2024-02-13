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

#![allow(non_camel_case_types)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_core::ConstBool;

	const INTERMEDIATES_NODES_SIZE: u64 = 4096;
	const MAX_LOCAL_ASSETS_STORAGE_ENTRY_SIZE: u64 =
		(/* biggest key on moonbeam */136) + (/* biggest value on moonbeam */142);

	/// Pallet for multi block migrations
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	/// If true, it means that LocalAssets storage has been removed.
	pub(crate) type LocalAssetsMigrationCompleted<T: Config> =
		StorageValue<_, bool, ValueQuery, ConstBool<false>>;

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::error]
	pub enum Error<T> {
		/// There are no more storage entries to be removed
		AllStorageEntriesHaveBeenRemoved,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO(rodrigo): This extrinsic should be removed once LocalAssets pallet storage is removed
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(0,
			INTERMEDIATES_NODES_SIZE + MAX_LOCAL_ASSETS_STORAGE_ENTRY_SIZE * <u64>::from(*limit))
			.saturating_add(<T as frame_system::Config>::DbWeight::get().reads_writes((*limit + 1).into(), (*limit).into()))
		)]
		pub fn clear_local_assets_storage(
			origin: OriginFor<T>,
			limit: u32,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			ensure!(
				!LocalAssetsMigrationCompleted::<T>::get(),
				Error::<T>::AllStorageEntriesHaveBeenRemoved
			);

			let hashed_prefix = sp_io::hashing::twox_128("LocalAssets".as_bytes());

			let keys_removed = match sp_io::storage::clear_prefix(&hashed_prefix, Some(limit)) {
				sp_io::KillStorageResult::AllRemoved(value) => {
					LocalAssetsMigrationCompleted::<T>::set(true);
					value
				}
				sp_io::KillStorageResult::SomeRemaining(value) => value,
			} as u64;

			log::info!("Removed {} keys 🧹", keys_removed);

			Ok(Some(
				Weight::from_parts(
					0,
					INTERMEDIATES_NODES_SIZE + MAX_LOCAL_ASSETS_STORAGE_ENTRY_SIZE * keys_removed,
				)
				.saturating_add(
					<T as frame_system::Config>::DbWeight::get()
						.reads_writes(keys_removed + 1, keys_removed),
				),
			)
			.into())
		}
	}
}
