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

use crate::mock::{
	events, AssetAccount, ExtBuilder, PCall, Precompiles, PrecompilesValue, Runtime,
	SelfReserveAccount,
};
use crate::{Currency, EvmAsset, MAX_ASSETS};
use pallet_xcm::Event as PolkadotXcmEvent;
use precompile_utils::{prelude::*, testing::*};
use sp_core::U256;
use sp_weights::Weight;
use xcm::latest::{Junction, Location};

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn test_selector_enum() {
	assert!(PCall::transfer_selectors().contains(&0xb9f813ff));
	assert!(PCall::transfer_multiasset_selectors().contains(&0xb4f76f96));
	assert!(PCall::transfer_multi_currencies_selectors().contains(&0xab946323));
	assert!(PCall::transfer_with_fee_selectors().contains(&0x3e506ef0));
	assert!(PCall::transfer_multiasset_with_fee_selectors().contains(&0x150c016a));
}

#[test]
fn modifiers() {
	ExtBuilder::default().build().execute_with(|| {
		let mut tester = PrecompilesModifierTester::new(precompiles(), Alice, Precompile1);

		tester.test_default_modifier(PCall::transfer_selectors());
		tester.test_default_modifier(PCall::transfer_multiasset_selectors());
		tester.test_default_modifier(PCall::transfer_multi_currencies_selectors());
		tester.test_default_modifier(PCall::transfer_with_fee_selectors());
		tester.test_default_modifier(PCall::transfer_multiasset_with_fee_selectors());
	});
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile1, vec![1u8, 2u8, 3u8])
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile1, vec![1u8, 2u8, 3u8, 4u8])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn transfer_self_reserve_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let destination = Location::new(
				1,
				[Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}],
			);

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer {
						currency_address: Address(SelfReserveAccount.into()),
						amount: 500.into(),
						destination: destination.clone().into(),
						weight: 4_000_000,
					},
				)
				.expect_cost(100000000)
				.expect_no_logs()
				.execute_returns(());

			let expected: crate::mock::RuntimeEvent = PolkadotXcmEvent::Attempted {
				outcome: xcm::latest::Outcome::Complete {
					used: Weight::from_parts(3000, 3000),
				},
			}
			.into();

			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_to_reserve_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let destination = Location::new(
				1,
				[Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}],
			);
			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer {
						currency_address: Address(AssetAccount(0u128).into()),
						amount: 500.into(),
						destination: destination.clone().into(),
						weight: 4_000_000,
					},
				)
				.expect_cost(100000000)
				.expect_no_logs()
				.execute_returns(());

			let expected: crate::mock::RuntimeEvent = PolkadotXcmEvent::Attempted {
				outcome: xcm::latest::Outcome::Complete {
					used: Weight::from_parts(2000, 2000),
				},
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_to_reserve_with_unlimited_weight_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let destination = Location::new(
				1,
				[Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}],
			);
			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer {
						currency_address: Address(AssetAccount(0u128).into()),
						amount: 500.into(),
						destination: destination.clone().into(),
						weight: u64::MAX,
					},
				)
				.expect_cost(100000000)
				.expect_no_logs()
				.execute_returns(());

			let expected: crate::mock::RuntimeEvent = PolkadotXcmEvent::Attempted {
				outcome: xcm::latest::Outcome::Complete {
					used: Weight::from_parts(2000, 2000),
				},
			}
			.into();

			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_to_reserve_with_fee_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let destination = Location::new(
				1,
				[Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}],
			);
			// We are transferring asset 0, which we have instructed to be the relay asset
			// Fees are not trully charged, so no worries
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer_with_fee {
						currency_address: Address(AssetAccount(0u128).into()),
						amount: 500.into(),
						_fee: 50.into(),
						destination: destination.clone().into(),
						weight: 4_000_000,
					},
				)
				.expect_cost(100000000)
				.expect_no_logs()
				.execute_returns(());

			let expected: crate::mock::RuntimeEvent = PolkadotXcmEvent::Attempted {
				outcome: xcm::latest::Outcome::Complete {
					used: Weight::from_parts(2000, 2000),
				},
			}
			.into();

			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_non_reserve_to_non_reserve_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let destination = Location::new(
				1,
				[Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}],
			);

			// We are transferring asset 1, which corresponds to another parachain Id asset
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer {
						currency_address: Address(AssetAccount(1u128).into()),
						amount: 500.into(),
						destination: destination.clone().into(),
						weight: 4_000_000,
					},
				)
				.expect_cost(100000000)
				.expect_no_logs()
				.execute_returns(());

			let expected: crate::mock::RuntimeEvent = PolkadotXcmEvent::Attempted {
				outcome: xcm::latest::Outcome::Complete {
					used: Weight::from_parts(3000, 3000),
				},
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_non_reserve_to_non_reserve_with_fee_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let destination = Location::new(
				1,
				[Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}],
			);

			// We are transferring asset 1, which corresponds to another parachain Id asset
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer_with_fee {
						currency_address: Address(AssetAccount(1u128).into()),
						amount: 500.into(),
						_fee: 50.into(),
						destination: destination.clone().into(),
						weight: 4_000_000,
					},
				)
				.expect_cost(100000000)
				.expect_no_logs()
				.execute_returns(());

			let expected: crate::mock::RuntimeEvent = PolkadotXcmEvent::Attempted {
				outcome: xcm::latest::Outcome::Complete {
					used: Weight::from_parts(3000, 3000),
				},
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_multi_asset_to_reserve_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let destination = Location::new(
				1,
				[Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}],
			);

			let asset = Location::parent();

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer_multiasset {
						asset: asset.clone().into(),
						amount: 500.into(),
						destination: destination.clone().into(),
						weight: 4_000_000,
					},
				)
				.expect_cost(100000000)
				.expect_no_logs()
				.execute_returns(());

			let expected: crate::mock::RuntimeEvent = PolkadotXcmEvent::Attempted {
				outcome: xcm::latest::Outcome::Complete {
					used: Weight::from_parts(2000, 2000),
				},
			}
			.into();

			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_multi_asset_self_reserve_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let destination = Location::new(
				1,
				[Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}],
			);

			let self_reserve = crate::mock::SelfReserve::get();

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer_multiasset {
						asset: self_reserve.clone().into(),
						amount: 500.into(),
						destination: destination.clone().into(),
						weight: 4_000_000,
					},
				)
				.expect_cost(100000000)
				.expect_no_logs()
				.execute_returns(());

			let expected: crate::mock::RuntimeEvent = PolkadotXcmEvent::Attempted {
				outcome: xcm::latest::Outcome::Complete {
					used: Weight::from_parts(3000, 3000),
				},
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_multi_asset_self_reserve_with_fee_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let destination = Location::new(
				1,
				[Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}],
			);

			let self_reserve = crate::mock::SelfReserve::get();

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer_multiasset_with_fee {
						asset: self_reserve.clone().into(),
						amount: 500.into(),
						_fee: 50.into(),
						destination: destination.clone().into(),
						weight: 4_000_000,
					},
				)
				.expect_cost(100000000)
				.expect_no_logs()
				.execute_returns(());

			let expected: crate::mock::RuntimeEvent = PolkadotXcmEvent::Attempted {
				outcome: xcm::latest::Outcome::Complete {
					used: Weight::from_parts(3000, 3000),
				},
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_multi_asset_non_reserve_to_non_reserve() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let destination = Location::new(
				1,
				[Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}],
			);

			let asset_location =
				Location::new(1, [Junction::Parachain(2), Junction::GeneralIndex(5u128)]);

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer_multiasset {
						asset: asset_location.clone().into(),
						amount: 500.into(),
						destination: destination.clone().into(),
						weight: 4_000_000,
					},
				)
				.expect_cost(100000000)
				.expect_no_logs()
				.execute_returns(());

			let expected: crate::mock::RuntimeEvent = PolkadotXcmEvent::Attempted {
				outcome: xcm::latest::Outcome::Complete {
					used: Weight::from_parts(3000, 3000),
				},
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_multi_asset_non_reserve_to_non_reserve_with_fee() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let destination = Location::new(
				1,
				[Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}],
			);

			let asset_location =
				Location::new(1, [Junction::Parachain(2), Junction::GeneralIndex(5u128)]);

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer_multiasset_with_fee {
						asset: asset_location.clone().into(),
						amount: 500.into(),
						_fee: 50.into(),
						destination: destination.clone().into(),
						weight: 4_000_000,
					},
				)
				.expect_cost(100000000)
				.expect_no_logs()
				.execute_returns(());

			let expected: crate::mock::RuntimeEvent = PolkadotXcmEvent::Attempted {
				outcome: xcm::latest::Outcome::Complete {
					used: Weight::from_parts(3000, 3000),
				},
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_multi_currencies() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let destination = Location::new(
				1,
				[Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}],
			);
			let currencies: Vec<Currency> = vec![
				(Address(AssetAccount(1u128).into()), U256::from(500)).into(),
				(Address(AssetAccount(2u128).into()), U256::from(500)).into(),
			];

			// We are transferring 2 assets
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer_multi_currencies {
						currencies: currencies.into(),
						fee_item: 0,
						destination: destination.clone().into(),
						weight: 4_000_000,
					},
				)
				.expect_cost(100000000)
				.expect_no_logs()
				.execute_returns(());

			let expected: crate::mock::RuntimeEvent = PolkadotXcmEvent::Attempted {
				outcome: xcm::latest::Outcome::Complete {
					used: Weight::from_parts(3000, 3000),
				},
			}
			.into();
			// Assert that the events vector contains the one expected
			assert!(events().contains(&expected));
		});
}

#[test]
fn transfer_multi_assets() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let destination = Location::new(
				1,
				[
					Junction::Parachain(2),
					Junction::AccountId32 {
						network: None,
						id: [1u8; 32],
					},
				],
			);

			let asset_1_location =
				Location::new(1, [Junction::Parachain(2), Junction::GeneralIndex(0u128)]);
			let asset_2_location =
				Location::new(1, [Junction::Parachain(2), Junction::GeneralIndex(1u128)]);

			let assets: Vec<EvmAsset> = vec![
				(asset_1_location.clone(), U256::from(500)).into(),
				(asset_2_location.clone(), U256::from(500)).into(),
			];

			// We are transferring 2 assets
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer_multi_assets {
						assets: assets.into(),
						fee_item: 0,
						destination: destination.clone().into(),
						weight: 4_000_000,
					},
				)
				.expect_cost(100000000)
				.expect_no_logs()
				.execute_returns(());

			let expected: crate::mock::RuntimeEvent = PolkadotXcmEvent::Attempted {
				outcome: xcm::latest::Outcome::Complete {
					used: Weight::from_parts(2000, 2000),
				},
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
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let destination = Location::new(
				1,
				[Junction::AccountId32 {
					network: None,
					id: [1u8; 32],
				}],
			);
			let mut currencies: Vec<Currency> = Vec::new();
			for i in 0..MAX_ASSETS + 1 {
				currencies.push((Address(AssetAccount(i as u128).into()), U256::from(500)).into());
			}

			// We are transferring 3 assets, when max is 2
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer_multi_currencies {
						currencies: currencies.into(),
						fee_item: 0,
						destination: destination.clone().into(),
						weight: 4_000_000,
					},
				)
				.execute_reverts(|output| output == b"currencies: Value is too large for length");
		});
}

#[test]
fn transfer_multi_assets_cannot_insert_more_than_max() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let destination = Location::new(
				1,
				[
					Junction::Parachain(2),
					Junction::AccountId32 {
						network: None,
						id: [1u8; 32],
					},
				],
			);

			let mut assets: Vec<EvmAsset> = Vec::new();
			for i in 0..MAX_ASSETS + 1 {
				let asset_location = Location::new(
					1,
					[Junction::Parachain(2), Junction::GeneralIndex(i as u128)],
				);
				assets.push((asset_location, U256::from(500)).into());
			}

			// We are transferring 3 assets, when max is 2
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer_multi_assets {
						assets: assets.into(),
						fee_item: 0,
						destination: destination.clone().into(),
						weight: 4_000_000,
					},
				)
				.execute_reverts(|output| output == b"assets: Value is too large for length");
		});
}

#[test]
fn transfer_multi_assets_is_not_sorted_error() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let destination = Location::new(
				1,
				[
					Junction::Parachain(2),
					Junction::AccountId32 {
						network: None,
						id: [1u8; 32],
					},
				],
			);

			// Disordered vec creation
			let asset_1_location =
				Location::new(1, [Junction::Parachain(2), Junction::GeneralIndex(1u128)]);
			let asset_2_location =
				Location::new(1, [Junction::Parachain(2), Junction::GeneralIndex(0u128)]);

			let assets: Vec<EvmAsset> = vec![
				(asset_1_location.clone(), U256::from(500)).into(),
				(asset_2_location.clone(), U256::from(500)).into(),
			];

			// We are transferring 3 assets, when max is 2
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer_multi_assets {
						assets: assets.into(),
						fee_item: 0,
						destination: destination.clone().into(),
						weight: 4_000_000,
					},
				)
				.execute_reverts(|output| {
					output == b"assets: Provided assets either not sorted nor deduplicated"
				});
		});
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	check_precompile_implements_solidity_interfaces(&["Xtokens.sol"], PCall::supports_selector)
}

#[test]
fn test_deprecated_solidity_selectors_are_supported() {
	for deprecated_function in [
		"transfer_with_fee(address,uint256,uint256,(uint8,bytes[]),uint64)",
		"transfer_multiasset((uint8,bytes[]),uint256,(uint8,bytes[]),uint64)",
		"transfer_multiasset_with_fee((uint8,bytes[]),uint256,uint256,(uint8,bytes[]),uint64)",
		"transfer_multi_currencies((address,uint256)[],uint32,(uint8,bytes[]),uint64)",
		"transfer_multi_assets(((uint8,bytes[]),uint256)[],uint32,(uint8,bytes[]),uint64)",
	] {
		let selector = compute_selector(deprecated_function);
		if !PCall::supports_selector(selector) {
			panic!(
				"failed decoding selector 0x{:x} => '{}' as Action",
				selector, deprecated_function,
			)
		}
	}
}
