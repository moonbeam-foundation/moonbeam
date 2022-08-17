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
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::ConstU32,
};
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

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);
type GetDataLimit = ConstU32<CALL_DATA_LIMIT>;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	IndexToAccount = "indexToAccount(uint16)",
	// DEPRECATED
	TransactInfo = "transactInfo((uint8,bytes[]))",
	TransactThroughDerivativeMultiLocation =
		"transactThroughDerivativeMultilocation(uint8,uint16,(uint8,bytes[]),uint64,bytes)",
	TransactThroughDerivative = "transactThroughDerivative(uint8,uint16,address,uint64,bytes)",
	TransactInfoWithSigned = "transactInfoWithSigned((uint8,bytes[]))",
	FeePerSecond = "feePerSecond((uint8,bytes[]))",
	TransactThroughSignedMultiLocation =
		"transactThroughSignedMultilocation((uint8,bytes[]),(uint8,bytes[]),uint64,bytes)",
	TransactThroughSigned = "transactThroughSigned((uint8,bytes[]),address,uint64,bytes)",

	// deprecated
	DeprecatedIndexToAccount = "index_to_account(uint16)",
	DeprecatedTransactInfo = "transact_info((uint8,bytes[]))",
	DeprecatedTransactThroughDerivativeMultiLocation =
		"transact_through_derivative_multilocation(uint8,uint16,(uint8,bytes[]),uint64,bytes)",
	DeprecatedTransactThroughDerivative =
		"transact_through_derivative(uint8,uint16,address,uint64,bytes)",
	DeprecatedTransactInfoWithSigned = "transact_info_with_signed((uint8,bytes[]))",
	DeprecatedFeePerSecond = "fee_per_second((uint8,bytes[]))",
	DeprecatedTransactThroughSignedMultiLocation =
		"transact_through_signed_multilocation((uint8,bytes[]),(uint8,bytes[]),uint64,bytes)",
	DeprecatedTransactThroughSigned =
		"transact_through_signed((uint8,bytes[]),address,uint64,bytes)",
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
			Action::TransactThroughDerivativeMultiLocation
			| Action::TransactThroughDerivative
			| Action::TransactThroughSignedMultiLocation
			| Action::TransactThroughSigned
			| Action::DeprecatedTransactThroughDerivativeMultiLocation
			| Action::DeprecatedTransactThroughDerivative
			| Action::DeprecatedTransactThroughSignedMultiLocation
			| Action::DeprecatedTransactThroughSigned => FunctionModifier::NonPayable,
			_ => FunctionModifier::View,
		})?;

		match selector {
			// Check for accessor methods first. These return results immediately
			Action::IndexToAccount | Action::DeprecatedIndexToAccount => {
				Self::account_index(handle)
			}
			// DEPRECATED
			Action::TransactInfo | Action::DeprecatedTransactInfo => Self::transact_info(handle),
			Action::TransactInfoWithSigned | Action::DeprecatedTransactInfoWithSigned => {
				Self::transact_info_with_signed(handle)
			}
			Action::FeePerSecond | Action::DeprecatedFeePerSecond => Self::fee_per_second(handle),
			Action::TransactThroughDerivativeMultiLocation
			| Action::DeprecatedTransactThroughDerivativeMultiLocation => {
				Self::transact_through_derivative_multilocation(handle)
			}
			Action::TransactThroughDerivative | Action::DeprecatedTransactThroughDerivative => {
				Self::transact_through_derivative(handle)
			}
			Action::TransactThroughSignedMultiLocation
			| Action::DeprecatedTransactThroughSignedMultiLocation => {
				Self::transact_through_signed_multilocation(handle)
			}
			Action::TransactThroughSigned | Action::DeprecatedTransactThroughSigned => {
				Self::transact_through_signed(handle)
			}
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

		read_args!(handle, { index: u16 });

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

		read_args!(handle, { multilocation: MultiLocation });

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

		read_args!(handle, { multilocation: MultiLocation });

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

		read_args!(handle, { multilocation: MultiLocation });

		// fetch data from pallet
		let fee_per_second: u128 =
			pallet_xcm_transactor::Pallet::<Runtime>::dest_asset_fee_per_second(multilocation)
				.ok_or(revert("Fee Per Second not set"))?;

		Ok(succeed(EvmDataWriter::new().write(fee_per_second).build()))
	}

	fn transact_through_derivative_multilocation(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			transactor: u8,
			index: u16,
			fee_asset: MultiLocation,
			weight: u64,
			inner_call: BoundedBytes<GetDataLimit>
		});
		let transactor = transactor
			.try_into()
			.map_err(|_| RevertReason::custom("Non-existent transactor").in_field("transactor"))?;
		let inner_call = inner_call.into_vec();

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_xcm_transactor::Call::<Runtime>::transact_through_derivative_multilocation {
				dest: transactor,
				index,
				fee_location: Box::new(xcm::VersionedMultiLocation::V1(fee_asset)),
				dest_weight: weight,
				inner_call,
			};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn transact_through_derivative(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			transactor: u8,
			index: u16,
			currency_id: Address,
			weight: u64,
			inner_call: BoundedBytes<GetDataLimit>
		});

		let transactor = transactor
			.try_into()
			.map_err(|_| RevertReason::custom("Non-existent transactor").in_field("transactor"))?;
		let inner_call = inner_call.into_vec();

		let to_account = Runtime::AddressMapping::into_account_id(currency_id.0);

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
			inner_call: inner_call,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn transact_through_signed_multilocation(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			dest: MultiLocation,
			fee_location: MultiLocation,
			weight: u64,
			call: BoundedBytes<GetDataLimit>
		});
		let call = call.into_vec();

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_signed_multilocation {
			dest: Box::new(xcm::VersionedMultiLocation::V1(dest)),
			fee_location: Box::new(xcm::VersionedMultiLocation::V1(fee_location)),
			dest_weight: weight,
			call,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	fn transact_through_signed(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(1 * RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		read_args!(handle, {
			dest: MultiLocation,
			fee_location_address: Address,
			weight: u64,
			call: BoundedBytes<GetDataLimit>
		});
		let to_address: H160 = fee_location_address.into();
		let to_account = Runtime::AddressMapping::into_account_id(to_address);
		let call = call.into_vec();

		// We convert the address into a currency
		// This involves a DB read in moonbeam, hence the db Read
		let currency_id: <Runtime as pallet_xcm_transactor::Config>::CurrencyId =
			Runtime::account_to_currency_id(to_account)
				.ok_or(revert("cannot convert into currency id"))?;

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_signed {
			dest: Box::new(xcm::VersionedMultiLocation::V1(dest)),
			fee_currency_id: currency_id,
			dest_weight: weight,
			call,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}
}
