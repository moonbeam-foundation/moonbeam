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
	use frame_support::traits::ReservableCurrency;
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
	pub trait Config: frame_system::Config + pallet_evm::Config + pallet_balances::Config {
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// There are no more storage entries to be removed
		AllStorageEntriesHaveBeenRemoved,
		/// The limit cannot be zero
		LimitCannotBeZero,
		/// The maximum number of assets cannot be zero
		MaxAssetsCannotBeZero,
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
		#[pallet::weight({
			// "*limit" is used twice to account to the possibility that we may need to unreserve 
			// deposits for every approval
			let possible_iterations = max_assets.saturating_add(*limit).saturating_add(*limit);
			let proof_size = INTERMEDIATES_NODES_SIZE + (MAX_LOCAL_ASSETS_STORAGE_ENTRY_SIZE
				.saturating_mul(<u64>::from(possible_iterations)));

			Weight::from_parts(0, proof_size)
			.saturating_add(<T as frame_system::Config>::DbWeight::get()
				.reads_writes((*max_assets + *limit + 1).into(), (*limit + 1).into()))
		})]
		pub fn clear_local_assets_storage(
			origin: OriginFor<T>,
			max_assets: u32,
			limit: u32,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			ensure!(limit != 0, Error::<T>::LimitCannotBeZero);
			ensure!(max_assets != 0, Error::<T>::MaxAssetsCannotBeZero);

			ensure!(
				!LocalAssetsMigrationCompleted::<T>::get(),
				Error::<T>::AllStorageEntriesHaveBeenRemoved
			);

			let mut allowed_removals = limit;

			const PALLET_PREFIX: &'static str = "LocalAssets";

			struct LocalAssetsStorageAsset;
			impl frame_support::traits::StorageInstance for LocalAssetsStorageAsset {
				const STORAGE_PREFIX: &'static str = "Asset";
				fn pallet_prefix() -> &'static str {
					PALLET_PREFIX
				}
			}
			struct LocalAssetsStorageApprovals;
			impl frame_support::traits::StorageInstance for LocalAssetsStorageApprovals {
				const STORAGE_PREFIX: &'static str = "Approvals";
				fn pallet_prefix() -> &'static str {
					PALLET_PREFIX
				}
			}

			/// Data concerning an approval.
			/// Duplicated the type from pallet_assets (The original type is private)
			#[derive(
				Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, MaxEncodedLen, TypeInfo,
			)]
			pub struct Approval<Balance, DepositBalance> {
				/// The amount of funds approved for the balance transfer from the owner to some delegated
				/// target.
				pub(super) amount: Balance,
				/// The amount reserved on the owner's account to hold this item in storage.
				pub(super) deposit: DepositBalance,
			}

			type AssetMap = frame_support::storage::types::StorageMap<
				LocalAssetsStorageAsset,
				Blake2_128Concat,
				u128,
				// It is fine to add a dummy `Value` type
				// The value is not going to be decoded, since we only care about the keys)
				(),
			>;

			for asset_id in AssetMap::iter_keys().take(max_assets as usize) {
				let approvals_iter = frame_support::storage::types::StorageNMap::<
					LocalAssetsStorageApprovals,
					(
						NMapKey<Blake2_128Concat, u128>,         // asset id
						NMapKey<Blake2_128Concat, T::AccountId>, // owner
						NMapKey<Blake2_128Concat, T::AccountId>, // delegate
					),
					Approval<T::Balance, T::Balance>,
				>::drain_prefix((asset_id,));
				for ((owner, _), approval) in approvals_iter {
					allowed_removals = allowed_removals.saturating_sub(1);
					// Unreserve deposit (most likely will be zero)
					pallet_balances::Pallet::<T>::unreserve(&owner, approval.deposit);
					// Check if the removal limit was reached
					if allowed_removals < 1 {
						break;
					}
				}
				// Remove asset, since it does not contain more approvals
				AssetMap::remove(asset_id);
				allowed_removals = allowed_removals.saturating_sub(1);
				// Check if the removal limit was reached
				if allowed_removals < 1 {
					break;
				}
			}

			// Remove remaining storage
			if allowed_removals > 0 {
				let hashed_prefix = sp_io::hashing::twox_128(PALLET_PREFIX.as_bytes());

				let keys_removed =
					match sp_io::storage::clear_prefix(&hashed_prefix, Some(allowed_removals)) {
						sp_io::KillStorageResult::AllRemoved(value) => {
							LocalAssetsMigrationCompleted::<T>::set(true);
							value
						}
						sp_io::KillStorageResult::SomeRemaining(value) => value,
					};

				allowed_removals = allowed_removals.saturating_sub(keys_removed);
			}

			log::info!(
				"Removed {} storge keys 🧹",
				limit.saturating_sub(allowed_removals)
			);

			Ok(Pays::No.into())
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
