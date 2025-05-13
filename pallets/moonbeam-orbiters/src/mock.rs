// Copyright 2019-2025 PureStake Inc.
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

//! A minimal runtime including the moonbeam-orbiters pallet

use crate as pallet_moonbeam_orbiters;
use frame_support::{
	construct_runtime, pallet_prelude::*, parameter_types, traits::Everything, weights::Weight,
};
use frame_system::{pallet_prelude::BlockNumberFor, EnsureRoot};
use nimbus_primitives::{AccountLookup, NimbusId};
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage, Perbill,
};

pub type AccountId = u64;
pub type Balance = u128;
pub type BlockNumber = BlockNumberFor<Test>;

type Block = frame_system::mocking::MockBlockU32<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		MoonbeamOrbiters: pallet_moonbeam_orbiters,
	}
);

// Pallet system configuration

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const MaximumBlockWeight: Weight = Weight::from_parts(1024, 1);
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeTask = RuntimeTask;
	type Nonce = u64;
	type Block = Block;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
	type ExtensionsWeightInfo = ();
}

// Pallet balances configuration

parameter_types! {
	pub const ExistentialDeposit: u128 = 0;
}

impl pallet_balances::Config for Test {
	type MaxReserves = ConstU32<2>;
	type ReserveIdentifier = [u8; 4];
	type MaxLocks = ();
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type RuntimeHoldReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeFreezeReason = ();
	type DoneSlashHandler = ();
}

// Pallet moonbeam-orbiters configuration

parameter_types! {
	pub OrbiterReserveIdentifier: [u8; 4] = [b'o', b'r', b'b', b'i'];
}

pub struct MockAccountLookup;
impl AccountLookup<AccountId> for MockAccountLookup {
	fn lookup_account(nimbus_id: &NimbusId) -> Option<AccountId> {
		let nimbus_id_bytes: &[u8] = nimbus_id.as_ref();

		if nimbus_id_bytes[0] % 3 == 0 {
			Some(nimbus_id_bytes[0] as AccountId)
		} else {
			None
		}
	}
}

impl pallet_moonbeam_orbiters::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type AccountLookup = MockAccountLookup;
	type AddCollatorOrigin = EnsureRoot<AccountId>;
	type Currency = Balances;
	type DelCollatorOrigin = EnsureRoot<AccountId>;
	/// Maximum number of orbiters per collator
	type MaxPoolSize = ConstU32<2>;
	/// Maximum number of round to keep on storage
	type MaxRoundArchive = ConstU32<4>;
	type OrbiterReserveIdentifier = OrbiterReserveIdentifier;
	type RotatePeriod = ConstU32<2>;
	/// Round index type.
	type RoundIndex = u32;
	type WeightInfo = ();
}

pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
	min_orbiter_deposit: Balance,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![],
			min_orbiter_deposit: 10_000,
		}
	}
}

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}
	pub(crate) fn with_min_orbiter_deposit(mut self, min_orbiter_deposit: Balance) -> Self {
		self.min_orbiter_deposit = min_orbiter_deposit;
		self
	}
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		pallet_moonbeam_orbiters::GenesisConfig::<Test> {
			min_orbiter_deposit: self.min_orbiter_deposit,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet moonbeam-orbiters storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

/// Rolls to the desired block. Returns the number of blocks played.
pub(crate) fn roll_to(n: BlockNumber) -> BlockNumber {
	let mut num_blocks = 0;
	let mut block = System::block_number();
	while block < n {
		block = roll_one_block();
		num_blocks += 1;
	}
	num_blocks
}

// Rolls forward one block. Returns the new block number.
fn roll_one_block() -> BlockNumber {
	MoonbeamOrbiters::on_finalize(System::block_number());
	Balances::on_finalize(System::block_number());
	System::on_finalize(System::block_number());
	System::set_block_number(System::block_number() + 1);
	System::reset_events();
	System::on_initialize(System::block_number());
	Balances::on_initialize(System::block_number());
	MoonbeamOrbiters::on_initialize(System::block_number());
	// Trigger a new round each two blocks
	let block_number = System::block_number();
	if block_number % 2 == 0 {
		MoonbeamOrbiters::on_new_round(block_number as u32 / 2);
	}
	System::block_number()
}
