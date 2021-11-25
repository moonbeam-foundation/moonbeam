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
	events, evm_test_context, precompile_address, CurrencyId, ExtBuilder, Precompiles,
	TestAccount::*,
};
use crate::{Action, PrecompileOutput};
use fp_evm::Context;
use num_enum::TryFromPrimitive;
use orml_xtokens::Event as XtokensEvent;
use pallet_evm::{ExitSucceed, PrecompileSet};
use precompile_utils::{error, Address, EvmDataWriter};
use sha3::{Digest, Keccak256};
use sp_core::U256;
use xcm::v1::{AssetId, Fungibility, Junction, Junctions, MultiAsset, MultiLocation, NetworkId};

#[test]
fn test_selector_enum() {
	let mut buffer = [0u8; 4];
	buffer.copy_from_slice(
		&Keccak256::digest(b"transfer(address,uint256,(uint8,bytes[]),uint64)")[0..4],
	);
	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::Transfer,
	);

	buffer.copy_from_slice(
		&Keccak256::digest(b"transfer_multiasset((uint8,bytes[]),uint256,(uint8,bytes[]),uint64)")
			[0..4],
	);
	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::TransferMultiAsset,
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
fn transfer_self_reserve_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let destination = MultiLocation::new(
				1,
				Junctions::X1(Junction::AccountId32 {
					network: NetworkId::Any,
					id: [1u8; 32],
				}),
			);
			assert_eq!(
				Precompiles::execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(SelfReserve.into()))
						.write(U256::from(500))
						.write(destination.clone())
						.write(U256::from(4000000))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);
			let expected: crate::mock::Event =
				XtokensEvent::Transferred(Alice, CurrencyId::SelfReserve, 500, destination).into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_to_reserve_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let destination = MultiLocation::new(
				1,
				Junctions::X1(Junction::AccountId32 {
					network: NetworkId::Any,
					id: [1u8; 32],
				}),
			);
			// We are transferring asset 0, which we have instructed to be the relay asset
			assert_eq!(
				Precompiles::execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(AssetId(0u128).into()))
						.write(U256::from(500))
						.write(destination.clone())
						.write(U256::from(4000000))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);
			let expected: crate::mock::Event =
				XtokensEvent::Transferred(Alice, CurrencyId::OtherReserve(0u128), 500, destination)
					.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_non_reserve_to_non_reserve_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let destination = MultiLocation::new(
				1,
				Junctions::X1(Junction::AccountId32 {
					network: NetworkId::Any,
					id: [1u8; 32],
				}),
			);

			// We are transferring asset 1, which corresponds to another parachain Id asset
			assert_eq!(
				Precompiles::execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(AssetId(1u128).into()))
						.write(U256::from(500))
						.write(destination.clone())
						.write(U256::from(4000000))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);
			let expected: crate::mock::Event =
				XtokensEvent::Transferred(Alice, CurrencyId::OtherReserve(1u128), 500, destination)
					.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_multi_asset_to_reserve_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let destination = MultiLocation::new(
				1,
				Junctions::X1(Junction::AccountId32 {
					network: NetworkId::Any,
					id: [1u8; 32],
				}),
			);

			let asset = MultiLocation::parent();

			assert_eq!(
				Precompiles::execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferMultiAsset)
						.write(asset.clone())
						.write(U256::from(500))
						.write(destination)
						.write(U256::from(4000000))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);
			let expected: crate::mock::Event = XtokensEvent::TransferredMultiAsset(
				Alice,
				MultiAsset {
					id: AssetId::Concrete(asset),
					fun: Fungibility::Fungible(500),
				},
				MultiLocation::parent(),
			)
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_multi_asset_self_reserve_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let destination = MultiLocation::new(
				1,
				Junctions::X1(Junction::AccountId32 {
					network: NetworkId::Any,
					id: [1u8; 32],
				}),
			);

			let self_reserve = crate::mock::SelfReserve::get();

			assert_eq!(
				Precompiles::execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferMultiAsset)
						.write(self_reserve.clone())
						.write(U256::from(500))
						.write(destination)
						.write(U256::from(4000000))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);
			let expected: crate::mock::Event = XtokensEvent::TransferredMultiAsset(
				Alice,
				MultiAsset {
					id: AssetId::Concrete(self_reserve),
					fun: Fungibility::Fungible(500),
				},
				MultiLocation::parent(),
			)
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_multi_asset_non_reserve_to_non_reserve() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let destination = MultiLocation::new(
				1,
				Junctions::X1(Junction::AccountId32 {
					network: NetworkId::Any,
					id: [1u8; 32],
				}),
			);

			let asset_location = MultiLocation::new(
				1,
				Junctions::X2(Junction::Parachain(2), Junction::GeneralIndex(5u128)),
			);

			assert_eq!(
				Precompiles::execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferMultiAsset)
						.write(asset_location.clone())
						.write(U256::from(500))
						.write(destination.clone())
						.write(U256::from(4000000))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);
			let expected: crate::mock::Event = XtokensEvent::TransferredMultiAsset(
				Alice,
				MultiAsset {
					id: AssetId::Concrete(asset_location),
					fun: Fungibility::Fungible(500),
				},
				MultiLocation::parent(),
			)
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}
