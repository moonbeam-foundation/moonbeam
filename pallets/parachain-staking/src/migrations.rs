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

use frame_support::{
	pallet_prelude::StorageVersion,
	traits::{Get, UncheckedOnRuntimeUpgrade},
	weights::Weight,
};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::{vec, vec::Vec};

use crate::*;

/// The in-code storage version.
pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

/// Old version of CandidateMetadata with single bond less request
#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct OldCandidateMetadata<Balance> {
	pub bond: Balance,
	pub delegation_count: u32,
	pub total_counted: Balance,
	pub lowest_top_delegation_amount: Balance,
	pub highest_bottom_delegation_amount: Balance,
	pub lowest_bottom_delegation_amount: Balance,
	pub top_capacity: CapacityStatus,
	pub bottom_capacity: CapacityStatus,
	pub request: Option<CandidateBondLessRequest<Balance>>,
	pub status: CollatorStatus,
}

/// Migration to convert single bond less request to multiple requests
pub struct MigrateCandidateBondLessRequests<T>(sp_std::marker::PhantomData<T>);
impl<T: Config> UncheckedOnRuntimeUpgrade for MigrateCandidateBondLessRequests<T> {
	fn on_runtime_upgrade() -> Weight {
		let mut reads = 0u64;
		let mut writes = 0u64;

		CandidateInfo::<T>::translate(|_key, old: OldCandidateMetadata<BalanceOf<T>>| {
			reads = reads.saturating_add(1);
			writes = writes.saturating_add(1);

			// Convert old single request to vector
			let bond_less_requests = if let Some(request) = old.request {
				vec![request]
			} else {
				Vec::new()
			};

			Some(CandidateMetadata {
				bond: old.bond,
				delegation_count: old.delegation_count,
				total_counted: old.total_counted,
				lowest_top_delegation_amount: old.lowest_top_delegation_amount,
				highest_bottom_delegation_amount: old.highest_bottom_delegation_amount,
				lowest_bottom_delegation_amount: old.lowest_bottom_delegation_amount,
				top_capacity: old.top_capacity,
				bottom_capacity: old.bottom_capacity,
				bond_less_requests,
				status: old.status,
			})
		});

		<T as frame_system::Config>::DbWeight::get().reads_writes(reads, writes)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::DispatchError> {
		let count = CandidateInfo::<T>::iter().count() as u32;
		Ok(count.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::DispatchError> {
		let expected_count: u32 = Decode::decode(&mut &state[..])
			.map_err(|_| sp_runtime::DispatchError::Other("Failed to decode pre-upgrade state"))?;
		let actual_count = CandidateInfo::<T>::iter().count() as u32;
		frame_support::ensure!(
			expected_count == actual_count,
			"CandidateInfo count mismatch after migration"
		);
		Ok(())
	}
}

pub type MigrateToV1<T> = frame_support::migrations::VersionedMigration<
	0,
	1,
	MigrateCandidateBondLessRequests<T>,
	crate::pallet::Pallet<T>,
	<T as frame_system::Config>::DbWeight,
>;
