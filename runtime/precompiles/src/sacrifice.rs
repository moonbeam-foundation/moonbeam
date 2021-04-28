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

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Decode;
use evm::{Context, ExitError, ExitSucceed};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{Config, Precompile, PrecompileSet};
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_simple::{ECRecover, Identity, Ripemd160, Sha256};
use sp_core::{H160, U256};
use sp_std::convert::TryInto;
use sp_std::{marker::PhantomData, vec::Vec};

/// A precompile intended to burn gas without actually doing any work.
/// Meant for testing.
///
/// Expects call data to include a single u256 representing the amount of gas to burn.
/// TODO: use feature flags / somehow prevent this from deployment onto live networks (or not?)
pub struct Sacrifice;

impl Precompile for Sacrifice {
	fn execute(
		input: &[u8],
		target_gas: Option<u64>,
		_context: &Context,
	) -> core::result::Result<(ExitSucceed, Vec<u8>, u64), ExitError> {
		const INPUT_SIZE_BYTES: usize = 32;

		// input should be exactly 32 bytes (one u256)
		if input.len() != INPUT_SIZE_BYTES {
			return Err(ExitError::Other(
				"input length for Sacrifice must be exactly 32 bytes".into(),
			));
		}

		let gas_cost: u64 = U256::from_big_endian(&input[..])
			.try_into()
			.map_err(|_| ExitError::Other("amount is too large for u64".into()))?;

		// ensure we can afford our sacrifice...
		if let Some(gas_left) = target_gas {
			if gas_left < gas_cost {
				return Err(ExitError::OutOfGas);
			}
		}

		Ok((ExitSucceed::Returned, Default::default(), gas_cost))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::time::{Duration, Instant};
	extern crate hex;

	#[test]
	fn test_invalid_input_length() -> std::result::Result<(), ExitError> {
		let cost: u64 = 1;

		// TODO: this is very not-DRY, it would be nice to have a default / test impl in Frontier
		let context: Context = Context {
			address: Default::default(),
			caller: Default::default(),
			apparent_value: From::from(0),
		};

		// should fail with input of 31 byte length
		let input: [u8; 31] = [0; 31];
		assert_eq!(
			Sacrifice::execute(&input, Some(cost), &context),
			Err(ExitError::Other(
				"input length for Sacrifice must be exactly 32 bytes".into()
			)),
		);

		// should fail with input of 33 byte length
		let input: [u8; 33] = [0; 33];
		assert_eq!(
			Sacrifice::execute(&input, Some(cost), &context),
			Err(ExitError::Other(
				"input length for Sacrifice must be exactly 32 bytes".into()
			)),
		);

		Ok(())
	}

	#[test]
	fn test_gas_consumption() -> std::result::Result<(), ExitError> {
		let mut input: [u8; 32] = [0; 32];
		input[24..32].copy_from_slice(&123456_u64.to_be_bytes());

		let context: Context = Context {
			address: Default::default(),
			caller: Default::default(),
			apparent_value: From::from(0),
		};

		assert_eq!(
			Sacrifice::execute(&input, None, &context),
			Ok((ExitSucceed::Returned, Default::default(), 123456)),
		);

		Ok(())
	}

	#[test]
	fn test_oog() -> std::result::Result<(), ExitError> {
		let mut input: [u8; 32] = [0; 32];
		input[24..32].copy_from_slice(&100_u64.to_be_bytes());

		let context: Context = Context {
			address: Default::default(),
			caller: Default::default(),
			apparent_value: From::from(0),
		};

		assert_eq!(
			Sacrifice::execute(&input, Some(99), &context),
			Err(ExitError::OutOfGas),
		);

		Ok(())
	}
}
