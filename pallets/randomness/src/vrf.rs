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
use crate::{Config, LocalVrfOutput, NotFirstBlock, RandomnessResults, RequestType};
use frame_support::{pallet_prelude::Weight, traits::Get};
use nimbus_primitives::{NimbusId, NIMBUS_ENGINE_ID};
use parity_scale_codec::Decode;
pub use session_keys_primitives::make_transcript;
use session_keys_primitives::{KeysLookup, PreDigest, VrfId, VRF_ENGINE_ID, VRF_INOUT_CONTEXT};
use sp_consensus_vrf::schnorrkel;
use sp_core::crypto::ByteArray;

/// VRF output
type Randomness = schnorrkel::Randomness;

/// Returns weight consumed in `on_initialize`
pub(crate) fn set_output<T: Config>() -> Weight {
	// Do not set the output in the first block (genesis or runtime upgrade)
	// because we do not have any input for author to sign (which would be set in last block)
	if NotFirstBlock::<T>::get().is_none() {
		NotFirstBlock::<T>::put(());
		LocalVrfOutput::<T>::put(Some(T::Hash::default()));
		return T::DbWeight::get().read + (T::DbWeight::get().write * 2);
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
		.expect("VRF PreDigest was not included in the digests (check rand key is in keystore)");
	let block_author_vrf_id =
		block_author_vrf_id.expect("VrfId encoded in pre-runtime digest must be valid");
	let pubkey = schnorrkel::PublicKey::from_bytes(block_author_vrf_id.as_slice())
		.expect("Expect VrfId to be valid schnorrkel public key");
	// VRF input is the previous VRF output
	let transcript = make_transcript::<T::Hash>(LocalVrfOutput::<T>::get().unwrap_or_default());
	// Verify VRF output + proof using input transcript and VrfId
	assert!(
		pubkey
			.vrf_verify(transcript.clone(), &vrf_output, &vrf_proof)
			.is_ok(),
		"VRF signature verification failed"
	);
	let vrf_output: Randomness = vrf_output
		.attach_input_hash(&pubkey, transcript)
		.ok()
		.map(|inout| inout.make_bytes(&VRF_INOUT_CONTEXT))
		.expect("VRF output encoded in pre-runtime digest must be valid");
	let randomness_output = T::Hash::decode(&mut &vrf_output[..])
		.ok()
		.expect("VRF output bytes can be decode into T::Hash");
	LocalVrfOutput::<T>::put(Some(randomness_output));
	// Supply randomness result
	let local_vrf_this_block = RequestType::Local(frame_system::Pallet::<T>::block_number());
	if let Some(mut results) = RandomnessResults::<T>::get(&local_vrf_this_block) {
		results.randomness = Some(randomness_output);
		RandomnessResults::<T>::insert(local_vrf_this_block, results);
	}
	// TODO: benchmark to fix this weight
	// reads + writes + margin_of_safety = 5_000_000_000
	6 * T::DbWeight::get().read + 2 * T::DbWeight::get().write + 5_000_000_000
}
