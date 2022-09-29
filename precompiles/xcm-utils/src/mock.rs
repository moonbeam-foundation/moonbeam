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
use frame_support::{
	construct_runtime, parameter_types,
	traits::{EnsureOrigin, Everything, OriginTrait, PalletInfo as PalletInfoTrait},
	weights::{RuntimeDbWeight, Weight},
};
use pallet_evm::{
	AddressMapping, EnsureAddressNever, EnsureAddressRoot, GasWeightMapping, Precompile,
	PrecompileOutput, PrecompileSet,
};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::{H256, U256};
use sp_io;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_std::borrow::Borrow;
use xcm::latest::{
	Error as XcmError,
	Junction::{AccountKey20, PalletInstance, Parachain},
	Junctions, MultiAsset, MultiLocation, NetworkId, Result as XcmResult, SendResult, SendXcm, Xcm,
};
use xcm_builder::AllowUnpaidExecutionFrom;
use xcm_builder::FixedWeightBounds;
use xcm_builder::SovereignSignedViaLocation;
use xcm_executor::traits::Convert;
use xcm_executor::{
	traits::{InvertLocation, TransactAsset, WeightTrader},
	Assets,
};
use Junctions::Here;

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
	}
);

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
	SelfReserve,
	Bogus,
	Precompile,
	// Parent multilocation address
	Parent,
	// Sibling multilocation address
	SiblingParachain(u32),
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
			_ => Self::Bogus,
		}
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
			TestAccount::Bogus => Default::default(),
			// Parent multilocation address
			TestAccount::Parent => {
				let multilocation = MultiLocation::parent();
				ParentIsPreset::<H160>::convert_ref(multilocation).unwrap()
			}
			// Sibling multilocation address
			TestAccount::SiblingParachain(para_id) => {
				let multilocation = MultiLocation {
					parents: 1,
					interior: Junctions::X1(Parachain(para_id)),
				};
				let account = SiblingParachainConvertsVia::<
					polkadot_parachain::primitives::Sibling,
					H160,
				>::convert_ref(multilocation)
				.unwrap();
				account
			}
		}
	}
}

pub struct MockParentMultilocationToAccountConverter;
impl Convert<MultiLocation, AccountId> for MockParentMultilocationToAccountConverter {
	fn convert_ref(location: impl Borrow<MultiLocation>) -> Result<AccountId, ()> {
		match location.borrow() {
			MultiLocation {
				parents: 1,
				interior: Here,
			} => Ok(TestAccount::Parent),
			_ => Err(()),
		}
	}

	fn reverse_ref(who: impl Borrow<AccountId>) -> Result<MultiLocation, ()> {
		match who.borrow() {
			TestAccount::Parent => Ok(MultiLocation::parent()),
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
			} => Ok(TestAccount::SiblingParachain(*id)),
			_ => Err(()),
		}
	}

	fn reverse_ref(who: impl Borrow<AccountId>) -> Result<MultiLocation, ()> {
		match who.borrow() {
			TestAccount::SiblingParachain(id) => Ok(MultiLocation {
				parents: 1,
				interior: Junctions::X1(Parachain(*id)),
			}),
			_ => Err(()),
		}
	}
}

pub type LocationToAccountId = (
	MockParachainMultilocationToAccountConverter,
	MockParentMultilocationToAccountConverter,
);

impl From<TestAccount> for [u8; 20] {
	fn from(value: TestAccount) -> [u8; 20] {
		let as_h160: H160 = value.into();
		as_h160.into()
	}
}

impl From<[u8; 20]> for TestAccount {
	fn from(value: [u8; 20]) -> TestAccount {
		let as_h160: H160 = value.into();
		TestAccount::into_account_id(as_h160)
	}
}

pub struct AccountIdToMultiLocation;
impl sp_runtime::traits::Convert<TestAccount, MultiLocation> for AccountIdToMultiLocation {
	fn convert(account: TestAccount) -> MultiLocation {
		let as_h160: H160 = account.into();
		MultiLocation::new(
			0,
			Junctions::X1(AccountKey20 {
				network: NetworkId::Any,
				key: as_h160.as_fixed_bytes().clone(),
			}),
		)
	}
}

parameter_types! {
	pub ParachainId: cumulus_primitives_core::ParaId = 100.into();
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

pub struct TestPrecompiles<R>(PhantomData<R>);

impl<R> PrecompileSet for TestPrecompiles<R>
where
	XcmUtilsPrecompile<R, XcmConfig>: Precompile,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		match handle.code_address() {
			a if a == precompile_address() => {
				Some(XcmUtilsPrecompile::<R, XcmConfig>::execute(handle))
			}
			_ => None,
		}
	}

	fn is_precompile(&self, address: H160) -> bool {
		address == precompile_address()
	}
}

pub type PCall = XcmUtilsPrecompileCall<Runtime, XcmConfig>;

pub fn precompile_address() -> H160 {
	H160::from_low_u64_be(1)
}

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub const PrecompilesValue: TestPrecompiles<Runtime> = TestPrecompiles(PhantomData);
}

/// A mapping function that converts Ethereum gas to Substrate weight
/// We are mocking this 1-1 to test db read charges too
pub struct MockGasWeightMapping;
impl GasWeightMapping for MockGasWeightMapping {
	fn gas_to_weight(gas: u64) -> Weight {
		Weight::from_ref_time(gas)
	}
	fn weight_to_gas(weight: Weight) -> u64 {
		weight.ref_time().into()
	}
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = MockGasWeightMapping;
	type CallOrigin = EnsureAddressRoot<TestAccount>;
	type WithdrawOrigin = EnsureAddressNever<TestAccount>;
	type AddressMapping = TestAccount;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesValue = PrecompilesValue;
	type PrecompilesType = TestPrecompiles<Self>;
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
pub type Barrier = AllowUnpaidExecutionFrom<Everything>;

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

parameter_types! {
	pub Ancestry: MultiLocation = Parachain(ParachainId::get().into()).into();

	pub const BaseXcmWeight: XcmV2Weight = 1000;
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;

	pub SelfLocation: MultiLocation = (1, Junctions::X1(Parachain(ParachainId::get().into()))).into();

	pub SelfReserve: MultiLocation = (
		1,
		Junctions::X2(
			Parachain(ParachainId::get().into()),
			PalletInstance(<Runtime as frame_system::Config>::PalletInfo::index::<Balances>().unwrap() as u8)
		)).into();
	pub MaxInstructions: u32 = 100;
}

use xcm_builder::{ParentIsPreset, SiblingParachainConvertsVia};
use xcm_primitives::XcmV2Weight;

pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, Origin>,
);
pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type Call = Call;
	type XcmSender = DoNothingRouter;
	type AssetTransactor = DummyAssetTransactor;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
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

pub(crate) struct ExtBuilder {}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {}
	}
}

impl ExtBuilder {
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.expect("Frame system builds valid default genesis config");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
