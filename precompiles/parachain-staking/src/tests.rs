// Copyright 2019-2021 PureStake Inc.
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
	events, evm_test_context, precompile_address, roll_to, set_points, Call, ExtBuilder, Origin,
	ParachainStaking, Precompiles, TestAccount,
};
use crate::PrecompileOutput;
use frame_support::{assert_ok, dispatch::Dispatchable};
use pallet_evm::Call as EvmCall;
use pallet_evm::{ExitError, ExitSucceed, PrecompileSet};
use parachain_staking::{Call as StakingCall, Event as StakingEvent};
use precompile_utils::OutputBuilder;
use sha3::{Digest, Keccak256};
use sp_core::U256;

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
		let bogus_selector = vec![1u8, 2u8, 3u8];

		// Expected result is an error stating there are too few bytes
		let expected_result = Some(Err(ExitError::Other(
			"input must at least contain a selector".into(),
		)));

		assert_eq!(
			Precompiles::execute(
				precompile_address(),
				&bogus_selector,
				None,
				&evm_test_context(),
			),
			expected_result
		);
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		let bogus_selector = vec![1u8, 2u8, 3u8, 4u8];

		// Expected result is an error stating there are such a selector does not exist
		let expected_result = Some(Err(ExitError::Other(
			"No parachain-staking wrapper method at given selector".into(),
		)));

		assert_eq!(
			Precompiles::execute(
				precompile_address(),
				&bogus_selector,
				None,
				&evm_test_context(),
			),
			expected_result
		);
	});
}

#[test]
fn min_nomination_works() {
	ExtBuilder::default().build().execute_with(|| {
		let selector = &Keccak256::digest(b"min_nomination()")[0..4];

		// Construct data to read minimum nomination constant
		let mut input_data = Vec::<u8>::from([0u8; 4]);
		input_data[0..4].copy_from_slice(&selector);

		// Expected result is 3
		let expected_one_result = Some(Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: OutputBuilder::new().write_u256(3u32).build(),
			cost: Default::default(),
			logs: Default::default(),
		}));

		// Assert that no props have been opened.
		assert_eq!(
			Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
			expected_one_result
		);
	});
}

#[test]
fn points_works() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1_000)])
		.with_candidates(vec![(TestAccount::Alice, 1_000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"points(uint256)")[0..4];

			// Construct data to read points for round 1
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			U256::one().to_big_endian(&mut input_data[4..36]);

			// Expected result is 100 points
			let expected_one_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: OutputBuilder::new().write_u256(100u32).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			set_points(1u32, TestAccount::Alice, 100);

			// Assert that there are total 100 points in round 1
			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_one_result
			);
		});
}
