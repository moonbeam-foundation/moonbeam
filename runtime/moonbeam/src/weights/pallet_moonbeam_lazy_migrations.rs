// Copyright 2025 Moonbeam foundation
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

//! Weights for `pallet_moonbeam_lazy_migrations`
//!
//! This file was simplified to only contain contract metadata functionality

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weights for `pallet_moonbeam_lazy_migrations`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_moonbeam_lazy_migrations::WeightInfo for WeightInfo<T> {
	/// Storage: `Evm::AccountCodes` (r:1 w:0)
	/// Proof: `Evm::AccountCodes` (`max_values`: None, `max_size`: Some(665300), added: 667775, mode: `MaxEncodedLen`)
	/// Storage: `MoonbeamLazyMigrations::ContractMetadata` (r:1 w:1)
	/// Proof: `MoonbeamLazyMigrations::ContractMetadata` (`max_values`: None, `max_size`: Some(30000), added: 32475, mode: `MaxEncodedLen`)
	fn create_contract_metadata() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `4695`
		//  Estimated: `668765`
		// Minimum execution time: 23_820_000 picoseconds.
		Weight::from_parts(24_780_000, 668765)
			.saturating_add(T::DbWeight::get().reads(2_u64))
			.saturating_add(T::DbWeight::get().writes(1_u64))
	}
}
