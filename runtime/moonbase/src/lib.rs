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
use pallet_evm_precompile_assets_erc20::AccountIdAssetIdConversion;

use account::AccountId20;
use sp_runtime::traits::Hash as THash;

use frame_support::{
	construct_runtime, parameter_types,
	traits::{
		Contains, Everything, FindAuthor, Get, Imbalance, InstanceFilter, Nothing, OffchainWorker,
		OnFinalize, OnIdle, OnInitialize, OnRuntimeUpgrade, OnUnbalanced,
		PalletInfo as PalletInfoTrait,
	},
	weights::{
		constants::{RocksDbWeight, WEIGHT_PER_SECOND},
		DispatchClass, GetDispatchInfo, IdentityFee, Weight,
	},
	PalletId,
};

use xcm_builder::{
	AccountKey20Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, ConvertedConcreteAssetId,
	CurrencyAdapter as XcmCurrencyAdapter, EnsureXcmOrigin, FixedWeightBounds, FungiblesAdapter,
	IsConcrete, LocationInverter, ParentAsSuperuser, ParentIsDefault, RelayChainAsNative,
	SiblingParachainAsNative, SiblingParachainConvertsVia, SignedAccountKey20AsNative,
	SovereignSignedViaLocation, TakeWeightCredit, UsingComponents,
};

use xcm_executor::traits::JustTry;

use frame_system::{EnsureOneOf, EnsureRoot, EnsureSigned};
pub use moonbeam_core_primitives::{
	AccountId, AccountIndex, Address, AssetId, Balance, BlockNumber, DigestItem, Hash, Header,
	Index, Signature,
};
use moonbeam_rpc_primitives_txpool::TxPoolResponse;
use pallet_balances::NegativeImbalance;
use pallet_ethereum::Call::transact;
use pallet_ethereum::Transaction as EthereumTransaction;
#[cfg(feature = "std")]
pub use pallet_evm::GenesisAccount;
use pallet_evm::{
	Account as EVMAccount, EnsureAddressNever, EnsureAddressRoot, FeeCalculator, GasWeightMapping,
	Runner,
};
use pallet_transaction_payment::{CurrencyAdapter, Multiplier, TargetedFeeAdjustment};
pub use parachain_staking::{InflationInfo, Range};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_api::impl_runtime_apis;
use sp_core::{u32_trait::*, OpaqueMetadata, H160, H256, U256};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{BlakeTwo256, Block as BlockT, Dispatchable, IdentityLookup, PostDispatchInfoOf},
	transaction_validity::{
		InvalidTransaction, TransactionSource, TransactionValidity, TransactionValidityError,
	},
	AccountId32, ApplyExtrinsicResult, FixedPointNumber, Perbill, Percent, Permill, Perquintill,
	SaturatedConversion,
};
use sp_std::{
	convert::{From, Into, TryFrom},
	prelude::*,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use xcm::latest::prelude::*;

use nimbus_primitives::{CanAuthor, NimbusId};

mod precompiles;
use precompiles::{MoonbasePrecompiles, ASSET_PRECOMPILE_ADDRESS_PREFIX};

use xcm_primitives::{
	AccountIdToCurrencyId, AccountIdToMultiLocation, AsAssetType, FirstAssetTrader,
	MultiNativeAsset, SignedToAccountId20, UtilityAvailableCalls, UtilityEncodeCall,
	XcmFeesToAccount, XcmTransact,
};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

pub type Precompiles = MoonbasePrecompiles<Runtime>;

/// UNIT, the native token, uses 18 decimals of precision.
pub mod currency {
	use super::Balance;

	// Provide a common factor between runtimes based on a supply of 10_000_000 tokens.
	pub const SUPPLY_FACTOR: Balance = 1;

	pub const WEI: Balance = 1;
	pub const KILOWEI: Balance = 1_000;
	pub const MEGAWEI: Balance = 1_000_000;
	pub const GIGAWEI: Balance = 1_000_000_000;
	pub const MICROUNIT: Balance = 1_000_000_000_000;
	pub const MILLIUNIT: Balance = 1_000_000_000_000_000;
	pub const UNIT: Balance = 1_000_000_000_000_000_000;
	pub const KILOUNIT: Balance = 1_000_000_000_000_000_000_000;

	pub const TRANSACTION_BYTE_FEE: Balance = 10 * MICROUNIT * SUPPLY_FACTOR;
	pub const STORAGE_BYTE_FEE: Balance = 100 * MICROUNIT * SUPPLY_FACTOR;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 1 * UNIT * SUPPLY_FACTOR + (bytes as Balance) * STORAGE_BYTE_FEE
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
	spec_version: 1002,
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
	type BaseCallFilter = MaintenanceMode;
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

pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
where
	R: pallet_balances::Config + pallet_treasury::Config,
	pallet_treasury::Pallet<R>: OnUnbalanced<NegativeImbalance<R>>,
{
	// this seems to be called for substrate-based transactions
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
		if let Some(fees) = fees_then_tips.next() {
			// for fees, 80% are burned, 20% to the treasury
			let (_, to_treasury) = fees.ration(80, 20);
			// Balances pallet automatically burns dropped Negative Imbalances by decreasing
			// total_supply accordingly
			<pallet_treasury::Pallet<R> as OnUnbalanced<_>>::on_unbalanced(to_treasury);
		}
	}

	// this is called from pallet_evm for Ethereum-based transactions
	// (technically, it calls on_unbalanced, which calls this when non-zero)
	fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
		// Balances pallet automatically burns dropped Negative Imbalances by decreasing
		// total_supply accordingly
		let (_, to_treasury) = amount.ration(80, 20);
		<pallet_treasury::Pallet<R> as OnUnbalanced<_>>::on_unbalanced(to_treasury);
	}
}

parameter_types! {
	pub const TransactionByteFee: Balance = currency::TRANSACTION_BYTE_FEE;
	pub OperationalFeeMultiplier: u8 = 5;
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees<Runtime>>;
	type TransactionByteFee = TransactionByteFee;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Runtime>;
}

impl pallet_sudo::Config for Runtime {
	type Call = Call;
	type Event = Event;
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
		(1 * currency::GIGAWEI * currency::SUPPLY_FACTOR).into()
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

// Instruct how to go from an H160 to an AssetID
// We just take the lowest 128 bits
impl AccountIdAssetIdConversion<AccountId, AssetId> for Runtime {
	/// The way to convert an account to assetId is by ensuring that the prefix is 0XFFFFFFFF
	/// and by taking the lowest 128 bits as the assetId
	fn account_to_asset_id(account: AccountId) -> Option<AssetId> {
		let h160_account: H160 = account.into();
		let mut data = [0u8; 16];
		let (prefix_part, id_part) = h160_account.as_fixed_bytes().split_at(4);
		if prefix_part == ASSET_PRECOMPILE_ADDRESS_PREFIX {
			data.copy_from_slice(id_part);
			let asset_id: AssetId = u128::from_be_bytes(data).into();
			Some(asset_id)
		} else {
			None
		}
	}

	// The opposite conversion
	fn asset_id_to_account(asset_id: AssetId) -> AccountId {
		let mut data = [0u8; 20];
		data[0..4].copy_from_slice(ASSET_PRECOMPILE_ADDRESS_PREFIX);
		data[4..20].copy_from_slice(&asset_id.to_be_bytes());
		AccountId::from(data)
	}
}

/// The author inherent provides an AccountId, but pallet evm needs an H160.
/// This simple adapter makes the conversion for any types T, U such that T: Into<U>
pub struct FindAuthorAdapter<T, U, Inner>(sp_std::marker::PhantomData<(T, U, Inner)>);

impl<T, U, Inner> FindAuthor<U> for FindAuthorAdapter<T, U, Inner>
where
	T: Into<U>,
	Inner: FindAuthor<T>,
{
	fn find_author<'a, I>(digests: I) -> Option<U>
	where
		I: 'a + IntoIterator<Item = (sp_runtime::ConsensusEngineId, &'a [u8])>,
	{
		Inner::find_author(digests).map(Into::into)
	}
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = FixedGasPrice;
	type GasWeightMapping = MoonbeamGasWeightMapping;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = runtime_common::IntoAddressMapping;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type Precompiles = MoonbasePrecompiles<Self>;
	type ChainId = EthereumChainId;
	type OnChargeTransaction = pallet_evm::EVMCurrencyAdapter<Balances, DealWithFees<Runtime>>;
	type BlockGasLimit = BlockGasLimit;
	type FindAuthor = FindAuthorAdapter<AccountId20, H160, AuthorInherent>;
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
	pub const TechCommitteeMotionDuration: BlockNumber = 3 * DAYS;
	/// The maximum number of Proposlas that can be open in the technical committee at once.
	pub const TechCommitteeMaxProposals: u32 = 100;
	/// The maximum number of technical committee members.
	pub const TechCommitteeMaxMembers: u32 = 100;
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
	type MotionDuration = TechCommitteeMotionDuration;
	type MaxProposals = TechCommitteeMaxProposals;
	type MaxMembers = TechCommitteeMaxMembers;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 1 * DAYS;
	pub const VotingPeriod: BlockNumber = 5 * DAYS;
	pub const VoteLockingPeriod: BlockNumber = 1 * DAYS;
	pub const FastTrackVotingPeriod: BlockNumber = 4 * HOURS;
	pub const EnactmentPeriod: BlockNumber = 1 * DAYS;
	pub const CooloffPeriod: BlockNumber = 7 * DAYS;
	pub const MinimumDeposit: Balance = 4 * currency::UNIT * currency::SUPPLY_FACTOR;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
	pub const PreimageByteDeposit: Balance = currency::STORAGE_BYTE_FEE;
	pub const InstantAllowed: bool = true;
}

impl pallet_democracy::Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type VoteLockingPeriod = VoteLockingPeriod;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	type MinimumDeposit = MinimumDeposit;
	/// To decide what their next motion is.
	type ExternalOrigin =
		pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilInstance>;
	/// To have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin =
		pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, CouncilInstance>;
	/// To have the next scheduled referendum be a straight default-carries (NTB) vote.
	type ExternalDefaultOrigin =
		pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, CouncilInstance>;
	/// To allow a shorter voting/enactment period for external proposals.
	type FastTrackOrigin =
		pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, TechCommitteeInstance>;
	/// To instant fast track.
	type InstantOrigin =
		pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, TechCommitteeInstance>;
	// To cancel a proposal which has been passed.
	type CancellationOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, CouncilInstance>,
	>;
	// To cancel a proposal before it has been passed.
	type CancelProposalOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<_3, _5, AccountId, TechCommitteeInstance>,
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
	pub const ProposalBondMinimum: Balance = 1 * currency::UNIT * currency::SUPPLY_FACTOR;
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

parameter_types! {
	// Add one item in storage and take 258 bytes
	pub const BasicDeposit: Balance = currency::deposit(1, 258);
	// Not add any item to the storage but takes 66 bytes
	pub const FieldDeposit: Balance = currency::deposit(0, 66);
	// Add one item in storage and take 53 bytes
	pub const SubAccountDeposit: Balance = currency::deposit(1, 53);
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

type IdentityForceOrigin = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilInstance>,
>;
type IdentityRegistrarOrigin = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilInstance>,
>;

impl pallet_identity::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = Treasury;
	type ForceOrigin = IdentityForceOrigin;
	type RegistrarOrigin = IdentityRegistrarOrigin;
	type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
}

pub struct TransactionConverter;

impl fp_rpc::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(&self, transaction: pallet_ethereum::Transaction) -> UncheckedExtrinsic {
		UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		)
	}
}

impl fp_rpc::ConvertTransaction<opaque::UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(
		&self,
		transaction: pallet_ethereum::Transaction,
	) -> opaque::UncheckedExtrinsic {
		let extrinsic = UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		);
		let encoded = extrinsic.encode();
		opaque::UncheckedExtrinsic::decode(&mut &encoded[..])
			.expect("Encoded extrinsic is always valid")
	}
}

impl pallet_ethereum::Config for Runtime {
	type Event = Event;
	type StateRoot = pallet_ethereum::IntermediateStateRoot;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnValidationData = ();
	type SelfParaId = ParachainInfo;
	type DmpMessageHandler = MaintenanceMode;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = MaintenanceMode;
	type ReservedXcmpWeight = ReservedXcmpWeight;
}

impl parachain_info::Config for Runtime {}

parameter_types! {
	/// Minimum round length is 2 minutes (10 * 12 second block times)
	pub const MinBlocksPerRound: u32 = 10;
	/// Blocks per round
	pub const DefaultBlocksPerRound: u32 = 2 * HOURS;
	/// Rounds before the collator leaving the candidates request can be executed
	pub const LeaveCandidatesDelay: u32 = 2;
	/// Rounds before the candidate bond increase/decrease can be executed
	pub const CandidateBondLessDelay: u32 = 2;
	/// Rounds before the delegator exit can be executed
	pub const LeaveDelegatorsDelay: u32 = 2;
	/// Rounds before the delegator revocation can be executed
	pub const RevokeDelegationDelay: u32 = 2;
	/// Rounds before the delegator bond increase/decrease can be executed
	pub const DelegationBondLessDelay: u32 = 2;
	/// Rounds before the reward is paid
	pub const RewardPaymentDelay: u32 = 2;
	/// Minimum collators selected per round, default at genesis and minimum forever after
	pub const MinSelectedCandidates: u32 = 8;
	/// Maximum delegators counted per candidate
	pub const MaxDelegatorsPerCandidate: u32 = 100;
	/// Maximum delegations per delegator
	pub const MaxDelegationsPerDelegator: u32 = 100;
	/// Default fixed percent a collator takes off the top of due rewards
	pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(20);
	/// Default percent of inflation set aside for parachain bond every round
	pub const DefaultParachainBondReservePercent: Percent = Percent::from_percent(30);
	/// Minimum stake required to become a collator
	pub const MinCollatorStk: u128 = 1000 * currency::UNIT * currency::SUPPLY_FACTOR;
	/// Minimum stake required to be reserved to be a candidate
	pub const MinCandidateStk: u128 = 500 * currency::UNIT * currency::SUPPLY_FACTOR;
	/// Minimum stake required to be reserved to be a delegator
	pub const MinDelegatorStk: u128 = 5 * currency::UNIT * currency::SUPPLY_FACTOR;
}

impl parachain_staking::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type MonetaryGovernanceOrigin = EnsureRoot<AccountId>;
	type MinBlocksPerRound = MinBlocksPerRound;
	type DefaultBlocksPerRound = DefaultBlocksPerRound;
	type LeaveCandidatesDelay = LeaveCandidatesDelay;
	type CandidateBondLessDelay = CandidateBondLessDelay;
	type LeaveDelegatorsDelay = LeaveDelegatorsDelay;
	type RevokeDelegationDelay = RevokeDelegationDelay;
	type DelegationBondLessDelay = DelegationBondLessDelay;
	type RewardPaymentDelay = RewardPaymentDelay;
	type MinSelectedCandidates = MinSelectedCandidates;
	type MaxDelegatorsPerCandidate = MaxDelegatorsPerCandidate;
	type MaxDelegationsPerDelegator = MaxDelegationsPerDelegator;
	type DefaultCollatorCommission = DefaultCollatorCommission;
	type DefaultParachainBondReservePercent = DefaultParachainBondReservePercent;
	type MinCollatorStk = MinCollatorStk;
	type MinCandidateStk = MinCandidateStk;
	type MinDelegation = MinDelegatorStk;
	type MinDelegatorStk = MinDelegatorStk;
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
	pub const MinimumReward: Balance = 0;
	pub const Initialized: bool = false;
	pub const InitializationPayment: Perbill = Perbill::from_percent(30);
	pub const MaxInitContributorsBatchSizes: u32 = 500;
	pub const RelaySignaturesThreshold: Perbill = Perbill::from_percent(100);
}

impl pallet_crowdloan_rewards::Config for Runtime {
	type Event = Event;
	type Initialized = Initialized;
	type InitializationPayment = InitializationPayment;
	type MaxInitContributors = MaxInitContributorsBatchSizes;
	type MinimumReward = MinimumReward;
	type RewardCurrency = Balances;
	type RelayChainAccountId = AccountId32;
	type RewardAddressChangeOrigin = EnsureSigned<Self::AccountId>;
	type RewardAddressRelayVoteThreshold = RelaySignaturesThreshold;
	type VestingBlockNumber = cumulus_primitives_core::relay_chain::BlockNumber;
	type VestingBlockProvider =
		cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Self>;
	type WeightInfo = pallet_crowdloan_rewards::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const DepositAmount: Balance = 100 * currency::UNIT * currency::SUPPLY_FACTOR;
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
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(
	Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, MaxEncodedLen, TypeInfo,
)]
pub enum ProxyType {
	/// All calls can be proxied. This is the trivial/most permissive filter.
	Any = 0,
	/// Only extrinsics that do not transfer funds.
	NonTransfer = 1,
	/// Only extrinsics related to governance (democracy and collectives).
	Governance = 2,
	/// Only extrinsics related to staking.
	Staking = 3,
	/// Allow to veto an announced proxy call.
	CancelProxy = 4,
	/// Allow extrinsic related to Balances.
	Balances = 5,
	/// Allow extrinsic related to AuthorMapping.
	AuthorMapping = 6,
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
						| Call::TechCommitteeCollective(..)
						| Call::Utility(..) | Call::Proxy(..)
						| Call::AuthorMapping(..)
						| Call::CrowdloanRewards(pallet_crowdloan_rewards::Call::claim { .. })
				)
			}
			ProxyType::Governance => matches!(
				c,
				Call::Democracy(..)
					| Call::CouncilCollective(..)
					| Call::TechCommitteeCollective(..)
					| Call::Utility(..)
			),
			ProxyType::Staking => matches!(
				c,
				Call::ParachainStaking(..) | Call::Utility(..) | Call::AuthorMapping(..)
			),
			ProxyType::CancelProxy => {
				matches!(
					c,
					Call::Proxy(pallet_proxy::Call::reject_announcement { .. })
				)
			}
			ProxyType::Balances => {
				matches!(c, Call::Balances(..) | Call::Utility(..))
			}
			ProxyType::AuthorMapping => {
				matches!(c, Call::AuthorMapping(..))
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

impl pallet_migrations::Config for Runtime {
	type Event = Event;
	//TODO wire up our correct list of migrations here. Maybe this shouldn't be in `runtime_common`.
	type MigrationsList = runtime_common::migrations::CommonMigrations<
		Runtime,
		CouncilCollective,
		TechCommitteeCollective,
	>;
}

parameter_types! {
	// The network Id of the relay
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;
	// The relay chain Origin type
	pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
	// The ancestry, defines the multilocation describing this consensus system
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
	// Self Reserve location, defines the multilocation identifiying the self-reserve currency
	// This is used to match it against our Balances pallet when we receive such a MultiLocation
	// (Parent, Self Para Id, Self Balances pallet index)
	pub SelfReserve: MultiLocation = MultiLocation {
		parents:1,
		interior: Junctions::X2(
			Parachain(ParachainInfo::parachain_id().into()),
			PalletInstance(<Runtime as frame_system::Config>::PalletInfo::index::<Balances>().unwrap() as u8)
		)
	};
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsDefault<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<polkadot_parachain::primitives::Sibling, AccountId>,
	// If we receive a MultiLocation of type AccountKey20, just generate a native account
	AccountKey20Aliases<RelayNetwork, AccountId>,
);

// The non-reserve fungible transactor type
// It will use pallet-assets, and the Id will be matched against AsAssetType
pub type FungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	Assets,
	// Use this currency when it is a fungible asset matching the given location or name:
	(
		ConvertedConcreteAssetId<
			AssetId,
			Balance,
			AsAssetType<AssetId, AssetType, AssetManager>,
			JustTry,
		>,
	),
	// Do a simple punn to convert an AccountId20 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We dont allow teleports.
	Nothing,
	// We dont track any teleports
	(),
>;

/// The transactor for our own chain currency.
pub type LocalAssetTransactor = XcmCurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<SelfReserve>,
	// We can convert the MultiLocations with our converter above:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We dont allow teleport
	(),
>;

// We use both transactors
pub type AssetTransactors = (LocalAssetTransactor, FungiblesTransactor);

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, Origin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognised.
	RelayChainAsNative<RelayChainOrigin, Origin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<Origin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	pallet_xcm::XcmPassthrough<Origin>,
	// Xcm Origins defined by a Multilocation of type AccountKey20 can be converted to a 20 byte-
	// account local origin
	SignedAccountKey20AsNative<RelayNetwork, Origin>,
);

parameter_types! {
	/// The amount of weight an XCM operation takes. This is safe overestimate.
	/// We should increase this to a value close to what polkadot charges
	/// We are charging less to make it work with current reserve_transfer_assets issue
	/// TODO: Once fixed in polkadot v0.9.12, we should go back to 1_000_000_000
	/// https://github.com/paritytech/polkadot/pull/4144
	pub UnitWeightCost: Weight = 100_000_000;
	/// Maximum number of instructions in a single XCM fragment. A sanity check against
	/// weight caculations getting too crazy.
	pub MaxInstructions: u32 = 100;
}

/// Xcm Weigher shared between multiple Xcm-related configs.
pub type XcmWeigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;

// Allow paid executions
pub type XcmBarrier = (
	TakeWeightCredit,
	AllowTopLevelPaidExecutionFrom<Everything>,
	AllowKnownQueryResponses<PolkadotXcm>,
	// Subscriptions for version tracking are OK.
	AllowSubscriptionsFrom<Everything>,
);

parameter_types! {
	/// Xcm fees will go to the treasury account
	pub XcmFeesAccount: AccountId = Treasury::account_id();
}

/// This is the struct that will handle the revenue from xcm fees
/// We do not burn anything because we want to mimic exactly what
/// the sovereign account has
pub type XcmFeesToAccount_ = XcmFeesToAccount<
	Assets,
	(
		ConvertedConcreteAssetId<
			AssetId,
			Balance,
			AsAssetType<AssetId, AssetType, AssetManager>,
			JustTry,
		>,
	),
	AccountId,
	XcmFeesAccount,
>;

pub struct XcmExecutorConfig;
impl xcm_executor::Config for XcmExecutorConfig {
	type Call = Call;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = AssetTransactors;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	// Filter to the reserve withdraw operations
	type IsReserve = MultiNativeAsset;
	type IsTeleporter = (); // No teleport
	type LocationInverter = LocationInverter<Ancestry>;
	type Barrier = XcmBarrier;
	type Weigher = XcmWeigher;
	// We use two traders
	// When we receive the self-reserve asset, we use pallet-transaction-payment
	// When we receive a non-reserve asset, we use AssetManager to fetch how many
	// units per second we should charge
	type Trader = (
		UsingComponents<
			IdentityFee<Balance>,
			SelfReserve,
			AccountId,
			Balances,
			DealWithFees<Runtime>,
		>,
		FirstAssetTrader<AssetId, AssetType, AssetManager, XcmFeesToAccount_>,
	);
	type ResponseHandler = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
}

type XcmExecutor = xcm_executor::XcmExecutor<XcmExecutorConfig>;

parameter_types! {
	pub const MaxDownwardMessageWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 10;
}

// Converts a Signed Local Origin into a MultiLocation
pub type LocalOriginToLocation = SignedToAccountId20<Origin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
	type Event = Event;
	type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmExecuteFilter = Nothing;
	type XcmExecutor = XcmExecutor;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type Weigher = XcmWeigher;
	type LocationInverter = LocationInverter<Ancestry>;
	type Origin = Origin;
	type Call = Call;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

// These parameters dont matter much as this will only be called by root with the forced arguments
// No deposit is substracted with those methods
parameter_types! {
	pub const AssetDeposit: Balance = 0;
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 0;
	pub const MetadataDepositPerByte: Balance = 0;
	pub const ExecutiveBody: BodyId = BodyId::Executive;
}

/// We allow root and Chain council to execute privileged asset operations.
pub type AssetsForceOrigin = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilInstance>,
>;

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

// Our AssetType. For now we only handle Xcm Assets
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum AssetType {
	Xcm(MultiLocation),
}
impl Default for AssetType {
	fn default() -> Self {
		Self::Xcm(MultiLocation::here())
	}
}

impl From<MultiLocation> for AssetType {
	fn from(location: MultiLocation) -> Self {
		Self::Xcm(location)
	}
}

impl Into<Option<MultiLocation>> for AssetType {
	fn into(self: Self) -> Option<MultiLocation> {
		match self {
			Self::Xcm(location) => Some(location),
		}
	}
}

// Implementation on how to retrieve the AssetId from an AssetType
// We simply hash the AssetType and take the lowest 128 bits
impl From<AssetType> for AssetId {
	fn from(asset: AssetType) -> AssetId {
		match asset {
			AssetType::Xcm(id) => {
				let mut result: [u8; 16] = [0u8; 16];
				let hash: H256 = id.using_encoded(<Runtime as frame_system::Config>::Hashing::hash);
				result.copy_from_slice(&hash.as_fixed_bytes()[0..16]);
				u128::from_le_bytes(result)
			}
		}
	}
}

// We instruct how to register the Assets
// In this case, we tell it to Create an Asset in pallet-assets
pub struct AssetRegistrar;
use frame_support::{pallet_prelude::DispatchResult, transactional};

impl pallet_asset_manager::AssetRegistrar<Runtime> for AssetRegistrar {
	#[transactional]
	fn create_asset(
		asset: AssetId,
		min_balance: Balance,
		metadata: AssetRegistrarMetadata,
	) -> DispatchResult {
		Assets::force_create(
			Origin::root(),
			asset,
			AssetManager::account_id(),
			true,
			min_balance,
		)?;

		// TODO uncomment when we feel comfortable
		/*
		// The asset has been created. Let's put the revert code in the precompile address
		let precompile_address = Runtime::asset_id_to_account(asset);
		pallet_evm::AccountCodes::<Runtime>::insert(
			precompile_address,
			vec![0x60, 0x00, 0x60, 0x00, 0xfd],
		);*/

		// Lastly, the metadata
		Assets::force_set_metadata(
			Origin::root(),
			asset,
			metadata.name,
			metadata.symbol,
			metadata.decimals,
			metadata.is_frozen,
		)
	}
}

#[derive(Clone, Default, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub struct AssetRegistrarMetadata {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
	pub is_frozen: bool,
}

impl pallet_asset_manager::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type AssetId = AssetId;
	type AssetRegistrarMetadata = AssetRegistrarMetadata;
	type AssetType = AssetType;
	type AssetRegistrar = AssetRegistrar;
	type AssetModifierOrigin = EnsureRoot<AccountId>;
	type WeightInfo = pallet_asset_manager::weights::SubstrateWeight<Runtime>;
}

// Our currencyId. We distinguish for now between SelfReserve, and Others, defined by their Id.
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum CurrencyId {
	SelfReserve,
	OtherReserve(AssetId),
}

impl AccountIdToCurrencyId<AccountId, CurrencyId> for Runtime {
	fn account_to_currency_id(account: AccountId) -> Option<CurrencyId> {
		match account {
			// the self-reserve currency is identified by the pallet-balances address
			a if a == H160::from_low_u64_be(2050).into() => Some(CurrencyId::SelfReserve),
			// the rest of the currencies, by their corresponding erc20 address
			_ => Runtime::account_to_asset_id(account)
				.map(|asset_id| CurrencyId::OtherReserve(asset_id)),
		}
	}
}

// How to convert from CurrencyId to MultiLocation
pub struct CurrencyIdtoMultiLocation<AssetXConverter>(sp_std::marker::PhantomData<AssetXConverter>);
impl<AssetXConverter> sp_runtime::traits::Convert<CurrencyId, Option<MultiLocation>>
	for CurrencyIdtoMultiLocation<AssetXConverter>
where
	AssetXConverter: xcm_executor::traits::Convert<MultiLocation, AssetId>,
{
	fn convert(currency: CurrencyId) -> Option<MultiLocation> {
		match currency {
			CurrencyId::SelfReserve => {
				let multi: MultiLocation = SelfReserve::get();
				Some(multi)
			}
			CurrencyId::OtherReserve(asset) => AssetXConverter::reverse_ref(asset).ok(),
		}
	}
}

parameter_types! {
	pub const BaseXcmWeight: Weight = 100_000_000;
	// This is how we are going to detect whether the asset is a Reserve asset
	// This however is the chain part only
	pub SelfLocation: MultiLocation = MultiLocation {
		parents:1,
		interior: Junctions::X1(
			Parachain(ParachainInfo::parachain_id().into())
		)
	};
}

impl orml_xtokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = AccountIdToMultiLocation<AccountId>;
	type CurrencyIdConvert =
		CurrencyIdtoMultiLocation<AsAssetType<AssetId, AssetType, AssetManager>>;
	type XcmExecutor = XcmExecutor;
	type SelfLocation = SelfLocation;
	type Weigher = XcmWeigher;
	type BaseXcmWeight = BaseXcmWeight;
	type LocationInverter = LocationInverter<Ancestry>;
}

// For now we only allow to transact in the relay, although this might change in the future
// Transactors just defines the chains in which we allow transactions to be issued through
// xcm
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum Transactors {
	Relay,
}

impl TryFrom<u8> for Transactors {
	type Error = ();
	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			0u8 => Ok(Transactors::Relay),
			_ => Err(()),
		}
	}
}

impl UtilityEncodeCall for Transactors {
	fn encode_call(self, call: UtilityAvailableCalls) -> Vec<u8> {
		match self {
			// Shall we use westend for moonbase? The tests are probably based on rococo
			// but moonbase-alpha is attached to westend-runtime I think
			Transactors::Relay => moonbeam_relay_encoder::westend::WestendEncoder.encode_call(call),
		}
	}
}

impl XcmTransact for Transactors {
	fn destination(self) -> MultiLocation {
		match self {
			Transactors::Relay => MultiLocation::parent(),
		}
	}
}

impl xcm_transactor::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Transactor = Transactors;
	type DerivativeAddressRegistrationOrigin = EnsureRoot<AccountId>;
	type SovereignAccountDispatcherOrigin = EnsureRoot<AccountId>;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = AccountIdToMultiLocation<AccountId>;
	type CurrencyIdToMultiLocation =
		CurrencyIdtoMultiLocation<AsAssetType<AssetId, AssetType, AssetManager>>;
	type XcmExecutor = XcmExecutor;
	type XcmSender = XcmRouter;
	type SelfLocation = SelfLocation;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type LocationInverter = LocationInverter<Ancestry>;
	type BaseXcmWeight = BaseXcmWeight;
}

/// Call filter used during Phase 3 of the Moonriver rollout
pub struct MaintenanceFilter;
impl Contains<Call> for MaintenanceFilter {
	fn contains(c: &Call) -> bool {
		match c {
			Call::Assets(_) => false,
			Call::Balances(_) => false,
			Call::CrowdloanRewards(_) => false,
			Call::Ethereum(_) => false,
			Call::EVM(_) => false,
			Call::XTokens(_) => false,
			Call::XcmTransactor(_) => false,
			_ => true,
		}
	}
}

/// Normal Call Filter
/// We dont allow to create nor mint assets, this for now is disabled
/// We only allow transfers. For now creation of assets will go through
/// asset-manager, while minting/burning only happens through xcm messages
/// This can change in the future
pub struct NormalFilter;
impl Contains<Call> for NormalFilter {
	fn contains(c: &Call) -> bool {
		match c {
			Call::Assets(method) => match method {
				pallet_assets::Call::transfer { .. } => true,
				pallet_assets::Call::transfer_keep_alive { .. } => true,
				pallet_assets::Call::approve_transfer { .. } => true,
				pallet_assets::Call::transfer_approved { .. } => true,
				pallet_assets::Call::cancel_approval { .. } => true,
				_ => false,
			},
			// We just want to enable this in case of live chains, since the default version
			// is populated at genesis
			Call::PolkadotXcm(method) => match method {
				pallet_xcm::Call::force_default_xcm_version { .. } => true,
				_ => false,
			},
			_ => true,
		}
	}
}

use cumulus_primitives_core::{
	relay_chain::BlockNumber as RelayBlockNumber, DmpMessageHandler, ParaId, XcmpMessageHandler,
};
pub struct MaintenanceDmpHandler;
impl DmpMessageHandler for MaintenanceDmpHandler {
	// This implementation makes messages be queued
	// Since the limit is 0, messages are queued for next iteration
	fn handle_dmp_messages(
		iter: impl Iterator<Item = (RelayBlockNumber, Vec<u8>)>,
		_limit: Weight,
	) -> Weight {
		DmpQueue::handle_dmp_messages(iter, 0)
	}
}

pub struct MaintenanceXcmpHandler;
impl XcmpMessageHandler for MaintenanceXcmpHandler {
	// This implementation makes messages be queued
	// Since the limit is 0, messages are queued for next iteration
	fn handle_xcmp_messages<'a, I: Iterator<Item = (ParaId, RelayBlockNumber, &'a [u8])>>(
		iter: I,
		_limit: Weight,
	) -> Weight {
		XcmpQueue::handle_xcmp_messages(iter, 0)
	}
}

/// The hooks we wnat to run in Maintenance Mode
pub struct MaintenanceHooks;

impl OnInitialize<BlockNumber> for MaintenanceHooks {
	fn on_initialize(n: BlockNumber) -> Weight {
		AllPallets::on_initialize(n)
	}
}

// return 0
// For some reason using empty tuple () isnt working
// There exist only two pallets that use onIdle and these are xcmp and dmp queues
// For some reason putting an empty tumple does not work (transaction never finishes)
// We use an empty onIdle, if on the future we want one of the pallets to execute it
// we need to provide it here
impl OnIdle<BlockNumber> for MaintenanceHooks {
	fn on_idle(_n: BlockNumber, _max_weight: Weight) -> Weight {
		0
	}
}

impl OnRuntimeUpgrade for MaintenanceHooks {
	fn on_runtime_upgrade() -> Weight {
		AllPallets::on_runtime_upgrade()
	}
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		AllPallets::pre_upgrade()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		AllPallets::post_upgrade()
	}
}

impl OnFinalize<BlockNumber> for MaintenanceHooks {
	fn on_finalize(n: BlockNumber) {
		AllPallets::on_finalize(n)
	}
}

impl OffchainWorker<BlockNumber> for MaintenanceHooks {
	fn offchain_worker(n: BlockNumber) {
		AllPallets::offchain_worker(n)
	}
}

// AllPallets here implies all the specfied pallets in the runtime, except frame_system,
// will run the associated hook
// AllPallets is simply a nested tuple containing all the pallets except System
// In cases where we need only specific pallets to run the hook,
// we should state them in nested tuples
impl pallet_maintenance_mode::Config for Runtime {
	type Event = Event;
	type NormalCallFilter = NormalFilter;
	type MaintenanceCallFilter = MaintenanceFilter;
	type MaintenanceOrigin =
		pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, TechCommitteeInstance>;
	type NormalDmpHandler = DmpQueue;
	type MaintenanceDmpHandler = MaintenanceDmpHandler;
	type NormalXcmpHandler = XcmpQueue;
	type MaintenanceXcmpHandler = MaintenanceXcmpHandler;
	// We use AllPallets because we dont want to change the hooks in normal operation
	type NormalExecutiveHooks = AllPallets;
	type MaitenanceExecutiveHooks = MaintenanceHooks;
}

impl pallet_proxy_genesis_companion::Config for Runtime {
	type ProxyType = ProxyType;
}

construct_runtime! {
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 0,
		Utility: pallet_utility::{Pallet, Call, Event} = 1,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 2,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 3,
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 4,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 5,
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>} = 6,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 7,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 8,
		EthereumChainId: pallet_ethereum_chain_id::{Pallet, Storage, Config} = 9,
		EVM: pallet_evm::{Pallet, Config, Call, Storage, Event<T>} = 10,
		Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Origin, Config} = 11,
		ParachainStaking: parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>} = 12,
		Scheduler: pallet_scheduler::{Pallet, Storage, Config, Event<T>, Call} = 13,
		Democracy: pallet_democracy::{Pallet, Storage, Config<T>, Event<T>, Call} = 14,
		CouncilCollective:
			pallet_collective::<Instance1>::{Pallet, Call, Storage, Event<T>, Origin<T>, Config<T>} = 15,
		TechCommitteeCollective:
			pallet_collective::<Instance2>::{Pallet, Call, Storage, Event<T>, Origin<T>, Config<T>} = 16,
		Treasury: pallet_treasury::{Pallet, Storage, Config, Event<T>, Call} = 17,
		AuthorInherent: pallet_author_inherent::{Pallet, Call, Storage, Inherent} = 18,
		AuthorFilter: pallet_author_slot_filter::{Pallet, Call, Storage, Event, Config} = 19,
		CrowdloanRewards: pallet_crowdloan_rewards::{Pallet, Call, Config<T>, Storage, Event<T>} = 20,
		AuthorMapping: pallet_author_mapping::{Pallet, Call, Config<T>, Storage, Event<T>} = 21,
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 22,
		MaintenanceMode: pallet_maintenance_mode::{Pallet, Call, Config, Storage, Event} = 23,
		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 24,
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 25,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 26,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 27,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Storage, Event<T>, Origin, Config} = 28,
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>} = 29,
		XTokens: orml_xtokens::{Pallet, Call, Storage, Event<T>} = 30,
		AssetManager: pallet_asset_manager::{Pallet, Call, Storage, Event<T>} = 31,
		Migrations: pallet_migrations::{Pallet, Storage, Config, Event<T>} = 32,
		XcmTransactor: xcm_transactor::{Pallet, Call, Storage, Event<T>} = 33,
		ProxyGenesisCompanion: pallet_proxy_genesis_companion::{Pallet, Config<T>} = 34,

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
pub type UncheckedExtrinsic =
	fp_self_contained::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = fp_self_contained::CheckedExtrinsic<AccountId, Call, SignedExtra, H160>;
/// Executive: handles dispatch to the various pallets.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	pallet_maintenance_mode::ExecutiveHooks<Runtime>,
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
			xt: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			// Filtered calls should not enter the tx pool as they'll fail if inserted.
			// If this call is not allowed, we return early.
			if !<Runtime as frame_system::Config>::BaseCallFilter::contains(&xt.0.function) {
				return InvalidTransaction::Call.into();
			}

			// This runtime uses Substrate's pallet transaction payment. This
			// makes the chain feel like a standard Substrate chain when submitting
			// frame transactions and using Substrate ecosystem tools. It has the downside that
			// transaction are not prioritized by gas_price. The following code reprioritizes
			// transactions to overcome this.
			//
			// A more elegant, ethereum-first solution is
			// a pallet that replaces pallet transaction payment, and allows users
			// to directly specify a gas price rather than computing an effective one.
			// #HopefullySomeday

			// First we pass the transactions to the standard FRAME executive. This calculates all the
			// necessary tags, longevity and other properties that we will leave unchanged.
			// This also assigns some priority that we don't care about and will overwrite next.
			let mut intermediate_valid = Executive::validate_transaction(source, xt.clone(), block_hash)?;

			let dispatch_info = xt.get_dispatch_info();

			// If this is a pallet ethereum transaction, then its priority is already set
			// according to gas price from pallet ethereum. If it is any other kind of transaction,
			// we modify its priority.
			Ok(match &xt.0.function {
				Call::Ethereum(transact { .. }) => intermediate_valid,
				_ if dispatch_info.class != DispatchClass::Normal => intermediate_valid,
				_ => {
					let tip = match xt.0.signature {
						None => 0,
						Some((_, _, ref signed_extra)) => {
							// Yuck, this depends on the index of charge transaction in Signed Extra
							let charge_transaction = &signed_extra.6;
							charge_transaction.tip()
						}
					};

					// Calculate the fee that will be taken by pallet transaction payment
					let fee: u64 = TransactionPayment::compute_fee(
						xt.encode().len() as u32,
						&dispatch_info,
						tip,
					).saturated_into();

					// Calculate how much gas this effectively uses according to the existing mapping
					let effective_gas =
						<Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
							dispatch_info.weight
						);

					// Here we calculate an ethereum-style effective gas price using the
					// current fee of the transaction. Because the weight -> gas conversion is
					// lossy, we have to handle the case where a very low weight maps to zero gas.
					let effective_gas_price = if effective_gas > 0 {
						fee / effective_gas
					} else {
						// If the effective gas was zero, we just act like it was 1.
						fee
					};

					// Overwrite the original prioritization with this ethereum one
					intermediate_valid.priority = effective_gas_price;
					intermediate_valid
				}
			})
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

runtime_common::impl_self_contained_call!();

#[cfg(test)]
mod tests {
	use super::{currency::*, *};

	#[test]
	fn currency_constants_are_correct() {
		assert_eq!(SUPPLY_FACTOR, 1);

		// txn fees
		assert_eq!(TRANSACTION_BYTE_FEE, Balance::from(10 * MICROUNIT));
		assert_eq!(OperationalFeeMultiplier::get(), 5_u8);
		assert_eq!(STORAGE_BYTE_FEE, Balance::from(100 * MICROUNIT));
		assert_eq!(FixedGasPrice::min_gas_price(), (1 * GIGAWEI).into());

		// democracy minimums
		assert_eq!(MinimumDeposit::get(), Balance::from(4 * UNIT));
		assert_eq!(PreimageByteDeposit::get(), Balance::from(100 * MICROUNIT));
		assert_eq!(ProposalBondMinimum::get(), Balance::from(1 * UNIT));

		// pallet_identity deposits
		assert_eq!(
			BasicDeposit::get(),
			Balance::from(1 * UNIT + 25800 * MICROUNIT)
		);
		assert_eq!(FieldDeposit::get(), Balance::from(6600 * MICROUNIT));
		assert_eq!(
			SubAccountDeposit::get(),
			Balance::from(1 * UNIT + 5300 * MICROUNIT)
		);

		// staking minimums
		assert_eq!(MinCollatorStk::get(), Balance::from(1 * KILOUNIT));
		assert_eq!(MinCandidateStk::get(), Balance::from(500 * UNIT));
		assert_eq!(MinDelegatorStk::get(), Balance::from(5 * UNIT));

		// crowdloan min reward
		assert_eq!(MinimumReward::get(), Balance::from(0u128));

		// deposit for AuthorMapping
		assert_eq!(DepositAmount::get(), Balance::from(100 * UNIT));

		// proxy deposits
		assert_eq!(
			ProxyDepositBase::get(),
			Balance::from(1 * UNIT + 800 * MICROUNIT)
		);
		assert_eq!(ProxyDepositFactor::get(), Balance::from(2100 * MICROUNIT));
		assert_eq!(
			AnnouncementDepositBase::get(),
			Balance::from(1 * UNIT + 800 * MICROUNIT)
		);
		assert_eq!(
			AnnouncementDepositFactor::get(),
			Balance::from(5600 * MICROUNIT)
		);
	}
}
