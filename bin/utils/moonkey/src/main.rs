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

//! Generate an Ethereum account.

use bip39::{Language, Mnemonic, MnemonicType, Seed};
use clap::{AppSettings, Clap};
use primitive_types::{H160, H256};
use secp256k1::{PublicKey, SecretKey};
use sha3::{Digest, Keccak256};
use tiny_hderive::bip32::ExtendedPrivKey;

#[derive(Clap)]
#[clap(version = "1.0", author = "PureStake")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
	/// Generate 12 words mnemonic instead of 24
	#[clap(long)]
	w12: bool,

	/// Specify the mnemonic
	#[clap(long)]
	mnemonic: Option<String>,

	/// The account index to use in the derivation path
	#[clap(long = "account-index")]
	account_index: Option<u32>,
}

fn main() {
	// Parses the options
	let opts: Opts = Opts::parse();

	// Retrieve the mnemonic from the args or generate random ones
	let mnemonic = if let Some(phrase) = opts.mnemonic {
		Mnemonic::from_phrase(phrase, Language::English).unwrap()
	} else {
		match opts.w12 {
			true => Mnemonic::new(MnemonicType::Words12, Language::English),
			false => Mnemonic::new(MnemonicType::Words24, Language::English),
		}
	};

	// Retrieves the seed from the mnemonic
	let seed = Seed::new(&mnemonic, "");

	// Generate the derivation path from the account-index
	let derivation_path = format!("m/44'/60'/0'/0/{}", opts.account_index.unwrap_or(0));

	// Derives the private key from
	let ext = ExtendedPrivKey::derive(seed.as_bytes(), derivation_path.as_str()).unwrap();
	let private_key = SecretKey::parse_slice(&ext.secret()).unwrap();

	// Retrieves the public key
	let public_key = PublicKey::from_secret_key(&private_key);

	// Compresses to a H160 address
	let mut res = [0u8; 64];
	res.copy_from_slice(&public_key.serialize()[1..65]);
	let address = H160::from(H256::from_slice(Keccak256::digest(&res).as_slice()));

	println!("Address:      {:?}", address);
	println!("Mnemonic:     {}", mnemonic.phrase());
	println!("Private Key:  {:?}", H256::from(private_key.serialize()));
	println!("Path:         {}", derivation_path);
}
