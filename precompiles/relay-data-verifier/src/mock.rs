// Copyright 2025 Moonbeam Foundation.
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
#![allow(non_camel_case_types)]

use super::*;
use cumulus_pallet_parachain_system::{RelayChainState, RelaychainStateProvider};
use frame_support::{
	construct_runtime, parameter_types,
	sp_runtime::traits::IdentityLookup,
	traits::Everything,
	weights::{RuntimeDbWeight, Weight},
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_evm::{
	EnsureAddressNever, EnsureAddressRoot, FrameSystemAccountProvider, SubstrateBlockHashMapping,
};
use parity_scale_codec::Decode;
use precompile_utils::{precompile_set::*, testing::MockAccount};
use sp_core::{Get, U256};
use sp_runtime::{traits::BlakeTwo256, BuildStorage};

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Balances: pallet_balances,
		Evm: pallet_evm,
		Timestamp: pallet_timestamp,
		RelayStorageRoots: pallet_relay_storage_roots,
		PrecompileBenchmarks: pallet_precompile_benchmarks
	}
);

pub type AccountId = MockAccount;

pub type Balance = u128;
type Block = frame_system::mocking::MockBlockU32<Runtime>;

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type DbWeight = ();
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
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
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
	type RuntimeHoldReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeFreezeReason = ();
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

pub struct PersistedValidationDataGetter;

impl RelaychainStateProvider for PersistedValidationDataGetter {
	fn current_relay_chain_state() -> RelayChainState {
		frame_support::storage::unhashed::get(b"MOCK_PERSISTED_VALIDATION_DATA").unwrap()
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn set_current_relay_chain_state(state: RelayChainState) {
		frame_support::storage::unhashed::put(b"MOCK_PERSISTED_VALIDATION_DATA", &state);
	}
}

pub fn set_current_relay_chain_state(block_number: u32, state_root: H256) {
	let state = RelayChainState {
		number: block_number,
		state_root,
	};
	frame_support::storage::unhashed::put(b"MOCK_PERSISTED_VALIDATION_DATA", &state);
	pallet_relay_storage_roots::Pallet::<Runtime>::set_relay_storage_root();
}

parameter_types! {
	pub const MaxStorageRoots: u32 = 3;
}

impl pallet_relay_storage_roots::Config for Runtime {
	type MaxStorageRoots = MaxStorageRoots;
	type RelaychainStateProvider = PersistedValidationDataGetter;
	type WeightInfo = ();
}

pub struct MockWeightInfo;

impl pallet_precompile_benchmarks::WeightInfo for MockWeightInfo {
	fn verify_entry(x: u32) -> Weight {
		Weight::from_parts(76_430_000, 0)
			.saturating_add(Weight::from_parts(678_469, 0).saturating_mul(x.into()))
	}
	fn latest_relay_block() -> Weight {
		Weight::from_parts(4_641_000, 1606)
			.saturating_add(<() as Get<RuntimeDbWeight>>::get().reads(1_u64))
	}
	fn p256_verify() -> Weight {
		Weight::from_parts(1_580_914_000, 0).saturating_mul(1u64)
	}
}

pub type Precompiles<R> = PrecompileSetBuilder<
	R,
	PrecompileAt<AddressU64<1>, RelayDataVerifierPrecompile<R, MockWeightInfo>>,
>;

pub type PCall = RelayDataVerifierPrecompileCall<Runtime, MockWeightInfo>;

const MAX_POV_SIZE: u64 = 5 * 1024 * 1024;
/// Block storage limit in bytes. Set to 40 KB.
const BLOCK_STORAGE_LIMIT: u64 = 40 * 1024;

parameter_types! {
	pub BlockGasLimit: U256 = U256::from(u64::MAX);
	pub PrecompilesValue: Precompiles<Runtime> = Precompiles::new();
	pub const WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub GasLimitPovSizeRatio: u64 = {
		let block_gas_limit = BlockGasLimit::get().min(u64::MAX.into()).low_u64();
		block_gas_limit.saturating_div(MAX_POV_SIZE)
	};
	pub GasLimitStorageGrowthRatio: u64 = {
		let block_gas_limit = BlockGasLimit::get().min(u64::MAX.into()).low_u64();
		block_gas_limit.saturating_div(BLOCK_STORAGE_LIMIT)
	};
}

pub struct RandomnessProvider;
impl
	frame_support::traits::Randomness<
		<Runtime as frame_system::Config>::Hash,
		BlockNumberFor<Runtime>,
	> for RandomnessProvider
{
	fn random(
		subject: &[u8],
	) -> (
		<Runtime as frame_system::Config>::Hash,
		BlockNumberFor<Runtime>,
	) {
		let output = <Runtime as frame_system::Config>::Hashing::hash(subject);
		let block_number = frame_system::Pallet::<Runtime>::block_number();
		(output, block_number)
	}
}

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
	type PrecompilesValue = PrecompilesValue;
	type PrecompilesType = Precompiles<Self>;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type OnCreate = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type SuicideQuickClearLimit = ConstU32<0>;
	type GasLimitStorageGrowthRatio = GasLimitStorageGrowthRatio;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Runtime>;
	type AccountProvider = FrameSystemAccountProvider<Runtime>;
	type RandomnessProvider = RandomnessProvider;
}

impl pallet_precompile_benchmarks::Config for Runtime {
	type WeightInfo = MockWeightInfo;
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
		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
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

pub fn fill_relay_storage_roots<T: pallet_relay_storage_roots::Config>() {
	(1..=T::MaxStorageRoots::get()).for_each(|i| {
		set_current_relay_chain_state(i, H256::default());
		pallet_relay_storage_roots::Pallet::<T>::set_relay_storage_root();
	})
}

// Storage Root: 767caa877bcea0d34dd515a202b75efa41bffbc9f814ab59e2c1c96716d4c65d
pub const STORAGE_ROOT: &[u8] = &[
	118, 124, 170, 135, 123, 206, 160, 211, 77, 213, 21, 162, 2, 183, 94, 250, 65, 191, 251, 201,
	248, 20, 171, 89, 226, 193, 201, 103, 22, 212, 198, 93,
];

// Timestamp key: f0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb
pub const TIMESTAMP_KEY: &[u8] = &[
	240, 195, 101, 195, 207, 89, 214, 113, 235, 114, 218, 14, 122, 65, 19, 196, 159, 31, 5, 21,
	244, 98, 205, 207, 132, 224, 241, 214, 4, 93, 252, 187,
];

// Total Issuance Key: c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80
pub const TOTAL_ISSUANCE_KEY: &[u8] = &[
	194, 38, 18, 118, 204, 157, 31, 133, 152, 234, 75, 106, 116, 177, 92, 47, 87, 200, 117, 228,
	207, 247, 65, 72, 228, 98, 143, 38, 75, 151, 76, 128,
];

// Treasury Approval Key: 89d139e01a5eb2256f222e5fc5dbe6b33c9c1284130706f5aea0c8b3d4c54d89
pub const TREASURY_APPROVALS_KEY: &[u8] = &[
	137, 209, 57, 224, 26, 94, 178, 37, 111, 34, 46, 95, 197, 219, 230, 179, 60, 156, 18, 132, 19,
	7, 6, 245, 174, 160, 200, 179, 212, 197, 77, 137,
];

// Mock a storage proof obtained from the relay chain using the
// state_getReadProof RPC call, for the following keys:
// TimeStamp:
// 0xf0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb
// Balances (Total Issuance):
// 0xc2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80
// Treasury Approvals
// 0x89d139e01a5eb2256f222e5fc5dbe6b33c9c1284130706f5aea0c8b3d4c54d89
// at Block Hash:
// 0x1272470f226fc0e955838262e8dd17a7d7bad6563739cc53a3b1744ddf0ea872

pub fn mocked_read_proof() -> ReadProof {
	// Mock a storage proof obtained from the relay chain using the
	// state_getReadProof RPC call, for the following
	let proof: Vec<Vec<u8>> = Vec::decode(&mut &include_bytes!("../proof").to_vec()[..]).unwrap();

	ReadProof {
		at: H256::default(),
		proof: BoundedVec::from(
			proof
				.iter()
				.map(|x| BoundedBytes::from(x.clone()))
				.collect::<Vec<_>>(),
		),
	}
}
