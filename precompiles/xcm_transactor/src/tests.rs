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
	evm_test_context, ExtBuilder, Origin, PrecompilesValue, Runtime, TestAccount::*,
	TestPrecompiles, XcmTransactor,
};
use crate::{Action, PrecompileOutput};

use fp_evm::PrecompileFailure;
use frame_support::assert_ok;
use num_enum::TryFromPrimitive;
use pallet_evm::{ExitSucceed, PrecompileSet};
use precompile_utils::{testing::*, Address, Bytes, EvmDataWriter};
use sha3::{Digest, Keccak256};
use sp_core::{H160, U256};
use sp_std::boxed::Box;
use std::assert_matches::assert_matches;
use xcm::v1::MultiLocation;

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn test_selector_enum() {
	assert_eq!(Action::IndexToAccount as u32, 0x71b0edfa);
	assert_eq!(Action::TransactInfo as u32, 0xf87f493f);
	assert_eq!(
		Action::TransactThroughDerivativeMultiLocation as u32,
		0x9f89f03e
	);
	assert_eq!(Action::TransactThroughDerivative as u32, 0x267d4062);
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
fn take_index_for_account() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let input = EvmDataWriter::new_with_selector(Action::IndexToAccount)
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
			let input = EvmDataWriter::new_with_selector(Action::TransactInfo)
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
				1,
				10000,
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
fn test_transactor_multilocation() {
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
				1,
				10000000
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
						Action::TransactThroughDerivativeMultiLocation,
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
fn test_transactor() {
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
				1,
				10000000
			));

			let bytes = Bytes(vec![1u8, 2u8, 3u8]);

			// We are transferring asset 0, which we have instructed to be the relay asset
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::TransactThroughDerivative)
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
