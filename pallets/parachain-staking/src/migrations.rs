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

use frame_support::{
	migrations::{SteppedMigration, SteppedMigrationError},
	pallet_prelude::{ConstU32, Zero},
	traits::{Get, OnRuntimeUpgrade},
	weights::{Weight, WeightMeter},
};
use parity_scale_codec::Decode;
use sp_io;

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

/// Migration to move `DelegationScheduledRequests` from a single `StorageMap` keyed by collator
/// into a `StorageDoubleMap` keyed by (collator, delegator) and to initialize the per-collator
/// counter `DelegationScheduledRequestsPerCollator`.
///
/// This assumes the on-chain data was written with the old layout where:
/// - Storage key: ParachainStaking::DelegationScheduledRequests
/// - Value type: BoundedVec<ScheduledRequest<..>, AddGet<MaxTop, MaxBottom>>
pub struct MigrateDelegationScheduledRequestsToDoubleMap<T>(sp_std::marker::PhantomData<T>);

impl<T> SteppedMigration for MigrateDelegationScheduledRequestsToDoubleMap<T>
where
	T: Config,
{
	/// Cursor keeps track of the last processed legacy storage key (the full
	/// storage key bytes for the legacy single-map entry). `None` means we have
	/// not processed any key yet.
	///
	/// Using a bounded vector keeps the on-chain cursor small while still being
	/// large enough to store the full key (prefix + hash + AccountId).
	type Cursor = frame_support::BoundedVec<u8, ConstU32<128>>;

	/// Identifier for this migration. Must be unique across all migrations.
	type Identifier = [u8; 16];

	fn id() -> Self::Identifier {
		// Arbitrary but fixed 16-byte identifier.
		*b"MB-DSR-MIG-00001"
	}

	fn step(
		cursor: Option<Self::Cursor>,
		meter: &mut WeightMeter,
	) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
		// NOTE: High-level algorithm
		// --------------------------
		// - We treat each invocation of `step` as having a fixed "budget"
		//   equal to at most 50% of the remaining weight in the `WeightMeter`.
		// - Within that budget we migrate as many *collators* (legacy map
		//   entries) as we can.
		// - For every collator we enforce two properties:
		//   1. Before we even read the legacy value from storage we ensure the
		//      remaining budget can pay for a *worst-case* collator.
		//   2. Once we know exactly how many requests `n` that collator has,
		//      we re-check the remaining budget against the *precise* cost for
		//      those `n` requests.
		// - Progress is tracked only by:
		//   * Removing legacy keys as they are migrated, and
		//   * Persisting the last processed legacy key in the `Cursor`. The
		//     next `step` resumes scanning directly after that key.
		/// Legacy scheduled request type used *only* for decoding the old single-map
		/// storage layout where the delegator was stored inside the value.
		#[derive(
			Clone,
			PartialEq,
			Eq,
			parity_scale_codec::Decode,
			parity_scale_codec::Encode,
			sp_runtime::RuntimeDebug,
		)]
		struct LegacyScheduledRequest<AccountId, Balance> {
			delegator: AccountId,
			when_executable: RoundIndex,
			action: DelegationAction<Balance>,
		}

		// Legacy value type under `ParachainStaking::DelegationScheduledRequests`.
		type OldScheduledRequests<T> = frame_support::BoundedVec<
			LegacyScheduledRequest<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
			AddGet<
				<T as pallet::Config>::MaxTopDelegationsPerCandidate,
				<T as pallet::Config>::MaxBottomDelegationsPerCandidate,
			>,
		>;

		// Upper bound for the number of legacy requests that can exist for a single
		// collator in the old layout.
		let max_requests_per_collator: u64 = <AddGet<
			<T as pallet::Config>::MaxTopDelegationsPerCandidate,
			<T as pallet::Config>::MaxBottomDelegationsPerCandidate,
		> as frame_support::traits::Get<u32>>::get() as u64;

		// Conservatively estimate the worst-case DB weight for migrating a single
		// legacy entry (one collator):
		//
		// - 1 read for the old value.
		// - For each request (up to max_requests_per_collator):
		//   - 1 read + 1 write for `DelegationScheduledRequests` (mutate).
		// - After migration of this collator:
		//   - Up to `max_requests_per_collator` reads when iterating the new
		//     double-map to compute the per-collator counter.
		//   - 1 write to set `DelegationScheduledRequestsPerCollator`.
		//   - 1 write to kill the old key.
		let db_weight = <T as frame_system::Config>::DbWeight::get();
		let worst_reads = 1 + 3 * max_requests_per_collator;
		let worst_writes = 2 * max_requests_per_collator + 2;
		let worst_per_collator = db_weight.reads_writes(worst_reads, worst_writes);

		// Safety margin baseline for this step: we will try to spend at most 50%
		// of the remaining block weight on this migration, but we only require
		// that the *full* remaining budget is sufficient to migrate one
		// worst-case collator. This avoids the situation where the 50% margin is
		// smaller than `worst_per_collator` (e.g. on production where
		// MaxTop/MaxBottom are much larger than in tests) and the migration
		// could never even start.
		let remaining = meter.remaining();
		if remaining.all_lt(worst_per_collator) {
			return Err(SteppedMigrationError::InsufficientWeight {
				required: worst_per_collator,
			});
		}
		let step_budget = remaining.saturating_div(2);

		// Hard cap on the number of collators we are willing to migrate in a
		// single step, regardless of the theoretical weight budget. This
		// prevents a single step from doing unbounded work even if the
		// `WeightMeter` is configured with a very large limit (for example in
		// testing), and keeps block execution times predictable on mainnet.
		const MAX_COLLATORS_PER_STEP: u32 = 8;

		let prefix = frame_support::storage::storage_prefix(
			b"ParachainStaking",
			b"DelegationScheduledRequests",
		);

		// Helper: find the next legacy (single-map) key after `start_from`.
		//
		// The key space is shared between the old single-map and the new
		// double-map under the same storage prefix:
		// - legacy:   Blake2_128Concat(collator)
		// - new:      Blake2_128Concat(collator) ++ Blake2_128Concat(delegator)
		//
		// We use the fact that legacy keys have *no* trailing bytes after the
		// collator AccountId, while new keys have at least one more encoded
		// component.
		fn next_legacy_key<T: Config>(
			prefix: &[u8],
			start_from: &[u8],
		) -> Option<(Vec<u8>, <T as frame_system::Config>::AccountId)> {
			let mut current = sp_io::storage::next_key(start_from)?;

			while current.starts_with(prefix) {
				// Strip the prefix and decode the first Blake2_128Concat-encoded key
				// which should correspond to the collator AccountId.
				let mut key_bytes = &current[prefix.len()..];

				// Must contain at least the 16 bytes of Blake2_128 hash.
				if key_bytes.len() < 16 {
					current = sp_io::storage::next_key(&current)?;
					continue;
				}

				// Skip the hash and decode the AccountId.
				key_bytes = &key_bytes[16..];
				let mut decoder = key_bytes;
				let maybe_collator =
					<<T as frame_system::Config>::AccountId as Decode>::decode(&mut decoder);

				if let Ok(collator) = maybe_collator {
					// If there are no remaining bytes, then this key corresponds to the
					// legacy single-map layout (one key per collator). If there *are*
					// remaining bytes, it is a new double-map key which we must skip.
					if decoder.is_empty() {
						return Some((current.clone(), collator));
					}
				}

				current = sp_io::storage::next_key(&current)?;
			}

			None
		}

		// Process as many legacy entries as possible within the per-step weight
		// budget. Progress is tracked by removing legacy keys from storage and
		// by persisting the last processed legacy key in the cursor, so the
		// next step can resume in O(1) reads.
		let mut used_in_step = Weight::zero();
		let mut processed_collators: u32 = 0;
		let mut start_from: Vec<u8> = cursor
			.map(|c| c.to_vec())
			.unwrap_or_else(|| prefix.to_vec());

		loop {
			let Some((full_key, collator)) = next_legacy_key::<T>(&prefix, &start_from) else {
				// No more legacy entries to migrate – we are done. Account for
				// the weight we actually used in this step.
				if !used_in_step.is_zero() {
					meter.consume(used_in_step);
				}
				return Ok(None);
			};

			// Decode the legacy value for this collator.
			let Some(bytes) = sp_io::storage::get(&full_key) else {
				// Nothing to migrate for this key; try the next one.
				start_from = full_key;
				continue;
			};

			let old_requests: OldScheduledRequests<T> =
				OldScheduledRequests::<T>::decode(&mut &bytes[..]).unwrap_or_default();

			let n = old_requests.len() as u64;
			// More precise weight estimate for this specific collator based on
			// the actual number of legacy requests `n`.
			let reads = 1 + 3 * n;
			let writes = 2 * n + 2;
			let weight_for_collator = db_weight.reads_writes(reads, writes);

			// Recompute remaining budget now that we know the precise weight
			// for this collator, and ensure we do not exceed the 50% per-step
			// safety margin.
			let remaining_budget = step_budget.saturating_sub(used_in_step);
			if weight_for_collator.any_gt(remaining_budget) {
				// Cannot fit this collator into the current block's budget.
				// Stop here and let the next step handle it.
				break;
			}

			// Rebuild storage using the new double-map layout for this collator.
			for request in old_requests.into_iter() {
				let delegator = request.delegator.clone();

				DelegationScheduledRequests::<T>::mutate(&collator, &delegator, |scheduled| {
					// This Error is safe to ignore given that in the current implementation we have at most one request per collator.
					let _ = scheduled.try_push(ScheduledRequest {
						when_executable: request.when_executable,
						action: request.action,
					});
				});
			}

			// Remove the legacy single-map key for this collator. This does *not* touch
			// the new double-map entries, which use longer keys under the same prefix.
			sp_io::storage::clear(&full_key);

			// Initialize the per-collator counter from the freshly migrated data: each
			// `(collator, delegator)` queued in the double map corresponds to one
			// delegator with at least one pending request towards this collator.
			let delegator_queues =
				DelegationScheduledRequests::<T>::iter_prefix(&collator).count() as u32;
			if delegator_queues > 0 {
				DelegationScheduledRequestsPerCollator::<T>::insert(&collator, delegator_queues);
			}

			used_in_step = used_in_step.saturating_add(weight_for_collator);
			start_from = full_key;
			processed_collators = processed_collators.saturating_add(1);

			// Always stop after a bounded number of collators, even if the
			// weight budget would allow more. The remaining work will be picked
			// up in the next step.
			if processed_collators >= MAX_COLLATORS_PER_STEP {
				break;
			}
		}

		if !used_in_step.is_zero() {
			meter.consume(used_in_step);
			let bounded_key =
				frame_support::BoundedVec::<u8, ConstU32<128>>::truncate_from(start_from);
			Ok(Some(bounded_key))
		} else {
			// We had enough theoretical budget but could not fit even a single
			// collator with the more precise estimate. Signal insufficient weight.
			Err(SteppedMigrationError::InsufficientWeight {
				required: worst_per_collator,
			})
		}
	}
}
