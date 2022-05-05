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
#![feature(assert_matches)]

extern crate alloc;

use crate::alloc::borrow::ToOwned;
use fp_evm::{
	Context, ExitError, ExitReason, ExitRevert, PrecompileFailure, PrecompileHandle, Transfer,
};
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::Get,
};
use pallet_evm::{GasWeightMapping, Log};
use sp_core::{H160, H256, U256};
use sp_std::{marker::PhantomData, vec, vec::Vec};

mod data;

pub use data::{Address, Bytes, EvmData, EvmDataReader, EvmDataWriter};
pub use precompile_utils_macro::{generate_function_selector, keccak256};

pub mod testing;

#[cfg(test)]
mod tests;

/// Alias for Result returning an EVM precompile error.
pub type EvmResult<T = ()> = Result<T, PrecompileFailure>;

/// Return an error with provided (static) text.
/// Using the `revert` function of `Gasometer` is prefered as erroring
/// consumed all the gas limit and the error message is not easily
/// retrievable.
pub fn error<T: Into<alloc::borrow::Cow<'static, str>>>(text: T) -> PrecompileFailure {
	PrecompileFailure::Error {
		exit_status: ExitError::Other(text.into()),
	}
}

/// Builder for PrecompileOutput.
#[derive(Clone, Debug)]
pub struct LogsBuilder {
	address: H160,
}

impl LogsBuilder {
	/// Create a new builder with no logs.
	/// Takes the address of the precompile (usualy `context.address`).
	pub fn new(address: H160) -> Self {
		Self { address }
	}

	/// Create a 0-topic log.
	#[must_use]
	pub fn log0(&self, data: impl Into<Vec<u8>>) -> Log {
		Log {
			address: self.address,
			topics: vec![],
			data: data.into(),
		}
	}

	/// Create a 1-topic log.
	#[must_use]
	pub fn log1(&self, topic0: impl Into<H256>, data: impl Into<Vec<u8>>) -> Log {
		Log {
			address: self.address,
			topics: vec![topic0.into()],
			data: data.into(),
		}
	}

	/// Create a 2-topics log.
	#[must_use]
	pub fn log2(
		&self,
		topic0: impl Into<H256>,
		topic1: impl Into<H256>,
		data: impl Into<Vec<u8>>,
	) -> Log {
		Log {
			address: self.address,
			topics: vec![topic0.into(), topic1.into()],
			data: data.into(),
		}
	}

	/// Create a 3-topics log.
	#[must_use]
	pub fn log3(
		&self,
		topic0: impl Into<H256>,
		topic1: impl Into<H256>,
		topic2: impl Into<H256>,
		data: impl Into<Vec<u8>>,
	) -> Log {
		Log {
			address: self.address,
			topics: vec![topic0.into(), topic1.into(), topic2.into()],
			data: data.into(),
		}
	}

	/// Create a 4-topics log.
	#[must_use]
	pub fn log4(
		&self,
		topic0: impl Into<H256>,
		topic1: impl Into<H256>,
		topic2: impl Into<H256>,
		topic3: impl Into<H256>,
		data: impl Into<Vec<u8>>,
	) -> Log {
		Log {
			address: self.address,
			topics: vec![topic0.into(), topic1.into(), topic2.into(), topic3.into()],
			data: data.into(),
		}
	}
}

/// Extension trait allowing to record logs into a PrecompileHandle.
pub trait LogExt {
	fn record(self, handle: &mut impl PrecompileHandle);
}

impl LogExt for Log {
	fn record(self, handle: &mut impl PrecompileHandle) {
		handle.log(self.address, self.topics, self.data);
	}
}

/// Helper functions requiring a Runtime.
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
		handle: &mut impl PrecompileHandleExt,
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

/// Represents modifiers a Solidity function can be annotated with.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FunctionModifier {
	/// Function that doesn't modify the state.
	View,
	/// Function that modifies the state but refuse receiving funds.
	/// Correspond to a Solidity function with no modifiers.
	NonPayable,
	/// Function that modifies the state and accept funds.
	Payable,
}

pub trait PrecompileHandleExt: PrecompileHandle {
	/// Record cost of a log manualy.
	/// This can be useful to record log costs early when their content have static size.
	#[must_use]
	fn record_log_costs_manual(&mut self, topics: usize, data_len: usize) -> EvmResult;

	/// Record cost of logs.
	#[must_use]
	fn record_log_costs(&mut self, logs: &[Log]) -> EvmResult;
}

impl<T: PrecompileHandle> PrecompileHandleExt for T {
	/// Record cost of a log manualy.
	/// This can be useful to record log costs early when their content have static size.
	#[must_use]
	fn record_log_costs_manual(&mut self, topics: usize, data_len: usize) -> EvmResult {
		// Cost calculation is copied from EVM code that is not publicly exposed by the crates.
		// https://github.com/rust-blockchain/evm/blob/master/gasometer/src/costs.rs#L148

		const G_LOG: u64 = 375;
		const G_LOGDATA: u64 = 8;
		const G_LOGTOPIC: u64 = 375;

		let topic_cost = G_LOGTOPIC
			.checked_mul(topics as u64)
			.ok_or(PrecompileFailure::Error {
				exit_status: ExitError::OutOfGas,
			})?;

		let data_cost = G_LOGDATA
			.checked_mul(data_len as u64)
			.ok_or(PrecompileFailure::Error {
				exit_status: ExitError::OutOfGas,
			})?;

		self.record_cost(G_LOG)?;
		self.record_cost(topic_cost)?;
		self.record_cost(data_cost)?;

		Ok(())
	}

	/// Record cost of logs.
	#[must_use]
	fn record_log_costs(&mut self, logs: &[Log]) -> EvmResult {
		for log in logs {
			self.record_log_costs_manual(log.topics.len(), log.data.len())?;
		}

		Ok(())
	}
}

#[must_use]
pub fn revert(output: impl AsRef<[u8]>) -> PrecompileFailure {
	PrecompileFailure::Revert {
		exit_status: ExitRevert::Reverted,
		output: output.as_ref().to_owned(),
	}
}

#[must_use]
/// Check that a function call is compatible with the context it is
/// called into.
pub fn check_function_modifier(
	context: &Context,
	is_static: bool,
	modifier: FunctionModifier,
) -> EvmResult {
	if is_static && modifier != FunctionModifier::View {
		return Err(revert("can't call non-static function in static context"));
	}

	if modifier != FunctionModifier::Payable && context.apparent_value > U256::zero() {
		return Err(revert("function is not payable"));
	}

	Ok(())
}
