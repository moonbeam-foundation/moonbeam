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

use crate::OpenTechCommitteeInstance;

use super::{
	currency, governance, xcm_config, AccountId, AssetId, Assets, Balance, Balances, Runtime,
	RuntimeCall, RuntimeEvent, RuntimeOrigin, FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
};

use super::moonbase_weights;
use moonkit_xcm_primitives::AccountIdAssetIdConversion;

use frame_support::{
	dispatch::GetDispatchInfo,
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU128, ConstU32, EitherOfDiverse},
	weights::Weight,
};

use frame_system::{EnsureNever, EnsureRoot};
use parity_scale_codec::{Compact, Decode, Encode};
use scale_info::TypeInfo;
use sp_core::H160;

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

// For foreign assets, these parameters dont matter much
// as this will only be called by root with the forced arguments
// No deposit is substracted with those methods
parameter_types! {
	pub const AssetDeposit: Balance = 100 * currency::UNIT * currency::SUPPLY_FACTOR;
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = currency::deposit(1,68);
	pub const MetadataDepositPerByte: Balance = currency::deposit(0, 1);
}

/// We allow Root and General Admin to execute privileged asset operations.
pub type AssetsForceOrigin =
	EitherOfDiverse<EnsureRoot<AccountId>, governance::custom_origins::GeneralAdmin>;

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
	type WeightInfo = moonbase_weights::pallet_assets::WeightInfo<Runtime>;
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
			crate::AssetManager::account_id(),
			is_sufficient,
			min_balance,
		)?;

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
	fn destroy_foreign_asset(asset: AssetId) -> DispatchResult {
		// Mark the asset as destroying
		Assets::start_destroy(RuntimeOrigin::root(), asset.into())
	}

	fn destroy_asset_dispatch_info_weight(asset: AssetId) -> Weight {
		// For us both of them (Foreign and Local) have the same annotated weight for a given
		// witness
		// We need to take the dispatch info from the destroy call, which is already annotated in
		// the assets pallet

		// This is the dispatch info of destroy
		RuntimeCall::Assets(
			pallet_assets::Call::<Runtime, ForeignAssetInstance>::start_destroy {
				id: asset.into(),
			},
		)
		.get_dispatch_info()
		.weight
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
		pallet_collective::EnsureProportionMoreThan<AccountId, OpenTechCommitteeInstance, 5, 9>,
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
	type WeightInfo = moonbase_weights::pallet_asset_manager::WeightInfo<Runtime>;
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
		if prefix_part == FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX {
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
