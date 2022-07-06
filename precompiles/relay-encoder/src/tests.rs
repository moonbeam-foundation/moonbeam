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
use crate::mock::{
	Account::{Alice, Precompile},
	ExtBuilder, PrecompilesValue, Runtime, TestPrecompiles,
};
use crate::test_relay_runtime::TestEncoder;
use crate::AvailableStakeCalls;
use crate::StakeEncodeCall;
use crate::*;
use pallet_staking::RewardDestination;
use pallet_staking::ValidatorPrefs;
use precompile_utils::testing::*;
use sp_core::{H256, U256};
use sp_runtime::Perbill;

fn precompiles() -> TestPrecompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn test_selector_enum() {
	assert_eq!(Action::EncodeBond as u32, 0xa82948d4);
	assert_eq!(Action::EncodeBondExtra as u32, 0x813667a0);
	assert_eq!(Action::EncodeUnbond as u32, 0x51b14e57);
	assert_eq!(Action::EncodeWithdrawUnbonded as u32, 0xd5ad108e);
	assert_eq!(Action::EncodeValidate as u32, 0xbb64ca0c);
	assert_eq!(Action::EncodeNominate as u32, 0xd2ea7b08);
	assert_eq!(Action::EncodeChill as u32, 0xb5eaac43);
	assert_eq!(Action::EncodeSetPayee as u32, 0x414be337);
	assert_eq!(Action::EncodeSetController as u32, 0x07f7c6dc);
	assert_eq!(Action::EncodeRebond as u32, 0x0922ee17);
}

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile, vec![1u8, 2u8, 3u8])
			.execute_reverts(|output| output == b"tried to parse selector out of bounds");
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile, vec![1u8, 2u8, 3u8, 4u8])
			.execute_reverts(|output| output == b"unknown selector");
	});
}

#[test]
fn test_encode_bond() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::EncodeBond)
						.write(H256::from([1u8; 32]))
						.write(U256::from(100))
						.write(RewardDestinationWrapper(RewardDestination::Controller))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(Bytes::from(
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
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::EncodeBondExtra)
						.write(U256::from(100))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(Bytes::from(
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
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::EncodeChill).build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(Bytes::from(
							TestEncoder::encode_call(AvailableStakeCalls::Chill).as_slice(),
						))
						.build(),
				);
		});
}

#[test]
fn test_encode_nominate() {
	ExtBuilder::default()
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::EncodeNominate)
						.write(vec![H256::from([1u8; 32]), H256::from([2u8; 32])])
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(Bytes::from(
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
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::EncodeRebond)
						.write(U256::from(100))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(Bytes::from(
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
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::EncodeSetController)
						.write(H256::from([1u8; 32]))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(Bytes::from(
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
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::EncodeSetPayee)
						.write(RewardDestinationWrapper(RewardDestination::Controller))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(Bytes::from(
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
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::EncodeUnbond)
						.write(U256::from(100))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(Bytes::from(
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
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::EncodeValidate)
						.write(U256::from(100))
						.write(true)
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(Bytes::from(
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
		.with_balances(vec![(Alice, 1000)])
		.build()
		.execute_with(|| {
			precompiles()
				.prepare_test(
					Alice,
					Precompile,
					EvmDataWriter::new_with_selector(Action::EncodeWithdrawUnbonded)
						.write(U256::from(100))
						.build(),
				)
				.expect_cost(0) // TODO: Test db read/write costs
				.expect_no_logs()
				.execute_returns(
					EvmDataWriter::new()
						.write(Bytes::from(
							TestEncoder::encode_call(AvailableStakeCalls::WithdrawUnbonded(
								100u32.into(),
							))
							.as_slice(),
						))
						.build(),
				);
		});
}
