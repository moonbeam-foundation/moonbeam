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

use crate::pallet;

use {
	frame_support::{
		construct_runtime, parameter_types,
		traits::{ConstU128, ConstU16, ConstU32, ConstU64, Everything, Hooks},
	},
	num_traits::Num,
	sp_core::H256,
	sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
		Perbill,
	},
	sp_std::convert::{TryFrom, TryInto},
};

pub type AccountId = u64;
pub type Balance = u128;
pub type BlockNumber = u64;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

pub const ACCOUNT_STAKING: u64 = 0;
pub const ACCOUNT_RESERVE: u64 = 1;
pub const ACCOUNT_CANDIDATE_1: u64 = 2;
pub const ACCOUNT_CANDIDATE_2: u64 = 3;
pub const ACCOUNT_DELEGATOR_1: u64 = 4;
pub const ACCOUNT_DELEGATOR_2: u64 = 5;

pub const KILO: u128 = 1000;
pub const MEGA: u128 = 1000 * KILO;
pub const GIGA: u128 = 1000 * MEGA;
pub const TERA: u128 = 1000 * GIGA;
pub const PETA: u128 = 1000 * TERA;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		LiquidStaking: pallet::{Pallet, Call, Storage, Event<T>},
	}
);

impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
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
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Runtime {
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 4];
	type MaxLocks = ();
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub BlockInflation: Perbill = Perbill::from_percent(1); // 1% each block.
	pub RewardsReserveCommission: Perbill = Perbill::from_percent(30);
	pub RewardsCollatorCommission: Perbill = Perbill::from_percent(20);
}

impl pallet::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type Balance = Balance;
	type StakingAccount = ConstU64<{ ACCOUNT_STAKING }>;
	type ReserveAccount = ConstU64<{ ACCOUNT_RESERVE }>;
	type InitialManualClaimShareValue = ConstU128<{ 1 * KILO }>;
	type InitialAutoCompoundingShareValue = ConstU128<{ 1 * KILO }>;
	type LeavingDelay = ConstU64<5>;
	type MinimumSelfDelegation = ConstU128<{ 10 * KILO }>;
	type BlockInflation = BlockInflation;
	type RewardsReserveCommission = RewardsReserveCommission;
	type RewardsCollatorCommission = RewardsCollatorCommission;
}

pub fn balance(who: &AccountId) -> Balance {
	Balances::usable_balance(who)
}

pub fn round_down<T: Num + Copy>(value: T, increment: T) -> T {
	if (value % increment).is_zero() {
		value
	} else {
		(value / increment) * increment
	}
}

pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![
				(ACCOUNT_CANDIDATE_1, 1 * PETA),
				(ACCOUNT_CANDIDATE_2, 1 * PETA),
				(ACCOUNT_DELEGATOR_1, 1 * PETA),
				(ACCOUNT_DELEGATOR_2, 1 * PETA),
			],
		}
	}
}

impl ExtBuilder {
	#[allow(dead_code)]
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

/// Rolls forward one block. Returns the new block number.
#[allow(dead_code)]
pub(crate) fn roll_one_block() -> u64 {
	LiquidStaking::on_finalize(System::block_number());
	Balances::on_finalize(System::block_number());
	System::on_finalize(System::block_number());
	System::set_block_number(System::block_number() + 1);
	System::on_initialize(System::block_number());
	Balances::on_initialize(System::block_number());
	LiquidStaking::on_initialize(System::block_number());
	System::block_number()
}

/// Rolls to the desired block. Returns the number of blocks played.
#[allow(dead_code)]
pub(crate) fn roll_to(n: u64) -> u64 {
	let mut num_blocks = 0;
	let mut block = System::block_number();
	while block < n {
		block = roll_one_block();
		num_blocks += 1;
	}
	num_blocks
}

#[allow(dead_code)]
pub(crate) fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}

#[allow(dead_code)]
pub(crate) fn events() -> Vec<pallet::Event<Runtime>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let Event::LiquidStaking(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}

/// Assert input equal to the last event emitted
#[macro_export]
macro_rules! assert_last_event {
	($event:expr) => {
		match &$event {
			e => assert_eq!(*e, crate::mock::last_event()),
		}
	};
}

/// Compares the system events with passed in events
/// Prints highlighted diff iff assert_eq fails
#[macro_export]
macro_rules! assert_eq_events {
	($events:expr) => {
		match &$events {
			e => similar_asserts::assert_eq!(*e, crate::mock::events()),
		}
	};
}

/// Compares the last N system events with passed in events, where N is the length of events passed
/// in.
///
/// Prints highlighted diff iff assert_eq fails.
/// The last events from frame_system will be taken in order to match the number passed to this
/// macro. If there are insufficient events from frame_system, they will still be compared; the
/// output may or may not be helpful.
///
/// Examples:
/// If frame_system has events [A, B, C, D, E] and events [C, D, E] are passed in, the result would
/// be a successful match ([C, D, E] == [C, D, E]).
///
/// If frame_system has events [A, B, C, D] and events [B, C] are passed in, the result would be an
/// error and a hopefully-useful diff will be printed between [C, D] and [B, C].
///
/// Note that events are filtered to only match parachain-staking (see events()).
#[macro_export]
macro_rules! assert_eq_last_events {
	($events:expr) => {
		assert_tail_eq!($events, crate::mock::events());
	};
}

/// Assert that one array is equal to the tail of the other. A more generic and testable version of
/// assert_eq_last_events.
#[macro_export]
macro_rules! assert_tail_eq {
	($tail:expr, $arr:expr) => {
		if $tail.len() != 0 {
			// 0-length always passes

			if $tail.len() > $arr.len() {
				similar_asserts::assert_eq!($tail, $arr); // will fail
			}

			let len_diff = $arr.len() - $tail.len();
			similar_asserts::assert_eq!($tail, $arr[len_diff..]);
		}
	};
}

/// Panics if an event is not found in the system log of events
#[macro_export]
macro_rules! assert_event_emitted {
	($event:expr) => {
		match &$event {
			e => {
				assert!(
					crate::mock::events().iter().find(|x| *x == e).is_some(),
					"Event {:?} was not found in events: \n {:?}",
					e,
					crate::mock::events()
				);
			}
		}
	};
}

/// Panics if an event is found in the system log of events
#[macro_export]
macro_rules! assert_event_not_emitted {
	($event:expr) => {
		match &$event {
			e => {
				assert!(
					crate::mock::events().iter().find(|x| *x == e).is_none(),
					"Event {:?} was found in events: \n {:?}",
					e,
					crate::mock::events()
				);
			}
		}
	};
}

#[test]
fn assert_tail_eq_works() {
	assert_tail_eq!(vec![1, 2], vec![0, 1, 2]);

	assert_tail_eq!(vec![1], vec![1]);

	assert_tail_eq!(
		vec![0u32; 0], // 0 length array
		vec![0u32; 1]  // 1-length array
	);

	assert_tail_eq!(vec![0u32, 0], vec![0u32, 0]);
}

#[test]
#[should_panic]
fn assert_tail_eq_panics_on_non_equal_tail() {
	assert_tail_eq!(vec![2, 2], vec![0, 1, 2]);
}

#[test]
#[should_panic]
fn assert_tail_eq_panics_on_empty_arr() {
	assert_tail_eq!(vec![2, 2], vec![0u32; 0]);
}

#[test]
#[should_panic]
fn assert_tail_eq_panics_on_longer_tail() {
	assert_tail_eq!(vec![1, 2, 3], vec![1, 2]);
}

#[test]
#[should_panic]
fn assert_tail_eq_panics_on_unequal_elements_same_length_array() {
	assert_tail_eq!(vec![1, 2, 3], vec![0, 1, 2]);
}
