#![cfg_attr(not(feature = "std"), no_std)]

use cumulus_primitives_core::relay_chain;
use parity_scale_codec::{Decode, Encode};
use sp_std::vec::Vec;

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	Debug,
	max_encoded_len::MaxEncodedLen,
)]
pub enum RelayChainProxyType {
	Any,
	NonTransfer,
	Governance,
	Staking,
	IdentityJudgement,
	CancelProxy,
}

impl Default for RelayChainProxyType {
	fn default() -> RelayChainProxyType {
		RelayChainProxyType::Any
	}
}

/// All possible messages that may be delivered to generic Substrate chain.
///
/// Note this enum may be used in the context of both Source (as part of `encode-call`)
/// and Target chain (as part of `encode-message/send-message`).

#[derive(Debug, PartialEq, Eq)]
pub enum AvailableUtilityCalls {
	AsDerivative(u16, Vec<u8>),
}

pub enum AvailableStakeCalls {
	Bond(
		relay_chain::AccountId,
		relay_chain::Balance,
		pallet_staking::RewardDestination<relay_chain::AccountId>,
	),
	BondExtra(relay_chain::Balance),
	Unbond(relay_chain::Balance),
	WithdrawUnbonded(u32),
	Validate(pallet_staking::ValidatorPrefs),
	Nominate(Vec<relay_chain::AccountId>),
	Chill,
	SetPayee(pallet_staking::RewardDestination<relay_chain::AccountId>),
	SetController(relay_chain::AccountId),
	Rebond(relay_chain::Balance),
}

pub trait UtilityEncodeCall {
	/// Encode call from the relay.
	fn encode_call(call: AvailableUtilityCalls) -> Vec<u8>;
}

pub trait StakeEncodeCall {
	/// Encode call from the relay.
	fn encode_call(call: AvailableStakeCalls) -> Vec<u8>;
}
