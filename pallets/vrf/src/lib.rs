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
//! `most_recent_relay_storage_root + most_recent_relay_slot_number`
//!

#![cfg_attr(not(feature = "std"), no_std)]
use nimbus_primitives::{NimbusId, NIMBUS_ENGINE_ID};
use session_keys_primitives::{KeysLookup, VrfId};
use sp_application_crypto::ByteArray;
use sp_consensus_babe::{digests::PreDigest, Slot, Transcript, BABE_ENGINE_ID};
use sp_consensus_vrf::schnorrkel;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use frame_support::pallet;

pub use pallet::*;

type Randomness = schnorrkel::Randomness;

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// VRF inputs from the relay chain
	#[derive(Default, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct VrfInput<RelayHash, SlotNumber> {
		/// Expect this to change every block
		pub relay_storage_root: RelayHash,
		/// Relay slot number
		/// received via `well_known_keys::CURRENT_SLOT` on parachain_system::RelayStateProof
		pub relay_slot_number: SlotNumber,
	}

	/// For the runtime to implement to expose cumulus data to this pallet and cost of getting data
	pub trait GetMostRecentVrfInputs<RelayHash, SlotNumber> {
		/// Returns most recent relay storage root and weight consumed by get
		fn get_most_recent_relay_storage_root() -> (RelayHash, Weight);
		/// Returns most recent relay slot number and weight consumed by get
		fn get_most_recent_relay_slot_number() -> (SlotNumber, Weight);
	}

	/// Exposes randomness in this pallet to the runtime
	pub trait GetMaybeRandomness<R> {
		fn get_current_randomness() -> Option<R>;
	}

	/// Make VRF transcript
	pub fn make_transcript<Hash: AsRef<[u8]>>(input: VrfInput<Hash, Slot>) -> Transcript {
		let mut transcript = Transcript::new(&BABE_ENGINE_ID);
		transcript.append_u64(b"relay slot number", *input.relay_slot_number);
		transcript.append_message(b"relay block hash", input.relay_storage_root.as_ref());
		transcript
	}

	/// Pallet for storing and exposing VRF outputs by block authors
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The relay block hash type
		type RelayBlockHash: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ Default
			+ Copy
			+ AsRef<[u8]>;
		/// Gets the most recent relay block hash and relay slot number in `on_initialize`
		/// and returns weight consumed for getting these values
		type MostRecentVrfInputGetter: GetMostRecentVrfInputs<Self::RelayBlockHash, Slot>;
		/// Takes input NimbusId and gets back VrfId
		type VrfKeyLookup: KeysLookup<NimbusId, VrfId>;
	}

	/// Current block randomness
	/// Set in `on_initialize`, before it will contain the randomness for this block
	#[pallet::storage]
	#[pallet::getter(fn current_randomness)]
	pub type CurrentRandomness<T> = StorageValue<_, Randomness, OptionQuery>;

	/// Last block randomness
	/// Set in `on_initialize`, before it will contain the randomness from the last block
	#[pallet::storage]
	#[pallet::getter(fn last_randomness)]
	pub type LastRandomness<T> = StorageValue<_, Randomness, OptionQuery>;

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
			let (most_recent_relay_storage_root, recent_rsr_wt) =
				T::MostRecentVrfInputGetter::get_most_recent_relay_storage_root();
			let (most_recent_relay_slot_number, recent_rsn_wt) =
				T::MostRecentVrfInputGetter::get_most_recent_relay_slot_number();
			let last_vrf_inputs = <MostRecentVrfInput<T>>::take();
			// AT LEAST print if input uniqueness assumptions are violated (no reuse)
			if last_vrf_inputs.relay_storage_root == most_recent_relay_storage_root
				|| last_vrf_inputs.relay_slot_number == most_recent_relay_slot_number
			{
				log::warn!(
					"VRF on_initialize: Relay storage root or slot number did not change between \
				current and last block. Nimbus would have panicked if slot number didn't change \
				so probably storage root did not change."
				);
			}
			let most_recent_vrf_input = VrfInput {
				relay_storage_root: most_recent_relay_storage_root,
				relay_slot_number: most_recent_relay_slot_number,
			};
			<MostRecentVrfInput<T>>::put(most_recent_vrf_input.clone());
			(
				recent_rsr_wt + recent_rsn_wt + T::DbWeight::get().write,
				most_recent_vrf_input,
			)
		}
		/// Returns weight consumed in `on_initialize`
		fn set_randomness(input: VrfInput<T::RelayBlockHash, Slot>) -> Weight {
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
								.expect("NimbusId encoded in preruntime digest must be valid");

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
			let transcript = make_transcript::<T::RelayBlockHash>(input);
			let randomness: Randomness = maybe_pre_digest
				.and_then(|digest| {
					digest
						.vrf_output()
						.and_then(|vrf_output| {
							vrf_output.0.attach_input_hash(&pubkey, transcript).ok()
						})
						.map(|inout| inout.make_bytes(&sp_consensus_babe::BABE_VRF_INOUT_CONTEXT))
				})
				.expect("VRF output encoded in pre-runtime digest must be valid");
			CurrentRandomness::<T>::put(randomness);
			T::DbWeight::get().read + 2 * T::DbWeight::get().write
		}
	}

	impl<T: Config> GetMaybeRandomness<T::Hash> for Pallet<T> {
		fn get_current_randomness() -> Option<T::Hash> {
			if let Some(r) = CurrentRandomness::<T>::get() {
				T::Hash::decode(&mut &r[..]).ok()
			} else {
				None
			}
		}
	}
}
