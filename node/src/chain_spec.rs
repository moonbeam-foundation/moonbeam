use serde::{Serialize, Deserialize};
use serde_json::{json};

use sc_service;
use sc_chain_spec::ChainSpecExtension;
use sp_core::{Pair, Public, sr25519};
use sp_consensus_babe::{AuthorityId as BabeId};
use grandpa_primitives::{AuthorityId as GrandpaId};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_runtime::{Perbill, traits::{Verify, IdentifyAccount}};
use sc_telemetry::TelemetryEndpoints;

use pallet_im_online::sr25519::{AuthorityId as ImOnlineId};

pub use node_primitives::{AccountId, Balance, Signature, Block};

use moonbeam_runtime::{
	GenesisConfig, AuthorityDiscoveryConfig, BabeConfig, BalancesConfig, ContractsConfig,
	GrandpaConfig, ImOnlineConfig, SessionConfig, SessionKeys, StakerStatus, StakingConfig,
	IndicesConfig, SudoConfig, SystemConfig, WASM_BINARY, MoonbeamCoreConfig
};

use moonbeam_runtime::constants::mb_genesis::*;
use moonbeam_runtime::constants::currency::*;

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client::BadBlocks<Block>,
}

pub type ChainSpec = sc_service::ChainSpec<
	GenesisConfig,
	Extensions,
>;

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(seed: &str) -> (
	AccountId,
	AccountId,
	GrandpaId,
	BabeId,
	ImOnlineId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

fn testnet_genesis(
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)>,
	endowed_accounts: Vec<AccountId>,
	root_key: AccountId,
	enable_println: bool
) -> GenesisConfig {

	const ENDOWMENT: Balance = 10_000_000 * GLMR;
	const STASH: Balance = 100 * GLMR;

	let keys = initial_authorities.iter().map(|x| {
		(x.0.clone(), session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()))
	}).collect::<Vec<_>>();

	GenesisConfig {
		frame_system: Some(SystemConfig {
			code: WASM_BINARY.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			balances: endowed_accounts.iter().cloned()
				.map(|k| (k, ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
				.collect(),
		}),
		pallet_session: Some(SessionConfig {
			keys: keys,
		}),
		// https://crates.parity.io/pallet_staking/struct.GenesisConfig.html
		pallet_staking: Some(StakingConfig {
			current_era: 0,
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)
			}).collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			.. Default::default()
		}),
		pallet_indices: Some(IndicesConfig {
			indices: vec![],
		}),
		pallet_sudo: Some(SudoConfig {
			key: root_key,
		}),
		pallet_babe: Some(BabeConfig {
			authorities: vec![],
		}),
		pallet_im_online: Some(ImOnlineConfig {
			keys: vec![],
		}),
		pallet_authority_discovery: Some(AuthorityDiscoveryConfig {
			keys: vec![],
		}),
		pallet_grandpa: Some(GrandpaConfig {
			authorities: vec![],
		}),
		pallet_contracts: Some(ContractsConfig {
			current_schedule: pallet_contracts::Schedule {
				enable_println, // this should only be enabled on development chains
				..Default::default()
			},
			gas_price: 1 * MILLIGLMR,
		}),
		pallet_vesting: Some(Default::default()),
		mb_core: Some(MoonbeamCoreConfig {
			treasury: TREASURY_ENDOWMENT,
			genesis_accounts: endowed_accounts,
		}),
	}
}

fn development_config_genesis() -> GenesisConfig {

	let seeds = vec![
		"Armstrong",
		"Aldrin",
		"Armstrong//stash",
		"Aldrin//stash"
	];
	let mut accounts: Vec<AccountId> = vec![];
	let mut initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)> = vec![];

	for s in seeds {
		accounts.push( get_account_id_from_seed::<sr25519::Public>(&s) );
        if !s.contains("//stash") {
            initial_authorities.push( get_authority_keys_from_seed(&s) );
        }
	}

	testnet_genesis(
		initial_authorities,
		accounts,
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		true,
	)
}

/// Development config
/// https://crates.parity.io/sc_chain_spec/struct.ChainSpec.html#method.from_genesis
pub fn development_config() -> ChainSpec {

	let properties = json!({
        "tokenSymbol": "GLMR",
		"tokenDecimals": 8
    });

	ChainSpec::from_genesis(
		"Development",
		"dev",
		development_config_genesis,
		vec![],
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])),
		None,
		Some(properties.as_object().unwrap().clone()),
		Default::default(),
	)
}
/// TODO multiple configs using _id
pub fn load_spec(_id: &str) -> Result<Option<ChainSpec>, String> {
	Ok(Some(development_config()))
}