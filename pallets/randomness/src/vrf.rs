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
use crate::{Config, GetVrfInput, LocalVrfOutput, NextVrfInput};
use frame_support::{pallet_prelude::Weight, traits::Get};
use nimbus_primitives::{NimbusId, NIMBUS_ENGINE_ID};
use parity_scale_codec::Decode;
pub use session_keys_primitives::make_transcript;
use session_keys_primitives::{KeysLookup, VrfId, VRF_ENGINE_ID, VRF_INOUT_CONTEXT};
use sp_consensus_babe::digests::PreDigest;
use sp_consensus_vrf::schnorrkel;
use sp_core::crypto::ByteArray;

/// VRF output
type Randomness = schnorrkel::Randomness;

/// Returns weight consumed in `on_initialize`
pub(crate) fn set_randomness<T: Config>() -> Weight {
	// first block will be just default if it is not set, 0 input is default
	// TODO: client will need to sign LastVrfInput::get().unwrap_or_default() with vrf keys
	let input = <NextVrfInput<T>>::get().unwrap_or_default();
	let mut block_author_vrf_id: Option<VrfId> = None;
	let maybe_pre_digest: Option<PreDigest> = <frame_system::Pallet<T>>::digest()
		.logs
		.iter()
		.filter_map(|s| s.as_pre_runtime())
		.filter_map(|(id, mut data)| {
			if id == VRF_ENGINE_ID {
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
	let transcript = make_transcript::<T::Hash>(input.slot_number, input.storage_root);
	let vrf_output: Randomness = maybe_pre_digest
		.and_then(|digest| {
			digest
				.vrf_output()
				.and_then(|vrf_output| vrf_output.0.attach_input_hash(&pubkey, transcript).ok())
				// TODO: replace with VRF_INOUT_CONTEXT
				.map(|inout| inout.make_bytes(&VRF_INOUT_CONTEXT))
		})
		.expect("VRF output encoded in pre-runtime digest must be valid");
	LocalVrfOutput::<T>::put(T::Hash::decode(&mut &vrf_output[..]).ok());
	T::DbWeight::get().read + 2 * T::DbWeight::get().write
}

/// Set vrf input in storage and log warning if either of the values did NOT change
pub(crate) fn set_vrf_input<T: Config>() {
	let input = T::VrfInputGetter::get_vrf_input();
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
