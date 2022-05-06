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

use std::assert_matches::assert_matches;

use crate::mock::{
	events, evm_test_context, precompile_address, CurrencyId, CurrencyIdToMultiLocation,
	ExtBuilder, PrecompilesValue, Runtime, TestAccount::*, TestPrecompiles,
};
use crate::{Action, Currency, EvmMultiAsset, PrecompileOutput};
use fp_evm::{Context, PrecompileFailure};
use num_enum::TryFromPrimitive;
use orml_xtokens::Event as XtokensEvent;
use pallet_evm::{ExitSucceed, PrecompileSet};
use precompile_utils::{Address, EvmDataWriter};
use sha3::{Digest, Keccak256};
use sp_core::U256;
use sp_runtime::traits::Convert;
use xcm::latest::{
	AssetId, Fungibility, Junction, Junctions, MultiAsset, MultiAssets, MultiLocation, NetworkId,
};

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

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

	buffer.copy_from_slice(
		&Keccak256::digest(b"transfer_with_fee(address,uint256,uint256,(uint8,bytes[]),uint64)")
			[0..4],
	);
	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::TransferWithFee,
	);

	buffer.copy_from_slice(
		&Keccak256::digest(
			b"transfer_multiasset_with_fee((uint8,bytes[]),uint256,uint256,(uint8,bytes[]),uint64)",
		)[0..4],
	);
	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::TransferMultiAssetWithFee,
	);
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
		let bogus_selector = vec![1u8, 2u8, 3u8];

		assert_matches!(
			precompiles().execute(
				precompile_address(),
				&bogus_selector,
				None,
				&evm_test_context(),
				false,
			),
			Some(Err(PrecompileFailure::Revert { output, ..}))
				if output == b"tried to parse selector out of bounds",
		);
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		let bogus_selector = vec![1u8, 2u8, 3u8, 4u8];

		assert_matches!(
			precompiles().execute(
				precompile_address(),
				&bogus_selector,
				None,
				&evm_test_context(),
				false,
			),
			Some(Err(PrecompileFailure::Revert { output, ..}))
				if output == b"unknown selector",
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
				precompiles().execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(SelfReserve.into()))
						.write(U256::from(500u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0u32),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);
			let expected_asset: MultiAsset = MultiAsset {
				id: AssetId::Concrete(
					CurrencyIdToMultiLocation::convert(CurrencyId::SelfReserve).unwrap(),
				),
				fun: Fungibility::Fungible(500),
			};
			let expected: crate::mock::Event = XtokensEvent::TransferredMultiAssets {
				sender: Alice,
				assets: vec![expected_asset.clone()].into(),
				fee: expected_asset,
				dest: destination,
			}
			.into();
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
				precompiles().execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(AssetId(0u128).into()))
						.write(U256::from(500u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0u32),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);
			let expected_asset: MultiAsset = MultiAsset {
				id: AssetId::Concrete(
					CurrencyIdToMultiLocation::convert(CurrencyId::OtherReserve(0u128)).unwrap(),
				),
				fun: Fungibility::Fungible(500),
			};
			let expected: crate::mock::Event = XtokensEvent::TransferredMultiAssets {
				sender: Alice,
				assets: vec![expected_asset.clone()].into(),
				fee: expected_asset,
				dest: destination,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_to_reserve_with_fee_works() {
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
			// Fees are not trully charged, so no worries
			assert_eq!(
				precompiles().execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferWithFee)
						.write(Address(AssetId(0u128).into()))
						.write(U256::from(500u64))
						.write(U256::from(50u64))
						.write(destination.clone())
						.write(U256::from(4000000u64))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0u32),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);

			let expected_asset: MultiAsset = MultiAsset {
				id: AssetId::Concrete(
					CurrencyIdToMultiLocation::convert(CurrencyId::OtherReserve(0u128)).unwrap(),
				),
				fun: Fungibility::Fungible(500),
			};
			let expected_fee: MultiAsset = MultiAsset {
				id: AssetId::Concrete(
					CurrencyIdToMultiLocation::convert(CurrencyId::OtherReserve(0u128)).unwrap(),
				),
				fun: Fungibility::Fungible(50),
			};
			let expected: crate::mock::Event = XtokensEvent::TransferredMultiAssets {
				sender: Alice,
				assets: vec![expected_asset.clone(), expected_fee.clone()].into(),
				fee: expected_fee,
				dest: destination,
			}
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
				precompiles().execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(AssetId(1u128).into()))
						.write(U256::from(500u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0u32),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);

			let expected_asset: MultiAsset = MultiAsset {
				id: AssetId::Concrete(
					CurrencyIdToMultiLocation::convert(CurrencyId::OtherReserve(1u128)).unwrap(),
				),
				fun: Fungibility::Fungible(500),
			};
			let expected: crate::mock::Event = XtokensEvent::TransferredMultiAssets {
				sender: Alice,
				assets: vec![expected_asset.clone()].into(),
				fee: expected_asset,
				dest: destination,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_non_reserve_to_non_reserve_with_fee_works() {
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
				precompiles().execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferWithFee)
						.write(Address(AssetId(1u128).into()))
						.write(U256::from(500u32))
						.write(U256::from(50u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0u32),
					},
					false
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);
			let expected_asset: MultiAsset = MultiAsset {
				id: AssetId::Concrete(
					CurrencyIdToMultiLocation::convert(CurrencyId::OtherReserve(1u128)).unwrap(),
				),
				fun: Fungibility::Fungible(500),
			};
			let expected_fee: MultiAsset = MultiAsset {
				id: AssetId::Concrete(
					CurrencyIdToMultiLocation::convert(CurrencyId::OtherReserve(1u128)).unwrap(),
				),
				fun: Fungibility::Fungible(50),
			};
			let expected: crate::mock::Event = XtokensEvent::TransferredMultiAssets {
				sender: Alice,
				assets: vec![expected_asset.clone(), expected_fee.clone()].into(),
				fee: expected_fee,
				dest: destination,
			}
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
				precompiles().execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferMultiAsset)
						.write(asset.clone())
						.write(U256::from(500u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0u32),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);

			let expected_asset: MultiAsset = MultiAsset {
				id: AssetId::Concrete(asset),
				fun: Fungibility::Fungible(500),
			};
			let expected: crate::mock::Event = XtokensEvent::TransferredMultiAssets {
				sender: Alice,
				assets: vec![expected_asset.clone()].into(),
				fee: expected_asset,
				dest: destination,
			}
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
				precompiles().execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferMultiAsset)
						.write(self_reserve.clone())
						.write(U256::from(500u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0u32),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);

			let expected_asset: MultiAsset = MultiAsset {
				id: AssetId::Concrete(self_reserve),
				fun: Fungibility::Fungible(500),
			};
			let expected: crate::mock::Event = XtokensEvent::TransferredMultiAssets {
				sender: Alice,
				assets: vec![expected_asset.clone()].into(),
				fee: expected_asset,
				dest: destination,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_multi_asset_self_reserve_with_fee_works() {
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
				precompiles().execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferMultiAssetWithFee)
						.write(self_reserve.clone())
						.write(U256::from(500u32))
						.write(U256::from(50u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0u32),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);

			let expected_asset: MultiAsset = MultiAsset {
				id: AssetId::Concrete(self_reserve.clone()),
				fun: Fungibility::Fungible(500),
			};
			let expected_fee: MultiAsset = MultiAsset {
				id: AssetId::Concrete(self_reserve),
				fun: Fungibility::Fungible(50),
			};
			let expected: crate::mock::Event = XtokensEvent::TransferredMultiAssets {
				sender: Alice,
				assets: vec![expected_asset.clone(), expected_fee.clone()].into(),
				fee: expected_fee,
				dest: destination,
			}
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
				precompiles().execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferMultiAsset)
						.write(asset_location.clone())
						.write(U256::from(500u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0u32),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);

			let expected_asset: MultiAsset = MultiAsset {
				id: AssetId::Concrete(asset_location),
				fun: Fungibility::Fungible(500),
			};
			let expected: crate::mock::Event = XtokensEvent::TransferredMultiAssets {
				sender: Alice,
				assets: vec![expected_asset.clone()].into(),
				fee: expected_asset,
				dest: destination,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_multi_asset_non_reserve_to_non_reserve_with_fee() {
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
				precompiles().execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferMultiAssetWithFee)
						.write(asset_location.clone())
						.write(U256::from(500u32))
						.write(U256::from(50u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0u32),
					},
					false
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);

			let expected_asset: MultiAsset = MultiAsset {
				id: AssetId::Concrete(asset_location.clone()),
				fun: Fungibility::Fungible(500),
			};
			let expected_fee: MultiAsset = MultiAsset {
				id: AssetId::Concrete(asset_location),
				fun: Fungibility::Fungible(50),
			};
			let expected: crate::mock::Event = XtokensEvent::TransferredMultiAssets {
				sender: Alice,
				assets: vec![expected_asset.clone(), expected_fee.clone()].into(),
				fee: expected_fee,
				dest: destination,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_multi_currencies() {
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
			let currencies: Vec<Currency> = vec![
				(Address(AssetId(1u128).into()), U256::from(500)).into(),
				(Address(AssetId(2u128).into()), U256::from(500)).into(),
			];

			// We are transferring 2 assets
			assert_eq!(
				precompiles().execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferMultiCurrencies)
						.write(currencies)
						.write(0u32)
						.write(destination.clone())
						.write(U256::from(4000000))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);
			let expected_asset_1: MultiAsset = MultiAsset {
				id: AssetId::Concrete(
					CurrencyIdToMultiLocation::convert(CurrencyId::OtherReserve(1u128)).unwrap(),
				),
				fun: Fungibility::Fungible(500),
			};
			let expected_asset_2: MultiAsset = MultiAsset {
				id: AssetId::Concrete(
					CurrencyIdToMultiLocation::convert(CurrencyId::OtherReserve(2u128)).unwrap(),
				),
				fun: Fungibility::Fungible(500),
			};
			let expected: crate::mock::Event = XtokensEvent::TransferredMultiAssets {
				sender: Alice,
				assets: vec![expected_asset_1.clone(), expected_asset_2].into(),
				fee: expected_asset_1,
				dest: destination,
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_multi_assets() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let destination = MultiLocation::new(
				1,
				Junctions::X2(
					Junction::Parachain(2),
					Junction::AccountId32 {
						network: NetworkId::Any,
						id: [1u8; 32],
					},
				),
			);

			let asset_1_location = MultiLocation::new(
				1,
				Junctions::X2(Junction::Parachain(2), Junction::GeneralIndex(0u128)),
			);
			let asset_2_location = MultiLocation::new(
				1,
				Junctions::X2(Junction::Parachain(2), Junction::GeneralIndex(1u128)),
			);

			let assets: Vec<EvmMultiAsset> = vec![
				(asset_1_location.clone(), U256::from(500)).into(),
				(asset_2_location.clone(), U256::from(500)).into(),
			];

			let multiassets = MultiAssets::from_sorted_and_deduplicated(vec![
				(asset_1_location.clone(), 500).into(),
				(asset_2_location, 500).into(),
			])
			.unwrap();

			// We are transferring 2 assets
			assert_eq!(
				precompiles().execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferMultiAssets)
						.write(assets)
						.write(0u32)
						.write(destination.clone())
						.write(U256::from(4000000))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					cost: 3000,
					output: vec![],
					logs: vec![]
				}))
			);
			let expected: crate::mock::Event = XtokensEvent::TransferredMultiAssets {
				sender: Alice,
				assets: multiassets,
				fee: (asset_1_location, 500).into(),
				dest: destination,
			}
			.into();
			println!("Events are {:?}", events());
			println!("Expected is {:?}", expected);
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_multi_currencies_cannot_insert_more_than_max() {
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
			let currencies: Vec<Currency> = vec![
				(Address(AssetId(1u128).into()), U256::from(500)).into(),
				(Address(AssetId(2u128).into()), U256::from(500)).into(),
				(Address(AssetId(3u128).into()), U256::from(500)).into(),
			];

			// We are transferring 3 assets, when max is 2
			assert_matches!(
				precompiles().execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferMultiCurrencies)
						.write(currencies)
						.write(0u32)
						.write(destination.clone())
						.write(U256::from(4000000))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Err(PrecompileFailure::Revert { output, ..}))
				if output == b"More than max number of assets given",
			);
		});
}

#[test]
fn transfer_multi_assets_cannot_insert_more_than_max() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let destination = MultiLocation::new(
				1,
				Junctions::X2(
					Junction::Parachain(2),
					Junction::AccountId32 {
						network: NetworkId::Any,
						id: [1u8; 32],
					},
				),
			);

			let asset_1_location = MultiLocation::new(
				1,
				Junctions::X2(Junction::Parachain(2), Junction::GeneralIndex(0u128)),
			);
			let asset_2_location = MultiLocation::new(
				1,
				Junctions::X2(Junction::Parachain(2), Junction::GeneralIndex(1u128)),
			);

			let asset_3_location = MultiLocation::new(
				1,
				Junctions::X2(Junction::Parachain(2), Junction::GeneralIndex(2u128)),
			);

			let assets: Vec<EvmMultiAsset> = vec![
				(asset_1_location.clone(), U256::from(500)).into(),
				(asset_2_location.clone(), U256::from(500)).into(),
				(asset_3_location.clone(), U256::from(500)).into(),
			];

			// We are transferring 3 assets, when max is 2
			assert_matches!(
				precompiles().execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferMultiAssets)
						.write(assets)
						.write(0u32)
						.write(destination.clone())
						.write(U256::from(4000000))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Err(PrecompileFailure::Revert { output, ..}))
				if output == b"More than max number of assets given",
			);
		});
}

#[test]
fn transfer_multi_assets_is_not_sorted_error() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let destination = MultiLocation::new(
				1,
				Junctions::X2(
					Junction::Parachain(2),
					Junction::AccountId32 {
						network: NetworkId::Any,
						id: [1u8; 32],
					},
				),
			);

			// Disordered vec creation
			let asset_1_location = MultiLocation::new(
				1,
				Junctions::X2(Junction::Parachain(2), Junction::GeneralIndex(1u128)),
			);
			let asset_2_location = MultiLocation::new(
				1,
				Junctions::X2(Junction::Parachain(2), Junction::GeneralIndex(0u128)),
			);

			let assets: Vec<EvmMultiAsset> = vec![
				(asset_1_location.clone(), U256::from(500)).into(),
				(asset_2_location.clone(), U256::from(500)).into(),
			];

			// We are transferring 3 assets, when max is 2
			assert_matches!(
				precompiles().execute(
					Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferMultiAssets)
						.write(assets)
						.write(0u32)
						.write(destination.clone())
						.write(U256::from(4000000))
						.build(),
					None,
					&Context {
						address: Precompile.into(),
						caller: Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Err(PrecompileFailure::Revert { output, ..}))
				if output == b"Provided vector either not sorted nor deduplicated",
			);
		});
}
