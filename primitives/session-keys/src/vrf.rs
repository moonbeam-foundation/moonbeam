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

//! VRF Key type, which is sr25519
use sp_application_crypto::KeyTypeId;
use sp_runtime::ConsensusEngineId;

impl From<nimbus_primitives::NimbusId> for VrfId {
	fn from(nimbus_id: nimbus_primitives::NimbusId) -> VrfId {
		nimbus_id.into()
	}
}

/// The ConsensusEngineId for nimbus consensus
/// this same identifier will be used regardless of the filters installed
pub const VRF_ENGINE_ID: ConsensusEngineId = *b"rand";

/// The KeyTypeId used in the Nimbus consensus framework regardles of wat filters are in place.
/// If this gets well adopted, we could move this definition to sp_core to avoid conflicts.
pub const VRF_KEY_ID: KeyTypeId = KeyTypeId(*b"rand");

// The strongly-typed crypto wrappers to be used by VRF in the keystore
mod vrf_crypto {
	use sp_application_crypto::{app_crypto, sr25519};
	app_crypto!(sr25519, crate::vrf::VRF_KEY_ID);
}

/// A nimbus author identifier (A public key).
pub type VrfId = vrf_crypto::Public;

/// A nimbus signature.
pub type VrfSignature = vrf_crypto::Signature;

sp_application_crypto::with_pair! {
	/// A nimbus keypair
	pub type VrfPair = vrf_crypto::Pair;
}
