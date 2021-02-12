// Copyright 2019-2020 PureStake Inc.
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

use crate::executor::wrapper::TraceExecutorWrapper;
use ethereum_types::{H160, U256};
use evm::{
	executor::{StackExecutor, StackState as StackStateT, StackSubstateMetadata},
	Capture, Config as EvmConfig, Transfer,
};
// use fp_evm::Vicinity;
use moonbeam_rpc_primitives_debug::TraceExecutorResponse;
use pallet_evm::{
	runner::stack::{Runner, SubstrateStackState},
	Config, ExitError, ExitReason, PrecompileSet, Vicinity,
};
use sp_std::{convert::Infallible, vec::Vec};

pub trait TraceRunner<T: Config> {
	fn execute_call<'config, F>(
		executor: &'config mut StackExecutor<'config, SubstrateStackState<'_, 'config, T>>,
		f: F,
	) -> Result<TraceExecutorResponse, ExitError>
	where
		F: FnOnce(
			&mut TraceExecutorWrapper<'config, SubstrateStackState<'_, 'config, T>>,
		) -> Capture<(ExitReason, Vec<u8>), Infallible>;

	fn execute_create<'config, F>(
		executor: &'config mut StackExecutor<'config, SubstrateStackState<'_, 'config, T>>,
		f: F,
	) -> Result<TraceExecutorResponse, ExitError>
	where
		F: FnOnce(
			&mut TraceExecutorWrapper<'config, SubstrateStackState<'_, 'config, T>>,
		) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Infallible>;

	fn trace_call(
		source: H160,
		target: H160,
		input: Vec<u8>,
		value: U256,
		gas_limit: u64,
		config: &EvmConfig,
	) -> Result<TraceExecutorResponse, ExitError>;

	fn trace_create(
		source: H160,
		init: Vec<u8>,
		value: U256,
		gas_limit: u64,
		config: &EvmConfig,
	) -> Result<TraceExecutorResponse, ExitError>;
}

impl<T: Config> TraceRunner<T> for Runner<T> {
	fn execute_call<'config, F>(
		executor: &'config mut StackExecutor<'config, SubstrateStackState<'_, 'config, T>>,
		f: F,
	) -> Result<TraceExecutorResponse, ExitError>
	where
		F: FnOnce(
			&mut TraceExecutorWrapper<'config, SubstrateStackState<'_, 'config, T>>,
		) -> Capture<(ExitReason, Vec<u8>), Infallible>,
	{
		let mut wrapper = TraceExecutorWrapper::new(executor, true);

		let execution_result = match f(&mut wrapper) {
			Capture::Exit((_reason, result)) => result,
			_ => unreachable!("Never reached?"),
		};

		Ok(TraceExecutorResponse {
			gas: U256::from(wrapper.inner.state().metadata().gasometer().gas()),
			return_value: execution_result,
			step_logs: wrapper.step_logs,
		})
	}

	fn execute_create<'config, F>(
		executor: &'config mut StackExecutor<'config, SubstrateStackState<'_, 'config, T>>,
		f: F,
	) -> Result<TraceExecutorResponse, ExitError>
	where
		F: FnOnce(
			&mut TraceExecutorWrapper<'config, SubstrateStackState<'_, 'config, T>>,
		) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Infallible>,
	{
		let mut wrapper = TraceExecutorWrapper::new(executor, true);

		let execution_result = match f(&mut wrapper) {
			Capture::Exit((_reason, _address, result)) => result,
			_ => unreachable!("Never reached?"),
		};

		Ok(TraceExecutorResponse {
			gas: U256::from(wrapper.inner.state().metadata().gasometer().gas()),
			return_value: execution_result,
			step_logs: wrapper.step_logs,
		})
	}

	fn trace_call(
		source: H160,
		target: H160,
		input: Vec<u8>,
		value: U256,
		gas_limit: u64,
		config: &EvmConfig,
	) -> Result<TraceExecutorResponse, ExitError> {
		let vicinity = Vicinity {
			gas_price: U256::zero(),
			origin: source,
		};
		let metadata = StackSubstateMetadata::new(gas_limit, &config);
		let state = SubstrateStackState::new(&vicinity, metadata);
		let mut executor =
			StackExecutor::new_with_precompile(state, config, T::Precompiles::execute);
		Self::execute_call(&mut executor, |executor| {
			executor.trace_call(
				source,
				target,
				Some(Transfer {
					source,
					target,
					value,
				}),
				value,
				input,
				gas_limit as u64,
			)
		})
	}

	fn trace_create(
		source: H160,
		init: Vec<u8>,
		value: U256,
		gas_limit: u64,
		config: &EvmConfig,
	) -> Result<TraceExecutorResponse, ExitError> {
		let vicinity = Vicinity {
			gas_price: U256::zero(),
			origin: source,
		};

		let metadata = StackSubstateMetadata::new(gas_limit, &config);
		let state = SubstrateStackState::new(&vicinity, metadata);
		let mut executor =
			StackExecutor::new_with_precompile(state, config, T::Precompiles::execute);
		Self::execute_create(&mut executor, |executor| {
			executor.trace_create(source, value, init, gas_limit as u64)
		})
	}
}
