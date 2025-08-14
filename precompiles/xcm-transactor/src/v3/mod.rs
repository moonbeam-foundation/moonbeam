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

//! Precompile to xcm transactor runtime methods via the EVM

use crate::functions::{CurrencyIdOf, GetDataLimit, TransactorOf, XcmTransactorWrapper};
use fp_evm::PrecompileHandle;
use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{H160, U256};
use sp_runtime::traits::Dispatchable;
use sp_std::{convert::TryFrom, marker::PhantomData};
use sp_weights::Weight;
use xcm::latest::Location;
use xcm_primitives::AccountIdToCurrencyId;

/// A precompile to wrap the functionality from xcm transactor
pub struct XcmTransactorPrecompileV3<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> XcmTransactorPrecompileV3<Runtime>
where
	Runtime: pallet_xcm_transactor::Config + pallet_evm::Config + frame_system::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_xcm_transactor::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	TransactorOf<Runtime>: TryFrom<u8>,
	Runtime::AccountId: Into<H160>,
	Runtime: AccountIdToCurrencyId<Runtime::AccountId, CurrencyIdOf<Runtime>>,
	<Runtime as pallet_evm::Config>::AddressMapping: AddressMapping<Runtime::AccountId>,
{
	#[precompile::public("indexToAccount(uint16)")]
	#[precompile::view]
	fn index_to_account(handle: &mut impl PrecompileHandle, index: u16) -> EvmResult<Address> {
		XcmTransactorWrapper::<Runtime>::account_index(handle, index)
	}

	#[precompile::public("transactInfoWithSigned((uint8,bytes[]))")]
	#[precompile::view]
	fn transact_info_with_signed(
		handle: &mut impl PrecompileHandle,
		location: Location,
	) -> EvmResult<(Weight, Weight, Weight)> {
		XcmTransactorWrapper::<Runtime>::transact_info_with_signed_v3(handle, location)
	}

	#[precompile::public("feePerSecond((uint8,bytes[]))")]
	#[precompile::view]
	fn fee_per_second(handle: &mut impl PrecompileHandle, location: Location) -> EvmResult<U256> {
		XcmTransactorWrapper::<Runtime>::fee_per_second(handle, location)
	}

	#[precompile::public(
		"transactThroughDerivativeMultilocation(\
		uint8,\
		uint16,\
		(uint8,bytes[]),\
		(uint64,uint64),\
		bytes,\
		uint256,\
		(uint64,uint64),\
		bool)"
	)]
	fn transact_through_derivative_multilocation(
		handle: &mut impl PrecompileHandle,
		transactor: u8,
		index: u16,
		fee_asset: Location,
		weight: Weight,
		inner_call: BoundedBytes<GetDataLimit>,
		fee_amount: Convert<U256, u128>,
		overall_weight: Weight,
		refund: bool,
	) -> EvmResult {
		XcmTransactorWrapper::<Runtime>::transact_through_derivative_multilocation_v3(
			handle,
			transactor,
			index,
			fee_asset,
			weight,
			inner_call,
			fee_amount.converted(),
			overall_weight,
			refund,
		)
	}

	#[precompile::public(
		"transactThroughDerivative(\
		uint8,\
		uint16,\
		address,\
		(uint64,uint64),\
		bytes,\
		uint256,\
		(uint64,uint64),\
		bool)"
	)]
	fn transact_through_derivative(
		handle: &mut impl PrecompileHandle,
		transactor: u8,
		index: u16,
		fee_asset: Address,
		weight: Weight,
		inner_call: BoundedBytes<GetDataLimit>,
		fee_amount: Convert<U256, u128>,
		overall_weight: Weight,
		refund: bool,
	) -> EvmResult {
		XcmTransactorWrapper::<Runtime>::transact_through_derivative_v3(
			handle,
			transactor,
			index,
			fee_asset,
			weight,
			inner_call,
			fee_amount.converted(),
			overall_weight,
			refund,
		)
	}

	#[precompile::public(
		"transactThroughSignedMultilocation(\
		(uint8,bytes[]),\
		(uint8,bytes[]),\
		(uint64,uint64),\
		bytes,\
		uint256,\
		(uint64,uint64),\
		bool)"
	)]
	fn transact_through_signed_multilocation(
		handle: &mut impl PrecompileHandle,
		dest: Location,
		fee_asset: Location,
		weight: Weight,
		call: BoundedBytes<GetDataLimit>,
		fee_amount: Convert<U256, u128>,
		overall_weight: Weight,
		refund: bool,
	) -> EvmResult {
		XcmTransactorWrapper::<Runtime>::transact_through_signed_multilocation_v3(
			handle,
			dest,
			fee_asset,
			weight,
			call,
			fee_amount.converted(),
			overall_weight,
			refund,
		)
	}

	#[precompile::public(
		"transactThroughSigned(\
		(uint8,bytes[]),\
		address,\
		(uint64,uint64),\
		bytes,\
		uint256,\
		(uint64,uint64),\
		bool)"
	)]
	fn transact_through_signed(
		handle: &mut impl PrecompileHandle,
		dest: Location,
		fee_asset: Address,
		weight: Weight,
		call: BoundedBytes<GetDataLimit>,
		fee_amount: Convert<U256, u128>,
		overall_weight: Weight,
		refund: bool,
	) -> EvmResult {
		XcmTransactorWrapper::<Runtime>::transact_through_signed_v3(
			handle,
			dest,
			fee_asset,
			weight,
			call,
			fee_amount.converted(),
			overall_weight,
			refund,
		)
	}

	#[precompile::public("encodeUtilityAsDerivative(uint8,uint16,bytes)")]
	#[precompile::public("encode_utility_as_derivative(uint8,uint16,bytes)")]
	#[precompile::view]
	fn encode_utility_as_derivative(
		handle: &mut impl PrecompileHandle,
		transactor: u8,
		index: u16,
		inner_call: BoundedBytes<GetDataLimit>,
	) -> EvmResult<UnboundedBytes> {
		XcmTransactorWrapper::<Runtime>::encode_utility_as_derivative(
			handle, transactor, index, inner_call,
		)
	}
}
