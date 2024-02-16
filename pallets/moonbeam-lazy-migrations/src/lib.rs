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
	use frame_support::traits::{LockIdentifier, LockableCurrency};
	use frame_system::pallet_prelude::*;
	use pallet_democracy::VotingOf;

	const INTERMEDIATES_NODES_SIZE: u64 = 4096;
	const MAX_LOCAL_ASSETS_STORAGE_ENTRY_SIZE: u64 =
		(/* biggest key on moonbeam */136) + (/* biggest value on moonbeam */142);

	/// Copied from pallet-democracy
	const DEMOCRACY_ID: LockIdentifier = *b"democrac";
	const MAX_DEMOCRACY_VOTINGOF_STORAGE_ENTRY_SIZE: u64 =
		(/* biggest key on moonbeam */60) + (/* biggest value on moonbeam */1440);
	const MAX_BALANCES_LOCKS_STORAGE_ENTRY_SIZE: u64 =
		(/* biggest key on moonbeam */60) + (/* biggest value on moonbeam */26 * 3);

	/// Pallet for multi block migrations
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	/// If true, it means that LocalAssets storage has been removed.
	pub(crate) type LocalAssetsMigrationCompleted<T: Config> = StorageValue<_, bool, ValueQuery>;

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_democracy::Config {}

	#[pallet::error]
	pub enum Error<T> {
		/// There are no more storage entries to be removed
		AllStorageEntriesHaveBeenRemoved,
		/// The limit cannot be zero
		LimitCannotBeZero,
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
			ensure!(limit != 0, "Limit cannot be zero!");

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

		// TODO(alexandru): This extrinsic should be removed once Gov V1 is removed.
		// Note: We don't need to unreserve any funds, as they are assumed to be already
		// unreserved prior to this operation and the proposal submission disabled.
		#[pallet::call_index(1)]
		#[pallet::weight(
			Weight::from_parts(0,
				(MAX_BALANCES_LOCKS_STORAGE_ENTRY_SIZE + MAX_DEMOCRACY_VOTINGOF_STORAGE_ENTRY_SIZE) * <u64>::from(*limit))
				.saturating_add(<T as frame_system::Config>::DbWeight::get()
				.reads_writes((*limit + 1).into(), (*limit + 1).into()).saturating_mul(2))
		)]
		pub fn unlock_democracy_funds(
			origin: OriginFor<T>,
			limit: u32,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			ensure!(limit != 0, Error::<T>::LimitCannotBeZero);

			// Unlock staked funds and remove the voting entry. This way we can keep track of what
			// is left without extra cost.
			let unlocked_accounts = VotingOf::<T>::iter()
				.drain()
				.take(limit as usize)
				.map(|(account, _)| T::Currency::remove_lock(DEMOCRACY_ID, &account))
				.count() as u64;

			log::info!("Unlocked {} accounts 🧹", unlocked_accounts);

			Ok(Some(
				Weight::from_parts(
					0,
					(MAX_BALANCES_LOCKS_STORAGE_ENTRY_SIZE
						+ MAX_DEMOCRACY_VOTINGOF_STORAGE_ENTRY_SIZE)
						* <u64>::from(limit),
				)
				.saturating_add(
					<T as frame_system::Config>::DbWeight::get()
						.reads_writes((limit + 1).into(), (limit + 1).into())
						.saturating_mul(2), // once for VotingOf and once for locks
				),
			)
			.into())
		}
	}
}
