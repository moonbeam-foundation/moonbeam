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

//! # Common Moonbeam Migrations

use core::marker::PhantomData;
use frame_support::migrations::SteppedMigration;
use frame_support::migrations::SteppedMigrationError;
use frame_support::traits::PalletInfoAccess;
use frame_support::weights::WeightMeter;
use pallet_migrations::WeightInfo;
use parity_scale_codec::Encode;
use sp_core::{parameter_types, twox_128, Get};
use sp_io::{storage::clear_prefix, KillStorageResult};
use sp_runtime::SaturatedConversion;

/// Remove all of a pallet's state and re-initializes it to the current in-code storage version.
///
/// It uses the multi block migration frame. Hence it is safe to use even on
/// pallets that contain a lot of storage.
///
/// # Parameters
///
/// - T: The runtime. Used to access the weight definition.
/// - P: The pallet name to be removed as defined in construct runtime
///
/// # Note
///
/// If your pallet does rely of some state in genesis you need to take care of that
/// separately. This migration only sets the storage version after wiping.
pub struct RemovePallet<T, P>(PhantomData<(T, P)>);

impl<T, P> RemovePallet<T, P>
where
	P: Get<&'static str>,
{
	#[cfg(feature = "try-runtime")]
	fn num_keys() -> u64 {
		let prefix = twox_128(P::get().as_bytes()).to_vec();
		frame_support::storage::KeyPrefixIterator::new(prefix.clone(), prefix, |_| Ok(())).count()
			as _
	}
}

impl<T, P> SteppedMigration for RemovePallet<T, P>
where
	T: pallet_migrations::Config,
	P: Get<&'static str>,
{
	type Cursor = bool;
	type Identifier = [u8; 16];

	fn id() -> Self::Identifier {
		("RemovePallet::", P::get()).using_encoded(twox_128)
	}

	fn step(
		cursor: Option<Self::Cursor>,
		meter: &mut WeightMeter,
	) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
		// we write the storage version in a separate block
		if cursor.unwrap_or(false) {
			let required = T::DbWeight::get().writes(1);
			meter
				.try_consume(required)
				.map_err(|_| SteppedMigrationError::InsufficientWeight { required })?;
			return Ok(None);
		}

		let base_weight = T::WeightInfo::reset_pallet_migration(0);
		let weight_per_key = T::WeightInfo::reset_pallet_migration(1).saturating_sub(base_weight);
		let key_budget = meter
			.remaining()
			.saturating_sub(base_weight)
			.checked_div_per_component(&weight_per_key)
			.unwrap_or_default()
			.saturated_into();

		if key_budget == 0 {
			return Err(SteppedMigrationError::InsufficientWeight {
				required: T::WeightInfo::reset_pallet_migration(1),
			});
		}

		let (keys_removed, is_done) =
			match clear_prefix(&twox_128(P::get().as_bytes()), Some(key_budget)) {
				KillStorageResult::AllRemoved(value) => (value, true),
				KillStorageResult::SomeRemaining(value) => (value, false),
			};

		meter.consume(T::WeightInfo::reset_pallet_migration(keys_removed));

		Ok(Some(is_done))
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, sp_runtime::TryRuntimeError> {
		let num_keys: u64 = Self::num_keys();
		log::info!(
			"RemovePallet<{}>: Trying to remove {num_keys} keys.",
			P::get()
		);
		Ok(num_keys.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		use parity_scale_codec::Decode;
		let keys_before = u64::decode(&mut state.as_ref()).expect("We encoded as u64 above; qed");
		let keys_now = Self::num_keys();
		log::info!(
			"RemovePallet<{}>: Keys remaining after migration: {keys_now}",
			P::get()
		);

		if keys_before <= keys_now {
			log::error!("RemovePallet<{}>: Did not remove any keys.", P::get());
			Err("RemovePallet failed")?;
		}

		if keys_now != 1 {
			log::error!("RemovePallet<{}>: Should have a single key after", P::get());
			Err("RemovePallet failed")?;
		}

		Ok(())
	}
}

/// Reset the pallet's storage.
///
/// It uses the multi block migration frame. Hence it is safe to use even on
/// pallets that contain a lot of storage.
///
/// # Parameters
///
/// - T: The runtime. Used to access the weight definition.
/// - Pallet: The pallet to be resetted
/// - Storage: The pallet storage to be resetted
pub struct ResetStorage<T, Pallet, Storage>(PhantomData<(T, Pallet, Storage)>);

impl<T, Pallet, Storage> ResetStorage<T, Pallet, Storage>
where
	Pallet: PalletInfoAccess,
	Storage: Get<&'static str>,
{
	#[cfg(feature = "try-runtime")]
	fn num_keys() -> u64 {
		let storage_prefix = frame_support::storage::storage_prefix(
			Pallet::name().as_bytes(),
			Storage::get().as_bytes(),
		)
		.to_vec();
		frame_support::storage::KeyPrefixIterator::new(
			storage_prefix.clone(),
			storage_prefix,
			|_| Ok(()),
		)
		.count() as _
	}
}

impl<T, Pallet, Storage> SteppedMigration for ResetStorage<T, Pallet, Storage>
where
	T: pallet_migrations::Config,
	Pallet: PalletInfoAccess,
	Storage: Get<&'static str>,
{
	type Cursor = bool;
	type Identifier = [u8; 16];

	fn id() -> Self::Identifier {
		("ResetStorage", Pallet::name(), Storage::get()).using_encoded(twox_128)
	}

	fn step(
		cursor: Option<Self::Cursor>,
		meter: &mut WeightMeter,
	) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
		// we write the storage version in a separate block
		if cursor.unwrap_or(false) {
			let required = T::DbWeight::get().writes(1);
			meter
				.try_consume(required)
				.map_err(|_| SteppedMigrationError::InsufficientWeight { required })?;
			return Ok(None);
		}

		let base_weight = T::WeightInfo::reset_pallet_migration(0);
		let weight_per_key = T::WeightInfo::reset_pallet_migration(1).saturating_sub(base_weight);
		let key_budget = meter
			.remaining()
			.saturating_sub(base_weight)
			.checked_div_per_component(&weight_per_key)
			.unwrap_or_default()
			.saturated_into();

		if key_budget == 0 {
			return Err(SteppedMigrationError::InsufficientWeight {
				required: T::WeightInfo::reset_pallet_migration(1),
			});
		}

		let storage_prefix = frame_support::storage::storage_prefix(
			Pallet::name().as_bytes(),
			Storage::get().as_bytes(),
		);
		let (keys_removed, is_done) = match clear_prefix(&storage_prefix, Some(key_budget)) {
			KillStorageResult::AllRemoved(value) => (value, true),
			KillStorageResult::SomeRemaining(value) => (value, false),
		};

		meter.consume(T::WeightInfo::reset_pallet_migration(keys_removed));

		Ok(Some(is_done))
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, sp_runtime::TryRuntimeError> {
		let num_keys: u64 = Self::num_keys();
		log::info!(
			"ResetStorage<{}, {}>: Trying to remove {num_keys} keys.",
			Pallet::name(),
			Storage::get()
		);
		Ok(num_keys.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		use parity_scale_codec::Decode;
		let keys_before = u64::decode(&mut state.as_ref()).expect("We encoded as u64 above; qed");
		let keys_now = Self::num_keys();
		log::info!(
			"ResetStorage<{}, {}>: Keys remaining after migration: {keys_now}",
			Pallet::name(),
			Storage::get()
		);

		if keys_before <= keys_now {
			log::error!(
				"ResetStorage<{}, {}>: Did not remove any keys.",
				Pallet::name(),
				Storage::get()
			);
			Err("ResetStorage failed")?;
		}

		if keys_now != 1 {
			log::error!(
				"ResetStorage<{}, {}>: Should have a single key after reset",
				Pallet::name(),
				Storage::get()
			);
			Err("ResetStorage failed")?;
		}

		Ok(())
	}
}

/// Unreleased migrations. Add new ones here:
pub type UnreleasedSingleBlockMigrations = ();

/// Migrations/checks that do not need to be versioned and can run on every update.
pub type PermanentSingleBlockMigrations<Runtime> =
	(pallet_xcm::migration::MigrateToLatestXcmVersion<Runtime>,);

/// All migrations that will run on the next runtime upgrade.
pub type SingleBlockMigrations<Runtime> = (
	UnreleasedSingleBlockMigrations,
	PermanentSingleBlockMigrations<Runtime>,
);

parameter_types! {
	pub const DestinationAssetFeePerSecondStorageName: &'static str = "DestinationAssetFeePerSecond";
}

/// List of common multiblock migrations to be executed by the pallet-migrations pallet.
/// The migrations listed here are common to every moonbeam runtime.
pub type MultiBlockMigrations<Runtime> = (
	ResetStorage<
		Runtime,
		pallet_xcm_transactor::Pallet<Runtime>,
		DestinationAssetFeePerSecondStorageName,
	>,
);
