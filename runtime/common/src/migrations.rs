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

use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_migrations::{GetMigrations, Migration};
use sp_std::{marker::PhantomData, prelude::*, vec};

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

// pub struct MigrateCodeToStateTrieV1<Runtime>(PhantomData<Runtime>);
// impl<Runtime> Migration for MigrateCodeToStateTrieV1<Runtime>
// where
// 	Runtime: frame_system::Config,
// {
// 	fn friendly_name(&self) -> &str {
// 		"MM_MigrateCodeToStateTrieVersion1"
// 	}

// 	fn migrate(&self, _available_weight: Weight) -> Weight {
// 		use cumulus_primitives_storage_weight_reclaim::get_proof_size;
// 		use sp_core::Get;

// 		let proof_size_before: u64 = get_proof_size().unwrap_or(0);

// 		let key = sp_core::storage::well_known_keys::CODE;
// 		let data = sp_io::storage::get(&key);
// 		if let Some(data) = data {
// 			sp_io::storage::set(&key, &data);
// 		}

// 		let proof_size_after: u64 = get_proof_size().unwrap_or(0);
// 		let proof_size_diff = proof_size_after.saturating_sub(proof_size_before);

// 		Weight::from_parts(0, proof_size_diff)
// 			.saturating_add(<Runtime as frame_system::Config>::DbWeight::get().reads_writes(1, 1))
// 	}

// 	#[cfg(feature = "try-runtime")]
// 	fn pre_upgrade(&self) -> Result<Vec<u8>, sp_runtime::DispatchError> {
// 		use parity_scale_codec::Encode;

// 		let key = sp_core::storage::well_known_keys::CODE;
// 		let data = sp_io::storage::get(&key);
// 		Ok(Encode::encode(&data))
// 	}

// 	#[cfg(feature = "try-runtime")]
// 	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
// 		use frame_support::ensure;
// 		use parity_scale_codec::Encode;
// 		use sp_core::storage::StorageKey;

// 		let key = StorageKey(sp_core::storage::well_known_keys::CODE.to_vec());
// 		let data = sp_io::storage::get(key.as_ref());

// 		ensure!(Encode::encode(&data) == state, "Invalid state");

// 		Ok(())
// 	}
// }

#[derive(parity_scale_codec::Decode, Eq, Ord, PartialEq, PartialOrd)]
enum OldAssetType {
	Xcm(xcm::v3::Location),
}

pub struct MigrateXcmFeesAssetsMeatdata<Runtime>(PhantomData<Runtime>);
impl<Runtime> Migration for MigrateXcmFeesAssetsMeatdata<Runtime>
where
	Runtime: pallet_transaction_payment::Config,
	Runtime: pallet_xcm_weight_trader::Config,
{
	fn friendly_name(&self) -> &str {
		"MM_MigrateXcmFeesAssetsMetadata"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		let supported_assets =
			if let Some(supported_assets) = frame_support::storage::migration::get_storage_value::<
				Vec<OldAssetType>,
			>(b"AssetManager", b"SupportedFeePaymentAssets", &[])
			{
				sp_std::collections::btree_set::BTreeSet::from_iter(
					supported_assets
						.into_iter()
						.map(|OldAssetType::Xcm(location_v3)| location_v3),
				)
			} else {
				return Weight::default();
			};

		let mut assets: Vec<(xcm::v4::Location, (bool, u128))> = Vec::new();

		for (OldAssetType::Xcm(location_v3), units_per_seconds) in
			frame_support::storage::migration::storage_key_iter::<
				OldAssetType,
				u128,
				frame_support::Blake2_128Concat,
			>(b"AssetManager", b"AssetTypeUnitsPerSecond")
		{
			let enabled = supported_assets.get(&location_v3).is_some();

			if let Ok(location_v4) = location_v3.try_into() {
				assets.push((location_v4, (enabled, units_per_seconds)));
			}
		}

		//***** Start mutate storage *****//

		// Write asset metadata in new pallet_xcm_weight_trader
		use frame_support::weights::WeightToFee as _;
		for (asset_location, (enabled, units_per_second)) in assets {
			let native_amount_per_second: u128 =
				<Runtime as pallet_transaction_payment::Config>::WeightToFee::weight_to_fee(
					&Weight::from_parts(
						frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND,
						0,
					),
				)
				.try_into()
				.unwrap_or(u128::MAX);
			let relative_price: u128 = native_amount_per_second
				.saturating_mul(10u128.pow(pallet_xcm_weight_trader::RELATIVE_PRICE_DECIMALS))
				.saturating_div(units_per_second);
			pallet_xcm_weight_trader::SupportedAssets::<Runtime>::insert(
				asset_location,
				(enabled, relative_price),
			);
		}

		// Remove storage value AssetManager::SupportedFeePaymentAssets
		frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
			b"AssetManager",
			b"SupportedFeePaymentAssets",
		));

		// Remove storage map AssetManager::AssetTypeUnitsPerSecond
		let _ = frame_support::storage::migration::clear_storage_prefix(
			b"AssetManager",
			b"AssetTypeUnitsPerSecond",
			&[],
			None,
			None,
		);

		Weight::default()
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, sp_runtime::DispatchError> {
		Ok(Default::default())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		assert!(frame_support::storage::migration::storage_key_iter::<
			OldAssetType,
			u128,
			frame_support::Blake2_128Concat,
		>(b"AssetManager", b"AssetTypeUnitsPerSecond")
		.next()
		.is_none());

		Ok(())
	}
}

pub struct CommonMigrations<Runtime>(PhantomData<Runtime>);

impl<Runtime> GetMigrations for CommonMigrations<Runtime>
where
	Runtime:
		pallet_xcm::Config + pallet_transaction_payment::Config + pallet_xcm_weight_trader::Config,
	Runtime::AccountId: Default,
	BlockNumberFor<Runtime>: Into<u64>,
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
			// Box::new(remove_pallet_democracy),
			// Box::new(remove_collectives_addresses),
			// Box::new(MigrateCodeToStateTrieV1::<Runtime>(Default::default())),
			// completed in runtime 3200
			Box::new(MigrateXcmFeesAssetsMeatdata::<Runtime>(Default::default())),
			// permanent migrations
			Box::new(MigrateToLatestXcmVersion::<Runtime>(Default::default())),
		]
	}
}
