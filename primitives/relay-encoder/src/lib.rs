use cumulus_primitives_core::relay_chain;
use parity_scale_codec::{Decode, Encode};

/// The type used to represent the kinds of proxying allowed.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, max_encoded_len::MaxEncodedLen)]
pub enum RelayChainProxyType {
	Any,
	NonTransfer,
	Governance,
	Staking,
	IdentityJudgement,
	CancelProxy,
}

/// All possible messages that may be delivered to generic Substrate chain.
///
/// Note this enum may be used in the context of both Source (as part of `encode-call`)
/// and Target chain (as part of `encode-message/send-message`).
	
#[derive(Debug, PartialEq, Eq)]
  pub enum AvailableCalls {
	CreateAnonymusProxy(RelayChainProxyType, relay_chain::BlockNumber, u16),
	Proxy(
		relay_chain::AccountId,
		Option<RelayChainProxyType>,
		Vec<u8>,
	),
	Bond(relay_chain::AccountId, relay_chain::Balance, pallet_staking::RewardDestination<relay_chain::AccountId>),
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

pub trait EncodeCall {
	/// Encode call from the relay.
	fn encode_call(call: AvailableCalls) -> Vec<u8>;
}
