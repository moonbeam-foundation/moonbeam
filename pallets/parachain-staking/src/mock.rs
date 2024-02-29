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
use crate as pallet_parachain_staking;
use crate::{
	pallet, AwardedPts, Config, Event as ParachainStakingEvent, InflationInfo, Points, Range,
	COLLATOR_LOCK_ID, DELEGATOR_LOCK_ID,
};
use block_author::BlockAuthor as BlockAuthorMap;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Everything, Get, LockIdentifier, OnFinalize, OnInitialize},
	weights::{constants::RocksDbWeight, Weight},
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_consensus_slots::Slot;
use sp_core::H256;
use sp_io;
use sp_runtime::BuildStorage;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	Perbill, Percent,
};

pub type AccountId = u64;
pub type Balance = u128;
pub type BlockNumber = BlockNumberFor<Test>;

type Block = frame_system::mocking::MockBlockU32<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		ParachainStaking: pallet_parachain_staking,
		BlockAuthor: block_author,
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const MaximumBlockWeight: Weight = Weight::from_parts(1024, 1);
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type DbWeight = RocksDbWeight;
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
impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 4];
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
impl block_author::Config for Test {}
const GENESIS_BLOCKS_PER_ROUND: BlockNumber = 5;
const GENESIS_COLLATOR_COMMISSION: Perbill = Perbill::from_percent(20);
const GENESIS_PARACHAIN_BOND_RESERVE_PERCENT: Percent = Percent::from_percent(30);
const GENESIS_NUM_SELECTED_CANDIDATES: u32 = 5;
parameter_types! {
	pub const MinBlocksPerRound: u32 = 3;
	pub const MaxOfflineRounds: u32 = 1;
	pub const LeaveCandidatesDelay: u32 = 2;
	pub const CandidateBondLessDelay: u32 = 2;
	pub const LeaveDelegatorsDelay: u32 = 2;
	pub const RevokeDelegationDelay: u32 = 2;
	pub const DelegationBondLessDelay: u32 = 2;
	pub const RewardPaymentDelay: u32 = 2;
	pub const MinSelectedCandidates: u32 = GENESIS_NUM_SELECTED_CANDIDATES;
	pub const MaxTopDelegationsPerCandidate: u32 = 4;
	pub const MaxBottomDelegationsPerCandidate: u32 = 4;
	pub const MaxDelegationsPerDelegator: u32 = 4;
	pub const MinCandidateStk: u128 = 10;
	pub const MinDelegation: u128 = 3;
	pub const MaxCandidates: u32 = 200;
}

pub struct StakingRoundSlotProvider;
impl Get<Slot> for StakingRoundSlotProvider {
	fn get() -> Slot {
		let block_number: u64 = System::block_number().into();
		Slot::from(block_number)
	}
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MonetaryGovernanceOrigin = frame_system::EnsureRoot<AccountId>;
	type MinBlocksPerRound = MinBlocksPerRound;
	type MaxOfflineRounds = MaxOfflineRounds;
	type LeaveCandidatesDelay = LeaveCandidatesDelay;
	type CandidateBondLessDelay = CandidateBondLessDelay;
	type LeaveDelegatorsDelay = LeaveDelegatorsDelay;
	type RevokeDelegationDelay = RevokeDelegationDelay;
	type DelegationBondLessDelay = DelegationBondLessDelay;
	type RewardPaymentDelay = RewardPaymentDelay;
	type MinSelectedCandidates = MinSelectedCandidates;
	type MaxTopDelegationsPerCandidate = MaxTopDelegationsPerCandidate;
	type MaxBottomDelegationsPerCandidate = MaxBottomDelegationsPerCandidate;
	type MaxDelegationsPerDelegator = MaxDelegationsPerDelegator;
	type MinCandidateStk = MinCandidateStk;
	type MinDelegation = MinDelegation;
	type BlockAuthor = BlockAuthor;
	type OnCollatorPayout = ();
	type PayoutCollatorReward = ();
	type OnInactiveCollator = ();
	type OnNewRound = ();
	type SlotProvider = StakingRoundSlotProvider;
	type WeightInfo = ();
	type MaxCandidates = MaxCandidates;
	type SlotsPerYear = frame_support::traits::ConstU32<{ 31_557_600 / 6 }>;
}

pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
	// [collator, amount]
	collators: Vec<(AccountId, Balance)>,
	// [delegator, collator, delegation_amount, auto_compound_percent]
	delegations: Vec<(AccountId, AccountId, Balance, Percent)>,
	// inflation config
	inflation: InflationInfo<Balance>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: vec![],
			delegations: vec![],
			collators: vec![],
			inflation: InflationInfo {
				expect: Range {
					min: 700,
					ideal: 700,
					max: 700,
				},
				// not used
				annual: Range {
					min: Perbill::from_percent(50),
					ideal: Perbill::from_percent(50),
					max: Perbill::from_percent(50),
				},
				// unrealistically high parameterization, only for testing
				round: Range {
					min: Perbill::from_percent(5),
					ideal: Perbill::from_percent(5),
					max: Perbill::from_percent(5),
				},
			},
		}
	}
}

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub(crate) fn with_candidates(mut self, collators: Vec<(AccountId, Balance)>) -> Self {
		self.collators = collators;
		self
	}

	pub(crate) fn with_delegations(
		mut self,
		delegations: Vec<(AccountId, AccountId, Balance)>,
	) -> Self {
		self.delegations = delegations
			.into_iter()
			.map(|d| (d.0, d.1, d.2, Percent::zero()))
			.collect();
		self
	}

	pub(crate) fn with_auto_compounding_delegations(
		mut self,
		delegations: Vec<(AccountId, AccountId, Balance, Percent)>,
	) -> Self {
		self.delegations = delegations;
		self
	}

	#[allow(dead_code)]
	pub(crate) fn with_inflation(mut self, inflation: InflationInfo<Balance>) -> Self {
		self.inflation = inflation;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");
		pallet_parachain_staking::GenesisConfig::<Test> {
			candidates: self.collators,
			delegations: self.delegations,
			inflation_config: self.inflation,
			collator_commission: GENESIS_COLLATOR_COMMISSION,
			parachain_bond_reserve_percent: GENESIS_PARACHAIN_BOND_RESERVE_PERCENT,
			blocks_per_round: GENESIS_BLOCKS_PER_ROUND,
			num_selected_candidates: GENESIS_NUM_SELECTED_CANDIDATES,
		}
		.assimilate_storage(&mut t)
		.expect("Parachain Staking's storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

/// Rolls forward one block. Returns the new block number.
fn roll_one_block() -> BlockNumber {
	Balances::on_finalize(System::block_number());
	System::on_finalize(System::block_number());
	System::set_block_number(System::block_number() + 1);
	System::reset_events();
	System::on_initialize(System::block_number());
	Balances::on_initialize(System::block_number());
	ParachainStaking::on_initialize(System::block_number());
	System::block_number()
}

/// Rolls to the desired block. Returns the number of blocks played.
pub(crate) fn roll_to(n: BlockNumber) -> BlockNumber {
	let mut num_blocks = 0;
	let mut block = System::block_number();
	while block < n {
		block = roll_one_block();
		num_blocks += 1;
	}
	num_blocks
}

/// Rolls desired number of blocks. Returns the final block.
pub(crate) fn roll_blocks(num_blocks: u32) -> BlockNumber {
	let mut block = System::block_number();
	for _ in 0..num_blocks {
		block = roll_one_block();
	}
	block
}

/// Rolls block-by-block to the beginning of the specified round.
/// This will complete the block in which the round change occurs.
/// Returns the number of blocks played.
pub(crate) fn roll_to_round_begin(round: BlockNumber) -> BlockNumber {
	let block = (round - 1) * GENESIS_BLOCKS_PER_ROUND;
	roll_to(block)
}

/// Rolls block-by-block to the end of the specified round.
/// The block following will be the one in which the specified round change occurs.
pub(crate) fn roll_to_round_end(round: BlockNumber) -> BlockNumber {
	let block = round * GENESIS_BLOCKS_PER_ROUND - 1;
	roll_to(block)
}

pub(crate) fn events() -> Vec<pallet::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let RuntimeEvent::ParachainStaking(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
}

/// Asserts that some events were never emitted.
///
/// # Example
///
/// ```
/// assert_no_events!();
/// ```
#[macro_export]
macro_rules! assert_no_events {
	() => {
		similar_asserts::assert_eq!(Vec::<Event<Test>>::new(), crate::mock::events())
	};
}

/// Asserts that emitted events match exactly the given input.
///
/// # Example
///
/// ```
/// assert_events_eq!(
///		Foo { x: 1, y: 2 },
///		Bar { value: "test" },
///		Baz { a: 10, b: 20 },
/// );
/// ```
#[macro_export]
macro_rules! assert_events_eq {
	($event:expr) => {
		similar_asserts::assert_eq!(vec![$event], crate::mock::events());
	};
	($($events:expr,)+) => {
		similar_asserts::assert_eq!(vec![$($events,)+], crate::mock::events());
	};
}

/// Asserts that some emitted events match the given input.
///
/// # Example
///
/// ```
/// assert_events_emitted!(
///		Foo { x: 1, y: 2 },
///		Baz { a: 10, b: 20 },
/// );
/// ```
#[macro_export]
macro_rules! assert_events_emitted {
	($event:expr) => {
		[$event].into_iter().for_each(|e| assert!(
			crate::mock::events().into_iter().find(|x| x == &e).is_some(),
			"Event {:?} was not found in events: \n{:#?}",
			e,
			crate::mock::events()
		));
	};
	($($events:expr,)+) => {
		[$($events,)+].into_iter().for_each(|e| assert!(
			crate::mock::events().into_iter().find(|x| x == &e).is_some(),
			"Event {:?} was not found in events: \n{:#?}",
			e,
			crate::mock::events()
		));
	};
}

/// Asserts that some events were never emitted.
///
/// # Example
///
/// ```
/// assert_events_not_emitted!(
///		Foo { x: 1, y: 2 },
///		Bar { value: "test" },
/// );
/// ```
#[macro_export]
macro_rules! assert_events_not_emitted {
	($event:expr) => {
		[$event].into_iter().for_each(|e| assert!(
			crate::mock::events().into_iter().find(|x| x != &e).is_some(),
			"Event {:?} was unexpectedly found in events: \n{:#?}",
			e,
			crate::mock::events()
		));
	};
	($($events:expr,)+) => {
		[$($events,)+].into_iter().for_each(|e| assert!(
			crate::mock::events().into_iter().find(|x| x != &e).is_some(),
			"Event {:?} was unexpectedly found in events: \n{:#?}",
			e,
			crate::mock::events()
		));
	};
}

/// Asserts that the emitted events are exactly equal to the input patterns.
///
/// # Example
///
/// ```
/// assert_events_eq_match!(
///		Foo { x: 1, .. },
///		Bar { .. },
///		Baz { a: 10, b: 20 },
/// );
/// ```
#[macro_export]
macro_rules! assert_events_eq_match {
	($index:expr;) => {
		assert_eq!(
			$index,
			crate::mock::events().len(),
			"Found {} extra event(s): \n{:#?}",
			crate::mock::events().len()-$index,
			crate::mock::events()
		);
	};
	($index:expr; $event:pat_param, $($events:pat_param,)*) => {
		assert!(
			matches!(
				crate::mock::events().get($index),
				Some($event),
			),
			"Event {:#?} was not found at index {}: \n{:#?}",
			stringify!($event),
			$index,
			crate::mock::events()
		);
		assert_events_eq_match!($index+1; $($events,)*);
	};
	($event:pat_param) => {
		assert_events_eq_match!(0; $event,);
	};
	($($events:pat_param,)+) => {
		assert_events_eq_match!(0; $($events,)+);
	};
}

/// Asserts that some emitted events match the input patterns.
///
/// # Example
///
/// ```
/// assert_events_emitted_match!(
///		Foo { x: 1, .. },
///		Baz { a: 10, b: 20 },
/// );
/// ```
#[macro_export]
macro_rules! assert_events_emitted_match {
	($event:pat_param) => {
		assert!(
			crate::mock::events().into_iter().any(|x| matches!(x, $event)),
			"Event {:?} was not found in events: \n{:#?}",
			stringify!($event),
			crate::mock::events()
		);
	};
	($event:pat_param, $($events:pat_param,)+) => {
		assert_events_emitted_match!($event);
		$(
			assert_events_emitted_match!($events);
		)+
	};
}

/// Asserts that the input patterns match none of the emitted events.
///
/// # Example
///
/// ```
/// assert_events_not_emitted_match!(
///		Foo { x: 1, .. },
///		Baz { a: 10, b: 20 },
/// );
/// ```
#[macro_export]
macro_rules! assert_events_not_emitted_match {
	($event:pat_param) => {
		assert!(
			crate::mock::events().into_iter().any(|x| !matches!(x, $event)),
			"Event {:?} was unexpectedly found in events: \n{:#?}",
			stringify!($event),
			crate::mock::events()
		);
	};
	($event:pat_param, $($events:pat_param,)+) => {
		assert_events_not_emitted_match!($event);
		$(
			assert_events_not_emitted_match!($events);
		)+
	};
}

// Same storage changes as ParachainStaking::on_finalize
pub(crate) fn set_author(round: BlockNumber, acc: u64, pts: u32) {
	<Points<Test>>::mutate(round, |p| *p += pts);
	<AwardedPts<Test>>::mutate(round, acc, |p| *p += pts);
}

// Allows to change the block author (default is always 0)
pub(crate) fn set_block_author(acc: u64) {
	<BlockAuthorMap<Test>>::set(acc);
}

/// fn to query the lock amount
pub(crate) fn query_lock_amount(account_id: u64, id: LockIdentifier) -> Option<Balance> {
	for lock in Balances::locks(&account_id) {
		if lock.id == id {
			return Some(lock.amount);
		}
	}
	None
}

#[test]
fn geneses() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 1000),
			(2, 300),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 9),
			(9, 4),
		])
		.with_candidates(vec![(1, 500), (2, 200)])
		.with_delegations(vec![(3, 1, 100), (4, 1, 100), (5, 2, 100), (6, 2, 100)])
		.build()
		.execute_with(|| {
			assert!(System::events().is_empty());
			// collators
			assert_eq!(
				ParachainStaking::get_collator_stakable_free_balance(&1),
				500
			);
			assert_eq!(query_lock_amount(1, COLLATOR_LOCK_ID), Some(500));
			assert!(ParachainStaking::is_candidate(&1));
			assert_eq!(query_lock_amount(2, COLLATOR_LOCK_ID), Some(200));
			assert_eq!(
				ParachainStaking::get_collator_stakable_free_balance(&2),
				100
			);
			assert!(ParachainStaking::is_candidate(&2));
			// delegators
			for x in 3..7 {
				assert!(ParachainStaking::is_delegator(&x));
				assert_eq!(ParachainStaking::get_delegator_stakable_free_balance(&x), 0);
				assert_eq!(query_lock_amount(x, DELEGATOR_LOCK_ID), Some(100));
			}
			// uninvolved
			for x in 7..10 {
				assert!(!ParachainStaking::is_delegator(&x));
			}
			// no delegator staking locks
			assert_eq!(query_lock_amount(7, DELEGATOR_LOCK_ID), None);
			assert_eq!(
				ParachainStaking::get_delegator_stakable_free_balance(&7),
				100
			);
			assert_eq!(query_lock_amount(8, DELEGATOR_LOCK_ID), None);
			assert_eq!(ParachainStaking::get_delegator_stakable_free_balance(&8), 9);
			assert_eq!(query_lock_amount(9, DELEGATOR_LOCK_ID), None);
			assert_eq!(ParachainStaking::get_delegator_stakable_free_balance(&9), 4);
			// no collator staking locks
			assert_eq!(
				ParachainStaking::get_collator_stakable_free_balance(&7),
				100
			);
			assert_eq!(ParachainStaking::get_collator_stakable_free_balance(&8), 9);
			assert_eq!(ParachainStaking::get_collator_stakable_free_balance(&9), 4);
		});
	ExtBuilder::default()
		.with_balances(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 100),
			(9, 100),
			(10, 100),
		])
		.with_candidates(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_delegations(vec![
			(6, 1, 10),
			(7, 1, 10),
			(8, 2, 10),
			(9, 2, 10),
			(10, 1, 10),
		])
		.build()
		.execute_with(|| {
			assert!(System::events().is_empty());
			// collators
			for x in 1..5 {
				assert!(ParachainStaking::is_candidate(&x));
				assert_eq!(query_lock_amount(x, COLLATOR_LOCK_ID), Some(20));
				assert_eq!(ParachainStaking::get_collator_stakable_free_balance(&x), 80);
			}
			assert!(ParachainStaking::is_candidate(&5));
			assert_eq!(query_lock_amount(5, COLLATOR_LOCK_ID), Some(10));
			assert_eq!(ParachainStaking::get_collator_stakable_free_balance(&5), 90);
			// delegators
			for x in 6..11 {
				assert!(ParachainStaking::is_delegator(&x));
				assert_eq!(query_lock_amount(x, DELEGATOR_LOCK_ID), Some(10));
				assert_eq!(
					ParachainStaking::get_delegator_stakable_free_balance(&x),
					90
				);
			}
		});
}

#[frame_support::pallet]
pub mod block_author {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::Get;

	#[pallet::config]
	pub trait Config: frame_system::Config {}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn block_author)]
	pub(super) type BlockAuthor<T> = StorageValue<_, AccountId, ValueQuery>;

	impl<T: Config> Get<AccountId> for Pallet<T> {
		fn get() -> AccountId {
			<BlockAuthor<T>>::get()
		}
	}
}

#[test]
fn roll_to_round_begin_works() {
	ExtBuilder::default().build().execute_with(|| {
		// these tests assume blocks-per-round of 5, as established by GENESIS_BLOCKS_PER_ROUND
		assert_eq!(System::block_number(), 1); // we start on block 1

		let num_blocks = roll_to_round_begin(1);
		assert_eq!(System::block_number(), 1); // no-op, we're already on this round
		assert_eq!(num_blocks, 0);

		let num_blocks = roll_to_round_begin(2);
		assert_eq!(System::block_number(), 5);
		assert_eq!(num_blocks, 4);

		let num_blocks = roll_to_round_begin(3);
		assert_eq!(System::block_number(), 10);
		assert_eq!(num_blocks, 5);
	});
}

#[test]
fn roll_to_round_end_works() {
	ExtBuilder::default().build().execute_with(|| {
		// these tests assume blocks-per-round of 5, as established by GENESIS_BLOCKS_PER_ROUND
		assert_eq!(System::block_number(), 1); // we start on block 1

		let num_blocks = roll_to_round_end(1);
		assert_eq!(System::block_number(), 4);
		assert_eq!(num_blocks, 3);

		let num_blocks = roll_to_round_end(2);
		assert_eq!(System::block_number(), 9);
		assert_eq!(num_blocks, 5);

		let num_blocks = roll_to_round_end(3);
		assert_eq!(System::block_number(), 14);
		assert_eq!(num_blocks, 5);
	});
}

#[test]
#[should_panic]
fn test_assert_events_eq_fails_if_event_missing() {
	ExtBuilder::default().build().execute_with(|| {
		inject_test_events();

		assert_events_eq!(
			ParachainStakingEvent::CollatorChosen {
				round: 2,
				collator_account: 1,
				total_exposed_amount: 10,
			},
			ParachainStakingEvent::NewRound {
				starting_block: 10,
				round: 2,
				selected_collators_number: 1,
				total_balance: 10,
			},
		);
	});
}

#[test]
#[should_panic]
fn test_assert_events_eq_fails_if_event_extra() {
	ExtBuilder::default().build().execute_with(|| {
		inject_test_events();

		assert_events_eq!(
			ParachainStakingEvent::CollatorChosen {
				round: 2,
				collator_account: 1,
				total_exposed_amount: 10,
			},
			ParachainStakingEvent::NewRound {
				starting_block: 10,
				round: 2,
				selected_collators_number: 1,
				total_balance: 10,
			},
			ParachainStakingEvent::Rewarded {
				account: 1,
				rewards: 100,
			},
			ParachainStakingEvent::Rewarded {
				account: 1,
				rewards: 200,
			},
		);
	});
}

#[test]
#[should_panic]
fn test_assert_events_eq_fails_if_event_wrong_order() {
	ExtBuilder::default().build().execute_with(|| {
		inject_test_events();

		assert_events_eq!(
			ParachainStakingEvent::Rewarded {
				account: 1,
				rewards: 100,
			},
			ParachainStakingEvent::CollatorChosen {
				round: 2,
				collator_account: 1,
				total_exposed_amount: 10,
			},
			ParachainStakingEvent::NewRound {
				starting_block: 10,
				round: 2,
				selected_collators_number: 1,
				total_balance: 10,
			},
		);
	});
}

#[test]
#[should_panic]
fn test_assert_events_eq_fails_if_event_wrong_value() {
	ExtBuilder::default().build().execute_with(|| {
		inject_test_events();

		assert_events_eq!(
			ParachainStakingEvent::CollatorChosen {
				round: 2,
				collator_account: 1,
				total_exposed_amount: 10,
			},
			ParachainStakingEvent::NewRound {
				starting_block: 10,
				round: 2,
				selected_collators_number: 1,
				total_balance: 10,
			},
			ParachainStakingEvent::Rewarded {
				account: 1,
				rewards: 50,
			},
		);
	});
}

#[test]
fn test_assert_events_eq_passes_if_all_events_present_single() {
	ExtBuilder::default().build().execute_with(|| {
		System::deposit_event(ParachainStakingEvent::Rewarded {
			account: 1,
			rewards: 100,
		});

		assert_events_eq!(ParachainStakingEvent::Rewarded {
			account: 1,
			rewards: 100,
		});
	});
}

#[test]
fn test_assert_events_eq_passes_if_all_events_present_multiple() {
	ExtBuilder::default().build().execute_with(|| {
		inject_test_events();

		assert_events_eq!(
			ParachainStakingEvent::CollatorChosen {
				round: 2,
				collator_account: 1,
				total_exposed_amount: 10,
			},
			ParachainStakingEvent::NewRound {
				starting_block: 10,
				round: 2,
				selected_collators_number: 1,
				total_balance: 10,
			},
			ParachainStakingEvent::Rewarded {
				account: 1,
				rewards: 100,
			},
		);
	});
}

#[test]
#[should_panic]
fn test_assert_events_emitted_fails_if_event_missing() {
	ExtBuilder::default().build().execute_with(|| {
		inject_test_events();

		assert_events_emitted!(ParachainStakingEvent::DelegatorExitScheduled {
			round: 2,
			delegator: 3,
			scheduled_exit: 4,
		});
	});
}

#[test]
#[should_panic]
fn test_assert_events_emitted_fails_if_event_wrong_value() {
	ExtBuilder::default().build().execute_with(|| {
		inject_test_events();

		assert_events_emitted!(ParachainStakingEvent::Rewarded {
			account: 1,
			rewards: 50,
		});
	});
}

#[test]
fn test_assert_events_emitted_passes_if_all_events_present_single() {
	ExtBuilder::default().build().execute_with(|| {
		System::deposit_event(ParachainStakingEvent::Rewarded {
			account: 1,
			rewards: 100,
		});

		assert_events_emitted!(ParachainStakingEvent::Rewarded {
			account: 1,
			rewards: 100,
		});
	});
}

#[test]
fn test_assert_events_emitted_passes_if_all_events_present_multiple() {
	ExtBuilder::default().build().execute_with(|| {
		inject_test_events();

		assert_events_emitted!(
			ParachainStakingEvent::CollatorChosen {
				round: 2,
				collator_account: 1,
				total_exposed_amount: 10,
			},
			ParachainStakingEvent::Rewarded {
				account: 1,
				rewards: 100,
			},
		);
	});
}

#[test]
#[should_panic]
fn test_assert_events_eq_match_fails_if_event_missing() {
	ExtBuilder::default().build().execute_with(|| {
		inject_test_events();

		assert_events_eq_match!(
			ParachainStakingEvent::CollatorChosen { .. },
			ParachainStakingEvent::NewRound { .. },
		);
	});
}

#[test]
#[should_panic]
fn test_assert_events_eq_match_fails_if_event_extra() {
	ExtBuilder::default().build().execute_with(|| {
		inject_test_events();

		assert_events_eq_match!(
			ParachainStakingEvent::CollatorChosen { .. },
			ParachainStakingEvent::NewRound { .. },
			ParachainStakingEvent::Rewarded { .. },
			ParachainStakingEvent::Rewarded { .. },
		);
	});
}

#[test]
#[should_panic]
fn test_assert_events_eq_match_fails_if_event_wrong_order() {
	ExtBuilder::default().build().execute_with(|| {
		inject_test_events();

		assert_events_eq_match!(
			ParachainStakingEvent::Rewarded { .. },
			ParachainStakingEvent::CollatorChosen { .. },
			ParachainStakingEvent::NewRound { .. },
		);
	});
}

#[test]
#[should_panic]
fn test_assert_events_eq_match_fails_if_event_wrong_value() {
	ExtBuilder::default().build().execute_with(|| {
		inject_test_events();

		assert_events_eq_match!(
			ParachainStakingEvent::CollatorChosen { .. },
			ParachainStakingEvent::NewRound { .. },
			ParachainStakingEvent::Rewarded { rewards: 50, .. },
		);
	});
}

#[test]
fn test_assert_events_eq_match_passes_if_all_events_present_single() {
	ExtBuilder::default().build().execute_with(|| {
		System::deposit_event(ParachainStakingEvent::Rewarded {
			account: 1,
			rewards: 100,
		});

		assert_events_eq_match!(ParachainStakingEvent::Rewarded { account: 1, .. });
	});
}

#[test]
fn test_assert_events_eq_match_passes_if_all_events_present_multiple() {
	ExtBuilder::default().build().execute_with(|| {
		inject_test_events();

		assert_events_eq_match!(
			ParachainStakingEvent::CollatorChosen {
				round: 2,
				collator_account: 1,
				..
			},
			ParachainStakingEvent::NewRound {
				starting_block: 10,
				..
			},
			ParachainStakingEvent::Rewarded {
				account: 1,
				rewards: 100,
			},
		);
	});
}

#[test]
#[should_panic]
fn test_assert_events_emitted_match_fails_if_event_missing() {
	ExtBuilder::default().build().execute_with(|| {
		inject_test_events();

		assert_events_emitted_match!(ParachainStakingEvent::DelegatorExitScheduled {
			round: 2,
			..
		});
	});
}

#[test]
#[should_panic]
fn test_assert_events_emitted_match_fails_if_event_wrong_value() {
	ExtBuilder::default().build().execute_with(|| {
		inject_test_events();

		assert_events_emitted_match!(ParachainStakingEvent::Rewarded { rewards: 50, .. });
	});
}

#[test]
fn test_assert_events_emitted_match_passes_if_all_events_present_single() {
	ExtBuilder::default().build().execute_with(|| {
		System::deposit_event(ParachainStakingEvent::Rewarded {
			account: 1,
			rewards: 100,
		});

		assert_events_emitted_match!(ParachainStakingEvent::Rewarded { rewards: 100, .. });
	});
}

#[test]
fn test_assert_events_emitted_match_passes_if_all_events_present_multiple() {
	ExtBuilder::default().build().execute_with(|| {
		inject_test_events();

		assert_events_emitted_match!(
			ParachainStakingEvent::CollatorChosen {
				total_exposed_amount: 10,
				..
			},
			ParachainStakingEvent::Rewarded {
				account: 1,
				rewards: 100,
			},
		);
	});
}

fn inject_test_events() {
	[
		ParachainStakingEvent::CollatorChosen {
			round: 2,
			collator_account: 1,
			total_exposed_amount: 10,
		},
		ParachainStakingEvent::NewRound {
			starting_block: 10,
			round: 2,
			selected_collators_number: 1,
			total_balance: 10,
		},
		ParachainStakingEvent::Rewarded {
			account: 1,
			rewards: 100,
		},
	]
	.into_iter()
	.for_each(System::deposit_event);
}
