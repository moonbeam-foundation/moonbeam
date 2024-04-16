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

//! # Migrations
//!
//! This module acts as a registry where each migration is defined. Each migration should implement
//! the "Migration" trait declared in the pallet-migrations crate.

use frame_support::{
	parameter_types,
	storage::unhashed::{clear_prefix, contains_prefixed_key},
	traits::OnRuntimeUpgrade,
	weights::Weight,
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_author_slot_filter::Config as AuthorSlotFilterConfig;
use pallet_migrations::{GetMigrations, Migration};
use pallet_parachain_staking::migrations::MigrateRoundWithFirstSlot;
use sp_core::Get;
use sp_std::{marker::PhantomData, prelude::*, vec};

pub struct PalletStakingRoundMigration<Runtime>(PhantomData<Runtime>);
impl<Runtime> Migration for PalletStakingRoundMigration<Runtime>
where
	Runtime: pallet_parachain_staking::Config,
	BlockNumberFor<Runtime>: Into<u64>,
{
	fn friendly_name(&self) -> &str {
		"MM_MigrateRoundWithFirstSlot"
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, sp_runtime::DispatchError> {
		MigrateRoundWithFirstSlot::<Runtime>::pre_upgrade()
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		MigrateRoundWithFirstSlot::<Runtime>::on_runtime_upgrade()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		MigrateRoundWithFirstSlot::<Runtime>::post_upgrade(state)
	}
}

/// Moonbeam has 134 keys to be migrated
/// Moonriver has 117 keys to be migrated
/// Moonbase has 172 keys to be migrated
pub const PALLET_IDENTITY_MIGRATION_KEY_LIMIT: u64 = 500;

pub struct PalletIdentityMigrateV0ToV1<Runtime, const KL: u64>(PhantomData<Runtime>);
impl<Runtime, const KL: u64> Migration for PalletIdentityMigrateV0ToV1<Runtime, KL>
where
	Runtime: pallet_identity::Config,
{
	fn friendly_name(&self) -> &str {
		"MM_PalletIdentityMigrateV0ToV1"
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, sp_runtime::DispatchError> {
		pallet_identity::migration::versioned::V0ToV1::<Runtime, KL>::pre_upgrade()
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		pallet_identity::migration::versioned::V0ToV1::<Runtime, KL>::on_runtime_upgrade()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		pallet_identity::migration::versioned::V0ToV1::<Runtime, KL>::post_upgrade(state)
	}
}

parameter_types! {
	pub const DemocracyPalletName: &'static str = "Democracy";
}

pub struct RemovePalletDemocracy<Runtime>(pub PhantomData<Runtime>);
impl<Runtime> Migration for RemovePalletDemocracy<Runtime>
where
	Runtime: frame_system::Config,
{
	fn friendly_name(&self) -> &str {
		"MM_RemoveDemocracyPallet"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		log::info!("Removing pallet democracy");

		// Democracy: f2794c22e353e9a839f12faab03a911b
		// VotingOf: e470c6afbbbc027eb288ade7595953c2
		let prefix =
			hex_literal::hex!("f2794c22e353e9a839f12faab03a911be470c6afbbbc027eb288ade7595953c2");
		if contains_prefixed_key(&prefix) {
			// PoV failsafe: do not execute the migration if there are VotingOf keys
			// that have not been cleaned up
			log::info!("Found keys for Democracy.VotingOf pre-removal - skipping migration",);
			return Weight::zero();
		};
		frame_support::migrations::RemovePallet::<
			DemocracyPalletName,
			<Runtime as frame_system::Config>::DbWeight,
		>::on_runtime_upgrade()
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, sp_runtime::DispatchError> {
		let _ = frame_support::migrations::RemovePallet::<
			DemocracyPalletName,
			<Runtime as frame_system::Config>::DbWeight,
		>::pre_upgrade();

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, _state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		let _ = frame_support::migrations::RemovePallet::<
			DemocracyPalletName,
			<Runtime as frame_system::Config>::DbWeight,
		>::post_upgrade(_state);

		Ok(())
	}
}

pub struct RemoveCollectivesAddresses<Runtime>(pub PhantomData<Runtime>);
impl<Runtime> Migration for RemoveCollectivesAddresses<Runtime>
where
	Runtime: frame_system::Config,
{
	fn friendly_name(&self) -> &str {
		"MM_RemoveCollectivesAddresses"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		log::info!("Removing gov v1 collective addresses storage");

		// CouncilCollective: d59b9be6f0a7187ca6630c1d0a9bb045
		let prefix = hex_literal::hex!("d59b9be6f0a7187ca6630c1d0a9bb045");
		let result = clear_prefix(&prefix, Some(10), None);
		log::info!("Removed {} CouncilCollective keys", result.unique);

		// record how many records have been deleted
		let mut writes = result.unique;

		// TechCommitteeCollective: a06bfb73a86f8f98d5c5dc14e20e8a03
		let prefix = hex_literal::hex!("a06bfb73a86f8f98d5c5dc14e20e8a03");
		let result = clear_prefix(&prefix, Some(10), None);
		log::info!("Removed {} TechCommitteeCollective", result.unique);

		// Account for weight
		writes = writes.saturating_add(result.unique);
		let reads = writes.saturating_add(2);

		let w = Runtime::DbWeight::get();
		w.reads_writes(reads.into(), writes.into())
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, sp_runtime::DispatchError> {
		// CouncilCollective: d59b9be6f0a7187ca6630c1d0a9bb045
		let prefix = hex_literal::hex!("d59b9be6f0a7187ca6630c1d0a9bb045");
		match contains_prefixed_key(&prefix) {
			true => log::info!("Found keys for CouncilCollective (pre-removal"),
			false => log::warn!("No keys found for CouncilCollective (pre-removal)"),
		};

		// TechCommitteeCollective: a06bfb73a86f8f98d5c5dc14e20e8a03
		let prefix = hex_literal::hex!("a06bfb73a86f8f98d5c5dc14e20e8a03");
		match contains_prefixed_key(&prefix) {
			true => log::info!("Found keys for TechCommitteeCollective (pre-removal)"),
			false => log::warn!("No keys found for TechCommitteeCollective (pre-removal)"),
		};

		Ok(sp_std::vec::Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, _state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		// CouncilCollective: d59b9be6f0a7187ca6630c1d0a9bb045
		let prefix = hex_literal::hex!("d59b9be6f0a7187ca6630c1d0a9bb045");
		match contains_prefixed_key(&prefix) {
			true => log::info!("Found keys for CouncilCollective (post-removal) ⚠️"),
			false => log::warn!("No keys found for CouncilCollective (post-removal) ✅"),
		};

		// TechCommitteeCollective: a06bfb73a86f8f98d5c5dc14e20e8a03
		let prefix = hex_literal::hex!("a06bfb73a86f8f98d5c5dc14e20e8a03");
		match contains_prefixed_key(&prefix) {
			true => log::info!("Found keys for TechCommitteeCollective (post-removal) ⚠️"),
			false => log::warn!("No keys found for TechCommitteeCollective (post-removal) ✅"),
		};
		Ok(())
	}
}

pub struct MigrateToLatestXcmVersion<Runtime>(PhantomData<Runtime>);
impl<Runtime> Migration for MigrateToLatestXcmVersion<Runtime>
where
	pallet_xcm::migration::MigrateToLatestXcmVersion<Runtime>: OnRuntimeUpgrade,
{
	fn friendly_name(&self) -> &str {
		"MM_MigrateToLatestXcmVersion"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		pallet_xcm::migration::MigrateToLatestXcmVersion::<Runtime>::on_runtime_upgrade()
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, sp_runtime::DispatchError> {
		pallet_xcm::migration::MigrateToLatestXcmVersion::<Runtime>::pre_upgrade()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		pallet_xcm::migration::MigrateToLatestXcmVersion::<Runtime>::post_upgrade(state)
	}
}

pub struct CommonMigrations<Runtime>(PhantomData<Runtime>);

impl<Runtime> GetMigrations for CommonMigrations<Runtime>
where
	Runtime: pallet_author_mapping::Config,
	Runtime: pallet_parachain_staking::Config,
	Runtime: pallet_scheduler::Config,
	Runtime: AuthorSlotFilterConfig,
	Runtime: pallet_preimage::Config,
	Runtime: pallet_asset_manager::Config,
	Runtime: pallet_xcm_transactor::Config,
	Runtime: pallet_moonbeam_orbiters::Config,
	Runtime: pallet_balances::Config,
	Runtime: pallet_referenda::Config,
	Runtime: pallet_xcm::Config,
	Runtime::AccountId: Default,
	BlockNumberFor<Runtime>: Into<u64>,
	Runtime: pallet_identity::Config,
{
	fn get_migrations() -> Vec<Box<dyn Migration>> {
		// let migration_author_mapping_twox_to_blake = AuthorMappingTwoXToBlake::<Runtime> {
		// 	0: Default::default(),
		// };

		// let migration_parachain_staking_purge_stale_storage =
		// 	ParachainStakingPurgeStaleStorage::<Runtime>(Default::default());
		// let migration_parachain_staking_manual_exits =
		// 	ParachainStakingManualExits::<Runtime>(Default::default());
		// let migration_parachain_staking_increase_max_delegations_per_candidate =
		// 	ParachainStakingIncreaseMaxDelegationsPerCandidate::<Runtime>(Default::default());
		// let migration_parachain_staking_split_candidate_state =
		// 	ParachainStakingSplitCandidateState::<Runtime>(Default::default());
		// let migration_parachain_staking_patch_incorrect_delegation_sums =
		//	ParachainStakingPatchIncorrectDelegationSums::<Runtime>(Default::default());

		// let migration_scheduler_v3 = SchedulerMigrationV3::<Runtime>(Default::default());

		// let migration_base_fee = MigrateBaseFeePerGas::<Runtime>(Default::default());

		// TODO: this is a lot of allocation to do upon every get() call. this *should* be avoided
		// except when pallet_migrations undergoes a runtime upgrade -- but TODO: review

		// let migration_author_slot_filter_eligible_ratio_to_eligibility_count =
		// 	AuthorSlotFilterEligibleRatioToEligiblityCount::<Runtime>(Default::default());
		// let migration_author_mapping_add_keys_to_registration_info =
		// 	AuthorMappingAddKeysToRegistrationInfo::<Runtime>(Default::default());
		// let staking_delegator_state_requests =
		// 	ParachainStakingSplitDelegatorStateIntoDelegationScheduledRequests::<Runtime>(
		// 		Default::default(),
		// 	);
		// let migration_author_mapping_add_account_id_to_nimbus_lookup =
		//	AuthorMappingAddAccountIdToNimbusLookup::<Runtime>(Default::default());

		// let xcm_transactor_max_weight =
		// 	XcmTransactorMaxTransactWeight::<Runtime>(Default::default());

		// let asset_manager_units_with_asset_type =
		// 	AssetManagerUnitsWithAssetType::<Runtime>(Default::default());

		// let asset_manager_populate_asset_type_id_storage =
		// 	AssetManagerPopulateAssetTypeIdStorage::<Runtime>(Default::default());

		// let asset_manager_change_statemine_prefixes = AssetManagerChangeStateminePrefixes::<
		// 	Runtime,
		// 	StatemineParaIdInfo,
		// 	StatemineAssetsInstanceInfo,
		// >(Default::default());

		// let xcm_supported_assets = XcmPaymentSupportedAssets::<Runtime>(Default::default());

		// let migration_elasticity = MigrateBaseFeeElasticity::<Runtime>(Default::default());
		//let staking_at_stake_auto_compound =
		//	ParachainStakingMigrateAtStakeAutoCompound::<Runtime>(Default::default());

		//let scheduler_to_v4 = SchedulerMigrationV4::<Runtime>(Default::default());
		//let democracy_migration_hash_to_bounded_call =
		//	DemocracryMigrationHashToBoundedCall::<Runtime>(Default::default());
		//let preimage_migration_hash_to_bounded_call =
		//	PreimageMigrationHashToBoundedCall::<Runtime>(Default::default());
		//let asset_manager_to_xcm_v3 =
		//	PalletAssetManagerMigrateXcmV2ToV3::<Runtime>(Default::default());
		//let xcm_transactor_to_xcm_v3 =
		//	PalletXcmTransactorMigrateXcmV2ToV3::<Runtime>(Default::default());
		//let remove_min_bond_for_old_orbiter_collators =
		//	RemoveMinBondForOrbiterCollators::<Runtime>(Default::default());
		// let missing_balances_migrations = MissingBalancesMigrations::<Runtime>(Default::default());
		// let fix_pallet_versions =
		// 	FixIncorrectPalletVersions::<Runtime, Treasury, OpenTech>(Default::default());
		// let pallet_referenda_migrate_v0_to_v1 =
		// 	PalletReferendaMigrateV0ToV1::<Runtime>(Default::default());
		//let pallet_collective_drop_gov_v1_collectives =
		//	PalletCollectiveDropGovV1Collectives::<Runtime>(Default::default());
		//let pallet_staking_round = PalletStakingRoundMigration::<Runtime>(Default::default());
		let remove_pallet_democracy = RemovePalletDemocracy::<Runtime>(Default::default());
		let remove_collectives_addresses =
			RemoveCollectivesAddresses::<Runtime>(Default::default());
		// This migration can be removed in RT3000
		let migrate_pallet_identity = PalletIdentityMigrateV0ToV1::<
			Runtime,
			{ PALLET_IDENTITY_MIGRATION_KEY_LIMIT },
		>(Default::default());

		vec![
			// completed in runtime 800
			// Box::new(migration_author_mapping_twox_to_blake),
			// completed in runtime 900
			// completed in runtime 1000
			// Box::new(migration_parachain_staking_purge_stale_storage),
			// completed in runtime 1000
			// Box::new(migration_parachain_staking_manual_exits),
			// completed in runtime 1101
			// Box::new(migration_parachain_staking_increase_max_delegations_per_candidate),
			// completed in runtime 1201
			// Box::new(migration_parachain_staking_split_candidate_state),
			// completed in runtime 1201
			// Box::new(xcm_transactor_max_weight),
			// completed in runtime 1201
			// Box::new(asset_manager_units_with_asset_type),
			// completed in runtime 1201
			// Box::new(asset_manager_change_statemine_prefixes),
			// completed in runtime 1201
			// Box::new(asset_manager_populate_asset_type_id_storage),
			// completed in runtime 1300
			// Box::new(migration_scheduler_v3),
			// completed in runtime 1300
			// Box::new(migration_parachain_staking_patch_incorrect_delegation_sums),
			// completed in runtime 1300
			// Box::new(migration_base_fee),
			// completed in runtime 1300
			// Box::new(xcm_supported_assets),
			// completed in runtime 1500
			// Box::new(migration_author_slot_filter_eligible_ratio_to_eligibility_count),
			// Box::new(migration_author_mapping_add_keys_to_registration_info),
			// Box::new(staking_delegator_state_requests),
			// completed in runtime 1600
			// Box::new(migration_author_mapping_add_account_id_to_nimbus_lookup),
			// completed in runtime 1600
			// Box::new(xcm_transactor_transact_signed),
			// completed in runtime 1700
			//Box::new(migration_elasticity),
			// completed in runtime 1900
			//Box::new(staking_at_stake_auto_compound),
			// completed in runtime 2000
			//Box::new(scheduler_to_v4),
			//Box::new(democracy_migration_hash_to_bounded_call),
			//Box::new(preimage_migration_hash_to_bounded_call),
			// completed in runtime 2100
			//Box::new(asset_manager_to_xcm_v3),
			//Box::new(xcm_transactor_to_xcm_v3),
			// completed in runtime 2600
			//Box::new(remove_min_bond_for_old_orbiter_collators),
			// completed in runtime 2700
			// Box::new(missing_balances_migrations),
			// Box::new(fix_pallet_versions),
			// Box::new(pallet_referenda_migrate_v0_to_v1),
			// completed in runtime 2800
			//Box::new(pallet_collective_drop_gov_v1_collectives),
			// completed in runtime 2801
			// Box::new(pallet_staking_round),
			// Box::new(pallet_collective_drop_gov_v1_collectives),
			// completed in runtime 2900
			Box::new(remove_pallet_democracy),
			Box::new(remove_collectives_addresses),
			Box::new(migrate_pallet_identity),
			// permanent migrations
			Box::new(MigrateToLatestXcmVersion::<Runtime>(Default::default())),
		]
	}
}
