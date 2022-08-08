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
use std::str::from_utf8;

use crate::{eip2612::Eip2612, mock::*, *};

use hex_literal::hex;
use libsecp256k1::{sign, Message, SecretKey};
use precompile_utils::testing::*;
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
		precompiles()
			.prepare_test(
				Account::Alice,
				Account::ForeignAssetId(0u128),
				vec![1u8, 2u8, 3u8],
			)
			.execute_reverts(|output| output == b"tried to parse selector out of bounds");
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

		precompiles()
			.prepare_test(
				Account::Alice,
				Account::ForeignAssetId(0u128),
				vec![1u8, 2u8, 3u8, 4u8],
			)
			.execute_reverts(|output| output == b"unknown selector");
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::TotalSupply).build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1000u64)).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1000u64)).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u64)).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
				)
				.expect_cost(36390756u64)
				.expect_log(log3(
					Account::ForeignAssetId(0u128),
					SELECTOR_LOG_APPROVAL,
					Account::Alice,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(500)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::MAX)
						.build(),
				)
				.expect_cost(36390756u64)
				.expect_log(log3(
					Account::ForeignAssetId(0u128),
					SELECTOR_LOG_APPROVAL,
					Account::Alice,
					Account::Bob,
					EvmDataWriter::new().write(U256::MAX).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(0u64)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(u128::MAX)).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
				)
				.execute_some();

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(500u64)).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u64)).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
				)
				.expect_cost(47402756u64) // 1 weight => 1 gas in mock
				.expect_log(log3(
					Account::ForeignAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					Account::Alice,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(400)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(400)).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(600)).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(Account::Charlie.into()))
						.write(U256::from(50))
						.build(),
				)
				.execute_reverts(|output| {
					from_utf8(&output)
						.unwrap()
						.contains("Dispatched call failed with error: DispatchErrorWithPostInfo")
						&& from_utf8(&output).unwrap().contains("BalanceLow")
				});
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
				)
				.execute_some();

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
				)
				.execute_some();

			precompiles()
				.prepare_test(
					Account::Bob, // Bob is the one sending transferFrom!
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Charlie.into()))
						.write(U256::from(400))
						.build(),
				)
				.expect_cost(61855756u64) // 1 weight => 1 gas in mock
				.expect_log(log3(
					Account::ForeignAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					Account::Alice,
					Account::Charlie,
					EvmDataWriter::new().write(U256::from(400)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(600)).build());

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0)).build());

			precompiles()
				.prepare_test(
					Account::Charlie,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Charlie.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(400)).build());
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
			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
				)
				.expect_cost(36390756u64)
				.expect_log(log3(
					Account::ForeignAssetId(0u128),
					SELECTOR_LOG_APPROVAL,
					Account::Alice,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(500)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			// We then approve 300. Non-incremental, so this is
			// the approved new value
			// Additionally, the gas used in this approval is higher because we
			// need to clear the previous one
			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(300))
						.build(),
				)
				.expect_cost(73149756u64)
				.expect_log(log3(
					Account::ForeignAssetId(0u128),
					SELECTOR_LOG_APPROVAL,
					Account::Alice,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(300)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			// This should fail, as now the new approved quantity is 300
			precompiles()
				.prepare_test(
					Account::Bob, // Bob is the one sending transferFrom!
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
				)
				.execute_reverts(|output| {
					output
						== b"Dispatched call failed with error: DispatchErrorWithPostInfo { \
					post_info: PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes }, \
					error: Module(ModuleError { index: 2, error: [10, 0, 0, 0], \
					message: Some(\"Unapproved\") }) }"
				});
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(300))
						.build(),
				)
				.execute_some();

			precompiles()
				.prepare_test(
					Account::Bob, // Bob is the one sending transferFrom!
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
				)
				.execute_reverts(|output| {
					output
						== b"Dispatched call failed with error: DispatchErrorWithPostInfo { \
					post_info: PostDispatchInfo { actual_weight: None, pays_fee: Pays::Yes }, \
					error: Module(ModuleError { index: 2, error: [10, 0, 0, 0], \
					message: Some(\"Unapproved\") }) }"
				});
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

			precompiles()
				.prepare_test(
					Account::Alice, // Alice sending transferFrom herself, no need for allowance.
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
				)
				.expect_cost(47402756u64) // 1 weight => 1 gas in mock
				.expect_log(log3(
					Account::ForeignAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					Account::Alice,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(400)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(600)).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(400)).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Name).build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write::<Bytes>("TestToken".into())
						.build(),
				);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Symbol).build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write::<Bytes>("Test".into()).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Decimals).build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(12u8).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Mint)
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
				)
				.execute_reverts(|output| output == b"unknown selector");

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Burn)
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
				)
				.execute_reverts(|output| output == b"unknown selector");
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Mint)
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
				)
				.expect_cost(30820756u64) // 1 weight => 1 gas in mock
				.expect_log(log3(
					Account::LocalAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					Account::Zero,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(400)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(400)).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Burn)
						.write(Address(Account::Alice.into()))
						.write(U256::from(400))
						.build(),
				)
				.expect_cost(35213756u64) // 1 weight => 1 gas in mock
				.expect_log(log3(
					Account::LocalAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					Account::Alice,
					Account::Zero,
					EvmDataWriter::new().write(U256::from(400)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(600)).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Freeze)
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(21670000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(Account::Alice.into()))
						.write(U256::from(400))
						.build(),
				)
				.execute_reverts(|output| {
					from_utf8(&output)
						.unwrap()
						.contains("Dispatched call failed with error: DispatchErrorWithPostInfo")
						&& from_utf8(&output).unwrap().contains("Frozen")
				});
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Freeze)
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(21670000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Thaw)
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(21503000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(Account::Alice.into()))
						.write(U256::from(400))
						.build(),
				)
				.expect_cost(47402756u64) // 1 weight => 1 gas in mock
				.expect_log(log3(
					Account::LocalAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					Account::Bob,
					Account::Alice,
					EvmDataWriter::new().write(U256::from(400)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::FreezeAsset).build(),
				)
				.expect_cost(18158000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(Account::Alice.into()))
						.write(U256::from(400))
						.build(),
				)
				.execute_reverts(|output| {
					from_utf8(&output)
						.unwrap()
						.contains("Dispatched call failed with error: DispatchErrorWithPostInfo")
						&& from_utf8(&output).unwrap().contains("Frozen")
				});
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::FreezeAsset).build(),
				)
				.expect_cost(18158000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::ThawAsset).build(),
				)
				.expect_cost(18525000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(Account::Alice.into()))
						.write(U256::from(400))
						.build(),
				)
				.expect_cost(47402756u64) // 1 weight => 1 gas in mock
				.expect_log(log3(
					Account::LocalAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					Account::Bob,
					Account::Alice,
					EvmDataWriter::new().write(U256::from(400)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::TransferOwnership)
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(19858000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			// Now Bob should be able to change ownership, and not Alice
			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::TransferOwnership)
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.execute_reverts(|output| {
					from_utf8(&output)
						.unwrap()
						.contains("Dispatched call failed with error: DispatchErrorWithPostInfo")
						&& from_utf8(&output).unwrap().contains("NoPermission")
				});

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::TransferOwnership)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(19858000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::SetTeam)
						.write(Address(Account::Bob.into()))
						.write(Address(Account::Bob.into()))
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(18045000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			// Now Bob should be able to mint, and not Alice
			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Mint)
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
				)
				.execute_reverts(|output| {
					from_utf8(&output)
						.unwrap()
						.contains("Dispatched call failed with error: DispatchErrorWithPostInfo")
						&& from_utf8(&output).unwrap().contains("NoPermission")
				});

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Mint)
						.write(Address(Account::Bob.into()))
						.write(U256::from(400))
						.build(),
				)
				.expect_cost(30820756u64) // 1 weight => 1 gas in mock
				.expect_log(log3(
					Account::LocalAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					Account::Zero,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(400)).build(),
				))
				.execute_returns(EvmDataWriter::new().write(true).build());

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(400)).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::SetMetadata)
						.write::<Bytes>("TestToken".into())
						.write::<Bytes>("Test".into())
						.write::<u8>(12)
						.build(),
				)
				.expect_cost(32448000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Name).build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write::<Bytes>("TestToken".into())
						.build(),
				);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Symbol).build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write::<Bytes>("Test".into()).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Decimals).build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(12u8).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::SetMetadata)
						.write::<Bytes>("TestToken".into())
						.write::<Bytes>("Test".into())
						.write::<u8>(12)
						.build(),
				)
				.expect_cost(32448000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::ClearMetadata).build(),
				)
				.expect_cost(32893000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(true).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Name).build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write::<Bytes>("".into()).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Symbol).build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write::<Bytes>("".into()).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Decimals).build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(0u8).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u8)).build());

			precompiles()
				.prepare_test(
					Account::Charlie,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Eip2612Permit)
						.write(Address(owner))
						.write(Address(spender))
						.write(value)
						.write(deadline)
						.write(v.serialize())
						.write(H256::from(rs.r.b32()))
						.write(H256::from(rs.s.b32()))
						.build(),
				)
				.expect_cost(36389000u64)
				.expect_log(log3(
					Account::ForeignAssetId(0u128),
					SELECTOR_LOG_APPROVAL,
					Account::Alice,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(500)).build(),
				))
				.execute_returns(vec![]);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(500u16)).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1u8)).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u8)).build());

			precompiles()
				.prepare_test(
					Account::Charlie,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Eip2612Permit)
						.write(Address(owner))
						.write(Address(spender))
						.write(value)
						.write(deadline)
						.write(v.serialize())
						.write(H256::from(rs.r.b32()))
						.write(H256::from(rs.s.b32()))
						.build(),
				)
				.expect_cost(36389000u64)
				.expect_log(log3(
					Account::ForeignAssetId(0u128),
					SELECTOR_LOG_APPROVAL,
					Account::Alice,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(500)).build(),
				))
				.execute_returns(vec![]);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(500u16)).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1u8)).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u8)).build());

			precompiles()
				.prepare_test(
					Account::Charlie,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Eip2612Permit)
						.write(Address(owner))
						.write(Address(spender))
						.write(value)
						.write(deadline)
						.write(v.serialize())
						.write(H256::from(rs.r.b32()))
						.write(H256::from(rs.s.b32()))
						.build(),
				)
				.execute_reverts(|output| output == b"invalid permit");

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u16)).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u8)).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u8)).build());

			precompiles()
				.prepare_test(
					Account::Charlie,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Eip2612Permit)
						.write(Address(owner))
						.write(Address(spender))
						.write(value)
						.write(deadline)
						.write(0u8)
						.write(H256::random())
						.write(H256::random())
						.build(),
				)
				.execute_reverts(|output| output == b"invalid permit");

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u16)).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u8)).build());
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u8)).build());

			precompiles()
				.prepare_test(
					Account::Charlie,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Eip2612Permit)
						.write(Address(owner))
						.write(Address(spender))
						.write(value)
						.write(deadline)
						.write(v.serialize())
						.write(H256::from(rs.r.b32()))
						.write(H256::from(rs.s.b32()))
						.build(),
				)
				.execute_reverts(|output| output == b"permit expired");

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Allowance)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u16)).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Eip2612Nonces)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0u8)).build());
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

			precompiles()
				.prepare_test(
					Account::Charlie,
					Account::ForeignAssetId(1u128),
					EvmDataWriter::new_with_selector(Action::Eip2612Permit)
						.write(Address(owner))
						.write(Address(spender))
						.write(value)
						.write(deadline)
						.write(v_real)
						.write(H256::from(r_real))
						.write(H256::from(s_real))
						.build(),
				)
				.expect_cost(36389000u64)
				.expect_log(log3(
					Account::ForeignAssetId(1u128),
					SELECTOR_LOG_APPROVAL,
					Account::Alice,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(1000)).build(),
				))
				.execute_returns(vec![]);
		});
}

#[test]
fn transfer_amount_overflow() {
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Transfer)
						.write(Address(Account::Bob.into()))
						.write(U256::from(u128::MAX) + 1)
						.build(),
				)
				.expect_cost(1756u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_reverts(|e| e == b"value too big for u128");

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Bob.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(0)).build());

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::BalanceOf)
						.write(Address(Account::Alice.into()))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().write(U256::from(1000)).build());
		});
}

#[test]
fn transfer_from_overflow() {
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
				)
				.execute_some();

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Approve)
						.write(Address(Account::Bob.into()))
						.write(U256::from(500))
						.build(),
				)
				.execute_some();

			precompiles()
				.prepare_test(
					Account::Bob, // Bob is the one sending transferFrom!
					Account::ForeignAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::TransferFrom)
						.write(Address(Account::Alice.into()))
						.write(Address(Account::Charlie.into()))
						.write(U256::from(u128::MAX) + 1)
						.build(),
				)
				.expect_cost(1756u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_reverts(|e| e == b"value too big for u128");
		});
}

#[test]
fn mint_overflow() {
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Mint)
						.write(Address(Account::Bob.into()))
						.write(U256::from(u128::MAX) + 1)
						.build(),
				)
				.expect_cost(1756u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_reverts(|e| e == b"value too big for u128");
		});
}

#[test]
fn burn_overflow() {
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

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					EvmDataWriter::new_with_selector(Action::Burn)
						.write(Address(Account::Alice.into()))
						.write(U256::from(u128::MAX) + 1)
						.build(),
				)
				.expect_cost(1756u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_reverts(|e| e == b"value too big for u128");
		});
}
