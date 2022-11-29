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
use cumulus_primitives_core::{
	relay_chain::BlockNumber as RelayChainBlockNumber, PersistedValidationData,
};
use cumulus_primitives_parachain_inherent::ParachainInherentData;
use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
use frame_support::{
	construct_runtime,
	dispatch::UnfilteredDispatchable,
	inherent::{InherentData, ProvideInherent},
	parameter_types,
	traits::{Everything, GenesisBuild, OnFinalize, OnInitialize},
	weights::Weight,
};
use frame_system::{EnsureSigned, RawOrigin};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::{precompile_set::*, testing::MockAccount};
use sp_core::{H256, U256};
use sp_io;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};

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
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>},
		Crowdloan: pallet_crowdloan_rewards::{Pallet, Call, Storage, Event<T>},
	}
);

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
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type DbWeight = ();
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
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
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
}

pub type Precompiles<R> =
	PrecompileSetBuilder<R, (PrecompileAt<AddressU64<1>, CrowdloanRewardsPrecompile<R>>,)>;

pub type PCall = CrowdloanRewardsPrecompileCall<Runtime>;

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub PrecompilesValue: Precompiles<Runtime> = Precompiles::new();
	pub const WeightPerGas: Weight = Weight::from_ref_time(1);
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

parameter_types! {
	pub const TestMaxInitContributors: u32 = 8;
	pub const TestMinimumReward: u128 = 0;
	pub const TestInitialized: bool = false;
	pub const TestInitializationPayment: Perbill = Perbill::from_percent(20);
	pub const RelaySignaturesThreshold: Perbill = Perbill::from_percent(100);
	pub const TestSignatureNetworkIdentifier: &'static [u8] = b"test-";
}

impl pallet_crowdloan_rewards::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Initialized = TestInitialized;
	type InitializationPayment = TestInitializationPayment;
	type MaxInitContributors = TestMaxInitContributors;
	type MinimumReward = TestMinimumReward;
	type RewardCurrency = Balances;
	type RelayChainAccountId = [u8; 32];
	type RewardAddressAssociateOrigin = EnsureSigned<Self::AccountId>;
	type RewardAddressRelayVoteThreshold = RelaySignaturesThreshold;
	type RewardAddressChangeOrigin = EnsureSigned<Self::AccountId>;
	type SignatureNetworkIdentifier = TestSignatureNetworkIdentifier;

	type VestingBlockNumber = cumulus_primitives_core::relay_chain::BlockNumber;
	type VestingBlockProvider =
		cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Self>;
	type WeightInfo = ();
}
pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
	crowdloan_pot: Balance,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![],
			crowdloan_pot: 0u32.into(),
		}
	}
}

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}
	pub(crate) fn with_crowdloan_pot(mut self, pot: Balance) -> Self {
		self.crowdloan_pot = pot;
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

		pallet_crowdloan_rewards::GenesisConfig::<Runtime> {
			funded_amount: self.crowdloan_pot,
		}
		.assimilate_storage(&mut t)
		.expect("Crowdloan Rewards storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

//TODO Add pallets here if necessary
pub(crate) fn roll_to(n: BlockNumber) {
	while System::block_number() < n {
		// Relay chain Stuff. I might actually set this to a number different than N
		let sproof_builder = RelayStateSproofBuilder::default();
		let (relay_parent_storage_root, relay_chain_state) =
			sproof_builder.into_state_root_and_proof();
		let vfp = PersistedValidationData {
			relay_parent_number: (System::block_number() + 1) as RelayChainBlockNumber,
			relay_parent_storage_root,
			..Default::default()
		};
		let inherent_data = {
			let mut inherent_data = InherentData::default();
			let system_inherent_data = ParachainInherentData {
				validation_data: vfp.clone(),
				relay_chain_state,
				downward_messages: Default::default(),
				horizontal_messages: Default::default(),
			};
			inherent_data
				.put_data(
					cumulus_primitives_parachain_inherent::INHERENT_IDENTIFIER,
					&system_inherent_data,
				)
				.expect("failed to put VFP inherent");
			inherent_data
		};

		ParachainSystem::on_initialize(System::block_number());
		ParachainSystem::create_inherent(&inherent_data)
			.expect("got an inherent")
			.dispatch_bypass_filter(RawOrigin::None.into())
			.expect("dispatch succeeded");
		ParachainSystem::on_finalize(System::block_number());

		Balances::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Balances::on_initialize(System::block_number());
	}
}

pub(crate) fn events() -> Vec<RuntimeEvent> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.collect::<Vec<_>>()
}
