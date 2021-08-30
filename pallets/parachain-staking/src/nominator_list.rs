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

#![cfg_attr(not(feature = "std"), no_std)]

use crate::*;
use sp_runtime::{
	traits::AtLeast32BitUnsigned,
};
use sp_std::prelude::*;

/// An in-memory struct representing a collator's list of nominators. This list is sorted by amount
/// of stake upon every iteration and the top N nominators are trivially available at any time. In
/// addition, several sums are maintained (also updated upon any mutation):
///
/// 1) the total sum of all nominators
/// 2) the sum of the top N nominators
/// 3) the sum of the nominators outside of the top N (the "bottom" nominators)
///
/// Design note: I find it to be a useful exercise to fully abstract away specifics. This first pass
/// doesn't reflect this, but consider what this struct and its functions might look like if they
/// had nothing to do with collators or nominators.
pub struct CollatorNominators<AccountId, Balance>  {
	/// Sorted list of all nominators for this collator
	pub nominators: Vec<AccountId>,
	/// All bonds, sorted by bonded amount
	pub bonds: Vec<Bond<AccountId, Balance>>,
	/// Maximum number of nominators that will be selected
	pub max_selected: u32,
	/// Amount of contribution from nominators that make the cut
	pub contribution: Balance,
	/// Total bond of all nominators (including those that don't make the cut)
	pub total_bond_amount: Balance,
}

impl<
	AccountId: Ord + Clone,
	Balance: AtLeast32BitUnsigned + Ord + Copy + sp_std::ops::AddAssign + sp_std::ops::SubAssign,
> CollatorNominators<AccountId, Balance>
{
	/// Insert a nominator. The nominator must not previously exist or will return an error.
	pub fn insert_nominator(&mut self, account: &AccountId, bond: &Balance) -> Result<NominatorAdded<Balance>, ()> {
		match self.nominators.binary_search(account) {
			Ok(_) => Err(()),
			Err(index) => {
				self.nominators.insert(index, account.clone());
				Ok(())
			}
		}?;

		let index = self.find_bond_insertion_index(&bond);
		let selected = self.is_selected(index);

		// TODO: a helper (like is_selected()) would be nice if this pattern is consistent
		// recalculate bond tallies
		self.total_bond_amount.saturating_add(bond.clone());
		if selected {
			self.contribution.saturating_add(bond.clone());
			// TODO: test carefully
			if self.nominators.len() > self.max_selected as usize {
				// we know we bumped someone out of the top here, so reduce by that amount
				assert!(index + 1 < self.nominators.len(), "indexing logic error");
				self.contribution.saturating_sub(self.bonds[index + 1].amount);
			}
		}

		self.bonds.insert(index, Bond { owner: account.clone(), amount: bond.clone() });

		if selected {
			// TODO: does this match the expected value for AddedToTop?
			Ok(NominatorAdded::AddedToTop { new_total: self.contribution, })
		} else {
			Ok(NominatorAdded::AddedToBottom)
		}
	}

	/// remove a nominator. this is immediate; this struct doesn't concern itself with the need to
	/// delay an exit, etc.
	///
	/// after removal, same accounting as insert_nominator()
	pub fn remove_nominator(&mut self) {}

	/// makes an adjustment (positive or negative) to some specific nominator's stake.
	pub fn adjust_nominator_stake(&mut self) {}

	/// adjust_num_selected_nominators. this adjusts the cutoff for "top" nominators.
	///
	/// if separate top vs. bottom containers for nominators are required in the design, it would
	/// need to mutate those and ensure that they are sorted.
	///
	/// in any case, it needs to do some accounting to adjust the sum of top and bottom nominations.
	pub fn adjust_num_selected_nominators(&mut self) {}

	/// accessors
	pub fn get_active_nominator_stake(&self) {} // get sum of top N nominator stakes
	pub fn get_inactive_nominator_stake(&self) {} // get sum of not top N nominator stakes
	pub fn get_total_nominator_stake(&self) {} // sum of the two above
	pub fn get_num_nominators(&self) {}

	/// reflect any modification to the state of this structure. this is a helper that can be called
	/// after any modification and should at any point result in proper sorting and accounting.
	///
	/// it is private and is expected to be called directly from functions such as
	/// insert_nominator(), adjust_nominator_stake(), etc.
	///
	/// TODO: this simple design may not allow for some optimizations (for example,
	/// adjust_nominator_stake() might know that it only reduced the lowest nominator and be able to
	/// avoid sorting, etc.)
	fn perform_sorting_and_accounting(&mut self) {} // TODO: better name?

	// utility functions (to help readability, reduce repetitiveness, or allow for testable code)

	/// Returns whether or not an item with the given index would appear in the selected group. Avoids
	/// repetitive logic which might appear to have off-by-one problems.
	#[inline]
	fn is_selected(&self, index: usize) -> bool {
		index < self.max_selected as usize
	}

	/// Uses binary search to locate the appropriate insertion index of a given bond amount.
	#[inline]
	fn find_bond_insertion_index(&self, amount: &Balance) -> usize {
		match self.bonds.binary_search_by(|bond| amount.cmp(&bond.amount)) {
			Ok(index) => {
				// upon duplicates, this may return any matching index. we want to find the next
				// non-matching index; this is where we would insert a new entry.
				// TODO: unit test this thoroughly
				let mut index = index;
				while *amount == self.bonds[index].amount && index < self.bonds.len() {
					index += 1;
				}
				// TODO: increment if at the end?
				index
			},
			Err(index) => index
		}
	}
}
