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

use frame_support::{
	dispatch::GetStorageVersion,
	traits::{Get, Hash as PreimageHash, OnRuntimeUpgrade, PalletInfoAccess},
	weights::Weight,
};
use pallet_asset_manager::{
	migrations::{
		ChangeStateminePrefixes, PopulateAssetTypeIdStorage, PopulateSupportedFeePaymentAssets,
		UnitsWithAssetType,
	},
	Config as AssetManagerConfig,
};
use pallet_author_mapping::{
	migrations::{AddAccountIdToNimbusLookup, AddKeysToRegistrationInfo},
	Config as AuthorMappingConfig,
};
use pallet_author_slot_filter::migration::EligibleRatioToEligiblityCount;
use pallet_author_slot_filter::Config as AuthorSlotFilterConfig;
use pallet_migrations::{GetMigrations, Migration};
use pallet_parachain_staking::{
	migrations::{
		MigrateAtStakeAutoCompound, PatchIncorrectDelegationSums, PurgeStaleStorage,
		SplitDelegatorStateIntoDelegationScheduledRequests,
	},
	Config as ParachainStakingConfig,
};
use pallet_xcm_transactor::{
	migrations::TransactSignedWeightAndFeePerSecond, Config as XcmTransactorConfig,
};
use sp_std::{marker::PhantomData, prelude::*};
use xcm::latest::MultiLocation;

/// This module acts as a registry where each migration is defined. Each migration should implement
/// the "Migration" trait declared in the pallet-migrations crate.

/// A moonbeam migration wrapping the similarly named migration in pallet-author-mapping
pub struct AuthorMappingAddAccountIdToNimbusLookup<T>(PhantomData<T>);
impl<T: AuthorMappingConfig> Migration for AuthorMappingAddAccountIdToNimbusLookup<T> {
	fn friendly_name(&self) -> &str {
		"MM_Author_Mapping_AddAccountIdToNimbusLookup"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		AddAccountIdToNimbusLookup::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		AddAccountIdToNimbusLookup::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		AddAccountIdToNimbusLookup::<T>::post_upgrade(state)
	}
}

/// A moonbeam migration wrapping the similarly named migration in pallet-author-mapping
pub struct AuthorMappingAddKeysToRegistrationInfo<T>(PhantomData<T>);
impl<T: AuthorMappingConfig> Migration for AuthorMappingAddKeysToRegistrationInfo<T> {
	fn friendly_name(&self) -> &str {
		"MM_Author_Mapping_AddKeysToRegistrationInfo"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		AddKeysToRegistrationInfo::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		AddKeysToRegistrationInfo::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		AddKeysToRegistrationInfo::<T>::post_upgrade(state)
	}
}

/// Patch delegations total mismatch
pub struct ParachainStakingPatchIncorrectDelegationSums<T>(PhantomData<T>);
impl<T: ParachainStakingConfig> Migration for ParachainStakingPatchIncorrectDelegationSums<T> {
	fn friendly_name(&self) -> &str {
		"MM_Parachain_Staking_Patch_Incorrect_Delegation_Sums"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		PatchIncorrectDelegationSums::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		PatchIncorrectDelegationSums::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		PatchIncorrectDelegationSums::<T>::post_upgrade(state)
	}
}

/// Migrate `AtStake` storage item to contain 0% auto-compound values
pub struct ParachainStakingMigrateAtStakeAutoCompound<T>(PhantomData<T>);
impl<T: ParachainStakingConfig> Migration for ParachainStakingMigrateAtStakeAutoCompound<T> {
	fn friendly_name(&self) -> &str {
		"MM_Parachain_Staking_Migrate_At_Stake_AutoCompound"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		MigrateAtStakeAutoCompound::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		MigrateAtStakeAutoCompound::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		MigrateAtStakeAutoCompound::<T>::post_upgrade(state)
	}
}

/// Staking split delegator state into [pallet_parachain_staking::DelegatorScheduledRequests]
pub struct ParachainStakingSplitDelegatorStateIntoDelegationScheduledRequests<T>(PhantomData<T>);
impl<T: ParachainStakingConfig> Migration
	for ParachainStakingSplitDelegatorStateIntoDelegationScheduledRequests<T>
{
	fn friendly_name(&self) -> &str {
		"MM_Parachain_Staking_Split_Delegator_State_Into_Delegation_Scheduled_Requests"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		SplitDelegatorStateIntoDelegationScheduledRequests::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		SplitDelegatorStateIntoDelegationScheduledRequests::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		SplitDelegatorStateIntoDelegationScheduledRequests::<T>::post_upgrade(state)
	}
}

// /// Staking transition from automatic to manual exits, delay bond_{more, less} requests
// pub struct ParachainStakingManualExits<T>(PhantomData<T>);
// impl<T: ParachainStakingConfig> Migration for ParachainStakingManualExits<T> {
// 	fn friendly_name(&self) -> &str {
// 		"MM_Parachain_Staking_ManualExits"
// 	}

// 	fn migrate(&self, _available_weight: Weight) -> Weight {
// 		RemoveExitQueue::<T>::on_runtime_upgrade()
// 	}

// 	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
// 	#[cfg(feature = "try-runtime")]
// 	fn pre_upgrade(&self) -> Result<(), &'static str> {
// 		RemoveExitQueue::<T>::pre_upgrade()
// 	}

// 	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
// 	#[cfg(feature = "try-runtime")]
// 	fn post_upgrade(&self) -> Result<(), &'static str> {
// 		RemoveExitQueue::<T>::post_upgrade()
// 	}
// }

/// A moonbeam migration wrapping the similarly named migration in parachain-staking
pub struct ParachainStakingPurgeStaleStorage<T>(PhantomData<T>);
impl<T: ParachainStakingConfig> Migration for ParachainStakingPurgeStaleStorage<T> {
	fn friendly_name(&self) -> &str {
		"MM_Parachain_Staking_PurgeStaleStorage"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		PurgeStaleStorage::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		PurgeStaleStorage::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		PurgeStaleStorage::<T>::post_upgrade(state)
	}
}

pub struct XcmTransactorTransactSignedWeightAndFeePerSecond<T>(PhantomData<T>);
impl<T: XcmTransactorConfig> Migration for XcmTransactorTransactSignedWeightAndFeePerSecond<T> {
	fn friendly_name(&self) -> &str {
		"MM_Xcm_Transactor_TransactSignedWeightAndFeePerSecond"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		TransactSignedWeightAndFeePerSecond::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		TransactSignedWeightAndFeePerSecond::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		TransactSignedWeightAndFeePerSecond::<T>::post_upgrade(state)
	}
}
pub struct AssetManagerUnitsWithAssetType<T>(PhantomData<T>);
impl<T: AssetManagerConfig> Migration for AssetManagerUnitsWithAssetType<T> {
	fn friendly_name(&self) -> &str {
		"MM_Asset_Manager_UnitsWithAssetType"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		UnitsWithAssetType::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		UnitsWithAssetType::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		UnitsWithAssetType::<T>::post_upgrade(state)
	}
}

pub struct AssetManagerPopulateAssetTypeIdStorage<T>(PhantomData<T>);
impl<T: AssetManagerConfig> Migration for AssetManagerPopulateAssetTypeIdStorage<T> {
	fn friendly_name(&self) -> &str {
		"MM_Asset_Manager_PopulateAssetTypeIdStorage"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		PopulateAssetTypeIdStorage::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		PopulateAssetTypeIdStorage::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		PopulateAssetTypeIdStorage::<T>::post_upgrade(state)
	}
}

pub struct AssetManagerChangeStateminePrefixes<T, StatemineParaIdInfo, StatemineAssetsPalletInfo>(
	PhantomData<(T, StatemineParaIdInfo, StatemineAssetsPalletInfo)>,
);
impl<T, StatemineParaIdInfo, StatemineAssetsPalletInfo> Migration
	for AssetManagerChangeStateminePrefixes<T, StatemineParaIdInfo, StatemineAssetsPalletInfo>
where
	T: AssetManagerConfig,
	StatemineParaIdInfo: Get<u32>,
	StatemineAssetsPalletInfo: Get<u8>,
	T::ForeignAssetType: Into<Option<MultiLocation>> + From<MultiLocation>,
{
	fn friendly_name(&self) -> &str {
		"MM_Asset_Manager_ChangeStateminePrefixes"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		ChangeStateminePrefixes::<
			T,
			StatemineParaIdInfo,
			StatemineAssetsPalletInfo
		>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		ChangeStateminePrefixes::<T, StatemineParaIdInfo, StatemineAssetsPalletInfo>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		ChangeStateminePrefixes::<T, StatemineParaIdInfo, StatemineAssetsPalletInfo>::post_upgrade(
			state,
		)
	}
}
pub struct XcmPaymentSupportedAssets<T>(PhantomData<T>);
impl<T: AssetManagerConfig> Migration for XcmPaymentSupportedAssets<T> {
	fn friendly_name(&self) -> &str {
		"MM_Xcm_Payment_Supported_Assets"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		PopulateSupportedFeePaymentAssets::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		PopulateSupportedFeePaymentAssets::<T>::pre_upgrade()
	}

	#[cfg(feature = "try-runtime")]
	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		PopulateSupportedFeePaymentAssets::<T>::post_upgrade(state)
	}
}

pub struct AuthorSlotFilterEligibleRatioToEligiblityCount<T>(PhantomData<T>);
impl<T> Migration for AuthorSlotFilterEligibleRatioToEligiblityCount<T>
where
	T: AuthorSlotFilterConfig,
{
	fn friendly_name(&self) -> &str {
		"MM_AuthorSlotFilter_EligibleRatioToEligiblityCount"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		EligibleRatioToEligiblityCount::<T>::on_runtime_upgrade()
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		EligibleRatioToEligiblityCount::<T>::pre_upgrade()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		EligibleRatioToEligiblityCount::<T>::post_upgrade(state)
	}
}

pub struct SchedulerMigrationV4<T>(PhantomData<T>);
impl<T> Migration for SchedulerMigrationV4<T>
where
	T: pallet_scheduler::Config<Hash = PreimageHash> + frame_system::Config,
{
	fn friendly_name(&self) -> &str {
		"MM_SchedulerMigrationV4"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		pallet_scheduler::migration::v3::MigrateToV4::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		pallet_scheduler::migration::v3::MigrateToV4::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		pallet_scheduler::migration::v3::MigrateToV4::<T>::post_upgrade(state)
	}
}

pub struct DemocracryMigrationHashToBoundedCall<T>(PhantomData<T>);
impl<T> Migration for DemocracryMigrationHashToBoundedCall<T>
where
	T: pallet_democracy::Config<Hash = PreimageHash> + frame_system::Config,
{
	fn friendly_name(&self) -> &str {
		"MM_DemocracryMigrationHashToBoundedCall"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		pallet_democracy::migrations::v1::Migration::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		pallet_democracy::migrations::v1::Migration::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		pallet_democracy::migrations::v1::Migration::<T>::post_upgrade(state)
	}
}

pub struct PreimageMigrationHashToBoundedCall<T>(PhantomData<T>);
impl<T> Migration for PreimageMigrationHashToBoundedCall<T>
where
	T: pallet_preimage::Config<Hash = PreimageHash> + frame_system::Config,
{
	fn friendly_name(&self) -> &str {
		"MM_PreimageMigrationHashToBoundedCall"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		pallet_preimage::migration::v1::Migration::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		pallet_preimage::migration::v1::Migration::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		pallet_preimage::migration::v1::Migration::<T>::post_upgrade(state)
	}
}

pub struct CommonMigrations<Runtime, Council, Tech>(PhantomData<(Runtime, Council, Tech)>);

impl<Runtime, Council, Tech> GetMigrations for CommonMigrations<Runtime, Council, Tech>
where
	Runtime: pallet_author_mapping::Config,
	Runtime: pallet_parachain_staking::Config,
	Runtime: pallet_scheduler::Config<Hash = PreimageHash>,
	Runtime: pallet_base_fee::Config,
	Runtime: AuthorSlotFilterConfig,
	Council: GetStorageVersion + PalletInfoAccess + 'static,
	Tech: GetStorageVersion + PalletInfoAccess + 'static,
	Runtime: pallet_democracy::Config<Hash = PreimageHash>,
	Runtime: pallet_preimage::Config<Hash = PreimageHash>,
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
		let staking_at_stake_auto_compound =
			ParachainStakingMigrateAtStakeAutoCompound::<Runtime>(Default::default());

		let scheduler_to_v4 = SchedulerMigrationV4::<Runtime>(Default::default());
		let democracy_migration_hash_to_bounded_call =
			DemocracryMigrationHashToBoundedCall::<Runtime>(Default::default());
		let preimage_migration_hash_to_bounded_call =
			PreimageMigrationHashToBoundedCall::<Runtime>(Default::default());
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
			Box::new(staking_at_stake_auto_compound),
			Box::new(scheduler_to_v4),
			Box::new(democracy_migration_hash_to_bounded_call),
			Box::new(preimage_migration_hash_to_bounded_call),
		]
	}
}
