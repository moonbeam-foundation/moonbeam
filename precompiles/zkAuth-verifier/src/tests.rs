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
use sp_runtime::Perbill;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn test_mocked_verification() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			let receipt = encoded_example_receipt();

			precompiles()
				.prepare_test(Alice, Precompile1, PCall::verify_proof { receipt })
				.expect_cost(1000)
				.expect_no_logs()
				.execute_returns(());
		});
}
