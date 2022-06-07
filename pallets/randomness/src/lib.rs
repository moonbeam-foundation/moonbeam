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

//! Randomness pallet

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;

pub use pallet::*;

pub mod instant;
pub mod traits;
pub mod types;
pub use instant::*;
pub use traits::*;
pub use types::*;

// pub mod weights;
// use weights::WeightInfo;
// #[cfg(any(test, feature = "runtime-benchmarks"))]
// mod benchmarks;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[pallet]
pub mod pallet {
	use super::*;
	// use crate::WeightInfo;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::Saturating;
	use sp_std::{convert::TryInto, vec::Vec};

	/// Request identifier, unique per request for randomness
	pub type RequestId = u64;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Currency in which the security deposit will be taken.
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// Get relay chain epoch index to insert into this pallet
		type RelayEpochIndex: GetEpochIndex<u64>;
		/// Get relay chain randomness to insert into this pallet
		type RelayRandomness: GetRelayRandomness<Self::Hash>;
		/// Get per block vrf randomness
		type LocalRandomness: pallet_vrf::GetMaybeRandomness<Self::Hash>;
		#[pallet::constant]
		/// The amount that should be taken as a security deposit when requesting randomness.
		type Deposit: Get<BalanceOf<Self>>;
		#[pallet::constant]
		/// Requests expire and can be purged from storage after this many blocks
		type ExpirationDelay: Get<Self::BlockNumber>;
		// /// Weight information for extrinsics in this pallet.
		// type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		NotYetImplemented,
		RequestCounterOverflowed,
		NotSufficientDeposit,
		CannotRequestPastRandomness,
		RequestDNE,
		RequestCannotYetBeFulfilled,
		OnlyRequesterCanIncreaseFee,
		NewFeeMustBeGreaterThanOldFee,
		RequestHasNotExpired,
		RequestedRandomnessNotCorrectlyUpdated,
		RequestExecutionOOG,
		RandomnessNotAvailable,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Randomness requested and request put in storage
		RandomnessRequested {
			id: RequestId,
			refund_address: T::AccountId,
			contract_address: T::AccountId,
			fee: BalanceOf<T>,
			salt: T::Hash,
			// TODO: split into event based on Request.Info pls, not 1 event for all requests
		},
		RequestFulfilled {
			id: RequestId,
		},
		RequestFeeIncreased {
			id: RequestId,
			new_fee: BalanceOf<T>,
		},
		RequestExpirationExecuted {
			id: RequestId,
		},
	}

	#[pallet::storage]
	#[pallet::getter(fn request)]
	/// Randomness requests not yet fulfilled or purged
	pub type Requests<T: Config> =
		StorageMap<_, Twox64Concat, RequestId, RequestState<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn request_count)]
	/// Number of randomness requests made so far, used to generate the next request's uid
	pub type RequestCount<T: Config> = StorageValue<_, RequestId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn current_epoch_index)]
	/// Most recent epoch index, when it changes => update the epoch randomness
	pub type CurrentEpochIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn current_block_randomness)]
	/// Relay chain current block randomness
	/// Some(randomness) or None if not updated
	/// TODO: replace with ParentBlockRandomness once
	/// https://github.com/paritytech/substrate/pull/11113 is merged
	pub type CurrentBlockRandomness<T: Config> = StorageValue<_, Option<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn last_current_block_randomness)]
	/// Last relay chain current block randomness
	/// Some(randomness) or None if not updated
	/// TODO: replace with LastParentBlockRandomness once
	/// https://github.com/paritytech/substrate/pull/11113 is merged
	pub type LastCurrentBlockRandomness<T: Config> = StorageValue<_, Option<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn one_epoch_ago_randomness)]
	/// Relay chain one epoch ago randomness
	/// Some(randomness) or None if not updated
	pub type OneEpochAgoRandomness<T: Config> = StorageValue<_, Option<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn last_one_epoch_ago_randomness)]
	/// Last relay chain one epoch ago randomness
	/// Some(randomness) or None if not updated
	pub type LastOneEpochAgoRandomness<T: Config> = StorageValue<_, Option<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn two_epochs_ago_randomness)]
	/// Relay chain two epochs ago randomness
	/// Some(randomness) or None if not updated
	pub type TwoEpochsAgoRandomness<T: Config> = StorageValue<_, Option<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn last_two_epochs_ago_randomness)]
	/// Last relay chain two epochs ago randomness
	/// Some(randomness) or None if not updated
	pub type LastTwoEpochsAgoRandomness<T: Config> = StorageValue<_, Option<T::Hash>, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// Get randomness from runtime and set it locally
		// TODO (once asynchronous backing is implemented):
		// only get new randomness iff relay block number changes
		fn on_initialize(_now: BlockNumberFor<T>) -> Weight {
			let last_epoch_index = <CurrentEpochIndex<T>>::get();
			let (maybe_new_epoch_index, get_epoch_index_wt) = T::RelayEpochIndex::get_epoch_index();
			let epoch_changed = maybe_new_epoch_index > last_epoch_index;
			let mut weight_consumed: Weight =
				T::DbWeight::get().read + T::DbWeight::get().write + get_epoch_index_wt;
			if epoch_changed {
				// insert new epoch information
				<CurrentEpochIndex<T>>::put(maybe_new_epoch_index);
				// update epoch randomness values
				let (last_one_epoch_ago_randomness, last_two_epochs_ago_randomness) = (
					<OneEpochAgoRandomness<T>>::get(),
					<TwoEpochsAgoRandomness<T>>::get(),
				);
				<LastOneEpochAgoRandomness<T>>::put(last_one_epoch_ago_randomness);
				<LastTwoEpochsAgoRandomness<T>>::put(last_two_epochs_ago_randomness);
				let (one_epoch_ago_randomness, one_epoch_ago_randomness_wt) =
					T::RelayRandomness::get_one_epoch_ago_randomness();
				let (two_epochs_ago_randomness, two_epochs_ago_randomness_wt) =
					T::RelayRandomness::get_two_epochs_ago_randomness();
				<OneEpochAgoRandomness<T>>::put(one_epoch_ago_randomness);
				<TwoEpochsAgoRandomness<T>>::put(two_epochs_ago_randomness);
				weight_consumed += 2 * T::DbWeight::get().read
					+ 5 * T::DbWeight::get().write
					+ one_epoch_ago_randomness_wt
					+ two_epochs_ago_randomness_wt;
			}
			let last_current_block_randomness = <CurrentBlockRandomness<T>>::get();
			<LastCurrentBlockRandomness<T>>::put(last_current_block_randomness);
			let (current_block_randomness, current_block_randomness_wt) =
				T::RelayRandomness::get_current_block_randomness();
			<CurrentBlockRandomness<T>>::put(current_block_randomness);
			weight_consumed
				+ T::DbWeight::get().read
				+ T::DbWeight::get().write
				+ current_block_randomness_wt
		}
	}

	// Utility functions
	impl<T: Config> Pallet<T> {
		pub(crate) fn concat_and_hash(a: T::Hash, b: T::Hash) -> [u8; 32] {
			let mut s = Vec::new();
			s.extend_from_slice(a.as_ref());
			s.extend_from_slice(b.as_ref());
			sp_io::hashing::blake2_256(&s)
		}
	}

	// Public functions for precompile usage only
	impl<T: Config> Pallet<T> {
		pub fn request_randomness(request: Request<T>) -> DispatchResult {
			ensure!(
				!request.can_be_fulfilled(),
				Error::<T>::CannotRequestPastRandomness
			);
			let deposit = T::Deposit::get().saturating_add(request.fee);
			T::Currency::can_reserve(&request.contract_address, deposit)
				.then(|| true)
				.ok_or(Error::<T>::NotSufficientDeposit)?;
			// get new request ID
			let request_id = <RequestCount<T>>::get();
			let next_id = request_id
				.checked_add(1u64)
				.ok_or(Error::<T>::RequestCounterOverflowed)?;
			T::Currency::reserve(&request.contract_address, deposit)?;
			// insert request
			<RequestCount<T>>::put(next_id);
			Self::deposit_event(Event::RandomnessRequested {
				id: request_id,
				refund_address: request.refund_address.clone(),
				contract_address: request.contract_address.clone(),
				fee: request.fee,
				salt: request.salt,
				//info: request.request.info,
			});
			<Requests<T>>::insert(request_id, RequestState::new(request, deposit));
			Ok(())
		}
		/// Prepare fulfillment
		/// Returns all arguments needed for fulfillment
		pub fn prepare_fulfillment(id: RequestId) -> Result<FulfillArgs<T>, DispatchError> {
			<Requests<T>>::get(id)
				.ok_or(Error::<T>::RequestDNE)?
				.prepare_fulfill()
		}
		/// Finish fulfillment
		/// Caller MUST ensure `id` corresponds to `request` or there will be side effects
		pub fn finish_fulfillment(
			id: RequestId,
			request: Request<T>,
			deposit: BalanceOf<T>,
			caller: &T::AccountId,
			cost_of_execution: BalanceOf<T>,
		) {
			request.finish_fulfill(deposit, caller, cost_of_execution);
			<Requests<T>>::remove(id);
			Self::deposit_event(Event::RequestFulfilled { id });
		}
		/// Increase fee associated with request
		pub fn increase_request_fee(
			caller: &T::AccountId,
			id: RequestId,
			new_fee: BalanceOf<T>,
		) -> DispatchResult {
			let mut request = <Requests<T>>::get(id).ok_or(Error::<T>::RequestDNE)?;
			// fulfill randomness request
			request.increase_fee(caller, new_fee)?;
			<Requests<T>>::insert(id, request);
			Self::deposit_event(Event::RequestFeeIncreased { id, new_fee });
			Ok(())
		}
		/// Execute request expiration
		/// transfers fee to caller && purges request iff it has expired
		/// does NOT try to fulfill the request
		pub fn execute_request_expiration(caller: &T::AccountId, id: RequestId) -> DispatchResult {
			let request = <Requests<T>>::get(id).ok_or(Error::<T>::RequestDNE)?;
			request.execute_expiration(&caller)?;
			<Requests<T>>::remove(id);
			Self::deposit_event(Event::RequestExpirationExecuted { id });
			Ok(())
		}
	}
}
