// Copyright 2019-2021 PureStake Inc.
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

impl relay_encoder_precompiles::StakeEncodeCall for WestendEncoder {
	fn encode_call(call: relay_encoder_precompiles::AvailableStakeCalls) -> Vec<u8> {
		match call {
			relay_encoder_precompiles::AvailableStakeCalls::Bond(a, b, c) => {
				RelayCall::Stake(StakeCall::Bond(a.into(), b, c)).encode()
			}

			relay_encoder_precompiles::AvailableStakeCalls::BondExtra(a) => {
				RelayCall::Stake(StakeCall::BondExtra(a)).encode()
			}

			relay_encoder_precompiles::AvailableStakeCalls::Unbond(a) => {
				RelayCall::Stake(StakeCall::Unbond(a)).encode()
			}

			relay_encoder_precompiles::AvailableStakeCalls::WithdrawUnbonded(a) => {
				RelayCall::Stake(StakeCall::WithdrawUnbonded(a)).encode()
			}

			relay_encoder_precompiles::AvailableStakeCalls::Validate(a) => {
				RelayCall::Stake(StakeCall::Validate(a)).encode()
			}

			relay_encoder_precompiles::AvailableStakeCalls::Chill => {
				RelayCall::Stake(StakeCall::Chill).encode()
			}

			relay_encoder_precompiles::AvailableStakeCalls::SetPayee(a) => {
				RelayCall::Stake(StakeCall::SetPayee(a.into())).encode()
			}

			relay_encoder_precompiles::AvailableStakeCalls::SetController(a) => {
				RelayCall::Stake(StakeCall::SetController(a.into())).encode()
			}

			relay_encoder_precompiles::AvailableStakeCalls::Rebond(a) => {
				RelayCall::Stake(StakeCall::Rebond(a.into())).encode()
			}

			relay_encoder_precompiles::AvailableStakeCalls::Nominate(a) => {
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
	use crate::westend::WestendEncoder;
	use frame_support::traits::PalletInfo;
	use relay_encoder_precompiles::StakeEncodeCall;
	use sp_runtime::Perbill;

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
			call: westend_runtime::Call::Staking(
				pallet_staking::Call::<westend_runtime::Runtime>::chill {},
			)
			.into(),
		}
		.encode();
		expected_encoded.append(&mut expected);

		let call_bytes = <WestendEncoder as StakeEncodeCall>::encode_call(
			relay_encoder_precompiles::AvailableStakeCalls::Chill,
		);

		expected_encoded.append(&mut expected);

		assert_eq!(
			xcm_primitives::UtilityEncodeCall::encode_call(
				WestendEncoder,
				xcm_primitives::UtilityAvailableCalls::AsDerivative(1, call_bytes)
			),
			expected_encoded
		);
	}

	#[test]
	fn test_stake_bond() {
		let mut expected_encoded: Vec<u8> = Vec::new();
		let relay_account: AccountId32 = [1u8; 32].into();

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<westend_runtime::Runtime>::bond {
			controller: relay_account.clone().into(),
			value: 100u32.into(),
			payee: pallet_staking::RewardDestination::Controller,
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as StakeEncodeCall>::encode_call(
				relay_encoder_precompiles::AvailableStakeCalls::Bond(
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
				relay_encoder_precompiles::AvailableStakeCalls::BondExtra(100u32.into(),)
			),
			expected_encoded
		);
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
				relay_encoder_precompiles::AvailableStakeCalls::Unbond(100u32.into(),)
			),
			expected_encoded
		);
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
				relay_encoder_precompiles::AvailableStakeCalls::WithdrawUnbonded(100u32,)
			),
			expected_encoded
		);
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
				relay_encoder_precompiles::AvailableStakeCalls::Validate(validator_prefs)
			),
			expected_encoded
		);
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
				relay_encoder_precompiles::AvailableStakeCalls::Nominate(
					vec![relay_account.into()]
				)
			),
			expected_encoded
		);
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
				relay_encoder_precompiles::AvailableStakeCalls::Chill
			),
			expected_encoded
		);
	}

	#[test]
	fn test_set_payee() {
		let mut expected_encoded: Vec<u8> = Vec::new();

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<westend_runtime::Runtime>::set_payee {
			payee: pallet_staking::RewardDestination::Controller,
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as StakeEncodeCall>::encode_call(
				relay_encoder_precompiles::AvailableStakeCalls::SetPayee(
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

		let index = <westend_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
			westend_runtime::Staking,
		>()
		.unwrap() as u8;
		expected_encoded.push(index);

		let mut expected = pallet_staking::Call::<westend_runtime::Runtime>::set_controller {
			controller: relay_account.clone().into(),
		}
		.encode();
		expected_encoded.append(&mut expected);

		assert_eq!(
			<WestendEncoder as StakeEncodeCall>::encode_call(
				relay_encoder_precompiles::AvailableStakeCalls::SetController(
					relay_account.clone().into()
				)
			),
			expected_encoded
		);
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
				relay_encoder_precompiles::AvailableStakeCalls::Rebond(100u32.into())
			),
			expected_encoded
		);
	}
}
