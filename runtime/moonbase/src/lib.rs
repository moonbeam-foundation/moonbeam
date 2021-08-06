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

//! The Moonbase Runtime.
//!
//! Primary features of this runtime include:
//! * Ethereum compatibility
//! * Moonbase tokenomics

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use cumulus_pallet_parachain_system::RelaychainBlockNumberProvider;
use fp_rpc::TransactionStatus;
use frame_support::{
	construct_runtime, match_type, parameter_types,
	traits::{All, Get, Imbalance, InstanceFilter, OnUnbalanced, OriginTrait},
	weights::{
		constants::{RocksDbWeight, WEIGHT_PER_SECOND},
		IdentityFee, Weight,
	},
	PalletId,
};
use frame_system::{EnsureOneOf, EnsureRoot};
pub use moonbeam_core_primitives::{
	AccountId, AccountIndex, Address, Balance, BlockNumber, DigestItem, Hash, Header, Index,
	Signature,
};
use moonbeam_rpc_primitives_txpool::TxPoolResponse;
use pallet_balances::NegativeImbalance;
use pallet_ethereum::Call::transact;
use pallet_ethereum::Transaction as EthereumTransaction;
use pallet_evm::{
	Account as EVMAccount, EnsureAddressNever, EnsureAddressRoot, FeeCalculator,
	IdentityAddressMapping, Runner,
};
use pallet_transaction_payment::{CurrencyAdapter, Multiplier, TargetedFeeAdjustment};
pub use parachain_staking::{InflationInfo, Range};
use parity_scale_codec::{Decode, Encode};
use sp_api::impl_runtime_apis;
use sp_core::{u32_trait::*, OpaqueMetadata, H160, H256, U256};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdConversion, BlakeTwo256, Block as BlockT, IdentifyAccount, IdentityLookup},
	transaction_validity::{TransactionSource, TransactionValidity},
	AccountId32, ApplyExtrinsicResult, FixedPointNumber, Perbill, Percent, Permill, Perquintill,
};
use sp_std::marker::PhantomData;
use xcm::v0::{BodyId, Junction, MultiAsset, MultiLocation, NetworkId, Xcm};

use sp_std::{
	convert::{TryFrom, TryInto},
	prelude::*,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use nimbus_primitives::{CanAuthor, NimbusId};

mod precompiles;
use precompiles::MoonbasePrecompiles;

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

pub type Precompiles = MoonbasePrecompiles<Runtime>;

/// UNIT, the native token, uses 18 decimals of precision.
pub mod currency {
	use super::Balance;

	pub const UNIT: Balance = 1_000_000_000_000_000_000;
	pub const KILOUNIT: Balance = UNIT * 1_000;
	pub const MILLIUNIT: Balance = UNIT / 1_000;
	pub const MICROUNIT: Balance = MILLIUNIT / 1_000;
	pub const NANOUNIT: Balance = MICROUNIT / 1_000;
	pub const CENTS: Balance = UNIT / 30_000;
	pub const MILLICENTS: Balance = CENTS / 1_000;
	pub const BYTE_FEE: Balance = 100 * MICROUNIT;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 1 * UNIT + (bytes as Balance) * BYTE_FEE
	}

	pub const fn relay_deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 2_000 * CENTS + (bytes as Balance) * 100 * MILLICENTS
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
			pub nimbus: AuthorInherent,
		}
	}
}

/// This runtime version.
/// The spec_version is composed of 2x2 digits. The first 2 digits represent major changes
/// that can't be skipped, such as data migration upgrades. The last 2 digits represent minor
/// changes which can be skipped.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("moonbase"),
	impl_name: create_runtime_str!("moonbase"),
	authoring_version: 3,
	spec_version: 0155,
	impl_version: 0,
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
	pub const BlockHashCount: BlockNumber = 256;
	pub const Version: RuntimeVersion = VERSION;
	/// We allow for one half second of compute with a 6 second average block time.
	/// These values are dictated by Polkadot for the parachain.
	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
		::with_sensible_defaults(MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO);
	/// We allow for 5 MB blocks.
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u16 = 1287;
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
	type DbWeight = RocksDbWeight;
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 1;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
	pub const ExistentialDeposit: u128 = 0;
}

impl pallet_balances::Config for Runtime {
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 4];
	type MaxLocks = MaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

type KsmInstance = pallet_balances::Instance1;

// Virtual-KSM balances.
impl pallet_balances::Config<KsmInstance> for Runtime {
	type MaxLocks = MaxLocks;
	type ReserveIdentifier = [u8; 4];
	type MaxReserves = MaxReserves;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	// This uses the pallets internal storage
	// TODO: explain this (Storedmap, etc..)
	type AccountStore = frame_support::traits::StorageMapShim<
		pallet_balances::Account<Runtime, KsmInstance>,
		(),
		AccountId,
		pallet_balances::AccountData<Balance>,
	>;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
where
	R: pallet_balances::Config + pallet_treasury::Config,
	pallet_treasury::Module<R>: OnUnbalanced<NegativeImbalance<R>>,
{
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
		if let Some(fees) = fees_then_tips.next() {
			// for fees, 80% are burned, 20% to the treasury
			let (_, to_treasury) = fees.ration(80, 20);
			// Balances module automatically burns dropped Negative Imbalances by decreasing
			// total_supply accordingly
			<pallet_treasury::Module<R> as OnUnbalanced<_>>::on_unbalanced(to_treasury);
		}
	}
}

parameter_types! {
	pub const TransactionByteFee: Balance = currency::BYTE_FEE;
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees<Runtime>>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Runtime>;
}

impl pallet_ethereum_chain_id::Config for Runtime {}

impl pallet_randomness_collective_flip::Config for Runtime {}

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
	/// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
	/// than this will decrease the weight and more will increase.
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	/// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
	/// change the fees more rapidly. This low value causes changes to occur slowly over time.
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
	/// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
	/// that combined with `AdjustmentVariable`, we can recover from the minimum.
	/// See `multiplier_can_grow_from_zero` in integration_tests.rs.
	/// This value is currently only used by pallet-transaction-payment as an assertion that the
	/// next multiplier is always > min value.
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000u128);
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> U256 {
		(1 * currency::NANOUNIT).into()
	}
}

/// Parameterized slow adjusting fee updated based on
/// https://w3f-research.readthedocs.io/en/latest/polkadot/overview/2-token-economics.html#-2.-slow-adjusting-mechanism // editorconfig-checker-disable-line
///
/// The adjustment algorithm boils down to:
///
/// diff = (previous_block_weight - target) / maximum_block_weight
/// next_multiplier = prev_multiplier * (1 + (v * diff) + ((v * diff)^2 / 2))
/// assert(next_multiplier > min)
///     where: v is AdjustmentVariable
///            target is TargetBlockFullness
///            min is MinimumMultiplier
pub type SlowAdjustingFeeUpdate<R> =
	TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;

impl pallet_evm::Config for Runtime {
	type FeeCalculator = FixedGasPrice;
	type GasWeightMapping = MoonbeamGasWeightMapping;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = IdentityAddressMapping;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type Precompiles = MoonbasePrecompiles<Self>;
	type ChainId = EthereumChainId;
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = NORMAL_DISPATCH_RATIO * BlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	/// The maximum amount of time (in blocks) for council members to vote on motions.
	/// Motions may end in fewer blocks if enough votes are cast to determine the result.
	pub const CouncilMotionDuration: BlockNumber = 3 * DAYS;
	/// The maximum number of Proposlas that can be open in the council at once.
	pub const CouncilMaxProposals: u32 = 100;
	/// The maximum number of council members.
	pub const CouncilMaxMembers: u32 = 100;

	/// The maximum amount of time (in blocks) for technical committee members to vote on motions.
	/// Motions may end in fewer blocks if enough votes are cast to determine the result.
	pub const TechComitteeMotionDuration: BlockNumber = 3 * DAYS;
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
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

impl pallet_collective::Config<TechCommitteeInstance> for Runtime {
	type Origin = Origin;
	type Event = Event;
	type Proposal = Call;
	type MotionDuration = TechComitteeMotionDuration;
	type MaxProposals = TechComitteeMaxProposals;
	type MaxMembers = TechComitteeMaxMembers;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 1 * DAYS;
	pub const VotingPeriod: BlockNumber = 5 * DAYS;
	pub const FastTrackVotingPeriod: BlockNumber = 4 * HOURS;
	pub const EnactmentPeriod: BlockNumber = 1 *DAYS;
	pub const CooloffPeriod: BlockNumber = 7 * DAYS;
	pub const MinimumDeposit: Balance = 4 * currency::UNIT;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
	pub const PreimageByteDeposit: Balance = currency::BYTE_FEE;
	pub const InstantAllowed: bool = true;
}

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
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
	type MaxProposals = MaxProposals;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 1 * currency::UNIT;
	pub const SpendPeriod: BlockNumber = 6 * DAYS;
	pub const TreasuryId: PalletId = PalletId(*b"pc/trsry");
	pub const MaxApprovals: u32 = 100;
}

type TreasuryApproveOrigin = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, CouncilInstance>,
>;

type TreasuryRejectOrigin = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilInstance>,
>;

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryId;
	type Currency = Balances;
	// At least three-fifths majority of the council is required (or root) to approve a proposal
	type ApproveOrigin = TreasuryApproveOrigin;
	// More than half of the council is required (or root) to reject a proposal
	type RejectOrigin = TreasuryRejectOrigin;
	type Event = Event;
	// If spending proposal rejected, transfer proposer bond to treasury
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = ();
	type BurnDestination = ();
	type MaxApprovals = MaxApprovals;
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
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
	type FindAuthor = AuthorInherent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnValidationData = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
}

impl parachain_info::Config for Runtime {}

parameter_types! {
	/// Minimum round length is 2 minutes (10 * 12 second block times)
	pub const MinBlocksPerRound: u32 = 10;
	/// Default BlocksPerRound is every hour (300 * 12 second block times)
	pub const DefaultBlocksPerRound: u32 = 300;
	/// Collator candidate exits are delayed by 2 hours (2 * 300 * block_time)
	pub const LeaveCandidatesDelay: u32 = 2;
	/// Nominator exits are delayed by 2 hours (2 * 300 * block_time)
	pub const LeaveNominatorsDelay: u32 = 2;
	/// Nomination revocations are delayed by 2 hours (2 * 300 * block_time)
	pub const RevokeNominationDelay: u32 = 2;
	/// Reward payments are delayed by 2 hours (2 * 300 * block_time)
	pub const RewardPaymentDelay: u32 = 2;
	/// Minimum 8 collators selected per round, default at genesis and minimum forever after
	pub const MinSelectedCandidates: u32 = 8;
	/// Maximum 10 nominators per collator
	pub const MaxNominatorsPerCollator: u32 = 10;
	/// Maximum 25 collators per nominator
	pub const MaxCollatorsPerNominator: u32 = 25;
	/// Default fixed percent a collator takes off the top of due rewards is 20%
	pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(20);
	/// Default percent of inflation set aside for parachain bond every round
	pub const DefaultParachainBondReservePercent: Percent = Percent::from_percent(30);
	/// Minimum stake required to become a collator is 1_000
	pub const MinCollatorStk: u128 = 1 * currency::KILOUNIT;
	/// Minimum stake required to be reserved to be a candidate is 100
	pub const MinCollatorCandidateStk: u128 = 100 * currency::UNIT;
	/// Minimum stake required to be reserved to be a nominator is 5
	pub const MinNominatorStk: u128 = 5 * currency::UNIT;
}

impl parachain_staking::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type MonetaryGovernanceOrigin = EnsureRoot<AccountId>;
	type MinBlocksPerRound = MinBlocksPerRound;
	type DefaultBlocksPerRound = DefaultBlocksPerRound;
	type LeaveCandidatesDelay = LeaveCandidatesDelay;
	type LeaveNominatorsDelay = LeaveNominatorsDelay;
	type RevokeNominationDelay = RevokeNominationDelay;
	type RewardPaymentDelay = RewardPaymentDelay;
	type MinSelectedCandidates = MinSelectedCandidates;
	type MaxNominatorsPerCollator = MaxNominatorsPerCollator;
	type MaxCollatorsPerNominator = MaxCollatorsPerNominator;
	type DefaultCollatorCommission = DefaultCollatorCommission;
	type DefaultParachainBondReservePercent = DefaultParachainBondReservePercent;
	type MinCollatorStk = MinCollatorStk;
	type MinCollatorCandidateStk = MinCollatorCandidateStk;
	type MinNomination = MinNominatorStk;
	type MinNominatorStk = MinNominatorStk;
	type WeightInfo = parachain_staking::weights::SubstrateWeight<Runtime>;
}

impl pallet_author_inherent::Config for Runtime {
	type AuthorId = NimbusId;
	type SlotBeacon = RelaychainBlockNumberProvider<Self>;
	type AccountLookup = AuthorMapping;
	type EventHandler = ParachainStaking;
	type CanAuthor = AuthorFilter;
}

impl pallet_author_slot_filter::Config for Runtime {
	type Event = Event;
	type RandomnessSource = RandomnessCollectiveFlip;
	type PotentialAuthors = ParachainStaking;
}

parameter_types! {
	// TODO to be revisited
	pub const VestingPeriod: BlockNumber = 4 * WEEKS;
	pub const MinimumReward: Balance = 0;
	pub const Initialized: bool = false;
	pub const InitializationPayment: Perbill = Perbill::from_percent(30);
	pub const MaxInitContributorsBatchSizes: u32 = 1000;
}

impl pallet_crowdloan_rewards::Config for Runtime {
	type Event = Event;
	type Initialized = Initialized;
	type InitializationPayment = InitializationPayment;
	type MaxInitContributors = MaxInitContributorsBatchSizes;
	type MinimumReward = MinimumReward;
	type RewardCurrency = Balances;
	type RelayChainAccountId = AccountId32;
}

parameter_types! {
	pub const DepositAmount: Balance = 100 * currency::UNIT;
}
// This is a simple session key manager. It should probably either work with, or be replaced
// entirely by pallet sessions
impl pallet_author_mapping::Config for Runtime {
	type Event = Event;
	type AuthorId = NimbusId;
	type DepositCurrency = Balances;
	type DepositAmount = DepositAmount;
	type WeightInfo = pallet_author_mapping::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = currency::deposit(1, 8);
	// Additional storage item size of 21 bytes (20 bytes AccountId + 1 byte sizeof(ProxyType)).
	pub const ProxyDepositFactor: Balance = currency::deposit(0, 21);
	pub const MaxProxies: u16 = 32;
	pub const AnnouncementDepositBase: Balance = currency::deposit(1, 8);
	// Additional storage item size of 56 bytes:
	// - 20 bytes AccountId
	// - 32 bytes Hasher (Blake2256)
	// - 4 bytes BlockNumber (u32)
	pub const AnnouncementDepositFactor: Balance = currency::deposit(0, 56);
	pub const MaxPending: u16 = 32;
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	Debug,
	max_encoded_len::MaxEncodedLen,
)]
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
			ProxyType::NonTransfer => {
				matches!(
					c,
					Call::System(..)
						| Call::Timestamp(..) | Call::ParachainStaking(..)
						| Call::Democracy(..) | Call::CouncilCollective(..)
						| Call::TechComitteeCollective(..)
						| Call::Utility(..) | Call::Proxy(..)
				)
			}
			ProxyType::Governance => matches!(
				c,
				Call::Democracy(..)
					| Call::CouncilCollective(..)
					| Call::TechComitteeCollective(..)
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
	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

parameter_types! {
	pub const KsmLocation: MultiLocation = MultiLocation::X1(Junction::Parent);
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	pub MoonbeamNetwork: NetworkId = NetworkId::Named("moon".into());
	pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = Junction::Parachain(ParachainInfo::parachain_id().into()).into();
	pub const Local: MultiLocation = MultiLocation::Null;
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
}

parameter_types! {
	pub const XcmWrapperId: PalletId = PalletId(*b"pc/xcmwr");
	pub SovereignAccount: AccountId32 = ParachainInfo::parachain_id().into_account();
	pub ProxyDepositAmount: Balance =  currency::relay_deposit(1, 8) + currency::relay_deposit(0, 33);
}
pub struct NativeToRelay;
impl sp_runtime::traits::Convert<Balance, cumulus_primitives_core::relay_chain::Balance>
	for NativeToRelay
{
	fn convert(val: u128) -> cumulus_primitives_core::relay_chain::Balance {
		// native is 18
		// relay is 12
		val / 1_000_000
	}
}

pub struct RelayToNative;
impl sp_runtime::traits::Convert<cumulus_primitives_core::relay_chain::Balance, Balance>
	for RelayToNative
{
	fn convert(val: u128) -> Balance {
		// native is 18
		// relay is 12
		val * 1_000_000
	}
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	xcm_builder::ParentIsDefault<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	xcm_builder::SiblingParachainConvertsVia<polkadot_parachain::primitives::Sibling, AccountId>,
	xcm_builder::AccountKey20Aliases<MoonbeamNetwork, AccountId>,
);

/// Matcher associated type for MultiCurrencyAdapter to convert assets into local types
pub struct IsConcreteWithAdjustment<T, FromRelayChainBalance>(
	PhantomData<(T, FromRelayChainBalance)>,
);
impl<T: Get<MultiLocation>, B, FromRelayChainBalance> xcm_executor::traits::MatchesFungible<B>
	for IsConcreteWithAdjustment<T, FromRelayChainBalance>
where
	B: TryFrom<u128>,
	FromRelayChainBalance: sp_runtime::traits::Convert<u128, u128>,
{
	fn matches_fungible(a: &MultiAsset) -> Option<B> {
		if let MultiAsset::ConcreteFungible { id, amount } = a {
			if id == &T::get() {
				// Convert relay chain decimals to local chain
				let local_amount = FromRelayChainBalance::convert(*amount);
				return sp_runtime::traits::CheckedConversion::checked_from(local_amount);
			}
		}
		None
	}
}

/// Means for transacting assets on this chain.
pub type KsmAssetTransactor = xcm_builder::CurrencyAdapter<
	// Use this currency:
	BalancesKsm,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcreteWithAdjustment<KsmLocation, RelayToNative>,
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
>;

use frame_support::traits::{fungibles, Contains};
use sp_runtime::traits::Zero;
use xcm_builder::{AsPrefixedGeneralIndex, ConvertedConcreteAssetId, FungiblesAdapter};
use xcm_executor::traits::JustTry;

/// Allow checking in assets that have issuance > 0.
pub struct CheckAsset<A>(PhantomData<A>);
impl<A> Contains<<A as fungibles::Inspect<AccountId>>::AssetId> for CheckAsset<A>
where
	A: fungibles::Inspect<AccountId>,
{
	fn contains(id: &<A as fungibles::Inspect<AccountId>>::AssetId) -> bool {
		!A::total_issuance(*id).is_zero()
	}
}

/// Converter struct implementing `AssetIdConversion` converting a numeric asset ID (must be `TryFrom/TryInto<u128>`) into
/// a `GeneralIndex` junction, prefixed by some `MultiLocation` value. The `MultiLocation` value will typically be a
/// `PalletInstance` junction.
use sp_std::borrow::Borrow;
pub struct AsParachainId<AssetId, ConvertAssetId>(PhantomData<(AssetId, ConvertAssetId)>);
impl<AssetId: Clone, ConvertAssetId: xcm_executor::traits::Convert<u32, AssetId>>
	xcm_executor::traits::Convert<MultiLocation, AssetId> for AsParachainId<AssetId, ConvertAssetId>
{
	fn convert_ref(id: impl Borrow<MultiLocation>) -> Result<AssetId, ()> {
		match id.borrow() {
			MultiLocation::X1(Junction::Parachain(id)) => ConvertAssetId::convert_ref(id),
			_ => Err(()),
		}
	}
	fn reverse_ref(what: impl Borrow<AssetId>) -> Result<MultiLocation, ()> {
		let id = ConvertAssetId::reverse_ref(what)?;
		Ok(MultiLocation::X1(Junction::Parachain(id)))
	}
}

pub type FungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	Assets,
	// Use this currency when it is a fungible asset matching the given location or name:
	(ConvertedConcreteAssetId<u32, Balance, AsParachainId<u32, JustTry>, JustTry>,),
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We only allow teleports of known assets.
	(),
	// We dont track any teleports
	(),
>;
/// Means for transacting assets on this chain.
// pub type AssetTransactors = (KsmAssetTransactor, FungiblesTransactor);

pub type AssetTransactors = FungiblesTransactor;
/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	xcm_builder::SovereignSignedViaLocation<LocationToAccountId, Origin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognised.
	xcm_builder::RelayChainAsNative<RelayChainOrigin, Origin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	xcm_builder::SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	xcm_builder::ParentAsSuperuser<Origin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	pallet_xcm::XcmPassthrough<Origin>,
	xcm_builder::SignedAccountKey20AsNative<MoonbeamNetwork, Origin>,
);

parameter_types! {
	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = 1_000_000_000;
}

match_type! {
	pub type ParentOrParentsExecutivePlurality: impl Contains<MultiLocation> = {
		MultiLocation::X1(Junction::Parent) |
		MultiLocation::X2(Junction::Parent, Junction::Plurality { id: xcm::v0::BodyId::Executive, .. })
	};
}

pub type XcmBarrier = (
	xcm_builder::TakeWeightCredit,
	xcm_builder::AllowTopLevelPaidExecutionFrom<All<MultiLocation>>,
	xcm_builder::AllowUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
	// ^^^ Parent and its exec plurality get free execution
);

pub struct XcmExecutorConfig;
impl xcm_executor::Config for XcmExecutorConfig {
	type Call = Call;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = AssetTransactors;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = xcm_builder::NativeAsset;
	type IsTeleporter = xcm_builder::NativeAsset; // <- should be enough to allow teleportation of KSM
	type LocationInverter = xcm_builder::LocationInverter<Ancestry>;
	type Barrier = XcmBarrier;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, Call>;
	type Trader =
		xcm_builder::UsingComponents<IdentityFee<Balance>, KsmLocation, AccountId, Balances, ()>;
	type ResponseHandler = (); // Don't handle responses for now.
}

type XcmExecutor = xcm_executor::XcmExecutor<XcmExecutorConfig>;

parameter_types! {
	pub const MaxDownwardMessageWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 10;
}

pub struct SignedToAccountId20<Origin, AccountId, Network>(
	sp_std::marker::PhantomData<(Origin, AccountId, Network)>,
);
impl<Origin: OriginTrait + Clone, AccountId: Into<[u8; 20]>, Network: Get<NetworkId>>
	xcm_executor::traits::Convert<Origin, MultiLocation>
	for SignedToAccountId20<Origin, AccountId, Network>
where
	Origin::PalletsOrigin: From<frame_system::RawOrigin<AccountId>>
		+ TryInto<frame_system::RawOrigin<AccountId>, Error = Origin::PalletsOrigin>,
{
	fn convert(o: Origin) -> Result<MultiLocation, Origin> {
		o.try_with_caller(|caller| match caller.try_into() {
			Ok(frame_system::RawOrigin::Signed(who)) => Ok(Junction::AccountKey20 {
				key: who.into(),
				network: Network::get(),
			}
			.into()),
			Ok(other) => Err(other.into()),
			Err(other) => Err(other),
		})
	}
}

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId20<Origin, AccountId, MoonbeamNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
	type Event = Event;
	type SendXcmOrigin = xcm_builder::EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = xcm_builder::EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmExecuteFilter = All<(MultiLocation, Xcm<Call>)>;
	type XcmExecutor = XcmExecutor;
	type XcmTeleportFilter = All<(MultiLocation, Vec<MultiAsset>)>;
	type XcmReserveTransferFilter = All<(MultiLocation, Vec<MultiAsset>)>;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, Call>;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor;
	type ChannelInfo = ParachainSystem;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

pub struct AccountKey20Convert;
impl
	sp_runtime::traits::Convert<
		<<Signature as sp_runtime::traits::Verify>::Signer as IdentifyAccount>::AccountId,
		[u8; 20],
	> for AccountKey20Convert
{
	fn convert(
		from: <<Signature as sp_runtime::traits::Verify>::Signer as IdentifyAccount>::AccountId,
	) -> [u8; 20] {
		from.into()
	}
}

parameter_types! {
	pub const AssetDeposit: Balance = 100 * currency::UNIT; // 100 DOLLARS deposit to create asset
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;
	/// Key = 32 bytes, Value = 36 bytes (32+1+1+1+1)
	// https://github.com/paritytech/substrate/blob/069917b/frame/assets/src/lib.rs#L257L271
	pub const MetadataDepositBase: Balance = currency::deposit(1, 68);
	pub const MetadataDepositPerByte: Balance = currency::deposit(0, 1);
	pub const ExecutiveBody: BodyId = BodyId::Executive;
}

use pallet_xcm::EnsureXcm;
use pallet_xcm::IsMajorityOfBody;

/// We allow root and the Relay Chain council to execute privileged asset operations.
pub type AssetsForceOrigin = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	EnsureXcm<IsMajorityOfBody<KsmLocation, ExecutiveBody>>,
>;

pub type AssetId = u32;
impl pallet_assets::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type AssetId = AssetId;
	type Currency = Balances;
	type ForceOrigin = AssetsForceOrigin;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
}

pub struct AssetIdtoMultiLocation<AssetXConverter>(sp_std::marker::PhantomData<AssetXConverter>);
impl<AssetXConverter> sp_runtime::traits::Convert<AssetId, Option<MultiLocation>>
	for AssetIdtoMultiLocation<AssetXConverter>
where
	AssetXConverter: xcm_executor::traits::Convert<MultiLocation, AssetId>,
{
	fn convert(asset: AssetId) -> Option<MultiLocation> {
		AssetXConverter::reverse_ref(asset).ok()
	}
}
parameter_types! {
	pub const BaseXcmWeight: Weight = 100_000_000;
}

pub struct AccountIdToMultiLocation;
impl sp_runtime::traits::Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
	fn convert(account: AccountId) -> MultiLocation {
		MultiLocation::X1(Junction::AccountKey20 {
			network: NetworkId::Any,
			key: account.into(),
		})
	}
}

impl orml_xtokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type CurrencyId = AssetId;
	type AccountIdToMultiLocation = AccountIdToMultiLocation;
	type CurrencyIdConvert = AssetIdtoMultiLocation<AsParachainId<AssetId, JustTry>>;
	type XcmExecutor = XcmExecutor;
	type SelfLocation = Ancestry;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, Call>;
	type BaseXcmWeight = BaseXcmWeight;
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
		BalancesKsm: pallet_balances::<Instance1>::{Pallet, Call, Storage, Config<T>, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>},
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
		ParachainInfo: parachain_info::{Pallet, Storage, Config},
		EthereumChainId: pallet_ethereum_chain_id::{Pallet, Storage, Config},
		EVM: pallet_evm::{Pallet, Config, Call, Storage, Event<T>},
		Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Config, ValidateUnsigned},
		ParachainStaking: parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>},
		Scheduler: pallet_scheduler::{Pallet, Storage, Config, Event<T>, Call},
		Democracy: pallet_democracy::{Pallet, Storage, Config<T>, Event<T>, Call},
		CouncilCollective:
			pallet_collective::<Instance1>::{Pallet, Call, Storage, Event<T>, Origin<T>, Config<T>},
		TechComitteeCollective:
			pallet_collective::<Instance2>::{Pallet, Call, Storage, Event<T>, Origin<T>, Config<T>},
		Treasury: pallet_treasury::{Pallet, Storage, Config, Event<T>, Call},
		AuthorInherent: pallet_author_inherent::{Pallet, Call, Storage, Inherent},
		AuthorFilter: pallet_author_slot_filter::{Pallet, Call, Storage, Event, Config},
		CrowdloanRewards: pallet_crowdloan_rewards::{Pallet, Call, Config<T>, Storage, Event<T>},
		AuthorMapping: pallet_author_mapping::{Pallet, Call, Config<T>, Storage, Event<T>},
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>},
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>},
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin},
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>},
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		XTokens: orml_xtokens::{Pallet, Call, Storage, Event<T>},

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

// All of our runtimes share most of their Runtime API implementations.
// We use a macro to implement this common part and add runtime-specific additional implementations.
// This macro expands to :
// ```
// impl_runtime_apis! {
//     // All impl blocks shared between all runtimes.
//
//     // Specific impls provided to the `impl_runtime_apis_plus_common!` macro.
// }
// ```
runtime_common::impl_runtime_apis_plus_common! {
	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}
}

// Check the timestamp and parachain inherents
struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data =
			cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
				relay_chain_slot,
				sp_std::time::Duration::from_secs(6),
			)
			.create_inherent_data()
			.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(&block)
	}
}

// Nimbus's Executive wrapper allows relay validators to verify the seal digest
cumulus_pallet_parachain_system::register_validate_block!(
	Runtime = Runtime,
	BlockExecutor = pallet_author_inherent::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
);
