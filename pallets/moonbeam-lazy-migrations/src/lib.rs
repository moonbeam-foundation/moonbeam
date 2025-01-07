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
pub mod mock;
#[cfg(test)]
mod tests;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;

mod foreign_asset;
pub mod weights;
pub use weights::WeightInfo;

use frame_support::pallet;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
pub use pallet::*;
use xcm::latest::Location;

const MAX_CONTRACT_CODE_SIZE: u64 = 25 * 1024;

environmental::environmental!(MIGRATING_FOREIGN_ASSETS: bool);

#[pallet]
pub mod pallet {
	use super::*;
	use crate::foreign_asset::ForeignAssetMigrationStatus;
	use sp_core::{H160, U256};

	pub const ARRAY_LIMIT: u32 = 1000;
	pub type GetArrayLimit = ConstU32<ARRAY_LIMIT>;

	/// Pallet for multi block migrations
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	pub(crate) type StateMigrationStatusValue<T: Config> =
		StorageValue<_, (StateMigrationStatus, u64), ValueQuery>;

	#[pallet::storage]
	pub(crate) type ForeignAssetMigrationStatusValue<T: Config> =
		StorageValue<_, ForeignAssetMigrationStatus, ValueQuery>;

	// List of approved foreign assets to be migrated
	#[pallet::storage]
	pub(crate) type ApprovedForeignAssets<T: Config> =
		StorageMap<_, Twox64Concat, u128, (), OptionQuery>;

	pub(crate) type StorageKey = BoundedVec<u8, ConstU32<1_024>>;

	#[derive(Clone, Encode, Decode, scale_info::TypeInfo, PartialEq, Eq, MaxEncodedLen, Debug)]
	pub enum StateMigrationStatus {
		NotStarted,
		Started(StorageKey),
		Error(BoundedVec<u8, ConstU32<1024>>),
		Complete,
	}

	impl Default for StateMigrationStatus {
		fn default() -> Self {
			StateMigrationStatus::NotStarted
		}
	}
	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ pallet_evm::Config
		+ pallet_balances::Config
		+ pallet_assets::Config<AssetId = u128>
		+ pallet_asset_manager::Config<AssetId = u128>
		+ pallet_moonbeam_foreign_assets::Config
	{
		// Origin that is allowed to start foreign assets migration
		type ForeignAssetMigratorOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The limit cannot be zero
		LimitCannotBeZero,
		/// The contract already have metadata
		ContractMetadataAlreadySet,
		/// Contract not exist
		ContractNotExist,
		/// The symbol length exceeds the maximum allowed
		SymbolTooLong,
		/// The name length exceeds the maximum allowed
		NameTooLong,
		/// The asset type was not found
		AssetTypeNotFound,
		/// Asset not found
		AssetNotFound,
		/// The location of the asset was not found
		LocationNotFound,
		/// Migration is not finished yet
		MigrationNotFinished,
		/// No migration in progress
		NoMigrationInProgress,
		/// Fail to mint the foreign asset
		MintFailed,
		/// Fail to add an approval
		ApprovalFailed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T as pallet_assets::Config>::Balance: Into<U256>,
		<T as pallet_asset_manager::Config>::ForeignAssetType: Into<Option<Location>>,
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

		#[pallet::call_index(3)]
		#[pallet::weight(
			<T as pallet::Config>::WeightInfo::approve_assets_to_migrate(assets.len() as u32)
		)]
		pub fn approve_assets_to_migrate(
			origin: OriginFor<T>,
			assets: BoundedVec<u128, GetArrayLimit>,
		) -> DispatchResultWithPostInfo {
			T::ForeignAssetMigratorOrigin::ensure_origin(origin.clone())?;

			assets.iter().try_for_each(|asset_id| {
				ensure!(
					pallet_assets::Asset::<T>::contains_key(*asset_id),
					Error::<T>::AssetNotFound
				);

				ApprovedForeignAssets::<T>::insert(asset_id, ());
				Ok::<(), Error<T>>(())
			})?;
			Ok(Pays::No.into())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::start_foreign_assets_migration())]
		pub fn start_foreign_assets_migration(
			origin: OriginFor<T>,
			asset_id: u128,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			Self::do_start_foreign_asset_migration(asset_id)?;
			Ok(Pays::No.into())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::migrate_foreign_asset_balances(*limit))]
		pub fn migrate_foreign_asset_balances(
			origin: OriginFor<T>,
			limit: u32,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			Self::do_migrate_foreign_asset_balances(limit)?;
			Ok(Pays::No.into())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::migrate_foreign_asset_approvals(*limit))]
		pub fn migrate_foreign_asset_approvals(
			origin: OriginFor<T>,
			limit: u32,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			Self::do_migrate_foreign_asset_approvals(limit)?;
			Ok(Pays::No.into())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::finish_foreign_assets_migration())]
		pub fn finish_foreign_assets_migration(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			Self::do_finish_foreign_asset_migration()?;
			Ok(Pays::No.into())
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

pub fn is_migrating_foreign_assets() -> bool {
	MIGRATING_FOREIGN_ASSETS::with(|v| *v).unwrap_or(false)
}
