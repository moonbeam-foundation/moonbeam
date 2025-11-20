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

use crate::AvailableStakeCalls;
use crate::StakeEncodeCall;
use pallet_xcm_transactor::chain_indices::AssetHubIndices;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::{AccountIdLookup, StaticLookup};
use sp_runtime::AccountId32;
use sp_std::vec::Vec;

/// Test AssetHub indices - using indices similar to real AssetHub
pub const TEST_ASSETHUB_INDICES: AssetHubIndices = AssetHubIndices {
	// Pallet indices
	utility: 40,
	proxy: 42,
	staking: 80,
	nomination_pools: 81,
	delegated_staking: 84,
	assets: 50,
	nfts: 52,

	// Utility call indices
	as_derivative: 1,
	batch: 0,
	batch_all: 2,

	// Proxy call indices
	proxy_call: 0,
	add_proxy: 1,
	remove_proxy: 2,

	// Staking call indices
	bond: 0,
	bond_extra: 1,
	unbond: 2,
	withdraw_unbonded: 3,
	validate: 4,
	nominate: 5,
	chill: 6,
	set_payee: 7,
	set_controller: 8,
	rebond: 19,
};

#[derive(Encode, Decode)]
pub enum AssetHubCall {
	#[codec(index = 80u8)]
	Stake(StakeCall),
}

#[derive(Encode, Decode)]
pub enum StakeCall {
	#[codec(index = 0u16)]
	Bond(
		#[codec(compact)] u128,
		pallet_staking::RewardDestination<AccountId32>,
	),
	#[codec(index = 1u16)]
	BondExtra(#[codec(compact)] u128),
	#[codec(index = 2u16)]
	Unbond(#[codec(compact)] u128),
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
	SetController,
	#[codec(index = 19u16)]
	Rebond(#[codec(compact)] u128),
}

pub struct TestEncoder;

impl StakeEncodeCall<()> for TestEncoder {
	fn encode_call(
		_transactor: (),
		call: AvailableStakeCalls,
	) -> Result<Vec<u8>, xcm::latest::Error> {
		let encoded = match call {
			AvailableStakeCalls::Bond(bonded_amount, reward_destination) => {
				AssetHubCall::Stake(StakeCall::Bond(bonded_amount, reward_destination)).encode()
			}
			AvailableStakeCalls::BondExtra(bonded_amount) => {
				AssetHubCall::Stake(StakeCall::BondExtra(bonded_amount)).encode()
			}
			AvailableStakeCalls::Unbond(bonded_amount) => {
				AssetHubCall::Stake(StakeCall::Unbond(bonded_amount)).encode()
			}
			AvailableStakeCalls::WithdrawUnbonded(num_slashing_spans) => {
				AssetHubCall::Stake(StakeCall::WithdrawUnbonded(num_slashing_spans)).encode()
			}
			AvailableStakeCalls::Validate(validator_prefs) => {
				AssetHubCall::Stake(StakeCall::Validate(validator_prefs)).encode()
			}
			AvailableStakeCalls::Nominate(targets) => {
				let nominated: Vec<<AccountIdLookup<AccountId32, ()> as StaticLookup>::Source> =
					targets.iter().map(|add| (*add).clone().into()).collect();
				AssetHubCall::Stake(StakeCall::Nominate(nominated)).encode()
			}
			AvailableStakeCalls::Chill => AssetHubCall::Stake(StakeCall::Chill).encode(),
			AvailableStakeCalls::SetPayee(reward_destination) => {
				AssetHubCall::Stake(StakeCall::SetPayee(reward_destination.into())).encode()
			}
			AvailableStakeCalls::SetController => {
				AssetHubCall::Stake(StakeCall::SetController).encode()
			}
			AvailableStakeCalls::Rebond(bonded_amount) => {
				AssetHubCall::Stake(StakeCall::Rebond(bonded_amount)).encode()
			}
		};

		Ok(encoded)
	}
}
