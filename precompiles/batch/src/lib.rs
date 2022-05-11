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
	check_function_modifier, Address, Bytes, EvmDataReader, EvmDataWriter,
	EvmResult, FunctionModifier,
};
use sp_core::U256;
use sp_std::{marker::PhantomData, vec};
#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	BatchSome = "batchSome(address[],uint256[],bytes[])",
	BatchAll = "batchAll(address[],uint256[],bytes[])",
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum BatchMode {
	Some,
	All,
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

		check_function_modifier(context, is_static, FunctionModifier::Payable)?;

		match selector {
			Action::BatchSome => Self::batch(handle, input, context, BatchMode::Some),
			Action::BatchAll => Self::batch(handle, input, context, BatchMode::All),
		}
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
		mode: BatchMode,
	) -> EvmResult<PrecompileOutput> {
		let addresses: Vec<Address> = input.read()?;
		let values: Vec<U256> = input.read()?;
		let calls_data: Vec<Bytes> = input.read()?;

		let len = addresses.len();
		let addresses = addresses.into_iter().enumerate();
		let values = values.into_iter().map(|x| Some(x)).fuse();
		let calls_data = calls_data.into_iter().map(|x| Some(x)).fuse();

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

			let (reason, output) = handle.call(
				address,
				transfer,
				call_data,
				Some(handle.remaining_gas()),
				false,
				&sub_context,
			);

			outputs.push(Bytes(output.clone()));

			match (reason, mode) {
				(ExitReason::Fatal(exit_status), _) => {
					return Err(PrecompileFailure::Fatal { exit_status })
				}
				(ExitReason::Revert(exit_status), BatchMode::All) => {
					return Err(PrecompileFailure::Revert {
						exit_status,
						output,
					})
				}
				(ExitReason::Error(exit_status), BatchMode::All) => {
					return Err(PrecompileFailure::Error { exit_status })
				}
				(ExitReason::Revert(_), BatchMode::Some)
				| (ExitReason::Error(_), BatchMode::Some) => {
					return Ok(PrecompileOutput {
						exit_status: ExitSucceed::Returned,
						output: EvmDataWriter::new()
							.write(U256::from(i))
							.write(outputs)
							.build(),
					})
				}
				(ExitReason::Succeed(_), _) => (),
			}
		}

		let mut output = EvmDataWriter::new();

		if mode == BatchMode::Some {
			output = output.write(U256::from(len));
		}

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: output.build(),
		})
	}
}
