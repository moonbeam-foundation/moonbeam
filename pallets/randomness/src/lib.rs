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

// pub mod weights;
// use weights::WeightInfo;
// #[cfg(any(test, feature = "runtime-benchmarks"))]
// mod benchmarks;
// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

#[pallet]
pub mod pallet {
	// use crate::WeightInfo;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, ExistenceRequirement::KeepAlive, ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{CheckedSub, Saturating};

	#[derive(PartialEq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Randomness storage item from BABE
	pub enum BabeRandomness {
		OneEpochAgo,
		TwoEpochsAgo,
		CurrentBlock,
	}

	#[derive(PartialEq, Copy, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Information regarding the request type
	pub enum RequestType<BlockNumber> {
		/// BABE randomness from reading relay chain state proof
		Babe {
			info: BabeRandomness,
			when: BlockNumber,
		},
		/// Per block VRF randomness
		Local { when: BlockNumber },
	}

	impl<BlockNumber: Copy> RequestType<BlockNumber> {
		pub fn when(&self) -> BlockNumber {
			match self {
				RequestType::Babe { when, .. } => *when,
				RequestType::Local { when } => *when,
			}
		}
	}

	#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Request<T: Config> {
		/// Fee is returned to this account upon execution
		pub refund_address: T::AccountId,
		/// Contract that consumes the randomness
		pub contract_address: T::AccountId,
		/// Fee to pay for execution
		pub fee: BalanceOf<T>,
		/// Salt to use once randomness is ready
		pub salt: T::Hash,
		/// Details regarding request type and when it is due
		pub info: RequestType<T::BlockNumber>,
	}

	impl<T: Config> Request<T> {
		fn can_be_fulfilled(&self) -> bool {
			self.info.when() <= frame_system::Pallet::<T>::block_number()
		}
	}

	#[derive(PartialEq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct RequestState<T: Config> {
		/// Fee is returned to this account upon execution
		pub request: Request<T>,
		/// Deposit taken for making request (stored in case config changes)
		pub deposit: BalanceOf<T>,
		/// All requests expire `T::ExpirationDelay` blocks after they are made
		pub expires: T::BlockNumber,
	}

	impl<T: Config> RequestState<T> {
		fn new(
			request: Request<T>,
			deposit: BalanceOf<T>,
		) -> Result<RequestState<T>, DispatchError> {
			let expires =
				frame_system::Pallet::<T>::block_number().saturating_add(T::ExpirationDelay::get());
			ensure!(
				request.info.when() < expires,
				Error::<T>::CannotBeFulfilledBeforeExpiry
			);
			Ok(RequestState {
				request,
				deposit,
				expires,
			})
		}
		fn fulfill(&self, caller: &T::AccountId) -> DispatchResult {
			ensure!(
				self.request.can_be_fulfilled(),
				Error::<T>::RequestCannotYetBeFulfilled
			);
			let raw_randomness: T::Hash = match self.request.info {
				RequestType::Local { .. } => return Err(Error::<T>::NotYetImplemented.into()),
				RequestType::Babe { info, .. } => {
					Pallet::<T>::get_most_recent_babe_randomness(info)
				}
			};
			let randomness = Pallet::<T>::concat_and_hash(raw_randomness, self.request.salt);
			T::RandomnessSender::send_randomness(self.request.contract_address.clone(), randomness);
			// return deposit + fee_excess to contract_address
			// refund cost_of_execution to caller?
			T::Currency::unreserve(
				&self.request.contract_address,
				self.deposit + self.request.fee,
			);
			T::Currency::transfer(
				&self.request.contract_address,
				caller,
				self.request.fee,
				KeepAlive,
			)
			.expect("just unreserved deposit + fee => fee must be transferrable");
			Ok(())
		}
		fn increase_fee(&mut self, caller: &T::AccountId, new_fee: BalanceOf<T>) -> DispatchResult {
			ensure!(
				caller == &self.request.contract_address,
				Error::<T>::OnlyRequesterCanIncreaseFee
			);
			let to_reserve = new_fee
				.checked_sub(&self.request.fee)
				.ok_or(Error::<T>::NewFeeMustBeGreaterThanOldFee)?;
			T::Currency::reserve(caller, to_reserve)?;
			self.request.fee = new_fee;
			Ok(())
		}
		/// Unreserve deposit + fee from contract_address
		/// Transfer fee to caller
		fn execute_expiration(&self, caller: &T::AccountId) -> DispatchResult {
			ensure!(
				frame_system::Pallet::<T>::block_number() >= self.expires,
				Error::<T>::RequestHasNotExpired
			);
			T::Currency::unreserve(
				&self.request.contract_address,
				self.deposit + self.request.fee,
			);
			T::Currency::transfer(
				&self.request.contract_address,
				caller,
				self.request.fee,
				KeepAlive,
			)
			.expect("just unreserved deposit + fee => fee must be transferrable");
			Ok(())
		}
	}

	/// Send randomness to a smart contract, triggered by this pallet
	pub trait SendRandomness<AccountId, R> {
		fn send_randomness(contract: AccountId, randomness: R);
	}

	/// Get babe randomness to insert into runtime
	pub trait GetRelayRandomness<R> {
		fn get_current_block_randomness() -> (R, Weight);
		fn get_one_epoch_ago_randomness() -> (R, Weight);
		fn get_two_epochs_ago_randomness() -> (R, Weight);
	}

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
		/// Get relay chain randomness to insert into this pallet
		type RelayRandomness: GetRelayRandomness<Self::Hash>;
		/// Send randomness to smart contract
		/// TODO: why can't Randomness = T::Hash?
		type RandomnessSender: SendRandomness<Self::AccountId, [u8; 32]>;
		#[pallet::constant]
		/// The amount that should be taken as a security deposit when requesting randomness.
		type Deposit: Get<BalanceOf<Self>>;
		#[pallet::constant]
		/// Number of blocks after a request is made when it can be purged from storage
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
		CannotBeFulfilledBeforeExpiry,
		RequestDNE,
		RequestCannotYetBeFulfilled,
		OnlyRequesterCanIncreaseFee,
		NewFeeMustBeGreaterThanOldFee,
		RequestHasNotExpired,
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
			info: RequestType<T::BlockNumber>,
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
		StorageMap<_, Blake2_128Concat, RequestId, RequestState<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn request_count)]
	/// Number of randomness requests made so far, used to generate the next request's uid
	pub type RequestCount<T: Config> = StorageValue<_, RequestId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn current_block_randomness)]
	/// Relay chain current block randomness
	/// TODO: replace with ParentBlockRandomness once
	/// https://github.com/paritytech/substrate/pull/11113 is merged
	pub type CurrentBlockRandomness<T: Config> = StorageValue<_, T::Hash, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn last_current_block_randomness)]
	/// Last relay chain current block randomness
	/// TODO: replace with LastParentBlockRandomness once
	/// https://github.com/paritytech/substrate/pull/11113 is merged
	pub type LastCurrentBlockRandomness<T: Config> = StorageValue<_, T::Hash, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn one_epoch_ago_randomness)]
	/// Relay chain one epoch ago randomness
	pub type OneEpochAgoRandomness<T: Config> = StorageValue<_, T::Hash, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn last_one_epoch_ago_randomness)]
	/// Last relay chain one epoch ago randomness
	pub type LastOneEpochAgoRandomness<T: Config> = StorageValue<_, T::Hash, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn two_epochs_ago_randomness)]
	/// Relay chain two epochs ago randomness
	pub type TwoEpochsAgoRandomness<T: Config> = StorageValue<_, T::Hash, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn last_two_epochs_ago_randomness)]
	/// Last relay chain two epochs ago randomness
	pub type LastTwoEpochsAgoRandomness<T: Config> = StorageValue<_, T::Hash, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_now: BlockNumberFor<T>) -> Weight {
			// Get randomness from runtime and set it locally
			// TODO: optimize by writing a PushRandomness trait which pushes to this pallet when
			// the item is updated instead...Need to know when the epoch changes from relay proof.
			let (
				last_current_block_randomness,
				last_one_epoch_ago_randomness,
				last_two_epochs_ago_randomness,
			) = (
				<CurrentBlockRandomness<T>>::get(),
				<OneEpochAgoRandomness<T>>::get(),
				<TwoEpochsAgoRandomness<T>>::get(),
			);
			<LastCurrentBlockRandomness<T>>::put(last_current_block_randomness);
			<LastOneEpochAgoRandomness<T>>::put(last_one_epoch_ago_randomness);
			<LastTwoEpochsAgoRandomness<T>>::put(last_two_epochs_ago_randomness);
			let (current_block_randomness, current_block_randomness_wt) =
				T::RelayRandomness::get_current_block_randomness();
			let (one_epoch_ago_randomness, one_epoch_ago_randomness_wt) =
				T::RelayRandomness::get_one_epoch_ago_randomness();
			let (two_epochs_ago_randomness, two_epochs_ago_randomness_wt) =
				T::RelayRandomness::get_two_epochs_ago_randomness();
			<CurrentBlockRandomness<T>>::put(current_block_randomness);
			<OneEpochAgoRandomness<T>>::put(one_epoch_ago_randomness);
			<TwoEpochsAgoRandomness<T>>::put(two_epochs_ago_randomness);
			3 * T::DbWeight::get().read
				+ 3 * T::DbWeight::get().write
				+ current_block_randomness_wt
				+ one_epoch_ago_randomness_wt
				+ two_epochs_ago_randomness_wt
		}
	}

	// Utility functions
	impl<T: Config> Pallet<T> {
		fn get_most_recent_babe_randomness(b: BabeRandomness) -> T::Hash {
			match b {
				BabeRandomness::OneEpochAgo => <OneEpochAgoRandomness<T>>::get(),
				BabeRandomness::TwoEpochsAgo => <TwoEpochsAgoRandomness<T>>::get(),
				BabeRandomness::CurrentBlock => <CurrentBlockRandomness<T>>::get(),
			}
		}
		fn concat_and_hash(a: T::Hash, b: T::Hash) -> [u8; 32] {
			let mut s = Vec::new();
			s.extend_from_slice(a.as_ref());
			s.extend_from_slice(b.as_ref());
			sp_io::hashing::blake2_256(&s)
		}
	}

	// This is where we expose pallet functionality for the precompile
	impl<T: Config> Pallet<T> {
		pub fn request_randomness(request: Request<T>) -> DispatchResult {
			ensure!(
				!request.can_be_fulfilled(),
				Error::<T>::CannotRequestPastRandomness
			);
			let deposit = T::Deposit::get().saturating_add(request.fee);
			// is the calling contract always the consuming contract??
			// or can the depositer be different from consumer
			T::Currency::can_reserve(&request.contract_address, deposit)
				.then(|| true)
				.ok_or(Error::<T>::NotSufficientDeposit)?;
			// get new request ID
			let request_id = <RequestCount<T>>::get();
			let next_id = request_id
				.checked_add(1u64)
				.ok_or(Error::<T>::RequestCounterOverflowed)?;
			T::Currency::reserve(&request.contract_address, deposit)?;
			let request: RequestState<T> = RequestState::new(request, deposit)?;
			// insert request
			<RequestCount<T>>::put(next_id);
			Self::deposit_event(Event::RandomnessRequested {
				id: request_id,
				refund_address: request.request.refund_address.clone(),
				contract_address: request.request.contract_address.clone(),
				fee: request.request.fee,
				salt: request.request.salt,
				info: request.request.info,
			});
			<Requests<T>>::insert(request_id, request);
			Ok(())
		}
		/// Execute fulfillment for randomness if it is due
		// TODO: fee management
		// execution costs - `request.fee` is refunded to `refund_address`
		// cost of execution refunded to the caller?
		pub fn execute_fulfillment(caller: T::AccountId, id: RequestId) -> DispatchResult {
			let request = <Requests<T>>::get(id).ok_or(Error::<T>::RequestDNE)?;
			// fulfill randomness request
			request.fulfill(&caller)?;
			<Requests<T>>::remove(id);
			Self::deposit_event(Event::RequestFulfilled { id });
			Ok(())
		}
		pub fn increase_request_fee(
			caller: T::AccountId,
			id: RequestId,
			new_fee: BalanceOf<T>,
		) -> DispatchResult {
			let mut request = <Requests<T>>::get(id).ok_or(Error::<T>::RequestDNE)?;
			// fulfill randomness request
			request.increase_fee(&caller, new_fee)?;
			<Requests<T>>::insert(id, request);
			Self::deposit_event(Event::RequestFeeIncreased { id, new_fee });
			Ok(())
		}
		/// Execute request expiration
		/// transfers fee to caller && purges request iff it has expired
		/// does NOT try to fulfill the request
		pub fn execute_request_expiration(caller: T::AccountId, id: RequestId) -> DispatchResult {
			let request = <Requests<T>>::get(id).ok_or(Error::<T>::RequestDNE)?;
			request.execute_expiration(&caller)?;
			<Requests<T>>::remove(id);
			Self::deposit_event(Event::RequestExpirationExecuted { id });
			Ok(())
		}
		pub fn instant_babe_randomness(
			contract_address: T::AccountId,
			babe_randomness: BabeRandomness,
			salt: T::Hash,
		) -> DispatchResult {
			let raw_randomness = Self::get_most_recent_babe_randomness(babe_randomness);
			let randomness = Self::concat_and_hash(raw_randomness, salt);
			T::RandomnessSender::send_randomness(contract_address, randomness);
			Ok(())
		}
		pub fn instant_local_randomness(
			_contract_address: T::AccountId,
			_salt: T::Hash,
		) -> DispatchResult {
			Err(Error::<T>::NotYetImplemented.into())
		}
	}
}
