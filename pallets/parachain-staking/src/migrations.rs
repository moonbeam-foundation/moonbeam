// Copyright 2019-2021 PureStake Inc.
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
use crate::{
	pallet::{migrate_nominator_to_delegator_state, RoundIndex},
	BalanceOf, Bond, CandidateState, CollatorCandidate, CollatorState2, Config, DelegatorState,
	ExitQueue2, NominatorState2, Pallet, Points, Round, Staked,
};
#[cfg(feature = "try-runtime")]
use crate::{Collator2, Delegator, Nominator2};
#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;
#[cfg(feature = "try-runtime")]
use frame_support::Twox64Concat;
extern crate alloc;
#[cfg(feature = "try-runtime")]
use alloc::format;
use frame_support::{
	pallet_prelude::PhantomData,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

/// Migration to properly increase maximum delegations per collator
/// This migration can be used to recompute the top and bottom delegations whenever
/// MaxDelegatorsPerCandidate changes (works for decrease as well)
pub struct IncreaseMaxDelegationsPerCandidate<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for IncreaseMaxDelegationsPerCandidate<T> {
	fn on_runtime_upgrade() -> Weight {
		let (mut reads, mut writes) = (0u64, 0u64);
		for (account, state) in <CandidateState<T>>::iter() {
			reads += 1u64;
			// 1. collect all delegations into single vec and order them
			let mut all_delegations = state.top_delegations.clone();
			let mut starting_bottom_delegations = state.bottom_delegations.clone();
			all_delegations.append(&mut starting_bottom_delegations);
			// sort all delegations from greatest to least
			all_delegations.sort_unstable_by(|a, b| b.amount.cmp(&a.amount));
			let top_n = T::MaxDelegatorsPerCandidate::get() as usize;
			// 2. split them into top and bottom using the T::MaxNominatorsPerCollator
			let top_delegations: Vec<Bond<T::AccountId, BalanceOf<T>>> =
				all_delegations.iter().take(top_n).cloned().collect();
			let bottom_delegations = if all_delegations.len() > top_n {
				let rest = all_delegations.len() - top_n;
				let bottom: Vec<Bond<T::AccountId, BalanceOf<T>>> =
					all_delegations.iter().rev().take(rest).cloned().collect();
				bottom
			} else {
				// empty, all nominations are in top
				Vec::new()
			};
			let (mut total_counted, mut total_backing): (BalanceOf<T>, BalanceOf<T>) =
				(state.bond.into(), state.bond.into());
			for Bond { amount, .. } in &top_delegations {
				total_counted += *amount;
				total_backing += *amount;
			}
			for Bond { amount, .. } in &bottom_delegations {
				total_backing += *amount;
			}
			// update candidate pool with new total counted if it changed
			if state.total_counted != total_counted && state.is_active() {
				reads += 1u64;
				writes += 1u64;
				<Pallet<T>>::update_active(account.clone(), total_counted);
			}
			<CandidateState<T>>::insert(
				account,
				CollatorCandidate {
					top_delegations,
					bottom_delegations,
					total_counted,
					total_backing,
					..state
				},
			);
			writes += 1u64;
		}
		let weight = T::DbWeight::get();
		// 20% of the max block weight as safety margin for computation
		weight.reads(reads) + weight.writes(writes) + 100_000_000_000
	}
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		// get delegation count for all candidates to check consistency
		for (account, state) in <CandidateState<T>>::iter() {
			// insert top + bottom into some temp map?
			let total_delegation_count =
				state.top_delegations.len() as u32 + state.bottom_delegations.len() as u32;
			Self::set_temp_storage(
				total_delegation_count,
				&format!("Candidate{}DelegationCount", account)[..],
			);
		}
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		// check that top + bottom are the same as the expected (stored in temp)
		for (account, state) in <CandidateState<T>>::iter() {
			let expected_count: u32 =
				Self::get_temp_storage(&format!("Candidate{}DelegationCount", account)[..])
					.expect("qed");
			let actual_count =
				state.top_delegations.len() as u32 + state.bottom_delegations.len() as u32;
			assert_eq!(expected_count, actual_count);
		}
		Ok(())
	}
}

/// Migration to replace the automatic ExitQueue with a manual exits API.
/// This migration is idempotent so it can be run more than once without any risk.
pub struct RemoveExitQueue<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for RemoveExitQueue<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "RemoveExitQueue", "running migration to remove staking exit queue");
		let exit_queue = <ExitQueue2<T>>::take();
		let (mut reads, mut writes) = (1u64, 0u64);
		let mut delegator_exits: BTreeMap<T::AccountId, RoundIndex> = BTreeMap::new();
		let mut delegation_revocations: BTreeMap<T::AccountId, (T::AccountId, RoundIndex)> =
			BTreeMap::new();
		// Track scheduled delegator exits and revocations before migrating state
		// Candidates already track exit info locally so no tracking is necessary
		for (delegator, is_revocation, when) in exit_queue.nominator_schedule {
			if let Some(revoking_candidate) = is_revocation {
				delegation_revocations.insert(delegator, (revoking_candidate, when));
			} else {
				delegator_exits.insert(delegator, when);
			}
		}
		// execute candidate migration
		for (candidate_id, collator_state) in <CollatorState2<T>>::drain() {
			let candidate_state: CollatorCandidate<T::AccountId, BalanceOf<T>> =
				collator_state.into();
			<CandidateState<T>>::insert(candidate_id, candidate_state);
			reads += 1u64;
			writes += 1u64;
		}
		// execute delegator migration
		for (delegator_id, nominator_state) in <NominatorState2<T>>::drain() {
			let mut delegator_state =
				migrate_nominator_to_delegator_state::<T>(delegator_id.clone(), nominator_state);
			// add exit if it exists
			if let Some(when) = delegator_exits.get(&delegator_id) {
				delegator_state.set_leaving(*when);
			}
			// add revocation if exists
			if let Some((candidate, when)) = delegation_revocations.get(&delegator_id) {
				delegator_state.hotfix_set_revoke::<T>(candidate.clone(), *when);
			}
			<DelegatorState<T>>::insert(delegator_id, delegator_state);
			reads += 1u64;
			writes += 1u64;
		}
		let db_weight = T::DbWeight::get();
		if reads > 1u64 {
			// 50% of the max block weight as safety margin for computation
			db_weight.reads(reads) + db_weight.writes(writes) + 250_000_000_000
		} else {
			// migration was already executed before
			db_weight.reads(reads)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		use frame_support::storage::migration::{storage_iter, storage_key_iter};

		let pallet_prefix: &[u8] = b"ParachainStaking";
		let collator_state_prefix: &[u8] = b"CollatorState2";
		let nominator_state_prefix: &[u8] = b"NominatorState2";

		// Assert new storage is empty
		assert!(CandidateState::<T>::iter().next().is_none());
		assert!(DelegatorState::<T>::iter().next().is_none());

		// Check number of old collator candidates, and set it aside in temp storage
		let old_collator_count = storage_iter::<Collator2<T::AccountId, BalanceOf<T>>>(
			pallet_prefix,
			collator_state_prefix,
		)
		.count() as u64;
		Self::set_temp_storage(old_collator_count, "old_collator_count");

		// Read first old candidate from old storage and set it aside in temp storage
		if old_collator_count > 0 {
			let example_collator = storage_key_iter::<
				T::AccountId,
				Collator2<T::AccountId, BalanceOf<T>>,
				Twox64Concat,
			>(pallet_prefix, collator_state_prefix)
			.next()
			.expect("We already confirmed that there was at least one item stored");

			Self::set_temp_storage(example_collator, "example_collator");
		}

		// Check number of new delegators, and set it aside in temp storage
		let old_nominator_count = storage_iter::<Nominator2<T::AccountId, BalanceOf<T>>>(
			pallet_prefix,
			nominator_state_prefix,
		)
		.count() as u64;
		Self::set_temp_storage(old_nominator_count, "old_nominator_count");

		// Read first new delegator from old storage and set it aside in temp storage
		if old_nominator_count > 0 {
			let example_nominator = storage_key_iter::<
				T::AccountId,
				Nominator2<T::AccountId, BalanceOf<T>>,
				Twox64Concat,
			>(pallet_prefix, nominator_state_prefix)
			.next()
			.expect("We already confirmed that there was at least one item stored");

			Self::set_temp_storage(example_nominator, "example_nominator");
		}
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		// Check number of candidates matches what was set aside in pre_upgrade
		let old_candidate_count: u64 = Self::get_temp_storage("old_collator_count")
			.expect("We stored the old collator candidate count so it should be there");
		let new_candidate_count = CandidateState::<T>::iter().count() as u64;
		assert_eq!(old_candidate_count, new_candidate_count);

		// Check that our example candidate is converted correctly
		if new_candidate_count > 0 {
			let (account, original_collator_state): (
				T::AccountId,
				Collator2<T::AccountId, BalanceOf<T>>,
			) = Self::get_temp_storage("example_collator").expect("qed");
			let new_candidate_state = CandidateState::<T>::get(account).expect("qed");
			let old_candidate_converted: CollatorCandidate<T::AccountId, BalanceOf<T>> =
				original_collator_state.into();
			assert_eq!(new_candidate_state, old_candidate_converted);
		}

		// Check number of delegators matches what was set aside in pre_upgrade
		let old_nominator_count: u64 = Self::get_temp_storage("old_nominator_count")
			.expect("We stored the old nominator count so it should be there");
		let new_delegator_count = DelegatorState::<T>::iter().count() as u64;
		assert_eq!(old_nominator_count, new_delegator_count);

		// Check that our example delegator is converted correctly
		if new_delegator_count > 0 {
			let (account, original_delegator_state): (
				T::AccountId,
				Nominator2<T::AccountId, BalanceOf<T>>,
			) = Self::get_temp_storage("example_nominator").expect("qed");
			let new_delegator_state = DelegatorState::<T>::get(&account).expect("qed");
			let old_delegator_converted: Delegator<T::AccountId, BalanceOf<T>> =
				migrate_nominator_to_delegator_state::<T>(account, original_delegator_state);
			assert_eq!(old_delegator_converted, new_delegator_state);
		}
		Ok(())
	}
}

/// Migration to purge staking storage bloat for `Points` and `AtStake` storage items
pub struct PurgeStaleStorage<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for PurgeStaleStorage<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "PurgeStaleStorage", "running migration to remove storage bloat");
		let current_round = <Round<T>>::get().current;
		let payment_delay = T::RewardPaymentDelay::get();
		let db_weight = T::DbWeight::get();
		let (reads, mut writes) = (3u64, 0u64);
		if current_round <= payment_delay {
			// early enough so no storage bloat exists yet
			// (only relevant for chains <= payment_delay rounds old)
			return db_weight.reads(reads);
		}
		// already paid out at the beginning of current round
		let most_recent_round_to_kill = current_round - payment_delay;
		for i in 1..=most_recent_round_to_kill {
			writes += 2u64;
			<Staked<T>>::remove(i);
			<Points<T>>::remove(i);
		}
		// 5% of the max block weight as safety margin for computation
		db_weight.reads(reads) + db_weight.writes(writes) + 25_000_000_000
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		// trivial migration
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		// expect only the storage items for the last 2 rounds to be stored
		let staked_count = Staked::<T>::iter().count() as u32;
		let points_count = Points::<T>::iter().count() as u32;
		let delay = T::RewardPaymentDelay::get();
		assert_eq!(
			staked_count, delay,
			"Expected {} for `Staked` count, Found: {}",
			delay, staked_count
		);
		assert_eq!(
			points_count, delay,
			"Expected {} for `Points` count, Found: {}",
			delay, staked_count
		);
		Ok(())
	}
}
