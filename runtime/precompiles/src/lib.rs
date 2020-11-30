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

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
use sp_core::H160;
use pallet_evm::{ExitError, ExitSucceed, Precompile};

pub struct ExperimentalMoonbeamPrecompiles;

/// Linear gas cost
fn ensure_linear_cost(
	target_gas: Option<usize>,
	len: usize,
	base: usize,
	word: usize
) -> Result<usize, pallet_evm::ExitError> {
	let cost = base.checked_add(
		word.checked_mul(len.saturating_add(31) / 32).ok_or(pallet_evm::ExitError::OutOfGas)?
	).ok_or(pallet_evm::ExitError::OutOfGas)?;
	if let Some(target_gas) = target_gas {
		if cost > target_gas {
			return Err(pallet_evm::ExitError::OutOfGas)
		}
	}
	Ok(cost)
}

// prepends "deadbeef" to any data provided
struct DeadbeefPrecompiled;

impl pallet_evm::Precompile for DeadbeefPrecompiled {
	fn execute(
		input: &[u8],
		target_gas: Option<usize>
	) -> core::result::Result<(pallet_evm::ExitSucceed, Vec<u8>, usize), pallet_evm::ExitError> {
		let cost = ensure_linear_cost(target_gas, input.len(), 15, 3)?;

		log::info!("Calling deadbeef precompiled contract");

		let mut result_vec: Vec<u8> = rustc_hex::FromHex::from_hex("deadbeef")
			.map_err(|_| pallet_evm::ExitError::Other(
				sp_std::borrow::Cow::Borrowed("unexpected deadbeef conversion")
			))?;
		result_vec.extend(input.to_vec());

		Ok((pallet_evm::ExitSucceed::Returned, result_vec, cost))
	}
}

type PrecompiledCallable = fn(&[u8], Option<usize>)
	-> core::result::Result<(pallet_evm::ExitSucceed, Vec<u8>, usize), pallet_evm::ExitError>;

fn get_precompiled_func_from_address(address: &H160) -> Option<PrecompiledCallable> {
	use core::str::FromStr;

	// Note that addresses from_str should not start with 0x, just the hex value
	let addr_deadbeef = H160::from_str("0000000000000000000000000000000000001000")
		.expect("Invalid address at precompiles generation");

	if *address == addr_deadbeef {
		return Some(DeadbeefPrecompiled::execute);
	}

	None
}

impl pallet_evm::Precompiles for ExperimentalMoonbeamPrecompiles {
	fn execute(
		address: H160,
		input: &[u8],
		target_gas: Option<usize>
	) -> Option<
		core::result::Result<
			(pallet_evm::ExitSucceed, Vec<u8>, usize),
			pallet_evm::ExitError,
		>
	> {
		match get_precompiled_func_from_address(&address) {
			Some(func) => return Some(func(input, target_gas)),
			_ => {},
		};

		None
	}
}



use ripemd160::Digest;
/// The ripemd precompile.
pub struct Ripemd160;

impl Precompile for Ripemd160 {
	fn execute(
		input: &[u8],
		target_gas: Option<usize>,
	) -> core::result::Result<(ExitSucceed, Vec<u8>, usize), ExitError> {
		let cost = ensure_linear_cost(target_gas, input.len(), 600, 120)?;

		let mut v32 = [0u8; 32];
		v32[12..32].copy_from_slice(&ripemd160::Ripemd160::digest(input));
		Ok((ExitSucceed::Returned, v32.to_vec(), cost))
	}
}


pub type MoonbeamPrecompiles =
(
	pallet_evm::precompiles::ECRecover,
	pallet_evm::precompiles::Sha256,
	// Reset to pallet_evm ripemd160 once
	// https://github.com/paritytech/substrate/pull/7296 is included
	Ripemd160,
	pallet_evm::precompiles::Identity,
);


#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
