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

use account::AccountId20;
use cumulus_pallet_parachain_system::RelaychainBlockNumberProvider;
use fp_rpc::TransactionStatus;

// Re-export required by get! macro.
#[cfg(feature = "std")]
pub use fp_evm::GenesisAccount;
pub use frame_support::traits::Get;
use frame_support::{
	construct_runtime,
	pallet_prelude::DispatchResult,
	parameter_types,
	traits::{
		ConstBool, ConstU128, ConstU16, ConstU32, ConstU64, ConstU8, Contains,
		Currency as CurrencyT, EnsureOneOf, EqualPrivilegeOnly, Imbalance, InstanceFilter,
		OffchainWorker, OnFinalize, OnIdle, OnInitialize, OnRuntimeUpgrade, OnUnbalanced,
	},
	weights::{
		constants::{RocksDbWeight, WEIGHT_PER_SECOND},
		ConstantMultiplier, DispatchClass, GetDispatchInfo, Weight, WeightToFeeCoefficient,
		WeightToFeeCoefficients, WeightToFeePolynomial,
	},
	PalletId,
};
use frame_system::{EnsureRoot, EnsureSigned};
pub use moonbeam_core_primitives::{
	AccountId, AccountIndex, Address, AssetId, Balance, BlockNumber, DigestItem, Hash, Header,
	Index, Signature,
};
use moonbeam_rpc_primitives_txpool::TxPoolResponse;
use pallet_balances::NegativeImbalance;
use pallet_ethereum::Call::transact;
use pallet_ethereum::Transaction as EthereumTransaction;
use pallet_evm::{
	Account as EVMAccount, EVMCurrencyAdapter, EnsureAddressNever, EnsureAddressRoot,
	FeeCalculator, GasWeightMapping, OnChargeEVMTransaction as OnChargeEVMTransactionT, Runner,
};
pub use pallet_parachain_staking::{InflationInfo, Range};
use pallet_transaction_payment::{CurrencyAdapter, Multiplier, TargetedFeeAdjustment};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use smallvec::smallvec;
use sp_api::impl_runtime_apis;
use sp_core::{OpaqueMetadata, H160, H256, U256};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{
		BlakeTwo256, Block as BlockT, DispatchInfoOf, Dispatchable, IdentityLookup,
		PostDispatchInfoOf, UniqueSaturatedInto,
	},
	transaction_validity::{
		InvalidTransaction, TransactionSource, TransactionValidity, TransactionValidityError,
	},
	ApplyExtrinsicResult, FixedPointNumber, Perbill, Percent, Permill, Perquintill,
	SaturatedConversion,
};
use sp_std::{convert::TryFrom, prelude::*};

use cumulus_primitives_core::{relay_chain::BlockNumber as RelayBlockNumber, DmpMessageHandler};

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use nimbus_primitives::{CanAuthor, NimbusId};

mod precompiles;
pub use precompiles::{
	MoonbeamPrecompiles, FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
	LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX,
};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

pub type Precompiles = MoonbeamPrecompiles<Runtime>;

pub mod asset_config;
pub mod xcm_config;

/// GLMR, the native token, uses 18 decimals of precision.
pub mod currency {
	use super::Balance;

	// Provide a common factor between runtimes based on a supply of 10_000_000 tokens.
	pub const SUPPLY_FACTOR: Balance = 100;

	pub const WEI: Balance = 1;
	pub const KILOWEI: Balance = 1_000;
	pub const MEGAWEI: Balance = 1_000_000;
	pub const GIGAWEI: Balance = 1_000_000_000;
	pub const MICROGLMR: Balance = 1_000_000_000_000;
	pub const MILLIGLMR: Balance = 1_000_000_000_000_000;
	pub const GLMR: Balance = 1_000_000_000_000_000_000;
	pub const KILOGLMR: Balance = 1_000_000_000_000_000_000_000;

	pub const TRANSACTION_BYTE_FEE: Balance = 1 * GIGAWEI * SUPPLY_FACTOR;
	pub const STORAGE_BYTE_FEE: Balance = 100 * MICROGLMR * SUPPLY_FACTOR;
	pub const WEIGHT_FEE: Balance = 50 * KILOWEI * SUPPLY_FACTOR;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 100 * MILLIGLMR * SUPPLY_FACTOR + (bytes as Balance) * STORAGE_BYTE_FEE
	}
}

/// Maximum weight per block
pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;

pub const MILLISECS_PER_BLOCK: u64 = 12000;
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;
pub const WEEKS: BlockNumber = DAYS * 7;
pub const MONTHS: BlockNumber = DAYS * 30;
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
			pub vrf: session_keys_primitives::VrfSessionKey,
		}
	}
}

/// This runtime version.
/// The spec_version is composed of 2x2 digits. The first 2 digits represent major changes
/// that can't be skipped, such as data migration upgrades. The last 2 digits represent minor
/// changes which can be skipped.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("moonbeam"),
	impl_name: create_runtime_str!("moonbeam"),
	authoring_version: 3,
	spec_version: 1701,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 2,
	state_version: 0,
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
	pub const Version: RuntimeVersion = VERSION;
	/// We allow for one half second of compute with a 6 second average block time.
	/// These values are dictated by Polkadot for the parachain.
	pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
		::with_sensible_defaults(MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO);
	/// We allow for 5 MB blocks.
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
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
	type BlockHashCount = ConstU32<256>;
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
	type SS58Prefix = ConstU16<1284>;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<1>;
	type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

impl pallet_balances::Config for Runtime {
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 4];
	type MaxLocks = ConstU32<50>;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<0>;
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

pub struct LengthToFee;
impl WeightToFeePolynomial for LengthToFee {
	type Balance = Balance;

	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		smallvec![
			WeightToFeeCoefficient {
				degree: 1,
				coeff_frac: Perbill::zero(),
				coeff_integer: currency::TRANSACTION_BYTE_FEE,
				negative: false,
			},
			WeightToFeeCoefficient {
				degree: 3,
				coeff_frac: Perbill::zero(),
				coeff_integer: 1 * currency::SUPPLY_FACTOR,
				negative: false,
			},
		]
	}
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees<Runtime>>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = ConstantMultiplier<Balance, ConstU128<{ currency::WEIGHT_FEE }>>;
	type LengthToFee = LengthToFee;
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
	pub PrecompilesValue: MoonbeamPrecompiles<Runtime> = MoonbeamPrecompiles::<_>::new();
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> (U256, Weight) {
		(
			(1 * currency::GIGAWEI * currency::SUPPLY_FACTOR).into(),
			0u64,
		)
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

use frame_support::traits::FindAuthor;
//TODO It feels like this shold be able to work for any T: H160, but I tried for
// embarassingly long and couldn't figure that out.

/// The author inherent provides a AccountId20, but pallet evm needs an H160.
/// This simple adapter makes the conversion.
pub struct FindAuthorAdapter<Inner>(sp_std::marker::PhantomData<Inner>);

impl<Inner> FindAuthor<H160> for FindAuthorAdapter<Inner>
where
	Inner: FindAuthor<AccountId20>,
{
	fn find_author<'a, I>(digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (sp_runtime::ConsensusEngineId, &'a [u8])>,
	{
		Inner::find_author(digests).map(Into::into)
	}
}

moonbeam_runtime_common::impl_on_charge_evm_transaction!();

impl pallet_evm::Config for Runtime {
	type FeeCalculator = FixedGasPrice;
	type GasWeightMapping = MoonbeamGasWeightMapping;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = moonbeam_runtime_common::IntoAddressMapping;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = MoonbeamPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = EthereumChainId;
	type OnChargeTransaction = OnChargeEVMTransaction<DealWithFees<Runtime>>;
	type BlockGasLimit = BlockGasLimit;
	type FindAuthor = FindAuthorAdapter<AuthorInherent>;
}

parameter_types! {
	pub DefaultBaseFeePerGas: U256 = (1 * currency::GIGAWEI * currency::SUPPLY_FACTOR).into();
}

pub struct BaseFeeThreshold;
impl pallet_base_fee::BaseFeeThreshold for BaseFeeThreshold {
	fn lower() -> Permill {
		Permill::zero()
	}
	fn ideal() -> Permill {
		Permill::from_parts(500_000)
	}
	fn upper() -> Permill {
		Permill::from_parts(1_000_000)
	}
}

impl pallet_base_fee::Config for Runtime {
	type Event = Event;
	type Threshold = BaseFeeThreshold;
	// Tells `pallet_base_fee` whether to calculate a new BaseFee `on_finalize` or not.
	type IsActive = ConstBool<false>;
	type DefaultBaseFeePerGas = DefaultBaseFeePerGas;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = NORMAL_DISPATCH_RATIO * BlockWeights::get().max_block;
}

impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = ConstU32<50>;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type PreimageProvider = ();
	type NoPreimagePostponement = ();
}

type CouncilInstance = pallet_collective::Instance1;
type TechCommitteeInstance = pallet_collective::Instance2;

impl pallet_collective::Config<CouncilInstance> for Runtime {
	type Origin = Origin;
	type Event = Event;
	type Proposal = Call;
	/// The maximum amount of time (in blocks) for council members to vote on motions.
	/// Motions may end in fewer blocks if enough votes are cast to determine the result.
	type MotionDuration = ConstU32<{ 3 * DAYS }>;
	/// The maximum number of Proposlas that can be open in the council at once.
	type MaxProposals = ConstU32<100>;
	/// The maximum number of council members.
	type MaxMembers = ConstU32<100>;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

impl pallet_collective::Config<TechCommitteeInstance> for Runtime {
	type Origin = Origin;
	type Event = Event;
	type Proposal = Call;
	/// The maximum amount of time (in blocks) for technical committee members to vote on motions.
	/// Motions may end in fewer blocks if enough votes are cast to determine the result.
	type MotionDuration = ConstU32<{ 3 * DAYS }>;
	/// The maximum number of Proposlas that can be open in the technical committee at once.
	type MaxProposals = ConstU32<100>;
	/// The maximum number of technical committee members.
	type MaxMembers = ConstU32<100>;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

impl pallet_democracy::Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = ConstU32<{ 2 * DAYS }>;
	type LaunchPeriod = ConstU32<{ 7 * DAYS }>;
	type VotingPeriod = ConstU32<{ 14 * DAYS }>;
	type VoteLockingPeriod = ConstU32<{ 7 * DAYS }>;
	type FastTrackVotingPeriod = ConstU32<{ 1 * DAYS }>;
	type MinimumDeposit = ConstU128<{ 4 * currency::GLMR * currency::SUPPLY_FACTOR }>;
	/// To decide what their next motion is.
	type ExternalOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilInstance, 1, 2>;
	/// To have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilInstance, 3, 5>;
	/// To have the next scheduled referendum be a straight default-carries (NTB) vote.
	type ExternalDefaultOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilInstance, 3, 5>;
	/// To allow a shorter voting/enactment period for external proposals.
	type FastTrackOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, TechCommitteeInstance, 1, 2>;
	/// To instant fast track.
	type InstantOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, TechCommitteeInstance, 3, 5>;
	// To cancel a proposal which has been passed.
	type CancellationOrigin = EnsureOneOf<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilInstance, 3, 5>,
	>;
	// To cancel a proposal before it has been passed.
	type CancelProposalOrigin = EnsureOneOf<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, TechCommitteeInstance, 3, 5>,
	>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cooloff period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechCommitteeInstance>;
	type CooloffPeriod = ConstU32<{ 7 * DAYS }>;
	type PreimageByteDeposit = ConstU128<{ currency::STORAGE_BYTE_FEE }>;
	type Slash = ();
	type InstantAllowed = ConstBool<true>;
	type Scheduler = Scheduler;
	type MaxVotes = ConstU32<100>;
	type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilInstance>;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
	type MaxProposals = ConstU32<100>;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const TreasuryId: PalletId = PalletId(*b"py/trsry");
}

type TreasuryApproveOrigin = EnsureOneOf<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, CouncilInstance, 3, 5>,
>;

type TreasuryRejectOrigin = EnsureOneOf<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, CouncilInstance, 1, 2>,
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
	type ProposalBondMinimum = ConstU128<{ 1 * currency::GLMR * currency::SUPPLY_FACTOR }>;
	type SpendPeriod = ConstU32<{ 6 * DAYS }>;
	type Burn = ();
	type BurnDestination = ();
	type MaxApprovals = ConstU32<100>;
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	type SpendFunds = ();
	type ProposalBondMaximum = ();
}

type IdentityForceOrigin = EnsureOneOf<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, CouncilInstance, 1, 2>,
>;
type IdentityRegistrarOrigin = EnsureOneOf<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, CouncilInstance, 1, 2>,
>;

impl pallet_identity::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	// Add one item in storage and take 258 bytes
	type BasicDeposit = ConstU128<{ currency::deposit(1, 258) }>;
	// Not add any item to the storage but takes 66 bytes
	type FieldDeposit = ConstU128<{ currency::deposit(0, 66) }>;
	// Add one item in storage and take 53 bytes
	type SubAccountDeposit = ConstU128<{ currency::deposit(1, 53) }>;
	type MaxSubAccounts = ConstU32<100>;
	type MaxAdditionalFields = ConstU32<100>;
	type MaxRegistrars = ConstU32<20>;
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
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnSystemEvent = ();
	type SelfParaId = ParachainInfo;
	type DmpMessageHandler = MaintenanceMode;
	type ReservedDmpWeight = ConstU64<{ MAXIMUM_BLOCK_WEIGHT / 4 }>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ConstU64<{ MAXIMUM_BLOCK_WEIGHT / 4 }>;
}
impl parachain_info::Config for Runtime {}

parameter_types! {
	/// Default fixed percent a collator takes off the top of due rewards
	pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(20);
	/// Default percent of inflation set aside for parachain bond every round
	pub const DefaultParachainBondReservePercent: Percent = Percent::from_percent(30);
}

pub struct OnCollatorPayout;
impl pallet_parachain_staking::OnCollatorPayout<AccountId, Balance> for OnCollatorPayout {
	fn on_collator_payout(
		for_round: pallet_parachain_staking::RoundIndex,
		collator_id: AccountId,
		amount: Balance,
	) -> Weight {
		MoonbeamOrbiters::distribute_rewards(for_round, collator_id, amount)
	}
}
pub struct OnNewRound;
impl pallet_parachain_staking::OnNewRound for OnNewRound {
	fn on_new_round(round_index: pallet_parachain_staking::RoundIndex) -> Weight {
		MoonbeamOrbiters::on_new_round(round_index)
	}
}

impl pallet_parachain_staking::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type MonetaryGovernanceOrigin = EnsureRoot<AccountId>;
	/// Minimum round length is 2 minutes (10 * 12 second block times)
	type MinBlocksPerRound = ConstU32<10>;
	/// Blocks per round
	type DefaultBlocksPerRound = ConstU32<{ 6 * HOURS }>;
	/// Rounds before the collator leaving the candidates request can be executed
	type LeaveCandidatesDelay = ConstU32<{ 4 * 7 }>;
	/// Rounds before the candidate bond increase/decrease can be executed
	type CandidateBondLessDelay = ConstU32<{ 4 * 7 }>;
	/// Rounds before the delegator exit can be executed
	type LeaveDelegatorsDelay = ConstU32<{ 4 * 7 }>;
	/// Rounds before the delegator revocation can be executed
	type RevokeDelegationDelay = ConstU32<{ 4 * 7 }>;
	/// Rounds before the delegator bond increase/decrease can be executed
	type DelegationBondLessDelay = ConstU32<{ 4 * 7 }>;
	/// Rounds before the reward is paid
	type RewardPaymentDelay = ConstU32<2>;
	/// Minimum collators selected per round, default at genesis and minimum forever after
	type MinSelectedCandidates = ConstU32<8>;
	/// Maximum top delegations per candidate
	type MaxTopDelegationsPerCandidate = ConstU32<300>;
	/// Maximum bottom delegations per candidate
	type MaxBottomDelegationsPerCandidate = ConstU32<50>;
	/// Maximum delegations per delegator
	type MaxDelegationsPerDelegator = ConstU32<100>;
	type DefaultCollatorCommission = DefaultCollatorCommission;
	type DefaultParachainBondReservePercent = DefaultParachainBondReservePercent;
	/// Minimum stake required to become a collator
	type MinCollatorStk = ConstU128<{ 1000 * currency::GLMR * currency::SUPPLY_FACTOR }>;
	/// Minimum stake required to be reserved to be a candidate
	type MinCandidateStk = ConstU128<{ 1000 * currency::GLMR * currency::SUPPLY_FACTOR }>;
	/// Minimum stake required to be reserved to be a delegator
	type MinDelegation = ConstU128<{ 500 * currency::MILLIGLMR * currency::SUPPLY_FACTOR }>;
	/// Minimum stake required to be reserved to be a delegator
	type MinDelegatorStk = ConstU128<{ 500 * currency::MILLIGLMR * currency::SUPPLY_FACTOR }>;
	type OnCollatorPayout = OnCollatorPayout;
	type OnNewRound = OnNewRound;
	type WeightInfo = pallet_parachain_staking::weights::SubstrateWeight<Runtime>;
}

impl pallet_author_inherent::Config for Runtime {
	type SlotBeacon = RelaychainBlockNumberProvider<Self>;
	type AccountLookup = MoonbeamOrbiters;
	type EventHandler = ParachainStaking;
	type CanAuthor = AuthorFilter;
	type WeightInfo = pallet_author_inherent::weights::SubstrateWeight<Runtime>;
}

impl pallet_author_slot_filter::Config for Runtime {
	type Event = Event;
	type RandomnessSource = RandomnessCollectiveFlip;
	type PotentialAuthors = ParachainStaking;
	type WeightInfo = pallet_author_slot_filter::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const InitializationPayment: Perbill = Perbill::from_percent(30);
	pub const RelaySignaturesThreshold: Perbill = Perbill::from_percent(100);
	pub const SignatureNetworkIdentifier:  &'static [u8] = b"moonbeam-";
}

impl pallet_crowdloan_rewards::Config for Runtime {
	type Event = Event;
	type Initialized = ConstBool<false>;
	type InitializationPayment = InitializationPayment;
	type MaxInitContributors = ConstU32<500>;
	type MinimumReward = ConstU128<0>;
	type RewardCurrency = Balances;
	type RelayChainAccountId = [u8; 32];
	type RewardAddressAssociateOrigin = EnsureSigned<Self::AccountId>;
	type RewardAddressChangeOrigin = EnsureSigned<Self::AccountId>;
	type RewardAddressRelayVoteThreshold = RelaySignaturesThreshold;
	type SignatureNetworkIdentifier = SignatureNetworkIdentifier;
	type VestingBlockNumber = cumulus_primitives_core::relay_chain::BlockNumber;
	type VestingBlockProvider =
		cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Self>;
	type WeightInfo = pallet_crowdloan_rewards::weights::SubstrateWeight<Runtime>;
}

// This is a simple session key manager. It should probably either work with, or be replaced
// entirely by pallet sessions
impl pallet_author_mapping::Config for Runtime {
	type Event = Event;
	type DepositCurrency = Balances;
	type DepositAmount = ConstU128<{ 100 * currency::GLMR * currency::SUPPLY_FACTOR }>;
	type Keys = session_keys_primitives::VrfId;
	type WeightInfo = pallet_author_mapping::weights::SubstrateWeight<Runtime>;
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
	/// Allow extrinsic related to IdentityJudgement.
	IdentityJudgement = 7,
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
						| Call::Identity(..) | Call::TechCommitteeCollective(..)
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
				Call::ParachainStaking(..)
					| Call::Utility(..) | Call::AuthorMapping(..)
					| Call::MoonbeamOrbiters(..)
			),
			ProxyType::CancelProxy => matches!(
				c,
				Call::Proxy(pallet_proxy::Call::reject_announcement { .. })
			),
			ProxyType::Balances => matches!(c, Call::Balances(..) | Call::Utility(..)),
			ProxyType::AuthorMapping => matches!(c, Call::AuthorMapping(..)),
			ProxyType::IdentityJudgement => matches!(
				c,
				Call::Identity(pallet_identity::Call::provide_judgement { .. }) | Call::Utility(..)
			),
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
	// One storage item; key size 32, value size 8
	type ProxyDepositBase = ConstU128<{ currency::deposit(1, 8) }>;
	// Additional storage item size of 21 bytes (20 bytes AccountId + 1 byte sizeof(ProxyType)).
	type ProxyDepositFactor = ConstU128<{ currency::deposit(0, 21) }>;
	type MaxProxies = ConstU32<32>;
	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
	type MaxPending = ConstU32<32>;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = ConstU128<{ currency::deposit(1, 8) }>;
	// Additional storage item size of 56 bytes:
	// - 20 bytes AccountId
	// - 32 bytes Hasher (Blake2256)
	// - 4 bytes BlockNumber (u32)
	type AnnouncementDepositFactor = ConstU128<{ currency::deposit(0, 56) }>;
}

impl pallet_migrations::Config for Runtime {
	type Event = Event;
	type MigrationsList = moonbeam_runtime_common::migrations::CommonMigrations<
		Runtime,
		CouncilCollective,
		TechCommitteeCollective,
	>;
}

/// Maintenance mode Call filter
pub struct MaintenanceFilter;
impl Contains<Call> for MaintenanceFilter {
	fn contains(c: &Call) -> bool {
		match c {
			Call::Assets(_) => false,
			Call::LocalAssets(_) => false,
			Call::Balances(_) => false,
			Call::CrowdloanRewards(_) => false,
			Call::Ethereum(_) => false,
			Call::EVM(_) => false,
			Call::Identity(_) => false,
			Call::XTokens(_) => false,
			Call::ParachainStaking(_) => false,
			Call::MoonbeamOrbiters(_) => false,
			Call::PolkadotXcm(_) => false,
			Call::Treasury(_) => false,
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
			// We want to disable create, as we dont want users to be choosing the
			// assetId of their choice
			// We also disable destroy, as we want to route destroy through the
			// asset-manager, which guarantees the removal both at the EVM and
			// substrate side of things
			Call::LocalAssets(method) => match method {
				pallet_assets::Call::create { .. } => false,
				pallet_assets::Call::destroy { .. } => false,
				_ => true,
			},
			// We just want to enable this in case of live chains, since the default version
			// is populated at genesis
			Call::PolkadotXcm(method) => match method {
				pallet_xcm::Call::force_default_xcm_version { .. } => true,
				_ => false,
			},
			// We filter anonymous proxy as they make "reserve" inconsistent
			// See: https://github.com/paritytech/substrate/blob/37cca710eed3dadd4ed5364c7686608f5175cce1/frame/proxy/src/lib.rs#L270 // editorconfig-checker-disable-line
			Call::Proxy(method) => match method {
				pallet_proxy::Call::anonymous { .. } => false,
				pallet_proxy::Call::kill_anonymous { .. } => false,
				_ => true,
			},
			// We filter for now transact through signed
			Call::XcmTransactor(method) => match method {
				pallet_xcm_transactor::Call::transact_through_signed_multilocation { .. } => false,
				pallet_xcm_transactor::Call::transact_through_signed { .. } => false,
				_ => true,
			},
			_ => true,
		}
	}
}

pub struct XcmExecutionManager;
impl pallet_maintenance_mode::PauseXcmExecution for XcmExecutionManager {
	fn suspend_xcm_execution() -> DispatchResult {
		XcmpQueue::suspend_xcm_execution(Origin::root())
	}
	fn resume_xcm_execution() -> DispatchResult {
		XcmpQueue::resume_xcm_execution(Origin::root())
	}
}

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

/// The hooks we want to run in Maintenance Mode
pub struct MaintenanceHooks;

impl OnInitialize<BlockNumber> for MaintenanceHooks {
	fn on_initialize(n: BlockNumber) -> Weight {
		AllPalletsReversedWithSystemFirst::on_initialize(n)
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
		AllPalletsReversedWithSystemFirst::on_runtime_upgrade()
	}
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		AllPalletsReversedWithSystemFirst::pre_upgrade()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		AllPalletsReversedWithSystemFirst::post_upgrade()
	}
}

impl OnFinalize<BlockNumber> for MaintenanceHooks {
	fn on_finalize(n: BlockNumber) {
		AllPalletsReversedWithSystemFirst::on_finalize(n)
	}
}

impl OffchainWorker<BlockNumber> for MaintenanceHooks {
	fn offchain_worker(n: BlockNumber) {
		AllPalletsReversedWithSystemFirst::offchain_worker(n)
	}
}

impl pallet_maintenance_mode::Config for Runtime {
	type Event = Event;
	type NormalCallFilter = NormalFilter;
	type MaintenanceCallFilter = MaintenanceFilter;
	type MaintenanceOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, TechCommitteeInstance, 2, 3>;
	type XcmExecutionManager = XcmExecutionManager;
	type NormalDmpHandler = DmpQueue;
	type MaintenanceDmpHandler = MaintenanceDmpHandler;
	// We use AllPalletsReversedWithSystemFirst because we dont want to change the hooks in normal
	// operation
	type NormalExecutiveHooks = AllPalletsReversedWithSystemFirst;
	type MaintenanceExecutiveHooks = MaintenanceHooks;
}

impl pallet_proxy_genesis_companion::Config for Runtime {
	type ProxyType = ProxyType;
}

parameter_types! {
	pub OrbiterReserveIdentifier: [u8; 4] = [b'o', b'r', b'b', b'i'];
}

impl pallet_moonbeam_orbiters::Config for Runtime {
	type Event = Event;
	type AccountLookup = AuthorMapping;
	type AddCollatorOrigin = EnsureRoot<AccountId>;
	type Currency = Balances;
	type DelCollatorOrigin = EnsureRoot<AccountId>;
	/// Maximum number of orbiters per collator
	type MaxPoolSize = ConstU32<8>;
	/// Maximum number of round to keep on storage
	type MaxRoundArchive = ConstU32<4>;
	type OrbiterReserveIdentifier = OrbiterReserveIdentifier;
	type RotatePeriod = ConstU32<1>;
	/// Round index type.
	type RoundIndex = pallet_parachain_staking::RoundIndex;
	type WeightInfo = pallet_moonbeam_orbiters::weights::SubstrateWeight<Runtime>;
}

construct_runtime! {
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		// System support stuff.
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 0,
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>} = 1,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 2,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 3,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 4,

		// Monetary stuff.
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 11,

		// Consensus support.
		ParachainStaking: pallet_parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>} = 20,
		AuthorInherent: pallet_author_inherent::{Pallet, Call, Storage, Inherent} = 21,
		AuthorFilter: pallet_author_slot_filter::{Pallet, Call, Storage, Event, Config} = 22,
		AuthorMapping: pallet_author_mapping::{Pallet, Call, Config<T>, Storage, Event<T>} = 23,
		MoonbeamOrbiters: pallet_moonbeam_orbiters::{Pallet, Call, Storage, Event<T>} = 24,

		// Handy utilities.
		Utility: pallet_utility::{Pallet, Call, Event} = 30,
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 31,
		MaintenanceMode: pallet_maintenance_mode::{Pallet, Call, Config, Storage, Event} = 32,
		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 33,
		Migrations: pallet_migrations::{Pallet, Storage, Config, Event<T>} = 34,
		ProxyGenesisCompanion: pallet_proxy_genesis_companion::{Pallet, Config<T>} = 35,

		// Has been permanently removed for safety reasons.
		// Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 40,

		// Ethereum compatibility.
		EthereumChainId: pallet_ethereum_chain_id::{Pallet, Storage, Config} = 50,
		EVM: pallet_evm::{Pallet, Config, Call, Storage, Event<T>} = 51,
		Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Origin, Config} = 52,
		BaseFee: pallet_base_fee::{Pallet, Call, Storage, Config<T>, Event} = 53,

		// Governance stuff.
		Scheduler: pallet_scheduler::{Pallet, Storage, Event<T>, Call} = 60,
		Democracy: pallet_democracy::{Pallet, Storage, Config<T>, Event<T>, Call} = 61,

		// Council stuff.
		CouncilCollective:
			pallet_collective::<Instance1>::{Pallet, Call, Storage, Event<T>, Origin<T>, Config<T>} = 70,
		TechCommitteeCollective:
			pallet_collective::<Instance2>::{Pallet, Call, Storage, Event<T>, Origin<T>, Config<T>} = 71,

		// Treasury stuff.
		Treasury: pallet_treasury::{Pallet, Storage, Config, Event<T>, Call} = 80,

		// Crowdloan stuff.
		CrowdloanRewards: pallet_crowdloan_rewards::{Pallet, Call, Config<T>, Storage, Event<T>} = 90,

		// XCM
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Storage, Event<T>} = 100,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 101,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 102,
		PolkadotXcm: pallet_xcm::{Pallet, Storage, Call, Event<T>, Origin, Config} = 103,
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>} = 104,
		AssetManager: pallet_asset_manager::{Pallet, Call, Storage, Event<T>} = 105,
		XTokens: orml_xtokens::{Pallet, Call, Storage, Event<T>} = 106,
		XcmTransactor: pallet_xcm_transactor::{Pallet, Call, Storage, Event<T>} = 107,
		LocalAssets: pallet_assets::<Instance1>::{Pallet, Call, Storage, Event<T>} = 108,
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
moonbeam_runtime_common::impl_runtime_apis_plus_common! {
	impl session_keys_primitives::VrfApi<Block> for Runtime {
		fn get_last_vrf_output() -> Option<<Block as BlockT>::Hash> {
			None
		}
		fn vrf_key_lookup(
			nimbus_id: nimbus_primitives::NimbusId
		) -> Option<session_keys_primitives::VrfId> {
			use session_keys_primitives::KeysLookup;
			AuthorMapping::lookup_keys(&nimbus_id)
		}
	}

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

moonbeam_runtime_common::impl_self_contained_call!();

// Shorthand for a Get field of a pallet Config.
#[macro_export]
macro_rules! get {
	($pallet:ident, $name:ident, $type:ty) => {
		<<$crate::Runtime as $pallet::Config>::$name as $crate::Get<$type>>::get()
	};
}

#[cfg(test)]
mod tests {
	use super::{currency::*, *};

	#[test]
	// Helps us to identify a Pallet Call in case it exceeds the 1kb limit.
	// Hint: this should be a rare case. If that happens, one or more of the dispatchable arguments
	// need to be Boxed.
	fn call_max_size() {
		const CALL_ALIGN: u32 = 1024;
		assert!(
			std::mem::size_of::<pallet_ethereum_chain_id::Call<Runtime>>() <= CALL_ALIGN as usize
		);
		assert!(std::mem::size_of::<pallet_evm::Call<Runtime>>() <= CALL_ALIGN as usize);
		assert!(std::mem::size_of::<pallet_ethereum::Call<Runtime>>() <= CALL_ALIGN as usize);
		assert!(
			std::mem::size_of::<pallet_parachain_staking::Call<Runtime>>() <= CALL_ALIGN as usize
		);
		assert!(
			std::mem::size_of::<pallet_author_inherent::Call<Runtime>>() <= CALL_ALIGN as usize
		);
		assert!(
			std::mem::size_of::<pallet_author_slot_filter::Call<Runtime>>() <= CALL_ALIGN as usize
		);
		assert!(
			std::mem::size_of::<pallet_crowdloan_rewards::Call<Runtime>>() <= CALL_ALIGN as usize
		);
		assert!(std::mem::size_of::<pallet_author_mapping::Call<Runtime>>() <= CALL_ALIGN as usize);
		assert!(
			std::mem::size_of::<pallet_maintenance_mode::Call<Runtime>>() <= CALL_ALIGN as usize
		);
		assert!(std::mem::size_of::<pallet_migrations::Call<Runtime>>() <= CALL_ALIGN as usize);
		assert!(
			std::mem::size_of::<pallet_proxy_genesis_companion::Call<Runtime>>()
				<= CALL_ALIGN as usize
		);
	}

	#[test]
	fn currency_constants_are_correct() {
		assert_eq!(SUPPLY_FACTOR, 100);

		// txn fees
		assert_eq!(TRANSACTION_BYTE_FEE, Balance::from(100 * GIGAWEI));
		assert_eq!(
			get!(pallet_transaction_payment, OperationalFeeMultiplier, u8),
			5_u8
		);
		assert_eq!(STORAGE_BYTE_FEE, Balance::from(10 * MILLIGLMR));
		assert_eq!(FixedGasPrice::min_gas_price().0, (100 * GIGAWEI).into());

		// democracy minimums
		assert_eq!(
			get!(pallet_democracy, MinimumDeposit, u128),
			Balance::from(400 * GLMR)
		);
		assert_eq!(
			get!(pallet_democracy, PreimageByteDeposit, u128),
			Balance::from(10 * MILLIGLMR)
		);
		assert_eq!(
			get!(pallet_treasury, ProposalBondMinimum, u128),
			Balance::from(100 * GLMR)
		);

		// pallet_identity deposits
		assert_eq!(
			get!(pallet_identity, BasicDeposit, u128),
			Balance::from(10 * GLMR + 2580 * MILLIGLMR)
		);
		assert_eq!(
			get!(pallet_identity, FieldDeposit, u128),
			Balance::from(660 * MILLIGLMR)
		);
		assert_eq!(
			get!(pallet_identity, SubAccountDeposit, u128),
			Balance::from(10 * GLMR + 530 * MILLIGLMR)
		);

		// staking minimums
		assert_eq!(
			get!(pallet_parachain_staking, MinCollatorStk, u128),
			Balance::from(100 * KILOGLMR)
		);
		assert_eq!(
			get!(pallet_parachain_staking, MinCandidateStk, u128),
			Balance::from(100 * KILOGLMR)
		);
		assert_eq!(
			get!(pallet_parachain_staking, MinDelegation, u128),
			Balance::from(50 * GLMR)
		);
		assert_eq!(
			get!(pallet_parachain_staking, MinDelegatorStk, u128),
			Balance::from(50 * GLMR)
		);

		// crowdloan min reward
		assert_eq!(
			get!(pallet_crowdloan_rewards, MinimumReward, u128),
			Balance::from(0u128)
		);

		// deposit for AuthorMapping
		assert_eq!(
			get!(pallet_author_mapping, DepositAmount, u128),
			Balance::from(10 * KILOGLMR)
		);

		// proxy deposits
		assert_eq!(
			get!(pallet_proxy, ProxyDepositBase, u128),
			Balance::from(10 * GLMR + 80 * MILLIGLMR)
		);
		assert_eq!(
			get!(pallet_proxy, ProxyDepositFactor, u128),
			Balance::from(210 * MILLIGLMR)
		);
		assert_eq!(
			get!(pallet_proxy, AnnouncementDepositBase, u128),
			Balance::from(10 * GLMR + 80 * MILLIGLMR)
		);
		assert_eq!(
			get!(pallet_proxy, AnnouncementDepositFactor, u128),
			Balance::from(560 * MILLIGLMR)
		);
	}

	#[test]
	// Required migration is
	// pallet_parachain_staking::migrations::IncreaseMaxTopDelegationsPerCandidate
	// Purpose of this test is to remind of required migration if constant is ever changed
	fn updating_maximum_delegators_per_candidate_requires_configuring_required_migration() {
		assert_eq!(
			get!(pallet_parachain_staking, MaxTopDelegationsPerCandidate, u32),
			300
		);
		assert_eq!(
			get!(
				pallet_parachain_staking,
				MaxBottomDelegationsPerCandidate,
				u32
			),
			50
		);
	}
}
