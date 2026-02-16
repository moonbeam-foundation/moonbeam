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

/// Migration to populate `PendingRevocations` from existing `DelegationScheduledRequests`.
///
/// Iterates over all `(collator, delegator)` entries in `DelegationScheduledRequests` and
/// inserts a `PendingRevocations` flag for each pair where a `Revoke` action exists.
///
/// At any given time the number of pending revokes is small (order of hundreds), so a
/// stepped migration with a generous per-step budget suffices.
pub struct MigratePopulatePendingRevocations<T>(sp_std::marker::PhantomData<T>);

impl<T> SteppedMigration for MigratePopulatePendingRevocations<T>
where
	T: Config,
{
	/// Cursor stores the last processed (collator, delegator) storage key bytes.
	/// `None` means we have not started yet.
	type Cursor = frame_support::BoundedVec<u8, ConstU32<256>>;

	type Identifier = [u8; 16];

	fn id() -> Self::Identifier {
		*b"MB-PENDREV-MIG01"
	}

	fn step(
		cursor: Option<Self::Cursor>,
		meter: &mut WeightMeter,
	) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
		let db_weight = <T as frame_system::Config>::DbWeight::get();

		// Each entry: 1 read (iter next) + 1 read (decode value) + 1 write (insert flag)
		let weight_per_entry = db_weight.reads_writes(2, 1);

		if meter.remaining().all_lt(weight_per_entry) {
			return Err(SteppedMigrationError::InsufficientWeight {
				required: weight_per_entry,
			});
		}

		// Use at most 50% of remaining weight.
		let step_budget = meter.remaining().saturating_div(2);

		const MAX_ENTRIES_PER_STEP: u32 = 100;

		let prefix = frame_support::storage::storage_prefix(
			b"ParachainStaking",
			b"DelegationScheduledRequests",
		);

		let mut start_from: Vec<u8> = cursor
			.map(|c| c.to_vec())
			.unwrap_or_else(|| prefix.to_vec());

		let mut used = Weight::zero();
		let mut processed: u32 = 0;

		loop {
			let remaining = step_budget.saturating_sub(used);
			if weight_per_entry.any_gt(remaining) || processed >= MAX_ENTRIES_PER_STEP {
				break;
			}

			let Some(next_key) = sp_io::storage::next_key(&start_from) else {
				// No more keys — migration complete.
				if !used.is_zero() {
					meter.consume(used);
				}
				return Ok(None);
			};

			if !next_key.starts_with(&prefix) {
				// Passed the end of the DelegationScheduledRequests prefix.
				if !used.is_zero() {
					meter.consume(used);
				}
				return Ok(None);
			}

			// We only want double-map entries (collator, delegator). Single-map legacy
			// entries should have been cleaned up by the previous migration, but we
			// filter them out by checking that the key is long enough to contain two
			// Blake2_128Concat-encoded AccountIds after the prefix.
			let key_suffix = &next_key[prefix.len()..];

			// Decode first key (collator): 16-byte hash + AccountId
			if key_suffix.len() < 16 {
				start_from = next_key;
				continue;
			}
			let mut collator_bytes = &key_suffix[16..];
			let collator = match <<T as frame_system::Config>::AccountId as Decode>::decode(
				&mut collator_bytes,
			) {
				Ok(c) => c,
				Err(_) => {
					start_from = next_key;
					continue;
				}
			};

			// The remaining bytes after the collator should contain the delegator key.
			// If empty, this is a legacy single-map entry — skip it.
			if collator_bytes.is_empty() {
				start_from = next_key;
				continue;
			}

			// Decode second key (delegator): 16-byte hash + AccountId
			if collator_bytes.len() < 16 {
				start_from = next_key;
				continue;
			}
			let mut delegator_bytes = &collator_bytes[16..];
			let delegator = match <<T as frame_system::Config>::AccountId as Decode>::decode(
				&mut delegator_bytes,
			) {
				Ok(d) => d,
				Err(_) => {
					start_from = next_key;
					continue;
				}
			};

			// Read the value and check for any Revoke action.
			let requests = DelegationScheduledRequests::<T>::get(&collator, &delegator);
			let has_revoke = requests
				.iter()
				.any(|req| matches!(req.action, DelegationAction::Revoke(_)));

			if has_revoke {
				PendingRevocations::<T>::insert(&collator, &delegator, ());
			}

			used = used.saturating_add(weight_per_entry);
			processed += 1;
			start_from = next_key;
		}

		if !used.is_zero() {
			meter.consume(used);
			let bounded_key =
				frame_support::BoundedVec::<u8, ConstU32<256>>::truncate_from(start_from);
			Ok(Some(bounded_key))
		} else {
			Err(SteppedMigrationError::InsufficientWeight {
				required: weight_per_entry,
			})
		}
	}
}
