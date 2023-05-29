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

extern crate alloc;

// Allows to use inside this crate `solidity::Codec` derive macro,which depends on
// `precompile_utils` being in the list of imported crates.
extern crate self as precompile_utils;

pub mod evm;
pub mod precompile_set;
pub mod substrate;

pub mod solidity;

#[cfg(feature = "testing")]
pub mod testing;

#[cfg(test)]
mod tests;

use fp_evm::PrecompileFailure;

// pub mod data;

// pub use data::{solidity::Codec, Reader, Writer};
pub use fp_evm::Precompile;
pub use precompile_utils_macro::{keccak256, precompile, precompile_name_from_address};

/// Alias for Result returning an EVM precompile error.
pub type EvmResult<T = ()> = Result<T, PrecompileFailure>;

pub mod prelude {
	pub use {
		crate::{
			evm::{
				handle::PrecompileHandleExt,
				logs::{log0, log1, log2, log3, log4, LogExt},
			},
			precompile_set::DiscriminantResult,
			solidity::{
				// We export solidity itself to encourage using `solidity::Codec` to avoid confusion
				// with parity_scale_codec,
				self,
				codec::{
					Address,
					BoundedBytes,
					BoundedString,
					BoundedVec,
					// Allow usage of Codec methods while not exporting the name directly.
					Codec as _,
					Convert,
					UnboundedBytes,
					UnboundedString,
				},
				revert::{
					revert, BacktraceExt, InjectBacktrace, MayRevert, Revert, RevertExt,
					RevertReason,
				},
			},
			substrate::{RuntimeHelper, TryDispatchError},
			EvmResult,
		},
		alloc::string::String,
		pallet_evm::{PrecompileHandle, PrecompileOutput},
		precompile_utils_macro::{keccak256, precompile},
	};
}
