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
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU32, EnsureOrigin, Everything, Nothing, OriginTrait, PalletInfo as _},
	weights::{RuntimeDbWeight, Weight},
};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot, GasWeightMapping};
use precompile_utils::{
	mock_account,
	precompile_set::*,
	testing::{AddressInPrefixedSet, MockAccount},
};
use sp_core::{H256, U256};
use sp_io;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_std::borrow::Borrow;
use xcm::latest::Error as XcmError;
use xcm_builder::AllowUnpaidExecutionFrom;
use xcm_builder::FixedWeightBounds;
use xcm_builder::IsConcrete;
use xcm_builder::SovereignSignedViaLocation;
use xcm_executor::traits::Convert;
use xcm_executor::{
	traits::{TransactAsset, WeightTrader},
	Assets,
};
use Junctions::Here;

pub type AccountId = MockAccount;
pub type Balance = u128;
pub type BlockNumber = u32;

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
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
	}
);

mock_account!(SelfReserveAccount, |_| MockAccount::from_u64(2));
mock_account!(ParentAccount, |_| MockAccount::from_u64(3));
// use simple encoding for parachain accounts.
mock_account!(
	SiblingParachainAccount(u32),
	|v: SiblingParachainAccount| { AddressInPrefixedSet(0xffffffff, v.0 as u128).into() }
);

use frame_system::RawOrigin as SystemRawOrigin;
use xcm::latest::Junction;
pub struct MockAccountToAccountKey20<Origin, AccountId>(PhantomData<(Origin, AccountId)>);

impl<Origin: OriginTrait + Clone, AccountId: Into<H160>> Convert<Origin, MultiLocation>
	for MockAccountToAccountKey20<Origin, AccountId>
where
	Origin::PalletsOrigin: From<SystemRawOrigin<AccountId>>
		+ TryInto<SystemRawOrigin<AccountId>, Error = Origin::PalletsOrigin>,
{
	fn convert(o: Origin) -> Result<MultiLocation, Origin> {
		o.try_with_caller(|caller| match caller.try_into() {
			Ok(SystemRawOrigin::Signed(who)) => {
				let account_h160: H160 = who.into();
				Ok(Junction::AccountKey20 {
					network: None,
					key: account_h160.into(),
				}
				.into())
			}
			Ok(other) => Err(other.into()),
			Err(other) => Err(other),
		})
	}
}

pub struct MockParentMultilocationToAccountConverter;
impl Convert<MultiLocation, AccountId> for MockParentMultilocationToAccountConverter {
	fn convert_ref(location: impl Borrow<MultiLocation>) -> Result<AccountId, ()> {
		match location.borrow() {
			MultiLocation {
				parents: 1,
				interior: Here,
			} => Ok(ParentAccount.into()),
			_ => Err(()),
		}
	}

	fn reverse_ref(who: impl Borrow<AccountId>) -> Result<MultiLocation, ()> {
		match who.borrow() {
			a if a == &AccountId::from(ParentAccount) => Ok(MultiLocation::parent()),
			_ => Err(()),
		}
	}
}

pub struct MockParachainMultilocationToAccountConverter;
impl Convert<MultiLocation, AccountId> for MockParachainMultilocationToAccountConverter {
	fn convert_ref(location: impl Borrow<MultiLocation>) -> Result<AccountId, ()> {
		match location.borrow() {
			MultiLocation {
				parents: 1,
				interior: Junctions::X1(Parachain(id)),
			} => Ok(SiblingParachainAccount(*id).into()),
			_ => Err(()),
		}
	}

	fn reverse_ref(who: impl Borrow<AccountId>) -> Result<MultiLocation, ()> {
		match who.borrow() {
			a if a.has_prefix_u32(0xffffffff) => Ok(MultiLocation {
				parents: 1,
				interior: Junctions::X1(Parachain(a.without_prefix() as u32)),
			}),
			_ => Err(()),
		}
	}
}

pub type LocationToAccountId = (
	MockParachainMultilocationToAccountConverter,
	MockParentMultilocationToAccountConverter,
	xcm_builder::AccountKey20Aliases<LocalNetworkId, AccountId>,
);

pub struct AccountIdToMultiLocation;
impl sp_runtime::traits::Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
	fn convert(account: AccountId) -> MultiLocation {
		let as_h160: H160 = account.into();
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
	pub ParachainId: cumulus_primitives_core::ParaId = 100.into();
	pub LocalNetworkId: Option<NetworkId> = None;
}

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
	pub const MockDbWeight: RuntimeDbWeight = RuntimeDbWeight {
		read: 1,
		write: 5,
	};
}

impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type DbWeight = MockDbWeight;
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
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
	type HoldIdentifier = ();
	type FreezeIdentifier = ();
	type MaxHolds = ();
	type MaxFreezes = ();
}

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

parameter_types! {
	pub MatcherLocation: MultiLocation = MultiLocation::here();
}
pub type LocalOriginToLocation = MockAccountToAccountKey20<RuntimeOrigin, AccountId>;
impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = TestSendXcm;
	type ExecuteXcmOrigin = xcm_builder::EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = frame_support::traits::Everything;
	type XcmExecutor = xcm_executor::XcmExecutor<XcmConfig>;
	// Do not allow teleports
	type XcmTeleportFilter = Everything;
	type XcmReserveTransferFilter = Everything;
	type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
	type UniversalLocation = Ancestry;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	// We use a custom one to test runtime ugprades
	type AdvertisedXcmVersion = ();
	type Currency = Balances;
	type CurrencyMatcher = IsConcrete<MatcherLocation>;
	type TrustedLockers = ();
	type SovereignAccountOf = ();
	type MaxLockers = ConstU32<8>;
	type WeightInfo = pallet_xcm::TestWeightInfo;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	type AdminOrigin = frame_system::EnsureRoot<AccountId>;
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
}
pub type Precompiles<R> = PrecompileSetBuilder<
	R,
	(
		PrecompileAt<
			AddressU64<1>,
			XcmUtilsPrecompile<R, XcmConfig>,
			CallableByContract<AllExceptXcmExecute<R, XcmConfig>>,
		>,
	),
>;

pub type PCall = XcmUtilsPrecompileCall<Runtime, XcmConfig>;

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

/// A mapping function that converts Ethereum gas to Substrate weight
/// We are mocking this 1-1 to test db read charges too
pub struct MockGasWeightMapping;
impl GasWeightMapping for MockGasWeightMapping {
	fn gas_to_weight(gas: u64, _without_base_weight: bool) -> Weight {
		Weight::from_parts(gas, 1)
	}
	fn weight_to_gas(weight: Weight) -> u64 {
		weight.ref_time().into()
	}
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = MockGasWeightMapping;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = AccountId;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesValue = PrecompilesValue;
	type PrecompilesType = Precompiles<Self>;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type OnCreate = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
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
pub type Barrier = AllowUnpaidExecutionFrom<Everything>;

pub struct ConvertOriginToLocal;
impl<Origin: OriginTrait> EnsureOrigin<Origin> for ConvertOriginToLocal {
	type Success = MultiLocation;

	fn try_origin(_: Origin) -> Result<MultiLocation, Origin> {
		Ok(MultiLocation::here())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<Origin, ()> {
		Ok(Origin::root())
	}
}

use sp_std::cell::RefCell;
use xcm::latest::opaque;
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

pub struct DummyAssetTransactor;
impl TransactAsset for DummyAssetTransactor {
	fn deposit_asset(_what: &MultiAsset, _who: &MultiLocation, _context: &XcmContext) -> XcmResult {
		Ok(())
	}

	fn withdraw_asset(
		_what: &MultiAsset,
		_who: &MultiLocation,
		_maybe_context: Option<&XcmContext>,
	) -> Result<Assets, XcmError> {
		Ok(Assets::default())
	}
}

pub struct DummyWeightTrader;
impl WeightTrader for DummyWeightTrader {
	fn new() -> Self {
		DummyWeightTrader
	}

	fn buy_weight(&mut self, weight: Weight, payment: Assets) -> Result<Assets, XcmError> {
		let asset_to_charge: MultiAsset =
			(MultiLocation::parent(), weight.ref_time() as u128).into();
		let unused = payment
			.checked_sub(asset_to_charge)
			.map_err(|_| XcmError::TooExpensive)?;

		Ok(unused)
	}
}

parameter_types! {
	pub const BaseXcmWeight: Weight = Weight::from_parts(1000u64, 0u64);
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;

	pub SelfLocation: MultiLocation =
		MultiLocation::new(1, Junctions::X1(Parachain(ParachainId::get().into())));

	pub SelfReserve: MultiLocation = MultiLocation::new(
		1,
		Junctions::X2(
			Parachain(ParachainId::get().into()),
			PalletInstance(<Runtime as frame_system::Config>::PalletInfo::index::<Balances>().unwrap() as u8)
		));
	pub MaxInstructions: u32 = 100;

	pub UniversalLocation: InteriorMultiLocation = Here;
	pub Ancestry: InteriorMultiLocation =
		X2(GlobalConsensus(RelayNetwork::get()), Parachain(ParachainId::get().into()).into());

	pub const MaxAssetsIntoHolding: u32 = 64;
}

pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
);
pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = TestSendXcm;
	type AssetTransactor = DummyAssetTransactor;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
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
