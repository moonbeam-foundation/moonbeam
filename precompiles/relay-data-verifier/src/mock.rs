// Copyright 2024 Moonbeam Foundation.
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
use cumulus_primitives_core::relay_chain;
use frame_support::{
	construct_runtime, parameter_types, sp_runtime::traits::IdentityLookup, traits::Everything,
	weights::Weight,
};
use pallet_evm::{EnsureAddressNever, EnsureAddressRoot, SubstrateBlockHashMapping};
use parity_scale_codec::{Decode, Encode};
use precompile_utils::{precompile_set::*, testing::MockAccount};
use sp_core::{Get, U256};
use sp_runtime::{
	traits::{BlakeTwo256, HashingFor},
	BuildStorage,
};
use sp_std::collections::btree_map::BTreeMap;
use sp_trie::PrefixedMemoryDB;
use std::{fs::File, io::Write};

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Balances: pallet_balances,
		Evm: pallet_evm,
		Timestamp: pallet_timestamp,
		RelayStorageRoots: pallet_relay_storage_roots,
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
	type MaxHolds = ();
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

pub type Precompiles<R> =
	PrecompileSetBuilder<R, PrecompileAt<AddressU64<1>, RelayDataVerifierPrecompile<R>>>;

pub type PCall = RelayDataVerifierPrecompileCall<Runtime>;

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
pub fn mock_raw_proof() -> Vec<Vec<u8>> {
	vec![
		"5f07c875e4cff74148e4628f264b974c8040628949b6ef4600c40000000000000000",
		"5f0c9c1284130706f5aea0c8b3d4c54d891501445f020000600200006102000062020000630200006402000065\
		020000660200006702000068020000690200006a0200006b0200006c0200006d0200006e0200006f020000" ,
		"80046480ae6dc31222118597e5aaf1a34a195c282822e9f147bd2113fc611c2d8ad0daaf80106e32398477afa8\
		7370d207850b5c2fcc9edde886bbbff970f30e870cb016ae806eb4916897c8a0f14604da0b634e5102e814e3b35\
		64d52a6a987b3822f35845980ce9727320ca95ab80f36e3f416706757f73bdc4b6a844b54184864cf6f4d3783" ,
		"8061008051c5d06fc458e469b187e464073a9b1a27b78bc92f79e7519439b85509aebe67807c2154d55dc4efdf\
		670330add5144d07ed6efa4bdc6ffae6f1dd5eaa2f429e3080af579d5ddc5c697d42bfc014076594e66c7b324cf\
		d3017810c4e93e4f6f0ae9e",
		"80ffff80a544fa461df3dc9358b0f7f88095a7e37d2037ce25934f9c47956687a94c79d7803413c0780b32567f\
		e87b4b5c073c992f0f50118f44f68ee4cea51bc7d1bc125c8000c1699c8f59a00b69d7034f91cad97e7637a93e3\
		f54984a01ca08c8dc9f9ad080699e1d4c85f1e4e73590d69882f9188db0445e1f6414dd753d69aa4a201ccdfb80\
		e2c14ce9239d367bde39f9625cf2dae689dff77760a6478bb5dc7a28309d95ce809992bee3f46c3be2e44aec660\
		c4a3109d71548441dd8bd4f8dcdeda20c6105f88002c9c0b5dbb322abfe7edfbb9167049d0824d19cab106c6223\
		3d7da53517f8ca80583d87fe18e8d9ed0f9601d98f7614a6f12bdcccbc9e62db443b0753fe1320ab800ab44d080\
		2168f45ff9cff687769b6d4664c8ca1bc94b086df19e000f805d33b801802363d7de5b2d26805f5c86c4ad99384\
		fa61184024cf597e2d65614625050580c161755bb505e8bdb1125229bad3bc41c2ede4dba0789c0c1fa2eac866b\
		bc6d580f697d83a00387c4123875066a7c74c97b09db562d99ce515032da7826564fc2d808ee71cb07ac490d2c0\
		1144fde0f85c784a9e45d1eb50e1fc7f71d414e26894b78090b075ba89594ceb80523aea74a75d35d16810920b3\
		6378e23cb173b408f2749807a57bac6b45c618551ec2afc20378cb9fe2da367249c9fa1975e1c81bd0a641d80a0\
		197196bf1ae5833408f7c6cb410ddaa9d524bfb29f6805a365ca353c19e931",
		"9e261276cc9d1f8598ea4b6a74b15c2f360080888a8ef6d6b18947204b9d2a2caec570f31bcca8de3d62cb3047\
		50bfe750e799802530be352ac1dcc99fe5693df3c6445cdf72b2e3ded3ccd8275851b24fdd8d53505f0e7b90120\
		96b41c4eb3aaf947f6ea42908010080fc6475d793cf00f4eefb53e649aa37823d402f10863ccd12868397067ed2\
		4e16",
		"9ec365c3cf59d671eb72da0e7a4113c41002505f0e7b9012096b41c4eb3aaf947f6ea429080000685f0f1f0515\
		f462cdcf84e0f1d6045dfcbb20c0e413b88d010000",
		"9f09d139e01a5eb2256f222e5fc5dbe6b3581580495b645f9c559f6d1b4047d2b84cdd96247886647e03c12d15\
		3b00247e17bfd2505f0e7b9012096b41c4eb3aaf947f6ea429080000585f0254e9d55588784fa2a62b726696e2b\
		1107002000080595d98af3421f8e2e99d30442ea36735a8047c30975f58d69e9684cfadd26e69805e53a3e74921\
		c6bf8c0e1c24d25a60d10fcbb7fa789d6c2263c568ce01c0aee180298a8183623b166f4e75de0160dc695e2620f\
		96bb4cc5b34a9467ddb937b0b1c",
	]
	.iter()
	.map(|x| hex::decode(x).unwrap())
	.collect()
}

pub fn mocked_read_proof() -> ReadProof {
	// Mock a storage proof obtained from the relay chain using the
	// state_getReadProof RPC call, for the following
	ReadProof {
		at: H256::default(),
		proof: BoundedVec::from(
			mock_raw_proof()
				.iter()
				.map(|x| BoundedBytes::from(x.clone()))
				.collect::<Vec<_>>(),
		),
	}
}

#[derive(Encode, Decode)]
pub struct MockedStorageProof {
	pub proof: Vec<Vec<u8>>,
	pub state_root: H256,
}

impl MockedStorageProof {
	pub fn build(entries: Vec<(Vec<u8>, Vec<u8>)>) -> MockedStorageProof {
		let (db, root) = PrefixedMemoryDB::<HashingFor<relay_chain::Block>>::default_with_root();
		let state_version = Default::default();
		let mut backend = sp_state_machine::TrieBackendBuilder::new(db, root).build();
		let mut relevant_keys = Vec::new();

		entries.into_iter().for_each(|(key, value)| {
			relevant_keys.push(key.clone());
			backend.insert(vec![(None, vec![(key, Some(value))])], state_version);
		});

		let root = *backend.root();
		let proof = sp_state_machine::prove_read(backend, relevant_keys).expect("prove read");
		MockedStorageProof {
			proof: proof.into_iter_nodes().collect(),
			state_root: root,
		}
	}

	pub fn build_benchmark_proof(entries: u32) -> MockedStorageProof {
		let entries = (0..entries)
			.map(|i| {
				let key = format!("key{i}").as_bytes().to_vec();
				let value = vec![i as u8; 128];
				(key, value)
			})
			.collect();
		Self::build(entries)
	}
}

#[test]
fn test_mocked_storage_proof() {
	let mut proofs = BTreeMap::new();
	(1..GetArrayLimit::get()).into_iter().for_each(|x| {
		let proof = MockedStorageProof::build_benchmark_proof(x);
		let proof = proof.encode();
		proofs.insert(x, proof);
	});
	let mut file = File::create(format!("benchmark_proofs")).unwrap();
	file.write_all(&proofs.encode()).unwrap();
}