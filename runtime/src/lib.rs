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

//! The Substrate Node Moonbeam runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use sp_std::{prelude::*, marker::PhantomData};
use codec::{Encode, Decode};
use sp_core::{crypto::KeyTypeId, OpaqueMetadata, U256, H160, H256};
use sp_runtime::{
    ApplyExtrinsicResult, generic, create_runtime_str, impl_opaque_keys, MultiSignature,
    transaction_validity::{TransactionValidity, TransactionSource, TransactionPriority},
};
use sp_runtime::traits::{
    BlakeTwo256, Block as BlockT, IdentityLookup, Verify, IdentifyAccount, NumberFor, Saturating, Convert, OpaqueKeys, SaturatedConversion, StaticLookup
};

use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use grandpa::fg_primitives;
use sp_version::RuntimeVersion;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_core::crypto::Public;

// A few exports that help ease life for downstream crates.
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use timestamp::Call as TimestampCall;
pub use balances::Call as BalancesCall;
pub use sp_runtime::{Permill, Perbill};
pub use frame_support::{
    construct_runtime, parameter_types, StorageValue, debug, RuntimeDebug,
    traits::{KeyOwnerProofSystem, Randomness, FindAuthor, Currency, Imbalance, OnUnbalanced, LockIdentifier},
    weights::{
            Weight, IdentityFee,
            constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
    },
    ConsensusEngineId,
};
use system::{EnsureRoot, EnsureOneOf};
use sp_core::{
        u32_trait::{/*_1, _2, */_3, _4},
};
use ethereum::{Block as EthereumBlock, Transaction as EthereumTransaction, Receipt as EthereumReceipt};
use evm::{Account as EVMAccount, FeeCalculator, HashedAddressMapping, EnsureAddressTruncated};
use frontier_rpc_primitives::{TransactionStatus};

#[cfg(any(feature = "std", test))]
pub use staking::StakerStatus;

use sp_runtime::curve::PiecewiseLinear;

pub mod constants;

/// An index to a block.
pub type BlockNumber = constants::time::BlockNumber;
pub type Moment = constants::time::Moment;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = constants::currency::Balance;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
        use super::*;

        pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

        /// Opaque block header type.
        pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
        /// Opaque block type.
        pub type Block = generic::Block<Header, UncheckedExtrinsic>;
        /// Opaque block identifier type.
        pub type BlockId = generic::BlockId<Block>;

        impl_opaque_keys! {
                pub struct SessionKeys {
                        pub aura: Aura,
                        pub grandpa: Grandpa,
                }
        }
}

pub const VERSION: RuntimeVersion = RuntimeVersion {
        spec_name: create_runtime_str!("node-moonbeam"),
        impl_name: create_runtime_str!("node-moonbeam"),
        authoring_version: 1,
        spec_version: 1,
        impl_version: 1,
        apis: RUNTIME_API_VERSIONS,
        transaction_version: 1,
};

pub const MILLISECS_PER_BLOCK: u64 = 6000;

pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
        NativeVersion {
                runtime_version: VERSION,
                can_author_with: Default::default(),
        }
}

parameter_types! {
        pub const BlockHashCount: BlockNumber = 2400;
        /// We allow for 2 seconds of compute with a 6 second average block time.
        pub const MaximumBlockWeight: Weight = 2 * WEIGHT_PER_SECOND;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
        /// Assume 10% of weight for average on_initialize calls.
        pub MaximumExtrinsicWeight: Weight = AvailableBlockRatio::get()
                .saturating_sub(Perbill::from_percent(10)) * MaximumBlockWeight::get();
        pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
        pub const Version: RuntimeVersion = VERSION;
}

// Configure FRAME pallets to include in runtime.

impl system::Trait for Runtime {
        /// The basic call filter to use in dispatchable.
        type BaseCallFilter = ();
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
        /// Maximum weight of each block.
        type MaximumBlockWeight = MaximumBlockWeight;
        /// The weight of database operations that the runtime can invoke.
        type DbWeight = RocksDbWeight;
        /// The weight of the overhead invoked on the block import process, independent of the
        /// extrinsics included in that block.
        type BlockExecutionWeight = BlockExecutionWeight;
        /// The base weight of any extrinsic processed by the runtime, independent of the
        /// logic of that extrinsic. (Signature verification, nonce increment, fee, etc...)
        type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
        /// The maximum weight that a single extrinsic of `Normal` dispatch class can have,
        /// idependent of the logic of that extrinsics. (Roughly max block weight - average on
        /// initialize cost).
        type MaximumExtrinsicWeight = MaximumExtrinsicWeight;
        /// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
        type MaximumBlockLength = MaximumBlockLength;
        /// Portion of the block weight that is available to all normal transactions.
        type AvailableBlockRatio = AvailableBlockRatio;
        /// Version of the runtime.
        type Version = Version;
        /// Converts a module to the index of the module in `construct_runtime!`.
        ///
        /// This type is being generated by `construct_runtime!`.
        type ModuleToIndex = ModuleToIndex;
        /// What to do if a new account is created.
        type OnNewAccount = ();
        /// What to do if an account is fully reaped from the system.
        type OnKilledAccount = ();
        /// The data to be stored in an account.
        type AccountData = balances::AccountData<Balance>;
        /// Weight information for the extrinsics of this pallet.
        type SystemWeightInfo = ();
}

impl aura::Trait for Runtime {
        type AuthorityId = AuraId;
}

impl grandpa::Trait for Runtime {
        type Event = Event;
        type Call = Call;

        type KeyOwnerProofSystem = ();

        type KeyOwnerProof =
                <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

        type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
                KeyTypeId,
                GrandpaId,
        )>>::IdentificationTuple;

        type HandleEquivocation = ();
}

/// Struct that handles the conversion of Balance -> `u64`. This is used for staking's election
/// calculation.
pub struct CurrencyToVoteHandler;

impl CurrencyToVoteHandler {
        fn factor() -> Balance { (Balances::total_issuance() / u64::max_value() as Balance).max(1) }
}

impl Convert<Balance, u64> for CurrencyToVoteHandler {
        fn convert(x: Balance) -> u64 { (x / Self::factor()) as u64 }
}

impl Convert<u128, Balance> for CurrencyToVoteHandler {
        fn convert(x: u128) -> Balance { x * Self::factor() }
}

parameter_types! {
        pub const MinimumPeriod: u64 = constants::time::SLOT_DURATION / 2;
}

impl timestamp::Trait for Runtime {
        /// A timestamp: milliseconds since the unix epoch.
        type Moment = constants::time::Moment;
        type OnTimestampSet = Aura;
        type MinimumPeriod = MinimumPeriod;
        type WeightInfo = ();
}

parameter_types! {
        pub const ExistentialDeposit: u128 = 500;
}

////////////////////// offchain

impl<LocalCall> system::offchain::CreateSignedTransaction<LocalCall> for Runtime where
        Call: From<LocalCall>,
{
        fn create_transaction<C: system::offchain::AppCrypto<Self::Public, Self::Signature>>(
                call: Call,
                public: <Signature as sp_runtime::traits::Verify>::Signer,
                account: AccountId,
                nonce: Index,
        ) -> Option<(Call, <UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload)> {
                // take the biggest period possible.
                let period = BlockHashCount::get()
                        .checked_next_power_of_two()
                        .map(|c| c / 2)
                        .unwrap_or(2) as u64;
                let current_block = System::block_number()
                        .saturated_into::<u64>()
                        // The `System::block_number` is initialized with `n+1`,
                        // so the actual block number is `n`.
                        .saturating_sub(1);
                let tip = 0;
                let extra: SignedExtra = (
                        system::CheckSpecVersion::<Runtime>::new(),
                        system::CheckTxVersion::<Runtime>::new(),
                        system::CheckGenesis::<Runtime>::new(),
                        system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
                        system::CheckNonce::<Runtime>::from(nonce),
                        system::CheckWeight::<Runtime>::new(),
                        transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
                );
                let raw_payload = generic::SignedPayload::new(call, extra).map_err(|e| {
                        debug::warn!("Unable to create signed payload: {:?}", e);
                }).ok()?;
                let signature = raw_payload.using_encoded(|payload| {
                        C::sign(payload, public)
                })?;
                // this is the original implementation
                // let address = Indices::unlookup(account);
                let address = IdentityLookup::unlookup(account);
                let (call, extra, _) = raw_payload.deconstruct();
                Some((call, (address, signature.into(), extra)))
        }
}

impl system::offchain::SigningTypes for Runtime {
        type Public = <Signature as sp_runtime::traits::Verify>::Signer;
        type Signature = Signature;
}

impl<C> system::offchain::SendTransactionTypes<C> for Runtime where
        Call: From<C>,
{
        type Extrinsic = UncheckedExtrinsic;
        type OverarchingCall = Call;
}

//////////////////////

////////////////////// Pallet-Collective instantiation
parameter_types! {
        pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
        pub const CouncilMaxProposals: u32 = 100;
}

type CouncilCollective = collective::Instance1;
impl collective::Trait<CouncilCollective> for Runtime {
        type Origin = Origin;
        type Proposal = Call;
        type Event = Event;
        type MotionDuration = CouncilMotionDuration;
        type MaxProposals = CouncilMaxProposals;
        type WeightInfo = ();
}
//////////////////////

////////////////////// historical instantiation

pub struct SessionBoundariesController
{
}

impl sp_api_hidden_includes_construct_runtime::hidden_include::traits::EstimateNextSessionRotation<u32> for SessionBoundariesController {
    // TODO: impl
    fn estimate_next_session_rotation(_now: BlockNumber) -> Option<BlockNumber> {
        None
    }
    // TODO: impl
    fn weight(_now: BlockNumber) -> Weight {
            // Weight note: `estimate_next_session_rotation` has no storage reads and trivial computational overhead.
            // There should be no risk to the chain having this weight value be zero for now.
            // However, this value of zero was not properly calculated, and so it would be reasonable
            // to come back here and properly calculate the weight of this function.
            0
    }
}

impl pallet_session::ShouldEndSession<u32> for SessionBoundariesController {
//    fn should_end_session(now: BlockNumber) -> bool {
//            let offset = Offset::get();
//            now >= offset && ((now - offset) % Period::get()).is_zero()
//    }

    // TODO: impl
    fn should_end_session(_now: BlockNumber) -> bool {
            false
    }
}

parameter_types! {
        pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl pallet_session::Trait for Runtime {
        type Event = Event;
        type ValidatorId = <Self as system::Trait>::AccountId;
        type ValidatorIdOf = staking::StashOf<Self>;
        type ShouldEndSession = SessionBoundariesController;
        type NextSessionRotation = SessionBoundariesController;
        type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
        type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
        type Keys = opaque::SessionKeys;
        type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
        type WeightInfo = ();
}

impl pallet_session::historical::Trait for Runtime {
        type FullIdentification = staking::Exposure<AccountId, Balance>;
        type FullIdentificationOf = staking::ExposureOf<Runtime>;
}

//////////////////////

////////////////////// Pallet-Staking instantiation

pallet_staking_reward_curve::build! {
        const REWARD_CURVE: PiecewiseLinear<'static> = curve!(
                min_inflation: 0_025_000,
                max_inflation: 0_100_000,
                ideal_stake: 0_500_000,
                falloff: 0_050_000,
                max_piece_count: 40,
                test_precision: 0_005_000,
        );
}

parameter_types! {
        pub const SessionsPerEra: sp_staking::SessionIndex = 6;
        pub const BondingDuration: staking::EraIndex = 24 * 28;
        pub const SlashDeferDuration: staking::EraIndex = 24 * 7; // 1/4 the bonding duration.
        pub const RewardCurve: &'static PiecewiseLinear<'static> = &REWARD_CURVE;
        pub const MaxNominatorRewardedPerValidator: u32 = 64;
        pub const ElectionLookahead: BlockNumber = time::EPOCH_DURATION_IN_BLOCKS / 4;
        pub const MaxIterations: u32 = 10;
        // 0.05%. The higher the value, the more strict solution acceptance becomes.
        pub MinSolutionScoreBump: Perbill = Perbill::from_rational_approximation(5u32, 10_000);
}

impl staking::Trait for Runtime {
        type Currency = Balances;
        type UnixTime = Timestamp;
        type CurrencyToVote = CurrencyToVoteHandler;
        type RewardRemainder = (); // Treasury;
        type Event = Event;
        type Slash = (); // Treasury; // send the slashed funds to the treasury.
        type Reward = (); // rewards are minted from the void
        type SessionsPerEra = SessionsPerEra;
        type BondingDuration = BondingDuration;
        type SlashDeferDuration = SlashDeferDuration;
        /// A super-majority of the council can cancel the slash.
        type SlashCancelOrigin = EnsureOneOf<
                AccountId,
                EnsureRoot<AccountId>,
                collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>
        >;
        type SessionInterface = Self;
        type RewardCurve = RewardCurve;
        type NextNewSession = (); // Session;
        type ElectionLookahead = ElectionLookahead;
        type Call = Call;
        type MaxIterations = MaxIterations;
        type MinSolutionScoreBump = MinSolutionScoreBump;
        type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
        type UnsignedPriority = StakingUnsignedPriority;
        type WeightInfo = ();
}

/// TODO: where do we get this in moonbeam? This was copied from node for staking
pub mod time {
        use super::{Moment, BlockNumber};

        /// Since BABE is probabilistic this is the average expected block time that
        /// we are targetting. Blocks will be produced at a minimum duration defined
        /// by `SLOT_DURATION`, but some slots will not be allocated to any
        /// authority and hence no block will be produced. We expect to have this
        /// block time on average following the defined slot duration and the value
        /// of `c` configured for BABE (where `1 - c` represents the probability of
        /// a slot being empty).
        /// This value is only used indirectly to define the unit constants below
        /// that are expressed in blocks. The rest of the code should use
        /// `SLOT_DURATION` instead (like the Timestamp pallet for calculating the
        /// minimum period).
        ///
        /// If using BABE with secondary slots (default) then all of the slots will
        /// always be assigned, in which case `MILLISECS_PER_BLOCK` and
        /// `SLOT_DURATION` should have the same value.
        ///
        /// <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
        pub const MILLISECS_PER_BLOCK: Moment = 3000;
        pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;

        pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

        // 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
        pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

        pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10 * MINUTES;
        pub const EPOCH_DURATION_IN_SLOTS: u64 = {
                const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

                (EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
        };

        // These time units are defined in number of blocks.
        pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
        pub const HOURS: BlockNumber = MINUTES * 60;
        pub const DAYS: BlockNumber = HOURS * 24;
}

// staking parameters
parameter_types! {
        pub const SessionDuration: BlockNumber = time::EPOCH_DURATION_IN_SLOTS as _;
        pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
        /// We prioritize im-online heartbeats over election solution submission.
        pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
}

//////////////////////

impl balances::Trait for Runtime {
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

impl transaction_payment::Trait for Runtime {
        type Currency = balances::Module<Runtime>;
        type OnTransactionPayment = ();
        type TransactionByteFee = TransactionByteFee;
        type WeightToFee = IdentityFee<Balance>;
        type FeeMultiplierUpdate = ();
}

impl sudo::Trait for Runtime {
        type Event = Event;
        type Call = Call;
}

/// Fixed gas price of `1`.
pub struct FixedGasPrice;

impl FeeCalculator for FixedGasPrice {
        fn min_gas_price() -> U256 {
                // Gas price is always one token per gas.
                1.into()
        }
}

parameter_types! {
        pub const ChainId: u64 = 43;
}

impl evm::Trait for Runtime {
        type FeeCalculator = FixedGasPrice;
        type CallOrigin = EnsureAddressTruncated;
        type WithdrawOrigin = EnsureAddressTruncated;
        type AddressMapping = HashedAddressMapping<BlakeTwo256>;
        type Currency = Balances;
        type Event = Event;
        type Precompiles = ();
        type ChainId = ChainId;
}

pub struct EthereumFindAuthor<F>(PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for EthereumFindAuthor<F>
{
        fn find_author<'a, I>(digests: I) -> Option<H160> where
                I: 'a + IntoIterator<Item=(ConsensusEngineId, &'a [u8])>
        {
                if let Some(author_index) = F::find_author(digests) {
                        let authority_id = Aura::authorities()[author_index as usize].clone();
                        return Some(H160::from_slice(&authority_id.to_raw_vec()[4..24]));
                }
                None
        }
}

impl ethereum::Trait for Runtime {
        type Event = Event;
        type FindAuthor = EthereumFindAuthor<Aura>;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
        pub enum Runtime where
                Block = Block,
                NodeBlock = opaque::Block,
                UncheckedExtrinsic = UncheckedExtrinsic
        {
                System: system::{Module, Call, Config, Storage, Event<T>},
                RandomnessCollectiveFlip: randomness_collective_flip::{Module, Call, Storage},
                Timestamp: timestamp::{Module, Call, Storage, Inherent},
                Aura: aura::{Module, Config<T>, Inherent},
                Grandpa: grandpa::{Module, Call, Storage, Config, Event},
                Balances: balances::{Module, Call, Storage, Config<T>, Event<T>},
                TransactionPayment: transaction_payment::{Module, Storage},
                Sudo: sudo::{Module, Call, Config<T>, Storage, Event<T>},
                Ethereum: ethereum::{Module, Call, Storage, Event, Config, ValidateUnsigned},
                EVM: evm::{Module, Config, Call, Storage, Event<T>},
                Session: pallet_session::{Module, Call, Storage, Event, Config<T>},
                Council: collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
                Historical: pallet_session::{Module},
                Staking: staking::{Module, Call, Config<T>, Storage, Event<T>, ValidateUnsigned},
        }
);

pub struct TransactionConverter;

impl frontier_rpc_primitives::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
        fn convert_transaction(&self, transaction: ethereum::Transaction) -> UncheckedExtrinsic {
                UncheckedExtrinsic::new_unsigned(ethereum::Call::<Runtime>::transact(transaction).into())
        }
}

impl frontier_rpc_primitives::ConvertTransaction<opaque::UncheckedExtrinsic> for TransactionConverter {
        fn convert_transaction(&self, transaction: ethereum::Transaction) -> opaque::UncheckedExtrinsic {
                let extrinsic = UncheckedExtrinsic::new_unsigned(ethereum::Call::<Runtime>::transact(transaction).into());
                let encoded = extrinsic.encode();
                opaque::UncheckedExtrinsic::decode(&mut &encoded[..]).expect("Encoded extrinsic is always valid")
        }
}

/// The address format for describing accounts.
pub type Address = AccountId;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
        system::CheckSpecVersion<Runtime>,
        system::CheckTxVersion<Runtime>,
        system::CheckGenesis<Runtime>,
        system::CheckEra<Runtime>,
        system::CheckNonce<Runtime>,
        system::CheckWeight<Runtime>,
        transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
        frame_executive::Executive<Runtime, Block, system::ChainContext<Runtime>, Runtime, AllModules>;

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
                fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
                        Executive::apply_extrinsic(extrinsic)
                }

                fn finalize_block() -> <Block as BlockT>::Header {
                        Executive::finalize_block()
                }

                fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
                        data.create_extrinsics()
                }

                fn check_inherents(
                        block: Block,
                        data: sp_inherents::InherentData,
                ) -> sp_inherents::CheckInherentsResult {
                        data.check_extrinsics(&block)
                }

                fn random_seed() -> <Block as BlockT>::Hash {
                        RandomnessCollectiveFlip::random_seed()
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

        impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
                fn slot_duration() -> u64 {
                        Aura::slot_duration()
                }

                fn authorities() -> Vec<AuraId> {
                        Aura::authorities()
                }
        }

        impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
                fn account_nonce(account: AccountId) -> Index {
                        System::account_nonce(account)
                }
        }

        impl frontier_rpc_primitives::EthereumRuntimeApi<Block> for Runtime {
                fn chain_id() -> u64 {
                        ChainId::get()
                }

                fn account_basic(address: H160) -> EVMAccount {
                        evm::Module::<Runtime>::account_basic(&address)
                }

                fn gas_price() -> U256 {
                        FixedGasPrice::min_gas_price()
                }

                fn account_code_at(address: H160) -> Vec<u8> {
                        evm::Module::<Runtime>::account_codes(address)
                }

                fn author() -> H160 {
                        <ethereum::Module<Runtime>>::find_author()
                }

                fn storage_at(address: H160, index: U256) -> H256 {
                        let mut tmp = [0u8; 32];
                        index.to_big_endian(&mut tmp);
                        evm::Module::<Runtime>::account_storages(address, H256::from_slice(&tmp[..]))
                }

                fn call(
                        from: H160,
                        data: Vec<u8>,
                        value: U256,
                        gas_limit: U256,
                        gas_price: U256,
                        nonce: Option<U256>,
                        action: ethereum::TransactionAction,
                ) -> Option<(Vec<u8>, U256)> {
                        match action {
                                ethereum::TransactionAction::Call(to) =>
                                        evm::Module::<Runtime>::execute_call(
                                                from,
                                                to,
                                                data,
                                                value,
                                                gas_limit.low_u32(),
                                                gas_price,
                                                nonce,
                                                false,
                                        ).ok().map(|(_, ret, gas)| (ret, gas)),
                                ethereum::TransactionAction::Create =>
                                        evm::Module::<Runtime>::execute_create(
                                                from,
                                                data,
                                                value,
                                                gas_limit.low_u32(),
                                                gas_price,
                                                nonce,
                                                false,
                                        ).ok().map(|(_, _, gas)| (vec![], gas)),
                        }
                }

                fn block_by_number(number: u32) -> (
                        Option<EthereumBlock>, Vec<Option<ethereum::TransactionStatus>>
                ) {
                        if let Some(block) = <ethereum::Module<Runtime>>::block_by_number(number) {
                                let statuses = <ethereum::Module<Runtime>>::block_transaction_statuses(&block);
                                return (
                                        Some(block),
                                        statuses
                                );
                        }
                        (None,vec![])
                }

                fn block_transaction_count_by_number(number: u32) -> Option<U256> {
                        if let Some(block) = <ethereum::Module<Runtime>>::block_by_number(number) {
                                return Some(U256::from(block.transactions.len()))
                        }
                        None
                }

                fn block_transaction_count_by_hash(hash: H256) -> Option<U256> {
                        if let Some(block) = <ethereum::Module<Runtime>>::block_by_hash(hash) {
                                return Some(U256::from(block.transactions.len()))
                        }
                        None
                }

                fn block_by_hash(hash: H256) -> Option<EthereumBlock> {
                        <ethereum::Module<Runtime>>::block_by_hash(hash)
                }

                fn block_by_hash_with_statuses(hash: H256) -> (
                        Option<EthereumBlock>, Vec<Option<ethereum::TransactionStatus>>
                ) {
                        if let Some(block) = <ethereum::Module<Runtime>>::block_by_hash(hash) {
                                let statuses = <ethereum::Module<Runtime>>::block_transaction_statuses(&block);
                                return (
                                        Some(block),
                                        statuses
                                );
                        }
                        (None, vec![])
                }

                fn transaction_by_hash(hash: H256) -> Option<(
                        EthereumTransaction,
                        EthereumBlock,
                        TransactionStatus,
                        Vec<EthereumReceipt>)> {
                        <ethereum::Module<Runtime>>::transaction_by_hash(hash)
                }

                fn transaction_by_block_hash_and_index(hash: H256, index: u32) -> Option<(
                        EthereumTransaction,
                        EthereumBlock,
                        TransactionStatus)> {
                        <ethereum::Module<Runtime>>::transaction_by_block_hash_and_index(hash, index)
                }

                fn transaction_by_block_number_and_index(number: u32, index: u32) -> Option<(
                        EthereumTransaction,
                        EthereumBlock,
                        TransactionStatus)> {
                        <ethereum::Module<Runtime>>::transaction_by_block_number_and_index(
                                number,
                                index
                        )
                }

                fn logs(
                        from_block: Option<u32>,
                        to_block: Option<u32>,
                        block_hash: Option<H256>,
                        address: Option<H160>,
                        topic: Option<Vec<H256>>
                ) -> Vec<(
                        H160, // address
                        Vec<H256>, // topics
                        Vec<u8>, // data
                        Option<H256>, // block_hash
                        Option<U256>, // block_number
                        Option<H256>, // transaction_hash
                        Option<U256>, // transaction_index
                        Option<U256>, // log index in block
                        Option<U256>, // log index in transaction
                )> {
                        let output = <ethereum::Module<Runtime>>::filtered_logs(
                                from_block,
                                to_block,
                                block_hash,
                                address,
                                topic
                        );
                        if let Some(output) = output {
                                return output;
                        }
                        return vec![];
                }
        }

        impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
                Block,
                Balance,
                UncheckedExtrinsic,
        > for Runtime {
                fn query_info(
                        uxt: UncheckedExtrinsic,
                        len: u32
                ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
                        TransactionPayment::query_info(uxt, len)
                }
        }

        impl sp_session::SessionKeys<Block> for Runtime {
                fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
                        opaque::SessionKeys::generate(seed)
                }

                fn decode_session_keys(
                        encoded: Vec<u8>,
                ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
                        opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
                }
        }

        impl fg_primitives::GrandpaApi<Block> for Runtime {
                fn grandpa_authorities() -> GrandpaAuthorityList {
                        Grandpa::grandpa_authorities()
                }

                fn submit_report_equivocation_unsigned_extrinsic(
                        _equivocation_proof: fg_primitives::EquivocationProof<
                                <Block as BlockT>::Hash,
                                NumberFor<Block>,
                        >,
                        _key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
                ) -> Option<()> {
                        None
                }

                fn generate_key_ownership_proof(
                        _set_id: fg_primitives::SetId,
                        _authority_id: GrandpaId,
                ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
                        // NOTE: this is the only implementation possible since we've
                        // defined our key owner proof type as a bottom type (i.e. a type
                        // with no values).
                        None
                }
        }
}
