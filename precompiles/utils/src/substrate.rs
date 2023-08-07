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
	crate::{evm::handle::using_precompile_handle, solidity::revert::revert},
	core::marker::PhantomData,
	fp_evm::{ExitError, PrecompileFailure, PrecompileHandle},
	frame_support::{
		dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
		pallet_prelude::*,
		traits::Get,
	},
	pallet_evm::GasWeightMapping,
};

#[derive(Debug)]
pub enum TryDispatchError {
	Evm(ExitError),
	Substrate(DispatchError),
}

impl From<TryDispatchError> for PrecompileFailure {
	fn from(f: TryDispatchError) -> PrecompileFailure {
		match f {
			TryDispatchError::Evm(e) => PrecompileFailure::Error { exit_status: e },
			TryDispatchError::Substrate(e) => {
				revert(alloc::format!("Dispatched call failed with error: {e:?}"))
			}
		}
	}
}

/// Helper functions requiring a Substrate runtime.
/// This runtime must of course implement `pallet_evm::Config`.
#[derive(Clone, Copy, Debug)]
pub struct RuntimeHelper<Runtime>(PhantomData<Runtime>);

impl<Runtime> RuntimeHelper<Runtime>
where
	Runtime: pallet_evm::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
{
	#[inline(always)]
	pub fn record_weight_v2_cost(
		handle: &mut impl PrecompileHandle,
		weight: Weight,
	) -> Result<(), ExitError> {
		// Make sure there is enough gas.
		let remaining_gas = handle.remaining_gas();
		let required_gas = Runtime::GasWeightMapping::weight_to_gas(weight);
		if required_gas > remaining_gas {
			return Err(ExitError::OutOfGas);
		}

		// Make sure there is enough remaining weight
		// TODO: record ref time when precompile will be benchmarked
		handle.record_external_cost(None, Some(weight.proof_size()))
	}

	#[inline(always)]
	pub fn refund_weight_v2_cost(
		handle: &mut impl PrecompileHandle,
		weight: Weight,
		maybe_actual_weight: Option<Weight>,
	) -> Result<u64, ExitError> {
		// Refund weights and compute used weight them record used gas
		// TODO: refund ref time when precompile will be benchmarked
		let used_weight = if let Some(actual_weight) = maybe_actual_weight {
			let refund_weight = weight.checked_sub(&actual_weight).unwrap_or_default();
			handle.refund_external_cost(None, Some(refund_weight.proof_size()));
			actual_weight
		} else {
			weight
		};
		let used_gas = Runtime::GasWeightMapping::weight_to_gas(used_weight);
		handle.record_cost(used_gas)?;
		Ok(used_gas)
	}

	/// Try to dispatch a Substrate call.
	/// Return an error if there are not enough gas, or if the call fails.
	/// If successful returns the used gas using the Runtime GasWeightMapping.
	pub fn try_dispatch<Call>(
		handle: &mut impl PrecompileHandle,
		origin: <Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin,
		call: Call,
	) -> Result<PostDispatchInfo, TryDispatchError>
	where
		Runtime::RuntimeCall: From<Call>,
	{
		let call = Runtime::RuntimeCall::from(call);
		let dispatch_info = call.get_dispatch_info();

		Self::record_weight_v2_cost(handle, dispatch_info.weight)
			.map_err(|e| TryDispatchError::Evm(e))?;

		// Dispatch call.
		// It may be possible to not record gas cost if the call returns Pays::No.
		// However while Substrate handle checking weight while not making the sender pay for it,
		// the EVM doesn't. It seems this safer to always record the costs to avoid unmetered
		// computations.
		let post_dispatch_info = using_precompile_handle(handle, || call.dispatch(origin))
			.map_err(|e| TryDispatchError::Substrate(e.error))?;

		Self::refund_weight_v2_cost(
			handle,
			dispatch_info.weight,
			post_dispatch_info.actual_weight,
		)
		.map_err(|e| TryDispatchError::Evm(e))?;

		Ok(post_dispatch_info)
	}
}

impl<Runtime> RuntimeHelper<Runtime>
where
	Runtime: pallet_evm::Config,
{
	/// Cost of a Substrate DB write in gas.
	pub fn db_write_gas_cost() -> u64 {
		<Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().writes(1),
		)
	}

	/// Cost of a Substrate DB read in gas.
	pub fn db_read_gas_cost() -> u64 {
		<Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().reads(1),
		)
	}
}
