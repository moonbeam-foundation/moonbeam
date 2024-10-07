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
use cumulus_primitives_core::{relay_chain::HrmpChannelId, ParaId};
use frame_support::traits::{
	ConstU32, EnsureOrigin, Everything, Nothing, OriginTrait, PalletInfo as PalletInfoTrait,
};
use frame_support::{construct_runtime, parameter_types, weights::Weight};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use parity_scale_codec::{Decode, Encode};
use precompile_utils::{
	mock_account,
	precompile_set::*,
	testing::{AddressInPrefixedSet, MockAccount},
};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_io;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_runtime::BuildStorage;
use xcm::latest::{prelude::*, Error as XcmError};
use xcm_builder::{AllowUnpaidExecutionFrom, FixedWeightBounds, IsConcrete};
use xcm_executor::{
	traits::{TransactAsset, WeightTrader},
	AssetsInHolding,
};

pub type AccountId = MockAccount;
pub type Balance = u128;
pub type AssetId = u128;

type Block = frame_system::mocking::MockBlockU32<Runtime>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Balances: pallet_balances,
		Evm: pallet_evm,
		Timestamp: pallet_timestamp,
		PolkadotXcm: pallet_xcm,
		XcmTransactor: pallet_xcm_transactor,
	}
);

mock_account!(AssetAccount(u128), |v: AssetAccount| AddressInPrefixedSet(
	0xffffffff, v.0
)
.into());
mock_account!(SelfReserveAccount, |_| MockAccount::from_u64(2));

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
}
parameter_types! {
	pub const ExistentialDeposit: u128 = 0;
}
impl pallet_balances::Config for Runtime {
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

pub type Precompiles<R> =
	PrecompileSetBuilder<R, (PrecompileAt<AddressU64<1>, XtokensPrecompile<R>>,)>;

pub type PCall = XtokensPrecompileCall<Runtime>;

const MAX_POV_SIZE: u64 = 5 * 1024 * 1024;
/// Block storage limit in bytes. Set to 40 KB.
const BLOCK_STORAGE_LIMIT: u64 = 40 * 1024;

parameter_types! {
	pub BlockGasLimit: U256 = U256::from(u64::MAX);
	pub PrecompilesValue: Precompiles<Runtime> = Precompiles::new();
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub GasLimitPovSizeRatio: u64 = {
		let block_gas_limit = BlockGasLimit::get().min(u64::MAX.into()).low_u64();
		block_gas_limit.saturating_div(MAX_POV_SIZE)
	};
	pub GasLimitStorageGrowthRatio: u64 = {
		let block_gas_limit = BlockGasLimit::get().min(u64::MAX.into()).low_u64();
		block_gas_limit.saturating_div(BLOCK_STORAGE_LIMIT)
	};
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = AccountId;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = Precompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type OnCreate = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type SuicideQuickClearLimit = ConstU32<0>;
	type GasLimitStorageGrowthRatio = GasLimitStorageGrowthRatio;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Runtime>;
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
	type Success = Location;

	fn try_origin(_: Origin) -> Result<Location, Origin> {
		Ok(Location::here())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<Origin, ()> {
		Ok(Origin::root())
	}
}

pub struct DoNothingRouter;
impl SendXcm for DoNothingRouter {
	type Ticket = ();

	fn validate(
		_destination: &mut Option<Location>,
		_message: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		Ok(((), Assets::new()))
	}

	fn deliver(_: Self::Ticket) -> Result<XcmHash, SendError> {
		Ok(XcmHash::default())
	}
}

pub type Barrier = AllowUnpaidExecutionFrom<Everything>;

pub struct DummyAssetTransactor;
impl TransactAsset for DummyAssetTransactor {
	fn deposit_asset(_what: &Asset, _who: &Location, _context: Option<&XcmContext>) -> XcmResult {
		Ok(())
	}

	fn withdraw_asset(
		_what: &Asset,
		_who: &Location,
		_maybe_context: Option<&XcmContext>,
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

parameter_types! {
	pub UniversalLocation: InteriorLocation = Here;
	pub MatcherLocation: Location = Location::here();
	pub const MaxAssetsIntoHolding: u32 = 64;
}

impl pallet_xcm::Config for Runtime {
	// The config types here are entirely configurable, since the only one that is sorely needed
	// is `XcmExecutor`, which will be used in unit tests located in xcm-executor.
	type RuntimeEvent = RuntimeEvent;
	type ExecuteXcmOrigin = ConvertOriginToLocal;
	type UniversalLocation = UniversalLocation;
	type SendXcmOrigin = ConvertOriginToLocal;
	type Weigher = xcm_builder::FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
	type XcmRouter = DoNothingRouter;
	type XcmExecuteFilter = frame_support::traits::Everything;
	type XcmExecutor = xcm_executor::XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = frame_support::traits::Everything;
	type XcmReserveTransferFilter = frame_support::traits::Everything;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = IsConcrete<MatcherLocation>;
	type TrustedLockers = ();
	type SovereignAccountOf = ();
	type MaxLockers = ConstU32<8>;
	type WeightInfo = pallet_xcm::TestWeightInfo;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	type AdminOrigin = frame_system::EnsureRoot<AccountId>;
}

#[derive(Encode, Decode)]
pub enum RelayCall {
	#[codec(index = 5u8)]
	// the index should match the position of the module in `construct_runtime!`
	Utility(UtilityCall),
	#[codec(index = 6u8)]
	// the index should match the position of the module in `construct_runtime!`
	Hrmp(HrmpCall),
}

#[derive(Encode, Decode)]
pub enum UtilityCall {
	#[codec(index = 1u8)]
	AsDerivative(u16),
}

// HRMP call encoding, needed for xcm transactor pallet
#[derive(Encode, Decode)]
pub enum HrmpCall {
	#[codec(index = 0u8)]
	InitOpenChannel(ParaId, u32, u32),
	#[codec(index = 1u8)]
	AcceptOpenChannel(ParaId),
	#[codec(index = 2u8)]
	CloseChannel(HrmpChannelId),
	#[codec(index = 6u8)]
	CancelOpenRequest(HrmpChannelId, u32),
}

#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum MockTransactors {
	Relay,
}

impl xcm_primitives::XcmTransact for MockTransactors {
	fn destination(self) -> Location {
		match self {
			MockTransactors::Relay => Location::parent(),
		}
	}
}

impl xcm_primitives::UtilityEncodeCall for MockTransactors {
	fn encode_call(self, call: xcm_primitives::UtilityAvailableCalls) -> Vec<u8> {
		match self {
			MockTransactors::Relay => match call {
				xcm_primitives::UtilityAvailableCalls::AsDerivative(a, b) => {
					let mut call =
						RelayCall::Utility(UtilityCall::AsDerivative(a.clone())).encode();
					call.append(&mut b.clone());
					call
				}
			},
		}
	}
}

parameter_types! {
	pub SelfLocationAbsolute: Location = Location {
		parents: 1,
		interior: [Parachain(ParachainId::get().into())].into(),
	};
}

impl pallet_xcm_transactor::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Transactor = MockTransactors;
	type DerivativeAddressRegistrationOrigin = frame_system::EnsureRoot<AccountId>;
	type SovereignAccountDispatcherOrigin = frame_system::EnsureRoot<AccountId>;
	type CurrencyId = CurrencyId;
	type AccountIdToLocation = AccountIdToLocation;
	type CurrencyIdToLocation = CurrencyIdToMultiLocation;
	type SelfLocation = SelfLocation;
	type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type BaseXcmWeight = BaseXcmWeight;
	type XcmSender = DoNothingRouter;
	type AssetTransactor = DummyAssetTransactor;
	type ReserveProvider = xcm_primitives::AbsoluteAndRelativeReserve<SelfLocationAbsolute>;
	type WeightInfo = ();
	type HrmpManipulatorOrigin = frame_system::EnsureRoot<AccountId>;
	type HrmpOpenOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxHrmpFee = ();
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = DoNothingRouter;
	type AssetTransactor = DummyAssetTransactor;
	type OriginConverter = pallet_xcm::XcmPassthrough<RuntimeOrigin>;
	type IsReserve = ();
	type IsTeleporter = ();
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
	type Trader = DummyWeightTrader;
	type ResponseHandler = ();
	type SubscriptionService = ();
	type AssetTrap = ();
	type AssetClaims = ();
	type CallDispatcher = RuntimeCall;
	type AssetLocker = ();
	type AssetExchanger = ();
	type PalletInstancesInfo = ();
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type SafeCallFilter = Everything;
	type Aliasers = Nothing;
	type TransactionalProcessor = ();
	type HrmpNewChannelOpenRequestHandler = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type XcmRecorder = ();
}

#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum CurrencyId {
	SelfReserve,
	OtherReserve(AssetId),
}

// Implement the trait, where we convert AccountId to AssetID
impl AccountIdToCurrencyId<AccountId, CurrencyId> for Runtime {
	/// The way to convert an account to assetId is by ensuring that the prefix is 0XFFFFFFFF
	/// and by taking the lowest 128 bits as the assetId
	fn account_to_currency_id(account: AccountId) -> Option<CurrencyId> {
		match account {
			a if a.has_prefix_u32(0xffffffff) => Some(CurrencyId::OtherReserve(a.without_prefix())),
			a if a == AccountId::from(SelfReserveAccount) => Some(CurrencyId::SelfReserve),
			_ => None,
		}
	}
}

pub struct CurrencyIdToMultiLocation;

impl sp_runtime::traits::Convert<CurrencyId, Option<Location>> for CurrencyIdToMultiLocation {
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
					Some(Location::new(1, [Parachain(2), GeneralIndex(asset)]))
				}
			}
		}
	}
}

pub struct AccountIdToLocation;
impl sp_runtime::traits::Convert<AccountId, Location> for AccountIdToLocation {
	fn convert(account: AccountId) -> Location {
		let as_h160: H160 = account.into();
		Location::new(
			1,
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
	pub const MaxAssetsForTransfer: usize = 2;

	pub SelfLocation: Location =
		Location::new(1, [Parachain(ParachainId::get().into())]);

	pub SelfReserve: Location = Location::new(
		1,
		[
			Parachain(ParachainId::get().into()),
			PalletInstance(
				<Runtime as frame_system::Config>::PalletInfo::index::<Balances>().unwrap() as u8
			)
		]);
	pub MaxInstructions: u32 = 100;
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
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
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

pub(crate) fn events() -> Vec<RuntimeEvent> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.collect::<Vec<_>>()
}
