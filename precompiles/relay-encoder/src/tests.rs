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
use crate::mock::{ExtBuilder, PCall, Precompiles, PrecompilesValue, Runtime};
use crate::test_relay_runtime::TestEncoder;
use crate::AvailableStakeCalls;
use crate::StakeEncodeCall;
use crate::*;
use pallet_staking::RewardDestination;
use pallet_staking::ValidatorPrefs;
use precompile_utils::testing::*;
use sp_core::U256;
use sp_runtime::Perbill;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selectors() {
	assert!(PCall::encode_bond_selectors().contains(&0xa82948d4));
	assert!(PCall::encode_bond_extra_selectors().contains(&0x813667a0));
	assert!(PCall::encode_unbond_selectors().contains(&0x51b14e57));
	assert!(PCall::encode_withdraw_unbonded_selectors().contains(&0xd5ad108e));
	assert!(PCall::encode_validate_selectors().contains(&0xbb64ca0c));
	assert!(PCall::encode_nominate_selectors().contains(&0xd2ea7b08));
	assert!(PCall::encode_chill_selectors().contains(&0xb5eaac43));
	assert!(PCall::encode_set_payee_selectors().contains(&0x414be337));
	assert!(PCall::encode_set_controller_selectors().contains(&0x07f7c6dc));
	assert!(PCall::encode_rebond_selectors().contains(&0x0922ee17));
}

#[test]
fn modifiers() {
	ExtBuilder::default().build().execute_with(|| {
		let mut tester =
			PrecompilesModifierTester::new(PrecompilesValue::get(), Alice, Precompile1);

		tester.test_view_modifier(PCall::encode_bond_selectors());
		tester.test_view_modifier(PCall::encode_bond_extra_selectors());
		tester.test_view_modifier(PCall::encode_unbond_selectors());
		tester.test_view_modifier(PCall::encode_withdraw_unbonded_selectors());
		tester.test_view_modifier(PCall::encode_validate_selectors());
		tester.test_view_modifier(PCall::encode_nominate_selectors());
		tester.test_view_modifier(PCall::encode_chill_selectors());
		tester.test_view_modifier(PCall::encode_set_payee_selectors());
		tester.test_view_modifier(PCall::encode_set_controller_selectors());
		tester.test_view_modifier(PCall::encode_rebond_selectors());
	});
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile1, vec![1u8, 2u8, 3u8])
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile1, vec![1u8, 2u8, 3u8, 4u8])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn test_encode_bond() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::encode_bond {
						controller_address: [1u8; 32].into(),
						amount: 100.into(),
						reward_destination: RewardDestinationWrapper(RewardDestination::Controller),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(UnboundedBytes::from(
							TestEncoder::encode_call(AvailableStakeCalls::Bond(
								[1u8; 32].into(),
								100u32.into(),
								RewardDestination::Controller,
							))
							.as_slice(),
						))
						.build(),
				);
		});
}

#[test]
fn test_encode_bond_more() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::encode_bond_extra { amount: 100.into() },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(UnboundedBytes::from(
							TestEncoder::encode_call(AvailableStakeCalls::BondExtra(100u32.into()))
								.as_slice(),
						))
						.build(),
				);
		});
}

#[test]
fn test_encode_chill() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(Alice, Precompile1, PCall::encode_chill {})
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(UnboundedBytes::from(
							TestEncoder::encode_call(AvailableStakeCalls::Chill).as_slice(),
						))
						.build(),
				);
		});
}

#[test]
fn test_encode_nominate() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::encode_nominate {
						nominees: vec![U256::from([1u8; 32]), U256::from([2u8; 32])].into(),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(UnboundedBytes::from(
							TestEncoder::encode_call(AvailableStakeCalls::Nominate(vec![
								[1u8; 32].into(),
								[2u8; 32].into(),
							]))
							.as_slice(),
						))
						.build(),
				);
		});
}

#[test]
fn test_encode_rebond() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::encode_rebond { amount: 100.into() },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(UnboundedBytes::from(
							TestEncoder::encode_call(AvailableStakeCalls::Rebond(100u128))
								.as_slice(),
						))
						.build(),
				);
		});
}

#[test]
fn test_encode_set_controller() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::encode_set_controller {
						controller: [1u8; 32].into(),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(UnboundedBytes::from(
							TestEncoder::encode_call(AvailableStakeCalls::SetController(
								[1u8; 32].into(),
							))
							.as_slice(),
						))
						.build(),
				)
		});
}

#[test]
fn test_encode_set_payee() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::encode_set_payee {
						reward_destination: RewardDestinationWrapper(RewardDestination::Controller),
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(UnboundedBytes::from(
							TestEncoder::encode_call(AvailableStakeCalls::SetPayee(
								RewardDestination::Controller,
							))
							.as_slice(),
						))
						.build(),
				);
		});
}

#[test]
fn test_encode_unbond() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::encode_unbond { amount: 100.into() },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(UnboundedBytes::from(
							TestEncoder::encode_call(AvailableStakeCalls::Unbond(100u32.into()))
								.as_slice(),
						))
						.build(),
				);
		});
}

#[test]
fn test_encode_validate() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::encode_validate {
						comission: 100.into(),
						blocked: true,
					},
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(UnboundedBytes::from(
							TestEncoder::encode_call(AvailableStakeCalls::Validate(
								ValidatorPrefs {
									commission: Perbill::from_parts(100u32.into()),
									blocked: true,
								},
							))
							.as_slice(),
						))
						.build(),
				);
		});
}

#[test]
fn test_encode_withdraw_unbonded() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile1,
					PCall::encode_withdraw_unbonded { slashes: 100 },
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(UnboundedBytes::from(
							TestEncoder::encode_call(AvailableStakeCalls::WithdrawUnbonded(
								100u32.into(),
							))
							.as_slice(),
						))
						.build(),
				);
		});
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	for file in ["RelayEncoder.sol"] {
		for solidity_fn in solidity::get_selectors(file) {
			assert_eq!(
				solidity_fn.compute_selector_hex(),
				solidity_fn.docs_selector,
				"documented selector for '{}' did not match for file '{}'",
				solidity_fn.signature(),
				file,
			);

			let selector = solidity_fn.compute_selector();
			if !PCall::supports_selector(selector) {
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
		"encode_bond(uint256,uint256,bytes)",
		"encode_bond_extra(uint256)",
		"encode_unbond(uint256)",
		"encode_withdraw_unbonded(uint32)",
		"encode_validate(uint256,bool)",
		"encode_nominate(uint256[])",
		"encode_chill()",
		"encode_set_payee(bytes)",
		"encode_set_controller(uint256)",
		"encode_rebond(uint256)",
	] {
		let selector = solidity::compute_selector(deprecated_function);
		if !PCall::supports_selector(selector) {
			panic!(
				"failed decoding selector 0x{:x} => '{}' as Action",
				selector, deprecated_function,
			)
		}
	}
}
