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

use sp_std::{vec::Vec, vec};
use frame_support::{
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

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
impl<T: Config> OnRuntimeUpgrade for MigrateCandidateBondLessRequests<T> {
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
