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
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
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

type MigrationNameFn = dyn FnMut() -> &'static str + Send + Sync;
type MigrationStepFn = dyn FnMut(Perbill, Weight) -> (Perbill, Weight) + Send + Sync;

#[derive(Default)]
pub struct MockMigrationManager {
	name_fn_callbacks: Vec<Arc<Mutex<MigrationNameFn>>>,
	step_fn_callbacks: Vec<Arc<Mutex<MigrationStepFn>>>,
}

impl MockMigrationManager {
	fn registerCallback(&mut self, name_fn: &MigrationNameFn, step_fn: &MigrationStepFn) {
		// self.name_fn_callbacks.push(Arc::new(name_fn));
		// self.step_fn_callbacks.push(Arc::new(step_fn));
	}

	fn invoke_name_fn(&mut self, index: usize) -> &'static str {
		// MigrationNameFn returns a String, we need a &str
		let arc = self.name_fn_callbacks[index].clone();
		let mut f = arc.lock().unwrap();
		f()
	}

	fn invoke_step_fn(&mut self, index: usize, previous_progress: Perbill, available_weight: Weight)
		-> (Perbill, Weight)
	{
		let arc = self.step_fn_callbacks[index].clone();
		let mut f = arc.lock().unwrap();
		f(previous_progress, available_weight)
	}

	fn generate_migrations_list(&self) -> Vec<Box<dyn Migration>> {
		panic!("FIXME");
	}
}

#[derive(Clone)]
pub struct MockMigration {
	pub index: usize,
}

impl Migration for MockMigration {
	fn friendly_name(&self) -> &str {
		MOCK_MIGRATIONS_LIST.lock().unwrap().invoke_name_fn(self.index)
	}
	fn step(&self, previous_progress: Perbill, available_weight: Weight) -> (Perbill, Weight) {
		MOCK_MIGRATIONS_LIST.lock().unwrap()
			.invoke_step_fn(self.index, previous_progress, available_weight)
	}
}

pub static MOCK_MIGRATIONS_LIST: Lazy<Mutex<MockMigrationManager>> = Lazy::new(|| {
	Default::default()
});

pub struct MockMigrations;
impl Get<Vec<Box<dyn Migration>>> for MockMigrations {
	fn get() -> Vec<Box<dyn Migration>> {
		MOCK_MIGRATIONS_LIST.lock().unwrap().generate_migrations_list()
	}
}

/*
#[derive(Clone)]
pub struct MockMigration {
	pub name: String,
	pub callback: MigrationStepFn,
}

impl Migration for MockMigration {
	fn friendly_name(&self) -> &str {
		&self.name[..]
	}
	fn step(&self, previous_progress: Perbill, available_weight: Weight) -> (Perbill, Weight) {
		let f = self.callback;
		f(previous_progress, available_weight)
	}
}

pub static MOCK_MIGRATIONS_LIST: Lazy<Mutex<Vec<MockMigration>>> = Lazy::new(|| {
	Mutex::new(vec![])
});
pub fn replace_mock_migrations_list(new_vec: &mut Vec<MockMigration>) {
	let mut list = MOCK_MIGRATIONS_LIST.lock().unwrap();
	list.clear();
	list.append(new_vec);
}

pub struct MockMigrations;
impl Get<Vec<Box<dyn Migration>>> for MockMigrations {
	fn get() -> Vec<Box<dyn Migration>> {

		let mut migrations_list: Vec<Box<dyn Migration>> = Vec::new();
		for mock in &*MOCK_MIGRATIONS_LIST.lock().unwrap() {
			migrations_list.push(Box::new(mock.clone()));
		}

		migrations_list
	}
}
*/

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
