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

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::WeightInfo;

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_core::H160;

	pub const ARRAY_LIMIT: u32 = 1000;
	pub type GetArrayLimit = ConstU32<ARRAY_LIMIT>;

	const INTERMEDIATES_NODES_SIZE: u64 = 4096;
	const MAX_LOCAL_ASSETS_STORAGE_ENTRY_SIZE: u64 =
		(/* biggest key on moonbeam */136) + (/* biggest value on moonbeam */142);

	/// Pallet for multi block migrations
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	/// If true, it means that LocalAssets storage has been removed.
	pub(crate) type LocalAssetsMigrationCompleted<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	/// The total number of suicided contracts that were removed
	pub(crate) type SuicidedContractsRemoved<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// There are no more storage entries to be removed
		AllStorageEntriesHaveBeenRemoved,
		/// The limit cannot be zero
		LimitCannotBeZero,
		/// The limit for unlocking funds is too high
		UnlockLimitTooHigh,
		/// There are no more VotingOf entries to be removed and democracy funds to be unlocked
		AllDemocracyFundsUnlocked,
		/// There must be at least one address
		AddressesLengthCannotBeZero,
		/// The contract is not corrupted (Still exist or properly suicided)
		ContractNotCorrupted,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO(rodrigo): This extrinsic should be removed once LocalAssets pallet storage is removed
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(0,
			INTERMEDIATES_NODES_SIZE + (MAX_LOCAL_ASSETS_STORAGE_ENTRY_SIZE * <u64>::from(*limit)))
			.saturating_add(<T as frame_system::Config>::DbWeight::get()
				.reads_writes((*limit + 1).into(), (*limit + 1).into()))
		)]
		pub fn clear_local_assets_storage(
			origin: OriginFor<T>,
			limit: u32,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			ensure!(limit != 0, Error::<T>::LimitCannotBeZero);

			let mut weight = <T as frame_system::Config>::DbWeight::get().reads(1);
			ensure!(
				!LocalAssetsMigrationCompleted::<T>::get(),
				Error::<T>::AllStorageEntriesHaveBeenRemoved
			);

			let hashed_prefix = sp_io::hashing::twox_128("LocalAssets".as_bytes());

			let keys_removed = match sp_io::storage::clear_prefix(&hashed_prefix, Some(limit)) {
				sp_io::KillStorageResult::AllRemoved(value) => {
					LocalAssetsMigrationCompleted::<T>::set(true);
					weight
						.saturating_accrue(<T as frame_system::Config>::DbWeight::get().writes(1));
					value
				}
				sp_io::KillStorageResult::SomeRemaining(value) => value,
			} as u64;

			log::info!("Removed {} keys 🧹", keys_removed);

			Ok(Some(
				weight
					.saturating_add(Weight::from_parts(
						0,
						INTERMEDIATES_NODES_SIZE
							+ MAX_LOCAL_ASSETS_STORAGE_ENTRY_SIZE * keys_removed,
					))
					.saturating_add(
						<T as frame_system::Config>::DbWeight::get()
							.reads_writes(keys_removed, keys_removed),
					),
			)
			.into())
		}

		// TODO(rodrigo): This extrinsic should be removed once the storage of destroyed contracts
		// has been removed
		#[pallet::call_index(1)]
		#[pallet::weight({
			let addresses_len = addresses.len() as u32;
			<T as crate::Config>::WeightInfo::clear_suicided_storage(addresses_len, *limit)
		})]
		pub fn clear_suicided_storage(
			origin: OriginFor<T>,
			addresses: BoundedVec<H160, GetArrayLimit>,
			limit: u32,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			ensure!(limit != 0, Error::<T>::LimitCannotBeZero);
			ensure!(
				addresses.len() != 0,
				Error::<T>::AddressesLengthCannotBeZero
			);

			let mut limit = limit as usize;

			for address in &addresses {
				// Ensure that the contract is corrupted by checking
				// that it has no code and at least one storage entry.
				let suicided = pallet_evm::Suicided::<T>::contains_key(&address);
				let has_code = pallet_evm::AccountCodes::<T>::contains_key(&address);
				ensure!(
					!suicided
						&& !has_code && pallet_evm::AccountStorages::<T>::iter_key_prefix(&address)
						.next()
						.is_some(),
					Error::<T>::ContractNotCorrupted
				);

				let deleted = pallet_evm::AccountStorages::<T>::drain_prefix(*address)
					.take(limit)
					.count();

				// Check if the storage of this contract has been completly removed
				if pallet_evm::AccountStorages::<T>::iter_key_prefix(&address)
					.next()
					.is_none()
				{
					// All entries got removed, lets count this address as migrated
					SuicidedContractsRemoved::<T>::mutate(|x| *x = x.saturating_add(1));
				}

				limit = limit.saturating_sub(deleted);
				if limit == 0 {
					return Ok(Pays::No.into());
				}
			}
			Ok(Pays::No.into())
		}
	}
}
