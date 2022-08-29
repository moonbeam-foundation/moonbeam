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

//! Precompile to xcm utils runtime methods via the EVM

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(assert_matches)]

use fp_evm::PrecompileHandle;
use frame_support::codec::{Decode, DecodeLimit as _};
use frame_support::dispatch::Dispatchable;
use frame_support::pallet_prelude::ConstU32;
use frame_support::traits::OriginTrait;
use frame_support::weights::{GetDispatchInfo, PostDispatchInfo};
use pallet_evm::AddressMapping;
use pallet_evm::PrecompileOutput;
use precompile_utils::prelude::*;
use sp_core::H160;
use sp_std::{fmt::Debug, marker::PhantomData};
use xcm::latest::{MultiLocation, OriginKind};
use xcm_executor::traits::ConvertOrigin;
pub type XcmOriginOf<XcmConfig> =
	<<XcmConfig as xcm_executor::Config>::Call as Dispatchable>::Origin;
pub type XcmAccountIdOf<XcmConfig> =
	<<<XcmConfig as xcm_executor::Config>::Call as Dispatchable>::Origin as OriginTrait>::AccountId;

pub type SystemCallOf<Runtime> = <Runtime as frame_system::Config>::Call;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type GetProposalLimit = ConstU32<{ 2u32.pow(16) }>;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	MultiLocationToAddress = "multilocationToAddress((uint8,bytes[]))",
	XcmExecute = "execute(bytes, uint64)",
}

/// A precompile to wrap the functionality from xcm-utils
pub struct XcmUtilsWrapper<Runtime, XcmConfig>(PhantomData<(Runtime, XcmConfig)>);

impl<Runtime, XcmConfig> pallet_evm::Precompile for XcmUtilsWrapper<Runtime, XcmConfig>
where
	Runtime: pallet_evm::Config + frame_system::Config + pallet_xcm::Config,
	XcmOriginOf<XcmConfig>: OriginTrait,
	XcmAccountIdOf<XcmConfig>: Into<H160>,
	XcmConfig: xcm_executor::Config,
	SystemCallOf<Runtime>: Dispatchable<PostInfo = PostDispatchInfo> + Decode + GetDispatchInfo,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::Call: From<pallet_xcm::Call<Runtime>>,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let selector = handle.read_selector()?;

		handle.check_function_modifier(match selector {
			Action::MultiLocationToAddress => FunctionModifier::View,
			Action::XcmExecute => FunctionModifier::NonPayable,
		})?;

		match selector {
			// Check for accessor methods first. These return results immediately
			Action::MultiLocationToAddress => Self::multilocation_to_address(handle),
			Action::XcmExecute => Self::xcm_execute(handle),
		}
	}
}

impl<Runtime, XcmConfig> XcmUtilsWrapper<Runtime, XcmConfig>
where
	Runtime: pallet_evm::Config + frame_system::Config + pallet_xcm::Config,
	XcmOriginOf<XcmConfig>: OriginTrait,
	XcmAccountIdOf<XcmConfig>: Into<H160>,
	XcmConfig: xcm_executor::Config,
	SystemCallOf<Runtime>: Dispatchable<PostInfo = PostDispatchInfo> + Decode + GetDispatchInfo,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin:
		From<Option<Runtime::AccountId>>,
	<Runtime as frame_system::Config>::Call: From<pallet_xcm::Call<Runtime>>,
{
	fn multilocation_to_address(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		// TODO: Change once precompiles are benchmarked
		// for now we charge a db read,
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		read_args!(handle, { multilocation: MultiLocation });

		let origin =
			XcmConfig::OriginConverter::convert_origin(multilocation, OriginKind::SovereignAccount)
				.map_err(|_| {
					RevertReason::custom("Failed multilocation conversion")
						.in_field("multilocation")
				})?;

		let account: H160 = origin
			.as_signed()
			.ok_or(
				RevertReason::custom("Failed multilocation conversion").in_field("multilocation"),
			)?
			.into();
		Ok(succeed(
			EvmDataWriter::new().write(Address(account)).build(),
		))
	}

	fn xcm_execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { message: BoundedBytes<GetProposalLimit>, max_weight: u64 });

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

		let message: Vec<_> = message.into_vec();
		let xcm = xcm::VersionedXcm::<SystemCallOf<Runtime>>::decode_all_with_depth_limit(
			xcm::MAX_XCM_DECODE_DEPTH,
			&mut message.as_slice(),
		)
		.map_err(|_e| RevertReason::custom("Failed xcm decoding").in_field("message"))?;

		let call = pallet_xcm::Call::<Runtime>::execute {
			message: Box::new(xcm),
			max_weight: max_weight,
		};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call)?;

		Ok(succeed([]))
	}
}
