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
	// balances
	balances: Vec<(AccountId, Balance)>,
	// [collator, amount]
	collators: Vec<(AccountId, Balance)>,
	// [nominator, collator, nomination_amount]
	nominators: Vec<(AccountId, AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![],
			nominators: vec![],
			collators: vec![],
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
			inflation_config: InflationInfo {
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
			use sp_core::U256;
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
			.dispatch(origin_of(AccountId::from(BOB))));
			assert_eq!(
				Balances::free_balance(AccountId::from(CHARLIE)),
				1_000 * GLMR,
			);
			// Charlie can stake now
			assert_ok!(ParachainStaking::join_candidates(
				origin_of(AccountId::from(CHARLIE)),
				1_000 * GLMR,
			),);
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
			// set parachain inherent data
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
			fn set_alice_as_author() {
				assert_ok!(
					Call::AuthorInherent(author_inherent::Call::<Runtime>::set_author(
						AccountId::from(ALICE)
					))
					.dispatch(inherent_origin())
				);
			}
			for x in 2..1201 {
				set_alice_as_author();
				run_to_block(x);
			}
			// no rewards doled out yet
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), 1_000 * GLMR,);
			assert_eq!(Balances::free_balance(AccountId::from(BOB)), 500 * GLMR,);
			set_alice_as_author();
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
