// Copyright (c) Moonsong Labs.
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

// “secp256r1” is a specific elliptic curve, also known as “P-256” and “prime256v1” curves.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use fp_evm::{
	ExitError, ExitSucceed, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput,
	PrecompileResult,
};
use p256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};

pub struct P256Verify;

impl P256Verify {
	/// https://github.com/ethereum/RIPs/blob/master/RIPS/rip-7212.md#precompiled-contract-gas-usage
	const BASE_GAS: u64 = 3_450;
	/// Expected input length (160 bytes)
	const INPUT_LENGTH: usize = 160;

	/// (Signed payload) Hash of the original message
	/// 32 bytes of the signed data hash
	fn message_hash(input: &[u8]) -> &[u8] {
		&input[..32]
	}

	/// r and s signature components
	fn signature(input: &[u8]) -> &[u8] {
		&input[32..96]
	}

	/// x and y coordinates of the public key
	fn public_key(input: &[u8]) -> &[u8] {
		&input[96..160]
	}

	/// Extract and validate signature from input
	fn verify_from_input(input: &[u8]) -> Option<()> {
		let message_hash = Self::message_hash(input);
		let signature = Self::signature(input);
		let public_key = Self::public_key(input);

		let mut uncompressed_pk = [0u8; 65];
		// (0x04) prefix indicates the public key is in its uncompressed from
		uncompressed_pk[0] = 0x04;
		uncompressed_pk[1..].copy_from_slice(public_key);

		// Will only fail if the signature is not exactly 64 bytes
		let signature = Signature::from_slice(signature).ok()?;

		let public_key = VerifyingKey::from_sec1_bytes(&uncompressed_pk).ok()?;

		public_key.verify_prehash(message_hash, &signature).ok()
	}
}

/// Implements RIP-7212 P256VERIFY precompile.
/// https://github.com/ethereum/RIPs/blob/master/RIPS/rip-7212.md
impl Precompile for P256Verify {
	fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
		handle.record_cost(Self::BASE_GAS)?;

		let input = handle.input();
		// Input data: 160 bytes of data including:
		// - 32 bytes of the signed data hash
		// - 32 bytes of the r component of the signature
		// - 32 bytes of the s component of the signature
		// - 32 bytes of the x coordinate of the public key
		// - 32 bytes of the y coordinate of the public key
		if input.len() != Self::INPUT_LENGTH {
			return Err(PrecompileFailure::Error {
				exit_status: ExitError::Other(
					"input length for P256VERIFY precompile should be exactly 160 bytes".into(),
				),
			});
		}

		let result = if Self::verify_from_input(input).is_some() {
			// If the signature verification process succeeds, it returns 1 in 32 bytes format.
			let mut result = [0u8; 32];
			result[31] = 1;

			result.to_vec()
		} else {
			// If the signature verification process fails, it does not return any output data.
			Vec::new()
		};

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: result.to_vec(),
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use fp_evm::Context;
	use hex_literal::hex;
	use precompile_utils::testing::MockHandle;

	fn prepare_handle(input: Vec<u8>, cost: u64) -> impl PrecompileHandle {
		let context: Context = Context {
			address: Default::default(),
			caller: Default::default(),
			apparent_value: From::from(0),
		};

		let mut handle = MockHandle::new(Default::default(), context);
		handle.input = input;
		handle.gas_limit = cost;

		handle
	}

	#[test]
	fn test_empty_input() -> Result<(), PrecompileFailure> {
		let input = Vec::<u8>::new();
		let cost = P256Verify::BASE_GAS;
		let mut handle = prepare_handle(input, cost);

		match P256Verify::execute(&mut handle) {
			Ok(_) => panic!("Test not expected to pass"),
			Err(e) => {
				assert_eq!(
					e,
					PrecompileFailure::Error {
						exit_status: ExitError::Other(
							"input length for P256VERIFY precompile should be exactly 160 bytes"
								.into()
						)
					}
				);
				Ok(())
			}
		}
	}

	#[test]
	fn test_out_of_gas() -> Result<(), PrecompileFailure> {
		let input = Vec::<u8>::new();
		let cost = P256Verify::BASE_GAS.checked_sub(1).unwrap();
		let mut handle = prepare_handle(input, cost);

		match P256Verify::execute(&mut handle) {
			Ok(_) => panic!("Test not expected to pass"),
			Err(e) => {
				assert_eq!(
					e,
					PrecompileFailure::Error {
						exit_status: ExitError::OutOfGas
					}
				);
				Ok(())
			}
		}
	}

	#[test]
	fn test_valid_signature() -> Result<(), PrecompileFailure> {
		let input = hex!("b5a77e7a90aa14e0bf5f337f06f597148676424fae26e175c6e5621c34351955289f319789da424845c9eac935245fcddd805950e2f02506d09be7e411199556d262144475b1fa46ad85250728c600c53dfd10f8b3f4adf140e27241aec3c2da3a81046703fccf468b48b145f939efdbb96c3786db712b3113bb2488ef286cdcef8afe82d200a5bb36b5462166e8ce77f2d831a52ef2135b2af188110beaefb1").to_vec();
		let cost = P256Verify::BASE_GAS;
		let mut handle = prepare_handle(input, cost);

		let mut expected_result = [0u8; 32];
		expected_result[31] = 1;

		match P256Verify::execute(&mut handle) {
			Ok(result) => Ok(assert_eq!(result.output, expected_result.to_vec())),
			Err(_) => panic!("Test not expected to fail"),
		}
	}
}
