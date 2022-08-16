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

//! VRF Key type
use nimbus_primitives::NimbusId;
use sp_application_crypto::{sr25519, KeyTypeId, UncheckedFrom};
use sp_consensus_babe::Transcript;
#[cfg(feature = "std")]
use sp_keystore::vrf::{VRFTranscriptData, VRFTranscriptValue};
use sp_runtime::{BoundToRuntimeAppPublic, ConsensusEngineId};

/// Make VRF transcript from the VrfInput
pub fn make_transcript<Hash: AsRef<[u8]>>(last_vrf_output: Hash) -> Transcript {
	let mut transcript = Transcript::new(&VRF_ENGINE_ID);
	transcript.append_message(b"last vrf output", last_vrf_output.as_ref());
	transcript
}

/// Make a VRF transcript data container
#[cfg(feature = "std")]
pub fn make_transcript_data<Hash: AsRef<[u8]>>(last_vrf_output: Hash) -> VRFTranscriptData {
	VRFTranscriptData {
		label: &VRF_ENGINE_ID,
		items: vec![(
			"last vrf output",
			VRFTranscriptValue::Bytes(last_vrf_output.as_ref().to_vec()),
		)],
	}
}

/// Struct to implement `BoundToRuntimeAppPublic` by assigning Public = VrfId
pub struct VrfSessionKey;

impl BoundToRuntimeAppPublic for VrfSessionKey {
	type Public = VrfId;
}

impl From<NimbusId> for VrfId {
	fn from(nimbus_id: NimbusId) -> VrfId {
		let nimbus_as_sr25519: sr25519::Public = nimbus_id.into();
		let sr25519_as_bytes: [u8; 32] = nimbus_as_sr25519.into();
		sr25519::Public::unchecked_from(sr25519_as_bytes).into()
	}
}

/// The ConsensusEngineId for VRF keys
pub const VRF_ENGINE_ID: ConsensusEngineId = *b"rand";

/// The KeyTypeId used for VRF keys
pub const VRF_KEY_ID: KeyTypeId = KeyTypeId(VRF_ENGINE_ID);

/// VRFInOut context.
pub static VRF_INOUT_CONTEXT: &[u8] = b"VRFInOutContext";

// The strongly-typed crypto wrappers to be used by VRF in the keystore
mod vrf_crypto {
	use sp_application_crypto::{app_crypto, sr25519};
	app_crypto!(sr25519, crate::VRF_KEY_ID);
}

/// A vrf public key.
pub type VrfId = vrf_crypto::Public;

/// A vrf signature.
pub type VrfSignature = vrf_crypto::Signature;

sp_application_crypto::with_pair! {
	/// A vrf key pair
	pub type VrfPair = vrf_crypto::Pair;
}

sp_api::decl_runtime_apis! {
	pub trait VrfApi {
		fn get_last_vrf_output() -> Option<Block::Hash>;
		fn vrf_key_lookup(nimbus_id: nimbus_primitives::NimbusId) -> Option<crate::VrfId>;
	}
}

#[test]
fn nimbus_to_vrf_id() {
	for x in 0u8..10u8 {
		let nimbus_id: NimbusId = sr25519::Public::unchecked_from([x; 32]).into();
		let expected_vrf_id: VrfId = sr25519::Public::unchecked_from([x; 32]).into();
		let nimbus_to_vrf_id: VrfId = nimbus_id.into();
		assert_eq!(expected_vrf_id, nimbus_to_vrf_id);
	}
}
