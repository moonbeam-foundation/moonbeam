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

//! # Parachain Staking Migrations
use crate::{pallet::*, Config, RoundIndex};
use frame_support::{
	traits::Get,
	weights::Weight,
};
use sp_runtime::Perbill;
use sp_std::prelude::*;

mod deprecated {
	use crate::{pallet::*, set::OrderedSet};
	use parity_scale_codec::{Decode, Encode};
	use sp_runtime::traits::Zero;

	#[derive(Encode, Decode)]
	/// DEPRECATED nominator state
	pub struct OldNominator<AccountId, Balance> {
		pub nominations: OrderedSet<Bond<AccountId, Balance>>,
		pub total: Balance,
	}

	impl<AccountId: Ord, Balance: Zero> From<OldNominator<AccountId, Balance>>
		for Nominator<AccountId, Balance>
	{
		fn from(other: OldNominator<AccountId, Balance>) -> Nominator<AccountId, Balance> {
			Nominator {
				nominations: other.nominations,
				revocations: OrderedSet::new(),
				total: other.total,
				scheduled_revocations_count: 0u32,
				scheduled_revocations_total: Zero::zero(),
				status: NominatorStatus::Active,
			}
		}
	}
}

/// Storage migration for delaying nomination exits and revocations
pub fn delay_nominator_exits_migration<T: Config>() -> (Perbill, Weight) {
	use frame_support::migration::{put_storage_value, take_storage_value, StorageIterator};

	let mut reads = 0u64;
	let mut writes = 0u64;

	// Migrate from old Nominator struct to our new one, which adds a few fields.

	reads += 1;
	for (key, old_nominator) in StorageIterator::<
		deprecated::OldNominator<T::AccountId, BalanceOf<T>>,
	>::new(b"ParachainStaking", b"NominatorState")
	.drain()
	{
		reads += 1;
		let new_nominator: Nominator<T::AccountId, BalanceOf<T>> = old_nominator.into();
		put_storage_value(b"ParachainStaking", b"NominatorState", &key, &new_nominator);
		writes += 1;
	}

	// Migrate from exit queue's Vec type to ExitQ

	// TODO: will this work for querying a standalone (non-map) storage item?
	reads += 1;
	if let Some(old_queue) = take_storage_value::<Vec<(T::AccountId, RoundIndex)>>(
		b"ParachainStaking",
		b"ExitQueue",
		b"",
	) {
		let mut candidates: Vec<T::AccountId> = Vec::new();
		for (acc, _) in old_queue.clone().into_iter() {
			candidates.push(acc);
		}

		let new_queue = ExitQ {
			candidates: candidates.into(),
			nominators_leaving: OrderedSet::new(),
			candidate_schedule: old_queue,
			nominator_schedule: Vec::new(),
		};
		put_storage_value(b"ParachainStaking", b"ExitQueue", b"", &new_queue);
		writes += 1;
	} else {
		// TODO (would we ever hit this case?)
	}

	let weight: Weight = 0u64
		.saturating_add( (reads as Weight).saturating_mul(T::DbWeight::get().reads(reads)))
		.saturating_add( (writes as Weight).saturating_mul(T::DbWeight::get().writes(writes)));

	(Perbill::one(), weight)
}
