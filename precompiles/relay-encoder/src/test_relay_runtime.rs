// We want to avoid including the rococo-runtime here.
// TODO: whenever a conclusion is taken from https://github.com/paritytech/substrate/issues/8158

use crate::AvailableStakeCalls;
use crate::StakeEncodeCall;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::{AccountIdLookup, StaticLookup};
use sp_runtime::AccountId32;
use sp_std::vec::Vec;

#[derive(Encode, Decode)]
pub enum RelayCall {
	#[codec(index = 1u8)]
	Stake(StakeCall),
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
	WithdrawUnbonded(u32),
	#[codec(index = 4u16)]
	Validate(pallet_staking::ValidatorPrefs),
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

pub struct TestEncoder;

impl StakeEncodeCall for TestEncoder {
	fn encode_call(call: AvailableStakeCalls) -> Vec<u8> {
		match call {
			AvailableStakeCalls::Bond(a, b, c) => {
				RelayCall::Stake(StakeCall::Bond(a.into(), b, c)).encode()
			}

			AvailableStakeCalls::BondExtra(a) => RelayCall::Stake(StakeCall::BondExtra(a)).encode(),

			AvailableStakeCalls::Unbond(a) => RelayCall::Stake(StakeCall::Unbond(a)).encode(),

			AvailableStakeCalls::WithdrawUnbonded(a) => {
				RelayCall::Stake(StakeCall::WithdrawUnbonded(a)).encode()
			}

			AvailableStakeCalls::Validate(a) => RelayCall::Stake(StakeCall::Validate(a)).encode(),

			AvailableStakeCalls::Chill => RelayCall::Stake(StakeCall::Chill).encode(),

			AvailableStakeCalls::SetPayee(a) => {
				RelayCall::Stake(StakeCall::SetPayee(a.into())).encode()
			}

			AvailableStakeCalls::SetController(a) => {
				RelayCall::Stake(StakeCall::SetController(a.into())).encode()
			}

			AvailableStakeCalls::Rebond(a) => {
				RelayCall::Stake(StakeCall::Rebond(a.into())).encode()
			}

			AvailableStakeCalls::Nominate(a) => {
				let nominated: Vec<<AccountIdLookup<AccountId32, ()> as StaticLookup>::Source> =
					a.iter().map(|add| (*add).clone().into()).collect();

				RelayCall::Stake(StakeCall::Nominate(nominated)).encode()
			}
		}
	}
}
