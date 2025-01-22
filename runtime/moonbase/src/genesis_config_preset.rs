extern crate alloc;

use crate::{
	currency::UNIT, AccountId, AuthorFilterConfig, AuthorMappingConfig, Balance, BalancesConfig,
	CrowdloanRewardsConfig, EVMConfig, EligibilityValue, EthereumChainIdConfig, EthereumConfig,
	InflationInfo, MaintenanceModeConfig, MoonbeamOrbitersConfig,
	OpenTechCommitteeCollectiveConfig, ParachainInfoConfig, ParachainStakingConfig,
	PolkadotXcmConfig, Precompiles, Range, RuntimeGenesisConfig, SudoConfig,
	TransactionPaymentConfig, TreasuryCouncilCollectiveConfig, XcmTransactorConfig, HOURS,
};
use alloc::{vec, vec::Vec};
use cumulus_primitives_core::ParaId;
pub use fp_evm::GenesisAccount;
use nimbus_primitives::NimbusId;
use pallet_transaction_payment::Multiplier;
use sp_runtime::{traits::One, Perbill, Percent};

const COLLATOR_COMMISSION: Perbill = Perbill::from_percent(20);
const PARACHAIN_BOND_RESERVE_PERCENT: Percent = Percent::from_percent(30);
const BLOCKS_PER_ROUND: u32 = 2 * HOURS;
const BLOCKS_PER_YEAR: u32 = 31_557_600 / 6;
const NUM_SELECTED_CANDIDATES: u32 = 8;
pub fn moonbase_inflation_config() -> InflationInfo<Balance> {
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
			min: 100_000 * UNIT,
			ideal: 200_000 * UNIT,
			max: 500_000 * UNIT,
		},
		// annual inflation
		annual,
		round: to_round_inflation(annual),
	}
}

pub fn testnet_genesis(
	root_key: AccountId,
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
		sudo: SudoConfig {
			key: Some(root_key),
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
			inflation_config: moonbase_inflation_config(),
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
			multiplier: Multiplier::from(8u128),
			..Default::default()
		},
		moonbeam_orbiters: MoonbeamOrbitersConfig {
			min_orbiter_deposit: One::one(),
		},
		xcm_transactor: XcmTransactorConfig {
			relay_indices: moonbeam_relay_encoder::westend::WESTEND_RELAY_INDICES,
			..Default::default()
		},
	};

	serde_json::to_value(&config).expect("Could not build genesis config.")
}
