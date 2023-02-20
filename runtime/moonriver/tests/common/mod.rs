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

#![allow(dead_code)]

use cumulus_primitives_parachain_inherent::ParachainInherentData;
use fp_evm::GenesisAccount;
use frame_support::{
	assert_ok,
	dispatch::Dispatchable,
	traits::{GenesisBuild, OnFinalize, OnInitialize},
};
pub use moonriver_runtime::{
	asset_config::AssetRegistrarMetadata,
	currency::{GIGAWEI, MOVR, SUPPLY_FACTOR, WEI},
	xcm_config::AssetType,
	AccountId, AssetId, AssetManager, Assets, AuthorInherent, Balance, Balances, CrowdloanRewards,
	Ethereum, Executive, InflationInfo, LocalAssets, ParachainStaking, Range, Runtime, RuntimeCall,
	RuntimeEvent, System, TransactionConverter, TransactionPaymentAsGasPrice, UncheckedExtrinsic,
	HOURS, WEEKS,
};
use nimbus_primitives::{NimbusId, NIMBUS_ENGINE_ID};
use sp_core::{Encode, H160};
use sp_runtime::{Digest, DigestItem, Perbill, Percent};

use std::collections::BTreeMap;

use fp_rpc::ConvertTransaction;
use pallet_transaction_payment::Multiplier;

// A valid signed Alice transfer.
pub const VALID_ETH_TX: &str =
	"02f86d8205018085174876e80085e8d4a5100082520894f24ff3a9cf04c71dbc94d0b566f7a27b9456\
	6cac8080c001a0e1094e1a52520a75c0255db96132076dd0f1263089f838bea548cbdbfc64a4d19f031c\
	92a8cb04e2d68d20a6158d542a07ac440cc8d07b6e36af02db046d92df";

// An invalid signed Alice transfer with a gas limit artifically set to 0.
pub const INVALID_ETH_TX: &str =
	"f86180843b9aca00809412cb274aad8251c875c0bf6872b67d9983e53fdd01801ca00e28ba2dd3c5a\
	3fd467d4afd7aefb4a34b373314fff470bb9db743a84d674a0aa06e5994f2d07eafe1c37b4ce5471ca\
	ecec29011f6f5bf0b1a552c55ea348df35f";

pub fn rpc_run_to_block(n: u32) {
	while System::block_number() < n {
		Ethereum::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		Ethereum::on_initialize(System::block_number());
	}
}

/// Utility function that advances the chain to the desired block number.
/// If an author is provided, that author information is injected to all the blocks in the meantime.
pub fn run_to_block(n: u32, author: Option<NimbusId>) {
	// Finalize the first block
	Ethereum::on_finalize(System::block_number());
	AuthorInherent::on_finalize(System::block_number());
	while System::block_number() < n {
		// Set the new block number and author
		match author {
			Some(ref author) => {
				let pre_digest = Digest {
					logs: vec![DigestItem::PreRuntime(NIMBUS_ENGINE_ID, author.encode())],
				};
				System::reset_events();
				System::initialize(
					&(System::block_number() + 1),
					&System::parent_hash(),
					&pre_digest,
				);
			}
			None => {
				System::set_block_number(System::block_number() + 1);
			}
		}

		// Initialize the new block
		AuthorInherent::on_initialize(System::block_number());
		ParachainStaking::on_initialize(System::block_number());
		Ethereum::on_initialize(System::block_number());

		// Finalize the block
		Ethereum::on_finalize(System::block_number());
		AuthorInherent::on_finalize(System::block_number());
		ParachainStaking::on_finalize(System::block_number());
	}
}

pub fn last_event() -> RuntimeEvent {
	System::events().pop().expect("Event expected").event
}

// Helper function to give a simple evm context suitable for tests.
// We can remove this once https://github.com/rust-blockchain/evm/pull/35
// is in our dependency graph.
pub fn evm_test_context() -> fp_evm::Context {
	fp_evm::Context {
		address: Default::default(),
		caller: Default::default(),
		apparent_value: From::from(0),
	}
}

// Test struct with the purpose of initializing xcm assets
#[derive(Clone)]
pub struct XcmAssetInitialization {
	pub asset_type: AssetType,
	pub metadata: AssetRegistrarMetadata,
	pub balances: Vec<(AccountId, Balance)>,
	pub is_sufficient: bool,
}

pub struct ExtBuilder {
	// [asset, Vec<Account, Balance>, owner]
	local_assets: Vec<(AssetId, Vec<(AccountId, Balance)>, AccountId)>,
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
	// [collator, amount]
	collators: Vec<(AccountId, Balance)>,
	// [delegator, collator, nomination_amount]
	delegations: Vec<(AccountId, AccountId, Balance, Percent)>,
	// per-round inflation config
	inflation: InflationInfo<Balance>,
	// AuthorId -> AccoutId mappings
	mappings: Vec<(NimbusId, AccountId)>,
	// Crowdloan fund
	crowdloan_fund: Balance,
	// Chain id
	chain_id: u64,
	// EVM genesis accounts
	evm_accounts: BTreeMap<H160, GenesisAccount>,
	// [assettype, metadata, Vec<Account, Balance,>, is_sufficient]
	xcm_assets: Vec<XcmAssetInitialization>,
	safe_xcm_version: Option<u32>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder {
			local_assets: vec![],
			balances: vec![],
			delegations: vec![],
			collators: vec![],
			inflation: InflationInfo {
				expect: Range {
					min: 100_000 * MOVR,
					ideal: 200_000 * MOVR,
					max: 500_000 * MOVR,
				},
				// not used
				annual: Range {
					min: Perbill::from_percent(50),
					ideal: Perbill::from_percent(50),
					max: Perbill::from_percent(50),
				},
				// unrealistically high parameterization, only for testing
				round: Range {
					min: Perbill::from_percent(5),
					ideal: Perbill::from_percent(5),
					max: Perbill::from_percent(5),
				},
			},
			mappings: vec![],
			crowdloan_fund: 0,
			chain_id: CHAIN_ID,
			evm_accounts: BTreeMap::new(),
			xcm_assets: vec![],
			safe_xcm_version: None,
		}
	}
}

impl ExtBuilder {
	pub fn with_evm_accounts(mut self, accounts: BTreeMap<H160, GenesisAccount>) -> Self {
		self.evm_accounts = accounts;
		self
	}

	pub fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn with_collators(mut self, collators: Vec<(AccountId, Balance)>) -> Self {
		self.collators = collators;
		self
	}

	pub fn with_delegations(mut self, delegations: Vec<(AccountId, AccountId, Balance)>) -> Self {
		self.delegations = delegations
			.into_iter()
			.map(|d| (d.0, d.1, d.2, Percent::zero()))
			.collect();
		self
	}

	pub fn with_crowdloan_fund(mut self, crowdloan_fund: Balance) -> Self {
		self.crowdloan_fund = crowdloan_fund;
		self
	}

	pub fn with_mappings(mut self, mappings: Vec<(NimbusId, AccountId)>) -> Self {
		self.mappings = mappings;
		self
	}

	#[allow(dead_code)]
	pub fn with_inflation(mut self, inflation: InflationInfo<Balance>) -> Self {
		self.inflation = inflation;
		self
	}

	pub fn with_local_assets(
		mut self,
		local_assets: Vec<(AssetId, Vec<(AccountId, Balance)>, AccountId)>,
	) -> Self {
		self.local_assets = local_assets;
		self
	}

	pub fn with_xcm_assets(mut self, xcm_assets: Vec<XcmAssetInitialization>) -> Self {
		self.xcm_assets = xcm_assets;
		self
	}

	pub fn with_safe_xcm_version(mut self, safe_xcm_version: u32) -> Self {
		self.safe_xcm_version = Some(safe_xcm_version);
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		pallet_parachain_staking::GenesisConfig::<Runtime> {
			candidates: self.collators,
			delegations: self.delegations,
			inflation_config: self.inflation,
			collator_commission: Perbill::from_percent(20),
			parachain_bond_reserve_percent: Percent::from_percent(30),
			blocks_per_round: 2 * HOURS,
			num_selected_candidates: 8,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		pallet_crowdloan_rewards::GenesisConfig::<Runtime> {
			funded_amount: self.crowdloan_fund,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		pallet_author_mapping::GenesisConfig::<Runtime> {
			mappings: self.mappings,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		<pallet_ethereum_chain_id::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
			&pallet_ethereum_chain_id::GenesisConfig {
				chain_id: self.chain_id,
			},
			&mut t,
		)
		.unwrap();

		<pallet_evm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
			&pallet_evm::GenesisConfig {
				accounts: self.evm_accounts,
			},
			&mut t,
		)
		.unwrap();

		<pallet_ethereum::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
			&pallet_ethereum::GenesisConfig {},
			&mut t,
		)
		.unwrap();

		<pallet_xcm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
			&pallet_xcm::GenesisConfig {
				safe_xcm_version: self.safe_xcm_version,
			},
			&mut t,
		)
		.unwrap();

		<pallet_transaction_payment::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
			&pallet_transaction_payment::GenesisConfig {
				multiplier: Multiplier::from(10u128),
			},
			&mut t,
		)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		let local_assets = self.local_assets.clone();
		let xcm_assets = self.xcm_assets.clone();
		ext.execute_with(|| {
			// If any local assets specified, we create them here
			for (asset_id, balances, owner) in local_assets.clone() {
				LocalAssets::force_create(root_origin(), asset_id.into(), owner, true, 1).unwrap();
				for (account, balance) in balances {
					LocalAssets::mint(origin_of(owner.into()), asset_id.into(), account, balance)
						.unwrap();
				}
			}
			// If any xcm assets specified, we register them here
			for xcm_asset_initialization in xcm_assets {
				let asset_id: AssetId = xcm_asset_initialization.asset_type.clone().into();
				AssetManager::register_foreign_asset(
					root_origin(),
					xcm_asset_initialization.asset_type,
					xcm_asset_initialization.metadata,
					1,
					xcm_asset_initialization.is_sufficient,
				)
				.unwrap();
				for (account, balance) in xcm_asset_initialization.balances {
					Assets::mint(
						origin_of(AssetManager::account_id()),
						asset_id.into(),
						account,
						balance,
					)
					.unwrap();
				}
			}
			System::set_block_number(1);
		});
		ext
	}
}

pub const CHAIN_ID: u64 = 1281;
pub const ALICE: [u8; 20] = [4u8; 20];
pub const ALICE_NIMBUS: [u8; 32] = [4u8; 32];
pub const BOB: [u8; 20] = [5u8; 20];
pub const CHARLIE: [u8; 20] = [6u8; 20];
pub const DAVE: [u8; 20] = [7u8; 20];
pub const EVM_CONTRACT: [u8; 20] = [8u8; 20];

pub fn origin_of(account_id: AccountId) -> <Runtime as frame_system::Config>::RuntimeOrigin {
	<Runtime as frame_system::Config>::RuntimeOrigin::signed(account_id)
}

pub fn inherent_origin() -> <Runtime as frame_system::Config>::RuntimeOrigin {
	<Runtime as frame_system::Config>::RuntimeOrigin::none()
}

pub fn root_origin() -> <Runtime as frame_system::Config>::RuntimeOrigin {
	<Runtime as frame_system::Config>::RuntimeOrigin::root()
}

/// Mock the inherent that sets validation data in ParachainSystem, which
/// contains the `relay_chain_block_number`, which is used in `author-filter` as a
/// source of randomness to filter valid authors at each block.
pub fn set_parachain_inherent_data() {
	use cumulus_primitives_core::PersistedValidationData;
	use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;
	let (relay_parent_storage_root, relay_chain_state) =
		RelayStateSproofBuilder::default().into_state_root_and_proof();
	let vfp = PersistedValidationData {
		relay_parent_number: 1u32,
		relay_parent_storage_root,
		..Default::default()
	};
	let parachain_inherent_data = ParachainInherentData {
		validation_data: vfp,
		relay_chain_state: relay_chain_state,
		downward_messages: Default::default(),
		horizontal_messages: Default::default(),
	};
	assert_ok!(RuntimeCall::ParachainSystem(
		cumulus_pallet_parachain_system::Call::<Runtime>::set_validation_data {
			data: parachain_inherent_data
		}
	)
	.dispatch(inherent_origin()));
}

pub fn unchecked_eth_tx(raw_hex_tx: &str) -> UncheckedExtrinsic {
	let converter = TransactionConverter;
	converter.convert_transaction(ethereum_transaction(raw_hex_tx))
}

pub fn ethereum_transaction(raw_hex_tx: &str) -> pallet_ethereum::Transaction {
	let bytes = hex::decode(raw_hex_tx).expect("Transaction bytes.");
	let transaction = ethereum::EnvelopedDecodable::decode(&bytes[..]);
	assert!(transaction.is_ok());
	transaction.unwrap()
}
