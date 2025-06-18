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
use sp_runtime::{FixedPointNumber, FixedU128, Saturating, StateVersion};

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
	const PARACHAIN_ID: u32 = MOONBEAM_POLKADOT_PARACHAIN_ID;
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
pub const MOONBEAM_POLKADOT_PARACHAIN_ID: u32 = 2004;

/// Name of the With-MoonbeamPolkadot messages pallet instance that is deployed at bridged chains.
pub const WITH_MOONBEAM_POLKADOT_MESSAGES_PALLET_NAME: &str = "BridgePolkadotMessages";

/// Name of the With-MoonbeamPolkadot bridge-relayers pallet instance that is deployed at bridged
/// chains.
pub const WITH_MOONBEAM_POLKADOT_RELAYERS_PALLET_NAME: &str = "BridgeRelayers";

decl_bridge_finality_runtime_apis!(moonbeam_polkadot);
decl_bridge_messages_runtime_apis!(moonbeam_polkadot, LaneId);

// TODO: Update values
frame_support::parameter_types! {
	/// The XCM fee that is paid for executing XCM program (with `ExportMessage` instruction) at the Kusama
	/// BridgeHub.
	/// (initially was calculated by test `BridgeHubKusama::can_calculate_weight_for_paid_export_message_with_reserve_transfer` + `33%`)
	pub const BaseXcmFeeInGlmr: u128 = 601_115_666;

	/// Transaction fee that is paid at the Kusama BridgeHub for delivering single inbound message.
	/// (initially was calculated by test `BridgeHubKusama::can_calculate_fee_for_complex_message_delivery_transaction` + `33%`)
	pub const BaseDeliveryFeeInGlmr: u128 = 3_142_112_953;

	/// Transaction fee that is paid at the Kusama BridgeHub for delivering single outbound message confirmation.
	/// (initially was calculated by test `BridgeHubKusama::can_calculate_fee_for_complex_message_confirmation_transaction` + `33%`)
	pub const BaseConfirmationFeeInGlmr: u128 = 575_036_072;
}

/// Compute the total estimated fee that needs to be paid in GLMR by the sender when sending
/// message from Moonbeam to Moonriver.
pub fn estimate_moonbeam_to_moonriver_message_fee(
	moonriver_base_delivery_fee_in_umovr: Balance,
) -> Balance {
	// Sender must pay:
	//
	// 1) an approximate cost of XCM execution (`ExportMessage` and surroundings) at Moonbeam;
	//
	// 2) the approximate cost of Polkadot -> Kusama message delivery transaction on Moonriver,
	//    converted into KSMs using 1:5 conversion rate;
	//
	// 3) the approximate cost of Polkadot -> Kusama message confirmation transaction on Moonbeam.
	BaseXcmFeeInGlmr::get()
		.saturating_add(convert_from_umovr_to_uglmr(
			moonriver_base_delivery_fee_in_umovr,
		))
		.saturating_add(BaseConfirmationFeeInGlmr::get())
}

/// Compute the per-byte fee that needs to be paid in GLMRs by the sender when sending
/// message from Moonbeam to Moonriver.
pub fn estimate_moonbeam_to_moonriver_byte_fee() -> Balance {
	// the sender pays for the same byte twice:
	// 1) the first part comes from the HRMP, when message travels from Moonbeam to Moonriver;
	// 2) the second part is the payment for bytes of the message delivery transaction, which is
	//    "mined" at Moonriver. Hence, we need to use byte fees from that chain and
	//    convert it to GLMRs here.

	// TODO: move this to a constants crate per runtime
	// Similar to: system_parachains_constants::polkadot::fee::TRANSACTION_BYTE_FEE
	const MOONRIVER_TRANSACTION_BYTE_FEE: Balance = 1_000_000_000;

	convert_from_umovr_to_uglmr(MOONRIVER_TRANSACTION_BYTE_FEE)
}

/// Convert from uMOVRs to uGLMRs.
fn convert_from_umovr_to_uglmr(price_in_umovr: Balance) -> Balance {
	// assuming exchange rate is 5 MOVR for 1 GLMR
	let ksm_to_dot_economic_rate = FixedU128::from_rational(1, 5);

	ksm_to_dot_economic_rate
		.saturating_mul(FixedU128::saturating_from_integer(price_in_umovr))
		.into_inner()
		/ FixedU128::DIV
}
