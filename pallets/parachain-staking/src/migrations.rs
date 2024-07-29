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

//use crate::{types::RoundInfo, Config, RoundIndex};
//use frame_support::pallet_prelude::*;
//use frame_support::traits::OnRuntimeUpgrade;
//use frame_system::pallet_prelude::*;

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

#[cfg(test)]
mod tests {
	use super::*;
	use sp_runtime::Saturating;

	use frame_support::traits::Saturating;
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

	#[test]
	fn test_compute_theoretical_first_slot() {
		assert_eq!(
			compute_theoretical_first_slot::<u32>(10, 5, 100, 12_000),
			90,
		);
	}
}
