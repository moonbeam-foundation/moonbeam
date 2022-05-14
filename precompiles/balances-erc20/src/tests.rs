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

use std::{assert_matches::assert_matches, str::from_utf8};

use crate::{eip2612::Eip2612, mock::*, *};

use fp_evm::{Context, PrecompileFailure};
use libsecp256k1::{sign, Message, SecretKey};
use pallet_evm::PrecompileSet;
use precompile_utils::{Bytes, EvmDataWriter, LogsBuilder};
use sha3::{Digest, Keccak256};
use sp_core::{H256, U256};

// No test of invalid selectors since we have a fallback behavior (deposit).
fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selectors() {
	assert_eq!(Action::BalanceOf as u32, 0x70a08231);
	assert_eq!(Action::TotalSupply as u32, 0x18160ddd);
	assert_eq!(Action::Approve as u32, 0x095ea7b3);
	assert_eq!(Action::Allowance as u32, 0xdd62ed3e);
	assert_eq!(Action::Transfer as u32, 0xa9059cbb);
	assert_eq!(Action::TransferFrom as u32, 0x23b872dd);
	assert_eq!(Action::Name as u32, 0x06fdde03);
	assert_eq!(Action::Symbol as u32, 0x95d89b41);
	assert_eq!(Action::Deposit as u32, 0xd0e30db0);
	assert_eq!(Action::Withdraw as u32, 0x2e1a7d4d);
	assert_eq!(Action::Eip2612Nonces as u32, 0x7ecebe00);
	assert_eq!(Action::Eip2612Permit as u32, 0xd505accf);
	assert_eq!(Action::Eip2612DomainSeparator as u32, 0x3644e515);

	assert_eq!(
		crate::SELECTOR_LOG_TRANSFER,
		&Keccak256::digest(b"Transfer(address,address,uint256)")[..]
	);

	assert_eq!(
		crate::SELECTOR_LOG_APPROVAL,
		&Keccak256::digest(b"Approval(address,address,uint256)")[..]
	);

	assert_eq!(
		crate::SELECTOR_LOG_DEPOSIT,
		&Keccak256::digest(b"Deposit(address,uint256)")[..]
	);

	assert_eq!(
		crate::SELECTOR_LOG_WITHDRAWAL,
		&Keccak256::digest(b"Withdrawal(address,uint256)")[..]
	);
}

#[test]
fn get_total_supply() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TotalSupply).build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(3500u64)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);
		});
}

#[test]
fn get_balances_known_user() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(1000u64)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);
		});
}

#[test]
fn get_balances_unknown_user() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0u64)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);
		});
}

#[test]
fn approve() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 1756u64,
					logs: LogsBuilder::new(Account::Precompile.into())
						.log3(
							SELECTOR_LOG_APPROVAL,
							Account::Alice,
							Account::Bob,
							EvmDataWriter::new().write(U256::from(500)).build(),
						)
						.build(),
				}))
			);
		});
}

#[test]
fn approve_saturating() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::MAX)
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 1756u64,
					logs: LogsBuilder::new(Account::Precompile.into())
						.log3(
							SELECTOR_LOG_APPROVAL,
							Account::Alice,
							Account::Bob,
							EvmDataWriter::new().write(U256::MAX).build(),
						)
						.build(),
				}))
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(u128::MAX)).build(),
					cost: 0u64,
					logs: vec![],
				}))
			);
		});
}

#[test]
fn check_allowance_existing() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			precompiles().execute(
				Account::Precompile.into(),
				&EvmDataWriter::new_with_selector(Action::Approve)
					.write(Address(Account::Bob.into()))
					.write(U256::from(500))
					.build(),
				None,
				&Context {
					address: Account::Precompile.into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
				false,
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(500u64)).build(),
					cost: 0u64,
					logs: Default::default(),
				}))
			);
		});
}

#[test]
fn check_allowance_not_existing() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0u64)).build(),
					cost: 0u64,
					logs: Default::default(),
				}))
			);
		});
}

#[test]
fn transfer() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 159201756u64, // 1 weight => 1 gas in mock
					logs: LogsBuilder::new(Account::Precompile.into())
						.log3(
							SELECTOR_LOG_TRANSFER,
							Account::Alice,
							Account::Bob,
							EvmDataWriter::new().write(U256::from(400)).build(),
						)
						.build(),
				}))
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(600)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(400)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);
		});
}

#[test]
fn transfer_not_enough_funds() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_matches!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(Account::Bob.into()))
						.write(U256::from(1400))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Err(PrecompileFailure::Revert { output: str, .. }))
					if from_utf8(&str).unwrap()
						.contains("Dispatched call failed with error: DispatchErrorWithPostInfo")
					&& from_utf8(&str).unwrap().contains("InsufficientBalance")
			);
		});
}

#[test]
fn transfer_from() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			precompiles().execute(
				Account::Precompile.into(),
				&EvmDataWriter::new_with_selector(Action::Approve)
					.write(Address(Account::Bob.into()))
					.write(U256::from(500))
					.build(),
				None,
				&Context {
					address: Account::Precompile.into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
				false,
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Bob.into(), // Bob is the one sending transferFrom!
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 159201756u64, // 1 weight => 1 gas in mock
					logs: LogsBuilder::new(Account::Precompile.into())
						.log3(
							SELECTOR_LOG_TRANSFER,
							Account::Alice,
							Account::Bob,
							EvmDataWriter::new().write(U256::from(400)).build(),
						)
						.build(),
				}))
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(600)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(400)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(100u64)).build(),
					cost: 0u64,
					logs: Default::default(),
				}))
			);
		});
}

#[test]
fn transfer_from_above_allowance() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			precompiles().execute(
				Account::Precompile.into(),
				&EvmDataWriter::new_with_selector(Action::Approve)
					.write(Address(Account::Bob.into()))
					.write(U256::from(300))
					.build(),
				None,
				&Context {
					address: Account::Precompile.into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
				false,
			);

			assert_matches!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Bob.into(), // Bob is the one sending transferFrom!
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Err(PrecompileFailure::Revert { output, ..}))
					if output == b"trying to spend more than allowed",
			);
		});
}

#[test]
fn transfer_from_self() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						// Alice sending transferFrom herself, no need for allowance.
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 159201756u64, // 1 weight => 1 gas in mock
					logs: LogsBuilder::new(Account::Precompile.into())
						.log3(
							SELECTOR_LOG_TRANSFER,
							Account::Alice,
							Account::Bob,
							EvmDataWriter::new().write(U256::from(400)).build(),
						)
						.build(),
				}))
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(600)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(400)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);
		});
}

#[test]
fn get_metadata_name() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Name).build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new()
						.write::<Bytes>("Mock token".into())
						.build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);
		});
}

#[test]
fn get_metadata_symbol() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Symbol).build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write::<Bytes>("MOCK".into()).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);
		});
}

#[test]
fn get_metadata_decimals() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Decimals).build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(18u8).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);
		});
}

fn deposit(data: Vec<u8>) {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			// Check precompile balance is 0.
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Precompile.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);

			// Deposit
			// We need to call using EVM pallet so we can check the EVM correctly sends the amount
			// to the precompile.
			Evm::call(
				Origin::root(),
				Account::Alice.into(),
				Account::Precompile.into(),
				data,
				From::from(500), // amount sent
				u64::MAX,        // gas limit
				0u32.into(),     // gas price
				None,            // max priority
				None,            // nonce
				vec![],          // access list
			)
			.expect("it works");

			assert_eq!(
				events(),
				vec![
					Event::System(frame_system::Event::NewAccount {
						account: Account::Precompile
					}),
					Event::Balances(pallet_balances::Event::Endowed {
						account: Account::Precompile,
						free_balance: 500
					}),
					// EVM make a transfer because some value is provided.
					Event::Balances(pallet_balances::Event::Transfer {
						from: Account::Alice,
						to: Account::Precompile,
						amount: 500
					}),
					// Precompile send it back since deposit should be a no-op.
					Event::Balances(pallet_balances::Event::Transfer {
						from: Account::Precompile,
						to: Account::Alice,
						amount: 500
					}),
					// Log is correctly emited.
					Event::Evm(pallet_evm::Event::Log(
						LogsBuilder::new(Account::Precompile.into())
							.log2(
								SELECTOR_LOG_DEPOSIT,
								Account::Alice,
								EvmDataWriter::new().write(U256::from(500)).build(),
							)
							.build()[0]
							.clone()
					)),
					Event::Evm(pallet_evm::Event::Executed(Account::Precompile.into())),
				]
			);

			// Check precompile balance is still 0.
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Precompile.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);

			// Check Alice balance is still 1000.
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(1000)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);
		});
}

#[test]
fn deposit_function() {
	deposit(EvmDataWriter::new_with_selector(Action::Deposit).build())
}

#[test]
fn deposit_fallback() {
	deposit(EvmDataWriter::new_with_selector(0x01234567u32).build())
}

#[test]
fn deposit_receive() {
	deposit(vec![])
}

#[test]
fn deposit_zero() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			// Check precompile balance is 0.
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Precompile.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);

			// Deposit
			// We need to call using EVM pallet so we can check the EVM correctly sends the amount
			// to the precompile.
			Evm::call(
				Origin::root(),
				Account::Alice.into(),
				Account::Precompile.into(),
				EvmDataWriter::new_with_selector(Action::Deposit).build(),
				From::from(0), // amount sent
				u64::MAX,      // gas limit
				0u32.into(),   // gas price
				None,          // max priority
				None,          // nonce
				vec![],        // access list
			)
			.expect("it works");

			assert_eq!(
				events(),
				vec![Event::Evm(pallet_evm::Event::ExecutedFailed(
					Account::Precompile.into()
				)),]
			);

			// Check precompile balance is still 0.
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Precompile.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);

			// Check Alice balance is still 1000.
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(1000)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);
		});
}

#[test]
fn withdraw() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			// Check precompile balance is 0.
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Precompile.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);

			// Withdraw
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Withdraw)
						.write(U256::from(500))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().build(),
					cost: 1381,
					logs: LogsBuilder::new(Account::Precompile.into())
						.log2(
							SELECTOR_LOG_WITHDRAWAL,
							Account::Alice,
							EvmDataWriter::new().write(U256::from(500)).build(),
						)
						.build(),
				}))
			);

			// Check Alice balance is still 1000.
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(1000)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);
		});
}

#[test]
fn withdraw_more_than_owned() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			// Check precompile balance is 0.
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Precompile.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0u32),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);

			// Withdraw
			assert_matches!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Withdraw)
						.write(U256::from(1001))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0u32),
					},
					false,
				),
				Some(Err(PrecompileFailure::Revert { output: str, .. }))
					if str == b"trying to withdraw more than owned"
			);

			// Check Alice balance is still 1000.
			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0u32),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(1000)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);
		});
}

#[test]
fn permit_valid() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			let owner: H160 = Account::Alice.into();
			let spender: H160 = Account::Bob.into();
			let value: U256 = 500u16.into();
			let deadline: U256 = 0u8.into(); // todo: proper timestamp

			let permit = Eip2612::<Runtime, NativeErc20Metadata>::generate_permit(
				Account::Precompile.into(),
				owner,
				spender,
				value,
				0u8.into(), // nonce
				deadline,
			);

			let secret_key = SecretKey::parse(&ALICE_SECRET_KEY).unwrap();
			let message = Message::parse(&permit);
			let (rs, v) = sign(&message, &secret_key);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0u8)).build(),
					cost: 0u64,
					logs: vec![],
				}))
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Permit)
						.write(Address(owner))
						.write(Address(spender))
						.write(value)
						.write(deadline)
						.write(v.serialize())
						.write(H256::from(rs.r.b32()))
						.write(H256::from(rs.s.b32()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Charlie.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: vec![],
					cost: 0u64,
					logs: LogsBuilder::new(Account::Precompile.into())
						.log3(
							SELECTOR_LOG_APPROVAL,
							Account::Alice,
							Account::Bob,
							EvmDataWriter::new().write(U256::from(500)).build(),
						)
						.build(),
				}))
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(500u16)).build(),
					cost: 0u64,
					logs: vec![],
				}))
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(1u8)).build(),
					cost: 0u64,
					logs: vec![],
				}))
			);
		});
}

#[test]
fn permit_invalid_nonce() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			let owner: H160 = Account::Alice.into();
			let spender: H160 = Account::Bob.into();
			let value: U256 = 500u16.into();
			let deadline: U256 = 0u8.into();

			let permit = Eip2612::<Runtime, NativeErc20Metadata>::generate_permit(
				Account::Precompile.into(),
				owner,
				spender,
				value,
				1u8.into(), // nonce
				deadline,
			);

			let secret_key = SecretKey::parse(&ALICE_SECRET_KEY).unwrap();
			let message = Message::parse(&permit);
			let (rs, v) = sign(&message, &secret_key);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0u8)).build(),
					cost: 0u64,
					logs: vec![],
				}))
			);

			assert_matches!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Permit)
						.write(Address(owner))
						.write(Address(spender))
						.write(value)
						.write(deadline)
						.write(v.serialize())
						.write(H256::from(rs.r.b32()))
						.write(H256::from(rs.s.b32()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Charlie.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Err(PrecompileFailure::Revert { output, ..}))
				if output == b"invalid permit"
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0u16)).build(),
					cost: 0u64,
					logs: vec![],
				}))
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0u8)).build(),
					cost: 0u64,
					logs: vec![],
				}))
			);
		});
}

#[test]
fn permit_invalid_signature() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			let owner: H160 = Account::Alice.into();
			let spender: H160 = Account::Bob.into();
			let value: U256 = 500u16.into();
			let deadline: U256 = 0u8.into();

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0u8)).build(),
					cost: 0u64,
					logs: vec![],
				}))
			);

			assert_matches!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Permit)
						.write(Address(owner))
						.write(Address(spender))
						.write(value)
						.write(deadline)
						.write(0u8)
						.write(H256::random())
						.write(H256::random())
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Charlie.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Err(PrecompileFailure::Revert { output, ..}))
				if output == b"invalid permit"
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0u16)).build(),
					cost: 0u64,
					logs: vec![],
				}))
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0u8)).build(),
					cost: 0u64,
					logs: vec![],
				}))
			);
		});
}

#[test]
fn permit_invalid_deadline() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			pallet_timestamp::Pallet::<Runtime>::set_timestamp(10_000);

			let owner: H160 = Account::Alice.into();
			let spender: H160 = Account::Bob.into();
			let value: U256 = 500u16.into();
			let deadline: U256 = 5u8.into(); // deadline < timestamp => expired

			let permit = Eip2612::<Runtime, NativeErc20Metadata>::generate_permit(
				Account::Precompile.into(),
				owner,
				spender,
				value,
				0u8.into(), // nonce
				deadline,
			);

			let secret_key = SecretKey::parse(&ALICE_SECRET_KEY).unwrap();
			let message = Message::parse(&permit);
			let (rs, v) = sign(&message, &secret_key);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0u8)).build(),
					cost: 0u64,
					logs: vec![],
				}))
			);

			assert_matches!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Permit)
						.write(Address(owner))
						.write(Address(spender))
						.write(value)
						.write(deadline)
						.write(v.serialize())
						.write(H256::from(rs.r.b32()))
						.write(H256::from(rs.s.b32()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Charlie.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Err(PrecompileFailure::Revert { output, ..}))
				if output == b"permit expired"
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0u16)).build(),
					cost: 0u64,
					logs: vec![],
				}))
			);

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0u8)).build(),
					cost: 0u64,
					logs: vec![],
				}))
			);
		});
}

// This test checks the validity of a metamask signed message against the permit precompile
// The code used to generate the signature is the following.
// You will need to import ALICE_PRIV_KEY in metamask.
// If you put this code in the developer tools console, it will log the signature
/*
await window.ethereum.enable();
const accounts = await window.ethereum.request({ method: "eth_requestAccounts" });

const value = 1000;

const fromAddress = "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
const deadline = 1;
const nonce = 0;
const spender = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const from = accounts[0];

const createPermitMessageData = function () {
	const message = {
	owner: from,
	spender: spender,
	value: value,
	nonce: nonce,
	deadline: deadline,
	};

	const typedData = JSON.stringify({
	types: {
		EIP712Domain: [
		{
			name: "name",
			type: "string",
		},
		{
			name: "version",
			type: "string",
		},
		{
			name: "chainId",
			type: "uint256",
		},
		{
			name: "verifyingContract",
			type: "address",
		},
		],
		Permit: [
		{
			name: "owner",
			type: "address",
		},
		{
			name: "spender",
			type: "address",
		},
		{
			name: "value",
			type: "uint256",
		},
		{
			name: "nonce",
			type: "uint256",
		},
		{
			name: "deadline",
			type: "uint256",
		},
		],
	},
	primaryType: "Permit",
	domain: {
		name: "Mock token",
		version: "1",
		chainId: 0,
		verifyingContract: "0x0000000000000000000000000000000000000001",
	},
	message: message,
	});

	return {
		typedData,
		message,
	};
};

const method = "eth_signTypedData_v4"
const messageData = createPermitMessageData();
const params = [from, messageData.typedData];

web3.currentProvider.sendAsync(
	{
		method,
		params,
		from,
	},
	function (err, result) {
		if (err) return console.dir(err);
		if (result.error) {
			alert(result.error.message);
		}
		if (result.error) return console.error('ERROR', result);
		console.log('TYPED SIGNED:' + JSON.stringify(result.result));

		const recovered = sigUtil.recoverTypedSignature_v4({
			data: JSON.parse(msgParams),
			sig: result.result,
		});

		if (
			ethUtil.toChecksumAddress(recovered) === ethUtil.toChecksumAddress(from)
		) {
			alert('Successfully recovered signer as ' + from);
		} else {
			alert(
				'Failed to verify signer when comparing ' + result + ' to ' + from
			);
		}
	}
);
*/

#[test]
fn permit_valid_with_metamask_signed_data() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			let owner: H160 = H160::from_slice(ALICE_PUBLIC_KEY.as_slice());
			let spender: H160 = Account::Bob.into();
			let value: U256 = 1000u16.into();
			let deadline: U256 = 1u16.into(); // todo: proper timestamp

			let rsv = hex_literal::hex!(
				"612960858951e133d05483804be5456a030be4ce6c000a855d865c0be75a8fc11d89ca96d5a153e8c
				7155ab1147f0f6d3326388b8d866c2406ce34567b7501a01b"
			)
			.as_slice();
			let (r, sv) = rsv.split_at(32);
			let (s, v) = sv.split_at(32);
			let v_real = v[0];
			let r_real: [u8; 32] = r.try_into().unwrap();
			let s_real: [u8; 32] = s.try_into().unwrap();

			assert_eq!(
				precompiles().execute(
					Account::Precompile.into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Permit)
						.write(Address(owner))
						.write(Address(spender))
						.write(value)
						.write(deadline)
						.write(v_real)
						.write(H256::from(r_real))
						.write(H256::from(s_real))
						.build(),
					None,
					&Context {
						address: Account::Precompile.into(),
						caller: Account::Charlie.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: vec![],
					cost: 0u64,
					logs: LogsBuilder::new(Account::Precompile.into(),)
						.log3(
							SELECTOR_LOG_APPROVAL,
							Account::Alice,
							Account::Bob,
							EvmDataWriter::new().write(U256::from(1000)).build(),
						)
						.build(),
				}))
			);
		});
}
