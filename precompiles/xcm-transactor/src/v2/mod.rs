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

use crate::functions::{CurrencyIdOf, TransactorOf, XcmTransactorWrapper};
use fp_evm::PrecompileHandle;
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::PrecompileOutput;
use precompile_utils::prelude::*;
use sp_core::H160;
use sp_std::{convert::TryFrom, fmt::Debug, marker::PhantomData};
use xcm_primitives::AccountIdToCurrencyId;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	IndexToAccount = "indexToAccount(uint16)",
	TransactInfoWithSigned = "transactInfoWithSigned((uint8,bytes[]))",
	FeePerSecond = "feePerSecond((uint8,bytes[]))",
	TransactThroughDerivative =
		"transactThroughDerivative(uint8,uint16,address,uint64,bytes,uint256,uint64)",
	TransactThroughDerivativeMultiLocation = "transactThroughDerivativeMultilocation(\
		uint8,\
		uint16,\
		(uint8,bytes[]),\
		uint64,bytes,\
		uint256,\
		uint64\
	)",
	TransactThroughSignedMultiLocation = "transactThroughSignedMultilocation(\
		(uint8,bytes[]),\
		(uint8,bytes[]),\
		uint64,\
		bytes,\
		uint256,\
		uint64\
	)",
	TransactThroughSigned =
		"transactThroughSigned((uint8,bytes[]),address,uint64,bytes,uint256,uint64)",
}

/// A precompile to wrap the functionality from xcm transactor
pub struct XcmTransactorWrapperV2<Runtime>(PhantomData<Runtime>);

impl<Runtime> pallet_evm::Precompile for XcmTransactorWrapperV2<Runtime>
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
			| Action::TransactThroughSigned => FunctionModifier::NonPayable,
			_ => FunctionModifier::View,
		})?;

		match selector {
			// Check for accessor methods first. These return results immediately
			Action::IndexToAccount => {
				XcmTransactorWrapper::<Runtime>::account_index(handle)
			}
			Action::TransactInfoWithSigned => {
				XcmTransactorWrapper::<Runtime>::transact_info_with_signed(handle)
			}
			Action::FeePerSecond => XcmTransactorWrapper::<Runtime>::fee_per_second(handle),
			Action::TransactThroughDerivativeMultiLocation => {
				XcmTransactorWrapper::<Runtime>::transact_through_derivative_multilocation_fee_weight(handle)
			}
			Action::TransactThroughDerivative => {
				XcmTransactorWrapper::<Runtime>::transact_through_derivative_fee_weight(handle)
			}
			Action::TransactThroughSignedMultiLocation => {
				XcmTransactorWrapper::<Runtime>::transact_through_signed_multilocation_fee_weight(handle)
			}
			Action::TransactThroughSigned => {
				XcmTransactorWrapper::<Runtime>::transact_through_signed_fee_weight(handle)
			}
		}
	}
}
