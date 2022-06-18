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
	use nimbus_primitives::{NimbusId, NIMBUS_ENGINE_ID};
	use pallet_evm::AddressMapping;
	use session_keys_primitives::{KeysLookup, VrfId};
	use sp_application_crypto::ByteArray;
	use sp_consensus_babe::{digests::PreDigest, Slot, Transcript, BABE_ENGINE_ID};
	use sp_consensus_vrf::schnorrkel;
	use sp_core::{H160, H256};
	use sp_runtime::traits::Saturating;
	use sp_std::{convert::TryInto, vec::Vec};

	/// Make VRF transcript
	fn make_transcript<Hash: AsRef<[u8]>>(input: VrfInput<Slot, Hash>) -> Transcript {
		let mut transcript = Transcript::new(&BABE_ENGINE_ID);
		transcript.append_u64(b"relay slot number", *input.slot_number);
		transcript.append_message(b"relay storage root", input.storage_root.as_ref());
		transcript
	}

	/// Request identifier, unique per request for randomness
	pub type RequestId = u64;
	/// VRF output
	type Randomness = schnorrkel::Randomness;

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
		/// Get the BABE data from the runtime
		type BabeDataGetter: GetBabeData<Self::BlockNumber, u64, Option<Self::Hash>>;
		/// Get the VRF input from the runtime
		type VrfInputGetter: GetVrfInput<VrfInput<Slot, Self::Hash>>;
		/// Takes NimbusId to return VrfId
		type VrfKeyLookup: KeysLookup<NimbusId, VrfId>;
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

	/// Current local per-block VRF randomness
	/// Set in `on_initialize`, before it will contain the randomness for this block
	#[pallet::storage]
	#[pallet::getter(fn local_vrf_output)]
	pub type LocalVrfOutput<T: Config> = StorageValue<_, Option<T::Hash>, ValueQuery>;

	/// VRF input for next block
	/// Set in `on_finalize` of previous block
	/// Used in `on_initialize` of this block to verify randomness
	#[pallet::storage]
	#[pallet::getter(fn next_vrf_input)]
	pub(crate) type NextVrfInput<T: Config> = StorageValue<_, VrfInput<Slot, T::Hash>>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// Set this block's randomness using the VRF output
		fn on_initialize(_now: BlockNumberFor<T>) -> Weight {
			// first block will be just default if it is not set, 0 input is default
			// TODO: client will need to sign LastVrfInput::get().unwrap_or_default() with vrf keys
			let vrf_input = <NextVrfInput<T>>::get().unwrap_or_default();
			Self::set_randomness(vrf_input)
		}
		// Set next block's VRF input in storage
		fn on_finalize(_now: BlockNumberFor<T>) {
			// Expect 1 read + 2 writes
			// Necessary because required data is killed in `ParachainSystem::on_initialize`
			Self::set_vrf_input(T::VrfInputGetter::get_vrf_input());
		}
	}

	impl<T: Config> Pallet<T> {
		/// Returns weight consumed in `on_initialize`
		fn set_randomness(input: VrfInput<Slot, T::Hash>) -> Weight {
			let mut block_author_vrf_id: Option<VrfId> = None;
			let maybe_pre_digest: Option<PreDigest> = <frame_system::Pallet<T>>::digest()
				.logs
				.iter()
				.filter_map(|s| s.as_pre_runtime())
				.filter_map(|(id, mut data)| {
					if id == BABE_ENGINE_ID {
						PreDigest::decode(&mut data).ok()
					} else {
						if id == NIMBUS_ENGINE_ID {
							let nimbus_id = NimbusId::decode(&mut data)
								.expect("NimbusId encoded in pre-runtime digest must be valid");

							block_author_vrf_id = Some(
								T::VrfKeyLookup::lookup_keys(&nimbus_id)
									.expect("No VRF Key Mapped to this NimbusId"),
							);
						}
						None
					}
				})
				.next();
			let block_author_vrf_id =
				block_author_vrf_id.expect("VrfId encoded in pre-runtime digest must be valid");
			let pubkey = schnorrkel::PublicKey::from_bytes(block_author_vrf_id.as_slice())
				.expect("Expect VrfId to be valid schnorrkel public key");
			let transcript = make_transcript::<T::Hash>(input);
			let vrf_output: Randomness = maybe_pre_digest
				.and_then(|digest| {
					digest
						.vrf_output()
						.and_then(|vrf_output| {
							vrf_output.0.attach_input_hash(&pubkey, transcript).ok()
						})
						.map(|inout| inout.make_bytes(&sp_consensus_babe::BABE_VRF_INOUT_CONTEXT))
				})
				.expect("VRF output encoded in pre-runtime digest must be valid");
			LocalVrfOutput::<T>::put(T::Hash::decode(&mut &vrf_output[..]).ok());
			T::DbWeight::get().read + 2 * T::DbWeight::get().write
		}
		/// Set vrf input in storage and log warning if either of the values did NOT change
		fn set_vrf_input(input: VrfInput<Slot, T::Hash>) {
			if let Some(previous_vrf_inputs) = <NextVrfInput<T>>::take() {
				// logs if input uniqueness assumptions are violated (no reuse of vrf inputs)
				if previous_vrf_inputs.storage_root == input.storage_root
					|| previous_vrf_inputs.slot_number == input.slot_number
				{
					log::warn!(
						"VRF on_initialize: storage root or slot number did not change between \
					current and last block. Nimbus would've panicked if slot number did not change \
					so probably storage root did not change."
					);
				}
			}
			<NextVrfInput<T>>::put(input);
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
