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

//! Small pallet responsible determining which accounts are eligible to author at the current
//! block height.
//!
//! Currently this pallet is tightly coupled to our stake pallet, but this design
//! should be generalized in the future.
//!
//! Using a randomness beacon supplied by the `Randomness` trait, this pallet takes the set of
//! currently staked accounts from pallet stake, and filters them down to a pseudorandom subset.
//! The current technique gives no preference to any particular author. In the future, we could
//! disfavor authors who are authoring a disproportionate amount of the time in an attempt to
//! "even the playing field".

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

pub use pallet::*;

#[pallet]
pub mod pallet {

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::traits::Randomness;
	use frame_support::traits::Vec;

	/// The Author Filter pallet
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config + stake::Config {
		/// The overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Deterministic on-chain pseudo-randomness used to do the filtering
		type RandomnessSource: Randomness<u32>;
	}

	impl<T: Config> author_inherent::CanAuthor<T::AccountId> for Pallet<T> {
		fn can_author(account: &T::AccountId) -> bool {
			CurrentEligible::<T>::get().contains(account)
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// At the beginning of each block, we grab the randomness and use it to reduce the author set
		fn on_initialize(_: T::BlockNumber) -> Weight {
			let staked : Vec<T::AccountId> = stake::Module::<T>::validators();

			let randomness = T::RandomnessSource::random(&*b"author_filter");

			//TODO actually do some reducing here.
			let eligible_subset = staked.clone();

			CurrentEligible::<T>::put(&eligible_subset);

			//Emit an event for debugging purposes
			<Pallet<T>>::deposit_event(
				Event::Filtered(randomness, staked, eligible_subset)
			);

			0 //TODO actual weight?
		}
	}

	// No dispatchible calls
	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	/// Storage item that holds the set of authors who are eligible to author at this height.
	#[pallet::storage]
	#[pallet::getter(fn chain_id)]
	pub type CurrentEligible<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// The staked authors have been filtered in this block. Here's some debugging info
		/// randomness, copmlete set, reduced set
		Filtered(u32, Vec<T::AccountId>, Vec<T::AccountId>),
	}

}
