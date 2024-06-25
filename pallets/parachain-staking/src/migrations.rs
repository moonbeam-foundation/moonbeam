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

use crate::{types::RoundInfo, Config, RoundIndex};
use frame_support::pallet_prelude::*;
use frame_support::storage::generator::StorageValue;
use frame_support::storage::unhashed;
use frame_support::traits::OnRuntimeUpgrade;
use frame_system::pallet_prelude::*;
use sp_runtime::Saturating;

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

/// Multiply round length by 2
pub struct MultiplyRoundLenBy2<T: Config>(core::marker::PhantomData<T>);

impl<T> OnRuntimeUpgrade for MultiplyRoundLenBy2<T>
where
	T: Config,
	BlockNumberFor<T>: From<u32> + Into<u64>,
{
	fn on_runtime_upgrade() -> frame_support::pallet_prelude::Weight {
		let mut round = crate::Round::<T>::get();

		// Multiply round length by 2
		round.length = round.length * 2;

		crate::Round::<T>::put(round);

		Default::default()
	}
}

/// Compute RoundInfo theoretical first_slot
pub struct ComputeTheoreticalFirstSlot<T: Config>(core::marker::PhantomData<T>);

impl<T> OnRuntimeUpgrade for ComputeTheoreticalFirstSlot<T>
where
	T: Config,
	BlockNumberFor<T>: From<u32> + Into<u64>,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
		Ok(Vec::new())
	}

	fn on_runtime_upgrade() -> frame_support::pallet_prelude::Weight {
		// Read round
		let mut round = crate::Round::<T>::get();

		// Compute theoretical `first_slot``
		round.first_slot = compute_theoretical_first_slot(
			<frame_system::Pallet<T>>::block_number(),
			round.first,
			u64::from(T::SlotProvider::get()),
			T::BlockTime::get(),
		);

		// Apply the migration (write new Round value)
		crate::Round::<T>::put(round);

		Default::default()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		Ok(())
	}
}

/// Migrates RoundInfo and add the field first_slot
pub struct MigrateRoundWithFirstSlot<T: Config>(core::marker::PhantomData<T>);

#[derive(Decode)]
struct RoundInfoRt2800 {
	/// Current round index
	pub current: RoundIndex,
	/// The first block of the current round
	pub first: u64,
	/// The length of the current round in number of blocks
	pub length: u32,
}
impl<BlockNumber: From<u32>> From<RoundInfoRt2800> for RoundInfo<BlockNumber> {
	fn from(round: RoundInfoRt2800) -> Self {
		Self {
			current: round.current,
			first: (round.first as u32).into(),
			length: round.length,
			first_slot: 0,
		}
	}
}

#[derive(Decode)]
struct RoundInfoRt2700 {
	/// Current round index
	pub current: RoundIndex,
	/// The first block of the current round
	pub first: u32,
	/// The length of the current round in number of blocks
	pub length: u32,
}
impl<BlockNumber: From<u32>> From<RoundInfoRt2700> for RoundInfo<BlockNumber> {
	fn from(round: RoundInfoRt2700) -> Self {
		Self {
			current: round.current,
			first: round.first.into(),
			length: round.length,
			first_slot: 0,
		}
	}
}

impl<T> OnRuntimeUpgrade for MigrateRoundWithFirstSlot<T>
where
	T: Config,
	BlockNumberFor<T>: From<u32> + Into<u64>,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
		let raw_key = crate::Round::<T>::storage_value_final_key();
		let maybe_raw_value = unhashed::get_raw(&raw_key);
		let len = maybe_raw_value
			.expect("ParachainStaking.Round should exist!")
			.len();
		ensure!(
			len == 12 || len == 16,
			"ParachainStaking.Round should have 12 or 16 bytes length!"
		);

		Ok(Vec::new())
	}

	fn on_runtime_upgrade() -> frame_support::pallet_prelude::Weight {
		let raw_key = crate::Round::<T>::storage_value_final_key();

		// Read old round info
		let mut round: RoundInfo<BlockNumberFor<T>> = if let Some(bytes) =
			unhashed::get_raw(&raw_key)
		{
			let len = bytes.len();
			match len {
				// Migration already done
				20 => {
					log::info!("MigrateRoundWithFirstSlot already applied.");
					return Default::default();
				}
				// Migrate from rt2800
				16 => match RoundInfoRt2800::decode(&mut &bytes[..]) {
					Ok(round) => round.into(),
					Err(e) => panic!("corrupted storage: fail to decode RoundInfoRt2800: {}", e),
				},
				// Migrate from rt2700
				12 => match RoundInfoRt2700::decode(&mut &bytes[..]) {
					Ok(round) => round.into(),
					Err(e) => panic!("corrupted storage: fail to decode RoundInfoRt2700: {}", e),
				},
				// Storage corrupted
				x => panic!(
					"corrupted storage: parachainStaking.Round invalid length: {} bytes",
					x
				),
			}
		} else {
			panic!("corrupted storage: parachainStaking.Round don't exist");
		};

		// Compute new field `first_slot``
		round.first_slot = compute_theoretical_first_slot(
			<frame_system::Pallet<T>>::block_number(),
			round.first,
			u64::from(T::SlotProvider::get()),
			T::BlockTime::get(),
		);

		// Fill DelayedPayouts for rounds N and N-1
		if let Some(delayed_payout) =
			<crate::DelayedPayouts<T>>::get(round.current.saturating_sub(2))
		{
			<crate::DelayedPayouts<T>>::insert(
				round.current.saturating_sub(1),
				delayed_payout.clone(),
			);
			<crate::DelayedPayouts<T>>::insert(round.current, delayed_payout);
		}

		// Apply the migration (write new Round value)
		crate::Round::<T>::put(round);

		Default::default()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		let _round = crate::Round::<T>::get(); // Should panic if SCALE decode fail
		Ok(())
	}
}

fn compute_theoretical_first_slot<BlockNumber: Saturating + Into<u64>>(
	current_block: BlockNumber,
	first_block: BlockNumber,
	current_slot: u64,
	block_time: u64,
) -> u64 {
	let blocks_since_first: u64 = (current_block.saturating_sub(first_block)).into();
	let slots_since_first = match block_time {
		12_000 => blocks_since_first * 2,
		6_000 => blocks_since_first,
		_ => panic!("Unsupported BlockTime"),
	};
	current_slot.saturating_sub(slots_since_first)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_compute_theoretical_first_slot() {
		assert_eq!(
			compute_theoretical_first_slot::<u32>(10, 5, 100, 12_000),
			90,
		);
	}
}
