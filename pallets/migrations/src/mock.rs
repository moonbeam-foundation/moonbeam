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

//! A minimal runtime including the migrations pallet
use super::*;
use crate as pallet_migrations;
use frame_support::{
	construct_runtime, pallet_prelude::*, parameter_types, traits::GenesisBuild, weights::Weight,
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

pub type AccountId = u64;
pub type BlockNumber = u64;

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
		Migrations: pallet_migrations::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

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
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
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
}

#[derive(Default)]
pub struct MockMigrationManager<'test> {
	name_fn_callbacks: Vec<Box<dyn 'test + FnMut() -> &'static str>>,
	step_fn_callbacks: Vec<Box<dyn 'test + FnMut(Perbill, Weight) -> (Perbill, Weight)>>,
}

impl<'test> MockMigrationManager<'test> {
	pub fn registerCallback<FN, FS>(&mut self, name_fn: FN, step_fn: FS)
	where
		FN: 'test + FnMut() -> &'static str,
		FS: 'test + FnMut(Perbill, Weight) -> (Perbill, Weight),
	{
		self.name_fn_callbacks.push(Box::new(name_fn));
		self.step_fn_callbacks.push(Box::new(step_fn));
	}

	pub(crate) fn invoke_name_fn(&mut self, index: usize) -> &'static str {
		self.name_fn_callbacks[index]()
	}

	pub(crate) fn invoke_step_fn(&mut self, index: usize, previous_progress: Perbill, available_weight: Weight)
		-> (Perbill, Weight)
	{
		self.step_fn_callbacks[index](previous_progress, available_weight)
	}

	fn generate_migrations_list(&self) -> Vec<Box<dyn Migration>> {
		let mut migrations: Vec<Box<dyn Migration>> = Vec::new();
		for i in 0..self.name_fn_callbacks.len() {
			migrations.push(Box::new(MockMigration{index: i}));
		}
		migrations
	}
}
environmental!(MOCK_MIGRATIONS_LIST: MockMigrationManager<'static>);

pub fn execute_with_mock_migrations<CB>(callback: &mut CB)
where
	CB: FnMut(&mut MockMigrationManager)
{
	let mut original_mgr: MockMigrationManager = Default::default();
	MOCK_MIGRATIONS_LIST::using(&mut original_mgr, || {
		MOCK_MIGRATIONS_LIST::with(|inner_mgr: &mut MockMigrationManager| {
			callback(inner_mgr);
		});

		// mimic the calls that would occur from the time a runtime upgrade starts until the
		// Migrations pallet indicates that all upgrades are complete

		ExtBuilder::default().build().execute_with(|| {
			let mut block_number = 1u64;
			Migrations::on_runtime_upgrade();

			while ! Migrations::is_fully_upgraded() {
				System::set_block_number(block_number);
				System::on_initialize(System::block_number());
				Migrations::on_initialize(System::block_number());
				Migrations::on_finalize(System::block_number());
				System::on_finalize(System::block_number());

				block_number += 1;

				if block_number > 99999 {
					panic!("Infinite loop?");
				}
			}
		});
	});
}

#[derive(Clone)]
pub struct MockMigration {
	pub index: usize,
}

impl Migration for MockMigration {
	fn friendly_name(&self) -> &str {
		let mut result: &str = "";
		MOCK_MIGRATIONS_LIST::with(|mgr: &mut MockMigrationManager| {
			result = mgr.invoke_name_fn(self.index);
		});
		result
	}
	fn step(&self, previous_progress: Perbill, available_weight: Weight) -> (Perbill, Weight) {
		let mut result: (Perbill, Weight) = (Perbill::zero(), 0u64.into());
		MOCK_MIGRATIONS_LIST::with(|mgr: &mut MockMigrationManager| {
			result = mgr.invoke_step_fn(self.index, previous_progress, available_weight);
		});
		result
	}
}

pub struct MockMigrations;
impl Get<Vec<Box<dyn Migration>>> for MockMigrations {
	fn get() -> Vec<Box<dyn Migration>> {
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
	completed_migrations: Vec<Vec<u8>>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			completed_migrations: vec![],
		}
	}
}

impl ExtBuilder {
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.expect("Frame system builds valid default genesis config");

		pallet_migrations::GenesisConfig::<Test> {
			completed_migrations: self.completed_migrations,
			dummy: Default::default(),
		}
		.assimilate_storage(&mut t)
		.expect("Pallet migration's storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub(crate) fn events() -> Vec<pallet_migrations::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let Event::pallet_migrations(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}
