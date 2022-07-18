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

use bip32::{
	Error as Bip32Error, ExtendedPrivateKey, PrivateKey as PrivateKeyT, PrivateKeyBytes,
	PublicKey as PublicKeyT, PublicKeyBytes,
};
use bip39::{Language, Mnemonic, MnemonicType, Seed};
use clap::Parser;
use libsecp256k1::{PublicKey, SecretKey};
use primitive_types::H256;
use sp_runtime::traits::IdentifyAccount;

#[derive(Debug, Parser)]
pub struct GenerateAccountKey {
	/// Generate 12 words mnemonic instead of 24
	#[clap(long, short = 'w')]
	w12: bool,

	/// Specify the mnemonic
	#[clap(long, short = 'm')]
	mnemonic: Option<String>,

	/// The account index to use in the derivation path
	#[clap(long = "account-index", short = 'a')]
	account_index: Option<u32>,
}

impl GenerateAccountKey {
	pub fn run(&self) {
		// Retrieve the mnemonic from the args or generate random ones
		let mnemonic = if let Some(phrase) = &self.mnemonic {
			Mnemonic::from_phrase(phrase, Language::English).expect("invalid mnemonic")
		} else {
			match self.w12 {
				true => Mnemonic::new(MnemonicType::Words12, Language::English),
				false => Mnemonic::new(MnemonicType::Words24, Language::English),
			}
		};

		// Retrieves the seed from the mnemonic
		let seed = Seed::new(&mnemonic, "");
		let derivation_path = format!("m/44'/60'/0'/0/{}", self.account_index.unwrap_or(0));
		let private_key = if let Some(private_key) =
			derivation_path.parse().ok().and_then(|derivation_path| {
				let extended = ExtendedPrivateKey::<Secp256k1SecretKey>::derive_from_path(
					&seed,
					&derivation_path,
				)
				.expect("invalid extended private key");
				Some(extended.private_key().0)
			}) {
			private_key
		} else {
			panic!("invalid extended private key");
		};

		// Retrieves the public key
		let public_key = PublicKey::from_secret_key(&private_key);

		// Convert into Ethereum-style address.
		let signer: account::EthereumSigner = public_key.into();
		let address = signer.into_account();

		println!("Address:      {:?}", address);
		println!("Mnemonic:     {}", mnemonic.phrase());
		println!("Private Key:  {:?}", H256::from(private_key.serialize()));
		println!("Path:         {}", derivation_path);
	}
}

// `libsecp256k1::PublicKey` wrapped type
pub struct Secp256k1PublicKey(pub PublicKey);
// `libsecp256k1::Secret`  wrapped type
pub struct Secp256k1SecretKey(pub SecretKey);

impl PublicKeyT for Secp256k1PublicKey {
	fn from_bytes(bytes: PublicKeyBytes) -> Result<Self, Bip32Error> {
		let public = PublicKey::parse_compressed(&bytes).map_err(|_| return Bip32Error::Decode)?;
		Ok(Self(public))
	}

	fn to_bytes(&self) -> PublicKeyBytes {
		self.0.serialize_compressed()
	}

	fn derive_child(&self, other: PrivateKeyBytes) -> Result<Self, Bip32Error> {
		let mut child = self.0.clone();
		let secret = SecretKey::parse(&other).map_err(|_| return Bip32Error::Decode)?;
		let _ = child.tweak_add_assign(&secret);
		Ok(Self(child))
	}
}

impl PrivateKeyT for Secp256k1SecretKey {
	type PublicKey = Secp256k1PublicKey;

	fn from_bytes(bytes: &PrivateKeyBytes) -> Result<Self, Bip32Error> {
		let secret = SecretKey::parse(&bytes).map_err(|_| return Bip32Error::Decode)?;
		Ok(Self(secret))
	}

	fn to_bytes(&self) -> PrivateKeyBytes {
		self.0.serialize()
	}

	fn derive_child(&self, other: PrivateKeyBytes) -> Result<Self, Bip32Error> {
		let mut child = self.0.clone();
		let secret = SecretKey::parse(&other).map_err(|_| return Bip32Error::Decode)?;
		let _ = child.tweak_add_assign(&secret);
		Ok(Self(child))
	}

	fn public_key(&self) -> Self::PublicKey {
		Secp256k1PublicKey(PublicKey::from_secret_key(&self.0))
	}
}
