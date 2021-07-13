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
use crate::{self as liquid_staking};
use cumulus_primitives_core::ParaId;
use frame_support::{
	construct_runtime,
	dispatch::Weight,
	parameter_types,
	traits::{OnFinalize, OnInitialize},
	PalletId,
};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::AccountIdLookup;
use sp_runtime::traits::StaticLookup;

use sp_io;
use sp_runtime::testing::H256;
use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
	AccountId32, Perbill,
};
use sp_std::convert::From;
use xcm::v0::{Error as XcmError, Junction, MultiLocation, SendXcm, Xcm};
use xcm_builder::FixedWeightBounds;

pub type Balance = u128;
pub type AccountId = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		LiquidStaking: liquid_staking::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
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
	pub const ExistentialDeposit: u128 = 1;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const TestMaxInitContributors: u32 = 8;
	pub const TestMinimumReward: u128 = 0;
	pub const TestInitialized: bool = false;
	pub const TestInitializationPayment: Perbill = Perbill::from_percent(20);
}

pub struct XcmExecutorConfig;
impl xcm_executor::Config for XcmExecutorConfig {
	type Call = Call;
	type XcmSender = ();
	// How to withdraw and deposit an asset.
	type AssetTransactor = ();
	type OriginConverter = ();
	type IsReserve = xcm_builder::NativeAsset;
	type IsTeleporter = xcm_builder::NativeAsset; // <- should be enough to allow teleportation of KSM
	type LocationInverter = xcm_builder::LocationInverter<Ancestry>;
	type Barrier = ();
	type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
	type Trader = ();
	type ResponseHandler = (); // Don't handle responses for now.
}

type XcmExecutor = xcm_executor::XcmExecutor<XcmExecutorConfig>;

parameter_types! {
	pub const LiquidStakingId: PalletId = PalletId(*b"pc/lqstk");
	pub Ancestry: MultiLocation = Junction::Parachain(1000u32.into()).into();
	pub UnitWeightCost: Weight = 10;
	pub SovereignAccount: AccountId32 = ParaId::from(1000u32).into_account();

}

pub struct HandleXcm;
impl SendXcm for HandleXcm {
	fn send_xcm(dest: MultiLocation, msg: Xcm<()>) -> Result<(), XcmError> {
		Ok(())
	}
}

#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	Debug,
	max_encoded_len::MaxEncodedLen,
)]
pub enum RelayProxyType {
	Any,
	NonTransfer,
	Governance,
	Staking,
	IdentityJudgement,
	CancelProxy,
}
impl Default for RelayProxyType {
	fn default() -> Self {
		Self::Any
	}
}

// We want to avoid including the rococo-runtime here.
// TODO: whenever a conclusion is taken from https://github.com/paritytech/substrate/issues/8158
#[derive(Encode, Decode)]
pub enum RelayCall {
	#[codec(index = 30u8)]
	// the index should match the position of the module in `construct_runtime!`
	Proxy(AnonymousProxyCall),
}

#[derive(Encode, Decode)]
pub enum RelayStakeCall {
	#[codec(index = 6u8)]
	Stake(StakeCall),
}

#[derive(Encode, Decode)]
pub enum AnonymousProxyCall {
	#[codec(index = 0u8)]
	proxy(AccountId32, Option<RelayProxyType>, RelayStakeCall),

	#[codec(index = 4u8)]
	// the index should match the position of the dispatchable in the target pallet
	anonymous(RelayProxyType, u32, u16),
}

#[derive(Encode, Decode)]
pub enum StakeCall {
	#[codec(index = 0u8)]
	// the index should match the position of the dispatchable in the target pallet
	bond(
		<AccountIdLookup<AccountId32, ()> as StaticLookup>::Source,
		#[codec(compact)] cumulus_primitives_core::relay_chain::Balance,
		pallet_staking::RewardDestination<AccountId32>,
	),
	bondExtra,
	cancelDeferredSlash,
	Chill,
	forceNewEra,
	forceNoEras,
	forceUnstake,
	increaseValidator,
	kick,
	nominate,
	payoutStakers,
	reapStash,
	rebond,
	scaleValidator,
	setController,
	setHistory,
	setInvulnerables,
	setPayee,
	setValidator,
	unbond,
	validate,
	withdrawUnbonded,
}

pub struct TestEncoder;

impl liquid_staking::EncodeCall<Test> for TestEncoder {
	fn encode_call(call: liquid_staking::AvailableCalls<Test>) -> Vec<u8> {
		match call {
			liquid_staking::AvailableCalls::CreateAnonymusProxy(a, b, c) => {
				RelayCall::Proxy(AnonymousProxyCall::anonymous(a, b, c)).encode()
			}

			liquid_staking::AvailableCalls::BondThroughAnonymousProxy(a, b) => {
				RelayCall::Proxy(AnonymousProxyCall::proxy(
					a.clone(),
					None,
					RelayStakeCall::Stake(StakeCall::bond(
						a.into(),
						b,
						pallet_staking::RewardDestination::Controller,
					)),
				))
				.encode()
			}
			_ => panic!("SAd"),
		}
	}
}

impl liquid_staking::Config for Test {
	type Event = Event;
	type RelayCurrency = Balances;
	type PalletId = LiquidStakingId;
	type RelayChainProxyType = RelayProxyType;
	type RelayChainAccountId = AccountId32;
	type SovereignAccount = SovereignAccount;
	type CallEncoder = TestEncoder;
	type XcmSender = HandleXcm;
	type XcmExecutor = XcmExecutor;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
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
		let mut storage = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}
		.assimilate_storage(&mut storage)
		.expect("Pallet balances storage can be assimilated");
		let mut ext = sp_io::TestExternalities::from(storage);

		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub(crate) fn events() -> Vec<liquid_staking::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let Event::liquid_staking(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}

pub(crate) fn roll_to(n: u64) {
	while System::block_number() < n {
		LiquidStaking::on_finalize(System::block_number());
		Balances::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Balances::on_initialize(System::block_number());
		LiquidStaking::on_initialize(System::block_number());
	}
}
