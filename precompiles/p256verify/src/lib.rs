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

// “secp256r1” is a specific elliptic curve, also known as “P-256”
// and “prime256v1” curves.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use core::marker::PhantomData;
use fp_evm::{
	ExitError, ExitSucceed, Precompile, PrecompileFailure, PrecompileHandle, PrecompileOutput,
	PrecompileResult,
};
use frame_support::{traits::Get, weights::Weight};
use p256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};

pub struct P256Verify<W: Get<Weight>>(PhantomData<W>);

impl<W: Get<Weight>> P256Verify<W> {
	/// Expected input length (160 bytes)
	const INPUT_LENGTH: usize = 160;

	/// Handle gas costs
	#[inline]
	fn handle_cost(handle: &mut impl PrecompileHandle) -> Result<(), ExitError> {
		let weight = W::get();
		handle.record_external_cost(Some(weight.ref_time()), None, None)
	}

	/// (Signed payload) Hash of the original message
	/// 32 bytes of the signed data hash
	#[inline]
	fn message_hash(input: &[u8]) -> &[u8] {
		&input[..32]
	}

	/// r and s signature components
	#[inline]
	fn signature(input: &[u8]) -> &[u8] {
		&input[32..96]
	}

	/// x and y coordinates of the public key
	#[inline]
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
impl<W: Get<Weight>> Precompile for P256Verify<W> {
	fn execute(handle: &mut impl PrecompileHandle) -> PrecompileResult {
		Self::handle_cost(handle)?;

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
	use frame_support::parameter_types;
	use hex_literal::hex;
	use precompile_utils::testing::MockHandle;

	parameter_types! {
		pub const DummyWeight: Weight = Weight::from_parts(3450, 0);
	}

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
	fn test_valid_signature() {
		let inputs = vec![
			(
				true,
				hex!(
					"b5a77e7a90aa14e0bf5f337f06f597148676424fae26e175c6e5621c34351955289f"
					"319789da424845c9eac935245fcddd805950e2f02506d09be7e411199556d2621444"
					"75b1fa46ad85250728c600c53dfd10f8b3f4adf140e27241aec3c2da3a81046703fc"
					"cf468b48b145f939efdbb96c3786db712b3113bb2488ef286cdcef8afe82d200a5bb"
					"36b5462166e8ce77f2d831a52ef2135b2af188110beaefb1"
				)
				.to_vec(),
				None,
			),
			(
				true,
				hex!(
					"4cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4da73b"
					"d4903f0ce3b639bbbf6e8e80d16931ff4bcf5993d58468e8fb19086e8cac36dbcd03"
					"009df8c59286b162af3bd7fcc0450c9aa81be5d10d312af6c66b1d604aebd3099c61"
					"8202fcfe16ae7770b0c49ab5eadf74b754204a3bb6060e44eff37618b065f9832de4"
					"ca6ca971a7a1adc826d0f7c00181a5fb2ddf79ae00b4e10e"
				)
				.to_vec(),
				None,
			),
			(
				false,
				hex!(
					"afec5769b5cf4e310a7d150508e82fb8e3eda1c2c94c61492d3bd8aea99e06c9e22466"
					"e928fdccef0de49e3503d2657d00494a00e764fd437bdafa05f5922b1fbbb77c6817cc"
					"f50748419477e843d5bac67e6a70e97dde5a57e0c983b777e1ad31a80482dadf89de63"
					"02b1988c82c29544c9c07bb910596158f6062517eb089a2f54c9a0f348752950094d32"
					"28d3b940258c75fe2a413cb70baa21dc2e352fc5"
				)
				.to_vec(),
				None,
			),
			(
				false,
				hex!(
					"3cee90eb86eaa050036147a12d49004b6b9c72bd725d39d4785011fe190f0b4da73bd4"
					"903f0ce3b639bbbf6e8e80d16931ff4bcf5993d58468e8fb19086e8cac36dbcd03009d"
					"f8c59286b162af3bd7fcc0450c9aa81be5d10d312af6c66b1d604aebd3099c618202fc"
					"fe16ae7770b0c49ab5eadf74b754204a3bb6060e44eff37618b065f9832de4ca6ca971"
					"a7a1adc826d0f7c00181a5fb2ddf79ae00b4e10e"
				)
				.to_vec(),
				None,
			),
			(
				false,
				hex!("4cee90eb86eaa050036147a12d49004b6a").to_vec(),
				Some(PrecompileFailure::Error {
					exit_status: ExitError::Other(
						"input length for P256VERIFY precompile should be exactly 160 bytes".into(),
					),
				}),
			),
		];
		for input in inputs {
			let cost = 3450;
			let mut handle = prepare_handle(input.1.clone(), cost);

			let mut success_result = [0u8; 32];
			success_result[31] = 1;

			let unsuccessful_result = Vec::<u8>::new();

			match (input.0, P256Verify::<DummyWeight>::execute(&mut handle)) {
				(true, Ok(result)) => assert_eq!(result.output, success_result.to_vec()),
				(false, Ok(result)) => assert_eq!(result.output, unsuccessful_result),
				(_, Err(e)) => {
					if let Some(err) = input.2 {
						assert_eq!(e, err)
					} else {
						panic!("Test not expected to fail for input: {:?}", input)
					}
				}
			}
		}
	}
}
