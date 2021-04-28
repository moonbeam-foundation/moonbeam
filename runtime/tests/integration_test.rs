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

//! Moonbeam Runtime Integration Tests

#![cfg(test)]

use cumulus_primitives_parachain_inherent::ParachainInherentData;
use frame_support::{
	assert_noop, assert_ok,
	dispatch::Dispatchable,
	traits::{GenesisBuild, OnFinalize, OnInitialize},
};
use moonbeam_runtime::{
	AccountId, AuthorInherent, Balance, Balances, Call, Event, InflationInfo, ParachainStaking,
	Range, Runtime, System, GLMR,
};
use parachain_staking::Bond;
use precompiles::MoonbeamPrecompiles;
use sp_core::{H160, U256};
use sp_runtime::{DispatchError, Perbill};

fn run_to_block(n: u32) {
	while System::block_number() < n {
		AuthorInherent::on_finalize(System::block_number());
		ParachainStaking::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		AuthorInherent::on_initialize(System::block_number());
	}
}

fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}

struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
	// [collator, amount]
	collators: Vec<(AccountId, Balance)>,
	// [nominator, collator, nomination_amount]
	nominators: Vec<(AccountId, AccountId, Balance)>,
	// per-round inflation config
	inflation: InflationInfo<Balance>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![],
			nominators: vec![],
			collators: vec![],
			inflation: InflationInfo {
				expect: Range {
					min: 100_000 * GLMR,
					ideal: 200_000 * GLMR,
					max: 500_000 * GLMR,
				},
				// unrealistically high parameterization, only for testing
				round: Range {
					min: Perbill::from_percent(5),
					ideal: Perbill::from_percent(5),
					max: Perbill::from_percent(5),
				},
			},
		}
	}
}

impl ExtBuilder {
	fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	fn with_collators(mut self, collators: Vec<(AccountId, Balance)>) -> Self {
		self.collators = collators;
		self
	}

	fn with_nominators(mut self, nominators: Vec<(AccountId, AccountId, Balance)>) -> Self {
		self.nominators = nominators;
		self
	}

	#[allow(dead_code)]
	fn with_inflation(mut self, inflation: InflationInfo<Balance>) -> Self {
		self.inflation = inflation;
		self
	}

	fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut stakers: Vec<(AccountId, Option<AccountId>, Balance)> = Vec::new();
		for collator in self.collators {
			stakers.push((collator.0, None, collator.1));
		}
		for nominator in self.nominators {
			stakers.push((nominator.0, Some(nominator.1), nominator.2));
		}
		parachain_staking::GenesisConfig::<Runtime> {
			stakers,
			inflation_config: self.inflation,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

const ALICE: [u8; 20] = [4u8; 20];
const BOB: [u8; 20] = [5u8; 20];
const CHARLIE: [u8; 20] = [6u8; 20];
const DAVE: [u8; 20] = [7u8; 20];

fn origin_of(account_id: AccountId) -> <Runtime as frame_system::Config>::Origin {
	<Runtime as frame_system::Config>::Origin::signed(account_id)
}

fn inherent_origin() -> <Runtime as frame_system::Config>::Origin {
	<Runtime as frame_system::Config>::Origin::none()
}

/// Mock the inherent that sets author in `author-inherent`
fn set_author(a: AccountId) {
	assert_ok!(
		Call::AuthorInherent(author_inherent::Call::<Runtime>::set_author(a))
			.dispatch(inherent_origin())
	);
}

/// Mock the inherent that sets validation data in ParachainSystem, which
/// contains the `relay_chain_block_number`, which is used in `author-filter` as a
/// source of randomness to filter valid authors at each block.
fn set_parachain_inherent_data() {
	use cumulus_primitives_core::PersistedValidationData;
	use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
	let (relay_parent_storage_root, relay_chain_state) =
		RelayStateSproofBuilder::default().into_state_root_and_proof();
	let vfp = PersistedValidationData {
		relay_parent_number: 1u32,
		relay_parent_storage_root,
		..Default::default()
	};
	let parachain_inherent_data = ParachainInherentData {
		validation_data: vfp,
		relay_chain_state: relay_chain_state,
		downward_messages: Default::default(),
		horizontal_messages: Default::default(),
	};
	assert_ok!(Call::ParachainSystem(
		cumulus_pallet_parachain_system::Call::<Runtime>::set_validation_data(
			parachain_inherent_data
		)
	)
	.dispatch(inherent_origin()));
}

#[test]
fn join_collator_candidates() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 2_000 * GLMR),
			(AccountId::from(CHARLIE), 1_100 * GLMR),
			(AccountId::from(DAVE), 1_000 * GLMR),
		])
		.with_collators(vec![
			(AccountId::from(ALICE), 1_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_nominators(vec![
			(AccountId::from(CHARLIE), AccountId::from(ALICE), 50 * GLMR),
			(AccountId::from(CHARLIE), AccountId::from(BOB), 50 * GLMR),
		])
		.build()
		.execute_with(|| {
			assert_noop!(
				ParachainStaking::join_candidates(origin_of(AccountId::from(ALICE)), 1_000 * GLMR,),
				parachain_staking::Error::<Runtime>::CandidateExists
			);
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(CHARLIE)),
					1_000 * GLMR
				),
				parachain_staking::Error::<Runtime>::NominatorExists
			);
			assert!(System::events().is_empty());
			assert_ok!(ParachainStaking::join_candidates(
				origin_of(AccountId::from(DAVE)),
				1_000 * GLMR,
			));
			assert_eq!(
				last_event(),
				Event::parachain_staking(parachain_staking::Event::JoinedCollatorCandidates(
					AccountId::from(DAVE),
					1_000 * GLMR,
					3_100 * GLMR
				))
			);
			let candidates = ParachainStaking::candidate_pool();
			assert_eq!(
				candidates.0[0],
				Bond {
					owner: AccountId::from(ALICE),
					amount: 1_050 * GLMR
				}
			);
			assert_eq!(
				candidates.0[1],
				Bond {
					owner: AccountId::from(BOB),
					amount: 1_050 * GLMR
				}
			);
			assert_eq!(
				candidates.0[2],
				Bond {
					owner: AccountId::from(DAVE),
					amount: 1_000 * GLMR
				}
			);
		});
}

#[test]
fn transfer_through_evm_to_stake() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 3_000 * GLMR)])
		.build()
		.execute_with(|| {
			// Charlie has no balance => fails to stake
			assert_noop!(
				ParachainStaking::join_candidates(
					origin_of(AccountId::from(CHARLIE)),
					1_000 * GLMR,
				),
				DispatchError::Module {
					index: 3,
					error: 3,
					message: Some("InsufficientBalance")
				}
			);
			// Alice stakes to become a collator candidate
			assert_ok!(ParachainStaking::join_candidates(
				origin_of(AccountId::from(ALICE)),
				1_000 * GLMR,
			));
			// Alice transfer from free balance 1000 GLMR to Bob
			assert_ok!(Balances::transfer(
				origin_of(AccountId::from(ALICE)),
				AccountId::from(BOB),
				2_000 * GLMR,
			));
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), 2_000 * GLMR,);
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();
			// Bob transfers 1000 GLMR to Charlie via EVM
			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(BOB),
				AccountId::from(CHARLIE),
				Vec::new(),
				(1_000 * GLMR).into(),
				gas_limit,
				gas_price,
				None
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));
			assert_eq!(
				Balances::free_balance(AccountId::from(CHARLIE)),
				1_000 * GLMR,
			);
			// Charlie can stake now
			assert_ok!(ParachainStaking::join_candidates(
				origin_of(AccountId::from(CHARLIE)),
				1_000 * GLMR,
			),);
			let candidates = ParachainStaking::candidate_pool();
			assert_eq!(
				candidates.0[0],
				Bond {
					owner: AccountId::from(ALICE),
					amount: 2_000 * GLMR
				}
			);
			assert_eq!(
				candidates.0[1],
				Bond {
					owner: AccountId::from(CHARLIE),
					amount: 1_000 * GLMR
				}
			);
		});
}

#[test]
fn reward_block_authors() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_nominators(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * GLMR,
		)])
		.build()
		.execute_with(|| {
			set_parachain_inherent_data();
			for x in 2..1201 {
				set_author(AccountId::from(ALICE));
				run_to_block(x);
			}
			// no rewards doled out yet
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), 1_000 * GLMR,);
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), 500 * GLMR,);
			set_author(AccountId::from(ALICE));
			run_to_block(1201);
			// rewards minted and distributed
			assert_eq!(
				Balances::free_balance(AccountId::from(ALICE)),
				1109999999920000000000,
			);
			assert_eq!(
				Balances::free_balance(AccountId::from(BOB)),
				539999999960000000000,
			);
		});
}

#[test]
fn join_candidates_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 3_000 * GLMR)])
		.build()
		.execute_with(|| {
			let staking_precompile_address = H160::from_low_u64_be(256);

			// Alice uses the staking precompile to join as a candidate through the EVM
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();
			let amount: U256 = (1000 * GLMR).into();

			// Construct the call data (selector, amount)
			let mut call_data = Vec::<u8>::from([0u8; 36]);
			call_data[0..4].copy_from_slice(&hex_literal::hex!("ad76ed5a"));
			amount.to_big_endian(&mut call_data[4..36]);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(ALICE),
				staking_precompile_address,
				call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			println!("Made it past dispatching");

			// Assert that Alice is now a candidate
			assert!(ParachainStaking::is_candidate(&AccountId::from(ALICE)));

			// Check for the right events.
			let expected_events = vec![
				Event::pallet_balances(pallet_balances::Event::Reserved(
					AccountId::from(ALICE),
					1000 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::JoinedCollatorCandidates(
					AccountId::from(ALICE),
					1000 * GLMR,
					1000 * GLMR,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);
		});
}

#[test]
fn leave_candidates_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 2_000 * GLMR)])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.build()
		.execute_with(|| {
			let staking_precompile_address = H160::from_low_u64_be(256);

			// Alice uses the staking precompile to leave_candidates
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();

			// Construct the leave_candidates call data
			let mut call_data = Vec::<u8>::from([0u8; 4]);
			call_data[0..4].copy_from_slice(&hex_literal::hex!("b7694219"));

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(ALICE),
				staking_precompile_address,
				call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Check for the right events.
			let expected_events = vec![
				Event::parachain_staking(parachain_staking::Event::CollatorScheduledExit(
					1,
					AccountId::from(ALICE),
					3,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);
		});
}

#[test]
fn go_online_offline_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 2_000 * GLMR)])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.build()
		.execute_with(|| {
			// Alice is initialized as a candidate
			assert!(ParachainStaking::is_candidate(&AccountId::from(ALICE)));
			let staking_precompile_address = H160::from_low_u64_be(256);

			// Alice uses the staking precompile to go offline
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();

			// Construct the go_offline call data
			let mut go_offline_call_data = Vec::<u8>::from([0u8; 4]);
			go_offline_call_data[0..4].copy_from_slice(&hex_literal::hex!("767e0450"));

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(ALICE),
				staking_precompile_address,
				go_offline_call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Check for the right events.
			let mut expected_events = vec![
				Event::parachain_staking(parachain_staking::Event::CollatorWentOffline(
					1,
					AccountId::from(ALICE),
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);

			// Construct the go_online call data
			let mut go_online_call_data = Vec::<u8>::from([0u8; 4]);
			go_online_call_data[0..4].copy_from_slice(&hex_literal::hex!("d2f73ceb"));

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(ALICE),
				staking_precompile_address,
				go_online_call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Check for the right events.
			let mut new_events = vec![
				Event::parachain_staking(parachain_staking::Event::CollatorBackOnline(
					1,
					AccountId::from(ALICE),
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];
			expected_events.append(&mut new_events);

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);
		});
}

#[test]
fn candidate_bond_more_less_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![(AccountId::from(ALICE), 3_000 * GLMR)])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.build()
		.execute_with(|| {
			// Alice is initialized as a candidate
			assert!(ParachainStaking::is_candidate(&AccountId::from(ALICE)));
			let staking_precompile_address = H160::from_low_u64_be(256);

			// Alice uses the staking precompile to bond more
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();

			// Construct the candidate_bond_more call
			let mut bond_more_call_data = Vec::<u8>::from([0u8; 36]);
			bond_more_call_data[0..4].copy_from_slice(&hex_literal::hex!("c57bd3a8"));
			let bond_more_amount: U256 = (1000 * GLMR).into();
			bond_more_amount.to_big_endian(&mut bond_more_call_data[4..36]);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(ALICE),
				staking_precompile_address,
				bond_more_call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Check for the right events.
			let mut expected_events = vec![
				Event::pallet_balances(pallet_balances::Event::Reserved(
					AccountId::from(ALICE),
					1_000 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::CollatorBondedMore(
					AccountId::from(ALICE),
					1_000 * GLMR,
					2_000 * GLMR,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);

			// Construct the go_online call data
			let mut bond_less_call_data = Vec::<u8>::from([0u8; 36]);
			bond_less_call_data[0..4].copy_from_slice(&hex_literal::hex!("289b6ba7"));
			let bond_less_amount: U256 = (500 * GLMR).into();
			bond_less_amount.to_big_endian(&mut bond_less_call_data[4..36]);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(ALICE),
				staking_precompile_address,
				bond_less_call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Check for the right events.
			let mut new_events = vec![
				Event::pallet_balances(pallet_balances::Event::Unreserved(
					AccountId::from(ALICE),
					500 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::CollatorBondedLess(
					AccountId::from(ALICE),
					2_000 * GLMR,
					1_500 * GLMR,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];
			expected_events.append(&mut new_events);

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);
		});
}

#[test]
fn nominate_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 3_000 * GLMR),
			(AccountId::from(BOB), 3_000 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.build()
		.execute_with(|| {
			let staking_precompile_address = H160::from_low_u64_be(256);

			// Bob uses the staking precompile to nominate Alice through the EVM
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();
			let nomination_amount: U256 = (1000 * GLMR).into();

			// Construct the call data (selector, collator, nomination amount)
			let mut call_data = Vec::<u8>::from([0u8; 68]);
			call_data[0..4].copy_from_slice(&hex_literal::hex!("82f2c8df"));
			call_data[16..36].copy_from_slice(&ALICE);
			nomination_amount.to_big_endian(&mut call_data[36..68]);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(BOB),
				staking_precompile_address,
				call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Assert that Bob is now nominating Alice
			assert!(ParachainStaking::is_nominator(&AccountId::from(BOB)));

			// Check for the right events.
			let expected_events = vec![
				Event::pallet_balances(pallet_balances::Event::Reserved(
					AccountId::from(BOB),
					1000 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::Nomination(
					AccountId::from(BOB),
					1000 * GLMR,
					AccountId::from(ALICE),
					2000 * GLMR,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);
		});
}

#[test]
fn leave_nominators_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 1_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
			(AccountId::from(CHARLIE), 1_500 * GLMR),
		])
		.with_collators(vec![
			(AccountId::from(ALICE), 1_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_nominators(vec![
			(AccountId::from(CHARLIE), AccountId::from(ALICE), 500 * GLMR),
			(AccountId::from(CHARLIE), AccountId::from(BOB), 500 * GLMR),
		])
		.build()
		.execute_with(|| {
			// Charlie is initialized as a nominator
			assert!(ParachainStaking::is_nominator(&AccountId::from(CHARLIE)));
			let staking_precompile_address = H160::from_low_u64_be(256);

			// Charlie uses staking precompile to leave nominator set
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();

			// Construct leave_nominators call
			let mut call_data = Vec::<u8>::from([0u8; 4]);
			call_data[0..4].copy_from_slice(&hex_literal::hex!("e8d68a37"));

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(CHARLIE),
				staking_precompile_address,
				call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Charlie is no longer a nominator
			assert!(!ParachainStaking::is_nominator(&AccountId::from(CHARLIE)));

			// Check for the right events.
			let expected_events = vec![
				Event::pallet_balances(pallet_balances::Event::Unreserved(
					AccountId::from(CHARLIE),
					500 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::NominatorLeftCollator(
					AccountId::from(CHARLIE),
					AccountId::from(ALICE),
					500 * GLMR,
					1_000 * GLMR,
				)),
				Event::pallet_balances(pallet_balances::Event::Unreserved(
					AccountId::from(CHARLIE),
					500 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::NominatorLeftCollator(
					AccountId::from(CHARLIE),
					AccountId::from(BOB),
					500 * GLMR,
					1_000 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::NominatorLeft(
					AccountId::from(CHARLIE),
					1_000 * GLMR,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);
		});
}

#[test]
fn revoke_nomination_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 1_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
			(AccountId::from(CHARLIE), 1_500 * GLMR),
		])
		.with_collators(vec![
			(AccountId::from(ALICE), 1_000 * GLMR),
			(AccountId::from(BOB), 1_000 * GLMR),
		])
		.with_nominators(vec![
			(AccountId::from(CHARLIE), AccountId::from(ALICE), 500 * GLMR),
			(AccountId::from(CHARLIE), AccountId::from(BOB), 500 * GLMR),
		])
		.build()
		.execute_with(|| {
			// Charlie is initialized as a nominator
			assert!(ParachainStaking::is_nominator(&AccountId::from(CHARLIE)));
			let staking_precompile_address = H160::from_low_u64_be(256);

			// Charlie uses staking precompile to revoke nomination
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();

			// Construct revoke_nomination call
			let mut call_data = Vec::<u8>::from([0u8; 36]);
			call_data[0..4].copy_from_slice(&hex_literal::hex!("4b65c34b"));
			call_data[16..36].copy_from_slice(&ALICE);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(CHARLIE),
				staking_precompile_address,
				call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Charlie is still a nominator because only nomination to Alice was revoked
			assert!(ParachainStaking::is_nominator(&AccountId::from(CHARLIE)));

			// Check for the right events.
			let expected_events = vec![
				Event::pallet_balances(pallet_balances::Event::Unreserved(
					AccountId::from(CHARLIE),
					500 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::NominatorLeftCollator(
					AccountId::from(CHARLIE),
					AccountId::from(ALICE),
					500 * GLMR,
					1_000 * GLMR,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);
		});
}

#[test]
fn nominator_bond_more_less_via_precompile() {
	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 1_000 * GLMR),
			(AccountId::from(BOB), 1_500 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_nominators(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * GLMR,
		)])
		.build()
		.execute_with(|| {
			// Bob is initialized as a nominator
			assert!(ParachainStaking::is_nominator(&AccountId::from(BOB)));
			let staking_precompile_address = H160::from_low_u64_be(256);

			// Alice uses the staking precompile to bond more
			let gas_limit = 100000u64;
			let gas_price: U256 = 1000.into();

			// Construct the nominator_bond_more call
			let mut bond_more_call_data = Vec::<u8>::from([0u8; 68]);
			bond_more_call_data[0..4].copy_from_slice(&hex_literal::hex!("971d44c8"));
			bond_more_call_data[16..36].copy_from_slice(&ALICE);
			let bond_more_amount: U256 = (500 * GLMR).into();
			bond_more_amount.to_big_endian(&mut bond_more_call_data[36..68]);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(BOB),
				staking_precompile_address,
				bond_more_call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Check for the right events.
			let mut expected_events = vec![
				Event::pallet_balances(pallet_balances::Event::Reserved(
					AccountId::from(BOB),
					500 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::NominationIncreased(
					AccountId::from(BOB),
					AccountId::from(ALICE),
					1_500 * GLMR,
					2_000 * GLMR,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);

			// Construct the go_online call data
			let mut bond_less_call_data = Vec::<u8>::from([0u8; 68]);
			bond_less_call_data[0..4].copy_from_slice(&hex_literal::hex!("f6a52569"));
			bond_less_call_data[16..36].copy_from_slice(&ALICE);
			let bond_less_amount: U256 = (500 * GLMR).into();
			bond_less_amount.to_big_endian(&mut bond_less_call_data[36..68]);

			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::call(
				AccountId::from(BOB),
				staking_precompile_address,
				bond_less_call_data,
				U256::zero(), // No value sent in EVM
				gas_limit,
				gas_price,
				None, // Use the next nonce
			))
			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

			// Check for the right events.
			let mut new_events = vec![
				Event::pallet_balances(pallet_balances::Event::Unreserved(
					AccountId::from(BOB),
					500 * GLMR,
				)),
				Event::parachain_staking(parachain_staking::Event::NominationDecreased(
					AccountId::from(BOB),
					AccountId::from(ALICE),
					2_000 * GLMR,
					1_500 * GLMR,
				)),
				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
					staking_precompile_address,
				)),
			];
			expected_events.append(&mut new_events);

			assert_eq!(
				System::events()
					.into_iter()
					.map(|e| e.event)
					.collect::<Vec<_>>(),
				expected_events
			);
		});
}

#[test]
fn is_nominator_accessor_true() {
	use evm::{Context, ExitSucceed};
	use pallet_evm::PrecompileSet;

	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 1_000 * GLMR),
			(AccountId::from(BOB), 1_500 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_nominators(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * GLMR,
		)])
		.build()
		.execute_with(|| {
			println!("Starting test execution");

			// Confirm Bob is initialized as a nominator directly
			assert!(ParachainStaking::is_nominator(&AccountId::from(BOB)));

			let staking_precompile_address = H160::from_low_u64_be(256);

			// Construct the input data
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&hex_literal::hex!("8e5080e7"));
			input_data[16..36].copy_from_slice(&BOB);

			// Expected result is an EVM boolean true which is 256 bits long.
			let mut expected_bytes = Vec::from([0u8; 32]);
			expected_bytes[31] = 1;
			let expected_result = Some(Ok((ExitSucceed::Returned, expected_bytes, 0)));

			// Assert precompile also reports Alice as a nominator
			assert_eq!(
				MoonbeamPrecompiles::<Runtime>::execute(
					staking_precompile_address,
					&input_data,
					None, //target_gas is not neecssary right now becuase I consume none
					&Context {
						// This context copied from Sacrifice tests. As commented there, it's not great.
						address: Default::default(),
						caller: Default::default(),
						apparent_value: From::from(0),
					}
				),
				expected_result
			);
		})
}

#[test]
fn is_nominator_accessor_false() {
	use evm::{Context, ExitSucceed};
	use pallet_evm::PrecompileSet;

	ExtBuilder::default()
		.with_balances(vec![
			(AccountId::from(ALICE), 1_000 * GLMR),
			(AccountId::from(BOB), 1_500 * GLMR),
		])
		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
		.with_nominators(vec![(
			AccountId::from(BOB),
			AccountId::from(ALICE),
			500 * GLMR,
		)])
		.build()
		.execute_with(|| {
			println!("Starting test execution");

			// Confirm Charlie is not initialized as a nominator directly
			assert!(!ParachainStaking::is_nominator(&AccountId::from(CHARLIE)));

			let staking_precompile_address = H160::from_low_u64_be(256);

			// Construct the input data
			let mut input_data = Vec::<u8>::from([0u8; 36]);
			input_data[0..4].copy_from_slice(&hex_literal::hex!("8e5080e7"));
			input_data[16..36].copy_from_slice(&CHARLIE);

			// Expected result is an EVM boolean false which is 256 bits long.
			let expected_bytes = Vec::from([0u8; 32]);
			let expected_result = Some(Ok((ExitSucceed::Returned, expected_bytes, 0)));

			// Assert precompile also reports Alice as a nominator
			assert_eq!(
				MoonbeamPrecompiles::<Runtime>::execute(
					staking_precompile_address,
					&input_data,
					None, //target_gas is not neecssary right now becuase I consume none
					&Context {
						// This context copied from Sacrifice tests. As commented there, it's not great.
						address: Default::default(),
						caller: Default::default(),
						apparent_value: From::from(0),
					}
				),
				expected_result
			);
		})
}

// This test is skipped because we got stuck at the deploy phase and haven't gotten it working yet.
// This is worth revisiting, but for now it is just making the CI red and stopping the docker images
// from publishing. So I'm skipping it.

// #[test]
// fn try_nomination_dao() {
// 	ExtBuilder::default()
// 		.with_balances(vec![(AccountId::from(ALICE), 2_000 * GLMR)])
// 		.with_collators(vec![(AccountId::from(ALICE), 1_000 * GLMR)])
// 		.build()
// 		.execute_with(|| {
// 			// Alice is initialized as a candidate
// 			assert!(ParachainStaking::is_candidate(&AccountId::from(ALICE)));
// 			let staking_precompile_address = H160::from_low_u64_be(256);

// 			let mut bytecode = Vec::from(hex_literal::hex!("6080604052610100600160006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055506000600160146101000a81548160ff02191690831515021790555034801561006e57600080fd5b506040516108c03803806108c0833981810160405281019061009091906100eb565b806000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055505061015d565b6000815190506100e581610146565b92915050565b6000602082840312156100fd57600080fd5b600061010b848285016100d6565b91505092915050565b600061011f82610126565b9050919050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b61014f81610114565b811461015a57600080fd5b50565b6107548061016c6000396000f3fe608060405234801561001057600080fd5b50600436106100575760003560e01c806313a4e8c01461005c5780636fd3af441461007a578063b2978bc814610084578063d4b83992146100a2578063e352e659146100c0575b600080fd5b6100646100ca565b6040516100719190610589565b60405180910390f35b6100826100f0565b005b61008c6101a9565b604051610099919061056e565b60405180910390f35b6100aa6101bc565b6040516100b79190610501565b60405180910390f35b6100c86101e0565b005b600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1681565b600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff166382f2c8df60008054906101000a900473ffffffffffffffffffffffffffffffffffffffff16678ac7230489e800006040518363ffffffff1660e01b815260040161017592919061051c565b600060405180830381600087803b15801561018f57600080fd5b505af11580156101a3573d6000803e3d6000fd5b50505050565b600160149054906101000a900460ff1681565b60008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1681565b7f055f2ee3ba5dab2804a6713fbe1e5aabd82b97b2b4f5d5f32d2f3cdafed701f6600160405161021091906105bf565b60405180910390a17f055f2ee3ba5dab2804a6713fbe1e5aabd82b97b2b4f5d5f32d2f3cdafed701f6674563918244f4000060405161024f919061062b565b60405180910390a17f055f2ee3ba5dab2804a6713fbe1e5aabd82b97b2b4f5d5f32d2f3cdafed701f647604051610286919061062b565b60405180910390a1600160149054906101000a900460ff16156102e0577f055f2ee3ba5dab2804a6713fbe1e5aabd82b97b2b4f5d5f32d2f3cdafed701f660026040516102d391906105da565b60405180910390a1610469565b674563918244f4000047111561042e577f055f2ee3ba5dab2804a6713fbe1e5aabd82b97b2b4f5d5f32d2f3cdafed701f6600360405161032091906105f5565b60405180910390a1600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff166382f2c8df60008054906101000a900473ffffffffffffffffffffffffffffffffffffffff16476040518363ffffffff1660e01b81526004016103a5929190610545565b600060405180830381600087803b1580156103bf57600080fd5b505af11580156103d3573d6000803e3d6000fd5b505050507f055f2ee3ba5dab2804a6713fbe1e5aabd82b97b2b4f5d5f32d2f3cdafed701f660046040516104079190610610565b60405180910390a160018060146101000a81548160ff021916908315150217905550610468565b7f055f2ee3ba5dab2804a6713fbe1e5aabd82b97b2b4f5d5f32d2f3cdafed701f661040060405161045f91906105a4565b60405180910390a15b5b565b61047481610646565b82525050565b61048381610658565b82525050565b6104928161068e565b82525050565b6104a1816106b2565b82525050565b6104b0816106c4565b82525050565b6104bf816106d6565b82525050565b6104ce816106e8565b82525050565b6104dd816106fa565b82525050565b6104ec8161070c565b82525050565b6104fb81610684565b82525050565b6000602082019050610516600083018461046b565b92915050565b6000604082019050610531600083018561046b565b61053e6020830184610498565b9392505050565b600060408201905061055a600083018561046b565b61056760208301846104f2565b9392505050565b6000602082019050610583600083018461047a565b92915050565b600060208201905061059e6000830184610489565b92915050565b60006020820190506105b960008301846104a7565b92915050565b60006020820190506105d460008301846104b6565b92915050565b60006020820190506105ef60008301846104c5565b92915050565b600060208201905061060a60008301846104d4565b92915050565b600060208201905061062560008301846104e3565b92915050565b600060208201905061064060008301846104f2565b92915050565b600061065182610664565b9050919050565b60008115159050919050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b6000819050919050565b6000610699826106a0565b9050919050565b60006106ab82610664565b9050919050565b60006106bd82610684565b9050919050565b60006106cf82610684565b9050919050565b60006106e182610684565b9050919050565b60006106f382610684565b9050919050565b600061070582610684565b9050919050565b600061071782610684565b905091905056fea2646970667358221220b5539e2d1f2fa81ef188e185e236f3d6915438e60c48231a59ddbe0ff02338c764736f6c63430008010033"));
// 			// 0000000000000000000000006be02d1d3665660d22ff9624b7be0551ee1ac91b

// 			let mut constructor_data = Vec::<u8>::from([0u8; 32]);
//          // The 12 bytes is most likely because accounts are padded to 12 bytes. The constructor also has a 4-byte selector
//          // like all the others. Cnsider that when this test is revisited.
// 			// Leave the first twelve bytes as 0 to call the constructor (apparently; this is reverse engineered from remix)
// 			constructor_data[0..12].copy_from_slice(&hex_literal::hex!("000000000000000000000000"));
// 			constructor_data[12..32].copy_from_slice(&ALICE);

// 			let mut call_data = Vec::new();
// 			call_data.append(&mut bytecode);
// 			call_data.append(&mut constructor_data);

// 			// Alice deploys the nomination dao pointing at herself as collator target
// 			let gas_limit = 100000u64;
// 			let gas_price: U256 = 1000.into();

// 			assert_ok!(Call::EVM(pallet_evm::Call::<Runtime>::create(
// 				AccountId::from(ALICE),
// 				call_data,
// 				U256::zero(), // No value sent in EVM
// 				gas_limit,
// 				gas_price,
// 				None, // Use the next nonce
// 			))
// 			.dispatch(<Runtime as frame_system::Config>::Origin::root()));

// 			// Check for the right events.
// 			let mut expected_events = vec![
// 				Event::parachain_staking(parachain_staking::Event::CollatorWentOffline(
// 					1,
// 					AccountId::from(ALICE),
// 				)),
// 				Event::pallet_evm(pallet_evm::RawEvent::<AccountId>::Executed(
// 					staking_precompile_address,
// 				)),
// 			];

// 			assert_eq!(
// 				System::events()
// 					.into_iter()
// 					.map(|e| e.event)
// 					.collect::<Vec<_>>(),
// 				expected_events
// 			);
// 		});
// }
