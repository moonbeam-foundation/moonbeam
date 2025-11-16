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

//! Kusama AssetHub pallet and call indices
//!
//! These indices have been verified against the actual Kusama AssetHub runtime metadata.
//!
//! ## Verification
//!
//! Verified using:
//! ```bash
//! subxt metadata --url wss://kusama-asset-hub-rpc.polkadot.io:443 --format json
//! ```
//!
//! ## Sources
//! - Runtime: Kusama AssetHub (asset-hub-kusama)
//! - Metadata version: V16
//! - Last verified: 2025-11-14

use pallet_xcm_transactor::chain_indices::AssetHubIndices;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::{AccountIdLookup, StaticLookup};
use sp_runtime::AccountId32;
use sp_std::vec::Vec;

/// Kusama AssetHub pallet and extrinsic indices
///
/// All indices have been verified against live Kusama AssetHub metadata.
/// Kusama AssetHub has the same pallet and call indices as Polkadot AssetHub.
pub const KUSAMA_ASSETHUB_INDICES: AssetHubIndices = AssetHubIndices {
	// Pallet indices (from AssetHub Kusama runtime metadata)
	utility: 40,
	proxy: 42,
	staking: 89,
	nomination_pools: 80,
	delegated_staking: 83,
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
	set_controller: 8, // Deprecated but present
	rebond: 19,
};

/// Root-level call enum for Kusama AssetHub
#[derive(Encode, Decode)]
pub enum AssetHubCall {
	#[codec(index = 89u8)]
	Staking(StakeCall),
}

/// Staking pallet call enum for Kusama AssetHub
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

pub struct KusamaAssetHubEncoder;

impl xcm_primitives::StakeEncodeCall<()> for KusamaAssetHubEncoder {
	fn encode_call(_transactor: (), call: xcm_primitives::AvailableStakeCalls) -> Vec<u8> {
		match call {
			xcm_primitives::AvailableStakeCalls::Bond(b, c) => {
				AssetHubCall::Staking(StakeCall::Bond(b, c)).encode()
			}

			xcm_primitives::AvailableStakeCalls::BondExtra(a) => {
				AssetHubCall::Staking(StakeCall::BondExtra(a)).encode()
			}

			xcm_primitives::AvailableStakeCalls::Unbond(a) => {
				AssetHubCall::Staking(StakeCall::Unbond(a)).encode()
			}

			xcm_primitives::AvailableStakeCalls::WithdrawUnbonded(a) => {
				AssetHubCall::Staking(StakeCall::WithdrawUnbonded(a)).encode()
			}

			xcm_primitives::AvailableStakeCalls::Validate(a) => {
				AssetHubCall::Staking(StakeCall::Validate(a)).encode()
			}

			xcm_primitives::AvailableStakeCalls::Chill => {
				AssetHubCall::Staking(StakeCall::Chill).encode()
			}

			xcm_primitives::AvailableStakeCalls::SetPayee(a) => {
				AssetHubCall::Staking(StakeCall::SetPayee(a.into())).encode()
			}

			xcm_primitives::AvailableStakeCalls::SetController => {
				AssetHubCall::Staking(StakeCall::SetController).encode()
			}

			xcm_primitives::AvailableStakeCalls::Rebond(a) => {
				AssetHubCall::Staking(StakeCall::Rebond(a.into())).encode()
			}

			xcm_primitives::AvailableStakeCalls::Nominate(a) => {
				let nominated: Vec<<AccountIdLookup<AccountId32, ()> as StaticLookup>::Source> =
					a.iter().map(|add| (*add).clone().into()).collect();

				AssetHubCall::Staking(StakeCall::Nominate(nominated)).encode()
			}
		}
	}
}
