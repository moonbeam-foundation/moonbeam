// Copyright 2024 Moonbeam Foundation.
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

use super::*;
use crate as pallet_moonbeam_foreign_assets;

use frame_support::traits::Everything;
use frame_support::{construct_runtime, pallet_prelude::*, parameter_types};
use frame_system::EnsureRoot;
use pallet_evm::SubstrateBlockHashMapping;
use precompile_utils::testing::MockAccount;
use sp_core::{H256, U256};
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_runtime::BuildStorage;
use xcm::latest::Location;

pub type Balance = u128;

pub type BlockNumber = u32;
type AccountId = MockAccount;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Timestamp: pallet_timestamp,
		EVM: pallet_evm,
		EvmForeignAssets: pallet_moonbeam_foreign_assets,
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
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
	type SS58Prefix = ();
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
}

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = pallet_evm::EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<AccountId>;
	type AddressMapping = AccountId;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type ChainId = ();
	type BlockGasLimit = BlockGasLimit;
	type OnChargeTransaction = ();
	type FindAuthor = ();
	type BlockHashMapping = SubstrateBlockHashMapping<Self>;
	type OnCreate = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type SuicideQuickClearLimit = ConstU32<0>;
	type GasLimitStorageGrowthRatio = GasLimitStorageGrowthRatio;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Test>;
}

/// Gets parameters of last `ForeignAssetCreatedHook::on_asset_created` hook invocation
pub fn get_asset_created_hook_invocation<ForeignAsset: Decode>() -> Option<(ForeignAsset, AssetId)>
{
	storage::unhashed::get_raw(b"____on_foreign_asset_created")
		.map(|output| Decode::decode(&mut output.as_slice()).expect("Decoding should work"))
}

/// Notes down parameters of current `ForeignAssetCreatedHook::on_asset_created` hook invocation
fn note_on_asset_created_hook_invocation<ForeignAsset: Encode>(
	foreign_asset: &ForeignAsset,
	asset_id: &AssetId,
) {
	storage::unhashed::put_raw(
		b"____on_foreign_asset_created",
		(foreign_asset, asset_id).encode().as_slice(),
	);
}

/// Test hook that records the hook invocation with exact params
pub struct NoteDownHook<ForeignAsset>(PhantomData<ForeignAsset>);

impl<ForeignAsset: Encode> ForeignAssetCreatedHook<ForeignAsset> for NoteDownHook<ForeignAsset> {
	fn on_asset_created(foreign_asset: &ForeignAsset, asset_id: &AssetId) {
		note_on_asset_created_hook_invocation(foreign_asset, asset_id);
	}
}

pub struct AccountIdToH160;
impl sp_runtime::traits::Convert<AccountId, H160> for AccountIdToH160 {
	fn convert(account_id: AccountId) -> H160 {
		account_id.into()
	}
}

parameter_types! {
	pub const ForeignAssetDeposit: u64 = 1;
}

impl crate::Config for Test {
	type AccountIdToH160 = AccountIdToH160;
	type AssetIdFilter = Everything;
	type EvmRunner = pallet_evm::runner::stack::Runner<Self>;
	type ForeignAssetCreatorOrigin = EnsureRoot<AccountId>;
	type ForeignAssetFreezerOrigin = EnsureRoot<AccountId>;
	type ForeignAssetModifierOrigin = EnsureRoot<AccountId>;
	type ForeignAssetUnfreezerOrigin = EnsureRoot<AccountId>;
	type OnForeignAssetCreated = NoteDownHook<Location>;
	type MaxForeignAssets = ConstU32<3>;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type XcmLocationToH160 = ();
	type ForeignAssetDeposit = ();
	type BlockNumber = BlockNumber;
	type Balance = Balance;

	type Currency = Balances;

}

pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder { balances: vec![] }
	}
}

impl ExtBuilder {
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub(crate) fn events() -> Vec<super::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let RuntimeEvent::EvmForeignAssets(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}

pub fn expect_events(e: Vec<super::Event<Test>>) {
	assert_eq!(events(), e);
}
