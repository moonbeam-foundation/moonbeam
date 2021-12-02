// Copyright 2019-2021 PureStake Inc.
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

use fp_evm::{Context, ExitSucceed, PrecompileOutput};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{AddressMapping, Precompile};
use precompile_utils::{Address, EvmData, EvmDataReader, EvmResult, Gasometer, RuntimeHelper};

use sp_core::{H160, U256};
use sp_std::boxed::Box;
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker::PhantomData,
};
use xcm::latest::{AssetId, Fungibility, MultiAsset, MultiLocation};
use xcm::{VersionedMultiAsset, VersionedMultiLocation};
use xcm_primitives::AccountIdToCurrencyId;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub type XBalanceOf<Runtime> = <Runtime as orml_xtokens::Config>::Balance;

pub type CurrencyIdOf<Runtime> = <Runtime as orml_xtokens::Config>::CurrencyId;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	Transfer = "transfer(address,uint256,(uint8,bytes[]),uint64)",
	TransferWithFee = "transfer_with_fee(address,uint256,uint256,(uint8,bytes[]),uint64)",
	TransferMultiAsset = "transfer_multiasset((uint8,bytes[]),uint256,(uint8,bytes[]),uint64)",
	TransferMultiAssetWithFee =
		"transfer_multiasset_with_fee((uint8,bytes[]),uint256,uint256,(uint8,bytes[]),uint64)",
}

/// A precompile to wrap the functionality from xtokens
pub struct XtokensWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for XtokensWrapper<Runtime>
where
	Runtime: orml_xtokens::Config + pallet_evm::Config + frame_system::Config,
	Runtime::AccountId: From<H160>,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<orml_xtokens::Call<Runtime>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	XBalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
	Runtime: AccountIdToCurrencyId<Runtime::AccountId, CurrencyIdOf<Runtime>>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
		_is_static: bool,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		let gasometer = &mut gasometer;

		let (mut input, selector) = EvmDataReader::new_with_selector(gasometer, input)?;
		let input = &mut input;

		match selector {
			Action::Transfer => Self::transfer(input, gasometer, context),
			Action::TransferWithFee => Self::transfer_with_fee(input, gasometer, context),
			Action::TransferMultiAsset => Self::transfer_multiasset(input, gasometer, context),
			Action::TransferMultiAssetWithFee => {
				Self::transfer_multiasset_with_fee(input, gasometer, context)
			}
		}
	}
}

impl<Runtime> XtokensWrapper<Runtime>
where
	Runtime: orml_xtokens::Config + pallet_evm::Config + frame_system::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<orml_xtokens::Call<Runtime>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	XBalanceOf<Runtime>: TryFrom<U256> + Into<U256> + EvmData,
	Runtime: AccountIdToCurrencyId<Runtime::AccountId, CurrencyIdOf<Runtime>>,
{
	fn transfer(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Bound check
		input.expect_arguments(gasometer, 2)?;
		let to_address: H160 = input.read::<Address>(gasometer)?.into();
		let amount: U256 = input.read(gasometer)?;

		// We use the MultiLocation, which we have instructed how to read
		// In the end we are using the encoding
		let destination: MultiLocation = input.read::<MultiLocation>(gasometer)?;

		// Bound check
		input.expect_arguments(gasometer, 1)?;
		let dest_weight: u64 = input.read::<u64>(gasometer)?;

		let to_account = Runtime::AddressMapping::into_account_id(to_address);
		// We convert the address into a currency id xtokens understands
		let currency_id: <Runtime as orml_xtokens::Config>::CurrencyId =
			Runtime::account_to_currency_id(to_account)
				.ok_or(gasometer.revert("cannot convert into currency id"))?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let amount = amount
			.try_into()
			.map_err(|_| gasometer.revert("Amount is too large for provided balance type"))?;

		let call = orml_xtokens::Call::<Runtime>::transfer {
			currency_id,
			amount,
			dest: Box::new(VersionedMultiLocation::V1(destination)),
			dest_weight,
		};

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn transfer_with_fee(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		input.expect_arguments(gasometer, 5)?;

		let to_address: H160 = input.read::<Address>(gasometer)?.into();
		let amount: U256 = input.read(gasometer)?;
		let fee: U256 = input.read(gasometer)?;

		// We use the MultiLocation, which we have instructed how to read
		// In the end we are using the encoding
		let destination: MultiLocation = input.read::<MultiLocation>(gasometer)?;

		let dest_weight: u64 = input.read::<u64>(gasometer)?;

		let to_account = Runtime::AddressMapping::into_account_id(to_address);
		// We convert the address into a currency id xtokens understands
		let currency_id: <Runtime as orml_xtokens::Config>::CurrencyId =
			Runtime::account_to_currency_id(to_account)
				.ok_or(gasometer.revert("cannot convert into currency id"))?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		// Transferred amount
		let amount = amount
			.try_into()
			.map_err(|_| gasometer.revert("Amount is too large for provided balance type"))?;

		// Fee amount
		let fee = fee
			.try_into()
			.map_err(|_| gasometer.revert("Amount is too large for provided balance type"))?;

		let call = orml_xtokens::Call::<Runtime>::transfer_with_fee {
			currency_id,
			amount,
			fee,
			dest: Box::new(VersionedMultiLocation::V1(destination)),
			dest_weight,
		};

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn transfer_multiasset(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// asset is defined as a multiLocation. For now we are assuming these are concrete
		// fungible assets
		let asset_multilocation: MultiLocation = input.read::<MultiLocation>(gasometer)?;
		// Bound check
		input.expect_arguments(gasometer, 1)?;
		let amount: U256 = input.read(gasometer)?;

		// read destination
		let destination: MultiLocation = input.read::<MultiLocation>(gasometer)?;

		// Bound check
		input.expect_arguments(gasometer, 1)?;
		let dest_weight: u64 = input.read::<u64>(gasometer)?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let to_balance = amount
			.try_into()
			.map_err(|_| gasometer.revert("Amount is too large for provided balance type"))?;

		let call = orml_xtokens::Call::<Runtime>::transfer_multiasset {
			asset: Box::new(VersionedMultiAsset::V1(MultiAsset {
				id: AssetId::Concrete(asset_multilocation),
				fun: Fungibility::Fungible(to_balance),
			})),
			dest: Box::new(VersionedMultiLocation::V1(destination)),
			dest_weight,
		};

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn transfer_multiasset_with_fee(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		input.expect_arguments(gasometer, 5)?;

		// asset is defined as a multiLocation. For now we are assuming these are concrete
		// fungible assets
		let asset_multilocation: MultiLocation = input.read::<MultiLocation>(gasometer)?;
		let amount: U256 = input.read(gasometer)?;
		let fee: U256 = input.read(gasometer)?;

		// read destination
		let destination: MultiLocation = input.read::<MultiLocation>(gasometer)?;

		let dest_weight: u64 = input.read::<u64>(gasometer)?;

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let amount = amount
			.try_into()
			.map_err(|_| gasometer.revert("Amount is too large for provided balance type"))?;
		let fee = fee
			.try_into()
			.map_err(|_| gasometer.revert("Amount is too large for provided balance type"))?;

		let call = orml_xtokens::Call::<Runtime>::transfer_multiasset_with_fee {
			asset: Box::new(VersionedMultiAsset::V1(MultiAsset {
				id: AssetId::Concrete(asset_multilocation.clone()),
				fun: Fungibility::Fungible(amount),
			})),
			fee: Box::new(VersionedMultiAsset::V1(MultiAsset {
				id: AssetId::Concrete(asset_multilocation),
				fun: Fungibility::Fungible(fee),
			})),
			dest: Box::new(VersionedMultiLocation::V1(destination)),
			dest_weight,
		};

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}
}
