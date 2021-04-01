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

use sp_std::{convert::Infallible, vec::Vec};

use crate::executor::wrapper::TraceExecutorWrapper;
use moonbeam_rpc_primitives_debug::single::{TraceType, TransactionTrace};

use ethereum_types::{H160, U256};
use evm::{
	executor::{StackExecutor, StackState as StackStateT, StackSubstateMetadata},
	Capture, Config as EvmConfig, Context, CreateScheme, Transfer,
};
use pallet_evm::{
	runner::stack::{Runner, SubstrateStackState},
	Config, ExitError, ExitReason, PrecompileSet, Vicinity,
};

pub trait TraceRunner<T: Config> {
	fn execute_call<'config, F>(
		executor: &'config mut StackExecutor<'config, SubstrateStackState<'_, 'config, T>>,
		trace_type: TraceType,
		f: F,
	) -> Result<TransactionTrace, ExitError>
	where
		F: FnOnce(
			&mut TraceExecutorWrapper<'config, SubstrateStackState<'_, 'config, T>>,
		) -> Capture<(ExitReason, Vec<u8>), Infallible>;

	/// Handle an Executor wrapper `create`. Used by `trace_create`.
	fn execute_create<'config, F>(
		executor: &'config mut StackExecutor<'config, SubstrateStackState<'_, 'config, T>>,
		trace_type: TraceType,
		f: F,
	) -> Result<TransactionTrace, ExitError>
	where
		F: FnOnce(
			&mut TraceExecutorWrapper<'config, SubstrateStackState<'_, 'config, T>>,
		) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Infallible>;

	/// Context creation for `call`. Typically called by the Runtime Api.
	fn trace_call(
		source: H160,
		target: H160,
		input: Vec<u8>,
		value: U256,
		gas_limit: u64,
		config: &EvmConfig,
		trace_type: TraceType,
	) -> Result<TransactionTrace, ExitError>;

	fn trace_create(
		source: H160,
		init: Vec<u8>,
		value: U256,
		gas_limit: u64,
		config: &EvmConfig,
		trace_type: TraceType,
	) -> Result<TransactionTrace, ExitError>;
}

impl<T: Config> TraceRunner<T> for Runner<T> {
	fn execute_call<'config, F>(
		executor: &'config mut StackExecutor<'config, SubstrateStackState<'_, 'config, T>>,
		trace_type: TraceType,
		f: F,
	) -> Result<TransactionTrace, ExitError>
	where
		F: FnOnce(
			&mut TraceExecutorWrapper<'config, SubstrateStackState<'_, 'config, T>>,
		) -> Capture<(ExitReason, Vec<u8>), Infallible>,
	{
		let mut wrapper = TraceExecutorWrapper::new(executor, true, trace_type);

		let execution_result = match f(&mut wrapper) {
			Capture::Exit((_reason, result)) => result,
			_ => unreachable!("Never reached?"),
		};

		match trace_type {
			TraceType::Raw => Ok(TransactionTrace::Raw {
				gas: U256::from(wrapper.inner.state().metadata().gasometer().gas()),
				return_value: execution_result,
				step_logs: wrapper.step_logs,
			}),
			TraceType::CallList => Ok(TransactionTrace::CallList(
				wrapper
					.entries
					.into_iter()
					.map(|(_, value)| value)
					.collect(),
			)),
		}
	}

	fn execute_create<'config, F>(
		executor: &'config mut StackExecutor<'config, SubstrateStackState<'_, 'config, T>>,
		trace_type: TraceType,
		f: F,
	) -> Result<TransactionTrace, ExitError>
	where
		F: FnOnce(
			&mut TraceExecutorWrapper<'config, SubstrateStackState<'_, 'config, T>>,
		) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Infallible>,
	{
		let mut wrapper = TraceExecutorWrapper::new(executor, true, trace_type);

		let execution_result = match f(&mut wrapper) {
			Capture::Exit((_reason, _address, result)) => result,
			_ => unreachable!("Never reached?"),
		};

		match trace_type {
			TraceType::Raw => Ok(TransactionTrace::Raw {
				gas: U256::from(wrapper.inner.state().metadata().gasometer().gas()),
				return_value: execution_result,
				step_logs: wrapper.step_logs,
			}),
			TraceType::CallList => Ok(TransactionTrace::CallList(
				wrapper
					.entries
					.into_iter()
					.map(|(_, value)| value)
					.collect(),
			)),
		}
	}

	fn trace_call(
		source: H160,
		target: H160,
		input: Vec<u8>,
		value: U256,
		gas_limit: u64,
		config: &EvmConfig,
		trace_type: TraceType,
	) -> Result<TransactionTrace, ExitError> {
		let vicinity = Vicinity {
			gas_price: U256::zero(),
			origin: source,
		};
		let metadata = StackSubstateMetadata::new(gas_limit, &config);
		let state = SubstrateStackState::new(&vicinity, metadata);
		let mut executor =
			StackExecutor::new_with_precompile(state, config, T::Precompiles::execute);
		let context = Context {
			caller: source,
			address: target,
			apparent_value: value,
		};

		Self::execute_call(&mut executor, trace_type, |executor| {
			executor.trace_call(
				target,
				Some(Transfer {
					source,
					target,
					value,
				}),
				input,
				Some(gas_limit as u64),
				false,
				false,
				false,
				context,
			)
		})
	}

	fn trace_create(
		source: H160,
		init: Vec<u8>,
		value: U256,
		gas_limit: u64,
		config: &EvmConfig,
		trace_type: TraceType,
	) -> Result<TransactionTrace, ExitError> {
		let vicinity = Vicinity {
			gas_price: U256::zero(),
			origin: source,
		};

		let metadata = StackSubstateMetadata::new(gas_limit, &config);
		let state = SubstrateStackState::new(&vicinity, metadata);
		let mut executor =
			StackExecutor::new_with_precompile(state, config, T::Precompiles::execute);
		let scheme = CreateScheme::Legacy { caller: source };
		Self::execute_create(&mut executor, trace_type, |executor| {
			executor.trace_create(source, scheme, value, init, Some(gas_limit as u64))
		})
	}
}
