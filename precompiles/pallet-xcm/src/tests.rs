// Copyright 2024 Moonbeam Foundation.
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
use crate::{mock::*, Location};
use precompile_utils::testing::*;
use sp_weights::Weight;
use xcm::latest::Junction::*;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	check_precompile_implements_solidity_interfaces(&["XcmInterface.sol"], PCall::supports_selector)
}

#[test]
fn selectors() {
	assert!(PCall::transfer_assets_selectors().contains(&0x650ef8c7));
}

#[test]
fn modifiers() {
	ExtBuilder::default().build().execute_with(|| {
		let mut tester =
			PrecompilesModifierTester::new(PrecompilesValue::get(), Alice, Precompile1);

		tester.test_default_modifier(PCall::transfer_assets_selectors());
	});
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
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
fn test_transfer_assets_works() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let dest = Location::new(1, [Parachain(2)]);

			// Specify the beneficiary from the destination's point of view
			let beneficiary = Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: [1; 20],
				}],
			);

			let destination_asset_location = Location::new(1, [Parachain(2), PalletInstance(3)]);

			let origin_asset_location = Location {
				parents: 0,
				interior: [PalletInstance(1)].into(),
			};

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer_assets {
						dest,
						beneficiary,
						assets: vec![
							(origin_asset_location, 100u128.into()),
							(destination_asset_location, 150u128.into()),
						]
						.into(),
						fee_asset_item: 0u32,
						// As we are indicating u64::MAX in ref_time, an Unlimited variant
						// will be applied at the end.
						weight: Weight::from_parts(u64::MAX, 80000),
					},
				)
				.expect_cost(100005001)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn test_transfer_assets_success_when_paying_fees_with_foreign_asset() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let dest = Location::new(1, [Parachain(2)]);

			// Specify the beneficiary from the destination's point of view
			let beneficiary = Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: [1; 20],
				}],
			);

			let destination_asset_location = Location::new(1, [Parachain(2), PalletInstance(3)]);

			let origin_asset_location = Location {
				parents: 0,
				interior: [PalletInstance(1)].into(),
			};

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer_assets {
						dest,
						beneficiary,
						assets: vec![
							(origin_asset_location, 100u128.into()),
							(destination_asset_location, 150u128.into()),
						]
						.into(),
						// We also act as a reserve for the foreign asset thus when can pay local
						// fees with it.
						fee_asset_item: 1u32,
						// As we are indicating u64::MAX in ref_time, an Unlimited variant
						// will be applied at the end.
						weight: Weight::from_parts(u64::MAX, 80000),
					},
				)
				.expect_cost(100005001)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn test_transfer_assets_fails_fees_unknown_reserve() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let dest = Location::new(1, [Parachain(3)]);

			// Specify the beneficiary from the destination's point of view
			let beneficiary = Location::new(
				0,
				[AccountKey20 {
					network: None,
					key: [1; 20],
				}],
			);

			let destination_asset_location = Location::new(1, [Parachain(3), PalletInstance(3)]);

			let origin_asset_location = Location {
				parents: 0,
				interior: [PalletInstance(1)].into(),
			};

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::transfer_assets {
						dest,
						beneficiary,
						assets: vec![
							(origin_asset_location, 100u128.into()),
							(destination_asset_location, 150u128.into()),
						]
						.into(),
						// No reserve will be found for this asset
						fee_asset_item: 1u32,
						// As we are indicating u64::MAX in ref_time, an Unlimited variant
						// will be applied at the end.
						weight: Weight::from_parts(u64::MAX, 80000),
					},
				)
				.expect_no_logs()
				.execute_reverts(|output| output.ends_with(b"InvalidAssetUnknownReserve\") })"));
		});
}
