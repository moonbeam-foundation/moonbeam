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
	BalanceOf, CandidateState, CollatorCandidate, CollatorState2, Config, DelegatorState,
	ExitQueue2, NominatorState2, Points, Round, Staked,
};
use frame_support::{
	pallet_prelude::PhantomData,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
};
use sp_std::collections::btree_map::BTreeMap;

/// Migration to replace the automatic ExitQueue with a manual exits API.
/// This migration is idempotent so it can be run more than once without any risk.
/// Returns (reads, writes)
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
		use frame_support::{storage::migration::storage_iter, traits::OnRuntimeUpgradeHelpersExt};

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
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;

		// Check number of candidates matches what was set aside in pre_upgrade
		let old_candidate_count: u64 = Self::get_temp_storage("old_collator_count")
			.expect("We stored the old collator candidate count so it should be there");
		let new_candidate_count = CandidateState::<T>::iter().count() as u64;
		assert_eq!(old_candidate_count, new_candidate_count);

		// Check that our example candidate is converted correctly
		if new_candidate_count > 0 {
			let (account, original_collator_state): (
				T::AuthorId,
				Collator2<T::AccountId, BalanceOf<T>>,
			) = Self::get_temp_storage("example_collator").expect("qed");
			let new_candidate_state = CandidateState::<T>::get(account).expect("qed");
			let old_candidate_converted: CollatorCandidate<_, _> = original_collator_state.into();
			assert_eq!(new_candidate_state, old_candidate_converted);
		}

		// Check number of delegators matches what was set aside in pre_upgrade
		let old_nominator_count: u64 = Self::get_temp_storage("old_nominator_count")
			.expect("We stored the old nominator count so it should be there");
		let new_delegator_count = DelegatorState::<T>::iter().count() as u64;
		assert_eq!(old_nominator_count, new_delegator_count);

		// Check that our example nominator is converted correctly
		if new_delegator_count > 0 {
			let (account, original_nominator_state): (
				T::AuthorId,
				Nominator2<T::AccountId, BalanceOf<T>>,
			) = Self::get_temp_storage("example_nominator").expect("qed");
			let new_candidate_state = DelegatorState::<T>::get(account).expect("qed");
			let old_candidate_converted: Delegator<_, _> = original_nominator_state.into();
			assert_eq!(old_candidate_converted, new_candidate_state);
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
