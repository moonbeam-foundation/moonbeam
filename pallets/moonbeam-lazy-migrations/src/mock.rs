// Copyright 2024 Moonbeam foundation
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

//! A minimal runtime including the multi block migrations pallet

use super::*;
use crate as pallet_moonbeam_lazy_migrations;
use frame_support::weights::constants::RocksDbWeight;
use frame_support::{construct_runtime, parameter_types, traits::Everything, weights::Weight};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot, FrameSystemAccountProvider};
use precompile_utils::testing::MockAccount;
use sp_core::{H160, H256, U256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage, Perbill,
};

pub type AssetId = u128;
pub type Balance = u128;
pub type AccountId = MockAccount;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Timestamp: pallet_timestamp,
		EVM: pallet_evm,
		LazyMigrations: pallet_moonbeam_lazy_migrations::{Pallet, Call},
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const MaximumBlockWeight: Weight = Weight::from_parts(1024, 1);
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type DbWeight = RocksDbWeight;
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

parameter_types! {
	pub const ExistentialDeposit: u128 = 0;
}
impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = ();
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

parameter_types! {
	pub const MinimumPeriod: u64 = 6000 / 2;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub BlockGasLimit: U256 = U256::from(u64::MAX);
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub GasLimitPovSizeRatio: u64 = 16;
	pub GasLimitStorageGrowthRatio: u64 = 366;
	pub SuicideQuickClearLimit: u32 = 0;
}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = AccountId;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type OnCreate = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type GasLimitStorageGrowthRatio = GasLimitStorageGrowthRatio;
	type Timestamp = Timestamp;
	type WeightInfo = ();
	type AccountProvider = FrameSystemAccountProvider<Test>;
}

parameter_types! {
	pub const AssetDeposit: u128 = 1;
	pub const MetadataDepositBase: u128 = 1;
	pub const MetadataDepositPerByte: u128 = 1;
	pub const ApprovalDeposit: u128 = 1;
	pub const AssetsStringLimit: u32 = 50;
	pub const AssetAccountDeposit: u128 = 1;
}

impl Config for Test {
	type WeightInfo = ();
}

// Constants for test accounts
pub const ALITH: AccountId = MockAccount(H160([1; 20]));
pub const BOB: AccountId = MockAccount(H160([2; 20]));

pub const PARA_A: AccountId = MockAccount(H160([2; 20]));
pub const PARA_B: AccountId = MockAccount(H160([2; 20]));
pub const PARA_C: AccountId = MockAccount(H160([2; 20]));

/// Externality builder for pallet migration's mock runtime
pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![(ALITH, 1000), (BOB, 1000)],
		}
	}
}

impl ExtBuilder {
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut storage = frame_system::GenesisConfig::<Test>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}
		.assimilate_storage(&mut storage)
		.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(storage);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
