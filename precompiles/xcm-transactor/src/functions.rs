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
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::ConstU32,
};
use pallet_evm::AddressMapping;
use pallet_xcm_transactor::{
	Currency, CurrencyPayment, RemoteTransactInfoWithMaxWeight, TransactWeights,
};
use precompile_utils::prelude::*;
use sp_core::{MaxEncodedLen, H160, U256};
use sp_runtime::traits::Dispatchable;
use sp_std::{
	boxed::Box,
	convert::{TryFrom, TryInto},
	marker::PhantomData,
	vec::Vec,
};
use sp_weights::Weight;
use xcm::latest::prelude::*;
use xcm::latest::Location;
use xcm_primitives::{
	AccountIdToCurrencyId, UtilityAvailableCalls, UtilityEncodeCall, DEFAULT_PROOF_SIZE,
};

/// A precompile to wrap the functionality from xcm transactor
pub struct XcmTransactorWrapper<Runtime>(PhantomData<Runtime>);

pub type TransactorOf<Runtime> = <Runtime as pallet_xcm_transactor::Config>::Transactor;
pub type CurrencyIdOf<Runtime> = <Runtime as pallet_xcm_transactor::Config>::CurrencyId;

pub const CALL_DATA_LIMIT: u32 = 2u32.pow(16);
pub type GetDataLimit = ConstU32<CALL_DATA_LIMIT>;

impl<Runtime> XcmTransactorWrapper<Runtime>
where
	Runtime: pallet_xcm_transactor::Config + pallet_evm::Config + frame_system::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_xcm_transactor::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	TransactorOf<Runtime>: TryFrom<u8>,
	Runtime::AccountId: Into<H160>,
	Runtime: AccountIdToCurrencyId<Runtime::AccountId, CurrencyIdOf<Runtime>>,
{
	pub(crate) fn account_index(
		handle: &mut impl PrecompileHandle,
		index: u16,
	) -> EvmResult<Address> {
		// storage item: IndexToAccount: Blake2_128(16) + u16(2) + AccountId(20)
		handle.record_db_read::<Runtime>(38)?;

		// fetch data from pallet
		let account: H160 = pallet_xcm_transactor::Pallet::<Runtime>::index_to_account(index)
			.ok_or(revert("No index assigned"))?
			.into();

		Ok(account.into())
	}

	pub(crate) fn transact_info(
		handle: &mut impl PrecompileHandle,
		multilocation: Location,
	) -> EvmResult<(u64, U256, u64)> {
		// fetch data from pallet
		// storage item: TransactInfoWithWeightLimit: Blake2_128(16) + Location
		// + RemoteTransactInfoWithMaxWeight
		handle.record_db_read::<Runtime>(
			16 + Location::max_encoded_len() + RemoteTransactInfoWithMaxWeight::max_encoded_len(),
		)?;
		let remote_transact_info: RemoteTransactInfoWithMaxWeight =
			pallet_xcm_transactor::Pallet::<Runtime>::transact_info(&multilocation)
				.ok_or(revert("Transact Info not set"))?;

		// fetch data from pallet
		// storage item: AssetTypeUnitsPerSecond: Blake2_128(16) + Location + u128(16)
		handle.record_db_read::<Runtime>(32 + Location::max_encoded_len())?;
		let fee_per_second: u128 =
			pallet_xcm_transactor::Pallet::<Runtime>::dest_asset_fee_per_second(&multilocation)
				.ok_or(revert("Fee Per Second not set"))?;

		Ok((
			remote_transact_info.transact_extra_weight.ref_time(),
			fee_per_second.into(),
			remote_transact_info.max_weight.ref_time(),
		))
	}

	pub(crate) fn transact_info_with_signed(
		handle: &mut impl PrecompileHandle,
		multilocation: Location,
	) -> EvmResult<(u64, u64, u64)> {
		// fetch data from pallet
		// storage item: TransactInfoWithWeightLimit: Blake2_128(16) + Location
		// + RemoteTransactInfoWithMaxWeight
		handle.record_db_read::<Runtime>(
			16 + Location::max_encoded_len() + RemoteTransactInfoWithMaxWeight::max_encoded_len(),
		)?;
		let remote_transact_info: RemoteTransactInfoWithMaxWeight =
			pallet_xcm_transactor::Pallet::<Runtime>::transact_info(multilocation)
				.ok_or(revert("Transact Info not set"))?;

		let transact_extra_weight_signed = remote_transact_info
			.transact_extra_weight_signed
			.unwrap_or(Weight::zero());

		Ok((
			remote_transact_info.transact_extra_weight.ref_time(),
			transact_extra_weight_signed.ref_time(),
			remote_transact_info.max_weight.ref_time(),
		))
	}

	pub(crate) fn fee_per_second(
		handle: &mut impl PrecompileHandle,
		location: Location,
	) -> EvmResult<U256> {
		// fetch data from pallet
		// storage item: AssetTypeUnitsPerSecond: Blake2_128(16) + Location + u128(16)
		handle.record_db_read::<Runtime>(32 + Location::max_encoded_len())?;
		let fee_per_second: u128 =
			pallet_xcm_transactor::Pallet::<Runtime>::dest_asset_fee_per_second(location)
				.ok_or(revert("Fee Per Second not set"))?;

		Ok(fee_per_second.into())
	}

	pub(crate) fn transact_through_derivative_multilocation(
		handle: &mut impl PrecompileHandle,
		transactor: u8,
		index: u16,
		fee_asset: Location,
		weight: u64,
		inner_call: BoundedBytes<GetDataLimit>,
	) -> EvmResult {
		let transactor = transactor
			.try_into()
			.map_err(|_| RevertReason::custom("Non-existent transactor").in_field("transactor"))?;
		let inner_call: Vec<_> = inner_call.into();

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_derivative {
			dest: transactor,
			index,
			fee: CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					fee_asset,
				))),
				fee_amount: None,
			},
			inner_call,
			weight_info: TransactWeights {
				// TODO overall weight None means we will retrieve the allowed proof size from storage.
				// In the v2 to v3 migration we set this value to DEFAULT_PROOF_SIZE, so setting
				// require_weight_at_most to DEFAULT_PROOF_SIZE/2 makes sense. Although we might
				// want to revisit this to use whatever storage value there is and divide it by 2.
				transact_required_weight_at_most: Weight::from_parts(
					weight,
					DEFAULT_PROOF_SIZE.saturating_div(2),
				),
				overall_weight: None,
			},
			refund: false,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	pub(crate) fn transact_through_derivative_multilocation_fee_weight(
		handle: &mut impl PrecompileHandle,
		transactor: u8,
		index: u16,
		fee_asset: Location,
		weight: u64,
		inner_call: BoundedBytes<GetDataLimit>,
		fee_amount: u128,
		overall_weight: u64,
	) -> EvmResult {
		let transactor = transactor
			.try_into()
			.map_err(|_| RevertReason::custom("Non-existent transactor").in_field("transactor"))?;

		let inner_call: Vec<_> = inner_call.into();

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_derivative {
			dest: transactor,
			index,
			fee: CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					fee_asset,
				))),
				fee_amount: Some(fee_amount),
			},
			inner_call,
			weight_info: TransactWeights {
				transact_required_weight_at_most: Weight::from_parts(
					weight,
					DEFAULT_PROOF_SIZE.saturating_div(2),
				),
				overall_weight: Some(Limited(Weight::from_parts(
					overall_weight,
					DEFAULT_PROOF_SIZE,
				))),
			},
			refund: false,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	pub(crate) fn transact_through_derivative(
		handle: &mut impl PrecompileHandle,
		transactor: u8,
		index: u16,
		currency_id: Address,
		weight: u64,
		inner_call: BoundedBytes<GetDataLimit>,
	) -> EvmResult {
		// No DB access before try_dispatch but lot of logical stuff
		// To prevent spam, we charge an arbitrary amoun of gas
		handle.record_cost(1000)?;

		let transactor = transactor
			.try_into()
			.map_err(|_| RevertReason::custom("Non-existent transactor").in_field("transactor"))?;
		let inner_call: Vec<_> = inner_call.into();

		let to_account = Runtime::AddressMapping::into_account_id(currency_id.0);

		// We convert the address into a currency
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
				transact_required_weight_at_most: Weight::from_parts(
					weight,
					DEFAULT_PROOF_SIZE.saturating_div(2),
				),
				overall_weight: None,
			},
			inner_call,
			refund: false,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	pub(crate) fn transact_through_derivative_fee_weight(
		handle: &mut impl PrecompileHandle,
		transactor: u8,
		index: u16,
		fee_asset: Address,
		weight: u64,
		inner_call: BoundedBytes<GetDataLimit>,
		fee_amount: u128,
		overall_weight: u64,
	) -> EvmResult {
		// No DB access before try_dispatch but lot of logical stuff
		// To prevent spam, we charge an arbitrary amoun of gas
		handle.record_cost(1000)?;

		let transactor = transactor
			.try_into()
			.map_err(|_| RevertReason::custom("Non-existent transactor").in_field("transactor"))?;
		let inner_call: Vec<_> = inner_call.into();

		let to_address: H160 = fee_asset.into();
		let to_account = Runtime::AddressMapping::into_account_id(to_address);

		// We convert the address into a currency
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
				transact_required_weight_at_most: Weight::from_parts(
					weight,
					DEFAULT_PROOF_SIZE.saturating_div(2),
				),
				overall_weight: Some(Limited(Weight::from_parts(
					overall_weight,
					DEFAULT_PROOF_SIZE,
				))),
			},
			inner_call,
			refund: false,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	pub(crate) fn transact_through_signed_multilocation(
		handle: &mut impl PrecompileHandle,
		dest: Location,
		fee_asset: Location,
		weight: u64,
		call: BoundedBytes<GetDataLimit>,
	) -> EvmResult {
		let call: Vec<_> = call.into();

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_signed {
			dest: Box::new(xcm::VersionedLocation::V4(dest)),
			fee: CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					fee_asset,
				))),
				fee_amount: None,
			},
			weight_info: TransactWeights {
				transact_required_weight_at_most: Weight::from_parts(
					weight,
					DEFAULT_PROOF_SIZE.saturating_div(2),
				),
				overall_weight: None,
			},
			refund: false,
			call,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	pub(crate) fn transact_through_signed_multilocation_fee_weight(
		handle: &mut impl PrecompileHandle,
		dest: Location,
		fee_asset: Location,
		weight: u64,
		call: BoundedBytes<GetDataLimit>,
		fee_amount: u128,
		overall_weight: u64,
	) -> EvmResult {
		let call: Vec<_> = call.into();

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_signed {
			dest: Box::new(xcm::VersionedLocation::V4(dest)),
			fee: CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					fee_asset,
				))),
				fee_amount: Some(fee_amount),
			},
			weight_info: TransactWeights {
				transact_required_weight_at_most: Weight::from_parts(
					weight,
					DEFAULT_PROOF_SIZE.saturating_div(2),
				),
				overall_weight: Some(Limited(Weight::from_parts(
					overall_weight,
					DEFAULT_PROOF_SIZE,
				))),
			},
			refund: false,
			call,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	pub(crate) fn transact_through_signed(
		handle: &mut impl PrecompileHandle,
		dest: Location,
		fee_asset: Address,
		weight: u64,
		call: BoundedBytes<GetDataLimit>,
	) -> EvmResult {
		// No DB access before try_dispatch but lot of logical stuff
		// To prevent spam, we charge an arbitrary amoun of gas
		handle.record_cost(1000)?;

		let to_address: H160 = fee_asset.into();
		let to_account = Runtime::AddressMapping::into_account_id(to_address);

		let call: Vec<_> = call.into();

		// We convert the address into a currency
		let currency_id: <Runtime as pallet_xcm_transactor::Config>::CurrencyId =
			Runtime::account_to_currency_id(to_account)
				.ok_or(revert("cannot convert into currency id"))?;

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_signed {
			dest: Box::new(xcm::VersionedLocation::V4(dest)),
			fee: CurrencyPayment {
				currency: Currency::AsCurrencyId(currency_id),
				fee_amount: None,
			},
			weight_info: TransactWeights {
				transact_required_weight_at_most: Weight::from_parts(
					weight,
					DEFAULT_PROOF_SIZE.saturating_div(2),
				),
				overall_weight: None,
			},
			refund: false,
			call,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	pub(crate) fn transact_through_signed_fee_weight(
		handle: &mut impl PrecompileHandle,
		dest: Location,
		fee_asset: Address,
		weight: u64,
		call: BoundedBytes<GetDataLimit>,
		fee_amount: u128,
		overall_weight: u64,
	) -> EvmResult {
		// No DB access before try_dispatch but lot of logical stuff
		// To prevent spam, we charge an arbitrary amoun of gas
		handle.record_cost(1000)?;

		let to_address: H160 = fee_asset.into();
		let to_account = Runtime::AddressMapping::into_account_id(to_address);

		let call: Vec<_> = call.into();

		// We convert the address into a currency
		let currency_id: <Runtime as pallet_xcm_transactor::Config>::CurrencyId =
			Runtime::account_to_currency_id(to_account)
				.ok_or(revert("cannot convert into currency id"))?;

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_signed {
			dest: Box::new(xcm::VersionedLocation::V4(dest)),
			fee: CurrencyPayment {
				currency: Currency::AsCurrencyId(currency_id),
				fee_amount: Some(fee_amount),
			},
			weight_info: TransactWeights {
				transact_required_weight_at_most: Weight::from_parts(
					weight,
					DEFAULT_PROOF_SIZE.saturating_div(2),
				),
				overall_weight: Some(Limited(Weight::from_parts(
					overall_weight,
					DEFAULT_PROOF_SIZE,
				))),
			},
			refund: false,
			call,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	pub(crate) fn encode_utility_as_derivative(
		handle: &mut impl PrecompileHandle,
		transactor: u8,
		index: u16,
		inner_call: BoundedBytes<GetDataLimit>,
	) -> EvmResult<UnboundedBytes> {
		// There is no DB read in this function,
		// we just account an arbitrary amount of gas to prevent spam
		// TODO replace by proper benchmarks
		handle.record_cost(1000)?;

		let transactor: TransactorOf<Runtime> = transactor
			.try_into()
			.map_err(|_| RevertReason::custom("Non-existent transactor").in_field("transactor"))?;

		let encoded = UtilityEncodeCall::encode_call(
			transactor,
			UtilityAvailableCalls::AsDerivative(index, inner_call.into()),
		)
		.as_slice()
		.into();
		Ok(encoded)
	}

	pub(crate) fn transact_info_with_signed_v3(
		handle: &mut impl PrecompileHandle,
		multilocation: Location,
	) -> EvmResult<(Weight, Weight, Weight)> {
		// fetch data from pallet
		// storage item: TransactInfoWithWeightLimit: Blake2_128(16) + Location
		// + RemoteTransactInfoWithMaxWeight
		handle.record_db_read::<Runtime>(
			16 + Location::max_encoded_len() + RemoteTransactInfoWithMaxWeight::max_encoded_len(),
		)?;

		let remote_transact_info: RemoteTransactInfoWithMaxWeight =
			pallet_xcm_transactor::Pallet::<Runtime>::transact_info(multilocation)
				.ok_or(revert("Transact Info not set"))?;

		let transact_extra_weight_signed = remote_transact_info
			.transact_extra_weight_signed
			.unwrap_or(Weight::zero());

		Ok((
			remote_transact_info.transact_extra_weight,
			transact_extra_weight_signed,
			remote_transact_info.max_weight,
		))
	}

	pub(crate) fn transact_through_derivative_multilocation_v3(
		handle: &mut impl PrecompileHandle,
		transactor: u8,
		index: u16,
		fee_asset: Location,
		weight: Weight,
		inner_call: BoundedBytes<GetDataLimit>,
		fee_amount: u128,
		overall_weight: Weight,
		refund: bool,
	) -> EvmResult {
		let transactor = transactor
			.try_into()
			.map_err(|_| RevertReason::custom("Non-existent transactor").in_field("transactor"))?;

		let inner_call: Vec<_> = inner_call.into();

		let overall_weight_limit = match overall_weight.ref_time() {
			u64::MAX => Unlimited,
			_ => Limited(overall_weight),
		};

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_derivative {
			dest: transactor,
			index,
			fee: CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					fee_asset,
				))),
				fee_amount: Some(fee_amount),
			},
			inner_call,
			weight_info: TransactWeights {
				transact_required_weight_at_most: weight,
				overall_weight: Some(overall_weight_limit),
			},
			refund,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	pub(crate) fn transact_through_derivative_v3(
		handle: &mut impl PrecompileHandle,
		transactor: u8,
		index: u16,
		fee_asset: Address,
		weight: Weight,
		inner_call: BoundedBytes<GetDataLimit>,
		fee_amount: u128,
		overall_weight: Weight,
		refund: bool,
	) -> EvmResult {
		// No DB access before try_dispatch but lot of logical stuff
		// To prevent spam, we charge an arbitrary amoun of gas
		handle.record_cost(1000)?;

		let transactor = transactor
			.try_into()
			.map_err(|_| RevertReason::custom("Non-existent transactor").in_field("transactor"))?;
		let inner_call: Vec<_> = inner_call.into();

		let to_address: H160 = fee_asset.into();
		let to_account = Runtime::AddressMapping::into_account_id(to_address);

		// We convert the address into a currency
		let currency_id: <Runtime as pallet_xcm_transactor::Config>::CurrencyId =
			Runtime::account_to_currency_id(to_account)
				.ok_or(revert("cannot convert into currency id"))?;

		let overall_weight_limit = match overall_weight.ref_time() {
			u64::MAX => Unlimited,
			_ => Limited(overall_weight),
		};

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
				overall_weight: Some(overall_weight_limit),
			},
			inner_call,
			refund,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	pub(crate) fn transact_through_signed_multilocation_v3(
		handle: &mut impl PrecompileHandle,
		dest: Location,
		fee_asset: Location,
		weight: Weight,
		call: BoundedBytes<GetDataLimit>,
		fee_amount: u128,
		overall_weight: Weight,
		refund: bool,
	) -> EvmResult {
		let call: Vec<_> = call.into();

		let overall_weight_limit = match overall_weight.ref_time() {
			u64::MAX => Unlimited,
			_ => Limited(overall_weight),
		};

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_signed {
			dest: Box::new(xcm::VersionedLocation::V4(dest)),
			fee: CurrencyPayment {
				currency: Currency::AsMultiLocation(Box::new(xcm::VersionedLocation::V4(
					fee_asset,
				))),
				fee_amount: Some(fee_amount),
			},
			weight_info: TransactWeights {
				transact_required_weight_at_most: weight,
				overall_weight: Some(overall_weight_limit),
			},
			refund,
			call,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	pub(crate) fn transact_through_signed_v3(
		handle: &mut impl PrecompileHandle,
		dest: Location,
		fee_asset: Address,
		weight: Weight,
		call: BoundedBytes<GetDataLimit>,
		fee_amount: u128,
		overall_weight: Weight,
		refund: bool,
	) -> EvmResult {
		// No DB access before try_dispatch but lot of logical stuff
		// To prevent spam, we charge an arbitrary amoun of gas
		handle.record_cost(1000)?;

		let to_address: H160 = fee_asset.into();
		let to_account = Runtime::AddressMapping::into_account_id(to_address);

		let call: Vec<_> = call.into();

		// We convert the address into a currency
		let currency_id: <Runtime as pallet_xcm_transactor::Config>::CurrencyId =
			Runtime::account_to_currency_id(to_account)
				.ok_or(revert("cannot convert into currency id"))?;

		let overall_weight_limit = match overall_weight.ref_time() {
			u64::MAX => Unlimited,
			_ => Limited(overall_weight),
		};

		// Depending on the Runtime, this might involve a DB read. This is not the case in
		// moonbeam, as we are using IdentityMapping
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_xcm_transactor::Call::<Runtime>::transact_through_signed {
			dest: Box::new(xcm::VersionedLocation::V4(dest)),
			fee: CurrencyPayment {
				currency: Currency::AsCurrencyId(currency_id),
				fee_amount: Some(fee_amount),
			},
			weight_info: TransactWeights {
				transact_required_weight_at_most: weight,
				overall_weight: Some(overall_weight_limit),
			},
			refund,
			call,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}
}
