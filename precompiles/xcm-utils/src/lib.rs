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
use frame_support::dispatch::Dispatchable;
use frame_support::traits::OriginTrait;
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

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	MultiLocationToAddress = "multilocationToAddress((uint8,bytes[]))",
}

/// A precompile to wrap the functionality from xcm-utils
pub struct XcmUtilsWrapper<Runtime, XcmConfig>(PhantomData<(Runtime, XcmConfig)>);

impl<Runtime, XcmConfig> pallet_evm::Precompile for XcmUtilsWrapper<Runtime, XcmConfig>
where
	Runtime: pallet_evm::Config + frame_system::Config,
	XcmOriginOf<XcmConfig>: OriginTrait,
	XcmAccountIdOf<XcmConfig>: Into<H160>,
	XcmConfig: xcm_executor::Config,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let selector = handle.read_selector()?;

		handle.check_function_modifier(match selector {
			Action::MultiLocationToAddress => FunctionModifier::View,
		})?;

		match selector {
			// Check for accessor methods first. These return results immediately
			Action::MultiLocationToAddress => Self::multilocation_to_address(handle),
		}
	}
}

impl<Runtime, XcmConfig> XcmUtilsWrapper<Runtime, XcmConfig>
where
	Runtime: pallet_evm::Config + frame_system::Config,
	XcmOriginOf<XcmConfig>: OriginTrait,
	XcmAccountIdOf<XcmConfig>: Into<H160>,
	XcmConfig: xcm_executor::Config,
{
	fn multilocation_to_address(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Bound check
		let mut input = handle.read_input()?;
		input.expect_arguments(1)?;
		let multilocation: MultiLocation = input.read::<MultiLocation>()?;

		let origin =
			XcmConfig::OriginConverter::convert_origin(multilocation, OriginKind::SovereignAccount)
				.map_err(|_| revert("Failed multilocation conversion"))?;
		let account: H160 = origin
			.as_signed()
			.ok_or(revert("Failed multilocation conversion"))?
			.into();
		Ok(succeed(
			EvmDataWriter::new().write(Address(account)).build(),
		))
	}
}
