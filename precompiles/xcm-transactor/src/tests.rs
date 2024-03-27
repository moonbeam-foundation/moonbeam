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
	AssetAddress, ExtBuilder, PCallV1, PCallV2, PCallV3, Precompiles, PrecompilesValue, Runtime,
	RuntimeOrigin, TransactorV1, TransactorV2, TransactorV3, XcmTransactor,
};

use frame_support::{assert_ok, weights::Weight};
use precompile_utils::{prelude::*, testing::*};
use sp_core::H160;
use sp_std::boxed::Box;
use xcm::latest::Location;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selectors() {
	assert!(PCallV1::index_to_account_selectors().contains(&0x3fdc4f36));
	assert!(PCallV1::transact_info_selectors().contains(&0xd07d87c3));
	assert!(PCallV1::transact_info_with_signed_selectors().contains(&0xb689e20c));
	assert!(PCallV1::fee_per_second_selectors().contains(&0x906c9990));
	assert!(PCallV1::transact_through_derivative_multilocation_selectors().contains(&0x94a63c54));
	assert!(PCallV1::transact_through_derivative_selectors().contains(&0x02ae072d));
	assert!(PCallV1::transact_through_signed_multilocation_selectors().contains(&0x71d31587));
	assert!(PCallV1::transact_through_signed_selectors().contains(&0x42ca339d));

	assert!(PCallV2::index_to_account_selectors().contains(&0x3fdc4f36));
	assert!(PCallV2::transact_info_with_signed_selectors().contains(&0xb689e20c));
	assert!(PCallV2::fee_per_second_selectors().contains(&0x906c9990));
	assert!(PCallV2::transact_through_derivative_multilocation_selectors().contains(&0xfe430475));
	assert!(PCallV2::transact_through_derivative_selectors().contains(&0x185de2ae));
	assert!(PCallV2::transact_through_signed_multilocation_selectors().contains(&0xd7ab340c));
	assert!(PCallV2::transact_through_signed_selectors().contains(&0xb648f3fe));

	assert!(PCallV3::index_to_account_selectors().contains(&0x3fdc4f36));
	assert!(PCallV3::transact_info_with_signed_selectors().contains(&0xb689e20c));
	assert!(PCallV3::fee_per_second_selectors().contains(&0x906c9990));
	assert!(PCallV3::transact_through_derivative_multilocation_selectors().contains(&0xbdacc26b));
	assert!(PCallV3::transact_through_derivative_selectors().contains(&0xca8c82d8));
	assert!(PCallV3::transact_through_signed_multilocation_selectors().contains(&0x27b1d492));
	assert!(PCallV3::transact_through_signed_selectors().contains(&0xb18270cf));
}

#[test]
fn modifiers() {
	ExtBuilder::default().build().execute_with(|| {
		let mut tester = PrecompilesModifierTester::new(precompiles(), Alice, TransactorV1);

		tester.test_view_modifier(PCallV1::index_to_account_selectors());
		tester.test_view_modifier(PCallV1::transact_info_selectors());
		tester.test_view_modifier(PCallV1::transact_info_with_signed_selectors());
		tester.test_view_modifier(PCallV1::fee_per_second_selectors());
		tester
			.test_default_modifier(PCallV1::transact_through_derivative_multilocation_selectors());
		tester.test_default_modifier(PCallV1::transact_through_derivative_selectors());
		tester.test_default_modifier(PCallV1::transact_through_signed_multilocation_selectors());
		tester.test_default_modifier(PCallV1::transact_through_signed_selectors());

		let mut tester = PrecompilesModifierTester::new(precompiles(), Alice, TransactorV2);

		tester.test_view_modifier(PCallV2::index_to_account_selectors());
		tester.test_view_modifier(PCallV2::transact_info_with_signed_selectors());
		tester.test_view_modifier(PCallV2::fee_per_second_selectors());
		tester
			.test_default_modifier(PCallV2::transact_through_derivative_multilocation_selectors());
		tester.test_default_modifier(PCallV2::transact_through_derivative_selectors());
		tester.test_default_modifier(PCallV2::transact_through_signed_multilocation_selectors());
		tester.test_default_modifier(PCallV2::transact_through_signed_selectors());
	});
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, TransactorV1, vec![1u8, 2u8, 3u8])
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, TransactorV1, vec![1u8, 2u8, 3u8, 4u8])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn take_index_for_account() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let input: Vec<_> = PCallV1::index_to_account { index: 0 }.into();

			// Assert that errors since no index is assigned
			precompiles()
				.prepare_test(Alice, TransactorV1, input.clone())
				.execute_reverts(|output| output == b"No index assigned");

			// register index
			assert_ok!(XcmTransactor::register(
				RuntimeOrigin::root(),
				Alice.into(),
				0
			));

			// Expected result is zero
			precompiles()
				.prepare_test(Alice, TransactorV1, input)
				.expect_cost(1)
				.expect_no_logs()
				.execute_returns(Address(H160::from(Alice)));
		});
}

#[test]
fn take_transact_info() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let input: Vec<_> = PCallV1::transact_info {
				location: Location::parent(),
			}
			.into();

			// Assert that errors since no index is assigned
			precompiles()
				.prepare_test(Alice, TransactorV1, input.clone())
				.execute_reverts(|output| output == b"Transact Info not set");

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				Weight::zero(),
				10000u64.into(),
				None
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1
			));

			precompiles()
				.prepare_test(Alice, TransactorV1, input)
				.expect_cost(2)
				.expect_no_logs()
				.execute_returns((0u64, 1u128, 10000u64));
		});
}
#[test]
fn take_transact_info_with_signed() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let input: Vec<_> = PCallV1::transact_info_with_signed {
				multilocation: Location::parent(),
			}
			.into();

			// Assert that errors since no index is assigned
			precompiles()
				.prepare_test(Alice, TransactorV1, input.clone())
				.execute_reverts(|output| output == b"Transact Info not set");

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				Weight::zero(),
				10000u64.into(),
				Some(1.into())
			));

			// Root can set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1
			));

			precompiles()
				.prepare_test(Alice, TransactorV1, input)
				.expect_cost(1)
				.expect_no_logs()
				.execute_returns((0u64, 1u128, 10_000u64));
		});
}

#[test]
fn take_fee_per_second() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let input: Vec<_> = PCallV1::fee_per_second {
				multilocation: Location::parent(),
			}
			.into();

			// Assert that errors
			precompiles()
				.prepare_test(Alice, TransactorV1, input.clone())
				.execute_reverts(|output| output == b"Fee Per Second not set");

			// Root can set fee per secnd
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1
			));
			precompiles()
				.prepare_test(Alice, TransactorV1, input)
				.expect_cost(1)
				.expect_no_logs()
				.execute_returns(1u64);
		});
}

#[test]
fn test_transact_derivative_multilocation_v2() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// register index
			assert_ok!(XcmTransactor::register(
				RuntimeOrigin::root(),
				Alice.into(),
				0
			));

			// we pay with our current self reserve.
			let fee_payer_asset = Location::parent();

			let bytes = vec![1u8, 2u8, 3u8];

			let total_weight = 1_000_000_000u64;
			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					TransactorV2,
					PCallV2::transact_through_derivative_multilocation {
						transactor: 0,
						index: 0,
						fee_asset: fee_payer_asset,
						weight: 4000000,
						inner_call: bytes.into(),
						fee_amount: u128::from(total_weight).into(),
						overall_weight: total_weight,
					},
				)
				.expect_cost(188253000)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn test_transact_derivative_multilocation_v3() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// register index
			assert_ok!(XcmTransactor::register(
				RuntimeOrigin::root(),
				Alice.into(),
				0
			));

			// we pay with our current self reserve.
			let fee_payer_asset = Location::parent();

			let bytes = vec![1u8, 2u8, 3u8];

			//let total_weight = 1_000_000_000u64;
			let total_weight = Weight::from_parts(1_000_000_000u64, 82_000u64);
			let require_weight_at_most = Weight::from_parts(4_000_000u64, 82_000u64);
			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					TransactorV3,
					PCallV3::transact_through_derivative_multilocation {
						transactor: 0,
						index: 0,
						fee_asset: fee_payer_asset,
						weight: require_weight_at_most,
						inner_call: bytes.into(),
						fee_amount: u128::from(total_weight.ref_time()).into(),
						overall_weight: total_weight,
						refund: false,
					},
				)
				.expect_cost(188253000)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn take_transact_info_with_signed_v3() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let input: Vec<_> = PCallV3::transact_info_with_signed {
				multilocation: Location::parent(),
			}
			.into();

			// Assert that errors since no index is assigned
			precompiles()
				.prepare_test(Alice, TransactorV3, input.clone())
				.execute_reverts(|output| output == b"Transact Info not set");

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				Weight::zero(),
				10000u64.into(),
				Some(1.into())
			));

			// Root can set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1
			));

			let expected_max_weight: Weight = 10_000u64.into();
			let expected_transact_extra_weight_signed: Weight = 1u64.into();
			let expected_transact_extra_weight: Weight = 0u64.into();

			precompiles()
				.prepare_test(Alice, TransactorV3, input)
				.expect_cost(1)
				.expect_no_logs()
				.execute_returns((
					expected_transact_extra_weight,
					expected_transact_extra_weight_signed,
					expected_max_weight,
				));
		});
}

#[test]
fn test_transact_derivative_multilocation() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// register index
			assert_ok!(XcmTransactor::register(
				RuntimeOrigin::root(),
				Alice.into(),
				0
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				Weight::zero(),
				10000000.into(),
				None
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1
			));

			// we pay with our current self reserve.
			let fee_payer_asset = Location::parent();

			let bytes = vec![1u8, 2u8, 3u8];

			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					TransactorV1,
					PCallV1::transact_through_derivative_multilocation {
						transactor: 0,
						index: 0,
						fee_asset: fee_payer_asset,
						weight: 4_000_000,
						inner_call: bytes.into(),
					},
				)
				.expect_cost(188253000)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn test_transact_derivative() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// register index
			assert_ok!(XcmTransactor::register(
				RuntimeOrigin::root(),
				Alice.into(),
				0
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				Weight::zero(),
				10000000.into(),
				None
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1
			));

			let bytes = vec![1u8, 2u8, 3u8];

			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					TransactorV1,
					PCallV1::transact_through_derivative {
						transactor: 0,
						index: 0,
						currency_id: Address(AssetAddress(0).into()),
						weight: 4_000_000,
						inner_call: bytes.into(),
					},
				)
				.expect_cost(188254000)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn test_transact_derivative_v2() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// register index
			assert_ok!(XcmTransactor::register(
				RuntimeOrigin::root(),
				Alice.into(),
				0
			));

			let bytes = vec![1u8, 2u8, 3u8];

			let total_weight = 1_000_000_000u64;

			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					TransactorV2,
					PCallV2::transact_through_derivative {
						transactor: 0,
						index: 0,
						fee_asset: Address(AssetAddress(0).into()),
						weight: 4_000_000,
						inner_call: bytes.into(),
						fee_amount: u128::from(total_weight).into(),
						overall_weight: total_weight,
					},
				)
				.expect_cost(188254000)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn test_transact_derivative_v3() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// register index
			assert_ok!(XcmTransactor::register(
				RuntimeOrigin::root(),
				Alice.into(),
				0
			));

			let bytes = vec![1u8, 2u8, 3u8];

			//let total_weight = 1_000_000_000u64;
			let total_weight = Weight::from_parts(1_000_000_000u64, 82_000u64);
			let require_weight_at_most = Weight::from_parts(4_000_000u64, 82_000u64);

			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					TransactorV3,
					PCallV3::transact_through_derivative {
						transactor: 0,
						index: 0,
						fee_asset: Address(AssetAddress(0).into()),
						weight: require_weight_at_most,
						inner_call: bytes.into(),
						fee_amount: u128::from(total_weight.ref_time()).into(),
						overall_weight: total_weight,
						refund: false,
					},
				)
				.expect_cost(188254000)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn test_transact_signed() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				Weight::zero(),
				10000000.into(),
				Some(1.into())
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1
			));

			// Destination
			let dest = Location::parent();

			let bytes = vec![1u8, 2u8, 3u8];

			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					TransactorV1,
					PCallV1::transact_through_signed {
						dest,
						fee_asset: Address(AssetAddress(0).into()),
						weight: 4_000_000,
						call: bytes.into(),
					},
				)
				.expect_cost(468449000)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn test_transact_signed_v2() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// Destination
			let dest = Location::parent();

			let bytes = vec![1u8, 2u8, 3u8];

			let total_weight = 1_000_000_000u64;

			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					TransactorV2,
					PCallV2::transact_through_signed {
						dest,
						fee_asset: Address(AssetAddress(0).into()),
						weight: 4_000_000,
						call: bytes.into(),
						fee_amount: u128::from(total_weight).into(),
						overall_weight: total_weight,
					},
				)
				.expect_cost(468449000)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn test_transact_signed_v3() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// register index
			assert_ok!(XcmTransactor::register(
				RuntimeOrigin::root(),
				Alice.into(),
				0
			));

			let bytes = vec![1u8, 2u8, 3u8];

			let total_weight = Weight::from_parts(1_000_000_000u64, 82_000u64);
			let require_weight_at_most = Weight::from_parts(4_000_000u64, 82_000u64);
			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					TransactorV3,
					PCallV3::transact_through_signed {
						dest: Location::parent(),
						fee_asset: Address(AssetAddress(0).into()),
						weight: require_weight_at_most,
						call: bytes.into(),
						fee_amount: u128::from(total_weight.ref_time()).into(),
						overall_weight: total_weight,
						refund: false,
					},
				)
				.expect_cost(468449000)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn test_transact_signed_multilocation() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				Weight::zero(),
				10000000.into(),
				Some(1.into())
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				RuntimeOrigin::root(),
				Box::new(xcm::VersionedLocation::V4(Location::parent())),
				1
			));

			// Destination
			let dest = Location::parent();

			let fee_payer_asset = Location::parent();

			let bytes = vec![1u8, 2u8, 3u8];

			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					TransactorV1,
					PCallV1::transact_through_signed_multilocation {
						dest,
						fee_asset: fee_payer_asset,
						weight: 4_000_000,
						call: bytes.into(),
					},
				)
				.expect_cost(468448000)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn test_transact_signed_multilocation_v2() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// Destination
			let dest = Location::parent();

			let fee_payer_asset = Location::parent();

			let bytes = vec![1u8, 2u8, 3u8];

			let total_weight = 1_000_000_000u64;

			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					TransactorV2,
					PCallV2::transact_through_signed_multilocation {
						dest,
						fee_asset: fee_payer_asset,
						weight: 4_000_000,
						call: bytes.into(),
						fee_amount: u128::from(total_weight).into(),
						overall_weight: total_weight,
					},
				)
				.expect_cost(468448000)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn test_transact_through_signed_multilocation_v3() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			// register index
			assert_ok!(XcmTransactor::register(
				RuntimeOrigin::root(),
				Alice.into(),
				0
			));

			// we pay with our current self reserve.
			let fee_payer_asset = Location::parent();

			let bytes = vec![1u8, 2u8, 3u8];

			let total_weight = Weight::from_parts(1_000_000_000u64, 82_000u64);
			let require_weight_at_most = Weight::from_parts(4_000_000u64, 82_000u64);
			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					TransactorV3,
					PCallV3::transact_through_signed_multilocation {
						dest: Location::parent(),
						fee_asset: fee_payer_asset,
						weight: require_weight_at_most,
						call: bytes.into(),
						fee_amount: u128::from(total_weight.ref_time()).into(),
						overall_weight: total_weight,
						refund: false,
					},
				)
				.expect_cost(468448000)
				.expect_no_logs()
				.execute_returns(());
		});
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented_v1() {
	check_precompile_implements_solidity_interfaces(
		&["src/v1/XcmTransactorV1.sol"],
		PCallV1::supports_selector,
	)
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented_v2() {
	check_precompile_implements_solidity_interfaces(
		&["src/v2/XcmTransactorV2.sol"],
		PCallV2::supports_selector,
	)
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented_v3() {
	check_precompile_implements_solidity_interfaces(
		&["src/v3/XcmTransactorV3.sol"],
		PCallV3::supports_selector,
	)
}

#[test]
fn test_deprecated_solidity_selectors_are_supported() {
	for deprecated_function in [
		"index_to_account(uint16)",
		"transact_info((uint8,bytes[]))",
		"transact_through_derivative_multilocation(uint8,uint16,(uint8,bytes[]),uint64,bytes)",
		"transact_through_derivative(uint8,uint16,address,uint64,bytes)",
		"transact_info_with_signed((uint8,bytes[]))",
		"fee_per_second((uint8,bytes[]))",
		"transact_through_signed_multilocation((uint8,bytes[]),(uint8,bytes[]),uint64,bytes)",
		"transact_through_signed((uint8,bytes[]),address,uint64,bytes)",
	] {
		let selector = compute_selector(deprecated_function);
		if !PCallV1::supports_selector(selector) {
			panic!(
				"failed decoding selector 0x{:x} => '{}' as Action",
				selector, deprecated_function,
			)
		}
	}
}
