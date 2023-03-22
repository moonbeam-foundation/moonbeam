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
use xcm::latest::prelude::*;
use frame_support::{
	pallet_prelude::PhantomData,
	storage::migration::storage_key_iter,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
	Blake2_128Concat,
	StoragePrefixedMap,
    Twox64Concat,
};
use parity_scale_codec::{Decode, Encode};
use xcm::v2::MultiLocation as OldMultiLocation;
use crate::{Config, DestinationAssetFeePerSecond, RemoteTransactInfoWithMaxWeight, TransactInfoWithWeightLimit};

#[cfg(feature = "try-runtime")]
#[derive(Clone, Eq, Debug, PartialEq, Encode, Decode)]
enum PreUpgradeState {
	TransactInfoWithWeightLimit(Vec<(OldMultiLocation, RemoteTransactInfoWithMaxWeight)>),
	DestinationAssetFeePerSecond(Vec<(OldMultiLocation, u128)>),
}

#[cfg(feature = "try-runtime")]
#[derive(Clone, Eq, Debug, PartialEq, Encode, Decode)]
enum PostUpgradeState {
	TransactInfoWithWeightLimit(Vec<(MultiLocation, RemoteTransactInfoWithMaxWeight)>),
	DestinationAssetFeePerSecond(Vec<(MultiLocation, u128)>),
}

#[cfg(feature = "try-runtime")]
impl From<PreUpgradeState> for PostUpgradeState {
	fn from(pre: PreUpgradeState) -> PostUpgradeState {
		match pre {
            PreUpgradeState::TransactInfoWithWeightLimit(items) => {
                let mut out: Vec<(MultiLocation, RemoteTransactInfoWithMaxWeight)> = Vec::new();
				for (old_key, value) in items.into_iter() {
					let new_key: MultiLocation = old_key.try_into()
						.expect("Multilocation v2 to v3");
					out.push((new_key, value));
				}
				PostUpgradeState::TransactInfoWithWeightLimit(out)
            },
            PreUpgradeState::DestinationAssetFeePerSecond(items) => {
                let mut out: Vec<(MultiLocation, u128)> = Vec::new();
				for (old_key, value) in items.into_iter() {
					let new_key: MultiLocation = old_key.try_into()
						.expect("Multilocation v2 to v3");
					out.push((new_key, value));
				}
				PostUpgradeState::DestinationAssetFeePerSecond(out)
            }
        }
    }
}

pub struct XcmV2ToV3XcmTransactor<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for XcmV2ToV3XcmTransactor<T> {
	fn on_runtime_upgrade() -> Weight {
		log::trace!(
			target: "XcmV2ToV3XcmTransactor",
			"Running XcmV2ToV3XcmTransactor migration"
		);
		// Migrates the pallet's storage from Xcm V2 to V3:
		//	- TransactInfoWithWeightLimit -> migrate map's key
		//	- DestinationAssetFeePerSecond -> migrate map's key

		// Shared module prefix
		let module_prefix = TransactInfoWithWeightLimit::<T>::module_prefix();
		// TransactInfoWithWeightLimit
		let transact_info_with_weight_limit = TransactInfoWithWeightLimit::<T>::storage_prefix();
		// DestinationAssetFeePerSecond
		let destination_asset_fee_per_second = DestinationAssetFeePerSecond::<T>::storage_prefix();

		// Db (read, write) count
		let mut db_weight_count: (u64, u64) = (0, 0);

		// Migrate `TransactInfoWithWeightLimit` key
		db_weight_count.0 += 1;
		let old_data = storage_key_iter::<OldMultiLocation, RemoteTransactInfoWithMaxWeight, Blake2_128Concat>(
			&module_prefix,
			transact_info_with_weight_limit,
		)
		.drain()
		.collect::<Vec<(OldMultiLocation, RemoteTransactInfoWithMaxWeight)>>();
		for (old_key, value) in old_data {
			db_weight_count.1 += 1;
			let new_key: MultiLocation = old_key.try_into().expect("Multilocation v2 to v3");
			TransactInfoWithWeightLimit::<T>::insert(new_key, value);
		}

		// Migrate `DestinationAssetFeePerSecond` key
		db_weight_count.0 += 1;
		let old_data = storage_key_iter::<OldMultiLocation, u128, Twox64Concat>(
			&module_prefix,
			destination_asset_fee_per_second,
		)
		.drain()
		.collect::<Vec<(OldMultiLocation, u128)>>();
		for (old_key, value) in old_data {
			db_weight_count.1 += 1;
			let new_key: MultiLocation = old_key.try_into().expect("Multilocation v2 to v3");
			DestinationAssetFeePerSecond::<T>::insert(new_key, value);
		}

		T::DbWeight::get().reads_writes(db_weight_count.0, db_weight_count.1)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		log::trace!(
			target: "XcmV2ToV3XcmTransactor",
			"Running XcmV2ToV3XcmTransactor pre_upgrade hook"
		);
		// Shared module prefix
		let module_prefix = TransactInfoWithWeightLimit::<T>::module_prefix();
		// TransactInfoWithWeightLimit
		let transact_info_with_weight_limit = TransactInfoWithWeightLimit::<T>::storage_prefix();
		// DestinationAssetFeePerSecond
		let destination_asset_fee_per_second = DestinationAssetFeePerSecond::<T>::storage_prefix();

		let mut result: Vec<(u32, PreUpgradeState)> = Vec::new();

		// TransactInfoWithWeightLimit pre-upgrade data
		let transact_info_with_weight_limit_storage_data: Vec<_> = storage_key_iter::<OldMultiLocation, RemoteTransactInfoWithMaxWeight, Blake2_128Concat>(
			module_prefix,
			transact_info_with_weight_limit,
		)
		.collect();
		result.push((transact_info_with_weight_limit_storage_data.len() as u32, PreUpgradeState::TransactInfoWithWeightLimit(transact_info_with_weight_limit_storage_data)));

		// DestinationAssetFeePerSecond pre-upgrade data
		let destination_asset_fee_per_second_storage_data: Vec<_> = storage_key_iter::<OldMultiLocation, u128, Twox64Concat>(
			module_prefix,
			destination_asset_fee_per_second,
		)
		.collect();
		result.push((destination_asset_fee_per_second_storage_data.len() as u32, PreUpgradeState::DestinationAssetFeePerSecond(destination_asset_fee_per_second_storage_data)));

		Ok(result.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		log::trace!(
			target: "XcmV2ToV3XcmTransactor",
			"Running XcmV2ToV3XcmTransactor post_upgrade hook"
		);
		let pre_upgrade_state: Vec<(u32, PreUpgradeState)> =
			Decode::decode(&mut &state[..]).expect("pre_upgrade provides a valid state; qed");

		// Shared module prefix
		let module_prefix = TransactInfoWithWeightLimit::<T>::module_prefix();
		// TransactInfoWithWeightLimit
		let transact_info_with_weight_limit = TransactInfoWithWeightLimit::<T>::storage_prefix();
		// DestinationAssetFeePerSecond
		let destination_asset_fee_per_second = DestinationAssetFeePerSecond::<T>::storage_prefix();

		// Expected post-upgrade
		let expected_post_upgrade_state: Vec<(u32, PostUpgradeState)> = pre_upgrade_state
			.into_iter()
			.map(|(item_count, value)| (item_count, value.into()))
			.collect();

		let mut actual_post_upgrade_state: Vec<(u32, PostUpgradeState)> = Vec::new();

		// Actual TransactInfoWithWeightLimit post-upgrade data
		let transact_info_with_weight_limit_storage_data: Vec<_> = storage_key_iter::<MultiLocation, RemoteTransactInfoWithMaxWeight, Blake2_128Concat>(
			module_prefix,
			transact_info_with_weight_limit,
		)
		.collect();
		actual_post_upgrade_state.push((transact_info_with_weight_limit_storage_data.len() as u32, PostUpgradeState::TransactInfoWithWeightLimit(transact_info_with_weight_limit_storage_data)));


		// Actual DestinationAssetFeePerSecond post-upgrade data
		let destination_asset_fee_per_second_storage_data: Vec<_> = storage_key_iter::<MultiLocation, u128, Twox64Concat>(
			module_prefix,
			destination_asset_fee_per_second,
		)
		.collect();
		actual_post_upgrade_state.push((destination_asset_fee_per_second_storage_data.len() as u32, PostUpgradeState::DestinationAssetFeePerSecond(destination_asset_fee_per_second_storage_data)));

		// Assert
		assert_eq!(expected_post_upgrade_state.encode(), actual_post_upgrade_state.encode());

		Ok(())
	}
}
