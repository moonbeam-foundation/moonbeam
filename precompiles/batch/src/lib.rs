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

use fp_evm::{Context, PrecompileOutput};
use pallet_evm::Precompile;
use precompile_utils::EvmResult;
use sp_std::marker::PhantomData;

/// Batch precompile. Should be registered as a "delegatable precompile" as it
/// must execute on the behalf of the user.
pub struct BatchPrecompile<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for BatchPrecompile<Runtime>
where
	Runtime: pallet_evm::Config,
{
	fn execute(
		input: &[u8],
		_target_gas: Option<u64>,
		_context: &Context,
		_is_static: bool,
	) -> EvmResult<PrecompileOutput> {
		let bytecode = include_bytes!("../bytecode.bin");

		Ok(PrecompileOutput::Execute {
			code: bytecode.to_vec(),
			input: input.to_vec(),
			cost: 0,
		})
	}
}