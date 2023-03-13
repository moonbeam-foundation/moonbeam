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

use cumulus_pallet_parachain_system::{RelayChainStateProof, RelaychainBlockNumberProvider};
use cumulus_primitives_core::relay_chain;
use fp_rpc::TransactionStatus;

use account::AccountId20;

// Re-export required by get! macro.
pub use frame_support::traits::Get;
use frame_support::{
	construct_runtime,
	dispatch::{DispatchClass, GetDispatchInfo},
	ensure,
	pallet_prelude::DispatchResult,
	parameter_types,
	traits::{
		ConstBool, ConstU128, ConstU16, ConstU32, ConstU64, ConstU8, Contains,
		Currency as CurrencyT, EitherOfDiverse, EqualPrivilegeOnly, FindAuthor, Imbalance,
		InstanceFilter, OffchainWorker, OnFinalize, OnIdle, OnInitialize, OnRuntimeUpgrade,
		OnUnbalanced,
	},
	weights::{
		constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
		ConstantMultiplier, Weight, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	},
	PalletId,
};

#[cfg(feature = "std")]
pub use fp_evm::GenesisAccount;
use frame_system::{EnsureRoot, EnsureSigned};
pub use moonbeam_core_primitives::{
	AccountId, AccountIndex, Address, AssetId, Balance, BlockNumber, DigestItem, Hash, Header,
	Index, Signature,
};
use moonbeam_rpc_primitives_txpool::TxPoolResponse;
pub use pallet_author_slot_filter::EligibilityValue;
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
use sp_api::impl_runtime_apis;
use sp_core::{OpaqueMetadata, H160, H256, U256};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{
		BlakeTwo256, Block as BlockT, DispatchInfoOf, Dispatchable, IdentityLookup,
		PostDispatchInfoOf, UniqueSaturatedInto, Zero,
	},
	transaction_validity::{
		InvalidTransaction, TransactionSource, TransactionValidity, TransactionValidityError,
	},
	ApplyExtrinsicResult, FixedPointNumber, Perbill, Permill, Perquintill,
};
use sp_std::{
	convert::{From, Into},
	prelude::*,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use nimbus_primitives::CanAuthor;

mod precompiles;
pub use precompiles::{
	MoonbasePrecompiles, PrecompileName, FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
	LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX,
};

use smallvec::smallvec;

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

pub type Precompiles = MoonbasePrecompiles<Runtime>;

pub mod asset_config;
pub mod governance;
pub mod xcm_config;
use governance::councils::*;

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

	pub const TRANSACTION_BYTE_FEE: Balance = 1 * GIGAWEI * SUPPLY_FACTOR;
	pub const STORAGE_BYTE_FEE: Balance = 100 * MICROUNIT * SUPPLY_FACTOR;
	pub const WEIGHT_FEE: Balance = 50 * KILOWEI * SUPPLY_FACTOR;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 1 * UNIT * SUPPLY_FACTOR + (bytes as Balance) * STORAGE_BYTE_FEE
	}
}

/// Maximum weight per block
pub const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_ref_time(WEIGHT_REF_TIME_PER_SECOND)
	.saturating_div(2)
	.set_proof_size(cumulus_primitives_core::relay_chain::v2::MAX_POV_SIZE as u64);

pub const MILLISECS_PER_BLOCK: u64 = 12000;
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;
pub const WEEKS: BlockNumber = DAYS * 7;
/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
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
	spec_name: create_runtime_str!("moonbase"),
	impl_name: create_runtime_str!("moonbase"),
	authoring_version: 4,
	spec_version: 2300,
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
pub const NORMAL_WEIGHT: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_mul(3).saturating_div(4);
// Here we assume Ethereum's base fee of 21000 gas and convert to weight, but we
// subtract roughly the cost of a balance transfer from it (about 1/3 the cost)
// and some cost to account for per-byte-fee.
// TODO: we should use benchmarking's overhead feature to measure this
pub const EXTRINSIC_BASE_WEIGHT: Weight = Weight::from_ref_time(10000 * WEIGHT_PER_GAS);

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
	/// TODO: this is left here so that `impl_runtime_apis_plus_common` will find the same type for
	/// `BlockWeights` in all runtimes. It can probably be removed once the custom
	/// `RuntimeBlockWeights` has been pushed to each runtime.
	pub BlockWeights: frame_system::limits::BlockWeights = RuntimeBlockWeights::get();
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
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
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
	type DbWeight = RocksDbWeight;
	type BaseCallFilter = MaintenanceMode;
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = ConstU16<1287>;
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
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
	type RuntimeEvent = RuntimeEvent;
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
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees<Runtime>>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = ConstantMultiplier<Balance, ConstU128<{ currency::WEIGHT_FEE }>>;
	type LengthToFee = LengthToFee;
	type FeeMultiplierUpdate = FastAdjustingFeeUpdate<Runtime>;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
}

impl pallet_ethereum_chain_id::Config for Runtime {}

/// Current approximation of the gas/s consumption considering
/// EVM execution over compiled WASM (on 4.4Ghz CPU).
/// Given the 500ms Weight, from which 75% only are used for transactions,
/// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 ~= 15_000_000.
pub const GAS_PER_SECOND: u64 = 40_000_000;

/// Approximate ratio of the amount of Weight per Gas.
/// u64 works for approximations because Weight is a very small unit compared to gas.
pub const WEIGHT_PER_GAS: u64 = WEIGHT_REF_TIME_PER_SECOND / GAS_PER_SECOND;

parameter_types! {
	pub BlockGasLimit: U256
		= U256::from(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT.ref_time() / WEIGHT_PER_GAS);
	/// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
	/// than this will decrease the weight and more will increase.
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	/// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
	/// change the fees more rapidly. This fast multiplier responds by doubling/halving in
	/// approximately one hour at extreme block congestion levels.
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(4, 1_000);
	/// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
	/// that combined with `AdjustmentVariable`, we can recover from the minimum.
	/// See `multiplier_can_grow_from_zero` in integration_tests.rs.
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 10);
	/// Maximum multiplier. We pick a value that is expensive but not impossibly so; it should act
	/// as a safety net.
	pub MaximumMultiplier: Multiplier = Multiplier::from(100_000u128);
	pub PrecompilesValue: MoonbasePrecompiles<Runtime> = MoonbasePrecompiles::<_>::new();
	pub WeightPerGas: Weight = Weight::from_ref_time(WEIGHT_PER_GAS);
}

pub struct TransactionPaymentAsGasPrice;
impl FeeCalculator for TransactionPaymentAsGasPrice {
	fn min_gas_price() -> (U256, Weight) {
		// TODO: transaction-payment differs from EIP-1559 in that its tip and length fees are not
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
			.saturating_mul_int(currency::WEIGHT_FEE.saturating_mul(WEIGHT_PER_GAS as u128));
		(
			min_gas_price.into(),
			<Runtime as frame_system::Config>::DbWeight::get().reads(1),
		)
	}
}

/// A "Fast" TargetedFeeAdjustment. Parameters chosen based on model described here:
/// https://research.web3.foundation/en/latest/polkadot/overview/2-token-economics.html#-1.-fast-adjusting-mechanism // editorconfig-checker-disable-line
///
/// The adjustment algorithm boils down to:
///
/// diff = (previous_block_weight - target) / maximum_block_weight
/// next_multiplier = prev_multiplier * (1 + (v * diff) + ((v * diff)^2 / 2))
/// assert(next_multiplier > min)
///     where: v is AdjustmentVariable
///            target is TargetBlockFullness
///            min is MinimumMultiplier
pub type FastAdjustingFeeUpdate<R> = TargetedFeeAdjustment<
	R,
	TargetBlockFullness,
	AdjustmentVariable,
	MinimumMultiplier,
	MaximumMultiplier,
>;

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

moonbeam_runtime_common::impl_on_charge_evm_transaction!();

impl pallet_evm::Config for Runtime {
	type FeeCalculator = TransactionPaymentAsGasPrice;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = moonbeam_runtime_common::IntoAddressMapping;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = MoonbasePrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = EthereumChainId;
	type OnChargeTransaction = OnChargeEVMTransaction<DealWithFees<Runtime>>;
	type BlockGasLimit = BlockGasLimit;
	type FindAuthor = FindAuthorAdapter<AccountId20, H160, AuthorInherent>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = NORMAL_DISPATCH_RATIO * RuntimeBlockWeights::get().max_block;
	pub const NoPreimagePostponement: Option<u32> = Some(10);
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = ConstU32<50>;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = Preimage;
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type BaseDeposit = ConstU128<{ 5 * currency::UNIT * currency::SUPPLY_FACTOR }>;
	type ByteDeposit = ConstU128<{ currency::STORAGE_BYTE_FEE }>;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const TreasuryId: PalletId = PalletId(*b"pc/trsry");
}

type TreasuryApproveOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, TreasuryCouncilInstance, 3, 5>,
>;

type TreasuryRejectOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, TreasuryCouncilInstance, 1, 2>,
>;

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryId;
	type Currency = Balances;
	// At least three-fifths majority of the council is required (or root) to approve a proposal
	type ApproveOrigin = TreasuryApproveOrigin;
	// More than half of the council is required (or root) to reject a proposal
	type RejectOrigin = TreasuryRejectOrigin;
	type RuntimeEvent = RuntimeEvent;
	// If spending proposal rejected, transfer proposer bond to treasury
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ConstU128<{ 1 * currency::UNIT * currency::SUPPLY_FACTOR }>;
	type SpendPeriod = ConstU32<{ 6 * DAYS }>;
	type Burn = ();
	type BurnDestination = ();
	type MaxApprovals = ConstU32<100>;
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	type SpendFunds = ();
	type ProposalBondMaximum = ();
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<Balance>; // Same as Polkadot
}

type IdentityForceOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EitherOfDiverse<
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilInstance, 1, 2>,
		governance::custom_origins::GeneralAdmin,
	>,
>;
type IdentityRegistrarOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EitherOfDiverse<
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilInstance, 1, 2>,
		governance::custom_origins::GeneralAdmin,
	>,
>;

impl pallet_identity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
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
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
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
	type InvalidEvmTransactionError = pallet_ethereum::InvalidTransactionWrapper;
	type ValidatedTransaction = pallet_ethereum::ValidatedTransaction<Self>;
	type XcmEthereumOrigin = pallet_ethereum_xcm::EnsureXcmEthereumTransaction;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type EnsureProxy = EthereumXcmEnsureProxy;
	type ControllerOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = ParachainInfo;
	type DmpMessageHandler = MaintenanceMode;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
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
		let extra_weight = if MoonbeamOrbiters::is_orbiter(for_round, collator_id) {
			MoonbeamOrbiters::distribute_rewards(for_round, collator_id, amount)
		} else {
			ParachainStaking::mint_collator_reward(for_round, collator_id, amount)
		};

		<Runtime as frame_system::Config>::DbWeight::get()
			.reads(1)
			.saturating_add(extra_weight)
	}
}

type MonetaryGovernanceOrigin =
	EitherOfDiverse<EnsureRoot<AccountId>, governance::custom_origins::GeneralAdmin>;

impl pallet_parachain_staking::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MonetaryGovernanceOrigin = MonetaryGovernanceOrigin;
	/// Minimum round length is 2 minutes (10 * 12 second block times)
	type MinBlocksPerRound = ConstU32<10>;
	/// Rounds before the collator leaving the candidates request can be executed
	type LeaveCandidatesDelay = ConstU32<2>;
	/// Rounds before the candidate bond increase/decrease can be executed
	type CandidateBondLessDelay = ConstU32<2>;
	/// Rounds before the delegator exit can be executed
	type LeaveDelegatorsDelay = ConstU32<2>;
	/// Rounds before the delegator revocation can be executed
	type RevokeDelegationDelay = ConstU32<2>;
	/// Rounds before the delegator bond increase/decrease can be executed
	type DelegationBondLessDelay = ConstU32<2>;
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
	/// Minimum stake required to become a collator
	type MinCollatorStk = ConstU128<{ 1000 * currency::UNIT * currency::SUPPLY_FACTOR }>;
	/// Minimum stake required to be reserved to be a candidate
	type MinCandidateStk = ConstU128<{ 500 * currency::UNIT * currency::SUPPLY_FACTOR }>;
	/// Minimum stake required to be reserved to be a delegator
	type MinDelegation = ConstU128<{ 1 * currency::UNIT * currency::SUPPLY_FACTOR }>;
	/// Minimum stake required to be reserved to be a delegator
	type MinDelegatorStk = ConstU128<{ 1 * currency::UNIT * currency::SUPPLY_FACTOR }>;
	type BlockAuthor = AuthorInherent;
	type OnCollatorPayout = ();
	type PayoutCollatorReward = PayoutCollatorOrOrbiterReward;
	type OnNewRound = OnNewRound;
	type WeightInfo = pallet_parachain_staking::weights::SubstrateWeight<Runtime>;
}

impl pallet_author_inherent::Config for Runtime {
	type SlotBeacon = RelaychainBlockNumberProvider<Self>;
	type AccountLookup = MoonbeamOrbiters;
	type CanAuthor = AuthorFilter;
	type WeightInfo = pallet_author_inherent::weights::SubstrateWeight<Runtime>;
}

impl pallet_author_slot_filter::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RandomnessSource = Randomness;
	type PotentialAuthors = ParachainStaking;
	type WeightInfo = pallet_author_slot_filter::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const InitializationPayment: Perbill = Perbill::from_percent(30);
	pub const RelaySignaturesThreshold: Perbill = Perbill::from_percent(100);
	pub const SignatureNetworkIdentifier:  &'static [u8] = b"moonbase-";

}

impl pallet_crowdloan_rewards::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Initialized = ConstBool<false>;
	type InitializationPayment = InitializationPayment;
	type MaxInitContributors = ConstU32<500>;
	// TODO to be revisited
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
	type RuntimeEvent = RuntimeEvent;
	type DepositCurrency = Balances;
	type DepositAmount = ConstU128<{ 100 * currency::UNIT * currency::SUPPLY_FACTOR }>;
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

fn is_governance_precompile(precompile_name: &precompiles::PrecompileName) -> bool {
	matches!(
		precompile_name,
		PrecompileName::DemocracyPrecompile
			| PrecompileName::CouncilInstance
			| PrecompileName::TechCommitteeInstance
			| PrecompileName::TreasuryCouncilInstance
			| PrecompileName::ReferendaPrecompile
			| PrecompileName::ConvictionVotingPrecompile
			| PrecompileName::PreimagePrecompile
			| PrecompileName::OpenTechCommitteeInstance,
	)
}

// Be careful: Each time this filter is modified, the substrate filter must also be modified
// consistently.
impl pallet_evm_precompile_proxy::EvmProxyCallFilter for ProxyType {
	fn is_evm_proxy_call_allowed(
		&self,
		call: &pallet_evm_precompile_proxy::EvmSubCall,
		recipient_has_code: bool,
	) -> bool {
		use pallet_evm::PrecompileSet as _;
		match self {
			ProxyType::Any => true,
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
				!recipient_has_code && !PrecompilesValue::get().is_precompile(call.to.0)
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
		}
	}
}

// Be careful: Each time this filter is modified, the EVM filter must also be modified consistently.
impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => {
				matches!(
					c,
					RuntimeCall::System(..)
						| RuntimeCall::ParachainSystem(..)
						| RuntimeCall::Timestamp(..)
						| RuntimeCall::ParachainStaking(..)
						| RuntimeCall::Democracy(..)
						| RuntimeCall::Referenda(..)
						| RuntimeCall::Preimage(..)
						| RuntimeCall::ConvictionVoting(..)
						| RuntimeCall::CouncilCollective(..)
						| RuntimeCall::TreasuryCouncilCollective(..)
						| RuntimeCall::TechCommitteeCollective(..)
						| RuntimeCall::OpenTechCommitteeCollective(..)
						| RuntimeCall::Identity(..)
						| RuntimeCall::Utility(..)
						| RuntimeCall::Proxy(..) | RuntimeCall::AuthorMapping(..)
						| RuntimeCall::CrowdloanRewards(
							pallet_crowdloan_rewards::Call::claim { .. }
						)
				)
			}
			ProxyType::Governance => matches!(
				c,
				RuntimeCall::Democracy(..)
					| RuntimeCall::Referenda(..)
					| RuntimeCall::Preimage(..)
					| RuntimeCall::ConvictionVoting(..)
					| RuntimeCall::CouncilCollective(..)
					| RuntimeCall::TreasuryCouncilCollective(..)
					| RuntimeCall::TechCommitteeCollective(..)
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
	type RuntimeEvent = RuntimeEvent;
	// TODO wire up our correct list of migrations here. Maybe this shouldn't be in
	// `moonbeam_runtime_common`.
	type MigrationsList = (
		moonbeam_runtime_common::migrations::CommonMigrations<
			Runtime,
			CouncilCollective,
			TechCommitteeCollective,
		>,
		moonbeam_runtime_common::migrations::ReferendaMigrations<
			Runtime,
			CouncilCollective,
			TechCommitteeCollective,
		>,
	);
	type XcmExecutionManager = XcmExecutionManager;
	type WeightInfo = pallet_migrations::weights::SubstrateWeight<Runtime>;
}

/// Maintenance mode Call filter
pub struct MaintenanceFilter;
impl Contains<RuntimeCall> for MaintenanceFilter {
	fn contains(c: &RuntimeCall) -> bool {
		match c {
			RuntimeCall::Assets(_) => false,
			RuntimeCall::LocalAssets(_) => false,
			RuntimeCall::Balances(_) => false,
			RuntimeCall::CrowdloanRewards(_) => false,
			RuntimeCall::Ethereum(_) => false,
			RuntimeCall::EVM(_) => false,
			RuntimeCall::Identity(_) => false,
			RuntimeCall::XTokens(_) => false,
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
			// We want to disable create, as we dont want users to be choosing the
			// assetId of their choice
			// We also disable destroy, as we want to route destroy through the
			// asset-manager, which guarantees the removal both at the EVM and
			// substrate side of things
			RuntimeCall::LocalAssets(method) => match method {
				pallet_assets::Call::create { .. } => false,
				pallet_assets::Call::start_destroy { .. } => false,
				_ => true,
			},
			// We filter anonymous proxy as they make "reserve" inconsistent
			// See: https://github.com/paritytech/substrate/blob/37cca710eed3dadd4ed5364c7686608f5175cce1/frame/proxy/src/lib.rs#L270 // editorconfig-checker-disable-line
			RuntimeCall::Proxy(method) => match method {
				pallet_proxy::Call::create_pure { .. } => false,
				pallet_proxy::Call::kill_pure { .. } => false,
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

use cumulus_primitives_core::{relay_chain::BlockNumber as RelayBlockNumber, DmpMessageHandler};

pub struct XcmExecutionManager;
impl xcm_primitives::PauseXcmExecution for XcmExecutionManager {
	fn suspend_xcm_execution() -> DispatchResult {
		XcmpQueue::suspend_xcm_execution(RuntimeOrigin::root())
	}
	fn resume_xcm_execution() -> DispatchResult {
		XcmpQueue::resume_xcm_execution(RuntimeOrigin::root())
	}
}

pub struct NormalDmpHandler;
impl DmpMessageHandler for NormalDmpHandler {
	// This implementation makes messages be queued
	// Since the limit is 0, messages are queued for next iteration
	fn handle_dmp_messages(
		iter: impl Iterator<Item = (RelayBlockNumber, Vec<u8>)>,
		limit: Weight,
	) -> Weight {
		(if Migrations::should_pause_xcm() {
			DmpQueue::handle_dmp_messages(iter, Weight::zero())
		} else {
			DmpQueue::handle_dmp_messages(iter, limit)
		}) + <Runtime as frame_system::Config>::DbWeight::get().reads(1)
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
		DmpQueue::handle_dmp_messages(iter, Weight::zero())
	}
}

/// The hooks we wnat to run in Maintenance Mode
pub struct MaintenanceHooks;

impl OnInitialize<BlockNumber> for MaintenanceHooks {
	fn on_initialize(n: BlockNumber) -> Weight {
		AllPalletsWithSystem::on_initialize(n)
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
		Weight::zero()
	}
}

impl OnRuntimeUpgrade for MaintenanceHooks {
	fn on_runtime_upgrade() -> Weight {
		AllPalletsWithSystem::on_runtime_upgrade()
	}
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		AllPalletsWithSystem::pre_upgrade()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		AllPalletsWithSystem::post_upgrade(state)
	}
}

impl OnFinalize<BlockNumber> for MaintenanceHooks {
	fn on_finalize(n: BlockNumber) {
		AllPalletsWithSystem::on_finalize(n)
	}
}

impl OffchainWorker<BlockNumber> for MaintenanceHooks {
	fn offchain_worker(n: BlockNumber) {
		AllPalletsWithSystem::offchain_worker(n)
	}
}

impl pallet_maintenance_mode::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type NormalCallFilter = NormalFilter;
	type MaintenanceCallFilter = MaintenanceFilter;
	type MaintenanceOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, TechCommitteeInstance, 2, 3>;
	type XcmExecutionManager = XcmExecutionManager;
	type NormalDmpHandler = NormalDmpHandler;
	type MaintenanceDmpHandler = MaintenanceDmpHandler;
	// We use AllPalletsWithSystem because we dont want to change the hooks in normal
	// operation
	type NormalExecutiveHooks = AllPalletsWithSystem;
	type MaintenanceExecutiveHooks = MaintenanceHooks;
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
	type RotatePeriod = ConstU32<3>;
	/// Round index type.
	type RoundIndex = pallet_parachain_staking::RoundIndex;
	type WeightInfo = pallet_moonbeam_orbiters::weights::SubstrateWeight<Runtime>;
}

/// Only callable after `set_validation_data` is called which forms this proof the same way
fn relay_chain_state_proof() -> RelayChainStateProof {
	let relay_storage_root = ParachainSystem::validation_data()
		.expect("set in `set_validation_data`")
		.relay_parent_storage_root;
	let relay_chain_state =
		ParachainSystem::relay_state_proof().expect("set in `set_validation_data`");
	RelayChainStateProof::new(ParachainInfo::get(), relay_storage_root, relay_chain_state)
		.expect("Invalid relay chain state proof, already constructed in `set_validation_data`")
}

pub struct BabeDataGetter;
impl pallet_randomness::GetBabeData<u64, Option<Hash>> for BabeDataGetter {
	// Tolerate panic here because only ever called in inherent (so can be omitted)
	fn get_epoch_index() -> u64 {
		if cfg!(feature = "runtime-benchmarks") {
			// storage reads as per actual reads
			let _relay_storage_root = ParachainSystem::validation_data();
			let _relay_chain_state = ParachainSystem::relay_state_proof();
			const BENCHMARKING_NEW_EPOCH: u64 = 10u64;
			return BENCHMARKING_NEW_EPOCH;
		}
		relay_chain_state_proof()
			.read_optional_entry(relay_chain::well_known_keys::EPOCH_INDEX)
			.ok()
			.flatten()
			.expect("expected to be able to read epoch index from relay chain state proof")
	}
	fn get_epoch_randomness() -> Option<Hash> {
		if cfg!(feature = "runtime-benchmarks") {
			// storage reads as per actual reads
			let _relay_storage_root = ParachainSystem::validation_data();
			let _relay_chain_state = ParachainSystem::relay_state_proof();
			let benchmarking_babe_output = Hash::default();
			return Some(benchmarking_babe_output);
		}
		relay_chain_state_proof()
			.read_optional_entry(relay_chain::well_known_keys::ONE_EPOCH_AGO_RANDOMNESS)
			.ok()
			.flatten()
	}
}

impl pallet_randomness::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddressMapping = moonbeam_runtime_common::IntoAddressMapping;
	type Currency = Balances;
	type BabeDataGetter = BabeDataGetter;
	type VrfKeyLookup = AuthorMapping;
	type Deposit = ConstU128<{ 1 * currency::UNIT * currency::SUPPLY_FACTOR }>;
	type MaxRandomWords = ConstU8<100>;
	type MinBlockDelay = ConstU32<2>;
	type MaxBlockDelay = ConstU32<2_000>;
	type BlockExpirationDelay = ConstU32<10_000>;
	type EpochExpirationDelay = ConstU64<10_000>;
}

impl pallet_root_testing::Config for Runtime {}

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
		// Previously 5: pallet_randomness_collective_flip
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Storage, Inherent, Event<T>} = 6,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Config, Event<T>} = 7,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 8,
		EthereumChainId: pallet_ethereum_chain_id::{Pallet, Storage, Config} = 9,
		EVM: pallet_evm::{Pallet, Config, Call, Storage, Event<T>} = 10,
		Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Origin, Config} = 11,
		ParachainStaking: pallet_parachain_staking::{Pallet, Call, Storage, Event<T>, Config<T>} = 12,
		Scheduler: pallet_scheduler::{Pallet, Storage, Event<T>, Call} = 13,
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
		Migrations: pallet_migrations::{Pallet, Call, Storage, Config, Event<T>} = 32,
		XcmTransactor: pallet_xcm_transactor::{Pallet, Call, Storage, Event<T>} = 33,
		ProxyGenesisCompanion: pallet_proxy_genesis_companion::{Pallet, Config<T>} = 34,
		LocalAssets: pallet_assets::<Instance1>::{Pallet, Call, Storage, Event<T>} = 36,
		MoonbeamOrbiters: pallet_moonbeam_orbiters::{Pallet, Call, Storage, Event<T>} = 37,
		EthereumXcm: pallet_ethereum_xcm::{Pallet, Call, Storage, Origin} = 38,
		Randomness: pallet_randomness::{Pallet, Call, Storage, Event<T>, Inherent} = 39,
		TreasuryCouncilCollective:
			pallet_collective::<Instance3>::{Pallet, Call, Storage, Event<T>, Origin<T>, Config<T>} = 40,
		ConvictionVoting: pallet_conviction_voting::{Pallet, Call, Storage, Event<T>} = 41,
		Referenda: pallet_referenda::{Pallet, Call, Storage, Event<T>} = 42,
		Origins: governance::custom_origins::{Origin} = 43,
		Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 44,
		Whitelist: pallet_whitelist::{Pallet, Call, Storage, Event<T>} = 45,
		OpenTechCommitteeCollective:
			pallet_collective::<Instance4>::{Pallet, Call, Storage, Event<T>, Origin<T>, Config<T>} = 46,
		RootTesting: pallet_root_testing::{Pallet, Call, Storage} = 47,
		Erc20XcmBridge: pallet_erc20_xcm_bridge::{Pallet} = 48,
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
			// according to effective priority fee from pallet ethereum. If it is any other kind of
			// transaction, we modify its priority. The goal is to arrive at a similar metric used
			// by pallet ethereum, which means we derive a fee-per-gas from the txn's tip and
			// weight.
			Ok(match &xt.0.function {
				RuntimeCall::Ethereum(transact { .. }) => intermediate_valid,
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

					let effective_gas =
						<Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
							dispatch_info.weight
						);
					let tip_per_gas = if effective_gas > 0 {
						tip.saturating_div(effective_gas as u128)
					} else {
						0
					};

					// Overwrite the original prioritization with this ethereum one
					intermediate_valid.priority = tip_per_gas as u64;
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
		relay_state_proof: &RelayChainStateProof,
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
		assert!(std::mem::size_of::<orml_xtokens::Call<Runtime>>() <= CALL_ALIGN as usize);
		assert!(std::mem::size_of::<pallet_asset_manager::Call<Runtime>>() <= CALL_ALIGN as usize);
		assert!(std::mem::size_of::<pallet_migrations::Call<Runtime>>() <= CALL_ALIGN as usize);
		assert!(std::mem::size_of::<pallet_xcm_transactor::Call<Runtime>>() <= CALL_ALIGN as usize);
		assert!(
			std::mem::size_of::<pallet_proxy_genesis_companion::Call<Runtime>>()
				<= CALL_ALIGN as usize
		);
	}

	#[test]
	fn currency_constants_are_correct() {
		assert_eq!(SUPPLY_FACTOR, 1);

		// txn fees
		assert_eq!(TRANSACTION_BYTE_FEE, Balance::from(1 * GIGAWEI));
		assert_eq!(
			get!(pallet_transaction_payment, OperationalFeeMultiplier, u8),
			5_u8
		);
		assert_eq!(STORAGE_BYTE_FEE, Balance::from(100 * MICROUNIT));

		// democracy minimums
		assert_eq!(
			get!(pallet_democracy, MinimumDeposit, u128),
			Balance::from(4 * UNIT)
		);
		assert_eq!(
			get!(pallet_preimage, ByteDeposit, u128),
			Balance::from(100 * MICROUNIT)
		);
		assert_eq!(
			get!(pallet_treasury, ProposalBondMinimum, u128),
			Balance::from(1 * UNIT)
		);

		// pallet_identity deposits
		assert_eq!(
			get!(pallet_identity, BasicDeposit, u128),
			Balance::from(1 * UNIT + 25800 * MICROUNIT)
		);
		assert_eq!(
			get!(pallet_identity, FieldDeposit, u128),
			Balance::from(6600 * MICROUNIT)
		);
		assert_eq!(
			get!(pallet_identity, SubAccountDeposit, u128),
			Balance::from(1 * UNIT + 5300 * MICROUNIT)
		);

		// staking minimums
		assert_eq!(
			get!(pallet_parachain_staking, MinCollatorStk, u128),
			Balance::from(1 * KILOUNIT)
		);
		assert_eq!(
			get!(pallet_parachain_staking, MinCandidateStk, u128),
			Balance::from(500 * UNIT)
		);
		assert_eq!(
			get!(pallet_parachain_staking, MinDelegation, u128),
			Balance::from(1 * UNIT)
		);
		assert_eq!(
			get!(pallet_parachain_staking, MinDelegatorStk, u128),
			Balance::from(1 * UNIT)
		);

		// crowdloan min reward
		assert_eq!(
			get!(pallet_crowdloan_rewards, MinimumReward, u128),
			Balance::from(0u128)
		);

		// deposit for AuthorMapping
		assert_eq!(
			get!(pallet_author_mapping, DepositAmount, u128),
			Balance::from(100 * UNIT)
		);

		// proxy deposits
		assert_eq!(
			get!(pallet_proxy, ProxyDepositBase, u128),
			Balance::from(1 * UNIT + 800 * MICROUNIT)
		);
		assert_eq!(
			get!(pallet_proxy, ProxyDepositFactor, u128),
			Balance::from(2100 * MICROUNIT)
		);
		assert_eq!(
			get!(pallet_proxy, AnnouncementDepositBase, u128),
			Balance::from(1 * UNIT + 800 * MICROUNIT)
		);
		assert_eq!(
			get!(pallet_proxy, AnnouncementDepositFactor, u128),
			Balance::from(5600 * MICROUNIT)
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
	fn test_proxy_type_can_be_decoded_from_valid_values() {
		let test_cases = vec![
			// (input, expected)
			(0u8, ProxyType::Any),
			(1, ProxyType::NonTransfer),
			(2, ProxyType::Governance),
			(3, ProxyType::Staking),
			(4, ProxyType::CancelProxy),
			(5, ProxyType::Balances),
			(6, ProxyType::AuthorMapping),
			(7, ProxyType::IdentityJudgement),
		];

		for (input, expected) in test_cases {
			let actual = ProxyType::decode(&mut input.to_le_bytes().as_slice());
			assert_eq!(
				Ok(expected),
				actual,
				"failed decoding ProxyType for value '{}'",
				input
			);
		}
	}

	#[test]
	fn configured_base_extrinsic_weight_is_evm_compatible() {
		let min_ethereum_transaction_weight = WeightPerGas::get() * 21_000;
		let base_extrinsic = <Runtime as frame_system::Config>::BlockWeights::get()
			.get(frame_support::dispatch::DispatchClass::Normal)
			.base_extrinsic;
		assert!(base_extrinsic.ref_time() <= min_ethereum_transaction_weight.ref_time());
	}
}
