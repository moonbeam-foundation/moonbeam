// Copyright 2024 Moonbeam Foundation.
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

use core::marker::PhantomData;
use fp_evm::{PrecompileFailure, PrecompileHandle};
use precompile_utils::prelude::*;
use sp_std::vec::Vec;

pub mod encoded_receipt;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub const JWT_VALIDATOR_ID: [u32; 8] = [
	1923256869, 654795233, 2887859926, 1709721587, 1196091263, 3916749566, 1248329059, 610202488,
];

pub struct ZkAuthVerifierPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> ZkAuthVerifierPrecompile<Runtime>
where
	Runtime: frame_system::Config,
{
	#[precompile::public("verify(uint8[])")]
	fn verify_proof(handle: &mut impl PrecompileHandle, receipt: Vec<u8>) -> EvmResult {
		//TODO: record cost
		handle.record_cost(1000)?;

		let receipt: risc0_zkvm::Receipt = postcard::from_bytes(&receipt)
			.map_err(|_| RevertReason::Custom("Receipt decoding failed".into()))?;

		receipt
			.verify(JWT_VALIDATOR_ID)
			.map_err(|_| RevertReason::Custom("Error verifying receipt".into()))?;
		Ok(())
	}
}
