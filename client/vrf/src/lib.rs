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
use sp_consensus_vrf::schnorrkel::{PublicKey, VRFOutput, VRFProof};
use sp_core::H256;
use sp_keystore::{SyncCryptoStore, SyncCryptoStorePtr};

/// Uses the runtime API to get the VRF inputs and sign them with the VRF key that
/// corresponds to the authoring NimbusId.
pub fn vrf_pre_digest<B, C>(
	client: &C,
	keystore: &SyncCryptoStorePtr,
	nimbus_id: NimbusId,
	parent: H256,
) -> sp_runtime::generic::DigestItem
where
	B: sp_runtime::traits::Block<Hash = sp_core::H256>,
	C: sp_api::ProvideRuntimeApi<B>,
	C::Api: VrfApi<B>,
{
	let at = sp_api::BlockId::Hash(parent);
	let runtime_api = client.runtime_api();

	let last_vrf_output = runtime_api.get_last_vrf_output(&at).ok().expect(
		"Runtime API not implemented; Runtime must be upgraded \
			new client is not backwards compatible with old runtimes",
	);
	let key: VrfId = runtime_api
		.vrf_key_lookup(&at, nimbus_id)
		.ok()
		.expect(
			"Runtime API: Failed to look up author's VRF key \
		new client is not backwards compatible with old runtimes",
		)
		.expect("No VRF key associated with author's NimbusId in VrfKeyLookup");
	let vrf_pre_digest = sign_vrf(last_vrf_output, key, &keystore);
	session_keys_primitives::digest::CompatibleDigestItem::vrf_pre_digest(vrf_pre_digest)
}

/// Signs the VrfInput using the private key corresponding to the input `key` public key
/// to be found in the input keystore
fn sign_vrf(last_vrf_output: H256, key: VrfId, keystore: &SyncCryptoStorePtr) -> PreDigest {
	let transcript = make_transcript(last_vrf_output.clone());
	let transcript_data = make_transcript_data(last_vrf_output);
	let try_sign =
		SyncCryptoStore::sr25519_vrf_sign(&**keystore, VrfId::ID, key.as_ref(), transcript_data);
	match try_sign {
		Ok(Some(signature)) => {
			let public = PublicKey::from_bytes(&key.to_raw_vec())
				.ok()
				.expect("Failed to convert VRF key to sp_consensus_vrf::PublicKey");
			if signature
				.output
				.attach_input_hash(&public, transcript)
				.is_err()
			{
				panic!("VRF signature cannot be validated using key and transcript");
			}
			PreDigest {
				vrf_output: VRFOutput(signature.output),
				vrf_proof: VRFProof(signature.proof),
			}
		}
		Ok(None) => panic!("VRF key not found in keystore"),
		_ => panic!("VRF signing failed, add code to propagate error if you see this"),
	}
}
