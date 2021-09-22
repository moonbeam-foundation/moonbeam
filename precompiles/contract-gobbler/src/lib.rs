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

//! Precompile that gobbles up any contract that calls it.

#![cfg_attr(not(feature = "std"), no_std)]

use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use pallet_evm::Precompile;
use precompile_utils::{EvmDataReader, EvmResult, Gasometer, RuntimeHelper};

use sp_std::{fmt::Debug, marker::PhantomData};

#[cfg(test)]
mod mock;
// #[cfg(test)]
// mod tests;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, num_enum::TryFromPrimitive)]
enum Action {
	Gobble = "gobble()",
}

/// A precompile that gobbles up any contract that calls it
pub struct ContractGobbler<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for ContractGobbler<Runtime>
where
	Runtime: pallet_evm::Config,
{
	fn execute(
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut input = EvmDataReader::new(input);

		match &input.read_selector()? {
			Action::Gobble => Self::gobble(target_gas, context),
		}
	}
}

impl<Runtime> ContractGobbler<Runtime>
where
	Runtime: pallet_evm::Config,
{
	fn gobble(target_gas: Option<u64>, context: &Context) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);

		log::trace!(
			target: "contract-gobbler",
			"About to gobble a contract at address {:?}", "TODO"
		);

		// Delete the bytecode through pallet_evm
		pallet_evm::AccountCodes::<Runtime>::remove(context.caller);

		gasometer.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}
}
