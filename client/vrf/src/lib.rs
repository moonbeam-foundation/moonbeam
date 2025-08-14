// Copyright 2019-2025 PureStake Inc.
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

use std::sync::Arc;

use nimbus_primitives::{DigestsProvider, NimbusId};
use schnorrkel::PublicKey;
use session_keys_primitives::{make_vrf_transcript, PreDigest, VrfApi, VrfId};
use sp_application_crypto::{AppCrypto, ByteArray};
use sp_core::H256;
use sp_keystore::{Keystore, KeystorePtr};

/// Uses the runtime API to get the VRF inputs and sign them with the VRF key that
/// corresponds to the authoring NimbusId.
pub fn vrf_pre_digest<B, C>(
	client: &C,
	keystore: &KeystorePtr,
	nimbus_id: NimbusId,
	parent: H256,
) -> Option<sp_runtime::generic::DigestItem>
where
	B: sp_runtime::traits::Block<Hash = sp_core::H256>,
	C: sp_api::ProvideRuntimeApi<B>,
	C::Api: VrfApi<B>,
{
	let runtime_api = client.runtime_api();

	// first ? for runtime API, second ? for if last vrf output is not available
	let last_vrf_output = runtime_api.get_last_vrf_output(parent).ok()??;
	// first ? for runtime API, second ? for not VRF key associated with NimbusId
	let key: VrfId = runtime_api.vrf_key_lookup(parent, nimbus_id).ok()??;
	let vrf_pre_digest = sign_vrf(last_vrf_output, key, &keystore)?;
	Some(session_keys_primitives::digest::CompatibleDigestItem::vrf_pre_digest(vrf_pre_digest))
}

/// Signs the VrfInput using the private key corresponding to the input `key` public key
/// to be found in the input keystore
fn sign_vrf(last_vrf_output: H256, key: VrfId, keystore: &KeystorePtr) -> Option<PreDigest> {
	let transcript = make_vrf_transcript(last_vrf_output);
	let try_sign = Keystore::sr25519_vrf_sign(
		&**keystore,
		VrfId::ID,
		key.as_ref(),
		&transcript.clone().into_sign_data(),
	);
	if let Ok(Some(signature)) = try_sign {
		let public = PublicKey::from_bytes(&key.to_raw_vec()).ok()?;
		if signature
			.pre_output
			.0
			.attach_input_hash(&public, transcript.0.clone())
			.is_err()
		{
			// VRF signature cannot be validated using key and transcript
			return None;
		}
		Some(PreDigest {
			vrf_output: signature.pre_output,
			vrf_proof: signature.proof,
		})
	} else {
		// VRF key not found in keystore or VRF signing failed
		None
	}
}

pub struct VrfDigestsProvider<B, C> {
	client: Arc<C>,
	keystore: Arc<dyn Keystore>,
	_marker: std::marker::PhantomData<B>,
}

impl<B, C> VrfDigestsProvider<B, C> {
	pub fn new(client: Arc<C>, keystore: Arc<dyn Keystore>) -> Self {
		Self {
			client,
			keystore,
			_marker: Default::default(),
		}
	}
}

impl<B, C> DigestsProvider<NimbusId, H256> for VrfDigestsProvider<B, C>
where
	B: sp_runtime::traits::Block<Hash = sp_core::H256>,
	C: sp_api::ProvideRuntimeApi<B>,
	C::Api: VrfApi<B>,
{
	type Digests = Option<sp_runtime::generic::DigestItem>;

	fn provide_digests(&self, nimbus_id: NimbusId, parent: H256) -> Self::Digests {
		vrf_pre_digest::<B, C>(&self.client, &self.keystore, nimbus_id, parent)
	}
}
