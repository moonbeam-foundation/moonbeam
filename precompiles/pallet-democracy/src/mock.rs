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
use frame_support::{
	construct_runtime, parameter_types, assert_noop,
	traits::{OnFinalize, OnInitialize},
};
use frame_system::{EnsureRoot, EnsureSigned};
//TODO should be necessary to ensure that precompile accessors return the right weight/
// use frame_system::limits::BlockWeights;
use sp_core::H256;
use sp_io;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

pub type AccountId = u64;
pub type Balance = u128;
pub type BlockNumber = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
use pallet_evm::{EnsureAddressRoot, EnsureAddressNever, AddressMapping, Precompile, PrecompileSet, ExitError};

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Evm: pallet_evm::{Pallet, Config, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Democracy: pallet_democracy::{Pallet, Storage, Config<T>, Event<T>, Call},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Config, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
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

/// A simple address mapping used for testing. Maps an address to its low byte.
/// Also has a helper method for going the other way in order to make valid calls.
/// TODO Check byte order.
pub struct TestMapping;
impl AddressMapping<u64> for TestMapping {
	fn into_account_id(h160_account: H160) -> u64 {
		h160_account.as_bytes()[0] as u64
	}
}

impl TestMapping {
	fn account_id_to_h160(account_id: u64) -> H160 {
		H160::from_low_u64_be(account_id)
	}
}

/// The democracy precompile is available at address zero in the mock runtime.
fn precompile_address() -> H160 {
	H160::from_low_u64_be(1)
}
type Precompiles = (DemocracyWrapper<Test>,);

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = TestMapping;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type Precompiles = Precompiles;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}
impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 10;
	pub const VotingPeriod: BlockNumber = 10;
	pub const FastTrackVotingPeriod: BlockNumber = 5;
	pub const EnactmentPeriod: BlockNumber = 10;
	pub const CooloffPeriod: BlockNumber = 10;
	pub const MinimumDeposit: Balance = 10;
	pub const MaxVotes: u32 = 10;
	pub const MaxProposals: u32 = 10;
	pub const PreimageByteDeposit: Balance = 10;
	pub const InstantAllowed: bool = false;
}

impl pallet_democracy::Config for Test {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
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
impl pallet_scheduler::Config for Test {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = ();
	type ScheduleOrigin = EnsureRoot<u64>;
	type MaxScheduledPerBlock = ();
	type WeightInfo = ();
}

pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![],
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
		// Evm: pallet_evm::{Pallet, Config, Call, Storage, Event<T>},
		// Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		// Democracy: pallet_democracy::{Pallet, Storage, Config<T>, Event<T>, Call},
		// Scheduler: pallet_scheduler::{Pallet, Call, Storage, Config, Event<T>},
	}
}

// pub(crate) fn last_event() -> Event {
// 	System::events().pop().expect("Event expected").event
// }

// pub(crate) fn events() -> Vec<pallet::Event<Test>> {
// 	System::events()
// 		.into_iter()
// 		.map(|r| r.event)
// 		.filter_map(|e| {
// 			if let Event::stake(inner) = e {
// 				Some(inner)
// 			} else {
// 				None
// 			}
// 		})
// 		.collect::<Vec<_>>()
// }

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

#[test]
fn selector_less_than_four_bytes() {
	ExtBuilder::default()
		.build()
		.execute_with(|| {
			// This selector is only three bytes long when four are required.
			let bogus_selector = vec![1u8, 2u8, 3u8];

			// Expected result is an error stating there are too few bytes
			let expected_result = Some(Err(ExitError::Other("input length less than 4 bytes".into())));

			assert_eq!(
				Precompiles::execute(
					precompile_address(),
					&bogus_selector,
					None,
					&evm_test_context(),
				),
				expected_result
			);
		});
}

#[test]
fn no_selector_exists_but_length_is_right() {
	ExtBuilder::default()
		.build()
		.execute_with(|| {
			// This selector is only three bytes long when four are required.
			let bogus_selector = vec![1u8, 2u8, 3u8, 4u8];

			// Expected result is an error stating there are too few bytes
			let expected_result = Some(Err(ExitError::Other("No democracy wrapper method at given selector".into())));

			assert_eq!(
				Precompiles::execute(
					precompile_address(),
					&bogus_selector,
					None,
					&evm_test_context(),
				),
				expected_result
			);
		});
}

#[test]
fn prop_count_zero() {
	ExtBuilder::default()
		.build()
		.execute_with(|| {
			let selector = hex_literal::hex!("56fdf547");

			// Construct data to read prop count
			let mut input_data = Vec::<u8>::from([0u8; 4]);
			input_data[0..4].copy_from_slice(&selector);

			// Expected result is zero. because no props are open yet.
			let expected_zero_result = Some(Ok(PrecompileOutput {
				exit_status: ExitSucceed::Returned,
				output: Vec::from([0u8; 32]),
				cost: Default::default(),
				logs: Default::default(),
			}));

			// Assert that no props have been opened.
			assert_eq!(
				Precompiles::execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
				),
				expected_zero_result
			);
		});
}

// #[test]
// fn prop_count_non_zero()

#[test]
fn prop_count_extra_data() {
	ExtBuilder::default()
		.build()
		.execute_with(|| {
			let selector = hex_literal::hex!("56fdf547");
			
			// Construct data to read prop count including a bogus extra byte
			let mut input_data = Vec::<u8>::from([0u8; 5]);

			// We still use the correct selector for prop_count
			input_data[0..4].copy_from_slice(&selector);

			// Expected result is an error stating there are too few bytes
			let expected_result = Some(Err(ExitError::Other("Incorrect input length for public_prop_count.".into())));

			assert_eq!(
				Precompiles::execute(
					precompile_address(),
					&input_data,
					None,
					&evm_test_context(),
				),
				expected_result
			);
		});
}
