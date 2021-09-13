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
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	traits::{EnsureOrigin, Everything, OriginTrait, PalletInfo as PalletInfoTrait},
	weights::Weight,
};

use xcm::v0::{Error as XcmError, MultiAsset, MultiLocation, Result as XcmResult, SendXcm, Xcm};

use frame_support::{
	construct_runtime,
	parameter_types,
	traits::{OnFinalize, OnInitialize},
};

use pallet_evm::{AddressMapping, EnsureAddressNever, EnsureAddressRoot, PrecompileSet};
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_io;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use xcm_executor::XcmExecutor;
use xcm::{
	v0::{
		Junction::{PalletInstance, Parachain, Parent},
		NetworkId
	},
};

use xcm_builder::{AllowUnpaidExecutionFrom, FixedWeightBounds};

use xcm_executor::{
	traits::{InvertLocation, TransactAsset, WeightTrader},
	Assets,
};


pub type AccountId = TestAccount;
pub type Balance = u128;
pub type BlockNumber = u64;
pub const PRECOMPILE_ADDRESS: u64 = 1;

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
		Evm: pallet_evm::{Pallet, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>},
		Xtokens: orml_xtokens::{Pallet, Call, Storage, Event<T>},
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
	}
);

// FRom https://github.com/PureStake/moonbeam/pull/518. Merge to common once is merged
#[derive(
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Clone,
	Copy,
	Encode,
	Decode,
	Debug,
	MaxEncodedLen,
	Serialize,
	Deserialize,
	derive_more::Display,
)]
pub enum TestAccount {
	Alice,
	Bob,
	Charlie,
	AssetId,
	Bogus,
}

impl Default for TestAccount {
	fn default() -> Self {
		Self::Bogus
	}
}

impl AddressMapping<TestAccount> for TestAccount {
	fn into_account_id(h160_account: H160) -> TestAccount {
		match h160_account {
			a if a == H160::repeat_byte(0xAA) => Self::Alice,
			a if a == H160::repeat_byte(0xBB) => Self::Bob,
			a if a == H160::repeat_byte(0xCC) => Self::Charlie,
			a if a == H160::repeat_byte(0xDD) => Self::AssetId,
			_ => Self::Bogus,
		}
	}
}

impl From<H160> for TestAccount {
	fn from(x: H160) -> TestAccount {
		TestAccount::into_account_id(x)
	}
}

impl From<TestAccount> for H160 {
	fn from(value: TestAccount) -> H160 {
		match value {
			TestAccount::Alice => H160::repeat_byte(0xAA),
			TestAccount::Bob => H160::repeat_byte(0xBB),
			TestAccount::Charlie => H160::repeat_byte(0xCC),
			TestAccount::AssetId => H160::repeat_byte(0xDD),
			TestAccount::Bogus => Default::default(),
		}
	}
}

pub type AssetId = u128;

parameter_types! {
	pub ParachainId: cumulus_primitives_core::ParaId = 100.into();
}

impl cumulus_pallet_parachain_system::Config for Test {
	type SelfParaId = ParachainId;
	type Event = Event;
	type OnValidationData = ();
	type OutboundXcmpMessageSource = ();
	type XcmpMessageHandler = ();
	type ReservedXcmpWeight = ();
	type DmpMessageHandler = ();
	type ReservedDmpWeight = ();
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = TestAccount;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
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
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
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
pub struct TestPrecompiles<R>(PhantomData<R>);

impl<R> PrecompileSet for TestPrecompiles<R>
where
	R: orml_xtokens::Config,
	R: pallet_evm::Config,
	R::AccountId: From<H160>,
	R::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	R::Call: From<orml_xtokens::Call<R>>,
	<R::Call as Dispatchable>::Origin: From<Option<R::AccountId>>,
	BalanceOf<R>: TryFrom<U256> + Into<U256>,
	<R as orml_xtokens::Config>::CurrencyId: From<R::AccountId>,

{
	fn execute(
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
	) -> Option<Result<PrecompileOutput, ExitError>> {
		match address {
			a if a == hash(PRECOMPILE_ADDRESS) => Some(XtokensWrapper::<R>::execute(
				input, target_gas, context,
			)),
			_ => None,
		}
	}
}

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}

pub type Precompiles = TestPrecompiles<Test>;

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressRoot<TestAccount>;
	type WithdrawOrigin = EnsureAddressNever<TestAccount>;
	type AddressMapping = TestAccount;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type Precompiles = Precompiles;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = ();
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
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
		Ok(MultiLocation::Null)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> Origin {
		Origin::root()
	}
}

pub struct DoNothingRouter;
impl SendXcm for DoNothingRouter {
	fn send_xcm(_dest: MultiLocation, _msg: Xcm<()>) -> XcmResult {
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
	fn invert_location(_: &MultiLocation) -> MultiLocation {
		MultiLocation::Null
	}
}

pub type LocalOriginToLocation = xcm_builder::SignedAccountKey20AsNative<RelayNetwork, Origin>;

impl pallet_xcm::Config for Test {
	// The config types here are entirely configurable, since the only one that is sorely needed
	// is `XcmExecutor`, which will be used in unit tests located in xcm-executor.
	type Event = Event;
	type ExecuteXcmOrigin = ConvertOriginToLocal;
	type LocationInverter = InvertNothing;
	type SendXcmOrigin = ConvertOriginToLocal;
	type Weigher = xcm_builder::FixedWeightBounds<BaseXcmWeight, Call>;
	type XcmRouter = DoNothingRouter;
	type XcmExecuteFilter = frame_support::traits::Everything;
	type XcmExecutor = xcm_executor::XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = frame_support::traits::Everything;
	type XcmReserveTransferFilter = frame_support::traits::Everything;
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
	type Weigher = FixedWeightBounds<BaseXcmWeight, Call>;
	type Trader = DummyWeightTrader;
	type ResponseHandler = ();
}
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode)]
pub enum CurrencyId {
	SelfReserve,
	OtherReserve(AssetId),
}

impl From<TestAccount> for CurrencyId {
	fn from(address: TestAccount) -> Self {
		CurrencyId::SelfReserve
	}
}

pub struct CurrencyIdToMultiLocation;

impl sp_runtime::traits::Convert<CurrencyId, Option<MultiLocation>> for CurrencyIdToMultiLocation {
	fn convert(currency: CurrencyId) -> Option<MultiLocation> {
		match currency {
			CurrencyId::SelfReserve => {
				let multi: MultiLocation = SelfReserve::get();
				Some(multi)
			}
			CurrencyId::OtherReserve(asset) => {
				Some(MultiLocation::X3(Parent, Parachain(2), PalletInstance(1)))
			}
		}
	}
}

pub struct AccountIdToMultiLocation;
impl sp_runtime::traits::Convert<TestAccount, MultiLocation>
	for AccountIdToMultiLocation
{
	fn convert(account: TestAccount) -> MultiLocation {
		let as_h160: H160 = account.into();
		MultiLocation::X1(Junction::AccountKey20 {
			network: NetworkId::Any,
			key: as_h160.as_fixed_bytes().clone(),
		})
	}
}

parameter_types! {
	pub Ancestry: MultiLocation = Parachain(ParachainId::get().into()).into();

	pub const BaseXcmWeight: Weight = 0;
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;

	pub SelfLocation: MultiLocation = MultiLocation::X2(Parent,  Parachain(ParachainId::get().into()).into());
	pub SelfReserve: MultiLocation = MultiLocation::X3(Parent, Parachain(ParachainId::get().into()).into(), PalletInstance(<Test as frame_system::Config>::PalletInfo::index::<Balances>().unwrap() as u8));
}

impl orml_xtokens::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = AccountIdToMultiLocation;
	type CurrencyIdConvert = CurrencyIdToMultiLocation;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type SelfLocation = SelfLocation;
	type Weigher = xcm_builder::FixedWeightBounds<BaseXcmWeight, Call>;
	type BaseXcmWeight = BaseXcmWeight;
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

//TODO Add pallets here if necessary
pub(crate) fn roll_to(n: u64) {
	while System::block_number() < n {
		Balances::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Balances::on_initialize(System::block_number());
	}
}

pub(crate) fn events() -> Vec<Event> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.collect::<Vec<_>>()
}

// Helper function to give a simple evm context suitable for tests.
// We can remove this once https://github.com/rust-blockchain/evm/pull/35
// is in our dependency graph.
pub fn evm_test_context() -> evm::Context {
	evm::Context {
		address: Default::default(),
		caller: Default::default(),
		apparent_value: From::from(0),
	}
}
