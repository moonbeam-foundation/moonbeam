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

use crate::{
	Config, DestinationAssetFeePerSecond, RemoteTransactInfoWithMaxWeight,
	TransactInfoWithWeightLimit, XcmV2Weight,
};
use frame_support::{
	pallet_prelude::PhantomData,
	storage::migration::storage_key_iter,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
	Blake2_128Concat,
};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use xcm::latest::MultiLocation;
//TODO sometimes this is unused, sometimes its necessary
use sp_std::vec::Vec;

// This is the old storage, we write it here so that we know how to decode existing data
#[derive(Default, Clone, Encode, Decode, RuntimeDebug, PartialEq, scale_info::TypeInfo)]
pub struct OldRemoteTransactInfo {
	/// Extra weight that transacting a call in a destination chain adds
	pub transact_extra_weight: Weight,
	/// Fee per call byte
	pub fee_per_byte: u128,
	/// Size of the tx metadata of a transaction in the destination chain
	pub metadata_size: u64,
	/// Minimum weight the destination chain charges for a transaction
	pub base_weight: XcmV2Weight,
	/// Fee per weight in the destination chain
	pub fee_per_weight: u128,
}

// This is the old storage, we write it here so that we know how to decode existing data
#[derive(Default, Clone, Encode, Decode, RuntimeDebug, PartialEq, scale_info::TypeInfo)]
pub struct OldRemoteTransactInfoWithFeePerSecond {
	/// Extra weight that transacting a call in a destination chain adds
	pub transact_extra_weight: XcmV2Weight,
	/// Fee per second
	pub fee_per_second: u128,
	/// MaxWeight
	pub max_weight: u64,
}

/*pub struct MaxTransactWeight<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MaxTransactWeight<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "MaxTransactWeight", "actually running it");
		let pallet_prefix: &[u8] = b"XcmTransactor";
		let storage_item_prefix: &[u8] = b"TransactInfo";

		// Read all the data into memory.
		// https://crates.parity.io/frame_support/storage/migration/fn.storage_key_iter.html
		let stored_data: Vec<_> = storage_key_iter::<
			MultiLocation,
			OldRemoteTransactInfo,
			Blake2_128Concat,
		>(pallet_prefix, storage_item_prefix)
		.collect();

		let migrated_count: Weight = stored_data
			.len()
			.try_into()
			.expect("There are between 0 and 2**64 mappings stored.");

		log::info!(target: "MaxTransactWeight", "Migrating {:?} elements", migrated_count);

		// Now remove the old storage
		// https://crates.parity.io/frame_support/storage/migration/fn.remove_storage_prefix.html
		remove_storage_prefix(pallet_prefix, storage_item_prefix, &[]);

		// Assert that old storage is empty
		assert!(
			storage_key_iter::<MultiLocation, OldRemoteTransactInfo, Blake2_128Concat>(
				pallet_prefix,
				storage_item_prefix
			)
			.next()
			.is_none()
		);

		// Write to the new storage with removed and added fields
		for (location, info) in stored_data {
			TransactInfoWithWeightLimit::<T>::insert(location, {
				RemoteTransactInfoWithMaxWeight {
					transact_extra_weight: info.transact_extra_weight,
					/// Fee per weight in the destination chain
					/// Make sure the new one reflects per second, and not per weight unit
					fee_per_second: info
						.fee_per_weight
						.saturating_mul(WEIGHT_PER_SECOND as u128),
					/// Max destination weight
					max_weight: 20000000000,
					transact_extra_weight_signed: None,
				}
			});
		}

		log::info!(target: "MaxTransactWeight", "almost done");

		// Return the weight used. For each migrated mapping there is a red to get it into
		// memory, a write to clear the old stored value, and a write to re-store it.
		let db_weights = T::DbWeight::get();
		migrated_count.saturating_mul(2 * db_weights.write + db_weights.read)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;

		let pallet_prefix: &[u8] = b"XcmTransactor";
		let storage_item_prefix: &[u8] = b"TransactInfo";

		// We want to test that:
		// There are no entries in the new storage beforehand
		// The same number of mappings exist before and after
		// As long as there are some mappings stored, one representative key maps to the
		// same value after the migration.
		// There are no entries in the old storage afterward

		// Assert new storage is empty
		assert!(TransactInfoWithWeightLimit::<T>::iter().next().is_none());

		// Check number of entries, and set it aside in temp storage
		let stored_data: Vec<_> = storage_key_iter::<
			MultiLocation,
			OldRemoteTransactInfo,
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

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;

		// Check number of entries matches what was set aside in pre_upgrade
		let old_mapping_count: u64 = Self::get_temp_storage("mapping_count")
			.expect("We stored a mapping count; it should be there; qed");
		let new_mapping_count = TransactInfoWithWeightLimit::<T>::iter().count() as u64;
		assert_eq!(old_mapping_count, new_mapping_count);

		// Check that our example pair is still well-mapped after the migration
		if new_mapping_count > 0 {
			let (account, original_info): (MultiLocation, OldRemoteTransactInfo) =
				Self::get_temp_storage("example_pair").expect("qed");
			let migrated_info = TransactInfoWithWeightLimit::<T>::get(account).expect("qed");
			// Check all the other params are equal
			assert_eq!(
				original_info.transact_extra_weight,
				migrated_info.transact_extra_weight
			);
			assert_eq!(
				original_info
					.fee_per_weight
					.saturating_mul(WEIGHT_PER_SECOND as u128),
				migrated_info.fee_per_second
			);
			assert_eq!(migrated_info.max_weight, 20000000000)
		}

		Ok(())
	}
}*/

pub struct TransactSignedWeightAndFeePerSecond<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for TransactSignedWeightAndFeePerSecond<T> {
	fn on_runtime_upgrade() -> Weight {
		log::info!(target: "TransactSignedWeightAndFeePerSecond", "actually running it");
		let pallet_prefix: &[u8] = b"XcmTransactor";
		let storage_item_prefix: &[u8] = b"TransactInfoWithWeightLimit";

		// Read all the data into memory.
		// https://crates.parity.io/frame_support/storage/migration/fn.storage_key_iter.html
		let stored_data: Vec<_> = storage_key_iter::<
			MultiLocation,
			OldRemoteTransactInfoWithFeePerSecond,
			Blake2_128Concat,
		>(pallet_prefix, storage_item_prefix)
		.collect();

		let migrated_count = stored_data.len() as u64;

		log::info!(
			target: "TransactSignedWeightAndFeePerSecond", "Migrating {:?} elements", migrated_count);

		// Write to storage with removed and added fields
		for (location, info) in stored_data {
			TransactInfoWithWeightLimit::<T>::insert(&location, {
				RemoteTransactInfoWithMaxWeight {
					transact_extra_weight: info.transact_extra_weight,
					/// Max destination weight
					max_weight: info.max_weight,
					transact_extra_weight_signed: None,
				}
			});

			DestinationAssetFeePerSecond::<T>::insert(&location, info.fee_per_second);
		}

		log::info!(target: "MaxTransactWeight", "almost done");

		// Return the weight used. For each migrated mapping there is a red to get it into
		// memory, a write to clear the old stored value, and a write to re-store it.
		let db_weights = T::DbWeight::get();
		db_weights.reads_writes(migrated_count, migrated_count.saturating_add(2))
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;

		let pallet_prefix: &[u8] = b"XcmTransactor";
		let storage_item_prefix: &[u8] = b"TransactInfoWithWeightLimit";

		// We want to test that:
		// The same number of mappings exist before and after
		// As long as there are some mappings stored, one representative key maps to the
		// same value after the migration.
		// There are no entries in the new DestinationAssetFeePerSecond beforehand

		// Assert DestinationAssetFeePerSecond storage is empty
		assert!(DestinationAssetFeePerSecond::<T>::iter().next().is_none());

		// Check number of entries, and set it aside in temp storage
		let stored_data: Vec<_> = storage_key_iter::<
			MultiLocation,
			OldRemoteTransactInfo,
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

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		use frame_support::traits::OnRuntimeUpgradeHelpersExt;

		// Check number of entries matches what was set aside in pre_upgrade
		let old_mapping_count: u64 = Self::get_temp_storage("mapping_count")
			.expect("We stored a mapping count; it should be there; qed");
		let new_mapping_count_transact_info =
			TransactInfoWithWeightLimit::<T>::iter().count() as u64;
		let new_mapping_count_fee_per_second =
			DestinationAssetFeePerSecond::<T>::iter().count() as u64;

		assert_eq!(old_mapping_count, new_mapping_count_transact_info);
		assert_eq!(old_mapping_count, new_mapping_count_fee_per_second);

		// Check that our example pair is still well-mapped after the migration
		if new_mapping_count_transact_info > 0 {
			let (location, original_info): (MultiLocation, OldRemoteTransactInfoWithFeePerSecond) =
				Self::get_temp_storage("example_pair").expect("qed");
			let migrated_info_transact_info =
				TransactInfoWithWeightLimit::<T>::get(&location).expect("qed");
			let migrated_info_fee_per_second =
				DestinationAssetFeePerSecond::<T>::get(&location).expect("qed");
			// Check all the other params are equal
			assert_eq!(
				original_info.transact_extra_weight,
				migrated_info_transact_info.transact_extra_weight
			);
			assert_eq!(
				original_info.max_weight,
				migrated_info_transact_info.max_weight
			);
			// Check that transact_extra_weight_signed is None
			assert_eq!(
				migrated_info_transact_info.transact_extra_weight_signed,
				None
			);
			assert_eq!(original_info.fee_per_second, migrated_info_fee_per_second);
		}

		Ok(())
	}
}
