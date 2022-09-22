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
use frame_support::{dispatch::Dispatchable, traits::OriginTrait};
use precompile_utils::prelude::*;
use sp_core::H160;
use sp_std::marker::PhantomData;
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

/// A precompile to wrap the functionality from xcm-utils
pub struct XcmUtilsPrecompile<Runtime, XcmConfig>(PhantomData<(Runtime, XcmConfig)>);

#[precompile_utils::precompile]
impl<Runtime, XcmConfig> XcmUtilsPrecompile<Runtime, XcmConfig>
where
	Runtime: pallet_evm::Config + frame_system::Config,
	XcmOriginOf<XcmConfig>: OriginTrait,
	XcmAccountIdOf<XcmConfig>: Into<H160>,
	XcmConfig: xcm_executor::Config,
{
	#[precompile::public("multilocationToAddress((uint8,bytes[]))")]
	#[precompile::view]
	fn multilocation_to_address(
		handle: &mut impl PrecompileHandle,
		multilocation: MultiLocation,
	) -> EvmResult<Address> {
		// TODO: Change once precompiles are benchmarked
		// for now we charge a db read,
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

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
		Ok(Address(account))
	}
}
