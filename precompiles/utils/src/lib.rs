// Copyright 2019-2021 PureStake Inc.
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

extern crate alloc;

use evm::ExitError;
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::Get,
};
use pallet_evm::{GasWeightMapping, Log};
use sp_core::{H160, H256};
use sp_std::{marker::PhantomData, vec, vec::Vec};

mod data;

pub use data::{Address, Bytes, EvmData, EvmDataReader, EvmDataWriter};
pub use precompile_utils_macro::{generate_function_selector, keccak256};

#[cfg(test)]
mod tests;

/// Alias for Result returning an EVM precompile error.
pub type EvmResult<T = ()> = Result<T, ExitError>;

/// Return an error with provided (static) text.
pub fn error<T: Into<alloc::borrow::Cow<'static, str>>>(text: T) -> ExitError {
	ExitError::Other(text.into())
}

/// Builder for PrecompileOutput.
#[derive(Clone, Debug)]
pub struct LogsBuilder {
	address: H160,
	logs: Vec<Log>,
}

impl LogsBuilder {
	/// Create a new builder with no logs.
	/// Takes the address of the precompile (usualy `context.address`).
	pub fn new(address: H160) -> Self {
		Self {
			logs: vec![],
			address,
		}
	}

	/// Returns the logs array.
	pub fn build(self) -> Vec<Log> {
		self.logs
	}

	/// Add a 0-topic log.
	pub fn log0<D>(mut self, data: D) -> Self
	where
		D: Into<Vec<u8>>,
	{
		self.logs.push(Log {
			address: self.address,
			data: data.into(),
			topics: vec![],
		});
		self
	}

	/// Add a 1-topic log.
	pub fn log1<D, T0>(mut self, topic0: T0, data: D) -> Self
	where
		D: Into<Vec<u8>>,
		T0: Into<H256>,
	{
		self.logs.push(Log {
			address: self.address,
			data: data.into(),
			topics: vec![topic0.into()],
		});
		self
	}

	/// Add a 2-topics log.
	pub fn log2<D, T0, T1>(mut self, topic0: T0, topic1: T1, data: D) -> Self
	where
		D: Into<Vec<u8>>,
		T0: Into<H256>,
		T1: Into<H256>,
	{
		self.logs.push(Log {
			address: self.address,
			data: data.into(),
			topics: vec![topic0.into(), topic1.into()],
		});
		self
	}

	/// Add a 3-topics log.
	pub fn log3<D, T0, T1, T2>(mut self, topic0: T0, topic1: T1, topic2: T2, data: D) -> Self
	where
		D: Into<Vec<u8>>,
		T0: Into<H256>,
		T1: Into<H256>,
		T2: Into<H256>,
	{
		self.logs.push(Log {
			address: self.address,
			data: data.into(),
			topics: vec![topic0.into(), topic1.into(), topic2.into()],
		});
		self
	}

	/// Add a 4-topics log.
	pub fn log4<D, T0, T1, T2, T3>(
		mut self,
		topic0: T0,
		topic1: T1,
		topic2: T2,
		topic3: T3,
		data: D,
	) -> Self
	where
		D: Into<Vec<u8>>,
		T0: Into<H256>,
		T1: Into<H256>,
		T2: Into<H256>,
		T3: Into<H256>,
	{
		self.logs.push(Log {
			address: self.address,
			data: data.into(),
			topics: vec![topic0.into(), topic1.into(), topic2.into(), topic3.into()],
		});
		self
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
		origin: <Runtime::Call as Dispatchable>::Origin,
		call: Call,
		target_gas: Option<u64>,
	) -> EvmResult<u64>
	where
		Runtime::Call: From<Call>,
	{
		let call = Runtime::Call::from(call);
		let dispatch_info = call.get_dispatch_info();

		// Make sure there is enough gas.
		if let Some(gas_limit) = target_gas {
			let required_gas = Runtime::GasWeightMapping::weight_to_gas(dispatch_info.weight);
			if required_gas > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}

		// Dispatch call.
		// It may be possible to not record gas cost if the call returns Pays::No.
		// However while Substrate handle checking weight while not making the sender pay for it,
		// the EVM doesn't. It seems this safer to always record the costs to avoid unmetered
		// computations.
		let used_weight = call
			.dispatch(origin)
			.map_err(|e| error(alloc::format!("Dispatched call failed with error: {:?}", e)))?
			.actual_weight;

		// Return used weight by converting weight to gas.
		Ok(Runtime::GasWeightMapping::weight_to_gas(
			used_weight.unwrap_or(dispatch_info.weight),
		))
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

/// Custom Gasometer to record costs in precompiles.
/// It is advised to record known costs as early as possible to
/// avoid unecessary computations if there is an Out of Gas.
#[derive(Clone, Copy, Debug)]
pub struct Gasometer {
	target_gas: Option<u64>,
	used_gas: u64,
}

impl Gasometer {
	/// Create a new Gasometer with provided gas limit.
	/// None is no limit.
	pub fn new(target_gas: Option<u64>) -> Self {
		Self {
			target_gas,
			used_gas: 0,
		}
	}

	/// Get used gas.
	pub fn used_gas(&self) -> u64 {
		self.used_gas
	}

	/// Record cost, and return error if it goes out of gas.
	pub fn record_cost(&mut self, cost: u64) -> EvmResult {
		self.used_gas = self.used_gas.checked_add(cost).ok_or(ExitError::OutOfGas)?;

		match self.target_gas {
			Some(gas_limit) if self.used_gas > gas_limit => Err(ExitError::OutOfGas),
			_ => Ok(()),
		}
	}

	/// Record cost of a log manualy.
	/// This can be useful to record log costs early when their content have static size.
	pub fn record_log_costs_manual(&mut self, topics: usize, data_len: usize) -> EvmResult {
		// Cost calculation is copied from EVM code that is not publicly exposed by the crates.
		// https://github.com/rust-blockchain/evm/blob/master/gasometer/src/costs.rs#L148

		const G_LOG: u64 = 375;
		const G_LOGDATA: u64 = 8;
		const G_LOGTOPIC: u64 = 375;

		let topic_cost = G_LOGTOPIC
			.checked_mul(topics as u64)
			.ok_or(ExitError::OutOfGas)?;

		let data_cost = G_LOGDATA
			.checked_mul(data_len as u64)
			.ok_or(ExitError::OutOfGas)?;

		self.record_cost(G_LOG)?;
		self.record_cost(topic_cost)?;
		self.record_cost(data_cost)?;

		Ok(())
	}

	/// Record cost of logs.
	pub fn record_log_costs(&mut self, logs: &[Log]) -> EvmResult {
		for log in logs {
			self.record_log_costs_manual(log.topics.len(), log.data.len())?;
		}

		Ok(())
	}

	/// Compute remaining gas.
	/// Returns error if out of gas.
	/// Returns None if no gas limit.
	pub fn remaining_gas(&self) -> EvmResult<Option<u64>> {
		Ok(match self.target_gas {
			None => None,
			Some(gas_limit) => Some(
				gas_limit
					.checked_sub(self.used_gas)
					.ok_or(ExitError::OutOfGas)?,
			),
		})
	}
}
