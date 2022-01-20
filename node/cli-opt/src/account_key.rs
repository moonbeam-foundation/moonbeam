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

use bip39::{Language, Mnemonic, MnemonicType, Seed};
use libsecp256k1::{PublicKey, SecretKey};
use primitive_types::H256;
use sp_runtime::traits::IdentifyAccount;
use structopt::StructOpt;
use tiny_hderive::bip32::ExtendedPrivKey;

#[derive(Debug, StructOpt)]
pub struct GenerateAccountKey {
	/// Generate 12 words mnemonic instead of 24
	#[structopt(long, short = "w")]
	w12: bool,

	/// Specify the mnemonic
	#[structopt(long, short = "m")]
	mnemonic: Option<String>,

	/// The account index to use in the derivation path
	#[structopt(long = "account-index", short = "a")]
	account_index: Option<u32>,
}

impl GenerateAccountKey {
	pub fn run(&self) {
		// Retrieve the mnemonic from the args or generate random ones
		let mnemonic = if let Some(phrase) = &self.mnemonic {
			Mnemonic::from_phrase(phrase, Language::English).unwrap()
		} else {
			match self.w12 {
				true => Mnemonic::new(MnemonicType::Words12, Language::English),
				false => Mnemonic::new(MnemonicType::Words24, Language::English),
			}
		};

		// Retrieves the seed from the mnemonic
		let seed = Seed::new(&mnemonic, "");

		// Generate the derivation path from the account-index
		let derivation_path = format!("m/44'/60'/0'/0/{}", self.account_index.unwrap_or(0));

		// Derives the private key from
		let ext = ExtendedPrivKey::derive(seed.as_bytes(), derivation_path.as_str()).unwrap();
		let private_key = SecretKey::parse_slice(&ext.secret()).unwrap();

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
