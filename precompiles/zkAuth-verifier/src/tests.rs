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

// TODO: finish test
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
						receipt: receipt.into(),
						to: Address(Alice.into()),
						value: U256::from(1u8),
						call_data: b"call".into(),
						gas_limit: 0u64,
					},
				)
				.with_subcall_handle(move |subcall| {
					let Subcall {
						address,
						transfer,
						input,
						target_gas,
						is_static,
						context,
					} = subcall;

					// Called from the precompile caller.
					assert_eq!(context.caller, Alice.into());
					assert_eq!(is_static, false);

					SubcallOutput::succeed()
				})
				.expect_cost(37600)
				.expect_no_logs()
				.execute_returns(());
		});
}
