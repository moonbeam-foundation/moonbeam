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

		read_args!(handle, { index: u16 });

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

	pub(crate) fn transact_info_with_signed(
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

	pub(crate) fn fee_per_second(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		handle.record_cost(1 * RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		read_args!(handle, { multilocation: MultiLocation });

		// fetch data from pallet
		let fee_per_second: u128 =
			pallet_xcm_transactor::Pallet::<Runtime>::dest_asset_fee_per_second(multilocation)
				.ok_or(revert("Fee Per Second not set"))?;

		Ok(succeed(EvmDataWriter::new().write(fee_per_second).build()))
	}

	pub(crate) fn transact_through_derivative_multilocation(
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
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_derivative {
			dest: transactor,
			index,
			fee: CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedMultiLocation::V1(
					fee_asset,
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
		read_args!(handle, {
			transactor: u8,
			index: u16,
			fee_asset: MultiLocation,
			weight: u64,
			inner_call: BoundedBytes<GetDataLimit>,
			fee_amount: u128,
			overall_weight: u64
		});

		let transactor = transactor
			.try_into()
			.map_err(|_| RevertReason::custom("Non-existent transactor").in_field("transactor"))?;

		let inner_call = inner_call.into_vec();

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_derivative {
			dest: transactor,
			index,
			fee: CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedMultiLocation::V1(
					fee_asset,
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
		read_args!(handle, {
			transactor: u8,
			index: u16,
			fee_asset: Address,
			weight: u64,
			inner_call: BoundedBytes<GetDataLimit>,
			fee_amount: u128,
			overall_weight: u64
		});

		let transactor = transactor
			.try_into()
			.map_err(|_| RevertReason::custom("Non-existent transactor").in_field("transactor"))?;
		let inner_call = inner_call.into_vec();

		let to_address: H160 = fee_asset.into();
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

	pub(crate) fn transact_through_signed_multilocation(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			dest: MultiLocation,
			fee_asset: MultiLocation,
			weight: u64,
			call: BoundedBytes<GetDataLimit>
		});
		let call = call.into_vec();

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_signed {
			dest: Box::new(xcm::VersionedMultiLocation::V1(dest)),
			fee: CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedMultiLocation::V1(
					fee_asset,
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

	pub(crate) fn transact_through_signed_multilocation_custom_fee_and_weight(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			dest: MultiLocation,
			fee_asset: MultiLocation,
			weight: u64,
			call: BoundedBytes<GetDataLimit>,
			fee_amount: u128,
			overall_weight: u64
		});
		let call = call.into_vec();

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_signed {
			dest: Box::new(xcm::VersionedMultiLocation::V1(dest)),
			fee: CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedMultiLocation::V1(
					fee_asset,
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

	pub(crate) fn transact_through_signed(
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<PrecompileOutput> {
		read_args!(handle, {
			dest: MultiLocation,
			fee_asset: Address,
			weight: u64,
			call: BoundedBytes<GetDataLimit>
		});

		let to_address: H160 = fee_asset.into();
		let to_account = Runtime::AddressMapping::into_account_id(to_address);

		let call = call.into_vec();

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
		read_args!(handle, {
			dest: MultiLocation,
			fee_asset: Address,
			weight: u64,
			call: BoundedBytes<GetDataLimit>,
			fee_amount: u128,
			overall_weight: u64
		});

		let to_address: H160 = fee_asset.into();
		let to_account = Runtime::AddressMapping::into_account_id(to_address);

		let call = call.into_vec();

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
