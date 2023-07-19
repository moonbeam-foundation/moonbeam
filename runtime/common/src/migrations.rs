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
	dispatch::GetStorageVersion,
	traits::{Get, Hash as PreimageHash, OnRuntimeUpgrade, PalletInfoAccess},
	weights::Weight,
};
use pallet_author_slot_filter::Config as AuthorSlotFilterConfig;
use pallet_evm_chain_id::Config as EvmChainIdConfig;
use pallet_migrations::{GetMigrations, Migration};
use sp_std::{marker::PhantomData, prelude::*};

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

pub struct PalletReferendaMigrateV0ToV1<T>(pub PhantomData<T>);
impl<T> Migration for PalletReferendaMigrateV0ToV1<T>
where
	T: pallet_referenda::Config<Hash = PreimageHash> + frame_system::Config,
{
	fn friendly_name(&self) -> &str {
		"MM_PalletReferendaMigrateV0ToV1"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		pallet_referenda::migration::v1::MigrateV0ToV1::<T>::on_runtime_upgrade()
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		pallet_referenda::migration::v1::MigrateV0ToV1::<T>::pre_upgrade()
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		pallet_referenda::migration::v1::MigrateV0ToV1::<T>::post_upgrade(state)
	}
}

pub struct ReferendaMigrations<Runtime, Council, Tech>(PhantomData<(Runtime, Council, Tech)>);

impl<Runtime, Council, Tech> GetMigrations for ReferendaMigrations<Runtime, Council, Tech>
where
	Runtime: pallet_author_mapping::Config,
	Runtime: pallet_parachain_staking::Config,
	Runtime: pallet_scheduler::Config<Hash = PreimageHash>,
	Runtime: AuthorSlotFilterConfig,
	Council: GetStorageVersion + PalletInfoAccess + 'static,
	Tech: GetStorageVersion + PalletInfoAccess + 'static,
	Runtime: pallet_democracy::Config<Hash = PreimageHash>,
	Runtime: pallet_preimage::Config<Hash = PreimageHash>,
	Runtime: pallet_referenda::Config,
{
	fn get_migrations() -> Vec<Box<dyn Migration>> {
		let pallet_referenda_migrate_v0_to_v1 =
			PalletReferendaMigrateV0ToV1::<Runtime>(Default::default());
		vec![Box::new(pallet_referenda_migrate_v0_to_v1)]
	}
}

pub struct EvmChainIdMigration<T>(PhantomData<T>);

impl<T> Migration for EvmChainIdMigration<T>
where
	T: EvmChainIdConfig,
{
	fn friendly_name(&self) -> &str {
		"MM_EvmChainIdMigration"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		// Get the value of the chain Id from the storage.
		let old_pallet_name = b"EthereumChainId";
		let new_pallet_name = b"EvmChainId";
		let storage_name = b"ChainId";

		let weight = T::DbWeight::get().reads_writes(1, 2);
		frame_support::migration::move_storage_from_pallet(
			storage_name,
			old_pallet_name,
			new_pallet_name,
		);

		weight
	}

	/// Run a standard pre-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, &'static str> {
		let module = b"EthereumChainId";
		let item = b"ChainId";

		frame_support::migration::get_storage_value::<u64>(module, item, &[])
			.map(|chain_id| chain_id.to_le_bytes().to_vec())
			.ok_or("chain id not found")
	}

	/// Run a standard post-runtime test. This works the same way as in a normal runtime upgrade.
	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), &'static str> {
		use pallet_evm_chain_id::ChainId;

		let chain_id = ChainId::<T>::get();
		let state = u64::from_le_bytes(state.try_into().expect("chain id should be 8 bytes"));

		if state == chain_id {
			Ok(())
		} else {
			Err("chain id not equal")
		}
	}
}

pub struct CommonMigrations<Runtime, Council, Tech>(PhantomData<(Runtime, Council, Tech)>);

impl<Runtime, Council, Tech> GetMigrations for CommonMigrations<Runtime, Council, Tech>
where
	Runtime: pallet_author_mapping::Config,
	Runtime: pallet_parachain_staking::Config,
	Runtime: pallet_scheduler::Config<Hash = PreimageHash>,
	Runtime: AuthorSlotFilterConfig,
	Council: GetStorageVersion + PalletInfoAccess + 'static,
	Tech: GetStorageVersion + PalletInfoAccess + 'static,
	Runtime: pallet_democracy::Config<Hash = PreimageHash>,
	Runtime: pallet_preimage::Config<Hash = PreimageHash>,
	Runtime: pallet_asset_manager::Config,
	<Runtime as pallet_asset_manager::Config>::ForeignAssetType: From<xcm::v3::MultiLocation>,
	Runtime: pallet_xcm_transactor::Config,
	Runtime: pallet_evm_chain_id::Config,
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
		let evm_chain_id_migration = EvmChainIdMigration::<Runtime>(Default::default());
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
			Box::new(evm_chain_id_migration),
		]
	}
}
