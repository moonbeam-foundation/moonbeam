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
	traits::{Everything, PollStatus, Polling, TotalIssuanceOf},
	weights::Weight,
};
use pallet_conviction_voting::TallyOf;
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot};
use precompile_utils::{precompile_set::*, testing::MockAccount};
use sp_core::{H256, U256};
use sp_runtime::{
	traits::{BlakeTwo256, ConstU32, IdentityLookup},
	DispatchError, Perbill,
};
use sp_std::collections::btree_map::BTreeMap;

pub type AccountId = MockAccount;
pub type Balance = u128;
pub type BlockNumber = u32;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

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
		ConvictionVoting: pallet_conviction_voting,
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const MaximumBlockWeight: Weight = Weight::from_ref_time(1024);
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
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
	type Lookup = IdentityLookup<AccountId>;
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
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}
parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
}
impl pallet_balances::Config for Runtime {
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 4];
	type MaxLocks = ();
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub PrecompilesValue: Precompiles<Runtime> = Precompiles::new();
	pub const WeightPerGas: Weight = Weight::from_ref_time(1);
}

pub type Precompiles<R> =
	PrecompileSetBuilder<R, (PrecompileAt<AddressU64<1>, ConvictionVotingPrecompile<R>>,)>;

pub type PCall = ConvictionVotingPrecompileCall<Runtime>;

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
	type PrecompilesType = Precompiles<Runtime>;
	type PrecompilesValue = PrecompilesValue;
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TestPollState {
	Ongoing(TallyOf<Runtime>, u8),
	Completed(u64, bool),
}
use TestPollState::*;

parameter_types! {
	pub static Polls: BTreeMap<u8, TestPollState> = vec![
		(1, Completed(1, true)),
		(2, Completed(2, false)),
		(3, Ongoing(Tally::from_parts(0, 0, 0), 0)),
	].into_iter().collect();
}

pub struct TestPolls;
impl Polling<TallyOf<Runtime>> for TestPolls {
	type Index = u8;
	type Votes = u128;
	type Moment = u32;
	type Class = u8;
	fn classes() -> Vec<u8> {
		vec![0, 1, 2]
	}
	fn as_ongoing(index: u8) -> Option<(TallyOf<Runtime>, Self::Class)> {
		Polls::get().remove(&index).and_then(|x| {
			if let TestPollState::Ongoing(t, c) = x {
				Some((t, c))
			} else {
				None
			}
		})
	}
	fn access_poll<R>(
		index: Self::Index,
		f: impl FnOnce(PollStatus<&mut TallyOf<Runtime>, u32, u8>) -> R,
	) -> R {
		let mut polls = Polls::get();
		let entry = polls.get_mut(&index);
		let r = match entry {
			Some(Ongoing(ref mut tally_mut_ref, class)) => {
				f(PollStatus::Ongoing(tally_mut_ref, *class))
			}
			Some(Completed(when, succeeded)) => f(PollStatus::Completed(
				(*when).try_into().unwrap(),
				*succeeded,
			)),
			None => f(PollStatus::None),
		};
		Polls::set(polls);
		r
	}
	fn try_access_poll<R>(
		index: Self::Index,
		f: impl FnOnce(PollStatus<&mut TallyOf<Runtime>, u32, u8>) -> Result<R, DispatchError>,
	) -> Result<R, DispatchError> {
		let mut polls = Polls::get();
		let entry = polls.get_mut(&index);
		let r = match entry {
			Some(Ongoing(ref mut tally_mut_ref, class)) => {
				f(PollStatus::Ongoing(tally_mut_ref, *class))
			}
			Some(Completed(when, succeeded)) => f(PollStatus::Completed(
				(*when).try_into().unwrap(),
				*succeeded,
			)),
			None => f(PollStatus::None),
		}?;
		Polls::set(polls);
		Ok(r)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn create_ongoing(class: Self::Class) -> Result<Self::Index, ()> {
		let mut polls = Polls::get();
		let i = polls.keys().rev().next().map_or(0, |x| x + 1);
		polls.insert(i, Ongoing(Tally::new(0), class));
		Polls::set(polls);
		Ok(i)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn end_ongoing(index: Self::Index, approved: bool) -> Result<(), ()> {
		let mut polls = Polls::get();
		match polls.get(&index) {
			Some(Ongoing(..)) => {}
			_ => return Err(()),
		}
		let now = frame_system::Pallet::<Runtime>::block_number();
		polls.insert(index, Completed(now, approved));
		Polls::set(polls);
		Ok(())
	}
}

impl pallet_conviction_voting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = pallet_balances::Pallet<Self>;
	type VoteLockingPeriod = ConstU32<3>;
	type MaxVotes = ConstU32<3>;
	type WeightInfo = ();
	type MaxTurnout = TotalIssuanceOf<Balances, Self::AccountId>;
	type Polls = TestPolls;
}

pub(crate) struct ExtBuilder {
	/// Endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder { balances: vec![] }
	}
}

impl ExtBuilder {
	/// Fund some accounts before starting the test
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
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
		});
		ext
	}
}

pub(crate) fn events() -> Vec<RuntimeEvent> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.collect::<Vec<_>>()
}
