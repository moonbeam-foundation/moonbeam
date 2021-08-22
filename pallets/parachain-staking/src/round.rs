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

//! Constants and functions for computing round-related information
use crate::{Config, Pallet};

pub const SECONDS_PER_YEAR: u32 = 31557600;
pub const SECONDS_PER_BLOCK: u32 = 12;
pub const BLOCKS_PER_YEAR: u32 = SECONDS_PER_YEAR / SECONDS_PER_BLOCK;
/// Number of blocks prior to next round when election results are computed and committed
pub const EARLY_ELECTION_OFFSET: u32 = 1;

pub fn rounds_per_year<T: Config>() -> u32 {
	let blocks_per_round = <Pallet<T>>::round().length;
	BLOCKS_PER_YEAR / blocks_per_round
}
