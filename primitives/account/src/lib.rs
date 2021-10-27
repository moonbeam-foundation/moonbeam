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

//! The Ethereum Signature implementation.
//!
//! It includes the Verify and IdentifyAccount traits for the AccountId20

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sha3::{Digest, Keccak256};
use sp_core::{ecdsa, H160, H256};

#[cfg(feature = "std")]
pub use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, sp_core::RuntimeDebug, TypeInfo)]
pub struct EthereumSignature(ecdsa::Signature);

impl From<ecdsa::Signature> for EthereumSignature {
	fn from(x: ecdsa::Signature) -> Self {
		EthereumSignature(x)
	}
}

impl sp_runtime::traits::Verify for EthereumSignature {
	type Signer = EthereumSigner;
	fn verify<L: sp_runtime::traits::Lazy<[u8]>>(&self, mut msg: L, signer: &H160) -> bool {
		let mut m = [0u8; 32];
		m.copy_from_slice(Keccak256::digest(msg.get()).as_slice());
		match sp_io::crypto::secp256k1_ecdsa_recover(self.0.as_ref(), &m) {
			Ok(pubkey) => {
				// TODO This conversion could use a comment. Why H256 first, then H160?
				H160::from(H256::from_slice(Keccak256::digest(&pubkey).as_slice())) == *signer
			}
			Err(sp_io::EcdsaVerifyError::BadRS) => {
				log::error!(target: "evm", "Error recovering: Incorrect value of R or S");
				false
			}
			Err(sp_io::EcdsaVerifyError::BadV) => {
				log::error!(target: "evm", "Error recovering: Incorrect value of V");
				false
			}
			Err(sp_io::EcdsaVerifyError::BadSignature) => {
				log::error!(target: "evm", "Error recovering: Invalid signature");
				false
			}
		}
	}
}

/// Public key for an Ethereum / H160 compatible account
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Encode, Decode, sp_core::RuntimeDebug)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub struct EthereumSigner([u8; 20]);

impl sp_runtime::traits::IdentifyAccount for EthereumSigner {
	type AccountId = H160;
	fn into_account(self) -> H160 {
		self.0.into()
	}
}

impl From<[u8; 20]> for EthereumSigner {
	fn from(x: [u8; 20]) -> Self {
		EthereumSigner(x)
	}
}

impl From<ecdsa::Public> for EthereumSigner {
	fn from(x: ecdsa::Public) -> Self {
		let decompressed = libsecp256k1::PublicKey::parse_slice(
			&x.0,
			Some(libsecp256k1::PublicKeyFormat::Compressed),
		)
		.expect("Wrong compressed public key provided")
		.serialize();
		let mut m = [0u8; 64];
		m.copy_from_slice(&decompressed[1..65]);
		let account = H160::from(H256::from_slice(Keccak256::digest(&m).as_slice()));
		EthereumSigner(account.into())
	}
}

impl From<libsecp256k1::PublicKey> for EthereumSigner {
	fn from(x: libsecp256k1::PublicKey) -> Self {
		let mut m = [0u8; 64];
		m.copy_from_slice(&x.serialize()[1..65]);
		let account = H160::from(H256::from_slice(Keccak256::digest(&m).as_slice()));
		EthereumSigner(account.into())
	}
}

#[cfg(feature = "std")]
impl std::fmt::Display for EthereumSigner {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(fmt, "ethereum signature: {:?}", H160::from_slice(&self.0))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use sp_core::{ecdsa, Pair};
	use sp_runtime::traits::IdentifyAccount;

	#[test]
	fn test_account_derivation_1() {
		// Test from https://asecuritysite.com/encryption/ethadd
		let secret_key =
			hex::decode("502f97299c472b88754accd412b7c9a6062ef3186fba0c0388365e1edec24875")
				.unwrap();
		let expected_hex_account = hex::decode("976f8456e4e2034179b284a23c0e0c8f6d3da50c").unwrap();

		let public_key = ecdsa::Pair::from_seed_slice(&secret_key).unwrap().public();
		let account: EthereumSigner = public_key.into();
		let expected_account = H160::from_slice(&expected_hex_account);
		assert_eq!(account.into_account(), expected_account);
	}
	#[test]
	fn test_account_derivation_2() {
		// Test from https://asecuritysite.com/encryption/ethadd
		let secret_key =
			hex::decode("0f02ba4d7f83e59eaa32eae9c3c4d99b68ce76decade21cdab7ecce8f4aef81a")
				.unwrap();
		let expected_hex_account = hex::decode("420e9f260b40af7e49440cead3069f8e82a5230f").unwrap();

		let public_key = ecdsa::Pair::from_seed_slice(&secret_key).unwrap().public();
		let account: EthereumSigner = public_key.into();
		let expected_account = H160::from_slice(&expected_hex_account);
		assert_eq!(account.into_account(), expected_account);
	}
}
