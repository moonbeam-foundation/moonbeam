// Copyright 2019-2021 PureStake Inc.
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

//! Test utilities
use super::*;
use crate::{self as xcm_transactor};
use frame_support::{construct_runtime, parameter_types};
use frame_support::{
	traits::{EnsureOrigin, Everything, OriginTrait, PalletInfo as PalletInfoTrait},
	weights::Weight,
};
use frame_system::EnsureRoot;
use parity_scale_codec::{Decode, Encode};

use sp_core::{H160, H256};
use sp_io;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use xcm::latest::{
	Error as XcmError,
	Junction::{AccountKey20, PalletInstance, Parachain},
	Junctions, MultiAsset, MultiLocation, NetworkId, Result as XcmResult, SendResult, SendXcm, Xcm,
};

use xcm_builder::{AllowUnpaidExecutionFrom, FixedWeightBounds};

use xcm_executor::{
	traits::{InvertLocation, TransactAsset, WeightTrader},
	Assets, XcmExecutor,
};

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
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		XcmTransactor: xcm_transactor::{Pallet, Call, Event<T>},
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
	}
);

pub type Balance = u128;

pub type AccountId = u64;

parameter_types! {
	pub ParachainId: cumulus_primitives_core::ParaId = 100.into();
}
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Nothing;
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Index = u64;
	type Call = Call;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type OnSetCode = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}
parameter_types! {
	pub const ExistentialDeposit: u128 = 0;
}
impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type MaxLocks = ();
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}
pub struct ConvertOriginToLocal;
impl<Origin: OriginTrait> EnsureOrigin<Origin> for ConvertOriginToLocal {
	type Success = MultiLocation;

	fn try_origin(_: Origin) -> Result<MultiLocation, Origin> {
		Ok(MultiLocation::here())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> Origin {
		Origin::root()
	}
}
pub struct DoNothingRouter;
impl SendXcm for DoNothingRouter {
	fn send_xcm(_dest: impl Into<MultiLocation>, _msg: Xcm<()>) -> SendResult {
		Ok(())
	}
}

pub type Barrier = AllowUnpaidExecutionFrom<Everything>;

pub struct DummyAssetTransactor;
impl TransactAsset for DummyAssetTransactor {
	fn deposit_asset(_what: &MultiAsset, _who: &MultiLocation) -> XcmResult {
		Ok(())
	}

	fn withdraw_asset(_what: &MultiAsset, _who: &MultiLocation) -> Result<Assets, XcmError> {
		Ok(Assets::default())
	}
}

pub struct DummyWeightTrader;
impl WeightTrader for DummyWeightTrader {
	fn new() -> Self {
		DummyWeightTrader
	}

	fn buy_weight(&mut self, _weight: Weight, _payment: Assets) -> Result<Assets, XcmError> {
		Ok(Assets::default())
	}
}
pub struct InvertNothing;
impl InvertLocation for InvertNothing {
	fn invert_location(_: &MultiLocation) -> sp_std::result::Result<MultiLocation, ()> {
		Ok(MultiLocation::here())
	}
}

impl pallet_xcm::Config for Test {
	// The config types here are entirely configurable, since the only one that is sorely needed
	// is `XcmExecutor`, which will be used in unit tests located in xcm-executor.
	type Event = Event;
	type ExecuteXcmOrigin = ConvertOriginToLocal;
	type LocationInverter = InvertNothing;
	type SendXcmOrigin = ConvertOriginToLocal;
	type Weigher = xcm_builder::FixedWeightBounds<BaseXcmWeight, Call, MaxInstructions>;
	type XcmRouter = DoNothingRouter;
	type XcmExecuteFilter = frame_support::traits::Everything;
	type XcmExecutor = xcm_executor::XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = frame_support::traits::Everything;
	type XcmReserveTransferFilter = frame_support::traits::Everything;
	type Origin = Origin;
	type Call = Call;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type Call = Call;
	type XcmSender = DoNothingRouter;
	type AssetTransactor = DummyAssetTransactor;
	type OriginConverter = pallet_xcm::XcmPassthrough<Origin>;
	type IsReserve = ();
	type IsTeleporter = ();
	type LocationInverter = InvertNothing;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<BaseXcmWeight, Call, MaxInstructions>;
	type Trader = DummyWeightTrader;
	type ResponseHandler = ();
	type SubscriptionService = ();
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
}

pub struct AccountIdToMultiLocation;
impl sp_runtime::traits::Convert<u64, MultiLocation> for AccountIdToMultiLocation {
	fn convert(_account: u64) -> MultiLocation {
		let as_h160: H160 = H160::repeat_byte(0xAA);
		MultiLocation::new(
			1,
			Junctions::X1(AccountKey20 {
				network: NetworkId::Any,
				key: as_h160.as_fixed_bytes().clone(),
			}),
		)
	}
}

parameter_types! {
	pub Ancestry: MultiLocation = Parachain(ParachainId::get().into()).into();

	pub const BaseXcmWeight: Weight = 1000;
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;

	pub SelfLocation: MultiLocation = (1, Junctions::X1(Parachain(ParachainId::get().into()))).into();

	pub SelfReserve: MultiLocation = (
		1,
		Junctions::X2(
			Parachain(ParachainId::get().into()),
			PalletInstance(<Test as frame_system::Config>::PalletInfo::index::<Balances>().unwrap() as u8)
		)).into();
	pub MaxInstructions: u32 = 100;
}

#[derive(Encode, Decode)]
pub enum RelayCall {
	#[codec(index = 0u8)]
	// the index should match the position of the module in `construct_runtime!`
	Utility(UtilityCall),
}

#[derive(Encode, Decode)]
pub enum UtilityCall {
	#[codec(index = 0u8)]
	AsDerivative(u16),
}

// Transactors for the mock runtime. Only relay chain
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, scale_info::TypeInfo)]
pub enum Transactors {
	Relay,
}

impl crate::XcmTransact for Transactors {
	fn destination(self) -> MultiLocation {
		match self {
			Transactors::Relay => MultiLocation::parent(),
		}
	}
}

impl crate::UtilityEncodeCall for Transactors {
	fn encode_call(self, call: UtilityAvailableCalls) -> Vec<u8> {
		match self {
			Transactors::Relay => match call {
				UtilityAvailableCalls::AsDerivative(a, b) => {
					let mut call =
						RelayCall::Utility(UtilityCall::AsDerivative(a.clone())).encode();
					call.append(&mut b.clone());
					call
				}
			},
		}
	}
}

impl Config for Test {
	type Event = Event;
	type Balance = Balance;
	type Transactor = Transactors;
	type DerivativeAddressRegistrationOrigin = EnsureRoot<u64>;
	type SovereignAccountDispatcherOrigin = EnsureRoot<u64>;
	type AccountIdToMultiLocation = AccountIdToMultiLocation;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type SelfLocation = SelfLocation;
	type Weigher = xcm_builder::FixedWeightBounds<BaseXcmWeight, Call, MaxInstructions>;
	type LocationInverter = InvertNothing;
	type BaseXcmWeight = BaseXcmWeight;
	type XcmSender = DoNothingRouter;
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
			if let Event::XcmTransactor(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}
