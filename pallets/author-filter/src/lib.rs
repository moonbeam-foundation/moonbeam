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

	use frame_support::debug;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::Randomness;
	use frame_support::traits::Vec;
	use frame_system::pallet_prelude::*;
	use sp_core::H256;
	use sp_runtime::Percent;

	/// The Author Filter pallet
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config + stake::Config {
		/// The overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Deterministic on-chain pseudo-randomness used to do the filtering
		type RandomnessSource: Randomness<H256>;
	}

	// This code will be called by the author-inherent pallet to check whether the reported author
	// of this block is eligible at this height. We calculate that result on demand and do not
	// record it instorage (although we do emit a debugging event for now).
	impl<T: Config> author_inherent::CanAuthor<T::AccountId> for Pallet<T> {
		fn can_author(account: &T::AccountId) -> bool {
			let mut staked: Vec<T::AccountId> = stake::Module::<T>::validators();

			let num_eligible = EligibleRatio::<T>::get().mul_ceil(staked.len());
			let mut eligible = Vec::with_capacity(num_eligible);

			for i in 0..num_eligible {
				// A context identifier for grabbing the randomness. Consists of two parts
				// - The constant string *b"filter" - to identify this pallet
				// - The index `i` when we're selecting the ith eligible author
				// Currently this has the weakness that the authors are based only on para-block
				// height. This will be aleviated in the future by adding entropy from the relay
				// chain inherent.
				let subject: [u8; 7] = [b'f', b'i', b'l', b't', b'e', b'r', i as u8];
				let randomness = T::RandomnessSource::random(&subject);
				let index = randomness.to_low_u64_be() as usize;

				// Move the selected author from the original vector into the eligible vector
				// TODO we could short-circuit this check by returning early when the claimed
				// author is selected. For now I'll leave it like this because:
				// 1. it is easier to understand what our core filtering logic is
				// 2. we currently show the entire filtered set in the debug event
				eligible.push(staked.remove(index % staked.len()));

				// Print some logs for debugging purposes.
				debug::RuntimeLogger::init();
				debug::info!("Filtering Authors");
				debug::info!("The randomness was {:?}", randomness);
				debug::info!("Eligible Authors are: {:?}", eligible);
				debug::info!("NOT Eligible Authors: {:?}", &staked);
				debug::info!("The id I'm checking is: {:?}", account);
				debug::info!("Was that author eligible: {}", eligible.contains(account));
			}

			// Emit an event for debugging purposes
			// let our_height = frame_system::Module::<T>::block_number();
			// <Pallet<T>>::deposit_event(Event::Filtered(our_height, eligible.clone()));

			eligible.contains(account)
		}
	}

	// No hooks
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Update the eligible ratio. Intended to be called by governance.
		#[pallet::weight(0)]
		pub fn set_eligible(origin: OriginFor<T>, new: Percent) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			EligibleRatio::<T>::put(&new);
			<Pallet<T>>::deposit_event(Event::EligibleUpdated(new));

			Ok(Default::default())
		}
	}

	/// The percentage of active staked authors that will be eligible at each height.
	#[pallet::storage]
	pub type EligibleRatio<T: Config> = StorageValue<_, Percent, ValueQuery, Half<T>>;

	// Default value for the `EligibleRatio` is one half.
	#[pallet::type_value]
	pub fn Half<T: Config>() -> Percent {
		Percent::from_percent(50)
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// The amount of eligible authors for the filter to select has been changed.
		EligibleUpdated(Percent),
		/// The staked authors have been filtered to these eligible authors in this block.
		/// This is a debugging and development event and should be removed eventually.
		/// Fields are: para block height, eligible authors
		Filtered(T::BlockNumber, Vec<T::AccountId>),
	}
}
