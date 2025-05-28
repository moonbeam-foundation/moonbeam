// Copyright 2019-2025 PureStake Inc.
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

extern crate alloc;

#[cfg(feature = "try-runtime")]
use alloc::vec::Vec;

use frame_support::{
	pallet_prelude::StorageVersion,
	traits::{Get, GetStorageVersion, OnRuntimeUpgrade},
	weights::Weight,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_runtime::RuntimeDebug;

use crate::*;

#[derive(
	Clone,
	PartialEq,
	Eq,
	parity_scale_codec::Decode,
	parity_scale_codec::Encode,
	sp_runtime::RuntimeDebug,
)]
/// Reserve information { account, percent_of_inflation }
pub struct OldParachainBondConfig<AccountId> {
	/// Account which receives funds intended for parachain bond
	pub account: AccountId,
	/// Percent of inflation set aside for parachain bond account
	pub percent: sp_runtime::Percent,
}

pub struct MigrateParachainBondConfig<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateParachainBondConfig<T> {
	fn on_runtime_upgrade() -> Weight {
		let (account, percent) = if let Some(config) =
			frame_support::storage::migration::get_storage_value::<
				OldParachainBondConfig<T::AccountId>,
			>(b"ParachainStaking", b"ParachainBondInfo", &[])
		{
			(config.account, config.percent)
		} else {
			return Weight::default();
		};

		let pbr = InflationDistributionAccount { account, percent };
		let treasury = InflationDistributionAccount::<T::AccountId>::default();
		let configs: InflationDistributionConfig<T::AccountId> = [pbr, treasury].into();

		//***** Start mutate storage *****//

		InflationDistributionInfo::<T>::put(configs);

		// Remove storage value ParachainStaking::ParachainBondInfo
		frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
			b"ParachainStaking",
			b"ParachainBondInfo",
		));

		Weight::default()
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		use frame_support::ensure;
		use parity_scale_codec::Encode;

		let state = frame_support::storage::migration::get_storage_value::<
			OldParachainBondConfig<T::AccountId>,
		>(b"ParachainStaking", b"ParachainBondInfo", &[]);

		ensure!(state.is_some(), "State not found");

		Ok(state
			.expect("should be Some(_) due to former call to ensure!")
			.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		use frame_support::ensure;

		let old_state: OldParachainBondConfig<T::AccountId> =
			parity_scale_codec::Decode::decode(&mut &state[..])
				.map_err(|_| sp_runtime::DispatchError::Other("Failed to decode old state"))?;

		let new_state = InflationDistributionInfo::<T>::get();

		let pbr = InflationDistributionAccount {
			account: old_state.account,
			percent: old_state.percent,
		};
		let treasury = InflationDistributionAccount::<T::AccountId>::default();
		let expected_new_state: InflationDistributionConfig<T::AccountId> = [pbr, treasury].into();

		ensure!(new_state == expected_new_state, "State migration failed");

		Ok(())
	}
}

/// Migration to convert balance locks to freezes for staking
pub struct LazyMigrationV0ToV1<T>(sp_std::marker::PhantomData<T>);

/// Migration state for MigrateLocksToFreezes
#[derive(
	Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, scale_info::TypeInfo, RuntimeDebug,
)]
pub enum MigrateLocksToFreezesState<AccountId> {
	/// Processing a specific candidate's delegators
	ProcessingCandidate {
		candidate: AccountId,
		top_index: u32,
		bottom_index: u32,
		migrated_candidates_count: u32,
	},
	/// Migration completed with total count
	Finished { total_candidates_migrated: u32 },
}

impl<T: Config> frame_support::migrations::SteppedMigration for LazyMigrationV0ToV1<T>
where
	// Ensure Currency::Balance and fungible Inspect::Balance are the same type
	<<T as Config>::Currency as frame_support::traits::Currency<T::AccountId>>::Balance:
		From<BalanceOf<T>>,
	BalanceOf<T>:
		From<<<T as Config>::Currency as frame_support::traits::Currency<T::AccountId>>::Balance>,
{
	type Cursor = MigrateLocksToFreezesState<T::AccountId>;
	type Identifier = frame_support::migrations::MigrationId<16>;

	fn id() -> Self::Identifier {
		frame_support::migrations::MigrationId {
			pallet_id: *b"para-staking-mbm",
			version_from: 0,
			version_to: 1,
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
		// Count the number of candidates
		let candidate_count = <CandidateInfo<T>>::iter().count() as u32;

		log::info!(
			"Pre-upgrade: Found {} candidates to migrate",
			candidate_count
		);

		// Encode the count for post-upgrade verification
		Ok(candidate_count.encode())
	}

	fn step(
		mut cursor: Option<Self::Cursor>,
		meter: &mut frame_support::weights::WeightMeter,
	) -> Result<Option<Self::Cursor>, frame_support::migrations::SteppedMigrationError> {
		use frame_support::traits::{fungible::MutateFreeze, LockableCurrency};

		// Check storage version - only run if we're on version 0
		if crate::Pallet::<T>::on_chain_storage_version() != Self::id().version_from as u16 {
			return Ok(None);
		}

		// Define weight for operations
		let single_migration_weight = T::DbWeight::get().reads_writes(1, 2);

		// Check that we have enough weight for at least the next step
		let required_weight = match &cursor {
			None => single_migration_weight, // First operation weight
			Some(MigrateLocksToFreezesState::ProcessingCandidate { .. }) => single_migration_weight,
			Some(MigrateLocksToFreezesState::Finished { .. }) => Weight::zero(),
		};
		if meter.remaining().any_lt(required_weight) {
			return Err(
				frame_support::migrations::SteppedMigrationError::InsufficientWeight {
					required: required_weight,
				},
			);
		}

		match cursor {
			None => {
				// Initialize with the first candidate
				let first_candidate = <CandidateInfo<T>>::iter_keys().next();

				match first_candidate {
					Some(candidate) => {
						log::info!("Starting migration with candidate {:?}", candidate);
						cursor = Some(MigrateLocksToFreezesState::ProcessingCandidate {
							candidate,
							top_index: 0,
							bottom_index: 0,
							migrated_candidates_count: 0,
						});
					}
					None => {
						// No candidates to migrate
						log::info!("No candidates to migrate");
						return Ok(Some(MigrateLocksToFreezesState::Finished {
							total_candidates_migrated: 0,
						}));
					}
				}
			}
			Some(MigrateLocksToFreezesState::Finished { .. }) => {
				// Migration already completed
				return Ok(None);
			}
			_ => {}
		}

		// Process based on current state
		match cursor.clone().unwrap() {
			MigrateLocksToFreezesState::ProcessingCandidate {
				candidate,
				top_index,
				bottom_index,
				migrated_candidates_count,
			} => {
				// Process delegators for this candidate in batches
				const BATCH_SIZE: u64 = 100;

				let mut processed = 0u64;
				let mut current_top_index = top_index;
				let mut current_bottom_index = bottom_index;

				// Get the candidate's top and bottom delegations once
				let top_delegations = <TopDelegations<T>>::get(&candidate);
				let bottom_delegations = <BottomDelegations<T>>::get(&candidate);

				// Process top delegators first
				if let Some(ref top) = top_delegations {
					let top_count = top.delegations.len() as u32;

					while current_top_index < top_count && processed < BATCH_SIZE {
						if !meter.can_consume(single_migration_weight) {
							// Return to continue processing
							return Ok(Some(MigrateLocksToFreezesState::ProcessingCandidate {
								candidate,
								top_index: current_top_index,
								bottom_index: current_bottom_index,
								migrated_candidates_count,
							}));
						}

						if let Some(delegation) = top.delegations.get(current_top_index as usize) {
							let delegator = &delegation.owner;

							// Get delegator state
							if let Some(delegator_state) = <DelegatorState<T>>::get(delegator) {
								// Check if this delegator already has a freeze (already migrated)
								// Since we can't directly check if a specific freeze exists, we'll
								// try to remove the lock and apply the freeze

								// Remove the old lock
								T::Currency::remove_lock(DELEGATOR_LOCK_ID, delegator);

								// Set a freeze for the total delegated amount
								match T::Currency::set_freeze(
									&FreezeReason::StakingDelegator.into(),
									delegator,
									delegator_state.total,
								) {
									Ok(_) => {
										log::debug!(
											"Migrated top delegator {:?} with total {:?}",
											delegator,
											delegator_state.total
										);
									}
									Err(e) => {
										// Re-apply the lock on failure
										T::Currency::set_lock(
											DELEGATOR_LOCK_ID,
											delegator,
											delegator_state.total.into(),
											frame_support::traits::WithdrawReasons::all(),
										);
										log::error!(
											"Failed to set freeze for delegator {:?}: {:?}",
											delegator,
											e
										);
										return Err(frame_support::migrations::SteppedMigrationError::Failed);
									}
								}

								meter.consume(single_migration_weight);
								processed += 1;
							}
						}

						current_top_index += 1;
					}
				}

				// Process bottom delegators if we still have capacity
				if let Some(ref bottom) = bottom_delegations {
					let bottom_count = bottom.delegations.len() as u32;

					while current_bottom_index < bottom_count && processed < BATCH_SIZE {
						if !meter.can_consume(single_migration_weight) {
							// Return to continue processing
							return Ok(Some(MigrateLocksToFreezesState::ProcessingCandidate {
								candidate,
								top_index: current_top_index,
								bottom_index: current_bottom_index,
								migrated_candidates_count,
							}));
						}

						if let Some(delegation) =
							bottom.delegations.get(current_bottom_index as usize)
						{
							let delegator = &delegation.owner;

							// Get delegator state
							if let Some(delegator_state) = <DelegatorState<T>>::get(delegator) {
								// Remove the old lock
								T::Currency::remove_lock(DELEGATOR_LOCK_ID, delegator);

								// Set a freeze for the total delegated amount
								match T::Currency::set_freeze(
									&FreezeReason::StakingDelegator.into(),
									delegator,
									delegator_state.total,
								) {
									Ok(_) => {
										log::debug!(
											"Migrated bottom delegator {:?} with total {:?}",
											delegator,
											delegator_state.total
										);
									}
									Err(e) => {
										// Re-apply the lock on failure
										T::Currency::set_lock(
											DELEGATOR_LOCK_ID,
											delegator,
											delegator_state.total.into(),
											frame_support::traits::WithdrawReasons::all(),
										);
										log::error!(
											"Failed to set freeze for delegator {:?}: {:?}",
											delegator,
											e
										);
										return Err(frame_support::migrations::SteppedMigrationError::Failed);
									}
								}

								meter.consume(single_migration_weight);
								processed += 1;
							}
						}

						current_bottom_index += 1;
					}
				}

				// Check if we've processed all delegators
				let top_done = top_delegations
					.as_ref()
					.map_or(true, |t| current_top_index >= t.delegations.len() as u32);
				let bottom_done = bottom_delegations
					.as_ref()
					.map_or(true, |b| current_bottom_index >= b.delegations.len() as u32);

				if !top_done || !bottom_done {
					// Still have delegators to process, return current state
					return Ok(Some(MigrateLocksToFreezesState::ProcessingCandidate {
						candidate,
						top_index: current_top_index,
						bottom_index: current_bottom_index,
						migrated_candidates_count,
					}));
				}

				// All delegators for this candidate have been processed, now migrate the candidate
				if let Some(candidate_info) = <CandidateInfo<T>>::get(&candidate) {
					// Check if we have weight for candidate migration
					if !meter.can_consume(single_migration_weight) {
						// Return to continue with candidate migration in next step
						return Ok(Some(MigrateLocksToFreezesState::ProcessingCandidate {
							candidate,
							top_index: current_top_index,
							bottom_index: current_bottom_index,
							migrated_candidates_count,
						}));
					}

					// Remove the old lock
					T::Currency::remove_lock(COLLATOR_LOCK_ID, &candidate);

					// Set a freeze for the bonded amount
					match T::Currency::set_freeze(
						&FreezeReason::StakingCollator.into(),
						&candidate,
						candidate_info.bond,
					) {
						Ok(_) => {
							log::info!(
								"Migrated candidate {:?} with bond {:?}",
								candidate,
								candidate_info.bond
							);
						}
						Err(e) => {
							// Re-apply the lock on failure
							T::Currency::set_lock(
								COLLATOR_LOCK_ID,
								&candidate,
								candidate_info.bond.into(),
								frame_support::traits::WithdrawReasons::all(),
							);
							log::error!(
								"Failed to set freeze for candidate {:?}: {:?}",
								candidate,
								e
							);
							return Err(frame_support::migrations::SteppedMigrationError::Failed);
						}
					}

					meter.consume(single_migration_weight);
				}

				// Move to next candidate
				let new_count = migrated_candidates_count + 1;

				// Find next candidate after this one
				let mut found_current = false;
				let mut next_candidate = None;
				for (cand, _) in <CandidateInfo<T>>::iter() {
					if found_current {
						next_candidate = Some(cand);
						break;
					}
					if cand == candidate {
						found_current = true;
					}
				}

				match next_candidate {
					Some(next_cand) => {
						log::info!(
							"Moving to next candidate {:?}, total migrated: {}",
							next_cand,
							new_count
						);
						Ok(Some(MigrateLocksToFreezesState::ProcessingCandidate {
							candidate: next_cand,
							top_index: 0,
							bottom_index: 0,
							migrated_candidates_count: new_count,
						}))
					}
					None => {
						// All candidates processed
						log::info!(
							"Migration completed. Total candidates migrated: {}",
							new_count
						);
						Ok(Some(MigrateLocksToFreezesState::Finished {
							total_candidates_migrated: new_count,
						}))
					}
				}
			}
			MigrateLocksToFreezesState::Finished {
				total_candidates_migrated,
			} => {
				// Verify against expected count
				let expected_count = <CandidateInfo<T>>::iter().count() as u32;

				if total_candidates_migrated != expected_count {
					log::error!(
						"Migration count mismatch: migrated {} but found {} candidates in storage",
						total_candidates_migrated,
						expected_count
					);
				} else {
					log::info!(
						"Migration verification passed: {} candidates migrated successfully",
						total_candidates_migrated
					);
				}

				// Update storage version to v1
				StorageVersion::new(Self::id().version_to as u16).put::<crate::Pallet<T>>();

				Ok(None)
			}
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		use frame_support::ensure;

		// Decode the expected candidate count from pre-upgrade
		let expected_candidate_count: u32 = parity_scale_codec::Decode::decode(&mut &state[..])
			.map_err(|_| {
				sp_runtime::TryRuntimeError::Other("Failed to decode pre-upgrade state")
			})?;

		// Count current candidates in storage (should be unchanged)
		let current_candidate_count = <CandidateInfo<T>>::iter().count() as u32;

		// Verify the counts match
		ensure!(
			current_candidate_count == expected_candidate_count,
			sp_runtime::TryRuntimeError::Other(
				"Candidate count in storage changed during migration"
			)
		);

		log::info!(
			"Post-upgrade: Successfully verified {} candidates remain in storage",
			current_candidate_count
		);

		Ok(())
	}
}
