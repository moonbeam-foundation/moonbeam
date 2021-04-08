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

use frame_support::{
	assert_noop, assert_ok,
	traits::{GenesisBuild, OnFinalize, OnInitialize},
};
use moonbeam_runtime::{AccountId, Balance, Event, InflationInfo, Range, Runtime, GLMR};
use sp_runtime::Perbill;

pub type SystemModule = frame_system::Pallet<Runtime>;
pub type StakingModule = parachain_staking::Pallet<Runtime>;

// fn run_to_block(n: u32) {
// 	while SystemModule::block_number() < n {
// 		StakingModule::on_finalize(SystemModule::block_number());
// 		SystemModule::set_block_number(SystemModule::block_number() + 1);
// 		StakingModule::on_initialize(SystemModule::block_number());
// 	}
// }

fn last_event() -> Event {
	SystemModule::events().pop().expect("Event expected").event
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, Balance)>,
	stakers: Vec<(AccountId, Option<AccountId>, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			endowed_accounts: vec![],
			stakers: vec![],
		}
	}
}

impl ExtBuilder {
	pub fn balances(mut self, endowed_accounts: Vec<(AccountId, Balance)>) -> Self {
		self.endowed_accounts = endowed_accounts;
		self
	}

	pub fn staking(mut self, stakers: Vec<(AccountId, Option<AccountId>, Balance)>) -> Self {
		self.stakers = stakers;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.endowed_accounts,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		parachain_staking::GenesisConfig::<Runtime> {
			stakers: self.stakers,
			inflation_config: InflationInfo {
				expect: Range {
					min: 100_000 * GLMR,
					ideal: 200_000 * GLMR,
					max: 500_000 * GLMR,
				},
				// 8766 rounds (hours) in a year
				round: Range {
					min: Perbill::from_parts(Perbill::from_percent(4).deconstruct() / 8766),
					ideal: Perbill::from_parts(Perbill::from_percent(5).deconstruct() / 8766),
					max: Perbill::from_parts(Perbill::from_percent(5).deconstruct() / 8766),
				},
			},
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| SystemModule::set_block_number(1));
		ext
	}
}

const ALICE: [u8; 20] = [4u8; 20];
const BOB: [u8; 20] = [5u8; 20];
const CARL: [u8; 20] = [6u8; 20];
const DAVE: [u8; 20] = [7u8; 20];

pub fn origin_of(account_id: AccountId) -> <Runtime as frame_system::Config>::Origin {
	<Runtime as frame_system::Config>::Origin::signed(account_id)
}

#[test]
fn join_collator_candidates() {
	ExtBuilder::default()
		.balances(vec![
			(AccountId::from(ALICE), 2_000 * GLMR),
			(AccountId::from(BOB), 2_000 * GLMR),
			(AccountId::from(CARL), 1_100 * GLMR),
			(AccountId::from(DAVE), 1_000 * GLMR),
		])
		.staking(vec![
			// collators
			(AccountId::from(ALICE), None, 1_000 * GLMR),
			(AccountId::from(BOB), None, 1_000 * GLMR),
			// nominators
			(
				AccountId::from(CARL),
				Some(AccountId::from(ALICE)),
				50 * GLMR,
			),
			(AccountId::from(CARL), Some(AccountId::from(BOB)), 50 * GLMR),
		])
		.build()
		.execute_with(|| {
			assert_noop!(
				StakingModule::join_candidates(origin_of(AccountId::from(ALICE)), 1_000 * GLMR,),
				parachain_staking::Error::<Runtime>::CandidateExists
			);
			assert_noop!(
				StakingModule::join_candidates(origin_of(AccountId::from(CARL)), 1_000 * GLMR),
				parachain_staking::Error::<Runtime>::NominatorExists
			);
			assert!(SystemModule::events().is_empty());
			assert_ok!(StakingModule::join_candidates(
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
