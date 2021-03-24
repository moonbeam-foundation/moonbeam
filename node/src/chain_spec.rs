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

//! Moonbeam Chain Specifications and utilities for building them.
//!
//! Learn more about Substrate chain specifications at
//! https://substrate.dev/docs/en/knowledgebase/integrate/chain-spec

use bip39::{Language, Mnemonic, Seed};
use cumulus_primitives::ParaId;
use log::debug;
use moonbeam_runtime::{
	AccountId, Balance, BalancesConfig, DemocracyConfig, EVMConfig, EthereumChainIdConfig,
	EthereumConfig, GenesisConfig, InflationInfo, ParachainInfoConfig, ParachainStakingConfig,
	Range, SchedulerConfig, SudoConfig, SystemConfig, GLMR, WASM_BINARY,
};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

use sp_core::{ecdsa, Pair, Public, H160, H256};
use sp_runtime::{
	traits::{BlakeTwo256, Hash},
	Perbill,
};
use std::convert::TryInto;
use std::{collections::BTreeMap, str::FromStr};
use tiny_hderive::bip32::ExtendedPrivKey;

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

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
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

/// Generate a chain spec for use with the development service.
pub fn development_chain_spec(mnemonic: Option<String>, num_accounts: Option<u32>) -> ChainSpec {
	// Default mnemonic if none was provided
	let parent_mnemonic = mnemonic.unwrap_or(
		"bottom drive obey lake curtain smoke basket hold race lonely fit walk".to_string(),
	);
	let mut accounts = generate_accounts(parent_mnemonic, num_accounts.unwrap_or(10));
	// We add Gerald here
	accounts.push(AccountId::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap());
	ChainSpec::from_genesis(
		"Moonbase Development Testnet",
		"development",
		ChainType::Development,
		move || {
			testnet_genesis(
				AccountId::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap(),
				// Validator
				vec![(
					AccountId::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap(),
					None,
					1_000 * GLMR,
				)],
				moonbeam_inflation_config(),
				accounts.clone(),
				Default::default(), // para_id
				1281,               //ChainId
			)
		},
		vec![],
		Some(
			TelemetryEndpoints::new(vec![("wss://telemetry.polkadot.io/submit/".to_string(), 0)])
				.expect("Polkadot Staging telemetry url is valid; qed"),
		),
		None,
		Some(serde_json::from_str("{\"tokenDecimals\": 18}").expect("Provided valid json map")),
		Extensions {
			relay_chain: "dev-service".into(),
			para_id: Default::default(),
		},
	)
}

/// Generate a default spec for the parachain service. Use this as a starting point when launching
/// a custom chain.
pub fn get_chain_spec(para_id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		// TODO Apps depends on this string to determine whether the chain is an ethereum compat
		// or not. We should decide the proper strings, and update Apps accordingly.
		// Or maybe Apps can be smart enough to say if the string contains "moonbeam" at all...
		"Moonbase Development Testnet",
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				AccountId::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap(),
				// Validator
				vec![(
					AccountId::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap(),
					None,
					1_000 * GLMR,
				)],
				moonbeam_inflation_config(),
				vec![AccountId::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap()],
				para_id,
				1280, //ChainId
			)
		},
		vec![],
		None,
		None,
		Some(serde_json::from_str("{\"tokenDecimals\": 18}").expect("Provided valid json map")),
		Extensions {
			relay_chain: "local_testnet".into(),
			para_id: para_id.into(),
		},
	)
}

pub fn moonbeam_inflation_config() -> InflationInfo<Balance> {
	InflationInfo {
		expect: Range {
			min: 100_000 * GLMR,
			ideal: 200_000 * GLMR,
			max: 500_000 * GLMR,
		},
		// 8766 rounds (hours) in a year
		round: Range {
			min: Perbill::from_parts(Perbill::from_percent(4).deconstruct() / 8766),
			ideal: Perbill::from_parts(Perbill::from_percent(5).deconstruct() / 8766),
			max: Perbill::from_parts(Perbill::from_percent(5).deconstruct() / 8766),
		},
	}
}

fn testnet_genesis(
	root_key: AccountId,
	stakers: Vec<(AccountId, Option<AccountId>, Balance)>,
	inflation_config: InflationInfo<Balance>,
	endowed_accounts: Vec<AccountId>,
	para_id: ParaId,
	chain_id: u64,
) -> GenesisConfig {
	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 80))
				.collect(),
		}),
		pallet_sudo: Some(SudoConfig { key: root_key }),
		parachain_info: Some(ParachainInfoConfig {
			parachain_id: para_id,
		}),
		pallet_ethereum_chain_id: Some(EthereumChainIdConfig { chain_id }),
		pallet_evm: Some(EVMConfig {
			accounts: BTreeMap::new(),
		}),
		pallet_ethereum: Some(EthereumConfig {}),
		pallet_democracy: Some(DemocracyConfig {}),
		pallet_scheduler: Some(SchedulerConfig {}),
		parachain_staking: Some(ParachainStakingConfig {
			stakers,
			inflation_config,
		}),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_derived_pairs_1() {
		let mnemonic =
			"bottom drive obey lake curtain smoke basket hold race lonely fit walk".to_string();
		let accounts = 10;
		let pairs = derive_bip44_pairs_from_mnemonic::<ecdsa::Public>(&mnemonic, accounts);
		let first_account =
			get_account_id_from_pair::<ecdsa::Public>(pairs.first().unwrap().clone()).unwrap();
		let last_account =
			get_account_id_from_pair::<ecdsa::Public>(pairs.last().unwrap().clone()).unwrap();

		let expected_first_account =
			AccountId::from_str("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac").unwrap();
		let expected_last_account =
			AccountId::from_str("2898FE7a42Be376C8BC7AF536A940F7Fd5aDd423").unwrap();
		assert_eq!(first_account, expected_first_account);
		assert_eq!(last_account, expected_last_account);
		assert_eq!(pairs.len(), 10);
	}
	#[test]
	fn test_derived_pairs_2() {
		let mnemonic =
			"slab nerve salon plastic filter inherit valve ozone crash thumb quality whale"
				.to_string();
		let accounts = 20;
		let pairs = derive_bip44_pairs_from_mnemonic::<ecdsa::Public>(&mnemonic, accounts);
		let first_account =
			get_account_id_from_pair::<ecdsa::Public>(pairs.first().unwrap().clone()).unwrap();
		let last_account =
			get_account_id_from_pair::<ecdsa::Public>(pairs.last().unwrap().clone()).unwrap();

		let expected_first_account =
			AccountId::from_str("1e56ca71b596f2b784a27a2fdffef053dbdeff83").unwrap();
		let expected_last_account =
			AccountId::from_str("4148202BF0c0Ad7697Cff87EbB83340C80c947f8").unwrap();
		assert_eq!(first_account, expected_first_account);
		assert_eq!(last_account, expected_last_account);
		assert_eq!(pairs.len(), 20);
	}
}
