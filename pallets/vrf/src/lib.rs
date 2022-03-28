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

//! Per-Block VRF Pallet
//! - create a new key (ed25519) which is required to be registered for all block authors
//! (so now block authors requires 3 keys)
//! use schnorrkel for the VRF generation logic
//! - input message is last_relay_block_hash_stored + slot_number  where
//! last_relay_block_hash_stored is in storage and updated whenever the relay parent hash changes
//! (this ensures the block author cannot choose which relay block to use once asynchronous backing
//! breaks 1 parachain block per relay parent block assumption)
//! - output is H256 (32 bytes) and proof (64 bytes)
//! - the output of the VRF is inserted into the runtime digest s.t. it will be verified upon
//! importing the block
//! for verification, we use FindAuthor to get the block author's AccountId and then verify that
//! the session key used belongs to the block author (so need additional AuthorMapping for new keys)
//! - store last_randomness and current_randomness on-chain, where each is  struct { 32 byte
//! random number, 64 byte proof}

#![cfg_attr(not(feature = "std"), no_std)]

use sp_application_crypto::ByteArray;
use sp_consensus_babe::{digests::PreDigest, AuthorityId, Slot, Transcript, BABE_ENGINE_ID};
use sp_consensus_vrf::schnorrkel;

// #[cfg(test)]
// mod mock;
// #[cfg(test)]
// mod tests;

use frame_support::pallet;

pub use pallet::*;

type MaybeRandomness = Option<schnorrkel::Randomness>;

#[pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use sp_runtime::traits::AtLeast32BitUnsigned;

	/// Make VRF transcript
	pub fn make_transcript<T: Config>(relay_slot: Slot, relay_hash: &[u8]) -> Transcript {
		let mut transcript = Transcript::new(&BABE_ENGINE_ID);
		transcript.append_u64(b"relay slot number", *relay_slot);
		transcript.append_message(b"chain randomness", &relay_hash[..]);
		transcript
	}

	/// Pallet for storing collator VRF outputs
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	/// This field should always be populated during block processing.
	///
	/// It is set in `on_initialize`, before it will contain the value from the last block.
	#[pallet::storage]
	#[pallet::getter(fn author_vrf_randomness)]
	pub(super) type AuthorVrfRandomness<T> = StorageValue<_, MaybeRandomness, ValueQuery>;

	/// Current set of authorities by AuthorityId
	#[pallet::storage]
	#[pallet::getter(fn authorities)]
	pub type Authorities<T> = StorageValue<_, Vec<AuthorityId>, ValueQuery>;

	// TODO:
	// Store last relay block hash
	// Store last relay slot number

	impl<T: Config> Pallet<T> {
		fn do_initialize(now: T::BlockNumber) {
			let maybe_pre_digest: Option<PreDigest> = <frame_system::Pallet<T>>::digest()
				.logs
				.iter()
				.filter_map(|s| s.as_pre_runtime())
				.filter_map(|(id, mut data)| {
					// TODO: change to our own ID
					if id == BABE_ENGINE_ID {
						PreDigest::decode(&mut data).ok()
					} else {
						None
					}
				})
				.next();
			let maybe_randomness: MaybeRandomness = maybe_pre_digest.and_then(|digest| {
				// the slot number of the current block being initialized
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
							let transcript = make_transcript::<T>(
								Slot::default(),
								// TODO: use the relay block hash,
								&Vec::default(),
							);

							vrf_output.0.attach_input_hash(&pubkey, transcript).ok()
						})
						.map(|inout| inout.make_bytes(&sp_consensus_babe::BABE_VRF_INOUT_CONTEXT))
				})
			});
			// Place the VRF output into the `AuthorVrfRandomness` storage item.
			AuthorVrfRandomness::<T>::put(maybe_randomness);
		}
	}
}
