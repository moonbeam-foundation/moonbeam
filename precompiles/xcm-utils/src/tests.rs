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
	ExtBuilder, PCall, PrecompilesValue, Runtime,
	TestAccount::{self, *},
	TestPrecompiles,
};

use codec::Encode;
use precompile_utils::{prelude::*, solidity, testing::*};
use sp_core::{H160, U256};
use xcm::prelude::*;

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn test_selector_enum() {
	assert!(PCall::multilocation_to_address_selectors().contains(&0x343b3e00));
	assert!(PCall::weight_message_selectors().contains(&0x25d54154));
	assert!(PCall::get_units_per_second_selectors().contains(&0x3f0f65db));
}

#[test]
fn modifiers() {
	ExtBuilder::default().build().execute_with(|| {
		let mut tester = PrecompilesModifierTester::new(precompiles(), Alice, Precompile);

		tester.test_view_modifier(PCall::multilocation_to_address_selectors());
		tester.test_view_modifier(PCall::weight_message_selectors());
		tester.test_view_modifier(PCall::get_units_per_second_selectors());
	});
}

#[test]
fn test_get_account_parent() {
	ExtBuilder::default().build().execute_with(|| {
		let input = PCall::multilocation_to_address {
			multilocation: MultiLocation::parent(),
		};

		let expected_address: H160 = TestAccount::Parent.into();

		precompiles()
			.prepare_test(Alice, Precompile, input)
			.expect_cost(1)
			.expect_no_logs()
			.execute_returns(
				EvmDataWriter::new()
					.write(Address(expected_address))
					.build(),
			);
	});
}

#[test]
fn test_get_account_sibling() {
	ExtBuilder::default().build().execute_with(|| {
		let input = PCall::multilocation_to_address {
			multilocation: MultiLocation {
				parents: 1,
				interior: Junctions::X1(Junction::Parachain(2000u32)),
			},
		};

		let expected_address: H160 = TestAccount::SiblingParachain(2000u32).into();

		precompiles()
			.prepare_test(Alice, Precompile, input)
			.expect_cost(1)
			.expect_no_logs()
			.execute_returns(
				EvmDataWriter::new()
					.write(Address(expected_address))
					.build(),
			);
	});
}

#[test]
fn test_weight_message() {
	ExtBuilder::default().build().execute_with(|| {
		let message: Vec<u8> = xcm::VersionedXcm::<()>::V2(Xcm(vec![ClearOrigin])).encode();

		let input = PCall::weight_message {
			message: message.into(),
		};

		precompiles()
			.prepare_test(Alice, Precompile, input)
			.expect_cost(0)
			.expect_no_logs()
			.execute_returns_encoded(1000u64);
	});
}

#[test]
fn test_get_units_per_second() {
	ExtBuilder::default().build().execute_with(|| {
		let input = PCall::get_units_per_second {
			multilocation: MultiLocation::parent(),
		};

		precompiles()
			.prepare_test(Alice, Precompile, input)
			.expect_cost(1)
			.expect_no_logs()
			.execute_returns_encoded(U256::from(1_000_000_000_000u128));
	});
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	for file in ["XcmUtils.sol"] {
		for solidity_fn in solidity::get_selectors(file) {
			assert_eq!(
				solidity_fn.compute_selector_hex(),
				solidity_fn.docs_selector,
				"documented selector for '{}' did not match for file '{}'",
				solidity_fn.signature(),
				file,
			);

			let selector = solidity_fn.compute_selector();
			if !PCall::supports_selector(selector) {
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
