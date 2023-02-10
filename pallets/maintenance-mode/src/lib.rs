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

//! A pallet to put your runtime into a restricted maintenance or safe mode. This is useful when
//! performing site maintenance, running data migrations, or protecting the chain during an attack.
//!
//! This introduces one storage read to fetch the base filter for each extrinsic. However, it should
//! be that the state cache eliminates this cost almost entirely. I wonder if that can or should be
//! reflected in the weight calculation.
//!
//! Possible future improvements
//! 1. This could be more configurable by letting the runtime developer specify a type (probably an
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
	#[cfg(feature = "xcm-support")]
	use cumulus_primitives_core::{
		relay_chain::BlockNumber as RelayBlockNumber, DmpMessageHandler,
	};
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{
		Contains, EnsureOrigin, OffchainWorker, OnFinalize, OnIdle, OnInitialize, OnRuntimeUpgrade,
	};
	use frame_system::pallet_prelude::*;
	#[cfg(feature = "xcm-support")]
	use sp_std::vec::Vec;
	#[cfg(feature = "xcm-support")]
	use xcm_primitives::PauseXcmExecution;

	/// Pallet for migrations
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type RuntimeEvent: From<Event> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The base call filter to be used in normal operating mode
		/// (When we aren't in the middle of a migration)
		type NormalCallFilter: Contains<Self::RuntimeCall>;
		/// The base call filter to be used when we are in the middle of migrations
		/// This should be very restrictive. Probably not allowing anything except possibly
		/// something like sudo or other emergency processes
		type MaintenanceCallFilter: Contains<Self::RuntimeCall>;
		/// The origin from which the call to enter or exit maintenance mode must come
		/// Take care when choosing your maintenance call filter to ensure that you'll still be
		/// able to return to normal mode. For example, if your MaintenanceOrigin is a council, make
		/// sure that your councilors can still cast votes.
		type MaintenanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Handler to suspend and resume XCM execution
		#[cfg(feature = "xcm-support")]
		type XcmExecutionManager: xcm_primitives::PauseXcmExecution;
		/// The DMP handler to be used in normal operating mode
		/// TODO: remove once https://github.com/paritytech/polkadot/pull/5035 is merged
		#[cfg(feature = "xcm-support")]
		type NormalDmpHandler: DmpMessageHandler;
		/// The DMP handler to be used in maintenance mode
		/// TODO: remove once https://github.com/paritytech/polkadot/pull/5035 is merged
		#[cfg(feature = "xcm-support")]
		type MaintenanceDmpHandler: DmpMessageHandler;
		/// The executive hooks that will be used in normal operating mode
		/// Important: Use AllPalletsWithSystem here if you dont want to modify the
		/// hooks behaviour
		type NormalExecutiveHooks: OnRuntimeUpgrade
			+ OnInitialize<Self::BlockNumber>
			+ OnIdle<Self::BlockNumber>
			+ OnFinalize<Self::BlockNumber>
			+ OffchainWorker<Self::BlockNumber>;
		/// The executive hooks that will be used in maintenance mode
		/// Important: Use AllPalletsWithSystem here if you dont want to modify the
		/// hooks behaviour
		type MaintenanceExecutiveHooks: OnRuntimeUpgrade
			+ OnInitialize<Self::BlockNumber>
			+ OnIdle<Self::BlockNumber>
			+ OnFinalize<Self::BlockNumber>
			+ OffchainWorker<Self::BlockNumber>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event {
		/// The chain was put into Maintenance Mode
		EnteredMaintenanceMode,
		/// The chain returned to its normal operating state
		NormalOperationResumed,
		/// The call to suspend on_idle XCM execution failed with inner error
		FailedToSuspendIdleXcmExecution { error: DispatchError },
		/// The call to resume on_idle XCM execution failed with inner error
		FailedToResumeIdleXcmExecution { error: DispatchError },
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
		/// * Three DB writes - 1 for the mode, 1 for suspending xcm execution, 1 for the event
		#[pallet::call_index(0)]
		#[pallet::weight(T::DbWeight::get().read + 3 * T::DbWeight::get().write)]
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
			// Suspend XCM execution
			if let Err(error) = T::XcmExecutionManager::suspend_xcm_execution() {
				<Pallet<T>>::deposit_event(Event::FailedToSuspendIdleXcmExecution { error });
			}

			// Event
			<Pallet<T>>::deposit_event(Event::EnteredMaintenanceMode);

			Ok(().into())
		}

		/// Return the chain to normal operating mode
		///
		/// Weight cost is:
		/// * One DB read to ensure we're in maintenance mode
		/// * Three DB writes - 1 for the mode, 1 for resuming xcm execution, 1 for the event
		#[pallet::call_index(1)]
		#[pallet::weight(T::DbWeight::get().read + 3 * T::DbWeight::get().write)]
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
			// Resume XCM execution
			if let Err(error) = T::XcmExecutionManager::resume_xcm_execution() {
				<Pallet<T>>::deposit_event(Event::FailedToResumeIdleXcmExecution { error });
			}

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

	impl<T: Config> Contains<T::RuntimeCall> for Pallet<T> {
		fn contains(call: &T::RuntimeCall) -> bool {
			if MaintenanceMode::<T>::get() {
				T::MaintenanceCallFilter::contains(call)
			} else {
				T::NormalCallFilter::contains(call)
			}
		}
	}
	#[cfg(feature = "xcm-support")]
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
}
