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

//! VRF pre digest object and conversion to DigestItem
use crate::vrf::{VrfSignature, VRF_ENGINE_ID};
use parity_scale_codec::{Decode, Encode};
use sp_consensus_vrf::schnorrkel::{VRFOutput, VRFProof};
use sp_runtime::{generic::DigestItem, RuntimeDebug};

/// Raw VRF pre-digest.
#[derive(Clone, RuntimeDebug, Encode, Decode)]
pub struct PreDigest {
	/// VRF output
	pub vrf_output: VRFOutput,
	/// VRF proof
	pub vrf_proof: VRFProof,
}

/// A digest item which is usable with moonbeam VRF.
pub trait CompatibleDigestItem: Sized {
	/// Construct a digest item which contains a VRF pre-digest.
	fn vrf_pre_digest(seal: PreDigest) -> Self;

	/// If this item is an VRF pre-digest, return it.
	fn as_vrf_pre_digest(&self) -> Option<PreDigest>;

	/// Construct a digest item which contains a VRF seal.
	fn vrf_seal(signature: VrfSignature) -> Self;

	/// If this item is a VRF signature, return the signature.
	fn as_vrf_seal(&self) -> Option<VrfSignature>;
}

impl CompatibleDigestItem for DigestItem {
	fn vrf_pre_digest(digest: PreDigest) -> Self {
		DigestItem::PreRuntime(VRF_ENGINE_ID, digest.encode())
	}

	fn as_vrf_pre_digest(&self) -> Option<PreDigest> {
		self.pre_runtime_try_to(&VRF_ENGINE_ID)
	}

	fn vrf_seal(signature: VrfSignature) -> Self {
		DigestItem::Seal(VRF_ENGINE_ID, signature.encode())
	}

	fn as_vrf_seal(&self) -> Option<VrfSignature> {
		self.seal_try_to(&VRF_ENGINE_ID)
	}
}
