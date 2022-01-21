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

use std::assert_matches::assert_matches;

use crate::mock::{
	evm_test_context, precompile_address, ExtBuilder, PrecompilesValue, Runtime,
	TestAccount::Alice, TestPrecompiles,
};
use crate::test_relay_runtime::TestEncoder;
use crate::StakeEncodeCall;
use crate::*;
use crate::{AvailableStakeCalls, PrecompileOutput};
use fp_evm::PrecompileFailure;
use num_enum::TryFromPrimitive;
use pallet_evm::{ExitSucceed, PrecompileSet};
use pallet_staking::RewardDestination;
use pallet_staking::ValidatorPrefs;
use precompile_utils::{Bytes, EvmDataWriter};
use sha3::{Digest, Keccak256};
use sp_core::{H256, U256};
use sp_runtime::Perbill;

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn test_selector_enum() {
	let mut buffer = [0u8; 4];
	buffer.copy_from_slice(&Keccak256::digest(b"encode_bond(uint256,uint256,bytes)")[0..4]);

	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::EncodeBond,
	);
	buffer.copy_from_slice(&Keccak256::digest(b"encode_bond_extra(uint256)")[0..4]);

	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::EncodeBondExtra,
	);

	buffer.copy_from_slice(&Keccak256::digest(b"encode_chill()")[0..4]);

	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::EncodeChill,
	);

	buffer.copy_from_slice(&Keccak256::digest(b"encode_nominate(uint256[])")[0..4]);

	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::EncodeNominate,
	);

	buffer.copy_from_slice(&Keccak256::digest(b"encode_rebond(uint256)")[0..4]);

	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::EncodeRebond,
	);

	buffer.copy_from_slice(&Keccak256::digest(b"encode_set_controller(uint256)")[0..4]);

	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::EncodeSetController,
	);

	buffer.copy_from_slice(&Keccak256::digest(b"encode_set_payee(bytes)")[0..4]);

	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::EncodeSetPayee,
	);

	buffer.copy_from_slice(&Keccak256::digest(b"encode_unbond(uint256)")[0..4]);

	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::EncodeUnbond,
	);

	buffer.copy_from_slice(&Keccak256::digest(b"encode_validate(uint256,bool)")[0..4]);

	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::EncodeValidate,
	);

	buffer.copy_from_slice(&Keccak256::digest(b"encode_withdraw_unbonded(uint32)")[0..4]);

	assert_eq!(
		Action::try_from_primitive(u32::from_be_bytes(buffer)).unwrap(),
		Action::EncodeWithdrawUnbonded,
	);
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
		let bogus_selector = vec![1u8, 2u8, 3u8];

		assert_matches!(
			precompiles().execute(
				precompile_address(),
				&bogus_selector,
				None,
				&evm_test_context(),
				false,
			),
			Some(Err(PrecompileFailure::Revert { output, .. }))
			if &output == b"tried to parse selector out of bounds"
		);
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		let bogus_selector = vec![1u8, 2u8, 3u8, 4u8];

		assert_matches!(
			precompiles().execute(
				precompile_address(),
				&bogus_selector,
				None,
				&evm_test_context(),
				false,
			),
			Some(Err(PrecompileFailure::Revert { output, ..}))
			if &output == b"unknown selector"
		);
	});
}

#[test]
fn test_encode_bond() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let controller_address: H256 = [1u8; 32].into();
			let amount: U256 = 100u32.into();

			let input_data = EvmDataWriter::new_with_selector(Action::EncodeBond)
				.write(controller_address)
				.write(amount)
				.write(RewardDestinationWrapper(RewardDestination::Controller))
				.build();

			let expected_bytes: Bytes = TestEncoder::encode_call(AvailableStakeCalls::Bond(
				[1u8; 32].into(),
				100u32.into(),
				RewardDestination::Controller,
			))
			.as_slice()
			.into();

			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
			let amount: U256 = 100u32.into();
			let input_data = EvmDataWriter::new_with_selector(Action::EncodeBondExtra)
				.write(amount)
				.build();

			let expected_bytes: Bytes =
				TestEncoder::encode_call(AvailableStakeCalls::BondExtra(100u32.into()))
					.as_slice()
					.into();

			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
			let input_data = EvmDataWriter::new_with_selector(Action::EncodeChill).build();

			let expected_bytes: Bytes = TestEncoder::encode_call(AvailableStakeCalls::Chill)
				.as_slice()
				.into();

			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
			let array: Vec<H256> = vec![[1u8; 32].into(), [2u8; 32].into()];

			let input_data = EvmDataWriter::new_with_selector(Action::EncodeNominate)
				.write(array)
				.build();

			let expected_bytes: Bytes =
				TestEncoder::encode_call(AvailableStakeCalls::Nominate(vec![
					[1u8; 32].into(),
					[2u8; 32].into(),
				]))
				.as_slice()
				.into();

			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
			let amount: U256 = 100u32.into();

			let input_data = EvmDataWriter::new_with_selector(Action::EncodeRebond)
				.write(amount)
				.build();

			let expected_bytes: Bytes =
				TestEncoder::encode_call(AvailableStakeCalls::Rebond(100u128))
					.as_slice()
					.into();

			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
			let controller: H256 = [1u8; 32].into();

			let input_data = EvmDataWriter::new_with_selector(Action::EncodeSetController)
				.write(controller)
				.build();

			let expected_bytes: Bytes =
				TestEncoder::encode_call(AvailableStakeCalls::SetController([1u8; 32].into()))
					.as_slice()
					.into();

			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
			let input_data = EvmDataWriter::new_with_selector(Action::EncodeSetPayee)
				.write(RewardDestinationWrapper(RewardDestination::Controller))
				.build();

			let expected_bytes: Bytes = TestEncoder::encode_call(AvailableStakeCalls::SetPayee(
				RewardDestination::Controller,
			))
			.as_slice()
			.into();

			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
			let amount: U256 = 100u32.into();

			let input_data = EvmDataWriter::new_with_selector(Action::EncodeUnbond)
				.write(amount)
				.build();

			let expected_bytes: Bytes =
				TestEncoder::encode_call(AvailableStakeCalls::Unbond(100u32.into()))
					.as_slice()
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
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
			let amount: U256 = 100u32.into();
			let blocked = true;

			let input_data = EvmDataWriter::new_with_selector(Action::EncodeValidate)
				.write(amount)
				.write(blocked)
				.build();

			let expected_bytes: Bytes =
				TestEncoder::encode_call(AvailableStakeCalls::Validate(ValidatorPrefs {
					commission: Perbill::from_parts(100u32.into()),
					blocked: true,
				}))
				.as_slice()
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
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
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
			let amount: U256 = 100u32.into();

			let input_data = EvmDataWriter::new_with_selector(Action::EncodeWithdrawUnbonded)
				.write(amount)
				.build();

			// Ethereum style
			let expected_bytes: Bytes =
				TestEncoder::encode_call(AvailableStakeCalls::WithdrawUnbonded(100u32.into()))
					.as_slice()
					.into();

			let expected_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: EvmDataWriter::new().write(expected_bytes).build(),
				cost: Default::default(),
				logs: Default::default(),
			}));

			assert_eq!(
				precompiles().execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
					false
				),
				expected_result
			);
		});
}
