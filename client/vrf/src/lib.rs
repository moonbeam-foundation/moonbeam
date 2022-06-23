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

use crate::digest::PreDigest;
use cumulus_client_consensus_common::{
	ParachainBlockImport, ParachainCandidate, ParachainConsensus,
};
use cumulus_primitives_core::{relay_chain::v2::Hash as PHash, ParaId, PersistedValidationData};
use log::{debug, info, warn};
// use nimbus_primitives::{
// 	AuthorFilterAPI, CompatibleDigestItem, NimbusApi, NimbusId, VRF_KEY_ID,
// };
use parking_lot::Mutex;
use sc_consensus::{BlockImport, BlockImportParams};
use session_keys_primitives::{make_transcript, make_transcript_data, VrfId, VRF_KEY_ID};
use sp_api::{ApiExt, BlockId, ProvideRuntimeApi};
use sp_application_crypto::{AppKey, ByteArray, CryptoTypePublicPair};
use sp_consensus::{
	BlockOrigin, EnableProofRecording, Environment, ProofRecording, Proposal, Proposer,
};
use sp_consensus_babe::Slot;
use sp_consensus_vrf::schnorrkel::{PublicKey, VRFOutput, VRFProof};
use sp_inherents::{CreateInherentDataProviders, InherentData, InherentDataProvider};
use sp_keystore::{SyncCryptoStore, SyncCryptoStorePtr};
use sp_runtime::{
	traits::{Block as BlockT, Header as HeaderT},
	DigestItem,
};
use std::convert::TryInto;
use std::{marker::PhantomData, sync::Arc, time::Duration};

const LOG_TARGET: &str = "signing-vrf";

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
