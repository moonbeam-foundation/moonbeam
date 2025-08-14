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

// We want to avoid including the rococo-runtime here.
// TODO: whenever a conclusion is taken from https://github.com/paritytech/substrate/issues/8158

use cumulus_primitives_core::{relay_chain::HrmpChannelId, ParaId};
use pallet_xcm_transactor::relay_indices::RelayChainIndices;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::{AccountIdLookup, StaticLookup};
use sp_runtime::AccountId32;
use sp_std::vec::Vec;

#[derive(Encode, Decode)]
pub enum RelayCall {
	#[codec(index = 16u8)]
	// the index should match the position of the module in `construct_runtime!`
	Utility(UtilityCall),

	#[codec(index = 6u8)]
	Stake(StakeCall),
	#[codec(index = 51u8)]
	// the index should match the position of the module in `construct_runtime!`
	Hrmp(HrmpCall),
}

#[derive(Encode, Decode)]
pub enum StakeCall {
	#[codec(index = 0u16)]
	// the index should match the position of the dispatchable in the target pallet
	Bond(
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
	SetController,
	#[codec(index = 19u16)]
	Rebond(#[codec(compact)] cumulus_primitives_core::relay_chain::Balance),
}

// Utility call encoding, needed for xcm transactor pallet
#[derive(Encode, Decode)]
pub enum UtilityCall {
	#[codec(index = 1u8)]
	AsDerivative(u16),
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
	#[codec(index = 6u8)]
	CancelOpenRequest(HrmpChannelId, u32),
}

pub struct WestendEncoder;

impl xcm_primitives::UtilityEncodeCall for WestendEncoder {
	fn encode_call(self, call: xcm_primitives::UtilityAvailableCalls) -> Vec<u8> {
		match call {
			xcm_primitives::UtilityAvailableCalls::AsDerivative(a, b) => {
				let mut call = RelayCall::Utility(UtilityCall::AsDerivative(a.clone())).encode();
				// If we encode directly we inject the call length,
				// so we just append the inner call after encoding the outer
				call.append(&mut b.clone());
				call
			}
		}
	}
}

impl xcm_primitives::HrmpEncodeCall for WestendEncoder {
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
			xcm_primitives::HrmpAvailableCalls::CancelOpenRequest(a, b) => {
				Ok(RelayCall::Hrmp(HrmpCall::CancelOpenRequest(a.clone(), b.clone())).encode())
			}
		}
	}
}
impl xcm_primitives::StakeEncodeCall for WestendEncoder {
	fn encode_call(call: xcm_primitives::AvailableStakeCalls) -> Vec<u8> {
		match call {
			xcm_primitives::AvailableStakeCalls::Bond(b, c) => {
				RelayCall::Stake(StakeCall::Bond(b, c)).encode()
			}

			xcm_primitives::AvailableStakeCalls::BondExtra(a) => {
				RelayCall::Stake(StakeCall::BondExtra(a)).encode()
			}

			xcm_primitives::AvailableStakeCalls::Unbond(a) => {
				RelayCall::Stake(StakeCall::Unbond(a)).encode()
			}

			xcm_primitives::AvailableStakeCalls::WithdrawUnbonded(a) => {
				RelayCall::Stake(StakeCall::WithdrawUnbonded(a)).encode()
			}

			xcm_primitives::AvailableStakeCalls::Validate(a) => {
				RelayCall::Stake(StakeCall::Validate(a)).encode()
			}

			xcm_primitives::AvailableStakeCalls::Chill => {
				RelayCall::Stake(StakeCall::Chill).encode()
			}

			xcm_primitives::AvailableStakeCalls::SetPayee(a) => {
				RelayCall::Stake(StakeCall::SetPayee(a.into())).encode()
			}

			xcm_primitives::AvailableStakeCalls::SetController => {
				RelayCall::Stake(StakeCall::SetController).encode()
			}

			xcm_primitives::AvailableStakeCalls::Rebond(a) => {
				RelayCall::Stake(StakeCall::Rebond(a.into())).encode()
			}

			xcm_primitives::AvailableStakeCalls::Nominate(a) => {
				let nominated: Vec<<AccountIdLookup<AccountId32, ()> as StaticLookup>::Source> =
					a.iter().map(|add| (*add).clone().into()).collect();

				RelayCall::Stake(StakeCall::Nominate(nominated)).encode()
			}
		}
	}
}

/// Westend pallet and extrinsic indices
pub const WESTEND_RELAY_INDICES: RelayChainIndices = RelayChainIndices {
	staking: 6u8,
	utility: 16u8,
	hrmp: 51u8,
	bond: 0u8,
	bond_extra: 1u8,
	unbond: 2u8,
	withdraw_unbonded: 3u8,
	validate: 4u8,
	nominate: 5u8,
	chill: 6u8,
	set_payee: 7u8,
	set_controller: 8u8,
	rebond: 19u8,
	as_derivative: 1u8,
	init_open_channel: 0u8,
	accept_open_channel: 1u8,
	close_channel: 2u8,
	cancel_open_request: 6u8,
};

#[cfg(test)]
mod tests {
	use super::*;
	use crate::westend::WestendEncoder;
	use frame_support::traits::PalletInfo;
	use sp_runtime::Perbill;
	use xcm_primitives::{StakeEncodeCall, UtilityEncodeCall};

	#[test]
	fn test_as_derivative() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Utility,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_utility::Call::<westend_runtime::Runtime>::as_derivative {
			index: 1,
			call: westend_runtime::RuntimeCall::Staking(pallet_staking::Call::<
				westend_runtime::Runtime,
			>::chill {})
			.into(),
		}
		.encode();
		expected_encoded.append(&mut expected);

		let call_bytes = <WestendEncoder as StakeEncodeCall>::encode_call(
			xcm_primitives::AvailableStakeCalls::Chill,
		);

		expected_encoded.append(&mut expected);

		assert_eq!(
			xcm_primitives::UtilityEncodeCall::encode_call(
				WestendEncoder,
				xcm_primitives::UtilityAvailableCalls::AsDerivative(1, call_bytes.clone())
			),
			expected_encoded.clone()
		);
		sp_io::TestExternalities::default().execute_with(|| {
			// Pallet-xcm-transactor default encoder returns same result
			// insert storage item as per migration to set the storage item
			pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(
				WESTEND_RELAY_INDICES,
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<
					moonbase_runtime::Runtime> as UtilityEncodeCall
				>::encode_call(
					pallet_xcm_transactor::Pallet(
						sp_std::marker::PhantomData::<moonbase_runtime::Runtime>::default()
					),
					xcm_primitives::UtilityAvailableCalls::AsDerivative(1, call_bytes)
				),
				expected_encoded
			);
		});
	}

	#[test]
	fn test_stake_bond() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let controller: AccountId32 = [1u8; 32].into();

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<westend_runtime::Runtime>::bond {
			value: 100u32.into(),
			payee: pallet_staking::RewardDestination::Account(controller.clone()),
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as StakeEncodeCall>::encode_call(
				xcm_primitives::AvailableStakeCalls::Bond(
					100u32.into(),
					pallet_staking::RewardDestination::Account(controller.clone()),
				)
			),
			expected_encoded.clone()
		);
		sp_io::TestExternalities::default().execute_with(|| {
			// Pallet-xcm-transactor default encoder returns same result
			// insert storage item as per migration to set the storage item
			pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(
				WESTEND_RELAY_INDICES,
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<
					moonbase_runtime::Runtime> as StakeEncodeCall
				>::encode_call(
					xcm_primitives::AvailableStakeCalls::Bond(
						100u32.into(),
						pallet_staking::RewardDestination::Account(controller)
					)
				),
				expected_encoded
			);
		});
	}
	#[test]
	fn test_stake_bond_extra() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<westend_runtime::Runtime>::bond_extra {
			max_additional: 100u32.into(),
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as StakeEncodeCall>::encode_call(
				xcm_primitives::AvailableStakeCalls::BondExtra(100u32.into(),)
			),
			expected_encoded.clone()
		);
		sp_io::TestExternalities::default().execute_with(|| {
			// Pallet-xcm-transactor default encoder returns same result
			// insert storage item as per migration to set the storage item
			pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(
				WESTEND_RELAY_INDICES,
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<
					moonbase_runtime::Runtime> as StakeEncodeCall
				>::encode_call(
					xcm_primitives::AvailableStakeCalls::BondExtra(100u32.into(),)
				),
				expected_encoded
			);
		});
	}
	#[test]
	fn test_stake_unbond() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<westend_runtime::Runtime>::unbond {
			value: 100u32.into(),
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as StakeEncodeCall>::encode_call(
				xcm_primitives::AvailableStakeCalls::Unbond(100u32.into(),)
			),
			expected_encoded.clone()
		);
		sp_io::TestExternalities::default().execute_with(|| {
			// Pallet-xcm-transactor default encoder returns same result
			// insert storage item as per migration to set the storage item
			pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(
				WESTEND_RELAY_INDICES,
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<
					moonbase_runtime::Runtime> as StakeEncodeCall
				>::encode_call(
					xcm_primitives::AvailableStakeCalls::Unbond(100u32.into(),)
				),
				expected_encoded
			);
		});
	}
	#[test]
	fn test_stake_withdraw_unbonded() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<westend_runtime::Runtime>::withdraw_unbonded {
			num_slashing_spans: 100u32,
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as StakeEncodeCall>::encode_call(
				xcm_primitives::AvailableStakeCalls::WithdrawUnbonded(100u32,)
			),
			expected_encoded.clone()
		);
		sp_io::TestExternalities::default().execute_with(|| {
			// Pallet-xcm-transactor default encoder returns same result
			// insert storage item as per migration to set the storage item
			pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(
				WESTEND_RELAY_INDICES,
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<
					moonbase_runtime::Runtime> as StakeEncodeCall
				>::encode_call(
					xcm_primitives::AvailableStakeCalls::WithdrawUnbonded(100u32,)
				),
				expected_encoded
			);
		});
	}
	#[test]
	fn test_stake_validate() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let validator_prefs = pallet_staking::ValidatorPrefs {
			commission: Perbill::from_percent(5),
			blocked: true,
		};

		let mut expected = pallet_staking::Call::<westend_runtime::Runtime>::validate {
			prefs: validator_prefs.clone(),
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as StakeEncodeCall>::encode_call(
				xcm_primitives::AvailableStakeCalls::Validate(validator_prefs.clone())
			),
			expected_encoded.clone()
		);
		sp_io::TestExternalities::default().execute_with(|| {
			// Pallet-xcm-transactor default encoder returns same result
			// insert storage item as per migration to set the storage item
			pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(
				WESTEND_RELAY_INDICES,
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<
					moonbase_runtime::Runtime> as StakeEncodeCall
				>::encode_call(
					xcm_primitives::AvailableStakeCalls::Validate(validator_prefs)
				),
				expected_encoded
			);
		});
	}
	#[test]
	fn test_stake_nominate() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let relay_account: AccountId32 = [1u8; 32].into();

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<westend_runtime::Runtime>::nominate {
			targets: vec![relay_account.clone().into()],
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as StakeEncodeCall>::encode_call(
				xcm_primitives::AvailableStakeCalls::Nominate(vec![relay_account.clone().into()])
			),
			expected_encoded.clone()
		);
		sp_io::TestExternalities::default().execute_with(|| {
			// Pallet-xcm-transactor default encoder returns same result
			// insert storage item as per migration to set the storage item
			pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(
				WESTEND_RELAY_INDICES,
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<
					moonbase_runtime::Runtime> as StakeEncodeCall
				>::encode_call(
					xcm_primitives::AvailableStakeCalls::Nominate(vec![
						relay_account.into()
					])
				),
				expected_encoded
			);
		});
	}
	#[test]
	fn test_stake_chill() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<westend_runtime::Runtime>::chill {}.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as StakeEncodeCall>::encode_call(
				xcm_primitives::AvailableStakeCalls::Chill
			),
			expected_encoded.clone()
		);
		sp_io::TestExternalities::default().execute_with(|| {
			// Pallet-xcm-transactor default encoder returns same result
			// insert storage item as per migration to set the storage item
			pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(
				WESTEND_RELAY_INDICES,
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<
					moonbase_runtime::Runtime> as StakeEncodeCall
				>::encode_call(
					xcm_primitives::AvailableStakeCalls::Chill
				),
				expected_encoded
			);
		});
	}

	#[test]
	fn test_set_payee() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let controller: AccountId32 = [1u8; 32].into();

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<westend_runtime::Runtime>::set_payee {
			payee: pallet_staking::RewardDestination::Account(controller.clone()),
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as StakeEncodeCall>::encode_call(
				xcm_primitives::AvailableStakeCalls::SetPayee(
					pallet_staking::RewardDestination::Account(controller.clone())
				)
			),
			expected_encoded.clone()
		);
		sp_io::TestExternalities::default().execute_with(|| {
			// Pallet-xcm-transactor default encoder returns same result
			// insert storage item as per migration to set the storage item
			pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(
				WESTEND_RELAY_INDICES,
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<
					moonbase_runtime::Runtime> as StakeEncodeCall
				>::encode_call(
					xcm_primitives::AvailableStakeCalls::SetPayee(
						pallet_staking::RewardDestination::Account(controller)
					)
				),
				expected_encoded
			);
		});
	}

	#[test]
	fn test_set_controller() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected =
			pallet_staking::Call::<westend_runtime::Runtime>::set_controller {}.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as StakeEncodeCall>::encode_call(
				xcm_primitives::AvailableStakeCalls::SetController
			),
			expected_encoded.clone()
		);
		sp_io::TestExternalities::default().execute_with(|| {
			// Pallet-xcm-transactor default encoder returns same result
			// insert storage item as per migration to set the storage item
			pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(
				WESTEND_RELAY_INDICES,
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<
					moonbase_runtime::Runtime> as StakeEncodeCall
				>::encode_call(
					xcm_primitives::AvailableStakeCalls::SetController
				),
				expected_encoded
			);
		});
	}
	#[test]
	fn test_rebond() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<westend_runtime::Runtime>::rebond {
			value: 100u32.into(),
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as StakeEncodeCall>::encode_call(
				xcm_primitives::AvailableStakeCalls::Rebond(100u32.into())
			),
			expected_encoded.clone()
		);
		sp_io::TestExternalities::default().execute_with(|| {
			// Pallet-xcm-transactor default encoder returns same result
			// insert storage item as per migration to set the storage item
			pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(
				WESTEND_RELAY_INDICES,
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<
					moonbase_runtime::Runtime> as StakeEncodeCall
				>::encode_call(
					xcm_primitives::AvailableStakeCalls::Rebond(100u32.into())
				),
				expected_encoded
			);
		});
	}

	#[test]
	fn test_hrmp_init() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Hrmp,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = polkadot_runtime_parachains::hrmp::Call::<
			westend_runtime::Runtime
		>::hrmp_init_open_channel {
			recipient: 1000u32.into(),
			proposed_max_capacity: 100u32,
			proposed_max_message_size: 100u32,
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as xcm_primitives::HrmpEncodeCall>::hrmp_encode_call(
				xcm_primitives::HrmpAvailableCalls::InitOpenChannel(
					1000u32.into(),
					100u32.into(),
					100u32.into()
				)
			),
			Ok(expected_encoded.clone())
		);
		sp_io::TestExternalities::default().execute_with(|| {
			// Pallet-xcm-transactor default encoder returns same result
			// insert storage item as per migration to set the storage item
			pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(
				WESTEND_RELAY_INDICES,
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<
					moonbase_runtime::Runtime> as xcm_primitives::HrmpEncodeCall
				>::hrmp_encode_call(
					xcm_primitives::HrmpAvailableCalls::InitOpenChannel(
						1000u32.into(),
						100u32.into(),
						100u32.into()
					)
				),
				Ok(expected_encoded)
			);
		});
	}

	#[test]
	fn test_hrmp_accept() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Hrmp,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = polkadot_runtime_parachains::hrmp::Call::<
			westend_runtime::Runtime
		>::hrmp_accept_open_channel {
			sender: 1000u32.into()
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as xcm_primitives::HrmpEncodeCall>::hrmp_encode_call(
				xcm_primitives::HrmpAvailableCalls::AcceptOpenChannel(1000u32.into(),)
			),
			Ok(expected_encoded.clone())
		);
		sp_io::TestExternalities::default().execute_with(|| {
			// Pallet-xcm-transactor default encoder returns same result
			// insert storage item as per migration to set the storage item
			pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(
				WESTEND_RELAY_INDICES,
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<
					moonbase_runtime::Runtime> as xcm_primitives::HrmpEncodeCall
				>::hrmp_encode_call(
					xcm_primitives::HrmpAvailableCalls::AcceptOpenChannel(1000u32.into(),)
				),
				Ok(expected_encoded)
			);
		});
	}

	#[test]
	fn test_hrmp_close() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Hrmp,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = polkadot_runtime_parachains::hrmp::Call::<
			westend_runtime::Runtime
		>::hrmp_close_channel {
			channel_id: HrmpChannelId {
				sender: 1000u32.into(),
				recipient: 1001u32.into()
			}
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as xcm_primitives::HrmpEncodeCall>::hrmp_encode_call(
				xcm_primitives::HrmpAvailableCalls::CloseChannel(HrmpChannelId {
					sender: 1000u32.into(),
					recipient: 1001u32.into()
				})
			),
			Ok(expected_encoded.clone())
		);
		sp_io::TestExternalities::default().execute_with(|| {
			// Pallet-xcm-transactor default encoder returns same result
			// insert storage item as per migration to set the storage item
			pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(
				WESTEND_RELAY_INDICES,
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<
					moonbase_runtime::Runtime> as xcm_primitives::HrmpEncodeCall
				>::hrmp_encode_call(
					xcm_primitives::HrmpAvailableCalls::CloseChannel(HrmpChannelId {
						sender: 1000u32.into(),
						recipient: 1001u32.into()
					})
				),
				Ok(expected_encoded)
			);
		});
	}

	#[test]
	fn test_hrmp_cancel() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Hrmp,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let channel_id = HrmpChannelId {
			sender: 1u32.into(),
			recipient: 1u32.into(),
		};
		let open_requests: u32 = 1;

		let mut expected = polkadot_runtime_parachains::hrmp::Call::<
			westend_runtime::Runtime
		>::hrmp_cancel_open_request {
			channel_id: channel_id.clone(),
			open_requests
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as xcm_primitives::HrmpEncodeCall>::hrmp_encode_call(
				xcm_primitives::HrmpAvailableCalls::CancelOpenRequest(
					channel_id.clone(),
					open_requests
				)
			),
			Ok(expected_encoded.clone())
		);
		sp_io::TestExternalities::default().execute_with(|| {
			// Pallet-xcm-transactor default encoder returns same result
			// insert storage item as per migration to set the storage item
			pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(
				WESTEND_RELAY_INDICES,
			);
			assert_eq!(
				<pallet_xcm_transactor::Pallet::<
					moonbase_runtime::Runtime> as xcm_primitives::HrmpEncodeCall
				>::hrmp_encode_call(
					xcm_primitives::HrmpAvailableCalls::CancelOpenRequest(
						channel_id.clone(),
						open_requests
					)
				),
				Ok(expected_encoded)
			);
		});
	}
}
