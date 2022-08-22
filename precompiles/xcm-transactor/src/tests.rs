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
	precompile_address_v1, precompile_address_v2, ExtBuilder, Origin, PrecompilesValue, Runtime,
	TestAccount::*, TestPrecompiles, XcmTransactor,
};
use crate::v1::Action as ActionV1;
use crate::v2::Action as ActionV2;

use frame_support::assert_ok;
use precompile_utils::{prelude::*, solidity, testing::*};
use sp_core::{H160, U256};
use sp_std::boxed::Box;
use xcm::v1::MultiLocation;

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn test_selector_enum() {
	assert_eq!(ActionV1::IndexToAccount as u32, 0x3fdc4f36);
	assert_eq!(ActionV1::TransactInfo as u32, 0xd07d87c3);
	assert_eq!(
		ActionV1::TransactThroughDerivativeMultiLocation as u32,
		0x94a63c54
	);
	assert_eq!(ActionV1::TransactThroughDerivative as u32, 0x02ae072d);
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, precompile_address_v1(), vec![1u8, 2u8, 3u8])
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile, vec![1u8, 2u8, 3u8, 4u8])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn take_index_for_account() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let input = EvmDataWriter::new_with_selector(ActionV1::IndexToAccount)
				.write(0u16)
				.build();

			// Assert that errors since no index is assigned
			precompiles()
				.prepare_test(Alice, Precompile, input.clone())
				.execute_reverts(|output| output == b"No index assigned");

			// register index
			assert_ok!(XcmTransactor::register(Origin::root(), Alice.into(), 0));

			// Expected result is zero
			precompiles()
				.prepare_test(Alice, Precompile, input)
				.expect_cost(1)
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(Address(H160::from(Alice)))
						.build(),
				);
		});
}

#[test]
fn take_transact_info() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let input = EvmDataWriter::new_with_selector(ActionV1::TransactInfo)
				.write(MultiLocation::parent())
				.build();

			// Assert that errors since no index is assigned
			precompiles()
				.prepare_test(Alice, Precompile, input.clone())
				.execute_reverts(|output| output == b"Transact Info not set");

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000u64,
				None
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1
			));

			precompiles()
				.prepare_test(Alice, Precompile, input)
				.expect_cost(2)
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(0u64)
						.write(1u128)
						.write(10000u64)
						.build(),
				);
		});
}
#[test]
fn take_transact_info_with_signed() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let input = EvmDataWriter::new_with_selector(ActionV1::TransactInfoWithSigned)
				.write(MultiLocation::parent())
				.build();

			// Assert that errors since no index is assigned
			precompiles()
				.prepare_test(Alice, Precompile, input.clone())
				.execute_reverts(|output| output == b"Transact Info not set");

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000u64,
				Some(1)
			));

			// Root can set fee per second
			assert_ok!(XcmTransactor::set_fee_per_second(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1
			));

			precompiles()
				.prepare_test(Alice, Precompile, input)
				.expect_cost(1)
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(0u64)
						.write(1u128)
						.write(10000u64)
						.build(),
				);
		});
}

#[test]
fn take_fee_per_second() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let input = EvmDataWriter::new_with_selector(ActionV1::FeePerSecond)
				.write(MultiLocation::parent())
				.build();

			// Assert that errors
			precompiles()
				.prepare_test(Alice, Precompile, input.clone())
				.execute_reverts(|output| output == b"Fee Per Second not set");

			// Root can set fee per secnd
			assert_ok!(XcmTransactor::set_fee_per_second(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1
			));
			precompiles()
				.prepare_test(Alice, Precompile, input)
				.expect_cost(1)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(1u64).build());
		});
}

#[test]
fn test_transact_derivative_multilocation_v2() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// register index
			assert_ok!(XcmTransactor::register(Origin::root(), Alice.into(), 0));

			// we pay with our current self reserve.
			let fee_payer_asset = MultiLocation::parent();

			let bytes = Bytes(vec![1u8, 2u8, 3u8]);

			let total_weight = 1_000_000_000u64;
			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					precompile_address_v2(),
					EvmDataWriter::new_with_selector(
						ActionV2::TransactThroughDerivativeMultiLocation,
					)
					.write(0u8)
					.write(0u16)
					.write(fee_payer_asset)
					.write(U256::from(4000000))
					.write(bytes)
					.write(total_weight as u128)
					.write(total_weight)
					.build(),
				)
				.expect_cost(4004000)
				.expect_no_logs()
				.execute_returns(vec![]);
		});
}

#[test]
fn test_transact_derivative_multilocation() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// register index
			assert_ok!(XcmTransactor::register(Origin::root(), Alice.into(), 0));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000000,
				None
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1
			));

			// we pay with our current self reserve.
			let fee_payer_asset = MultiLocation::parent();

			let bytes = Bytes(vec![1u8, 2u8, 3u8]);

			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(
						ActionV1::TransactThroughDerivativeMultiLocation,
					)
					.write(0u8)
					.write(0u16)
					.write(fee_payer_asset)
					.write(U256::from(4000000))
					.write(bytes)
					.build(),
				)
				.expect_cost(4004000)
				.expect_no_logs()
				.execute_returns(vec![]);
		});
}

#[test]
fn test_transact_derivative() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// register index
			assert_ok!(XcmTransactor::register(Origin::root(), Alice.into(), 0));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000000,
				None
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1
			));

			let bytes = Bytes(vec![1u8, 2u8, 3u8]);

			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(ActionV1::TransactThroughDerivative)
						.write(0u8)
						.write(0u16)
						.write(Address(AssetId(0).into()))
						.write(U256::from(4000000))
						.write(bytes)
						.build(),
				)
				.expect_cost(4004001)
				.expect_no_logs()
				.execute_returns(vec![]);
		});
}

#[test]
fn test_transact_derivative_v2() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// register index
			assert_ok!(XcmTransactor::register(Origin::root(), Alice.into(), 0));

			let bytes = Bytes(vec![1u8, 2u8, 3u8]);

			let total_weight = 1_000_000_000u64;

			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					precompile_address_v2(),
					EvmDataWriter::new_with_selector(ActionV2::TransactThroughDerivative)
						.write(0u8)
						.write(0u16)
						.write(Address(AssetId(0).into()))
						.write(U256::from(4000000))
						.write(bytes)
						.write(total_weight as u128)
						.write(total_weight)
						.build(),
				)
				.expect_cost(4004001)
				.expect_no_logs()
				.execute_returns(vec![]);
		});
}

#[test]
fn test_transact_signed() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000000,
				Some(1)
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1
			));

			// Destination
			let dest = MultiLocation::parent();

			let bytes: Bytes = vec![1u8, 2u8, 3u8].as_slice().into();

			let total_weight = 1_000_000_000u64;

			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(ActionV1::TransactThroughSigned)
						.write(dest)
						.write(Address(AssetId(0).into()))
						.write(U256::from(4000000))
						.write(bytes)
						.write(total_weight as u128)
						.write(total_weight)
						.build(),
				)
				.expect_cost(428130001)
				.expect_no_logs()
				.execute_returns(vec![]);
		});
}

#[test]
fn test_transact_signed_v2() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// Destination
			let dest = MultiLocation::parent();

			let bytes: Bytes = vec![1u8, 2u8, 3u8].as_slice().into();

			let total_weight = 1_000_000_000u64;

			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					precompile_address_v2(),
					EvmDataWriter::new_with_selector(ActionV2::TransactThroughSigned)
						.write(dest)
						.write(Address(AssetId(0).into()))
						.write(U256::from(4000000))
						.write(bytes)
						.write(total_weight as u128)
						.write(total_weight)
						.build(),
				)
				.expect_cost(428130001)
				.expect_no_logs()
				.execute_returns(vec![]);
		});
}

#[test]
fn test_transact_signed_multilocation() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// Root can set transact info
			assert_ok!(XcmTransactor::set_transact_info(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				0,
				10000000,
				Some(1)
			));

			// Root can set transact info
			assert_ok!(XcmTransactor::set_fee_per_second(
				Origin::root(),
				Box::new(xcm::VersionedMultiLocation::V1(MultiLocation::parent())),
				1
			));

			// Destination
			let dest = MultiLocation::parent();

			let fee_payer_asset = MultiLocation::parent();

			let bytes: Bytes = vec![1u8, 2u8, 3u8].as_slice().into();

			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(ActionV1::TransactThroughSignedMultiLocation)
						.write(dest)
						.write(fee_payer_asset)
						.write(U256::from(4000000))
						.write(bytes)
						.build(),
				)
				.expect_cost(428130000)
				.expect_no_logs()
				.execute_returns(vec![]);
		});
}

#[test]
fn test_transact_signed_multilocation_v2() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			// Destination
			let dest = MultiLocation::parent();

			let fee_payer_asset = MultiLocation::parent();

			let bytes: Bytes = vec![1u8, 2u8, 3u8].as_slice().into();

			let total_weight = 1_000_000_000u64;

			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					precompile_address_v2(),
					EvmDataWriter::new_with_selector(ActionV2::TransactThroughSignedMultiLocation)
						.write(dest)
						.write(fee_payer_asset)
						.write(U256::from(4000000))
						.write(bytes)
						.write(total_weight as u128)
						.write(total_weight)
						.build(),
				)
				.expect_cost(428130000)
				.expect_no_logs()
				.execute_returns(vec![]);
		});
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented_v1() {
	for file in ["src/v1/XcmTransactorV1.sol"] {
		for solidity_fn in solidity::get_selectors(file) {
			assert_eq!(
				solidity_fn.compute_selector_hex(),
				solidity_fn.docs_selector,
				"documented selector for '{}' did not match for file '{}'",
				solidity_fn.signature(),
				file,
			);

			let selector = solidity_fn.compute_selector();
			if ActionV1::try_from(selector).is_err() {
				panic!(
					"failed decoding selector 0x{:x} => '{}' as Action for file '{}'",
					selector,
					solidity_fn.signature(),
					file,
				)
			}
		}
	}
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented_v2() {
	for file in ["src/v2/XcmTransactorV2.sol"] {
		for solidity_fn in solidity::get_selectors(file) {
			assert_eq!(
				solidity_fn.compute_selector_hex(),
				solidity_fn.docs_selector,
				"documented selector for '{}' did not match for file '{}'",
				solidity_fn.signature(),
				file,
			);

			let selector = solidity_fn.compute_selector();
			if ActionV2::try_from(selector).is_err() {
				panic!(
					"failed decoding selector 0x{:x} => '{}' as Action for file '{}'",
					selector,
					solidity_fn.signature(),
					file,
				)
			}
		}
	}
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
		let selector = solidity::compute_selector(deprecated_function);
		if Action::try_from(selector).is_err() {
			panic!(
				"failed decoding selector 0x{:x} => '{}' as Action",
				selector, deprecated_function,
			)
		}
	}
}
