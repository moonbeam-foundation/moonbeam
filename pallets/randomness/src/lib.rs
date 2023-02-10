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

//! # Randomness Pallet
//!
//! This pallet provides access to 2 sources of randomness:
//! 1. local VRF, produced by collators per block
//! 2. relay chain BABE one epoch ago randomness, produced by the relay chain per relay chain epoch
//! These options are represented as `type::RequestType`.
//!
//! There are no extrinsics for this pallet. Instead, public functions on `Pallet<T: Config>` expose
//! user actions for the precompile i.e. `request_randomness`.
//!
//! ## Local VRF
//! The local VRF randomness is produced every block by the collator that authors the block.
//!
//! This pallet is default configured to look for the `VrfOutput` in `frame_system::digests()`. If
//! it cannot find the `VrfOutput` in `frame_system::digests()`, then it will panic and the block
//! will be invalid.
//!
//! Next, the `VrfOutput` is verified using the block author's `VrfId` and the VRF input, which is
//! last block's `VrfOutput`. If verification fails, then it will panic (block is invalid).
//!
//! Finally, the output is transformed into the randomness bytes stored on-chain and put in
//! `LocalVrfOutput`. Any pending randomness results for this block are filled with the
//! output randomness bytes.
//!
//! The function which contains this logic is `vrf::verify_and_set_output`. It is called in every
//! block's `on_initialize`.
//!
//! ## Babe Epoch Randomness
//! Babe epoch randomness is retrieved once every relay chain epoch.
//!
//! The `set_babe_randomness_results` mandatory inherent reads the Babe epoch randomness from the
//! relay chain state proof and fills any pending `RandomnessResults` for this epoch randomness.
//!
//! `Config::BabeDataGetter` is responsible for reading the epoch index and epoch randomness
//! from the relay chain state proof. The moonbeam `GetBabeData` implementation is in the runtime.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet;
pub use pallet::*;
use sp_std::vec::Vec;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;
pub mod types;
pub mod vrf;
pub use types::*;
pub mod weights;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Read babe randomness info from the relay chain state proof
pub trait GetBabeData<EpochIndex, Randomness> {
	fn get_epoch_index() -> EpochIndex;
	fn get_epoch_randomness() -> Randomness;
}

#[pallet]
pub mod pallet {
	use super::*;
	use crate::weights::{SubstrateWeight, WeightInfo};
	use frame_support::traits::{Currency, ExistenceRequirement::KeepAlive};
	use frame_support::{pallet_prelude::*, PalletId};
	use frame_system::pallet_prelude::*;
	use nimbus_primitives::NimbusId;
	use pallet_evm::AddressMapping;
	use session_keys_primitives::{InherentError, KeysLookup, VrfId, INHERENT_IDENTIFIER};
	use sp_core::{H160, H256};
	use sp_runtime::traits::{AccountIdConversion, Hash, Saturating};
	use sp_std::convert::TryInto;

	/// The Randomness's pallet id
	pub const PALLET_ID: PalletId = PalletId(*b"moonrand");

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
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Address mapping to convert from H160 to AccountId
		type AddressMapping: AddressMapping<Self::AccountId>;
		/// Currency in which the security deposit will be taken.
		type Currency: Currency<Self::AccountId>;
		/// Get the BABE data from the runtime
		type BabeDataGetter: GetBabeData<u64, Option<Self::Hash>>;
		/// Takes NimbusId to return VrfId
		type VrfKeyLookup: KeysLookup<NimbusId, VrfId>;
		#[pallet::constant]
		/// The amount that should be taken as a security deposit when requesting randomness.
		type Deposit: Get<BalanceOf<Self>>;
		/// Maximum number of random words that can be requested per request
		#[pallet::constant]
		type MaxRandomWords: Get<u8>;
		/// Local per-block VRF requests must be at least this many blocks after the block in which
		/// they were requested
		#[pallet::constant]
		type MinBlockDelay: Get<Self::BlockNumber>;
		/// Local per-block VRF requests must be at most this many blocks after the block in which
		/// they were requested
		#[pallet::constant]
		type MaxBlockDelay: Get<Self::BlockNumber>;
		/// Local requests expire and can be purged from storage after this many blocks/epochs
		#[pallet::constant]
		type BlockExpirationDelay: Get<Self::BlockNumber>;
		/// Babe requests expire and can be purged from storage after this many blocks/epochs
		#[pallet::constant]
		type EpochExpirationDelay: Get<u64>;
	}

	#[pallet::error]
	pub enum Error<T> {
		RequestCounterOverflowed,
		RequestFeeOverflowed,
		MustRequestAtLeastOneWord,
		CannotRequestMoreWordsThanMax,
		CannotRequestRandomnessAfterMaxDelay,
		CannotRequestRandomnessBeforeMinDelay,
		RequestDNE,
		RequestCannotYetBeFulfilled,
		OnlyRequesterCanIncreaseFee,
		RequestHasNotExpired,
		RandomnessResultDNE,
		RandomnessResultNotFilled,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		RandomnessRequestedBabeEpoch {
			id: RequestId,
			refund_address: H160,
			contract_address: H160,
			fee: BalanceOf<T>,
			gas_limit: u64,
			num_words: u8,
			salt: H256,
			earliest_epoch: u64,
		},
		RandomnessRequestedLocal {
			id: RequestId,
			refund_address: H160,
			contract_address: H160,
			fee: BalanceOf<T>,
			gas_limit: u64,
			num_words: u8,
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

	/// Randomness requests not yet fulfilled or purged
	#[pallet::storage]
	#[pallet::getter(fn requests)]
	pub type Requests<T: Config> = StorageMap<_, Twox64Concat, RequestId, RequestState<T>>;

	/// Number of randomness requests made so far, used to generate the next request's uid
	#[pallet::storage]
	#[pallet::getter(fn request_count)]
	pub type RequestCount<T: Config> = StorageValue<_, RequestId, ValueQuery>;

	/// Current local per-block VRF randomness
	/// Set in `on_initialize`
	#[pallet::storage]
	#[pallet::getter(fn local_vrf_output)]
	pub type LocalVrfOutput<T: Config> = StorageValue<_, Option<T::Hash>, ValueQuery>;

	/// Relay epoch
	#[pallet::storage]
	#[pallet::getter(fn relay_epoch)]
	pub(crate) type RelayEpoch<T: Config> = StorageValue<_, u64, ValueQuery>;

	/// Ensures the mandatory inherent was included in the block
	#[pallet::storage]
	#[pallet::getter(fn inherent_included)]
	pub(crate) type InherentIncluded<T: Config> = StorageValue<_, ()>;

	/// Records whether this is the first block (genesis or runtime upgrade)
	#[pallet::storage]
	#[pallet::getter(fn not_first_block)]
	pub type NotFirstBlock<T: Config> = StorageValue<_, ()>;

	/// Snapshot of randomness to fulfill all requests that are for the same raw randomness
	/// Removed once $value.request_count == 0
	#[pallet::storage]
	#[pallet::getter(fn randomness_results)]
	pub type RandomnessResults<T: Config> =
		StorageMap<_, Twox64Concat, RequestType<T>, RandomnessResult<T::Hash>>;

	/// Previous local per-block VRF randomness
	/// Set in `on_finalize` of last block
	#[pallet::storage]
	#[pallet::getter(fn previous_local_vrf_output)]
	pub type PreviousLocalVrfOutput<T: Config> = StorageValue<_, T::Hash, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Populates `RandomnessResults` due this epoch with BABE epoch randomness
		#[pallet::call_index(0)]
		#[pallet::weight((
			SubstrateWeight::<T>::set_babe_randomness_results(),
			DispatchClass::Mandatory
		))]
		pub fn set_babe_randomness_results(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			ensure_none(origin)?;
			let last_relay_epoch_index = <RelayEpoch<T>>::get();
			let relay_epoch_index = T::BabeDataGetter::get_epoch_index();
			if relay_epoch_index > last_relay_epoch_index {
				let babe_one_epoch_ago_this_block = RequestType::BabeEpoch(relay_epoch_index);
				// populate `RandomnessResults` for BABE epoch randomness
				if let Some(mut results) =
					<RandomnessResults<T>>::get(&babe_one_epoch_ago_this_block)
				{
					if let Some(randomness) = T::BabeDataGetter::get_epoch_randomness() {
						results.randomness = Some(randomness);
						<RandomnessResults<T>>::insert(babe_one_epoch_ago_this_block, results);
					} else {
						log::warn!(
							"Failed to fill BABE epoch randomness results \
							REQUIRE HOTFIX TO FILL EPOCH RANDOMNESS RESULTS FOR EPOCH {:?}",
							relay_epoch_index
						);
					}
				}
			}
			<RelayEpoch<T>>::put(relay_epoch_index);
			<InherentIncluded<T>>::put(());
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
					"Inherent required to set babe randomness results",
				),
			)))
		}

		// The empty-payload inherent extrinsic.
		fn create_inherent(_data: &InherentData) -> Option<Self::Call> {
			Some(Call::set_babe_randomness_results {})
		}

		fn is_inherent(call: &Self::Call) -> bool {
			matches!(call, Call::set_babe_randomness_results { .. })
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_now: BlockNumberFor<T>) -> Weight {
			// Do not set the output in the first block (genesis or runtime upgrade)
			// because we do not have any input for author to sign
			if NotFirstBlock::<T>::get().is_none() {
				NotFirstBlock::<T>::put(());
				LocalVrfOutput::<T>::put(Some(T::Hash::default()));
				return T::DbWeight::get().reads_writes(1, 2);
			}
			// Verify VRF output included by block author and set it in storage
			vrf::verify_and_set_output::<T>();
			SubstrateWeight::<T>::on_initialize()
		}
		fn on_finalize(_now: BlockNumberFor<T>) {
			// Ensure the mandatory inherent was included in the block or the block is invalid
			assert!(
				<InherentIncluded<T>>::take().is_some(),
				"Mandatory randomness inherent not included; InherentIncluded storage item is empty"
			);

			// set previous vrf output
			PreviousLocalVrfOutput::<T>::put(
				LocalVrfOutput::<T>::get().expect("LocalVrfOutput must exist; qed"),
			);
		}
	}

	// Randomness trait
	impl<T: Config> frame_support::traits::Randomness<T::Hash, BlockNumberFor<T>> for Pallet<T> {
		/// Uses the vrf output of previous block to generate a random seed. The provided `subject`
		/// must have the property to uniquely generate different randomness given the same vrf
		/// output (e.g. relay block number).
		///
		/// In our case the `subject` is provided via Nimbus and consists of three parts:
		///       1. Constant string *b"filter" - to identify author-slot-filter pallet
		///       2. First 2 bytes of index.to_le_bytes() when selecting the ith eligible author
		///       3. First 4 bytes of slot_number.to_be_bytes()
		///
		/// Note: This needs to be updated when asynchronous backing is in effect,
		///       as it will be unsafe.
		fn random(subject: &[u8]) -> (T::Hash, BlockNumberFor<T>) {
			let local_vrf_output = PreviousLocalVrfOutput::<T>::get();
			let block_number = frame_system::Pallet::<T>::block_number();
			let mut digest = Vec::new();
			digest.extend_from_slice(local_vrf_output.as_ref());
			digest.extend_from_slice(subject);
			let randomness = T::Hashing::hash(digest.as_slice());
			(randomness, block_number)
		}
	}

	// Read-only functions
	impl<T: Config> Pallet<T> {
		/// Returns the pallet account
		pub fn account_id() -> T::AccountId {
			PALLET_ID.into_account_truncating()
		}
		/// Returns total balance in the pallet account
		pub fn total_locked() -> BalanceOf<T> {
			// expect free balance == usable balance for pallet account because it is not controlled
			// by anyone so balance should never be locked
			T::Currency::free_balance(&Self::account_id())
		}
	}

	// Public functions for precompile usage only
	impl<T: Config> Pallet<T> {
		/// Make request for future randomness
		pub fn request_randomness(
			request: Request<BalanceOf<T>, RequestType<T>>,
		) -> Result<RequestId, sp_runtime::DispatchError> {
			let request = RequestState::new(request.into())?;
			let (fee, contract_address, info) = (
				request.request.fee,
				request.request.contract_address,
				request.request.info.clone(),
			);
			let total_to_reserve = T::Deposit::get().saturating_add(fee);
			let contract_address = T::AddressMapping::into_account_id(contract_address);
			// get new request ID
			let request_id = <RequestCount<T>>::get();
			let next_id = request_id
				.checked_add(1u64)
				.ok_or(Error::<T>::RequestCounterOverflowed)?;
			// send deposit to reserve account
			T::Currency::transfer(
				&contract_address,
				&Self::account_id(),
				total_to_reserve,
				KeepAlive,
			)?;
			let info_key: RequestType<T> = info.into();
			if let Some(existing_randomness_snapshot) = <RandomnessResults<T>>::take(&info_key) {
				<RandomnessResults<T>>::insert(
					&info_key,
					existing_randomness_snapshot.increment_request_count(),
				);
			} else {
				<RandomnessResults<T>>::insert(&info_key, RandomnessResult::new());
			}
			// insert request
			<RequestCount<T>>::put(next_id);
			request.request.emit_randomness_requested_event(request_id);
			<Requests<T>>::insert(request_id, request);
			Ok(request_id)
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
			request: Request<BalanceOf<T>, RequestInfo<T>>,
			deposit: BalanceOf<T>,
			caller: &H160,
			cost_of_execution: BalanceOf<T>,
		) {
			request.finish_fulfill(deposit, caller, cost_of_execution);
			let info_key: RequestType<T> = request.info.into();
			if let Some(result) = RandomnessResults::<T>::take(&info_key) {
				if let Some(new_result) = result.decrement_request_count() {
					RandomnessResults::<T>::insert(&info_key, new_result);
				} // else RandomnessResult is removed from storage
			}
			<Requests<T>>::remove(id);
			Self::deposit_event(Event::RequestFulfilled { id });
		}
		/// Increase fee associated with request
		pub fn increase_request_fee(
			caller: &H160,
			id: RequestId,
			fee_increase: BalanceOf<T>,
		) -> DispatchResult {
			let mut request = <Requests<T>>::get(id).ok_or(Error::<T>::RequestDNE)?;
			// Increase randomness request fee
			let new_fee = request.increase_fee(caller, fee_increase)?;
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
			let info_key: RequestType<T> = request.request.info.into();
			if let Some(result) = RandomnessResults::<T>::take(&info_key) {
				if let Some(new_result) = result.decrement_request_count() {
					RandomnessResults::<T>::insert(&info_key, new_result);
				} // else RandomnessResult is removed from storage
			}
			<Requests<T>>::remove(id);
			Self::deposit_event(Event::RequestExpirationExecuted { id });
			Ok(())
		}
	}
}
