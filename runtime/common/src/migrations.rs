// Copyright 2019-2020 PureStake Inc.
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
	traits::{Get, OnRuntimeUpgrade, PalletInfoAccess},
	weights::Weight,
};
#[cfg(feature = "xcm-support")]
use pallet_asset_manager::{
	migrations::{ChangeStateminePrefixes, PopulateAssetTypeIdStorage, UnitsWithAssetType},
	Config as AssetManagerConfig,
};
use pallet_author_mapping::{migrations::TwoXToBlake, Config as AuthorMappingConfig};
use pallet_migrations::{GetMigrations, Migration};
use parachain_staking::{
	migrations::{
		IncreaseMaxDelegationsPerCandidate, PurgeStaleStorage, RemoveExitQueue,
		SplitCandidateStateToDecreasePoV,
	},
	Config as ParachainStakingConfig,
};
use sp_std::{marker::PhantomData, prelude::*};
#[cfg(feature = "xcm-support")]
use xcm::latest::MultiLocation;
#[cfg(feature = "xcm-support")]
use xcm_transactor::{migrations::MaxTransactWeight, Config as XcmTransactorConfig};

/// This module acts as a registry where each migration is defined. Each migration should implement
/// the "Migration" trait declared in the pallet-migrations crate.

/// Staking split candidate state
pub struct ParachainStakingSplitCandidateState<T>(PhantomData<T>);
impl<T: ParachainStakingConfig> Migration for ParachainStakingSplitCandidateState<T> {
	fn friendly_name(&self) -> &str {
		"MM_Parachain_Staking_Split_Candidate_State"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		SplitCandidateStateToDecreasePoV::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		SplitCandidateStateToDecreasePoV::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		SplitCandidateStateToDecreasePoV::<T>::post_upgrade()
	}
}

/// Staking increase max counted delegations per collator candidate
pub struct ParachainStakingIncreaseMaxDelegationsPerCandidate<T>(PhantomData<T>);
impl<T: ParachainStakingConfig> Migration
	for ParachainStakingIncreaseMaxDelegationsPerCandidate<T>
{
	fn friendly_name(&self) -> &str {
		"MM_Parachain_Staking_IncreaseMaxDelegationsPerCandidate_v2"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		IncreaseMaxDelegationsPerCandidate::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		IncreaseMaxDelegationsPerCandidate::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		IncreaseMaxDelegationsPerCandidate::<T>::post_upgrade()
	}
}

/// Staking transition from automatic to manual exits, delay bond_{more, less} requests
pub struct ParachainStakingManualExits<T>(PhantomData<T>);
impl<T: ParachainStakingConfig> Migration for ParachainStakingManualExits<T> {
	fn friendly_name(&self) -> &str {
		"MM_Parachain_Staking_ManualExits"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		RemoveExitQueue::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		RemoveExitQueue::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		RemoveExitQueue::<T>::post_upgrade()
	}
}

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
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		PurgeStaleStorage::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		PurgeStaleStorage::<T>::post_upgrade()
	}
}

/// A moonbeam migration wrapping the similarly named migration in pallet-author-mapping
pub struct AuthorMappingTwoXToBlake<T>(PhantomData<T>);
impl<T: AuthorMappingConfig> Migration for AuthorMappingTwoXToBlake<T> {
	fn friendly_name(&self) -> &str {
		"MM_Author_Mapping_TwoXToBlake"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		TwoXToBlake::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		TwoXToBlake::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		TwoXToBlake::<T>::post_upgrade()
	}
}

const COUNCIL_OLD_PREFIX: &str = "Instance1Collective";
const TECH_OLD_PREFIX: &str = "Instance2Collective";

pub struct MigrateCollectivePallets<Runtime, Council, Tech>(PhantomData<(Runtime, Council, Tech)>);
impl<Runtime, Council, Tech> Migration for MigrateCollectivePallets<Runtime, Council, Tech>
where
	Runtime: frame_system::Config,
	Council: GetStorageVersion + PalletInfoAccess,
	Tech: GetStorageVersion + PalletInfoAccess,
{
	fn friendly_name(&self) -> &str {
		"MM_Collective_Pallets_v0.9.11_Prefixes"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		pallet_collective::migrations::v4::migrate::<Runtime, Council, _>(COUNCIL_OLD_PREFIX)
			+ pallet_collective::migrations::v4::migrate::<Runtime, Tech, _>(TECH_OLD_PREFIX)
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		pallet_collective::migrations::v4::pre_migrate::<Council, _>(COUNCIL_OLD_PREFIX);
		pallet_collective::migrations::v4::pre_migrate::<Tech, _>(TECH_OLD_PREFIX);
		Ok(())
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		pallet_collective::migrations::v4::post_migrate::<Council, _>(COUNCIL_OLD_PREFIX);
		pallet_collective::migrations::v4::post_migrate::<Tech, _>(TECH_OLD_PREFIX);
		Ok(())
	}
}

#[cfg(feature = "xcm-support")]
pub struct XcmTransactorMaxTransactWeight<T>(PhantomData<T>);
#[cfg(feature = "xcm-support")]
impl<T: XcmTransactorConfig> Migration for XcmTransactorMaxTransactWeight<T> {
	fn friendly_name(&self) -> &str {
		"MM_Xcm_Transactor_MaxTransactWeight"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		MaxTransactWeight::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		MaxTransactWeight::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		MaxTransactWeight::<T>::post_upgrade()
	}
}

#[cfg(feature = "xcm-support")]
pub struct AssetManagerUnitsWithAssetType<T>(PhantomData<T>);
#[cfg(feature = "xcm-support")]
impl<T: AssetManagerConfig> Migration for AssetManagerUnitsWithAssetType<T> {
	fn friendly_name(&self) -> &str {
		"MM_Asset_Manager_UnitsWithAssetType"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		UnitsWithAssetType::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		UnitsWithAssetType::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		UnitsWithAssetType::<T>::post_upgrade()
	}
}

#[cfg(feature = "xcm-support")]
pub struct AssetManagerPopulateAssetTypeIdStorage<T>(PhantomData<T>);
#[cfg(feature = "xcm-support")]
impl<T: AssetManagerConfig> Migration for AssetManagerPopulateAssetTypeIdStorage<T> {
	fn friendly_name(&self) -> &str {
		"MM_Asset_Manager_PopulateAssetTypeIdStorage"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		PopulateAssetTypeIdStorage::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		PopulateAssetTypeIdStorage::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		PopulateAssetTypeIdStorage::<T>::post_upgrade()
	}
}

#[cfg(feature = "xcm-support")]
pub struct AssetManagerChangeStateminePrefixes<T, StatemineParaIdInfo, StatemineAssetsPalletInfo>(
	PhantomData<(T, StatemineParaIdInfo, StatemineAssetsPalletInfo)>,
);
#[cfg(feature = "xcm-support")]
impl<T, StatemineParaIdInfo, StatemineAssetsPalletInfo> Migration
	for AssetManagerChangeStateminePrefixes<T, StatemineParaIdInfo, StatemineAssetsPalletInfo>
where
	T: AssetManagerConfig,
	StatemineParaIdInfo: Get<u32>,
	StatemineAssetsPalletInfo: Get<u8>,
	T::AssetType: Into<Option<MultiLocation>> + From<MultiLocation>,
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
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		ChangeStateminePrefixes::<T, StatemineParaIdInfo, StatemineAssetsPalletInfo>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		ChangeStateminePrefixes::<T, StatemineParaIdInfo, StatemineAssetsPalletInfo>::post_upgrade()
	}
}

pub struct CommonMigrations<Runtime, Council, Tech>(PhantomData<(Runtime, Council, Tech)>);

impl<Runtime, Council, Tech> GetMigrations for CommonMigrations<Runtime, Council, Tech>
where
	Runtime: pallet_author_mapping::Config + parachain_staking::Config,
	Council: GetStorageVersion + PalletInfoAccess + 'static,
	Tech: GetStorageVersion + PalletInfoAccess + 'static,
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
		let migration_parachain_staking_split_candidate_state =
			ParachainStakingSplitCandidateState::<Runtime>(Default::default());

		// TODO: this is a lot of allocation to do upon every get() call. this *should* be avoided
		// except when pallet_migrations undergoes a runtime upgrade -- but TODO: review

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
			Box::new(migration_parachain_staking_split_candidate_state),
		]
	}
}

//TODO: Once the statemine prefix migration is applied,
// we can remove StatemineParaIdInfo and StatemineAssetsInstanceInfo
// but for now we need a way to pass these parameters, which are distinct for each of the runtimes
#[cfg(feature = "xcm-support")]
pub struct XcmMigrations<Runtime, StatemineParaIdInfo, StatemineAssetsInstanceInfo>(
	PhantomData<(Runtime, StatemineParaIdInfo, StatemineAssetsInstanceInfo)>,
);

#[cfg(feature = "xcm-support")]
impl<Runtime, StatemineParaIdInfo, StatemineAssetsInstanceInfo> GetMigrations
	for XcmMigrations<Runtime, StatemineParaIdInfo, StatemineAssetsInstanceInfo>
where
	Runtime: xcm_transactor::Config + pallet_migrations::Config + pallet_asset_manager::Config,
	StatemineParaIdInfo: Get<u32> + 'static,
	StatemineAssetsInstanceInfo: Get<u8> + 'static,
	<Runtime as pallet_asset_manager::Config>::AssetType:
		Into<Option<MultiLocation>> + From<MultiLocation>,
{
	fn get_migrations() -> Vec<Box<dyn Migration>> {
		let xcm_transactor_max_weight =
			XcmTransactorMaxTransactWeight::<Runtime>(Default::default());

		let asset_manager_units_with_asset_type =
			AssetManagerUnitsWithAssetType::<Runtime>(Default::default());

		let asset_manager_populate_asset_type_id_storage =
			AssetManagerPopulateAssetTypeIdStorage::<Runtime>(Default::default());

		let asset_manager_change_statemine_prefixes = AssetManagerChangeStateminePrefixes::<
			Runtime,
			StatemineParaIdInfo,
			StatemineAssetsInstanceInfo,
		>(Default::default());

		// TODO: this is a lot of allocation to do upon every get() call. this *should* be avoided
		// except when pallet_migrations undergoes a runtime upgrade -- but TODO: review

		vec![
			Box::new(xcm_transactor_max_weight),
			Box::new(asset_manager_units_with_asset_type),
			Box::new(asset_manager_change_statemine_prefixes),
			Box::new(asset_manager_populate_asset_type_id_storage),
		]
	}
}
