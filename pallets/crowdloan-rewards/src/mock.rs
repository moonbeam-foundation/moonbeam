// Copyright 2019-2020 PureStake Inc.
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
use frame_support::traits::GenesisBuild;
use frame_support::traits::Get;
use frame_support::{
	impl_outer_event, impl_outer_origin, parameter_types,
	traits::{OnFinalize, OnInitialize},
	weights::Weight,
};
use sp_core::ed25519;
use sp_core::Pair;
use sp_core::H256;
use sp_io;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};
use sp_std::convert::From;
use sp_std::convert::TryInto;

pub type AccountId = u64;
pub type Balance = u128;
pub type BlockNumber = u64;
pub struct TestVestingPeriod(pub BlockNumber);

impl Get<u64> for TestVestingPeriod {
	fn get() -> u64 {
		return 8u64;
	}
}

impl_outer_origin! {
	pub enum Origin for Test where system = frame_system {}
}

mod crowdloan_rewards {
	pub use super::super::*;
}

impl_outer_event! {
	pub enum MetaEvent for Test {
		frame_system<T>,
		pallet_balances<T>,
		crowdloan_rewards<T>,
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
impl Config for Test {
	type Event = MetaEvent;
	type RewardCurrency = Balances;
	type RelayChainAccountId = [u8; 32];
	type VestingPeriod = TestVestingPeriod;
}
pub type Balances = pallet_balances::Module<Test>;
pub type Crowdloan = Module<Test>;
pub type Sys = frame_system::Module<Test>;

fn genesis(
	assigned: Vec<([u8; 32], AccountId, u32)>,
	unassigned: Vec<([u8; 32], u32)>,
) -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	GenesisConfig::<Test> {
		associated: assigned,
		unassociated: unassigned,
		reward_ratio: 1,
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	let mut ext = sp_io::TestExternalities::from(storage);
	ext.execute_with(|| Sys::set_block_number(1));
	ext
}

pub(crate) fn get_ed25519_pairs(num: u32) -> Vec<ed25519::Pair> {
	let seed: u128 = 12345678901234567890123456789012;
	let mut pairs = Vec::new();
	for i in 0..num {
		pairs.push(ed25519::Pair::from_seed(
			(seed.clone() + i as u128)
				.to_string()
				.as_bytes()
				.try_into()
				.unwrap(),
		))
	}
	pairs
}

pub(crate) fn two_assigned_three_unassigned() -> sp_io::TestExternalities {
	let pairs = get_ed25519_pairs(3);
	genesis(
		vec![
			// validators
			([1u8; 32].into(), 1, 500),
			([2u8; 32].into(), 2, 500),
		],
		vec![
			// validators
			(pairs[0].public().into(), 500),
			(pairs[1].public().into(), 500),
			(pairs[2].public().into(), 500),
		],
	)
}

pub(crate) fn events() -> Vec<Event<Test>> {
	Sys::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let MetaEvent::crowdloan_rewards(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}

pub(crate) fn roll_to(n: u64) {
	while Sys::block_number() < n {
		Crowdloan::on_finalize(Sys::block_number());
		Balances::on_finalize(Sys::block_number());
		Sys::on_finalize(Sys::block_number());
		Sys::set_block_number(Sys::block_number() + 1);
		Sys::on_initialize(Sys::block_number());
		Balances::on_initialize(Sys::block_number());
		Crowdloan::on_initialize(Sys::block_number());
	}
}
