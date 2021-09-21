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

use crate::{BalanceOf, Config, MappingWithDeposit, RegistrationInfo};
use frame_support::{
	pallet_prelude::PhantomData,
	storage::migration::{storage_iter, remove_storage_prefix, storage_key_iter},
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
	Twox64Concat,
};
use sp_std::convert::TryInto;
//TODO sometimes this is unused, sometimes its necessary
use sp_std::vec::Vec;

/// Migrates the AuthorMapping's storage map fro mthe insecure Twox64 hasher to the secure
/// BlakeTwo hasher.
pub struct TwoXToBlake<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for TwoXToBlake<T> {
	fn on_runtime_upgrade() -> Weight {
		let pallet_prefix: &[u8] = b"AuthorMapping";
		let storage_item_prefix: &[u8] = b"MappingWithDeposit";

		// Read all the data into memory.
		// https://crates.parity.io/frame_support/storage/migration/fn.storage_key_iter.html
		let stored_data: Vec<_> = storage_key_iter::<
			T::AuthorId,
			RegistrationInfo<T::AccountId, BalanceOf<T>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.collect();

		let migrated_count: Weight = stored_data
			.len()
			.try_into()
			.expect("There are between 0 and 2**64 mappings stored.");

		// Now remove the old storage
		// https://crates.parity.io/frame_support/storage/migration/fn.remove_storage_prefix.html
		remove_storage_prefix(pallet_prefix, storage_item_prefix, &[]);

		// Assert that old storage is empty
		assert!(storage_key_iter::<
			T::AuthorId,
			RegistrationInfo<T::AccountId, BalanceOf<T>>,
			Twox64Concat,
		>(pallet_prefix, storage_item_prefix)
		.next()
		.is_none());

		// Write the mappings back to storage with the new secure hasher
		for (author_id, account_id) in stored_data {
			MappingWithDeposit::<T>::insert(author_id, account_id);
		}

		// Return the weight used. For each migrated mapping there is a red to get it into
		// memory, a write to clear the old stored value, and a write to re-store it.
		let db_weights = T::DbWeight::get();
		migrated_count.saturating_mul(2 * db_weights.write + db_weights.read)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		let pallet_prefix: &[u8] = b"AuthorMapping";
		let storage_item_prefix: &[u8] = b"MappingWithDeposit";

		// We want to test that:
		// There are no entries in the new storage beforehand
		// The same number of mappings exist before and after
		// As long as there are some mappings stored, one representative key maps to the
		// same value after the migration.
		// There are no entries in the old storage afterward

		// Assert new storage is empty
		// Because the pallet and item prefixes are the same, the old storage is still at this
		// key. However, the values can't be decoded so the assertion passes.
		assert!(MappingWithDeposit::<T>::iter().next().is_none());

		// Check number of entries, and set it aside in temp storage
		let mapping_count = storage_iter::<RegistrationInfo<T::AccountId, BalanceOf<T>>>(
			pallet_prefix,
			storage_item_prefix,
		)
		.count() as u64;
		Self::set_temp_storage(mapping_count, "mapping_count");

		// Read an example pair from old storage and set it aside in temp storage
		if mapping_count > 0 {
			let example_pair = storage_key_iter::<
				T::AuthorId,
				RegistrationInfo<T::AccountId, BalanceOf<T>>,
				Twox64Concat,
			>(pallet_prefix, storage_item_prefix)
			.next()
			.expect("We already confirmed that there was at least one item stored");

			Self::set_temp_storage(example_pair, "example_pair");
		}

		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		// Check number of entries matches what was set aside in pre_upgrade
		let old_mapping_count: u64 = Self::get_temp_storage("mapping_count")
			.expect("We stored a mapping count; it should be there; qed");
		let new_mapping_count = MappingWithDeposit::<T>::iter().count() as u64;
		assert_eq!(old_mapping_count, new_mapping_count);

		// Check that our example pair is still well-mapped after the migration
		if new_mapping_count > 0 {
			let (account, original_info): (
				T::AuthorId,
				RegistrationInfo<T::AccountId, BalanceOf<T>>,
			) = Self::get_temp_storage("example_pair").expect("qed");
			let migrated_info = MappingWithDeposit::<T>::get(account).expect("qed");
			assert_eq!(original_info, migrated_info);
		}

		Ok(())
	}
}
