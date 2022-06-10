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

use crate::{
	mock::{
		balance,
		Account::{Alice, Bob, Charlie, David, Precompile, Revert},
		Call, ExtBuilder, Origin, PrecompilesValue, Runtime, TestPrecompiles, ALICE_SECRET_KEY,
	},
	Action, CallPermitPrecompile,
};
use evm::ExitReason;
use fp_evm::{ExitError, ExitRevert, ExitSucceed};
use frame_support::{assert_ok, dispatch::Dispatchable};
use libsecp256k1::{sign, Message, SecretKey};
use pallet_evm::Call as EvmCall;
use precompile_utils::{call_cost, testing::*, Address, Bytes, EvmDataWriter, LogExt, LogsBuilder};
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

#[test]
fn selectors() {
	assert_eq!(Action::Dispatch as u32, 0xb5ea0966);
	assert_eq!(Action::Nonces as u32, 0x7ecebe00);
	assert_eq!(Action::DomainSeparator as u32, 0x3644e515);
}

#[test]
fn valid_permit_returns() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let from: H160 = Alice.into();
			let to: H160 = Bob.into();
			let value: U256 = 42u8.into();
			let data: Vec<u8> = b"Test".to_vec();
			let gas_limit = 100_000u64;
			let nonce: U256 = 0u8.into();
			let deadline: U256 = 1_000u32.into();

			let permit = CallPermitPrecompile::<Runtime>::generate_permit(
				Precompile.into(),
				from,
				to,
				value,
				data.clone(),
				gas_limit,
				nonce,
				deadline,
			);

			dbg!(H256::from(permit));

			let secret_key = SecretKey::parse(&ALICE_SECRET_KEY).unwrap();
			let message = Message::parse(&permit);
			let (rs, v) = sign(&message, &secret_key);

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::Nonces)
						.write(Address(Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u8)).build());

			let call_cost = call_cost(value, <Runtime as pallet_evm::Config>::config());

			precompiles()
				.prepare_test(
					Charlie, // can be anyone
					Precompile,
					EvmDataWriter::new_with_selector(Action::Dispatch)
						.write(Address(from))
						.write(Address(to))
						.write(value)
						.write(Bytes(data))
						.write(gas_limit)
						.write(deadline)
						.write(v.serialize())
						.write(H256::from(rs.r.b32()))
						.write(H256::from(rs.s.b32()))
						.build(),
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

					// Called on the behalf of the permit maker.
					assert_eq!(context.caller, Alice.into());
					assert_eq!(address, Bob.into());
					assert_eq!(is_static, false);
					assert_eq!(target_gas, Some(100_000), "forward requested gas");

					let transfer = transfer.expect("there is a transfer");
					assert_eq!(transfer.source, Alice.into());
					assert_eq!(transfer.target, Bob.into());
					assert_eq!(transfer.value, 42u8.into());

					assert_eq!(context.address, Bob.into());
					assert_eq!(context.apparent_value, 42u8.into());

					assert_eq!(&input, b"Test");

					SubcallOutput {
						reason: ExitReason::Succeed(ExitSucceed::Returned),
						output: b"TEST".to_vec(),
						cost: 13,
						logs: vec![
							LogsBuilder::new(Bob.into()).log1(H256::repeat_byte(0x11), vec![])
						],
					}
				})
				.with_target_gas(Some(call_cost + 100_000))
				.expect_cost(call_cost + 13)
				.expect_log(LogsBuilder::new(Bob.into()).log1(H256::repeat_byte(0x11), vec![]))
				.execute_returns(b"TEST".to_vec());
		})
}

#[test]
fn valid_permit_reverts() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let from: H160 = Alice.into();
			let to: H160 = Bob.into();
			let value: U256 = 42u8.into();
			let data: Vec<u8> = b"Test".to_vec();
			let gas_limit = 100_000u64;
			let nonce: U256 = 0u8.into();
			let deadline: U256 = 1_000u32.into();

			let permit = CallPermitPrecompile::<Runtime>::generate_permit(
				Precompile.into(),
				from,
				to,
				value,
				data.clone(),
				gas_limit,
				nonce,
				deadline,
			);

			dbg!(H256::from(permit));

			let secret_key = SecretKey::parse(&ALICE_SECRET_KEY).unwrap();
			let message = Message::parse(&permit);
			let (rs, v) = sign(&message, &secret_key);

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::Nonces)
						.write(Address(Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u8)).build());

			let call_cost = call_cost(value, <Runtime as pallet_evm::Config>::config());

			precompiles()
				.prepare_test(
					Charlie, // can be anyone
					Precompile,
					EvmDataWriter::new_with_selector(Action::Dispatch)
						.write(Address(from))
						.write(Address(to))
						.write(value)
						.write(Bytes(data))
						.write(gas_limit)
						.write(deadline)
						.write(v.serialize())
						.write(H256::from(rs.r.b32()))
						.write(H256::from(rs.s.b32()))
						.build(),
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

					// Called on the behalf of the permit maker.
					assert_eq!(context.caller, Alice.into());
					assert_eq!(address, Bob.into());
					assert_eq!(is_static, false);
					assert_eq!(target_gas, Some(100_000), "forward requested gas");

					let transfer = transfer.expect("there is a transfer");
					assert_eq!(transfer.source, Alice.into());
					assert_eq!(transfer.target, Bob.into());
					assert_eq!(transfer.value, 42u8.into());

					assert_eq!(context.address, Bob.into());
					assert_eq!(context.apparent_value, 42u8.into());

					assert_eq!(&input, b"Test");

					SubcallOutput {
						reason: ExitReason::Revert(ExitRevert::Reverted),
						output: b"TEST".to_vec(),
						cost: 13,
						logs: vec![],
					}
				})
				.with_target_gas(Some(call_cost + 100_000))
				.expect_cost(call_cost + 13)
				.expect_no_logs()
				.execute_reverts(|x| x == b"TEST".to_vec());
		})
}

#[test]
fn invalid_permit_nonce() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let from: H160 = Alice.into();
			let to: H160 = Bob.into();
			let value: U256 = 42u8.into();
			let data: Vec<u8> = b"Test".to_vec();
			let gas_limit = 100_000u64;
			let nonce: U256 = 1u8.into(); // WRONG NONCE
			let deadline: U256 = 1_000u32.into();

			let permit = CallPermitPrecompile::<Runtime>::generate_permit(
				Precompile.into(),
				from,
				to,
				value,
				data.clone(),
				gas_limit,
				nonce,
				deadline,
			);

			dbg!(H256::from(permit));

			let secret_key = SecretKey::parse(&ALICE_SECRET_KEY).unwrap();
			let message = Message::parse(&permit);
			let (rs, v) = sign(&message, &secret_key);

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::Nonces)
						.write(Address(Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u8)).build());

			let call_cost = call_cost(value, <Runtime as pallet_evm::Config>::config());

			precompiles()
				.prepare_test(
					Charlie, // can be anyone
					Precompile,
					EvmDataWriter::new_with_selector(Action::Dispatch)
						.write(Address(from))
						.write(Address(to))
						.write(value)
						.write(Bytes(data))
						.write(gas_limit)
						.write(deadline)
						.write(v.serialize())
						.write(H256::from(rs.r.b32()))
						.write(H256::from(rs.s.b32()))
						.build(),
				)
				.with_subcall_handle(move |_| panic!("should not perform subcall"))
				.with_target_gas(Some(call_cost + 100_000))
				.expect_cost(0)
				.execute_reverts(|x| x == b"invalid permit");
		})
}

#[test]
fn invalid_permit_gas_limit_too_low() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let from: H160 = Alice.into();
			let to: H160 = Bob.into();
			let value: U256 = 42u8.into();
			let data: Vec<u8> = b"Test".to_vec();
			let gas_limit = 100_000u64;
			let nonce: U256 = 0u8.into();
			let deadline: U256 = 1_000u32.into();

			let permit = CallPermitPrecompile::<Runtime>::generate_permit(
				Precompile.into(),
				from,
				to,
				value,
				data.clone(),
				gas_limit,
				nonce,
				deadline,
			);

			dbg!(H256::from(permit));

			let secret_key = SecretKey::parse(&ALICE_SECRET_KEY).unwrap();
			let message = Message::parse(&permit);
			let (rs, v) = sign(&message, &secret_key);

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::Nonces)
						.write(Address(Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u8)).build());

			let call_cost = call_cost(value, <Runtime as pallet_evm::Config>::config());

			precompiles()
				.prepare_test(
					Charlie, // can be anyone
					Precompile,
					EvmDataWriter::new_with_selector(Action::Dispatch)
						.write(Address(from))
						.write(Address(to))
						.write(value)
						.write(Bytes(data))
						.write(gas_limit)
						.write(deadline)
						.write(v.serialize())
						.write(H256::from(rs.r.b32()))
						.write(H256::from(rs.s.b32()))
						.build(),
				)
				.with_subcall_handle(move |_| panic!("should not perform subcall"))
				.with_target_gas(Some(call_cost + 99_999))
				.expect_cost(0)
				.execute_reverts(|x| x == b"gaslimit is too low to dispatch provided call");
		})
}

#[test]
fn invalid_permit_gas_limit_overflow() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			let from: H160 = Alice.into();
			let to: H160 = Bob.into();
			let value: U256 = 42u8.into();
			let data: Vec<u8> = b"Test".to_vec();
			let gas_limit = u64::MAX;
			let nonce: U256 = 0u8.into();
			let deadline: U256 = 1_000u32.into();

			let permit = CallPermitPrecompile::<Runtime>::generate_permit(
				Precompile.into(),
				from,
				to,
				value,
				data.clone(),
				gas_limit,
				nonce,
				deadline,
			);

			dbg!(H256::from(permit));

			let secret_key = SecretKey::parse(&ALICE_SECRET_KEY).unwrap();
			let message = Message::parse(&permit);
			let (rs, v) = sign(&message, &secret_key);

			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::Nonces)
						.write(Address(Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u8)).build());

			let call_cost = call_cost(value, <Runtime as pallet_evm::Config>::config());

			precompiles()
				.prepare_test(
					Charlie, // can be anyone
					Precompile,
					EvmDataWriter::new_with_selector(Action::Dispatch)
						.write(Address(from))
						.write(Address(to))
						.write(value)
						.write(Bytes(data))
						.write(gas_limit)
						.write(deadline)
						.write(v.serialize())
						.write(H256::from(rs.r.b32()))
						.write(H256::from(rs.s.b32()))
						.build(),
				)
				.with_subcall_handle(move |_| panic!("should not perform subcall"))
				.with_target_gas(Some(100_000))
				.expect_cost(0)
				.execute_reverts(|x| x == b"call require too much gas (u64 overflow)");
		})
}
