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
use frame_support::{
	construct_runtime, parameter_types,
	traits::{EqualPrivilegeOnly, Everything},
};
use frame_system::EnsureRoot;
use pallet_evm::{
	AddressMapping, EnsureAddressNever, EnsureAddressRoot, PrecompileSet, SubstrateBlockHashMapping,
};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::{H160, H256, U256};
use sp_io;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};

pub type AccountId = Account;
pub type Balance = u128;
pub type BlockNumber = u32;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

pub const PRECOMPILE_ADDRESS: u64 = 1;

#[derive(
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Clone,
	Encode,
	Decode,
	Debug,
	MaxEncodedLen,
	Serialize,
	Deserialize,
	derive_more::Display,
	TypeInfo,
)]
pub enum Account {
	Alice,
	Bob,
	Charlie,
	Bogus,
	Precompile,
}

impl Default for Account {
	fn default() -> Self {
		Self::Bogus
	}
}

impl AddressMapping<Account> for Account {
	fn into_account_id(h160_account: H160) -> Account {
		match h160_account {
			a if a == H160::repeat_byte(0xAA) => Self::Alice,
			a if a == H160::repeat_byte(0xBB) => Self::Bob,
			a if a == H160::repeat_byte(0xCC) => Self::Charlie,
			a if a == H160::from_low_u64_be(PRECOMPILE_ADDRESS) => Self::Precompile,
			_ => Self::Bogus,
		}
	}
}

impl From<H160> for Account {
	fn from(x: H160) -> Account {
		Account::into_account_id(x)
	}
}

impl From<Account> for H160 {
	fn from(value: Account) -> H160 {
		match value {
			Account::Alice => H160::repeat_byte(0xAA),
			Account::Bob => H160::repeat_byte(0xBB),
			Account::Charlie => H160::repeat_byte(0xCC),
			Account::Precompile => H160::from_low_u64_be(PRECOMPILE_ADDRESS),
			Account::Bogus => Default::default(),
		}
	}
}

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Evm: pallet_evm::{Pallet, Config, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		AuthorMapping: pallet_author_mapping::{Pallet, Storage, Config<T>, Event<T>, Call},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
	}
);

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
	type AccountId = Account;
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

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub const PrecompilesValue: Precompiles<Runtime> = Precompiles(PhantomData);
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressRoot<Account>;
	type WithdrawOrigin = EnsureAddressNever<Account>;
	type AddressMapping = Account;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = Precompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = SubstrateBlockHashMapping<Self>;
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
	pub const DepositAmount: Balance = 10;
}

impl pallet_author_mapping::Config for Runtime {
	type Event = Event;
	type DepositCurrency = Balances;
	type DepositAmount = DepositAmount;
	type Keys = nimbus_primitives::NimbusId;
	type WeightInfo = ();
}

impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = ();
	type ScheduleOrigin = EnsureRoot<Account>;
	type MaxScheduledPerBlock = ();
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly; // TODO : Simplest type, maybe there is better ?
	type PreimageProvider = ();
	type NoPreimagePostponement = ();
}

#[derive(Default)]
pub struct Precompiles<R>(PhantomData<R>);

impl<R> PrecompileSet for Precompiles<R>
where
	AuthorMappingPrecompile<R>: Precompile,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		match handle.code_address() {
			a if a == hash(PRECOMPILE_ADDRESS) => {
				Some(AuthorMappingPrecompile::<R>::execute(handle))
			}
			_ => None,
		}
	}

	fn is_precompile(&self, address: H160) -> bool {
		address == hash(PRECOMPILE_ADDRESS)
	}
}

pub type PCall = AuthorMappingPrecompileCall<Runtime>;

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
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

#[test]
fn test_account_id_mapping_works() {
	// Bidirectional conversions for normal accounts
	assert_eq!(
		Account::Alice,
		Account::into_account_id(Account::Alice.into())
	);
	assert_eq!(Account::Bob, Account::into_account_id(Account::Bob.into()));
	assert_eq!(
		Account::Charlie,
		Account::into_account_id(Account::Charlie.into())
	);

	// Bidirectional conversion between bogus and default H160
	assert_eq!(Account::Bogus, Account::into_account_id(H160::default()));
	assert_eq!(H160::default(), Account::Bogus.into());

	// All other H160s map to bogus
	assert_eq!(Account::Bogus, Account::into_account_id(H160::zero()));
	assert_eq!(
		Account::Bogus,
		Account::into_account_id(H160::repeat_byte(0x12))
	);
	assert_eq!(
		Account::Bogus,
		Account::into_account_id(H160::repeat_byte(0xFF))
	);
}
