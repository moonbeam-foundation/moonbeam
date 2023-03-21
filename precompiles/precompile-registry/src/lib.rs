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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use core::marker::PhantomData;
use fp_evm::PrecompileSet;
use precompile_utils::{precompile_set::IsActivePrecompile, prelude::*};
use sp_core::Get;

const DUMMY_CODE: [u8; 5] = [0x60, 0x00, 0x60, 0x00, 0xfd];

pub struct PrecompileRegistry<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> PrecompileRegistry<Runtime>
where
	Runtime: pallet_evm::Config,
	Runtime::PrecompilesType: IsActivePrecompile,
{
	#[precompile::public("isPrecompile(address)")]
	#[precompile::view]
	fn is_precompile(handle: &mut impl PrecompileHandle, address: Address) -> EvmResult<bool> {
		// We consider the precompile set is optimized to do at most one storage read.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let active = <Runtime::PrecompilesValue>::get().is_precompile(address.0);
		Ok(active)
	}

	#[precompile::public("isActivePrecompile(address)")]
	#[precompile::view]
	fn is_active_precompile(
		handle: &mut impl PrecompileHandle,
		address: Address,
	) -> EvmResult<bool> {
		// We consider the precompile set is optimized to do at most one storage read.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let active = <Runtime::PrecompilesValue>::get().is_active_precompile(address.0);
		Ok(active)
	}

	#[precompile::public("updateAccountCode(address)")]
	fn update_account_code(handle: &mut impl PrecompileHandle, address: Address) -> EvmResult<()> {
		// We consider the precompile set is optimized to do at most one storage read.
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// We will write into the account code.
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		// Prevent touching addresses that are not precompiles.
		if !<Runtime::PrecompilesValue>::get().is_precompile(address.0) {
			return Err(revert("provided address is not a precompile"));
		}

		pallet_evm::Pallet::<Runtime>::create_account(address.0, DUMMY_CODE.to_vec());

		Ok(())
	}
}
