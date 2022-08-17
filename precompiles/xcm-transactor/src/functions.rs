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

//! Common functions to access xcm-transactor pallet dispatchables

use fp_evm::PrecompileHandle;
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::ConstU32,
};
use pallet_evm::{AddressMapping, PrecompileOutput};
use pallet_xcm_transactor::{
	Currency, CurrencyPayment, RemoteTransactInfoWithMaxWeight, TransactWeights,
};
use precompile_utils::prelude::*;
use sp_core::H160;
use sp_std::{
	boxed::Box,
	convert::{TryFrom, TryInto},
	marker::PhantomData,
};
use xcm::latest::MultiLocation;
use xcm_primitives::AccountIdToCurrencyId;

/// A precompile to wrap the functionality from xcm transactor
pub struct XcmTransactorWrapper<Runtime>(PhantomData<Runtime>);

pub type TransactorOf<Runtime> = <Runtime as pallet_xcm_transactor::Config>::Transactor;
pub type CurrencyIdOf<Runtime> = <Runtime as pallet_xcm_transactor::Config>::CurrencyId;

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);
type GetDataLimit = ConstU32<CALL_DATA_LIMIT>;

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
	pub(crate) fn account_index(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
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

	pub(crate) fn transact_info(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
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

	pub(crate) fn transact_info_with_signed(
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

	pub(crate) fn fee_per_second(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		handle.record_cost(1 * RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let mut input = handle.read_input()?;

		let multilocation: MultiLocation = input.read::<MultiLocation>()?;

		// fetch data from pallet
		let fee_per_second: u128 =
			pallet_xcm_transactor::Pallet::<Runtime>::dest_asset_fee_per_second(multilocation)
				.ok_or(revert("Fee Per Second not set"))?;

		Ok(succeed(EvmDataWriter::new().write(fee_per_second).build()))
	}

	pub(crate) fn transact_through_derivative_multilocation(
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
		let inner_call = input.read::<BoundedBytes<GetDataLimit>>()?.into_vec();

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_derivative {
			dest: transactor,
			index,
			fee: CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedMultiLocation::V1(
					fee_multilocation,
				))),
				fee_amount: None,
			},
			inner_call,
			weight_info: TransactWeights {
				transact_required_weight_at_most: weight,
				overall_weight: None,
			},
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	pub(crate) fn transact_through_derivative_multilocation_custom_fee_and_weight(
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
		let inner_call = input.read::<BoundedBytes<GetDataLimit>>()?.into_vec();

		// overall weight
		let fee_amount = input.read::<u128>()?;

		// overall weight
		let overall_weight = input.read::<u64>()?;

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_derivative {
			dest: transactor,
			index,
			fee: CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedMultiLocation::V1(
					fee_multilocation,
				))),
				fee_amount: Some(fee_amount),
			},
			inner_call,
			weight_info: TransactWeights {
				transact_required_weight_at_most: weight,
				overall_weight: Some(overall_weight),
			},
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	pub(crate) fn transact_through_derivative(
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
		let inner_call = input.read::<BoundedBytes<GetDataLimit>>()?.into_vec();

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
			fee: CurrencyPayment {
				currency: Currency::AsCurrencyId(currency_id),
				fee_amount: None,
			},
			weight_info: TransactWeights {
				transact_required_weight_at_most: weight,
				overall_weight: None,
			},
			inner_call,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	pub(crate) fn transact_through_derivative_custom_fee_and_weight(
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
		let inner_call = input.read::<BoundedBytes<GetDataLimit>>()?.into_vec();

		// overall weight
		let fee_amount = input.read::<u128>()?;

		// overall weight
		let overall_weight = input.read::<u64>()?;

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
			fee: CurrencyPayment {
				currency: Currency::AsCurrencyId(currency_id),
				fee_amount: Some(fee_amount),
			},
			weight_info: TransactWeights {
				transact_required_weight_at_most: weight,
				overall_weight: Some(overall_weight),
			},
			inner_call,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	pub(crate) fn transact_through_signed_multilocation_custom_fee_and_weight(
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
		let call = input.read::<BoundedBytes<GetDataLimit>>()?.into_vec();

		// overall weight
		let fee_amount = input.read::<u128>()?;

		// overall weight
		let overall_weight = input.read::<u64>()?;

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_signed {
			dest: Box::new(xcm::VersionedMultiLocation::V1(dest)),
			fee: CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedMultiLocation::V1(
					fee_multilocation,
				))),
				fee_amount: Some(fee_amount),
			},
			weight_info: TransactWeights {
				transact_required_weight_at_most: weight,
				overall_weight: Some(overall_weight),
			},
			call,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	pub(crate) fn transact_through_signed_multilocation(
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
		let call = input.read::<BoundedBytes<GetDataLimit>>()?.into_vec();

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_signed {
			dest: Box::new(xcm::VersionedMultiLocation::V1(dest)),
			fee: CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedMultiLocation::V1(
					fee_multilocation,
				))),
				fee_amount: None,
			},
			weight_info: TransactWeights {
				transact_required_weight_at_most: weight,
				overall_weight: None,
			},
			call,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	pub(crate) fn transact_through_signed(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;

		// Bound check
		input.expect_arguments(4)?;

		// read destination
		let dest: MultiLocation = input.read::<MultiLocation>()?;

		// read currencyId
		let to_address: H160 = input.read::<Address>()?.into();

		let to_account = Runtime::AddressMapping::into_account_id(to_address);

		// read weight amount
		let weight: u64 = input.read::<u64>()?;

		// call
		let call = input.read::<BoundedBytes<GetDataLimit>>()?.into_vec();

		// We convert the address into a currency
		// This involves a DB read in moonbeam, hence the db Read
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let currency_id: <Runtime as pallet_xcm_transactor::Config>::CurrencyId =
			Runtime::account_to_currency_id(to_account)
				.ok_or(revert("cannot convert into currency id"))?;

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_signed {
			dest: Box::new(xcm::VersionedMultiLocation::V1(dest)),
			fee: CurrencyPayment {
				currency: Currency::AsCurrencyId(currency_id),
				fee_amount: None,
			},
			weight_info: TransactWeights {
				transact_required_weight_at_most: weight,
				overall_weight: None,
			},
			call,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}

	pub(crate) fn transact_through_signed_custom_fee_and_weight(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;

		// Bound check
		input.expect_arguments(4)?;

		// read destination
		let dest: MultiLocation = input.read::<MultiLocation>()?;

		// read currencyId
		let to_address: H160 = input.read::<Address>()?.into();

		// read weight amount
		let weight: u64 = input.read::<u64>()?;

		// call
		let call = input.read::<BoundedBytes<GetDataLimit>>()?.into_vec();

		// overall weight
		let fee_amount = input.read::<u128>()?;

		// overall weight
		let overall_weight = input.read::<u64>()?;

		let to_account = Runtime::AddressMapping::into_account_id(to_address);

		// We convert the address into a currency
		// This involves a DB read in moonbeam, hence the db Read
		handle.record_cost(1 * RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let currency_id: <Runtime as pallet_xcm_transactor::Config>::CurrencyId =
			Runtime::account_to_currency_id(to_account)
				.ok_or(revert("cannot convert into currency id"))?;

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_signed {
			dest: Box::new(xcm::VersionedMultiLocation::V1(dest)),
			fee: CurrencyPayment {
				currency: Currency::AsCurrencyId(currency_id),
				fee_amount: Some(fee_amount),
			},
			weight_info: TransactWeights {
				transact_required_weight_at_most: weight,
				overall_weight: Some(overall_weight),
			},
			call,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}
}
