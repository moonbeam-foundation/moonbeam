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

//! Precompile to xcm transactor runtime methods via the EVM

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(assert_matches)]

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{AddressMapping, PrecompileOutput};
use pallet_xcm_transactor::RemoteTransactInfoWithMaxWeight;
use precompile_utils::prelude::*;
use sp_core::H160;
use sp_std::{
	boxed::Box,
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker::PhantomData,
};
use xcm::latest::MultiLocation;
use xcm_primitives::AccountIdToCurrencyId;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub type TransactorOf<Runtime> = <Runtime as pallet_xcm_transactor::Config>::Transactor;
pub type CurrencyIdOf<Runtime> = <Runtime as pallet_xcm_transactor::Config>::CurrencyId;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	IndexToAccount = "index_to_account(uint16)",
	// DEPRECATED
	TransactInfo = "transact_info((uint8,bytes[]))",
	TransactThroughDerivativeMultiLocation =
		"transact_through_derivative_multilocation(uint8,uint16,(uint8,bytes[]),uint64,bytes)",
	TransactThroughDerivative = "transact_through_derivative(uint8,uint16,address,uint64,bytes)",
	TransactInfoWithSigned = "transact_info_with_signed((uint8,bytes[]))",
	FeePerSecond = "fee_per_second((uint8,bytes[]))",
	TransactThroughSignedMultiLocation =
		"transact_through_signed_multilocation((uint8,bytes[]),(uint8,bytes[]),uint64,bytes)",
	TransactThroughSigned = "transact_through_signed((uint8,bytes[]),address,uint64,bytes)",
}

/// A precompile to wrap the functionality from xcm transactor
pub struct XcmTransactorWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> pallet_evm::Precompile for XcmTransactorWrapper<Runtime>
where
	Runtime: pallet_xcm_transactor::Config + pallet_evm::Config + frame_system::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_xcm_transactor::Call<Runtime>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	TransactorOf<Runtime>: TryFrom<u8>,
	Runtime::AccountId: Into<H160>,
	Runtime: AccountIdToCurrencyId<Runtime::AccountId, CurrencyIdOf<Runtime>>,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let selector = handle.read_selector()?;

		handle.check_function_modifier(match selector {
			Action::TransactThroughDerivativeMultiLocation | Action::TransactThroughDerivative => {
				FunctionModifier::NonPayable
			}
			_ => FunctionModifier::View,
		})?;

		match selector {
			// Check for accessor methods first. These return results immediately
			Action::IndexToAccount => Self::account_index(handle),
			// DEPRECATED
			Action::TransactInfo => Self::transact_info(handle),
			Action::TransactInfoWithSigned => Self::transact_info_with_signed(handle),
			Action::FeePerSecond => Self::fee_per_second(handle),
			Action::TransactThroughDerivativeMultiLocation => {
				Self::transact_through_derivative_multilocation(handle)
			}
			Action::TransactThroughDerivative => Self::transact_through_derivative(handle),
			Action::TransactThroughSignedMultiLocation => {
				Self::transact_through_signed_multilocation(handle)
			}
			Action::TransactThroughSigned => Self::transact_through_signed(handle),
		}
	}
}

impl<Runtime> XcmTransactorWrapper<Runtime>
where
	Runtime: pallet_xcm_transactor::Config + pallet_evm::Config + frame_system::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_xcm_transactor::Call<Runtime>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	TransactorOf<Runtime>: TryFrom<u8>,
	Runtime::AccountId: Into<H160>,
	Runtime: AccountIdToCurrencyId<Runtime::AccountId, CurrencyIdOf<Runtime>>,
{
	fn account_index(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Bound check
		let mut input = handle.read_input()?;
		input.expect_arguments(1)?;
		let index: u16 = input.read::<u16>()?;

		// fetch data from pallet
		let account: H160 = pallet_xcm_transactor::Pallet::<Runtime>::index_to_account(index)
			.ok_or(revert("No index assigned"))?
			.into();

		Ok(succeed(
			EvmDataWriter::new().write(Address(account)).build(),
		))
	}

	fn transact_info(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(2 * RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let mut input = handle.read_input()?;
		let multilocation: MultiLocation = input.read::<MultiLocation>()?;

		// fetch data from pallet
		let remote_transact_info: RemoteTransactInfoWithMaxWeight =
			pallet_xcm_transactor::Pallet::<Runtime>::transact_info(&multilocation)
				.ok_or(revert("Transact Info not set"))?;

		// fetch data from pallet
		let fee_per_second: u128 =
			pallet_xcm_transactor::Pallet::<Runtime>::dest_asset_fee_per_second(&multilocation)
				.ok_or(revert("Fee Per Second not set"))?;

		Ok(succeed(
			EvmDataWriter::new()
				.write(remote_transact_info.transact_extra_weight)
				.write(fee_per_second)
				.write(remote_transact_info.max_weight)
				.build(),
		))
	}

	fn transact_info_with_signed(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		handle.record_cost(1 * RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let mut input = handle.read_input()?;
		let multilocation: MultiLocation = input.read::<MultiLocation>()?;

		// fetch data from pallet
		let remote_transact_info: RemoteTransactInfoWithMaxWeight =
			pallet_xcm_transactor::Pallet::<Runtime>::transact_info(multilocation)
				.ok_or(revert("Transact Info not set"))?;

		let transact_extra_weight_signed = remote_transact_info
			.transact_extra_weight_signed
			.unwrap_or(0);
		Ok(succeed(
			EvmDataWriter::new()
				.write(remote_transact_info.transact_extra_weight)
				.write(transact_extra_weight_signed)
				.write(remote_transact_info.max_weight)
				.build(),
		))
	}

	fn fee_per_second(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(1 * RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let mut input = handle.read_input()?;

		let multilocation: MultiLocation = input.read::<MultiLocation>()?;

		// fetch data from pallet
		let fee_per_second: u128 =
			pallet_xcm_transactor::Pallet::<Runtime>::dest_asset_fee_per_second(multilocation)
				.ok_or(revert("Fee Per Second not set"))?;

		Ok(succeed(EvmDataWriter::new().write(fee_per_second).build()))
	}

	fn transact_through_derivative_multilocation(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		// Bound check
		input.expect_arguments(5)?;

		// Does not need DB read
		let transactor: TransactorOf<Runtime> = input
			.read::<u8>()?
			.try_into()
			.map_err(|_| revert("Non-existent transactor"))?;
		let index: u16 = input.read::<u16>()?;

		// read fee location
		// defined as a multiLocation. For now we are assuming these are concrete
		// fungible assets
		let fee_multilocation: MultiLocation = input.read::<MultiLocation>()?;
		// read fee amount
		let weight: u64 = input.read::<u64>()?;

		// inner call
		let inner_call = input.read::<Bytes>()?;

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_xcm_transactor::Call::<Runtime>::transact_through_derivative_multilocation {
				dest: transactor,
				index,
				fee_location: Box::new(xcm::VersionedMultiLocation::V1(fee_multilocation)),
				dest_weight: weight,
				inner_call: inner_call.0,
			};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn transact_through_derivative(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		// Bound check
		input.expect_arguments(5)?;
		let transactor: TransactorOf<Runtime> = input
			.read::<u8>()?
			.try_into()
			.map_err(|_| revert("Non-existent transactor"))?;
		let index: u16 = input.read::<u16>()?;

		// read currencyId
		let to_address: H160 = input.read::<Address>()?.into();

		// read fee amount
		let weight: u64 = input.read::<u64>()?;

		// inner call
		let inner_call = input.read::<Bytes>()?;

		let to_account = Runtime::AddressMapping::into_account_id(to_address);

		// We convert the address into a currency
		// This involves a DB read in moonbeam, hence the db Read
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let currency_id: <Runtime as pallet_xcm_transactor::Config>::CurrencyId =
			Runtime::account_to_currency_id(to_account)
				.ok_or(revert("cannot convert into currency id"))?;

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_derivative {
			dest: transactor,
			index,
			currency_id,
			dest_weight: weight,
			inner_call: inner_call.0,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn transact_through_signed_multilocation(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;

		// Bound check
		input.expect_arguments(4)?;

		// read destination
		let dest: MultiLocation = input.read::<MultiLocation>()?;

		// read fee location
		// defined as a multiLocation. For now we are assuming these are concrete
		// fungible assets
		let fee_multilocation: MultiLocation = input.read::<MultiLocation>()?;
		// read weight amount
		let weight: u64 = input.read::<u64>()?;

		// call
		let call = input.read::<Bytes>()?;

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_signed_multilocation {
			dest: Box::new(xcm::VersionedMultiLocation::V1(dest)),
			fee_location: Box::new(xcm::VersionedMultiLocation::V1(fee_multilocation)),
			dest_weight: weight,
			call: call.0,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn transact_through_signed(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(1 * RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let mut input = handle.read_input()?;

		// Bound check
		input.expect_arguments(4)?;

		// read destination
		let dest: MultiLocation = input.read::<MultiLocation>()?;

		// read currencyId
		let to_address: H160 = input.read::<Address>()?.into();

		let to_account = Runtime::AddressMapping::into_account_id(to_address);

		// We convert the address into a currency
		// This involves a DB read in moonbeam, hence the db Read

		let currency_id: <Runtime as pallet_xcm_transactor::Config>::CurrencyId =
			Runtime::account_to_currency_id(to_account)
				.ok_or(revert("cannot convert into currency id"))?;

		// read weight amount
		let weight: u64 = input.read::<u64>()?;

		// call
		let call = input.read::<Bytes>()?;

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_signed {
			dest: Box::new(xcm::VersionedMultiLocation::V1(dest)),
			fee_currency_id: currency_id,
			dest_weight: weight,
			call: call.0,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}
}
