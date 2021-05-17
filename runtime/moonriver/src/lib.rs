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

//! The Moonbeam Runtime.
//!
//! Primary features of this runtime include:
//! * Ethereum compatibility
//! * Moonbeam tokenomics

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub use moonbeam_core_primitives::{AccountId, Balance, BlockNumber, Hash, Header, Index, Signature};
use frame_support::{
	construct_runtime, parameter_types,
	traits::Randomness,
	weights::{constants::WEIGHT_PER_SECOND, IdentityFee, Weight},
};
use frame_system::EnsureRoot;

use pallet_transaction_payment::CurrencyAdapter;
pub use parachain_staking::{InflationInfo, Range};
use sp_api::impl_runtime_apis;
use sp_core::{OpaqueMetadata, H160, H256, U256};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{BlakeTwo256, Block as BlockT, IdentityLookup},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, Perbill,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// Maximum weight per block
pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datastructures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;

	impl_opaque_keys! {
		pub struct SessionKeys {}
	}
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("moonbeam"),
	impl_name: create_runtime_str!("moonbeam"),
	authoring_version: 3,
	spec_version: 36,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 2,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
	pub const BlockHashCount: BlockNumber = 250;
	pub const Version: RuntimeVersion = VERSION;
	/// We allow for one half second of compute with a 6 second average block time.
	/// These values are dictated by Polkadot for the parachain.
	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
		::with_sensible_defaults(WEIGHT_PER_SECOND / 2, NORMAL_DISPATCH_RATIO);
	/// We allow for 5 MB blocks.
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = IdentityLookup<AccountId>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Maximum weight of each block. With a default weight system of 1byte == 1weight, 4mb is ok.
	type BlockWeights = BlockWeights;
	/// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
	type BlockLength = BlockLength;
	/// Runtime version.
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 1;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const MaxLocks: u32 = 50;
	pub const ExistentialDeposit: u128 = 0;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const TransactionByteFee: Balance = 1;
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}

impl pallet_sudo::Config for Runtime {
	type Call = Call;
	type Event = Event;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) * BlockWeights::get().max_block;
}

impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = ();
	type WeightInfo = ();
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnValidationData = ();
	type SelfParaId = ParachainInfo;
	type DownwardMessageHandlers = ();
	type OutboundXcmpMessageSource = ();
	type XcmpMessageHandler = ();
	type ReservedXcmpWeight = ReservedXcmpWeight;
}

impl parachain_info::Config for Runtime {}

/// GLMR, the native token, uses 18 decimals of precision.
pub const GLMR: Balance = 1_000_000_000_000_000_000;

parameter_types! {
	/// Minimum round length is 2 minutes (20 * 6 second block times)
	pub const MinBlocksPerRound: u32 = 20;
	/// Default BlocksPerRound is every hour (600 * 6 second block times)
	pub const DefaultBlocksPerRound: u32 = 600;
	/// Reward payments and collator exit requests are delayed by 2 hours (2 * 600 * block_time)
	pub const BondDuration: u32 = 2;
	/// Minimum 8 collators selected per round, default at genesis and minimum forever after
	pub const MinSelectedCandidates: u32 = 8;
	/// Maximum 10 nominators per collator
	pub const MaxNominatorsPerCollator: u32 = 10;
	/// Maximum 25 collators per nominator
	pub const MaxCollatorsPerNominator: u32 = 25;
	/// The fixed percent a collator takes off the top of due rewards is 20%
	pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(20);
	/// Minimum stake required to be reserved to be a collator is 1_000
	pub const MinCollatorStk: u128 = 1_000 * GLMR;
	/// Minimum stake required to be reserved to be a nominator is 5
	pub const MinNominatorStk: u128 = 5 * GLMR;
}
impl parachain_staking::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type MinBlocksPerRound = MinBlocksPerRound;
	type DefaultBlocksPerRound = DefaultBlocksPerRound;
	type BondDuration = BondDuration;
	type MinSelectedCandidates = MinSelectedCandidates;
	type MaxNominatorsPerCollator = MaxNominatorsPerCollator;
	type MaxCollatorsPerNominator = MaxCollatorsPerNominator;
	type DefaultCollatorCommission = DefaultCollatorCommission;
	type MinCollatorStk = MinCollatorStk;
	type MinCollatorCandidateStk = MinCollatorStk;
	type MinNomination = MinNominatorStk;
	type MinNominatorStk = MinNominatorStk;
	type WeightInfo = parachain_staking::weights::SubstrateWeight<Runtime>;
}

impl pallet_author_inherent::Config for Runtime {
	type AuthorId = AccountId;
	type EventHandler = ParachainStaking;
	// We cannot run the full filtered author checking logic in the preliminary check because it
	// depends on entropy from the relay chain. Instead we just make sure that the author is staked
	// in the preliminary check. The final check including filtering happens during block execution.
	type PreliminaryCanAuthor = ParachainStaking;
	type FullCanAuthor = AuthorFilter;
}

impl pallet_author_slot_filter::Config for Runtime {
	type AuthorId = AccountId;
	type Event = Event;
	type RandomnessSource = RandomnessCollectiveFlip;
	type PotentialAuthors = ParachainStaking;
}

construct_runtime! {
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Utility: pallet_utility::{Pallet, Call, Event},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Call, Storage},
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>},
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
		ParachainInfo: parachain_info::{Pallet, Storage, Config},
		ParachainStaking: parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>},
		Scheduler: pallet_scheduler::{Pallet, Storage, Config, Event<T>, Call},
		// The order matters here. Inherents will be included in the order specified here.
		// Concretely we need the author inherent to come after the parachain_system inherent.
		AuthorInherent: pallet_author_inherent::{Pallet, Call, Storage, Inherent},
		AuthorFilter: pallet_author_slot_filter::{Pallet, Storage, Event, Config},
	}
}

/// The address format for describing accounts.
pub type Address = AccountId;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various pallets.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPallets,
>;

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(
			extrinsic: <Block as BlockT>::Extrinsic,
		) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(
			data: sp_inherents::InherentData
		) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			RandomnessCollectiveFlip::random_seed().0
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
		for Runtime {

		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}

		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl author_filter_api::AuthorFilterAPI<Block, AccountId> for Runtime {
		fn can_author(author: AccountId, relay_parent: u32) -> bool {
			// Rather than referring to the author filter directly here,
			// refer to it via the author inherent config. This avoid the possibility
			// of accidentally using different filters in different places.
			// This will make more sense when the CanAuthor trait is revised so its method accepts
			// the slot number. Basically what is currently called the "helper" should be the main method.
			AuthorFilter::can_author_helper(&author, relay_parent)
		}
	}

	impl moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block> for Runtime {
		fn trace_transaction(
			_extrinsics: Vec<<Block as BlockT>::Extrinsic>,
			_transaction: &pallet_ethereum::Transaction,
			_trace_type: moonbeam_rpc_primitives_debug::single::TraceType,
		) -> Result<
			moonbeam_rpc_primitives_debug::single::TransactionTrace,
			sp_runtime::DispatchError
		> {
			Err(sp_runtime::DispatchError::Other("TBD"))
		}

		fn trace_block(
			_extrinsics: Vec<<Block as BlockT>::Extrinsic>,
		) -> Result<
			Vec<
				moonbeam_rpc_primitives_debug::block::TransactionTrace>,
				sp_runtime::DispatchError
			> {
			Err(sp_runtime::DispatchError::Other("TBD"))
		}
	}

	impl moonbeam_rpc_primitives_txpool::TxPoolRuntimeApi<Block> for Runtime {
		fn extrinsic_filter(
			_xts: Vec<<Block as BlockT>::Extrinsic>
		) -> Vec<pallet_ethereum::Transaction> {
			Vec::new()
		}
	}

	impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
		fn chain_id() -> u64 {
			0u64
		}

		fn account_basic(_address: H160) -> pallet_evm::Account {
			pallet_evm::Account {
				balance: U256::zero(),
				nonce: U256::zero(),
			}
		}

		fn gas_price() -> U256 {
			U256::default()
		}

		fn account_code_at(_address: H160) -> Vec<u8> {
			Vec::new()
		}

		fn author() -> H160 {
			H160::default()
		}

		fn storage_at(_address: H160, _index: U256) -> H256 {
			H256::default()
		}

		fn call(
			_from: H160,
			_to: H160,
			_data: Vec<u8>,
			_value: U256,
			_gas_limit: U256,
			_gas_price: Option<U256>,
			_nonce: Option<U256>,
			_estimate: bool,
		) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
			Err(sp_runtime::DispatchError::Other("TBD"))
		}

		fn create(
			_from: H160,
			_data: Vec<u8>,
			_value: U256,
			_gas_limit: U256,
			_gas_price: Option<U256>,
			_nonce: Option<U256>,
			_estimate: bool,
		) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
			Err(sp_runtime::DispatchError::Other("TBD"))
		}

		fn current_transaction_statuses() -> Option<Vec<fp_rpc::TransactionStatus>> {
			None
		}

		fn current_block() -> Option<pallet_ethereum::Block> {
			None
		}

		fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
			None
		}

		fn current_all() -> (
			Option<pallet_ethereum::Block>,
			Option<Vec<pallet_ethereum::Receipt>>,
			Option<Vec<fp_rpc::TransactionStatus>>
		) {
			(None, None, None)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
			) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			use parachain_staking::Pallet as ParachainStakingBench;

			let whitelist: Vec<TrackedStorageKey> = vec![];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, parachain_staking, ParachainStakingBench::<Runtime>);
			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

cumulus_pallet_parachain_system::register_validate_block!(Runtime, Executive);
