// Copyright 2019-2020 PureStake Inc.
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

//! Token Factory Mock Runtime
use crate as token_factory;
use frame_support::parameter_types;
use sp_core::{H160, H256};
use sp_io::TestExternalities;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	ModuleId,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances::{Module, Call, Config<T>, Storage, Event<T>},
		Evm: pallet_evm::{Module, Call, Storage, Event<T>},
		TokenFactory: token_factory::{Module, Call, Storage, Event<T>},
	}
);

pub struct BlockEverything;
impl frame_support::traits::Filter<Call> for BlockEverything {
	fn filter(_: &Call) -> bool {
		false
	}
}

parameter_types! {
	pub const MinimumPeriod: u64 = 6000 / 2;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

pub struct FixedGasPrice;
impl pallet_evm::FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> sp_core::U256 {
		1.into()
	}
}

impl pallet_sudo::Config for Test {
	type Event = Event;
	type Call = Call;
}

parameter_types! {
	pub const TransactionByteFee: u64 = 1;
	pub const ChainId: u64 = 42;
	pub const EVMModuleId: ModuleId = ModuleId(*b"py/evmpa");
}

impl pallet_evm::Config for Test {
	type FeeCalculator = FixedGasPrice;
	type GasWeightMapping = ();
	type CallOrigin = pallet_evm::EnsureAddressSame;
	type WithdrawOrigin = pallet_evm::EnsureAddressSame;
	type AddressMapping = pallet_evm::IdentityAddressMapping;
	type Currency = Balances;
	type Event = Event;
	type Precompiles = ();
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type ChainId = ChainId;
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}
impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = ();
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}
type Balance = u64;
type AccountId = H160;
parameter_types! {
	pub const MaxLocks: u32 = 50;
	pub const ExistentialDeposit: u64 = 500;
}
impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}
pub struct AccountToH160;
impl sp_runtime::traits::Convert<H160, H160> for AccountToH160 {
	fn convert(from: H160) -> H160 {
		from
	}
}
impl token_factory::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type TokenId = u8;
	type AccountToH160 = AccountToH160;
}

pub(crate) fn last_event() -> Event {
	System::events().pop().expect("Event expected").event
}

pub(crate) fn root_address() -> H160 {
	use sp_std::str::FromStr;
	H160::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap()
}

pub(crate) fn deploy_address() -> H160 {
	use sp_std::str::FromStr;
	H160::from_str("c2bf5f29a4384b1ab0c063e1c666f02121b6084a").unwrap()
}

pub(crate) fn genesis(balances: Vec<(AccountId, Balance)>) -> TestExternalities {
	let mut storage = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	let genesis = pallet_balances::GenesisConfig::<Test> { balances };
	genesis.assimilate_storage(&mut storage).unwrap();
	let genesis = pallet_sudo::GenesisConfig::<Test> {
		key: root_address(),
	};
	genesis.assimilate_storage(&mut storage).unwrap();
	let mut ext = sp_io::TestExternalities::from(storage);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
