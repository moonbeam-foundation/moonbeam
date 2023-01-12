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

#![allow(unused)]

use crate::delegation_requests::{DelegationAction, ScheduledRequest};
use crate::pallet::{DelegationScheduledRequests, DelegatorState, Total};
#[allow(deprecated)]
use crate::types::deprecated::{
	CollatorSnapshot as OldCollatorSnapshot, DelegationChange, Delegator as OldDelegator,
};
use crate::types::{CollatorSnapshot, Delegator};
use crate::{
	AtStake, BalanceOf, Bond, BondWithAutoCompound, BottomDelegations, CandidateInfo,
	CandidateMetadata, CapacityStatus, CollatorCandidate, Config, DelayedPayouts, Delegations,
	Event, Pallet, Points, Round, RoundIndex, Staked, TopDelegations,
};
use frame_support::Twox64Concat;
use parity_scale_codec::{Decode, Encode};
extern crate alloc;
#[cfg(feature = "try-runtime")]
use alloc::{format, string::ToString};
use frame_support::{
	migration::storage_key_iter,
	pallet_prelude::PhantomData,
	storage,
	traits::{Get, OnRuntimeUpgrade, ReservableCurrency},
	weights::Weight,
};
#[cfg(feature = "try-runtime")]
use scale_info::prelude::string::String;
use sp_runtime::traits::{Saturating, Zero};
use sp_runtime::Percent;
use sp_std::{convert::TryInto, vec, vec::Vec};

/// Migrate `AtStake` storage item to include auto-compound value for unpaid rounds.
pub struct MigrateAtStakeAutoCompound<T>(PhantomData<T>);
impl<T: Config> MigrateAtStakeAutoCompound<T> {
	/// Get keys for the `AtStake` storage for the rounds up to `RewardPaymentDelay` rounds ago.
	/// We migrate only the last unpaid rounds due to the presence of stale entries in `AtStake`
	/// which significantly increase the PoV size.
	fn unpaid_rounds_keys() -> impl Iterator<Item = (RoundIndex, T::AccountId, Vec<u8>)> {
		let current_round = <Round<T>>::get().current;
		let max_unpaid_round = current_round.saturating_sub(T::RewardPaymentDelay::get());
		(max_unpaid_round..=current_round)
			.into_iter()
			.flat_map(|round| {
				<AtStake<T>>::iter_key_prefix(round).map(move |candidate| {
					let key = <AtStake<T>>::hashed_key_for(round.clone(), candidate.clone());
					(round, candidate, key)
				})
			})
	}
}
impl<T: Config> OnRuntimeUpgrade for MigrateAtStakeAutoCompound<T> {
	#[allow(deprecated)]
	fn on_runtime_upgrade() -> Weight {
		log::info!(
			target: "MigrateAtStakeAutoCompound",
			"running migration to add auto-compound values"
		);
		let mut reads = 0u64;
		let mut writes = 0u64;
		for (round, candidate, key) in Self::unpaid_rounds_keys() {
			let old_state: OldCollatorSnapshot<T::AccountId, BalanceOf<T>> =
				storage::unhashed::get(&key).expect("unable to decode value");
			reads = reads.saturating_add(1);
			writes = writes.saturating_add(1);
			log::info!(
				target: "MigrateAtStakeAutoCompound",
				"migration from old format round {:?}, candidate {:?}", round, candidate
			);
			let new_state = CollatorSnapshot {
				bond: old_state.bond,
				delegations: old_state
					.delegations
					.into_iter()
					.map(|d| BondWithAutoCompound {
						owner: d.owner,
						amount: d.amount,
						auto_compound: Percent::zero(),
					})
					.collect(),
				total: old_state.total,
			};
			storage::unhashed::put(&key, &new_state);
		}

		T::DbWeight::get().reads_writes(reads, writes)
	}

	#[allow(deprecated)]
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		let mut num_to_update = 0u32;
		let mut rounds_candidates: Vec<(RoundIndex, T::AccountId)> = vec![];
		use sp_std::collections::btree_map::BTreeMap;
		let mut state_map: BTreeMap<String, String> = BTreeMap::new();

		for (round, candidate, key) in Self::unpaid_rounds_keys() {
			let state: OldCollatorSnapshot<T::AccountId, BalanceOf<T>> =
				storage::unhashed::get(&key).expect("unable to decode value");

			num_to_update = num_to_update.saturating_add(1);
			rounds_candidates.push((round.clone(), candidate.clone()));
			let mut delegation_str = vec![];
			for d in state.delegations {
				delegation_str.push(format!(
					"owner={:?}_amount={:?}_autoCompound=0%",
					d.owner, d.amount
				));
			}
			state_map.insert(
				(&*format!("round_{:?}_candidate_{:?}", round, candidate)).to_string(),
				format!(
					"bond={:?}_total={:?}_delegations={:?}",
					state.bond, state.total, delegation_str
				),
			);
		}

		rounds_candidates.sort();
		Ok((state_map, rounds_candidates, num_to_update).encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		use sp_std::collections::btree_map::BTreeMap;

		let (state_map, rounds_candidates_received, num_updated_received): (
			BTreeMap<String, String>,
			Vec<(RoundIndex, T::AccountId)>,
			u32,
		) = Decode::decode(&mut &state[..]).expect("pre_upgrade provides a valid state; qed");

		let mut num_updated = 0u32;
		let mut rounds_candidates = vec![];
		for (round, candidate, _) in Self::unpaid_rounds_keys() {
			let state = <AtStake<T>>::get(&round, &candidate);
			num_updated = num_updated.saturating_add(1);
			rounds_candidates.push((round.clone(), candidate.clone()));
			let mut delegation_str = vec![];
			for d in state.delegations {
				delegation_str.push(format!(
					"owner={:?}_amount={:?}_autoCompound={:?}",
					d.owner, d.amount, d.auto_compound
				));
			}
			assert_eq!(
				Some(&format!(
					"bond={:?}_total={:?}_delegations={:?}",
					state.bond, state.total, delegation_str
				)),
				state_map
					.get(&((&*format!("round_{:?}_candidate_{:?}", round, candidate)).to_string())),
				"incorrect delegations migration for round_{:?}_candidate_{:?}",
				round,
				candidate,
			);
		}

		rounds_candidates.sort();
		assert_eq!(rounds_candidates, rounds_candidates_received);
		assert_eq!(num_updated, num_updated_received);
		Ok(())
	}
}
