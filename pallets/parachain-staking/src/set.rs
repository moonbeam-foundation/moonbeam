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

use frame_support::traits::Get;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{BoundedVec, RuntimeDebug};
use sp_std::prelude::*;

/// An ordered set backed by `Vec`
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, PartialEq, Eq, Encode, Decode, Default, Clone, TypeInfo)]
pub struct OrderedSet<T>(pub Vec<T>);

impl<T: Ord> OrderedSet<T> {
	/// Create a new empty set
	pub fn new() -> Self {
		Self(Vec::new())
	}

	/// Create a set from a `Vec`.
	/// `v` will be sorted and dedup first.
	pub fn from(mut v: Vec<T>) -> Self {
		v.sort();
		v.dedup();
		Self::from_sorted_set(v)
	}

	/// Create a set from a `Vec`.
	/// Assume `v` is sorted and contain unique elements.
	pub fn from_sorted_set(v: Vec<T>) -> Self {
		Self(v)
	}

	/// Insert an element.
	/// Return true if insertion happened.
	pub fn insert(&mut self, value: T) -> bool {
		match self.0.binary_search(&value) {
			Ok(_) => false,
			Err(loc) => {
				self.0.insert(loc, value);
				true
			}
		}
	}

	/// Remove an element.
	/// Return true if removal happened.
	pub fn remove(&mut self, value: &T) -> bool {
		match self.0.binary_search(value) {
			Ok(loc) => {
				self.0.remove(loc);
				true
			}
			Err(_) => false,
		}
	}

	/// Return if the set contains `value`
	pub fn contains(&self, value: &T) -> bool {
		self.0.binary_search(value).is_ok()
	}

	/// Clear the set
	pub fn clear(&mut self) {
		self.0.clear();
	}
}

impl<T: Ord> From<Vec<T>> for OrderedSet<T> {
	fn from(v: Vec<T>) -> Self {
		Self::from(v)
	}
}

/// An ordered set backed by `BoundedVec`
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(RuntimeDebug, PartialEq, Eq, Encode, Decode, Clone, TypeInfo)]
#[scale_info(skip_type_params(S))]
pub struct BoundedOrderedSet<T, S: Get<u32>>(pub BoundedVec<T, S>);

impl<T, S: Get<u32>> sp_std::default::Default for BoundedOrderedSet<T, S> {
	fn default() -> Self {
		BoundedOrderedSet(BoundedVec::default())
	}
}

impl<T: Ord, S: Get<u32>> BoundedOrderedSet<T, S> {
	/// Create a new empty set
	pub fn new() -> Self {
		Self(BoundedVec::default())
	}

	/// Create a set from a `Vec`.
	/// `v` will be sorted and dedup first.
	pub fn from(mut v: BoundedVec<T, S>) -> Self {
		v.sort();
		let v = v.try_mutate(|inner| inner.dedup()).expect(
			"input is a valid BoundedVec and deduping can only reduce the number of entires; qed",
		);
		Self::from_sorted_set(v)
	}

	/// Create a set from a `Vec`.
	/// Assume `v` is sorted and contain unique elements.
	pub fn from_sorted_set(v: BoundedVec<T, S>) -> Self {
		Self(v)
	}

	/// Insert an element.
	/// Return true if insertion happened.
	pub fn try_insert(&mut self, value: T) -> Result<bool, ()> {
		match self.0.binary_search(&value) {
			Ok(_) => Ok(false),
			Err(loc) => self.0.try_insert(loc, value).map(|_| true).map_err(|_| ()),
		}
	}

	/// Remove an element.
	/// Return true if removal happened.
	pub fn remove(&mut self, value: &T) -> bool {
		match self.0.binary_search(value) {
			Ok(loc) => {
				self.0.remove(loc);
				true
			}
			Err(_) => false,
		}
	}

	/// Return if the set contains `value`
	pub fn contains(&self, value: &T) -> bool {
		self.0.binary_search(value).is_ok()
	}

	/// Clear the set
	pub fn clear(mut self) {
		let v = self.0.try_mutate(|inner| inner.clear()).expect(
			"input is a valid BoundedVec and clearing can only reduce the number of entires; qed",
		);
		self.0 = v;
	}
}

impl<T: Ord, S: Get<u32>> From<BoundedVec<T, S>> for BoundedOrderedSet<T, S> {
	fn from(v: BoundedVec<T, S>) -> Self {
		Self::from(v)
	}
}
