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

use std::{assert_matches::assert_matches, str::from_utf8};

use crate::mock::*;
use crate::*;

use fp_evm::{Context, PrecompileFailure};
use pallet_evm::PrecompileSet;
use precompile_utils::{error, Bytes, EvmDataWriter, LogsBuilder};
use sha3::{Digest, Keccak256};

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
		let bogus_selector = vec![1u8, 2u8, 3u8];

		assert_eq!(
			precompiles().execute(
				Account::Precompile.into(),
				&bogus_selector,
				None,
				&Context {
					address: Account::Precompile.into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
				false,
			),
			Some(Err(error("tried to parse selector out of bounds")))
		);
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		let bogus_selector = vec![1u8, 2u8, 3u8, 4u8];

		assert_eq!(
			precompiles().execute(
				Account::Precompile.into(),
				&bogus_selector,
				None,
				&Context {
					address: Account::Precompile.into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
				false,
			),
			Some(Err(error("unknown selector")))
		);
	});
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
	assert_eq!(Action::Decimals as u32, 0x313ce567);

	assert_eq!(
		crate::SELECTOR_LOG_TRANSFER,
		&Keccak256::digest(b"Transfer(address,address,uint256)")[..]
	);

	assert_eq!(
		crate::SELECTOR_LOG_APPROVAL,
		&Keccak256::digest(b"Approval(address,address,uint256)")[..]
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
					cost: 195953756u64, // 1 weight => 1 gas in mock
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
					if from_utf8(&str).unwrap().contains("Dispatched call failed with error: DispatchErrorWithPostInfo")
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
					cost: 195953756u64, // 1 weight => 1 gas in mock
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
				Some(Err(error("trying to spend more than allowed"))),
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
					cost: 195953756u64, // 1 weight => 1 gas in mock
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
