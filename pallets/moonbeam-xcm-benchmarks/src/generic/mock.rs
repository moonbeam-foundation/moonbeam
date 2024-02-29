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
//! A mock runtime for XCM benchmarking.

use crate::{generic, mock::*, *};
use frame_benchmarking::BenchmarkError;
use frame_support::{
	parameter_types,
	traits::{Everything, OriginTrait},
};
use parity_scale_codec::Decode;
use sp_core::{ConstU64, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup, TrailingZeroInput},
	BuildStorage,
};
use xcm::latest::prelude::*;
use xcm_builder::{
	test_utils::{
		Assets, TestAssetExchanger, TestAssetLocker, TestAssetTrap, TestSubscriptionService,
		TestUniversalAliases,
	},
	AllowUnpaidExecutionFrom,
};
use xcm_executor::traits::ConvertOrigin;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		PolkadotXcmBenchmarks: pallet_xcm_benchmarks::generic,
		XcmGenericBenchmarks: generic,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub UniversalLocation: InteriorLocation = Here;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeTask = RuntimeTask;
	type Nonce = u64;
	type Block = Block;
	type Hash = H256;
	type RuntimeCall = RuntimeCall;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
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

impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 4];
	type MaxLocks = ();
	type Balance = u64;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<0>;
	type AccountStore = System;
	type WeightInfo = ();
	type RuntimeHoldReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeFreezeReason = ();
}

/// The benchmarks in this pallet should never need an asset transactor to begin with.
pub struct NoAssetTransactor;
impl xcm_executor::traits::TransactAsset for NoAssetTransactor {
	fn deposit_asset(_: &Asset, _: &Location, _: Option<&XcmContext>) -> Result<(), XcmError> {
		unreachable!();
	}

	fn withdraw_asset(_: &Asset, _: &Location, _: Option<&XcmContext>) -> Result<Assets, XcmError> {
		unreachable!();
	}
}

parameter_types! {
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsIntoHolding: u32 = 64;
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = DevNull;
	type AssetTransactor = NoAssetTransactor;
	type OriginConverter = AlwaysSignedByDefault<RuntimeOrigin>;
	type IsReserve = AllAssetLocationsPass;
	type IsTeleporter = ();
	type UniversalLocation = UniversalLocation;
	type Barrier = AllowUnpaidExecutionFrom<Everything>;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = xcm_builder::FixedRateOfFungible<WeightPrice, ()>;
	type ResponseHandler = DevNull;
	type AssetTrap = TestAssetTrap;
	type AssetLocker = TestAssetLocker;
	type AssetExchanger = TestAssetExchanger;
	type AssetClaims = TestAssetTrap;
	type SubscriptionService = TestSubscriptionService;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type FeeManager = ();
	// No bridges yet...
	type MessageExporter = ();
	type UniversalAliases = TestUniversalAliases;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
	type Aliasers = ();
}

impl pallet_xcm_benchmarks::Config for Test {
	type XcmConfig = XcmConfig;
	type AccountIdConverter = AccountIdConverter;
	type DeliveryHelper = ();
	fn valid_destination() -> Result<Location, BenchmarkError> {
		let valid_destination: Location = Junction::AccountId32 {
			network: None,
			id: [0u8; 32],
		}
		.into();

		Ok(valid_destination)
	}
	fn worst_case_holding(_depositable_count: u32) -> Assets {
		crate::mock::mock_worst_case_holding()
	}
}

impl pallet_xcm_benchmarks::generic::Config for Test {
	type RuntimeCall = RuntimeCall;
	type TransactAsset = Balances;

	fn worst_case_response() -> (u64, Response) {
		let assets: Assets = (Concrete(Here.into()), 100).into();
		(0, Response::Assets(assets))
	}

	fn worst_case_asset_exchange() -> Result<(Assets, Assets), BenchmarkError> {
		Err(BenchmarkError::Skip)
	}

	fn universal_alias() -> Result<(Location, Junction), BenchmarkError> {
		Err(BenchmarkError::Skip)
	}

	fn export_message_origin_and_destination(
	) -> Result<(Location, NetworkId, Junctions), BenchmarkError> {
		Err(BenchmarkError::Skip)
	}

	fn transact_origin_and_runtime_call() -> Result<(Location, RuntimeCall), BenchmarkError> {
		Ok((
			Default::default(),
			frame_system::Call::remark_with_event { remark: vec![] }.into(),
		))
	}

	fn subscribe_origin() -> Result<Location, BenchmarkError> {
		Ok(Default::default())
	}

	fn claimable_asset() -> Result<(Location, Location, Assets), BenchmarkError> {
		let assets: Assets = (Concrete(Here.into()), 100).into();
		let ticket = Location {
			parents: 0,
			interior: X1(GeneralIndex(0)),
		};
		Ok((Default::default(), ticket, assets))
	}

	fn unlockable_asset() -> Result<(Location, Location, Asset), BenchmarkError> {
		Err(BenchmarkError::Skip)
	}

	fn alias_origin() -> Result<(Location, Location), BenchmarkError> {
		Ok((Default::default(), Default::default()))
	}
}

impl generic::Config for Test {}
impl Config for Test {}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = RuntimeGenesisConfig {
		..Default::default()
	}
	.build_storage()
	.unwrap();
	t.into()
}

pub struct AlwaysSignedByDefault<Origin>(core::marker::PhantomData<Origin>);
impl<Origin> ConvertOrigin<Origin> for AlwaysSignedByDefault<Origin>
where
	Origin: OriginTrait,
	<Origin as OriginTrait>::AccountId: Decode,
{
	fn convert_origin(_origin: impl Into<Location>, _kind: OriginKind) -> Result<Origin, Location> {
		Ok(Origin::signed(
			<Origin as OriginTrait>::AccountId::decode(&mut TrailingZeroInput::zeroes())
				.expect("infinite length input; no invalid inputs for type; qed"),
		))
	}
}
