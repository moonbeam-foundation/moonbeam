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

//! Scheduled requests functionality for delegators

use frame_support::ensure;
use frame_support::traits::{Get, ReservableCurrency};
use frame_support::{dispatch::DispatchResultWithPostInfo, RuntimeDebug};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::traits::Saturating;

use crate::pallet::{
	BalanceOf, CandidateInfo, Config, DelegationScheduledRequests, DelegatorState, Error, Event,
	Pallet, Round, RoundIndex, Total,
};
use crate::Delegator;

/// An action that can be performed upon a delegation
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, PartialOrd, Ord)]
pub enum DelegationAction<Balance> {
	Revoke(Balance),
	Decrease(Balance),
}

impl<Balance: Copy> DelegationAction<Balance> {
	/// Returns the wrapped amount value.
	pub fn amount(&self) -> Balance {
		match self {
			DelegationAction::Revoke(amount) => *amount,
			DelegationAction::Decrease(amount) => *amount,
		}
	}
}

/// Represents a scheduled request that define a [DelegationAction]. The request is executable
/// iff the provided [RoundIndex] is achieved.
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, PartialOrd, Ord)]
pub struct ScheduledRequest<AccountId, Balance> {
	pub delegator: AccountId,
	pub when_executable: RoundIndex,
	pub action: DelegationAction<Balance>,
}

/// Represents a cancelled scheduled request for emitting an event.
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct CancelledScheduledRequest<Balance> {
	pub when_executable: RoundIndex,
	pub action: DelegationAction<Balance>,
}

impl<A, B> From<ScheduledRequest<A, B>> for CancelledScheduledRequest<B> {
	fn from(request: ScheduledRequest<A, B>) -> Self {
		CancelledScheduledRequest {
			when_executable: request.when_executable,
			action: request.action,
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Schedules a [DelegationAction::Revoke] for the delegator, towards a given collator.
	pub(crate) fn delegation_schedule_revoke(
		collator: T::AccountId,
		delegator: T::AccountId,
	) -> DispatchResultWithPostInfo {
		let mut state = <DelegatorState<T>>::get(&delegator).ok_or(<Error<T>>::DelegatorDNE)?;
		let mut scheduled_requests = <DelegationScheduledRequests<T>>::get(&collator);

		ensure!(
			!scheduled_requests.iter().any(|x| x.delegator == delegator),
			<Error<T>>::PendingDelegationRequestAlreadyExists,
		);

		let bonded_amount = state
			.get_bond_amount(&collator)
			.ok_or(<Error<T>>::DelegationDNE)?;
		let now = <Round<T>>::get().current;
		let when = now.saturating_add(T::RevokeDelegationDelay::get());
		scheduled_requests.push(ScheduledRequest {
			delegator: delegator.clone(),
			action: DelegationAction::Revoke(bonded_amount),
			when_executable: when,
		});
		state.less_total = state.less_total.saturating_add(bonded_amount);
		<DelegationScheduledRequests<T>>::insert(collator.clone(), scheduled_requests);
		<DelegatorState<T>>::insert(delegator.clone(), state);

		Self::deposit_event(Event::DelegationRevocationScheduled {
			round: now,
			delegator,
			candidate: collator,
			scheduled_exit: when,
		});
		Ok(().into())
	}

	/// Schedules a [DelegationAction::Decrease] for the delegator, towards a given collator.
	pub(crate) fn delegation_schedule_bond_decrease(
		collator: T::AccountId,
		delegator: T::AccountId,
		decrease_amount: BalanceOf<T>,
	) -> DispatchResultWithPostInfo {
		let mut state = <DelegatorState<T>>::get(&delegator).ok_or(<Error<T>>::DelegatorDNE)?;
		let mut scheduled_requests = <DelegationScheduledRequests<T>>::get(&collator);

		ensure!(
			!scheduled_requests.iter().any(|x| x.delegator == delegator),
			<Error<T>>::PendingDelegationRequestAlreadyExists,
		);

		let bonded_amount = state
			.get_bond_amount(&collator)
			.ok_or(<Error<T>>::DelegationDNE)?;
		ensure!(
			bonded_amount > decrease_amount,
			<Error<T>>::DelegatorBondBelowMin
		);
		let new_amount: BalanceOf<T> = (bonded_amount - decrease_amount).into();
		ensure!(
			new_amount >= T::MinDelegation::get(),
			<Error<T>>::DelegationBelowMin
		);

		// Net Total is total after pending orders are executed
		let net_total = state.total.saturating_sub(state.less_total);
		// Net Total is always >= MinDelegatorStk
		let max_subtracted_amount = net_total.saturating_sub(T::MinDelegatorStk::get().into());
		ensure!(
			decrease_amount <= max_subtracted_amount,
			<Error<T>>::DelegatorBondBelowMin
		);

		let now = <Round<T>>::get().current;
		let when = now.saturating_add(T::RevokeDelegationDelay::get());
		scheduled_requests.push(ScheduledRequest {
			delegator: delegator.clone(),
			action: DelegationAction::Decrease(decrease_amount),
			when_executable: when,
		});
		state.less_total = state.less_total.saturating_add(decrease_amount);
		<DelegationScheduledRequests<T>>::insert(collator.clone(), scheduled_requests);
		<DelegatorState<T>>::insert(delegator.clone(), state);

		Self::deposit_event(Event::DelegationDecreaseScheduled {
			delegator,
			candidate: collator,
			amount_to_decrease: decrease_amount,
			execute_round: when,
		});
		Ok(().into())
	}

	/// Cancels the delegator's existing [ScheduledRequest] towards a given collator.
	pub(crate) fn delegation_cancel_request(
		collator: T::AccountId,
		delegator: T::AccountId,
	) -> DispatchResultWithPostInfo {
		let mut state = <DelegatorState<T>>::get(&delegator).ok_or(<Error<T>>::DelegatorDNE)?;
		let mut scheduled_requests = <DelegationScheduledRequests<T>>::get(&collator);
		let request_idx = scheduled_requests
			.iter()
			.position(|x| x.delegator == delegator)
			.ok_or(<Error<T>>::PendingDelegationRequestDNE)?;

		let request = scheduled_requests.remove(request_idx);
		let amount = request.action.amount();
		state.less_total = state.less_total.saturating_sub(amount);
		<DelegationScheduledRequests<T>>::insert(collator.clone(), scheduled_requests);
		<DelegatorState<T>>::insert(delegator.clone(), state);

		Self::deposit_event(Event::CancelledDelegationRequest {
			delegator,
			collator,
			cancelled_request: request.into(),
		});
		Ok(().into())
	}

	/// Executes the delegator's existing [ScheduledRequest] towards a given collator.
	pub(crate) fn delegation_execute_scheduled_request(
		collator: T::AccountId,
		delegator: T::AccountId,
	) -> DispatchResultWithPostInfo {
		let mut state = <DelegatorState<T>>::get(&delegator).ok_or(<Error<T>>::DelegatorDNE)?;
		let mut scheduled_requests = <DelegationScheduledRequests<T>>::get(&collator);
		let request_idx = scheduled_requests
			.iter()
			.position(|x| x.delegator == delegator)
			.ok_or(<Error<T>>::PendingDelegationRequestDNE)?;
		let request = &scheduled_requests[request_idx];

		let now = <Round<T>>::get().current;
		ensure!(
			request.when_executable <= now,
			<Error<T>>::PendingDelegationRequestNotDueYet
		);

		match request.action {
			DelegationAction::Revoke(amount) => {
				// revoking last delegation => leaving set of delegators
				let leaving = if state.delegations.0.len() == 1usize {
					true
				} else {
					ensure!(
						state.total.saturating_sub(T::MinDelegatorStk::get().into()) >= amount,
						<Error<T>>::DelegatorBondBelowMin
					);
					false
				};

				// remove from pending requests
				let amount = scheduled_requests.remove(request_idx).action.amount();
				state.less_total = state.less_total.saturating_sub(amount);

				// remove delegation from delegator state
				state.rm_delegation(&collator);

				// remove delegation from collator state delegations
				Self::delegator_leaves_candidate(collator.clone(), delegator.clone(), amount)?;
				Self::deposit_event(Event::DelegationRevoked {
					delegator: delegator.clone(),
					candidate: collator.clone(),
					unstaked_amount: amount,
				});
				if leaving {
					<DelegatorState<T>>::remove(&delegator);
					Self::deposit_event(Event::DelegatorLeft {
						delegator,
						unstaked_amount: amount,
					});
				} else {
					<DelegationScheduledRequests<T>>::insert(collator, scheduled_requests);
					<DelegatorState<T>>::insert(&delegator, state);
				}
				Ok(().into())
			}
			DelegationAction::Decrease(_) => {
				// remove from pending requests
				let amount = scheduled_requests.remove(request_idx).action.amount();
				state.less_total = state.less_total.saturating_sub(amount);

				// decrease delegation
				for x in &mut state.delegations.0 {
					if x.owner == collator {
						return if x.amount > amount {
							let amount_before: BalanceOf<T> = x.amount.into();
							x.amount = x.amount.saturating_sub(amount);
							state.total = state.total.saturating_sub(amount);
							let new_total: BalanceOf<T> = state.total.into();
							ensure!(
								new_total >= T::MinDelegation::get(),
								<Error<T>>::DelegationBelowMin
							);
							ensure!(
								new_total >= T::MinDelegatorStk::get(),
								<Error<T>>::DelegatorBondBelowMin
							);
							let mut collator_info = <CandidateInfo<T>>::get(&collator)
								.ok_or(<Error<T>>::CandidateDNE)?;
							T::Currency::unreserve(&delegator, amount);
							// need to go into decrease_delegation
							let in_top = collator_info.decrease_delegation::<T>(
								&collator,
								delegator.clone(),
								amount_before,
								amount,
							)?;
							<CandidateInfo<T>>::insert(&collator, collator_info);
							let new_total_staked = <Total<T>>::get().saturating_sub(amount);
							<Total<T>>::put(new_total_staked);

							<DelegationScheduledRequests<T>>::insert(
								collator.clone(),
								scheduled_requests,
							);
							<DelegatorState<T>>::insert(delegator.clone(), state);
							Self::deposit_event(Event::DelegationDecreased {
								delegator,
								candidate: collator.clone(),
								amount,
								in_top,
							});
							Ok(().into())
						} else {
							// must rm entire delegation if x.amount <= less or cancel request
							Err(<Error<T>>::DelegationBelowMin.into())
						};
					}
				}
				Err(<Error<T>>::DelegationDNE.into())
			}
		}
	}

	/// Removes the delegator's existing [ScheduledRequest] towards a given collator.
	/// The state needs to be persisted by the caller of this function.
	/// Returns [Error::PendingDelegationRequestDNE] if request does not exist.
	pub(crate) fn delegation_remove_request_with_state(
		collator: &T::AccountId,
		delegator: &T::AccountId,
		state: &mut Delegator<T::AccountId, BalanceOf<T>>,
	) -> DispatchResultWithPostInfo {
		let mut scheduled_requests = <DelegationScheduledRequests<T>>::get(collator);

		let request_idx = scheduled_requests
			.iter()
			.position(|x| &x.delegator == delegator)
			.ok_or(<Error<T>>::PendingDelegationRequestDNE)?;

		let request = scheduled_requests.remove(request_idx);
		let amount = request.action.amount();
		state.less_total = state.less_total.saturating_sub(amount);
		<DelegationScheduledRequests<T>>::insert(collator, scheduled_requests);

		Ok(().into())
	}

	/// Returns true if a [ScheduledRequest] exists for a given delegation
	pub fn delegation_request_exists(collator: &T::AccountId, delegator: &T::AccountId) -> bool {
		<DelegationScheduledRequests<T>>::get(collator)
			.iter()
			.any(|x| &x.delegator == delegator)
	}
}
