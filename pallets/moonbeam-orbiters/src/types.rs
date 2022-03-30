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
// along with Moonbeam.  If not, see <http://www.gnu.org/licenses/>

use scale_info::TypeInfo;
use sp_runtime::{
	codec::{Decode, Encode},
	RuntimeDebug,
};
use sp_std::vec::Vec;

#[derive(Decode, Encode, RuntimeDebug, TypeInfo)]
pub struct CollatorPoolInfo<AccountId> {
	orbiters: Vec<AccountId>,
	maybe_current_orbiter: Option<AccountId>,
	next_orbiter: u32,
}

impl<AccountId> Default for CollatorPoolInfo<AccountId> {
	fn default() -> Self {
		Self {
			orbiters: Vec::new(),
			maybe_current_orbiter: None,
			next_orbiter: 0,
		}
	}
}

impl<AccountId: Clone + PartialEq> CollatorPoolInfo<AccountId> {
	pub fn add_orbiter(&mut self, orbiter: AccountId) {
		self.orbiters.insert(self.next_orbiter as usize, orbiter);
		self.next_orbiter += 1;
	}
	pub fn remove_orbiter(&mut self, orbiter: &AccountId) -> bool {
		for (index, orbiter_) in self.orbiters.iter().enumerate() {
			if orbiter_ == orbiter {
				self.orbiters.remove(index);
				return true;
			}
		}
		false
	}
	pub fn next_orbiter(&mut self) -> Option<AccountId> {
		if self.next_orbiter >= self.orbiters.len() as u32 {
			self.next_orbiter = 0;
		}
		if let Some(next_orbiter) = self.orbiters.get(self.next_orbiter as usize) {
			self.maybe_current_orbiter = Some(next_orbiter.clone());
			self.next_orbiter += 1;
			Some(next_orbiter.clone())
		} else {
			None
		}
	}
	pub fn get_current_orbiter(&self) -> &Option<AccountId> {
		&self.maybe_current_orbiter
	}
	pub fn get_orbiters(&self) -> &[AccountId] {
		&self.orbiters
	}
}

#[derive(Decode, Encode, RuntimeDebug, TypeInfo)]
pub struct RoundAuthors<AccountId> {
	data: Vec<(AccountId, u32)>,
	blocks_count: u32,
}

impl<AccountId> Default for RoundAuthors<AccountId> {
	fn default() -> Self {
		Self {
			data: Vec::new(),
			blocks_count: 0,
		}
	}
}

impl<AccountId: Ord> RoundAuthors<AccountId> {
	pub fn add_author(&mut self, author: AccountId) {
		match self
			.data
			.binary_search_by(|(account, _counter)| account.cmp(&author))
		{
			Ok(index) => self.data[index].1 = self.data[index].1.saturating_add(1),
			Err(index) => self.data.insert(index, (author, 1)),
		};
		self.blocks_count += 1;
	}
	pub fn into_data(self) -> (Vec<(AccountId, u32)>, u32) {
		let Self { data, blocks_count } = self;
		(data, blocks_count)
	}
}
