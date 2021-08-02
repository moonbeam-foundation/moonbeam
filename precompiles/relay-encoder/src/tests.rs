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
	evm_test_context, precompile_address, ExtBuilder, Precompiles, TestAccount::Alice,
};
use crate::test_relay_runtime::TestEncoder;
use crate::StakeEncodeCall;
use crate::{AvailableStakeCalls, PrecompileOutput};
use pallet_evm::{ExitError, ExitSucceed, PrecompileSet};
use pallet_staking::RewardDestination;
use pallet_staking::ValidatorPrefs;
use precompile_utils::{Bytes, EvmDataWriter};
use sha3::{Digest, Keccak256};
use sp_core::{H256, U256};
use sp_runtime::Perbill;

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
		let bogus_selector = vec![1u8, 2u8, 3u8];

		// Expected result is an error stating there are too few bytes
		let expected_result = Some(Err(ExitError::Other(
			"tried to parse selector out of bounds".into(),
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
			"No relay wrapper method at given selector".into(),
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
fn test_encode_bond() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"encode_bond(uint256,uint256,uint8,uint256)")[0..4];

			// Construct data to read prop count
			let mut input_data = Vec::<u8>::from([0u8; 132]);
			input_data[0..4].copy_from_slice(&selector);
			input_data[4..36].copy_from_slice(&[1u8; 32]);

			let amount: U256 = 100u32.into();
			let mut buffer = [0u8; 32];
			amount.to_big_endian(&mut buffer);
			input_data[36..68].copy_from_slice(&buffer);

			let reward_dest: U256 = 2u32.into();
			let mut buffer2 = [0u8; 32];
			reward_dest.to_big_endian(&mut buffer2);
			input_data[68..100].copy_from_slice(&buffer2);

			input_data[100..132].copy_from_slice(&[0u8; 32]);

			let expected_bytes: Bytes = TestEncoder::encode_call(AvailableStakeCalls::Bond(
				[1u8; 32].into(),
				100u32.into(),
				RewardDestination::Controller,
			))
			.into();

			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_result
			);
		});
}

#[test]
fn test_encode_bond_more() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"encode_bond_extra(uint256)")[0..4];

			// Construct data to read prop count
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);
			let amount: U256 = 100u32.into();
			let mut buffer = [0u8; 32];
			amount.to_big_endian(&mut buffer);
			input_data[4..36].copy_from_slice(&buffer);

			let expected_bytes: Bytes =
				TestEncoder::encode_call(AvailableStakeCalls::BondExtra(100u32.into())).into();

			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_result
			);
		});
}

#[test]
fn test_encode_chill() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"encode_chill()")[0..4];

			// Construct data to read prop count
			let mut input_data = Vec::<u8>::from([0u8; 4]);
			input_data[0..4].copy_from_slice(&selector);

			let expected_bytes: Bytes = TestEncoder::encode_call(AvailableStakeCalls::Chill).into();

			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_result
			);
		});
}

#[test]
fn test_encode_nominate() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"encode_nominate(uint256[])")[0..4];

			let array: Vec<H256> = vec![[1u8; 32].into(), [2u8; 32].into()];
			let input = EvmDataWriter::new().write(array.clone()).build();

			let mut input_data = Vec::<u8>::from([0u8; 4]);
			input_data[0..4].copy_from_slice(&selector);

			input_data.extend_from_slice(input.as_ref());

			let expected_bytes: Bytes =
				TestEncoder::encode_call(AvailableStakeCalls::Nominate(vec![
					[1u8; 32].into(),
					[2u8; 32].into(),
				]))
				.into();

			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_result
			);
		});
}

#[test]
fn test_encode_rebond() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"encode_rebond(uint256)")[0..4];

			// Construct data to read prop count
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);

			let amount: U256 = 100u32.into();
			let mut buffer = [0u8; 32];
			amount.to_big_endian(&mut buffer);
			input_data[4..36].copy_from_slice(&buffer);

			// Ethereum style
			let expected_bytes: Bytes =
				TestEncoder::encode_call(AvailableStakeCalls::Rebond(100u128)).into();

			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_result
			);
		});
}

#[test]
fn test_encode_set_controller() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"encode_set_controller(uint256)")[0..4];

			// Construct data to read prop count
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);

			input_data[4..36].copy_from_slice(&[1u8; 32]);

			// Ethereum style
			let expected_bytes: Bytes =
				TestEncoder::encode_call(AvailableStakeCalls::SetController([1u8; 32].into()))
					.into();

			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_result
			);
		});
}

#[test]
fn test_encode_set_payee() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"encode_set_payee(uint8,uint256)")[0..4];

			// Construct data to read prop count
			let mut input_data = Vec::<u8>::from([0u8; 68]);
			input_data[0..4].copy_from_slice(&selector);

			let reward_dest: U256 = 2u32.into();
			let mut buffer = [0u8; 32];
			reward_dest.to_big_endian(&mut buffer);
			input_data[4..36].copy_from_slice(&buffer);

			input_data[36..68].copy_from_slice(&[0u8; 32]);

			// Ethereum style
			let expected_bytes: Bytes = TestEncoder::encode_call(AvailableStakeCalls::SetPayee(
				RewardDestination::Controller,
			))
			.into();

			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_result
			);
		});
}

#[test]
fn test_encode_unbond() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"encode_unbond(uint256)")[0..4];

			// Construct data to read prop count
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);

			let amount: U256 = 100u32.into();
			let mut buffer = [0u8; 32];
			amount.to_big_endian(&mut buffer);
			input_data[4..36].copy_from_slice(&buffer);

			// Ethereum style
			let expected_bytes: Bytes =
				TestEncoder::encode_call(AvailableStakeCalls::Unbond(100u32.into())).into();

			// Expected result is one
			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that no props have been opened.
			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_result
			);
		});
}

#[test]
fn test_encode_validate() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"encode_validate(uint256,bool)")[0..4];

			// Construct data to read prop count
			let mut input_data = Vec::<u8>::from([0u8; 68]);
			input_data[0..4].copy_from_slice(&selector);

			let per_bill: U256 = 100u32.into();
			let mut buffer = [0u8; 32];
			per_bill.to_big_endian(&mut buffer);
			input_data[4..36].copy_from_slice(&buffer);

			let blocked: U256 = 1u32.into();
			buffer = [0u8; 32];
			blocked.to_big_endian(&mut buffer);
			input_data[36..68].copy_from_slice(&buffer);

			// Ethereum style
			let expected_bytes: Bytes =
				TestEncoder::encode_call(AvailableStakeCalls::Validate(ValidatorPrefs {
					commission: Perbill::from_parts(100u32.into()),
					blocked: true,
				}))
				.into();

			// Expected result is one
			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that no props have been opened.
			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_result
			);
		});
}

#[test]
fn test_encode_withdraw_unbonded() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let selector = &Keccak256::digest(b"encode_withdraw_unbonded(uint32)")[0..4];

			// Construct data to read prop count
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&selector);

			let amount: U256 = 100u32.into();
			let mut buffer = [0u8; 32];
			amount.to_big_endian(&mut buffer);
			input_data[4..36].copy_from_slice(&buffer);

			// Ethereum style
			let expected_bytes: Bytes =
				TestEncoder::encode_call(AvailableStakeCalls::WithdrawUnbonded(100u32.into()))
					.into();

			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				Precompiles::execute(precompile_address(), &input_data, None, &evm_test_context()),
				expected_result
			);
		});
}
