// Copyright 2019-2025 PureStake Inc.
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
use crate as pallet_xcm_transactor;
use cumulus_primitives_core::Assets;
use frame_support::traits::PalletInfo as PalletInfoTrait;
use frame_support::{construct_runtime, parameter_types, weights::Weight};
use frame_system::EnsureRoot;
use parity_scale_codec::{Decode, Encode};

use sp_core::{H160, H256};
use sp_io;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_runtime::BuildStorage;
use xcm::latest::{
	opaque, Asset, Error as XcmError, Instruction, InteriorLocation,
	Junction::{AccountKey20, GlobalConsensus, PalletInstance, Parachain},
	Location, NetworkId, Result as XcmResult, SendError, SendResult, SendXcm, Xcm, XcmContext,
	XcmHash,
};
use xcm::{IntoVersion, VersionedXcm, WrapVersion};
use xcm_primitives::{UtilityAvailableCalls, UtilityEncodeCall, XcmTransact};

use sp_std::cell::RefCell;
use xcm_executor::{
	traits::{TransactAsset, WeightBounds, WeightTrader},
	AssetsInHolding,
};
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Timestamp: pallet_timestamp,
		XcmTransactor: pallet_xcm_transactor,
	}
);

pub type Balance = u128;
pub type AccountId = u64;

parameter_types! {
	pub ParachainId: cumulus_primitives_core::ParaId = 100.into();
}
parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(Weight::from_parts(1024, 1));
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Nothing;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type RuntimeCall = RuntimeCall;
	type RuntimeTask = RuntimeTask;
	type Block = Block;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
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
	pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

const XCM_VERSION_ROOT_KEY: &'static [u8] = b"XCM_VERSION_ROOT_KEY";

pub struct CustomVersionWrapper;
impl WrapVersion for CustomVersionWrapper {
	fn wrap_version<RuntimeCall>(
		_dest: &xcm::latest::Location,
		xcm: impl Into<VersionedXcm<RuntimeCall>>,
	) -> Result<VersionedXcm<RuntimeCall>, ()> {
		let xcm_version: u32 =
			frame_support::storage::unhashed::get(XCM_VERSION_ROOT_KEY).unwrap_or(3);
		let xcm_converted = xcm.into().into_version(xcm_version)?;
		Ok(xcm_converted)
	}
}

impl CustomVersionWrapper {
	pub fn set_version(version: u32) {
		frame_support::storage::unhashed::put(XCM_VERSION_ROOT_KEY, &version);
	}
}

pub struct DoNothingRouter;
impl SendXcm for DoNothingRouter {
	type Ticket = ();

	fn validate(
		_destination: &mut Option<Location>,
		_message: &mut Option<opaque::Xcm>,
	) -> SendResult<Self::Ticket> {
		Ok(((), Assets::new()))
	}

	fn deliver(_: Self::Ticket) -> Result<XcmHash, SendError> {
		Ok(XcmHash::default())
	}
}

pub struct DummyAssetTransactor;
impl TransactAsset for DummyAssetTransactor {
	fn deposit_asset(_what: &Asset, _who: &Location, _context: Option<&XcmContext>) -> XcmResult {
		Ok(())
	}

	fn withdraw_asset(
		_what: &Asset,
		_who: &Location,
		_context: Option<&XcmContext>,
	) -> Result<AssetsInHolding, XcmError> {
		Ok(AssetsInHolding::default())
	}
}

pub struct DummyWeightTrader;
impl WeightTrader for DummyWeightTrader {
	fn new() -> Self {
		DummyWeightTrader
	}

	fn buy_weight(
		&mut self,
		_weight: Weight,
		_payment: AssetsInHolding,
		_context: &XcmContext,
	) -> Result<AssetsInHolding, XcmError> {
		Ok(AssetsInHolding::default())
	}
}

use sp_std::marker::PhantomData;
pub struct DummyWeigher<C>(PhantomData<C>);

impl<C: Decode> WeightBounds<C> for DummyWeigher<C> {
	fn weight(_message: &mut Xcm<C>) -> Result<Weight, ()> {
		Ok(Weight::zero())
	}
	fn instr_weight(_instruction: &Instruction<C>) -> Result<Weight, ()> {
		Ok(Weight::zero())
	}
}

pub struct AccountIdToLocation;
impl sp_runtime::traits::Convert<u64, Location> for AccountIdToLocation {
	fn convert(_account: u64) -> Location {
		let as_h160: H160 = H160::repeat_byte(0xAA);
		Location::new(
			0,
			[AccountKey20 {
				network: None,
				key: as_h160.as_fixed_bytes().clone(),
			}],
		)
	}
}

parameter_types! {
	pub Ancestry: Location = Parachain(ParachainId::get().into()).into();

	pub const BaseXcmWeight: Weight = Weight::from_parts(1000u64, 1000u64);
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;

	pub SelfLocation: Location = Location::here();

	pub SelfReserve: Location = Location::new(
		1,
		[
			Parachain(ParachainId::get().into()),
			PalletInstance(
				<Test as frame_system::Config>::PalletInfo::index::<Balances>().unwrap() as u8
			)
		]);
	pub MaxInstructions: u32 = 100;

	pub UniversalLocation: InteriorLocation =
		[GlobalConsensus(RelayNetwork::get()), Parachain(ParachainId::get().into())].into();
}

#[derive(Encode, Decode)]
pub enum RelayCall {
	#[codec(index = 0u8)]
	// the index should match the position of the module in `construct_runtime!`
	Utility(UtilityCall),
	#[codec(index = 1u8)]
	// the index should match the position of the module in `construct_runtime!`
	Hrmp(HrmpCall),
}

#[derive(Encode, Decode)]
pub enum UtilityCall {
	#[codec(index = 0u8)]
	AsDerivative(u16),
}

#[derive(Encode, Decode)]
pub enum HrmpCall {
	#[codec(index = 0u8)]
	Init(),
	#[codec(index = 1u8)]
	Accept(),
	#[codec(index = 2u8)]
	Close(),
	#[codec(index = 6u8)]
	Cancel(),
}

// Transactors for the mock runtime. Only relay chain
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, scale_info::TypeInfo)]
pub enum Transactors {
	Relay,
}

#[cfg(feature = "runtime-benchmarks")]
impl Default for Transactors {
	fn default() -> Self {
		Transactors::Relay
	}
}

impl XcmTransact for Transactors {
	fn destination(self) -> Location {
		match self {
			Transactors::Relay => Location::parent(),
		}
	}
}

impl UtilityEncodeCall for Transactors {
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

pub type AssetId = u128;
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, scale_info::TypeInfo)]
pub enum CurrencyId {
	SelfReserve,
	OtherReserve(AssetId),
}

pub struct CurrencyIdToLocation;

impl sp_runtime::traits::Convert<CurrencyId, Option<Location>> for CurrencyIdToLocation {
	fn convert(currency: CurrencyId) -> Option<Location> {
		match currency {
			CurrencyId::SelfReserve => {
				let multi: Location = SelfReserve::get();
				Some(multi)
			}
			// To distinguish between relay and others, specially for reserve asset
			CurrencyId::OtherReserve(asset) => {
				if asset == 0 {
					Some(Location::parent())
				} else {
					Some(Location::new(1, [Parachain(2)]))
				}
			}
		}
	}
}

#[cfg(feature = "runtime-benchmarks")]
impl From<Location> for CurrencyId {
	fn from(location: Location) -> CurrencyId {
		if location == SelfReserve::get() {
			CurrencyId::SelfReserve
		} else if location == Location::parent() {
			CurrencyId::OtherReserve(0)
		} else {
			CurrencyId::OtherReserve(1)
		}
	}
}

// Simulates sending a XCM message
thread_local! {
	pub static SENT_XCM: RefCell<Vec<(Location, opaque::Xcm)>> = RefCell::new(Vec::new());
}
pub fn sent_xcm() -> Vec<(Location, opaque::Xcm)> {
	SENT_XCM.with(|q| (*q.borrow()).clone())
}
pub struct TestSendXcm;
impl SendXcm for TestSendXcm {
	type Ticket = ();

	fn validate(
		destination: &mut Option<Location>,
		message: &mut Option<opaque::Xcm>,
	) -> SendResult<Self::Ticket> {
		SENT_XCM.with(|q| {
			q.borrow_mut()
				.push((destination.clone().unwrap(), message.clone().unwrap()))
		});
		CustomVersionWrapper::wrap_version(&destination.clone().unwrap(), message.clone().unwrap())
			.map_err(|()| SendError::DestinationUnsupported)?;
		Ok(((), Assets::new()))
	}

	fn deliver(_: Self::Ticket) -> Result<XcmHash, SendError> {
		Ok(XcmHash::default())
	}
}

parameter_types! {
	pub MaxFee: Asset = (Location::parent(), 1_000_000_000_000u128).into();
	pub SelfLocationAbsolute: Location = Location {
		parents: 1,
		interior: [Parachain(ParachainId::get().into())].into(),
	};
}
pub type MaxHrmpRelayFee = xcm_builder::Case<MaxFee>;

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Transactor = Transactors;
	type DerivativeAddressRegistrationOrigin = EnsureRoot<u64>;
	type SovereignAccountDispatcherOrigin = EnsureRoot<u64>;
	type AssetTransactor = DummyAssetTransactor;
	type CurrencyId = CurrencyId;
	type CurrencyIdToLocation = CurrencyIdToLocation;
	type AccountIdToLocation = AccountIdToLocation;
	type SelfLocation = SelfLocation;
	type Weigher = DummyWeigher<RuntimeCall>;
	type UniversalLocation = UniversalLocation;
	type BaseXcmWeight = BaseXcmWeight;
	type XcmSender = TestSendXcm;
	type ReserveProvider = xcm_primitives::AbsoluteAndRelativeReserve<SelfLocationAbsolute>;
	type WeightInfo = ();
	type HrmpManipulatorOrigin = EnsureRoot<u64>;
	type HrmpOpenOrigin = EnsureRoot<u64>;
	type MaxHrmpFee = MaxHrmpRelayFee;
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
			if let RuntimeEvent::XcmTransactor(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}
