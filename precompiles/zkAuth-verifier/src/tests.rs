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
use crate::encoded_receipt::encoded_example_receipt;
use crate::mock::{ExtBuilder, PCall, Precompiles, PrecompilesValue, Runtime};
use crate::*;
use precompile_utils::testing::*;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	check_precompile_implements_solidity_interfaces(&["ZkAuth.sol"], PCall::supports_selector)
}

#[test]
fn selectors() {
	assert!(PCall::verify_proof_selectors().contains(&0x55c265fe));
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
fn test_mocked_verification() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			crate::storage::ImageId::set(Some(JWT_VALIDATOR_ID));
			let receipt = encoded_example_receipt();

			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::verify_proof {
						receipt: receipt.clone().into(),
					},
				)
				.expect_cost(0)
				.expect_no_logs()
				.execute_returns(UnboundedBytes::from(vec![1u8, 2u8, 3u8]));
		});
}
