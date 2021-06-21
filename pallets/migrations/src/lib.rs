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

//! # Migrations

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use sp_std::prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
	/// A Migration that must happen on-chain upon a runtime-upgrade
	pub struct Migration {
		// TODO: this would involve some metadata about the migration as well as a means of calling
		// the actual migration function
	}

	/// Our list of migrations. Any ordering considerations can be specified here (?).
	// const MIGRATIONS: // TODO: this would be a constant vec (or similar) of individual migrations

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		// errors in this pallet would be quite bad...
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
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

			let weight: u32 = 0;

			let info = process_runtime_upgrades();
			weight += info.actual_weight.expect("Weight not provided");

			// now flag that we are done with our runtime upgrade
			<FullyUpgraded<T>>::put(true);

			weight
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn is_fully_upgraded)]
	/// True if all required migrations have completed
	type FullyUpgraded<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		// TODO: this isn't a call, but it should forward weight info
		#[pallet::weight(0)]
		pub fn process_runtime_upgrades(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			// TODO: iterate over MIGRATIONS here and ensure that each one has been fully applied.
			// additionally, write to storage about our progress if multi-block-update functionality
			// is required.
		}
	}
}
