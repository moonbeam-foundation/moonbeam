// Copyright 2019-2020 PureStake Inc.
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

//! # Migration Pallet

#![allow(non_camel_case_types)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_support::{pallet, weights::Weight};

pub use pallet::*;

#[cfg(test)]
#[macro_use]
extern crate environmental;

/// A Migration that must happen on-chain upon a runtime-upgrade
pub trait Migration {
	/// A human-readable name for this migration. Also used as storage key.
	fn friendly_name(&self) -> &str;

	/// Perform the required migration and return the weight consumed.
	/// 
	/// Currently there is no way to migrate across blocks, so this method must (1) perform its full
	/// migration and (2) not produce a block that has gone over-weight. Not meeting these strict
	/// constraints will lead to a bricked chain upon a runtime upgrade because the parachain will
	/// not be able to produce a block that the relay chain will accept.
	fn migrate(&self, available_weight: Weight) -> Weight;
}

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;

	/// Pallet for migrations
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The list of migrations that will be performed
		type MigrationsList: Get<Vec<Box<dyn Migration>>>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		RuntimeUpgradeStarted(),
		RuntimeUpgradeCompleted(Weight),
		MigrationStarted(Vec<u8>),
		MigrationCompleted(Vec<u8>, Weight),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// on_runtime_upgrade is expected to be called exactly once after a runtime upgrade.
		/// We use this as a chance to flag that we are now in upgrade-mode and begin our
		/// migrations.
		fn on_runtime_upgrade() -> Weight {
			log::warn!("Performing on_runtime_upgrade");

			let mut weight: Weight = 0u64.into();
			// TODO: derive a suitable value here, which is probably something < max_block
			let available_weight: Weight = T::BlockWeights::get().max_block;

			// start by flagging that we are not fully upgraded
			<FullyUpgraded<T>>::put(false);
			weight += T::DbWeight::get().writes(1);
			Self::deposit_event(Event::RuntimeUpgradeStarted());

			weight += perform_runtime_upgrades::<T>(available_weight.saturating_sub(weight));

			if !<FullyUpgraded<T>>::get() {
				log::error!(
					"migrations weren't completed in on_runtime_upgrade(), but we're not
				configured for multi-block migrations; state is potentially inconsistent!"
				);
			}

			weight
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn is_fully_upgraded)]
	/// True if all required migrations have completed
	type FullyUpgraded<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn migration_state)]
	/// MigrationState tracks the progress of a migration.
	/// Maps name (Vec<u8>) -> whether or not migration has been completed (bool)
	type MigrationState<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, bool, OptionQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub completed_migrations: Vec<Vec<u8>>,
		pub phantom: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				completed_migrations: vec![],
				phantom: PhantomData,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for migration_name in &self.completed_migrations {
				<MigrationState<T>>::insert(migration_name, true);
			}
		}
	}

	fn perform_runtime_upgrades<T: Config>(available_weight: Weight) -> Weight {
		let mut weight: Weight = 0u64.into();

		for migration in &T::MigrationsList::get() {
			let migration_name = migration.friendly_name();
			let migration_name_as_bytes = migration_name.as_bytes();
			log::trace!("evaluating migration {}", migration_name);

			let migration_done =
				<MigrationState<T>>::get(migration_name_as_bytes).unwrap_or(false);

			if ! migration_done {
				<Pallet<T>>::deposit_event(Event::MigrationStarted(
					migration_name_as_bytes.into(),
				));

				// when we go overweight, leave a warning... there's nothing we can really do about
				// this scenario other than hope that the block is actually accepted.
				let available_for_step = if available_weight > weight {
					available_weight - weight
				} else {
					log::error!(
						"previous migration went overweight;
						ignoring and providing migration {} 0 weight.",
						migration_name,
					);

					0u64.into()
				};

				log::trace!(
					"performing migration {}, avail weight: {}",
					migration_name,
					available_for_step
				);

				let consumed_weight = migration.migrate(available_for_step);
				<Pallet<T>>::deposit_event(Event::MigrationCompleted(
					migration_name_as_bytes.into(),
					consumed_weight,
				));
				<MigrationState<T>>::insert(migration_name_as_bytes, true);

				weight += consumed_weight;
				if weight > available_weight {
					log::error!(
						"Migration {} consumed more weight than it was given! ({} > {})",
						migration_name,
						consumed_weight,
						available_for_step
					);
				}
			}
		}

		<FullyUpgraded<T>>::put(true);
		weight += T::DbWeight::get().writes(1);
		<Pallet<T>>::deposit_event(Event::RuntimeUpgradeCompleted(weight));

		weight
	}
}
