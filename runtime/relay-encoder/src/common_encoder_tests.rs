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

use crate::*;
use cumulus_primitives_core::relay_chain::v2::HrmpChannelId;
use frame_support::traits::PalletInfo;
use pallet_xcm_transactor::traits::*;
use parity_scale_codec::Encode;
use xcm_primitives::UtilityEncodeCall;

#[test]
fn test_westend_hrmp_close_eq() {
	sp_io::TestExternalities::default().execute_with(|| {
        // insert storage item as per migration
        pallet_xcm_transactor::RelayIndices::<moonbase_runtime::Runtime>::put(WESTEND_RELAY_INDICES);
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
            <pallet_xcm_transactor::Pallet::<moonbase_runtime::Runtime> as xcm_primitives::HrmpEncodeCall>::hrmp_encode_call(
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
fn test_kusama_as_derivative_eq() {
	sp_io::TestExternalities::default().execute_with(|| {
        let mut expected_encoded: Vec<u8> = Vec::new();
        let index = <kusama_runtime::Runtime as frame_system::Config>::PalletInfo::index::<
            kusama_runtime::Utility,
        >()
        .unwrap() as u8;
        expected_encoded.push(index);
        // insert storage item as per migration
        // TODO: replace with migration running
        pallet_xcm_transactor::RelayIndices::<moonriver_runtime::Runtime>::put(KUSAMA_RELAY_INDICES);

        let mut expected = pallet_utility::Call::<kusama_runtime::Runtime>::as_derivative {
            index: 1,
            call: kusama_runtime::RuntimeCall::Staking(pallet_staking::Call::<
                kusama_runtime::Runtime,
            >::chill {})
            .into(),
        }
        .encode();

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
