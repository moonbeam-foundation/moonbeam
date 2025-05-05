// Copyright 2019-2025 PureStake Inc.
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
	sent_xcm, AccountId, Balances, ExtBuilder, PCall, ParentAccount, Precompiles, PrecompilesValue,
	Runtime, SiblingParachainAccount, System,
};
use frame_support::{traits::PalletInfo, weights::Weight};
use parity_scale_codec::Encode;
use precompile_utils::{prelude::*, testing::*};
use sp_core::{H160, U256};
use xcm::prelude::*;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn test_selector_enum() {
	assert!(PCall::multilocation_to_address_selectors().contains(&0x343b3e00));
	assert!(PCall::weight_message_selectors().contains(&0x25d54154));
	assert!(PCall::get_units_per_second_selectors().contains(&0x3f0f65db));
}

#[test]
fn modifiers() {
	ExtBuilder::default().build().execute_with(|| {
		let mut tester = PrecompilesModifierTester::new(precompiles(), Alice, Precompile1);

		tester.test_view_modifier(PCall::multilocation_to_address_selectors());
		tester.test_view_modifier(PCall::weight_message_selectors());
		tester.test_view_modifier(PCall::get_units_per_second_selectors());
	});
}

#[test]
fn test_get_account_parent() {
	ExtBuilder::default().build().execute_with(|| {
		let input = PCall::multilocation_to_address {
			location: Location::parent(),
		};

		let expected_address: H160 = ParentAccount.into();

		precompiles()
			.prepare_test(Alice, Precompile1, input)
			.expect_cost(2)
			.expect_no_logs()
			.execute_returns(Address(expected_address));
	});
}

#[test]
fn test_get_account_sibling() {
	ExtBuilder::default().build().execute_with(|| {
		let input = PCall::multilocation_to_address {
			location: Location {
				parents: 1,
				interior: [Junction::Parachain(2000u32)].into(),
			}
			.into(),
		};

		let expected_address: H160 = SiblingParachainAccount(2000u32).into();

		precompiles()
			.prepare_test(Alice, Precompile1, input)
			.expect_cost(2)
			.expect_no_logs()
			.execute_returns(Address(expected_address));
	});
}

#[test]
fn test_weight_message() {
	ExtBuilder::default().build().execute_with(|| {
		let message: Vec<u8> = xcm::VersionedXcm::<()>::V4(Xcm(vec![ClearOrigin])).encode();

		let input = PCall::weight_message {
			message: message.into(),
		};

		precompiles()
			.prepare_test(Alice, Precompile1, input)
			.expect_cost(1)
			.expect_no_logs()
			.execute_returns(1000u64);
	});
}

#[test]
fn test_get_units_per_second() {
	ExtBuilder::default().build().execute_with(|| {
		let input = PCall::get_units_per_second {
			location: Location::parent(),
		};

		precompiles()
			.prepare_test(Alice, Precompile1, input)
			.expect_cost(2)
			.expect_no_logs()
			.execute_returns(U256::from(1_000_000_000_000u128));
	});
}

#[test]
fn test_executor_clear_origin() {
	ExtBuilder::default().build().execute_with(|| {
		let xcm_to_execute = VersionedXcm::<()>::V4(Xcm(vec![ClearOrigin])).encode();

		let input = PCall::xcm_execute {
			message: xcm_to_execute.into(),
			weight: 10000u64,
		};

		precompiles()
			.prepare_test(Alice, Precompile1, input)
			.expect_cost(100001001)
			.expect_no_logs()
			.execute_returns(());
	})
}

#[test]
fn test_executor_send() {
	ExtBuilder::default().build().execute_with(|| {
		let withdrawn_asset: Asset = (Location::parent(), 1u128).into();
		let xcm_to_execute = VersionedXcm::<()>::V4(Xcm(vec![
			WithdrawAsset(vec![withdrawn_asset].into()),
			InitiateReserveWithdraw {
				assets: AssetFilter::Wild(All),
				reserve: Location::parent(),
				xcm: Xcm(vec![]),
			},
		]))
		.encode();

		let input = PCall::xcm_execute {
			message: xcm_to_execute.into(),
			weight: 10000u64,
		};

		precompiles()
			.prepare_test(Alice, Precompile1, input)
			.expect_cost(100002001)
			.expect_no_logs()
			.execute_returns(());

		let sent_messages = sent_xcm();
		let (_, sent_message) = sent_messages.first().unwrap();
		// Lets make sure the message is as expected
		assert!(sent_message.0.contains(&ClearOrigin));
	});
}

#[test]
fn test_executor_transact() {
	ExtBuilder::default()
		.with_balances(vec![(CryptoAlith.into(), 1000000000)])
		.build()
		.execute_with(|| {
			let mut encoded: Vec<u8> = Vec::new();
			let index =
				<Runtime as frame_system::Config>::PalletInfo::index::<Balances>().unwrap() as u8;

			encoded.push(index);

			// Then call bytes
			let mut call_bytes = pallet_balances::Call::<Runtime>::transfer_allow_death {
				dest: CryptoBaltathar.into(),
				value: 100u32.into(),
			}
			.encode();
			encoded.append(&mut call_bytes);
			let xcm_to_execute = VersionedXcm::<()>::V4(Xcm(vec![Transact {
				origin_kind: OriginKind::SovereignAccount,
				require_weight_at_most: Weight::from_parts(1_000_000_000u64, 5206u64),
				call: encoded.into(),
			}]))
			.encode();

			let input = PCall::xcm_execute {
				message: xcm_to_execute.into(),
				weight: 2_000_000_000u64,
			};

			precompiles()
				.prepare_test(CryptoAlith, Precompile1, input)
				.expect_cost(1100001001)
				.expect_no_logs()
				.execute_returns(());

			// Transact executed
			let baltathar_account: AccountId = CryptoBaltathar.into();
			assert_eq!(System::account(baltathar_account).data.free, 100);
		});
}

#[test]
fn test_send_clear_origin() {
	ExtBuilder::default().build().execute_with(|| {
		let xcm_to_send = VersionedXcm::<()>::V4(Xcm(vec![ClearOrigin])).encode();

		let input = PCall::xcm_send {
			dest: Location::parent(),
			message: xcm_to_send.into(),
		};

		precompiles()
			.prepare_test(CryptoAlith, Precompile1, input)
			// Only the cost of TestWeightInfo
			.expect_cost(100000001)
			.expect_no_logs()
			.execute_returns(());

		let sent_messages = sent_xcm();
		let (_, sent_message) = sent_messages.first().unwrap();
		// Lets make sure the message is as expected
		assert!(sent_message.0.contains(&ClearOrigin));
	})
}

#[test]
fn execute_fails_if_called_by_smart_contract() {
	ExtBuilder::default()
		.with_balances(vec![
			(CryptoAlith.into(), 1000),
			(CryptoBaltathar.into(), 1000),
		])
		.build()
		.execute_with(|| {
			// Set code to Alice address as it if was a smart contract.
			pallet_evm::AccountCodes::<Runtime>::insert(H160::from(Alice), vec![10u8]);
			pallet_evm::AccountCodesMetadata::<Runtime>::insert(
				H160::from(Alice),
				pallet_evm::CodeMetadata {
					size: 10,
					hash: sp_core::H256::default(),
				},
			);

			let xcm_to_execute = VersionedXcm::<()>::V4(Xcm(vec![ClearOrigin])).encode();

			let input = PCall::xcm_execute {
				message: xcm_to_execute.into(),
				weight: 10000u64,
			};

			PrecompilesValue::get()
				.prepare_test(Alice, Precompile1, input)
				.execute_reverts(|output| output == b"Function not callable by smart contracts");
		})
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	check_precompile_implements_solidity_interfaces(&["XcmUtils.sol"], PCall::supports_selector)
}
