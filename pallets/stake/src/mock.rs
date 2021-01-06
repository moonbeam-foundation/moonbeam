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
	Perbill,
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
	type FullIdentification = pallet_staking::Exposure<u64, u128>;
	type FullIdentificationOf = ExposureOf<Self>;
}
impl author::Config for Test {
	type FindAuthor = Author11;
	type EventHandler = Module<Test>;
}
parameter_types! {
	pub const BlocksPerRound: u32 = 5;
	pub const BondDuration: u32 = 2;
	pub const MaxValidators: u32 = 5;
	pub const MaxNominatorsPerValidator: usize = 10;
	pub const MaxFee: Perbill = Perbill::from_percent(50);
	pub const MinValidatorStk: u128 = 10;
	pub const MinNominatorStk: u128 = 5;
}
impl Config for Test {
	type Event = MetaEvent;
	type Currency = Balances;
	type SessionInterface = Self;
	type NextNewSession = pallet_session::Module<Test>;
	type BlocksPerRound = BlocksPerRound;
	type BondDuration = BondDuration;
	type MaxValidators = MaxValidators;
	type MaxNominatorsPerValidator = MaxNominatorsPerValidator;
	type MaxFee = MaxFee;
	type MinValidatorStk = MinValidatorStk;
	type MinNominatorStk = MinNominatorStk;
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

pub fn genesis2() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	let genesis = pallet_balances::GenesisConfig::<Test> {
		balances: vec![
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
	};
	genesis.assimilate_storage(&mut storage).unwrap();
	GenesisConfig::<Test> {
		stakers: vec![
			// validators
			(1, None, 100),
			(2, None, 90),
			(3, None, 80),
			(4, None, 70),
			(5, None, 60),
			(6, None, 50),
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
