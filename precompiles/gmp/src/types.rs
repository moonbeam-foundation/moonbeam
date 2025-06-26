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

//! Precompile to receive GMP callbacks and forward to XCM

use parity_scale_codec::{Decode, Encode};
use precompile_utils::prelude::*;
use sp_core::{H256, U256};
use sp_std::vec::Vec;

// The Polkadot-sdk removed support for XCM Location V1, but the GMP precompile still needs to support it,
// so we have to wrap VersionedLocation to re-add support for XCM Location V1.
#[derive(Encode, Decode, Debug)]
pub enum VersionedLocation {
	#[codec(index = 1)] // v2 is same as v1 and therefore re-using the v1 index
	V2(deprecated_xcm_v2::MultiLocationV2),
	#[codec(index = 3)]
	V3(xcm::v3::MultiLocation),
	#[codec(index = 4)]
	V4(xcm::v4::Location),
	#[codec(index = 5)]
	V5(xcm::v5::Location),
}

impl TryFrom<VersionedLocation> for xcm::latest::Location {
	type Error = ();

	fn try_from(value: VersionedLocation) -> Result<Self, Self::Error> {
		match value {
			VersionedLocation::V2(location) => {
				xcm::VersionedLocation::V3(location.try_into()?).try_into()
			}
			VersionedLocation::V3(location) => xcm::VersionedLocation::V3(location).try_into(),
			VersionedLocation::V4(location) => xcm::VersionedLocation::V4(location).try_into(),
			VersionedLocation::V5(location) => xcm::VersionedLocation::V5(location).try_into(),
		}
	}
}

// A user action which will attempt to route the transferred assets to the account/chain specified
// by the given Location. Recall that a Location can contain both a chain and an account
// on that chain, as this one should.
#[derive(Encode, Decode, Debug)]
pub struct XcmRoutingUserAction {
	pub destination: VersionedLocation,
}

// A user action which is the same as XcmRoutingUserAction but also allows a fee to be paid. The
// fee is paid in the same asset being transferred, and must be <= the amount being sent.
#[derive(Encode, Decode, Debug)]
pub struct XcmRoutingUserActionWithFee {
	pub destination: VersionedLocation,
	pub fee: U256,
}

// A simple versioning wrapper around the initial XcmRoutingUserAction use-case. This should make
// future breaking changes easy to add in a backwards-compatible way.
#[derive(Encode, Decode, Debug)]
#[non_exhaustive]
pub enum VersionedUserAction {
	V1(XcmRoutingUserAction),
	V2(XcmRoutingUserActionWithFee),
}

// Struct representing a Wormhole VM
// The main purpose of this struct is to decode the ABI encoded struct returned from certain calls
// in the Wormhole Ethereum contracts.
//
// https://github.com/wormhole-foundation/wormhole/blob/main/ethereum/contracts/Structs.sol
#[derive(Debug, solidity::Codec)]
pub struct WormholeVM {
	pub version: u8,
	pub timestamp: u32,
	pub nonce: u32,
	pub emitter_chain_id: u16,
	pub emitter_address: H256,
	pub sequence: u64,
	pub consistency_level: u8,
	pub payload: BoundedBytes<crate::GetCallDataLimit>,

	pub guardian_set_index: u32,
	pub signatures: Vec<WormholeSignature>, // TODO: review: can this allow unbounded allocations?
	pub hash: H256,
}

// Struct representing a Wormhole Signature struct
#[derive(Debug, solidity::Codec)]
pub struct WormholeSignature {
	pub r: U256,
	pub s: U256,
	pub v: u8,
	pub guardian_index: u8,
}

// Struct representing a wormhole "BridgeStructs.TransferWithPayload" struct
// As with WormholeVM, the main purpose of this struct is to decode the ABI encoded struct when it
// returned from calls to Wormhole Ethereum contracts.
#[derive(Debug, solidity::Codec)]
pub struct WormholeTransferWithPayloadData {
	pub payload_id: u8,
	pub amount: U256,
	pub token_address: H256,
	pub token_chain: u16,
	pub to: H256,
	pub to_chain: u16,
	pub from_address: H256,
	pub payload: BoundedBytes<crate::GetCallDataLimit>,
}

/// Reimplement the deprecated xcm v2 Location types to allow for backwards compatibility
mod deprecated_xcm_v2 {
	use super::*;

	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug)]
	pub struct MultiLocationV2 {
		pub parents: u8,
		pub interior: JunctionsV2,
	}

	impl TryFrom<MultiLocationV2> for xcm::v3::MultiLocation {
		type Error = ();

		fn try_from(value: MultiLocationV2) -> Result<Self, Self::Error> {
			Ok(xcm::v3::MultiLocation::new(
				value.parents,
				xcm::v3::Junctions::try_from(value.interior)?,
			))
		}
	}

	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug)]
	pub enum JunctionsV2 {
		/// The interpreting consensus system.
		Here,
		/// A relative path comprising 1 junction.
		X1(JunctionV2),
		/// A relative path comprising 2 junctions.
		X2(JunctionV2, JunctionV2),
		/// A relative path comprising 3 junctions.
		X3(JunctionV2, JunctionV2, JunctionV2),
		/// A relative path comprising 4 junctions.
		X4(JunctionV2, JunctionV2, JunctionV2, JunctionV2),
		/// A relative path comprising 5 junctions.
		X5(JunctionV2, JunctionV2, JunctionV2, JunctionV2, JunctionV2),
		/// A relative path comprising 6 junctions.
		X6(
			JunctionV2,
			JunctionV2,
			JunctionV2,
			JunctionV2,
			JunctionV2,
			JunctionV2,
		),
		/// A relative path comprising 7 junctions.
		X7(
			JunctionV2,
			JunctionV2,
			JunctionV2,
			JunctionV2,
			JunctionV2,
			JunctionV2,
			JunctionV2,
		),
		/// A relative path comprising 8 junctions.
		X8(
			JunctionV2,
			JunctionV2,
			JunctionV2,
			JunctionV2,
			JunctionV2,
			JunctionV2,
			JunctionV2,
			JunctionV2,
		),
	}

	impl TryFrom<JunctionsV2> for xcm::v3::Junctions {
		type Error = ();

		fn try_from(value: JunctionsV2) -> Result<Self, Self::Error> {
			use JunctionsV2::*;
			Ok(match value {
				Here => Self::Here,
				X1(j1) => Self::X1(j1.try_into()?),
				X2(j1, j2) => Self::X2(j1.try_into()?, j2.try_into()?),
				X3(j1, j2, j3) => Self::X3(j1.try_into()?, j2.try_into()?, j3.try_into()?),
				X4(j1, j2, j3, j4) => Self::X4(
					j1.try_into()?,
					j2.try_into()?,
					j3.try_into()?,
					j4.try_into()?,
				),
				X5(j1, j2, j3, j4, j5) => Self::X5(
					j1.try_into()?,
					j2.try_into()?,
					j3.try_into()?,
					j4.try_into()?,
					j5.try_into()?,
				),
				X6(j1, j2, j3, j4, j5, j6) => Self::X6(
					j1.try_into()?,
					j2.try_into()?,
					j3.try_into()?,
					j4.try_into()?,
					j5.try_into()?,
					j6.try_into()?,
				),
				X7(j1, j2, j3, j4, j5, j6, j7) => Self::X7(
					j1.try_into()?,
					j2.try_into()?,
					j3.try_into()?,
					j4.try_into()?,
					j5.try_into()?,
					j6.try_into()?,
					j7.try_into()?,
				),
				X8(j1, j2, j3, j4, j5, j6, j7, j8) => Self::X8(
					j1.try_into()?,
					j2.try_into()?,
					j3.try_into()?,
					j4.try_into()?,
					j5.try_into()?,
					j6.try_into()?,
					j7.try_into()?,
					j8.try_into()?,
				),
			})
		}
	}

	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug)]
	pub enum JunctionV2 {
		/// An indexed parachain belonging to and operated by the context.
		///
		/// Generally used when the context is a Polkadot Relay-chain.
		Parachain(#[codec(compact)] u32),
		/// A 32-byte identifier for an account of a specific network that is respected as a sovereign
		/// endpoint within the context.
		///
		/// Generally used when the context is a Substrate-based chain.
		AccountId32 { network: NetworkIdV2, id: [u8; 32] },
		/// An 8-byte index for an account of a specific network that is respected as a sovereign
		/// endpoint within the context.
		///
		/// May be used when the context is a Frame-based chain and includes e.g. an indices pallet.
		AccountIndex64 {
			network: NetworkIdV2,
			#[codec(compact)]
			index: u64,
		},
		/// A 20-byte identifier for an account of a specific network that is respected as a sovereign
		/// endpoint within the context.
		///
		/// May be used when the context is an Ethereum or Bitcoin chain or smart-contract.
		AccountKey20 { network: NetworkIdV2, key: [u8; 20] },
		/// An instanced, indexed pallet that forms a constituent part of the context.
		///
		/// Generally used when the context is a Frame-based chain.
		PalletInstance(u8),
		/// A non-descript index within the context location.
		///
		/// Usage will vary widely owing to its generality.
		///
		/// NOTE: Try to avoid using this and instead use a more specific item.
		GeneralIndex(#[codec(compact)] u128),
		/// A nondescript datum acting as a key within the context location.
		///
		/// Usage will vary widely owing to its generality.
		///
		/// NOTE: Try to avoid using this and instead use a more specific item.
		GeneralKey(sp_runtime::WeakBoundedVec<u8, sp_core::ConstU32<32>>),
		/// The unambiguous child.
		///
		/// Not currently used except as a fallback when deriving ancestry.
		OnlyChild,
		// The GMP precompile doesn't need to support Plurality Junction
		//Plurality { id: BodyId, part: BodyPart },
	}

	impl TryFrom<JunctionV2> for xcm::v3::Junction {
		type Error = ();

		fn try_from(value: JunctionV2) -> Result<Self, ()> {
			use JunctionV2::*;
			Ok(match value {
				Parachain(id) => Self::Parachain(id),
				AccountId32 { network, id } => Self::AccountId32 {
					network: network.into(),
					id,
				},
				AccountIndex64 { network, index } => Self::AccountIndex64 {
					network: network.into(),
					index,
				},
				AccountKey20 { network, key } => Self::AccountKey20 {
					network: network.into(),
					key,
				},
				PalletInstance(index) => Self::PalletInstance(index),
				GeneralIndex(id) => Self::GeneralIndex(id),
				GeneralKey(key) => match key.len() {
					len @ 0..=32 => Self::GeneralKey {
						length: len as u8,
						data: {
							let mut data = [0u8; 32];
							data[..len].copy_from_slice(&key[..]);
							data
						},
					},
					_ => return Err(()),
				},
				OnlyChild => Self::OnlyChild,
			})
		}
	}

	#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug)]
	pub enum NetworkIdV2 {
		/// Unidentified/any.
		Any,
		/// Some named network.
		Named(sp_runtime::WeakBoundedVec<u8, sp_core::ConstU32<32>>),
		/// The Polkadot Relay chain
		Polkadot,
		/// Kusama.
		Kusama,
	}

	impl From<NetworkIdV2> for Option<xcm::v3::NetworkId> {
		fn from(old: NetworkIdV2) -> Option<xcm::v3::NetworkId> {
			use NetworkIdV2::*;
			match old {
				Any => None,
				Named(_) => None,
				Polkadot => Some(xcm::v3::NetworkId::Polkadot),
				Kusama => Some(xcm::v3::NetworkId::Kusama),
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_versioned_user_action_decode() {
		// Encoded payload from this wormhole transfer:
		// https://wormholescan.io/#/tx/0x7a6985578742291842d25d80091cb8661f9ebf9301b266d6d4cd324758310569?view=advanced
		let encoded_payload = hex::decode(
			"0001010200c91f0100c862582c20ec0a5429c6d2239da9908f4b6c93ab4e2589784f8a5452f65f0e45",
		)
		.unwrap();

		// Ensure we can decode the VersionedUserAction
		let _versioned_user_action = VersionedUserAction::decode(&mut &encoded_payload[..])
			.expect("Failed to decode VersionedUserAction");
	}
}
