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

//! Precompile to interact with pallet_balances instances using the ERC20 interface standard.

#![cfg_attr(not(feature = "std"), no_std)]

use evm::{ExitError, ExitReason};
use fp_evm::{Context, Log, PrecompileFailure, PrecompileHandle, PrecompileOutput, Transfer};
use precompile_utils::{
	check_function_modifier, keccak256, succeed, Address, Bytes, EvmDataReader, EvmDataWriter,
	EvmResult, FunctionModifier, LogExt, LogsBuilder, PrecompileHandleExt,
};
use sp_core::{H160, U256};
use sp_std::{iter::repeat, marker::PhantomData, vec, vec::Vec};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[precompile_utils::generate_function_selector]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Action {
	BatchSome = "batchSome(address[],uint256[],bytes[],bool)",
	BatchSomeUntilFailure = "batchSomeUntilFailure(address[],uint256[],bytes[],bool)",
	BatchAll = "batchAll(address[],uint256[],bytes[],bool)",
}

pub const LOG_SUBCALL_SUCCEEDED: [u8; 32] = keccak256!("SubcallSucceeded(uint256)");
pub const LOG_SUBCALL_FAILED: [u8; 32] = keccak256!("SubcallFailed(uint256)");

pub fn log_subcall_succeeded(address: impl Into<H160>, index: usize) -> Log {
	LogsBuilder::new(address.into()).log1(
		LOG_SUBCALL_SUCCEEDED,
		EvmDataWriter::new().write(U256::from(index)).build(),
	)
}

pub fn log_subcall_failed(address: impl Into<H160>, index: usize) -> Log {
	LogsBuilder::new(address.into()).log1(
		LOG_SUBCALL_FAILED,
		EvmDataWriter::new().write(U256::from(index)).build(),
	)
}

/// Batch precompile.
pub struct BatchPrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> pallet_evm::Precompile for BatchPrecompile<Runtime>
where
	Runtime: pallet_evm::Config,
{
	fn execute(
		handle: &mut impl PrecompileHandle,
		input: &[u8],
		_target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> EvmResult<PrecompileOutput> {
		let (mut input, selector) = EvmDataReader::new_with_selector(input)?;
		let input = &mut input;

		// No funds are transfered to the precompile address.
		// Transfers will directly be made on the behalf of the user by the precompile.
		check_function_modifier(context, is_static, FunctionModifier::NonPayable)?;

		Self::batch(handle, input, context, selector)
	}
}

impl<Runtime> BatchPrecompile<Runtime>
where
	Runtime: pallet_evm::Config,
{
	fn batch(
		handle: &mut impl PrecompileHandle,
		input: &mut EvmDataReader,
		context: &Context,
		action: Action,
	) -> EvmResult<PrecompileOutput> {
		let addresses: Vec<Address> = input.read()?;
		let values: Vec<U256> = input.read()?;
		let calls_data: Vec<Bytes> = input.read()?;
		let emit_logs: bool = input.read()?;

		let addresses = addresses.into_iter().enumerate();
		let values = values.into_iter().map(|x| Some(x)).chain(repeat(None));
		let calls_data = calls_data.into_iter().map(|x| Some(x)).chain(repeat(None));

		let mut outputs = vec![];

		for ((i, address), (value, call_data)) in addresses.zip(values.zip(calls_data)) {
			let address = address.0;
			let value = value.unwrap_or(U256::zero());
			let call_data = call_data.unwrap_or(Bytes(vec![])).0;

			let sub_context = Context {
				caller: context.caller,
				address: address.clone(),
				apparent_value: value,
			};

			let transfer = if value.is_zero() {
				None
			} else {
				Some(Transfer {
					source: context.caller,
					target: address.clone(),
					value,
				})
			};

			// We reserve enough gas to emit a final log .
			// If not enough gas we stop there according to Action strategy.
			let remaining_gas = handle.remaining_gas();
			let gas_limit = match (
				emit_logs,
				action,
				remaining_gas
					.checked_sub(log_subcall_failed(handle.code_address(), i).compute_cost()? + 1),
			) {
				(false, _, _) => remaining_gas,
				(true, _, Some(gas_limit)) => gas_limit,
				(true, Action::BatchAll, None) => {
					return Err(PrecompileFailure::Error {
						exit_status: ExitError::OutOfGas,
					})
				}
				(true, Action::BatchSome | Action::BatchSomeUntilFailure, None) => {
					return Ok(succeed([]))
				}
			};

			let (reason, output) = handle.call(
				address,
				transfer,
				call_data,
				Some(gas_limit),
				false,
				&sub_context,
			);

			outputs.push(Bytes(output.clone()));

			// Logs
			// We reserved enough gas so this should not OOG.
			if emit_logs {
				match reason {
					ExitReason::Revert(_) | ExitReason::Error(_) => {
						let log = log_subcall_failed(handle.code_address(), i);
						handle.record_log_costs(&[&log])?;
						log.record(handle)?
					}
					ExitReason::Succeed(_) => {
						let log = log_subcall_succeeded(handle.code_address(), i);
						handle.record_log_costs(&[&log])?;
						log.record(handle)?
					}
					_ => (),
				}
			}

			// How to proceed
			match (action, reason) {
				// _: Fatal is always fatal
				(_, ExitReason::Fatal(exit_status)) => {
					return Err(PrecompileFailure::Fatal { exit_status })
				}

				// BatchAll : Reverts and errors are immediatly forwarded.
				(Action::BatchAll, ExitReason::Revert(exit_status)) => {
					return Err(PrecompileFailure::Revert {
						exit_status,
						output,
					})
				}
				(Action::BatchAll, ExitReason::Error(exit_status)) => {
					return Err(PrecompileFailure::Error { exit_status })
				}

				// BatchSomeUntilFailure : Reverts and errors prevent subsequent subcalls to
				// be executed but the precompile still succeed.
				//
				// BatchSome : since Error consume all gas (or can be out of gas), we stop
				// to not out of gas when processing next subcall.
				(Action::BatchSomeUntilFailure, ExitReason::Revert(_) | ExitReason::Error(_))
				| (Action::BatchSome, ExitReason::Error(_)) => return Ok(succeed([])),

				// BatchSome: Reverts and errors don't prevent subsequent subcalls to be executed,
				// but they are not counted as success.
				(Action::BatchSome, ExitReason::Revert(_)) => (),

				// Success
				(_, ExitReason::Succeed(_)) => (),
			}
		}

		Ok(succeed([]))
	}
}
