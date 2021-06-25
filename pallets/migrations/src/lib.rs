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

use frame_support::pallet;
pub mod migrations;

/// A Migration that must happen on-chain upon a runtime-upgrade
pub trait Migration {
	// TODO: this would involve some metadata about the migration as well as a means of calling
	// the actual migration function

	// fn friendly_name() -> &str;
}

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	#[allow(unused_imports)] // TODO: why does it detect this as unused?
	use sp_std::prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Our list of migrations. Any ordering considerations can be specified here (?).
	const MIGRATIONS: [&dyn Migration; 3] = [
		&migrations::MM_001_AuthorMappingAddDeposit {},
		&migrations::MM_002_StakingFixTotalBalance {},
		&migrations::MM_003_StakingTransitionBoundedSet {},
	];

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
			// start by flagging that we are not fully upgraded
			<FullyUpgraded<T>>::put(false);

			let mut weight: Weight = 0u64.into();

			weight += process_runtime_upgrades();

			// now flag that we are done with our runtime upgrade
			<FullyUpgraded<T>>::put(true);

			weight.into()
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn is_fully_upgraded)]
	/// True if all required migrations have completed
	type FullyUpgraded<T: Config> = StorageValue<_, bool, ValueQuery>;

	fn process_runtime_upgrades() -> Weight {
		// TODO: iterate over MIGRATIONS here and ensure that each one has been fully applied.
		// additionally, write to storage about our progress if multi-block-update functionality
		// is required.
		0u64.into()
	}
}
