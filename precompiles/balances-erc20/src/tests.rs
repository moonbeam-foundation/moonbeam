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

use crate::mock::*;
use crate::*;
use pallet_evm::PrecompileSet;
use precompile_utils::{
	error, EvmDataReader, EvmDataWriter, EvmResult, Gasometer, LogsBuilder, RuntimeHelper,
};

const SELECTOR_TOTAL_SUPPLY: &[u8; 4] = &[0x7c, 0x80, 0xaa, 0x9f];

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
		let bogus_selector = vec![1u8, 2u8, 3u8];

		// Expected result is an error stating there are too few bytes
		let expected_result = Some(Err(error("input must at least contain a selector")));

		assert_eq!(
			Precompiles::<Runtime>::execute(
				Account::Precompile.into(),
				&bogus_selector,
				None,
				&evm::Context {
					address: Account::Precompile.into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
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
		let expected_result = Some(Err(error("unknown selector")));

		assert_eq!(
			Precompiles::<Runtime>::execute(
				Account::Precompile.into(),
				&bogus_selector,
				None,
				&evm::Context {
					address: Account::Precompile.into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
			),
			expected_result
		);
	});
}

#[test]
fn total_supply_bad_input() {
	ExtBuilder::default().build().execute_with(|| {
		let data = EvmDataWriter::new()
			.write_bytes(SELECTOR_TOTAL_SUPPLY)
			.write_bool(true)
			.build(); // extra argument

		// Expected result is an error stating there are such a selector does not exist
		let expected_result = Some(Err(error("input doesn't match expected length")));

		assert_eq!(
			Precompiles::<Runtime>::execute(
				Account::Precompile.into(),
				&data,
				None,
				&evm::Context {
					address: Account::Precompile.into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
			),
			expected_result
		);
	});
}
