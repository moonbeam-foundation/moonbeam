// Copyright 2019-2022 PureStake Inc.
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

use sp_std::prelude::*;

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

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		Ok(())
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		Ok(())
	}
}

// The migration trait
pub trait GetMigrations {
	// Migration list Getter
	fn get_migrations() -> Vec<Box<dyn Migration>>;
}

#[impl_trait_for_tuples::impl_for_tuples(30)]
impl GetMigrations for Tuple {
	fn get_migrations() -> Vec<Box<dyn Migration>> {
		let mut migrations = Vec::new();

		for_tuples!( #( migrations.extend(Tuple::get_migrations()); )* );

		migrations
	}
}

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Pallet for migrations
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The list of migrations that will be performed
		type MigrationsList: GetMigrations;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Runtime upgrade started
		RuntimeUpgradeStarted(),
		/// Runtime upgrade completed
		RuntimeUpgradeCompleted { weight: Weight },
		/// Migration started
		MigrationStarted { migration_name: Vec<u8> },
		/// Migration completed
		MigrationCompleted {
			migration_name: Vec<u8>,
			consumed_weight: Weight,
		},
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// on_runtime_upgrade is expected to be called exactly once after a runtime upgrade.
		/// We use this as a chance to flag that we are now in upgrade-mode and begin our
		/// migrations.
		fn on_runtime_upgrade() -> Weight {
			log::warn!("Performing on_runtime_upgrade");

			let mut weight = Weight::zero();
			// TODO: derive a suitable value here, which is probably something < max_block
			let available_weight: Weight = T::BlockWeights::get().max_block;

			// start by flagging that we are not fully upgraded
			<FullyUpgraded<T>>::put(false);
			weight = weight.saturating_add(T::DbWeight::get().writes(1));
			Self::deposit_event(Event::RuntimeUpgradeStarted());

			weight = weight.saturating_add(perform_runtime_upgrades::<T>(
				available_weight.saturating_sub(weight),
			));

			if !<FullyUpgraded<T>>::get() {
				log::error!(
					"migrations weren't completed in on_runtime_upgrade(), but we're not
				configured for multi-block migrations; state is potentially inconsistent!"
				);
			}

			weight
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			use frame_support::traits::OnRuntimeUpgradeHelpersExt;

			let mut failed = false;
			for migration in &T::MigrationsList::get_migrations() {
				let migration_name = migration.friendly_name();
				let migration_name_as_bytes = migration_name.as_bytes();

				let migration_done = <MigrationState<T>>::get(migration_name_as_bytes);
				if migration_done {
					continue;
				}
				log::debug!(
					target: "pallet-migrations",
					"invoking pre_upgrade() on migration {}", migration_name
				);

				// dump the migration name to temp storage so post_upgrade will know which
				// migrations were performed (as opposed to skipped)
				Self::set_temp_storage(true, migration_name);

				match migration.pre_upgrade() {
					Ok(()) => {
						log::info!("migration {} pre_upgrade() => Ok()", migration_name);
					}
					Err(msg) => {
						log::error!("migration {} pre_upgrade() => Err({})", migration_name, msg);
						failed = true;
					}
				}
			}

			if failed {
				Err("One or more pre_upgrade tests failed; see output above.")
			} else {
				Ok(())
			}
		}

		/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			use frame_support::traits::OnRuntimeUpgradeHelpersExt;

			// TODO: my desire to DRY all the things feels like this code is very repetitive...

			let mut failed = false;
			for migration in &T::MigrationsList::get_migrations() {
				let migration_name = migration.friendly_name();

				// we can't query MigrationState because on_runtime_upgrade() would have
				// unconditionally set it to true, so we read a hint from temp storage which was
				// left for us by pre_upgrade()
				match Self::get_temp_storage::<bool>(migration_name) {
					Some(value) => assert!(true == value, "our dummy value might as well be true"),
					None => continue,
				}

				log::debug!(
					target: "pallet-migrations",
					"invoking post_upgrade() on migration {}", migration_name
				);

				let result = migration.post_upgrade();
				match result {
					Ok(()) => {
						log::info!("migration {} post_upgrade() => Ok()", migration_name);
					}
					Err(msg) => {
						log::error!(
							"migration {} post_upgrade() => Err({})",
							migration_name,
							msg
						);
						failed = true;
					}
				}
			}

			if failed {
				Err("One or more post_upgrade tests failed; see output above.")
			} else {
				Ok(())
			}
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
	pub(crate) type MigrationState<T: Config> =
		StorageMap<_, Twox64Concat, Vec<u8>, bool, ValueQuery>;

	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig;

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			// When building a new genesis, all listed migrations should be considered as already
			// applied, they only make sense for networks that had been launched in the past.
			for migration_name in T::MigrationsList::get_migrations()
				.into_iter()
				.map(|migration| migration.friendly_name().as_bytes().to_vec())
			{
				<MigrationState<T>>::insert(migration_name, true);
			}
		}
	}

	fn perform_runtime_upgrades<T: Config>(available_weight: Weight) -> Weight {
		let mut weight = Weight::zero();

		for migration in &T::MigrationsList::get_migrations() {
			let migration_name = migration.friendly_name();
			let migration_name_as_bytes = migration_name.as_bytes();
			log::debug!( target: "pallet-migrations", "evaluating migration {}", migration_name);

			let migration_done = <MigrationState<T>>::get(migration_name_as_bytes);

			if !migration_done {
				<Pallet<T>>::deposit_event(Event::MigrationStarted {
					migration_name: migration_name_as_bytes.into(),
				});

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

					Weight::zero()
				};

				log::info!( target: "pallet-migrations",
					"performing migration {}, available weight: {}",
					migration_name,
					available_for_step
				);

				let consumed_weight = migration.migrate(available_for_step);
				<Pallet<T>>::deposit_event(Event::MigrationCompleted {
					migration_name: migration_name_as_bytes.into(),
					consumed_weight: consumed_weight,
				});
				<MigrationState<T>>::insert(migration_name_as_bytes, true);

				weight = weight.saturating_add(consumed_weight);
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
		weight = weight.saturating_add(T::DbWeight::get().writes(1));
		<Pallet<T>>::deposit_event(Event::RuntimeUpgradeCompleted { weight });

		weight
	}
}
