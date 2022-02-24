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
};
use frame_system::{EnsureSigned, RawOrigin};
use pallet_evm::{AddressMapping, EnsureAddressNever, EnsureAddressRoot, PrecompileSet};
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_io;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};
pub type AccountId = H160;
pub type Balance = u128;
pub type BlockNumber = u64;
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
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>},
		Crowdloan: pallet_crowdloan_rewards::{Pallet, Call, Storage, Event<T>},
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
	scale_info::TypeInfo,
)]
pub enum TestAccount {
	Alice,
	Bob,
	Charlie,
	Bogus,
	Precompile,
}

/// And ipmlementation of Frontier's AddressMapping trait for Moonbeam Accounts.
/// This is basically identical to Frontier's own IdentityAddressMapping, but it works for any type
/// that is Into<H160> like AccountId20 for example.
pub struct IntoAddressMapping;

impl<T: From<H160>> AddressMapping<T> for IntoAddressMapping {
	fn into_account_id(address: H160) -> T {
		address.into()
	}
}

impl Default for TestAccount {
	fn default() -> Self {
		Self::Bogus
	}
}

impl From<TestAccount> for H160 {
	fn from(value: TestAccount) -> H160 {
		match value {
			TestAccount::Alice => H160::repeat_byte(0xAA),
			TestAccount::Bob => H160::repeat_byte(0xBB),
			TestAccount::Charlie => H160::repeat_byte(0xCC),
			TestAccount::Precompile => H160::from_low_u64_be(PRECOMPILE_ADDRESS),
			TestAccount::Bogus => Default::default(),
		}
	}
}

parameter_types! {
	pub ParachainId: cumulus_primitives_core::ParaId = 100.into();
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type SelfParaId = ParachainId;
	type Event = Event;
	type OnSystemEvent = ();
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
impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = H160;
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

/// The crowdloan precompile is available at address one in the mock runtime.
pub fn precompile_address() -> H160 {
	H160::from_low_u64_be(1)
}

#[derive(Debug, Clone, Copy)]
pub struct TestPrecompiles<R>(PhantomData<R>);

impl<R> PrecompileSet for TestPrecompiles<R>
where
	CrowdloanRewardsWrapper<R>: Precompile,
{
	fn execute(
		&self,
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> Option<EvmResult<PrecompileOutput>> {
		match address {
			a if a == precompile_address() => Some(CrowdloanRewardsWrapper::<R>::execute(
				input, target_gas, context, is_static,
			)),
			_ => None,
		}
	}

	fn is_precompile(&self, address: H160) -> bool {
		address == precompile_address()
	}
}

parameter_types! {
	pub const PrecompilesValue: TestPrecompiles<Runtime> = TestPrecompiles(PhantomData);
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = IntoAddressMapping;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesValue = PrecompilesValue;
	type PrecompilesType = TestPrecompiles<Self>;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = ();
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
	type Event = Event;
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
pub(crate) fn roll_to(n: u64) {
	while System::block_number() < n {
		// Relay chain Stuff. I might actually set this to a number different than N
		let sproof_builder = RelayStateSproofBuilder::default();
		let (relay_parent_storage_root, relay_chain_state) =
			sproof_builder.into_state_root_and_proof();
		let vfp = PersistedValidationData {
			relay_parent_number: (System::block_number() + 1u64) as RelayChainBlockNumber,
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

pub(crate) fn events() -> Vec<Event> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.collect::<Vec<_>>()
}

// Helper function to give a simple evm context suitable for tests.
// We can remove this once https://github.com/rust-blockchain/evm/pull/35
// is in our dependency graph.
pub fn evm_test_context() -> fp_evm::Context {
	fp_evm::Context {
		address: Default::default(),
		caller: Default::default(),
		apparent_value: From::from(0),
	}
}
