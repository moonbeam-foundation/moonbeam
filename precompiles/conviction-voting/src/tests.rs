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
use crate::{
	mock::*, VoteDirection, SELECTOR_LOG_DELEGATED, SELECTOR_LOG_UNDELEGATED,
	SELECTOR_LOG_UNLOCKED, SELECTOR_LOG_VOTED, SELECTOR_LOG_VOTE_REMOVED,
	SELECTOR_LOG_VOTE_REMOVED_OTHER,
};
use precompile_utils::{prelude::*, testing::*, EvmDataWriter};

use frame_support::{
	assert_ok,
	dispatch::{Dispatchable, Pays, PostDispatchInfo},
};
use pallet_evm::{Call as EvmCall, Event as EvmEvent};
use sp_core::{H160, H256, U256};
use sp_runtime::{
	traits::PostDispatchInfoOf, DispatchError, DispatchErrorWithPostInfo, DispatchResultWithInfo,
};

const ONGOING_POLL_INDEX: u32 = 3;

fn evm_call(input: Vec<u8>) -> EvmCall<Runtime> {
	EvmCall::call {
		source: Alice.into(),
		target: Precompile1.into(),
		input,
		value: U256::zero(),
		gas_limit: u64::max_value(),
		max_fee_per_gas: 0.into(),
		max_priority_fee_per_gas: Some(U256::zero()),
		nonce: None,
		access_list: Vec::new(),
	}
}

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	for file in ["ConvictionVoting.sol"] {
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

fn vote(
	direction: VoteDirection,
	vote_amount: U256,
	conviction: u8,
) -> DispatchResultWithInfo<PostDispatchInfoOf<RuntimeCall>> {
	let input = match direction {
		// Vote Yes
		VoteDirection::Yes => PCall::vote_yes {
			poll_index: ONGOING_POLL_INDEX,
			vote_amount,
			conviction,
		}
		.into(),
		// Vote No
		VoteDirection::No => PCall::vote_no {
			poll_index: ONGOING_POLL_INDEX,
			vote_amount,
			conviction,
		}
		.into(),
		// Unsupported
		_ => {
			return Err(DispatchErrorWithPostInfo {
				post_info: PostDispatchInfo {
					actual_weight: None,
					pays_fee: Pays::No,
				},
				error: DispatchError::Other("Vote direction not supported"),
			})
		}
	};
	RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root())
}

#[test]
fn vote_logs_work() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			// Vote Yes
			assert_ok!(vote(VoteDirection::Yes, 100_000.into(), 0.into()));

			// Vote No
			assert_ok!(vote(VoteDirection::No, 99_000.into(), 1.into()));

			// Assert vote events are emitted.
			assert!(vec![
				EvmEvent::Log {
					log: log2(
						Precompile1,
						SELECTOR_LOG_VOTED,
						H256::from_low_u64_be(ONGOING_POLL_INDEX as u64),
						EvmDataWriter::new()
							.write::<Address>(H160::from(Alice).into()) // caller
							.write::<bool>(true) // vote
							.write::<U256>(100_000.into()) // amount
							.write::<u8>(0) // conviction
							.build(),
					),
				}
				.into(),
				EvmEvent::Log {
					log: log2(
						Precompile1,
						SELECTOR_LOG_VOTED,
						H256::from_low_u64_be(ONGOING_POLL_INDEX as u64),
						EvmDataWriter::new()
							.write::<Address>(H160::from(Alice).into()) // caller
							.write::<bool>(false) // vote
							.write::<U256>(99_000.into()) // amount
							.write::<u8>(1) // conviction
							.build(),
					),
				}
				.into()
			]
			.iter()
			.all(|log| events().contains(log)));
		})
}

#[test]
fn remove_vote_logs_work() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			// Vote..
			assert_ok!(vote(VoteDirection::Yes, 100_000.into(), 0.into()));

			// ..and remove
			let input = PCall::remove_vote {
				poll_index: ONGOING_POLL_INDEX,
			}
			.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Assert remove vote event is emitted.
			assert!(events().contains(
				&EvmEvent::Log {
					log: log2(
						Precompile1,
						SELECTOR_LOG_VOTE_REMOVED,
						H256::from_low_u64_be(ONGOING_POLL_INDEX as u64),
						EvmDataWriter::new()
							.write::<Address>(H160::from(Alice).into()) // caller
							.build(),
					),
				}
				.into()
			));
		})
}

#[test]
fn remove_other_vote_logs_work() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			// Vote..
			assert_ok!(vote(VoteDirection::Yes, 100_000.into(), 0.into()));

			// ..and remove other
			let input = PCall::remove_other_vote {
				target: H160::from(Alice).into(),
				track_id: 0u16,
				poll_index: ONGOING_POLL_INDEX,
			}
			.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Assert remove other vote event is emitted.
			assert!(events().contains(
				&EvmEvent::Log {
					log: log2(
						Precompile1,
						SELECTOR_LOG_VOTE_REMOVED_OTHER,
						H256::from_low_u64_be(ONGOING_POLL_INDEX as u64),
						EvmDataWriter::new()
							.write::<Address>(H160::from(Alice).into()) // caller
							.write::<Address>(H160::from(Alice).into()) // target
							.write::<u16>(0u16) // track id
							.build(),
					),
				}
				.into()
			));
		})
}

#[test]
fn delegate_undelegate_logs_work() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			// Delegate
			let input = PCall::delegate {
				track_id: 0u16,
				representative: H160::from(Bob).into(),
				conviction: 0.into(),
				amount: 100_000.into(),
			}
			.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Assert delegate event is emitted.
			assert!(events().contains(
				&EvmEvent::Log {
					log: log2(
						Precompile1,
						SELECTOR_LOG_DELEGATED,
						H256::from_low_u64_be(0 as u64), // track id
						EvmDataWriter::new()
							.write::<Address>(H160::from(Alice).into()) // from
							.write::<Address>(H160::from(Bob).into()) // to
							.write::<U256>(100_000.into()) // amount
							.write::<u8>(0u8) // conviction
							.build(),
					),
				}
				.into()
			));

			// Undelegate
			let input = PCall::undelegate { track_id: 0u16 }.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Assert undelegate event is emitted.
			assert!(events().contains(
				&EvmEvent::Log {
					log: log2(
						Precompile1,
						SELECTOR_LOG_UNDELEGATED,
						H256::from_low_u64_be(0 as u64), // track id
						EvmDataWriter::new()
							.write::<Address>(H160::from(Alice).into()) // caller
							.build(),
					),
				}
				.into()
			));
		})
}

#[test]
fn unlock_logs_work() {
	ExtBuilder::default()
		.with_balances(vec![(Alice.into(), 100_000)])
		.build()
		.execute_with(|| {
			// Vote
			assert_ok!(vote(VoteDirection::Yes, 100_000.into(), 0.into()));

			// Remove
			let input = PCall::remove_vote {
				poll_index: ONGOING_POLL_INDEX,
			}
			.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Unlock
			let input = PCall::unlock {
				track_id: 0u16,
				target: H160::from(Alice).into(),
			}
			.into();
			assert_ok!(RuntimeCall::Evm(evm_call(input)).dispatch(RuntimeOrigin::root()));

			// Assert unlock event is emitted.
			assert!(events().contains(
				&EvmEvent::Log {
					log: log2(
						Precompile1,
						SELECTOR_LOG_UNLOCKED,
						H256::from_low_u64_be(0 as u64), // track id
						EvmDataWriter::new()
							.write::<Address>(H160::from(Alice).into()) // caller
							.build(),
					),
				}
				.into()
			));
		})
}
