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

//! The Moonbeam Runtime.
//!
//! This runtime powers both the moonbeam standalone node and the moonbeam parachain
//! By default it builds the parachain runtime. To enable the standalone runtime, enable
//! the `standalone` feature.
//!
//! Primary features of this runtime include:
//! * Ethereum compatability
//! * Moonbeam tokenomics
//! * Dual parachain / standalone support

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod runtime;
pub use runtime::*; // TODO : Re-export only main types.

use parity_scale_codec::{Decode, Encode};

use sp_runtime::{
	generic, impl_opaque_keys,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
};
use sp_std::{convert::TryFrom, marker::PhantomData, prelude::*};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

pub use frame_support::{
	construct_runtime, parameter_types,
	traits::{FindAuthor, Get, Randomness},
	weights::{constants::WEIGHT_PER_SECOND, IdentityFee, Weight},
	ConsensusEngineId, StorageValue,
};
use pallet_evm::{EnsureAddressNever, EnsureAddressSame, IdentityAddressMapping};
use pallet_transaction_payment::CurrencyAdapter;

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

// TODO : Update new paths in other crates.
pub use common::*;

/// Type aliases for common types used in the runtime.
pub mod common {
	use super::*;

	/// An index to a block.
	pub type BlockNumber = u32;

	/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
	pub type Signature = account::EthereumSignature;

	/// Some way of identifying an account on the chain. We intentionally make it equivalent
	/// to the public key of our transaction signing scheme.
	pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

	/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
	/// never know...
	pub type AccountIndex = u32;

	/// Balance of an account.
	pub type Balance = u128;

	/// Index of a transaction in the chain.
	pub type Index = u32;

	/// A hash of some data used by the chain.
	pub type Hash = sp_core::H256;

	/// Digest item type.
	pub type DigestItem = generic::DigestItem<Hash>;

	/// Minimum time between blocks. Slot duration is double this.
	pub const MINIMUM_PERIOD: u64 = 3000;

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
		generic::UncheckedExtrinsic<Address, runtime::Call, Signature, SignedExtra>;

	/// Extrinsic type that has already been checked.
	pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, runtime::Call, SignedExtra>;
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datastructures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	pub type Block = generic::Block<common::Header, UncheckedExtrinsic>;

	#[cfg(not(feature = "standalone"))]
	impl_opaque_keys! {
		pub struct SessionKeys {}
	}

	#[cfg(feature = "standalone")]
	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: runtime::VERSION,
		can_author_with: Default::default(),
	}
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

pub mod system {
	use super::*;

	parameter_types! {
		pub const BlockHashCount: common::BlockNumber = 250;
		pub const Version: RuntimeVersion = runtime::VERSION;
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
		type AccountId = common::AccountId;
		/// The aggregated dispatch type that is available for extrinsics.
		type Call = runtime::Call;
		/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
		type Lookup = IdentityLookup<common::AccountId>;
		/// The index type for storing how many extrinsics an account has signed.
		type Index = common::Index;
		/// The index type for blocks.
		type BlockNumber = common::BlockNumber;
		/// The type for hashing blocks and tries.
		type Hash = common::Hash;
		/// The hashing algorithm used.
		type Hashing = BlakeTwo256;
		/// The header type.
		type Header = generic::Header<common::BlockNumber, BlakeTwo256>;
		/// The ubiquitous event type.
		type Event = runtime::Event;
		/// The ubiquitous origin type.
		type Origin = runtime::Origin;
		/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
		type BlockHashCount = BlockHashCount;
		/// Maximum weight of each block.
		/// With a default weight system of 1byte == 1weight, 4mb is ok.
		type BlockWeights = BlockWeights;
		/// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
		type BlockLength = BlockLength;
		/// Runtime version.
		type Version = Version;
		type PalletInfo = runtime::PalletInfo;
		type AccountData = pallet_balances::AccountData<common::Balance>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type DbWeight = ();
		type BaseCallFilter = ();
		type SystemWeightInfo = ();
		/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
		type SS58Prefix = SS58Prefix;
	}
}

pub mod timestamp {
	use super::*;

	parameter_types! {
		// When running in standalone mode, this controls the block time.
		// Slot duration is double the minimum period.
		// https://github.com/paritytech/substrate/blob/e4803bd/frame/aura/src/lib.rs#L197-L199
		// We maintain a six second block time in standalone to imitate parachain-like performance
		// This value is stored in a seperate constant because it is used in our mock timestamp provider
		pub const MinimumPeriod: u64 = common::MINIMUM_PERIOD;
	}

	impl pallet_timestamp::Config for Runtime {
		/// A timestamp: milliseconds since the unix epoch.
		type Moment = u64;
		type OnTimestampSet = ();
		type MinimumPeriod = MinimumPeriod;
		type WeightInfo = ();
	}
}

// TODO : Be more precise about the currency (GLMR ?)
pub mod balances {
	use super::*;

	parameter_types! {
		pub const MaxLocks: u32 = 50;
		pub const ExistentialDeposit: u128 = 0;
	}

	impl pallet_balances::Config for Runtime {
		type MaxLocks = MaxLocks;
		/// The type for recording an account's balance.
		type Balance = common::Balance;
		/// The ubiquitous event type.
		type Event = runtime::Event;
		type DustRemoval = ();
		type ExistentialDeposit = ExistentialDeposit;
		type AccountStore = runtime::System;
		type WeightInfo = ();
	}
}

pub mod tx_payment {
	use super::*;

	parameter_types! {
		pub const TransactionByteFee: common::Balance = 1;
	}

	impl pallet_transaction_payment::Config for Runtime {
		type OnChargeTransaction = CurrencyAdapter<runtime::Balances, ()>;
		type TransactionByteFee = TransactionByteFee;
		type WeightToFee = IdentityFee<common::Balance>;
		type FeeMultiplierUpdate = ();
	}
}

impl pallet_sudo::Config for Runtime {
	type Call = runtime::Call;
	type Event = runtime::Event;
}

impl pallet_ethereum_chain_id::Config for Runtime {}

pub mod evm {
	use super::*;

	/// Current approximation of the gas/s consumption considering
	/// EVM execution over compiled WASM (on 4.4Ghz CPU).
	/// Given the 500ms Weight, from which 75% only are used for transactions,
	/// the total EVM execution gas limit is: GAS_PER_SECOND * 0.500 * 0.75 => 3_000_000.
	pub const GAS_PER_SECOND: u64 = 8_000_000;

	/// Approximate ratio of the amount of Weight per Gas.
	/// u64 works for approximations because Weight is a very small unit compared to gas.
	pub const WEIGHT_PER_GAS: u64 = WEIGHT_PER_SECOND / GAS_PER_SECOND;

	pub struct MoonbeamGasWeightMapping;

	impl pallet_evm::GasWeightMapping for MoonbeamGasWeightMapping {
		fn gas_to_weight(gas: usize) -> Weight {
			Weight::try_from(gas.saturating_mul(WEIGHT_PER_GAS as usize)).unwrap_or(Weight::MAX)
		}
		fn weight_to_gas(weight: Weight) -> usize {
			usize::try_from(weight.wrapping_div(WEIGHT_PER_GAS)).unwrap_or(usize::MAX)
		}
	}

	impl pallet_evm::Config for Runtime {
		type FeeCalculator = ();
		type GasWeightMapping = MoonbeamGasWeightMapping;
		type CallOrigin = EnsureAddressSame;
		type WithdrawOrigin = EnsureAddressNever<common::AccountId>;
		type AddressMapping = IdentityAddressMapping;
		type Currency = runtime::Balances;
		type Event = runtime::Event;
		type Runner = pallet_evm::runner::stack::Runner<Self>;
		type Precompiles = precompiles::MoonbeamPrecompiles<Self>;
		type ChainId = runtime::EthereumChainId;
	}
}

pub struct TransactionConverter;

impl fp_rpc::ConvertTransaction<common::UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(
		&self,
		transaction: pallet_ethereum::Transaction,
	) -> common::UncheckedExtrinsic {
		common::UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<Runtime>::transact(transaction).into(),
		)
	}
}

impl fp_rpc::ConvertTransaction<opaque::UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(
		&self,
		transaction: pallet_ethereum::Transaction,
	) -> opaque::UncheckedExtrinsic {
		let extrinsic = common::UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<Runtime>::transact(transaction).into(),
		);
		let encoded = extrinsic.encode();
		opaque::UncheckedExtrinsic::decode(&mut &encoded[..])
			.expect("Encoded extrinsic is always valid")
	}
}

pub mod ethereum {
	use super::*;

	pub struct EthereumFindAuthor<F>(PhantomData<F>);

	impl pallet_ethereum::Config for Runtime {
		type Event = runtime::Event;
		#[cfg(not(feature = "standalone"))]
		type FindAuthor = EthereumFindAuthor<runtime::PhantomAura>;
		#[cfg(feature = "standalone")]
		type FindAuthor = EthereumFindAuthor<Aura>;
		type StateRoot = pallet_ethereum::IntermediateStateRoot;
	}
}

pub use staking::GLMR;
pub mod staking {
	use super::*;

	// 18 decimals
	pub const GLMR: common::Balance = 1_000_000_000_000_000_000;

	parameter_types! {
		/// Moonbeam starts a new round every 2 minutes (20 * block_time)
		pub const BlocksPerRound: u32 = 20;
		/// Reward payments and validator exit requests are delayed by
		/// 4 minutes (2 * 20 * block_time)
		pub const BondDuration: u32 = 2;
		/// Maximum 8 valid block authors at any given time
		pub const MaxValidators: u32 = 8;
		/// Maximum 10 nominators per validator
		pub const MaxNominatorsPerValidator: usize = 10;
		/// Issue 49 new tokens as rewards to validators every 2 minutes (round)
		pub const IssuancePerRound: u128 = 49 * GLMR;
		/// The maximum percent a validator can take off the top of its rewards is 50%
		pub const MaxFee: Perbill = Perbill::from_percent(50);
		/// Minimum stake required to be reserved to be a validator is 5
		pub const MinValidatorStk: u128 = 100_000 * GLMR;
		/// Minimum stake required to be reserved to be a nominator is 5
		pub const MinNominatorStk: u128 = 5 * GLMR;
	}
	impl stake::Config for Runtime {
		type Event = runtime::Event;
		type Currency = runtime::Balances;
		type BlocksPerRound = BlocksPerRound;
		type BondDuration = BondDuration;
		type MaxValidators = MaxValidators;
		type MaxNominatorsPerValidator = MaxNominatorsPerValidator;
		type IssuancePerRound = IssuancePerRound;
		type MaxFee = MaxFee;
		type MinValidatorStk = MinValidatorStk;
		type MinNominatorStk = MinNominatorStk;
	}
	impl author_inherent::Config for Runtime {
		type Event = runtime::Event;
		type EventHandler = runtime::Stake;
		type CanAuthor = runtime::Stake;
	}
}
