// Copyright 2019-2021 PureStake Inc.
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

//! A pallet to put your runtime into a restricted maintenance or safe mode. This is useful when
//! performing site maintenance, running data migrations, or protecting the chain during an attack.
//!
//! This introduces one storage read to fetch the base filter for each extrinsic. Hoever, it should
//! be that the state cache eliminates this cost almost entirely. I wonder if that can or should be
//! reflected in the weight calculation.
//!
//! Possible future improvements
//! 1. This could be more configureable by letting the runtime developer specify a type (probably an
//! enum) that can be converted into a filter. Similar end result (but different implementation) as
//! Acala has it
//! github.com/AcalaNetwork/Acala/blob/pause-transaction/modules/transaction-pause/src/lib.rs#L71
//!
//! 2. Automatically enable maintenance mode after a long timeout is detected between blocks.
//! To implement this we would couple to the timestamp pallet and store the timestamp of the
//! previous block.
//!
//! 3. Different origins for entering and leaving maintenance mode.
//!
//! 4. Maintenance mode timeout. To avoid getting stuck in maintenance mode. It could automatically
//! switch back to normal mode after a pre-decided number of blocks. Maybe there could be an
//! extrinsic to extend the maintenance time.

#![allow(non_camel_case_types)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod types;
pub use types::*;

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {
	use cumulus_primitives_core::{
		relay_chain::BlockNumber as RelayBlockNumber, DmpMessageHandler, ParaId, XcmpMessageHandler,
	};
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{
		Contains, EnsureOrigin, OffchainWorker, OnFinalize, OnIdle, OnInitialize, OnRuntimeUpgrade,
	};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	/// Pallet for migrations
	#[pallet::pallet]
	#[pallet::generate_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;
		/// The base call filter to be used in normal operating mode
		/// (When we aren't in the middle of a migration)
		type NormalCallFilter: Contains<Self::Call>;
		/// The base call filter to be used when we are in the middle of migrations
		/// This should be very restrictive. Probably not allowing anything except possibly
		/// something like sudo or other emergency processes
		type MaintenanceCallFilter: Contains<Self::Call>;
		/// The origin from which the call to enter or exit maintenance mode must come
		/// Take care when choosing your maintenance call filter to ensure that you'll still be
		/// able to return to normal mode. For example, if your MaintenanceOrigin is a council, make
		/// sure that your councilors can still cast votes.
		type MaintenanceOrigin: EnsureOrigin<Self::Origin>;
		/// The DMP handler to be used in normal operating mode
		type NormalDmpHandler: DmpMessageHandler;
		/// The DMP handler to be used in maintenance mode
		type MaintenanceDmpHandler: DmpMessageHandler;
		/// The XCMP handler to be used in normal operating mode
		type NormalXcmpHandler: XcmpMessageHandler;
		/// The XCMP handler to be used in maintenance mode
		type MaintenanceXcmpHandler: XcmpMessageHandler;
		/// The OnIdle hooks that should be called on normal operating mode
		type NormalOnIdle: OnIdle<Self::BlockNumber>;
		/// The OnIdle hooks that should be called on maintenance mode
		type MaintenanceOnIdle: OnIdle<Self::BlockNumber>;
		/// The OnInitialize hooks that should be called on normal operating mode
		type NormalOnInitialize: OnInitialize<Self::BlockNumber>;
		/// The OnInitialize hooks that should be called on maintenance mode
		type MaintenanceOnInitialize: OnInitialize<Self::BlockNumber>;
		/// The OnFinalize hooks that should be called on normal operating mode
		type NormalOnFinalize: OnFinalize<Self::BlockNumber>;
		/// The OnFinalize hooks that should be called on maintenance mode
		type MaintenanceOnFinalize: OnFinalize<Self::BlockNumber>;
		/// The OffchainWorker hooks that should be called on normal operating mode
		type NormalOffchainWorker: OffchainWorker<Self::BlockNumber>;
		/// The OffchainWorker hooks that should be called on maintenance mode
		type MaintenanceOffchainWorker: OffchainWorker<Self::BlockNumber>;
		/// The OnRuntimeUpgrade hooks that should be called on normal operating mode
		type NormalOnRuntimeUpgrade: OnRuntimeUpgrade;
		/// The OnRuntimeUpgrade hook that should be called on maintenance mode
		type MaintenanceOnRuntimeUpgrade: OnRuntimeUpgrade;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event {
		/// The chain was put into Maintenance Mode
		EnteredMaintenanceMode,
		/// The chain returned to its normal operating state
		NormalOperationResumed,
	}

	/// An error that can occur while executing this pallet's extrinsics.
	#[pallet::error]
	pub enum Error<T> {
		/// The chain cannot enter maintenance mode because it is already in maintenance mode
		AlreadyInMaintenanceMode,
		/// The chain cannot resume normal operation because it is not in maintenance mode
		NotInMaintenanceMode,
	}

	#[pallet::storage]
	#[pallet::getter(fn maintenance_mode)]
	/// Whether the site is in maintenance mode
	type MaintenanceMode<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Place the chain in maintenance mode
		///
		/// Weight cost is:
		/// * One DB read to ensure we're not already in maintenance mode
		/// * Two DB writes - 1 for the mode and 1 for the event
		#[pallet::weight(T::DbWeight::get().read + 2 * T::DbWeight::get().write)]
		pub fn enter_maintenance_mode(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			// Ensure Origin
			T::MaintenanceOrigin::ensure_origin(origin)?;

			// Ensure we're not aleady in maintenance mode.
			// This test is not strictly necessary, but seeing the error may help a confused chain
			// operator during an emergency
			ensure!(
				!MaintenanceMode::<T>::get(),
				Error::<T>::AlreadyInMaintenanceMode
			);

			// Write to storage
			MaintenanceMode::<T>::put(true);

			// Event
			<Pallet<T>>::deposit_event(Event::EnteredMaintenanceMode);

			Ok(().into())
		}

		/// Return the chain to normal operating mode
		///
		/// Weight cost is:
		/// * One DB read to ensure we're in maintenance mode
		/// * Two DB writes - 1 for the mode and 1 for the event
		#[pallet::weight(T::DbWeight::get().read + 2 * T::DbWeight::get().write)]
		pub fn resume_normal_operation(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			// Ensure Origin
			T::MaintenanceOrigin::ensure_origin(origin)?;

			// Ensure we're actually in maintenance mode.
			// This test is not strictly necessary, but seeing the error may help a confused chain
			// operator during an emergency
			ensure!(
				MaintenanceMode::<T>::get(),
				Error::<T>::NotInMaintenanceMode
			);

			// Write to storage
			MaintenanceMode::<T>::put(false);

			// Event
			<Pallet<T>>::deposit_event(Event::NormalOperationResumed);

			Ok(().into())
		}
	}

	#[derive(Default)]
	#[pallet::genesis_config]
	/// Genesis config for maintenance mode pallet
	pub struct GenesisConfig {
		/// Whether to launch in maintenance mode
		pub start_in_maintenance_mode: bool,
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			if self.start_in_maintenance_mode {
				MaintenanceMode::<T>::put(true);
			}
		}
	}

	impl<T: Config> Contains<T::Call> for Pallet<T> {
		fn contains(call: &T::Call) -> bool {
			if MaintenanceMode::<T>::get() {
				T::MaintenanceCallFilter::contains(call)
			} else {
				T::NormalCallFilter::contains(call)
			}
		}
	}
	impl<T: Config> DmpMessageHandler for Pallet<T> {
		fn handle_dmp_messages(
			iter: impl Iterator<Item = (RelayBlockNumber, Vec<u8>)>,
			limit: Weight,
		) -> Weight {
			if MaintenanceMode::<T>::get() {
				T::MaintenanceDmpHandler::handle_dmp_messages(iter, limit)
			} else {
				T::NormalDmpHandler::handle_dmp_messages(iter, limit)
			}
		}
	}

	impl<T: Config> XcmpMessageHandler for Pallet<T> {
		fn handle_xcmp_messages<'a, I: Iterator<Item = (ParaId, RelayBlockNumber, &'a [u8])>>(
			iter: I,
			limit: Weight,
		) -> Weight {
			if MaintenanceMode::<T>::get() {
				T::MaintenanceXcmpHandler::handle_xcmp_messages(iter, limit)
			} else {
				T::NormalXcmpHandler::handle_xcmp_messages(iter, limit)
			}
		}
	}
}
