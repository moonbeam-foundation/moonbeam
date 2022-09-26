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

//! Auto-compounding functionality for staking rewards

use crate::pallet::{
	AutoCompoundingInfo, CandidateInfo, Config, DelegatorState, Error, Event, Pallet,
};
use frame_support::ensure;
use frame_support::{dispatch::DispatchResultWithPostInfo, RuntimeDebug};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::Percent;
use sp_std::{vec, vec::Vec};

/// Represents the auto-compounding amount for a delegation.
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, PartialOrd, Ord)]
pub struct AutoCompoundingDelegation<AccountId> {
	pub delegator: AccountId,
	pub value: Percent,
}

impl<AccountId> AutoCompoundingDelegation<AccountId>
where
	AccountId: Eq,
{
	/// Create a new [AutoCompoundingDelegation] object.
	pub fn new(delegator: AccountId) -> Self {
		AutoCompoundingDelegation {
			delegator,
			value: Percent::zero(),
		}
	}
}

/// Represents the auto-compounding amount for a collator and its delegations.
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, PartialOrd, Ord)]
pub struct AutoCompounding<AccountId> {
	pub candidate: AccountId,
	pub value: Percent,
	pub delegations: Vec<AutoCompoundingDelegation<AccountId>>,
}

impl<AccountId> AutoCompounding<AccountId>
where
	AccountId: Eq,
{
	/// Create a new [AutoCompounding] object.
	pub fn new(candidate: AccountId) -> Self {
		AutoCompounding {
			candidate,
			value: Percent::zero(),
			delegations: vec![],
		}
	}

	/// Sets the auto-compounding value for a delegation.
	pub fn set_delegation_value(&mut self, delegator: AccountId, value: Percent) {
		let maybe_delegation = self
			.delegations
			.iter_mut()
			.find(|entry| entry.delegator == delegator);

		let mut delegation = if let Some(delegation) = maybe_delegation {
			delegation
		} else {
			let new_entry = AutoCompoundingDelegation::new(delegator);
			self.delegations.push(new_entry);
			self.delegations.last_mut().expect("cannot fail; qed")
		};

		delegation.value = value;
	}

	/// Removes the auto-compounding value for a delegation.
	pub fn remove_delegation_value(&mut self, delegator: &AccountId) {
		if let Some(index) = self
			.delegations
			.iter()
			.position(|entry| &entry.delegator == delegator)
		{
			self.delegations.remove(index);
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Sets the auto-compounding value for a candidate.
	pub(crate) fn candidate_set_auto_compounding(
		candidate: T::AccountId,
		value: Percent,
	) -> DispatchResultWithPostInfo {
		ensure!(
			<CandidateInfo<T>>::get(&candidate).is_some(),
			<Error<T>>::CandidateDNE,
		);

		let mut state = <AutoCompoundingInfo<T>>::get(&candidate)
			.unwrap_or_else(|| AutoCompounding::new(candidate.clone()));
		state.value = value;

		<AutoCompoundingInfo<T>>::insert(candidate.clone(), state);
		Self::deposit_event(Event::CandidateAutoCompoundingSet { candidate, value });

		Ok(().into())
	}

	/// Sets the auto-compounding value for a delegation.
	pub(crate) fn delegation_set_auto_compounding(
		candidate: T::AccountId,
		delegator: T::AccountId,
		value: Percent,
		delegation_count_hint: u32,
		candidate_auto_compounding_delegation_count_hint: u32,
	) -> DispatchResultWithPostInfo {
		let delegator_state =
			<DelegatorState<T>>::get(&delegator).ok_or(<Error<T>>::DelegatorDNE)?;
		ensure!(
			delegator_state.delegations.0.len() <= delegation_count_hint as usize,
			<Error<T>>::TooLowDelegationCountToAutoCompound,
		);
		ensure!(
			delegator_state
				.delegations
				.0
				.iter()
				.any(|b| b.owner == candidate),
			<Error<T>>::DelegationDNE,
		);

		let mut state = <AutoCompoundingInfo<T>>::get(&candidate)
			.unwrap_or_else(|| AutoCompounding::new(candidate.clone()));

		ensure!(
			state.delegations.len() <= candidate_auto_compounding_delegation_count_hint as usize,
			<Error<T>>::TooLowCandidateAutoCompoundingDelegationCountToAutoCompound,
		);
		state.set_delegation_value(delegator.clone(), value);
		<AutoCompoundingInfo<T>>::insert(candidate.clone(), state);
		Self::deposit_event(Event::DelegationAutoCompoundingSet {
			candidate,
			delegator,
			value,
		});

		Ok(().into())
	}

	/// Removes the auto-compounding value for a delegation. This should be called when the
	/// delegation is revoked to cleanup storage.
	pub(crate) fn delegation_remove_auto_compounding(
		candidate: &T::AccountId,
		delegator: &T::AccountId,
	) {
		if let Some(mut state) = <AutoCompoundingInfo<T>>::get(candidate) {
			state.remove_delegation_value(delegator);
			<AutoCompoundingInfo<T>>::insert(candidate, state);
		}
	}
}
