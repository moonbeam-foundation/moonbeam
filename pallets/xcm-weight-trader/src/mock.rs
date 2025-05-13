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
use crate as pallet_xcm_weight_trader;
use frame_support::{
	construct_runtime, ord_parameter_types, parameter_types,
	traits::{Currency, Everything},
	weights::{constants::RocksDbWeight, IdentityFee},
};
use frame_system::EnsureSignedBy;
use pallet_moonbeam_foreign_assets::AssetCreate;
use sp_core::H256;
use sp_runtime::traits::MaybeEquivalence;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};
use xcm::opaque::latest::Junctions;
use xcm::v5::{Asset, Error as XcmError, Junction, Location, Result as XcmResult, XcmContext};

type AccountId = u64;
type Balance = u128;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		XcmWeightTrader: pallet_xcm_weight_trader::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
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

pub struct AccountIdToLocation;
impl Convert<AccountId, Location> for AccountIdToLocation {
	fn convert(account: AccountId) -> Location {
		Location::new(
			0,
			[Junction::AccountIndex64 {
				network: None,
				index: account,
			}],
		)
	}
}

pub struct AssetLocationFilter;
impl Contains<Location> for AssetLocationFilter {
	fn contains(location: &Location) -> bool {
		*location == <Test as Config>::NativeLocation::get() || *location == Location::parent()
	}
}

pub fn get_parent_asset_deposited() -> Option<(AccountId, Balance)> {
	storage::unhashed::get_raw(b"____parent_asset_deposited")
		.map(|output| Decode::decode(&mut output.as_slice()).expect("Decoding should work"))
}

pub struct MockAssetTransactor;
impl TransactAsset for MockAssetTransactor {
	fn deposit_asset(asset: &Asset, who: &Location, _context: Option<&XcmContext>) -> XcmResult {
		match (asset.id.clone(), asset.fun.clone()) {
			(XcmAssetId(location), Fungibility::Fungible(amount)) => {
				let who = match who.interior.iter().next() {
					Some(Junction::AccountIndex64 { index, .. }) => index,
					_ => panic!("invalid location"),
				};
				if location == <Test as Config>::NativeLocation::get() {
					let _ = Balances::deposit_creating(who, amount);
					Ok(())
				} else if location == Location::parent() {
					storage::unhashed::put_raw(
						b"____parent_asset_deposited",
						(who, amount).encode().as_slice(),
					);
					Ok(())
				} else {
					Err(XcmError::AssetNotFound)
				}
			}
			_ => Err(XcmError::AssetNotFound),
		}
	}
}

ord_parameter_types! {
	pub const AddAccount: u64 = 1;
	pub const EditAccount: u64 = 2;
	pub const PauseAccount: u64 = 3;
	pub const ResumeAccount: u64 = 4;
	pub const RemoveAccount: u64 = 5;
}

parameter_types! {
	pub NativeLocation: Location = Location::here();
	pub XcmFeesAccount: AccountId = 101;
	pub NotFilteredLocation: Location = Location::parent();
}

impl Config for Test {
	type AccountIdToLocation = AccountIdToLocation;
	type AddSupportedAssetOrigin = EnsureSignedBy<AddAccount, AccountId>;
	type AssetLocationFilter = AssetLocationFilter;
	type AssetTransactor = MockAssetTransactor;
	type Balance = Balance;
	type EditSupportedAssetOrigin = EnsureSignedBy<EditAccount, AccountId>;
	type NativeLocation = NativeLocation;
	type PauseSupportedAssetOrigin = EnsureSignedBy<PauseAccount, AccountId>;
	type RemoveSupportedAssetOrigin = EnsureSignedBy<RemoveAccount, AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type ResumeSupportedAssetOrigin = EnsureSignedBy<ResumeAccount, AccountId>;
	type WeightInfo = ();
	type WeightToFee = IdentityFee<Balance>;
	type XcmFeesAccount = XcmFeesAccount;
	#[cfg(feature = "runtime-benchmarks")]
	type NotFilteredLocation = NotFilteredLocation;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	sp_tracing::try_init_simple();
	let mut t = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();

	let balances = vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100)];
	pallet_balances::GenesisConfig::<Test> { balances }
		.assimilate_storage(&mut t)
		.unwrap();

	t.into()
}
