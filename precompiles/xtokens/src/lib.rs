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

//! Precompile to xtokens runtime methods via the EVM

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(assert_matches)]

use fp_evm::PrecompileHandle;
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::Get,
};
use pallet_evm::AddressMapping;
use precompile_utils::{data::String, prelude::*};
use sp_core::{H160, U256};
use sp_std::{
	boxed::Box,
	convert::{TryFrom, TryInto},
	marker::PhantomData,
	vec::Vec,
};
use xcm::{
	latest::{AssetId, Fungibility, MultiAsset, MultiAssets, MultiLocation, WeightLimit},
	VersionedMultiAsset, VersionedMultiAssets, VersionedMultiLocation,
};
use xcm_primitives::AccountIdToCurrencyId;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub type XBalanceOf<Runtime> = <Runtime as orml_xtokens::Config>::Balance;
pub type MaxAssetsForTransfer<Runtime> = <Runtime as orml_xtokens::Config>::MaxAssetsForTransfer;

pub type CurrencyIdOf<Runtime> = <Runtime as orml_xtokens::Config>::CurrencyId;

pub struct GetMaxAssets<R>(PhantomData<R>);

impl<R> Get<u32> for GetMaxAssets<R>
where
	R: orml_xtokens::Config,
{
	fn get() -> u32 {
		<R as orml_xtokens::Config>::MaxAssetsForTransfer::get() as u32
	}
}

/// A precompile to wrap the functionality from xtokens
pub struct XtokensPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
#[precompile::test_concrete_types(mock::Runtime)]
impl<Runtime> XtokensPrecompile<Runtime>
where
	Runtime: orml_xtokens::Config + pallet_evm::Config + frame_system::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<orml_xtokens::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	XBalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
	Runtime: AccountIdToCurrencyId<Runtime::AccountId, CurrencyIdOf<Runtime>>,
{
	#[precompile::public("transfer(address,uint256,(uint8,bytes[]),uint64)")]
	fn transfer(
		handle: &mut impl PrecompileHandle,
		currency_address: Address,
		amount: U256,
		destination: MultiLocation,
		weight: u64,
	) -> EvmResult {
		let to_address: H160 = currency_address.into();
		let to_account = Runtime::AddressMapping::into_account_id(to_address);

		// We convert the address into a currency id xtokens understands
		let currency_id: <Runtime as orml_xtokens::Config>::CurrencyId =
			Runtime::account_to_currency_id(to_account)
				.ok_or(revert("cannot convert into currency id"))?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let amount = amount
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").in_field("amount"))?;

		let call = orml_xtokens::Call::<Runtime>::transfer {
			currency_id,
			amount,
			dest: Box::new(VersionedMultiLocation::V1(destination)),
			dest_weight_limit: WeightLimit::Limited(weight),
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("transferWithFee(address,uint256,uint256,(uint8,bytes[]),uint64)")]
	#[precompile::public("transfer_with_fee(address,uint256,uint256,(uint8,bytes[]),uint64)")]
	fn transfer_with_fee(
		handle: &mut impl PrecompileHandle,
		currency_address: Address,
		amount: U256,
		fee: U256,
		destination: MultiLocation,
		weight: u64,
	) -> EvmResult {
		let to_address: H160 = currency_address.into();
		let to_account = Runtime::AddressMapping::into_account_id(to_address);

		// We convert the address into a currency id xtokens understands
		let currency_id: <Runtime as orml_xtokens::Config>::CurrencyId =
			Runtime::account_to_currency_id(to_account).ok_or(
				RevertReason::custom("Cannot convert into currency id").in_field("currencyAddress"),
			)?;

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		// Transferred amount
		let amount = amount
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").in_field("amount"))?;

		// Fee amount
		let fee = fee
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").in_field("fee"))?;

		let call = orml_xtokens::Call::<Runtime>::transfer_with_fee {
			currency_id,
			amount,
			fee,
			dest: Box::new(VersionedMultiLocation::V1(destination)),
			dest_weight_limit: WeightLimit::Limited(weight),
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}

	#[precompile::public("transferMultiasset((uint8,bytes[]),uint256,(uint8,bytes[]),uint64)")]
	#[precompile::public("transfer_multiasset((uint8,bytes[]),uint256,(uint8,bytes[]),uint64)")]
	fn transfer_multiasset(
		handle: &mut impl PrecompileHandle,
		asset: MultiLocation,
		amount: U256,
		destination: MultiLocation,
		weight: u64,
	) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let to_balance = amount
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").in_field("amount"))?;

		let call = orml_xtokens::Call::<Runtime>::transfer_multiasset {
			asset: Box::new(VersionedMultiAsset::V1(MultiAsset {
				id: AssetId::Concrete(asset),
				fun: Fungibility::Fungible(to_balance),
			})),
			dest: Box::new(VersionedMultiLocation::V1(destination)),
			dest_weight_limit: WeightLimit::Limited(weight),
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

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
		asset: MultiLocation,
		amount: U256,
		fee: U256,
		destination: MultiLocation,
		weight: u64,
	) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let amount = amount
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").in_field("amount"))?;
		let fee = fee
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").in_field("fee"))?;

		let call = orml_xtokens::Call::<Runtime>::transfer_multiasset_with_fee {
			asset: Box::new(VersionedMultiAsset::V1(MultiAsset {
				id: AssetId::Concrete(asset.clone()),
				fun: Fungibility::Fungible(amount),
			})),
			fee: Box::new(VersionedMultiAsset::V1(MultiAsset {
				id: AssetId::Concrete(asset),
				fun: Fungibility::Fungible(fee),
			})),
			dest: Box::new(VersionedMultiLocation::V1(destination)),
			dest_weight_limit: WeightLimit::Limited(weight),
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

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
		currencies: BoundedVec<Currency, GetMaxAssets<Runtime>>,
		fee_item: u32,
		destination: MultiLocation,
		weight: u64,
	) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		// Build all currencies
		let currencies: Vec<_> = currencies.into();
		let currencies = currencies
			.into_iter()
			.enumerate()
			.map(|(index, currency)| {
				let address_as_h160: H160 = currency.address.into();
				let amount = currency.amount.try_into().map_err(|_| {
					RevertReason::value_is_too_large("balance type")
						.in_array(index)
						.in_field("currencies")
				})?;

				Ok((
					Runtime::account_to_currency_id(Runtime::AddressMapping::into_account_id(
						address_as_h160,
					))
					.ok_or(
						RevertReason::custom("Cannot convert into currency id")
							.in_array(index)
							.in_field("currencies"),
					)?,
					amount,
				))
			})
			.collect::<EvmResult<_>>()?;

		let call = orml_xtokens::Call::<Runtime>::transfer_multicurrencies {
			currencies,
			fee_item,
			dest: Box::new(VersionedMultiLocation::V1(destination)),
			dest_weight_limit: WeightLimit::Limited(weight),
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

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
		assets: BoundedVec<EvmMultiAsset, GetMaxAssets<Runtime>>,
		fee_item: u32,
		destination: MultiLocation,
		weight: u64,
	) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let assets: Vec<_> = assets.into();
		let multiasset_vec: EvmResult<Vec<MultiAsset>> = assets
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
		let multiassets =
			MultiAssets::from_sorted_and_deduplicated(multiasset_vec?).map_err(|_| {
				RevertReason::custom("Provided assets either not sorted nor deduplicated")
					.in_field("assets")
			})?;

		let call = orml_xtokens::Call::<Runtime>::transfer_multiassets {
			assets: Box::new(VersionedMultiAssets::V1(multiassets)),
			fee_item,
			dest: Box::new(VersionedMultiLocation::V1(destination)),
			dest_weight_limit: WeightLimit::Limited(weight),
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(())
	}
}

// Currency
#[derive(EvmData)]
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

#[derive(EvmData)]
pub struct EvmMultiAsset {
	location: MultiLocation,
	amount: U256,
}

impl From<(MultiLocation, U256)> for EvmMultiAsset {
	fn from(tuple: (MultiLocation, U256)) -> Self {
		EvmMultiAsset {
			location: tuple.0,
			amount: tuple.1,
		}
	}
}
