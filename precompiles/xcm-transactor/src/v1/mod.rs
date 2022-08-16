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

use fp_evm::PrecompileHandle;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::PrecompileOutput;

use crate::functions::{CurrencyIdOf, TransactorOf, XcmTransactorWrapper};
use precompile_utils::prelude::*;
use sp_core::H160;
use sp_std::{convert::TryFrom, fmt::Debug, marker::PhantomData};
use xcm_primitives::AccountIdToCurrencyId;

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
	TransactThroughDerivativeCustomFeeAndWeight =
		"transactThroughDerivative(uint8,uint16,address,uint64,bytes,uint256,uint64)",
	TransactThroughDerivativeMultiLocationCustomFeeAndWeight =
		"transactThroughDerivativeMultilocation(\
		uint8,\
		uint16,\
		(uint8,bytes[]),\
		uint64,bytes,\
		uint256,\
		uint64\
	)",
	TransactThroughSignedMultiLocationCustomFeeAndWeight = "transactThroughSignedMultilocation(\
		(uint8,bytes[]),\
		(uint8,bytes[]),\
		uint64,\
		bytes,\
		uint256,\
		uint64\
	)",
	TransactThroughSignedCustomFeeAndWeight =
		"transactThroughSigned((uint8,bytes[]),address,uint64,bytes,uint256,uint64)",

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
pub struct XcmTransactorWrapperV1<Runtime>(PhantomData<Runtime>);

impl<Runtime> pallet_evm::Precompile for XcmTransactorWrapperV1<Runtime>
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
			| Action::DeprecatedTransactThroughSigned
			| Action::TransactThroughSignedMultiLocationCustomFeeAndWeight
			| Action::TransactThroughSignedCustomFeeAndWeight
			| Action::TransactThroughDerivativeCustomFeeAndWeight
			| Action::TransactThroughDerivativeMultiLocationCustomFeeAndWeight => {
				FunctionModifier::NonPayable
			}
			_ => FunctionModifier::View,
		})?;

		match selector {
			// Check for accessor methods first. These return results immediately
			Action::IndexToAccount | Action::DeprecatedIndexToAccount => {
				XcmTransactorWrapper::<Runtime>::account_index(handle)
			}
			// DEPRECATED
			Action::TransactInfo | Action::DeprecatedTransactInfo => XcmTransactorWrapper::<Runtime>::transact_info(handle),
			Action::TransactInfoWithSigned | Action::DeprecatedTransactInfoWithSigned => {
				XcmTransactorWrapper::<Runtime>::transact_info_with_signed(handle)
			}
			Action::FeePerSecond | Action::DeprecatedFeePerSecond => XcmTransactorWrapper::<Runtime>::fee_per_second(handle),
			Action::TransactThroughDerivativeMultiLocation
			| Action::DeprecatedTransactThroughDerivativeMultiLocation => {
				XcmTransactorWrapper::<Runtime>::transact_through_derivative_multilocation(handle)
			}
			Action::TransactThroughDerivative | Action::DeprecatedTransactThroughDerivative => {
				XcmTransactorWrapper::<Runtime>::transact_through_derivative(handle)
			}
			Action::TransactThroughSignedMultiLocation
			| Action::DeprecatedTransactThroughSignedMultiLocation => {
				XcmTransactorWrapper::<Runtime>::transact_through_signed_multilocation(handle)
			}
			Action::TransactThroughSigned | Action::DeprecatedTransactThroughSigned => {
				XcmTransactorWrapper::<Runtime>::transact_through_signed(handle)
			}
			Action::TransactThroughDerivativeCustomFeeAndWeight => {
				XcmTransactorWrapper::<Runtime>::transact_through_derivative_custom_fee_and_weight(handle)
			}
			Action::TransactThroughDerivativeMultiLocationCustomFeeAndWeight => {
				XcmTransactorWrapper::<Runtime>::transact_through_derivative_multilocation_custom_fee_and_weight(handle)
			}
			Action::TransactThroughSignedMultiLocationCustomFeeAndWeight => {
				XcmTransactorWrapper::<Runtime>::transact_through_signed_multilocation_custom_fee_and_weight(handle)
			}
			Action::TransactThroughSignedCustomFeeAndWeight => {
				XcmTransactorWrapper::<Runtime>::transact_through_signed_custom_fee_and_weight(handle)
			}
		}
	}
}
