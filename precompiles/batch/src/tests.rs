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
	Account::{Alice, Bob, Charlie, Precompile},
	ExtBuilder, PrecompilesValue, Runtime, TestPrecompiles,
};
use crate::Action;
use evm::ExitReason;
use fp_evm::{ExitError, ExitRevert, ExitSucceed};
use precompile_utils::{testing::*, Address, Bytes, EvmDataWriter, LogsBuilder};
use sp_core::{H256, U256};

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selectors() {
	assert_eq!(Action::BatchSome as u32, 0x9205a0ba);
	assert_eq!(Action::BatchSomeUntilFailure as u32, 0xc803ba9a);
	assert_eq!(Action::BatchAll as u32, 0x2d41531c);
}

#[test]
fn batch_some_empty() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				EvmDataWriter::new_with_selector(Action::BatchSome)
					.write::<Vec<Address>>(vec![])
					.write::<Vec<U256>>(vec![])
					.write::<Vec<Bytes>>(vec![])
					.build(),
			)
			.with_subcall_handle(|Subcall { .. }| panic!("there should be no subcall"))
			.execute_returns(
				EvmDataWriter::new()
					.write(U256::from(0u8))
					.write::<Vec<Bytes>>(vec![])
					.build(),
			)
	})
}

#[test]
fn batch_some_until_failure_empty() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				EvmDataWriter::new_with_selector(Action::BatchSomeUntilFailure)
					.write::<Vec<Address>>(vec![])
					.write::<Vec<U256>>(vec![])
					.write::<Vec<Bytes>>(vec![])
					.build(),
			)
			.with_subcall_handle(|Subcall { .. }| panic!("there should be no subcall"))
			.execute_returns(
				EvmDataWriter::new()
					.write(U256::from(0u8))
					.write::<Vec<Bytes>>(vec![])
					.build(),
			)
	})
}

#[test]
fn batch_all_empty() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				EvmDataWriter::new_with_selector(Action::BatchAll)
					.write::<Vec<Address>>(vec![])
					.write::<Vec<U256>>(vec![])
					.write::<Vec<Bytes>>(vec![])
					.build(),
			)
			.with_subcall_handle(|Subcall { .. }| panic!("there should be no subcall"))
			.execute_returns(EvmDataWriter::new().write::<Vec<Bytes>>(vec![]).build())
	})
}

fn batch_returns(
	precompiles: &TestPrecompiles<Runtime>,
	action: Action,
) -> PrecompilesTester<TestPrecompiles<Runtime>> {
	let mut counter = 0;
	precompiles
		.prepare_test(
			Alice,
			Precompile,
			EvmDataWriter::new_with_selector(action)
				.write(vec![Address(Bob.into()), Address(Charlie.into())])
				.write(vec![U256::from(1u8), U256::from(2u8)])
				.write(vec![
					Bytes::from(b"one".as_slice()),
					Bytes::from(b"two".as_slice()),
				])
				.build(),
		)
		.with_target_gas(Some(100_000))
		.with_subcall_handle(
			move |Subcall {
			          address,
			          transfer,
			          input,
			          target_gas,
			          is_static,
			          context,
			      }| {
				// Called from the precompile caller.
				assert_eq!(context.caller, Alice.into());
				assert_eq!(is_static, false);

				match address {
					a if a == Bob.into() => {
						assert_eq!(counter, 0, "this is the first call");
						counter += 1;

						assert_eq!(target_gas, Some(100_000), "batch forward all gas");
						let transfer = transfer.expect("there is a transfer");
						assert_eq!(transfer.source, Alice.into());
						assert_eq!(transfer.target, Bob.into());
						assert_eq!(transfer.value, 1u8.into());

						assert_eq!(context.address, Bob.into());
						assert_eq!(context.apparent_value, 1u8.into());

						assert_eq!(&input, b"one");

						SubcallOutput {
							reason: ExitReason::Succeed(ExitSucceed::Returned),
							output: b"ONE".to_vec(),
							cost: 13,
							logs: vec![
								LogsBuilder::new(Bob.into()).log1(H256::repeat_byte(0x11), vec![])
							],
						}
					}
					a if a == Charlie.into() => {
						assert_eq!(counter, 1, "this is the second call");
						counter += 1;

						assert_eq!(target_gas, Some(100_000 - 13), "batch forward all gas");
						let transfer = transfer.expect("there is a transfer");
						assert_eq!(transfer.source, Alice.into());
						assert_eq!(transfer.target, Charlie.into());
						assert_eq!(transfer.value, 2u8.into());

						assert_eq!(context.address, Charlie.into());
						assert_eq!(context.apparent_value, 2u8.into());

						assert_eq!(&input, b"two");

						SubcallOutput {
							reason: ExitReason::Succeed(ExitSucceed::Returned),
							output: b"TWO".to_vec(),
							cost: 17,
							logs: vec![LogsBuilder::new(Charlie.into())
								.log1(H256::repeat_byte(0x22), vec![])],
						}
					}
					_ => panic!("unexpected subcall"),
				}
			},
		)
		.expect_log(LogsBuilder::new(Bob.into()).log1(H256::repeat_byte(0x11), vec![]))
		.expect_log(LogsBuilder::new(Charlie.into()).log1(H256::repeat_byte(0x22), vec![]))
		.expect_cost(13 + 17)
}

#[test]
fn batch_some_returns() {
	ExtBuilder::default().build().execute_with(|| {
		batch_returns(&precompiles(), Action::BatchSome).execute_returns(
			EvmDataWriter::new()
				.write(U256::from(2u8)) // successfully performed the 2 subcalls.
				.write::<Vec<Bytes>>(vec![
					Bytes(b"ONE".as_slice().into()),
					Bytes(b"TWO".as_slice().into()),
				])
				.build(),
		)
	})
}

#[test]
fn batch_some_until_failure_returns() {
	ExtBuilder::default().build().execute_with(|| {
		batch_returns(&precompiles(), Action::BatchSomeUntilFailure).execute_returns(
			EvmDataWriter::new()
				.write(U256::from(2u8)) // successfully performed the 2 subcalls.
				.write::<Vec<Bytes>>(vec![
					Bytes(b"ONE".as_slice().into()),
					Bytes(b"TWO".as_slice().into()),
				])
				.build(),
		)
	})
}

#[test]
fn batch_all_returns() {
	ExtBuilder::default().build().execute_with(|| {
		batch_returns(&precompiles(), Action::BatchAll).execute_returns(
			EvmDataWriter::new()
				.write::<Vec<Bytes>>(vec![
					Bytes(b"ONE".as_slice().into()),
					Bytes(b"TWO".as_slice().into()),
				])
				.build(),
		)
	})
}

fn batch_out_of_gas(
	precompiles: &TestPrecompiles<Runtime>,
	action: Action,
) -> PrecompilesTester<TestPrecompiles<Runtime>> {
	precompiles
		.prepare_test(
			Alice,
			Precompile,
			EvmDataWriter::new_with_selector(action)
				.write(vec![Address(Bob.into())])
				.write(vec![U256::from(1u8)])
				.write(vec![Bytes::from(b"one".as_slice())])
				.build(),
		)
		.with_target_gas(Some(10_000))
		.with_subcall_handle(
			move |Subcall {
			          address,
			          transfer,
			          input,
			          target_gas,
			          is_static,
			          context,
			      }| {
				// Called from the precompile caller.
				assert_eq!(context.caller, Alice.into());
				assert_eq!(is_static, false);

				match address {
					a if a == Bob.into() => {
						assert_eq!(target_gas, Some(10_000), "batch forward all gas");
						let transfer = transfer.expect("there is a transfer");
						assert_eq!(transfer.source, Alice.into());
						assert_eq!(transfer.target, Bob.into());
						assert_eq!(transfer.value, 1u8.into());

						assert_eq!(context.address, Bob.into());
						assert_eq!(context.apparent_value, 1u8.into());

						assert_eq!(&input, b"one");

						SubcallOutput {
							reason: ExitReason::Error(ExitError::OutOfGas),
							output: b"ONE".to_vec(),
							cost: 11_000,
							logs: vec![],
						}
					}
					_ => panic!("unexpected subcall"),
				}
			},
		)
}

#[test]
fn batch_some_out_of_gas() {
	ExtBuilder::default().build().execute_with(|| {
		batch_out_of_gas(&precompiles(), Action::BatchSome).execute_returns(
			EvmDataWriter::new()
				.write(U256::from(0u8))
				.write::<Vec<Bytes>>(vec![Bytes(vec![])])
				.build(),
		)
	})
}

#[test]
fn batch_some_until_failure_out_of_gas() {
	ExtBuilder::default().build().execute_with(|| {
		batch_out_of_gas(&precompiles(), Action::BatchSomeUntilFailure).execute_returns(
			EvmDataWriter::new()
				.write(U256::from(0u8))
				.write::<Vec<Bytes>>(vec![Bytes(vec![])])
				.build(),
		)
	})
}

#[test]
fn batch_all_out_of_gas() {
	ExtBuilder::default().build().execute_with(|| {
		batch_out_of_gas(&precompiles(), Action::BatchAll).execute_error(ExitError::OutOfGas)
	})
}

fn batch_incomplete(
	precompiles: &TestPrecompiles<Runtime>,
	action: Action,
) -> PrecompilesTester<TestPrecompiles<Runtime>> {
	let mut counter = 0;
	precompiles
		.prepare_test(
			Alice,
			Precompile,
			EvmDataWriter::new_with_selector(action)
				.write(vec![
					Address(Bob.into()),
					Address(Charlie.into()),
					Address(Alice.into()),
				])
				.write(vec![U256::from(1u8), U256::from(2u8)])
				.write(vec![Bytes::from(b"one".as_slice())])
				.build(),
		)
		.with_target_gas(Some(100_000))
		.with_subcall_handle(
			move |Subcall {
			          address,
			          transfer,
			          input,
			          target_gas,
			          is_static,
			          context,
			      }| {
				// Called from the precompile caller.
				assert_eq!(context.caller, Alice.into());
				assert_eq!(is_static, false);

				match address {
					a if a == Bob.into() => {
						assert_eq!(counter, 0, "this is the first call");
						counter += 1;

						assert_eq!(target_gas, Some(100_000), "batch forward all gas");
						let transfer = transfer.expect("there is a transfer");
						assert_eq!(transfer.source, Alice.into());
						assert_eq!(transfer.target, Bob.into());
						assert_eq!(transfer.value, 1u8.into());

						assert_eq!(context.address, Bob.into());
						assert_eq!(context.apparent_value, 1u8.into());

						assert_eq!(&input, b"one");

						SubcallOutput {
							reason: ExitReason::Succeed(ExitSucceed::Returned),
							output: b"ONE".to_vec(),
							cost: 13,
							logs: vec![
								LogsBuilder::new(Bob.into()).log1(H256::repeat_byte(0x11), vec![])
							],
						}
					}
					a if a == Charlie.into() => {
						assert_eq!(counter, 1, "this is the second call");
						counter += 1;

						assert_eq!(target_gas, Some(100_000 - 13), "batch forward all gas");
						let transfer = transfer.expect("there is a transfer");
						assert_eq!(transfer.source, Alice.into());
						assert_eq!(transfer.target, Charlie.into());
						assert_eq!(transfer.value, 2u8.into());

						assert_eq!(context.address, Charlie.into());
						assert_eq!(context.apparent_value, 2u8.into());

						assert_eq!(&input, b"");

						SubcallOutput {
							reason: ExitReason::Revert(ExitRevert::Reverted),
							output: b"Revert message".to_vec(),
							cost: 17,
							logs: vec![],
						}
					}
					a if a == Alice.into() => {
						assert_eq!(counter, 2, "this is the third call");
						counter += 1;

						assert_eq!(target_gas, Some(100_000 - 13 - 17), "batch forward all gas");
						assert!(transfer.is_none());

						assert_eq!(context.address, Alice.into());
						assert_eq!(context.apparent_value, 0u8.into());

						assert_eq!(&input, b"");

						SubcallOutput {
							reason: ExitReason::Succeed(ExitSucceed::Returned),
							output: b"THREE".to_vec(),
							cost: 19,
							logs: vec![
								LogsBuilder::new(Bob.into()).log1(H256::repeat_byte(0x33), vec![])
							],
						}
					}
					_ => panic!("unexpected subcall"),
				}
			},
		)
}

#[test]
fn batch_some_incomplete() {
	ExtBuilder::default().build().execute_with(|| {
		batch_incomplete(&precompiles(), Action::BatchSome).execute_returns(
			EvmDataWriter::new()
				.write(U256::from(2u8)) // 2 out of 3 succeeded
				.write::<Vec<Bytes>>(vec![
					Bytes(b"ONE".to_vec()),
					Bytes(b"Revert message".to_vec()),
					Bytes(b"THREE".to_vec()),
				])
				.build(),
		)
	})
}

#[test]
fn batch_some_until_failure_incomplete() {
	ExtBuilder::default().build().execute_with(|| {
		batch_incomplete(&precompiles(), Action::BatchSomeUntilFailure).execute_returns(
			EvmDataWriter::new()
				.write(U256::from(1u8)) // failed at index 1
				.write::<Vec<Bytes>>(vec![
					Bytes(b"ONE".to_vec()),
					Bytes(b"Revert message".to_vec()),
				])
				.build(),
		)
	})
}

#[test]
fn batch_all_incomplete() {
	ExtBuilder::default().build().execute_with(|| {
		batch_incomplete(&precompiles(), Action::BatchAll)
			.execute_reverts(|output| output == b"Revert message")
	})
}
