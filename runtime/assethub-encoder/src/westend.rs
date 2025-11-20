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

//! Westend AssetHub pallet and call indices
//!
//! These indices have been verified against the actual Westend AssetHub runtime metadata.
//!
//! ## Verification
//!
//! Verified using:
//! ```bash
//! subxt metadata --url wss://westend-asset-hub-rpc.polkadot.io:443 --format json
//! ```
//!
//! ## Sources
//! - Runtime: Westend AssetHub (asset-hub-westend)
//! - Metadata version: V16
//! - Last verified: 2025-11-14

use pallet_xcm_transactor::chain_indices::AssetHubIndices;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::{AccountIdLookup, StaticLookup};
use sp_runtime::AccountId32;
use sp_std::vec::Vec;

/// Westend AssetHub pallet and extrinsic indices
///
/// All indices have been verified against live Westend AssetHub metadata.
///
/// **NOTE**: Westend has different pallet indices than Polkadot/Kusama:
/// - Staking is at index 80 (not 89)
/// - NominationPools is at index 81 (not 80)
/// - DelegatedStaking is at index 84 (not 83)
pub const WESTEND_ASSETHUB_INDICES: AssetHubIndices = AssetHubIndices {
	// Pallet indices (from AssetHub Westend runtime metadata)
	// NOTE: Different from Polkadot/Kusama
	utility: 40,
	proxy: 42,
	staking: 80,           // Different from Polkadot (89)
	nomination_pools: 81,  // Different from Polkadot (80)
	delegated_staking: 84, // Different from Polkadot (83)
	assets: 50,
	nfts: 52,

	// Utility call indices (same as Polkadot/Kusama)
	as_derivative: 1,
	batch: 0,
	batch_all: 2,

	// Proxy call indices (same as Polkadot/Kusama)
	proxy_call: 0,
	add_proxy: 1,
	remove_proxy: 2,

	// Staking call indices (same as Polkadot/Kusama)
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

/// Root-level call enum for Westend AssetHub
/// NOTE: Westend uses index 80 for Staking (different from Polkadot/Kusama which use 89)
#[derive(Encode, Decode)]
pub enum AssetHubCall {
	#[codec(index = 80u8)]
	Staking(StakeCall),
}

/// Staking pallet call enum for Westend AssetHub
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

pub struct WestendAssetHubEncoder;

impl xcm_primitives::StakeEncodeCall<()> for WestendAssetHubEncoder {
	fn encode_call(
		_transactor: (),
		call: xcm_primitives::AvailableStakeCalls,
	) -> Result<Vec<u8>, xcm::latest::Error> {
		let encoded = match call {
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
		};

		Ok(encoded)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::westend::WestendAssetHubEncoder;

	use xcm_primitives::StakeEncodeCall;

	#[test]
	fn test_stake_bond() {
		let controller: AccountId32 = [1u8; 32].into();

		// Expected encoding: [pallet_index, call_index, ...call_data]
		// Pallet: 80 (Staking on Westend AssetHub)
		// Call: 0 (bond)
		let mut expected_encoded: Vec<u8> = Vec::new();
		expected_encoded.push(80u8); // Staking pallet index

		let mut expected = StakeCall::Bond(
			100u32.into(),
			pallet_staking::RewardDestination::Account(controller.clone()),
		)
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendAssetHubEncoder as StakeEncodeCall<()>>::encode_call(
				(),
				xcm_primitives::AvailableStakeCalls::Bond(
					100u32.into(),
					pallet_staking::RewardDestination::Account(controller.clone()),
				)
			)
			.unwrap(),
			expected_encoded.clone()
		);

		sp_io::TestExternalities::default().execute_with(|| {
			// Pallet-xcm-transactor encoder returns same result when configured
			pallet_xcm_transactor::ChainIndicesMap::<moonbase_runtime::Runtime>::insert(
				moonbase_runtime::xcm_config::Transactors::AssetHub,
				pallet_xcm_transactor::chain_indices::ChainIndices::AssetHub(
					WESTEND_ASSETHUB_INDICES,
				),
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<moonbase_runtime::Runtime> as StakeEncodeCall<
					moonbase_runtime::xcm_config::Transactors,
				>>::encode_call(
					moonbase_runtime::xcm_config::Transactors::AssetHub,
					xcm_primitives::AvailableStakeCalls::Bond(
						100u32.into(),
						pallet_staking::RewardDestination::Account(controller),
					)
				)
				.unwrap(),
				expected_encoded
			);
		});
	}

	#[test]
	fn test_stake_bond_extra() {
		// Expected encoding: [pallet_index, call_index, ...call_data]
		let mut expected_encoded: Vec<u8> = Vec::new();
		expected_encoded.push(80u8); // Staking pallet index

		let mut expected = StakeCall::BondExtra(100u32.into()).encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendAssetHubEncoder as StakeEncodeCall<()>>::encode_call(
				(),
				xcm_primitives::AvailableStakeCalls::BondExtra(100u32.into())
			)
			.unwrap(),
			expected_encoded.clone()
		);

		sp_io::TestExternalities::default().execute_with(|| {
			pallet_xcm_transactor::ChainIndicesMap::<moonbase_runtime::Runtime>::insert(
				moonbase_runtime::xcm_config::Transactors::AssetHub,
				pallet_xcm_transactor::chain_indices::ChainIndices::AssetHub(
					WESTEND_ASSETHUB_INDICES,
				),
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<moonbase_runtime::Runtime> as StakeEncodeCall<
					moonbase_runtime::xcm_config::Transactors,
				>>::encode_call(
					moonbase_runtime::xcm_config::Transactors::AssetHub,
					xcm_primitives::AvailableStakeCalls::BondExtra(100u32.into())
				)
				.unwrap(),
				expected_encoded
			);
		});
	}

	#[test]
	fn test_stake_unbond() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		expected_encoded.push(80u8);

		let mut expected = StakeCall::Unbond(100u32.into()).encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendAssetHubEncoder as StakeEncodeCall<()>>::encode_call(
				(),
				xcm_primitives::AvailableStakeCalls::Unbond(100u32.into())
			)
			.unwrap(),
			expected_encoded.clone()
		);

		sp_io::TestExternalities::default().execute_with(|| {
			pallet_xcm_transactor::ChainIndicesMap::<moonbase_runtime::Runtime>::insert(
				moonbase_runtime::xcm_config::Transactors::AssetHub,
				pallet_xcm_transactor::chain_indices::ChainIndices::AssetHub(
					WESTEND_ASSETHUB_INDICES,
				),
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<moonbase_runtime::Runtime> as StakeEncodeCall<
					moonbase_runtime::xcm_config::Transactors,
				>>::encode_call(
					moonbase_runtime::xcm_config::Transactors::AssetHub,
					xcm_primitives::AvailableStakeCalls::Unbond(100u32.into())
				)
				.unwrap(),
				expected_encoded
			);
		});
	}

	#[test]
	fn test_stake_withdraw_unbonded() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		expected_encoded.push(80u8);

		let mut expected = StakeCall::WithdrawUnbonded(100u32).encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendAssetHubEncoder as StakeEncodeCall<()>>::encode_call(
				(),
				xcm_primitives::AvailableStakeCalls::WithdrawUnbonded(100u32,)
			)
			.unwrap(),
			expected_encoded.clone()
		);

		sp_io::TestExternalities::default().execute_with(|| {
			pallet_xcm_transactor::ChainIndicesMap::<moonbase_runtime::Runtime>::insert(
				moonbase_runtime::xcm_config::Transactors::AssetHub,
				pallet_xcm_transactor::chain_indices::ChainIndices::AssetHub(
					WESTEND_ASSETHUB_INDICES,
				),
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<moonbase_runtime::Runtime> as StakeEncodeCall<
					moonbase_runtime::xcm_config::Transactors,
				>>::encode_call(
					moonbase_runtime::xcm_config::Transactors::AssetHub,
					xcm_primitives::AvailableStakeCalls::WithdrawUnbonded(100u32,)
				)
				.unwrap(),
				expected_encoded
			);
		});
	}

	#[test]
	fn test_stake_validate() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		expected_encoded.push(80u8);

		let mut expected = StakeCall::Validate(pallet_staking::ValidatorPrefs::default()).encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendAssetHubEncoder as StakeEncodeCall<()>>::encode_call(
				(),
				xcm_primitives::AvailableStakeCalls::Validate(
					pallet_staking::ValidatorPrefs::default()
				)
			)
			.unwrap(),
			expected_encoded.clone()
		);

		sp_io::TestExternalities::default().execute_with(|| {
			pallet_xcm_transactor::ChainIndicesMap::<moonbase_runtime::Runtime>::insert(
				moonbase_runtime::xcm_config::Transactors::AssetHub,
				pallet_xcm_transactor::chain_indices::ChainIndices::AssetHub(
					WESTEND_ASSETHUB_INDICES,
				),
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<moonbase_runtime::Runtime> as StakeEncodeCall<
					moonbase_runtime::xcm_config::Transactors,
				>>::encode_call(
					moonbase_runtime::xcm_config::Transactors::AssetHub,
					xcm_primitives::AvailableStakeCalls::Validate(
						pallet_staking::ValidatorPrefs::default()
					)
				)
				.unwrap(),
				expected_encoded
			);
		});
	}

	#[test]
	fn test_stake_nominate() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		expected_encoded.push(80u8);

		let account1: AccountId32 = [1u8; 32].into();
		let account2: AccountId32 = [2u8; 32].into();
		let targets = vec![account1.clone(), account2.clone()];

		let nominated: Vec<<AccountIdLookup<AccountId32, ()> as StaticLookup>::Source> =
			targets.iter().map(|add| (*add).clone().into()).collect();

		let mut expected = StakeCall::Nominate(nominated).encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendAssetHubEncoder as StakeEncodeCall<()>>::encode_call(
				(),
				xcm_primitives::AvailableStakeCalls::Nominate(targets.clone())
			)
			.unwrap(),
			expected_encoded.clone()
		);

		sp_io::TestExternalities::default().execute_with(|| {
			pallet_xcm_transactor::ChainIndicesMap::<moonbase_runtime::Runtime>::insert(
				moonbase_runtime::xcm_config::Transactors::AssetHub,
				pallet_xcm_transactor::chain_indices::ChainIndices::AssetHub(
					WESTEND_ASSETHUB_INDICES,
				),
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<moonbase_runtime::Runtime> as StakeEncodeCall<
					moonbase_runtime::xcm_config::Transactors,
				>>::encode_call(
					moonbase_runtime::xcm_config::Transactors::AssetHub,
					xcm_primitives::AvailableStakeCalls::Nominate(targets)
				)
				.unwrap(),
				expected_encoded
			);
		});
	}

	#[test]
	fn test_stake_chill() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		expected_encoded.push(80u8);

		let mut expected = StakeCall::Chill.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendAssetHubEncoder as StakeEncodeCall<()>>::encode_call(
				(),
				xcm_primitives::AvailableStakeCalls::Chill
			)
			.unwrap(),
			expected_encoded.clone()
		);

		sp_io::TestExternalities::default().execute_with(|| {
			pallet_xcm_transactor::ChainIndicesMap::<moonbase_runtime::Runtime>::insert(
				moonbase_runtime::xcm_config::Transactors::AssetHub,
				pallet_xcm_transactor::chain_indices::ChainIndices::AssetHub(
					WESTEND_ASSETHUB_INDICES,
				),
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<moonbase_runtime::Runtime> as StakeEncodeCall<
					moonbase_runtime::xcm_config::Transactors,
				>>::encode_call(
					moonbase_runtime::xcm_config::Transactors::AssetHub,
					xcm_primitives::AvailableStakeCalls::Chill
				)
				.unwrap(),
				expected_encoded
			);
		});
	}

	#[test]
	fn test_set_payee() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		expected_encoded.push(80u8);

		let controller: AccountId32 = [1u8; 32].into();
		let mut expected = StakeCall::SetPayee(pallet_staking::RewardDestination::Account(
			controller.clone(),
		))
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendAssetHubEncoder as StakeEncodeCall<()>>::encode_call(
				(),
				xcm_primitives::AvailableStakeCalls::SetPayee(
					pallet_staking::RewardDestination::Account(controller.clone()).into()
				)
			)
			.unwrap(),
			expected_encoded.clone()
		);

		sp_io::TestExternalities::default().execute_with(|| {
			pallet_xcm_transactor::ChainIndicesMap::<moonbase_runtime::Runtime>::insert(
				moonbase_runtime::xcm_config::Transactors::AssetHub,
				pallet_xcm_transactor::chain_indices::ChainIndices::AssetHub(
					WESTEND_ASSETHUB_INDICES,
				),
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<moonbase_runtime::Runtime> as StakeEncodeCall<
					moonbase_runtime::xcm_config::Transactors,
				>>::encode_call(
					moonbase_runtime::xcm_config::Transactors::AssetHub,
					xcm_primitives::AvailableStakeCalls::SetPayee(
						pallet_staking::RewardDestination::Account(controller).into()
					)
				)
				.unwrap(),
				expected_encoded
			);
		});
	}

	#[test]
	fn test_set_controller() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		expected_encoded.push(80u8);

		let mut expected = StakeCall::SetController.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendAssetHubEncoder as StakeEncodeCall<()>>::encode_call(
				(),
				xcm_primitives::AvailableStakeCalls::SetController
			)
			.unwrap(),
			expected_encoded.clone()
		);

		sp_io::TestExternalities::default().execute_with(|| {
			pallet_xcm_transactor::ChainIndicesMap::<moonbase_runtime::Runtime>::insert(
				moonbase_runtime::xcm_config::Transactors::AssetHub,
				pallet_xcm_transactor::chain_indices::ChainIndices::AssetHub(
					WESTEND_ASSETHUB_INDICES,
				),
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<moonbase_runtime::Runtime> as StakeEncodeCall<
					moonbase_runtime::xcm_config::Transactors,
				>>::encode_call(
					moonbase_runtime::xcm_config::Transactors::AssetHub,
					xcm_primitives::AvailableStakeCalls::SetController
				)
				.unwrap(),
				expected_encoded
			);
		});
	}

	#[test]
	fn test_rebond() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		expected_encoded.push(80u8);

		let mut expected = StakeCall::Rebond(100u32.into()).encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendAssetHubEncoder as StakeEncodeCall<()>>::encode_call(
				(),
				xcm_primitives::AvailableStakeCalls::Rebond(100u32.into())
			)
			.unwrap(),
			expected_encoded.clone()
		);

		sp_io::TestExternalities::default().execute_with(|| {
			pallet_xcm_transactor::ChainIndicesMap::<moonbase_runtime::Runtime>::insert(
				moonbase_runtime::xcm_config::Transactors::AssetHub,
				pallet_xcm_transactor::chain_indices::ChainIndices::AssetHub(
					WESTEND_ASSETHUB_INDICES,
				),
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<moonbase_runtime::Runtime> as StakeEncodeCall<
					moonbase_runtime::xcm_config::Transactors,
				>>::encode_call(
					moonbase_runtime::xcm_config::Transactors::AssetHub,
					xcm_primitives::AvailableStakeCalls::Rebond(100u32.into())
				)
				.unwrap(),
				expected_encoded
			);
		});
	}
}
