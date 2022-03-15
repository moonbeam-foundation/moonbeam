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

use frame_support::assert_ok;
use std::{assert_matches::assert_matches, str::from_utf8};

use crate::mock::*;
use crate::*;

use fp_evm::{Context, PrecompileFailure};
use pallet_evm::PrecompileSet;
use precompile_utils::{EvmDataWriter, LogsBuilder};
use sha3::{Digest, Keccak256};

fn precompiles() -> Erc20AssetsPrecompileSet<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Assets::force_create(
			Origin::root(),
			0u128,
			Account::Alice.into(),
			true,
			1
		));
		assert_ok!(Assets::mint(
			Origin::signed(Account::Alice),
			0u128,
			Account::Alice.into(),
			1000
		));
		// This selector is only three bytes long when four are required.
		let bogus_selector = vec![1u8, 2u8, 3u8];

		assert_matches!(
			precompiles().execute(
				Account::AssetId(0u128).into(),
				&bogus_selector,
				None,
				&Context {
					address: Account::AssetId(1u128).into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0u32),
				},
				false,
			),
			Some(Err(PrecompileFailure::Revert { output, .. }))
			if output == b"tried to parse selector out of bounds"
		);
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(Assets::force_create(
			Origin::root(),
			0u128,
			Account::Alice.into(),
			true,
			1
		));
		assert_ok!(Assets::mint(
			Origin::signed(Account::Alice),
			0u128,
			Account::Alice.into(),
			1000
		));
		let bogus_selector = vec![1u8, 2u8, 3u8, 4u8];

		assert_matches!(
			precompiles().execute(
				Account::AssetId(0u128).into(),
				&bogus_selector,
				None,
				&Context {
					address: Account::AssetId(1u128).into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0u32),
				},
				false,
			),
			Some(Err(PrecompileFailure::Revert { output, .. }))
			if output == b"unknown selector"
		);
	});
}

#[test]
fn selectors() {
	assert_eq!(u32::from(Action::BalanceOf), 0x70a08231);
	assert_eq!(u32::from(Action::TotalSupply), 0x18160ddd);
	assert_eq!(u32::from(Action::Approve), 0x095ea7b3);
	assert_eq!(u32::from(Action::Allowance), 0xdd62ed3e);
	assert_eq!(u32::from(Action::Transfer), 0xa9059cbb);
	assert_eq!(u32::from(Action::TransferFrom), 0x23b872dd);
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
			assert_ok!(Assets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));
			assert_eq!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::TotalSupply).build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
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
fn get_balances_known_user() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));
			assert_eq!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
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
			assert_ok!(Assets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));
			assert_eq!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
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
			assert_ok!(Assets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			assert_eq!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 30832756u64,
					logs: LogsBuilder::new(Account::AssetId(0u128).into())
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
			assert_ok!(Assets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			assert_eq!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::MAX)
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 30832756u64,
					logs: LogsBuilder::new(Account::AssetId(0u128).into())
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
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
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
			assert_ok!(Assets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles().execute(
				Account::AssetId(0u128).into(),
				&EvmDataWriter::new_with_selector(Action::Approve)
					.write(Address(Account::Bob.into()))
					.write(U256::from(500))
					.build(),
				None,
				&Context {
					address: Account::AssetId(0u128).into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
				false,
			);

			assert_eq!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
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
			assert_ok!(Assets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));
			assert_eq!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
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
			assert_ok!(Assets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			assert_eq!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 44001756u64, // 1 weight => 1 gas in mock
					logs: LogsBuilder::new(Account::AssetId(0u128).into())
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
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Bob.into(),
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
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
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
		});
}

#[test]
fn transfer_not_enough_founds() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1
			));

			assert_matches!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(Account::Charlie.into()))
						.write(U256::from(50))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Err(PrecompileFailure::Revert { output: str, ..}))
					if from_utf8(&str).unwrap()
						.contains("Dispatched call failed with error: DispatchErrorWithPostInfo")
					&& from_utf8(&str).unwrap().contains("BalanceLow")
			);
		});
}

#[test]
fn transfer_from() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles().execute(
				Account::AssetId(0u128).into(),
				&EvmDataWriter::new_with_selector(Action::Approve)
					.write(Address(Account::Bob.into()))
					.write(U256::from(500))
					.build(),
				None,
				&Context {
					address: Account::AssetId(0u128).into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
				false,
			);

			assert_eq!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Charlie.into()))
						.write(U256::from(400))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Bob.into(), // Bob is the one sending transferFrom!
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 56268756u64, // 1 weight => 1 gas in mock
					logs: LogsBuilder::new(Account::AssetId(0u128).into())
						.log3(
							SELECTOR_LOG_TRANSFER,
							Account::Alice,
							Account::Charlie,
							EvmDataWriter::new().write(U256::from(400)).build(),
						)
						.build(),
				}))
			);

			assert_eq!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
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
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Bob.into(),
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

			assert_eq!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Charlie.into()))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Charlie.into(),
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
fn transfer_from_non_incremental_approval() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			// We first approve 500
			assert_eq!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 30832756u64,
					logs: LogsBuilder::new(Account::AssetId(0u128).into())
						.log3(
							SELECTOR_LOG_APPROVAL,
							Account::Alice,
							Account::Bob,
							EvmDataWriter::new().write(U256::from(500)).build(),
						)
						.build(),
				}))
			);

			// We then approve 300. Non-incremental, so this is
			// the approved new value
			// Additionally, the gas used in this approval is higher because we
			// need to clear the previous one
			assert_eq!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(300))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 62796756u64,
					logs: LogsBuilder::new(Account::AssetId(0u128).into())
						.log3(
							SELECTOR_LOG_APPROVAL,
							Account::Alice,
							Account::Bob,
							EvmDataWriter::new().write(U256::from(300)).build(),
						)
						.build(),
				}))
			);

			// This should fail, as now the new approved quantity is 300
			assert_matches!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Bob.into(), // Bob is the one sending transferFrom!
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Err(PrecompileFailure::Revert { output, ..}))
				if output == b"Dispatched call failed with error: DispatchErrorWithPostInfo { \
					post_info: PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes }, \
					error: Module(ModuleError { index: 2, error: 10, message: Some(\"Unapproved\") }) }"
			);
		});
}

#[test]
fn transfer_from_above_allowance() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles().execute(
				Account::AssetId(0u128).into(),
				&EvmDataWriter::new_with_selector(Action::Approve)
					.write(Address(Account::Bob.into()))
					.write(U256::from(300))
					.build(),
				None,
				&Context {
					address: Account::AssetId(0u128).into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
				false,
			);

			assert_matches!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Bob.into(), // Bob is the one sending transferFrom!
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Err(PrecompileFailure::Revert { output, ..}))
				if output == b"Dispatched call failed with error: DispatchErrorWithPostInfo { \
					post_info: PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes }, \
					error: Module(ModuleError { index: 2, error: 10, message: Some(\"Unapproved\") }) }"
			);
		});
}

#[test]
fn transfer_from_self() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			assert_eq!(
				precompiles().execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
						// Alice sending transferFrom herself, no need for allowance.
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 44001756u64, // 1 weight => 1 gas in mock
					logs: LogsBuilder::new(Account::AssetId(0u128).into())
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
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
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
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::AssetId(0u128).into(),
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
fn get_metadata() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(Assets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(Assets::force_set_metadata(
				Origin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			assert_ok!(Assets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));
			{
				assert_eq!(
					precompiles().execute(
						Account::AssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Name).build(),
						None,
						&Context {
							address: Account::AssetId(0u128).into(),
							// Alice sending transferFrom herself, no need for allowance.
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new()
							.write::<Bytes>("TestToken".into())
							.build(),
						cost: Default::default(),
						logs: Default::default(),
					}))
				);

				assert_eq!(
					precompiles().execute(
						Account::AssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Symbol).build(),
						None,
						&Context {
							address: Account::AssetId(0u128).into(),
							// Alice sending transferFrom herself, no need for allowance.
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write::<Bytes>("Test".into()).build(),
						cost: Default::default(),
						logs: Default::default(),
					}))
				);

				assert_eq!(
					precompiles().execute(
						Account::AssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Decimals).build(),
						None,
						&Context {
							address: Account::AssetId(0u128).into(),
							// Alice sending transferFrom herself, no need for allowance.
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(12u8).build(),
						cost: Default::default(),
						logs: Default::default(),
					}))
				);
			};
		});
}
