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

//! Test utilities
use crate::*;
use frame_support::{
	impl_outer_event, impl_outer_origin, parameter_types,
	traits::{GenesisBuild, OnFinalize, OnInitialize},
	weights::Weight,
};
use sp_core::H256;
use sp_io;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

pub type AccountId = u64;
pub type Balance = u128;
pub type BlockNumber = u64;

impl_outer_origin! {
	pub enum Origin for Test where system = frame_system {}
}

mod stake {
	pub use super::super::*;
}

impl_outer_event! {
	pub enum MetaEvent for Test {
		frame_system<T>,
		pallet_balances<T>,
		stake<T>,
	}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Test;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = MetaEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = ();
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
}
parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}
impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = Balance;
	type Event = MetaEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Module<Test>;
	type WeightInfo = ();
}
parameter_types! {
	pub const BlocksPerRound: u32 = 5;
	pub const BondDuration: u32 = 2;
	pub const TotalSelectedCandidates: u32 = 5;
	pub const MaxNominatorsPerCollator: u32 = 4;
	pub const MaxCollatorsPerNominator: u32 = 4;
	pub const MaxFee: Perbill = Perbill::from_percent(50);
	pub const MinCollatorStk: u128 = 10;
	pub const MinNominatorStk: u128 = 5;
	pub const MinNomination: u128 = 3;
}
impl crate::pallet::Config for Test {
	type Event = MetaEvent;
	type Currency = Balances;
	type BlocksPerRound = BlocksPerRound;
	type BondDuration = BondDuration;
	type TotalSelectedCandidates = TotalSelectedCandidates;
	type MaxNominatorsPerCollator = MaxNominatorsPerCollator;
	type MaxCollatorsPerNominator = MaxCollatorsPerNominator;
	type MaxFee = MaxFee;
	type MinCollatorStk = MinCollatorStk;
	type MinCollatorCandidateStk = MinCollatorStk;
	type MinNominatorStk = MinNominatorStk;
	type MinNomination = MinNomination;
}
pub type Balances = pallet_balances::Module<Test>;
pub type Stake = Module<Test>;
pub type Sys = frame_system::Module<Test>;

fn genesis(
	balances: Vec<(AccountId, Balance)>,
	stakers: Vec<(AccountId, Option<AccountId>, Balance)>,
) -> sp_io::TestExternalities {
	let expect: Range<Balance> = Range {
		min: 700,
		ideal: 700,
		max: 700,
	};
	// very unrealistic test parameterization, would be dumb to have per-round inflation this high
	let round: Range<Perbill> = Range {
		min: Perbill::from_percent(5),
		ideal: Perbill::from_percent(5),
		max: Perbill::from_percent(5),
	};
	let inflation_config: InflationInfo<Balance> = InflationInfo { expect, round };
	let mut storage = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	let genesis = pallet_balances::GenesisConfig::<Test> { balances };
	genesis.assimilate_storage(&mut storage).unwrap();
	GenesisConfig::<Test> {
		stakers,
		inflation_config,
	}
	.assimilate_storage(&mut storage)
	.unwrap();
	let mut ext = sp_io::TestExternalities::from(storage);
	ext.execute_with(|| Sys::set_block_number(1));
	ext
}

pub(crate) fn two_collators_four_nominators() -> sp_io::TestExternalities {
	genesis(
		vec![
			(1, 1000),
			(2, 300),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 9),
			(9, 4),
		],
		vec![
			// collators
			(1, None, 500),
			(2, None, 200),
			// nominators
			(3, Some(1), 100),
			(4, Some(1), 100),
			(5, Some(2), 100),
			(6, Some(2), 100),
		],
	)
}

pub(crate) fn five_collators_no_nominators() -> sp_io::TestExternalities {
	genesis(
		vec![
			(1, 1000),
			(2, 1000),
			(3, 1000),
			(4, 1000),
			(5, 1000),
			(6, 1000),
			(7, 33),
			(8, 33),
			(9, 33),
		],
		vec![
			// collators
			(1, None, 100),
			(2, None, 90),
			(3, None, 80),
			(4, None, 70),
			(5, None, 60),
			(6, None, 50),
		],
	)
}

pub(crate) fn five_collators_five_nominators() -> sp_io::TestExternalities {
	genesis(
		vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 100),
			(9, 100),
			(10, 100),
		],
		vec![
			// collators
			(1, None, 20),
			(2, None, 20),
			(3, None, 20),
			(4, None, 20),
			(5, None, 10),
			// nominators
			(6, Some(1), 10),
			(7, Some(1), 10),
			(8, Some(2), 10),
			(9, Some(2), 10),
			(10, Some(1), 10),
		],
	)
}

pub(crate) fn one_collator_two_nominators() -> sp_io::TestExternalities {
	genesis(
		vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100), (6, 100)],
		vec![
			// collators
			(1, None, 20),
			// nominators
			(2, Some(1), 10),
			(3, Some(1), 10),
		],
	)
}

pub(crate) fn roll_to(n: u64) {
	while Sys::block_number() < n {
		Stake::on_finalize(Sys::block_number());
		Balances::on_finalize(Sys::block_number());
		Sys::on_finalize(Sys::block_number());
		Sys::set_block_number(Sys::block_number() + 1);
		Sys::on_initialize(Sys::block_number());
		Balances::on_initialize(Sys::block_number());
		Stake::on_initialize(Sys::block_number());
	}
}

pub(crate) fn last_event() -> MetaEvent {
	Sys::events().pop().expect("Event expected").event
}

pub(crate) fn events() -> Vec<Event<Test>> {
	Sys::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let MetaEvent::stake(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}

// Same storage changes as EventHandler::note_author impl
pub(crate) fn set_author(round: u32, acc: u64, pts: u32) {
	<Points<Test>>::mutate(round, |p| *p += pts);
	<AwardedPts<Test>>::mutate(round, acc, |p| *p += pts);
}
