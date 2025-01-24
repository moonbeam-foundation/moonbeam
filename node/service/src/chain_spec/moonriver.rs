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

//! Moonriver Chain Specifications and utilities for building them.
//!
//! Learn more about Substrate chain specifications at
//! https://substrate.dev/docs/en/knowledgebase/integrate/chain-spec

#[cfg(test)]
use crate::chain_spec::{derive_bip44_pairs_from_mnemonic, get_account_id_from_pair};
use crate::chain_spec::{generate_accounts, get_from_seed, Extensions};
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use moonriver_runtime::{
	currency::MOVR, AccountId, AuthorFilterConfig, AuthorMappingConfig, Balance, BalancesConfig,
	CrowdloanRewardsConfig, EVMConfig, EligibilityValue, EthereumChainIdConfig, EthereumConfig,
	GenesisAccount, InflationInfo, MaintenanceModeConfig, OpenTechCommitteeCollectiveConfig,
	ParachainInfoConfig, ParachainStakingConfig, PolkadotXcmConfig, Precompiles, Range,
	RuntimeGenesisConfig, TransactionPaymentConfig, TreasuryCouncilCollectiveConfig, HOURS,
	WASM_BINARY,
};
use nimbus_primitives::NimbusId;
use pallet_transaction_payment::Multiplier;
use sc_service::ChainType;
#[cfg(test)]
use sp_core::ecdsa;
use sp_runtime::{Perbill, Percent};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;

/// Generate a chain spec for use with the development service.
pub fn development_chain_spec(mnemonic: Option<String>, num_accounts: Option<u32>) -> ChainSpec {
	// Default mnemonic if none was provided
	let parent_mnemonic = mnemonic.unwrap_or_else(|| {
		"bottom drive obey lake curtain smoke basket hold race lonely fit walk".to_string()
	});
	let mut accounts = generate_accounts(parent_mnemonic, num_accounts.unwrap_or(10));
	// We add Gerald here
	accounts.push(AccountId::from(hex!(
		"6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b"
	)));

	ChainSpec::builder(
		WASM_BINARY.expect("WASM binary was not build, please build it!"),
		Extensions {
			relay_chain: "dev-service".into(),
			para_id: Default::default(),
		},
	)
	.with_name("Moonriver Development Testnet")
	.with_id("moonriver_dev")
	.with_chain_type(ChainType::Development)
	.with_properties(
		serde_json::from_str(
			"{\"tokenDecimals\": 18, \"tokenSymbol\": \"MOVR\", \"SS58Prefix\": 1285}",
		)
		.expect("Provided valid json map"),
	)
	.with_genesis_config(testnet_genesis(
		// Treasury Council members: Baltathar, Charleth and Dorothy
		vec![accounts[1], accounts[2], accounts[3]],
		// Open Tech committee members: Alith and Baltathar
		vec![accounts[0], accounts[1]],
		// Collator Candidate: Alice -> Alith
		vec![(
			AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")),
			get_from_seed::<NimbusId>("Alice"),
			100_000 * MOVR,
		)],
		// Delegations
		vec![],
		accounts.clone(),
		3_000_000 * MOVR,
		Default::default(), // para_id
		1281,               //ChainId
	))
	.build()
}

/// Generate a default spec for the parachain service. Use this as a starting point when launching
/// a custom chain.
pub fn get_chain_spec(para_id: ParaId) -> ChainSpec {
	ChainSpec::builder(
		WASM_BINARY.expect("WASM binary was not build, please build it!"),
		Extensions {
			relay_chain: "kusama-local".into(),
			para_id: para_id.into(),
		},
	)
	// TODO Apps depends on this string to determine whether the chain is an ethereum compat
	// or not. We should decide the proper strings, and update Apps accordingly.
	// Or maybe Apps can be smart enough to say if the string contains "moonbeam" at all...
	.with_name("Moonriver Local Testnet")
	.with_id("moonriver_local")
	.with_chain_type(ChainType::Local)
	.with_properties(
		serde_json::from_str(
			"{\"tokenDecimals\": 18, \"tokenSymbol\": \"MOVR\", \"SS58Prefix\": 1285}",
		)
		.expect("Provided valid json map"),
	)
	.with_genesis_config(testnet_genesis(
		// Treasury Council members: Baltathar, Charleth and Dorothy
		vec![
			AccountId::from(hex!("3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0")),
			AccountId::from(hex!("798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc")),
			AccountId::from(hex!("773539d4Ac0e786233D90A233654ccEE26a613D9")),
		],
		// Open Tech committee members: Alith and Baltathar
		vec![
			AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")),
			AccountId::from(hex!("3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0")),
		],
		// Collator Candidates
		vec![
			// Alice -> Alith
			(
				AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")),
				get_from_seed::<NimbusId>("Alice"),
				100_000 * MOVR,
			),
			// Bob -> Baltathar
			(
				AccountId::from(hex!("3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0")),
				get_from_seed::<NimbusId>("Bob"),
				100_000 * MOVR,
			),
		],
		// Delegations
		vec![],
		// Endowed: Alith, Baltathar, Charleth and Dorothy
		vec![
			AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac")),
			AccountId::from(hex!("3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0")),
			AccountId::from(hex!("798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc")),
			AccountId::from(hex!("773539d4Ac0e786233D90A233654ccEE26a613D9")),
		],
		3_000_000 * MOVR,
		para_id,
		1280, //ChainId
	))
	.build()
}

const COLLATOR_COMMISSION: Perbill = Perbill::from_percent(20);
const PARACHAIN_BOND_RESERVE_PERCENT: Percent = Percent::from_percent(30);
const BLOCKS_PER_ROUND: u32 = 2 * HOURS;
const BLOCKS_PER_YEAR: u32 = 31_557_600 / 12;
const NUM_SELECTED_CANDIDATES: u32 = 8;
pub fn moonriver_inflation_config() -> InflationInfo<Balance> {
	fn to_round_inflation(annual: Range<Perbill>) -> Range<Perbill> {
		use pallet_parachain_staking::inflation::perbill_annual_to_perbill_round;
		perbill_annual_to_perbill_round(
			annual,
			// rounds per year
			BLOCKS_PER_YEAR / BLOCKS_PER_ROUND,
		)
	}
	let annual = Range {
		min: Perbill::from_percent(4),
		ideal: Perbill::from_percent(5),
		max: Perbill::from_percent(5),
	};
	InflationInfo {
		// staking expectations
		expect: Range {
			min: 100_000 * MOVR,
			ideal: 200_000 * MOVR,
			max: 500_000 * MOVR,
		},
		// annual inflation
		annual,
		round: to_round_inflation(annual),
	}
}

pub fn testnet_genesis(
	treasury_council_members: Vec<AccountId>,
	open_tech_committee_members: Vec<AccountId>,
	candidates: Vec<(AccountId, NimbusId, Balance)>,
	delegations: Vec<(AccountId, AccountId, Balance, Percent)>,
	endowed_accounts: Vec<AccountId>,
	crowdloan_fund_pot: Balance,
	para_id: ParaId,
	chain_id: u64,
) -> serde_json::Value {
	// This is the simplest bytecode to revert without returning any data.
	// We will pre-deploy it under all of our precompiles to ensure they can be called from
	// within contracts.
	// (PUSH1 0x00 PUSH1 0x00 REVERT)
	let revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];

	let config = RuntimeGenesisConfig {
		system: Default::default(),
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 80))
				.collect(),
		},
		crowdloan_rewards: CrowdloanRewardsConfig {
			funded_amount: crowdloan_fund_pot,
		},
		parachain_info: ParachainInfoConfig {
			parachain_id: para_id,
			..Default::default()
		},
		ethereum_chain_id: EthereumChainIdConfig {
			chain_id,
			..Default::default()
		},
		evm: EVMConfig {
			// We need _some_ code inserted at the precompile address so that
			// the evm will actually call the address.
			accounts: Precompiles::used_addresses()
				.map(|addr| {
					(
						addr.into(),
						GenesisAccount {
							nonce: Default::default(),
							balance: Default::default(),
							storage: Default::default(),
							code: revert_bytecode.clone(),
						},
					)
				})
				.collect(),
			..Default::default()
		},
		ethereum: EthereumConfig {
			..Default::default()
		},
		parachain_staking: ParachainStakingConfig {
			candidates: candidates
				.iter()
				.cloned()
				.map(|(account, _, bond)| (account, bond))
				.collect(),
			delegations,
			inflation_config: moonriver_inflation_config(),
			collator_commission: COLLATOR_COMMISSION,
			parachain_bond_reserve_percent: PARACHAIN_BOND_RESERVE_PERCENT,
			blocks_per_round: BLOCKS_PER_ROUND,
			num_selected_candidates: NUM_SELECTED_CANDIDATES,
		},
		treasury_council_collective: TreasuryCouncilCollectiveConfig {
			phantom: Default::default(),
			members: treasury_council_members,
		},
		open_tech_committee_collective: OpenTechCommitteeCollectiveConfig {
			phantom: Default::default(),
			members: open_tech_committee_members,
		},
		author_filter: AuthorFilterConfig {
			eligible_count: EligibilityValue::new_unchecked(50),
			..Default::default()
		},
		author_mapping: AuthorMappingConfig {
			mappings: candidates
				.iter()
				.cloned()
				.map(|(account_id, author_id, _)| (author_id, account_id))
				.collect(),
		},
		proxy_genesis_companion: Default::default(),
		treasury: Default::default(),
		migrations: Default::default(),
		maintenance_mode: MaintenanceModeConfig {
			start_in_maintenance_mode: false,
			..Default::default()
		},
		// This should initialize it to whatever we have set in the pallet
		polkadot_xcm: PolkadotXcmConfig::default(),
		transaction_payment: TransactionPaymentConfig {
			multiplier: Multiplier::from(10u128),
			..Default::default()
		},
	};

	serde_json::to_value(&config).expect("Could not build genesis config.")
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
		let first_account = get_account_id_from_pair(pairs.first().unwrap().clone()).unwrap();
		let last_account = get_account_id_from_pair(pairs.last().unwrap().clone()).unwrap();

		let expected_first_account =
			AccountId::from(hex!("f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac"));
		let expected_last_account =
			AccountId::from(hex!("2898FE7a42Be376C8BC7AF536A940F7Fd5aDd423"));
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
		let first_account = get_account_id_from_pair(pairs.first().unwrap().clone()).unwrap();
		let last_account = get_account_id_from_pair(pairs.last().unwrap().clone()).unwrap();

		let expected_first_account =
			AccountId::from(hex!("1e56ca71b596f2b784a27a2fdffef053dbdeff83"));
		let expected_last_account =
			AccountId::from(hex!("4148202BF0c0Ad7697Cff87EbB83340C80c947f8"));
		assert_eq!(first_account, expected_first_account);
		assert_eq!(last_account, expected_last_account);
		assert_eq!(pairs.len(), 20);
	}
}
