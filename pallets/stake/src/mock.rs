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
use frame_support::{
	impl_outer_event, impl_outer_origin, parameter_types, traits::FindAuthor, weights::Weight,
};
use sp_core::H256;
use sp_io;
use sp_runtime::{
	testing::{Header, UintAuthorityId},
	traits::{BlakeTwo256, IdentityLookup},
	ModuleId, Perbill,
};
use std::{cell::RefCell, collections::HashSet};

pub type AccountId = u64;
pub type Balance = u128;
pub type BlockNumber = u64;

impl_outer_origin! {
	pub enum Origin for Test where system = frame_system {}
}

thread_local! {
	static SESSION: RefCell<(Vec<AccountId>, HashSet<AccountId>)> = RefCell::new(Default::default());
}

/// Another session handler struct to test on_disabled.
pub struct OtherSessionHandler;
impl pallet_session::OneSessionHandler<AccountId> for OtherSessionHandler {
	type Key = UintAuthorityId;

	fn on_genesis_session<'a, I: 'a>(_: I)
	where
		I: Iterator<Item = (&'a AccountId, Self::Key)>,
		AccountId: 'a,
	{
	}

	fn on_new_session<'a, I: 'a>(_: bool, validators: I, _: I)
	where
		I: Iterator<Item = (&'a AccountId, Self::Key)>,
		AccountId: 'a,
	{
		SESSION.with(|x| {
			*x.borrow_mut() = (validators.map(|x| x.0.clone()).collect(), HashSet::new())
		});
	}

	fn on_disabled(validator_index: usize) {
		SESSION.with(|d| {
			let mut d = d.borrow_mut();
			let value = d.0[validator_index];
			d.1.insert(value);
		})
	}
}

impl sp_runtime::BoundToRuntimeAppPublic for OtherSessionHandler {
	type Public = UintAuthorityId;
}

/// Author of block is always 11
pub struct Author11;
impl FindAuthor<AccountId> for Author11 {
	fn find_author<'a, I>(_digests: I) -> Option<AccountId>
	where
		I: 'a + IntoIterator<Item = (frame_support::ConsensusEngineId, &'a [u8])>,
	{
		Some(11)
	}
}

mod stake {
	pub use super::super::*;
}

impl_outer_event! {
	pub enum Event for Test {
		frame_system<T>,
		pallet_balances<T>,
		pallet_session,
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
}
impl System for Test {
	type BaseCallFilter = ();
	type DbWeight = (); // tried frame_support::weights::RocksDbWeight but nothing changed
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = ();
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type MaximumBlockWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = ();
	type BlockExecutionWeight = ();
	type AvailableBlockRatio = AvailableBlockRatio;
}
parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}
impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Module<Test>;
	type WeightInfo = ();
}
parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(16);
	pub static Period: BlockNumber = 5;
	pub static Offset: BlockNumber = 0;
}
sp_runtime::impl_opaque_keys! {
	pub struct SessionKeys {
		pub other: OtherSessionHandler,
	}
}
impl pallet_session::Config for Test {
	type Event = Event;
	type ValidatorId = AccountId;
	type ValidatorIdOf = crate::StashOf<Test>;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = ();
	type SessionManager = pallet_session::historical::NoteHistoricalRoot<Test, Stake>;
	type SessionHandler = (OtherSessionHandler,);
	type Keys = SessionKeys;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type WeightInfo = ();
}
impl pallet_session::historical::Config for Test {
	type FullIdentification = substrate::Exposure<u64, u128>;
	type FullIdentificationOf = ExposureOf<Self>;
}
parameter_types! {
	pub const UncleGenerations: u64 = 0;
}
impl pallet_authorship::Config for Test {
	type FindAuthor = Author11;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = Module<Test>;
}
parameter_types! {
	pub const MaxValidators: usize = 5;
	pub const MaxNomPerVal: usize = 10;
	pub const MinNomPerVal: usize = 1;
	pub const MinStakeBond: u128 = 5;
	pub const MinNomBond: u128 = 3;
	pub const MaxValFee: Perbill = Perbill::from_percent(50);
	pub const BlocksPerRound: u64 = 10;
	pub const HistoryDepth: u32 = 5;
	pub const Reward: u128 = 10;
	pub const Treasury: ModuleId = ModuleId(*b"py/trsry");
}
impl Config for Test {
	type Event = Event;
	type Currency = pallet_balances::Module<Test>;
	type SessionInterface = Self;
	type NextNewSession = pallet_session::Module<Test>;
	type MaxValidators = MaxValidators;
	type MaxNomPerVal = MaxNomPerVal;
	type MinNomPerVal = MinNomPerVal;
	type MinStakeBond = MinStakeBond;
	type MinNomBond = MinNomBond;
	type MaxValFee = MaxValFee;
	type BlocksPerRound = BlocksPerRound;
	type HistoryDepth = HistoryDepth;
	type Reward = Reward;
	type Treasury = Treasury;
}
pub type Sys = frame_system::Module<Test>;
pub type Balances = pallet_balances::Module<Test>;
type Stake = Module<Test>;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	let genesis = pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 1000), (2, 100), (3, 100), (4, 100), (5, 100), (6, 100)],
	};
	genesis.assimilate_storage(&mut storage).unwrap();
	let mut ext = sp_io::TestExternalities::from(storage);
	ext.execute_with(|| Sys::set_block_number(1));
	ext
}
