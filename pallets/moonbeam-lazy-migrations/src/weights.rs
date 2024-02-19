// Copyright 2024 Moonbeam foundation
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

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_moonbeam_lazy_migrations.
pub trait WeightInfo {
	fn clear_suicided_storage(a: u32, l: u32, ) -> Weight;
}

/// Weights for pallet_moonbeam_lazy_migrations using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `EVM::AccountCodes` (r:1000 w:0)
	/// Proof: `EVM::AccountCodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EVM::AccountStorages` (r:33000 w:32000)
	/// Proof: `EVM::AccountStorages` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[0, 1000]`.
	/// The range of component `l` is `[0, 32500]`.
	fn clear_suicided_storage(a: u32, l: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0 + a * (2 ±0) + l * (87 ±0)`
		//  Estimated: `7953 + a * (2352 ±14) + l * (2564 ±0)`
		// Minimum execution time: 1_850_000 picoseconds.
		Weight::from_parts(1_900_000, 7953)
			// Standard Error: 1_276_197
			.saturating_add(Weight::from_parts(18_091_220, 0).saturating_mul(a.into()))
			// Standard Error: 39_260
			.saturating_add(Weight::from_parts(5_558_165, 0).saturating_mul(l.into()))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(l.into())))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(l.into())))
			.saturating_add(Weight::from_parts(0, 2352).saturating_mul(a.into()))
			.saturating_add(Weight::from_parts(0, 2564).saturating_mul(l.into()))
	}
}

// For backwards compatibility and tests
impl WeightInfo for () {
	/// Storage: `EVM::AccountCodes` (r:1000 w:0)
	/// Proof: `EVM::AccountCodes` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `EVM::AccountStorages` (r:33000 w:32000)
	/// Proof: `EVM::AccountStorages` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `a` is `[0, 1000]`.
	/// The range of component `l` is `[0, 32500]`.
	fn clear_suicided_storage(a: u32, l: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0 + a * (2 ±0) + l * (87 ±0)`
		//  Estimated: `7953 + a * (2352 ±14) + l * (2564 ±0)`
		// Minimum execution time: 1_850_000 picoseconds.
		Weight::from_parts(1_900_000, 7953)
			// Standard Error: 1_276_197
			.saturating_add(Weight::from_parts(18_091_220, 0).saturating_mul(a.into()))
			// Standard Error: 39_260
			.saturating_add(Weight::from_parts(5_558_165, 0).saturating_mul(l.into()))
			.saturating_add(RocksDbWeight::get().reads((2_u64).saturating_mul(a.into())))
			.saturating_add(RocksDbWeight::get().reads((1_u64).saturating_mul(l.into())))
			.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(l.into())))
			.saturating_add(Weight::from_parts(0, 2352).saturating_mul(a.into()))
			.saturating_add(Weight::from_parts(0, 2564).saturating_mul(l.into()))
	}
}