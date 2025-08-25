// Copyright 2025 Moonbeam foundation
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

//! # Moonbeam bridge primitives

#![cfg_attr(not(feature = "std"), no_std)]

pub use bp_bridge_hub_cumulus::{
	BlockLength, BlockWeights, Hasher, Nonce, SignedBlock, AVERAGE_BLOCK_INTERVAL,
	MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX, MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX,
};
use bp_messages::{ChainWithMessages, MessageNonce};

use bp_runtime::{
	decl_bridge_finality_runtime_apis, decl_bridge_messages_runtime_apis, Chain, ChainId, Parachain,
};
use frame_support::{dispatch::DispatchClass, weights::Weight};
pub use moonbeam_core_primitives::{AccountId, Balance, BlockNumber, Hash, Header, Signature};
use sp_runtime::StateVersion;
use xcm::latest::prelude::{Junction, Location, NetworkId};

/// Bridge lane identifier.
pub type LaneId = bp_messages::HashedLaneId;

/// Moonbeam parachain.
pub struct Moonbeam;

impl Chain for Moonbeam {
	const ID: ChainId = *b"mnbm";

	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hasher = Hasher;
	type Header = Header;

	type AccountId = AccountId;
	type Balance = Balance;
	type Nonce = Nonce;
	type Signature = Signature;

	const STATE_VERSION: StateVersion = StateVersion::V1;

	fn max_extrinsic_size() -> u32 {
		*BlockLength::get().max.get(DispatchClass::Normal)
	}

	fn max_extrinsic_weight() -> Weight {
		BlockWeights::get()
			.get(DispatchClass::Normal)
			.max_extrinsic
			.unwrap_or(Weight::MAX)
	}
}

impl Parachain for Moonbeam {
	const PARACHAIN_ID: u32 = PARACHAIN_ID;
	const MAX_HEADER_SIZE: u32 = 4_096;
}

impl ChainWithMessages for Moonbeam {
	const WITH_CHAIN_MESSAGES_PALLET_NAME: &'static str =
		WITH_MOONBEAM_POLKADOT_MESSAGES_PALLET_NAME;

	const MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX: MessageNonce =
		MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	const MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX: MessageNonce =
		MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
}

/// Identifier of Moonbeam parachain in the Polkadot relay chain.
pub const PARACHAIN_ID: u32 = 2004;

/// Name of the With-MoonbeamPolkadot messages pallet instance that is deployed at bridged chains.
pub const WITH_MOONBEAM_POLKADOT_MESSAGES_PALLET_NAME: &str = "BridgePolkadotMessages";

decl_bridge_finality_runtime_apis!(moonbeam_polkadot);
decl_bridge_messages_runtime_apis!(moonbeam_polkadot, LaneId);

frame_support::parameter_types! {
	pub GlobalConsensusLocation: Location = Location::new(
		2,
		[
			Junction::GlobalConsensus(NetworkId::Polkadot),
			Junction::Parachain(Moonbeam::PARACHAIN_ID)
		]
	);
}

/// Bridging primitives describing the Kusama relay chain, which we need for the other side.
/// Same approach as in https://github.com/polkadot-fellows/runtimes/pull/627
pub mod bp_kusama {
	use super::{decl_bridge_finality_runtime_apis, Chain, ChainId, StateVersion, Weight};
	use bp_header_chain::ChainWithGrandpa;
	pub use bp_polkadot_core::*;

	/// Kusama Chain
	pub struct Kusama;

	impl Chain for Kusama {
		const ID: ChainId = *b"ksma";

		type BlockNumber = BlockNumber;
		type Hash = Hash;
		type Hasher = Hasher;
		type Header = Header;

		type AccountId = AccountId;
		type Balance = Balance;
		type Nonce = Nonce;
		type Signature = Signature;

		const STATE_VERSION: StateVersion = StateVersion::V1;

		fn max_extrinsic_size() -> u32 {
			max_extrinsic_size()
		}

		fn max_extrinsic_weight() -> Weight {
			max_extrinsic_weight()
		}
	}

	impl ChainWithGrandpa for Kusama {
		const WITH_CHAIN_GRANDPA_PALLET_NAME: &'static str = WITH_KUSAMA_GRANDPA_PALLET_NAME;
		const MAX_AUTHORITIES_COUNT: u32 = MAX_AUTHORITIES_COUNT;
		const REASONABLE_HEADERS_IN_JUSTIFICATION_ANCESTRY: u32 =
			REASONABLE_HEADERS_IN_JUSTIFICATION_ANCESTRY;
		const MAX_MANDATORY_HEADER_SIZE: u32 = MAX_MANDATORY_HEADER_SIZE;
		const AVERAGE_HEADER_SIZE: u32 = AVERAGE_HEADER_SIZE;
	}

	/// Name of the parachains pallet in the Kusama runtime.
	pub const PARAS_PALLET_NAME: &str = "Paras";
	/// Name of the With-Kusama GRANDPA pallet instance that is deployed at bridged chains.
	pub const WITH_KUSAMA_GRANDPA_PALLET_NAME: &str = "BridgeKusamaGrandpa";
	/// Name of the With-Kusama parachains pallet instance that is deployed at bridged chains.
	pub const WITH_KUSAMA_BRIDGE_PARACHAINS_PALLET_NAME: &str = "BridgeKusamaParachains";

	/// Maximal size of encoded `bp_parachains::ParaStoredHeaderData` structure among all Polkadot
	/// parachains.
	///
	/// It includes the block number and state root, so it shall be near 40 bytes, but let's have
	/// some reserve.
	pub const MAX_NESTED_PARACHAIN_HEAD_DATA_SIZE: u32 = 128;

	decl_bridge_finality_runtime_apis!(kusama, grandpa);
}
