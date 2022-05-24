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

#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;
use frame_support::{
	dispatch::GetStorageVersion,
	storage::migration::get_storage_value,
	traits::{Get, OnRuntimeUpgrade, PalletInfoAccess},
	weights::Weight,
};
#[cfg(feature = "xcm-support")]
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
use pallet_base_fee::Config as BaseFeeConfig;
use pallet_migrations::{GetMigrations, Migration};
use parachain_staking::{
	migrations::{
		IncreaseMaxDelegationsPerCandidate, PatchIncorrectDelegationSums, PurgeStaleStorage,
		SplitCandidateStateToDecreasePoV, SplitDelegatorStateIntoDelegationScheduledRequests,
	},
	Config as ParachainStakingConfig,
};
use sp_runtime::Permill;
use sp_std::{marker::PhantomData, prelude::*};
#[cfg(feature = "xcm-support")]
use xcm::latest::MultiLocation;
#[cfg(feature = "xcm-support")]
use xcm_transactor::{
	migrations::TransactSignedWeightAndFeePerSecond, Config as XcmTransactorConfig,
};

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
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		AddAccountIdToNimbusLookup::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		AddAccountIdToNimbusLookup::<T>::post_upgrade()
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
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		AddKeysToRegistrationInfo::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		AddKeysToRegistrationInfo::<T>::post_upgrade()
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
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		PatchIncorrectDelegationSums::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		PatchIncorrectDelegationSums::<T>::post_upgrade()
	}
}

/// Staking split delegator state into [parachain_staking::DelegatorScheduledRequests]
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
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		SplitDelegatorStateIntoDelegationScheduledRequests::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		SplitDelegatorStateIntoDelegationScheduledRequests::<T>::post_upgrade()
	}
}

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
// pub struct AuthorMappingTwoXToBlake<T>(PhantomData<T>);
// impl<T: AuthorMappingConfig> Migration for AuthorMappingTwoXToBlake<T> {
// 	fn friendly_name(&self) -> &str {
// 		"MM_Author_Mapping_TwoXToBlake"
// 	}

// 	fn migrate(&self, _available_weight: Weight) -> Weight {
// 		TwoXToBlake::<T>::on_runtime_upgrade()
// 	}

// 	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
// 	#[cfg(feature = "try-runtime")]
// 	fn pre_upgrade(&self) -> Result<(), &'static str> {
// 		TwoXToBlake::<T>::pre_upgrade()
// 	}

// 	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
// 	#[cfg(feature = "try-runtime")]
// 	fn post_upgrade(&self) -> Result<(), &'static str> {
// 		TwoXToBlake::<T>::post_upgrade()
// 	}
// }

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

/// BaseFee pallet, set missing storage values.
pub struct BaseFeePerGas<T>(PhantomData<T>);
impl<T: BaseFeeConfig> OnRuntimeUpgrade for BaseFeePerGas<T> {
	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		let module: &[u8] = b"BaseFee";
		// Verify the storage before the upgrade is empty
		{
			let item: &[u8] = b"BaseFeePerGas";
			let value = get_storage_value::<sp_core::U256>(module, item, &[]);
			Self::set_temp_storage(value.is_none(), "base_fee_is_empty");
		}
		// Elasticity storage value
		{
			let item: &[u8] = b"Elasticity";
			let value = get_storage_value::<Permill>(module, item, &[]);
			Self::set_temp_storage(value.is_none(), "elasticity_is_empty");
		}

		Ok(())
	}

	fn on_runtime_upgrade() -> Weight {
		let module: &[u8] = b"BaseFee";
		let db_weights = T::DbWeight::get();
		let mut weight: Weight = 2 * db_weights.read;
		// BaseFeePerGas storage value
		{
			let item: &[u8] = b"BaseFeePerGas";
			let current_value = get_storage_value::<sp_core::U256>(module, item, &[]);
			if current_value.is_none() {
				// Put the default configured value in storage
				let write = pallet_base_fee::Pallet::<T>::set_base_fee_per_gas_inner(
					T::DefaultBaseFeePerGas::get(),
				);
				weight = weight.saturating_add(write);
			}
		}
		// Elasticity storage value
		{
			let item: &[u8] = b"Elasticity";
			let current_value = get_storage_value::<Permill>(module, item, &[]);
			if current_value.is_none() {
				// Put the default value in storage
				let write = pallet_base_fee::Pallet::<T>::set_elasticity_inner(
					Permill::from_parts(125_000),
				);
				weight = weight.saturating_add(write);
			}
		}
		weight
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		if Self::get_temp_storage::<bool>("base_fee_is_empty").is_some()
			&& Self::get_temp_storage::<bool>("elasticity_is_empty").is_some()
		{
			// Verify the storage after the upgrade matches the runtime configured default
			let module: &[u8] = b"BaseFee";
			// BaseFeePerGas storage value
			{
				let item: &[u8] = b"BaseFeePerGas";
				let value = get_storage_value::<sp_core::U256>(module, item, &[]);
				assert_eq!(value, Some(T::DefaultBaseFeePerGas::get()));
			}
			// Elasticity storage value
			{
				let item: &[u8] = b"Elasticity";
				let value = get_storage_value::<Permill>(module, item, &[]);
				assert_eq!(value, Some(Permill::from_parts(125_000)));
			}
		}

		Ok(())
	}
}

pub struct MigrateBaseFeePerGas<T>(PhantomData<T>);
// This is not strictly a migration, just an `on_runtime_upgrade` alternative to open a democracy
// proposal to set this values through an extrinsic.
impl<T: BaseFeeConfig> Migration for MigrateBaseFeePerGas<T> {
	fn friendly_name(&self) -> &str {
		"MM_Base_Fee_Per_Gas"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		BaseFeePerGas::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		BaseFeePerGas::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		BaseFeePerGas::<T>::post_upgrade()
	}
}

// #[cfg(feature = "xcm-support")]
// pub struct XcmTransactorMaxTransactWeight<T>(PhantomData<T>);
// #[cfg(feature = "xcm-support")]
// impl<T: XcmTransactorConfig> Migration for XcmTransactorMaxTransactWeight<T> {
// 	fn friendly_name(&self) -> &str {
// 		"MM_Xcm_Transactor_MaxTransactWeight"
// 	}

// 	fn migrate(&self, _available_weight: Weight) -> Weight {
// 		MaxTransactWeight::<T>::on_runtime_upgrade()
// 	}

// 	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
// 	#[cfg(feature = "try-runtime")]
// 	fn pre_upgrade(&self) -> Result<(), &'static str> {
// 		MaxTransactWeight::<T>::pre_upgrade()
// 	}

// 	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
// 	#[cfg(feature = "try-runtime")]
// 	fn post_upgrade(&self) -> Result<(), &'static str> {
// 		MaxTransactWeight::<T>::post_upgrade()
// 	}
// }

#[cfg(feature = "xcm-support")]
pub struct XcmTransactorTransactSignedWeightAndFeePerSecond<T>(PhantomData<T>);
#[cfg(feature = "xcm-support")]
impl<T: XcmTransactorConfig> Migration for XcmTransactorTransactSignedWeightAndFeePerSecond<T> {
	fn friendly_name(&self) -> &str {
		"MM_Xcm_Transactor_TransactSignedWeightAndFeePerSecond"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		TransactSignedWeightAndFeePerSecond::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		TransactSignedWeightAndFeePerSecond::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		TransactSignedWeightAndFeePerSecond::<T>::post_upgrade()
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
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		ChangeStateminePrefixes::<T, StatemineParaIdInfo, StatemineAssetsPalletInfo>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		ChangeStateminePrefixes::<T, StatemineParaIdInfo, StatemineAssetsPalletInfo>::post_upgrade()
	}
}

#[cfg(feature = "xcm-support")]
pub struct XcmPaymentSupportedAssets<T>(PhantomData<T>);
#[cfg(feature = "xcm-support")]
impl<T: AssetManagerConfig> Migration for XcmPaymentSupportedAssets<T> {
	fn friendly_name(&self) -> &str {
		"MM_Xcm_Payment_Supported_Assets"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		PopulateSupportedFeePaymentAssets::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		PopulateSupportedFeePaymentAssets::<T>::pre_upgrade()
	}

	#[cfg(feature = "try-runtime")]
	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	fn post_upgrade(&self) -> Result<(), &'static str> {
		PopulateSupportedFeePaymentAssets::<T>::post_upgrade()
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
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		EligibleRatioToEligiblityCount::<T>::pre_upgrade()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		EligibleRatioToEligiblityCount::<T>::post_upgrade()
	}
}

pub struct SchedulerMigrationV3<T>(PhantomData<T>);
impl<T: pallet_scheduler::Config> Migration for SchedulerMigrationV3<T> {
	fn friendly_name(&self) -> &str {
		"MM_SchedulerMigrationV3"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		pallet_scheduler::Pallet::<T>::migrate_v2_to_v3()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<(), &'static str> {
		pallet_scheduler::Pallet::<T>::pre_migrate_to_v3()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self) -> Result<(), &'static str> {
		pallet_scheduler::Pallet::<T>::post_migrate_to_v3()
	}
}

pub struct CommonMigrations<Runtime, Council, Tech>(PhantomData<(Runtime, Council, Tech)>);

impl<Runtime, Council, Tech> GetMigrations for CommonMigrations<Runtime, Council, Tech>
where
	Runtime: pallet_author_mapping::Config,
	Runtime: parachain_staking::Config,
	Runtime: pallet_scheduler::Config,
	Runtime: pallet_base_fee::Config,
	Runtime: AuthorSlotFilterConfig,
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
		let migration_author_mapping_add_account_id_to_nimbus_lookup =
			AuthorMappingAddAccountIdToNimbusLookup::<Runtime>(Default::default());
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
			// completed in runtime 1300
			// Box::new(migration_scheduler_v3),
			// completed in runtime 1300
			// Box::new(migration_parachain_staking_patch_incorrect_delegation_sums),
			// completed in runtime 1300
			// Box::new(migration_base_fee),
			// completed in runtime 1500
			// Box::new(migration_author_slot_filter_eligible_ratio_to_eligibility_count),
			// Box::new(migration_author_mapping_add_keys_to_registration_info),
			// Box::new(staking_delegator_state_requests),

			// planned in runtime 1600
			Box::new(migration_author_mapping_add_account_id_to_nimbus_lookup),
		]
	}
}

#[cfg(feature = "xcm-support")]
pub struct XcmMigrations<Runtime>(PhantomData<Runtime>);

#[cfg(feature = "xcm-support")]
impl<Runtime> GetMigrations for XcmMigrations<Runtime>
where
	Runtime: xcm_transactor::Config + pallet_migrations::Config + pallet_asset_manager::Config,
	<Runtime as pallet_asset_manager::Config>::ForeignAssetType:
		Into<Option<MultiLocation>> + From<MultiLocation>,
{
	fn get_migrations() -> Vec<Box<dyn Migration>> {
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

		// TODO: this is a lot of allocation to do upon every get() call. this *should* be avoided
		// except when pallet_migrations undergoes a runtime upgrade -- but TODO: review

		let xcm_transactor_transact_signed =
			XcmTransactorTransactSignedWeightAndFeePerSecond::<Runtime>(Default::default());

		vec![
			// completed in runtime 1201
			// Box::new(xcm_transactor_max_weight),
			// completed in runtime 1201
			// Box::new(asset_manager_units_with_asset_type),
			// completed in runtime 1201
			// Box::new(asset_manager_change_statemine_prefixes),
			// completed in runtime 1201
			// Box::new(asset_manager_populate_asset_type_id_storage),
			// completed in runtime 1300
			// Box::new(xcm_supported_assets),

			// planned in runtime 1600
			Box::new(xcm_transactor_transact_signed),
		]
	}
}
