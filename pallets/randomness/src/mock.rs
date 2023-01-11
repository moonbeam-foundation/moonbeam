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

//! A minimal runtime including the pallet-randomness pallet
use super::*;
use crate as pallet_randomness;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Everything, GenesisBuild},
	weights::Weight,
};
use nimbus_primitives::NimbusId;
use pallet_evm::IdentityAddressMapping;
use session_keys_primitives::VrfId;
use sp_core::{H160, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
};
use sp_std::convert::{TryFrom, TryInto};

pub type AccountId = H160;
pub type Balance = u128;
pub type BlockNumber = u32;

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
		AuthorMapping: pallet_author_mapping::{Pallet, Call, Storage, Config<T>, Event<T>},
		Randomness: pallet_randomness::{Pallet, Call, Storage, Event<T>, Inherent},
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const MaximumBlockWeight: Weight = Weight::from_ref_time(1024);
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
	pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Test {
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
}

parameter_types! {
	pub const DepositAmount: Balance = 100;
}
impl pallet_author_mapping::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type DepositCurrency = Balances;
	type DepositAmount = DepositAmount;
	type Keys = VrfId;
	type WeightInfo = ();
}

pub struct BabeDataGetter;
impl crate::GetBabeData<u64, Option<H256>> for BabeDataGetter {
	fn get_epoch_index() -> u64 {
		1u64
	}
	fn get_epoch_randomness() -> Option<H256> {
		Some(H256::default())
	}
}

parameter_types! {
	pub const Deposit: u128 = 10;
	pub const MaxRandomWords: u8 = 1;
	pub const MinBlockDelay: u32 = 2;
	pub const MaxBlockDelay: u32 = 20;
}
impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type AddressMapping = IdentityAddressMapping;
	type Currency = Balances;
	type BabeDataGetter = BabeDataGetter;
	type VrfKeyLookup = AuthorMapping;
	type Deposit = Deposit;
	type MaxRandomWords = MaxRandomWords;
	type MinBlockDelay = MinBlockDelay;
	type MaxBlockDelay = MaxBlockDelay;
	type BlockExpirationDelay = MaxBlockDelay;
	type EpochExpirationDelay = MaxBlockDelay;
}

pub(crate) fn events() -> Vec<pallet::Event<Test>> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let RuntimeEvent::Randomness(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>()
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

/// Externality builder for pallet randomness mock runtime
pub(crate) struct ExtBuilder {
	/// Balance amounts per AccountId
	balances: Vec<(AccountId, Balance)>,
	/// AuthorId -> AccountId mappings
	mappings: Vec<(NimbusId, AccountId)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			balances: Vec::new(),
			mappings: Vec::new(),
		}
	}
}

impl ExtBuilder {
	#[allow(dead_code)]
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	#[allow(dead_code)]
	pub(crate) fn with_mappings(mut self, mappings: Vec<(NimbusId, AccountId)>) -> Self {
		self.mappings = mappings;
		self
	}

	#[allow(dead_code)]
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		pallet_author_mapping::GenesisConfig::<Test> {
			mappings: self.mappings,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet author mapping's storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub const ALICE: H160 = H160::repeat_byte(0xAA);
pub const BOB: H160 = H160::repeat_byte(0xBB);

/// Helps test same effects for all 4 variants of RequestType
pub fn build_default_request(
	info: RequestType<Test>,
) -> Request<BalanceOf<Test>, RequestType<Test>> {
	Request {
		refund_address: BOB,
		contract_address: ALICE,
		fee: 5,
		gas_limit: 100u64,
		num_words: 1u8,
		salt: H256::default(),
		info,
	}
}
