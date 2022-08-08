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

extern crate alloc;

pub mod costs;
pub mod handle;
pub mod logs;
pub mod modifier;
pub mod precompile_set;
pub mod substrate;

#[cfg(feature = "testing")]
pub mod solidity;

#[cfg(feature = "testing")]
pub mod testing;

#[cfg(test)]
mod tests;

use crate::alloc::borrow::ToOwned;
use fp_evm::{
	ExitError, ExitRevert, ExitSucceed, PrecompileFailure, PrecompileHandle, PrecompileOutput,
};

pub mod data;
use data::{Bytes, EvmDataWriter};

// pub use data::{Address, Bytes, EvmData, EvmDataReader, EvmDataWriter};
// pub use fp_evm::Precompile;
// pub use precompile_utils_macro::{generate_function_selector, keccak256};

/// Return an error with provided (static) text.
/// Using the `revert` function of `Gasometer` is preferred as erroring
/// consumed all the gas limit and the error message is not easily
/// retrievable.
#[must_use]
pub fn error<T: Into<alloc::borrow::Cow<'static, str>>>(text: T) -> PrecompileFailure {
	PrecompileFailure::Error {
		exit_status: ExitError::Other(text.into()),
	}
}

/// Generic error to build abi-encoded revert output.
/// See: https://docs.soliditylang.org/en/latest/control-structures.html?highlight=revert#revert
#[precompile_utils_macro::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Error {
	Generic = "Error(string)",
}

#[must_use]
pub fn revert(output: impl AsRef<[u8]>) -> PrecompileFailure {
	PrecompileFailure::Revert {
		exit_status: ExitRevert::Reverted,
		output: EvmDataWriter::new_with_selector(Error::Generic)
			.write::<Bytes>(Bytes(output.as_ref().to_owned()))
			.build(),
	}
}

#[must_use]
pub fn succeed(output: impl AsRef<[u8]>) -> PrecompileOutput {
	PrecompileOutput {
		exit_status: ExitSucceed::Returned,
		output: output.as_ref().to_owned(),
	}
}

/// Alias for Result returning an EVM precompile error.
pub type EvmResult<T = ()> = Result<T, PrecompileFailure>;

/// Trait similar to `fp_evm::Precompile` but with a `&self` parameter to manage some
/// state (this state is only kept in a single transaction and is lost afterward).
pub trait StatefulPrecompile {
	/// Instanciate the precompile.
	/// Will be called once when building the PrecompileSet at the start of each
	/// Ethereum transaction.
	fn new() -> Self;

	/// Execute the precompile with a reference to its state.
	fn execute(&self, handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput>;
}

pub mod prelude {
	pub use {
		crate::{
			data::{
				Address, BoundedBytes, BoundedVec, Bytes, EvmData, EvmDataReader, EvmDataWriter,
			},
			error,
			handle::PrecompileHandleExt,
			logs::{log0, log1, log2, log3, log4, LogExt},
			modifier::{check_function_modifier, FunctionModifier},
			revert,
			substrate::RuntimeHelper,
			succeed, EvmResult, StatefulPrecompile,
		},
		pallet_evm::PrecompileHandle,
		precompile_utils_macro::{generate_function_selector, keccak256},
	};
}
