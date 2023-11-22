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
	traits::{Everything, PalletInfo as PalletInfoTrait},
	weights::Weight,
};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot, SubstrateBlockHashMapping};
use parity_scale_codec::{Decode, Encode};
use precompile_utils::{precompile_set::*, testing::MockAccount};
use scale_info::TypeInfo;
use sp_core::{H160, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage, Perbill,
};
use xcm::latest::{prelude::*, Error as XcmError};
use xcm_builder::FixedWeightBounds;
use xcm_executor::{traits::TransactAsset, Assets};

pub type AccountId = MockAccount;
pub type Balance = u128;

type Block = frame_system::mocking::MockBlockU32<Runtime>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime	{
		System: frame_system,
		Balances: pallet_balances,
		Evm: pallet_evm,
		ParachainSystem: cumulus_pallet_parachain_system,
		Timestamp: pallet_timestamp,
		XcmTransactor: pallet_xcm_transactor,
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
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
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub ParachainId: cumulus_primitives_core::ParaId = 100.into();
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type SelfParaId = ParachainId;
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type OutboundXcmpMessageSource = ();
	type XcmpMessageHandler = ();
	type ReservedXcmpWeight = ();
	type DmpMessageHandler = ();
	type ReservedDmpWeight = ();
	type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
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
	type MaxHolds = ();
	type MaxFreezes = ();
}

pub type AssetId = u128;

#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum CurrencyId {
	SelfReserve,
	OtherReserve(AssetId),
}

pub struct DoNothingRouter;
impl SendXcm for DoNothingRouter {
	type Ticket = ();

	fn validate(
		_destination: &mut Option<MultiLocation>,
		_message: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		Ok(((), MultiAssets::new()))
	}

	fn deliver(_: Self::Ticket) -> Result<XcmHash, SendError> {
		Ok(XcmHash::default())
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
				<Runtime as frame_system::Config>::PalletInfo::index::<Balances>().unwrap() as u8
			)
		));
	pub MaxInstructions: u32 = 100;

	pub UniversalLocation: InteriorMultiLocation = Here;
}

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

impl xcm_primitives::XcmTransact for MockTransactors {
	fn destination(self) -> MultiLocation {
		match self {
			MockTransactors::Relay => MultiLocation::parent(),
		}
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

impl pallet_xcm_transactor::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Transactor = MockTransactors;
	type DerivativeAddressRegistrationOrigin = frame_system::EnsureRoot<AccountId>;
	type SovereignAccountDispatcherOrigin = frame_system::EnsureRoot<AccountId>;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = AccountIdToMultiLocation;
	type CurrencyIdToMultiLocation = CurrencyIdToMultiLocation;
	type SelfLocation = SelfLocation;
	type Weigher = FixedWeightBounds<BaseXcmWeight, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type BaseXcmWeight = BaseXcmWeight;
	type XcmSender = DoNothingRouter;
	type AssetTransactor = DummyAssetTransactor;
	type ReserveProvider = orml_traits::location::RelativeReserveProvider;
	type WeightInfo = ();
	type HrmpManipulatorOrigin = frame_system::EnsureRoot<AccountId>;
	type MaxHrmpFee = ();
}

pub type Precompiles<R> =
	PrecompileSetBuilder<R, PrecompileAt<AddressU64<1>, RelayEncoderPrecompile<R>>>;

pub type PCall = RelayEncoderPrecompileCall<Runtime>;

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
	type PrecompilesValue = PrecompilesValue;
	type PrecompilesType = Precompiles<Self>;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = SubstrateBlockHashMapping<Self>;
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

parameter_types! {
	pub const TestMaxInitContributors: u32 = 8;
	pub const TestMinimumReward: u128 = 0;
	pub const TestInitialized: bool = false;
	pub const TestInitializationPayment: Perbill = Perbill::from_percent(20);
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
		ext.execute_with(|| {
			pallet_xcm_transactor::RelayIndices::<Runtime>::put(
				crate::test_relay_runtime::TEST_RELAY_INDICES,
			);
		});
		ext
	}
}
