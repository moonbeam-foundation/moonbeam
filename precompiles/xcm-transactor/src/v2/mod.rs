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

use crate::functions::{CurrencyIdOf, GetDataLimit, TransactorOf, XcmTransactorWrapper};
use fp_evm::PrecompileHandle;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use precompile_utils::prelude::*;
use sp_core::{H160, U256};
use sp_std::{convert::TryFrom, marker::PhantomData};
use xcm::latest::MultiLocation;
use xcm_primitives::AccountIdToCurrencyId;

/// A precompile to wrap the functionality from xcm transactor
pub struct XcmTransactorPrecompileV2<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> XcmTransactorPrecompileV2<Runtime>
where
	Runtime: pallet_xcm_transactor::Config + pallet_evm::Config + frame_system::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_xcm_transactor::Call<Runtime>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	TransactorOf<Runtime>: TryFrom<u8>,
	Runtime::AccountId: Into<H160>,
	Runtime: AccountIdToCurrencyId<Runtime::AccountId, CurrencyIdOf<Runtime>>,
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
		multilocation: MultiLocation,
	) -> EvmResult<(u64, u64, u64)> {
		XcmTransactorWrapper::<Runtime>::transact_info_with_signed(handle, multilocation)
	}

	#[precompile::public("feePerSecond((uint8,bytes[]))")]
	#[precompile::view]
	fn fee_per_second(
		handle: &mut impl PrecompileHandle,
		multilocation: MultiLocation,
	) -> EvmResult<U256> {
		XcmTransactorWrapper::<Runtime>::fee_per_second(handle, multilocation)
	}

	#[precompile::public(
		"transactThroughDerivativeMultilocation(\
		uint8,\
		uint16,\
		(uint8,bytes[]),\
		uint64,bytes,\
		uint256,\
		uint64)"
	)]
	fn transact_through_derivative_multilocation(
		handle: &mut impl PrecompileHandle,
		transactor: u8,
		index: u16,
		fee_asset: MultiLocation,
		weight: u64,
		inner_call: BoundedBytes<GetDataLimit>,
		fee_amount: SolidityConvert<U256, u128>,
		overall_weight: u64,
	) -> EvmResult {
		XcmTransactorWrapper::<Runtime>::transact_through_derivative_multilocation_fee_weight(
			handle,
			transactor,
			index,
			fee_asset,
			weight,
			inner_call,
			fee_amount.converted(),
			overall_weight,
		)
	}

	#[precompile::public(
		"transactThroughDerivative(\
		uint8,\
		uint16,\
		address,\
		uint64,\
		bytes,\
		uint256,\
		uint64)"
	)]
	fn transact_through_derivative(
		handle: &mut impl PrecompileHandle,
		transactor: u8,
		index: u16,
		fee_asset: Address,
		weight: u64,
		inner_call: BoundedBytes<GetDataLimit>,
		fee_amount: SolidityConvert<U256, u128>,
		overall_weight: u64,
	) -> EvmResult {
		XcmTransactorWrapper::<Runtime>::transact_through_derivative_fee_weight(
			handle,
			transactor,
			index,
			fee_asset,
			weight,
			inner_call,
			fee_amount.converted(),
			overall_weight,
		)
	}

	#[precompile::public(
		"transactThroughSignedMultilocation(\
		(uint8,bytes[]),\
		(uint8,bytes[]),\
		uint64,\
		bytes,\
		uint256,\
		uint64)"
	)]
	fn transact_through_signed_multilocation(
		handle: &mut impl PrecompileHandle,
		dest: MultiLocation,
		fee_asset: MultiLocation,
		weight: u64,
		call: BoundedBytes<GetDataLimit>,
		fee_amount: SolidityConvert<U256, u128>,
		overall_weight: u64,
	) -> EvmResult {
		XcmTransactorWrapper::<Runtime>::transact_through_signed_multilocation_fee_weight(
			handle,
			dest,
			fee_asset,
			weight,
			call,
			fee_amount.converted(),
			overall_weight,
		)
	}

	#[precompile::public(
		"transactThroughSigned((uint8,bytes[]),address,uint64,bytes,uint256,uint64)"
	)]
	fn transact_through_signed(
		handle: &mut impl PrecompileHandle,
		dest: MultiLocation,
		fee_asset: Address,
		weight: u64,
		call: BoundedBytes<GetDataLimit>,
		fee_amount: SolidityConvert<U256, u128>,
		overall_weight: u64,
	) -> EvmResult {
		XcmTransactorWrapper::<Runtime>::transact_through_signed_fee_weight(
			handle,
			dest,
			fee_asset,
			weight,
			call,
			fee_amount.converted(),
			overall_weight,
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
