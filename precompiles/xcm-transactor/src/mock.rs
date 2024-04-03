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
use crate::v1::{XcmTransactorPrecompileV1, XcmTransactorPrecompileV1Call};
use crate::v2::{XcmTransactorPrecompileV2, XcmTransactorPrecompileV2Call};
use crate::v3::{XcmTransactorPrecompileV3, XcmTransactorPrecompileV3Call};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{EnsureOrigin, Everything, OriginTrait, PalletInfo as PalletInfoTrait},
	weights::{RuntimeDbWeight, Weight},
};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot, GasWeightMapping};
use parity_scale_codec::{Decode, Encode};
use precompile_utils::{
	mock_account,
	precompile_set::*,
	testing::{AddressInPrefixedSet, MockAccount},
};
use scale_info::TypeInfo;
use sp_core::{ConstU32, H160, H256, U256};
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_runtime::BuildStorage;
use xcm::latest::{prelude::*, Error as XcmError};
use xcm_builder::FixedWeightBounds;
use xcm_executor::{
	traits::{TransactAsset, WeightTrader},
	AssetsInHolding,
};
use xcm_primitives::AccountIdToCurrencyId;

pub type AccountId = MockAccount;
pub type Balance = u128;

type Block = frame_system::mocking::MockBlockU32<Runtime>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime	{
		System: frame_system,
		Balances: pallet_balances,
		Evm: pallet_evm,
		Timestamp: pallet_timestamp,
		XcmTransactor: pallet_xcm_transactor,
	}
);

pub struct AccountIdToLocation;
impl sp_runtime::traits::Convert<AccountId, Location> for AccountIdToLocation {
	fn convert(account: AccountId) -> Location {
		let as_h160: H160 = account.into();
		Location::new(
			0,
			[AccountKey20 {
				network: None,
				key: as_h160.as_fixed_bytes().clone(),
			}],
		)
	}
}

pub type AssetId = u128;

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

pub type Precompiles<R> = PrecompileSetBuilder<
	R,
	(
		PrecompileAt<AddressU64<1>, XcmTransactorPrecompileV1<R>, CallableByContract>,
		PrecompileAt<AddressU64<2>, XcmTransactorPrecompileV2<R>, CallableByContract>,
		PrecompileAt<AddressU64<4>, XcmTransactorPrecompileV3<R>, CallableByContract>,
	),
>;

mock_account!(TransactorV1, |_| MockAccount::from_u64(1));
mock_account!(TransactorV2, |_| MockAccount::from_u64(2));
mock_account!(TransactorV3, |_| MockAccount::from_u64(4));
mock_account!(SelfReserveAddress, |_| MockAccount::from_u64(3));
mock_account!(AssetAddress(u128), |value: AssetAddress| {
	AddressInPrefixedSet(0xffffffff, value.0).into()
});

pub type PCallV1 = XcmTransactorPrecompileV1Call<Runtime>;
pub type PCallV2 = XcmTransactorPrecompileV2Call<Runtime>;
pub type PCallV3 = XcmTransactorPrecompileV3Call<Runtime>;

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

#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, scale_info::TypeInfo)]
pub enum CurrencyId {
	SelfReserve,
	OtherReserve(AssetId),
}

parameter_types! {
	pub Ancestry: Location = Parachain(ParachainId::get().into()).into();

	pub const BaseXcmWeight: Weight = Weight::from_parts(1000u64, 1000u64);
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;

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

	pub UniversalLocation: InteriorLocation = Here;
}

impl pallet_xcm_transactor::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Transactor = MockTransactors;
	type DerivativeAddressRegistrationOrigin = frame_system::EnsureRoot<AccountId>;
	type SovereignAccountDispatcherOrigin = frame_system::EnsureRoot<AccountId>;
	type CurrencyId = CurrencyId;
	type AccountIdToLocation = AccountIdToLocation;
	type CurrencyIdToLocation = CurrencyIdToLocation;
	type SelfLocation = SelfLocation;
	type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type BaseXcmWeight = BaseXcmWeight;
	type XcmSender = DoNothingRouter;
	type AssetTransactor = DummyAssetTransactor;
	type ReserveProvider = orml_traits::location::RelativeReserveProvider;
	type WeightInfo = ();
	type HrmpManipulatorOrigin = frame_system::EnsureRoot<AccountId>;
	type HrmpOpenOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxHrmpFee = ();
}

// We need to use the encoding from the relay mock runtime
#[derive(Encode, Decode)]
pub enum RelayCall {
	#[codec(index = 5u8)]
	// the index should match the position of the module in `construct_runtime!`
	Utility(UtilityCall),
}

#[derive(Encode, Decode)]
pub enum UtilityCall {
	#[codec(index = 1u8)]
	AsDerivative(u16),
}

#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum MockTransactors {
	Relay,
}

impl TryFrom<u8> for MockTransactors {
	type Error = ();

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			0x0 => Ok(MockTransactors::Relay),
			_ => Err(()),
		}
	}
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

// Implement the trait, where we convert AccountId to AssetID
impl AccountIdToCurrencyId<AccountId, CurrencyId> for Runtime {
	/// The way to convert an account to assetId is by ensuring that the prefix is 0XFFFFFFFF
	/// and by taking the lowest 128 bits as the assetId
	fn account_to_currency_id(account: AccountId) -> Option<CurrencyId> {
		match account {
			a if a.has_prefix_u32(0xffffffff) => Some(CurrencyId::OtherReserve(a.without_prefix())),
			a if a == SelfReserveAddress.into() => Some(CurrencyId::SelfReserve),
			_ => None,
		}
	}
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
					Some(Location::new(1, [Parachain(2), GeneralIndex(asset)]))
				}
			}
		}
	}
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
