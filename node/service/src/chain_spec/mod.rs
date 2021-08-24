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
use bip39::{Language, Mnemonic, Seed};
use log::debug;
pub use moonbeam_core_primitives::AccountId;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use sp_core::{ecdsa, Pair, Public, H160, H256};
use sp_runtime::traits::{BlakeTwo256, Hash};
use std::convert::TryInto;
use tiny_hderive::bip32::ExtendedPrivKey;

pub mod fake_spec;
#[cfg(feature = "moonbase-runtime")]
pub mod moonbase;
#[cfg(feature = "moonbeam-runtime")]
pub mod moonbeam;
#[cfg(feature = "moonriver-runtime")]
pub mod moonriver;
#[cfg(feature = "moonbeam-runtime")]
pub mod test_spec;

#[cfg(not(feature = "moonbase-runtime"))]
pub mod moonbase {
	pub type ChainSpec = crate::chain_spec::fake_spec::FakeSpec;
	pub fn chain_spec_from_json_file(_: std::path::PathBuf) -> Result<ChainSpec, String> {
		panic!("moonbase runtime not enabled")
	}
	pub fn development_chain_spec(_: Option<String>, _: Option<u32>) -> ChainSpec {
		panic!("moonbase runtime not enabled")
	}
}
#[cfg(not(feature = "moonriver-runtime"))]
pub mod moonriver {
	pub type ChainSpec = crate::chain_spec::fake_spec::FakeSpec;
	pub fn chain_spec_from_json_file(_: std::path::PathBuf) -> Result<ChainSpec, String> {
		panic!("moonriver runtime not enabled")
	}
	pub fn development_chain_spec(_: Option<String>, _: Option<u32>) -> ChainSpec {
		panic!("moonriver runtime not enabled")
	}
}
#[cfg(not(feature = "moonbeam-runtime"))]
pub mod moonbeam {
	pub type ChainSpec = crate::chain_spec::fake_spec::FakeSpec;
	pub fn chain_spec_from_json_file(_: std::path::PathBuf) -> Result<ChainSpec, String> {
		panic!("moonbeam runtime not enabled")
	}
	pub fn development_chain_spec(_: Option<String>, _: Option<u32>) -> ChainSpec {
		panic!("moonbeam runtime not enabled")
	}
}

#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension, ChainSpecGroup)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

/// Helper function to derive `num_accounts` child pairs from mnemonics
/// Substrate derive function cannot be used because the derivation is different than Ethereum's
/// https://substrate.dev/rustdocs/v2.0.0/src/sp_core/ecdsa.rs.html#460-470
pub fn derive_bip44_pairs_from_mnemonic<TPublic: Public>(
	mnemonic: &str,
	num_accounts: u32,
) -> Vec<TPublic::Pair> {
	let seed = Mnemonic::from_phrase(mnemonic, Language::English)
		.map(|x| Seed::new(&x, ""))
		.expect("Wrong mnemonic provided");

	let mut childs = Vec::new();
	for i in 0..num_accounts {
		if let Some(child_pair) =
			ExtendedPrivKey::derive(seed.as_bytes(), format!("m/44'/60'/0'/0/{}", i).as_ref())
				.ok()
				.map(|account| TPublic::Pair::from_seed_slice(&account.secret()).ok())
				.flatten()
		{
			childs.push(child_pair);
		} else {
			log::error!("An error ocurred while deriving key {} from parent", i)
		}
	}
	childs
}

/// Helper function to get an AccountId from Key Pair
/// We need the full decompressed public key to derive an ethereum-style account
/// Substrate does not provide a method to obtain the full decompressed public key
/// Therefore, this function uses the secp256k1_ecdsa_recover method to recover the full key
/// A solution without using the private key would imply solving the secp256k1 curve equation
/// The latter is currently not possible with current substrate methods
pub fn get_account_id_from_pair<TPublic: Public>(pair: TPublic::Pair) -> Option<AccountId> {
	let test_message = [1u8; 32];
	let signature: [u8; 65] = pair.sign(&test_message).as_ref().try_into().ok()?;
	let pubkey = sp_io::crypto::secp256k1_ecdsa_recover(
		&signature,
		BlakeTwo256::hash_of(&test_message).as_fixed_bytes(),
	)
	.ok()?;
	Some(H160::from(H256::from_slice(
		Keccak256::digest(&pubkey).as_slice(),
	)))
}

/// Function to generate accounts given a mnemonic and a number of child accounts to be generated
/// Defaults to a default mnemonic if no mnemonic is supplied
pub fn generate_accounts(mnemonic: String, num_accounts: u32) -> Vec<AccountId> {
	let childs = derive_bip44_pairs_from_mnemonic::<ecdsa::Public>(&mnemonic, num_accounts);
	debug!("Account Generation");
	childs
		.iter()
		.map(|par| {
			let account = get_account_id_from_pair::<ecdsa::Public>(par.clone());
			debug!(
				"private_key {} --------> Account {:x?}",
				sp_core::hexdisplay::HexDisplay::from(&par.clone().seed()),
				account
			);
			account
		})
		.flatten()
		.collect()
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}
