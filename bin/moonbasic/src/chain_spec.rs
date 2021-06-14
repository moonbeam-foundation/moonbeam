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
use cumulus_primitives_core::ParaId;
use evm::GenesisAccount;
use log::debug;
use moonbase_runtime::{
	currency::UNITS, AccountId, AuthorFilterConfig, AuthorMappingConfig, Balance, BalancesConfig,
	CouncilCollectiveConfig, CrowdloanRewardsConfig, DemocracyConfig, EVMConfig,
	EthereumChainIdConfig, EthereumConfig, GenesisConfig, InflationInfo, ParachainInfoConfig,
	ParachainStakingConfig, Precompiles, Range, SchedulerConfig, SudoConfig, SystemConfig,
	TechComitteeCollectiveConfig, WASM_BINARY,
};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

use nimbus_primitives::NimbusId;
use sp_core::{ecdsa, Pair, Public, H160, H256};
use sp_runtime::{
	traits::{BlakeTwo256, Hash},
	Perbill,
};
use std::convert::TryInto;
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

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

//TODO I think I can get rid of this extension entirely.
/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
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
	let accounts = generate_accounts(parent_mnemonic, num_accounts.unwrap_or(10));
	ChainSpec::from_genesis(
		"Moonbase Development Testnet",
		"development",
		ChainType::Development,
		move || {
			testnet_genesis(
				// Alith is Sudo
				accounts[0],
				// Collator Candidate: Alice -> Alith
				vec![(
					accounts[0],
					get_from_seed::<NimbusId>("Alice"),
					1_000 * UNITS,
				)],
				// Nominations
				vec![],
				accounts.clone(),
				3_000_000 * UNITS,
				Default::default(), // para_id
				1281,               //ChainId
			)
		},
		vec![],
		None,
		None,
		Some(serde_json::from_str("{\"tokenDecimals\": 18}").expect("Provided valid json map")),
		Extensions {
			relay_chain: "dev-service".into(),
			para_id: Default::default(),
		},
	)
}

pub fn moonbeam_inflation_config() -> InflationInfo<Balance> {
	InflationInfo {
		expect: Range {
			min: 100_000 * UNITS,
			ideal: 200_000 * UNITS,
			max: 500_000 * UNITS,
		},
		annual: Range {
			min: Perbill::from_percent(4),
			ideal: Perbill::from_percent(5),
			max: Perbill::from_percent(5),
		},
		// 8766 rounds (hours) in a year
		round: Range {
			min: Perbill::from_parts(Perbill::from_percent(4).deconstruct() / 8766),
			ideal: Perbill::from_parts(Perbill::from_percent(5).deconstruct() / 8766),
			max: Perbill::from_parts(Perbill::from_percent(5).deconstruct() / 8766),
		},
	}
}

pub fn testnet_genesis(
	root_key: AccountId,
	candidates: Vec<(AccountId, NimbusId, Balance)>,
	nominations: Vec<(AccountId, AccountId, Balance)>,
	endowed_accounts: Vec<AccountId>,
	crowdloan_fund_pot: Balance,
	para_id: ParaId,
	chain_id: u64,
) -> GenesisConfig {
	// This is supposed the be the simplest bytecode to revert without returning any data.
	// We will pre-deploy it under all of our precompiles to ensure they can be called from
	// within contracts. TODO We should have a test to ensure this is the right bytecode.
	// (PUSH1 0x00 PUSH1 0x00 REVERT)
	let revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];

	GenesisConfig {
		frame_system: SystemConfig {
			code: WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 80))
				.collect(),
		},
		pallet_crowdloan_rewards: CrowdloanRewardsConfig {
			funded_amount: crowdloan_fund_pot,
		},
		pallet_sudo: SudoConfig { key: root_key },
		parachain_info: ParachainInfoConfig {
			parachain_id: para_id,
		},
		pallet_ethereum_chain_id: EthereumChainIdConfig { chain_id },
		pallet_evm: EVMConfig {
			// We need _some_ code inserted at the precompile address so that
			// the evm will actually call the address.
			accounts: Precompiles::used_addresses()
				.map(|addr| {
					(
						addr,
						GenesisAccount {
							nonce: Default::default(),
							balance: Default::default(),
							storage: Default::default(),
							code: revert_bytecode.clone(),
						},
					)
				})
				.collect(),
		},
		pallet_ethereum: EthereumConfig {},
		pallet_democracy: DemocracyConfig::default(),
		pallet_scheduler: SchedulerConfig {},
		parachain_staking: ParachainStakingConfig {
			candidates: candidates
				.iter()
				.cloned()
				.map(|(account, _, bond)| (account, bond))
				.collect(),
			nominations,
			inflation_config: moonbeam_inflation_config(),
		},
		pallet_collective_Instance1: CouncilCollectiveConfig {
			phantom: Default::default(),
			members: vec![], // TODO : Set members
		},
		pallet_collective_Instance2: TechComitteeCollectiveConfig {
			phantom: Default::default(),
			members: vec![], // TODO : Set members
		},
		pallet_author_slot_filter: AuthorFilterConfig {
			eligible_ratio: sp_runtime::Percent::from_percent(50),
		},
		pallet_author_mapping: AuthorMappingConfig {
			mappings: candidates
				.iter()
				.cloned()
				.map(|(account_id, author_id, _)| (author_id, account_id))
				.collect(),
		},
		pallet_treasury: Default::default(),
	}
}
