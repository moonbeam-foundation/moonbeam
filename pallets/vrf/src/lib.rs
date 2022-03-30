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

//! VRF Pallet
//!
//! Stores VRF output per block by the block author as well as the VRF inputs:
//! `most_recent_relay_block_hash + most_recent_relay_slot_number`
//!

#![cfg_attr(not(feature = "std"), no_std)]

use sp_application_crypto::ByteArray;
use sp_consensus_babe::{digests::PreDigest, AuthorityId, Slot, Transcript, BABE_ENGINE_ID};
use sp_consensus_vrf::schnorrkel;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_support::pallet;

pub use pallet::*;

type MaybeRandomness = Option<schnorrkel::Randomness>;

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::Convert;

	/// VRF inputs from the relay chain
	/// TODO: needs custom Default implementation?
	#[derive(Default, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct VrfInput<RelayHash, SlotNumber> {
		/// TODO: rename to storage_root if parentStorageRoot always changes, even if block is empty
		pub relay_block_hash: RelayHash,
		/// Relay slot number
		/// received via `well_known_keys::CURRENT_SLOT` on parachain_system::RelayStateProof
		pub relay_slot_number: SlotNumber,
	}

	/// For the runtime to implement to expose cumulus data to this pallet and cost of getting data
	pub trait GetMostRecentVrfInputs<RelayHash, SlotNumber> {
		/// Returns most recent relay block hash and weight consumed by get
		fn get_most_recent_relay_block_hash() -> (RelayHash, Weight);
		/// Returns most recent relay slot number and weight consumed by get
		fn get_most_recent_relay_slot_number() -> (SlotNumber, Weight);
	}

	/// This trait tells us if the round changed in the current block
	/// => new set of authorities to be put in storage
	pub trait RoundChangedThisBlock {
		fn round_changed_this_block() -> bool;
	}

	/// Exposes randomness in this pallet to the runtime
	pub trait GetRandomness {
		type Randomness;
		fn get_last_randomness() -> Self::Randomness;
		fn get_current_randomness() -> Self::Randomness;
	}

	/// Make VRF transcript
	pub fn make_transcript<Hash: AsRef<[u8]>>(input: VrfInput<Hash, Slot>) -> Transcript {
		let mut transcript = Transcript::new(&BABE_ENGINE_ID);
		transcript.append_u64(b"relay slot number", *input.relay_slot_number);
		transcript.append_message(b"relay block hash", input.relay_block_hash.as_ref());
		transcript
	}

	/// Pallet for storing and exposing VRF outputs by block authors
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The relay block hash type (probably H256)
		type RelayBlockHash: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ Default
			+ Copy
			+ AsRef<[u8]>;
		/// Gets the most recent relay block hash and relay slot number in `on_initialize`
		/// and returns weight consumed for getting these values
		type MostRecentVrfInputGetter: GetMostRecentVrfInputs<Self::RelayBlockHash, Slot>;
		/// Convert account to VRF key, presumably via AuthorMapping instance
		/// TODO: maybe TryConvert and map to error because fallible conversion
		type AccountToVrfId: Convert<Self::AccountId, AuthorityId>;
		/// Whether or not the round changed this block
		type RoundChanged: RoundChangedThisBlock;
		/// Get the selected candidate accounts from staking
		type SelectedCandidates: Get<Vec<Self::AccountId>>;
	}

	/// Current block randomness
	/// Set in `on_initialize`, before it will contain the randomness from the last block
	#[pallet::storage]
	#[pallet::getter(fn current_randomness)]
	pub type CurrentRandomness<T> = StorageValue<_, MaybeRandomness, ValueQuery>;

	/// Last block randomness
	/// Set in `on_initialize`, before it will contain the randomness from the last last block
	#[pallet::storage]
	#[pallet::getter(fn last_randomness)]
	pub type LastRandomness<T> = StorageValue<_, MaybeRandomness, ValueQuery>;

	/// Current set of authorities by AuthorityId
	/// Set in `on_initialize` upon round changes
	#[pallet::storage]
	#[pallet::getter(fn authorities)]
	pub type Authorities<T> = StorageValue<_, Vec<AuthorityId>, ValueQuery>;

	/// Most recent VRF input from relay chain data
	/// Set in `on_initialize` before setting randomness
	#[pallet::storage]
	#[pallet::getter(fn most_recent_vrf_input)]
	pub(crate) type MostRecentVrfInput<T: Config> =
		StorageValue<_, VrfInput<T::RelayBlockHash, Slot>, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_now: BlockNumberFor<T>) -> Weight {
			let (set_inputs_weight, relay_based_vrf_input) = Self::set_most_recent_vrf_inputs();
			set_inputs_weight + Self::set_randomness(relay_based_vrf_input)
		}
	}

	impl<T: Config> Pallet<T> {
		/// Returns weight consumed and arguments for setting randomness
		fn set_most_recent_vrf_inputs() -> (Weight, VrfInput<T::RelayBlockHash, Slot>) {
			// TODO: log if different/equal to current value? as in new or not
			let (most_recent_relay_block_hash, recent_rbh_wt) =
				T::MostRecentVrfInputGetter::get_most_recent_relay_block_hash();
			let (most_recent_relay_slot_number, recent_rsn_wt) =
				T::MostRecentVrfInputGetter::get_most_recent_relay_slot_number();
			let most_recent_vrf_input = VrfInput {
				relay_block_hash: most_recent_relay_block_hash,
				relay_slot_number: most_recent_relay_slot_number,
			};
			<MostRecentVrfInput<T>>::put(most_recent_vrf_input.clone());
			(
				recent_rbh_wt + recent_rsn_wt + T::DbWeight::get().write,
				most_recent_vrf_input,
			)
		}
		/// Returns weight consumed in `on_initialize`
		fn set_randomness(input: VrfInput<T::RelayBlockHash, Slot>) -> Weight {
			let maybe_pre_digest: Option<PreDigest> = <frame_system::Pallet<T>>::digest()
				.logs
				.iter()
				.filter_map(|s| s.as_pre_runtime())
				.filter_map(|(id, mut data)| {
					if id == BABE_ENGINE_ID {
						PreDigest::decode(&mut data).ok()
					} else {
						None
					}
				})
				.next();
			let maybe_randomness: MaybeRandomness = maybe_pre_digest.and_then(|digest| {
				// Get the authority index of the current block author
				let authority_index = digest.authority_index();
				// Extract out the VRF output if we have it
				digest.vrf_output().and_then(|vrf_output| {
					// Reconstruct the bytes of VRFInOut using the authority id.
					Authorities::<T>::get()
						.get(authority_index as usize)
						.and_then(|author| {
							schnorrkel::PublicKey::from_bytes(author.as_slice()).ok()
						})
						.and_then(|pubkey| {
							let transcript = make_transcript::<T::RelayBlockHash>(input);
							vrf_output.0.attach_input_hash(&pubkey, transcript).ok()
						})
						.map(|inout| inout.make_bytes(&sp_consensus_babe::BABE_VRF_INOUT_CONTEXT))
				})
			});
			// Place last VRF output into the `LastRandomness` storage item
			LastRandomness::<T>::put(CurrentRandomness::<T>::take());
			// Place the current VRF output into the `CurrentRandomness` storage item.
			CurrentRandomness::<T>::put(maybe_randomness);
			T::DbWeight::get().read + 2 * T::DbWeight::get().write
		}
		/// Set authorities in storage if round changes
		pub fn on_new_round() -> Weight {
			let selected_candidates = T::SelectedCandidates::get();
			// one read per selected candidate to convert to AuthorityId used in VRF
			let returned_weight: Weight = selected_candidates.len() as u64
				* T::DbWeight::get().read
				+ T::DbWeight::get().write;
			Authorities::<T>::put(
				selected_candidates
					.into_iter()
					// 1 read
					.map(|x| T::AccountToVrfId::convert(x))
					.collect::<Vec<AuthorityId>>(),
			);
			returned_weight
		}
	}

	impl<T: Config> GetRandomness for Pallet<T> {
		type Randomness = MaybeRandomness;
		fn get_last_randomness() -> Self::Randomness {
			LastRandomness::<T>::get()
		}
		fn get_current_randomness() -> Self::Randomness {
			CurrentRandomness::<T>::get()
		}
	}
}
