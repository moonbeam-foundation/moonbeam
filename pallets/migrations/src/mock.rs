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

//! A minimal runtime including the migrations pallet

use super::*;
use crate as pallet_migrations;
use frame_support::{
	construct_runtime,
	pallet_prelude::*,
	parameter_types,
	traits::{Everything, GenesisBuild},
	weights::{constants::RocksDbWeight, Weight},
};
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

pub type AccountId = u64;
pub type BlockNumber = u32;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Migrations: pallet_migrations::{Pallet, Storage, Config, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const MaximumBlockWeight: Weight = Weight::from_ref_time(1024);
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type DbWeight = RocksDbWeight;
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

/// MockMigrationManager stores the test-side callbacks/closures used in the Migrations list glue.
/// It is is expected to exist as a singleton, but only remain relevant within the scope of a test.
///
/// Tests should use execute_with_mock_migrations(), which will create a MockMigrationManager and
/// provide it to the test, ensuring that the MockMigrationManager remains valid for the duration of
/// the callback.
///
/// A pair of callbacks provided to register_callback() will map directly to a single instance of
/// Migration (done by the MockMigration glue below). Treat each pair of callbacks as though it were
/// a custom implementation of the Migration trait just as a normal Pallet would.
pub struct MockMigrationManager<'test> {
	name_fn_callbacks: Vec<Box<dyn 'test + FnMut() -> &'static str>>,
	migrate_fn_callbacks: Vec<Box<dyn 'test + FnMut(Weight) -> Weight>>,
	pre_upgrade_fn_callbacks: Vec<Box<dyn 'test + FnMut() -> Result<(), &'static str>>>,
	post_upgrade_fn_callbacks: Vec<Box<dyn 'test + FnMut() -> Result<(), &'static str>>>,
}

impl Default for MockMigrationManager<'_> {
	fn default() -> Self {
		Self {
			name_fn_callbacks: Default::default(),
			migrate_fn_callbacks: Default::default(),
			pre_upgrade_fn_callbacks: Default::default(),
			post_upgrade_fn_callbacks: Default::default(),
		}
	}
}

impl<'test> MockMigrationManager<'test> {
	pub fn register_callback<FN, FM>(&mut self, name_fn: FN, migrate_fn: FM)
	where
		FN: 'test + FnMut() -> &'static str,
		FM: 'test + FnMut(Weight) -> Weight,
	{
		self.name_fn_callbacks.push(Box::new(name_fn));
		self.migrate_fn_callbacks.push(Box::new(migrate_fn));
		self.pre_upgrade_fn_callbacks.push(Box::new(|| Ok(())));
		self.post_upgrade_fn_callbacks.push(Box::new(|| Ok(())));
	}
	#[cfg(feature = "try-runtime")]
	pub fn register_callback_with_try_fns<FN, FM, FT1, FT2>(
		&mut self,
		name_fn: FN,
		migrate_fn: FM,
		pre_upgrade_fn: FT1,
		post_upgrade_fn: FT2,
	) where
		FN: 'test + FnMut() -> &'static str,
		FM: 'test + FnMut(Weight) -> Weight,
		FT1: 'test + FnMut() -> Result<(), &'static str>,
		// no two closures, even if identical, have the same type
		FT2: 'test + FnMut() -> Result<(), &'static str>,
	{
		self.name_fn_callbacks.push(Box::new(name_fn));
		self.migrate_fn_callbacks.push(Box::new(migrate_fn));
		self.pre_upgrade_fn_callbacks.push(Box::new(pre_upgrade_fn));
		self.post_upgrade_fn_callbacks
			.push(Box::new(post_upgrade_fn));
	}

	pub(crate) fn invoke_name_fn(&mut self, index: usize) -> &'static str {
		self.name_fn_callbacks[index]()
	}

	pub(crate) fn invoke_migrate_fn(&mut self, index: usize, available_weight: Weight) -> Weight {
		self.migrate_fn_callbacks[index](available_weight)
	}

	#[cfg(feature = "try-runtime")]
	pub(crate) fn invoke_pre_upgrade(&mut self, index: usize) -> Result<(), &'static str> {
		self.pre_upgrade_fn_callbacks[index]()
	}

	#[cfg(feature = "try-runtime")]
	pub(crate) fn invoke_post_upgrade(&mut self, index: usize) -> Result<(), &'static str> {
		self.post_upgrade_fn_callbacks[index]()
	}

	fn generate_migrations_list(&self) -> Vec<Box<dyn Migration>> {
		let mut migrations: Vec<Box<dyn Migration>> = Vec::new();
		for i in 0..self.name_fn_callbacks.len() {
			migrations.push(Box::new(MockMigration { index: i }));
		}
		migrations
	}
}

// Our global Migrations list. Necessary because the Get impl must be fulfilled with nothing but
// a static context.
environmental!(MOCK_MIGRATIONS_LIST: MockMigrationManager<'static>);

/// Utility method for tests to implement their logic with a pre-generated MockMigrationManager.
/// This helps avoid lifetime issues between the implied 'static lifetime of MOCK_MIGRATIONS_LIST
/// and the function-scoped lifetimes of closures used in tests.
pub fn execute_with_mock_migrations<CB, ECB>(callback: &mut CB, post_migration_callback: &mut ECB)
where
	CB: FnMut(&mut MockMigrationManager),
	ECB: FnMut(),
{
	let mut original_mgr: MockMigrationManager = Default::default();
	MOCK_MIGRATIONS_LIST::using(&mut original_mgr, || {
		MOCK_MIGRATIONS_LIST::with(|inner_mgr: &mut MockMigrationManager| {
			callback(inner_mgr);
		});
		post_migration_callback();
	});
}

#[derive(Clone)]
pub struct MockMigration {
	pub index: usize,
}

/// The implementation of Migration for our glue: MockMigration contains nothing more than an index
/// which is used inside of the callbacks at runtime to look up our global callbacks stored in
/// MOCK_MIGRATIONS_LIST and invoke those.
impl Migration for MockMigration {
	fn friendly_name(&self) -> &str {
		let mut result: &str = "";
		MOCK_MIGRATIONS_LIST::with(|mgr: &mut MockMigrationManager| {
			result = mgr.invoke_name_fn(self.index);
		});
		result
	}
	fn migrate(&self, available_weight: Weight) -> Weight {
		let mut result = Weight::zero();
		MOCK_MIGRATIONS_LIST::with(|mgr: &mut MockMigrationManager| {
			result = mgr.invoke_migrate_fn(self.index, available_weight);
		});
		result
	}
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		let mut result: Result<(), &'static str> = Err("closure didn't set result");
		MOCK_MIGRATIONS_LIST::with(|mgr: &mut MockMigrationManager| {
			result = mgr.invoke_pre_upgrade(self.index);
		});
		result
	}
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		let mut result: Result<(), &'static str> = Err("closure didn't set result");
		MOCK_MIGRATIONS_LIST::with(|mgr: &mut MockMigrationManager| {
			result = mgr.invoke_post_upgrade(self.index);
		});
		result
	}
}

/// Implementation of Migrations. Generates a Vec of MockMigrations on the fly based on the current
/// contents of MOCK_MIGRATIONS_LIST.
pub struct MockMigrations;
impl GetMigrations for MockMigrations {
	fn get_migrations() -> Vec<Box<dyn Migration>> {
		let mut migrations: Vec<Box<dyn Migration>> = Vec::new();
		MOCK_MIGRATIONS_LIST::with(|mgr: &mut MockMigrationManager| {
			migrations = mgr.generate_migrations_list();
		});
		migrations
	}
}

impl Config for Test {
	type Event = Event;
	type MigrationsList = MockMigrations;
}

/// Externality builder for pallet migration's mock runtime
pub(crate) struct ExtBuilder {
	uncompleted_migrations: Vec<&'static str>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			uncompleted_migrations: Vec::new(),
		}
	}
}

impl ExtBuilder {
	pub(crate) fn with_uncompleted_migrations(uncompleted_migrations: Vec<&'static str>) -> Self {
		Self {
			uncompleted_migrations,
		}
	}
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut storage = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.expect("Frame system builds valid default genesis config");

		GenesisBuild::<Test>::assimilate_storage(&pallet_migrations::GenesisConfig, &mut storage)
			.expect("Pallet migration's storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(storage);
		if !self.uncompleted_migrations.is_empty() {
			for migration_name in self.uncompleted_migrations {
				ext.insert(
					<crate::pallet::MigrationState<Test>>::hashed_key_for(
						migration_name.as_bytes(),
					),
					false.encode(),
				);
			}
		}
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub(crate) fn events() -> Vec<pallet_migrations::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let Event::Migrations(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}

#[cfg(feature = "try-runtime")]
pub(crate) fn invoke_all_upgrade_hooks() -> Weight {
	Migrations::pre_upgrade().expect("pre-upgrade hook succeeds");
	let weight = Migrations::on_runtime_upgrade();
	Migrations::post_upgrade().expect("post-upgrade hook succeeds");

	weight
}

pub(crate) fn roll_to(block_number: BlockNumber, invoke_on_runtime_upgrade_first: bool) {
	if invoke_on_runtime_upgrade_first {
		Migrations::on_runtime_upgrade();
	}

	while System::block_number() < block_number {
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Migrations::on_initialize(System::block_number());
		Migrations::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
	}
}

pub(crate) fn roll_until_upgraded(invoke_on_runtime_upgrade_first: bool) {
	if invoke_on_runtime_upgrade_first {
		Migrations::on_runtime_upgrade();
	}

	while !Migrations::is_fully_upgraded() {
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Migrations::on_initialize(System::block_number());
		Migrations::on_finalize(System::block_number());
		System::on_finalize(System::block_number());

		if System::block_number() > 99999 {
			panic!("Infinite loop?");
		}
	}
}
