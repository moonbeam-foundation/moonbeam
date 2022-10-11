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
	AutoCompoundingDelegations, BalanceOf, CandidateInfo, Config, DelegatorState, Error, Event,
	Pallet, Total,
};
use crate::types::{Bond, BondAdjust, Delegator};
use frame_support::ensure;
use frame_support::traits::Get;
use frame_support::{dispatch::DispatchResultWithPostInfo, RuntimeDebug};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::traits::Saturating;
use sp_runtime::Percent;
use sp_std::vec::Vec;

/// Represents the auto-compounding amount for a delegation.
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, PartialOrd, Ord)]
pub struct DelegationAutoCompoundConfig<AccountId> {
	pub delegator: AccountId,
	pub value: Percent,
}

/// Sets the auto-compounding value for a delegation.
pub fn set_delegation_config<AccountId: Eq>(
	delegations_config: &mut Vec<DelegationAutoCompoundConfig<AccountId>>,
	delegator: AccountId,
	value: Percent,
) {
	let maybe_delegation = delegations_config
		.iter_mut()
		.find(|entry| entry.delegator == delegator);

	let mut delegation = if let Some(delegation) = maybe_delegation {
		delegation
	} else {
		delegations_config.push(DelegationAutoCompoundConfig {
			delegator,
			value: Percent::zero(),
		});
		delegations_config.last_mut().expect("cannot fail; qed")
	};

	delegation.value = value;
}

/// Removes the auto-compounding value for a delegation.
pub fn remove_delegation_config<AccountId: Eq>(
	delegations_config: &mut Vec<DelegationAutoCompoundConfig<AccountId>>,
	delegator: &AccountId,
) -> bool {
	if let Some(index) = delegations_config
		.iter()
		.position(|entry| &entry.delegator == delegator)
	{
		delegations_config.remove(index);
		true
	} else {
		false
	}
}

impl<T: Config> Pallet<T> {
	/// Delegates and sets the auto-compounding config
	pub(crate) fn delegate_with_auto_compound_config(
		candidate: T::AccountId,
		delegator: T::AccountId,
		amount: BalanceOf<T>,
		auto_compound: Percent,
		candidate_delegation_count_hint: u32,
		candidate_auto_compounding_delegation_count_hint: u32,
		delegation_count_hint: u32,
	) -> DispatchResultWithPostInfo {
		// check that caller can lock the amount before any changes to storage
		ensure!(
			Self::get_delegator_stakable_free_balance(&delegator) >= amount,
			Error::<T>::InsufficientBalance
		);

		let mut delegator_state = if let Some(mut state) = <DelegatorState<T>>::get(&delegator) {
			// delegation after first
			ensure!(
				amount >= T::MinDelegation::get(),
				Error::<T>::DelegationBelowMin
			);
			ensure!(
				delegation_count_hint >= state.delegations.0.len() as u32,
				Error::<T>::TooLowDelegationCountToDelegate
			);
			ensure!(
				(state.delegations.0.len() as u32) < T::MaxDelegationsPerDelegator::get(),
				Error::<T>::ExceedMaxDelegationsPerDelegator
			);
			ensure!(
				state.add_delegation(Bond {
					owner: candidate.clone(),
					amount
				}),
				Error::<T>::AlreadyDelegatedCandidate
			);
			state
		} else {
			// first delegation
			ensure!(
				amount >= T::MinDelegatorStk::get(),
				Error::<T>::DelegatorBondBelowMin
			);
			ensure!(!Self::is_candidate(&delegator), Error::<T>::CandidateExists);
			Delegator::new(delegator.clone(), candidate.clone(), amount)
		};
		let mut candidate_state =
			<CandidateInfo<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
		ensure!(
			candidate_delegation_count_hint >= candidate_state.delegation_count,
			Error::<T>::TooLowCandidateDelegationCountToDelegate
		);
		let mut auto_compounding_state = <AutoCompoundingDelegations<T>>::get(&candidate);
		ensure!(
			auto_compounding_state.len() as u32 <= candidate_auto_compounding_delegation_count_hint,
			<Error<T>>::TooLowCandidateAutoCompoundingDelegationCountToDelegate,
		);

		// add delegation to candidate
		let (delegator_position, less_total_staked) = candidate_state.add_delegation::<T>(
			&candidate,
			Bond {
				owner: delegator.clone(),
				amount,
			},
		)?;

		// lock delegator amount
		delegator_state.adjust_bond_lock::<T>(BondAdjust::Increase(amount))?;

		// adjust total locked,
		// only is_some if kicked the lowest bottom as a consequence of this new delegation
		let net_total_increase = if let Some(less) = less_total_staked {
			amount.saturating_sub(less)
		} else {
			amount
		};
		let new_total_locked = <Total<T>>::get().saturating_add(net_total_increase);

		// set auto-compound config
		if !auto_compound.is_zero() {
			set_delegation_config(
				&mut auto_compounding_state,
				delegator.clone(),
				auto_compound.clone(),
			);
			<AutoCompoundingDelegations<T>>::insert(candidate.clone(), auto_compounding_state);
		}

		<Total<T>>::put(new_total_locked);
		<CandidateInfo<T>>::insert(&candidate, candidate_state);
		<DelegatorState<T>>::insert(&delegator, delegator_state);
		Self::deposit_event(Event::Delegation {
			delegator: delegator,
			locked_amount: amount,
			candidate: candidate,
			delegator_position: delegator_position,
			auto_compound,
		});

		Ok(().into())
	}

	/// Sets the auto-compounding value for a delegation. The config is removed if value is zero.
	pub(crate) fn delegation_set_auto_compounding_config(
		candidate: T::AccountId,
		delegator: T::AccountId,
		value: Percent,
		candidate_auto_compounding_delegation_count_hint: u32,
		delegation_count_hint: u32,
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

		let mut auto_compounding_state = <AutoCompoundingDelegations<T>>::get(&candidate);
		ensure!(
			auto_compounding_state.len()
				<= candidate_auto_compounding_delegation_count_hint as usize,
			<Error<T>>::TooLowCandidateAutoCompoundingDelegationCountToAutoCompound,
		);
		if value.is_zero() {
			remove_delegation_config(&mut auto_compounding_state, &delegator);
		} else {
			set_delegation_config(&mut auto_compounding_state, delegator.clone(), value);
		}
		<AutoCompoundingDelegations<T>>::insert(candidate.clone(), auto_compounding_state);
		Self::deposit_event(Event::DelegationAutoCompoundingSet {
			candidate,
			delegator,
			value,
		});

		Ok(().into())
	}

	/// Removes the auto-compounding value for a delegation. This should be called when the
	/// delegation is revoked to cleanup storage. Storage is only written iff the entry existed.
	pub(crate) fn delegation_remove_auto_compounding_config(
		candidate: &T::AccountId,
		delegator: &T::AccountId,
	) {
		let mut auto_compounding_state = <AutoCompoundingDelegations<T>>::get(candidate);
		if remove_delegation_config(&mut auto_compounding_state, delegator) {
			<AutoCompoundingDelegations<T>>::insert(candidate, auto_compounding_state);
		}
	}

	/// Returns the value of auto-compound, if it exists for a given delegation, zero otherwise.
	pub fn delegation_auto_compound(collator: &T::AccountId, delegator: &T::AccountId) -> Percent {
		<AutoCompoundingDelegations<T>>::get(collator)
			.iter()
			.find(|d| &d.delegator == delegator)
			.map_or_else(|| Percent::zero(), |d| d.value.clone())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_set_delegation_config_inserts_config_if_entry_missing() {
		let mut delegations_config = vec![];
		set_delegation_config(&mut delegations_config, 1, Percent::from_percent(50));
		assert_eq!(
			vec![DelegationAutoCompoundConfig {
				delegator: 1,
				value: Percent::from_percent(50),
			}],
			delegations_config,
		);
	}

	#[test]
	fn test_set_delegation_config_updates_config_if_entry_exists() {
		let mut delegations_config = vec![DelegationAutoCompoundConfig {
			delegator: 1,
			value: Percent::from_percent(10),
		}];
		set_delegation_config(&mut delegations_config, 1, Percent::from_percent(50));
		assert_eq!(
			vec![DelegationAutoCompoundConfig {
				delegator: 1,
				value: Percent::from_percent(50),
			}],
			delegations_config,
		);
	}

	#[test]
	fn test_remove_delegation_config_returns_false_if_entry_was_missing() {
		let mut delegations_config = vec![];
		assert_eq!(false, remove_delegation_config(&mut delegations_config, &1),);
	}

	#[test]
	fn test_remove_delegation_config_returns_true_if_entry_existed() {
		let mut delegations_config = vec![DelegationAutoCompoundConfig {
			delegator: 1,
			value: Percent::from_percent(10),
		}];
		assert_eq!(true, remove_delegation_config(&mut delegations_config, &1));
	}
}
