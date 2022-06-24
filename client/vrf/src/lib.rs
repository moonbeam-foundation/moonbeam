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

//! VRF client primitives for client-side verification

pub mod digest;

pub use crate::digest::PreDigest;
use session_keys_primitives::{make_transcript, make_transcript_data, VrfId};
use sp_application_crypto::{AppKey, ByteArray};
use sp_consensus_babe::Slot;
use sp_consensus_vrf::schnorrkel::{PublicKey, VRFOutput, VRFProof};
use sp_keystore::{SyncCryptoStore, SyncCryptoStorePtr};

/// Returns VRF pre-digest which includes an output signing the input info
// TODO: get VrfInput via runtime API before calling this in client
// TODO: get key via runtime API that gets it using the NimbusId that the client is using and the
// KeysLookup in the runtime
// `nimbus_consensus::produce_candidate` seems like place where the PreDigest needs to be added
pub fn vrf_predigest<Hash: AsRef<[u8]> + Clone>(
	relay_slot_number: Slot,
	relay_storage_root: Hash,
	key: VrfId,
	keystore: &SyncCryptoStorePtr,
) -> Option<crate::digest::PreDigest> {
	let transcript = make_transcript(relay_slot_number, relay_storage_root.clone());
	let transcript_data = make_transcript_data(relay_slot_number, relay_storage_root);
	let try_sign =
		SyncCryptoStore::sr25519_vrf_sign(&**keystore, VrfId::ID, key.as_ref(), transcript_data);
	if let Ok(Some(signature)) = try_sign {
		let public = PublicKey::from_bytes(&key.to_raw_vec()).ok()?;
		if signature
			.output
			.attach_input_hash(&public, transcript)
			.is_err()
		{
			// signature cannot be validated using key and transcript
			return None;
		}
		// TODO: verify we only need this output, the VrfId, and the VrfInput to validate or will
		// need to add fields to PreDigest
		Some(PreDigest {
			vrf_output: VRFOutput(signature.output),
			vrf_proof: VRFProof(signature.proof),
		})
	} else {
		// Either `key` not found in keystore (if None returned) or an Err if something else failed
		None
	}
}
