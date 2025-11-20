// Copyright 2019-2025 PureStake Inc.
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

extern crate alloc;

use alloc::vec::Vec;

use frame_support::{traits::OnRuntimeUpgrade, weights::Weight};

use crate::*;

#[derive(
	Clone,
	PartialEq,
	Eq,
	parity_scale_codec::Decode,
	parity_scale_codec::Encode,
	sp_runtime::RuntimeDebug,
)]
/// Reserve information { account, percent_of_inflation }
pub struct OldParachainBondConfig<AccountId> {
	/// Account which receives funds intended for parachain bond
	pub account: AccountId,
	/// Percent of inflation set aside for parachain bond account
	pub percent: sp_runtime::Percent,
}

pub struct MigrateParachainBondConfig<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for MigrateParachainBondConfig<T> {
	fn on_runtime_upgrade() -> Weight {
		let (account, percent) = if let Some(config) =
			frame_support::storage::migration::get_storage_value::<
				OldParachainBondConfig<T::AccountId>,
			>(b"ParachainStaking", b"ParachainBondInfo", &[])
		{
			(config.account, config.percent)
		} else {
			return Weight::default();
		};

		let pbr = InflationDistributionAccount { account, percent };
		let treasury = InflationDistributionAccount::<T::AccountId>::default();
		let configs: InflationDistributionConfig<T::AccountId> = [pbr, treasury].into();

		//***** Start mutate storage *****//

		InflationDistributionInfo::<T>::put(configs);

		// Remove storage value ParachainStaking::ParachainBondInfo
		frame_support::storage::unhashed::kill(&frame_support::storage::storage_prefix(
			b"ParachainStaking",
			b"ParachainBondInfo",
		));

		Weight::default()
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		use frame_support::ensure;
		use parity_scale_codec::Encode;

		let state = frame_support::storage::migration::get_storage_value::<
			OldParachainBondConfig<T::AccountId>,
		>(b"ParachainStaking", b"ParachainBondInfo", &[]);

		ensure!(state.is_some(), "State not found");

		Ok(state
			.expect("should be Some(_) due to former call to ensure!")
			.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		use frame_support::ensure;

		let old_state: OldParachainBondConfig<T::AccountId> =
			parity_scale_codec::Decode::decode(&mut &state[..])
				.map_err(|_| sp_runtime::DispatchError::Other("Failed to decode old state"))?;

		let new_state = InflationDistributionInfo::<T>::get();

		let pbr = InflationDistributionAccount {
			account: old_state.account,
			percent: old_state.percent,
		};
		let treasury = InflationDistributionAccount::<T::AccountId>::default();
		let expected_new_state: InflationDistributionConfig<T::AccountId> = [pbr, treasury].into();

		ensure!(new_state == expected_new_state, "State migration failed");

		Ok(())
	}
}

/// Migration to move `DelegationScheduledRequests` from a single `StorageMap` keyed by collator
/// into a `StorageDoubleMap` keyed by (collator, delegator) and to initialize the per-collator
/// counter `DelegationScheduledRequestsPerCollator`.
///
/// This assumes the on-chain data was written with the old layout where:
/// - Storage key: ParachainStaking::DelegationScheduledRequests
/// - Value type: BoundedVec<ScheduledRequest<..>, AddGet<MaxTop, MaxBottom>>
pub struct MigrateDelegationScheduledRequestsToDoubleMap<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateDelegationScheduledRequestsToDoubleMap<T> {
	fn on_runtime_upgrade() -> Weight {
		use frame_support::storage::migration::storage_key_iter;

		type OldScheduledRequests<T> = frame_support::BoundedVec<
			ScheduledRequest<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
			AddGet<
				<T as pallet::Config>::MaxTopDelegationsPerCandidate,
				<T as pallet::Config>::MaxBottomDelegationsPerCandidate,
			>,
		>;

		// Collect all existing entries under the old layout.
		let mut entries: Vec<(
			<T as frame_system::Config>::AccountId,
			OldScheduledRequests<T>,
		)> = Vec::new();

		for (collator, requests) in storage_key_iter::<
			<T as frame_system::Config>::AccountId,
			OldScheduledRequests<T>,
			frame_support::Blake2_128Concat,
		>(b"ParachainStaking", b"DelegationScheduledRequests")
		{
			entries.push((collator, requests));
		}

		// Clear all existing keys for DelegationScheduledRequests to avoid mixing
		// old layout entries with the new double-map layout.
		//
		// We use the low-level `sp_io::storage::clear_prefix` on the full storage prefix so
		// that *all* existing keys under `ParachainStaking::DelegationScheduledRequests`
		// (from the legacy single-map layout) are removed before we insert the new keys for
		// the double-map layout.
		let prefix = frame_support::storage::storage_prefix(
			b"ParachainStaking",
			b"DelegationScheduledRequests",
		);
		let _ = frame_support::storage::unhashed::clear_prefix(&prefix, None, None);

		// Rebuild storage using the new layout and initialize the per-collator counters.
		for (collator, old_requests) in entries.into_iter() {
			let mut per_collator_count: u32 = 0;

			for request in old_requests.into_iter() {
				let delegator = request.delegator.clone();
				let mut new_vec: frame_support::BoundedVec<
					ScheduledRequest<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
					frame_support::traits::ConstU32<50>,
				> = Default::default();

				if new_vec.try_push(request).is_err() {
					// This should not happen since we only ever push a single element.
					continue;
				}

				DelegationScheduledRequests::<T>::insert(&collator, &delegator, new_vec);
				per_collator_count = per_collator_count.saturating_add(1);
			}

			if per_collator_count > 0 {
				DelegationScheduledRequestsPerCollator::<T>::insert(&collator, per_collator_count);
			}
		}

		Weight::zero()
	}
}
