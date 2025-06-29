// Copyright 2019-2025 PureStake Inc.
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
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 512.
#![recursion_limit = "512"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

extern crate alloc;

use account::AccountId20;
use alloc::borrow::Cow;
use cumulus_pallet_parachain_system::{
	RelayChainStateProof, RelayStateProof, RelaychainDataProvider, ValidationData,
};
use fp_rpc::TransactionStatus;

use cumulus_primitives_core::{relay_chain, AggregateMessageOrigin};
#[cfg(feature = "std")]
pub use fp_evm::GenesisAccount;
pub use frame_support::traits::Get;
use frame_support::{
	construct_runtime,
	dispatch::{DispatchClass, GetDispatchInfo, PostDispatchInfo},
	ensure,
	pallet_prelude::DispatchResult,
	parameter_types,
	traits::{
		fungible::{Balanced, Credit, HoldConsideration, Inspect, NativeOrWithId},
		ConstBool, ConstU128, ConstU16, ConstU32, ConstU64, ConstU8, Contains, EitherOf,
		EitherOfDiverse, EqualPrivilegeOnly, InstanceFilter, LinearStoragePrice, OnFinalize,
		OnUnbalanced,
	},
	weights::{
		constants::WEIGHT_REF_TIME_PER_SECOND, ConstantMultiplier, Weight, WeightToFeeCoefficient,
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
use moonbeam_runtime_common::{
	impl_asset_conversion::AssetRateConverter,
	impl_multiasset_paymaster::MultiAssetPaymaster,
	timestamp::{ConsensusHookWrapperForRelayTimestamp, RelayTimestamp},
};
pub use pallet_author_slot_filter::EligibilityValue;
use pallet_ethereum::Call::transact;
use pallet_ethereum::{PostLogContent, Transaction as EthereumTransaction};
use pallet_evm::{
	Account as EVMAccount, EVMFungibleAdapter, EnsureAddressNever, EnsureAddressRoot,
	FeeCalculator, FrameSystemAccountProvider, GasWeightMapping, IdentityAddressMapping,
	OnChargeEVMTransaction as OnChargeEVMTransactionT, Runner,
};
pub use pallet_parachain_staking::{weights::WeightInfo, InflationInfo, Range};
use pallet_transaction_payment::{FungibleAdapter, Multiplier, TargetedFeeAdjustment};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use smallvec::smallvec;
use sp_api::impl_runtime_apis;
use sp_consensus_slots::Slot;
use sp_core::{OpaqueMetadata, H160, H256, U256};
use sp_runtime::generic::Preamble;
use sp_runtime::{
	generic, impl_opaque_keys,
	traits::{
		BlakeTwo256, Block as BlockT, DispatchInfoOf, Dispatchable, IdentityLookup,
		PostDispatchInfoOf, UniqueSaturatedInto, Zero,
	},
	transaction_validity::{
		InvalidTransaction, TransactionSource, TransactionValidity, TransactionValidityError,
	},
	ApplyExtrinsicResult, DispatchErrorWithPostInfo, FixedPointNumber, Perbill, Permill,
	Perquintill, SaturatedConversion,
};
use sp_std::{convert::TryFrom, prelude::*};
use xcm::{
	Version as XcmVersion, VersionedAssetId, VersionedAssets, VersionedLocation, VersionedXcm,
};
use xcm_runtime_apis::{
	dry_run::{CallDryRunEffects, Error as XcmDryRunApiError, XcmDryRunEffects},
	fees::Error as XcmPaymentApiError,
};

use runtime_params::*;

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use nimbus_primitives::CanAuthor;

mod migrations;
mod precompiles;
pub use precompiles::{
	MoonbeamPrecompiles, PrecompileName, FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

pub type Precompiles = MoonbeamPrecompiles<Runtime>;

pub mod asset_config;
#[cfg(not(feature = "disable-genesis-builder"))]
pub mod genesis_config_preset;
pub mod governance;
pub mod runtime_params;
mod weights;
pub mod xcm_config;

use governance::councils::*;
pub(crate) use weights as moonbeam_weights;
pub use weights::xcm as moonbeam_xcm_weights;

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
	pub const WEIGHT_FEE: Balance = 50 * KILOWEI * SUPPLY_FACTOR / 4;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 100 * MILLIGLMR * SUPPLY_FACTOR + (bytes as Balance) * STORAGE_BYTE_FEE
	}
}

/// Maximum PoV size we support right now.
// Reference: https://github.com/polkadot-fellows/runtimes/pull/553
pub const MAX_POV_SIZE: u32 = 10 * 1024 * 1024;

/// Maximum weight per block
pub const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
	WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2),
	MAX_POV_SIZE as u64,
);

pub const MILLISECS_PER_BLOCK: u64 = 6_000;
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
	spec_name: Cow::Borrowed("moonbeam"),
	impl_name: Cow::Borrowed("moonbeam"),
	authoring_version: 3,
	spec_version: 3800,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 3,
	system_version: 1,
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
pub const NORMAL_WEIGHT: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_mul(3).saturating_div(4);
// Here we assume Ethereum's base fee of 21000 gas and convert to weight, but we
// subtract roughly the cost of a balance transfer from it (about 1/3 the cost)
// and some cost to account for per-byte-fee.
// TODO: we should use benchmarking's overhead feature to measure this
pub const EXTRINSIC_BASE_WEIGHT: Weight = Weight::from_parts(10000 * WEIGHT_PER_GAS, 0);

pub struct RuntimeBlockWeights;
impl Get<frame_system::limits::BlockWeights> for RuntimeBlockWeights {
	fn get() -> frame_system::limits::BlockWeights {
		frame_system::limits::BlockWeights::builder()
			.for_class(DispatchClass::Normal, |weights| {
				weights.base_extrinsic = EXTRINSIC_BASE_WEIGHT;
				weights.max_total = NORMAL_WEIGHT.into();
			})
			.for_class(DispatchClass::Operational, |weights| {
				weights.max_total = MAXIMUM_BLOCK_WEIGHT.into();
				weights.reserved = (MAXIMUM_BLOCK_WEIGHT - NORMAL_WEIGHT).into();
			})
			.avg_block_initialization(Perbill::from_percent(10))
			.build()
			.expect("Provided BlockWeight definitions are valid, qed")
	}
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	/// We allow for 5 MB blocks.
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
}

impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = IdentityLookup<AccountId>;
	/// The index type for storing how many extrinsics an account has signed.
	type Nonce = Index;
	/// The index type for blocks.
	type Block = Block;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// The aggregated RuntimeTask type.
	type RuntimeTask = RuntimeTask;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = ConstU32<256>;
	/// Maximum weight of each block. With a default weight system of 1byte == 1weight, 4mb is ok.
	type BlockWeights = RuntimeBlockWeights;
	/// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
	type BlockLength = BlockLength;
	/// Runtime version.
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = moonbeam_weights::db::rocksdb::constants::RocksDbWeight;
	type BaseCallFilter = MaintenanceMode;
	type SystemWeightInfo = moonbeam_weights::frame_system::WeightInfo<Runtime>;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = ConstU16<1284>;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = MultiBlockMigrations;
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
	type ExtensionsWeightInfo = moonbeam_weights::frame_system_extensions::WeightInfo<Runtime>;
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = moonbeam_weights::pallet_utility::WeightInfo<Runtime>;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = ConstU64<3000>;
	type WeightInfo = moonbeam_weights::pallet_timestamp::WeightInfo<Runtime>;
}

#[cfg(not(feature = "runtime-benchmarks"))]
parameter_types! {
	pub const ExistentialDeposit: Balance = 0;
}

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub const ExistentialDeposit: Balance = 1;
}

impl pallet_balances::Config for Runtime {
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 4];
	type MaxLocks = ConstU32<50>;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type FreezeIdentifier = ();
	type MaxFreezes = ConstU32<0>;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type WeightInfo = moonbeam_weights::pallet_balances::WeightInfo<Runtime>;
	type DoneSlashHandler = ();
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
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = FungibleAdapter<
		Balances,
		DealWithSubstrateFeesAndTip<
			Runtime,
			dynamic_params::runtime_config::FeesTreasuryProportion,
		>,
	>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = ConstantMultiplier<Balance, ConstU128<{ currency::WEIGHT_FEE }>>;
	type LengthToFee = LengthToFee;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Runtime>;
	type WeightInfo = weights::pallet_transaction_payment::WeightInfo<Runtime>;
}

impl pallet_evm_chain_id::Config for Runtime {}

/// Current approximation of the gas/s consumption considering
/// EVM execution over compiled WASM (on 4.4Ghz CPU).
/// Given the 2000ms Weight, from which 75% only are used for transactions,
/// the total EVM execution gas limit is: GAS_PER_SECOND * 2 * 0.75 ~= 60_000_000.
pub const GAS_PER_SECOND: u64 = 40_000_000;

/// Approximate ratio of the amount of Weight per Gas.
/// u64 works for approximations because Weight is a very small unit compared to gas.
pub const WEIGHT_PER_GAS: u64 = WEIGHT_REF_TIME_PER_SECOND / GAS_PER_SECOND;

/// The highest amount of new storage that can be created in a block (160KB).
/// Originally 40KB, then multiplied by 4 when the block deadline was increased from 500ms to 2000ms.
/// Reference: https://github.com/moonbeam-foundation/moonbeam/blob/master/MBIPS/MBIP-5.md#specification
pub const BLOCK_STORAGE_LIMIT: u64 = 160 * 1024;

parameter_types! {
	pub BlockGasLimit: U256
		= U256::from(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT.ref_time() / WEIGHT_PER_GAS);
	/// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
	/// than this will decrease the weight and more will increase.
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(35);
	/// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
	/// change the fees more rapidly. This low value causes changes to occur slowly over time.
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(4, 1_000);
	/// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
	/// that combined with `AdjustmentVariable`, we can recover from the minimum.
	/// See `multiplier_can_grow_from_zero` in integration_tests.rs.
	/// This value is currently only used by pallet-transaction-payment as an assertion that the
	/// next multiplier is always > min value.
	pub MinimumMultiplier: Multiplier = Multiplier::from(1u128);
	/// Maximum multiplier. We pick a value that is expensive but not impossibly so; it should act
	/// as a safety net.
	pub MaximumMultiplier: Multiplier = Multiplier::from(100_000u128);
	pub PrecompilesValue: MoonbeamPrecompiles<Runtime> = MoonbeamPrecompiles::<_>::new();
	pub WeightPerGas: Weight = Weight::from_parts(WEIGHT_PER_GAS, 0);
	/// The amount of gas per pov. A ratio of 8 if we convert ref_time to gas and we compare
	/// it with the pov_size for a block. E.g.
	/// ceil(
	///     (max_extrinsic.ref_time() / max_extrinsic.proof_size()) / WEIGHT_PER_GAS
	/// )
	/// We should re-check `xcm_config::Erc20XcmBridgeTransferGasLimit` when changing this value
	pub const GasLimitPovSizeRatio: u64 = 8;
	/// The amount of gas per storage (in bytes): BLOCK_GAS_LIMIT / BLOCK_STORAGE_LIMIT
	/// The current definition of BLOCK_STORAGE_LIMIT is 160 KB, resulting in a value of 366.
	pub GasLimitStorageGrowthRatio: u64 = 366;
}

pub struct TransactionPaymentAsGasPrice;
impl FeeCalculator for TransactionPaymentAsGasPrice {
	fn min_gas_price() -> (U256, Weight) {
		// note: transaction-payment differs from EIP-1559 in that its tip and length fees are not
		//       scaled by the multiplier, which means its multiplier will be overstated when
		//       applied to an ethereum transaction
		// note: transaction-payment uses both a congestion modifier (next_fee_multiplier, which is
		//       updated once per block in on_finalize) and a 'WeightToFee' implementation. Our
		//       runtime implements this as a 'ConstantModifier', so we can get away with a simple
		//       multiplication here.
		// It is imperative that `saturating_mul_int` be performed as late as possible in the
		// expression since it involves fixed point multiplication with a division by a fixed
		// divisor. This leads to truncation and subsequent precision loss if performed too early.
		// This can lead to min_gas_price being same across blocks even if the multiplier changes.
		// There's still some precision loss when the final `gas_price` (used_gas * min_gas_price)
		// is computed in frontier, but that's currently unavoidable.
		let min_gas_price = TransactionPayment::next_fee_multiplier()
			.saturating_mul_int((currency::WEIGHT_FEE).saturating_mul(WEIGHT_PER_GAS as u128));
		(
			min_gas_price.into(),
			<Runtime as frame_system::Config>::DbWeight::get().reads(1),
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
pub type SlowAdjustingFeeUpdate<R> = TargetedFeeAdjustment<
	R,
	TargetBlockFullness,
	AdjustmentVariable,
	MinimumMultiplier,
	MaximumMultiplier,
>;

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
	type FeeCalculator = TransactionPaymentAsGasPrice;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = IdentityAddressMapping;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = MoonbeamPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = EthereumChainId;
	type OnChargeTransaction = OnChargeEVMTransaction<
		DealWithEthereumBaseFees<Runtime, dynamic_params::runtime_config::FeesTreasuryProportion>,
		DealWithEthereumPriorityFees<Runtime>,
	>;
	type BlockGasLimit = BlockGasLimit;
	type FindAuthor = FindAuthorAdapter<AuthorInherent>;
	type OnCreate = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type GasLimitStorageGrowthRatio = GasLimitStorageGrowthRatio;
	type Timestamp = RelayTimestamp;
	type AccountProvider = FrameSystemAccountProvider<Runtime>;
	type WeightInfo = moonbeam_weights::pallet_evm::WeightInfo<Runtime>;
}

parameter_types! {
	pub MaxServiceWeight: Weight = NORMAL_DISPATCH_RATIO * RuntimeBlockWeights::get().max_block;
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaxServiceWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = ConstU32<50>;
	type WeightInfo = moonbeam_weights::pallet_scheduler::WeightInfo<Runtime>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = Preimage;
}

parameter_types! {
	pub const PreimageBaseDeposit: Balance = 5 * currency::GLMR * currency::SUPPLY_FACTOR ;
	pub const PreimageByteDeposit: Balance = currency::STORAGE_BYTE_FEE;
	pub const PreimageHoldReason: RuntimeHoldReason =
		RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = moonbeam_weights::pallet_preimage::WeightInfo<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type Consideration = HoldConsideration<
		AccountId,
		Balances,
		PreimageHoldReason,
		LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
	>;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const TreasuryId: PalletId = PalletId(*b"py/trsry");
	pub TreasuryAccount: AccountId = Treasury::account_id();
	pub const MaxSpendBalance: crate::Balance = crate::Balance::max_value();
}

type RootOrTreasuryCouncilOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, TreasuryCouncilInstance, 1, 2>,
>;

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryId;
	type Currency = Balances;
	// More than half of the council is required (or root) to reject a proposal
	type RejectOrigin = RootOrTreasuryCouncilOrigin;
	type RuntimeEvent = RuntimeEvent;
	type SpendPeriod = ConstU32<{ 6 * DAYS }>;
	type Burn = ();
	type BurnDestination = ();
	type MaxApprovals = ConstU32<100>;
	type WeightInfo = moonbeam_weights::pallet_treasury::WeightInfo<Runtime>;
	type SpendFunds = ();
	type SpendOrigin =
		frame_system::EnsureWithSuccess<RootOrTreasuryCouncilOrigin, AccountId, MaxSpendBalance>;
	type AssetKind = NativeOrWithId<AssetId>;
	type Beneficiary = AccountId;
	type BeneficiaryLookup = IdentityLookup<AccountId>;
	type Paymaster = MultiAssetPaymaster<Runtime, TreasuryAccount, Balances>;
	type BalanceConverter = AssetRateConverter<Runtime, Balances>;
	type PayoutPeriod = ConstU32<{ 30 * DAYS }>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = BenchmarkHelper;
	type BlockNumberProvider = System;
}

parameter_types! {
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
	pub const PendingUsernameExpiration: u32 = 7 * DAYS;
	pub const MaxSuffixLength: u32 = 7;
	pub const MaxUsernameLength: u32 = 32;
}

type IdentityForceOrigin =
	EitherOfDiverse<EnsureRoot<AccountId>, governance::custom_origins::GeneralAdmin>;
type IdentityRegistrarOrigin =
	EitherOfDiverse<EnsureRoot<AccountId>, governance::custom_origins::GeneralAdmin>;

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	// Add one item in storage and take 258 bytes
	type BasicDeposit = ConstU128<{ currency::deposit(1, 258) }>;
	// Does not add any item to the storage but takes 1 bytes
	type ByteDeposit = ConstU128<{ currency::deposit(0, 1) }>;
	// Add one item in storage and take 53 bytes
	type SubAccountDeposit = ConstU128<{ currency::deposit(1, 53) }>;
	type MaxSubAccounts = MaxSubAccounts;
	type IdentityInformation = pallet_identity::legacy::IdentityInfo<MaxAdditionalFields>;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = Treasury;
	type ForceOrigin = IdentityForceOrigin;
	type RegistrarOrigin = IdentityRegistrarOrigin;
	type OffchainSignature = Signature;
	type SigningPublicKey = <Signature as sp_runtime::traits::Verify>::Signer;
	type UsernameAuthorityOrigin = EnsureRoot<AccountId>;
	type PendingUsernameExpiration = PendingUsernameExpiration;
	type MaxSuffixLength = MaxSuffixLength;
	type MaxUsernameLength = MaxUsernameLength;
	type WeightInfo = moonbeam_weights::pallet_identity::WeightInfo<Runtime>;
	type UsernameDeposit = ConstU128<{ currency::deposit(0, MaxUsernameLength::get()) }>;
	type UsernameGracePeriod = ConstU32<{ 30 * DAYS }>;
}

pub struct TransactionConverter;

impl fp_rpc::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(&self, transaction: pallet_ethereum::Transaction) -> UncheckedExtrinsic {
		UncheckedExtrinsic::new_bare(
			pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		)
	}
}

impl fp_rpc::ConvertTransaction<opaque::UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(
		&self,
		transaction: pallet_ethereum::Transaction,
	) -> opaque::UncheckedExtrinsic {
		let extrinsic = UncheckedExtrinsic::new_bare(
			pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		);
		let encoded = extrinsic.encode();
		opaque::UncheckedExtrinsic::decode(&mut &encoded[..])
			.expect("Encoded extrinsic is always valid")
	}
}

parameter_types! {
	pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self::Version>;
	type PostLogContent = PostBlockAndTxnHashes;
	type ExtraDataLength = ConstU32<30>;
}

/// Maximum number of blocks simultaneously accepted by the Runtime, not yet included
/// into the relay chain.
const UNINCLUDED_SEGMENT_CAPACITY: u32 = 3;
/// How many parachain blocks are processed by the relay chain per parent. Limits the
/// number of blocks authored per slot.
const BLOCK_PROCESSING_VELOCITY: u32 = 1;

type ConsensusHook = pallet_async_backing::consensus_hook::FixedVelocityConsensusHook<
	Runtime,
	BLOCK_PROCESSING_VELOCITY,
	UNINCLUDED_SEGMENT_CAPACITY,
>;

impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = ParachainInfo;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = EmergencyParaXcm;
	type ConsensusHook = ConsensusHookWrapperForRelayTimestamp<Runtime, ConsensusHook>;
	type DmpQueue = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
	type WeightInfo = moonbeam_weights::cumulus_pallet_parachain_system::WeightInfo<Runtime>;
	type SelectCore = cumulus_pallet_parachain_system::DefaultCoreSelector<Runtime>;
}

pub struct EthereumXcmEnsureProxy;
impl xcm_primitives::EnsureProxy<AccountId> for EthereumXcmEnsureProxy {
	fn ensure_ok(delegator: AccountId, delegatee: AccountId) -> Result<(), &'static str> {
		// The EVM implicitely contains an Any proxy, so we only allow for "Any" proxies
		let def: pallet_proxy::ProxyDefinition<AccountId, ProxyType, BlockNumber> =
			pallet_proxy::Pallet::<Runtime>::find_proxy(
				&delegator,
				&delegatee,
				Some(ProxyType::Any),
			)
			.map_err(|_| "proxy error: expected `ProxyType::Any`")?;
		// We only allow to use it for delay zero proxies, as the call will immediatly be executed
		ensure!(def.delay.is_zero(), "proxy delay is Non-zero`");
		Ok(())
	}
}

impl pallet_ethereum_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type InvalidEvmTransactionError = pallet_ethereum::InvalidTransactionWrapper;
	type ValidatedTransaction = pallet_ethereum::ValidatedTransaction<Self>;
	type XcmEthereumOrigin = pallet_ethereum_xcm::EnsureXcmEthereumTransaction;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type EnsureProxy = EthereumXcmEnsureProxy;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ForceOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
	// Reserved weight is 1/4 of MAXIMUM_BLOCK_WEIGHT
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
}

impl parachain_info::Config for Runtime {}

pub struct OnNewRound;
impl pallet_parachain_staking::OnNewRound for OnNewRound {
	fn on_new_round(round_index: pallet_parachain_staking::RoundIndex) -> Weight {
		MoonbeamOrbiters::on_new_round(round_index)
	}
}
pub struct PayoutCollatorOrOrbiterReward;
impl pallet_parachain_staking::PayoutCollatorReward<Runtime> for PayoutCollatorOrOrbiterReward {
	fn payout_collator_reward(
		for_round: pallet_parachain_staking::RoundIndex,
		collator_id: AccountId,
		amount: Balance,
	) -> Weight {
		let extra_weight =
			if MoonbeamOrbiters::is_collator_pool_with_active_orbiter(for_round, collator_id) {
				MoonbeamOrbiters::distribute_rewards(for_round, collator_id, amount)
			} else {
				ParachainStaking::mint_collator_reward(for_round, collator_id, amount)
			};

		<Runtime as frame_system::Config>::DbWeight::get()
			.reads(1)
			.saturating_add(extra_weight)
	}
}

pub struct OnInactiveCollator;
impl pallet_parachain_staking::OnInactiveCollator<Runtime> for OnInactiveCollator {
	fn on_inactive_collator(
		collator_id: AccountId,
		round: pallet_parachain_staking::RoundIndex,
	) -> Result<Weight, DispatchErrorWithPostInfo<PostDispatchInfo>> {
		let extra_weight = if !MoonbeamOrbiters::is_collator_pool_with_active_orbiter(
			round,
			collator_id.clone(),
		) {
			ParachainStaking::go_offline_inner(collator_id)?;
			<Runtime as pallet_parachain_staking::Config>::WeightInfo::go_offline(
				pallet_parachain_staking::MAX_CANDIDATES,
			)
		} else {
			Weight::zero()
		};

		Ok(<Runtime as frame_system::Config>::DbWeight::get()
			.reads(1)
			.saturating_add(extra_weight))
	}
}
type MonetaryGovernanceOrigin =
	EitherOfDiverse<EnsureRoot<AccountId>, governance::custom_origins::GeneralAdmin>;

pub struct RelayChainSlotProvider;
impl Get<Slot> for RelayChainSlotProvider {
	fn get() -> Slot {
		let slot_info = pallet_async_backing::pallet::Pallet::<Runtime>::slot_info();
		slot_info.unwrap_or_default().0
	}
}

parameter_types! {
	// Voted by the moonbeam community on this referenda: https://moonbeam.polkassembly.network/referenda/116
	pub const LinearInflationThreshold: Option<Balance> = Some(1_200_000_000 * currency::GLMR);
}

impl pallet_parachain_staking::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MonetaryGovernanceOrigin = MonetaryGovernanceOrigin;
	/// Minimum round length is 2 minutes (10 * 12 second block times)
	type MinBlocksPerRound = ConstU32<10>;
	/// If a collator doesn't produce any block on this number of rounds, it is notified as inactive
	type MaxOfflineRounds = ConstU32<1>;
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
	/// Minimum stake required to be reserved to be a candidate
	type MinCandidateStk = ConstU128<{ 5_000 * currency::GLMR * currency::SUPPLY_FACTOR }>;
	/// Minimum stake required to be reserved to be a delegator
	type MinDelegation = ConstU128<{ 500 * currency::MILLIGLMR * currency::SUPPLY_FACTOR }>;
	type BlockAuthor = AuthorInherent;
	type OnCollatorPayout = ();
	type PayoutCollatorReward = PayoutCollatorOrOrbiterReward;
	type OnInactiveCollator = OnInactiveCollator;
	type OnNewRound = OnNewRound;
	type SlotProvider = RelayChainSlotProvider;
	type WeightInfo = moonbeam_weights::pallet_parachain_staking::WeightInfo<Runtime>;
	type MaxCandidates = ConstU32<200>;
	type SlotDuration = ConstU64<6_000>;
	type BlockTime = ConstU64<6_000>;
	type LinearInflationThreshold = LinearInflationThreshold;
}

impl pallet_author_inherent::Config for Runtime {
	type SlotBeacon = RelaychainDataProvider<Self>;
	type AccountLookup = MoonbeamOrbiters;
	type CanAuthor = AuthorFilter;
	type AuthorId = AccountId;
	type WeightInfo = moonbeam_weights::pallet_author_inherent::WeightInfo<Runtime>;
}

impl pallet_author_slot_filter::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RandomnessSource = Randomness;
	type PotentialAuthors = ParachainStaking;
	type WeightInfo = moonbeam_weights::pallet_author_slot_filter::WeightInfo<Runtime>;
}

impl pallet_async_backing::Config for Runtime {
	type AllowMultipleBlocksPerSlot = ConstBool<true>;
	type GetAndVerifySlot = pallet_async_backing::RelaySlot;
	type ExpectedBlockTime = ConstU64<6000>;
}

parameter_types! {
	pub const InitializationPayment: Perbill = Perbill::from_percent(30);
	pub const RelaySignaturesThreshold: Perbill = Perbill::from_percent(100);
	pub const SignatureNetworkIdentifier:  &'static [u8] = b"moonbeam-";
}

impl pallet_crowdloan_rewards::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
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
	type VestingBlockNumber = relay_chain::BlockNumber;
	type VestingBlockProvider = RelaychainDataProvider<Self>;
	type WeightInfo = moonbeam_weights::pallet_crowdloan_rewards::WeightInfo<Runtime>;
}

// This is a simple session key manager. It should probably either work with, or be replaced
// entirely by pallet sessions
impl pallet_author_mapping::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type DepositCurrency = Balances;
	type DepositAmount = ConstU128<{ 100 * currency::GLMR * currency::SUPPLY_FACTOR }>;
	type Keys = session_keys_primitives::VrfId;
	type WeightInfo = moonbeam_weights::pallet_author_mapping::WeightInfo<Runtime>;
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
	MaxEncodedLen,
	TypeInfo,
	Serialize,
	Deserialize,
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

fn is_governance_precompile(precompile_name: &precompiles::PrecompileName) -> bool {
	matches!(
		precompile_name,
		PrecompileName::ConvictionVotingPrecompile
			| PrecompileName::PreimagePrecompile
			| PrecompileName::ReferendaPrecompile
			| PrecompileName::OpenTechCommitteeInstance
			| PrecompileName::TreasuryCouncilInstance
	)
}

// Be careful: Each time this filter is modified, the substrate filter must also be modified
// consistently.
impl pallet_evm_precompile_proxy::EvmProxyCallFilter for ProxyType {
	fn is_evm_proxy_call_allowed(
		&self,
		call: &pallet_evm_precompile_proxy::EvmSubCall,
		recipient_has_code: bool,
		gas: u64,
	) -> precompile_utils::EvmResult<bool> {
		Ok(match self {
			ProxyType::Any => {
				match PrecompileName::from_address(call.to.0) {
					// Any precompile that can execute a subcall should be forbidden here,
					// to ensure that unauthorized smart contract can't be called
					// indirectly.
					// To be safe, we only allow the precompiles we need.
					Some(
						PrecompileName::AuthorMappingPrecompile
						| PrecompileName::ParachainStakingPrecompile,
					) => true,
					Some(ref precompile) if is_governance_precompile(precompile) => true,
					// All non-whitelisted precompiles are forbidden
					Some(_) => false,
					// Allow evm transfer to "simple" account (no code nor precompile)
					// For the moment, no smart contract other than precompiles is allowed.
					// In the future, we may create a dynamic whitelist to authorize some audited
					// smart contracts through governance.
					None => {
						// If the address is not recognized, allow only evm transfer to "simple"
						// accounts (no code nor precompile).
						// Note: Checking the presence of the code is not enough because some
						// precompiles have no code.
						!recipient_has_code
							&& !precompile_utils::precompile_set::is_precompile_or_fail::<Runtime>(
								call.to.0, gas,
							)?
					}
				}
			}
			ProxyType::NonTransfer => {
				call.value == U256::zero()
					&& match PrecompileName::from_address(call.to.0) {
						Some(
							PrecompileName::AuthorMappingPrecompile
							| PrecompileName::ParachainStakingPrecompile,
						) => true,
						Some(ref precompile) if is_governance_precompile(precompile) => true,
						_ => false,
					}
			}
			ProxyType::Governance => {
				call.value == U256::zero()
					&& matches!(
						PrecompileName::from_address(call.to.0),
						Some(ref precompile) if is_governance_precompile(precompile)
					)
			}
			ProxyType::Staking => {
				call.value == U256::zero()
					&& matches!(
						PrecompileName::from_address(call.to.0),
						Some(
							PrecompileName::AuthorMappingPrecompile
								| PrecompileName::ParachainStakingPrecompile
						)
					)
			}
			// The proxy precompile does not contain method cancel_proxy
			ProxyType::CancelProxy => false,
			ProxyType::Balances => {
				// Allow only "simple" accounts as recipient (no code nor precompile).
				// Note: Checking the presence of the code is not enough because some precompiles
				// have no code.
				!recipient_has_code
					&& !precompile_utils::precompile_set::is_precompile_or_fail::<Runtime>(
						call.to.0, gas,
					)?
			}
			ProxyType::AuthorMapping => {
				call.value == U256::zero()
					&& matches!(
						PrecompileName::from_address(call.to.0),
						Some(PrecompileName::AuthorMappingPrecompile)
					)
			}
			// There is no identity precompile
			ProxyType::IdentityJudgement => false,
		})
	}
}

// Be careful: Each time this filter is modified, the EVM filter must also be modified consistently.
impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => match c {
				RuntimeCall::Identity(
					pallet_identity::Call::add_sub { .. } | pallet_identity::Call::set_subs { .. },
				) => false,
				call => {
					matches!(
						call,
						RuntimeCall::System(..)
							| RuntimeCall::ParachainSystem(..)
							| RuntimeCall::Timestamp(..)
							| RuntimeCall::ParachainStaking(..)
							| RuntimeCall::Referenda(..)
							| RuntimeCall::Preimage(..)
							| RuntimeCall::ConvictionVoting(..)
							| RuntimeCall::TreasuryCouncilCollective(..)
							| RuntimeCall::OpenTechCommitteeCollective(..)
							| RuntimeCall::Utility(..)
							| RuntimeCall::Proxy(..)
							| RuntimeCall::Identity(..)
							| RuntimeCall::AuthorMapping(..)
							| RuntimeCall::CrowdloanRewards(
								pallet_crowdloan_rewards::Call::claim { .. }
							)
					)
				}
			},
			ProxyType::Governance => matches!(
				c,
				RuntimeCall::Referenda(..)
					| RuntimeCall::Preimage(..)
					| RuntimeCall::ConvictionVoting(..)
					| RuntimeCall::TreasuryCouncilCollective(..)
					| RuntimeCall::OpenTechCommitteeCollective(..)
					| RuntimeCall::Utility(..)
			),
			ProxyType::Staking => matches!(
				c,
				RuntimeCall::ParachainStaking(..)
					| RuntimeCall::Utility(..)
					| RuntimeCall::AuthorMapping(..)
					| RuntimeCall::MoonbeamOrbiters(..)
			),
			ProxyType::CancelProxy => matches!(
				c,
				RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. })
			),
			ProxyType::Balances => {
				matches!(c, RuntimeCall::Balances(..) | RuntimeCall::Utility(..))
			}
			ProxyType::AuthorMapping => matches!(c, RuntimeCall::AuthorMapping(..)),
			ProxyType::IdentityJudgement => matches!(
				c,
				RuntimeCall::Identity(pallet_identity::Call::provide_judgement { .. })
					| RuntimeCall::Utility(..)
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
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	// One storage item; key size 32, value size 8
	type ProxyDepositBase = ConstU128<{ currency::deposit(1, 8) }>;
	// Additional storage item size of 21 bytes (20 bytes AccountId + 1 byte sizeof(ProxyType)).
	type ProxyDepositFactor = ConstU128<{ currency::deposit(0, 21) }>;
	type MaxProxies = ConstU32<32>;
	type WeightInfo = moonbeam_weights::pallet_proxy::WeightInfo<Runtime>;
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
	type RuntimeEvent = RuntimeEvent;
	type MigrationsList = (
		moonbeam_runtime_common::migrations::CommonMigrations<Runtime>,
		migrations::MoonbeamMigrations,
	);
	type XcmExecutionManager = XcmExecutionManager;
}

pub type ForeignAssetMigratorOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EitherOfDiverse<
		pallet_collective::EnsureProportionMoreThan<AccountId, OpenTechCommitteeInstance, 5, 9>,
		EitherOf<
			governance::custom_origins::GeneralAdmin,
			governance::custom_origins::FastGeneralAdmin,
		>,
	>,
>;

impl pallet_moonbeam_lazy_migrations::Config for Runtime {
	type ForeignAssetMigratorOrigin = ForeignAssetMigratorOrigin;
	type WeightInfo = moonbeam_weights::pallet_moonbeam_lazy_migrations::WeightInfo<Runtime>;
}

/// Maintenance mode Call filter
pub struct MaintenanceFilter;
impl Contains<RuntimeCall> for MaintenanceFilter {
	fn contains(c: &RuntimeCall) -> bool {
		match c {
			RuntimeCall::Assets(_) => false,
			RuntimeCall::Balances(_) => false,
			RuntimeCall::CrowdloanRewards(_) => false,
			RuntimeCall::Ethereum(_) => false,
			RuntimeCall::EVM(_) => false,
			RuntimeCall::Identity(_) => false,
			RuntimeCall::ParachainStaking(_) => false,
			RuntimeCall::MoonbeamOrbiters(_) => false,
			RuntimeCall::PolkadotXcm(_) => false,
			RuntimeCall::Treasury(_) => false,
			RuntimeCall::XcmTransactor(_) => false,
			RuntimeCall::EthereumXcm(_) => false,
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

impl Contains<RuntimeCall> for NormalFilter {
	fn contains(c: &RuntimeCall) -> bool {
		match c {
			RuntimeCall::Assets(method) => match method {
				pallet_assets::Call::transfer { .. } => true,
				pallet_assets::Call::transfer_keep_alive { .. } => true,
				pallet_assets::Call::approve_transfer { .. } => true,
				pallet_assets::Call::transfer_approved { .. } => true,
				pallet_assets::Call::cancel_approval { .. } => true,
				pallet_assets::Call::destroy_accounts { .. } => true,
				pallet_assets::Call::destroy_approvals { .. } => true,
				pallet_assets::Call::finish_destroy { .. } => true,
				_ => false,
			},
			// We just want to enable this in case of live chains, since the default version
			// is populated at genesis
			RuntimeCall::PolkadotXcm(method) => match method {
				pallet_xcm::Call::force_default_xcm_version { .. } => true,
				pallet_xcm::Call::transfer_assets { .. } => true,
				pallet_xcm::Call::transfer_assets_using_type_and_then { .. } => true,
				_ => false,
			},
			// We filter anonymous proxy as they make "reserve" inconsistent
			// See: https://github.com/paritytech/substrate/blob/37cca710eed3dadd4ed5364c7686608f5175cce1/frame/proxy/src/lib.rs#L270 // editorconfig-checker-disable-line
			RuntimeCall::Proxy(method) => match method {
				pallet_proxy::Call::create_pure { .. } => false,
				pallet_proxy::Call::kill_pure { .. } => false,
				pallet_proxy::Call::proxy { real, .. } => {
					!pallet_evm::AccountCodes::<Runtime>::contains_key(H160::from(*real))
				}
				_ => true,
			},
			// Filtering the EVM prevents possible re-entrancy from the precompiles which could
			// lead to unexpected scenarios.
			// See https://github.com/PureStake/sr-moonbeam/issues/30
			// Note: It is also assumed that EVM calls are only allowed through `Origin::Root` so
			// this can be seen as an additional security
			RuntimeCall::EVM(_) => false,
			_ => true,
		}
	}
}

pub struct XcmExecutionManager;
impl moonkit_xcm_primitives::PauseXcmExecution for XcmExecutionManager {
	fn suspend_xcm_execution() -> DispatchResult {
		XcmpQueue::suspend_xcm_execution(RuntimeOrigin::root())
	}
	fn resume_xcm_execution() -> DispatchResult {
		XcmpQueue::resume_xcm_execution(RuntimeOrigin::root())
	}
}

impl pallet_maintenance_mode::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type NormalCallFilter = NormalFilter;
	type MaintenanceCallFilter = MaintenanceFilter;
	type MaintenanceOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, OpenTechCommitteeInstance, 5, 9>;
	type XcmExecutionManager = XcmExecutionManager;
}

impl pallet_proxy_genesis_companion::Config for Runtime {
	type ProxyType = ProxyType;
}

parameter_types! {
	pub OrbiterReserveIdentifier: [u8; 4] = [b'o', b'r', b'b', b'i'];
}

type AddCollatorOrigin =
	EitherOfDiverse<EnsureRoot<AccountId>, governance::custom_origins::GeneralAdmin>;
type DelCollatorOrigin =
	EitherOfDiverse<EnsureRoot<AccountId>, governance::custom_origins::GeneralAdmin>;

impl pallet_moonbeam_orbiters::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AccountLookup = AuthorMapping;
	type AddCollatorOrigin = AddCollatorOrigin;
	type Currency = Balances;
	type DelCollatorOrigin = DelCollatorOrigin;
	/// Maximum number of orbiters per collator
	type MaxPoolSize = ConstU32<8>;
	/// Maximum number of round to keep on storage
	type MaxRoundArchive = ConstU32<4>;
	type OrbiterReserveIdentifier = OrbiterReserveIdentifier;
	type RotatePeriod = ConstU32<1>;
	/// Round index type.
	type RoundIndex = pallet_parachain_staking::RoundIndex;
	type WeightInfo = moonbeam_weights::pallet_moonbeam_orbiters::WeightInfo<Runtime>;
}

/// Only callable after `set_validation_data` is called which forms this proof the same way
fn relay_chain_state_proof<Runtime>() -> RelayChainStateProof
where
	Runtime: cumulus_pallet_parachain_system::Config,
{
	let relay_storage_root = ValidationData::<Runtime>::get()
		.expect("set in `set_validation_data`")
		.relay_parent_storage_root;
	let relay_chain_state =
		RelayStateProof::<Runtime>::get().expect("set in `set_validation_data`");
	RelayChainStateProof::new(ParachainInfo::get(), relay_storage_root, relay_chain_state)
		.expect("Invalid relay chain state proof, already constructed in `set_validation_data`")
}

pub struct BabeDataGetter<Runtime>(sp_std::marker::PhantomData<Runtime>);
impl<Runtime> pallet_randomness::GetBabeData<u64, Option<Hash>> for BabeDataGetter<Runtime>
where
	Runtime: cumulus_pallet_parachain_system::Config,
{
	// Tolerate panic here because only ever called in inherent (so can be omitted)
	fn get_epoch_index() -> u64 {
		if cfg!(feature = "runtime-benchmarks") {
			// storage reads as per actual reads
			let _relay_storage_root = ValidationData::<Runtime>::get();
			let _relay_chain_state = RelayStateProof::<Runtime>::get();
			const BENCHMARKING_NEW_EPOCH: u64 = 10u64;
			return BENCHMARKING_NEW_EPOCH;
		}
		relay_chain_state_proof::<Runtime>()
			.read_optional_entry(relay_chain::well_known_keys::EPOCH_INDEX)
			.ok()
			.flatten()
			.expect("expected to be able to read epoch index from relay chain state proof")
	}
	fn get_epoch_randomness() -> Option<Hash> {
		if cfg!(feature = "runtime-benchmarks") {
			// storage reads as per actual reads
			let _relay_storage_root = ValidationData::<Runtime>::get();
			let _relay_chain_state = RelayStateProof::<Runtime>::get();
			let benchmarking_babe_output = Hash::default();
			return Some(benchmarking_babe_output);
		}
		relay_chain_state_proof::<Runtime>()
			.read_optional_entry(relay_chain::well_known_keys::ONE_EPOCH_AGO_RANDOMNESS)
			.ok()
			.flatten()
	}
}

impl pallet_randomness::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddressMapping = sp_runtime::traits::ConvertInto;
	type Currency = Balances;
	type BabeDataGetter = BabeDataGetter<Runtime>;
	type VrfKeyLookup = AuthorMapping;
	type Deposit = runtime_params::PalletRandomnessDepositU128;
	type MaxRandomWords = ConstU8<100>;
	type MinBlockDelay = ConstU32<2>;
	type MaxBlockDelay = ConstU32<2_000>;
	type BlockExpirationDelay = ConstU32<10_000>;
	type EpochExpirationDelay = ConstU64<10_000>;
	type WeightInfo = moonbeam_weights::pallet_randomness::WeightInfo<Runtime>;
}

impl pallet_root_testing::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

parameter_types! {
	// One storage item; key size is 32 + 20; value is size 4+4+16+20 bytes = 44 bytes.
	pub const DepositBase: Balance = currency::deposit(1, 96);
	// Additional storage item size of 20 bytes.
	pub const DepositFactor: Balance = currency::deposit(0, 20);
	pub const MaxSignatories: u32 = 100;
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = moonbeam_weights::pallet_multisig::WeightInfo<Runtime>;
}

impl pallet_relay_storage_roots::Config for Runtime {
	type MaxStorageRoots = ConstU32<30>;
	type RelaychainStateProvider = cumulus_pallet_parachain_system::RelaychainDataProvider<Self>;
	type WeightInfo = moonbeam_weights::pallet_relay_storage_roots::WeightInfo<Runtime>;
}

impl pallet_precompile_benchmarks::Config for Runtime {
	type WeightInfo = moonbeam_weights::pallet_precompile_benchmarks::WeightInfo<Runtime>;
}

impl pallet_parameters::Config for Runtime {
	type AdminOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeParameters = RuntimeParameters;
	type WeightInfo = moonbeam_weights::pallet_parameters::WeightInfo<Runtime>;
}

/// List of multiblock migrations to be executed by the pallet_multiblock_migrations.
type MultiBlockMigrationList = ();

impl pallet_multiblock_migrations::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	// TODO fully replace pallet_migrations with multiblock migrations.
	#[cfg(not(feature = "runtime-benchmarks"))]
	type Migrations = MultiBlockMigrationList;
	// Benchmarks need mocked migrations to guarantee that they succeed.
	#[cfg(feature = "runtime-benchmarks")]
	type Migrations = pallet_multiblock_migrations::mock_helpers::MockedMigrations;
	type CursorMaxLen = ConstU32<65_536>;
	type IdentifierMaxLen = ConstU32<256>;
	type MigrationStatusHandler = ();
	type FailedMigrationHandler = MaintenanceMode;
	type MaxServiceWeight = MaxServiceWeight;
	type WeightInfo = weights::pallet_multiblock_migrations::WeightInfo<Runtime>;
}

construct_runtime! {
	pub enum Runtime
	{
		// System support stuff.
		System: frame_system::{Pallet, Call, Storage, Config<T>, Event<T>} = 0,
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>} = 1,
		// Previously 2: pallet_randomness_collective_flip
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 3,
		ParachainInfo: parachain_info::{Pallet, Storage, Config<T>} = 4,
		RootTesting: pallet_root_testing::{Pallet, Call, Storage, Event<T>} = 5,

		// Monetary stuff.
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Config<T>, Event<T>} = 11,

		// Consensus support.
		ParachainStaking: pallet_parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>} = 20,
		AuthorInherent: pallet_author_inherent::{Pallet, Call, Storage, Inherent} = 21,
		AuthorFilter: pallet_author_slot_filter::{Pallet, Call, Storage, Event, Config<T>} = 22,
		AuthorMapping: pallet_author_mapping::{Pallet, Call, Config<T>, Storage, Event<T>} = 23,
		MoonbeamOrbiters: pallet_moonbeam_orbiters::{Pallet, Call, Storage, Event<T>} = 24,
		AsyncBacking: pallet_async_backing::{Pallet, Storage} = 25,

		// Handy utilities.
		Utility: pallet_utility::{Pallet, Call, Event} = 30,
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 31,
		MaintenanceMode: pallet_maintenance_mode::{Pallet, Call, Config<T>, Storage, Event} = 32,
		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>} = 33,
		Migrations: pallet_migrations::{Pallet, Storage, Config<T>, Event<T>} = 34,
		ProxyGenesisCompanion: pallet_proxy_genesis_companion::{Pallet, Config<T>} = 35,
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 36,
		MoonbeamLazyMigrations: pallet_moonbeam_lazy_migrations::{Pallet, Call, Storage} = 37,
		Parameters: pallet_parameters = 38,

		// Has been permanently removed for safety reasons.
		// Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>} = 40,

		// Ethereum compatibility.
		EthereumChainId: pallet_evm_chain_id::{Pallet, Storage, Config<T>} = 50,
		EVM: pallet_evm::{Pallet, Config<T>, Call, Storage, Event<T>} = 51,
		Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Origin, Config<T>} = 52,

		// Governance stuff.
		Scheduler: pallet_scheduler::{Pallet, Storage, Event<T>, Call} = 60,
		// Democracy: 61,
		Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>, HoldReason} = 62,
		ConvictionVoting: pallet_conviction_voting::{Pallet, Call, Storage, Event<T>} = 63,
		Referenda: pallet_referenda::{Pallet, Call, Storage, Event<T>} = 64,
		Origins: governance::custom_origins::{Origin} = 65,
		Whitelist: pallet_whitelist::{Pallet, Call, Storage, Event<T>} = 66,

		// Council stuff.
		// CouncilCollective: 70
		// TechCommitteeCollective: 71
		TreasuryCouncilCollective:
			pallet_collective::<Instance3>::{Pallet, Call, Storage, Event<T>, Origin<T>, Config<T>} = 72,
		OpenTechCommitteeCollective:
			pallet_collective::<Instance4>::{Pallet, Call, Storage, Event<T>, Origin<T>, Config<T>} = 73,

		// Treasury stuff.
		Treasury: pallet_treasury::{Pallet, Storage, Config<T>, Event<T>, Call} = 80,

		// Crowdloan stuff.
		CrowdloanRewards: pallet_crowdloan_rewards::{Pallet, Call, Config<T>, Storage, Event<T>} = 90,

		// XCM
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Storage, Event<T>} = 100,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 101,
		// Previously 102: DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>}
		PolkadotXcm: pallet_xcm::{Pallet, Storage, Call, Event<T>, Origin, Config<T>} = 103,
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>} = 104,
		AssetManager: pallet_asset_manager::{Pallet, Call, Storage, Event<T>} = 105,
		// Previously 106: XTokens
		XcmTransactor: pallet_xcm_transactor::{Pallet, Call, Storage, Event<T>} = 107,
		// Previously 108: pallet_assets::<Instance1>
		EthereumXcm: pallet_ethereum_xcm::{Pallet, Call, Storage, Origin, Event<T>} = 109,
		Erc20XcmBridge: pallet_erc20_xcm_bridge::{Pallet} = 110,
		MessageQueue: pallet_message_queue::{Pallet, Call, Storage, Event<T>} = 111,
		EvmForeignAssets: pallet_moonbeam_foreign_assets::{Pallet, Call, Storage, Event<T>} = 114,
		XcmWeightTrader: pallet_xcm_weight_trader::{Pallet, Call, Storage, Event<T>} = 115,
		EmergencyParaXcm: pallet_emergency_para_xcm::{Pallet, Call, Storage, Event} = 116,
		MultiBlockMigrations: pallet_multiblock_migrations = 117,

		// Utils
		RelayStorageRoots: pallet_relay_storage_roots::{Pallet, Storage} = 112,

		// TODO should not be included in production
		PrecompileBenchmarks: pallet_precompile_benchmarks::{Pallet} = 113,

		// Randomness
		Randomness: pallet_randomness::{Pallet, Call, Storage, Event<T>, Inherent} = 120,
	}
}

#[cfg(feature = "runtime-benchmarks")]
use moonbeam_runtime_common::benchmarking::BenchmarkHelper;
use moonbeam_runtime_common::deal_with_fees::{
	DealWithEthereumBaseFees, DealWithEthereumPriorityFees, DealWithSubstrateFeesAndTip,
};

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	frame_support::parameter_types! {
		pub const MaxBalance: crate::Balance = crate::Balance::max_value();
	}

	frame_benchmarking::define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
		[frame_system_extensions, frame_system_benchmarking::extensions::Pallet::<Runtime>]
		[pallet_utility, Utility]
		[pallet_timestamp, Timestamp]
		[pallet_balances, Balances]
		[pallet_evm, EVM]
		[pallet_assets, Assets]
		[pallet_parachain_staking, ParachainStaking]
		[pallet_scheduler, Scheduler]
		[pallet_treasury, Treasury]
		[pallet_author_inherent, AuthorInherent]
		[pallet_author_slot_filter, AuthorFilter]
		[pallet_crowdloan_rewards, CrowdloanRewards]
		[pallet_author_mapping, AuthorMapping]
		[pallet_proxy, Proxy]
		[pallet_transaction_payment, PalletTransactionPaymentBenchmark::<Runtime>]
		[pallet_identity, Identity]
		[cumulus_pallet_parachain_system, ParachainSystem]
		[cumulus_pallet_xcmp_queue, XcmpQueue]
		[pallet_message_queue, MessageQueue]
		[pallet_xcm, PalletXcmExtrinsicsBenchmark::<Runtime>]
		[pallet_asset_manager, AssetManager]
		[pallet_xcm_transactor, XcmTransactor]
		[pallet_moonbeam_foreign_assets, EvmForeignAssets]
		[pallet_moonbeam_orbiters, MoonbeamOrbiters]
		[pallet_randomness, Randomness]
		[pallet_conviction_voting, ConvictionVoting]
		[pallet_referenda, Referenda]
		[pallet_preimage, Preimage]
		[pallet_whitelist, Whitelist]
		[pallet_multisig, Multisig]
		[pallet_multiblock_migrations, MultiBlockMigrations]
		[pallet_moonbeam_lazy_migrations, MoonbeamLazyMigrations]
		[pallet_relay_storage_roots, RelayStorageRoots]
		[pallet_precompile_benchmarks, PrecompileBenchmarks]
		[pallet_parameters, Parameters]
		[pallet_xcm_weight_trader, XcmWeightTrader]
		[pallet_collective, TreasuryCouncilCollective]
		[pallet_collective, OpenTechCommitteeCollective]
	);
}

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
	cumulus_primitives_storage_weight_reclaim::StorageWeightReclaim<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	fp_self_contained::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic =
	fp_self_contained::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra, H160>;
/// Executive: handles dispatch to the various pallets.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
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
				RuntimeCall::Ethereum(transact { .. }) => intermediate_valid,
				_ if dispatch_info.class != DispatchClass::Normal => intermediate_valid,
				_ => {
					let tip = match &xt.0.preamble {
						Preamble::Bare(_) => 0,
						Preamble::Signed(_, _, signed_extra) => {
							// Yuck, this depends on the index of ChargeTransactionPayment in SignedExtra
							// Get the 7th item from the tuple
							let charge_transaction_payment = &signed_extra.7;
							charge_transaction_payment.tip()
						},
						Preamble::General(_, _) => 0,
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
							dispatch_info.total_weight()
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

	impl async_backing_primitives::UnincludedSegmentApi<Block> for Runtime {
		fn can_build_upon(
			included_hash: <Block as BlockT>::Hash,
			slot: async_backing_primitives::Slot,
		) -> bool {
			ConsensusHook::can_build_upon(included_hash, slot)
		}
	}
}

#[allow(dead_code)]
struct CheckInherents;

// Parity has decided to depreciate this trait, but does not offer a satisfactory replacement,
// see issue: https://github.com/paritytech/polkadot-sdk/issues/2841
#[allow(deprecated)]
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
		inherent_data.check_extrinsics(block)
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
		assert!(std::mem::size_of::<pallet_evm_chain_id::Call<Runtime>>() <= CALL_ALIGN as usize);
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
			std::mem::size_of::<pallet_moonbeam_lazy_migrations::Call<Runtime>>()
				<= CALL_ALIGN as usize
		);
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

		// pallet_identity deposits
		assert_eq!(
			get!(pallet_identity, BasicDeposit, u128),
			Balance::from(10 * GLMR + 2580 * MILLIGLMR)
		);
		assert_eq!(
			get!(pallet_identity, ByteDeposit, u128),
			Balance::from(10 * MILLIGLMR)
		);
		assert_eq!(
			get!(pallet_identity, SubAccountDeposit, u128),
			Balance::from(10 * GLMR + 530 * MILLIGLMR)
		);

		// staking minimums
		assert_eq!(
			get!(pallet_parachain_staking, MinCandidateStk, u128),
			Balance::from(500_000 * GLMR)
		);
		assert_eq!(
			get!(pallet_parachain_staking, MinDelegation, u128),
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
	fn max_offline_rounds_lower_or_eq_than_reward_payment_delay() {
		assert!(
			get!(pallet_parachain_staking, MaxOfflineRounds, u32)
				<= get!(pallet_parachain_staking, RewardPaymentDelay, u32)
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

	#[test]
	fn configured_base_extrinsic_weight_is_evm_compatible() {
		let min_ethereum_transaction_weight = WeightPerGas::get() * 21_000;
		let base_extrinsic = <Runtime as frame_system::Config>::BlockWeights::get()
			.get(frame_support::dispatch::DispatchClass::Normal)
			.base_extrinsic;
		assert!(base_extrinsic.ref_time() <= min_ethereum_transaction_weight.ref_time());
	}

	#[test]
	fn test_storage_growth_ratio_is_correct() {
		// This is the highest amount of new storage that can be created in a block 160 KB
		let block_storage_limit = 160 * 1024;
		let expected_storage_growth_ratio = BlockGasLimit::get()
			.low_u64()
			.saturating_div(block_storage_limit);
		let actual_storage_growth_ratio =
			<Runtime as pallet_evm::Config>::GasLimitStorageGrowthRatio::get();
		assert_eq!(
			expected_storage_growth_ratio, actual_storage_growth_ratio,
			"Storage growth ratio is not correct"
		);
	}
}
