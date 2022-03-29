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

//! A minimal runtime including the vrf pallet
use crate::{AuthorityId, GetMostRecentVrfInputs, RoundChangedThisBlock, Slot};
use frame_support::{
	construct_runtime, pallet_prelude::*, parameter_types, traits::Everything, weights::Weight,
};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::{ByteArray, H256};
use sp_io;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, Convert, IdentityLookup},
	Perbill, RuntimeDebug,
};

pub type AccountId = u64;
pub type Balance = u128;
pub type BlockNumber = u64;

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
		Vrf: crate::{Pallet, Storage},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
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
	type SS58Prefix = ();
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
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

pub struct RoundNeverChanges;
impl RoundChangedThisBlock for RoundNeverChanges {
	fn round_changed_this_block() -> bool {
		false
	}
}

pub struct HardcodedSelectedCandidates;
impl Get<Vec<AccountId>> for HardcodedSelectedCandidates {
	fn get() -> Vec<AccountId> {
		vec![1, 2, 3]
	}
}

pub struct MostRecentVrfInputGetter;
impl GetMostRecentVrfInputs<H256, Slot> for MostRecentVrfInputGetter {
	fn get_most_recent_relay_block_hash() -> H256 {
		H256::default()
	}
	fn get_most_recent_relay_slot_number() -> Slot {
		Slot::default()
	}
}

pub struct AccountToVrfIdConverter;
impl Convert<AccountId, AuthorityId> for AccountToVrfIdConverter {
	fn convert(from: AccountId) -> AuthorityId {
		todo!()
	}
}

parameter_types! {
	pub const DepositAmount: Balance = 100;
}
impl crate::Config for Runtime {
	/// The relay block hash type (probably H256)
	type RelayBlockHash = H256;
	/// Gets the most recent relay block hash and relay slot number in `on_initialize`
	type MostRecentVrfInputGetter = MostRecentVrfInputGetter;
	/// Convert account to VRF key, presumably by a AuthorMapping pallet instance
	type AccountToVrfId = AccountToVrfIdConverter;
	/// Round never changes
	type RoundChanged = RoundNeverChanges;
	/// Get the selected candidate accounts from staking
	type SelectedCandidates = HardcodedSelectedCandidates;
}

/// Externality builder for pallet author mapping's mock runtime
/// Allows configuring balances and initial mappings
pub(crate) struct ExtBuilder {
	/// Accounts endowed with balances
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

pub(crate) fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}
