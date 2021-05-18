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

//! Moonbase Chain Specifications and utilities for building them.
//!
//! Learn more about Substrate chain specifications at
//! https://substrate.dev/docs/en/knowledgebase/integrate/chain-spec

#[cfg(test)]
use crate::chain_spec::{derive_bip44_pairs_from_mnemonic, get_account_id_from_pair};
use crate::chain_spec::{generate_accounts, Extensions};
use cumulus_primitives_core::ParaId;
use evm::GenesisAccount;
use moonbase_runtime::{
	AccountId, AuthorFilterConfig, Balance, BalancesConfig, CouncilCollectiveConfig,
	DemocracyConfig, EVMConfig, EthereumChainIdConfig, EthereumConfig, GenesisConfig,
	InflationInfo, ParachainInfoConfig, ParachainStakingConfig, Range, SchedulerConfig, SudoConfig,
	SystemConfig, TechComitteeCollectiveConfig, GLMR, WASM_BINARY,
};
use sc_service::ChainType;
use sp_core::H160;
use sp_runtime::Perbill;
use std::str::FromStr;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

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
		"moonbase_dev",
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
		None,
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
	stakers: Vec<(AccountId, Option<AccountId>, Balance)>,
	inflation_config: InflationInfo<Balance>,
	endowed_accounts: Vec<AccountId>,
	para_id: ParaId,
	chain_id: u64,
) -> GenesisConfig {
	// This is supposed the be the simplest bytecode to revert without returning any data.
	// We will pre-deploy it under all of our precompiles to ensure they can be called from
	// within contracts. TODO We should have a test to ensure this is the right bytecode.
	// (PUSH1 0x00 PUSH1 0x00 REVERT)
	let revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];
	// TODO consider whether this should be imported from moonbeam precompiles
	let precompile_addresses = vec![1, 2, 3, 4, 5, 6, 7, 8, 1024, 1025, 2048]
		.into_iter()
		.map(H160::from_low_u64_be);
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
		pallet_sudo: SudoConfig { key: root_key },
		parachain_info: ParachainInfoConfig {
			parachain_id: para_id,
		},
		pallet_ethereum_chain_id: EthereumChainIdConfig { chain_id },
		pallet_evm: EVMConfig {
			// We need _some_ code inserted at the precompile address so that
			// the evm will actually call the address.
			// TODO Cleanly fetch the addresses from
			// the runtime/moonbeam precompiles and systematically fill them with code
			// that will revert if it is called by accident (it shouldn't be because
			// it is shadowed by the precompile).
			// This one is for the parachain staking precompile wrappers
			accounts: precompile_addresses
				.map(|a| {
					(
						a,
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
		pallet_democracy: DemocracyConfig {},
		pallet_scheduler: SchedulerConfig {},
		parachain_staking: ParachainStakingConfig {
			stakers,
			inflation_config,
		},
		pallet_collective_Instance1: CouncilCollectiveConfig {
			phantom: Default::default(),
			members: vec![], // TODO : Set members
		},
		pallet_collective_Instance2: TechComitteeCollectiveConfig {
			phantom: Default::default(),
			members: vec![], // TODO : Set members
		},
		pallet_author_slot_filter: AuthorFilterConfig { eligible_ratio: 50 },
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
