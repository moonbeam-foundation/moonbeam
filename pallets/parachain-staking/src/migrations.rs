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

use crate::Config;
use frame_support::storage::generator::StorageValue;
use frame_support::storage::unhashed;
use frame_support::traits::OnRuntimeUpgrade;

/// Migrates RoundInfo and add the field first_slot
pub struct MigrateRoundWithFirstSlot<T: Config>(core::marker::PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateRoundWithFirstSlot<T> {
	fn on_runtime_upgrade() -> frame_support::pallet_prelude::Weight {
		let raw_key = crate::Round::<T>::storage_prefix();

		if let Some(bytes) = unhashed::get_raw(&raw_key) {
			let len = bytes.len();
			match len {
				20 => {
					// migration already done
				}
				16 => {
					// migrate from 2800
				}
				12 => {
					// migrater from 2700
				}
				_ => panic!("corrupted storage"),
			}
		}

		Default::default()
	}
}
