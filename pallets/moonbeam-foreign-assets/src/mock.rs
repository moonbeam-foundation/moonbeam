// Copyright Moonsong Labs
// This file is part of Moonkit.

// Moonkit is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Moonkit is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Moonkit.  If not, see <http://www.gnu.org/licenses/>.

use super::*;
use crate as pallet_foreign_asset_creator;
use std::marker::PhantomData;

use frame_support::{
	construct_runtime, parameter_types, storage,
	traits::{ConstU32, Everything},
};
use frame_system::EnsureRoot;
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_runtime::BuildStorage;
use xcm::latest::prelude::*;

type Block = frame_system::mocking::MockBlock<Test>;

pub type AccountId = u64;
pub type Balance = u64;

construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		ForeignAssetCreator: pallet_foreign_asset_creator,
		Assets: pallet_assets,
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
}
impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Block = Block;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type RuntimeTask = ();
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
}

parameter_types! {
	pub const AssetDeposit: u64 = 0;
	pub const ApprovalDeposit: u64 = 0;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: u64 = 0;
	pub const MetadataDepositPerByte: u64 = 0;
}

type AssetId = u32;

impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type AssetIdParameter = parity_scale_codec::Compact<AssetId>;
	type Currency = Balances;
	type CreateOrigin = frame_support::traits::NeverEnsureOrigin<AccountId>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = StringLimit;
	type Freezer = ();
	type Extra = ();
	type CallbackHandle = ();
	type WeightInfo = ();
	type RemoveItemsLimit = ConstU32<1000>;
	pallet_assets::runtime_benchmarks_enabled! {
		type BenchmarkHelper = ();
	}
}

/// Gets parameters of last `ForeignAssetCreatedHook::on_asset_created` hook invocation
pub fn get_asset_created_hook_invocation<
	ForeignAsset: Decode,
	AssetId: Decode,
	AssetBalance: Decode,
>() -> Option<(ForeignAsset, AssetId, AssetBalance)> {
	storage::unhashed::get_raw(b"____on_foreign_asset_created")
		.map(|output| Decode::decode(&mut output.as_slice()).expect("Decoding should work"))
}

/// Notes down parameters of current `ForeignAssetCreatedHook::on_asset_created` hook invocation
fn note_on_asset_created_hook_invocation<
	ForeignAsset: Encode,
	AssetId: Encode,
	AssetBalance: Encode,
>(
	foreign_asset: &ForeignAsset,
	asset_id: &AssetId,
	min_balance: &AssetBalance,
) {
	storage::unhashed::put_raw(
		b"____on_foreign_asset_created",
		(foreign_asset, asset_id, min_balance).encode().as_slice(),
	);
}

/// Gets parameters of last `ForeignAssetDestroyedHook::on_asset_destroyed` hook invocation
pub fn get_asset_destroyed_hook_invocation<ForeignAsset: Decode, AssetId: Decode>(
) -> Option<(ForeignAsset, AssetId)> {
	storage::unhashed::get_raw(b"____on_foreign_asset_destroyed")
		.map(|output| Decode::decode(&mut output.as_slice()).expect("Decoding should work"))
}

/// Notes down parameters of current `ForeignAssetDestroyedHook::on_asset_destroyed` hook invocation
fn note_on_asset_destroyed_hook_invocation<ForeignAsset: Encode, AssetId: Encode>(
	foreign_asset: &ForeignAsset,
	asset_id: &AssetId,
) {
	storage::unhashed::put_raw(
		b"____on_foreign_asset_destroyed",
		(foreign_asset, asset_id).encode().as_slice(),
	);
}

/// Test hook that records the hook invocation with exact params
pub struct NoteDownHook<ForeignAsset, AssetId, AssetBalance>(
	PhantomData<(ForeignAsset, AssetId, AssetBalance)>,
);

impl<ForeignAsset: Encode, AssetId: Encode, AssetBalance: Encode>
	ForeignAssetCreatedHook<ForeignAsset, AssetId, AssetBalance>
	for NoteDownHook<ForeignAsset, AssetId, AssetBalance>
{
	fn on_asset_created(
		foreign_asset: &ForeignAsset,
		asset_id: &AssetId,
		min_balance: &AssetBalance,
	) {
		note_on_asset_created_hook_invocation(foreign_asset, asset_id, min_balance);
	}
}

impl<ForeignAsset: Encode, AssetId: Encode, AssetBalance>
	ForeignAssetDestroyedHook<ForeignAsset, AssetId>
	for NoteDownHook<ForeignAsset, AssetId, AssetBalance>
{
	fn on_asset_destroyed(foreign_asset: &ForeignAsset, asset_id: &AssetId) {
		note_on_asset_destroyed_hook_invocation(foreign_asset, asset_id);
	}
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type ForeignAsset = Location;
	type ForeignAssetCreatorOrigin = EnsureRoot<AccountId>;
	type ForeignAssetModifierOrigin = EnsureRoot<AccountId>;
	type ForeignAssetDestroyerOrigin = EnsureRoot<AccountId>;
	type Fungibles = Assets;
	type WeightInfo = ();
	type OnForeignAssetCreated = NoteDownHook<Location, AssetId, Balance>;
	type OnForeignAssetDestroyed = NoteDownHook<Location, AssetId, Balance>;
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
			if let RuntimeEvent::ForeignAssetCreator(inner) = e {
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
