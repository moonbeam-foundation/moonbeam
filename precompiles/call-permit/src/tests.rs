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
		Account::{Alice, Bob, Charlie, Precompile},
		ExtBuilder, PrecompilesValue, Runtime, TestPrecompiles, ALICE_SECRET_KEY,
	},
	Action, CallPermitPrecompile,
};
use evm::ExitReason;
use fp_evm::{ExitRevert, ExitSucceed};
use libsecp256k1::{sign, Message, SecretKey};
use precompile_utils::{costs::call_cost, prelude::*, testing::*};
use sp_core::{H160, H256, U256};

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

fn dispatch_cost() -> u64 {
	CallPermitPrecompile::<Runtime>::dispatch_inherent_cost()
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
						logs: vec![log1(Bob, H256::repeat_byte(0x11), vec![])],
					}
				})
				.with_target_gas(Some(call_cost + 100_000 + dispatch_cost()))
				.expect_cost(call_cost + 13 + dispatch_cost())
				.expect_log(log1(Bob, H256::repeat_byte(0x11), vec![]))
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
				.with_target_gas(Some(call_cost + 100_000 + dispatch_cost()))
				.expect_cost(call_cost + 13 + dispatch_cost())
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
				.with_target_gas(Some(call_cost + 100_000 + dispatch_cost()))
				.expect_cost(dispatch_cost())
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
				.with_target_gas(Some(call_cost + 99_999 + dispatch_cost()))
				.expect_cost(dispatch_cost())
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
				.with_target_gas(Some(100_000 + dispatch_cost()))
				.expect_cost(dispatch_cost())
				.execute_reverts(|x| x == b"call require too much gas (u64 overflow)");
		})
}

// // This test checks the validity of a metamask signed message against the permit precompile
// // The code used to generate the signature is the following.
// // You will need to import ALICE_PRIV_KEY in metamask.
// // If you put this code in the developer tools console, it will log the signature

// await window.ethereum.enable();
// const accounts = await window.ethereum.request({ method: "eth_requestAccounts" });

// const from = accounts[0];
// const to = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
// const value = 42;
// const data = "0xdeadbeef";
// const gaslimit = 100000;
// const nonce = 0;
// const deadline = 1000;

// const createPermitMessageData = function () {
// 	const message = {
// 	from: from,
// 	to: to,
// 	value: value,
//    data: data,
//    gaslimit: gaslimit,
// 	nonce: nonce,
// 	deadline: deadline,
// 	};

// 	const typedData = JSON.stringify({
// 	types: {
// 		EIP712Domain: [
// 		{
// 			name: "name",
// 			type: "string",
// 		},
// 		{
// 			name: "version",
// 			type: "string",
// 		},
// 		{
// 			name: "chainId",
// 			type: "uint256",
// 		},
// 		{
// 			name: "verifyingContract",
// 			type: "address",
// 		},
// 		],
// 		CallPermit: [
// 		{
// 			name: "from",
// 			type: "address",
// 		},
// 		{
// 			name: "to",
// 			type: "address",
// 		},
// 		{
// 			name: "value",
// 			type: "uint256",
// 		},
//       {
// 			name: "data",
// 			type: "bytes",
// 		},
// 		{
// 			name: "gaslimit",
// 			type: "uint64",
// 		},
// 		{
// 			name: "nonce",
// 			type: "uint256",
// 		},
// 		{
// 			name: "deadline",
// 			type: "uint256",
// 		},
// 		],
// 	},
// 	primaryType: "CallPermit",
// 	domain: {
// 		name: "Call Permit Precompile",
// 		version: "1",
// 		chainId: 0,
// 		verifyingContract: "0x0000000000000000000000000000000000000001",
// 	},
// 	message: message,
// 	});

// 	return {
// 		typedData,
// 		message,
// 	};
// };

// const method = "eth_signTypedData_v4"
// const messageData = createPermitMessageData();
// const params = [from, messageData.typedData];

// web3.currentProvider.sendAsync(
// 	{
// 		method,
// 		params,
// 		from,
// 	},
// 	function (err, result) {
// 		if (err) return console.dir(err);
// 		if (result.error) {
// 			alert(result.error.message);
// 		}
// 		if (result.error) return console.error('ERROR', result);
// 		console.log('TYPED SIGNED:' + JSON.stringify(result.result));

// 		const recovered = sigUtil.recoverTypedSignature_v4({
// 			data: JSON.parse(msgParams),
// 			sig: result.result,
// 		});

// 		if (
// 			ethUtil.toChecksumAddress(recovered) === ethUtil.toChecksumAddress(from)
// 		) {
// 			alert('Successfully recovered signer as ' + from);
// 		} else {
// 			alert(
// 				'Failed to verify signer when comparing ' + result + ' to ' + from
// 			);
// 		}
// 	}
// );
#[test]
fn valid_permit_returns_with_metamask_signed_data() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 2000)])
		.build()
		.execute_with(|| {
			let from: H160 = Alice.into();
			let to: H160 = Bob.into();
			let value: U256 = 42u8.into();
			let data: Vec<u8> = hex_literal::hex!("deadbeef").to_vec();
			let gas_limit = 100_000u64;
			let deadline: U256 = 1_000u32.into();

			// Made with MetaMask
			let rsv = hex_literal::hex!(
				"56b497d556cb1b57a16aac6e8d53f3cbf1108df467ffcb937a3744369a27478f608de05
				34b8e0385e55ffd97cbafcfeac12ab52d0b74a2dea582bc8de46f257d1c"
			)
			.as_slice();
			let (r, sv) = rsv.split_at(32);
			let (s, v) = sv.split_at(32);
			let v_real = v[0];
			let r_real: [u8; 32] = r.try_into().unwrap();
			let s_real: [u8; 32] = s.try_into().unwrap();

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
						.write(Bytes(data.clone()))
						.write(gas_limit)
						.write(deadline)
						.write(v_real)
						.write(H256::from(r_real))
						.write(H256::from(s_real))
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

					assert_eq!(&input, &data);

					SubcallOutput {
						reason: ExitReason::Succeed(ExitSucceed::Returned),
						output: b"TEST".to_vec(),
						cost: 13,
						logs: vec![log1(Bob, H256::repeat_byte(0x11), vec![])],
					}
				})
				.with_target_gas(Some(call_cost + 100_000 + dispatch_cost()))
				.expect_cost(call_cost + 13 + dispatch_cost())
				.expect_log(log1(Bob, H256::repeat_byte(0x11), vec![]))
				.execute_returns(b"TEST".to_vec());
		})
}
