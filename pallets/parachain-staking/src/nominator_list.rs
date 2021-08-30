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
pub struct CollatorNominators {
	// TODO: data structures here; i'm focusing on the exposed functionality for now
}

impl CollatorNominators {

	/// insert a nominator. needs to sort and do some accounting for totals.
	pub fn insert_nominator() {}

	/// remove a nominator. this is immediate; this struct doesn't concern itself with the need to
	/// delay an exit, etc.
	///
	/// after removal, same accounting as insert_nominator()
	pub fn remove_nominator() {}

	/// makes an adjustment (positive or negative) to some specific nominator's stake.
	pub fn adjust_nominator_stake() {}

	/// adjust_num_selected_nominators. this adjusts the cutoff for "top" nominators.
	///
	/// if separate top vs. bottom containers for nominators are required in the design, it would
	/// need to mutate those and ensure that they are sorted.
	///
	/// in any case, it needs to do some accounting to adjust the sum of top and bottom nominations.
	pub fn adjust_num_selected_nominators() {}

	/// accessors
	pub fn get_active_nominator_stake() {} // get sum of top N nominator stakes
	pub fn get_inactive_nominator_stake() {} // get sum of not top N nominator stakes
	pub fn get_total_nominator_stake() {} // sum of the two above
	pub fn get_num_nominators() {}

	/// reflect any modification to the state of this structure. this is a helper that can be called
	/// after any modification and should at any point result in proper sorting and accounting.
	///
	/// it is private and is expected to be called directly from functions such as
	/// insert_nominator(), adjust_nominator_stake(), etc.
	///
	/// TODO: this simple design may not allow for some optimizations (for example,
	/// adjust_nominator_stake() might know that it only reduced the lowest nominator and be able to
	/// avoid sorting, etc.)
	fn perform_sorting_and_accounting() {} // TODO: better name?
}

