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

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use {
	super::*, core::cmp::Ordering, frame_support::RuntimeDebug, sp_runtime::traits::CheckedAdd,
	sp_std::collections::btree_set::BTreeSet,
};

/// Candidate info stored in a sorted list.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, PartialEq, Eq, Encode, Decode, Clone, TypeInfo)]
pub struct Candidate<C, S> {
	candidate: C,
	stake: S,
}

impl<C: Ord, S: Ord> Ord for Candidate<C, S> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.stake
			.cmp(&other.stake)
			.reverse()
			.then(self.candidate.cmp(&self.candidate))
	}
}

impl<C: Ord, S: Ord> PartialOrd for Candidate<C, S> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

pub fn update_candidate_stake<T: Config>(
	candidate: T::AccountId,
	new_stake: BalanceOf<T>,
) -> Result<(), Error<T>> {
	let before_stake = CandidatesStake::<T>::get(&candidate);
	CandidatesStake::<T>::insert(&candidate, new_stake);

	let ac_self_delegation = pools::auto_compounding::shares_to_stake(
		&candidate,
		AutoCompoundingShares::<T>::get(&candidate, &candidate),
	)?;

	let mc_self_delegation = pools::manual_claim::shares_to_stake(
		&candidate,
		&ManualClaimShares::<T>::get(&candidate, &candidate),
	)?;

	let self_delegation = ac_self_delegation
		.checked_add(&mc_self_delegation)
		.ok_or(Error::MathOverflow)?;

	SortedEligibleCandidates::<T>::mutate(|list| {
		// Remove old data if it existed.
		let old_position = match list.binary_search(&Candidate {
			candidate: candidate.clone(),
			stake: before_stake,
		}) {
			Ok(pos) => {
				let _ = list.remove(pos);
				Some(pos as u32)
			}
			Err(_) => None,
		};

		let new_position = if self_delegation >= T::MinimumSelfDelegation::get() {
			// Insert candidate in the sorted list.
			let entry = Candidate {
				candidate: candidate.clone(),
				stake: new_stake,
			};

			let pos = list
				.binary_search(&entry)
				.expect_err("Candidate should be present at most once in the list.");
			list.insert(pos, entry);
			Some(pos as u32)
		} else {
			None
		};

		// If candidate was or is now in the top we need to update
		// the collator set.
		let set_size = MaxCollatorSetSize::<T>::get();
		match (old_position, new_position) {
			(Some(pos), _) | (_, Some(pos)) if pos < set_size => {
				let set: BTreeSet<_> = list
					.iter()
					.take(set_size as usize)
					.map(|c| c.candidate.clone())
					.collect();
				CollatorSet::<T>::put(set);
			}
			_ => (),
		}

		Pallet::<T>::deposit_event(Event::<T>::UpdatedCandidatePosition {
			candidate,
			stake: new_stake,
			self_delegation,
			before: old_position,
			after: new_position,
		});
	});

	Ok(())
}

pub fn add_stake<T: Config>(candidate: T::AccountId, stake: BalanceOf<T>) -> Result<(), Error<T>> {
	ensure!(!Zero::is_zero(&stake), Error::StakeMustBeNonZero);

	let new_stake = CandidatesStake::<T>::get(&candidate)
		.checked_add(&stake)
		.ok_or(Error::MathOverflow)?;

	let new_total_staked = CandidatesTotalStaked::<T>::get()
		.checked_add(&stake)
		.ok_or(Error::MathOverflow)?;

	update_candidate_stake::<T>(candidate.clone(), new_stake)?;
	CandidatesTotalStaked::<T>::set(new_total_staked);

	Pallet::<T>::deposit_event(Event::<T>::IncreasedStake { candidate, stake });

	Ok(())
}

pub fn sub_stake<T: Config>(candidate: T::AccountId, stake: BalanceOf<T>) -> Result<(), Error<T>> {
	ensure!(!Zero::is_zero(&stake), Error::StakeMustBeNonZero);

	let new_stake = CandidatesStake::<T>::get(&candidate)
		.checked_sub(&stake)
		.ok_or(Error::MathUnderflow)?;

	let new_total_staked = CandidatesTotalStaked::<T>::get()
		.checked_sub(&stake)
		.ok_or(Error::MathUnderflow)?;

	update_candidate_stake::<T>(candidate.clone(), new_stake)?;
	CandidatesTotalStaked::<T>::set(new_total_staked);

	Pallet::<T>::deposit_event(Event::<T>::DecreasedStake { candidate, stake });

	Ok(())
}

pub fn stake<T: Config>(candidate: &T::AccountId) -> BalanceOf<T> {
	CandidatesStake::<T>::get(candidate)
}
