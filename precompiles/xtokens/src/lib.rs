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

//! Precompile to xtokens runtime methods via the EVM

#![cfg_attr(not(feature = "std"), no_std)]

use account::SYSTEM_ACCOUNT_SIZE;
use fp_evm::PrecompileHandle;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{ConstU32, H160, U256};
use sp_runtime::traits::{Convert, Dispatchable};
use sp_std::{boxed::Box, convert::TryInto, marker::PhantomData, vec::Vec};
use sp_weights::Weight;
use xcm::{
	latest::{Asset, AssetId, Assets, Fungibility, Location, WeightLimit},
	VersionedAssets, VersionedLocation,
};
use xcm_primitives::{
	split_location_into_chain_part_and_beneficiary, AccountIdToCurrencyId, DEFAULT_PROOF_SIZE,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub type CurrencyIdOf<Runtime> = <Runtime as pallet_xcm_transactor::Config>::CurrencyId;
pub type CurrencyIdToLocationOf<Runtime> =
	<Runtime as pallet_xcm_transactor::Config>::CurrencyIdToLocation;

const MAX_ASSETS: u32 = 20;

/// A precompile to wrap the functionality from xtokens
pub struct XtokensPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
#[precompile::test_concrete_types(mock::Runtime)]
impl<Runtime> XtokensPrecompile<Runtime>
where
	Runtime: pallet_evm::Config
		+ pallet_xcm::Config
		+ pallet_xcm_transactor::Config
		+ frame_system::Config,
	<Runtime as frame_system::Config>::RuntimeCall:
		Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime as frame_system::Config>::RuntimeCall: From<pallet_xcm::Call<Runtime>>,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin:
		From<Option<Runtime::AccountId>>,
	Runtime: AccountIdToCurrencyId<Runtime::AccountId, CurrencyIdOf<Runtime>>,
	<Runtime as pallet_evm::Config>::AddressMapping: AddressMapping<Runtime::AccountId>,
{
	#[precompile::public("transfer(address,uint256,(uint8,bytes[]),uint64)")]
	fn transfer(
		handle: &mut impl PrecompileHandle,
		currency_address: Address,
		amount: U256,
		destination: Location,
		weight: u64,
	) -> EvmResult {
		let to_address: H160 = currency_address.into();
		let to_account = Runtime::AddressMapping::into_account_id(to_address);

		// We convert the address into a currency id xtokens understands
		let currency_id: CurrencyIdOf<Runtime> = Runtime::account_to_currency_id(to_account)
			.ok_or(revert("cannot convert into currency id"))?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let amount = amount
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").in_field("amount"))?;

		let dest_weight_limit = if weight == u64::MAX {
			WeightLimit::Unlimited
		} else {
			WeightLimit::Limited(Weight::from_parts(weight, DEFAULT_PROOF_SIZE))
		};

		let asset = Self::currency_to_asset(currency_id, amount).ok_or(
			RevertReason::custom("Cannot convert currency into xcm asset")
				.in_field("currency_address"),
		)?;

		let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(destination)
			.ok_or_else(|| RevertReason::custom("Invalid destination").in_field("destination"))?;

		let call = pallet_xcm::Call::<Runtime>::transfer_assets {
			dest: Box::new(VersionedLocation::from(chain_part)),
			beneficiary: Box::new(VersionedLocation::from(beneficiary)),
			assets: Box::new(VersionedAssets::V4(asset.into())),
			fee_asset_item: 0,
			weight_limit: dest_weight_limit,
		};

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			call,
			SYSTEM_ACCOUNT_SIZE,
		)?;

		Ok(())
	}

	// transfer_with_fee no longer take the fee parameter into account since we start using
	// pallet-xcm. Now, if you want to limit the maximum amount of fees, you'll have to use a
	// different asset from the one you wish to transfer and use transfer_multi* selectors.
	#[precompile::public("transferWithFee(address,uint256,uint256,(uint8,bytes[]),uint64)")]
	#[precompile::public("transfer_with_fee(address,uint256,uint256,(uint8,bytes[]),uint64)")]
	fn transfer_with_fee(
		handle: &mut impl PrecompileHandle,
		currency_address: Address,
		amount: U256,
		_fee: U256,
		destination: Location,
		weight: u64,
	) -> EvmResult {
		let to_address: H160 = currency_address.into();
		let to_account = Runtime::AddressMapping::into_account_id(to_address);

		// We convert the address into a currency id xtokens understands
		let currency_id: CurrencyIdOf<Runtime> = Runtime::account_to_currency_id(to_account)
			.ok_or(
				RevertReason::custom("Cannot convert into currency id").in_field("currencyAddress"),
			)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		// Transferred amount
		let amount = amount
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").in_field("amount"))?;

		let dest_weight_limit = if weight == u64::MAX {
			WeightLimit::Unlimited
		} else {
			WeightLimit::Limited(Weight::from_parts(weight, DEFAULT_PROOF_SIZE))
		};

		let asset = Self::currency_to_asset(currency_id, amount).ok_or(
			RevertReason::custom("Cannot convert currency into xcm asset")
				.in_field("currency_address"),
		)?;

		let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(destination)
			.ok_or_else(|| RevertReason::custom("Invalid destination").in_field("destination"))?;

		let call = pallet_xcm::Call::<Runtime>::transfer_assets {
			dest: Box::new(VersionedLocation::from(chain_part)),
			beneficiary: Box::new(VersionedLocation::from(beneficiary)),
			assets: Box::new(VersionedAssets::V4(asset.into())),
			fee_asset_item: 0,
			weight_limit: dest_weight_limit,
		};

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			call,
			SYSTEM_ACCOUNT_SIZE,
		)?;

		Ok(())
	}

	#[precompile::public("transferMultiasset((uint8,bytes[]),uint256,(uint8,bytes[]),uint64)")]
	#[precompile::public("transfer_multiasset((uint8,bytes[]),uint256,(uint8,bytes[]),uint64)")]
	fn transfer_multiasset(
		handle: &mut impl PrecompileHandle,
		asset: Location,
		amount: U256,
		destination: Location,
		weight: u64,
	) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let to_balance = amount
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").in_field("amount"))?;

		let dest_weight_limit = if weight == u64::MAX {
			WeightLimit::Unlimited
		} else {
			WeightLimit::Limited(Weight::from_parts(weight, DEFAULT_PROOF_SIZE))
		};

		let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(destination)
			.ok_or_else(|| RevertReason::custom("Invalid destination").in_field("destination"))?;

		let call = pallet_xcm::Call::<Runtime>::transfer_assets {
			dest: Box::new(VersionedLocation::from(chain_part)),
			beneficiary: Box::new(VersionedLocation::from(beneficiary)),
			assets: Box::new(VersionedAssets::V4(
				Asset {
					id: AssetId(asset),
					fun: Fungibility::Fungible(to_balance),
				}
				.into(),
			)),
			fee_asset_item: 0,
			weight_limit: dest_weight_limit,
		};

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			call,
			SYSTEM_ACCOUNT_SIZE,
		)?;

		Ok(())
	}

	#[precompile::public(
		"transferMultiassetWithFee((uint8,bytes[]),uint256,uint256,(uint8,bytes[]),uint64)"
	)]
	#[precompile::public(
		"transfer_multiasset_with_fee((uint8,bytes[]),uint256,uint256,(uint8,bytes[]),uint64)"
	)]
	fn transfer_multiasset_with_fee(
		handle: &mut impl PrecompileHandle,
		asset: Location,
		amount: U256,
		_fee: U256,
		destination: Location,
		weight: u64,
	) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let amount = amount
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").in_field("amount"))?;

		let dest_weight_limit = if weight == u64::MAX {
			WeightLimit::Unlimited
		} else {
			WeightLimit::Limited(Weight::from_parts(weight, DEFAULT_PROOF_SIZE))
		};

		let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(destination)
			.ok_or_else(|| RevertReason::custom("Invalid destination").in_field("destination"))?;

		let call = pallet_xcm::Call::<Runtime>::transfer_assets {
			dest: Box::new(VersionedLocation::from(chain_part)),
			beneficiary: Box::new(VersionedLocation::from(beneficiary)),
			assets: Box::new(VersionedAssets::V4(
				Asset {
					id: AssetId(asset.clone()),
					fun: Fungibility::Fungible(amount),
				}
				.into(),
			)),
			fee_asset_item: 0,
			weight_limit: dest_weight_limit,
		};

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			call,
			SYSTEM_ACCOUNT_SIZE,
		)?;

		Ok(())
	}

	#[precompile::public(
		"transferMultiCurrencies((address,uint256)[],uint32,(uint8,bytes[]),uint64)"
	)]
	#[precompile::public(
		"transfer_multi_currencies((address,uint256)[],uint32,(uint8,bytes[]),uint64)"
	)]
	fn transfer_multi_currencies(
		handle: &mut impl PrecompileHandle,
		currencies: BoundedVec<Currency, ConstU32<MAX_ASSETS>>,
		fee_item: u32,
		destination: Location,
		weight: u64,
	) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		// Build all currencies
		let currencies: Vec<_> = currencies.into();
		let assets = currencies
			.into_iter()
			.enumerate()
			.map(|(index, currency)| {
				let address_as_h160: H160 = currency.address.into();
				let amount = currency.amount.try_into().map_err(|_| {
					RevertReason::value_is_too_large("balance type")
						.in_array(index)
						.in_field("currencies")
				})?;

				let currency_id = Runtime::account_to_currency_id(
					Runtime::AddressMapping::into_account_id(address_as_h160),
				)
				.ok_or(
					RevertReason::custom("Cannot convert into currency id")
						.in_array(index)
						.in_field("currencies"),
				)?;

				Self::currency_to_asset(currency_id, amount).ok_or(
					RevertReason::custom("Cannot convert currency into xcm asset")
						.in_array(index)
						.in_field("currencies")
						.into(),
				)
			})
			.collect::<EvmResult<Vec<_>>>()?;

		let dest_weight_limit = if weight == u64::MAX {
			WeightLimit::Unlimited
		} else {
			WeightLimit::Limited(Weight::from_parts(weight, DEFAULT_PROOF_SIZE))
		};

		let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(destination)
			.ok_or_else(|| RevertReason::custom("Invalid destination").in_field("destination"))?;

		let call = pallet_xcm::Call::<Runtime>::transfer_assets {
			dest: Box::new(VersionedLocation::from(chain_part)),
			beneficiary: Box::new(VersionedLocation::from(beneficiary)),
			assets: Box::new(VersionedAssets::V4(assets.into())),
			fee_asset_item: fee_item,
			weight_limit: dest_weight_limit,
		};

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			call,
			SYSTEM_ACCOUNT_SIZE,
		)?;

		Ok(())
	}

	#[precompile::public(
		"transferMultiAssets(((uint8,bytes[]),uint256)[],uint32,(uint8,bytes[]),uint64)"
	)]
	#[precompile::public(
		"transfer_multi_assets(((uint8,bytes[]),uint256)[],uint32,(uint8,bytes[]),uint64)"
	)]
	fn transfer_multi_assets(
		handle: &mut impl PrecompileHandle,
		assets: BoundedVec<EvmAsset, ConstU32<MAX_ASSETS>>,
		fee_item: u32,
		destination: Location,
		weight: u64,
	) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let assets: Vec<_> = assets.into();
		let multiasset_vec: EvmResult<Vec<Asset>> = assets
			.into_iter()
			.enumerate()
			.map(|(index, evm_multiasset)| {
				let to_balance: u128 = evm_multiasset.amount.try_into().map_err(|_| {
					RevertReason::value_is_too_large("balance type")
						.in_array(index)
						.in_field("assets")
				})?;
				Ok((evm_multiasset.location, to_balance).into())
			})
			.collect();

		// Since multiassets sorts them, we need to check whether the index is still correct,
		// and error otherwise as there is not much we can do other than that
		let assets = Assets::from_sorted_and_deduplicated(multiasset_vec?).map_err(|_| {
			RevertReason::custom("Provided assets either not sorted nor deduplicated")
				.in_field("assets")
		})?;

		let dest_weight_limit = if weight == u64::MAX {
			WeightLimit::Unlimited
		} else {
			WeightLimit::Limited(Weight::from_parts(weight, DEFAULT_PROOF_SIZE))
		};

		let (chain_part, beneficiary) = split_location_into_chain_part_and_beneficiary(destination)
			.ok_or_else(|| RevertReason::custom("Invalid destination").in_field("destination"))?;

		let call = pallet_xcm::Call::<Runtime>::transfer_assets {
			dest: Box::new(VersionedLocation::from(chain_part)),
			beneficiary: Box::new(VersionedLocation::from(beneficiary)),
			assets: Box::new(VersionedAssets::V4(assets)),
			fee_asset_item: fee_item,
			weight_limit: dest_weight_limit,
		};

		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(origin).into(),
			call,
			SYSTEM_ACCOUNT_SIZE,
		)?;

		Ok(())
	}

	fn currency_to_asset(currency_id: CurrencyIdOf<Runtime>, amount: u128) -> Option<Asset> {
		Some(Asset {
			fun: Fungibility::Fungible(amount),
			id: AssetId(<CurrencyIdToLocationOf<Runtime>>::convert(currency_id)?),
		})
	}
}

// Currency
#[derive(solidity::Codec)]
pub struct Currency {
	address: Address,
	amount: U256,
}

impl From<(Address, U256)> for Currency {
	fn from(tuple: (Address, U256)) -> Self {
		Currency {
			address: tuple.0,
			amount: tuple.1,
		}
	}
}

#[derive(solidity::Codec)]
pub struct EvmAsset {
	location: Location,
	amount: U256,
}

impl From<(Location, U256)> for EvmAsset {
	fn from(tuple: (Location, U256)) -> Self {
		EvmAsset {
			location: tuple.0,
			amount: tuple.1,
		}
	}
}
