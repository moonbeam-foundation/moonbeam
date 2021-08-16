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

//! A pallet to put your runtime into a restricted maintenance or safe mode. This is useful when
//! performing site maintenance, running data migrations, or protecting the chain during an attack.
//!
//! This introduces one storage read to fetch the base filter for each extrinsic. Hoever, it should
//! be that the state cache eliminates this cost almost entirely. I wonder if that can or should be
//! reflected in the weight calculation.
//!
//! This could be more configureable by letting the runtime developer specify a type (probably an
//! enum) that can be converted into a filter and an origin.

#![allow(non_camel_case_types)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_support::{pallet, weights::Weight};

pub use pallet::*;

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{EnsureOrigin, Filter};
	use frame_system::pallet_prelude::*;

	/// Pallet for migrations
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The base call filter to be used in normal operating mode
		/// (When we aren't in the middle of a migration)
		type NormalCallFilter: Filter<Self::Call>;
		/// The base call filter to be used when we are in the middle of migrations
		/// This should be very restrictive. Probably not allowing anything except possibly
		/// something like sudo or other emergency processes
		type MaintenanceCallFilter: Filter<Self::Call>;
		/// The origin from which the call to enter or exit maintenance mode must come
		/// Take care when choosing your maintenance call filter to ensure that you'll still be
		/// able to return to normal mode. For example, if your MaintenanceOrigin is a council, make
		/// sure that your councilors can still cast votes.
		type MaintenanceOrigin: EnsureOrigin<Self::Origin>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The chain was put into Maintenance Mode
		EnteredMaintenanceMode,
		/// The chain returned to its normal operating state
		NormalOperationResumed,
	}

	#[pallet::storage]
	#[pallet::getter(fn maintenance_mode)]
	/// Whether the site is in maintenance mode
	type MaintenanceMode<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn previous_timestamp)]
	/// The timestamp recorded on the previous block. Used to automatically enter maintenance mode
	/// after a long delay in block production
	type PreviousTimestamp<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: T::BlockNumber) -> Weight {
			// Idea: Detect if too much time has elapsed since the last block, and enter maintenance
			// mode automatically.

			// Idea: It might be nice to configure which pallets' on init and on _finalize hooks get called
			// This would allow to determine whether eg staking elections should still occur and 
			// democracy referenda still mature

			0
		}

		fn on_finalize(_n: T::BlockNumber) {
			// Record this block's timestamp for comparison in the next block
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Place the chain in maintenance mode
		#[pallet::weight(0)]
		pub fn enter_maintenance_mode(
			origin: OriginFor<T>,
		) -> DispatchResultWithPostInfo {
			// Ensure Origin
			T::MaintenanceOrigin::ensure_origin(origin)?;

			// Write to storage
			MaintenanceMode::<T>::put(true);

			// Event
			<Pallet<T>>::deposit_event(Event::EnteredMaintenanceMode);
			
			Ok(().into())
		}

		/// Return the chain to normal operating mode
		#[pallet::weight(0)]
		pub fn resume_normal_operation(
			origin: OriginFor<T>,
		) -> DispatchResultWithPostInfo {
			// Ensure Origin
			T::MaintenanceOrigin::ensure_origin(origin)?;

			// Write to storage
			MaintenanceMode::<T>::put(false);

			// Event
			<Pallet<T>>::deposit_event(Event::NormalOperationResumed);

			
			Ok(().into())
		}
	}

	impl<T: Config> Filter<T::Call> for Pallet<T> {
		fn filter(call: &T::Call) -> bool {
			if MaintenanceMode::<T>::get() {
				T::NormalCallFilter::filter(call)
			} else {
				T::MaintenanceCallFilter::filter(call)
			}
		}
	}
}
