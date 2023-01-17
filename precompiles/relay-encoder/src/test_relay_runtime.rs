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

use crate::AvailableStakeCalls;
use crate::StakeEncodeCall;
use cumulus_primitives_core::{relay_chain::v2::HrmpChannelId, ParaId};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::{AccountIdLookup, StaticLookup};
use sp_runtime::AccountId32;
use sp_std::vec::Vec;

#[derive(Encode, Decode)]
pub enum RelayCall {
	#[codec(index = 1u8)]
	Stake(StakeCall),
	#[codec(index = 2u8)]
	Hrmp(HrmpCall),
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

// HRMP call encoding, needed for xcm transactor pallet
#[derive(Encode, Decode)]
pub enum HrmpCall {
	#[codec(index = 0u8)]
	InitOpenChannel(ParaId, u32, u32),
	#[codec(index = 1u8)]
	AcceptOpenChannel(ParaId),
	#[codec(index = 2u8)]
	CloseChannel(HrmpChannelId),
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

impl xcm_primitives::HrmpEncodeCall for TestEncoder {
	fn hrmp_encode_call(
		call: xcm_primitives::HrmpAvailableCalls,
	) -> Result<Vec<u8>, xcm::latest::Error> {
		match call {
			xcm_primitives::HrmpAvailableCalls::InitOpenChannel(a, b, c) => Ok(RelayCall::Hrmp(
				HrmpCall::InitOpenChannel(a.clone(), b.clone(), c.clone()),
			)
			.encode()),
			xcm_primitives::HrmpAvailableCalls::AcceptOpenChannel(a) => {
				Ok(RelayCall::Hrmp(HrmpCall::AcceptOpenChannel(a.clone())).encode())
			}
			xcm_primitives::HrmpAvailableCalls::CloseChannel(a) => {
				Ok(RelayCall::Hrmp(HrmpCall::CloseChannel(a.clone())).encode())
			}
		}
	}
}
