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
pub mod types;
pub use instant::*;
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
	use pallet_evm::AddressMapping;
	use session_keys_primitives::{
		InherentError, MaybeGetRandomness, SetRelayRandomness, SetVrfInputs, INHERENT_IDENTIFIER,
	};
	use sp_core::{H160, H256};
	use sp_runtime::traits::Saturating;
	use sp_std::{convert::TryInto, vec::Vec};

	/// Request identifier, unique per request for randomness
	pub type RequestId = u64;

	pub type BalanceOf<T> = <<T as Config>::ReserveCurrency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Address mapping to convert from H160 to AccountId
		type AddressMapping: AddressMapping<Self::AccountId>;
		/// Currency in which the security deposit will be taken.
		type ReserveCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// Set BABE randomness values in `set_relay_data` inherent from the runtime
		type RelayRandomnessSetter: SetRelayRandomness;
		/// Set vrf inputs in `set_relay_data` inherent from the runtime
		type VrfInputSetter: SetVrfInputs;
		/// Get per block vrf randomness
		type LocalRandomness: MaybeGetRandomness<Self::Hash>;
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
		RequestCounterOverflowed,
		InsufficientDeposit,
		CannotRequestPastRandomness,
		RequestDNE,
		RequestCannotYetBeFulfilled,
		OnlyRequesterCanIncreaseFee,
		NewFeeMustBeGreaterThanOldFee,
		RequestHasNotExpired,
		RequestExecutionOOG,
		RandomnessNotAvailable,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		RandomnessRequestedCurrentBlock {
			id: RequestId,
			refund_address: H160,
			contract_address: H160,
			fee: BalanceOf<T>,
			gas_limit: u64,
			salt: H256,
			earliest_block: T::BlockNumber,
		},
		RandomnessRequestedBabeOneEpochAgo {
			id: RequestId,
			refund_address: H160,
			contract_address: H160,
			fee: BalanceOf<T>,
			gas_limit: u64,
			salt: H256,
			earliest_epoch: u64,
		},
		RandomnessRequestedBabeTwoEpochsAgo {
			id: RequestId,
			refund_address: H160,
			contract_address: H160,
			fee: BalanceOf<T>,
			gas_limit: u64,
			salt: H256,
			earliest_epoch: u64,
		},
		RandomnessRequestedLocal {
			id: RequestId,
			refund_address: H160,
			contract_address: H160,
			fee: BalanceOf<T>,
			gas_limit: u64,
			salt: H256,
			earliest_block: T::BlockNumber,
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
	#[pallet::getter(fn requests)]
	/// Randomness requests not yet fulfilled or purged
	pub type Requests<T: Config> = StorageMap<_, Twox64Concat, RequestId, RequestState<T>>;

	#[pallet::storage]
	#[pallet::getter(fn request_count)]
	/// Number of randomness requests made so far, used to generate the next request's uid
	pub type RequestCount<T: Config> = StorageValue<_, RequestId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn current_epoch_index)]
	/// Most recent epoch index, when it changes => update the epoch randomness
	pub type CurrentEpochIndex<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn current_relay_block_number)]
	/// Most recent relay block number
	pub type CurrentRelayBlockNumber<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn current_block_randomness)]
	/// Relay chain current block randomness
	/// Some(randomness) or None if not updated
	/// TODO: replace with ParentBlockRandomness once
	/// https://github.com/paritytech/substrate/pull/11113 is merged
	pub type CurrentBlockRandomness<T: Config> = StorageValue<_, Option<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn one_epoch_ago_randomness)]
	/// Relay chain one epoch ago randomness
	/// Some(randomness) or None if not updated
	pub type OneEpochAgoRandomness<T: Config> = StorageValue<_, Option<T::Hash>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn two_epochs_ago_randomness)]
	/// Relay chain two epochs ago randomness
	/// Some(randomness) or None if not updated
	pub type TwoEpochsAgoRandomness<T: Config> = StorageValue<_, Option<T::Hash>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This inherent is a workaround to run code after the "real" inherents have executed,
		/// but before on_initialize, where ParachainSystem kills data required to form the
		/// RelayChainStateProof (which must be read to get BABE randomness for example).
		// This should go into on_post_inherents when it is ready
		// https://github.com/paritytech/substrate/pull/10128
		// Weight is 10_000 margin of safety + number of reads/writes
		#[pallet::weight((
			10_000 + 7 * T::DbWeight::get().write + T::DbWeight::get().read,
			DispatchClass::Mandatory
		))]
		pub fn set_relay_data(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_none(origin)?;

			// Expected to call `Self::set_relay_randomness` with inputs read from the runtime
			// Therefore expect 5 writes
			T::RelayRandomnessSetter::set_relay_randomness();
			// Expected to call `pallet_vrf::set_vrf_inputs` with inputs read from the runtime
			// Therefore expect 2 writes + 1 read
			T::VrfInputSetter::set_vrf_inputs();

			Ok(Pays::No.into())
		}
	}

	#[pallet::inherent]
	impl<T: Config> ProvideInherent for Pallet<T> {
		type Call = Call<T>;
		type Error = InherentError;
		const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

		fn is_inherent_required(_: &InherentData) -> Result<Option<Self::Error>, Self::Error> {
			// Return Ok(Some(_)) unconditionally because this inherent is required in every block
			// If it is not found, throw a VrfInherentRequired error.
			Ok(Some(InherentError::Other(
				sp_runtime::RuntimeString::Borrowed(
					"Inherent required to set relay data for randomness",
				),
			)))
		}

		// The empty-payload inherent extrinsic.
		fn create_inherent(_data: &InherentData) -> Option<Self::Call> {
			Some(Call::set_relay_data {})
		}

		fn is_inherent(call: &Self::Call) -> bool {
			matches!(call, Call::set_relay_data { .. })
		}
	}

	// Utility functions
	impl<T: Config> Pallet<T> {
		pub(crate) fn concat_and_hash(a: T::Hash, b: H256) -> [u8; 32] {
			let mut s = Vec::new();
			s.extend_from_slice(a.as_ref());
			s.extend_from_slice(b.as_ref());
			sp_io::hashing::blake2_256(&s)
		}
		/// For the runtime to set the randomness storage values in this pallet
		pub fn set_relay_randomness(
			block_number: T::BlockNumber,
			epoch_index: Option<u64>,
			current_block_randomness: Option<T::Hash>,
			one_epoch_ago_randomness: Option<T::Hash>,
			two_epochs_ago_randomness: Option<T::Hash>,
		) {
			<CurrentRelayBlockNumber<T>>::put(block_number);
			if let Some(epoch) = epoch_index {
				<CurrentEpochIndex<T>>::put(epoch);
			} else {
				log::warn!("Error reading epoch index from relay chain state proof");
			}
			if current_block_randomness.is_none() {
				log::warn!("Error reading current block randomness from relay chain state proof");
			}
			if one_epoch_ago_randomness.is_none() {
				log::warn!("Error reading one epoch ago randomness from relay chain state proof");
			}
			if two_epochs_ago_randomness.is_none() {
				log::warn!("Error reading two epochs ago randomness from relay chain state proof");
			}
			<CurrentBlockRandomness<T>>::put(current_block_randomness);
			<OneEpochAgoRandomness<T>>::put(one_epoch_ago_randomness);
			<TwoEpochsAgoRandomness<T>>::put(two_epochs_ago_randomness);
		}
	}

	// Public functions for precompile usage only
	impl<T: Config> Pallet<T> {
		pub fn request_randomness(request: Request<T>) -> DispatchResult {
			ensure!(
				!request.can_be_fulfilled(),
				Error::<T>::CannotRequestPastRandomness
			);
			let total_to_reserve = T::Deposit::get().saturating_add(request.fee);
			let contract_address =
				T::AddressMapping::into_account_id(request.contract_address.clone());
			T::ReserveCurrency::can_reserve(&contract_address, total_to_reserve)
				.then(|| true)
				.ok_or(Error::<T>::InsufficientDeposit)?;
			// get new request ID
			let request_id = <RequestCount<T>>::get();
			let next_id = request_id
				.checked_add(1u64)
				.ok_or(Error::<T>::RequestCounterOverflowed)?;
			T::ReserveCurrency::reserve(&contract_address, total_to_reserve)?;
			// insert request
			<RequestCount<T>>::put(next_id);
			request.emit_randomness_requested_event(request_id);
			<Requests<T>>::insert(request_id, RequestState::new(request));
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
			caller: &H160,
			cost_of_execution: BalanceOf<T>,
		) {
			request.finish_fulfill(deposit, caller, cost_of_execution);
			<Requests<T>>::remove(id);
			Self::deposit_event(Event::RequestFulfilled { id });
		}
		/// Increase fee associated with request
		pub fn increase_request_fee(
			caller: &H160,
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
		pub fn execute_request_expiration(caller: &H160, id: RequestId) -> DispatchResult {
			let request = <Requests<T>>::get(id).ok_or(Error::<T>::RequestDNE)?;
			let caller = T::AddressMapping::into_account_id(caller.clone());
			request.execute_expiration(&caller)?;
			<Requests<T>>::remove(id);
			Self::deposit_event(Event::RequestExpirationExecuted { id });
			Ok(())
		}
	}
}
