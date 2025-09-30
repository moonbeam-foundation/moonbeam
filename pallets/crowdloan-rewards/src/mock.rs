// Copyright 2025 Moonbeam Foundation.
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

//! Test utilities for crowdloan-rewards pallet

use crate::{self as pallet_crowdloan_rewards, Config};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Everything, OnFinalize, OnInitialize},
	weights::{constants::RocksDbWeight, Weight},
	PalletId,
};
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{
	traits::{BlakeTwo256, BlockNumberProvider, IdentityLookup},
	BuildStorage, Perbill,
};

pub type AccountId = AccountId32;
pub type Balance = u128;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		CrowdloanRewards: pallet_crowdloan_rewards,
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const MaximumBlockWeight: Weight = Weight::from_parts(1024, 1);
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type DbWeight = RocksDbWeight;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeTask = RuntimeTask;
	type Nonce = u64;
	type Block = frame_system::mocking::MockBlockU32<Test>;
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

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Test {
	type MaxReserves = ();
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

pub struct MockVestingBlockNumberProvider;
impl BlockNumberProvider for MockVestingBlockNumberProvider {
	type BlockNumber = u32;

	fn current_block_number() -> Self::BlockNumber {
		System::block_number()
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn set_block_number(n: Self::BlockNumber) {
		frame_system::Pallet::<Test>::set_block_number(n);
	}
}

parameter_types! {
	pub const Initialized: bool = true;
	pub const InitializationPayment: Perbill = Perbill::from_percent(25);
	pub const MaxInitContributors: u32 = 100;
	pub const MinimumReward: Balance = 1_000;
	pub const RewardAddressRelayVoteThreshold: Perbill = Perbill::from_percent(60);
	pub const SignatureNetworkIdentifier: &'static [u8] = b"TEST_NET";
	pub const CrowdloanPalletId: PalletId = PalletId(*b"Crowdloa");
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Initialized = Initialized;
	type InitializationPayment = InitializationPayment;
	type MaxInitContributors = MaxInitContributors;
	type MinimumReward = MinimumReward;
	type RewardAddressRelayVoteThreshold = RewardAddressRelayVoteThreshold;
	type RewardCurrency = Balances;
	type RelayChainAccountId = AccountId;
	type RewardAddressChangeOrigin = frame_system::EnsureRoot<AccountId>;
	type SignatureNetworkIdentifier = SignatureNetworkIdentifier;
	type RewardAddressAssociateOrigin = frame_system::EnsureRoot<AccountId>;
	type VestingBlockNumber = u32;
	type VestingBlockProvider = MockVestingBlockNumberProvider;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	new_test_ext_with_config(default_crowdloan_genesis_config())
}

// Build genesis with custom crowdloan config
pub fn new_test_ext_with_config(
	crowdloan_config: crate::GenesisConfig<Test>,
) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			// Pallet account with initial funds
			(CrowdloanRewards::account_id(), 1_000_000_000),
			// Test accounts
			(AccountId::from([1u8; 32]), 100_000),
			(AccountId::from([2u8; 32]), 100_000),
			(AccountId::from([3u8; 32]), 100_000),
		],
		dev_accounts: None,
	}
	.assimilate_storage(&mut t)
	.unwrap();

	crowdloan_config.assimilate_storage(&mut t).unwrap();

	t.into()
}

// Default crowdloan genesis config with some initialized state
pub fn default_crowdloan_genesis_config() -> crate::GenesisConfig<Test> {
	crate::GenesisConfig {
		funded_accounts: vec![
			// Associated account with rewards
			(
				AccountId::from([10u8; 32]),      // relay account
				Some(AccountId::from([1u8; 32])), // native account
				10_000u128,                       // reward
			),
		],
		init_vesting_block: 1u32,
		end_vesting_block: 100u32,
	}
}

// Empty crowdloan genesis config
pub fn empty_crowdloan_genesis_config() -> crate::GenesisConfig<Test> {
	crate::GenesisConfig {
		funded_accounts: vec![],
		init_vesting_block: 1u32,
		end_vesting_block: 100u32,
	}
}

// Helper functions for test setup
pub fn run_to_block(n: u32) {
	while System::block_number() < n {
		CrowdloanRewards::on_finalize(System::block_number());
		Balances::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Balances::on_initialize(System::block_number());
		CrowdloanRewards::on_initialize(System::block_number());
	}
}
