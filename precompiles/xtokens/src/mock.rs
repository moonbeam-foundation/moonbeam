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
use codec::{Decode, Encode, MaxEncodedLen};
use fp_evm::Precompile;
use frame_support::traits::{EnsureOrigin, Everything, OriginTrait, PalletInfo as PalletInfoTrait};
use frame_support::{construct_runtime, parameter_types};
use orml_traits::{location::AbsoluteReserveProvider, parameter_type_with_key};
use pallet_evm::{AddressMapping, EnsureAddressNever, EnsureAddressRoot, PrecompileSet};
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_io;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use xcm::latest::{
	Error as XcmError,
	Junction::{AccountKey20, GeneralIndex, PalletInstance, Parachain},
	Junctions, MultiAsset, MultiLocation, NetworkId, Result as XcmResult, SendResult, SendXcm, Xcm,
};

use scale_info::TypeInfo;

use xcm_builder::{AllowUnpaidExecutionFrom, FixedWeightBounds};

use xcm_executor::{
	traits::{InvertLocation, TransactAsset, WeightTrader},
	Assets, XcmExecutor,
};
use xcm_primitives::XcmV2Weight;

pub type AccountId = TestAccount;
pub type Balance = u128;
pub type BlockNumber = u32;
pub const PRECOMPILE_ADDRESS: u64 = 1;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Evm: pallet_evm::{Pallet, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
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
	TypeInfo,
)]
pub enum TestAccount {
	Alice,
	Bob,
	Charlie,
	AssetId(u128),
	SelfReserve,
	Bogus,
	Precompile,
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
			a if a == H160::repeat_byte(0xDD) => Self::SelfReserve,
			a if a == H160::from_low_u64_be(PRECOMPILE_ADDRESS) => Self::Precompile,
			_ => {
				let mut data = [0u8; 16];
				let (prefix_part, id_part) = h160_account.as_fixed_bytes().split_at(4);
				if prefix_part == &[255u8; 4] {
					data.copy_from_slice(id_part);

					return Self::AssetId(u128::from_be_bytes(data));
				}
				Self::Bogus
			}
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
			TestAccount::Precompile => H160::from_low_u64_be(PRECOMPILE_ADDRESS),
			TestAccount::SelfReserve => H160::repeat_byte(0xDD),
			TestAccount::AssetId(asset_id) => {
				let mut data = [0u8; 20];
				let id_as_bytes = asset_id.to_be_bytes();
				data[0..4].copy_from_slice(&[255u8; 4]);
				data[4..20].copy_from_slice(&id_as_bytes);
				H160::from_slice(&data)
			}
			TestAccount::Bogus => Default::default(),
		}
	}
}

pub type AssetId = u128;

parameter_types! {
	pub ParachainId: cumulus_primitives_core::ParaId = 100.into();
}

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Runtime {
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
	type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
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
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}
parameter_types! {
	pub const ExistentialDeposit: u128 = 0;
}
impl pallet_balances::Config for Runtime {
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

// These parameters dont matter much as this will only be called by root with the forced arguments
// No deposit is substracted with those methods
parameter_types! {
	pub const AssetDeposit: Balance = 0;
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 0;
	pub const MetadataDepositPerByte: Balance = 0;
}

pub struct TestPrecompiles<R>(PhantomData<R>);

impl<R> PrecompileSet for TestPrecompiles<R>
where
	XtokensPrecompile<R>: Precompile,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		match handle.code_address() {
			a if a == precompile_address() => Some(XtokensPrecompile::<R>::execute(handle)),
			_ => None,
		}
	}

	fn is_precompile(&self, address: H160) -> bool {
		address == precompile_address()
	}
}

pub type PCall = XtokensPrecompileCall<Runtime>;

pub fn precompile_address() -> H160 {
	H160::from_low_u64_be(1)
}

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub const PrecompilesValue: TestPrecompiles<Runtime> = TestPrecompiles(PhantomData);
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressRoot<TestAccount>;
	type WithdrawOrigin = EnsureAddressNever<TestAccount>;
	type AddressMapping = TestAccount;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = TestPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}
impl pallet_timestamp::Config for Runtime {
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

	fn buy_weight(&mut self, _weight: XcmV2Weight, _payment: Assets) -> Result<Assets, XcmError> {
		Ok(Assets::default())
	}
}

pub struct InvertNothing;
impl InvertLocation for InvertNothing {
	fn invert_location(_: &MultiLocation) -> sp_std::result::Result<MultiLocation, ()> {
		Ok(MultiLocation::here())
	}

	fn ancestry() -> MultiLocation {
		MultiLocation::here()
	}
}

impl pallet_xcm::Config for Runtime {
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
	type AssetTrap = ();
	type AssetClaims = ();
	type CallDispatcher = Call;
}

#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum CurrencyId {
	SelfReserve,
	OtherReserve(AssetId),
}

impl Into<Option<CurrencyId>> for TestAccount {
	fn into(self) -> Option<CurrencyId> {
		match self {
			TestAccount::SelfReserve => Some(CurrencyId::SelfReserve),
			TestAccount::AssetId(asset_id) => Some(CurrencyId::OtherReserve(asset_id)),
			_ => None,
		}
	}
}

// Implement the trait, where we convert AccountId to AssetID
impl AccountIdToCurrencyId<AccountId, CurrencyId> for Runtime {
	/// The way to convert an account to assetId is by ensuring that the prefix is 0XFFFFFFFF
	/// and by taking the lowest 128 bits as the assetId
	fn account_to_currency_id(account: AccountId) -> Option<CurrencyId> {
		match account {
			TestAccount::AssetId(asset_id) => Some(CurrencyId::OtherReserve(asset_id)),
			TestAccount::SelfReserve => Some(CurrencyId::SelfReserve),
			_ => None,
		}
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
			// To distinguish between relay and others, specially for reserve asset
			CurrencyId::OtherReserve(asset) => {
				if asset == 0 {
					Some(MultiLocation::parent())
				} else {
					Some(MultiLocation::new(
						1,
						Junctions::X2(Parachain(2), GeneralIndex(asset)),
					))
				}
			}
		}
	}
}

pub struct AccountIdToMultiLocation;
impl sp_runtime::traits::Convert<TestAccount, MultiLocation> for AccountIdToMultiLocation {
	fn convert(account: TestAccount) -> MultiLocation {
		let as_h160: H160 = account.into();
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

	pub const BaseXcmWeight: XcmV2Weight = 1000;
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;
	pub const MaxAssetsForTransfer: usize = 2;

	pub SelfLocation: MultiLocation = (1, Junctions::X1(Parachain(ParachainId::get().into()))).into();

	pub SelfReserve: MultiLocation = (
		1,
		Junctions::X2(
			Parachain(ParachainId::get().into()),
			PalletInstance(<Runtime as frame_system::Config>::PalletInfo::index::<Balances>().unwrap() as u8)
		)).into();
	pub MaxInstructions: u32 = 100;
}

parameter_type_with_key! {
	pub ParachainMinFee: |_location: MultiLocation| -> Option<u128> {
		Some(u128::MAX)
	};
}

impl orml_xtokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = AccountIdToMultiLocation;
	type CurrencyIdConvert = CurrencyIdToMultiLocation;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type SelfLocation = SelfLocation;
	type Weigher = xcm_builder::FixedWeightBounds<BaseXcmWeight, Call, MaxInstructions>;
	type BaseXcmWeight = BaseXcmWeight;
	type LocationInverter = InvertNothing;
	type MaxAssetsForTransfer = MaxAssetsForTransfer;
	type MinXcmFee = ParachainMinFee;
	type MultiLocationsFilter = Everything;
	type ReserveProvider = AbsoluteReserveProvider;
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
			.build_storage::<Runtime>()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub(crate) fn events() -> Vec<Event> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.collect::<Vec<_>>()
}
