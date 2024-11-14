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
use frame_support::traits::AsEnsureOriginWithArg;
use frame_support::{
	construct_runtime, ord_parameter_types, parameter_types,
	traits::{EqualPrivilegeOnly, Everything, SortedMembers},
	weights::{RuntimeDbWeight, Weight},
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_asset_manager::AssetRegistrar;
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::testing::MockAccount;
use sp_core::{ConstU32, H160, H256, U256};
use sp_runtime::{
	traits::{BlakeTwo256, Hash, IdentityLookup},
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
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		AssetManager: pallet_asset_manager::{Pallet, Call, Storage, Event<T>},
		MoonbeamForeignAssets: pallet_moonbeam_foreign_assets::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const MaximumBlockWeight: Weight = Weight::from_parts(1024, 1);
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const SS58Prefix: u8 = 42;
}

parameter_types! {
	pub const MockDbWeight: RuntimeDbWeight = RuntimeDbWeight {
		read: 1_000_000,
		write: 1,
	};
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type DbWeight = MockDbWeight;
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
}

impl pallet_scheduler::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = ();
	type ScheduleOrigin = EnsureRoot<Self::AccountId>;
	type MaxScheduledPerBlock = ConstU32<100>;
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = ();
}

ord_parameter_types! {
	pub const One: u64 = 1;
	pub const Two: u64 = 2;
	pub const Three: u64 = 3;
	pub const Four: u64 = 4;
	pub const Five: u64 = 5;
	pub const Six: u64 = 6;
}
pub struct OneToFive;
impl SortedMembers<u64> for OneToFive {
	fn sorted_members() -> Vec<u64> {
		vec![1, 2, 3, 4, 5]
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn add(_m: &u64) {}
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
	type SuicideQuickClearLimit = SuicideQuickClearLimit;
}

parameter_types! {
	pub const AssetDeposit: u128 = 1;
	pub const MetadataDepositBase: u128 = 1;
	pub const MetadataDepositPerByte: u128 = 1;
	pub const ApprovalDeposit: u128 = 1;
	pub const AssetsStringLimit: u32 = 50;
	pub const AssetAccountDeposit: u128 = 1;
}

impl pallet_assets::Config<()> for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type AssetAccountDeposit = AssetAccountDeposit;
	type WeightInfo = ();
	type RemoveItemsLimit = ConstU32<656>;
	type AssetIdParameter = AssetId;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type CallbackHandle = ();
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum MockAssetType {
	Xcm(Location),
	MockAsset(AssetId),
}

impl Default for MockAssetType {
	fn default() -> Self {
		Self::MockAsset(0)
	}
}

impl From<MockAssetType> for AssetId {
	fn from(asset: MockAssetType) -> AssetId {
		match asset {
			MockAssetType::MockAsset(id) => id,
			MockAssetType::Xcm(id) => {
				let mut result: [u8; 16] = [0u8; 16];
				let hash: H256 = id.using_encoded(<Test as frame_system::Config>::Hashing::hash);
				result.copy_from_slice(&hash.as_fixed_bytes()[0..16]);
				u128::from_le_bytes(result)
			}
		}
	}
}

impl From<Location> for MockAssetType {
	fn from(location: Location) -> Self {
		Self::Xcm(location)
	}
}

impl Into<Option<Location>> for MockAssetType {
	fn into(self) -> Option<Location> {
		match self {
			Self::Xcm(location) => Some(location),
			_ => None,
		}
	}
}

pub struct MockAssetPalletRegistrar;

impl AssetRegistrar<Test> for MockAssetPalletRegistrar {
	fn create_foreign_asset(
		_asset: u128,
		_min_balance: u128,
		_metadata: u32,
		_is_sufficient: bool,
	) -> Result<(), DispatchError> {
		Ok(())
	}

	fn destroy_foreign_asset(_asset: u128) -> Result<(), DispatchError> {
		Ok(())
	}

	fn destroy_asset_dispatch_info_weight(_asset: u128) -> Weight {
		Weight::from_parts(0, 0)
	}
}

impl pallet_asset_manager::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type AssetRegistrarMetadata = u32;
	type ForeignAssetType = MockAssetType;
	type AssetRegistrar = MockAssetPalletRegistrar;
	type ForeignAssetModifierOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ();
}

pub struct AccountIdToH160;
impl sp_runtime::traits::Convert<AccountId, H160> for AccountIdToH160 {
	fn convert(account_id: AccountId) -> H160 {
		account_id.into()
	}
}

impl pallet_moonbeam_foreign_assets::Config for Test {
	type AccountIdToH160 = AccountIdToH160;
	type AssetIdFilter = Everything;
	type EvmRunner = pallet_evm::runner::stack::Runner<Self>;
	type ForeignAssetCreatorOrigin = EnsureRoot<AccountId>;
	type ForeignAssetFreezerOrigin = EnsureRoot<AccountId>;
	type ForeignAssetModifierOrigin = EnsureRoot<AccountId>;
	type ForeignAssetUnfreezerOrigin = EnsureRoot<AccountId>;
	type OnForeignAssetCreated = ();
	type MaxForeignAssets = ConstU32<3>;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type XcmLocationToH160 = ();
}

impl Config for Test {
	type WeightInfo = ();
	type ForeignAssetMigratorOrigin = EnsureRoot<AccountId>;
}

/// Externality builder for pallet migration's mock runtime
pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![
				(AccountId::from([1; 20]), 1000),
				(AccountId::from([2; 20]), 1000),
			],
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
