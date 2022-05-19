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
	balance, setup_revert_contract,
	Account::{Alice, Bob, Charlie, David, Precompile, Revert},
	Call, ExtBuilder, Origin, PrecompilesValue, Runtime, TestPrecompiles,
};
use crate::{
	log_subcall_failed, log_subcall_succeeded, Action, LOG_SUBCALL_FAILED, LOG_SUBCALL_SUCCEEDED,
};
use evm::ExitReason;
use fp_evm::{ExitError, ExitRevert, ExitSucceed};
use frame_support::{assert_ok, dispatch::Dispatchable};
use pallet_evm::Call as EvmCall;
use precompile_utils::{testing::*, Address, Bytes, EvmDataWriter, LogExt, LogsBuilder};
use sp_core::{H160, H256, U256};

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

fn evm_call(from: impl Into<H160>, input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: from.into(),
		target: Precompile.into(),
		input,
		value: U256::zero(), // No value sent in EVM
		gas_limit: u64::max_value(),
		max_fee_per_gas: 0.into(),
		max_priority_fee_per_gas: Some(U256::zero()),
		nonce: None, // Use the next nonce
		access_list: Vec::new(),
	}
}

fn costs(_: Action) -> (u64, u64) {
	let return_log_cost = log_subcall_failed(Precompile, 0).compute_cost().unwrap();
	let gas_reserve = return_log_cost + 1;
	(return_log_cost, gas_reserve)
}

#[test]
fn selectors() {
	assert_eq!(Action::BatchSome as u32, 0x79df4b9c);
	assert_eq!(Action::BatchSomeUntilFailure as u32, 0xcf0491c7);
	assert_eq!(Action::BatchAll as u32, 0x96e292b8);
	assert_eq!(
		LOG_SUBCALL_FAILED,
		hex_literal::hex!("dbc5d06f4f877f959b1ff12d2161cdd693fa8e442ee53f1790b2804b24881f05")
	);
	assert_eq!(
		LOG_SUBCALL_SUCCEEDED,
		hex_literal::hex!("bf855484633929c3d6688eb3caf8eff910fb4bef030a8d7dbc9390d26759714d")
	);
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
					.write::<Vec<U256>>(vec![])
					.build(),
			)
			.with_subcall_handle(|Subcall { .. }| panic!("there should be no subcall"))
			.execute_returns(Vec::new())
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
					.write::<Vec<U256>>(vec![])
					.build(),
			)
			.with_subcall_handle(|Subcall { .. }| panic!("there should be no subcall"))
			.execute_returns(Vec::new())
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
					.write::<Vec<U256>>(vec![])
					.build(),
			)
			.with_subcall_handle(|Subcall { .. }| panic!("there should be no subcall"))
			.execute_returns(Vec::new())
	})
}

fn batch_returns(
	precompiles: &TestPrecompiles<Runtime>,
	action: Action,
) -> PrecompilesTester<TestPrecompiles<Runtime>> {
	let mut counter = 0;

	let (return_log_cost, gas_reserve) = costs(action);

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
				.write::<Vec<U256>>(vec![])
				.write(true)
				.build(),
		)
		.with_target_gas(Some(100_000))
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

			match address {
				a if a == Bob.into() => {
					assert_eq!(counter, 0, "this is the first call");
					counter += 1;

					assert_eq!(
						target_gas,
						Some(100_000 - gas_reserve),
						"batch forward all gas"
					);
					let transfer = transfer.expect("there is a transfer");
					assert_eq!(transfer.source, Alice.into());
					assert_eq!(transfer.target, Bob.into());
					assert_eq!(transfer.value, 1u8.into());

					assert_eq!(context.address, Bob.into());
					assert_eq!(context.apparent_value, 1u8.into());

					assert_eq!(&input, b"one");

					SubcallOutput {
						reason: ExitReason::Succeed(ExitSucceed::Returned),
						output: Vec::new(),
						cost: 13,
						logs: vec![
							LogsBuilder::new(Bob.into()).log1(H256::repeat_byte(0x11), vec![])
						],
					}
				}
				a if a == Charlie.into() => {
					assert_eq!(counter, 1, "this is the second call");
					counter += 1;

					assert_eq!(
						target_gas,
						Some(100_000 - 13 - gas_reserve - return_log_cost),
						"batch forward all gas"
					);
					let transfer = transfer.expect("there is a transfer");
					assert_eq!(transfer.source, Alice.into());
					assert_eq!(transfer.target, Charlie.into());
					assert_eq!(transfer.value, 2u8.into());

					assert_eq!(context.address, Charlie.into());
					assert_eq!(context.apparent_value, 2u8.into());

					assert_eq!(&input, b"two");

					SubcallOutput {
						reason: ExitReason::Succeed(ExitSucceed::Returned),
						output: Vec::new(),
						cost: 17,
						logs: vec![
							LogsBuilder::new(Charlie.into()).log1(H256::repeat_byte(0x22), vec![])
						],
					}
				}
				_ => panic!("unexpected subcall"),
			}
		})
		.expect_cost(13 + 17 + return_log_cost * 2)
}

#[test]
fn batch_some_returns() {
	ExtBuilder::default().build().execute_with(|| {
		batch_returns(&precompiles(), Action::BatchSome)
			.expect_log(LogsBuilder::new(Bob.into()).log1(H256::repeat_byte(0x11), vec![]))
			.expect_log(log_subcall_succeeded(Precompile, 0))
			.expect_log(LogsBuilder::new(Charlie.into()).log1(H256::repeat_byte(0x22), vec![]))
			.expect_log(log_subcall_succeeded(Precompile, 1))
			.execute_returns(Vec::new())
	})
}

#[test]
fn batch_some_until_failure_returns() {
	ExtBuilder::default().build().execute_with(|| {
		batch_returns(&precompiles(), Action::BatchSomeUntilFailure)
			.expect_log(LogsBuilder::new(Bob.into()).log1(H256::repeat_byte(0x11), vec![]))
			.expect_log(log_subcall_succeeded(Precompile, 0))
			.expect_log(LogsBuilder::new(Charlie.into()).log1(H256::repeat_byte(0x22), vec![]))
			.expect_log(log_subcall_succeeded(Precompile, 1))
			.execute_returns(Vec::new())
	})
}

#[test]
fn batch_all_returns() {
	ExtBuilder::default().build().execute_with(|| {
		batch_returns(&precompiles(), Action::BatchAll)
			.expect_log(LogsBuilder::new(Bob.into()).log1(H256::repeat_byte(0x11), vec![]))
			.expect_log(log_subcall_succeeded(Precompile, 0))
			.expect_log(LogsBuilder::new(Charlie.into()).log1(H256::repeat_byte(0x22), vec![]))
			.expect_log(log_subcall_succeeded(Precompile, 1))
			.execute_returns(Vec::new())
	})
}

fn batch_out_of_gas(
	precompiles: &TestPrecompiles<Runtime>,
	action: Action,
) -> PrecompilesTester<TestPrecompiles<Runtime>> {
	let (_, gas_reserve) = costs(action);

	precompiles
		.prepare_test(
			Alice,
			Precompile,
			EvmDataWriter::new_with_selector(action)
				.write(vec![Address(Bob.into())])
				.write(vec![U256::from(1u8)])
				.write(vec![Bytes::from(b"one".as_slice())])
				.write::<Vec<U256>>(vec![])
				.build(),
		)
		.with_target_gas(Some(50_000))
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

			match address {
				a if a == Bob.into() => {
					assert_eq!(
						target_gas,
						Some(50_000 - gas_reserve),
						"batch forward all gas"
					);
					let transfer = transfer.expect("there is a transfer");
					assert_eq!(transfer.source, Alice.into());
					assert_eq!(transfer.target, Bob.into());
					assert_eq!(transfer.value, 1u8.into());

					assert_eq!(context.address, Bob.into());
					assert_eq!(context.apparent_value, 1u8.into());

					assert_eq!(&input, b"one");

					SubcallOutput {
						reason: ExitReason::Error(ExitError::OutOfGas),
						output: Vec::new(),
						cost: 11_000,
						logs: vec![],
					}
				}
				_ => panic!("unexpected subcall"),
			}
		})
}

#[test]
fn batch_some_out_of_gas() {
	ExtBuilder::default().build().execute_with(|| {
		batch_out_of_gas(&precompiles(), Action::BatchSome)
			.expect_log(log_subcall_failed(Precompile, 0))
			.execute_returns(Vec::new())
	})
}

#[test]
fn batch_some_until_failure_out_of_gas() {
	ExtBuilder::default().build().execute_with(|| {
		batch_out_of_gas(&precompiles(), Action::BatchSomeUntilFailure)
			.expect_log(log_subcall_failed(Precompile, 0))
			.execute_returns(Vec::new())
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

	let (return_log_cost, gas_reserve) = costs(action);

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
				.write::<Vec<U256>>(vec![])
				.build(),
		)
		.with_target_gas(Some(100_000))
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

			match address {
				a if a == Bob.into() => {
					assert_eq!(counter, 0, "this is the first call");
					counter += 1;

					assert_eq!(
						target_gas,
						Some(100_000 - gas_reserve),
						"batch forward all gas"
					);
					let transfer = transfer.expect("there is a transfer");
					assert_eq!(transfer.source, Alice.into());
					assert_eq!(transfer.target, Bob.into());
					assert_eq!(transfer.value, 1u8.into());

					assert_eq!(context.address, Bob.into());
					assert_eq!(context.apparent_value, 1u8.into());

					assert_eq!(&input, b"one");

					SubcallOutput {
						reason: ExitReason::Succeed(ExitSucceed::Returned),
						output: Vec::new(),
						cost: 13,
						logs: vec![
							LogsBuilder::new(Bob.into()).log1(H256::repeat_byte(0x11), vec![])
						],
					}
				}
				a if a == Charlie.into() => {
					assert_eq!(counter, 1, "this is the second call");
					counter += 1;

					assert_eq!(
						target_gas,
						Some(100_000 - 13 - gas_reserve - return_log_cost),
						"batch forward all gas"
					);
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

					assert_eq!(
						target_gas,
						Some(100_000 - 13 - 17 - gas_reserve - return_log_cost * 2),
						"batch forward all gas"
					);
					assert!(transfer.is_none());

					assert_eq!(context.address, Alice.into());
					assert_eq!(context.apparent_value, 0u8.into());

					assert_eq!(&input, b"");

					SubcallOutput {
						reason: ExitReason::Succeed(ExitSucceed::Returned),
						output: Vec::new(),
						cost: 19,
						logs: vec![
							LogsBuilder::new(Alice.into()).log1(H256::repeat_byte(0x33), vec![])
						],
					}
				}
				_ => panic!("unexpected subcall"),
			}
		})
}

#[test]
fn batch_some_incomplete() {
	ExtBuilder::default().build().execute_with(|| {
		let (return_log_cost, _) = costs(Action::BatchSome);

		batch_incomplete(&precompiles(), Action::BatchSome)
			.expect_log(LogsBuilder::new(Bob.into()).log1(H256::repeat_byte(0x11), vec![]))
			.expect_log(log_subcall_succeeded(Precompile, 0))
			.expect_log(log_subcall_failed(Precompile, 1))
			.expect_log(LogsBuilder::new(Alice.into()).log1(H256::repeat_byte(0x33), vec![]))
			.expect_log(log_subcall_succeeded(Precompile, 2))
			.expect_cost(13 + 17 + 19 + return_log_cost * 3)
			.execute_returns(Vec::new())
	})
}

#[test]
fn batch_some_until_failure_incomplete() {
	ExtBuilder::default().build().execute_with(|| {
		let (return_log_cost, _) = costs(Action::BatchSomeUntilFailure);

		batch_incomplete(&precompiles(), Action::BatchSomeUntilFailure)
			.expect_log(LogsBuilder::new(Bob.into()).log1(H256::repeat_byte(0x11), vec![]))
			.expect_log(log_subcall_succeeded(Precompile, 0))
			.expect_log(log_subcall_failed(Precompile, 1))
			.expect_cost(13 + 17 + return_log_cost * 2)
			.execute_returns(Vec::new())
	})
}

#[test]
fn batch_all_incomplete() {
	ExtBuilder::default().build().execute_with(|| {
		batch_incomplete(&precompiles(), Action::BatchAll)
			.execute_reverts(|output| output == b"Revert message")
	})
}

fn batch_log_out_of_gas(
	precompiles: &TestPrecompiles<Runtime>,
	action: Action,
) -> PrecompilesTester<TestPrecompiles<Runtime>> {
	let (_, gas_reserve) = costs(action);

	precompiles
		.prepare_test(
			Alice,
			Precompile,
			EvmDataWriter::new_with_selector(action)
				.write(vec![Address(Bob.into())])
				.write(vec![U256::from(1u8)])
				.write(vec![Bytes::from(b"one".as_slice())])
				.write::<Vec<U256>>(vec![])
				.build(),
		)
		.with_target_gas(Some(gas_reserve - 1))
		.with_subcall_handle(move |_subcall| panic!("there shouldn't be any subcalls"))
}

#[test]
fn batch_all_log_out_of_gas() {
	ExtBuilder::default().build().execute_with(|| {
		batch_log_out_of_gas(&precompiles(), Action::BatchAll).execute_error(ExitError::OutOfGas);
	})
}

#[test]
fn batch_some_log_out_of_gas() {
	ExtBuilder::default().build().execute_with(|| {
		batch_log_out_of_gas(&precompiles(), Action::BatchSome)
			.expect_no_logs()
			.execute_returns(Vec::new());
	})
}

#[test]
fn batch_some_until_failure_log_out_of_gas() {
	ExtBuilder::default().build().execute_with(|| {
		batch_log_out_of_gas(&precompiles(), Action::BatchSomeUntilFailure)
			.expect_no_logs()
			.execute_returns(Vec::new());
	})
}

fn batch_gas_limit(
	precompiles: &TestPrecompiles<Runtime>,
	action: Action,
) -> PrecompilesTester<TestPrecompiles<Runtime>> {
	let (_, gas_reserve) = costs(action);

	precompiles
		.prepare_test(
			Alice,
			Precompile,
			EvmDataWriter::new_with_selector(action)
				.write(vec![Address(Bob.into())])
				.write(vec![U256::from(1u8)])
				.write(vec![Bytes::from(b"one".as_slice())])
				.write(vec![50_000 - gas_reserve + 1])
				.build(),
		)
		.with_target_gas(Some(50_000))
		.with_subcall_handle(move |_subcall| panic!("there shouldn't be any subcalls"))
}

#[test]
fn batch_all_gas_limit() {
	ExtBuilder::default().build().execute_with(|| {
		batch_gas_limit(&precompiles(), Action::BatchAll).execute_error(ExitError::OutOfGas);
	})
}

#[test]
fn batch_some_gas_limit() {
	ExtBuilder::default().build().execute_with(|| {
		let (return_log_cost, _) = costs(Action::BatchSome);

		batch_gas_limit(&precompiles(), Action::BatchSome)
			.expect_log(log_subcall_failed(Precompile, 0))
			.expect_cost(return_log_cost)
			.execute_returns(Vec::new());
	})
}

#[test]
fn batch_some_until_failure_gas_limit() {
	ExtBuilder::default().build().execute_with(|| {
		batch_gas_limit(&precompiles(), Action::BatchSomeUntilFailure)
			.expect_log(log_subcall_failed(Precompile, 0))
			.execute_returns(Vec::new());
	})
}

#[test]
fn evm_batch_some_transfers_enough() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 10_000)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Evm(evm_call(
				Alice,
				EvmDataWriter::new_with_selector(Action::BatchSome)
					.write(vec![Address(Bob.into()), Address(Charlie.into()),])
					.write(vec![U256::from(1_000u16), U256::from(2_000u16)])
					.write::<Vec<Bytes>>(vec![])
					.write::<Vec<U256>>(vec![])
					.write(true)
					.build()
			))
			.dispatch(Origin::root()));
		})
}

#[test]
fn evm_batch_some_until_failure_transfers_enough() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 10_000)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Evm(evm_call(
				Alice,
				EvmDataWriter::new_with_selector(Action::BatchSomeUntilFailure)
					.write(vec![Address(Bob.into()), Address(Charlie.into()),])
					.write(vec![U256::from(1_000u16), U256::from(2_000u16)])
					.write::<Vec<Bytes>>(vec![])
					.write::<Vec<U256>>(vec![])
					.write(true)
					.build()
			))
			.dispatch(Origin::root()));
		})
}

#[test]
fn evm_batch_all_transfers_enough() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 10_000)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Evm(evm_call(
				Alice,
				EvmDataWriter::new_with_selector(Action::BatchAll)
					.write(vec![Address(Bob.into()), Address(Charlie.into()),])
					.write(vec![U256::from(1_000u16), U256::from(2_000u16)])
					.write::<Vec<Bytes>>(vec![])
					.write::<Vec<U256>>(vec![])
					.write(true)
					.build()
			))
			.dispatch(Origin::root()));

			assert_eq!(balance(Bob), 1_000);
			assert_eq!(balance(Charlie), 2_000);
		})
}

#[test]
fn evm_batch_some_transfers_too_much() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 10_000)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Evm(evm_call(
				Alice,
				EvmDataWriter::new_with_selector(Action::BatchSome)
					.write(vec![
						Address(Bob.into()),
						Address(Charlie.into()),
						Address(David.into()),
					])
					.write(vec![
						U256::from(9_000u16),
						U256::from(2_000u16),
						U256::from(500u16)
					])
					.write::<Vec<Bytes>>(vec![])
					.write::<Vec<U256>>(vec![])
					.write(true)
					.build()
			))
			.dispatch(Origin::root()));

			assert_eq!(balance(Alice), 500); // gasprice = 0
			assert_eq!(balance(Bob), 9_000);
			assert_eq!(balance(Charlie), 0);
			assert_eq!(balance(David), 500);
		})
}

#[test]
fn evm_batch_some_until_failure_transfers_too_much() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 10_000)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Evm(evm_call(
				Alice,
				EvmDataWriter::new_with_selector(Action::BatchSomeUntilFailure)
					.write(vec![
						Address(Bob.into()),
						Address(Charlie.into()),
						Address(David.into()),
					])
					.write(vec![
						U256::from(9_000u16),
						U256::from(2_000u16),
						U256::from(500u16)
					])
					.write::<Vec<Bytes>>(vec![])
					.write::<Vec<U256>>(vec![])
					.write(true)
					.build()
			))
			.dispatch(Origin::root()));

			assert_eq!(balance(Alice), 1_000); // gasprice = 0
			assert_eq!(balance(Bob), 9_000);
			assert_eq!(balance(Charlie), 0);
			assert_eq!(balance(David), 0);
		})
}

#[test]
fn evm_batch_all_transfers_too_much() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 10_000)])
		.build()
		.execute_with(|| {
			assert_ok!(Call::Evm(evm_call(
				Alice,
				EvmDataWriter::new_with_selector(Action::BatchAll)
					.write(vec![
						Address(Bob.into()),
						Address(Charlie.into()),
						Address(David.into()),
					])
					.write(vec![
						U256::from(9_000u16),
						U256::from(2_000u16),
						U256::from(500u16)
					])
					.write::<Vec<Bytes>>(vec![])
					.write::<Vec<U256>>(vec![])
					.write(true)
					.build()
			))
			.dispatch(Origin::root()));

			assert_eq!(balance(Alice), 10_000); // gasprice = 0
			assert_eq!(balance(Bob), 0);
			assert_eq!(balance(Charlie), 0);
			assert_eq!(balance(David), 0);
		})
}

#[test]
fn evm_batch_some_contract_revert() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 10_000)])
		.build()
		.execute_with(|| {
			setup_revert_contract();

			assert_ok!(Call::Evm(evm_call(
				Alice,
				EvmDataWriter::new_with_selector(Action::BatchSome)
					.write(vec![
						Address(Bob.into()),
						Address(Revert.into()),
						Address(David.into()),
					])
					.write(vec![
						U256::from(1_000u16),
						U256::from(2_000),
						U256::from(3_000u16)
					])
					.write::<Vec<Bytes>>(vec![])
					.write::<Vec<U256>>(vec![])
					.write(true)
					.build()
			))
			.dispatch(Origin::root()));

			assert_eq!(balance(Alice), 6_000); // gasprice = 0
			assert_eq!(balance(Bob), 1_000);
			assert_eq!(balance(Revert), 0);
			assert_eq!(balance(David), 3_000);
		})
}

#[test]
fn evm_batch_some_until_failure_contract_revert() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 10_000)])
		.build()
		.execute_with(|| {
			setup_revert_contract();

			assert_ok!(Call::Evm(evm_call(
				Alice,
				EvmDataWriter::new_with_selector(Action::BatchSomeUntilFailure)
					.write(vec![
						Address(Bob.into()),
						Address(Revert.into()),
						Address(David.into()),
					])
					.write(vec![
						U256::from(1_000u16),
						U256::from(2_000),
						U256::from(3_000u16)
					])
					.write::<Vec<Bytes>>(vec![])
					.write::<Vec<U256>>(vec![])
					.write(true)
					.build()
			))
			.dispatch(Origin::root()));

			assert_eq!(balance(Alice), 9_000); // gasprice = 0
			assert_eq!(balance(Bob), 1_000);
			assert_eq!(balance(Revert), 0);
			assert_eq!(balance(David), 0);
		})
}

#[test]
fn evm_batch_all_contract_revert() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 10_000)])
		.build()
		.execute_with(|| {
			setup_revert_contract();

			assert_ok!(Call::Evm(evm_call(
				Alice,
				EvmDataWriter::new_with_selector(Action::BatchAll)
					.write(vec![
						Address(Bob.into()),
						Address(Revert.into()),
						Address(David.into()),
					])
					.write(vec![
						U256::from(1_000u16),
						U256::from(2_000),
						U256::from(3_000u16)
					])
					.write::<Vec<Bytes>>(vec![])
					.write::<Vec<U256>>(vec![])
					.write(true)
					.build()
			))
			.dispatch(Origin::root()));

			assert_eq!(balance(Alice), 10_000); // gasprice = 0
			assert_eq!(balance(Bob), 0);
			assert_eq!(balance(Revert), 0);
			assert_eq!(balance(David), 0);
		})
}
