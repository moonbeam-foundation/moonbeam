// Copyright 2025 Moonbeam Foundation.
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

#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt::Display;
use core::marker::PhantomData;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::OriginTrait;
use moonkit_xcm_primitives::AccountIdAssetIdConversion;
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{MaxEncodedLen, H160, U256};
use sp_runtime::traits::{Dispatchable, StaticLookup};
use xcm::latest::prelude::Location;

pub type ForeignAssetInstance = ();

/// Alias for the Balance type for old foreign assets
pub type BalanceOf<Runtime> = <Runtime as pallet_assets::Config<ForeignAssetInstance>>::Balance;

/// Alias for the Asset Id type for old foreign assets
pub type AssetIdOf<Runtime> = <Runtime as pallet_assets::Config<ForeignAssetInstance>>::AssetId;

pub struct ForeignAssetMigratorPrecompile<Runtime>(PhantomData<Runtime>);

impl<R> Clone for ForeignAssetMigratorPrecompile<R> {
	fn clone(&self) -> Self {
		Self(PhantomData)
	}
}

impl<R> Default for ForeignAssetMigratorPrecompile<R> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<Runtime> ForeignAssetMigratorPrecompile<Runtime> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

#[precompile_utils::precompile]
impl<Runtime> ForeignAssetMigratorPrecompile<Runtime>
where
	Runtime: pallet_asset_manager::Config<AssetId = u128>
		+ pallet_assets::Config<ForeignAssetInstance, AssetId = u128>
		+ pallet_evm::Config
		+ pallet_moonbeam_foreign_assets::Config
		+ frame_system::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_assets::Call<Runtime, ForeignAssetInstance>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256> + solidity::Codec,
	Runtime: AccountIdAssetIdConversion<Runtime::AccountId, AssetIdOf<Runtime>>,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin: OriginTrait,
	AssetIdOf<Runtime>: Display,
	Runtime::AccountId: Into<H160>,
	<Runtime as pallet_asset_manager::Config>::ForeignAssetType: Into<Option<Location>>,
	Runtime::AddressMapping: AddressMapping<Runtime::AccountId>,
{
	#[precompile::public("migrateAccounts(address,uint32)")]
	fn migrate_accounts(
		handle: &mut impl PrecompileHandle,
		asset_address: Address,
		n: u32,
	) -> EvmResult<U256> {
		let new_asset_id = Self::get_new_asset_id_with_same_xcm_location(handle, asset_address)?;

		// Account proof size for N accounts keys
		// Storage item: Account
		// Key1: Blake2_128(16) + AssetId(16)
		// Key2: Blake2_128(16) + AccountId(20)
		handle.record_db_read::<Runtime>(68 * n as usize)?;

		// Get N account ids
		// IMPORTANT: we can't mutate storage while iterating, that's why we collect N accounts ids
		// in a Vec BEFORE processing them!
		let accounts_ids =
			pallet_assets::pallet::Account::<Runtime, ForeignAssetInstance>::iter_key_prefix(
				new_asset_id,
			)
			.take(n as usize)
			.collect::<sp_std::vec::Vec<_>>();

		// Refound gas if there is less than N accounts
		if accounts_ids.len() < n as usize {
			let n_diff = ((n as usize) - accounts_ids.len()) as u64;
			handle.refund_external_cost(
				Some(n_diff * RuntimeHelper::<Runtime>::db_read_gas_cost()),
				Some(n_diff * 68),
			);
		}

		// IMPORTANT: we can't mutate storage while iterating, that's why we collect N accounts ids
		// in a Vec BEFORE processing them!
		let mut counter = 0;
		for account_id in accounts_ids {
			Self::migrate_account_inner(handle, new_asset_id, account_id.into())?;
			counter += 1;
		}

		Ok(U256([counter, 0, 0, 0]))
	}

	#[precompile::public("migrateAccount(address,address)")]
	fn migrate_account(
		handle: &mut impl PrecompileHandle,
		asset_address: Address,
		account: Address,
	) -> EvmResult<()> {
		let new_asset_id = Self::get_new_asset_id_with_same_xcm_location(handle, asset_address)?;

		Self::migrate_account_inner(handle, new_asset_id, account.into())?;

		Ok(())
	}

	fn get_new_asset_id_with_same_xcm_location(
		handle: &mut impl PrecompileHandle,
		asset_address: Address,
	) -> EvmResult<u128> {
		let asset_address: H160 = asset_address.into();

		let asset_address = Runtime::AddressMapping::into_account_id(asset_address);

		// compute asset id from asset address
		let asset_id = match Runtime::account_to_asset_id(asset_address) {
			Some((_, asset_id)) => asset_id,
			None => return Err(revert("invalid asset address")),
		};

		// Storage item: AssetIdType
		// key:  Blake2_128(16) + AssetId(16)
		// value: XCMv3 Location
		handle.record_db_read::<Runtime>(32 + xcm::v3::Location::max_encoded_len())?;

		// Get asset XCM location
		let asset_type = pallet_asset_manager::Pallet::<Runtime>::asset_id_type(&asset_id)
			.ok_or(revert("asset id doesn't exist"))?;
		let xcm_location = asset_type.into().ok_or(revert("invalid XCM Location"))?;

		// Storage item: AssetsByLocation
		// key:  Blake2_128(16) + XCM latest Location
		// value: AssetId(16) + AssetStatus(1)
		handle.record_db_read::<Runtime>(33 + Location::max_encoded_len())?;

		// Get new asset id
		let (new_asset_id, _) =
			pallet_moonbeam_foreign_assets::Pallet::<Runtime>::assets_by_location(&xcm_location)
				.ok_or(revert("new foreign asset doesn't exist"))?;

		Ok(new_asset_id)
	}

	fn migrate_account_inner(
		handle: &mut impl PrecompileHandle,
		new_asset_id: u128,
		account: H160,
	) -> EvmResult<()> {
		let account_id = Runtime::AddressMapping::into_account_id(account);

		// Storage item: Account
		// Key1: Blake2_128(16) + AssetId(16)
		// key2: Blake2_128(16) + AccountId(20)
		// Value: AssetAccount(19 + Extra)
		handle.record_db_read::<Runtime>(
			87 + <Runtime as pallet_assets::Config<ForeignAssetInstance>>::Extra::max_encoded_len(),
		)?;

		// Get old asset balance
		let amount = pallet_assets::Pallet::<Runtime, ForeignAssetInstance>::balance(
			new_asset_id,
			&account_id,
		);

		// Burn account balance on hold foreign asset
		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(pallet_asset_manager::Pallet::<Runtime>::account_id()).into(),
			pallet_assets::Call::<Runtime, ForeignAssetInstance>::burn {
				id: new_asset_id.into(),
				who: Runtime::Lookup::unlookup(account_id.clone()),
				amount,
			},
			0,
		)?;

		// Mint same balance for new asset
		pallet_moonbeam_foreign_assets::Pallet::<Runtime>::mint_into(
			new_asset_id,
			account_id,
			amount.into(),
		)
		.map_err(|_| revert("fail to mint new foreign asset"))?;

		Ok(())
	}
}
