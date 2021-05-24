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

//! The Moonriver Runtime.
//!
//! Primary features of this runtime include:
//! * Ethereum compatibility
//! * Moonriver tokenomics

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use fp_rpc::TransactionStatus;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{Filter, Get, Imbalance, InstanceFilter, OnUnbalanced},
	weights::{constants::WEIGHT_PER_SECOND, IdentityFee, Weight},
	PalletId,
};
use frame_system::{EnsureOneOf, EnsureRoot};
pub use moonbeam_core_primitives::{
	AccountId, AccountIndex, Address, Balance, BlockNumber, DigestItem, Hash, Header, Index,
	Signature,
};
use moonbeam_extensions_evm::runner::stack::TraceRunner as TraceRunnerT;
use pallet_balances::NegativeImbalance;
use pallet_ethereum::Call::transact;
use pallet_ethereum::{Transaction as EthereumTransaction, TransactionAction};
use pallet_evm::{
	Account as EVMAccount, EnsureAddressNever, EnsureAddressRoot, FeeCalculator,
	IdentityAddressMapping, Runner,
};
use pallet_transaction_payment::CurrencyAdapter;
pub use parachain_staking::{InflationInfo, Range};
use parity_scale_codec::{Decode, Encode};
use sha3::{Digest, Keccak256};
use sp_api::impl_runtime_apis;
use sp_core::{u32_trait::*, OpaqueMetadata, H160, H256, U256};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{BlakeTwo256, Block as BlockT, IdentityLookup},
	transaction_validity::{TransactionSource, TransactionValidity},
	AccountId32, ApplyExtrinsicResult, Perbill, Permill,
};
use sp_std::{convert::TryFrom, prelude::*};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use nimbus_primitives::{CanAuthor, NimbusId};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

/// MOVR, the native token, uses 18 decimals of precision.
pub mod currency {
	use super::Balance;

	pub const MOVR: Balance = 1_000_000_000_000_000_000;
	pub const CENTS: Balance = MOVR / 100;
	pub const GRAND: Balance = MOVR * 1_000;
	pub const MILLICENTS: Balance = CENTS / 1_000;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 100 * CENTS + (bytes as Balance) * 10 * MILLICENTS
	}
}

/// Maximum weight per block
pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

pub const MILLISECS_PER_BLOCK: u64 = 12000;
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;
pub const WEEKS: BlockNumber = DAYS * 7;
/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datastructures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub author_inherent: AuthorInherent,
		}
	}
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("moonriver"),
	impl_name: create_runtime_str!("moonriver"),
	authoring_version: 3,
	spec_version: 38,
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

/// Returns if calls are allowed through the filter
pub struct BaseFilter;
impl Filter<Call> for BaseFilter {
	fn filter(c: &Call) -> bool {
		match c {
			Call::Balances(_) => false,
			Call::CrowdloanRewards(_) => false,
			Call::Democracy(_) => false,
			Call::Ethereum(_) => false,
			Call::EVM(_) => false,
			_ => true,
		}
	}
}

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
	type BaseCallFilter = BaseFilter;
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

pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
where
	R: pallet_balances::Config + pallet_treasury::Config<pallet_treasury::Instance1>,
	pallet_treasury::Module<R, pallet_treasury::Instance1>: OnUnbalanced<NegativeImbalance<R>>,
{
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
		if let Some(fees) = fees_then_tips.next() {
			// for fees, 80% are burned, 20% to the treasury
			let (_, to_treasury) = fees.ration(80, 20);
			// Balances module automatically burns dropped Negative Imbalances by decreasing
			// total_supply accordingly
			<pallet_treasury::Module<R, pallet_treasury::Instance1> as OnUnbalanced<_>>
				::on_unbalanced(to_treasury);
		}
	}
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

impl pallet_ethereum_chain_id::Config for Runtime {}

/// Current approximation of the gas/s consumption considering
/// EVM execution over compiled WASM (on 4.4Ghz CPU).
/// Given the 500ms Weight, from which 75% only are used for transactions,
/// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 ~= 15_000_000.
pub const GAS_PER_SECOND: u64 = 40_000_000;

/// Approximate ratio of the amount of Weight per Gas.
/// u64 works for approximations because Weight is a very small unit compared to gas.
pub const WEIGHT_PER_GAS: u64 = WEIGHT_PER_SECOND / GAS_PER_SECOND;

pub struct MoonbeamGasWeightMapping;

impl pallet_evm::GasWeightMapping for MoonbeamGasWeightMapping {
	fn gas_to_weight(gas: u64) -> Weight {
		gas.saturating_mul(WEIGHT_PER_GAS)
	}
	fn weight_to_gas(weight: Weight) -> u64 {
		u64::try_from(weight.wrapping_div(WEIGHT_PER_GAS)).unwrap_or(u32::MAX as u64)
	}
}

parameter_types! {
	pub BlockGasLimit: U256
		= U256::from(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT / WEIGHT_PER_GAS);
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> U256 {
		1_000_000_000.into()
	}
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = FixedGasPrice;
	type GasWeightMapping = MoonbeamGasWeightMapping;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = IdentityAddressMapping;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type Precompiles = precompiles::MoonbeamPrecompiles<Self>;
	type ChainId = EthereumChainId;
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
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
	/// The maximum amount of time (in blocks) for council members to vote on motions.
	/// Motions may end in fewer blocks if enough votes are cast to determine the result.
	pub const CouncilMotionDuration: BlockNumber = 100;
	/// The maximum number of Proposlas that can be open in the council at once.
	pub const CouncilMaxProposals: u32 = 100;
	/// The maximum number of council members.
	pub const CouncilMaxMembers: u32 = 100;

	/// The maximum amount of time (in blocks) for technical committee members to vote on motions.
	/// Motions may end in fewer blocks if enough votes are cast to determine the result.
	pub const TechComitteeMotionDuration: BlockNumber = 100;
	/// The maximum number of Proposlas that can be open in the technical committee at once.
	pub const TechComitteeMaxProposals: u32 = 100;
	/// The maximum number of technical committee members.
	pub const TechComitteeMaxMembers: u32 = 100;
}

type CouncilInstance = pallet_collective::Instance1;
type TechCommitteeInstance = pallet_collective::Instance2;

impl pallet_collective::Config<CouncilInstance> for Runtime {
	type Origin = Origin;
	type Event = Event;
	type Proposal = Call;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = (); // TODO : Better Weight Info ?
}

impl pallet_collective::Config<TechCommitteeInstance> for Runtime {
	type Origin = Origin;
	type Event = Event;
	type Proposal = Call;
	type MotionDuration = TechComitteeMotionDuration;
	type MaxProposals = TechComitteeMaxProposals;
	type MaxMembers = TechComitteeMaxMembers;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = (); // TODO : Better Weight Info ?
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 1 * DAYS;
	pub const VotingPeriod: BlockNumber = 5 * DAYS;
	pub const FastTrackVotingPeriod: BlockNumber = 1 * DAYS;
	pub const EnactmentPeriod: BlockNumber = 1 *DAYS;
	pub const CooloffPeriod: BlockNumber = 7 * DAYS;
	pub const MinimumDeposit: Balance = 4 * currency::MOVR;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
	pub const PreimageByteDeposit: Balance = currency::MOVR / 1_000;
	pub const InstantAllowed: bool = false;
}

// todo : ensure better origins
impl pallet_democracy::Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin =
		pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilInstance>;
	/// A majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin =
		pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilInstance>;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin =
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilInstance>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin =
		pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, TechCommitteeInstance>;
	/// Instant is currently not allowed.
	type InstantOrigin =
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechCommitteeInstance>;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilInstance>,
	>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechCommitteeInstance>,
	>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cooloff period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechCommitteeInstance>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type Slash = ();
	type InstantAllowed = InstantAllowed;
	type Scheduler = Scheduler;
	type MaxVotes = MaxVotes;
	type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilInstance>;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
	type MaxProposals = MaxProposals;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 1 * currency::MOVR;
	pub const SpendPeriod: BlockNumber = 6 * DAYS;
	pub const CommunityTreasuryId: PalletId = PalletId(*b"pc/trsry");
	pub const ParachainBondPalletId: PalletId = PalletId(*b"pb/trsry");
	pub const MaxApprovals: u32 = 100;
}

type CommunityTreasuryInstance = pallet_treasury::Instance1;
type ParachainBondTreasuryInstance = pallet_treasury::Instance2;

impl pallet_treasury::Config<CommunityTreasuryInstance> for Runtime {
	type PalletId = CommunityTreasuryId;
	type Currency = Balances;
	// Democracy dispatches Root
	type ApproveOrigin = EnsureRoot<AccountId>;
	// Democracy dispatches Root
	type RejectOrigin = EnsureRoot<AccountId>;
	type Event = Event;
	// If spending proposal rejected, transfer proposer bond to treasury
	type OnSlash = CommunityTreasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = ();
	type BurnDestination = ();
	type MaxApprovals = MaxApprovals;
	type WeightInfo = ();
	type SpendFunds = ();
}

impl pallet_treasury::Config<ParachainBondTreasuryInstance> for Runtime {
	type PalletId = ParachainBondPalletId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRoot<AccountId>;
	type RejectOrigin = EnsureRoot<AccountId>;
	type Event = Event;
	// If spending proposal rejected, transfer proposer bond to treasury
	type OnSlash = ParachainBondTreasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = ();
	type BurnDestination = ();
	type MaxApprovals = MaxApprovals;
	type WeightInfo = ();
	type SpendFunds = ();
}

pub struct TransactionConverter;

impl fp_rpc::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(&self, transaction: pallet_ethereum::Transaction) -> UncheckedExtrinsic {
		UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<Runtime>::transact(transaction).into(),
		)
	}
}

impl fp_rpc::ConvertTransaction<opaque::UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(
		&self,
		transaction: pallet_ethereum::Transaction,
	) -> opaque::UncheckedExtrinsic {
		let extrinsic = UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<Runtime>::transact(transaction).into(),
		);
		let encoded = extrinsic.encode();
		opaque::UncheckedExtrinsic::decode(&mut &encoded[..])
			.expect("Encoded extrinsic is always valid")
	}
}

impl pallet_ethereum::Config for Runtime {
	type Event = Event;
	type FindAuthor = pallet_author_mapping::MappedFindAuthor<Self, AuthorInherent>;
	type StateRoot = pallet_ethereum::IntermediateStateRoot;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnValidationData = ();
	type SelfParaId = ParachainInfo;
	type DmpMessageHandler = ();
	type ReservedDmpWeight = ();
	type OutboundXcmpMessageSource = ();
	type XcmpMessageHandler = ();
	type ReservedXcmpWeight = ReservedXcmpWeight;
}

impl parachain_info::Config for Runtime {}

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
	pub const MinCollatorStk: u128 = 1_000 * currency::MOVR;
	/// Minimum stake required to be reserved to be a nominator is 5
	pub const MinNominatorStk: u128 = 5 * currency::MOVR;
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
	type AuthorId = NimbusId;
	type SlotBeacon = pallet_author_inherent::RelayChainBeacon<Self>;
	//TODO This is making me think the mapping should just happen in the author inherent pallet
	// Or maybe the sessions pallet will interface really naturally with the author inherent pallet?
	type EventHandler = pallet_author_mapping::MappedEventHandler<Self, ParachainStaking>;
	type PreliminaryCanAuthor = pallet_author_mapping::MappedCanAuthor<Self, ParachainStaking>;
	type FullCanAuthor = pallet_author_mapping::MappedCanAuthor<Self, AuthorFilter>;
}

impl pallet_author_slot_filter::Config for Runtime {
	// All of our filtering is going to happen in the runtime's accountId type (same as staking.)
	// Maybe I should remove this associated type entirely
	type AuthorId = AccountId;
	type Event = Event;
	type RandomnessSource = RandomnessCollectiveFlip;
	type PotentialAuthors = ParachainStaking;
}

parameter_types! {
	// TODO to be revisited
	pub const VestingPeriod: BlockNumber = 48 * WEEKS;
	pub const MinimumReward: Balance = 0;
	pub const Initialized: bool = false;
	pub const InitializationPayment: Perbill = Perbill::from_percent(20);
}

impl pallet_crowdloan_rewards::Config for Runtime {
	type Event = Event;
	type Initialized = Initialized;
	type InitializationPayment = InitializationPayment;
	type MinimumReward = MinimumReward;
	type RewardCurrency = Balances;
	type RelayChainAccountId = AccountId32;
	type VestingPeriod = VestingPeriod;
}
// This is a simple session key manager. It should probably either work with, or be replaced
// entirely by pallet sessions
impl pallet_author_mapping::Config for Runtime {
	type AuthorId = NimbusId;
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = currency::deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = currency::deposit(0, 33);
	pub const MaxProxies: u16 = 32;
	pub const AnnouncementDepositBase: Balance = currency::deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = currency::deposit(0, 66);
	pub const MaxPending: u16 = 32;
}

/// The type used to represent the kinds of proxying allowed.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug)]
pub enum ProxyType {
	/// All calls can be proxied. This is the trivial/most permissive filter.
	Any,
	/// Only extrinsics that do not transfer funds.
	NonTransfer,
	/// Only extrinsics related to governance (democracy and collectives).
	Governance,
	/// Only extrinsics related to staking.
	Staking,
	/// Allow to veto an announced proxy call.
	CancelProxy,
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

impl InstanceFilter<Call> for ProxyType {
	fn filter(&self, c: &Call) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => matches!(
				c,
				Call::System(..) |
				Call::Timestamp(..) |
				Call::ParachainStaking(..) |
				// Call::Session(..) |
				Call::Democracy(..) |
				Call::CouncilCollective(..) |
				Call::TechComitteeCollective(..) |
				// Call::Treasury(..) |
				Call::Utility(..) |
				Call::Scheduler(..) |
				Call::Proxy(..)
			),
			ProxyType::Governance => matches!(
				c,
				Call::Democracy(..)
					| Call::CouncilCollective(..)
					| Call::TechComitteeCollective(..)
					// | Call::Treasury(..) 
					| Call::Utility(..)
			),
			ProxyType::Staking => matches!(c, Call::ParachainStaking(..) | Call::Utility(..)),
			ProxyType::CancelProxy => {
				matches!(c, Call::Proxy(pallet_proxy::Call::reject_announcement(..)))
			}
		}
	}

	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type WeightInfo = ();
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
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
		EthereumChainId: pallet_ethereum_chain_id::{Pallet, Storage, Config},
		EVM: pallet_evm::{Pallet, Config, Call, Storage, Event<T>},
		Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Config, ValidateUnsigned},
		ParachainStaking: parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>},
		Scheduler: pallet_scheduler::{Pallet, Storage, Config, Event<T>, Call},
		Democracy: pallet_democracy::{Pallet, Storage, Config, Event<T>, Call},
		CouncilCollective:
			pallet_collective::<Instance1>::{Pallet, Call, Event<T>, Origin<T>, Config<T>},
		TechComitteeCollective:
			pallet_collective::<Instance2>::{Pallet, Call, Event<T>, Origin<T>, Config<T>},
		CommunityTreasury: pallet_treasury::<Instance1>::{Pallet, Storage, Config, Event<T>, Call},
		ParachainBondTreasury:
			pallet_treasury::<Instance2>::{Pallet, Storage, Config, Event<T>, Call},
		// The order matters here. Inherents will be included in the order specified here.
		// Concretely we need the author inherent to come after the parachain_system inherent.
		AuthorInherent: pallet_author_inherent::{Pallet, Call, Storage, Inherent},
		AuthorFilter: pallet_author_slot_filter::{Pallet, Storage, Event, Config},
		CrowdloanRewards: pallet_crowdloan_rewards::{Pallet, Call, Config<T>, Storage, Event<T>},
		AuthorMapping: pallet_author_mapping::{Pallet, Config<T>, Storage},
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>},
	}
}

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
	impl moonbeam_rpc_primitives_debug::DebugRuntimeApi<Block> for Runtime {
		fn trace_transaction(
			extrinsics: Vec<<Block as BlockT>::Extrinsic>,
			transaction: &EthereumTransaction,
			trace_type: moonbeam_rpc_primitives_debug::single::TraceType,
		) -> Result<
			moonbeam_rpc_primitives_debug::single::TransactionTrace,
			sp_runtime::DispatchError
		> {
			// Get the caller;
			let mut sig = [0u8; 65];
			let mut msg = [0u8; 32];
			sig[0..32].copy_from_slice(&transaction.signature.r()[..]);
			sig[32..64].copy_from_slice(&transaction.signature.s()[..]);
			sig[64] = transaction.signature.standard_v();
			msg.copy_from_slice(
				&pallet_ethereum::TransactionMessage::from(transaction.clone()).hash()[..]
			);

			let from = match sp_io::crypto::secp256k1_ecdsa_recover(&sig, &msg) {
				Ok(pk) => H160::from(
					H256::from_slice(Keccak256::digest(&pk).as_slice())
				),
				_ => H160::default()
			};

			// Apply the a subset of extrinsics: all the substrate-specific or ethereum transactions
			// that preceded the requested transaction.
			for ext in extrinsics.into_iter() {
				let _ = match &ext.function {
					Call::Ethereum(transact(t)) => {
						if t == transaction {
							break;
						}
						Executive::apply_extrinsic(ext)
					},
					_ => Executive::apply_extrinsic(ext)
				};
			}

			let mut c = <Runtime as pallet_evm::Config>::config().clone();
			c.estimate = true;
			let config = Some(c);

			// Use the runner extension to interface with our evm's trace executor and return the
			// TraceExecutorResult.
			match transaction.action {
				TransactionAction::Call(to) => {
					if let Ok(res) = <Runtime as pallet_evm::Config>::Runner::trace_call(
						from,
						to,
						transaction.input.clone(),
						transaction.value,
						transaction.gas_limit.low_u64(),
						config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
						trace_type,
					) {
						return Ok(res);
					} else {
						return Err(sp_runtime::DispatchError::Other("Evm error"));
					}
				},
				TransactionAction::Create => {
					if let Ok(res) = <Runtime as pallet_evm::Config>::Runner::trace_create(
						from,
						transaction.input.clone(),
						transaction.value,
						transaction.gas_limit.low_u64(),
						config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
						trace_type,
					) {
						return Ok(res);
					} else {
						return Err(sp_runtime::DispatchError::Other("Evm error"));
					}
				}
			}
		}

		fn trace_block(
			extrinsics: Vec<<Block as BlockT>::Extrinsic>,
		) -> Result<
			Vec<
				moonbeam_rpc_primitives_debug::block::TransactionTrace>,
				sp_runtime::DispatchError
			> {
			use moonbeam_rpc_primitives_debug::{single, block, CallResult, CreateResult, CreateType};

			let mut config = <Runtime as pallet_evm::Config>::config().clone();
			config.estimate = true;

			let mut traces = vec![];
			let mut eth_tx_index = 0;

			// Apply all extrinsics. Ethereum extrinsics are traced.
			for ext in extrinsics.into_iter() {
				match &ext.function {
					Call::Ethereum(transact(transaction)) => {
						// Get the caller;
						let mut sig = [0u8; 65];
						let mut msg = [0u8; 32];
						sig[0..32].copy_from_slice(&transaction.signature.r()[..]);
						sig[32..64].copy_from_slice(&transaction.signature.s()[..]);
						sig[64] = transaction.signature.standard_v();
						msg.copy_from_slice(
							&pallet_ethereum::TransactionMessage::from(transaction.clone())
								.hash()[..]
						);

						let from = match sp_io::crypto::secp256k1_ecdsa_recover(&sig, &msg) {
							Ok(pk) => H160::from(
								H256::from_slice(Keccak256::digest(&pk).as_slice())
							),
							_ => H160::default()
						};

						// Use the runner extension to interface with our evm's trace executor and
						// return the TraceExecutorResult.
						let tx_traces = match transaction.action {
							TransactionAction::Call(to) => {
								<Runtime as pallet_evm::Config>::Runner::trace_call(
									from,
									to,
									transaction.input.clone(),
									transaction.value,
									transaction.gas_limit.low_u64(),
									&config,
									single::TraceType::CallList,
								).map_err(|_| sp_runtime::DispatchError::Other("Evm error"))?

							},
							TransactionAction::Create => {
								<Runtime as pallet_evm::Config>::Runner::trace_create(
									from,
									transaction.input.clone(),
									transaction.value,
									transaction.gas_limit.low_u64(),
									&config,
									single::TraceType::CallList,
								).map_err(|_| sp_runtime::DispatchError::Other("Evm error"))?
							}
						};

						let tx_traces = match tx_traces {
							single::TransactionTrace::CallList(t) => t,
							_ => return Err(sp_runtime::DispatchError::Other("Runtime API error")),
						};

						// Convert traces from "single" format to "block" format.
						let mut tx_traces: Vec<_> = tx_traces.into_iter().map(|trace|
							match trace.inner {
								single::CallInner::Call {
									input, to, res, call_type
								} => block::TransactionTrace {
									action: block::TransactionTraceAction::Call {
										call_type,
										from: trace.from,
										gas: trace.gas,
										input,
										to,
										value: trace.value,
									},
									// Can't be known here, must be inserted upstream.
									block_hash: H256::default(),
									// Can't be known here, must be inserted upstream.
									block_number: 0,
									output: match res {
										CallResult::Output(output) => {
											block::TransactionTraceOutput::Result(
												block::TransactionTraceResult::Call {
													gas_used: trace.gas_used,
													output
												})
										},
										CallResult::Error(error) =>
											block::TransactionTraceOutput::Error(error),
									},
									subtraces: trace.subtraces,
									trace_address: trace.trace_address,
									// Can't be known here, must be inserted upstream.
									transaction_hash: H256::default(),
									transaction_position: eth_tx_index,
								},
								single::CallInner::Create { init, res } => block::TransactionTrace {
									action: block::TransactionTraceAction::Create {
										creation_method: CreateType::Create,
										from: trace.from,
										gas: trace.gas,
										init,
										value: trace.value,
									},
									// Can't be known here, must be inserted upstream.
									block_hash: H256::default(),
									// Can't be known here, must be inserted upstream.
									block_number: 0,
									output: match res {
										CreateResult::Success {
											created_contract_address_hash,
											created_contract_code
										} => {
											block::TransactionTraceOutput::Result(
												block::TransactionTraceResult::Create {
													gas_used: trace.gas_used,
													code: created_contract_code,
													address: created_contract_address_hash,
												}
											)
										},
										CreateResult::Error {
											error
										} => block::TransactionTraceOutput::Error(error),
									},
									subtraces: trace.subtraces,
									trace_address: trace.trace_address,
									// Can't be known here, must be inserted upstream.
									transaction_hash: H256::default(),
									transaction_position: eth_tx_index,

								},
								single::CallInner::SelfDestruct {
									balance,
									refund_address
								} => block::TransactionTrace {
									action: block::TransactionTraceAction::Suicide {
										address: from,
										balance,
										refund_address,
									},
									// Can't be known here, must be inserted upstream.
									block_hash: H256::default(),
									// Can't be known here, must be inserted upstream.
									block_number: 0,
									output: block::TransactionTraceOutput::Result(
												block::TransactionTraceResult::Suicide
											),
									subtraces: trace.subtraces,
									trace_address: trace.trace_address,
									// Can't be known here, must be inserted upstream.
									transaction_hash: H256::default(),
									transaction_position: eth_tx_index,

								},
							}
						).collect();

						traces.append(&mut tx_traces);

						eth_tx_index += 1;
					},
					_ => {let _ = Executive::apply_extrinsic(ext); }
				};
			}

			Ok(traces)
		}
	}

	impl moonbeam_rpc_primitives_txpool::TxPoolRuntimeApi<Block> for Runtime {
		fn extrinsic_filter(
			xts: Vec<<Block as BlockT>::Extrinsic>
		) -> Vec<pallet_ethereum::Transaction> {
			xts.into_iter().filter_map(|xt| match xt.function {
				Call::Ethereum(transact(t)) => Some(t),
				_ => None
			}).collect()
		}
	}

	impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
		fn chain_id() -> u64 {
			<Runtime as pallet_evm::Config>::ChainId::get()
		}

		fn account_basic(address: H160) -> EVMAccount {
			EVM::account_basic(&address)
		}

		fn gas_price() -> U256 {
			<Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price()
		}

		fn account_code_at(address: H160) -> Vec<u8> {
			EVM::account_codes(address)
		}

		fn author() -> H160 {
			Ethereum::find_author()
		}

		fn storage_at(address: H160, index: U256) -> H256 {
			let mut tmp = [0u8; 32];
			index.to_big_endian(&mut tmp);
			EVM::account_storages(address, H256::from_slice(&tmp[..]))
		}

		fn call(
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			gas_price: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
		) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			<Runtime as pallet_evm::Config>::Runner::call(
				from,
				to,
				data,
				value,
				gas_limit.low_u64(),
				gas_price,
				nonce,
				config.as_ref().unwrap_or_else(|| <Runtime as pallet_evm::Config>::config()),
			).map_err(|err| err.into())
		}

		fn create(
			from: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			gas_price: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
		) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			#[allow(clippy::or_fun_call)] // suggestion not helpful here
			<Runtime as pallet_evm::Config>::Runner::create(
				from,
				data,
				value,
				gas_limit.low_u64(),
				gas_price,
				nonce,
				config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
			).map_err(|err| err.into())
		}

		fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
			Ethereum::current_transaction_statuses()
		}

		fn current_block() -> Option<pallet_ethereum::Block> {
			Ethereum::current_block()
		}

		fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
			Ethereum::current_receipts()
		}

		fn current_all() -> (
			Option<pallet_ethereum::Block>,
			Option<Vec<pallet_ethereum::Receipt>>,
			Option<Vec<TransactionStatus>>
		) {
			(
				Ethereum::current_block(),
				Ethereum::current_receipts(),
				Ethereum::current_transaction_statuses()
			)
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

	impl nimbus_primitives::AuthorFilterAPI<Block, nimbus_primitives::NimbusId> for Runtime {
		fn can_author(author: nimbus_primitives::NimbusId, slot: u32) -> bool {
			<Runtime as pallet_author_inherent::Config>::FullCanAuthor::can_author(&author, &slot)
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info() -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info()
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
			use pallet_crowdloan_rewards::Pallet as PalletCrowdloanRewardsBench;
			let whitelist: Vec<TrackedStorageKey> = vec![];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, parachain_staking, ParachainStakingBench::<Runtime>);
			add_benchmark!(
				params, batches, pallet_crowdloan_rewards, PalletCrowdloanRewardsBench::<Runtime>
			);
			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

// Notice we're using Nimbus's Executive wrapper to pop (and in the future verify) the seal digest.
cumulus_pallet_parachain_system::register_validate_block!(
	Runtime,
	pallet_author_inherent::BlockExecutor<Runtime, Executive>
);
