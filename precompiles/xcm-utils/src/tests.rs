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
	ExtBuilder, PrecompilesValue, Runtime,
	TestAccount::{self, *},
	TestPrecompiles,
};
use crate::Action;

use precompile_utils::{prelude::*, testing::*};
use sp_core::H160;
use xcm::latest::{Junction, Junctions, MultiLocation};

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn test_selector_enum() {
	assert_eq!(Action::MultiLocationToAddress as u32, 0x343b3e00);
}

#[test]
fn test_get_account_parent() {
	ExtBuilder::default().build().execute_with(|| {
		let input = EvmDataWriter::new_with_selector(Action::MultiLocationToAddress)
			.write(MultiLocation::parent())
			.build();

		let expected_address: H160 = TestAccount::Parent.into();
		// Expected result is zero
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
		let input = EvmDataWriter::new_with_selector(Action::MultiLocationToAddress)
			.write(MultiLocation {
				parents: 1,
				interior: Junctions::X1(Junction::Parachain(2000u32)),
			})
			.build();

		let expected_address: H160 = TestAccount::SiblingParachain(2000u32).into();
		// Expected result is zero
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
