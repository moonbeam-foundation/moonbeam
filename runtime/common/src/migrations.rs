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
#[cfg(feature = "try-runtime")]
use frame_support::migration::get_storage_value;
use frame_support::{
	parameter_types,
	sp_runtime::traits::{Block as BlockT, Header as HeaderT},
	traits::OnRuntimeUpgrade,
	weights::Weight,
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_author_slot_filter::Config as AuthorSlotFilterConfig;
use pallet_migrations::{GetMigrations, Migration};
use pallet_parachain_staking::{Round, RoundIndex, RoundInfo};
use parity_scale_codec::{Decode, Encode};
use sp_consensus_slots::Slot;
use sp_core::Get;
use sp_std::{marker::PhantomData, prelude::*, vec};

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode)]
pub struct OldRoundInfo<BlockNumber> {
	pub current: RoundIndex,
	pub first: BlockNumber,
	pub length: u32,
}
pub struct UpdateFirstRoundNumberValue<T>(pub PhantomData<T>);
impl<T> Migration for UpdateFirstRoundNumberValue<T>
where
	T: pallet_parachain_staking::Config,
	T: pallet_async_backing::Config,
	T: frame_system::Config,
	u32: From<<<<T as frame_system::Config>::Block as BlockT>::Header as HeaderT>::Number>,
{
	fn friendly_name(&self) -> &str {
		"MM_UpdateFirstRoundNumberValue"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		let _ = Round::<T>::translate::<OldRoundInfo<BlockNumberFor<T>>, _>(|v0| {
			let old_current = v0
				.expect("old current round value must be present!")
				.current;
			let old_first: u32 = u32::from(v0.expect("old first should be present!").first);
			let old_length = v0.expect("old round length value must be present!").length;

			// Fetch the last parachain block
			let para_block: u32 = u32::from(frame_system::Pallet::<T>::block_number());

			// Calculate how many blocks have passed so far in this round
			let para_block_diff: u64 = para_block.saturating_sub(old_first).into();

			// Read the last relay slot from the SlotInfo storage
			let relay_slot = pallet_async_backing::Pallet::<T>::slot_info()
				.unwrap_or((Slot::from(284_000_000u64), 0u32))
				.0;

			// Calculate the new first
			let new_first = u64::from(relay_slot).saturating_sub(para_block_diff);

			Some(RoundInfo {
				current: old_current,
				first: new_first,
				length: old_length,
			})
		});

		T::DbWeight::get().reads_writes(1, 1)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, sp_runtime::DispatchError> {
		let module: &[u8] = b"ParachainStaking";
		let item: &[u8] = b"Round";
		let pre_round_info = get_storage_value::<RoundInfo<BlockNumberFor<T>>>(module, item, &[]);
		Ok(pre_round_info.unwrap_or_default().encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		let pre_round_info =
			<RoundInfo<BlockNumberFor<T>> as Decode>::decode(&mut &*state).unwrap_or_default();
		let post_round_info = pallet_parachain_staking::Pallet::<T>::round();

		let slot_after = pallet_async_backing::Pallet::<T>::slot_info()
			.unwrap_or((Slot::from(280_000_000u64), 0u32))
			.0;

		ensure!(
			u64::from(slot_after) > post_round_info.first,
			"Post-round first must be lower than last relay slot"
		);
		ensure!(
			post_round_info.current >= pre_round_info.current,
			"Post-round number must be higher than or equal pre-round one"
		);
		ensure!(
			pre_round_info.length == post_round_info.length,
			"Post-round length must be equal to pre-round one"
		);
		Ok(())
	}
}

/// Translates the Round.first value type from BlockNumberFor to u64
pub struct UpdateFirstRoundNumberType<T>(pub PhantomData<T>);
impl<T> Migration for UpdateFirstRoundNumberType<T>
where
	T: pallet_parachain_staking::Config,
	T: frame_system::Config,
	u64: From<<<<T as frame_system::Config>::Block as BlockT>::Header as HeaderT>::Number>,
{
	fn friendly_name(&self) -> &str {
		"MM_UpdateFirstRoundNumberType"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		let _ = Round::<T>::translate::<OldRoundInfo<BlockNumberFor<T>>, _>(|v0| {
			let old_current = v0
				.expect("old current round value must be present!")
				.current;

			let new_first: u64 = u64::from(v0.expect("old first should be present!").first);
			let old_length = v0.expect("old round length value must be present!").length;

			Some(RoundInfo {
				current: old_current,
				first: new_first,
				length: old_length,
			})
		});

		T::DbWeight::get().reads_writes(1, 1)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, sp_runtime::DispatchError> {
		let module: &[u8] = b"ParachainStaking";
		let item: &[u8] = b"Round";
		let pre_round_info = get_storage_value::<RoundInfo<BlockNumberFor<T>>>(module, item, &[]);
		Ok(pre_round_info.unwrap_or_default().encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		let pre_round_info =
			<RoundInfo<BlockNumberFor<T>> as Decode>::decode(&mut &*state).unwrap_or_default();
		let post_round_info = pallet_parachain_staking::Pallet::<T>::round();
		ensure!(
			post_round_info.first == u64::from(pre_round_info.first),
			"Post-round number must be equal to pre-round one"
		);
		ensure!(
			pre_round_info.length == post_round_info.length,
			"Post-round length must be equal to pre-round one"
		);
		Ok(())
	}
}

parameter_types! {
	pub const CouncilPalletName: &'static str = "Council";
	pub const TechnicalCommitteePalletName: &'static str = "TechnicalCommittee";
}

pub struct PalletCollectiveDropGovV1Collectives<Runtime>(pub PhantomData<Runtime>);
impl<Runtime> Migration for PalletCollectiveDropGovV1Collectives<Runtime>
where
	Runtime: frame_system::Config,
{
	fn friendly_name(&self) -> &str {
		"MM_RemoveGovV1Collectives"
	}

	fn migrate(&self, _available_weight: Weight) -> Weight {
		log::info!("Removing Council and Tech from pallet_collective");

		let mut weight = Weight::zero();

		let w = frame_support::migrations::RemovePallet::<
			CouncilPalletName,
			<Runtime as frame_system::Config>::DbWeight,
		>::on_runtime_upgrade();
		weight = weight.saturating_add(w);

		let w = frame_support::migrations::RemovePallet::<
			TechnicalCommitteePalletName,
			<Runtime as frame_system::Config>::DbWeight,
		>::on_runtime_upgrade();
		weight = weight.saturating_add(w);
		weight
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade(&self) -> Result<Vec<u8>, sp_runtime::DispatchError> {
		frame_support::migrations::RemovePallet::<
			TechnicalCommitteePalletName,
			<Runtime as frame_system::Config>::DbWeight,
		>::pre_upgrade();

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(&self, _state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		frame_support::migrations::RemovePallet::<
			TechnicalCommitteePalletName,
			<Runtime as frame_system::Config>::DbWeight,
		>::post_upgrade(_state);
		Ok(())
	}
}

pub struct CommonMigrations<Runtime>(PhantomData<Runtime>);

impl<Runtime> GetMigrations for CommonMigrations<Runtime>
where
	Runtime: pallet_author_mapping::Config,
	Runtime: pallet_parachain_staking::Config,
	Runtime: pallet_scheduler::Config,
	Runtime: AuthorSlotFilterConfig,
	Runtime: pallet_democracy::Config,
	Runtime: pallet_preimage::Config,
	Runtime: pallet_asset_manager::Config,
	<Runtime as pallet_asset_manager::Config>::ForeignAssetType: From<xcm::v4::Location>,
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
		// let missing_balances_migrations = MissingBalancesMigrations::<Runtime>(Default::default());
		// let fix_pallet_versions =
		// 	FixIncorrectPalletVersions::<Runtime, Treasury, OpenTech>(Default::default());
		// let pallet_referenda_migrate_v0_to_v1 =
		// 	PalletReferendaMigrateV0ToV1::<Runtime>(Default::default());
		let pallet_collective_drop_gov_v1_collectives =
			PalletCollectiveDropGovV1Collectives::<Runtime>(Default::default());

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
			Box::new(pallet_collective_drop_gov_v1_collectives),
		]
	}
}
