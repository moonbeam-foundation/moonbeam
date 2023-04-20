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
	TransactInfoWithWeightLimit,
};
use frame_support::{
	pallet_prelude::PhantomData,
	storage::migration::storage_key_iter,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
	Blake2_128Concat, StoragePrefixedMap, Twox64Concat,
};
use parity_scale_codec::{Decode, Encode};
use sp_std::vec::Vec;
use xcm::latest::prelude::*;
use xcm::v2::MultiLocation as OldMultiLocation;
use xcm_primitives::DEFAULT_PROOF_SIZE;

#[cfg(feature = "try-runtime")]
#[derive(Clone, Eq, Debug, PartialEq, Encode, Decode)]
enum PreUpgradeState {
	TransactInfoWithWeightLimit(Vec<(OldMultiLocation, OldRemoteTransactInfoWithMaxWeight)>),
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
				for (old_key, old_value) in items.into_iter() {
					let new_key: MultiLocation =
						old_key.try_into().expect("Multilocation v2 to v3");
					let new_value: RemoteTransactInfoWithMaxWeight = old_value.into();
					out.push((new_key, new_value));
				}
				PostUpgradeState::TransactInfoWithWeightLimit(out)
			}
			PreUpgradeState::DestinationAssetFeePerSecond(items) => {
				let mut out: Vec<(MultiLocation, u128)> = Vec::new();
				for (old_key, value) in items.into_iter() {
					let new_key: MultiLocation =
						old_key.try_into().expect("Multilocation v2 to v3");
					out.push((new_key, value));
				}
				PostUpgradeState::DestinationAssetFeePerSecond(out)
			}
		}
	}
}

#[derive(Default, Debug, Clone, Encode, Decode, Eq, PartialEq)]
pub struct OldRemoteTransactInfoWithMaxWeight {
	pub transact_extra_weight: u64,
	pub max_weight: u64,
	pub transact_extra_weight_signed: Option<u64>,
}

impl From<OldRemoteTransactInfoWithMaxWeight> for RemoteTransactInfoWithMaxWeight {
	fn from(old: OldRemoteTransactInfoWithMaxWeight) -> RemoteTransactInfoWithMaxWeight {
		// This only accounts for everything outside the Transact instruction
		// A.K.A. outside require weight at most.
		let transact_extra_weight: Weight = Weight::from_parts(
			old.transact_extra_weight,
			DEFAULT_PROOF_SIZE.saturating_div(2),
		);
		let max_weight: Weight = Weight::from_parts(
			old.max_weight,
			cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64,
		);
		let transact_extra_weight_signed: Option<Weight> =
			if let Some(w) = old.transact_extra_weight_signed {
				Some(Weight::from_parts(w, DEFAULT_PROOF_SIZE))
			} else {
				None
			};
		RemoteTransactInfoWithMaxWeight {
			transact_extra_weight,
			max_weight,
			transact_extra_weight_signed,
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

		// Migrate both `TransactInfoWithWeightLimit` key and value
		db_weight_count.0 += 1;
		let old_data = storage_key_iter::<
			OldMultiLocation,
			OldRemoteTransactInfoWithMaxWeight,
			Blake2_128Concat,
		>(&module_prefix, transact_info_with_weight_limit)
		.drain()
		.collect::<Vec<(OldMultiLocation, OldRemoteTransactInfoWithMaxWeight)>>();
		for (old_key, old_value) in old_data {
			db_weight_count.1 += 1;
			let new_key: MultiLocation = old_key.try_into().expect("Multilocation v2 to v3");
			let new_value: RemoteTransactInfoWithMaxWeight = old_value.into();
			TransactInfoWithWeightLimit::<T>::insert(new_key, new_value);
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

		let mut result: Vec<PreUpgradeState> = Vec::new();

		// TransactInfoWithWeightLimit pre-upgrade data
		let transact_info_with_weight_limit_storage_data: Vec<_> =
			storage_key_iter::<
				OldMultiLocation,
				OldRemoteTransactInfoWithMaxWeight,
				Blake2_128Concat,
			>(module_prefix, transact_info_with_weight_limit)
			.collect();
		result.push(PreUpgradeState::TransactInfoWithWeightLimit(
			transact_info_with_weight_limit_storage_data,
		));

		// DestinationAssetFeePerSecond pre-upgrade data
		let destination_asset_fee_per_second_storage_data: Vec<_> =
			storage_key_iter::<OldMultiLocation, u128, Twox64Concat>(
				module_prefix,
				destination_asset_fee_per_second,
			)
			.collect();
		result.push(PreUpgradeState::DestinationAssetFeePerSecond(
			destination_asset_fee_per_second_storage_data,
		));

		Ok(result.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
		log::trace!(
			target: "XcmV2ToV3XcmTransactor",
			"Running XcmV2ToV3XcmTransactor post_upgrade hook"
		);
		let pre_upgrade_state: Vec<PreUpgradeState> =
			Decode::decode(&mut &state[..]).expect("pre_upgrade provides a valid state; qed");

		// Shared module prefix
		let module_prefix = TransactInfoWithWeightLimit::<T>::module_prefix();
		// TransactInfoWithWeightLimit
		let transact_info_with_weight_limit = TransactInfoWithWeightLimit::<T>::storage_prefix();
		// DestinationAssetFeePerSecond
		let destination_asset_fee_per_second = DestinationAssetFeePerSecond::<T>::storage_prefix();

		// First we convert pre-state to post-state. This is equivalent to what the migration
		// should do. If this conversion and the result of the migration match, we consider it a
		// success.
		let to_post_upgrade: Vec<PostUpgradeState> = pre_upgrade_state
			.into_iter()
			.map(|value| value.into())
			.collect();

		// Because the order of the storage and the pre-upgrade vector is likely different,
		// we encode everything, which is easier to sort and compare.
		let mut expected_post_upgrade_state: Vec<Vec<u8>> = Vec::new();
		for item in to_post_upgrade.iter() {
			match item {
				PostUpgradeState::TransactInfoWithWeightLimit(items) => {
					for inner in items.into_iter() {
						expected_post_upgrade_state.push(inner.encode())
					}
				}
				PostUpgradeState::DestinationAssetFeePerSecond(items) => {
					for inner in items.into_iter() {
						expected_post_upgrade_state.push(inner.encode())
					}
				}
			}
		}

		let mut actual_post_upgrade_state: Vec<Vec<u8>> = Vec::new();

		// Actual TransactInfoWithWeightLimit post-upgrade data
		let transact_info_with_weight_limit_storage_data: Vec<_> =
			storage_key_iter::<MultiLocation, RemoteTransactInfoWithMaxWeight, Blake2_128Concat>(
				module_prefix,
				transact_info_with_weight_limit,
			)
			.collect();
		for item in transact_info_with_weight_limit_storage_data.iter() {
			actual_post_upgrade_state.push(item.encode())
		}

		// Actual DestinationAssetFeePerSecond post-upgrade data
		let destination_asset_fee_per_second_storage_data: Vec<_> =
			storage_key_iter::<MultiLocation, u128, Twox64Concat>(
				module_prefix,
				destination_asset_fee_per_second,
			)
			.collect();
		for item in destination_asset_fee_per_second_storage_data.iter() {
			actual_post_upgrade_state.push(item.encode())
		}

		// Both state blobs are sorted.
		expected_post_upgrade_state.sort();
		actual_post_upgrade_state.sort();

		// Assert
		assert_eq!(expected_post_upgrade_state, actual_post_upgrade_state);

		Ok(())
	}
}
