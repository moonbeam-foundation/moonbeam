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

use crate::{eip2612::Eip2612, mock::*, *};

use fp_evm::{Context, PrecompileFailure};
use hex_literal::hex;
use libsecp256k1::{sign, Message, SecretKey};
use pallet_evm::PrecompileSet;
use precompile_utils::{EvmDataWriter, LogsBuilder};
use sha3::{Digest, Keccak256};
use sp_core::H256;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ForeignAssets::force_create(
			Origin::root(),
			0u128,
			Account::Alice.into(),
			true,
			1
		));
		// This selector is only three bytes long when four are required.
		let bogus_selector = vec![1u8, 2u8, 3u8];

		assert_matches!(
			precompiles().execute(
				Account::ForeignAssetId(0u128).into(),
				&bogus_selector,
				None,
				&Context {
					address: Account::ForeignAssetId(1u128).into(),
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
		assert_ok!(ForeignAssets::force_create(
			Origin::root(),
			0u128,
			Account::Alice.into(),
			true,
			1
		));
		let bogus_selector = vec![1u8, 2u8, 3u8, 4u8];

		assert_matches!(
			precompiles().execute(
				Account::ForeignAssetId(0u128).into(),
				&bogus_selector,
				None,
				&Context {
					address: Account::ForeignAssetId(1u128).into(),
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
	assert_eq!(Action::BalanceOf as u32, 0x70a08231);
	assert_eq!(Action::TotalSupply as u32, 0x18160ddd);
	assert_eq!(Action::Approve as u32, 0x095ea7b3);
	assert_eq!(Action::Allowance as u32, 0xdd62ed3e);
	assert_eq!(Action::Transfer as u32, 0xa9059cbb);
	assert_eq!(Action::TransferFrom as u32, 0x23b872dd);
	assert_eq!(Action::Name as u32, 0x06fdde03);
	assert_eq!(Action::Symbol as u32, 0x95d89b41);
	assert_eq!(Action::Decimals as u32, 0x313ce567);
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
}

#[test]
fn get_total_supply() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));
			assert_eq!(
				precompiles().execute(
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::TotalSupply).build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));
			assert_eq!(
				precompiles().execute(
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_eq!(
				precompiles().execute(
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			assert_eq!(
				precompiles().execute(
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 30832756u64,
					logs: LogsBuilder::new(Account::ForeignAssetId(0u128).into())
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
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			assert_eq!(
				precompiles().execute(
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::MAX)
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 30832756u64,
					logs: LogsBuilder::new(Account::ForeignAssetId(0u128).into())
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles().execute(
				Account::ForeignAssetId(0u128).into(),
				&EvmDataWriter::new_with_selector(Action::Approve)
					.write(Address(Account::Bob.into()))
					.write(U256::from(500))
					.build(),
				None,
				&Context {
					address: Account::ForeignAssetId(0u128).into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
				false,
			);

			assert_eq!(
				precompiles().execute(
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_eq!(
				precompiles().execute(
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			assert_eq!(
				precompiles().execute(
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 44001756u64, // 1 weight => 1 gas in mock
					logs: LogsBuilder::new(Account::ForeignAssetId(0u128).into())
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1
			));

			assert_matches!(
				precompiles().execute(
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(Account::Charlie.into()))
						.write(U256::from(50))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles().execute(
				Account::ForeignAssetId(0u128).into(),
				&EvmDataWriter::new_with_selector(Action::Approve)
					.write(Address(Account::Bob.into()))
					.write(U256::from(500))
					.build(),
				None,
				&Context {
					address: Account::ForeignAssetId(0u128).into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
				false,
			);

			assert_eq!(
				precompiles().execute(
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Charlie.into()))
						.write(U256::from(400))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
						caller: Account::Bob.into(), // Bob is the one sending transferFrom!
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 56268756u64, // 1 weight => 1 gas in mock
					logs: LogsBuilder::new(Account::ForeignAssetId(0u128).into())
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Charlie.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			// We first approve 500
			assert_eq!(
				precompiles().execute(
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 30832756u64,
					logs: LogsBuilder::new(Account::ForeignAssetId(0u128).into())
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(300))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(true).build(),
					cost: 62796756u64,
					logs: LogsBuilder::new(Account::ForeignAssetId(0u128).into())
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
						caller: Account::Bob.into(), // Bob is the one sending transferFrom!
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Err(PrecompileFailure::Revert { output, ..}))
				if output == b"Dispatched call failed with error: DispatchErrorWithPostInfo { \
					post_info: PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes }, \
					error: Module(ModuleError { index: 2, error: [10, 0, 0, 0], \
					message: Some(\"Unapproved\") }) }"
			);
		});
}

#[test]
fn transfer_from_above_allowance() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles().execute(
				Account::ForeignAssetId(0u128).into(),
				&EvmDataWriter::new_with_selector(Action::Approve)
					.write(Address(Account::Bob.into()))
					.write(U256::from(300))
					.build(),
				None,
				&Context {
					address: Account::ForeignAssetId(0u128).into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
				false,
			);

			assert_matches!(
				precompiles().execute(
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
						caller: Account::Bob.into(), // Bob is the one sending transferFrom!
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Err(PrecompileFailure::Revert { output, ..}))
				if output == b"Dispatched call failed with error: DispatchErrorWithPostInfo { \
					post_info: PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes }, \
					error: Module(ModuleError { index: 2, error: [10, 0, 0, 0], \
					message: Some(\"Unapproved\") }) }"
			);
		});
}

#[test]
fn transfer_from_self() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			assert_eq!(
				precompiles().execute(
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
					logs: LogsBuilder::new(Account::ForeignAssetId(0u128).into())
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::force_set_metadata(
				Origin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			{
				assert_eq!(
					precompiles().execute(
						Account::ForeignAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Name).build(),
						None,
						&Context {
							address: Account::ForeignAssetId(0u128).into(),
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
						Account::ForeignAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Symbol).build(),
						None,
						&Context {
							address: Account::ForeignAssetId(0u128).into(),
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
						Account::ForeignAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Decimals).build(),
						None,
						&Context {
							address: Account::ForeignAssetId(0u128).into(),
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

#[test]
fn local_functions_cannot_be_accessed_by_foreign_assets() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::force_set_metadata(
				Origin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			{
				assert_matches!(
					precompiles().execute(
						Account::ForeignAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Mint)
							.write(Address(Account::Bob.into()))
							.write(U256::from(400))
							.build(),
						None,
						&Context {
							address: Account::ForeignAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Err(PrecompileFailure::Revert { output, .. }))
					if output == b"unknown selector"
				);
			};
			{
				assert_matches!(
					precompiles().execute(
						Account::ForeignAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Burn)
							.write(Address(Account::Bob.into()))
							.write(U256::from(400))
							.build(),
						None,
						&Context {
							address: Account::ForeignAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Err(PrecompileFailure::Revert { output, .. }))
					if output == b"unknown selector"
				);
			};
		});
}

#[test]
fn mint_local_assets() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				Origin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			{
				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Mint)
							.write(Address(Account::Bob.into()))
							.write(U256::from(400))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 26633756u64, // 1 weight => 1 gas in mock
						logs: LogsBuilder::new(Account::LocalAssetId(0u128).into())
							.log3(
								SELECTOR_LOG_TRANSFER,
								Account::Zero,
								Account::Bob,
								EvmDataWriter::new().write(U256::from(400)).build(),
							)
							.build(),
					}))
				);

				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::BalanceOf)
							.write(Address(Account::Bob.into()))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
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
			};
		});
}

#[test]
fn burn_local_assets() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				Origin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			assert_ok!(LocalAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));
			{
				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Burn)
							.write(Address(Account::Alice.into()))
							.write(U256::from(400))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 30049756u64, // 1 weight => 1 gas in mock
						logs: LogsBuilder::new(Account::LocalAssetId(0u128).into())
							.log3(
								SELECTOR_LOG_TRANSFER,
								Account::Alice,
								Account::Zero,
								EvmDataWriter::new().write(U256::from(400)).build(),
							)
							.build(),
					}))
				);

				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::BalanceOf)
							.write(Address(Account::Alice.into()))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
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
			};
		});
}

#[test]
fn freeze_local_assets() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				Origin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			assert_ok!(LocalAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Bob.into(),
				1000
			));
			{
				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Freeze)
							.write(Address(Account::Bob.into()))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 18309000u64, // 1 weight => 1 gas in mock
						logs: vec![],
					}))
				);

				assert_matches!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Transfer)
							.write(Address(Account::Alice.into()))
							.write(U256::from(400))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Bob.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Err(PrecompileFailure::Revert { output: str, ..}))
						if from_utf8(&str).unwrap()
							.contains("Dispatched call failed with error: DispatchErrorWithPostInfo")
						&& from_utf8(&str).unwrap().contains("Frozen")
				);
			};
		});
}

#[test]
fn thaw_local_assets() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				Origin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			assert_ok!(LocalAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Bob.into(),
				1000
			));
			{
				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Freeze)
							.write(Address(Account::Bob.into()))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 18309000u64, // 1 weight => 1 gas in mock
						logs: vec![],
					}))
				);

				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Thaw)
							.write(Address(Account::Bob.into()))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 18290000u64, // 1 weight => 1 gas in mock
						logs: vec![],
					}))
				);

				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Transfer)
							.write(Address(Account::Alice.into()))
							.write(U256::from(400))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Bob.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 44001756u64, // 1 weight => 1 gas in mock
						logs: LogsBuilder::new(Account::LocalAssetId(0u128).into())
							.log3(
								SELECTOR_LOG_TRANSFER,
								Account::Bob,
								Account::Alice,
								EvmDataWriter::new().write(U256::from(400)).build(),
							)
							.build(),
					}))
				);
			};
		});
}

#[test]
fn freeze_asset_local_asset() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				Origin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			assert_ok!(LocalAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Bob.into(),
				1000
			));
			{
				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::FreezeAsset).build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 14744000u64, // 1 weight => 1 gas in mock
						logs: vec![],
					}))
				);

				assert_matches!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Transfer)
							.write(Address(Account::Alice.into()))
							.write(U256::from(400))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Bob.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Err(PrecompileFailure::Revert { output: str, ..}))
						if from_utf8(&str).unwrap()
							.contains("Dispatched call failed with error: DispatchErrorWithPostInfo")
						&& from_utf8(&str).unwrap().contains("Frozen")
				);
			};
		});
}

#[test]
fn thaw_asset_local_assets() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				Origin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			assert_ok!(LocalAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Bob.into(),
				1000
			));
			{
				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::FreezeAsset).build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 14744000u64, // 1 weight => 1 gas in mock
						logs: vec![],
					}))
				);

				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::ThawAsset).build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 14833000u64, // 1 weight => 1 gas in mock
						logs: vec![],
					}))
				);

				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Transfer)
							.write(Address(Account::Alice.into()))
							.write(U256::from(400))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Bob.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 44001756u64, // 1 weight => 1 gas in mock
						logs: LogsBuilder::new(Account::LocalAssetId(0u128).into())
							.log3(
								SELECTOR_LOG_TRANSFER,
								Account::Bob,
								Account::Alice,
								EvmDataWriter::new().write(U256::from(400)).build(),
							)
							.build(),
					}))
				);
			};
		});
}

#[test]
fn transfer_ownership_local_assets() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				Origin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			{
				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::TransferOwnership)
							.write(Address(Account::Bob.into()))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 16654000u64, // 1 weight => 1 gas in mock
						logs: vec![],
					}))
				);

				// Now Bob should be able to change ownership, and not Alice
				assert_matches!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::TransferOwnership)
							.write(Address(Account::Bob.into()))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Err(PrecompileFailure::Revert { output: str, ..}))
						if from_utf8(&str).unwrap()
							.contains("Dispatched call failed with error: DispatchErrorWithPostInfo")
						&& from_utf8(&str).unwrap().contains("NoPermission")
				);

				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::TransferOwnership)
							.write(Address(Account::Alice.into()))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Bob.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 16654000u64, // 1 weight => 1 gas in mock
						logs: vec![],
					}))
				);
			};
		});
}

#[test]
fn set_team_local_assets() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				Origin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			{
				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::SetTeam)
							.write(Address(Account::Bob.into()))
							.write(Address(Account::Bob.into()))
							.write(Address(Account::Bob.into()))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 15351000u64, // 1 weight => 1 gas in mock
						logs: vec![],
					}))
				);

				// Now Bob should be able to mint, and not Alice
				assert_matches!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Mint)
							.write(Address(Account::Bob.into()))
							.write(U256::from(400))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Err(PrecompileFailure::Revert { output: str, ..}))
						if from_utf8(&str).unwrap()
							.contains("Dispatched call failed with error: DispatchErrorWithPostInfo")
						&& from_utf8(&str).unwrap().contains("NoPermission")
				);

				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Mint)
							.write(Address(Account::Bob.into()))
							.write(U256::from(400))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Bob.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 26633756u64, // 1 weight => 1 gas in mock
						logs: LogsBuilder::new(Account::LocalAssetId(0u128).into())
							.log3(
								SELECTOR_LOG_TRANSFER,
								Account::Zero,
								Account::Bob,
								EvmDataWriter::new().write(U256::from(400)).build(),
							)
							.build(),
					}))
				);

				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::BalanceOf)
							.write(Address(Account::Bob.into()))
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
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
			};
		});
}

#[test]
fn set_metadata() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				Origin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			{
				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::SetMetadata)
							.write::<Bytes>("TestToken".into())
							.write::<Bytes>("Test".into())
							.write::<u8>(12)
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 27654000u64, // 1 weight => 1 gas in mock
						logs: vec![],
					}))
				);

				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Name).build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
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
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Symbol).build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
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
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Decimals).build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
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

#[test]
fn clear_metadata() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				Origin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			{
				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::SetMetadata)
							.write::<Bytes>("TestToken".into())
							.write::<Bytes>("Test".into())
							.write::<u8>(12)
							.build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 27654000u64, // 1 weight => 1 gas in mock
						logs: vec![],
					}))
				);

				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::ClearMetadata).build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(true).build(),
						cost: 27710000u64, // 1 weight => 1 gas in mock
						logs: vec![],
					}))
				);

				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Name).build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write::<Bytes>("".into()).build(),
						cost: Default::default(),
						logs: Default::default(),
					}))
				);

				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Symbol).build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write::<Bytes>("".into()).build(),
						cost: Default::default(),
						logs: Default::default(),
					}))
				);

				assert_eq!(
					precompiles().execute(
						Account::LocalAssetId(0u128).into(),
						&EvmDataWriter::new_with_selector(Action::Decimals).build(),
						None,
						&Context {
							address: Account::LocalAssetId(0u128).into(),
							caller: Account::Alice.into(),
							apparent_value: From::from(0),
						},
						false,
					),
					Some(Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new().write(0u8).build(),
						cost: Default::default(),
						logs: Default::default(),
					}))
				);
			};
		});
}

#[test]
fn permit_valid() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			let owner: H160 = Account::Alice.into();
			let spender: H160 = Account::Bob.into();
			let value: U256 = 500u16.into();
			let deadline: U256 = 0u8.into(); // todo: proper timestamp

			let permit = Eip2612::<Runtime, IsLocal, pallet_assets::Instance1>::generate_permit(
				Account::ForeignAssetId(0u128).into(),
				0u128,
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
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
						address: Account::ForeignAssetId(0u128).into(),
						caller: Account::Charlie.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: vec![],
					cost: 30831000u64,
					logs: LogsBuilder::new(Account::ForeignAssetId(0u128).into())
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
fn permit_valid_named_asset() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));
			assert_ok!(ForeignAssets::set_metadata(
				Origin::signed(Account::Alice),
				0u128,
				b"Test token".to_vec(),
				b"TEST".to_vec(),
				18
			));

			let owner: H160 = Account::Alice.into();
			let spender: H160 = Account::Bob.into();
			let value: U256 = 500u16.into();
			let deadline: U256 = 0u8.into(); // todo: proper timestamp

			let permit = Eip2612::<Runtime, IsLocal, pallet_assets::Instance1>::generate_permit(
				Account::ForeignAssetId(0u128).into(),
				0u128,
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
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
						address: Account::ForeignAssetId(0u128).into(),
						caller: Account::Charlie.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: vec![],
					cost: 30831000u64,
					logs: LogsBuilder::new(Account::ForeignAssetId(0u128).into())
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			let owner: H160 = Account::Alice.into();
			let spender: H160 = Account::Bob.into();
			let value: U256 = 500u16.into();
			let deadline: U256 = 0u8.into();

			let permit = Eip2612::<Runtime, IsLocal, pallet_assets::Instance1>::generate_permit(
				Account::ForeignAssetId(0u128).into(),
				0u128,
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
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
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			let owner: H160 = Account::Alice.into();
			let spender: H160 = Account::Bob.into();
			let value: U256 = 500u16.into();
			let deadline: U256 = 0u8.into();

			assert_eq!(
				precompiles().execute(
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
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
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			pallet_timestamp::Pallet::<Runtime>::set_timestamp(10_000);

			let owner: H160 = Account::Alice.into();
			let spender: H160 = Account::Bob.into();
			let value: U256 = 500u16.into();
			let deadline: U256 = 5u8.into(); // deadline < timestamp => expired

			let permit = Eip2612::<Runtime, IsLocal, pallet_assets::Instance1>::generate_permit(
				Account::ForeignAssetId(0u128).into(),
				0u128,
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
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
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
					Account::ForeignAssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&Context {
						address: Account::ForeignAssetId(0u128).into(),
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
		name: "Unnamed XC20 #1",
		version: "1",
		chainId: 0,
		verifyingContract: "0xffffffff00000000000000000000000000000001",
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
			// assetId 1
			assert_ok!(ForeignAssets::force_create(
				Origin::root(),
				1u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				Origin::signed(Account::Alice),
				1u128,
				Account::Alice.into(),
				1000
			));

			let owner: H160 = H160::from_slice(ALICE_PUBLIC_KEY.as_slice());
			let spender: H160 = Account::Bob.into();
			let value: U256 = 1000u16.into();
			let deadline: U256 = 1u16.into(); // todo: proper timestamp

			let rsv = hex!(
				"3aac886f06729d76067b6b0dbae23978fe48224b10b5648265b8f0e8c4cf25ff7625965d64bf9a6069d
				b00ef5771b65fd24dd118531fc6e86b61a238ca76b9a11c"
			)
			.as_slice();
			let (r, sv) = rsv.split_at(32);
			let (s, v) = sv.split_at(32);
			let v_real = v[0];
			let r_real: [u8; 32] = r.try_into().unwrap();
			let s_real: [u8; 32] = s.try_into().unwrap();

			assert_eq!(
				precompiles().execute(
					Account::ForeignAssetId(1u128).into(),
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
						address: Account::ForeignAssetId(1u128).into(),
						caller: Account::Charlie.into(),
						apparent_value: From::from(0),
					},
					false,
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: vec![],
					cost: 30831000u64,
					logs: LogsBuilder::new(Account::ForeignAssetId(1u128).into())
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
