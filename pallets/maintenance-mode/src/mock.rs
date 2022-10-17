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

//! A minimal runtime including the maintenance-mode pallet
use super::*;
use crate as pallet_maintenance_mode;
use cumulus_primitives_core::{relay_chain::BlockNumber as RelayBlockNumber, DmpMessageHandler};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{
		Contains, Everything, GenesisBuild, OffchainWorker, OnFinalize, OnIdle, OnInitialize,
		OnRuntimeUpgrade,
	},
	weights::Weight,
};
use frame_system::EnsureRoot;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

//TODO use TestAccount once it is in a common place (currently it lives with democracy precompiles)
pub type AccountId = u64;
pub type BlockNumber = u32;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		MaintenanceMode: pallet_maintenance_mode::{Pallet, Call, Storage, Event, Config},
		MockPalletMaintenanceHooks: mock_pallet_maintenance_hooks::{Pallet, Call, Event},
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const MaximumBlockWeight: Weight = Weight::from_ref_time(1024);
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Test {
	type BaseCallFilter = MaintenanceMode;
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

/// During maintenance mode we will not allow any calls.
pub struct MaintenanceCallFilter;
impl Contains<Call> for MaintenanceCallFilter {
	fn contains(_: &Call) -> bool {
		false
	}
}

pub struct MaintenanceDmpHandler;
#[cfg(feature = "xcm-support")]
impl DmpMessageHandler for MaintenanceDmpHandler {
	// This implementation makes messages be queued
	// Since the limit is 0, messages are queued for next iteration
	fn handle_dmp_messages(
		_iter: impl Iterator<Item = (RelayBlockNumber, Vec<u8>)>,
		_limit: Weight,
	) -> Weight {
		return Weight::from_ref_time(1);
	}
}

pub struct NormalDmpHandler;
#[cfg(feature = "xcm-support")]
impl DmpMessageHandler for NormalDmpHandler {
	// This implementation makes messages be queued
	// Since the limit is 0, messages are queued for next iteration
	fn handle_dmp_messages(
		_iter: impl Iterator<Item = (RelayBlockNumber, Vec<u8>)>,
		_limit: Weight,
	) -> Weight {
		Weight::zero()
	}
}

impl mock_pallet_maintenance_hooks::Config for Test {
	type Event = Event;
}

// Pallet to throw events, used to test maintenance mode hooks
#[frame_support::pallet]
pub mod mock_pallet_maintenance_hooks {
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event {
		MaintenanceOnIdle,
		MaintenanceOnInitialize,
		MaintenanceOffchainWorker,
		MaintenanceOnFinalize,
		MaintenanceOnRuntimeUpgrade,
		NormalOnIdle,
		NormalOnInitialize,
		NormalOffchainWorker,
		NormalOnFinalize,
		NormalOnRuntimeUpgrade,
	}
}

pub struct MaintenanceHooks;

impl OnInitialize<BlockNumber> for MaintenanceHooks {
	fn on_initialize(_n: BlockNumber) -> Weight {
		MockPalletMaintenanceHooks::deposit_event(
			mock_pallet_maintenance_hooks::Event::MaintenanceOnInitialize,
		);
		Weight::from_ref_time(1)
	}
}

impl OnIdle<BlockNumber> for MaintenanceHooks {
	fn on_idle(_n: BlockNumber, _max_weight: Weight) -> Weight {
		MockPalletMaintenanceHooks::deposit_event(
			mock_pallet_maintenance_hooks::Event::MaintenanceOnIdle,
		);
		Weight::from_ref_time(1)
	}
}

impl OnRuntimeUpgrade for MaintenanceHooks {
	fn on_runtime_upgrade() -> Weight {
		MockPalletMaintenanceHooks::deposit_event(
			mock_pallet_maintenance_hooks::Event::MaintenanceOnRuntimeUpgrade,
		);
		Weight::from_ref_time(1)
	}
}

impl OnFinalize<BlockNumber> for MaintenanceHooks {
	fn on_finalize(_n: BlockNumber) {
		MockPalletMaintenanceHooks::deposit_event(
			mock_pallet_maintenance_hooks::Event::MaintenanceOnFinalize,
		);
	}
}

impl OffchainWorker<BlockNumber> for MaintenanceHooks {
	fn offchain_worker(_n: BlockNumber) {
		MockPalletMaintenanceHooks::deposit_event(
			mock_pallet_maintenance_hooks::Event::MaintenanceOffchainWorker,
		);
	}
}

pub struct NormalHooks;

impl OnInitialize<BlockNumber> for NormalHooks {
	fn on_initialize(_n: BlockNumber) -> Weight {
		MockPalletMaintenanceHooks::deposit_event(
			mock_pallet_maintenance_hooks::Event::NormalOnInitialize,
		);
		Weight::zero()
	}
}

impl OnIdle<BlockNumber> for NormalHooks {
	fn on_idle(_n: BlockNumber, _max_weight: Weight) -> Weight {
		MockPalletMaintenanceHooks::deposit_event(
			mock_pallet_maintenance_hooks::Event::NormalOnIdle,
		);
		Weight::zero()
	}
}

impl OnRuntimeUpgrade for NormalHooks {
	fn on_runtime_upgrade() -> Weight {
		MockPalletMaintenanceHooks::deposit_event(
			mock_pallet_maintenance_hooks::Event::NormalOnRuntimeUpgrade,
		);
		Weight::zero()
	}
}

impl OnFinalize<BlockNumber> for NormalHooks {
	fn on_finalize(_n: BlockNumber) {
		MockPalletMaintenanceHooks::deposit_event(
			mock_pallet_maintenance_hooks::Event::NormalOnFinalize,
		);
	}
}

impl OffchainWorker<BlockNumber> for NormalHooks {
	fn offchain_worker(_n: BlockNumber) {
		MockPalletMaintenanceHooks::deposit_event(
			mock_pallet_maintenance_hooks::Event::NormalOffchainWorker,
		);
	}
}

impl Config for Test {
	type Event = Event;
	type NormalCallFilter = Everything;
	type MaintenanceCallFilter = MaintenanceCallFilter;
	type MaintenanceOrigin = EnsureRoot<AccountId>;
	#[cfg(feature = "xcm-support")]
	type XcmExecutionManager = ();
	#[cfg(feature = "xcm-support")]
	type NormalDmpHandler = NormalDmpHandler;
	#[cfg(feature = "xcm-support")]
	type MaintenanceDmpHandler = MaintenanceDmpHandler;
	type NormalExecutiveHooks = NormalHooks;
	type MaintenanceExecutiveHooks = MaintenanceHooks;
}

/// Externality builder for pallet maintenance mode's mock runtime
pub(crate) struct ExtBuilder {
	maintenance_mode: bool,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			maintenance_mode: false,
		}
	}
}

impl ExtBuilder {
	pub(crate) fn with_maintenance_mode(mut self, m: bool) -> Self {
		self.maintenance_mode = m;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.expect("Frame system builds valid default genesis config");

		GenesisBuild::<Test>::assimilate_storage(
			&pallet_maintenance_mode::GenesisConfig {
				start_in_maintenance_mode: self.maintenance_mode,
			},
			&mut t,
		)
		.expect("Pallet maintenance mode storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub(crate) fn events() -> Vec<pallet_maintenance_mode::Event> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let Event::MaintenanceMode(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}

pub(crate) fn mock_events() -> Vec<mock_pallet_maintenance_hooks::Event> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let Event::MockPalletMaintenanceHooks(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}
