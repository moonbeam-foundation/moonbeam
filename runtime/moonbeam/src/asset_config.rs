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

//! Asset configuration for Moonbase.
//!

use super::{
	currency, governance, xcm_config, AccountId, AssetId, AssetManager, Assets, Balance, Balances,
	CouncilInstance, LocalAssets, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin,
	FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX, LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX,
};

use frame_support::{
	dispatch::GetDispatchInfo,
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU128, ConstU32, EitherOfDiverse},
	weights::Weight,
};
use moonbeam_runtime_common::weights as moonbeam_weights;
use pallet_evm_precompileset_assets_erc20::AccountIdAssetIdConversion;
use sp_runtime::traits::Hash as THash;

use frame_system::{EnsureNever, EnsureRoot};
use sp_core::{H160, H256};

use parity_scale_codec::{Compact, Decode, Encode};
use scale_info::TypeInfo;

use sp_std::{
	convert::{From, Into},
	prelude::*,
};

// Number of items that can be destroyed with our configured max extrinsic proof size.
// x = (a - b) / c where:
// 		a: maxExtrinsic proof size
// 		b: base proof size for destroy_accounts in pallet_assets weights
// 		c: proof size for each item
// 		656.87 = (3_407_872 - 8232) / 5180
const REMOVE_ITEMS_LIMIT: u32 = 656;

// Not to disrupt the previous asset instance, we assign () to Foreign
pub type ForeignAssetInstance = ();
pub type LocalAssetInstance = pallet_assets::Instance1;

// For foreign assets, these parameters dont matter much
// as this will only be called by root with the forced arguments
// No deposit is substracted with those methods
// For local assets, they do matter. We use similar parameters
// to those in statemine (except for approval)
parameter_types! {
	pub const AssetDeposit: Balance = 100 * currency::GLMR * currency::SUPPLY_FACTOR;
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = currency::deposit(1,68);
	pub const MetadataDepositPerByte: Balance = currency::deposit(0, 1);
}

/// We allow root and Chain council to execute privileged asset operations.
pub type AssetsForceOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EitherOfDiverse<
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilInstance, 1, 2>,
		governance::custom_origins::GeneralAdmin,
	>,
>;

// Required for runtime benchmarks
pallet_assets::runtime_benchmarks_enabled! {
	pub struct BenchmarkHelper;
	impl<AssetIdParameter> pallet_assets::BenchmarkHelper<AssetIdParameter> for BenchmarkHelper
	where
		AssetIdParameter: From<u128>,
	{
		fn create_asset_id_parameter(id: u32) -> AssetIdParameter {
			(id as u128).into()
		}
	}
}

// Foreign assets
impl pallet_assets::Config<ForeignAssetInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
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
	type AssetAccountDeposit = ConstU128<{ currency::deposit(1, 18) }>;
	type WeightInfo = moonbeam_weights::pallet_assets::WeightInfo<Runtime>;
	type RemoveItemsLimit = ConstU32<{ REMOVE_ITEMS_LIMIT }>;
	type AssetIdParameter = Compact<AssetId>;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureNever<AccountId>>;
	type CallbackHandle = ();
	pallet_assets::runtime_benchmarks_enabled! {
		type BenchmarkHelper = BenchmarkHelper;
	}
}

// Local assets
impl pallet_assets::Config<LocalAssetInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type Currency = Balances;
	type ForceOrigin = EnsureNever<AccountId>;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type AssetAccountDeposit = ConstU128<{ currency::deposit(1, 18) }>;
	type WeightInfo = moonbeam_weights::pallet_assets::WeightInfo<Runtime>;
	type RemoveItemsLimit = ConstU32<{ REMOVE_ITEMS_LIMIT }>;
	type AssetIdParameter = Compact<AssetId>;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureNever<AccountId>>;
	type CallbackHandle = ();
	pallet_assets::runtime_benchmarks_enabled! {
		type BenchmarkHelper = BenchmarkHelper;
	}
}

// We instruct how to register the Assets
// In this case, we tell it to Create an Asset in pallet-assets
pub struct AssetRegistrar;
use frame_support::{pallet_prelude::DispatchResult, transactional};

impl pallet_asset_manager::AssetRegistrar<Runtime> for AssetRegistrar {
	#[transactional]
	fn create_foreign_asset(
		asset: AssetId,
		min_balance: Balance,
		metadata: AssetRegistrarMetadata,
		is_sufficient: bool,
	) -> DispatchResult {
		Assets::force_create(
			RuntimeOrigin::root(),
			asset.into(),
			AssetManager::account_id(),
			is_sufficient,
			min_balance,
		)?;

		// TODO uncomment when we feel comfortable
		/*
		// The asset has been created. Let's put the revert code in the precompile address
		let precompile_address = Runtime::asset_id_to_account(ASSET_PRECOMPILE_ADDRESS_PREFIX, asset);
		pallet_evm::AccountCodes::<Runtime>::insert(
			precompile_address,
			vec![0x60, 0x00, 0x60, 0x00, 0xfd],
		);*/

		// Lastly, the metadata
		Assets::force_set_metadata(
			RuntimeOrigin::root(),
			asset.into(),
			metadata.name,
			metadata.symbol,
			metadata.decimals,
			metadata.is_frozen,
		)
	}

	#[transactional]
	fn create_local_asset(
		asset: AssetId,
		_creator: AccountId,
		min_balance: Balance,
		is_sufficient: bool,
		owner: AccountId,
	) -> DispatchResult {
		// We create with root, because we need to decide whether we want to create the asset
		// as sufficient. Take into account this does not hold any reserved amount
		// in pallet-assets
		LocalAssets::force_create(
			RuntimeOrigin::root(),
			asset.into(),
			owner,
			is_sufficient,
			min_balance,
		)?;

		// No metadata needs to be set, as this can be set through regular calls

		// The asset has been created. Let's put the revert code in the precompile address
		let precompile_address: H160 =
			Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, asset).into();
		pallet_evm::AccountCodes::<Runtime>::insert(
			precompile_address,
			vec![0x60, 0x00, 0x60, 0x00, 0xfd],
		);
		Ok(())
	}

	#[transactional]
	fn destroy_foreign_asset(asset: AssetId) -> DispatchResult {
		// Mark the asset as destroying
		Assets::start_destroy(RuntimeOrigin::root(), asset.into())?;

		// We remove the EVM revert code
		// This does not panick even if there is no code in the address
		let precompile_address: H160 =
			Runtime::asset_id_to_account(FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX, asset).into();
		pallet_evm::AccountCodes::<Runtime>::remove(precompile_address);
		Ok(())
	}

	#[transactional]
	fn destroy_local_asset(asset: AssetId) -> DispatchResult {
		// Mark the asset as destroying
		LocalAssets::start_destroy(RuntimeOrigin::root(), asset.into())?;

		// We remove the EVM revert code
		// This does not panick even if there is no code in the address
		let precompile_address: H160 =
			Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, asset).into();
		pallet_evm::AccountCodes::<Runtime>::remove(precompile_address);
		Ok(())
	}

	fn destroy_asset_dispatch_info_weight(asset: AssetId) -> Weight {
		// For us both of them (Foreign and Local) have the same annotated weight for a given
		// witness
		// We need to take the dispatch info from the destroy call, which is already annotated in
		// the assets pallet
		// Additionally, we need to add a DB write for removing the precompile revert code in the
		// EVM

		// This is the dispatch info of destroy
		let call_weight = RuntimeCall::Assets(
			pallet_assets::Call::<Runtime, ForeignAssetInstance>::start_destroy {
				id: asset.into(),
			},
		)
		.get_dispatch_info()
		.weight;

		// This is the db write
		call_weight.saturating_add(<Runtime as frame_system::Config>::DbWeight::get().writes(1))
	}
}

pub struct LocalAssetIdCreator;
impl pallet_asset_manager::LocalAssetIdCreator<Runtime> for LocalAssetIdCreator {
	fn create_asset_id_from_metadata(local_asset_counter: u128) -> AssetId {
		// Our means of converting a local asset counter to an assetId
		// We basically hash (local asset counter)
		let mut result: [u8; 16] = [0u8; 16];
		let hash: H256 =
			local_asset_counter.using_encoded(<Runtime as frame_system::Config>::Hashing::hash);
		result.copy_from_slice(&hash.as_fixed_bytes()[0..16]);
		u128::from_le_bytes(result)
	}
}

#[derive(Clone, Default, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub struct AssetRegistrarMetadata {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
	pub is_frozen: bool,
}

pub type ForeignAssetModifierOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EitherOfDiverse<
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilInstance, 1, 2>,
		governance::custom_origins::GeneralAdmin,
	>,
>;
pub type LocalAssetModifierOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EitherOfDiverse<
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilInstance, 1, 2>,
		governance::custom_origins::GeneralAdmin,
	>,
>;

impl pallet_asset_manager::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type AssetRegistrarMetadata = AssetRegistrarMetadata;
	type ForeignAssetType = xcm_config::AssetType;
	type AssetRegistrar = AssetRegistrar;
	type ForeignAssetModifierOrigin = ForeignAssetModifierOrigin;
	type LocalAssetModifierOrigin = LocalAssetModifierOrigin;
	type LocalAssetIdCreator = LocalAssetIdCreator;
	type Currency = Balances;
	type LocalAssetDeposit = AssetDeposit;
	type WeightInfo = moonbeam_weights::pallet_asset_manager::WeightInfo<Runtime>;
}

// Instruct how to go from an H160 to an AssetID
// We just take the lowest 128 bits
impl AccountIdAssetIdConversion<AccountId, AssetId> for Runtime {
	/// The way to convert an account to assetId is by ensuring that the prefix is 0XFFFFFFFF
	/// and by taking the lowest 128 bits as the assetId
	fn account_to_asset_id(account: AccountId) -> Option<(Vec<u8>, AssetId)> {
		let h160_account: H160 = account.into();
		let mut data = [0u8; 16];
		let (prefix_part, id_part) = h160_account.as_fixed_bytes().split_at(4);
		if prefix_part == FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX
			|| prefix_part == LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX
		{
			data.copy_from_slice(id_part);
			let asset_id: AssetId = u128::from_be_bytes(data).into();
			Some((prefix_part.to_vec(), asset_id))
		} else {
			None
		}
	}

	// The opposite conversion
	fn asset_id_to_account(prefix: &[u8], asset_id: AssetId) -> AccountId {
		let mut data = [0u8; 20];
		data[0..4].copy_from_slice(prefix);
		data[4..20].copy_from_slice(&asset_id.to_be_bytes());
		AccountId::from(data)
	}
}
