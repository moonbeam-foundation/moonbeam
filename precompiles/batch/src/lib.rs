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

use evm::ExitReason;
use fp_evm::{
	Context, ExitSucceed, PrecompileFailure, PrecompileHandle, PrecompileOutput, Transfer,
};
use pallet_evm::Precompile;
use precompile_utils::{
	check_function_modifier, Address, Bytes, EvmDataReader, EvmDataWriter, EvmResult,
	FunctionModifier,
};
use sp_core::U256;
use sp_std::{iter::repeat, marker::PhantomData, vec, vec::Vec};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[precompile_utils::generate_function_selector]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Action {
	BatchSome = "batchSome(address[],uint256[],bytes[])",
	BatchSomeUntilFailure = "batchSomeUntilFailure(address[],uint256[],bytes[])",
	BatchAll = "batchAll(address[],uint256[],bytes[])",
}

/// Batch precompile.
pub struct BatchPrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for BatchPrecompile<Runtime>
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

		let addresses = addresses.into_iter().enumerate();
		let values = values.into_iter().map(|x| Some(x)).chain(repeat(None));
		let calls_data = calls_data.into_iter().map(|x| Some(x)).chain(repeat(None));

		let mut outputs = vec![];
		let mut success_counter = 0;

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

			let (reason, output) = handle.call(
				address,
				transfer,
				call_data,
				Some(handle.remaining_gas()),
				false,
				&sub_context,
			);

			outputs.push(Bytes(output.clone()));

			match (reason, action) {
				// _: Fatal is always fatal
				(ExitReason::Fatal(exit_status), _) => {
					return Err(PrecompileFailure::Fatal { exit_status })
				}

				// BatchAll : Reverts and errors are immediatly forwarded.
				(ExitReason::Revert(exit_status), Action::BatchAll) => {
					return Err(PrecompileFailure::Revert {
						exit_status,
						output,
					})
				}
				(ExitReason::Error(exit_status), Action::BatchAll) => {
					return Err(PrecompileFailure::Error { exit_status })
				}

				// BatchSomeUntilFailure : Reverts and errors prevent subsequent subcalls to
				// be executed but the precompile still succeed.
				(ExitReason::Revert(_), Action::BatchSomeUntilFailure)
				| (ExitReason::Error(_), Action::BatchSomeUntilFailure) => {
					return Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new()
							.write(U256::from(i))
							.write(outputs)
							.build(),
					})
				}

				// BatchSome: Reverts and errors don't prevent subsequent subcalls to be executed,
				// but they are not counted as success.
				(ExitReason::Revert(_), Action::BatchSome)
				| (ExitReason::Error(_), Action::BatchSome) => (),

				// Success
				(ExitReason::Succeed(_), _) => success_counter += 1,
			}
		}

		let mut output = EvmDataWriter::new();

		if let Action::BatchSome | Action::BatchSomeUntilFailure = action {
			output = output.write(U256::from(success_counter));
		}

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: output.write(outputs).build(),
		})
	}
}
