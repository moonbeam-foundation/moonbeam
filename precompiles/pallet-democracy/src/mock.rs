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
	traits::{EqualPrivilegeOnly, Everything, OnFinalize, OnInitialize},
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_democracy::VoteThreshold;
use pallet_evm::{
	AddressMapping, EnsureAddressNever, EnsureAddressRoot, PrecompileSet, SubstrateBlockHashMapping,
};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::{H256, U256};
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
		Democracy: pallet_democracy::{Pallet, Storage, Config<T>, Event<T>, Call},
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
	pub const LaunchPeriod: BlockNumber = 10;
	pub const VotingPeriod: BlockNumber = 10;
	pub const VoteLockingPeriod: BlockNumber = 10;
	pub const FastTrackVotingPeriod: BlockNumber = 5;
	pub const EnactmentPeriod: BlockNumber = 10;
	pub const CooloffPeriod: BlockNumber = 10;
	pub const MinimumDeposit: Balance = 10;
	pub const MaxVotes: u32 = 10;
	pub const MaxProposals: u32 = 10;
	pub const PreimageByteDeposit: Balance = 10;
	pub const InstantAllowed: bool = false;
}

impl pallet_democracy::Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type VoteLockingPeriod = VoteLockingPeriod;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	type MinimumDeposit = MinimumDeposit;
	type ExternalOrigin = EnsureRoot<AccountId>;
	type ExternalMajorityOrigin = EnsureRoot<AccountId>;
	type ExternalDefaultOrigin = EnsureRoot<AccountId>;
	type FastTrackOrigin = EnsureRoot<AccountId>;
	type InstantOrigin = EnsureRoot<AccountId>;
	type CancellationOrigin = EnsureRoot<AccountId>;
	type CancelProposalOrigin = EnsureRoot<AccountId>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	type VetoOrigin = EnsureSigned<AccountId>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type Slash = ();
	type InstantAllowed = InstantAllowed;
	type Scheduler = Scheduler;
	type MaxVotes = MaxVotes;
	type OperationalPreimageOrigin = EnsureSigned<AccountId>;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
	type MaxProposals = MaxProposals;
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
	DemocracyPrecompile<R>: Precompile,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		match handle.code_address() {
			a if a == hash(PRECOMPILE_ADDRESS) => Some(DemocracyPrecompile::<R>::execute(handle)),
			_ => None,
		}
	}

	fn is_precompile(&self, address: H160) -> bool {
		address == hash(PRECOMPILE_ADDRESS)
	}
}

pub type PCall = DemocracyPrecompileCall<Runtime>;

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}

/// Build test externalities, prepopulated with data for testing democracy precompiles
pub(crate) struct ExtBuilder {
	/// Endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
	/// Referenda that already exist (don't need a proposal and launch period delay)
	referenda: Vec<(H256, VoteThreshold, BlockNumber)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![],
			referenda: vec![],
		}
	}
}

impl ExtBuilder {
	/// Fund some accounts before starting the test
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	/// Put some referenda into storage before starting the test
	pub(crate) fn with_referenda(
		mut self,
		referenda: Vec<(H256, VoteThreshold, BlockNumber)>,
	) -> Self {
		self.referenda = referenda;
		self
	}

	/// Build the test externalities for use in tests
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.balances.clone(),
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| {
			System::set_block_number(1);

			// Pallet democracy doesn't have a meaningful genesis config, so we use
			// its helper method to initialize the referenda
			for (hash, thresh, delay) in self.referenda {
				Democracy::internal_start_referendum(hash, thresh, delay);
			}
		});
		ext
	}
}

pub(crate) fn roll_to(n: BlockNumber) {
	// We skip timestamp's on_finalize because it requires that the timestamp inherent be set
	// We may be able to simulate this by poking its storage directly, but I don't see any value
	// added from doing that.
	while System::block_number() < n {
		Scheduler::on_finalize(System::block_number());
		Democracy::on_finalize(System::block_number());
		// Timestamp::on_finalize(System::block_number());
		Evm::on_finalize(System::block_number());
		Balances::on_finalize(System::block_number());
		System::on_finalize(System::block_number());

		System::set_block_number(System::block_number() + 1);

		System::on_initialize(System::block_number());
		Balances::on_initialize(System::block_number());
		Evm::on_initialize(System::block_number());
		Timestamp::on_initialize(System::block_number());
		Democracy::on_initialize(System::block_number());
		Scheduler::on_initialize(System::block_number());
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
