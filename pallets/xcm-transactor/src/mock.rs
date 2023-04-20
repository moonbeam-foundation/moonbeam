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

//! Test utilities

use super::*;
use crate as pallet_xcm_transactor;
use cumulus_primitives_core::MultiAssets;
use frame_support::traits::PalletInfo as PalletInfoTrait;
use frame_support::{construct_runtime, parameter_types, weights::Weight};
use frame_system::EnsureRoot;
use parity_scale_codec::{Decode, Encode};

use sp_core::{H160, H256};
use sp_io;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use xcm::latest::{
	opaque, Error as XcmError, Instruction, InteriorMultiLocation,
	Junction::{AccountKey20, PalletInstance, Parachain},
	Junctions, MultiAsset, MultiLocation, NetworkId, Result as XcmResult, SendError, SendResult,
	SendXcm, Xcm, XcmContext, XcmHash,
};
use xcm_primitives::{
	HrmpAvailableCalls, HrmpEncodeCall, UtilityAvailableCalls, UtilityEncodeCall, XcmTransact,
};

use sp_std::cell::RefCell;
use xcm_executor::{
	traits::{TransactAsset, WeightBounds, WeightTrader},
	Assets,
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
		XcmTransactor: pallet_xcm_transactor::{Pallet, Call, Event<T>},
	}
);

pub type Balance = u128;
pub type BlockNumber = u32;
pub type AccountId = u64;

parameter_types! {
	pub ParachainId: cumulus_primitives_core::ParaId = 100.into();
}
parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(Weight::from_ref_time(1024));
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Nothing;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type RuntimeCall = RuntimeCall;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
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
pub struct DoNothingRouter;
impl SendXcm for DoNothingRouter {
	type Ticket = ();

	fn validate(
		_destination: &mut Option<MultiLocation>,
		_message: &mut Option<opaque::Xcm>,
	) -> SendResult<Self::Ticket> {
		Ok(((), MultiAssets::new()))
	}

	fn deliver(_: Self::Ticket) -> Result<XcmHash, SendError> {
		Ok(XcmHash::default())
	}
}

pub struct DummyAssetTransactor;
impl TransactAsset for DummyAssetTransactor {
	fn deposit_asset(_what: &MultiAsset, _who: &MultiLocation, _context: &XcmContext) -> XcmResult {
		Ok(())
	}

	fn withdraw_asset(
		_what: &MultiAsset,
		_who: &MultiLocation,
		_context: Option<&XcmContext>,
	) -> Result<Assets, XcmError> {
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

pub struct AccountIdToMultiLocation;
impl sp_runtime::traits::Convert<u64, MultiLocation> for AccountIdToMultiLocation {
	fn convert(_account: u64) -> MultiLocation {
		let as_h160: H160 = H160::repeat_byte(0xAA);
		MultiLocation::new(
			0,
			Junctions::X1(AccountKey20 {
				network: None,
				key: as_h160.as_fixed_bytes().clone(),
			}),
		)
	}
}

parameter_types! {
	pub Ancestry: MultiLocation = Parachain(ParachainId::get().into()).into();

	pub const BaseXcmWeight: Weight = Weight::from_parts(1000u64, 1000u64);
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;

	pub SelfLocation: MultiLocation =
		MultiLocation::new(1, Junctions::X1(Parachain(ParachainId::get().into())));

	pub SelfReserve: MultiLocation = MultiLocation::new(
		1,
		Junctions::X2(
			Parachain(ParachainId::get().into()),
			PalletInstance(
				<Test as frame_system::Config>::PalletInfo::index::<Balances>().unwrap() as u8
			)
		));
	pub MaxInstructions: u32 = 100;

	pub UniversalLocation: InteriorMultiLocation = RelayNetwork::get().into();
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
	fn destination(self) -> MultiLocation {
		match self {
			Transactors::Relay => MultiLocation::parent(),
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

pub struct MockHrmpEncoder;

impl HrmpEncodeCall for MockHrmpEncoder {
	fn hrmp_encode_call(call: HrmpAvailableCalls) -> Result<Vec<u8>, XcmError> {
		match call {
			HrmpAvailableCalls::InitOpenChannel(_, _, _) => {
				Ok(RelayCall::Hrmp(HrmpCall::Init()).encode())
			}
			HrmpAvailableCalls::AcceptOpenChannel(_) => {
				Ok(RelayCall::Hrmp(HrmpCall::Accept()).encode())
			}
			HrmpAvailableCalls::CloseChannel(_) => Ok(RelayCall::Hrmp(HrmpCall::Close()).encode()),
			HrmpAvailableCalls::CancelOpenRequest(_, _) => {
				Ok(RelayCall::Hrmp(HrmpCall::Cancel()).encode())
			}
		}
	}
}

pub type AssetId = u128;
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, scale_info::TypeInfo)]
pub enum CurrencyId {
	SelfReserve,
	OtherReserve(AssetId),
}

pub struct CurrencyIdToMultiLocation;

impl sp_runtime::traits::Convert<CurrencyId, Option<MultiLocation>> for CurrencyIdToMultiLocation {
	fn convert(currency: CurrencyId) -> Option<MultiLocation> {
		match currency {
			CurrencyId::SelfReserve => {
				let multi: MultiLocation = SelfReserve::get();
				Some(multi)
			}
			// To distinguish between relay and others, specially for reserve asset
			CurrencyId::OtherReserve(asset) => {
				if asset == 0 {
					Some(MultiLocation::parent())
				} else {
					Some(MultiLocation::new(1, Junctions::X1(Parachain(2))))
				}
			}
		}
	}
}

#[cfg(feature = "runtime-benchmarks")]
impl From<MultiLocation> for CurrencyId {
	fn from(location: MultiLocation) -> CurrencyId {
		if location == SelfReserve::get() {
			CurrencyId::SelfReserve
		} else if location == MultiLocation::parent() {
			CurrencyId::OtherReserve(0)
		} else {
			CurrencyId::OtherReserve(1)
		}
	}
}

// Simulates sending a XCM message
thread_local! {
	pub static SENT_XCM: RefCell<Vec<(MultiLocation, opaque::Xcm)>> = RefCell::new(Vec::new());
}
pub fn sent_xcm() -> Vec<(MultiLocation, opaque::Xcm)> {
	SENT_XCM.with(|q| (*q.borrow()).clone())
}
pub struct TestSendXcm;
impl SendXcm for TestSendXcm {
	type Ticket = ();

	fn validate(
		destination: &mut Option<MultiLocation>,
		message: &mut Option<opaque::Xcm>,
	) -> SendResult<Self::Ticket> {
		SENT_XCM.with(|q| {
			q.borrow_mut()
				.push((destination.clone().unwrap(), message.clone().unwrap()))
		});
		Ok(((), MultiAssets::new()))
	}

	fn deliver(_: Self::Ticket) -> Result<XcmHash, SendError> {
		Ok(XcmHash::default())
	}
}

parameter_types! {
	pub MaxFee: MultiAsset = (MultiLocation::parent(), 1_000_000_000_000u128).into();
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
	type CurrencyIdToMultiLocation = CurrencyIdToMultiLocation;
	type AccountIdToMultiLocation = AccountIdToMultiLocation;
	type SelfLocation = SelfLocation;
	type Weigher = DummyWeigher<RuntimeCall>;
	type UniversalLocation = UniversalLocation;
	type BaseXcmWeight = BaseXcmWeight;
	type XcmSender = TestSendXcm;
	type ReserveProvider = orml_traits::location::RelativeReserveProvider;
	type WeightInfo = ();
	type HrmpManipulatorOrigin = EnsureRoot<u64>;
	type MaxHrmpFee = MaxHrmpRelayFee;
	type HrmpEncoder = MockHrmpEncoder;
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
			if let RuntimeEvent::XcmTransactor(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}
