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

//! Testing utilities.

use super::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{construct_runtime, parameter_types, traits::Everything, weights::Weight};

use fp_evm::PrecompileSet;
use frame_system::EnsureRoot;
use pallet_evm::{AddressMapping, EnsureAddressNever, EnsureAddressRoot};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::{H160, H256};
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};

pub type AccountId = Account;
pub type AssetId = u128;
pub type Balance = u128;
pub type BlockNumber = u32;
pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
pub type Block = frame_system::mocking::MockBlock<Runtime>;

/// The foreign asset precompile address prefix. Addresses that match against this prefix will
/// be routed to Erc20AssetsPrecompileSet being marked as foreign
pub const FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[255u8; 4];

/// The local asset precompile address prefix. Addresses that match against this prefix will
/// be routed to Erc20AssetsPrecompileSet being marked as local
pub const LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[255u8, 255u8, 255u8, 254u8];

/// To test EIP2612 permits we need to have cryptographic accounts.
pub const ALICE_PUBLIC_KEY: [u8; 20] =
	hex_literal::hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac");

/// To test EIP2612 permits we need to have cryptographic accounts.
pub const ALICE_SECRET_KEY: [u8; 32] =
	hex_literal::hex!("5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133");

/// A simple account type.
#[derive(
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Clone,
	Encode,
	Decode,
	Debug,
	MaxEncodedLen,
	Serialize,
	Deserialize,
	derive_more::Display,
	TypeInfo,
)]
pub enum Account {
	Alice,
	Bob,
	Charlie,
	Bogus,
	ForeignAssetId(AssetId),
	LocalAssetId(AssetId),
	Zero,
}

impl Default for Account {
	fn default() -> Self {
		Self::Bogus
	}
}

impl AddressMapping<Account> for Account {
	fn into_account_id(h160_account: H160) -> Account {
		match h160_account {
			a if a == H160::from(&ALICE_PUBLIC_KEY) => Self::Alice,
			a if a == H160::repeat_byte(0xBB) => Self::Bob,
			a if a == H160::repeat_byte(0xCC) => Self::Charlie,
			a if a == H160::repeat_byte(0x00) => Self::Zero,
			_ => {
				let mut data = [0u8; 16];
				let (prefix_part, id_part) = h160_account.as_fixed_bytes().split_at(4);
				if prefix_part == &[255u8; 4] {
					data.copy_from_slice(id_part);

					return Self::ForeignAssetId(u128::from_be_bytes(data));
				} else if prefix_part == &[255u8, 255u8, 255u8, 254u8] {
					data.copy_from_slice(id_part);

					return Self::LocalAssetId(u128::from_be_bytes(data));
				}
				Self::Bogus
			}
		}
	}
}

// Implement the trait, where we convert AccountId to AssetID
impl AccountIdAssetIdConversion<AccountId, AssetId> for Runtime {
	/// The way to convert an account to assetId is by ensuring that the prefix is 0XFFFFFFFF
	/// and by taking the lowest 128 bits as the assetId
	fn account_to_asset_id(account: AccountId) -> Option<(Vec<u8>, AssetId)> {
		match account {
			Account::ForeignAssetId(asset_id) => {
				Some((FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX.to_vec(), asset_id))
			}
			Account::LocalAssetId(asset_id) => {
				Some((LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX.to_vec(), asset_id))
			}
			_ => None,
		}
	}

	// Not used for now
	fn asset_id_to_account(prefix: &[u8], asset_id: AssetId) -> AccountId {
		if prefix == LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX {
			Account::LocalAssetId(asset_id)
		} else {
			Account::ForeignAssetId(asset_id)
		}
	}
}

impl From<Account> for H160 {
	fn from(x: Account) -> H160 {
		match x {
			Account::Alice => H160::from(&ALICE_PUBLIC_KEY),
			Account::Bob => H160::repeat_byte(0xBB),
			Account::Charlie => H160::repeat_byte(0xCC),
			Account::Zero => H160::repeat_byte(0x00),
			Account::ForeignAssetId(asset_id) => {
				let mut data = [0u8; 20];
				let id_as_bytes = asset_id.to_be_bytes();
				data[0..4].copy_from_slice(&[255u8; 4]);
				data[4..20].copy_from_slice(&id_as_bytes);
				H160::from_slice(&data)
			}
			Account::LocalAssetId(asset_id) => {
				let mut data = [0u8; 20];
				let id_as_bytes = asset_id.to_be_bytes();
				data[0..4].copy_from_slice(&[255u8, 255u8, 255u8, 254u8]);
				data[4..20].copy_from_slice(&id_as_bytes);
				H160::from_slice(&data)
			}
			Account::Bogus => Default::default(),
		}
	}
}

impl From<Account> for H256 {
	fn from(x: Account) -> H256 {
		let x: H160 = x.into();
		x.into()
	}
}

parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Runtime {
	type BaseCallFilter = Everything;
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 0;
}

impl pallet_balances::Config for Runtime {
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type MaxLocks = ();
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub const PrecompilesValue: Precompiles<Runtime> = Precompiles(PhantomData);
	pub WeightPerGas: Weight = Weight::from_ref_time(1);
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type CallOrigin = EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = EnsureAddressNever<AccountId>;
	type AddressMapping = AccountId;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type PrecompilesType = Precompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = BlockGasLimit;
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
}

type ForeignAssetInstance = pallet_assets::Instance1;
type LocalAssetInstance = pallet_assets::Instance2;

// These parameters dont matter much as this will only be called by root with the forced arguments
// No deposit is substracted with those methods
parameter_types! {
	pub const AssetDeposit: Balance = 0;
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 0;
	pub const MetadataDepositPerByte: Balance = 0;
	pub const AssetAccountDeposit: Balance = 0;
}

impl pallet_assets::Config<ForeignAssetInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type AssetAccountDeposit = AssetAccountDeposit;
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
}

impl pallet_assets::Config<LocalAssetInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type AssetAccountDeposit = AssetAccountDeposit;
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
}

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		ForeignAssets: pallet_assets::<Instance1>::{Pallet, Call, Storage, Event<T>},
		Evm: pallet_evm::{Pallet, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		LocalAssets: pallet_assets::<Instance2>::{Pallet, Call, Storage, Event<T>}
	}
);

pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder { balances: vec![] }
	}
}

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

#[derive(Default)]
pub struct Precompiles<R>(PhantomData<R>);

impl<R> PrecompileSet for Precompiles<R>
where
	Erc20AssetsPrecompileSet<R, IsForeign, pallet_assets::Instance1>: PrecompileSet,
	Erc20AssetsPrecompileSet<R, IsLocal, pallet_assets::Instance2>: PrecompileSet,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<EvmResult<PrecompileOutput>> {
		match handle.code_address() {
			// If the address matches asset prefix, the we route through the foreign  asset precompile set
			a if &a.to_fixed_bytes()[0..4] == LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX => {
				Erc20AssetsPrecompileSet::<R, IsLocal, pallet_assets::Instance2>::new()
					.execute(handle)
			}
			// If the address matches asset prefix, the we route through the local asset precompile set
			a if &a.to_fixed_bytes()[0..4] == FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX => {
				Erc20AssetsPrecompileSet::<R, IsForeign, pallet_assets::Instance1>::new()
					.execute(handle)
			}
			_ => None,
		}
	}

	fn is_precompile(&self, address: H160) -> bool {
		Erc20AssetsPrecompileSet::<R, IsForeign, pallet_assets::Instance1>::new()
			.is_precompile(address)
	}
}

pub type LocalPCall = Erc20AssetsPrecompileSetCall<Runtime, IsLocal, pallet_assets::Instance2>;
pub type ForeignPCall = Erc20AssetsPrecompileSetCall<Runtime, IsLocal, pallet_assets::Instance1>;
