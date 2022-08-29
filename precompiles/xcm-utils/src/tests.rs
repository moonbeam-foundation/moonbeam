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
use crate::mock::*;
use crate::Action;

use codec::Encode;
use precompile_utils::{prelude::*, solidity, testing::*};
use sp_core::H160;
use xcm::latest::prelude::*;
use xcm::VersionedXcm;

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn test_selector_enum() {
	assert_eq!(Action::MultiLocationToAddress as u32, 0x343b3e00);
}

#[test]
fn test_get_account_parent() {
	ExtBuilder::default().build().execute_with(|| {
		let input = EvmDataWriter::new_with_selector(Action::MultiLocationToAddress)
			.write(MultiLocation::parent())
			.build();

		let expected_address: H160 = TestAccount::Parent.into();

		precompiles()
			.prepare_test(TestAccount::Alice, TestAccount::Precompile, input)
			.expect_cost(1)
			.expect_no_logs()
			.execute_returns(
				EvmDataWriter::new()
					.write(Address(expected_address))
					.build(),
			);
	});
}

#[test]
fn test_get_account_sibling() {
	ExtBuilder::default().build().execute_with(|| {
		let input = EvmDataWriter::new_with_selector(Action::MultiLocationToAddress)
			.write(MultiLocation {
				parents: 1,
				interior: Junctions::X1(Junction::Parachain(2000u32)),
			})
			.build();

		let expected_address: H160 = TestAccount::SiblingParachain(2000u32).into();

		precompiles()
			.prepare_test(TestAccount::Alice, TestAccount::Precompile, input)
			.expect_cost(1)
			.expect_no_logs()
			.execute_returns(
				EvmDataWriter::new()
					.write(Address(expected_address))
					.build(),
			);
	});
}

#[test]
fn test_executor_clear_origin() {
	ExtBuilder::default().build().execute_with(|| {
		let xcm_to_execute = Bytes(VersionedXcm::<()>::V2(Xcm(vec![ClearOrigin])).encode());

		let input = EvmDataWriter::new_with_selector(Action::XcmExecute)
			.write(xcm_to_execute)
			.write(10000u64)
			.build();

		precompiles()
			.prepare_test(TestAccount::Alice, TestAccount::Precompile, input)
			.expect_cost(100001000)
			.expect_no_logs()
			.execute_returns(EvmDataWriter::new().build());
	});
}

#[test]
fn test_executor_send() {
	ExtBuilder::default().build().execute_with(|| {
		let withdrawn_asset: MultiAsset = (MultiLocation::parent(), 1u128).into();
		let xcm_to_execute = Bytes(
			VersionedXcm::<()>::V2(Xcm(vec![
				WithdrawAsset(vec![withdrawn_asset].into()),
				InitiateReserveWithdraw {
					assets: MultiAssetFilter::Wild(All),
					reserve: MultiLocation::parent(),
					xcm: Xcm(vec![]),
				},
			]))
			.encode(),
		);

		let input = EvmDataWriter::new_with_selector(Action::XcmExecute)
			.write(xcm_to_execute)
			.write(10000u64)
			.build();

		precompiles()
			.prepare_test(TestAccount::Alice, TestAccount::Precompile, input)
			.expect_cost(100002000)
			.expect_no_logs()
			.execute_returns(EvmDataWriter::new().build());

		let sent_messages = sent_xcm();
		let (_, sent_message) = sent_messages.first().unwrap();
		// Lets make sure the message is as expected
		assert!(sent_message.0.contains(&ClearOrigin));
	});
}

use frame_support::traits::PalletInfo;
#[test]
fn test_executor_transact() {
	ExtBuilder::default()
		.with_balances(vec![(TestAccount::Alice, 1000000000)])
		.build()
		.execute_with(|| {
			let mut encoded: Vec<u8> = Vec::new();
			let index =
				<Runtime as frame_system::Config>::PalletInfo::index::<Balances>().unwrap() as u8;

			encoded.push(index);

			// Then call bytes
			let mut call_bytes = pallet_balances::Call::<Runtime>::transfer {
				dest: TestAccount::Bob,
				value: 100u32.into(),
			}
			.encode();
			encoded.append(&mut call_bytes);
			let xcm_to_execute = Bytes(
				VersionedXcm::<()>::V2(Xcm(vec![Transact {
					origin_type: OriginKind::SovereignAccount,
					require_weight_at_most: 1_000_000_000u64,
					call: encoded.into(),
				}]))
				.encode(),
			);

			let input = EvmDataWriter::new_with_selector(Action::XcmExecute)
				.write(xcm_to_execute)
				.write(2000000000u64)
				.build();

			precompiles()
				.prepare_test(TestAccount::Alice, TestAccount::Precompile, input)
				.expect_cost(1100001000)
				.expect_no_logs()
				.execute_returns(EvmDataWriter::new().build());

			// Transact executed
			assert_eq!(System::account(TestAccount::Bob).data.free, 100);
		});
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	for file in ["XcmUtils.sol"] {
		for solidity_fn in solidity::get_selectors(file) {
			assert_eq!(
				solidity_fn.compute_selector_hex(),
				solidity_fn.docs_selector,
				"documented selector for '{}' did not match for file '{}'",
				solidity_fn.signature(),
				file,
			);

			let selector = solidity_fn.compute_selector();
			if Action::try_from(selector).is_err() {
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
