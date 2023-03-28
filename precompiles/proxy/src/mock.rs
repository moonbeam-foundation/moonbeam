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
use crate::{ProxyPrecompile, ProxyPrecompileCall};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Everything, InstanceFilter},
	weights::Weight,
};
use pallet_evm::{EnsureAddressNever, EnsureAddressOrigin, SubstrateBlockHashMapping};
use precompile_utils::{
	precompile_set::{
		AddressU64, CallableByContract, CallableByPrecompile, OnlyFrom, PrecompileAt,
		PrecompileSetBuilder, RevertPrecompile, SubcallWithMaxNesting,
	},
	testing::MockAccount,
};
use scale_info::TypeInfo;
use sp_core::{H160, H256, U256};
use sp_io;
use sp_runtime::codec::{Decode, Encode, MaxEncodedLen};
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};

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
		Evm: pallet_evm::{Pallet, Config, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Proxy: pallet_proxy::{Pallet, Storage, Event<T>, Call},
	}
);

parameter_types! {
	pub const BlockHashCount: u32 = 250;
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
impl pallet_balances::Config for Runtime {
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type MaxLocks = ();
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

pub type Precompiles<R> = PrecompileSetBuilder<
	R,
	(
		PrecompileAt<
			AddressU64<1>,
			ProxyPrecompile<R>,
			(
				SubcallWithMaxNesting<1>,
				CallableByContract<crate::OnlyIsProxyAndProxy<R>>,
				// Batch is the only precompile allowed to call Proxy.
				CallableByPrecompile<OnlyFrom<AddressU64<2>>>,
			),
		>,
		RevertPrecompile<AddressU64<2>>,
	),
>;

pub type PCall = ProxyPrecompileCall<Runtime>;

pub struct EnsureAddressAlways;
impl<OuterOrigin> EnsureAddressOrigin<OuterOrigin> for EnsureAddressAlways {
	type Success = ();

	fn try_address_origin(
		_address: &H160,
		_origin: OuterOrigin,
	) -> Result<Self::Success, OuterOrigin> {
		Ok(())
	}

	fn ensure_address_origin(
		_address: &H160,
		_origin: OuterOrigin,
	) -> Result<Self::Success, sp_runtime::traits::BadOrigin> {
		Ok(())
	}
}

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub PrecompilesValue: Precompiles<Runtime> = Precompiles::new();
	pub const WeightPerGas: Weight = Weight::from_ref_time(1);
}
impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressAlways;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = AccountId;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = Precompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type OnCreate = ();
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

#[repr(u8)]
#[derive(
	Debug, Eq, PartialEq, Ord, PartialOrd, Decode, MaxEncodedLen, Encode, Clone, Copy, TypeInfo,
)]
pub enum ProxyType {
	Any = 0,
	Something = 1,
	Nothing = 2,
}

impl std::default::Default for ProxyType {
	fn default() -> Self {
		ProxyType::Any
	}
}

impl crate::EvmProxyCallFilter for ProxyType {
	fn is_evm_proxy_call_allowed(
		&self,
		_call: &crate::EvmSubCall,
		_recipient_has_code: bool,
	) -> bool {
		match self {
			Self::Any => true,
			Self::Something => true,
			Self::Nothing => false,
		}
	}
}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, _: &RuntimeCall) -> bool {
		true
	}

	fn is_superset(&self, o: &Self) -> bool {
		(*self as u8) > (*o as u8)
	}
}

parameter_types! {
	pub const ProxyDepositBase: u64 = 100;
	pub const ProxyDepositFactor: u64 = 1;
	pub const MaxProxies: u32 = 5;
	pub const MaxPending: u32 = 5;
}
impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type WeightInfo = ();
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = ();
	type AnnouncementDepositFactor = ();
}

/// Build test externalities, prepopulated with data for testing democracy precompiles
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
