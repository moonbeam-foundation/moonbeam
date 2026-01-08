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

//! # Moonriver bridge primitives

#![cfg_attr(not(feature = "std"), no_std)]

pub use bp_bridge_hub_cumulus::{
	BlockLength, BlockWeights, Hasher, Nonce, SignedBlock, AVERAGE_BLOCK_INTERVAL,
	MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX, MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX,
};
use bp_messages::{ChainWithMessages, MessageNonce};

pub use moonbeam_core_primitives::{AccountId, Balance, BlockNumber, Hash, Header, Signature};

use bp_runtime::{
	decl_bridge_finality_runtime_apis, decl_bridge_messages_runtime_apis, Chain, ChainId, Parachain,
};
use frame_support::{dispatch::DispatchClass, weights::Weight};
use sp_runtime::StateVersion;
use xcm::latest::prelude::{Junction, Location, NetworkId};

/// Identifier of Moonriver parachain in the Kusama relay chain.
pub const PARACHAIN_ID: u32 = 2023;

/// Bridge lane identifier.
pub type LaneId = bp_messages::HashedLaneId;

/// Moonriver parachain.
pub struct Moonriver;

impl Chain for Moonriver {
	const ID: ChainId = *b"mnrv";

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

impl Parachain for Moonriver {
	const PARACHAIN_ID: u32 = PARACHAIN_ID;
	const MAX_HEADER_SIZE: u32 = 4_096;
}

impl ChainWithMessages for Moonriver {
	const WITH_CHAIN_MESSAGES_PALLET_NAME: &'static str =
		WITH_MOONRIVER_KUSAMA_MESSAGES_PALLET_NAME;

	const MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX: MessageNonce =
		MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	const MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX: MessageNonce =
		MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
}

/// Name of the With-MoonriverKusama messages pallet instance that is deployed at bridged chains.
pub const WITH_MOONRIVER_KUSAMA_MESSAGES_PALLET_NAME: &str = "BridgeKusamaMessages";

decl_bridge_finality_runtime_apis!(moonriver_kusama);
decl_bridge_messages_runtime_apis!(moonriver_kusama, LaneId);

frame_support::parameter_types! {
	pub GlobalConsensusLocation: Location = Location::new(
		2,
		[
			Junction::GlobalConsensus(NetworkId::Kusama),
			Junction::Parachain(Moonriver::PARACHAIN_ID)
		]
	);
}

/// Bridging primitives describing the Polkadot relay chain, which we need for the other side.
/// Same approach as in https://github.com/polkadot-fellows/runtimes/pull/627
pub mod bp_polkadot {
	use super::{decl_bridge_finality_runtime_apis, Chain, ChainId, StateVersion, Weight};
	use bp_header_chain::ChainWithGrandpa;
	pub use bp_polkadot_core::*;

	/// Polkadot Chain
	pub struct Polkadot;

	impl Chain for Polkadot {
		const ID: ChainId = *b"pdot";

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

	impl ChainWithGrandpa for Polkadot {
		const WITH_CHAIN_GRANDPA_PALLET_NAME: &'static str = WITH_POLKADOT_GRANDPA_PALLET_NAME;
		const MAX_AUTHORITIES_COUNT: u32 = MAX_AUTHORITIES_COUNT;
		const REASONABLE_HEADERS_IN_JUSTIFICATION_ANCESTRY: u32 =
			REASONABLE_HEADERS_IN_JUSTIFICATION_ANCESTRY;
		const MAX_MANDATORY_HEADER_SIZE: u32 = MAX_MANDATORY_HEADER_SIZE;
		const AVERAGE_HEADER_SIZE: u32 = AVERAGE_HEADER_SIZE;
	}

	/// Name of the parachains pallet in the Polkadot runtime.
	pub const PARAS_PALLET_NAME: &str = "Paras";
	/// Name of the With-Polkadot GRANDPA pallet instance that is deployed at bridged chains.
	pub const WITH_POLKADOT_GRANDPA_PALLET_NAME: &str = "BridgePolkadotGrandpa";
	/// Name of the With-Polkadot parachains pallet instance that is deployed at bridged chains.
	pub const WITH_POLKADOT_BRIDGE_PARACHAINS_PALLET_NAME: &str = "BridgePolkadotParachains";

	/// Maximal size of encoded `bp_parachains::ParaStoredHeaderData` structure among all Polkadot
	/// parachains.
	///
	/// It includes the block number and state root, so it shall be near 40 bytes, but let's have
	/// some reserve.
	pub const MAX_NESTED_PARACHAIN_HEAD_DATA_SIZE: u32 = 128;

	decl_bridge_finality_runtime_apis!(polkadot, grandpa);
}
