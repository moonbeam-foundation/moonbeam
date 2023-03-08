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
		use pallet_xcm_transactor::{relay_indices::*, traits::*};
		use parity_scale_codec::Encode;
		use xcm_primitives::UtilityEncodeCall;
		sp_io::TestExternalities::default().execute_with(|| {
			let mut expected_encoded: Vec<u8> = Vec::new();
			pallet_xcm_transactor::RelayIndices::<moonriver_runtime::Runtime>::put(
				RelayChainIndices {
					pallets: PalletIndices {
						staking: 6u8,
						utility: 24u8,
						hrmp: 1u8,
					},
					calls: CallIndices {
						staking: StakingIndices {
							bond: 1u8,
							bond_extra: 1u8,
							unbond: 1u8,
							withdraw_unbonded: 1u8,
							validate: 1u8,
							nominate: 1u8,
							chill: 6u8,
							set_payee: 1u8,
							set_controller: 1u8,
							rebond: 1u8,
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

			let call_bytes =
				<pallet_xcm_transactor::Pallet::<moonriver_runtime::Runtime> as StakeEncodeCall>::encode_call(
					AvailableStakeCalls::Chill,
				);

			expected_encoded.append(&mut expected);

			assert_eq!(
				<pallet_xcm_transactor::Pallet::<moonriver_runtime::Runtime> as UtilityEncodeCall>::encode_call(
					pallet_xcm_transactor::Pallet(sp_std::marker::PhantomData::<moonriver_runtime::Runtime>::default()),
					xcm_primitives::UtilityAvailableCalls::AsDerivative(1, call_bytes)
				),
				expected_encoded
			);
		});
	}
}
