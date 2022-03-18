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


use fp_evm::{Context, ExitSucceed, PrecompileOutput};
use pallet_evm::{AddressMapping, Precompile};
use precompile_utils::{
	Bytes,
	Address, EvmDataReader, EvmDataWriter, EvmResult, FunctionModifier, Gasometer, RuntimeHelper,
};
use sp_std::marker::PhantomData;
use sp_core::U256;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	BatchSome = "batch_some(address[],uint256[],bytes[])",
	BatchAll = "batch_all(address[],uint256[],bytes[])",
}

enum BatchMode {
	RevertOnFailure,
	StopOnFailure,
}

pub struct BatchPrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for BatchPrecompile<Runtime>
where
	Runtime: pallet_evm::Config,
{
	fn execute(
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		let (mut input, selector) = EvmDataReader::new_with_selector(&mut gasometer, input)?;

		gasometer.check_function_modifier(
			context,
			is_static,
			FunctionModifier::Payable,
		)?;


		match selector {
			Action::BatchSome => Self::batch(&mut input, &mut gasometer, context, BatchMode::StopOnFailure),
		}
	}
}

impl<Runtime> BatchPrecompile<Runtime>
where
	Runtime: pallet_evm::Config,
{
	fn batch(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
		mode: BatchMode,
	) -> EvmResult<PrecompileOutput> {
		let to_list: Vec<Address> = input.read(gasometer)?;
		let value_list: Vec<U256> = input.read(gasometer)?;
		let data_list: Vec<Bytes> = input.read(gasometer)?;


		todo!()
	}
}