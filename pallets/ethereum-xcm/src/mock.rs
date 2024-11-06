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

//! Test utilities

use ethereum::{TransactionAction, TransactionSignature};
use frame_support::{
	parameter_types,
	traits::{ConstU32, FindAuthor, InstanceFilter},
	weights::Weight,
	ConsensusEngineId, PalletId,
};
use frame_system::{pallet_prelude::BlockNumberFor, EnsureRoot};
use pallet_evm::{
	AddressMapping, EnsureAddressTruncated, FeeCalculator, FrameSystemAccountProvider,
};
use rlp::RlpStream;
use sp_core::{hashing::keccak_256, H160, H256, U256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, BuildStorage,
};

use super::*;
use pallet_ethereum::{IntermediateStateRoot, PostLogContent};
use sp_runtime::{
	traits::DispatchInfoOf,
	transaction_validity::{TransactionValidity, TransactionValidityError},
};

pub type BlockNumber = BlockNumberFor<Test>;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime! {
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Timestamp: pallet_timestamp,
		EVM: pallet_evm,
		Ethereum: pallet_ethereum,
		EthereumXcm: crate,
		Proxy: pallet_proxy,
	}
}

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(Weight::from_parts(1024, 1));
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeTask = RuntimeTask;
	type Nonce = u64;
	type Block = Block;
	type Hash = H256;
	type RuntimeCall = RuntimeCall;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId32;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

parameter_types! {
	// For weight estimation, we assume that the most locks on an individual account will be 50.
	// This number may need to be adjusted in the future if this assumption no longer holds true.
	pub const MaxLocks: u32 = 50;
	pub const ExistentialDeposit: u64 = 500;
}

impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	type Balance = u64;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type RuntimeHoldReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeFreezeReason = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 6000 / 2;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

pub struct FixedGasPrice;
impl FeeCalculator for FixedGasPrice {
	fn min_gas_price() -> (U256, Weight) {
		(1.into(), Weight::zero())
	}
}

pub struct FindAuthorTruncated;
impl FindAuthor<H160> for FindAuthorTruncated {
	fn find_author<'a, I>(_digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		Some(address_build(0).address)
	}
}

const MAX_POV_SIZE: u64 = 5 * 1024 * 1024;
/// Block storage limit in bytes. Set to 40 KB.
const BLOCK_STORAGE_LIMIT: u64 = 40 * 1024;

parameter_types! {
	pub const TransactionByteFee: u64 = 1;
	pub const ChainId: u64 = 42;
	pub const EVMModuleId: PalletId = PalletId(*b"py/evmpa");
	pub const BlockGasLimit: U256 = U256::MAX;
	pub WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub GasLimitPovSizeRatio: u64 = {
		let block_gas_limit = BlockGasLimit::get().min(u64::MAX.into()).low_u64();
		block_gas_limit.saturating_div(MAX_POV_SIZE)
	};
	pub GasLimitStorageGrowthRatio: u64 = {
		let block_gas_limit = BlockGasLimit::get().min(u64::MAX.into()).low_u64();
		block_gas_limit.saturating_div(BLOCK_STORAGE_LIMIT)
	};
}

pub struct HashedAddressMapping;

impl AddressMapping<AccountId32> for HashedAddressMapping {
	fn into_account_id(address: H160) -> AccountId32 {
		let mut data = [0u8; 32];
		data[0..20].copy_from_slice(&address[..]);
		AccountId32::from(Into::<[u8; 32]>::into(data))
	}
}

impl pallet_evm::Config for Test {
	type FeeCalculator = FixedGasPrice;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressTruncated;
	type WithdrawOrigin = EnsureAddressTruncated;
	type AddressMapping = HashedAddressMapping;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type ChainId = ChainId;
	type BlockGasLimit = BlockGasLimit;
	type OnChargeTransaction = ();
	type FindAuthor = FindAuthorTruncated;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type OnCreate = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type SuicideQuickClearLimit = ConstU32<0>;
	type GasLimitStorageGrowthRatio = GasLimitStorageGrowthRatio;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Test>;
	type AccountProvider = FrameSystemAccountProvider<Test>;
}

parameter_types! {
	pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = IntermediateStateRoot<<Test as frame_system::Config>::Version>;
	type PostLogContent = PostBlockAndTxnHashes;
	type ExtraDataLength = ConstU32<30>;
}

parameter_types! {
	pub ReservedXcmpWeight: Weight = Weight::from_parts(u64::max_value(), 1);
}

#[derive(
	Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, MaxEncodedLen, TypeInfo,
)]
pub enum ProxyType {
	NotAllowed = 0,
	Any = 1,
}

impl pallet_evm_precompile_proxy::EvmProxyCallFilter for ProxyType {}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, _c: &RuntimeCall) -> bool {
		match self {
			ProxyType::NotAllowed => false,
			ProxyType::Any => true,
		}
	}
	fn is_superset(&self, _o: &Self) -> bool {
		false
	}
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::NotAllowed
	}
}

parameter_types! {
	pub const ProxyCost: u64 = 1;
}

impl pallet_proxy::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyCost;
	type ProxyDepositFactor = ProxyCost;
	type MaxProxies = ConstU32<32>;
	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Test>;
	type MaxPending = ConstU32<32>;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = ProxyCost;
	type AnnouncementDepositFactor = ProxyCost;
}

pub struct EthereumXcmEnsureProxy;
impl xcm_primitives::EnsureProxy<AccountId32> for EthereumXcmEnsureProxy {
	fn ensure_ok(delegator: AccountId32, delegatee: AccountId32) -> Result<(), &'static str> {
		let f = |x: &pallet_proxy::ProxyDefinition<AccountId32, ProxyType, BlockNumber>| -> bool {
			x.delegate == delegatee && (x.proxy_type == ProxyType::Any)
		};
		Proxy::proxies(delegator)
			.0
			.into_iter()
			.find(f)
			.map(|_| ())
			.ok_or("proxy error: expected `ProxyType::Any`")
	}
}

impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type InvalidEvmTransactionError = pallet_ethereum::InvalidTransactionWrapper;
	type ValidatedTransaction = pallet_ethereum::ValidatedTransaction<Self>;
	type XcmEthereumOrigin = crate::EnsureXcmEthereumTransaction;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type EnsureProxy = EthereumXcmEnsureProxy;
	type ControllerOrigin = EnsureRoot<AccountId32>;
	type ForceOrigin = EnsureRoot<AccountId32>;
}

impl fp_self_contained::SelfContainedCall for RuntimeCall {
	type SignedInfo = H160;

	fn is_self_contained(&self) -> bool {
		match self {
			RuntimeCall::Ethereum(call) => call.is_self_contained(),
			_ => false,
		}
	}

	fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
		match self {
			RuntimeCall::Ethereum(call) => call.check_self_contained(),
			_ => None,
		}
	}

	fn validate_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<TransactionValidity> {
		match self {
			RuntimeCall::Ethereum(call) => call.validate_self_contained(info, dispatch_info, len),
			_ => None,
		}
	}

	fn pre_dispatch_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<Result<(), TransactionValidityError>> {
		match self {
			RuntimeCall::Ethereum(call) => {
				call.pre_dispatch_self_contained(info, dispatch_info, len)
			}
			_ => None,
		}
	}

	fn apply_self_contained(
		self,
		info: Self::SignedInfo,
	) -> Option<sp_runtime::DispatchResultWithInfo<sp_runtime::traits::PostDispatchInfoOf<Self>>> {
		use sp_runtime::traits::Dispatchable as _;
		match self {
			call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) => {
				Some(call.dispatch(RuntimeOrigin::from(
					pallet_ethereum::RawOrigin::EthereumTransaction(info),
				)))
			}
			_ => None,
		}
	}
}

pub struct AccountInfo {
	pub address: H160,
	pub account_id: AccountId32,
	pub private_key: H256,
}

fn address_build(seed: u8) -> AccountInfo {
	let private_key = H256::from_slice(&[(seed + 1) as u8; 32]);
	let secret_key = libsecp256k1::SecretKey::parse_slice(&private_key[..]).unwrap();
	let public_key = &libsecp256k1::PublicKey::from_secret_key(&secret_key).serialize()[1..65];
	let address = H160::from(H256::from(keccak_256(public_key)));

	let mut data = [0u8; 32];
	data[0..20].copy_from_slice(&address[..]);

	AccountInfo {
		private_key,
		account_id: AccountId32::from(Into::<[u8; 32]>::into(data)),
		address,
	}
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext(accounts_len: usize) -> (Vec<AccountInfo>, sp_io::TestExternalities) {
	// sc_cli::init_logger("");
	let mut ext = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();

	let pairs = (0..accounts_len)
		.map(|i| address_build(i as u8))
		.collect::<Vec<_>>();

	let balances: Vec<_> = (0..accounts_len)
		.map(|i| (pairs[i].account_id.clone(), 10_000_000))
		.collect();

	pallet_balances::GenesisConfig::<Test> { balances }
		.assimilate_storage(&mut ext)
		.unwrap();

	(pairs, ext.into())
}

pub struct LegacyUnsignedTransaction {
	pub nonce: U256,
	pub gas_price: U256,
	pub gas_limit: U256,
	pub action: TransactionAction,
	pub value: U256,
	pub input: Vec<u8>,
}

impl LegacyUnsignedTransaction {
	fn signing_rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(9);
		s.append(&self.nonce);
		s.append(&self.gas_price);
		s.append(&self.gas_limit);
		s.append(&self.action);
		s.append(&self.value);
		s.append(&self.input);
		s.append(&ChainId::get());
		s.append(&0u8);
		s.append(&0u8);
	}

	fn signing_hash(&self) -> H256 {
		let mut stream = RlpStream::new();
		self.signing_rlp_append(&mut stream);
		H256::from(keccak_256(&stream.out()))
	}

	pub fn sign(&self, key: &H256) -> Transaction {
		self.sign_with_chain_id(key, ChainId::get())
	}

	pub fn sign_with_chain_id(&self, key: &H256, chain_id: u64) -> Transaction {
		let hash = self.signing_hash();
		let msg = libsecp256k1::Message::parse(hash.as_fixed_bytes());
		let s = libsecp256k1::sign(
			&msg,
			&libsecp256k1::SecretKey::parse_slice(&key[..]).unwrap(),
		);
		let sig = s.0.serialize();

		let sig = TransactionSignature::new(
			s.1.serialize() as u64 % 2 + chain_id * 2 + 35,
			H256::from_slice(&sig[0..32]),
			H256::from_slice(&sig[32..64]),
		)
		.unwrap();

		Transaction::Legacy(ethereum::LegacyTransaction {
			nonce: self.nonce,
			gas_price: self.gas_price,
			gas_limit: self.gas_limit,
			action: self.action,
			value: self.value,
			input: self.input.clone(),
			signature: sig,
		})
	}
}

pub struct EIP2930UnsignedTransaction {
	pub nonce: U256,
	pub gas_price: U256,
	pub gas_limit: U256,
	pub action: TransactionAction,
	pub value: U256,
	pub input: Vec<u8>,
}

impl EIP2930UnsignedTransaction {
	pub fn sign(&self, secret: &H256, chain_id: Option<u64>) -> Transaction {
		let secret = {
			let mut sk: [u8; 32] = [0u8; 32];
			sk.copy_from_slice(&secret[0..]);
			libsecp256k1::SecretKey::parse(&sk).unwrap()
		};
		let chain_id = chain_id.unwrap_or(ChainId::get());
		let msg = ethereum::EIP2930TransactionMessage {
			chain_id,
			nonce: self.nonce,
			gas_price: self.gas_price,
			gas_limit: self.gas_limit,
			action: self.action,
			value: self.value,
			input: self.input.clone(),
			access_list: vec![],
		};
		let signing_message = libsecp256k1::Message::parse_slice(&msg.hash()[..]).unwrap();

		let (signature, recid) = libsecp256k1::sign(&signing_message, &secret);
		let rs = signature.serialize();
		let r = H256::from_slice(&rs[0..32]);
		let s = H256::from_slice(&rs[32..64]);
		Transaction::EIP2930(ethereum::EIP2930Transaction {
			chain_id: msg.chain_id,
			nonce: msg.nonce,
			gas_price: msg.gas_price,
			gas_limit: msg.gas_limit,
			action: msg.action,
			value: msg.value,
			input: msg.input.clone(),
			access_list: msg.access_list,
			odd_y_parity: recid.serialize() != 0,
			r,
			s,
		})
	}
}

pub struct EIP1559UnsignedTransaction {
	pub nonce: U256,
	pub max_priority_fee_per_gas: U256,
	pub max_fee_per_gas: U256,
	pub gas_limit: U256,
	pub action: TransactionAction,
	pub value: U256,
	pub input: Vec<u8>,
}

impl EIP1559UnsignedTransaction {
	pub fn sign(&self, secret: &H256, chain_id: Option<u64>) -> Transaction {
		let secret = {
			let mut sk: [u8; 32] = [0u8; 32];
			sk.copy_from_slice(&secret[0..]);
			libsecp256k1::SecretKey::parse(&sk).unwrap()
		};
		let chain_id = chain_id.unwrap_or(ChainId::get());
		let msg = ethereum::EIP1559TransactionMessage {
			chain_id,
			nonce: self.nonce,
			max_priority_fee_per_gas: self.max_priority_fee_per_gas,
			max_fee_per_gas: self.max_fee_per_gas,
			gas_limit: self.gas_limit,
			action: self.action,
			value: self.value,
			input: self.input.clone(),
			access_list: vec![],
		};
		let signing_message = libsecp256k1::Message::parse_slice(&msg.hash()[..]).unwrap();

		let (signature, recid) = libsecp256k1::sign(&signing_message, &secret);
		let rs = signature.serialize();
		let r = H256::from_slice(&rs[0..32]);
		let s = H256::from_slice(&rs[32..64]);
		Transaction::EIP1559(ethereum::EIP1559Transaction {
			chain_id: msg.chain_id,
			nonce: msg.nonce,
			max_priority_fee_per_gas: msg.max_priority_fee_per_gas,
			max_fee_per_gas: msg.max_fee_per_gas,
			gas_limit: msg.gas_limit,
			action: msg.action,
			value: msg.value,
			input: msg.input.clone(),
			access_list: msg.access_list,
			odd_y_parity: recid.serialize() != 0,
			r,
			s,
		})
	}
}
