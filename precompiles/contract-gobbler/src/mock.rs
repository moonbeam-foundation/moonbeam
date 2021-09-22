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
use frame_system::RawOrigin;
use pallet_evm::{AddressMapping, EnsureAddressNever, EnsureAddressRoot, PrecompileSet};
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_io;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};
pub type AccountId = TestAccount;
pub type Balance = u128;
pub type BlockNumber = u64;

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
			TestAccount::Bogus => Default::default(),
		}
	}
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
	type OnSetCode = ();
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

/// The crowdloan precompile is available at address one in the mock runtime.
pub fn precompile_address() -> H160 {
	H160::from_low_u64_be(1)
}

// #[derive(Debug, Clone, Copy)]
// pub struct TestPrecompiles<R>(PhantomData<R>);

// impl<R> PrecompileSet for TestPrecompiles<R>
// where
// 	R::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + Decode,
// 	<R::Call as Dispatchable>::Origin: From<Option<R::AccountId>>,
// 	R: pallet_crowdloan_rewards::Config + pallet_evm::Config,
// 	BalanceOf<R>: TryFrom<sp_core::U256> + Debug,
// 	R::Call: From<pallet_crowdloan_rewards::Call<R>>,
// {
// 	fn execute(
// 		address: H160,
// 		input: &[u8],
// 		target_gas: Option<u64>,
// 		context: &Context,
// 	) -> Option<Result<PrecompileOutput, ExitError>> {
// 		match address {
// 			a if a == precompile_address() => Some(CrowdloanRewardsWrapper::<R>::execute(
// 				input, target_gas, context,
// 			)),
// 			_ => None,
// 		}
// 	}
// }

// pub type Precompiles = TestPrecompiles<Test>;

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressRoot<TestAccount>;
	type WithdrawOrigin = EnsureAddressNever<TestAccount>;
	type AddressMapping = TestAccount;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type Precompiles = (ContractGobbler,);
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = ();
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
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
