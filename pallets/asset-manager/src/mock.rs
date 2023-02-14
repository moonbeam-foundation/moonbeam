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

use super::*;
use crate as pallet_asset_manager;
use parity_scale_codec::{Decode, Encode};

use frame_support::{
	construct_runtime, parameter_types, traits::Everything, weights::Weight, RuntimeDebug,
};
use frame_system::EnsureRoot;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::traits::Hash as THash;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_runtime::DispatchError;
use xcm::latest::prelude::*;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub type AccountId = u64;
pub type Balance = u64;
pub type BlockNumber = u32;

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		AssetManager: pallet_asset_manager::{Pallet, Call, Storage, Event<T>},
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
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
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
}

parameter_types! {
	pub const AssetDeposit: u64 = 1;
	pub const ApprovalDeposit: u64 = 1;
	pub const StringLimit: u32 = 50;
	pub const MetadataDepositBase: u64 = 1;
	pub const MetadataDepositPerByte: u64 = 1;
}

parameter_types! {
	pub const StatemineParaIdInfo: u32 = 1000u32;
	pub const StatemineAssetsInstanceInfo: u8 = 50u8;
}

pub type AssetId = u32;
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum MockAssetType {
	MockAsset(AssetId),
	Xcm(MultiLocation),
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
				let mut result: [u8; 4] = [0u8; 4];
				let hash: H256 = id.using_encoded(<Test as frame_system::Config>::Hashing::hash);
				result.copy_from_slice(&hash.as_fixed_bytes()[0..4]);
				u32::from_le_bytes(result)
			}
		}
	}
}

impl From<MultiLocation> for MockAssetType {
	fn from(location: MultiLocation) -> Self {
		Self::Xcm(location)
	}
}

impl Into<Option<MultiLocation>> for MockAssetType {
	fn into(self) -> Option<MultiLocation> {
		match self {
			Self::Xcm(location) => Some(location),
			_ => None,
		}
	}
}

pub struct MockAssetPalletRegistrar;

impl AssetRegistrar<Test> for MockAssetPalletRegistrar {
	fn create_foreign_asset(
		_asset: u32,
		_min_balance: u64,
		_metadata: u32,
		_is_sufficient: bool,
	) -> Result<(), DispatchError> {
		Ok(())
	}

	fn create_local_asset(
		_asset: u32,
		_account: u64,
		_min_balance: u64,
		_is_sufficient: bool,
		_owner: u64,
	) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn destroy_foreign_asset(_asset: u32) -> Result<(), DispatchError> {
		Ok(())
	}

	fn destroy_local_asset(_asset: u32) -> Result<(), DispatchError> {
		Ok(())
	}

	fn destroy_asset_dispatch_info_weight(_asset: u32) -> Weight {
		Weight::from_ref_time(0)
	}
}

pub struct MockLocalAssetIdCreator;
impl pallet_asset_manager::LocalAssetIdCreator<Test> for MockLocalAssetIdCreator {
	fn create_asset_id_from_metadata(local_asset_counter: u128) -> AssetId {
		// Our means of converting a creator to an assetId
		// We basically hash nonce+account
		let mut result: [u8; 4] = [0u8; 4];
		let big_endian = local_asset_counter.to_le_bytes();
		result.copy_from_slice(&big_endian[0..4]);
		u32::from_le_bytes(result)
	}
}

parameter_types! {
	pub const LocalAssetDeposit: u64 = 1;
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u64;
	type AssetId = u32;
	type AssetRegistrarMetadata = u32;
	type ForeignAssetType = MockAssetType;
	type AssetRegistrar = MockAssetPalletRegistrar;
	type ForeignAssetModifierOrigin = EnsureRoot<u64>;
	type LocalAssetModifierOrigin = EnsureRoot<u64>;
	type LocalAssetIdCreator = MockLocalAssetIdCreator;
	type Currency = Balances;
	type LocalAssetDeposit = LocalAssetDeposit;
	type WeightInfo = ();
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
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
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
			if let RuntimeEvent::AssetManager(inner) = e {
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
