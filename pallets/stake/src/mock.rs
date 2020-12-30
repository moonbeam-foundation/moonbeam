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
	impl_outer_event, impl_outer_origin, parameter_types,
	traits::{FindAuthor, OnFinalize, OnInitialize},
	weights::Weight,
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
	pub enum MetaEvent for Test {
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
	type Event = MetaEvent;
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
	type Event = MetaEvent;
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
	type Event = MetaEvent;
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
	pub const MaxValidators: u32 = 5;
	pub const MaxNominatorsPerValidator: usize = 10;
	pub const MinNominatorsPerValidator: usize = 0;
	pub const MinCandidateBond: u128 = 10;
	pub const MinValidatorBond: u128 = 20;
	pub const MinNominatorBond: u128 = 5;
	pub const MaxValidatorFee: Perbill = Perbill::from_percent(50);
	pub const MaxStrikes: u8 = 3;
	pub const StrikeFee: u128 = 1;
	pub const SlashWindow: u32 = 10;
	pub const SlashPct: Perbill = Perbill::from_percent(50);
	pub const Pts2StakeRewardRatio: Perbill = Perbill::from_percent(50);
	pub const BlocksPerRound: u32 = 5;
	pub const Reward: u128 = 10;
	pub const Treasury: ModuleId = ModuleId(*b"py/trsry");
}
impl Config for Test {
	type Event = MetaEvent;
	type Currency = Balances;
	type SessionInterface = Self;
	type NextNewSession = pallet_session::Module<Test>;
	type MaxValidators = MaxValidators;
	type MaxNominatorsPerValidator = MaxNominatorsPerValidator;
	type MinNominatorsPerValidator = MinNominatorsPerValidator;
	type MinCandidateBond = MinCandidateBond;
	type MinValidatorBond = MinValidatorBond;
	type MinNominatorBond = MinNominatorBond;
	type MaxValidatorFee = MaxValidatorFee;
	type SlashOrigin = frame_system::EnsureRoot<u64>;
	type MaxStrikes = MaxStrikes;
	type StrikeFee = StrikeFee;
	type SlashWindow = SlashWindow;
	type SlashPct = SlashPct;
	type Pts2StakeRewardRatio = Pts2StakeRewardRatio;
	type BlocksPerRound = BlocksPerRound;
	type Reward = Reward;
	type Treasury = Treasury;
}
pub type Sys = frame_system::Module<Test>;
pub type Balances = pallet_balances::Module<Test>;
pub type Stake = Module<Test>;

pub fn genesis() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	let genesis = pallet_balances::GenesisConfig::<Test> {
		balances: vec![
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
	};
	genesis.assimilate_storage(&mut storage).unwrap();
	GenesisConfig::<Test> {
		stakers: vec![
			// validators
			(1, None, 500),
			(2, None, 200),
			// nominators
			(3, Some(1), 100),
			(4, Some(1), 100),
			(5, Some(2), 100),
			(6, Some(2), 100),
		],
	}
	.assimilate_storage(&mut storage)
	.unwrap();
	let mut ext = sp_io::TestExternalities::from(storage);
	ext.execute_with(|| Sys::set_block_number(1));
	ext
}

pub fn roll_to(n: u64) {
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

pub fn last_event() -> MetaEvent {
	Sys::events().pop().expect("Event expected").event
}
