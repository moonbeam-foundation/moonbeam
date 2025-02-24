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

pub use pallet::*;

pub mod benchmarking;

#[cfg(test)]
pub mod mock;

#[frame_support::pallet]
pub mod pallet {
	#[pallet::config]
	pub trait Config<I: 'static = ()>:
		frame_system::Config + crate::Config + pallet_xcm_benchmarks::generic::Config
	{
	}

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(_);
}
