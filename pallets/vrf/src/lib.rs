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
//! `most_recent_storage_root + most_recent_slot_number`
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
	/// Both inputs are expected to change every block (TODO: should we enforce this?)
	#[derive(Default, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct VrfInput<RelayHash, SlotNumber> {
		/// Relay parent block slot number
		pub slot_number: SlotNumber,
		/// Relay parent block storage root
		pub storage_root: RelayHash,
	}

	/// For the runtime to implement to expose cumulus data to this pallet and cost of getting data
	pub trait GetVrfInputs<SlotNumber, StorageRoot> {
		/// Returns most recent relay slot number and weight consumed by get
		fn get_slot_number() -> (SlotNumber, Weight);
		/// Returns most recent relay storage root and weight consumed by get
		fn get_storage_root() -> (StorageRoot, Weight);
	}

	/// Exposes randomness in this pallet to the runtime
	pub trait MaybeGetRandomness<R> {
		fn maybe_get_randomness() -> Option<R>;
	}

	/// Make VRF transcript
	pub fn make_transcript<Hash: AsRef<[u8]>>(input: VrfInput<Hash, Slot>) -> Transcript {
		let mut transcript = Transcript::new(&BABE_ENGINE_ID);
		transcript.append_u64(b"relay slot number", *input.slot_number);
		transcript.append_message(b"relay storage root", input.storage_root.as_ref());
		transcript
	}

	/// Pallet for storing and exposing VRF outputs by block authors
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Gets the most recent relay block hash and relay slot number in `on_initialize`
		/// and returns weight consumed for getting these values
		type VrfInputs: GetVrfInputs<Slot, Self::Hash>;
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
	/// TODO: remove this, don't see why it is necessary or useful
	#[pallet::storage]
	#[pallet::getter(fn last_randomness)]
	pub type LastRandomness<T> = StorageValue<_, Randomness, OptionQuery>;

	/// Most recent VRF input from relay chain data
	/// Set in `on_initialize` before setting randomness
	#[pallet::storage]
	#[pallet::getter(fn current_vrf_input)]
	pub(crate) type CurrentVrfInput<T: Config> =
		StorageValue<_, VrfInput<T::Hash, Slot>, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_now: BlockNumberFor<T>) -> Weight {
			let (set_inputs_weight, relay_based_vrf_input) = Self::set_most_recent_vrf_inputs();
			set_inputs_weight + Self::set_randomness(relay_based_vrf_input)
		}
		// TODO: set vrf inputs with inherent without inputs
		// explain why it is safe to use, because `ValidationData` is killed in on_initialize
		// WHY NOT JUST SET THE VRF INPUT IN ON FINALIZE?
		// make sure vrf input is set
		// fn on_finalize()
	}

	impl<T: Config> Pallet<T> {
		/// Returns weight consumed and arguments for setting randomness
		fn set_most_recent_vrf_inputs() -> (Weight, VrfInput<T::Hash, Slot>) {
			let (storage_root, recent_rsr_wt) = T::VrfInputs::get_storage_root();
			let (slot_number, recent_rsn_wt) = T::VrfInputs::get_slot_number();
			let last_vrf_inputs = <CurrentVrfInput<T>>::take();
			// logs if input uniqueness assumptions are violated (no reuse of vrf inputs)
			if last_vrf_inputs.storage_root == storage_root
				|| last_vrf_inputs.slot_number == slot_number
			{
				log::warn!(
					"VRF on_initialize: Relay storage root or slot number did not change between \
				current and last block. Nimbus would have panicked if slot number did not change \
				so likely storage root did not change."
				);
			}
			let inputs = VrfInput {
				storage_root,
				slot_number,
			};
			<CurrentVrfInput<T>>::put(inputs.clone());
			(
				recent_rsr_wt + recent_rsn_wt + T::DbWeight::get().write,
				inputs,
			)
		}
		/// Returns weight consumed in `on_initialize`
		fn set_randomness(input: VrfInput<T::Hash, Slot>) -> Weight {
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
			let transcript = make_transcript::<T::Hash>(input);
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

	impl<T: Config> MaybeGetRandomness<T::Hash> for Pallet<T> {
		fn maybe_get_randomness() -> Option<T::Hash> {
			if let Some(r) = CurrentRandomness::<T>::get() {
				T::Hash::decode(&mut &r[..]).ok()
			} else {
				None
			}
		}
	}
}
