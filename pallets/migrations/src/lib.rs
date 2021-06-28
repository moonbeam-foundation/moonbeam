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

use frame_support::{pallet, weights::Weight};
use sp_runtime::Perbill;
pub mod migrations;

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

/// Our list of migrations. Any ordering considerations can be specified here (?).
const MIGRATIONS: [&dyn Migration; 3] = [
	&migrations::MM_001_AuthorMappingAddDeposit {},
	&migrations::MM_002_StakingFixTotalBalance {},
	&migrations::MM_003_StakingTransitionBoundedSet {},
];

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	#[allow(unused_imports)] // TODO: why does it detect this as unused?
	use sp_std::prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::error]
	pub enum Error<T> {
		// errors in this pallet would be quite bad...
	}

	#[pallet::event]
	pub enum Event<T: Config> {
		// e.g. runtime upgrade started, completed, etc.
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

			let mut weight: Weight = 0u64.into();

			weight += process_runtime_upgrades::<T>();

			// now flag that we are done with our runtime upgrade
			<FullyUpgraded<T>>::put(true);

			weight.into()
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn is_fully_upgraded)]
	/// True if all required migrations have completed
	type FullyUpgraded<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn migration_state)]
	/// MigrationState tracks the progress of a migration. Migrations with progress < 1 
	type MigrationState<T: Config> = StorageMap<
		_,
		Twox64Concat,
		String,
		Perbill,
		OptionQuery, // TODO: what is this...?
	>;

	fn process_runtime_upgrades<T: Config>() -> Weight {
		log::info!("stepping runtime upgrade");

		// TODO: query proper value or make configurable
		let available_weight = 500_000_000_000u64.into();
		let mut weight: Weight = 0u64.into();

		for migration in &MIGRATIONS {

			// let migration_name = migration.friendly_name();
			let migration_name = migration.friendly_name();
			log::trace!("evaluating migration {}", migration_name);

			let migration_state = <MigrationState<T>>::get(migration_name)
				.unwrap_or(Perbill::zero());

			if migration_state < Perbill::one() {

				let available_for_step = available_weight - weight;

				// perform a step of this migration
				let (updated_progress, consumed_weight)
					= migration.step(migration_state, available_weight);

				weight += consumed_weight;
				if weight > available_weight {
					// TODO: the intent here is to complain obnoxiously so that this is caught
					// during development. In production, this should probably be tolerated because
					// failing is catastrophic.
					log::error!("Migration {} consumed more weight than it was given! ({} > {})",
						migration_name, consumed_weight, available_for_step);
				}

				<MigrationState<T>>::insert(migration_name, updated_progress);
			}

		}

		weight
	}
}
