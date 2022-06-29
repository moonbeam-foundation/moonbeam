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

use nimbus_primitives::NimbusId;
use session_keys_primitives::{make_transcript, make_transcript_data, PreDigest, VrfApi, VrfId};
use sp_application_crypto::{AppKey, ByteArray};
use sp_consensus_babe::Slot;
use sp_consensus_vrf::schnorrkel::{PublicKey, VRFOutput, VRFProof};
use sp_core::H256;
use sp_keystore::{SyncCryptoStore, SyncCryptoStorePtr};

/// Uses the runtime API to get the VRF inputs and sign them with the VRF key that
/// corresponds to the authoring NimbusId
pub fn vrf_pre_digest<B, C>(
	client: &C,
	keystore: &SyncCryptoStorePtr,
	nimbus_id: NimbusId,
	parent: H256,
) -> Option<sp_runtime::generic::DigestItem>
where
	B: sp_runtime::traits::Block<Hash = sp_core::H256>,
	C: sp_api::ProvideRuntimeApi<B>,
	C::Api: VrfApi<B>,
{
	let at = sp_api::BlockId::Hash(parent);
	let relay_slot_number: Slot = client
		.runtime_api()
		.get_relay_slot_number(&at)
		.expect("api error");
	let relay_storage_root: H256 = client
		.runtime_api()
		.get_relay_storage_root(&at)
		.expect("api error");
	let key: VrfId = client
		.runtime_api()
		.vrf_key_lookup(&at, nimbus_id)
		.expect("api error")?;
	let vrf_pre_digest = sign_vrf(relay_slot_number, relay_storage_root, key, &keystore)?;
	Some(session_keys_primitives::digest::CompatibleDigestItem::vrf_pre_digest(vrf_pre_digest))
}

/// Uses the runtime API to get mock VRF inputs and sign them with the VRF key that
/// corresponds to the authoring NimbusId
pub fn mock_vrf_pre_digest<B, C>(
	client: &C,
	keystore: &SyncCryptoStorePtr,
	nimbus_id: NimbusId,
	parent: H256,
) -> Option<sp_runtime::generic::DigestItem>
where
	B: sp_runtime::traits::Block<Hash = sp_core::H256>,
	C: sp_api::ProvideRuntimeApi<B>,
	C::Api: VrfApi<B>,
{
	let at = sp_api::BlockId::Hash(parent);
	let relay_slot_number: Slot = Slot::default();
	let relay_storage_root = parent;
	let key: VrfId = client
		.runtime_api()
		.vrf_key_lookup(&at, nimbus_id)
		.expect("api error")?;
	let vrf_pre_digest = sign_vrf(relay_slot_number, relay_storage_root, key, &keystore)?;
	Some(session_keys_primitives::digest::CompatibleDigestItem::vrf_pre_digest(vrf_pre_digest))
}

/// Signs the VrfInput using the private key corresponding to the input `key` public key
/// to be found in the input keystore
/// Returns None if key not found in keystore or if signature output cannot be validated by input
/// If successful, returns Some(VRF pre-digest)
fn sign_vrf(
	relay_slot_number: Slot,
	relay_storage_root: H256,
	key: VrfId,
	keystore: &SyncCryptoStorePtr,
) -> Option<PreDigest> {
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
		Some(PreDigest {
			vrf_output: VRFOutput(signature.output),
			vrf_proof: VRFProof(signature.proof),
		})
	} else {
		// Either `key` not found in keystore (if None returned) or an Err if something else failed
		None
	}
}
