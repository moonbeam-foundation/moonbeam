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

//! Utils related to Substrate features:
//! - Substrate call dispatch.
//! - Substrate DB read and write costs

use {
	crate::{revert, EvmResult},
	core::marker::PhantomData,
	fp_evm::{ExitError, PrecompileFailure, PrecompileHandle},
	frame_support::{
		dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
		traits::Get,
	},
	pallet_evm::GasWeightMapping,
};

/// Helper functions requiring a Substrate runtime.
/// This runtime must of course implement `pallet_evm::Config`.
#[derive(Clone, Copy, Debug)]
pub struct RuntimeHelper<Runtime>(PhantomData<Runtime>);

impl<Runtime> RuntimeHelper<Runtime>
where
	Runtime: pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
{
	/// Try to dispatch a Substrate call.
	/// Return an error if there are not enough gas, or if the call fails.
	/// If successful returns the used gas using the Runtime GasWeightMapping.
	pub fn try_dispatch<Call>(
		handle: &mut impl PrecompileHandle,
		origin: <Runtime::Call as Dispatchable>::Origin,
		call: Call,
	) -> EvmResult<()>
	where
		Runtime::Call: From<Call>,
	{
		let call = Runtime::Call::from(call);
		let dispatch_info = call.get_dispatch_info();

		// Make sure there is enough gas.
		let remaining_gas = handle.remaining_gas();
		let required_gas = Runtime::GasWeightMapping::weight_to_gas(dispatch_info.weight);
		if required_gas > remaining_gas {
			return Err(PrecompileFailure::Error {
				exit_status: ExitError::OutOfGas,
			});
		}

		// Dispatch call.
		// It may be possible to not record gas cost if the call returns Pays::No.
		// However while Substrate handle checking weight while not making the sender pay for it,
		// the EVM doesn't. It seems this safer to always record the costs to avoid unmetered
		// computations.
		let used_weight = call
			.dispatch(origin)
			.map_err(|e| revert(alloc::format!("Dispatched call failed with error: {:?}", e)))?
			.actual_weight;

		let used_gas =
			Runtime::GasWeightMapping::weight_to_gas(used_weight.unwrap_or(dispatch_info.weight));

		handle.record_cost(used_gas)?;

		Ok(())
	}
}

impl<Runtime> RuntimeHelper<Runtime>
where
	Runtime: pallet_evm::Config,
{
	/// Cost of a Substrate DB write in gas.
	pub fn db_write_gas_cost() -> u64 {
		<Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().write,
		)
	}

	/// Cost of a Substrate DB read in gas.
	pub fn db_read_gas_cost() -> u64 {
		<Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		)
	}
}
