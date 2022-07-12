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
use fp_evm::{
	Context, Log, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput, Transfer,
};
use precompile_utils::{costs::call_cost, prelude::*};
use sp_core::{H160, U256};
use sp_std::{iter::repeat, marker::PhantomData, vec, vec::Vec};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[generate_function_selector]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Action {
	BatchSome = "batchSome(address[],uint256[],bytes[],uint64[])",
	BatchSomeUntilFailure = "batchSomeUntilFailure(address[],uint256[],bytes[],uint64[])",
	BatchAll = "batchAll(address[],uint256[],bytes[],uint64[])",
}

pub const LOG_SUBCALL_SUCCEEDED: [u8; 32] = keccak256!("SubcallSucceeded(uint256)");
pub const LOG_SUBCALL_FAILED: [u8; 32] = keccak256!("SubcallFailed(uint256)");

pub fn log_subcall_succeeded(address: impl Into<H160>, index: usize) -> Log {
	log1(
		address,
		LOG_SUBCALL_SUCCEEDED,
		EvmDataWriter::new().write(U256::from(index)).build(),
	)
}

pub fn log_subcall_failed(address: impl Into<H160>, index: usize) -> Log {
	log1(
		address,
		LOG_SUBCALL_FAILED,
		EvmDataWriter::new().write(U256::from(index)).build(),
	)
}

/// Batch precompile.
#[derive(Debug, Clone)]
pub struct BatchPrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for BatchPrecompile<Runtime>
where
	Runtime: pallet_evm::Config,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let selector = handle.read_selector()?;

		// No funds are transfered to the precompile address.
		// Transfers will directly be made on the behalf of the user by the precompile.
		handle.check_function_modifier(FunctionModifier::NonPayable)?;

		Self::batch(handle, selector)
	}
}

impl<Runtime> BatchPrecompile<Runtime>
where
	Runtime: pallet_evm::Config,
{
	fn batch(handle: &mut impl PrecompileHandle, action: Action) -> EvmResult<PrecompileOutput> {
		let mut input = handle.read_input()?;
		let addresses: Vec<Address> = input.read()?;
		let values: Vec<U256> = input.read()?;
		let calls_data: Vec<Bytes> = input.read()?;
		let gas_limits: Vec<u64> = input.read()?;

		let addresses = addresses.into_iter().enumerate();
		let values = values.into_iter().map(|x| Some(x)).chain(repeat(None));
		let calls_data = calls_data.into_iter().map(|x| Some(x)).chain(repeat(None));
		let gas_limits = gas_limits.into_iter().map(|x|
			// x = 0 => forward all remaining gas
			if x == 0 {
				None
			} else {
				Some(x)
			}
		).chain(repeat(None));

		// Cost of batch log. (doesn't change when index changes)
		let log_cost = log_subcall_failed(handle.code_address(), 0)
			.compute_cost()
			.map_err(|_| revert("failed to compute log cost"))?;

		for ((i, address), (value, (call_data, gas_limit))) in
			addresses.zip(values.zip(calls_data.zip(gas_limits)))
		{
			let address = address.0;
			let value = value.unwrap_or(U256::zero());
			let call_data = call_data.unwrap_or(Bytes(vec![])).0;

			let sub_context = Context {
				caller: handle.context().caller,
				address: address.clone(),
				apparent_value: value,
			};

			let transfer = if value.is_zero() {
				None
			} else {
				Some(Transfer {
					source: handle.context().caller,
					target: address.clone(),
					value,
				})
			};

			// We reserve enough gas to emit a final log and perform the subcall itself.
			// If not enough gas we stop there according to Action strategy.
			let remaining_gas = handle.remaining_gas();

			let forwarded_gas = match (remaining_gas.checked_sub(log_cost), action) {
				(Some(remaining), _) => remaining,
				(None, Action::BatchAll) => {
					return Err(PrecompileFailure::Error {
						exit_status: ExitError::OutOfGas,
					})
				}
				(None, _) => {
					return Ok(succeed([]));
				}
			};

			// Cost of the call itself that the batch precompile must pay.
			let call_cost = call_cost(value, <Runtime as pallet_evm::Config>::config());

			let forwarded_gas = match forwarded_gas.checked_sub(call_cost) {
				Some(remaining) => remaining,
				None => {
					let log = log_subcall_failed(handle.code_address(), i);
					handle.record_log_costs(&[&log])?;
					log.record(handle)?;

					match action {
						Action::BatchAll => {
							return Err(PrecompileFailure::Error {
								exit_status: ExitError::OutOfGas,
							})
						}
						Action::BatchSomeUntilFailure => return Ok(succeed([])),
						Action::BatchSome => continue,
					}
				}
			};

			// If there is a provided gas limit we ensure there is enough gas remaining.
			let forwarded_gas = match gas_limit {
				None => forwarded_gas, // provide all gas if no gas limit,
				Some(limit) => {
					if limit > forwarded_gas {
						let log = log_subcall_failed(handle.code_address(), i);
						handle.record_log_costs(&[&log])?;
						log.record(handle)?;

						match action {
							Action::BatchAll => {
								return Err(PrecompileFailure::Error {
									exit_status: ExitError::OutOfGas,
								})
							}
							Action::BatchSomeUntilFailure => return Ok(succeed([])),
							Action::BatchSome => continue,
						}
					}
					limit
				}
			};

			let (reason, output) = handle.call(
				address,
				transfer,
				call_data,
				Some(forwarded_gas),
				false,
				&sub_context,
			);

			// Logs
			// We reserved enough gas so this should not OOG.
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
				(Action::BatchSomeUntilFailure, ExitReason::Revert(_) | ExitReason::Error(_)) => {
					return Ok(succeed([]))
				}

				// Success or ignored revert/error.
				(_, _) => (),
			}
		}

		Ok(succeed([]))
	}
}
