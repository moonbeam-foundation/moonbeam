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
use crate::{Config, LocalVrfOutput, RandomnessResults, RequestType};
use nimbus_primitives::{NimbusId, NIMBUS_ENGINE_ID};
use parity_scale_codec::Decode;
pub use session_keys_primitives::make_transcript;
use session_keys_primitives::{KeysLookup, PreDigest, VrfId, VRF_ENGINE_ID, VRF_INOUT_CONTEXT};
use sp_consensus_vrf::schnorrkel;
use sp_core::crypto::ByteArray;

/// VRF output
type Randomness = schnorrkel::Randomness;

/// Gets VRF output from system digests and verifies it using the block author's VrfId
/// Transforms VRF output into randomness value and puts it into `LocalVrfOutput`
/// Fills the `RandomnessResult` associated with the current block if any requests exist
pub(crate) fn verify_and_set_output<T: Config>() {
	let mut block_author_vrf_id: Option<VrfId> = None;
	// Get VrfOutput and VrfProof from system digests
	// Expect client to insert VrfOutput, VrfProof into digests by setting
	// `BuildNimbusConsensusParams.additional_digests_provider` to `moonbeam_vrf::vrf_pre_digest`
	// (see moonbeam/node/service/src/lib.rs)
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
					panic!("VRF digest encoded in pre-runtime digest must be valid");
				}
			} else {
				if id == NIMBUS_ENGINE_ID {
					let nimbus_id = NimbusId::decode(&mut data)
						.expect("NimbusId encoded in pre-runtime digest must be valid");

					// Get the block author's VrfId, the public key corresponding to the private
					// key used to generate the VrfOutput, VrfProof
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
	let block_author_vrf_id = schnorrkel::PublicKey::from_bytes(block_author_vrf_id.as_slice())
		.expect("Expect VrfId to be valid schnorrkel public key");
	// VRF input is last block's VRF output
	let vrf_input_transcript =
		make_transcript::<T::Hash>(LocalVrfOutput::<T>::get().unwrap_or_default());
	// Verify VRF output + proof using input transcript + block author's VrfId
	assert!(
		block_author_vrf_id
			.vrf_verify(vrf_input_transcript.clone(), &vrf_output, &vrf_proof)
			.is_ok(),
		"VRF signature verification failed"
	);
	// Transform VrfOutput into randomness bytes stored on-chain
	let randomness: Randomness = vrf_output
		.attach_input_hash(&block_author_vrf_id, vrf_input_transcript)
		.ok()
		.map(|inout| inout.make_bytes(&VRF_INOUT_CONTEXT))
		.expect("Transforming VrfOutput into randomness bytes failed");
	let randomness = T::Hash::decode(&mut &randomness[..])
		.ok()
		.expect("Bytes can be decoded into T::Hash");
	LocalVrfOutput::<T>::put(Some(randomness));
	// Supply randomness result if any requests exist for the VRF output this block
	let local_vrf_this_block = RequestType::Local(frame_system::Pallet::<T>::block_number());
	if let Some(mut results) = RandomnessResults::<T>::get(&local_vrf_this_block) {
		results.randomness = Some(randomness);
		RandomnessResults::<T>::insert(local_vrf_this_block, results);
	}
}
