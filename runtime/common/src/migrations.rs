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

#[cfg(feature = "try-runtime")]
use frame_support::ensure;
use frame_support::{
	pallet_prelude::GetStorageVersion,
	traits::{OnRuntimeUpgrade, PalletInfoAccess, StorageVersion},
	weights::Weight,
};
use pallet_author_slot_filter::Config as AuthorSlotFilterConfig;
use pallet_migrations::{GetMigrations, Migration};
use sp_core::Get;
#[cfg(feature = "try-runtime")]
use sp_runtime::traits::Zero;
use sp_std::{marker::PhantomData, prelude::*};

pub struct PalletReferendaMigrateV0ToV1<T>(pub PhantomData<T>);
impl<T> Migration for PalletReferendaMigrateV0ToV1<T>
where
	T: pallet_referenda::Config + frame_system::Config,
{
	fn friendly_name(&self) -> &str {
		"MM_PalletReferendaMigrateV0ToV1"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		pallet_referenda::migration::v1::MigrateV0ToV1::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, sp_runtime::DispatchError> {
		pallet_referenda::migration::v1::MigrateV0ToV1::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		pallet_referenda::migration::v1::MigrateV0ToV1::<T>::post_upgrade(state)
	}
}

pub struct MissingBalancesMigrations<T>(PhantomData<T>);
impl<T> Migration for MissingBalancesMigrations<T>
where
	T: pallet_balances::Config,
	<T as frame_system::Config>::AccountId: Default,
{
	fn friendly_name(&self) -> &str {
		"MM_MissingBalancesMigrations"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		pallet_balances::migration::MigrateToTrackInactive::<T, ()>::on_runtime_upgrade();
		pallet_balances::migration::ResetInactive::<T, ()>::on_runtime_upgrade();
		pallet_balances::migration::MigrateToTrackInactive::<T, ()>::on_runtime_upgrade()
	}
}

pub struct FixIncorrectPalletVersions<Runtime, Treasury, OpenTech>(
	pub PhantomData<(Runtime, Treasury, OpenTech)>,
);
impl<Runtime, Treasury, OpenTech> Migration
	for FixIncorrectPalletVersions<Runtime, Treasury, OpenTech>
where
	Treasury: GetStorageVersion + PalletInfoAccess,
	OpenTech: GetStorageVersion + PalletInfoAccess,
	Runtime: frame_system::Config,
	Runtime: pallet_referenda::Config,
{
	fn friendly_name(&self) -> &str {
		"MM_FixIncorrectPalletVersions"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		log::info!("Setting collectives pallet versions to 4");
		StorageVersion::new(4).put::<Treasury>();
		StorageVersion::new(4).put::<OpenTech>();
		Runtime::DbWeight::get().writes(2)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, sp_runtime::DispatchError> {
		ensure!(
			<Treasury as GetStorageVersion>::on_chain_storage_version() == 0,
			"TreasuryCouncilCollective storage version should be 0"
		);
		ensure!(
			<OpenTech as GetStorageVersion>::on_chain_storage_version() == 0,
			"OpenTechCommitteeCollective storage version should be 0"
		);

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, _state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		ensure!(
			<Treasury as GetStorageVersion>::on_chain_storage_version() == 4,
			"Treasury storage version should be 4"
		);
		ensure!(
			<OpenTech as GetStorageVersion>::on_chain_storage_version() == 4,
			"OpenTech storage version should be 4"
		);
		Ok(())
	}
}

pub struct CommonMigrations<Runtime, Council, Tech, Treasury, OpenTech>(
	PhantomData<(Runtime, Council, Tech, Treasury, OpenTech)>,
);

impl<Runtime, Council, Tech, Treasury, OpenTech> GetMigrations
	for CommonMigrations<Runtime, Council, Tech, Treasury, OpenTech>
where
	Runtime: pallet_author_mapping::Config,
	Runtime: pallet_parachain_staking::Config,
	Runtime: pallet_scheduler::Config,
	Runtime: AuthorSlotFilterConfig,
	Council: GetStorageVersion + PalletInfoAccess + 'static,
	Tech: GetStorageVersion + PalletInfoAccess + 'static,
	Treasury: GetStorageVersion + PalletInfoAccess + 'static,
	OpenTech: GetStorageVersion + PalletInfoAccess + 'static,
	Runtime: pallet_democracy::Config,
	Runtime: pallet_preimage::Config,
	Runtime: pallet_asset_manager::Config,
	<Runtime as pallet_asset_manager::Config>::ForeignAssetType: From<xcm::v3::MultiLocation>,
	Runtime: pallet_xcm_transactor::Config,
	Runtime: pallet_moonbeam_orbiters::Config,
	Runtime: pallet_balances::Config,
	Runtime: pallet_referenda::Config,
	Runtime::AccountId: Default,
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

		// RT2700
		let missing_balances_migrations = MissingBalancesMigrations::<Runtime>(Default::default());
		let fix_pallet_versions =
			FixIncorrectPalletVersions::<Runtime, Treasury, OpenTech>(Default::default());
		let pallet_referenda_migrate_v0_to_v1 =
			PalletReferendaMigrateV0ToV1::<Runtime>(Default::default());

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
			//Box::new(asset_manager_to_xcm_v3),
			//Box::new(xcm_transactor_to_xcm_v3),
			// completed in runtime 2600
			//Box::new(remove_min_bond_for_old_orbiter_collators),
			Box::new(missing_balances_migrations),
			Box::new(fix_pallet_versions),
			Box::new(pallet_referenda_migrate_v0_to_v1),
		]
	}
}
