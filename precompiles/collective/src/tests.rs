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
	assert_event_emitted, hash, log_closed, log_executed, log_proposed, log_voted,
	mock::{
		Account::{self, Alice, Bob, Charlie, Precompile},
		ExtBuilder, Precompiles, PrecompilesValue, Runtime,
	},
	Action,
};
use frame_support::dispatch::Encode;
use precompile_utils::{prelude::*, solidity, testing::*};
use sp_core::H256;
use sp_runtime::DispatchError;

fn precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

// fn evm_call(input: Vec<u8>) -> EvmCall<Runtime> {
// 	EvmCall::call {
// 		source: Alice.into(),
// 		target: Precompile.into(),
// 		input,
// 		value: U256::zero(), // No value sent in EVM
// 		gas_limit: u64::max_value(),
// 		max_fee_per_gas: 0.into(),
// 		max_priority_fee_per_gas: Some(U256::zero()),
// 		nonce: None, // Use the next nonce
// 		access_list: Vec::new(),
// 	}
// }

#[test]
fn test_solidity_interface_has_all_function_selectors_documented_and_implemented() {
	for file in ["Collective.sol"] {
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

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default().build().execute_with(|| {
		// This selector is only three bytes long when four are required.
		precompiles()
			.prepare_test(Alice, Precompile, vec![1u8, 2u8, 3u8])
			.execute_reverts(|output| output == b"Tried to read selector out of bounds");
	});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(Alice, Precompile, vec![1u8, 2u8, 3u8, 4u8])
			.execute_reverts(|output| output == b"Unknown selector");
	});
}

#[test]
fn selectors() {
	assert_eq!(Action::Execute as u32, 0x09c5eabe);
	assert_eq!(Action::Propose as u32, 0xc57f3260);
	assert_eq!(Action::Vote as u32, 0x73e37688);
	assert_eq!(Action::Close as u32, 0x638d9d47);
	assert_eq!(Action::ProposalHash as u32, 0xfc379417);
}

#[test]
fn non_member_cannot_propose() {
	ExtBuilder::default().build().execute_with(|| {
		let proposal = pallet_treasury::Call::<Runtime>::spend {
			amount: 1,
			beneficiary: Account::Alice,
		};
		let proposal: <Runtime as frame_system::Config>::Call = proposal.into();
		let proposal = proposal.encode();

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Propose)
					.write(1u32)
					.write(Bytes(proposal))
					.build(),
			)
			.expect_no_logs()
			.execute_reverts(|output| output.ends_with(b"NotMember\") })"));
	});
}

#[test]
fn non_member_cannot_vote() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Vote)
					.write(H256::zero())
					.write(1u32)
					.write(false)
					.build(),
			)
			.expect_no_logs()
			.execute_reverts(|output| output.ends_with(b"NotMember\") })"));
	});
}

#[test]
fn non_member_cannot_execute() {
	ExtBuilder::default().build().execute_with(|| {
		let proposal = pallet_treasury::Call::<Runtime>::spend {
			amount: 1,
			beneficiary: Account::Alice,
		};
		let proposal: <Runtime as frame_system::Config>::Call = proposal.into();
		let proposal = proposal.encode();

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Vote)
					.write(Bytes(proposal))
					.build(),
			)
			.expect_no_logs()
			.execute_reverts(|output| output.ends_with(b"NotMember\") })"));
	});
}

#[test]
fn cannot_vote_for_unknown_proposal() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				Bob,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Vote)
					.write(H256::zero())
					.write(1u32)
					.write(false)
					.build(),
			)
			.expect_no_logs()
			.execute_reverts(|output| output.ends_with(b"ProposalMissing\") })"));
	});
}

#[test]
fn cannot_close_unknown_proposal() {
	ExtBuilder::default().build().execute_with(|| {
		precompiles()
			.prepare_test(
				Bob,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Vote)
					.write(H256::zero())
					.write(1u32)
					.write(0u64)
					.write(0u32)
					.build(),
			)
			.expect_no_logs()
			.execute_reverts(|output| output.ends_with(b"ProposalMissing\") })"));
	});
}

#[test]
fn member_can_make_instant_proposal() {
	ExtBuilder::default().build().execute_with(|| {
		let proposal = pallet_treasury::Call::<Runtime>::spend {
			amount: 1,
			beneficiary: Account::Alice,
		};
		let proposal: <Runtime as frame_system::Config>::Call = proposal.into();
		let proposal = proposal.encode();
		let proposal_hash: H256 = hash::<Runtime>(&proposal);

		// Proposal is executed. The proposal call will itself fail but it
		// still counts as a success according to pallet_collective.
		precompiles()
			.prepare_test(
				Bob,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Propose)
					.write(1u32)
					.write(Bytes(proposal))
					.build(),
			)
			.expect_log(log_executed(Precompile, proposal_hash))
			.execute_returns(EvmDataWriter::new().write(0u32).build());

		assert_event_emitted!(pallet_collective::Event::Executed {
			proposal_hash,
			result: Err(DispatchError::BadOrigin)
		}
		.into());
	});
}

#[test]
fn member_can_make_delayed_proposal() {
	ExtBuilder::default().build().execute_with(|| {
		let proposal = pallet_treasury::Call::<Runtime>::spend {
			amount: 1,
			beneficiary: Account::Alice,
		};
		let proposal: <Runtime as frame_system::Config>::Call = proposal.into();
		let proposal = proposal.encode();
		let proposal_hash: H256 = hash::<Runtime>(&proposal);

		precompiles()
			.prepare_test(
				Bob,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Propose)
					.write(2u32)
					.write(Bytes(proposal))
					.build(),
			)
			.expect_log(log_proposed(Precompile, Bob, 0, proposal_hash, 2))
			.execute_returns(EvmDataWriter::new().write(0u32).build());

		assert_event_emitted!(pallet_collective::Event::Proposed {
			account: Bob,
			proposal_index: 0,
			proposal_hash,
			threshold: 2,
		}
		.into());
	});
}

#[test]
fn member_can_vote_on_proposal() {
	ExtBuilder::default().build().execute_with(|| {
		let proposal = pallet_treasury::Call::<Runtime>::spend {
			amount: 1,
			beneficiary: Account::Alice,
		};
		let proposal: <Runtime as frame_system::Config>::Call = proposal.into();
		let proposal = proposal.encode();
		let proposal_hash: H256 = hash::<Runtime>(&proposal);

		precompiles()
			.prepare_test(
				Bob,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Propose)
					.write(2u32)
					.write(Bytes(proposal))
					.build(),
			)
			.expect_log(log_proposed(Precompile, Bob, 0, proposal_hash, 2))
			.execute_returns(EvmDataWriter::new().write(0u32).build());

		precompiles()
			.prepare_test(
				Charlie,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Vote)
					.write(proposal_hash)
					.write(0u32)
					.write(true)
					.build(),
			)
			.expect_log(log_voted(Precompile, Charlie, proposal_hash, true))
			.execute_returns(vec![]);

		assert_event_emitted!(pallet_collective::Event::Voted {
			account: Charlie,
			proposal_hash,
			voted: true,
			yes: 1,
			no: 0,
		}
		.into());
	});
}

#[test]
fn cannot_close_if_not_enough_votes() {
	ExtBuilder::default().build().execute_with(|| {
		let proposal = pallet_treasury::Call::<Runtime>::spend {
			amount: 1,
			beneficiary: Account::Alice,
		};
		let proposal: <Runtime as frame_system::Config>::Call = proposal.into();
		let proposal = proposal.encode();
		let proposal_hash: H256 = hash::<Runtime>(&proposal);
		let proposal_len = proposal.len() as u64;

		precompiles()
			.prepare_test(
				Bob,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Propose)
					.write(2u32)
					.write(Bytes(proposal))
					.build(),
			)
			.expect_log(log_proposed(Precompile, Bob, 0, proposal_hash, 2))
			.execute_returns(EvmDataWriter::new().write(0u32).build());

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Close)
					.write(proposal_hash)
					.write(0u32)
					.write(10_000_000u64)
					.write(proposal_len)
					.build(),
			)
			.expect_no_logs()
			.execute_reverts(|output| output.ends_with(b"TooEarly\") })"));
	});
}

#[test]
fn can_close_execute_if_enough_votes() {
	ExtBuilder::default().build().execute_with(|| {
		let proposal = pallet_treasury::Call::<Runtime>::spend {
			amount: 1,
			beneficiary: Account::Alice,
		};
		let proposal: <Runtime as frame_system::Config>::Call = proposal.into();
		let proposal = proposal.encode();
		let proposal_hash: H256 = hash::<Runtime>(&proposal);
		let proposal_len = proposal.len() as u64;

		precompiles()
			.prepare_test(
				Bob,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Propose)
					.write(2u32)
					.write(Bytes(proposal))
					.build(),
			)
			.expect_log(log_proposed(Precompile, Bob, 0, proposal_hash, 2))
			.execute_returns(EvmDataWriter::new().write(0u32).build());

		precompiles()
			.prepare_test(
				Bob,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Vote)
					.write(proposal_hash)
					.write(0u32)
					.write(true)
					.build(),
			)
			.expect_log(log_voted(Precompile, Bob, proposal_hash, true))
			.execute_returns(vec![]);

		precompiles()
			.prepare_test(
				Charlie,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Vote)
					.write(proposal_hash)
					.write(0u32)
					.write(true)
					.build(),
			)
			.expect_log(log_voted(Precompile, Charlie, proposal_hash, true))
			.execute_returns(vec![]);

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Close)
					.write(proposal_hash)
					.write(0u32)
					.write(100_000_000u64)
					.write(proposal_len)
					.build(),
			)
			.expect_log(log_executed(Precompile, proposal_hash))
			.execute_returns(EvmDataWriter::new().write(true).build());

		assert_event_emitted!(pallet_collective::Event::Closed {
			proposal_hash,
			yes: 2,
			no: 0,
		}
		.into());

		assert_event_emitted!(pallet_collective::Event::Approved { proposal_hash }.into());

		assert_event_emitted!(pallet_collective::Event::Executed {
			proposal_hash,
			result: Ok(())
		}
		.into());

		assert_event_emitted!(pallet_treasury::Event::SpendApproved {
			proposal_index: 0,
			amount: 1,
			beneficiary: Alice,
		}
		.into());
	});
}

#[test]
fn can_close_refuse_if_enough_votes() {
	ExtBuilder::default().build().execute_with(|| {
		let proposal = pallet_treasury::Call::<Runtime>::spend {
			amount: 1,
			beneficiary: Account::Alice,
		};
		let proposal: <Runtime as frame_system::Config>::Call = proposal.into();
		let proposal = proposal.encode();
		let proposal_hash: H256 = hash::<Runtime>(&proposal);
		let proposal_len = proposal.len() as u64;

		precompiles()
			.prepare_test(
				Bob,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Propose)
					.write(2u32)
					.write(Bytes(proposal))
					.build(),
			)
			.expect_log(log_proposed(Precompile, Bob, 0, proposal_hash, 2))
			.execute_returns(EvmDataWriter::new().write(0u32).build());

		precompiles()
			.prepare_test(
				Bob,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Vote)
					.write(proposal_hash)
					.write(0u32)
					.write(false)
					.build(),
			)
			.expect_log(log_voted(Precompile, Bob, proposal_hash, false))
			.execute_returns(vec![]);

		precompiles()
			.prepare_test(
				Charlie,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Vote)
					.write(proposal_hash)
					.write(0u32)
					.write(false)
					.build(),
			)
			.expect_log(log_voted(Precompile, Charlie, proposal_hash, false))
			.execute_returns(vec![]);

		precompiles()
			.prepare_test(
				Alice,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Close)
					.write(proposal_hash)
					.write(0u32)
					.write(100_000_000u64)
					.write(proposal_len)
					.build(),
			)
			.expect_log(log_closed(Precompile, proposal_hash))
			.execute_returns(EvmDataWriter::new().write(false).build());

		assert_event_emitted!(pallet_collective::Event::Closed {
			proposal_hash,
			yes: 0,
			no: 2,
		}
		.into());

		assert_event_emitted!(pallet_collective::Event::Disapproved { proposal_hash }.into());
	});
}

#[test]
fn multiple_propose_increase_index() {
	ExtBuilder::default().build().execute_with(|| {
		let proposal = pallet_treasury::Call::<Runtime>::spend {
			amount: 1,
			beneficiary: Account::Alice,
		};
		let proposal: <Runtime as frame_system::Config>::Call = proposal.into();
		let proposal = proposal.encode();
		let proposal_hash: H256 = hash::<Runtime>(&proposal);

		precompiles()
			.prepare_test(
				Bob,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Propose)
					.write(2u32)
					.write(Bytes(proposal))
					.build(),
			)
			.expect_log(log_proposed(Precompile, Bob, 0, proposal_hash, 2))
			.execute_returns(EvmDataWriter::new().write(0u32).build());

		let proposal = pallet_treasury::Call::<Runtime>::spend {
			amount: 2,
			beneficiary: Account::Alice,
		};
		let proposal: <Runtime as frame_system::Config>::Call = proposal.into();
		let proposal = proposal.encode();
		let proposal_hash: H256 = hash::<Runtime>(&proposal);

		precompiles()
			.prepare_test(
				Bob,
				Precompile,
				EvmDataWriter::new_with_selector(Action::Propose)
					.write(2u32)
					.write(Bytes(proposal))
					.build(),
			)
			.expect_log(log_proposed(Precompile, Bob, 1, proposal_hash, 2))
			.execute_returns(EvmDataWriter::new().write(1u32).build());
	});
}
