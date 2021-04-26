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

use crate::executor::wrapper::{PrecompileExecutable, TraceExecutorWrapper};
use moonbeam_rpc_primitives_debug::single::{TraceType, TransactionTrace};

use ethereum_types::{H160, U256};
use evm::{
	executor::{StackExecutor, StackState as StackStateT, StackSubstateMetadata},
	gasometer, Capture, Config as EvmConfig, Context, CreateScheme, Transfer,
};
use frame_support::ensure;
use pallet_evm::{
	runner::stack::{Runner, SubstrateStackState},
	Config, Error, ExitError, ExitReason, Module, OnChargeEVMTransaction, PrecompileSet, Vicinity,
};

pub enum TraceRunnerError<T: Config> {
	EvmExitError(ExitError),
	RuntimeExitError(Error<T>),
}

pub trait TraceRunner<T: Config> {
	/// Handle an Executor wrapper `call`.
	fn execute_call<'config, F>(
		executor: &'config mut StackExecutor<'config, SubstrateStackState<'_, 'config, T>>,
		trace_type: TraceType,
		precompile: PrecompileExecutable,
		f: F,
	) -> Result<TransactionTrace, ExitError>
	where
		F: FnOnce(
			&mut TraceExecutorWrapper<'config, SubstrateStackState<'_, 'config, T>>,
		) -> Capture<(ExitReason, Vec<u8>), Infallible>;

	/// Handle an Executor wrapper `create`.
	fn execute_create<'config, F>(
		executor: &'config mut StackExecutor<'config, SubstrateStackState<'_, 'config, T>>,
		trace_type: TraceType,
		f: F,
	) -> Result<TransactionTrace, ExitError>
	where
		F: FnOnce(
			&mut TraceExecutorWrapper<'config, SubstrateStackState<'_, 'config, T>>,
		) -> Capture<(ExitReason, Option<H160>, Vec<u8>), Infallible>;

	/// Interfaces runtime api and executor wrapper.
	fn trace(
		source: H160,
		target: Option<H160>,
		input: Vec<u8>,
		value: U256,
		gas_limit: u64,
		gas_price: U256,
		nonce: U256,
		config: &EvmConfig,
		trace_type: TraceType,
	) -> Result<TransactionTrace, TraceRunnerError<T>>;
}

impl<T: Config> TraceRunner<T> for Runner<T> {
	fn execute_call<'config, F>(
		executor: &'config mut StackExecutor<'config, SubstrateStackState<'_, 'config, T>>,
		trace_type: TraceType,
		precompile: PrecompileExecutable,
		f: F,
	) -> Result<TransactionTrace, ExitError>
	where
		F: FnOnce(
			&mut TraceExecutorWrapper<'config, SubstrateStackState<'_, 'config, T>>,
		) -> Capture<(ExitReason, Vec<u8>), Infallible>,
	{
		let mut wrapper = TraceExecutorWrapper::new(executor, true, trace_type, Some(precompile));

		let execution_result = match f(&mut wrapper) {
			Capture::Exit((_reason, result)) => result,
			_ => unreachable!("Never reached?"),
		};

		match trace_type {
			TraceType::Raw { .. } => Ok(TransactionTrace::Raw {
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
		let mut wrapper = TraceExecutorWrapper::new(executor, true, trace_type, None);

		let execution_result = match f(&mut wrapper) {
			Capture::Exit((_reason, _address, result)) => result,
			_ => unreachable!("Never reached?"),
		};

		match trace_type {
			TraceType::Raw { .. } => Ok(TransactionTrace::Raw {
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

	fn trace(
		source: H160,
		target: Option<H160>,
		input: Vec<u8>,
		value: U256,
		gas_limit: u64,
		gas_price: U256,
		nonce: U256,
		config: &EvmConfig,
		trace_type: TraceType,
	) -> Result<TransactionTrace, TraceRunnerError<T>> {
		let vicinity = Vicinity {
			gas_price,
			origin: source,
		};

		let metadata = StackSubstateMetadata::new(gas_limit, &config);
		let state = SubstrateStackState::new(&vicinity, metadata);

		let mut executor =
			StackExecutor::new_with_precompile(state, config, T::Precompiles::execute);

		let total_fee = gas_price
			.checked_mul(U256::from(gas_limit))
			.ok_or(TraceRunnerError::RuntimeExitError(Error::<T>::FeeOverflow))?;

		let total_payment =
			value
				.checked_add(total_fee)
				.ok_or(TraceRunnerError::RuntimeExitError(
					Error::<T>::PaymentOverflow,
				))?;
		let source_account = Module::<T>::account_basic(&source);
		ensure!(
			source_account.balance >= total_payment,
			TraceRunnerError::RuntimeExitError(Error::<T>::BalanceLow)
		);

		ensure!(
			source_account.nonce == nonce,
			TraceRunnerError::RuntimeExitError(Error::<T>::InvalidNonce)
		);

		// Deduct fee from the `source` account.
		let fee = T::OnChargeTransaction::withdraw_fee(&source, total_fee)
			.map_err(|e| TraceRunnerError::RuntimeExitError(e))?;

		let transaction_cost = gasometer::create_transaction_cost(&input);
		let _ = executor
			.state_mut()
			.metadata_mut()
			.gasometer_mut()
			.record_transaction(transaction_cost)
			.map_err(|e| TraceRunnerError::EvmExitError(e))?;

		let mut actual_fee: U256 = U256::default();

		let res = {
			if let Some(target) = target {
				// Call context
				let context = Context {
					caller: source,
					address: target,
					apparent_value: value,
				};
				Self::execute_call(
					&mut executor,
					trace_type,
					T::Precompiles::execute,
					|executor| {
						let res = executor.trace_call(
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
						);
						actual_fee = executor.inner.fee(gas_price);
						res
					},
				)
				.map_err(|e| TraceRunnerError::EvmExitError(e))?
			} else {
				// Create context
				let scheme = CreateScheme::Legacy { caller: source };
				Self::execute_create(&mut executor, trace_type, |executor| {
					let res =
						executor.trace_create(source, scheme, value, input, Some(gas_limit as u64));
					actual_fee = executor.inner.fee(gas_price);
					res
				})
				.map_err(|e| TraceRunnerError::EvmExitError(e))?
			}
		};
		// Refund fees to the `source` account if deducted more before,
		T::OnChargeTransaction::correct_and_deposit_fee(&source, actual_fee, fee)
			.map_err(|e| TraceRunnerError::RuntimeExitError(e))?;
		Ok(res)
	}
}
