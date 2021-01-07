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

use pallet_evm::LinearCostPrecompile;
use pallet_evm_precompile_simple::{ECRecover, Identity, Ripemd160, Sha256};
use sp_std::prelude::*;
// use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_modexp::Modexp;

/// An example of implementing a simple precompile.
/// prepends "deadbeef" to any data provided
struct DeadbeefPrecompiled;

impl LinearCostPrecompile for DeadbeefPrecompiled {
	const BASE: usize = 15;
	const WORD: usize = 3;

	fn execute(
		input: &[u8],
		_: usize,
	) -> core::result::Result<(pallet_evm::ExitSucceed, Vec<u8>), pallet_evm::ExitError> {
		log::info!("Calling deadbeef precompiled contract");

		let mut result_vec: Vec<u8> = rustc_hex::FromHex::from_hex("deadbeef").map_err(|_| {
			pallet_evm::ExitError::Other(sp_std::borrow::Cow::Borrowed(
				"unexpected deadbeef conversion",
			))
		})?;
		result_vec.extend(input.to_vec());

		Ok((pallet_evm::ExitSucceed::Returned, result_vec))
	}
}

/// The PrecompileSet installed in the Moonbeam runtime.
/// We include the nine Istanbul precompiles
/// (https://github.com/ethereum/go-ethereum/blob/3c46f557/core/vm/contracts.go#L69)
/// as well as a special precompile for dispatching Substrate extrinsics
///
/// TODO I had trouble getting the BN precompiles to compile.
/// Also, Why are the BN precompiles in geth called bn256*, but in Frontier they are called Bn128*
pub type MoonbeamPrecompiles<Runtime> = (
	ECRecover,
	Sha256,
	Ripemd160,
	Identity,
	Modexp,
	// Bn128Add,
	// Bn128Mul,
	// Bn128Pairing,
	Dispatch<Runtime>,
);
