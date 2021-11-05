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

use frame_support::assert_ok;
use std::assert_matches::assert_matches;

use crate::mock::*;
use crate::*;

use pallet_evm::PrecompileSet;
use precompile_utils::{error, EvmDataWriter, LogsBuilder};
use sha3::{Digest, Keccak256};

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

		assert_eq!(
			Erc20AssetsPrecompileSet::<Runtime>::execute(
				Account::AssetId(0u128).into(),
				&bogus_selector,
				None,
				&evm::Context {
					address: Account::AssetId(1u128).into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
			),
			Some(Err(error("tried to parse selector out of bounds")))
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

		assert_eq!(
			Erc20AssetsPrecompileSet::<Runtime>::execute(
				Account::AssetId(0u128).into(),
				&bogus_selector,
				None,
				&evm::Context {
					address: Account::AssetId(1u128).into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
			),
			Some(Err(error("unknown selector")))
		);
	});
}

#[test]
fn selectors() {
	assert_eq!(u32::from(Action::BalanceOf), 0x70a08231);
	assert_eq!(u32::from(Action::TotalSupply), 0x18160ddd);
	assert_eq!(u32::from(Action::Approve), 0x095ea7b3);
	//assert_eq!(u32::from(Action::Allowance), 0xdd62ed3e);
	assert_eq!(u32::from(Action::Transfer), 0xa9059cbb);
	assert_eq!(u32::from(Action::TransferFrom), 0x23b872dd);

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
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::TotalSupply).build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
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
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
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
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
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
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: Default::default(),
					cost: 56999756u64,
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

// This should be added once https://github.com/paritytech/substrate/pull/9757 is merged.
#[ignore]
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

			Erc20AssetsPrecompileSet::<Runtime>::execute(
				Account::AssetId(0u128).into(),
				&EvmDataWriter::new_with_selector(Action::Approve)
					.write(Address(Account::Bob.into()))
					.write(U256::from(500))
					.build(),
				None,
				&evm::Context {
					address: Account::AssetId(0u128).into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
			);

			assert_eq!(
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
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

// This should be added once https://github.com/paritytech/substrate/pull/9757 is merged.
#[ignore]
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
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
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
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: Default::default(),
					cost: 83206756u64, // 1 weight => 1 gas in mock
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
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Bob.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(400)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);

			assert_eq!(
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
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
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(Account::Charlie.into()))
						.write(U256::from(50))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Err(ExitError::Other(str)))
					if str.contains("Dispatched call failed with error: DispatchErrorWithPostInfo")
					&& str.contains("BalanceLow")
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

			Erc20AssetsPrecompileSet::<Runtime>::execute(
				Account::AssetId(0u128).into(),
				&EvmDataWriter::new_with_selector(Action::Approve)
					.write(Address(Account::Bob.into()))
					.write(U256::from(500))
					.build(),
				None,
				&evm::Context {
					address: Account::AssetId(0u128).into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
			);

			assert_eq!(
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Charlie.into()))
						.write(U256::from(400))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Bob.into(), // Bob is the one sending transferFrom!
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: Default::default(),
					cost: 107172756u64, // 1 weight => 1 gas in mock
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
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(600)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);

			assert_eq!(
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Bob.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(0)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);

			assert_eq!(
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Charlie.into()))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Charlie.into(),
						apparent_value: From::from(0),
					},
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
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: Default::default(),
					cost: 56999756u64,
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
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(300))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: Default::default(),
					cost: 114357756u64,
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
			assert_eq!(
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Bob.into(), // Bob is the one sending transferFrom!
						apparent_value: From::from(0),
					},
				),
				Some(Err(error(
					"Dispatched call failed with error: DispatchErrorWithPostInfo { \
					post_info: PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes }, \
					error: Module { index: 2, error: 10, message: Some(\"Unapproved\") } }"
				))),
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

			Erc20AssetsPrecompileSet::<Runtime>::execute(
				Account::AssetId(0u128).into(),
				&EvmDataWriter::new_with_selector(Action::Approve)
					.write(Address(Account::Bob.into()))
					.write(U256::from(300))
					.build(),
				None,
				&evm::Context {
					address: Account::AssetId(0u128).into(),
					caller: Account::Alice.into(),
					apparent_value: From::from(0),
				},
			);

			assert_eq!(
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Bob.into(), // Bob is the one sending transferFrom!
						apparent_value: From::from(0),
					},
				),
				Some(Err(error(
					"Dispatched call failed with error: DispatchErrorWithPostInfo { \
					post_info: PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes }, \
					error: Module { index: 2, error: 10, message: Some(\"Unapproved\") } }"
				))),
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
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						// Alice sending transferFrom herself, no need for allowance.
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: Default::default(),
					cost: 83206756u64, // 1 weight => 1 gas in mock
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
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
				),
				Some(Ok(PrecompileOutput {
					exit_status: ExitSucceed::Returned,
					output: EvmDataWriter::new().write(U256::from(600)).build(),
					cost: Default::default(),
					logs: Default::default(),
				}))
			);

			assert_eq!(
				Erc20AssetsPrecompileSet::<Runtime>::execute(
					Account::AssetId(0u128).into(),
					&EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
					None,
					&evm::Context {
						address: Account::AssetId(0u128).into(),
						caller: Account::Alice.into(),
						apparent_value: From::from(0),
					},
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
