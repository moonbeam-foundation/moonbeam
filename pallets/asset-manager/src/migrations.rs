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

use crate::{AssetIdType, AssetTypeId, AssetTypeUnitsPerSecond, Config, SupportedFeePaymentAssets};
use frame_support::{
	pallet_prelude::PhantomData,
	storage::migration::storage_key_iter,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
	Blake2_128Concat,
};
//TODO sometimes this is unused, sometimes its necessary
use sp_std::vec::Vec;
use xcm::latest::prelude::*;

/// Migration that changes the mapping AssetId -> units_per_second to
/// a mapping of the form ForeignAssetType -> units_per_second
/// It does so by removing the AssetTypeUnitsPerSecond storage and
/// populating the new AssetTypeUnitsPerSecond
pub struct UnitsWithAssetType<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for UnitsWithAssetType<T> {
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

	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "UnitsWithForeignAssetType", "actually running it");
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdUnitsPerSecond";

		// Read all the data into memory.
		// https://crates.parity.io/frame_support/storage/migration/fn.storage_key_iter.html
		let stored_data: Vec<_> = storage_key_iter::<T::AssetId, u128, Blake2_128Concat>(
			pallet_prefix,
			storage_item_prefix,
		)
		.drain()
		.collect();

		let migrated_count = stored_data.len() as u64;

		log::info!(target: "UnitsWithForeignAssetType", "Migrating {:?} elements", migrated_count);

		// Write to the new storage with removed and added fields
		for (asset_id, units) in stored_data {
			// Read the ForeignAssetType for the assetId
			if let Some(asset_type) = AssetIdType::<T>::get(&asset_id) {
				// Populate with ForeignAssetType as key
				AssetTypeUnitsPerSecond::<T>::insert(&asset_type, units)
			}
		}

		log::info!(target: "UnitsWithForeignAssetType", "almost done");

		// Return the weight used. For each migrated mapping there is a read to get it into
		// memory, a read to get ForeignAssetType and
		// a write to clear the old stored value, and a write to re-store it.
		let db_weights = T::DbWeight::get();
		let rw_count = migrated_count.saturating_mul(2u64);
		db_weights.reads_writes(rw_count, rw_count)
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;

		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdUnitsPerSecond";
		// Assert that old storage is empty
		assert!(storage_key_iter::<T::AssetId, u128, Blake2_128Concat>(
			pallet_prefix,
			storage_item_prefix
		)
		.next()
		.is_none());

		// Check number of entries matches what was set aside in pre_upgrade
		let old_mapping_count: u64 = Self::get_temp_storage("mapping_count")
			.expect("We stored a mapping count; it should be there; qed");
		let new_mapping_count = AssetTypeUnitsPerSecond::<T>::iter().count() as u64;
		assert_eq!(old_mapping_count, new_mapping_count);

		// Check that our example pair is still well-mapped after the migration
		if new_mapping_count > 0 {
			let (asset_id, units): (T::AssetId, u128) =
				Self::get_temp_storage("example_pair").expect("qed");

			let asset_type = AssetIdType::<T>::get(asset_id)
				.expect("AssetIdType should have the ForeignAssetType");

			let migrated_units = AssetTypeUnitsPerSecond::<T>::get(asset_type).expect("qed");
			// Check units are identical
			assert_eq!(migrated_units, units);
		}

		Ok(())
	}
}

/// Migration that reads data from the AssetIdType mapping (AssetId -> ForeignAssetType)
/// and populates the reverse mapping AssetTypeId (ForeignAssetType -> AssetId)
pub struct PopulateAssetTypeIdStorage<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for PopulateAssetTypeIdStorage<T> {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;

		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdType";

		// We want to test that:
		// The new storage item is empty
		// The same number of mappings exist before and after
		// As long as there are some mappings stored,
		// there will exist the reserve mapping in the new storage

		// Assert new storage is empty
		assert!(AssetTypeId::<T>::iter().next().is_none());

		// Check number of entries, and set it aside in temp storage
		let stored_data: Vec<_> = storage_key_iter::<
			T::AssetId,
			T::ForeignAssetType,
			Blake2_128Concat,
		>(pallet_prefix, storage_item_prefix)
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

	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "PopulateAssetTypeIdStorage", "actually running it");
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdType";

		// Read all the data into memory.
		// https://crates.parity.io/frame_support/storage/migration/fn.storage_key_iter.html
		let stored_data: Vec<_> = storage_key_iter::<
			T::AssetId,
			T::ForeignAssetType,
			Blake2_128Concat,
		>(pallet_prefix, storage_item_prefix)
		.collect();

		let migrated_count = stored_data.len() as u64;

		log::info!(
			target: "PopulateAssetTypeIdStorage",
			"Migrating {:?} elements",
			migrated_count
		);

		// Write to the new storage
		for (asset_id, asset_type) in stored_data {
			// Populate with ForeignAssetType as key
			AssetTypeId::<T>::insert(&asset_type, asset_id)
		}

		log::info!(target: "PopulateAssetTypeIdStorage", "almost done");

		// Return the weight used. For each migrated mapping there is a read to get it into
		// memory,  and a write to populate the new storage.
		let db_weights = T::DbWeight::get();
		db_weights.reads_writes(migrated_count, migrated_count)
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
			let (asset_id, asset_type): (T::AssetId, T::ForeignAssetType) =
				Self::get_temp_storage("example_pair").expect("qed");

			let stored_asset_id =
				AssetTypeId::<T>::get(asset_type).expect("AssetTypeId should have the assetId");

			// Check assetIds are identical
			assert_eq!(asset_id, stored_asset_id);
		}

		Ok(())
	}
}

/// Migration that reads the existing ForeignAssetTypes looking for old Statemine prefixes of
/// the form (Parachain, GeneralIndex) and changes them for the new prefix
/// (Parachain, PalletInstance, GeneralIndex)
pub struct ChangeStateminePrefixes<T, StatemineParaIdInfo, StatemineAssetsInstanceInfo>(
	PhantomData<(T, StatemineParaIdInfo, StatemineAssetsInstanceInfo)>,
);
impl<T, StatemineParaIdInfo, StatemineAssetsInstanceInfo> OnRuntimeUpgrade
	for ChangeStateminePrefixes<T, StatemineParaIdInfo, StatemineAssetsInstanceInfo>
where
	T: Config,
	StatemineParaIdInfo: Get<u32>,
	StatemineAssetsInstanceInfo: Get<u8>,
	T::ForeignAssetType: Into<Option<MultiLocation>> + From<MultiLocation>,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;

		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdType";

		// We want to test that:
		// If there exists an ForeignAssetType matching the Statemine, it gets overwritten

		// Check number of entries, and set it aside in temp storage
		let stored_data: Vec<_> = storage_key_iter::<
			T::AssetId,
			T::ForeignAssetType,
			Blake2_128Concat,
		>(pallet_prefix, storage_item_prefix)
		.collect();

		let statemine_para_id = StatemineParaIdInfo::get();

		let mut found = false;

		for (asset_id, asset_type) in stored_data {
			let location: Option<MultiLocation> = asset_type.clone().into();
			match location {
				Some(MultiLocation {
					parents: 1,
					interior: X2(Parachain(para_id), GeneralIndex(_)),
				}) if para_id == statemine_para_id => {
					// We are going to note that we found at least one entry matching
					found = true;
					// And we are going to record its data
					Self::set_temp_storage((asset_id, asset_type), "example_pair");
					break;
				}
				_ => continue,
			}
		}
		Self::set_temp_storage(found, "matching_type_found");

		Ok(())
	}

	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "ChangeStateminePrefixes", "actually running it");
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdType";

		// Read all the data into memory.
		// https://crates.parity.io/frame_support/storage/migration/fn.storage_key_iter.html
		let stored_data: Vec<_> = storage_key_iter::<
			T::AssetId,
			T::ForeignAssetType,
			Blake2_128Concat,
		>(pallet_prefix, storage_item_prefix)
		.collect();

		let mut read_count = stored_data.len() as u64;
		let mut write_count = 0u64;

		log::info!(target: "ChangeStateminePrefixes", "Evaluating {:?} elements", read_count);

		let db_weights = T::DbWeight::get();

		let statemine_para_id = StatemineParaIdInfo::get();
		let statemine_assets_pallet = StatemineAssetsInstanceInfo::get();
		// Write to the new storage
		for (asset_id, asset_type) in stored_data {
			let location: Option<MultiLocation> = asset_type.clone().into();
			match location {
				Some(MultiLocation {
					parents: 1,
					interior: X2(Parachain(para_id), GeneralIndex(index)),
				}) if para_id == statemine_para_id => {
					let new_location = MultiLocation {
						parents: 1,
						interior: X3(
							Parachain(para_id),
							PalletInstance(statemine_assets_pallet),
							GeneralIndex(index),
						),
					};
					let new_asset_type: T::ForeignAssetType = new_location.into();
					// Insert new asset type previous asset type
					AssetIdType::<T>::insert(&asset_id, &new_asset_type);

					// This is checked in case AssetManagerPopulateForeignAssetTypeIdStorage runs first
					if AssetTypeId::<T>::get(&asset_type) == Some(asset_id) {
						// We need to update ForeignAssetTypeId too
						AssetTypeId::<T>::remove(&asset_type);
						AssetTypeId::<T>::insert(&new_asset_type, asset_id);

						// Update weight due to this branch
						write_count += 2;
					}

					// This is checked in case UnitsWithForeignAssetType runs first
					if let Some(units) = AssetTypeUnitsPerSecond::<T>::take(&asset_type) {
						// We need to update AssetTypeUnitsPerSecond too
						AssetTypeUnitsPerSecond::<T>::insert(&new_asset_type, units);

						// Update weight due to this branch
						write_count += 2;
					}

					// Update used weight
					read_count += 2;
					write_count += 1;
				}
				_ => continue,
			}
		}

		log::info!(target: "ChangeStateminePrefixes", "almost done");

		// Return the weight used.
		db_weights.reads_writes(read_count, write_count)
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;

		// Check if we found a matching type
		let found: bool = Self::get_temp_storage("matching_type_found")
			.expect("We stored a matching_type_found and should be here; qed");

		let statemine_para_id = StatemineParaIdInfo::get();
		let statemine_assets_pallet = StatemineAssetsInstanceInfo::get();

		// Check that our example pair suffered the correct migration
		if found {
			let (asset_id, asset_type): (T::AssetId, T::ForeignAssetType) =
				Self::get_temp_storage("example_pair").expect("qed");
			let location: Option<MultiLocation> = asset_type.into();

			match location {
				Some(MultiLocation {
					parents: 1,
					interior: X2(Parachain(para_id), GeneralIndex(index)),
				}) if para_id == statemine_para_id => {
					let stored_asset_type =
						AssetIdType::<T>::get(asset_id).expect("This entry should be updated");

					let expected_new_asset_type: T::ForeignAssetType = MultiLocation {
						parents: 1,
						interior: X3(
							Parachain(para_id),
							PalletInstance(statemine_assets_pallet),
							GeneralIndex(index),
						),
					}
					.into();

					// Check ForeignAssetTypes are identical
					assert_eq!(stored_asset_type, expected_new_asset_type);
				}
				_ => panic!("This should never have entered this path"),
			}
		}

		Ok(())
	}
}

pub struct PopulateSupportedFeePaymentAssets<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for PopulateSupportedFeePaymentAssets<T> {
	fn on_runtime_upgrade() -> Weight {
		log::trace!(
			target: "PopulateSupportedFeePaymentAssets",
			"Running PopulateSupportedFeePaymentAssets migration"
		);
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetTypeUnitsPerSecond";

		log::trace!(
			target: "PopulateSupportedFeePaymentAssets",
			"grabbing from AssetTypeUnitsPerSecond"
		);

		// Read all the data into memory.
		// https://crates.parity.io/frame_support/storage/migration/fn.storage_key_iter.html
		let stored_data: Vec<_> = storage_key_iter::<T::ForeignAssetType, u128, Blake2_128Concat>(
			pallet_prefix,
			storage_item_prefix,
		)
		.collect();

		let migrated_count = stored_data.len() as u64;

		log::trace!(
			target: "PopulateSupportedFeePaymentAssets",
			"PopulateSupportedFeePaymentAssets pushing {:?} elements to SupportedFeePaymentAssets",
			migrated_count
		);

		// Collect in a vec
		let mut supported_assets: Vec<T::ForeignAssetType> = Vec::new();
		for (asset_type, _) in stored_data {
			supported_assets.push(asset_type);
		}

		supported_assets.sort();

		// Push value
		SupportedFeePaymentAssets::<T>::put(&supported_assets);

		log::trace!(
			target: "PopulateSupportedFeePaymentAssets",
			"SupportedFeePaymentAssets populated now having {:?} elements",
			supported_assets.len()
		);

		// Return the weight used. For each migrated mapping there is a read to get it into
		// memory
		// A final one write makes it push to the new storage item
		let db_weights = T::DbWeight::get();
		db_weights.reads_writes(migrated_count, 1)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;

		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetTypeUnitsPerSecond";

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
		let stored_data: Vec<_> = storage_key_iter::<T::ForeignAssetType, u128, Blake2_128Concat>(
			pallet_prefix,
			storage_item_prefix,
		)
		.collect();
		let mapping_count = stored_data.len();
		Self::set_temp_storage(mapping_count as u64, "mapping_count");

		// Read an example pair from old storage and set it aside in temp storage
		if mapping_count > 0 {
			let example_key = stored_data
				.iter()
				.next()
				.expect("We already confirmed that there was at least one item stored")
				.clone()
				.0;

			Self::set_temp_storage(example_key, "example_pair");
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
			let asset_type: T::ForeignAssetType =
				Self::get_temp_storage("example_pair").expect("qed");
			let migrated_info = SupportedFeePaymentAssets::<T>::get();
			// Check that the asset_id exists in migrated_info
			assert!(migrated_info.contains(&asset_type));
		}

		Ok(())
	}
}
