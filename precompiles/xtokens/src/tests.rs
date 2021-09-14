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

use crate::mock::{
	events, evm_test_context, precompile_address, roll_to, Call, CurrencyId, ExtBuilder, Origin,
	Precompiles, TestAccount::Alice, TestAccount::Bob, TestAccount::Charlie, Xtokens,
};
use crate::{Action, PrecompileOutput};
use frame_support::{assert_ok, dispatch::Dispatchable};
use num_enum::TryFromPrimitive;
use orml_xtokens::{Call as XtokensCall, Event as XtokensEvent};
use pallet_evm::{Call as EvmCall, ExitSucceed, PrecompileSet};
use precompile_utils::{error, Address, Bytes, EvmDataWriter};
use sha3::{Digest, Keccak256};
use sp_core::{H160, U256};
use xcm::v0::{
	Junction::{AccountId32, PalletInstance, Parachain, Parent},
	NetworkId,
};
use xcm_simulator::MultiLocation;

#[test]
fn test_selector_enum() {
	let mut buffer = [0u8; 4];
	buffer.copy_from_slice(&Keccak256::digest(b"transfer(address, u256, bytes, u64)")[0..4]);
	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::Transfer,
	);
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
		let bogus_selector = vec![1u8, 2u8, 3u8];

		// Expected result is an error stating there are too few bytes
		let expected_result = Some(Err(error("tried to parse selector out of bounds")));

		assert_eq!(
			Precompiles::execute(
				precompile_address(),
				&bogus_selector,
				None,
				&evm_test_context(),
			),
			expected_result
		);
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		let bogus_selector = vec![1u8, 2u8, 3u8, 4u8];

		// Expected result is an error stating there are such a selector does not exist
		let expected_result = Some(Err(error("unknown selector")));

		assert_eq!(
			Precompiles::execute(
				precompile_address(),
				&bogus_selector,
				None,
				&evm_test_context(),
			),
			expected_result
		);
	});
}

#[test]
fn transfer_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"transfer(address, u256, bytes, u64)")[0..4];
			// AccountKey32 create
			let mut key_32_junction = vec![2u8];
			key_32_junction.extend_from_slice(&[0u8; 32]);
			let input = EvmDataWriter::new()
				.write_raw_bytes(selector)
				.write(Address(H160::from(Alice)))
				.write(50u128)
				.write(vec![Bytes(vec![0u8]), Bytes(key_32_junction)])
				.write(1u64)
				.build();

			// Make sure the call goes through successfully
			assert_ok!(Call::Evm(EvmCall::call(
				Alice.into(),
				precompile_address(),
				input,
				U256::zero(), // No value sent in EVM
				u64::max_value(),
				0.into(),
				None, // Use the next nonce
			))
			.dispatch(Origin::root()));

			let expected: crate::mock::Event = XtokensEvent::Transferred(
				Alice,
				CurrencyId::SelfReserve,
				50u32.into(),
				MultiLocation::X2(
					Parent,
					AccountId32 {
						network: NetworkId::Any,
						id: [
							0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
							0, 0, 0, 0, 0, 0, 0, 0,
						],
					},
				),
			)
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}
