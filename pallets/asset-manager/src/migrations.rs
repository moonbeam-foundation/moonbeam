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

use crate::{AssetIdType, AssetTypeId, AssetTypeUnitsPerSecond, Config};
use frame_support::{
	pallet_prelude::PhantomData,
	storage::migration::{remove_storage_prefix, storage_key_iter},
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
	Blake2_128Concat,
};
use sp_std::convert::TryInto;
//TODO sometimes this is unused, sometimes its necessary
use sp_std::vec::Vec;

pub struct AssetManagerUnitsWithAssetType<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for AssetManagerUnitsWithAssetType<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "AssetManagerUnitsWithAssetType", "actually running it");
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

		log::info!(target: "AssetManagerUnitsWithAssetType", "Migrating {:?} elements", migrated_count);

		// Now remove the old storage
		// https://crates.parity.io/frame_support/storage/migration/fn.remove_storage_prefix.html
		remove_storage_prefix(pallet_prefix, storage_item_prefix, &[]);

		// Assert that old storage is empty
		assert!(storage_key_iter::<T::AssetId, u128, Blake2_128Concat>(
			pallet_prefix,
			storage_item_prefix
		)
		.next()
		.is_none());

		// Write to the new storage with removed and added fields
		for (asset_id, units) in stored_data {
			// Read the assetType for the assetId
			if let Some(asset_type) = AssetIdType::<T>::get(&asset_id) {
				// Populate with assetType as key
				AssetTypeUnitsPerSecond::<T>::insert(&asset_type, units)
			}
		}

		log::info!(target: "AssetManagerUnitsWithAssetType", "almost done");

		// Return the weight used. For each migrated mapping there is a read to get it into
		// memory, a read to get assetType and
		// a write to clear the old stored value, and a write to re-store it.
		let db_weights = T::DbWeight::get();
		migrated_count.saturating_mul(2 * db_weights.write + 2 * db_weights.read)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
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
		assert!(AssetTypeUnitsPerSecond::<T>::iter().next().is_none());

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
			let example_pair = stored_data
				.iter()
				.next()
				.expect("We already confirmed that there was at least one item stored");

			Self::set_temp_storage(example_pair, "example_pair");
		}

		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;

		// Check number of entries matches what was set aside in pre_upgrade
		let old_mapping_count: u64 = Self::get_temp_storage("mapping_count")
			.expect("We stored a mapping count; it should be there; qed");
		let new_mapping_count = AssetTypeUnitsPerSecond::<T>::iter().count() as u64;
		assert_eq!(old_mapping_count, new_mapping_count);

		// Check that our example pair is still well-mapped after the migration
		if new_mapping_count > 0 {
			let (asset_id, units): (T::AssetId, u128) =
				Self::get_temp_storage("example_pair").expect("qed");

			let asset_type =
				AssetIdType::<T>::get(asset_id).expect("AssetIdType should have the assetType");

			let migrated_units = AssetTypeUnitsPerSecond::<T>::get(asset_type).expect("qed");
			// Check units are identical
			assert_eq!(migrated_units, units);
		}

		Ok(())
	}
}

pub struct AssetManagerPopulateAssetTypeIdStorage<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for AssetManagerPopulateAssetTypeIdStorage<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "AssetManagerPopulateAssetTypeIdStorage", "actually running it");
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdType";

		// Read all the data into memory.
		// https://crates.parity.io/frame_support/storage/migration/fn.storage_key_iter.html
		let stored_data: Vec<_> = storage_key_iter::<T::AssetId, T::AssetType, Blake2_128Concat>(
			pallet_prefix,
			storage_item_prefix,
		)
		.collect();

		let migrated_count: Weight = stored_data
			.len()
			.try_into()
			.expect("There are between 0 and 2**64 mappings stored.");

		log::info!(target: "AssetManagerPopulateAssetTypeIdStorage", "Migrating {:?} elements", migrated_count);

		// Write to the new storage
		for (asset_id, asset_type) in stored_data {
			// Populate with assetType as key
			AssetTypeId::<T>::insert(&asset_type, asset_id)
		}

		log::info!(target: "AssetManagerPopulateAssetTypeIdStorage", "almost done");

		// Return the weight used. For each migrated mapping there is a read to get it into
		// memory,  and a write to populate the new storage.
		let db_weights = T::DbWeight::get();
		migrated_count.saturating_mul(db_weights.write + db_weights.read)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;

		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdType";

		// We want to test that:
		// The new storage item is empty
		// The same number of mappings exist before and after
		// As long as there are some mappings stored, there will exist the reserve mapping in the new storage

		// Assert new storage is empty
		assert!(AssetTypeId::<T>::iter().next().is_none());

		// Check number of entries, and set it aside in temp storage
		let stored_data: Vec<_> = storage_key_iter::<T::AssetId, T::AssetType, Blake2_128Concat>(
			pallet_prefix,
			storage_item_prefix,
		)
		.collect();
		let mapping_count = stored_data.len();
		Self::set_temp_storage(mapping_count as u32, "mapping_count");

		// Read an example pair from old storage and set it aside in temp storage
		if mapping_count > 0 {
			let example_pair = stored_data
				.iter()
				.next()
				.expect("We already confirmed that there was at least one item stored");

			Self::set_temp_storage(example_pair, "example_pair");
		}

		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;

		// Check number of entries matches what was set aside in pre_upgrade
		let mapping_count: u64 = Self::get_temp_storage("mapping_count")
			.expect("We stored a mapping count; it should be there; qed");
		let new_mapping_count = AssetTypeId::<T>::iter().count() as u64;
		assert_eq!(mapping_count, new_mapping_count);

		// Check that our example pair is still well-mapped after the migration
		if new_mapping_count > 0 {
			let (asset_id, asset_type): (T::AssetId, T::AssetType) =
				Self::get_temp_storage("example_pair").expect("qed");

			let stored_asset_id =
				AssetTypeId::<T>::get(asset_type).expect("AssetTypeId should have the assetId");

			// Check assetIds are identical
			assert_eq!(asset_id, stored_asset_id);
		}

		Ok(())
	}
}
