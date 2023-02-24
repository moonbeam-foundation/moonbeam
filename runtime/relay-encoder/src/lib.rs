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

//! Encoder for relay runtimes
#![cfg_attr(not(feature = "std"), no_std)]

pub mod kusama;
pub mod polkadot;
pub mod westend;

#[cfg(test)]
mod common_encoder_tests {
	#[test]
	fn test_common_encoder_as_derivative() {
		use frame_support::traits::PalletInfo;
		use moonbeam_runtime_common::relay_encoder::CommonEncoder;
		use pallet_evm_precompile_relay_encoder::StakeEncodeCall;
		use pallet_xcm_transactor::relay_indices::*;
		use parity_scale_codec::Encode;
		sp_io::TestExternalities::default().execute_with(|| {
			let mut expected_encoded: Vec<u8> = Vec::new();
			pallet_xcm_transactor::RelayPalletIndices::<moonriver_runtime::Runtime>::put(
				RelayIndices {
					pallets: PalletIndices {
						staking: 6u8,
						utility: 24u8,
						hrmp: 1u8,
					},
					calls: CallIndices {
						staking: StakingIndices {
							bond: 1u16,
							bond_extra: 1u16,
							unbond: 1u16,
							withdraw_unbonded: 1u16,
							validate: 1u16,
							nominate: 1u16,
							chill: 6u16,
							set_payee: 1u16,
							set_controller: 1u16,
							rebond: 1u16,
						},
						utility: UtilityIndices { as_derivative: 1u8 },
						hrmp: HrmpIndices {
							init_open_channel: 1u8,
							accept_open_channel: 1u8,
							close_channel: 1u8,
						},
					},
				},
			);
			// TODO: insert the encoding correctly as per existing code...
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

			let common_encoder =
				CommonEncoder(sp_std::marker::PhantomData::<moonriver_runtime::Runtime>::default());
			let call_bytes =
				<CommonEncoder<moonriver_runtime::Runtime> as StakeEncodeCall>::encode_call(
					pallet_evm_precompile_relay_encoder::AvailableStakeCalls::Chill,
				);

			expected_encoded.append(&mut expected);

			assert_eq!(
				xcm_primitives::UtilityEncodeCall::encode_call(
					common_encoder,
					xcm_primitives::UtilityAvailableCalls::AsDerivative(1, call_bytes)
				),
				expected_encoded
			);
		});
	}
}
