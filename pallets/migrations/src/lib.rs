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
use sp_runtime::Perbill;

pub use pallet::*;

/// A Migration that must happen on-chain upon a runtime-upgrade
pub trait Migration {
	/// A human-readable name for this migration. Also used as storage key.
	fn friendly_name(&self) -> &str;

	/// Step through this migration, taking up to `available_weight` of execution time and providing
	/// a status on the progress as well as the consumed weight. This allows a migration to perform
	/// its logic in small batches across as many blocks as needed.
	///
	/// Implementations should perform as much migration work as possible and then leave their
	/// pallet in a valid state from which another 'step' of migration work can be performed. In no
	/// case should a step consume more than `available_weight`.
	///
	/// This should return a perbill indicating the aggregate progress of the migration. If
	/// `Perbill::one()` is returned, the migration is considered complete and no further calls to
	/// `step()` will be made. Any value less than `Perbill::one()` will result in another future
	/// call to `step()`. Indeed, values < 1 are arbitrary, but the intent is to indicate progress
	/// (so they should at least be monotonically increasing).
	fn step(&self, previous_progress: Perbill, available_weight: Weight) -> (Perbill, Weight);
}

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::Filter;
	use frame_system::pallet_prelude::*;
	#[allow(unused_imports)] // TODO: why does it detect this as unused?
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
		/// The base call filter to be used in normal operating mode
		/// (When we aren't in the middle of a migration)
		type NormalCallFilter: Filter<Self::Call>;
		/// The base call filter to be used when we are in the middle of migrations
		/// This should be very restrictive. Probably not allowing anything except possibly
		/// something like sudo or other emergency processes
		type MigrationCallFilter: Filter<Self::Call>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		// e.g. runtime upgrade started, completed, etc.
		RuntimeUpgradeStarted(),
		RuntimeUpgradeCompleted(),
		MigrationStarted(Vec<u8>),
		MigrationProgress(Vec<u8>, Perbill),
		MigrationCompleted(Vec<u8>),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// on_runtime_upgrade is expected to be called exactly once after a runtime upgrade.
		/// We use this as a chance to flag that we are now in upgrade-mode and begin our
		/// migrations.
		///
		/// In the event that a migration is expected to take more than one block, ongoing migration
		/// work could continue from block-to-block in this pallet's on_initialize function.
		fn on_runtime_upgrade() -> Weight {
			log::warn!("Performing on_runtime_upgrade");

			// start by flagging that we are not fully upgraded
			<FullyUpgraded<T>>::put(false);
			Self::deposit_event(Event::RuntimeUpgradeStarted());

			let mut weight: Weight = 0u64.into();

			weight += process_runtime_upgrades::<T>();

			weight.into()
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn is_fully_upgraded)]
	/// True if all required migrations have completed
	type FullyUpgraded<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn migration_state)]
	/// MigrationState tracks the progress of a migration.
	/// Maps name (Vec<u8>) -> migration progress (Perbill)
	type MigrationState<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, Perbill, OptionQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub completed_migrations: Vec<Vec<u8>>,
		pub dummy: PhantomData<T>, // TODO:
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				completed_migrations: vec![],
				dummy: PhantomData,
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for migration_name in &self.completed_migrations {
				<MigrationState<T>>::insert(migration_name, Perbill::one());
			}
		}
	}

	impl<T: Config> Filter<T::Call> for Pallet<T> {
		fn filter(call: &T::Call) -> bool {
			if FullyUpgraded::<T>::get() {
				T::NormalCallFilter::filter(call)
			} else {
				T::MigrationCallFilter::filter(call)
			}
		}
	}

	fn process_runtime_upgrades<T: Config>() -> Weight {
		log::info!("stepping runtime upgrade");

		// TODO: query proper value or make configurable
		let available_weight = 500_000_000_000u64.into();
		let mut weight: Weight = 0u64.into();
		let mut done: bool = true;

		for migration in &T::MigrationsList::get() {
			// let migration_name = migration.friendly_name();
			let migration_name = migration.friendly_name();
			log::trace!("evaluating migration {}", migration_name);

			let migration_state =
				<MigrationState<T>>::get(migration_name.as_bytes()).unwrap_or(Perbill::zero());

			if migration_state < Perbill::one() {
				// TODO: we don't currently have a reliable way to know "started"
				// TODO: multiple calls to as_bytes() or to_vec() may be expensive
				<Pallet<T>>::deposit_event(Event::MigrationStarted(
					migration_name.as_bytes().to_vec(),
				));

				let available_for_step = available_weight - weight;
				log::trace!(
					"stepping migration {}, prev: {:?}, avail weight: {}",
					migration_name,
					migration_state,
					available_for_step
				);

				// perform a step of this migration
				<Pallet<T>>::deposit_event(Event::MigrationStarted(migration_name.into()));
				let (updated_progress, consumed_weight) =
					migration.step(migration_state, available_for_step);

				weight += consumed_weight;
				if weight > available_weight {
					// TODO: the intent here is to complain obnoxiously so that this is caught
					// during development. In production, this should probably be tolerated because
					// failing is catastrophic.
					log::error!(
						"Migration {} consumed more weight than it was given! ({} > {})",
						migration_name,
						consumed_weight,
						available_for_step
					);
				}

				// make note of any unfinished migrations
				if updated_progress < Perbill::one() {
					done = false;
				}

				if migration_state != updated_progress {
					<MigrationState<T>>::insert(migration_name.as_bytes(), updated_progress);
				}
			}
		}

		if done {
			<Pallet<T>>::deposit_event(Event::RuntimeUpgradeCompleted());
			<FullyUpgraded<T>>::put(true);
		}

		weight
	}
}
