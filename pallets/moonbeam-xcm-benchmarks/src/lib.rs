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

//! Pallet that serves no other purpose than benchmarking raw messages [`Xcm`].

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod generic;

#[cfg(test)]
mod mock;

// Only weights should be accesible, no need for the rest outside benchmarking
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
/// A base trait for all individual pallets
pub trait Config: frame_system::Config + pallet_xcm_benchmarks::Config {}
