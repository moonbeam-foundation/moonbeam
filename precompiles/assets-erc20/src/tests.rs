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

use crate::{eip2612::Eip2612, mock::*, *};
use frame_support::assert_ok;
use hex_literal::hex;
use libsecp256k1::{sign, Message, SecretKey};
use precompile_utils::{solidity, testing::*};
use sha3::{Digest, Keccak256};
use sp_core::H256;
use std::str::from_utf8;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ForeignAssets::force_create(
			RuntimeOrigin::root(),
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
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(ForeignAssets::force_create(
			RuntimeOrigin::root(),
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
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn selectors() {
	assert!(ForeignPCall::balance_of_selectors().contains(&0x70a08231));
	assert!(ForeignPCall::total_supply_selectors().contains(&0x18160ddd));
	assert!(ForeignPCall::approve_selectors().contains(&0x095ea7b3));
	assert!(ForeignPCall::allowance_selectors().contains(&0xdd62ed3e));
	assert!(ForeignPCall::transfer_selectors().contains(&0xa9059cbb));
	assert!(ForeignPCall::transfer_from_selectors().contains(&0x23b872dd));
	assert!(ForeignPCall::name_selectors().contains(&0x06fdde03));
	assert!(ForeignPCall::symbol_selectors().contains(&0x95d89b41));
	assert!(ForeignPCall::decimals_selectors().contains(&0x313ce567));
	assert!(ForeignPCall::eip2612_nonces_selectors().contains(&0x7ecebe00));
	assert!(ForeignPCall::eip2612_permit_selectors().contains(&0xd505accf));
	assert!(ForeignPCall::eip2612_domain_separator_selectors().contains(&0x3644e515));

	assert!(ForeignPCall::mint_selectors().contains(&0x40c10f19));
	assert!(ForeignPCall::burn_selectors().contains(&0x9dc29fac));
	assert!(ForeignPCall::freeze_selectors().contains(&0x8d1fdf2f));
	assert!(ForeignPCall::thaw_selectors().contains(&0x5ea20216));
	assert!(ForeignPCall::freeze_asset_selectors().contains(&0xd4937f51));
	assert!(ForeignPCall::thaw_asset_selectors().contains(&0x51ec2ad7));
	assert!(ForeignPCall::transfer_ownership_selectors().contains(&0xf2fde38b));
	assert!(ForeignPCall::set_team_selectors().contains(&0xc7d93c59));
	assert!(ForeignPCall::set_metadata_selectors().contains(&0x37d2c2f4));
	assert!(ForeignPCall::clear_metadata_selectors().contains(&0xefb6d432));

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
fn modifiers() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			let mut tester = PrecompilesModifierTester::new(
				precompiles(),
				Account::Alice,
				Account::ForeignAssetId(0u128),
			);

			tester.test_view_modifier(ForeignPCall::balance_of_selectors());
			tester.test_view_modifier(ForeignPCall::total_supply_selectors());
			tester.test_default_modifier(ForeignPCall::approve_selectors());
			tester.test_view_modifier(ForeignPCall::allowance_selectors());
			tester.test_default_modifier(ForeignPCall::transfer_selectors());
			tester.test_default_modifier(ForeignPCall::transfer_from_selectors());
			tester.test_view_modifier(ForeignPCall::name_selectors());
			tester.test_view_modifier(ForeignPCall::symbol_selectors());
			tester.test_view_modifier(ForeignPCall::decimals_selectors());
			tester.test_view_modifier(ForeignPCall::eip2612_nonces_selectors());
			tester.test_default_modifier(ForeignPCall::eip2612_permit_selectors());
			tester.test_view_modifier(ForeignPCall::eip2612_domain_separator_selectors());

			tester.test_default_modifier(ForeignPCall::mint_selectors());
			tester.test_default_modifier(ForeignPCall::burn_selectors());
			tester.test_default_modifier(ForeignPCall::freeze_selectors());
			tester.test_default_modifier(ForeignPCall::thaw_selectors());
			tester.test_default_modifier(ForeignPCall::freeze_asset_selectors());
			tester.test_default_modifier(ForeignPCall::thaw_asset_selectors());
			tester.test_default_modifier(ForeignPCall::transfer_ownership_selectors());
			tester.test_default_modifier(ForeignPCall::set_team_selectors());
			tester.test_default_modifier(ForeignPCall::set_metadata_selectors());
			tester.test_default_modifier(ForeignPCall::clear_metadata_selectors());
		});
}

#[test]
fn get_total_supply() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::total_supply {},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(1000u64));
		});
}

#[test]
fn get_balances_known_user() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::balance_of {
						who: Address(Account::Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(1000u64));
		});
}

#[test]
fn get_balances_unknown_user() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::balance_of {
						who: Address(Account::Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u64));
		});
}

#[test]
fn approve() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::approve {
						spender: Address(Account::Bob.into()),
						value: 500.into(),
					},
				)
				.expect_cost(46033756u64)
				.expect_log(log3(
					Account::ForeignAssetId(0u128),
					SELECTOR_LOG_APPROVAL,
					Account::Alice,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(500)).build(),
				))
				.execute_returns_encoded(true);
		});
}

#[test]
fn approve_saturating() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::approve {
						spender: Address(Account::Bob.into()),
						value: U256::MAX,
					},
				)
				.expect_cost(46033756u64)
				.expect_log(log3(
					Account::ForeignAssetId(0u128),
					SELECTOR_LOG_APPROVAL,
					Account::Alice,
					Account::Bob,
					EvmDataWriter::new().write(U256::MAX).build(),
				))
				.execute_returns_encoded(true);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::allowance {
						owner: Address(Account::Alice.into()),
						spender: Address(Account::Bob.into()),
					},
				)
				.expect_cost(0u64)
				.expect_no_logs()
				.execute_returns_encoded(U256::from(u128::MAX));
		});
}

#[test]
fn check_allowance_existing() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::approve {
						spender: Address(Account::Bob.into()),
						value: 500.into(),
					},
				)
				.execute_some();

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::allowance {
						owner: Address(Account::Alice.into()),
						spender: Address(Account::Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(500u64));
		});
}

#[test]
fn check_allowance_not_existing() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::allowance {
						owner: Address(Account::Alice.into()),
						spender: Address(Account::Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u64));
		});
}

#[test]
fn transfer() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::transfer {
						to: Address(Account::Bob.into()),
						value: 400.into(),
					},
				)
				.expect_cost(58180756u64) // 1 weight => 1 gas in mock
				.expect_log(log3(
					Account::ForeignAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					Account::Alice,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(400)).build(),
				))
				.execute_returns_encoded(true);

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::ForeignAssetId(0u128),
					ForeignPCall::balance_of {
						who: Address(Account::Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(400));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::balance_of {
						who: Address(Account::Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(600));
		});
}

#[test]
fn transfer_not_enough_founds() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::transfer {
						to: Address(Account::Charlie.into()),
						value: 50.into(),
					},
				)
				.execute_reverts(|output| {
					from_utf8(&output)
						.unwrap()
						.contains("Dispatched call failed with error: ")
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
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::approve {
						spender: Address(Account::Bob.into()),
						value: 500.into(),
					},
				)
				.execute_some();

			// TODO: Duplicate approve (noop)?
			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::approve {
						spender: Address(Account::Bob.into()),
						value: 500.into(),
					},
				)
				.execute_some();

			precompiles()
				.prepare_test(
					Account::Bob, // Bob is the one sending transferFrom!
					Account::ForeignAssetId(0u128),
					ForeignPCall::transfer_from {
						from: Address(Account::Alice.into()),
						to: Address(Account::Charlie.into()),
						value: 400.into(),
					},
				)
				.expect_cost(73187756u64) // 1 weight => 1 gas in mock
				.expect_log(log3(
					Account::ForeignAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					Account::Alice,
					Account::Charlie,
					EvmDataWriter::new().write(U256::from(400)).build(),
				))
				.execute_returns_encoded(true);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::balance_of {
						who: Address(Account::Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(600));

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::ForeignAssetId(0u128),
					ForeignPCall::balance_of {
						who: Address(Account::Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0));

			precompiles()
				.prepare_test(
					Account::Charlie,
					Account::ForeignAssetId(0u128),
					ForeignPCall::balance_of {
						who: Address(Account::Charlie.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(400));
		});
}

#[test]
fn transfer_from_non_incremental_approval() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			// We first approve 500
			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::approve {
						spender: Address(Account::Bob.into()),
						value: 500.into(),
					},
				)
				.expect_cost(46033756u64)
				.expect_log(log3(
					Account::ForeignAssetId(0u128),
					SELECTOR_LOG_APPROVAL,
					Account::Alice,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(500)).build(),
				))
				.execute_returns_encoded(true);

			// We then approve 300. Non-incremental, so this is
			// the approved new value
			// Additionally, the gas used in this approval is higher because we
			// need to clear the previous one
			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::approve {
						spender: Address(Account::Bob.into()),
						value: 300.into(),
					},
				)
				.expect_cost(93745756u64)
				.expect_log(log3(
					Account::ForeignAssetId(0u128),
					SELECTOR_LOG_APPROVAL,
					Account::Alice,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(300)).build(),
				))
				.execute_returns_encoded(true);

			// This should fail, as now the new approved quantity is 300
			precompiles()
				.prepare_test(
					Account::Bob, // Bob is the one sending transferFrom!
					Account::ForeignAssetId(0u128),
					ForeignPCall::transfer_from {
						from: Address(Account::Alice.into()),
						to: Address(Account::Bob.into()),
						value: 500.into(),
					},
				)
				.execute_reverts(|output| {
					output
						== b"Dispatched call failed with error: Module(ModuleError { index: 2, error: [10, 0, 0, 0], \
					message: Some(\"Unapproved\") })"
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
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::approve {
						spender: Address(Account::Bob.into()),
						value: 300.into(),
					},
				)
				.execute_some();

			precompiles()
				.prepare_test(
					Account::Bob, // Bob is the one sending transferFrom!
					Account::ForeignAssetId(0u128),
					ForeignPCall::transfer_from {
						from: Address(Account::Alice.into()),
						to: Address(Account::Bob.into()),
						value: 400.into(),
					},
				)
				.execute_reverts(|output| {
					output
						== b"Dispatched call failed with error: Module(ModuleError { index: 2, error: [10, 0, 0, 0], \
					message: Some(\"Unapproved\") })"
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
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice, // Alice sending transferFrom herself, no need for allowance.
					Account::ForeignAssetId(0u128),
					ForeignPCall::transfer_from {
						from: Address(Account::Alice.into()),
						to: Address(Account::Bob.into()),
						value: 400.into(),
					},
				)
				.expect_cost(58180756u64) // 1 weight => 1 gas in mock
				.expect_log(log3(
					Account::ForeignAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					Account::Alice,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(400)).build(),
				))
				.execute_returns_encoded(true);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::balance_of {
						who: Address(Account::Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(600));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::balance_of {
						who: Address(Account::Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(400));
		});
}

#[test]
fn get_metadata() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::force_set_metadata(
				RuntimeOrigin::root(),
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
					ForeignPCall::name {},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write::<UnboundedBytes>("TestToken".into())
						.build(),
				);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::symbol {},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write::<UnboundedBytes>("Test".into())
						.build(),
				);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::decimals {},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(12u8);
		});
}

#[test]
fn local_functions_cannot_be_accessed_by_foreign_assets() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::force_set_metadata(
				RuntimeOrigin::root(),
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
					ForeignPCall::mint {
						to: Address(Account::Bob.into()),
						value: 400.into(),
					},
				)
				.execute_reverts(|output| output == b"Unknown selector");

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::burn {
						from: Address(Account::Bob.into()),
						value: 400.into(),
					},
				)
				.execute_reverts(|output| output == b"Unknown selector");
		});
}

#[test]
fn mint_local_assets() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				RuntimeOrigin::root(),
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
					LocalPCall::mint {
						to: Address(Account::Bob.into()),
						value: 400.into(),
					},
				)
				.expect_cost(36218756u64) // 1 weight => 1 gas in mock
				.expect_log(log3(
					Account::LocalAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					Account::Zero,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(400)).build(),
				))
				.execute_returns_encoded(true);

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::LocalAssetId(0u128),
					LocalPCall::balance_of {
						who: Address(Account::Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(400));
		});
}

#[test]
fn burn_local_assets() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				RuntimeOrigin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			assert_ok!(LocalAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::burn {
						from: Address(Account::Alice.into()),
						value: 400.into(),
					},
				)
				.expect_cost(45808756u64) // 1 weight => 1 gas in mock
				.expect_log(log3(
					Account::LocalAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					Account::Alice,
					Account::Zero,
					EvmDataWriter::new().write(U256::from(400)).build(),
				))
				.execute_returns_encoded(true);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::balance_of {
						who: Address(Account::Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(600));
		});
}

#[test]
fn freeze_local_assets() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				RuntimeOrigin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			assert_ok!(LocalAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Bob.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::freeze {
						account: Address(Account::Bob.into()),
					},
				)
				.expect_cost(27689000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns_encoded(true);

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::LocalAssetId(0u128),
					LocalPCall::transfer {
						to: Address(Account::Alice.into()),
						value: 400.into(),
					},
				)
				.execute_reverts(|output| {
					from_utf8(&output)
						.unwrap()
						.contains("Dispatched call failed with error: ")
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
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				RuntimeOrigin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			assert_ok!(LocalAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Bob.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::freeze {
						account: Address(Account::Bob.into()),
					},
				)
				.expect_cost(27689000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns_encoded(true);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::thaw {
						account: Address(Account::Bob.into()),
					},
				)
				.expect_cost(27591000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns_encoded(true);

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::LocalAssetId(0u128),
					LocalPCall::transfer {
						to: Address(Account::Alice.into()),
						value: 400.into(),
					},
				)
				.expect_cost(58180756u64) // 1 weight => 1 gas in mock
				.expect_log(log3(
					Account::LocalAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					Account::Bob,
					Account::Alice,
					EvmDataWriter::new().write(U256::from(400)).build(),
				))
				.execute_returns_encoded(true);
		});
}

#[test]
fn freeze_asset_local_asset() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				RuntimeOrigin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			assert_ok!(LocalAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Bob.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::freeze_asset {},
				)
				.expect_cost(24269000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns_encoded(true);

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::LocalAssetId(0u128),
					LocalPCall::transfer {
						to: Address(Account::Alice.into()),
						value: 400.into(),
					},
				)
				.execute_reverts(|output| {
					from_utf8(&output)
						.unwrap()
						.contains("Dispatched call failed with error: ")
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
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				RuntimeOrigin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			assert_ok!(LocalAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Bob.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::freeze_asset {},
				)
				.expect_cost(24269000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns_encoded(true);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::thaw_asset {},
				)
				.expect_cost(23527000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns_encoded(true);

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::LocalAssetId(0u128),
					LocalPCall::transfer {
						to: Address(Account::Alice.into()),
						value: 400.into(),
					},
				)
				.expect_cost(58180756u64) // 1 weight => 1 gas in mock
				.expect_log(log3(
					Account::LocalAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					Account::Bob,
					Account::Alice,
					EvmDataWriter::new().write(U256::from(400)).build(),
				))
				.execute_returns_encoded(true);
		});
}

#[test]
fn transfer_ownership_local_assets() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				RuntimeOrigin::root(),
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
					LocalPCall::transfer_ownership {
						owner: Address(Account::Bob.into()),
					},
				)
				.expect_cost(24597000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns_encoded(true);

			// Now Bob should be able to change ownership, and not Alice
			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::transfer_ownership {
						owner: Address(Account::Bob.into()),
					},
				)
				.execute_reverts(|output| {
					from_utf8(&output)
						.unwrap()
						.contains("Dispatched call failed with error: ")
						&& from_utf8(&output).unwrap().contains("NoPermission")
				});

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::LocalAssetId(0u128),
					LocalPCall::transfer_ownership {
						owner: Address(Account::Alice.into()),
					},
				)
				.expect_cost(24597000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns_encoded(true);
		});
}

#[test]
fn set_team_local_assets() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				RuntimeOrigin::root(),
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
					LocalPCall::set_team {
						issuer: Address(Account::Bob.into()),
						admin: Address(Account::Bob.into()),
						freezer: Address(Account::Bob.into()),
					},
				)
				.expect_cost(23173000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns_encoded(true);

			// Now Bob should be able to mint, and not Alice
			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::mint {
						to: Address(Account::Bob.into()),
						value: 400.into(),
					},
				)
				.execute_reverts(|output| {
					from_utf8(&output)
						.unwrap()
						.contains("Dispatched call failed with error: ")
						&& from_utf8(&output).unwrap().contains("NoPermission")
				});

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::LocalAssetId(0u128),
					LocalPCall::mint {
						to: Address(Account::Bob.into()),
						value: 400.into(),
					},
				)
				.expect_cost(36218756u64) // 1 weight => 1 gas in mock
				.expect_log(log3(
					Account::LocalAssetId(0u128),
					SELECTOR_LOG_TRANSFER,
					Account::Zero,
					Account::Bob,
					EvmDataWriter::new().write(U256::from(400)).build(),
				))
				.execute_returns_encoded(true);

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::LocalAssetId(0u128),
					LocalPCall::balance_of {
						who: Address(Account::Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(400));
		});
}

#[test]
fn set_metadata() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				RuntimeOrigin::root(),
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
					LocalPCall::set_metadata {
						name: "TestToken".into(),
						symbol: "Test".into(),
						decimals: 12,
					},
				)
				.expect_cost(42869113u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns_encoded(true);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::name {},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write::<UnboundedBytes>("TestToken".into())
						.build(),
				);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::symbol {},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write::<UnboundedBytes>("Test".into())
						.build(),
				);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::decimals {},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(12u8);
		});
}

#[test]
fn clear_metadata() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				RuntimeOrigin::root(),
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
					LocalPCall::set_metadata {
						name: "TestToken".into(),
						symbol: "Test".into(),
						decimals: 12,
					},
				)
				.expect_cost(42869113u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns_encoded(true);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::clear_metadata {},
				)
				.expect_cost(42912000u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_returns_encoded(true);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::name {},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write::<UnboundedBytes>("".into())
						.build(),
				);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::symbol {},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write::<UnboundedBytes>("".into())
						.build(),
				);

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::decimals {},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(0u8);
		});
}

#[test]
fn permit_valid() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
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
					ForeignPCall::eip2612_nonces {
						owner: Address(Account::Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u8));

			precompiles()
				.prepare_test(
					Account::Charlie,
					Account::ForeignAssetId(0u128),
					ForeignPCall::eip2612_permit {
						owner: Address(owner),
						spender: Address(spender),
						value,
						deadline,
						v: v.serialize(),
						r: H256::from(rs.r.b32()),
						s: H256::from(rs.s.b32()),
					},
				)
				.expect_cost(46032000u64)
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
					ForeignPCall::allowance {
						owner: Address(Account::Alice.into()),
						spender: Address(Account::Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(500u16));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::eip2612_nonces {
						owner: Address(Account::Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(1u8));
		});
}

#[test]
fn permit_valid_named_asset() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));
			assert_ok!(ForeignAssets::set_metadata(
				RuntimeOrigin::signed(Account::Alice),
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
					ForeignPCall::eip2612_nonces {
						owner: Address(Account::Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u8));

			precompiles()
				.prepare_test(
					Account::Charlie,
					Account::ForeignAssetId(0u128),
					ForeignPCall::eip2612_permit {
						owner: Address(owner),
						spender: Address(spender),
						value,
						deadline,
						v: v.serialize(),
						r: H256::from(rs.r.b32()),
						s: H256::from(rs.s.b32()),
					},
				)
				.expect_cost(46032000u64)
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
					ForeignPCall::allowance {
						owner: Address(Account::Alice.into()),
						spender: Address(Account::Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(500u16));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::eip2612_nonces {
						owner: Address(Account::Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(1u8));
		});
}

#[test]
fn permit_invalid_nonce() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
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
					ForeignPCall::eip2612_nonces {
						owner: Address(Account::Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u8));

			precompiles()
				.prepare_test(
					Account::Charlie,
					Account::ForeignAssetId(0u128),
					ForeignPCall::eip2612_permit {
						owner: Address(owner),
						spender: Address(spender),
						value,
						deadline,
						v: v.serialize(),
						r: H256::from(rs.r.b32()),
						s: H256::from(rs.s.b32()),
					},
				)
				.execute_reverts(|output| output == b"Invalid permit");

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::allowance {
						owner: Address(Account::Alice.into()),
						spender: Address(Account::Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u16));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::eip2612_nonces {
						owner: Address(Account::Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u8));
		});
}

#[test]
fn permit_invalid_signature() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
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
					ForeignPCall::eip2612_nonces {
						owner: Address(Account::Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u8));

			precompiles()
				.prepare_test(
					Account::Charlie,
					Account::ForeignAssetId(0u128),
					ForeignPCall::eip2612_permit {
						owner: Address(owner),
						spender: Address(spender),
						value,
						deadline,
						v: 0,
						r: H256::random(),
						s: H256::random(),
					},
				)
				.execute_reverts(|output| output == b"Invalid permit");

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::allowance {
						owner: Address(Account::Alice.into()),
						spender: Address(Account::Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u16));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::eip2612_nonces {
						owner: Address(Account::Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u8));
		});
}

#[test]
fn permit_invalid_deadline() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
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
					ForeignPCall::eip2612_nonces {
						owner: Address(Account::Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u8));

			precompiles()
				.prepare_test(
					Account::Charlie,
					Account::ForeignAssetId(0u128),
					ForeignPCall::eip2612_permit {
						owner: Address(owner),
						spender: Address(spender),
						value,
						deadline,
						v: v.serialize(),
						r: H256::from(rs.r.b32()),
						s: H256::from(rs.s.b32()),
					},
				)
				.execute_reverts(|output| output == b"Permit expired");

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::allowance {
						owner: Address(Account::Alice.into()),
						spender: Address(Account::Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u16));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::eip2612_nonces {
						owner: Address(Account::Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0u8));
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
				RuntimeOrigin::root(),
				1u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
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
					ForeignPCall::eip2612_permit {
						owner: Address(owner),
						spender: Address(spender),
						value,
						deadline,
						v: v_real,
						r: H256::from(r_real),
						s: H256::from(s_real),
					},
				)
				.expect_cost(46032000u64)
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
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::transfer {
						to: Address(Account::Bob.into()),
						value: U256::from(u128::MAX) + 1,
					},
				)
				.expect_cost(1756u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_reverts(|e| e == b"value: Value is too large for balance type");

			precompiles()
				.prepare_test(
					Account::Bob,
					Account::ForeignAssetId(0u128),
					ForeignPCall::balance_of {
						who: Address(Account::Bob.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(0));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::balance_of {
						who: Address(Account::Alice.into()),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns_encoded(U256::from(1000));
		});
}

#[test]
fn transfer_from_overflow() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000)])
		.build()
		.execute_with(|| {
			assert_ok!(ForeignAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(ForeignAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::approve {
						spender: Address(Account::Bob.into()),
						value: 500.into(),
					},
				)
				.execute_some();

			// TODO: Duplicate approve of same value (noop?)
			precompiles()
				.prepare_test(
					Account::Alice,
					Account::ForeignAssetId(0u128),
					ForeignPCall::approve {
						spender: Address(Account::Bob.into()),
						value: 500.into(),
					},
				)
				.execute_some();

			precompiles()
				.prepare_test(
					Account::Bob, // Bob is the one sending transferFrom!
					Account::ForeignAssetId(0u128),
					ForeignPCall::transfer_from {
						from: Address(Account::Alice.into()),
						to: Address(Account::Charlie.into()),
						value: U256::from(u128::MAX) + 1,
					},
				)
				.expect_cost(1756u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_reverts(|e| e == b"value: Value is too large for balance type");
		});
}

#[test]
fn mint_overflow() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				RuntimeOrigin::root(),
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
					LocalPCall::mint {
						to: Address(Account::Bob.into()),
						value: U256::from(u128::MAX) + 1,
					},
				)
				.expect_cost(1756u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_reverts(|e| e == b"value: Value is too large for balance type");
		});
}

#[test]
fn burn_overflow() {
	ExtBuilder::default()
		.with_balances(vec![(Account::Alice, 1000), (Account::Bob, 2500)])
		.build()
		.execute_with(|| {
			assert_ok!(LocalAssets::force_create(
				RuntimeOrigin::root(),
				0u128,
				Account::Alice.into(),
				true,
				1
			));
			assert_ok!(LocalAssets::force_set_metadata(
				RuntimeOrigin::root(),
				0u128,
				b"TestToken".to_vec(),
				b"Test".to_vec(),
				12,
				false
			));
			assert_ok!(LocalAssets::mint(
				RuntimeOrigin::signed(Account::Alice),
				0u128,
				Account::Alice.into(),
				1000
			));

			precompiles()
				.prepare_test(
					Account::Alice,
					Account::LocalAssetId(0u128),
					LocalPCall::burn {
						from: Address(Account::Alice.into()),
						value: U256::from(u128::MAX) + 1,
					},
				)
				.expect_cost(1756u64) // 1 weight => 1 gas in mock
				.expect_no_logs()
				.execute_reverts(|e| e == b"value: Value is too large for balance type");
		});
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	for file in ["ERC20.sol", "LocalAsset.sol", "Permit.sol"] {
		for solidity_fn in solidity::get_selectors(file) {
			assert_eq!(
				solidity_fn.compute_selector_hex(),
				solidity_fn.docs_selector,
				"documented selector for '{}' did not match for file '{}'",
				solidity_fn.signature(),
				file,
			);

			let selector = solidity_fn.compute_selector();
			if !LocalPCall::supports_selector(selector) {
				panic!(
					"failed decoding selector 0x{:x} => '{}' as Action for file '{}'",
					selector,
					solidity_fn.signature(),
					file,
				)
			}
		}
	}
}

#[test]
fn test_deprecated_solidity_selectors_are_supported() {
	for deprecated_function in [
		"freeze_asset()",
		"thaw_asset()",
		"transfer_ownership(address)",
		"set_team(address,address,address)",
		"set_metadata(string,string,uint8)",
		"clear_metadata()",
	] {
		let selector = solidity::compute_selector(deprecated_function);
		if !LocalPCall::supports_selector(selector) {
			panic!(
				"failed decoding selector 0x{:x} => '{}' as Action",
				selector, deprecated_function,
			)
		}
	}
}
