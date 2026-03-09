// Copyright 2019-2025 Moonbeam Foundation.
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

//! Minimal Asset Hub mock for XCM integration tests.
//!
//! This provides a simplified Asset Hub that can:
//! - Receive/send XCMP messages from/to sibling parachains
//! - Receive DMP messages from relay chain
//! - Handle reserve asset transfers

use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	traits::{Everything, Nothing},
};
use sp_core::ConstU32;
use sp_io::TestExternalities;
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use sp_weights::Weight;
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowTopLevelPaidExecutionFrom, FixedWeightBounds,
	FrameTransactionalProcessor, FungibleAdapter, IsConcrete, ParentIsPreset,
	SiblingParachainConvertsVia, SignedAccountId32AsNative, SovereignSignedViaLocation,
	TakeWeightCredit,
};
use xcm_executor::XcmExecutor;
use xcm_simulator::mock_message_queue;

pub type AccountId = AccountId32;
pub type Balance = u128;

pub const ASSET_HUB_PARA_ID: u32 = 1000;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = frame_system::mocking::MockBlock<Runtime>;
	type AccountData = pallet_balances::AccountData<Balance>;
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = ();
	type MaxFreezes = ConstU32<0>;
	type DoneSlashHandler = ();
}

parameter_types! {
	pub const ParachainId: u32 = ASSET_HUB_PARA_ID;
}

impl mock_message_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

parameter_types! {
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;
	pub RelayLocation: Location = Location::parent();
	pub UniversalLocation: InteriorLocation = [
		GlobalConsensus(RelayNetwork::get()),
		Parachain(ASSET_HUB_PARA_ID),
	].into();
	pub UnitWeightCost: Weight = Weight::from_parts(1_000_000, 64 * 1024);
	pub const MaxInstructions: u32 = 100;
}

pub type LocationToAccountId = (
	ParentIsPreset<AccountId>,
	SiblingParachainConvertsVia<polkadot_parachain::primitives::Sibling, AccountId>,
	AccountId32Aliases<RelayNetwork, AccountId>,
);

pub type LocalAssetTransactor =
	FungibleAdapter<Balances, IsConcrete<RelayLocation>, LocationToAccountId, AccountId, ()>;

pub type XcmOriginToCallOrigin = (
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
);

pub type Barrier = (TakeWeightCredit, AllowTopLevelPaidExecutionFrom<Everything>);

pub type XcmRouter = crate::networks::ParachainXcmRouter<Runtime>;

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = XcmOriginToCallOrigin;
	type IsReserve = ();
	type IsTeleporter = ();
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = ();
	type ResponseHandler = ();
	type AssetTrap = ();
	type AssetClaims = ();
	type SubscriptionService = ();
	type PalletInstancesInfo = ();
	type MaxAssetsIntoHolding = ConstU32<64>;
	type AssetLocker = ();
	type AssetExchanger = ();
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
	type Aliasers = Nothing;
	type TransactionalProcessor = FrameTransactionalProcessor;
	type HrmpNewChannelOpenRequestHandler = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type XcmRecorder = ();
	type XcmEventEmitter = ();
}

construct_runtime! {
	pub enum Runtime {
		System: frame_system,
		Balances: pallet_balances,
		MsgQueue: mock_message_queue,
	}
}

/// Create test externalities for Asset Hub
pub fn asset_hub_ext() -> TestExternalities {
	let mut t = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(AccountId32::new([1u8; 32]), 1_000_000_000_000_000),
			(AccountId32::new([2u8; 32]), 1_000_000_000_000_000),
		],
		dev_accounts: None,
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
		// Set parachain ID for mock_message_queue
		mock_message_queue::ParachainId::<Runtime>::set(ASSET_HUB_PARA_ID.into());
	});
	ext
}
