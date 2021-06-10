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

pub mod precompiles;
pub use moonbeam_core_primitives::Balance;

/// UNITS, the native token, uses 18 decimals of precision.
pub mod currency {
	use super::Balance;

	pub const UNITS: Balance = 1_000_000_000_000_000_000;
	pub const MILLIUNITS: Balance = UNITS / 1000;
	pub const MICROUNITS: Balance = MILLIUNITS / 1000;
	pub const NANOUNITS: Balance = MICROUNITS / 1000;

	pub const KILOUNITS: Balance = UNITS * 1_000;

	pub const BYTE_FEE: Balance = 100 * MICROUNITS;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 1 * UNITS + (bytes as Balance) * BYTE_FEE
	}
}
