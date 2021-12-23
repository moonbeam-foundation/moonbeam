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

use crate::{Config, SupportedFeePaymentAssets};
use frame_support::{
	pallet_prelude::PhantomData,
	storage::migration::storage_key_iter,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
	Blake2_128Concat,
};
use sp_std::convert::TryInto;
//TODO sometimes this is unused, sometimes its necessary
use sp_std::vec::Vec;

pub struct PopulateSupportedFeePaymentAssets<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for PopulateSupportedFeePaymentAssets<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "PopulateSupportedFeePaymentAssets", "actually running it");
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdUnitsPerSecond";

		// Read all the data into memory.
		// https://crates.parity.io/frame_support/storage/migration/fn.storage_key_iter.html
		let stored_data: Vec<_> = storage_key_iter::<T::AssetId, u128, Blake2_128Concat>(
			pallet_prefix,
			storage_item_prefix,
		)
		.collect();

		let migrated_count: Weight = stored_data
			.len()
			.try_into()
			.expect("There are between 0 and 2**64 mappings stored.");

		log::info!(target: "AssetIdUnitsPerSecond", "Migrating {:?} elements", migrated_count);

		// Collect in a vec
		let mut supported_assets: Vec<T::AssetId> = Vec::new();
		for (asset_id, _) in stored_data {
			supported_assets.push(asset_id);
		}

		// Push value
		SupportedFeePaymentAssets::<T>::put(supported_assets);

		log::info!(target: "AssetIdUnitsPerSecond", "almost done");

		// Return the weight used. For each migrated mapping there is a red to get it into
		// memory
		// A final one write makes it push to the new storage item
		let db_weights = T::DbWeight::get();
		let weight = migrated_count.saturating_mul(db_weights.read);
		weight.saturating_add(db_weights.write)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		use frame_support::storage::migration::{storage_iter, storage_key_iter};
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;

		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdUnitsPerSecond";

		// We want to test that:
		// There are no entries in the new storage beforehand
		// The same number of mappings exist before and after
		// As long as there are some mappings stored, one representative key maps to the
		// same value after the migration.
		// There are no entries in the old storage afterward

		// Assert new storage is empty
		// Because the pallet and item prefixes are the same, the old storage is still at this
		// key. However, the values can't be decoded so the assertion passes.
		assert!(SupportedFeePaymentAssets::<T>::get().len() == 0);

		// Check number of entries, and set it aside in temp storage
		let stored_data: Vec<_> = storage_key_iter::<T::AssetId, u128, Blake2_128Concat>(
			pallet_prefix,
			storage_item_prefix,
		)
		.collect();
		let mapping_count = stored_data.len();
		Self::set_temp_storage(mapping_count as u32, "mapping_count");

		// Read an example pair from old storage and set it aside in temp storage
		if mapping_count > 0 {
			let example_key = stored_data
				.iter()
				.next()
				.expect("We already confirmed that there was at least one item stored")
				.0;

			Self::set_temp_storage(example_key, "example_key");
		}

		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;

		// Check number of entries matches what was set aside in pre_upgrade
		let old_mapping_count: u64 = Self::get_temp_storage("mapping_count")
			.expect("We stored a mapping count; it should be there; qed");
		let new_mapping_count = SupportedFeePaymentAssets::<T>::get().len() as u64;
		assert_eq!(old_mapping_count, new_mapping_count);

		// Check that our example pair is still well-mapped after the migration
		if new_mapping_count > 0 {
			let asset_id: T::AssetId = Self::get_temp_storage("example_pair").expect("qed");
			let migrated_info = SupportedFeePaymentAssets::<T>::get();
			// Check that the asset_id exists in migrated_info
			assert!(migrated_info.contais(asset_id));
		}

		Ok(())
	}
}
