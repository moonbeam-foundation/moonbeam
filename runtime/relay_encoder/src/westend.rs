// We want to avoid including the rococo-runtime here.
// TODO: whenever a conclusion is taken from https://github.com/paritytech/substrate/issues/8158

use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::{AccountIdLookup, StaticLookup};
use sp_runtime::AccountId32;
use sp_runtime::Perbill;
use sp_std::vec::Vec;

#[derive(Encode, Decode)]
pub enum RelayCall {
	#[codec(index = 22u8)]
	// the index should match the position of the module in `construct_runtime!`
	Proxy(AnonymousProxyCall),
	#[codec(index = 6u8)]
	Stake(StakeCall),
}

#[derive(Encode, Decode)]
pub enum AnonymousProxyCall {
	#[codec(index = 0u8)]
	Proxy(
		AccountId32,
		Option<relay_encoder::RelayChainProxyType>,
		Vec<u8>,
	),

	#[codec(index = 4u8)]
	// the index should match the position of the dispatchable in the target pallet
	Anonymous(relay_encoder::RelayChainProxyType, u32, u16),
}

#[derive(Encode, Decode)]
pub enum StakeCall {
	#[codec(index = 0u16)]
	// the index should match the position of the dispatchable in the target pallet
	Bond(
		<AccountIdLookup<AccountId32, ()> as StaticLookup>::Source,
		#[codec(compact)] cumulus_primitives_core::relay_chain::Balance,
		pallet_staking::RewardDestination<AccountId32>,
	),
	#[codec(index = 1u16)]
	BondExtra(#[codec(compact)] cumulus_primitives_core::relay_chain::Balance),
	#[codec(index = 2u16)]
	Unbond(#[codec(compact)] cumulus_primitives_core::relay_chain::Balance),
	#[codec(index = 3u16)]
	WithdrawUnbonded(#[codec(compact)] u32),
	#[codec(index = 4u16)]
	Validate(#[codec(compact)] Perbill, bool),
	#[codec(index = 5u16)]
	Nominate(Vec<<AccountIdLookup<AccountId32, ()> as StaticLookup>::Source>),
	#[codec(index = 6u16)]
	Chill,
	#[codec(index = 7u16)]
	SetPayee(pallet_staking::RewardDestination<AccountId32>),
	#[codec(index = 8u16)]
	SetController(<AccountIdLookup<AccountId32, ()> as StaticLookup>::Source),
	#[codec(index = 19u16)]
	Rebond(#[codec(compact)] cumulus_primitives_core::relay_chain::Balance),
}

pub struct PolkadotEncoder;

impl relay_encoder::EncodeCall for PolkadotEncoder {
	fn encode_call(call: relay_encoder::AvailableCalls) -> Vec<u8> {
		match call {
			relay_encoder::AvailableCalls::CreateAnonymusProxy(a, b, c) => {
				RelayCall::Proxy(AnonymousProxyCall::Anonymous(a, b, c)).encode()
			}

			relay_encoder::AvailableCalls::Proxy(a, b, c) => {
				RelayCall::Proxy(AnonymousProxyCall::Proxy(a, b, c)).encode()
			}

			relay_encoder::AvailableCalls::Bond(a, b) => {
		
				RelayCall::Stake(StakeCall::Bond(
					a.into(),
					b,
					pallet_staking::RewardDestination::Controller,
				)).encode()
			}

			relay_encoder::AvailableCalls::BondExtra(a) => {
				RelayCall::Stake(StakeCall::BondExtra(
					a
				)).encode()
			}

			relay_encoder::AvailableCalls::Unbond(a) => {
				RelayCall::Stake(StakeCall::Unbond(
					a
				)).encode()
			}

			relay_encoder::AvailableCalls::WithdrawUnbonded(a) => {
				RelayCall::Stake(StakeCall::WithdrawUnbonded(
					a
				)).encode()
			}

			relay_encoder::AvailableCalls::Validate(a, b) => {
				RelayCall::Stake(StakeCall::Validate(
					a,
					b
				)).encode()
			}

			relay_encoder::AvailableCalls::Chill => {
				RelayCall::Stake(StakeCall::Chill).encode()
			}

			relay_encoder::AvailableCalls::SetPayee(a) => {
				RelayCall::Stake(StakeCall::SetPayee(a.into())).encode()
			}

			relay_encoder::AvailableCalls::SetController(a) => {
				RelayCall::Stake(StakeCall::SetController(a.into())).encode()
			}

			relay_encoder::AvailableCalls::Rebond(a) => {
				RelayCall::Stake(StakeCall::Rebond(a.into())).encode()
			}

			relay_encoder::AvailableCalls::Nominate(a) => {
				let nominated: Vec<<AccountIdLookup<AccountId32, ()> as StaticLookup>::Source> =
					a.iter().map(|add| (*add).clone().into()).collect();
				
					RelayCall::Stake(StakeCall::Nominate(nominated)).encode()
			
			}
		}
	}
}
