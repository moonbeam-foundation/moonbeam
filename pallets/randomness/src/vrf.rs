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

//! VRF logic
use crate::{
	Config, CurrentVrfInput, IsFirstBlock, LocalVrfOutput, RandomnessResults, RequestType,
};
use frame_support::{pallet_prelude::Weight, traits::Get};
use nimbus_primitives::{NimbusId, NIMBUS_ENGINE_ID};
use parity_scale_codec::Decode;
pub use session_keys_primitives::make_transcript;
use session_keys_primitives::{
	GetVrfInput, KeysLookup, PreDigest, VrfId, VRF_ENGINE_ID, VRF_INOUT_CONTEXT,
};
use sp_consensus_vrf::schnorrkel;
use sp_core::crypto::ByteArray;

/// VRF output
type Randomness = schnorrkel::Randomness;

/// Set vrf input in storage and log warning if either of the values did NOT change
/// Called in previous block's `on_finalize`
pub(crate) fn set_input<T: Config>() {
	let input = T::VrfInputGetter::get_vrf_input();
	if let Some(last_vrf_input) = <CurrentVrfInput<T>>::take() {
		// logs if input uniqueness assumptions are violated (no reuse of vrf inputs)
		if last_vrf_input.storage_root == input.storage_root
			|| last_vrf_input.slot_number == input.slot_number
		{
			log::warn!(
				"VRF on_initialize: storage root or slot number did not change between \
			current and last block. Nimbus would've panicked if slot number did not change \
			so probably storage root did not change."
			);
		}
	}
	<CurrentVrfInput<T>>::put(input);
}

/// Returns weight consumed in `on_initialize`
pub(crate) fn set_output<T: Config>() -> Weight {
	let input = <CurrentVrfInput<T>>::get().expect("VrfInput must be set to verify VrfOutput");
	// Do not set the output in the first block (genesis or runtime upgrade)
	// because we do not have any output
	if <IsFirstBlock<T>>::take().is_none() {
		<IsFirstBlock<T>>::put(());
		return 0;
	}
	let mut block_author_vrf_id: Option<VrfId> = None;
	let PreDigest {
		vrf_output,
		vrf_proof,
	} = <frame_system::Pallet<T>>::digest()
		.logs
		.iter()
		.filter_map(|s| s.as_pre_runtime())
		.filter_map(|(id, mut data)| {
			if id == VRF_ENGINE_ID {
				if let Ok(vrf_digest) = PreDigest::decode(&mut data) {
					Some(vrf_digest)
				} else {
					panic!("failed to decode VRF PreDigest");
				}
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
		.next()
		.expect("VRF PreDigest must be decoded from the digests");
	let block_author_vrf_id =
		block_author_vrf_id.expect("VrfId encoded in pre-runtime digest must be valid");
	let pubkey = schnorrkel::PublicKey::from_bytes(block_author_vrf_id.as_slice())
		.expect("Expect VrfId to be valid schnorrkel public key");
	let transcript = make_transcript::<T::Hash>(input.slot_number, input.storage_root);
	// NOTE: this is verified by the BlockExecutor when executing the block
	debug_assert!(pubkey
		.vrf_verify(transcript.clone(), &vrf_output, &vrf_proof)
		.is_ok());
	let vrf_output: Randomness = vrf_output
		.attach_input_hash(&pubkey, transcript)
		.ok()
		.map(|inout| inout.make_bytes(&VRF_INOUT_CONTEXT))
		.expect("VRF output encoded in pre-runtime digest must be valid");
	let raw_randomness_output = T::Hash::decode(&mut &vrf_output[..]).ok();
	LocalVrfOutput::<T>::put(raw_randomness_output);
	// Supply randomness result
	let local_vrf_this_block = RequestType::Local(frame_system::Pallet::<T>::block_number());
	if let Some(mut results) = RandomnessResults::<T>::get(&local_vrf_this_block) {
		if let Some(randomness) = raw_randomness_output {
			results.randomness = Some(randomness);
			RandomnessResults::<T>::insert(local_vrf_this_block, results);
		} else {
			log::warn!("Could not decode VRF output bytes into Hash Type");
		}
	}
	T::DbWeight::get().read // TODO: update weight
}
