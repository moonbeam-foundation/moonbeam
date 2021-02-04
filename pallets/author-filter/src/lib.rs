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

	impl<T: Config> author_inherent::CanAuthor<T::AccountId> for Pallet<T> {
		fn can_author(account: &T::AccountId) -> bool {
			CurrentEligible::<T>::get().contains(account)
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// At the beginning of each block, we calculate the set of eligible authors for this block.
		// TODO Design Decision:
		// If we move this logic to on_finalize to calculate for the next block, we get to know in
		// advance who the next eligible authors are. That is nice because it is easy to know in
		// from offchain who will author next. You just need to read storage.
		// On the other hand, it leads to liveness attacks. If the eligible authors collude to not
		// author, then the chain is bricked. We can 't even force them out with governance because
		// governance stops when the chain is stalled. In that way, the `EligibleRatio` _is_ our
		// security assumption. By leaving this in on_initialize, we can rely on Polkadot's
		// randomness beacon having a different value when there is a different relay parent.
		fn on_initialize(_: T::BlockNumber) -> Weight {
			let mut staked: Vec<T::AccountId> = stake::Module::<T>::validators();

			let num_eligible = EligibleRatio::<T>::get() * staked.len();
			let mut eligible = Vec::with_capacity(num_eligible);

			for i in 0..num_eligible {
				// A context identifier for grabbing the randomness.
				// Of the form *b"filter4"
				let subject: [u8; 7] = [b'f', b'i', b'l', b't', b'e', b'r', i as u8];
				let index = T::RandomnessSource::random(&subject).to_low_u64_be() as usize;

				// Move the selected author from the original vector into the eligible vector
				eligible.push(staked.remove(index % staked.len()));
			}

			CurrentEligible::<T>::put(&eligible);

			// Emit an event for debugging purposes
			<Pallet<T>>::deposit_event(Event::Filtered(eligible));

			0 //TODO actual weight?
		}
	}

	// No dispatchible calls
	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	/// The set of authors who are eligible to author at this height.
	#[pallet::storage]
	#[pallet::getter(fn current_eligible)]
	pub type CurrentEligible<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

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
		/// The staked authors have been filtered to these eligible authors in this block
		Filtered(Vec<T::AccountId>),
	}
}
