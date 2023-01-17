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

// We want to avoid including the rococo-runtime here.
// TODO: whenever a conclusion is taken from https://github.com/paritytech/substrate/issues/8158

use cumulus_primitives_core::{relay_chain::v2::HrmpChannelId, ParaId};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::{AccountIdLookup, StaticLookup};
use sp_runtime::AccountId32;
use sp_std::vec::Vec;

#[derive(Encode, Decode)]
pub enum RelayCall {
	#[codec(index = 6u8)]
	Stake(StakeCall),
	#[codec(index = 24u8)]
	// the index should match the position of the module in `construct_runtime!`
	Utility(UtilityCall),
	#[codec(index = 60u8)]
	// the index should match the position of the module in `construct_runtime!`
	Hrmp(HrmpCall),
}

// Utility call encoding, needed for xcm transactor pallet
#[derive(Encode, Decode)]
pub enum UtilityCall {
	#[codec(index = 1u8)]
	AsDerivative(u16),
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

pub struct KusamaEncoder;

impl xcm_primitives::UtilityEncodeCall for KusamaEncoder {
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

impl xcm_primitives::HrmpEncodeCall for KusamaEncoder {
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

impl pallet_evm_precompile_relay_encoder::StakeEncodeCall for KusamaEncoder {
	fn encode_call(call: pallet_evm_precompile_relay_encoder::AvailableStakeCalls) -> Vec<u8> {
		match call {
			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Bond(a, b, c) => {
				RelayCall::Stake(StakeCall::Bond(a.into(), b, c)).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::BondExtra(a) => {
				RelayCall::Stake(StakeCall::BondExtra(a)).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Unbond(a) => {
				RelayCall::Stake(StakeCall::Unbond(a)).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::WithdrawUnbonded(a) => {
				RelayCall::Stake(StakeCall::WithdrawUnbonded(a)).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Validate(a) => {
				RelayCall::Stake(StakeCall::Validate(a)).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Chill => {
				RelayCall::Stake(StakeCall::Chill).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::SetPayee(a) => {
				RelayCall::Stake(StakeCall::SetPayee(a.into())).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::SetController(a) => {
				RelayCall::Stake(StakeCall::SetController(a.into())).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Rebond(a) => {
				RelayCall::Stake(StakeCall::Rebond(a.into())).encode()
			}

			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Nominate(a) => {
				let nominated: Vec<<AccountIdLookup<AccountId32, ()> as StaticLookup>::Source> =
					a.iter().map(|add| (*add).clone().into()).collect();

				RelayCall::Stake(StakeCall::Nominate(nominated)).encode()
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::kusama::KusamaEncoder;
	use frame_support::traits::PalletInfo;
	use pallet_evm_precompile_relay_encoder::StakeEncodeCall;
	use sp_runtime::Perbill;

	#[test]
	fn test_as_derivative() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Utility,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_utility::Call::<kusama_runtime::Runtime>::as_derivative {
			index: 1,
			call: kusama_runtime::RuntimeCall::Staking(pallet_staking::Call::<
				kusama_runtime::Runtime,
			>::chill {})
			.into(),
		}
		.encode();
		expected_encoded.append(&mut expected);

		let call_bytes = <KusamaEncoder as StakeEncodeCall>::encode_call(
			pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Chill,
		);

		expected_encoded.append(&mut expected);

		assert_eq!(
			xcm_primitives::UtilityEncodeCall::encode_call(
				KusamaEncoder,
				xcm_primitives::UtilityAvailableCalls::AsDerivative(1, call_bytes)
			),
			expected_encoded
		);
	}

	#[test]
	fn test_stake_bond() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let relay_account: AccountId32 = [1u8; 32].into();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<kusama_runtime::Runtime>::bond {
			controller: relay_account.clone().into(),
			value: 100u32.into(),
			payee: pallet_staking::RewardDestination::Controller,
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Bond(
					relay_account.into(),
					100u32.into(),
					pallet_staking::RewardDestination::Controller
				)
			),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_bond_extra() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<kusama_runtime::Runtime>::bond_extra {
			max_additional: 100u32.into(),
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				pallet_evm_precompile_relay_encoder::AvailableStakeCalls::BondExtra(100u32.into(),)
			),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_unbond() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<kusama_runtime::Runtime>::unbond {
			value: 100u32.into(),
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Unbond(100u32.into(),)
			),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_withdraw_unbonded() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<kusama_runtime::Runtime>::withdraw_unbonded {
			num_slashing_spans: 100u32,
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				pallet_evm_precompile_relay_encoder::AvailableStakeCalls::WithdrawUnbonded(100u32,)
			),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_validate() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let validator_prefs = pallet_staking::ValidatorPrefs {
			commission: Perbill::from_percent(5),
			blocked: true,
		};

		let mut expected = pallet_staking::Call::<kusama_runtime::Runtime>::validate {
			prefs: validator_prefs.clone(),
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Validate(validator_prefs)
			),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_nominate() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let relay_account: AccountId32 = [1u8; 32].into();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<kusama_runtime::Runtime>::nominate {
			targets: vec![relay_account.clone().into()],
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Nominate(vec![
					relay_account.into()
				])
			),
			expected_encoded
		);
	}
	#[test]
	fn test_stake_chill() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<kusama_runtime::Runtime>::chill {}.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Chill
			),
			expected_encoded
		);
	}

	#[test]
	fn test_set_payee() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<kusama_runtime::Runtime>::set_payee {
			payee: pallet_staking::RewardDestination::Controller,
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				pallet_evm_precompile_relay_encoder::AvailableStakeCalls::SetPayee(
					pallet_staking::RewardDestination::Controller
				)
			),
			expected_encoded
		);
	}

	#[test]
	fn test_set_controller() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let relay_account: AccountId32 = [1u8; 32].into();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<kusama_runtime::Runtime>::set_controller {
			controller: relay_account.clone().into(),
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				pallet_evm_precompile_relay_encoder::AvailableStakeCalls::SetController(
					relay_account.clone().into()
				)
			),
			expected_encoded
		);
	}
	#[test]
	fn test_rebond() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<kusama_runtime::Runtime>::rebond {
			value: 100u32.into(),
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as StakeEncodeCall>::encode_call(
				pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Rebond(100u32.into())
			),
			expected_encoded
		);
	}

	#[test]
	fn test_hrmp_init() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Hrmp,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = polkadot_runtime_parachains::hrmp::Call::<
			kusama_runtime::Runtime
		>::hrmp_init_open_channel {
			recipient: 1000u32.into(),
			proposed_max_capacity: 100u32,
			proposed_max_message_size: 100u32,
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as xcm_primitives::HrmpEncodeCall>::hrmp_encode_call(
				xcm_primitives::HrmpAvailableCalls::InitOpenChannel(
					1000u32.into(),
					100u32.into(),
					100u32.into()
				)
			),
			Ok(expected_encoded)
		);
	}

	#[test]
	fn test_hrmp_accept() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Hrmp,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = polkadot_runtime_parachains::hrmp::Call::<
			kusama_runtime::Runtime
		>::hrmp_accept_open_channel {
			sender: 1000u32.into()
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as xcm_primitives::HrmpEncodeCall>::hrmp_encode_call(
				xcm_primitives::HrmpAvailableCalls::AcceptOpenChannel(1000u32.into(),)
			),
			Ok(expected_encoded)
		);
	}

	#[test]
	fn test_hrmp_close() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			kusama_runtime::Hrmp,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = polkadot_runtime_parachains::hrmp::Call::<
			kusama_runtime::Runtime
		>::hrmp_close_channel {
			channel_id: HrmpChannelId {
				sender: 1000u32.into(),
				recipient: 1001u32.into()
			}
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<KusamaEncoder as xcm_primitives::HrmpEncodeCall>::hrmp_encode_call(
				xcm_primitives::HrmpAvailableCalls::CloseChannel(HrmpChannelId {
					sender: 1000u32.into(),
					recipient: 1001u32.into()
				})
			),
			Ok(expected_encoded)
		);
	}
}
