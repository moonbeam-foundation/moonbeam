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

use crate::mock::{
	events, CurrencyId, CurrencyIdToMultiLocation, ExtBuilder, PrecompilesValue, Runtime,
	TestAccount::*, TestPrecompiles,
};
use crate::{Action, Currency, EvmMultiAsset};
use orml_xtokens::Event as XtokensEvent;
use precompile_utils::{prelude::*, testing::*};
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
	assert_eq!(Action::Transfer as u32, 0xb9f813ff);
	assert_eq!(Action::TransferMultiAsset as u32, 0xb38c60fa);
	assert_eq!(Action::TransferMultiCurrencies as u32, 0x8a362d5c);
	assert_eq!(Action::TransferWithFee as u32, 0x94f69115);
	assert_eq!(Action::TransferMultiAssetWithFee as u32, 0x89a570fc);
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile, vec![1u8, 2u8, 3u8])
			.execute_reverts(|output| output == b"tried to parse selector out of bounds");
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile, vec![1u8, 2u8, 3u8, 4u8])
			.execute_reverts(|output| output == b"unknown selector");
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

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(SelfReserve.into()))
						.write(U256::from(500u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
				)
				.expect_cost(2000)
				.expect_no_logs()
				.execute_returns(vec![]);

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
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(AssetId(0u128).into()))
						.write(U256::from(500u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
				)
				.expect_cost(3000)
				.expect_no_logs()
				.execute_returns(vec![]);

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
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::TransferWithFee)
						.write(Address(AssetId(0u128).into()))
						.write(U256::from(500u64))
						.write(U256::from(50u64))
						.write(destination.clone())
						.write(U256::from(4000000u64))
						.build(),
				)
				.expect_cost(3000)
				.expect_no_logs()
				.execute_returns(vec![]);

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
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(AssetId(1u128).into()))
						.write(U256::from(500u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
				)
				.expect_cost(3000)
				.expect_no_logs()
				.execute_returns(vec![]);

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
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::TransferWithFee)
						.write(Address(AssetId(1u128).into()))
						.write(U256::from(500u32))
						.write(U256::from(50u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
				)
				.expect_cost(3000)
				.expect_no_logs()
				.execute_returns(vec![]);

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

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::TransferMultiAsset)
						.write(asset.clone())
						.write(U256::from(500u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
				)
				.expect_cost(3000)
				.expect_no_logs()
				.execute_returns(vec![]);

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

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::TransferMultiAsset)
						.write(self_reserve.clone())
						.write(U256::from(500u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
				)
				.expect_cost(2000)
				.expect_no_logs()
				.execute_returns(vec![]);

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

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::TransferMultiAssetWithFee)
						.write(self_reserve.clone())
						.write(U256::from(500u32))
						.write(U256::from(50u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
				)
				.expect_cost(2000)
				.expect_no_logs()
				.execute_returns(vec![]);

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

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::TransferMultiAsset)
						.write(asset_location.clone())
						.write(U256::from(500u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
				)
				.expect_cost(3000)
				.expect_no_logs()
				.execute_returns(vec![]);

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

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::TransferMultiAssetWithFee)
						.write(asset_location.clone())
						.write(U256::from(500u32))
						.write(U256::from(50u32))
						.write(destination.clone())
						.write(U256::from(4000000u32))
						.build(),
				)
				.expect_cost(3000)
				.expect_no_logs()
				.execute_returns(vec![]);

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
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::TransferMultiCurrencies)
						.write(currencies)
						.write(0u32)
						.write(destination.clone())
						.write(U256::from(4000000))
						.build(),
				)
				.expect_cost(3000)
				.expect_no_logs()
				.execute_returns(vec![]);

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
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::TransferMultiAssets)
						.write(assets)
						.write(0u32)
						.write(destination.clone())
						.write(U256::from(4000000))
						.build(),
				)
				.expect_cost(3000)
				.expect_no_logs()
				.execute_returns(vec![]);

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
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::TransferMultiCurrencies)
						.write(currencies)
						.write(0u32)
						.write(destination.clone())
						.write(U256::from(4000000))
						.build(),
				)
				.execute_reverts(|output| output == b"More than max number of assets given");
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
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::TransferMultiAssets)
						.write(assets)
						.write(0u32)
						.write(destination.clone())
						.write(U256::from(4000000))
						.build(),
				)
				.execute_reverts(|output| output == b"More than max number of assets given");
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
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::TransferMultiAssets)
						.write(assets)
						.write(0u32)
						.write(destination.clone())
						.write(U256::from(4000000))
						.build(),
				)
				.execute_reverts(|output| {
					output == b"Provided vector either not sorted nor deduplicated"
				});
		});
}
