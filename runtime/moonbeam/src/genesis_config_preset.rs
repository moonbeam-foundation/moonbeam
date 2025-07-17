// Copyright 2019-2025 PureStake Inc.
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

extern crate alloc;

use crate::{
	currency::GLMR, currency::SUPPLY_FACTOR, AccountId, AuthorFilterConfig, AuthorMappingConfig,
	Balance, Balances, BalancesConfig, BridgeKusamaGrandpaConfig, BridgeKusamaMessagesConfig,
	BridgeKusamaParachainsConfig, BridgeXcmOverMoonriverConfig, CrowdloanRewardsConfig, EVMConfig,
	EligibilityValue, EthereumChainIdConfig, EthereumConfig, EvmForeignAssetsConfig, InflationInfo,
	MaintenanceModeConfig, OpenTechCommitteeCollectiveConfig, ParachainInfoConfig,
	ParachainStakingConfig, PolkadotXcmConfig, Precompiles, Range, RuntimeGenesisConfig,
	TransactionPaymentConfig, TreasuryCouncilCollectiveConfig, XcmWeightTraderConfig, HOURS,
};
use alloc::{vec, vec::Vec};
use bp_messages::MessagesOperatingMode;
use bp_runtime::BasicOperatingMode;
use cumulus_primitives_core::ParaId;
use fp_evm::GenesisAccount;
use frame_support::pallet_prelude::PalletInfoAccess;
use nimbus_primitives::NimbusId;
use pallet_moonbeam_foreign_assets::EvmForeignAssetInfo;
use pallet_transaction_payment::Multiplier;
use pallet_xcm_weight_trader::XcmWeightTraderAssetInfo;
use sp_genesis_builder::PresetId;
use sp_keyring::Sr25519Keyring;
use sp_runtime::{Perbill, Percent};
use xcm::prelude::{GlobalConsensus, Junctions, Location, NetworkId, PalletInstance, Parachain};

const COLLATOR_COMMISSION: Perbill = Perbill::from_percent(20);
const PARACHAIN_BOND_RESERVE_PERCENT: Percent = Percent::from_percent(30);
const BLOCKS_PER_ROUND: u32 = 6 * HOURS;
const BLOCKS_PER_YEAR: u32 = 31_557_600 / 12;
const NUM_SELECTED_CANDIDATES: u32 = 8;

pub fn moonbeam_inflation_config() -> InflationInfo<Balance> {
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
			min: 100_000 * GLMR * SUPPLY_FACTOR,
			ideal: 200_000 * GLMR * SUPPLY_FACTOR,
			max: 500_000 * GLMR * SUPPLY_FACTOR,
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
				.map(|k| (k, 1 << 110))
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
			inflation_config: moonbeam_inflation_config(),
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
		polkadot_xcm: PolkadotXcmConfig {
			supported_version: vec![
				// Required for bridging Moonbeam with Moonriver
				(
					bp_moonriver::GlobalConsensusLocation::get(),
					xcm::latest::VERSION,
				),
			],
			..Default::default()
		},
		transaction_payment: TransactionPaymentConfig {
			multiplier: Multiplier::from(8u128),
			..Default::default()
		},
		evm_foreign_assets: EvmForeignAssetsConfig {
			assets: vec![EvmForeignAssetInfo {
				asset_id: 1,
				name: b"xcMOVR".to_vec().try_into().expect("Invalid asset name"),
				symbol: b"xcMOVR".to_vec().try_into().expect("Invalid asset symbol"),
				decimals: 18,
				xcm_location: Location::new(
					2,
					[
						GlobalConsensus(crate::bridge_config::KusamaGlobalConsensusNetwork::get()),
						Parachain(<bp_moonriver::Moonriver as bp_runtime::Parachain>::PARACHAIN_ID),
						PalletInstance(<Balances as PalletInfoAccess>::index() as u8),
					],
				),
			}],
			_phantom: Default::default(),
		},
		xcm_weight_trader: XcmWeightTraderConfig {
			assets: vec![XcmWeightTraderAssetInfo {
				location: Location::new(
					2,
					[
						GlobalConsensus(crate::bridge_config::KusamaGlobalConsensusNetwork::get()),
						Parachain(<bp_moonriver::Moonriver as bp_runtime::Parachain>::PARACHAIN_ID),
						PalletInstance(<Balances as PalletInfoAccess>::index() as u8),
					],
				),
				relative_price: GLMR,
			}],
			_phantom: Default::default(),
		},
		bridge_kusama_grandpa: BridgeKusamaGrandpaConfig {
			owner: Some(endowed_accounts[0]),
			init_data: None,
		},
		bridge_kusama_parachains: BridgeKusamaParachainsConfig {
			owner: Some(endowed_accounts[0]),
			operating_mode: BasicOperatingMode::Normal,
			..Default::default()
		},
		bridge_kusama_messages: BridgeKusamaMessagesConfig {
			owner: Some(endowed_accounts[0]),
			opened_lanes: vec![],
			operating_mode: MessagesOperatingMode::Basic(BasicOperatingMode::Normal),
			_phantom: Default::default(),
		},
		bridge_xcm_over_moonriver: BridgeXcmOverMoonriverConfig {
			opened_bridges: vec![(
				Location::new(
					1,
					[Parachain(
						<bp_moonbeam::Moonbeam as bp_runtime::Parachain>::PARACHAIN_ID,
					)],
				),
				Junctions::from([
					NetworkId::Kusama.into(),
					Parachain(<bp_moonriver::Moonriver as bp_runtime::Parachain>::PARACHAIN_ID),
				]),
				Some(Default::default()),
			)],
			_phantom: Default::default(),
		},
	};

	serde_json::to_value(&config).expect("Could not build genesis config.")
}

/// Generate a chain spec for use with the development service.
pub fn development() -> serde_json::Value {
	testnet_genesis(
		// Treasury Council members: Baltathar, Charleth and Dorothy
		vec![
			AccountId::from(sp_core::hex2array!(
				"3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0"
			)),
			AccountId::from(sp_core::hex2array!(
				"798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc"
			)),
			AccountId::from(sp_core::hex2array!(
				"773539d4Ac0e786233D90A233654ccEE26a613D9"
			)),
		],
		// Open Tech committee members: Alith and Baltathar
		vec![
			AccountId::from(sp_core::hex2array!(
				"f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac"
			)),
			AccountId::from(sp_core::hex2array!(
				"3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0"
			)),
		],
		// Collator Candidate: Alice -> Alith
		vec![(
			AccountId::from(sp_core::hex2array!(
				"f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac"
			)),
			NimbusId::from(Sr25519Keyring::Alice.public()),
			20_000 * GLMR * SUPPLY_FACTOR,
		)],
		// Delegations
		vec![],
		vec![
			AccountId::from(sp_core::hex2array!(
				"f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac"
			)),
			AccountId::from(sp_core::hex2array!(
				"3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0"
			)),
			AccountId::from(sp_core::hex2array!(
				"798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc"
			)),
			AccountId::from(sp_core::hex2array!(
				"773539d4Ac0e786233D90A233654ccEE26a613D9"
			)),
		],
		1_500_000 * GLMR * SUPPLY_FACTOR,
		Default::default(), // para_id
		1281,               //ChainId
	)
}

/// Provides the JSON representation of predefined genesis config for given `id`.
pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
	let patch = match id.as_str() {
		sp_genesis_builder::DEV_RUNTIME_PRESET => development(),
		_ => return None,
	};
	Some(
		serde_json::to_string(&patch)
			.expect("serialization to json is expected to work. qed.")
			.into_bytes(),
	)
}

/// List of supported presets.
pub fn preset_names() -> Vec<PresetId> {
	vec![PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET)]
}
