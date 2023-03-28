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
#[cfg(feature = "try-runtime")]
use frame_support::storage::{generator::StorageValue, migration::get_storage_value};
use frame_support::{
	pallet_prelude::PhantomData,
	storage::migration::storage_key_iter,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
	Blake2_128Concat, StoragePrefixedMap,
};
use parity_scale_codec::{Decode, Encode};
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
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdUnitsPerSecond";

		let mut state_vec: Vec<(u32, (T::AssetId, u128))> = Vec::new();

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

		// Read an example pair from old storage and set it aside in temp storage
		if mapping_count > 0 {
			let example_pair = stored_data
				.iter()
				.next()
				.expect("We already confirmed that there was at least one item stored");

			state_vec.push((mapping_count as u32, example_pair.clone()))
		}

		Ok(state_vec.encode())
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
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		let state_vec: Vec<(u32, (T::AssetId, u128))> =
			Decode::decode(&mut &state[..]).expect("pre_upgrade provides a valid state; qed");

		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdUnitsPerSecond";
		// Assert that old storage is empty
		assert!(storage_key_iter::<T::AssetId, u128, Blake2_128Concat>(
			pallet_prefix,
			storage_item_prefix
		)
		.next()
		.is_none());

		if state_vec.len() > 0 {
			let (old_mapping_count, (asset_id, units)) =
				state_vec.first().expect("we should have an element");
			// Check number of entries matches what was set aside in pre_upgrade
			let new_mapping_count = AssetTypeUnitsPerSecond::<T>::iter().count() as u32;
			assert_eq!(old_mapping_count.clone(), new_mapping_count);

			let asset_type = AssetIdType::<T>::get(asset_id)
				.expect("AssetIdType should have the ForeignAssetType");

			let migrated_units = AssetTypeUnitsPerSecond::<T>::get(asset_type).expect("qed");
			// Check units are identical
			assert_eq!(migrated_units, units.clone());
		}

		Ok(())
	}
}

/// Migration that reads data from the AssetIdType mapping (AssetId -> ForeignAssetType)
/// and populates the reverse mapping AssetTypeId (ForeignAssetType -> AssetId)
pub struct PopulateAssetTypeIdStorage<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for PopulateAssetTypeIdStorage<T> {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		let pallet_prefix: &[u8] = b"AssetManager";
		let storage_item_prefix: &[u8] = b"AssetIdType";

		// We want to test that:
		// The new storage item is empty
		// The same number of mappings exist before and after
		// As long as there are some mappings stored,
		// there will exist the reserve mapping in the new storage

		// Assert new storage is empty
		assert!(AssetTypeId::<T>::iter().next().is_none());

		let mut state_vec: Vec<(u32, (T::AssetId, T::ForeignAssetType))> = Vec::new();

		// Check number of entries, and set it aside in temp storage
		let stored_data: Vec<_> = storage_key_iter::<
			T::AssetId,
			T::ForeignAssetType,
			Blake2_128Concat,
		>(pallet_prefix, storage_item_prefix)
		.collect();
		let mapping_count = stored_data.len();

		// Read an example pair from old storage and set it aside in temp storage
		if mapping_count > 0 {
			let example_pair = stored_data
				.iter()
				.next()
				.expect("We already confirmed that there was at least one item stored");

			state_vec.push((mapping_count as u32, example_pair.clone()))
		}

		Ok(state_vec.encode())
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
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		let state_vec: Vec<(u32, (T::AssetId, T::ForeignAssetType))> =
			Decode::decode(&mut &state[..]).expect("pre_upgrade provides a valid state; qed");

		// Check that our example pair is still well-mapped after the migration
		if state_vec.len() > 0 {
			let (mapping_count, (asset_id, asset_type)) =
				state_vec.first().expect("we should have an element");
			// Check number of entries matches what was set aside in pre_upgrade

			let new_mapping_count = AssetTypeId::<T>::iter().count() as u32;
			assert_eq!(mapping_count.clone(), new_mapping_count);

			// Check that our example pair is still well-mapped after the migration
			let stored_asset_id =
				AssetTypeId::<T>::get(asset_type).expect("AssetTypeId should have the assetId");

			// Check assetIds are identical
			assert_eq!(asset_id.clone(), stored_asset_id);
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
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
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

		let mut state_vec: Vec<(T::AssetId, T::ForeignAssetType)> = Vec::new();

		for (asset_id, asset_type) in stored_data {
			let location: Option<MultiLocation> = asset_type.clone().into();
			match location {
				Some(MultiLocation {
					parents: 1,
					interior: X2(Parachain(para_id), GeneralIndex(_)),
				}) if para_id == statemine_para_id => {
					// We are going to note that we found at least one entry matching
					state_vec.push((asset_id, asset_type));
					// And we are going to record its data
					break;
				}
				_ => continue,
			}
		}

		Ok(state_vec.encode())
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
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		let state_vec: Vec<(T::AssetId, T::ForeignAssetType)> =
			Decode::decode(&mut &state[..]).expect("pre_upgrade provides a valid state; qed");

		let statemine_para_id = StatemineParaIdInfo::get();
		let statemine_assets_pallet = StatemineAssetsInstanceInfo::get();

		// Check that our example pair suffered the correct migration
		if state_vec.len() > 0 {
			let (asset_id, asset_type) = state_vec.first().expect("qed");

			let location: Option<MultiLocation> = asset_type.clone().into();

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
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		let mut state_vec: Vec<(u32, T::ForeignAssetType)> = Vec::new();

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

		// Read an example pair from old storage and set it aside in temp storage
		if mapping_count > 0 {
			let example_key = stored_data
				.iter()
				.next()
				.expect("We already confirmed that there was at least one item stored")
				.clone()
				.0;

			state_vec.push((mapping_count as u32, example_key))
		}

		Ok(state_vec.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		let state_vec: Vec<(u32, T::ForeignAssetType)> =
			Decode::decode(&mut &state[..]).expect("pre_upgrade provides a valid state; qed");

		// Check that our example pair is still well-mapped after the migration
		if state_vec.len() > 0 {
			let (old_mapping_count, asset_type) = state_vec.first().expect("qed");
			let new_mapping_count = SupportedFeePaymentAssets::<T>::get().len() as u32;
			assert_eq!(old_mapping_count.clone(), new_mapping_count);
			let migrated_info = SupportedFeePaymentAssets::<T>::get();
			// Check that the asset_id exists in migrated_info
			assert!(migrated_info.contains(&asset_type));
		}

		Ok(())
	}
}

#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode)]
pub(crate) enum OldAssetType {
	Xcm(xcm::v2::MultiLocation),
}

impl Into<Option<xcm::v2::MultiLocation>> for OldAssetType {
	fn into(self) -> Option<xcm::v2::MultiLocation> {
		match self {
			Self::Xcm(location) => Some(location),
		}
	}
}

#[cfg(feature = "try-runtime")]
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode)]
enum PreUpgradeState<T: Config> {
	AssetIdType(Vec<(T::AssetId, OldAssetType)>),
	AssetTypeId(Vec<(OldAssetType, T::AssetId)>),
	AssetTypeUnitsPerSecond(Vec<(OldAssetType, u128)>),
	SupportedFeePaymentAssets(Vec<OldAssetType>),
}

#[cfg(feature = "try-runtime")]
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode)]
enum PostUpgradeState<T: Config> {
	AssetIdType(Vec<(T::AssetId, T::ForeignAssetType)>),
	AssetTypeId(Vec<(T::ForeignAssetType, T::AssetId)>),
	AssetTypeUnitsPerSecond(Vec<(T::ForeignAssetType, u128)>),
	SupportedFeePaymentAssets(Vec<T::ForeignAssetType>),
}

#[cfg(feature = "try-runtime")]
impl<T: Config> From<PreUpgradeState<T>> for PostUpgradeState<T>
where
	T::ForeignAssetType: From<MultiLocation>,
{
	fn from(pre: PreUpgradeState<T>) -> PostUpgradeState<T> {
		match pre {
			PreUpgradeState::AssetIdType(items) => {
				let mut out: Vec<(T::AssetId, T::ForeignAssetType)> = Vec::new();
				for (key, value) in items.into_iter() {
					let old_multilocation: Option<xcm::v2::MultiLocation> = value.into();
					let old_multilocation: xcm::v2::MultiLocation =
						old_multilocation.expect("old storage convert to XcmV2 Multilocation");
					let new_multilocation: MultiLocation = old_multilocation
						.try_into()
						.expect("Multilocation v2 to v3");
					out.push((key, new_multilocation.into()));
				}
				PostUpgradeState::AssetIdType(out)
			}
			PreUpgradeState::AssetTypeId(items) => {
				let mut out: Vec<(T::ForeignAssetType, T::AssetId)> = Vec::new();
				for (key, value) in items.into_iter() {
					let old_multilocation: Option<xcm::v2::MultiLocation> = key.into();
					let old_multilocation: xcm::v2::MultiLocation =
						old_multilocation.expect("old storage convert to XcmV2 Multilocation");
					let new_multilocation: MultiLocation = old_multilocation
						.try_into()
						.expect("Multilocation v2 to v3");
					let new_key: T::ForeignAssetType = new_multilocation.into();
					out.push((new_key, value));
				}
				PostUpgradeState::AssetTypeId(out)
			}
			PreUpgradeState::AssetTypeUnitsPerSecond(items) => {
				let mut out: Vec<(T::ForeignAssetType, u128)> = Vec::new();
				for (key, value) in items.into_iter() {
					let old_multilocation: Option<xcm::v2::MultiLocation> = key.into();
					let old_multilocation: xcm::v2::MultiLocation =
						old_multilocation.expect("old storage convert to XcmV2 Multilocation");
					let new_multilocation: MultiLocation = old_multilocation
						.try_into()
						.expect("Multilocation v2 to v3");
					out.push((new_multilocation.into(), value));
				}
				PostUpgradeState::AssetTypeUnitsPerSecond(out)
			}
			PreUpgradeState::SupportedFeePaymentAssets(items) => {
				let mut out: Vec<T::ForeignAssetType> = Vec::new();
				for value in items.into_iter() {
					let old_multilocation: Option<xcm::v2::MultiLocation> = value.into();
					let old_multilocation: xcm::v2::MultiLocation =
						old_multilocation.expect("old storage convert to XcmV2 Multilocation");
					let new_multilocation: MultiLocation = old_multilocation
						.try_into()
						.expect("Multilocation v2 to v3");
					out.push(new_multilocation.into());
				}
				PostUpgradeState::SupportedFeePaymentAssets(out)
			}
		}
	}
}

pub struct XcmV2ToV3AssetManager<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for XcmV2ToV3AssetManager<T>
where
	T::ForeignAssetType: From<MultiLocation>,
{
	fn on_runtime_upgrade() -> Weight {
		log::trace!(
			target: "XcmV2ToV3AssetManager",
			"Running XcmV2ToV3AssetManager migration"
		);
		// Migrates the pallet's storage from Xcm V2 to V3:
		//	- AssetIdType -> migrate map's value
		//	- AssetTypeId -> migrate map's key
		//	- AssetTypeUnitsPerSecond -> migrate map's key
		//	- SupportedFeePaymentAssets -> migrate value

		// Shared module prefix
		let module_prefix = AssetIdType::<T>::module_prefix();
		// AssetTypeId
		let asset_type_id_storage_prefix = AssetTypeId::<T>::storage_prefix();
		// AssetTypeUnitsPerSecond
		let units_per_second_storage_prefix = AssetTypeUnitsPerSecond::<T>::storage_prefix();

		// Db (read, write) count
		let mut db_weight_count: (u64, u64) = (0, 0);

		// Migrate `AssetIdType` value
		let _ = AssetIdType::<T>::translate::<OldAssetType, _>(|_key, value| {
			db_weight_count.0 += 1;
			db_weight_count.1 += 1;
			let old_multilocation: Option<xcm::v2::MultiLocation> = value.into();
			let old_multilocation: xcm::v2::MultiLocation =
				old_multilocation.expect("old storage convert to XcmV2 Multilocation");
			let new_multilocation: MultiLocation = old_multilocation
				.try_into()
				.expect("Multilocation v2 to v3");
			Some(new_multilocation.into())
		});

		// Migrate `AssetTypeId` key
		db_weight_count.0 += 1;
		let old_data = storage_key_iter::<OldAssetType, T::AssetId, Blake2_128Concat>(
			&module_prefix,
			asset_type_id_storage_prefix,
		)
		.drain()
		.collect::<Vec<(OldAssetType, T::AssetId)>>();
		for (old_key, value) in old_data {
			db_weight_count.1 += 1;
			let old_key: Option<xcm::v2::MultiLocation> = old_key.into();
			let old_key: xcm::v2::MultiLocation =
				old_key.expect("old storage convert to XcmV2 Multilocation");
			let v3_multilocation: MultiLocation =
				old_key.try_into().expect("Multilocation v2 to v3");
			let new_key: T::ForeignAssetType = v3_multilocation.into();
			AssetTypeId::<T>::insert(new_key, value);
		}

		// Migrate `AssetTypeUnitsPerSecond` key
		db_weight_count.0 += 1;
		let old_data = storage_key_iter::<OldAssetType, u128, Blake2_128Concat>(
			&module_prefix,
			units_per_second_storage_prefix,
		)
		.drain()
		.collect::<Vec<(OldAssetType, u128)>>();
		for (old_key, value) in old_data {
			db_weight_count.1 += 1;
			let old_key: Option<xcm::v2::MultiLocation> = old_key.into();
			let old_key: xcm::v2::MultiLocation =
				old_key.expect("old storage convert to XcmV2 Multilocation");
			let v3_multilocation: MultiLocation =
				old_key.try_into().expect("Multilocation v2 to v3");
			let new_key: T::ForeignAssetType = v3_multilocation.into();
			AssetTypeUnitsPerSecond::<T>::insert(new_key, value);
		}

		// Migrate `SupportedFeePaymentAssets` value
		let _ = SupportedFeePaymentAssets::<T>::translate::<Vec<OldAssetType>, _>(|value| {
			db_weight_count.0 += 1;
			db_weight_count.1 += 1;
			let new_value: Vec<T::ForeignAssetType> = value
				.unwrap_or_default()
				.into_iter()
				.map(|old_value| {
					let old_multilocation: Option<xcm::v2::MultiLocation> = old_value.into();
					let old_multilocation: xcm::v2::MultiLocation =
						old_multilocation.expect("old storage convert to XcmV2 Multilocation");
					let new_multilocation: MultiLocation = old_multilocation
						.try_into()
						.expect("Multilocation v2 to v3");
					new_multilocation.into()
				})
				.collect();
			Some(new_value)
		});

		T::DbWeight::get().reads_writes(db_weight_count.0, db_weight_count.1)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		log::trace!(
			target: "XcmV2ToV3AssetManager",
			"Running XcmV2ToV3AssetManager pre_upgrade hook"
		);
		// Shared module prefix
		let module_prefix = AssetIdType::<T>::module_prefix();
		// AssetIdType
		let asset_id_type_storage_prefix = AssetIdType::<T>::storage_prefix();
		// AssetTypeId
		let asset_type_id_storage_prefix = AssetTypeId::<T>::storage_prefix();
		// AssetTypeUnitsPerSecond
		let units_per_second_storage_prefix = AssetTypeUnitsPerSecond::<T>::storage_prefix();
		// SupportedFeePaymentAssets
		let supported_fee_storage_prefix = SupportedFeePaymentAssets::<T>::storage_prefix();

		let mut result: Vec<PreUpgradeState<T>> = Vec::new();

		// AssetIdType pre-upgrade data
		let asset_id_type_storage_data: Vec<_> = storage_key_iter::<
			T::AssetId,
			OldAssetType,
			Blake2_128Concat,
		>(module_prefix, asset_id_type_storage_prefix)
		.collect();
		result.push(PreUpgradeState::<T>::AssetIdType(
			asset_id_type_storage_data,
		));

		// AssetTypeId pre-upgrade data
		let asset_type_id_storage_data: Vec<_> = storage_key_iter::<
			OldAssetType,
			T::AssetId,
			Blake2_128Concat,
		>(module_prefix, asset_type_id_storage_prefix)
		.collect();
		result.push(PreUpgradeState::<T>::AssetTypeId(
			asset_type_id_storage_data,
		));

		// AssetTypeUnitsPerSecond pre-upgrade data
		let units_per_second_storage_data: Vec<_> =
			storage_key_iter::<OldAssetType, u128, Blake2_128Concat>(
				module_prefix,
				units_per_second_storage_prefix,
			)
			.collect();
		result.push(PreUpgradeState::<T>::AssetTypeUnitsPerSecond(
			units_per_second_storage_data,
		));

		// SupportedFeePaymentAssets pre-upgrade data
		let supported_fee_storage_data: Vec<_> = get_storage_value::<Vec<OldAssetType>>(
			module_prefix,
			supported_fee_storage_prefix,
			&[],
		)
		.expect("SupportedFeePaymentAssets value");
		result.push(PreUpgradeState::<T>::SupportedFeePaymentAssets(
			supported_fee_storage_data,
		));

		Ok(result.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		log::trace!(
			target: "XcmV2ToV3AssetManager",
			"Running XcmV2ToV3AssetManager post_upgrade hook"
		);
		let pre_upgrade_state: Vec<PreUpgradeState<T>> =
			Decode::decode(&mut &state[..]).expect("pre_upgrade provides a valid state; qed");

		// Shared module prefix
		let module_prefix = AssetIdType::<T>::module_prefix();
		// AssetIdType
		let asset_id_type_storage_prefix = AssetIdType::<T>::storage_prefix();
		// AssetTypeId
		let asset_type_id_storage_prefix = AssetTypeId::<T>::storage_prefix();
		// AssetTypeUnitsPerSecond
		let units_per_second_storage_prefix = AssetTypeUnitsPerSecond::<T>::storage_prefix();

		// First we convert pre-state to post-state. This is equivalent to what the migration
		// should do. If this conversion and the result of the migration match, we consider it a
		// success.
		let to_post_upgrade: Vec<PostUpgradeState<T>> = pre_upgrade_state
			.into_iter()
			.map(|value| value.into())
			.collect();

		// Because the order of the storage and the pre-upgrade vector is likely different,
		// we encode everything, which is easier to sort and compare.
		let mut expected_post_upgrade_state: Vec<Vec<u8>> = Vec::new();
		for item in to_post_upgrade.iter() {
			match item {
				// Vec<(T::AssetId, T::ForeignAssetType)>
				PostUpgradeState::AssetIdType(items) => {
					for inner in items.into_iter() {
						expected_post_upgrade_state.push(inner.encode())
					}
				}
				// Vec<(T::ForeignAssetType, T::AssetId)>
				PostUpgradeState::AssetTypeId(items) => {
					for inner in items.into_iter() {
						expected_post_upgrade_state.push(inner.encode())
					}
				}
				// Vec<(T::ForeignAssetType, u128)>
				PostUpgradeState::AssetTypeUnitsPerSecond(items) => {
					for inner in items.into_iter() {
						expected_post_upgrade_state.push(inner.encode())
					}
				}
				// Vec<T::ForeignAssetType>
				PostUpgradeState::SupportedFeePaymentAssets(items) => {
					for inner in items.into_iter() {
						expected_post_upgrade_state.push(inner.encode())
					}
				}
			}
		}

		// Then we retrieve the actual state after migration.
		let mut actual_post_upgrade_state: Vec<Vec<u8>> = Vec::new();

		// Actual AssetIdType post-upgrade data
		let asset_id_type_storage_data: Vec<_> = storage_key_iter::<
			T::AssetId,
			T::ForeignAssetType,
			Blake2_128Concat,
		>(module_prefix, asset_id_type_storage_prefix)
		.collect();
		for item in asset_id_type_storage_data.iter() {
			actual_post_upgrade_state.push(item.encode())
		}

		// Actual AssetTypeId post-upgrade data
		let asset_type_id_storage_data: Vec<_> = storage_key_iter::<
			T::ForeignAssetType,
			T::AssetId,
			Blake2_128Concat,
		>(module_prefix, asset_type_id_storage_prefix)
		.collect();
		for item in asset_type_id_storage_data.iter() {
			actual_post_upgrade_state.push(item.encode())
		}

		// Actual AssetTypeUnitsPerSecond post-upgrade data
		let units_per_second_storage_data: Vec<_> =
			storage_key_iter::<T::ForeignAssetType, u128, Blake2_128Concat>(
				module_prefix,
				units_per_second_storage_prefix,
			)
			.collect();
		for item in units_per_second_storage_data.iter() {
			actual_post_upgrade_state.push(item.encode())
		}

		// Actual SupportedFeePaymentAssets post-upgrade data
		let supported_fee_storage_data: Vec<_> = SupportedFeePaymentAssets::<T>::get();
		for item in supported_fee_storage_data.iter() {
			actual_post_upgrade_state.push(item.encode())
		}

		// Both state blobs are sorted.
		expected_post_upgrade_state.sort();
		actual_post_upgrade_state.sort();

		// Assert equality
		assert_eq!(expected_post_upgrade_state, actual_post_upgrade_state);

		Ok(())
	}
}
