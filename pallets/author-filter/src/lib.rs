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

	use frame_support::log;
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
	pub trait Config:
		frame_system::Config + parachain_staking::Config + cumulus_pallet_parachain_system::Config
	{
		/// The overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Deterministic on-chain pseudo-randomness used to do the filtering
		type RandomnessSource: Randomness<H256, Self::BlockNumber>;
	}

	// This code will be called by the author-inherent pallet to check whether the reported author
	// of this block is eligible at this height. We calculate that result on demand and do not
	// record it instorage (although we do emit a debugging event for now).
	// This implementation relies on the relay parent's block number from the validation data
	// inherent. Therefore this implementation must not be used as a preliminary check (only final)
	// Further, the validation data inherent **must** be included before this check is
	// performed. Concretely the validation data inherent must be included before the author
	// inherent.
	impl<T: Config> author_inherent::CanAuthor<T::AccountId> for Pallet<T> {
		fn can_author(account: &T::AccountId) -> bool {
			let mut staked = <parachain_staking::Pallet<T>>::selected_candidates();

			let num_eligible = EligibleRatio::<T>::get().mul_ceil(staked.len());
			let mut eligible = Vec::with_capacity(num_eligible);

			// Grab the relay parent height as a temporary source of relay-based entropy
			let validation_data = cumulus_pallet_parachain_system::Module::<T>::validation_data()
				.expect("validation data was set in parachain system inherent");
			let relay_height = validation_data.relay_parent_number;

			for i in 0..num_eligible {
				// A context identifier for grabbing the randomness. Consists of three parts
				// - The constant string *b"filter" - to identify this pallet
				// - The index `i` when we're selecting the ith eligible author
				// - The relay parent block number so that the eligible authors at the next height
				//   change. Avoids liveness attacks from colluding minorities of active authors.
				// Third one will not be necessary once we leverage the relay chain's randomness.
				let subject: [u8; 8] = [
					b'f',
					b'i',
					b'l',
					b't',
					b'e',
					b'r',
					i as u8,
					relay_height as u8,
				];
				let randomness = T::RandomnessSource::random(&subject).0;
				// Cast to u32 first so we get the same result on wasm and 64-bit platforms.
				let index = (randomness.to_low_u64_be() as u32) as usize;

				// Move the selected author from the original vector into the eligible vector
				// TODO we could short-circuit this check by returning early when the claimed
				// author is selected. For now I'll leave it like this because:
				// 1. it is easier to understand what our core filtering logic is
				// 2. we currently show the entire filtered set in the debug event
				eligible.push(staked.remove(index % staked.len()));

				// Print some logs for debugging purposes.
				log::trace!(target:"author-filter", "Filtering Authors");
				log::trace!(
					target:"author-filter",
					"The randomness was {:?}",
					randomness
				);
				log::trace!(
					target:"author-filter",
					"NOT Eligible Authors: {:?}",
					&staked
				);
				log::trace!(
					target:"author-filter",
					"Eligible Authors are: {:?}",
					eligible
				);
				log::trace!(
					target:"author-filter",
					"The id I'm checking is: {:?}",
					account
				);
				log::trace!(
					target:"author-filter",
					"Was that author eligible: {}",
					eligible.contains(account)
				);
			}

			// Emit an event for debugging purposes
			// let our_height = frame_system::Module::<T>::block_number();
			// <Pallet<T>>::deposit_event(Event::Filtered(our_height, relay_height, eligible.clone()));

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
		/// Fields are: para block height, relay block height, eligible authors
		Filtered(T::BlockNumber, u32, Vec<T::AccountId>),
	}
}
