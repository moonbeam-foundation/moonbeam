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
use crate::{Config, Points, Round, Staked};
use frame_support::{
	pallet_prelude::PhantomData,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
};

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
			// early enough so no storage bloat exists yet (only relevant for chains <= payment_delay rounds old)
			return db_weight.reads(reads);
		}
		// already paid out at the beginning of current round
		let first_round_to_kill = current_round - payment_delay;
		for i in 1..=first_round_to_kill {
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
		// trivial migration
		Ok(())
	}
}
