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

use crate::pallet::{
	BalanceOf, CandidateInfo, Config, DelegationScheduledRequests, DelegatorState, Error, Event,
	Pallet, Round, RoundIndex, Total,
};
use crate::weights::WeightInfo;
use crate::{auto_compound::AutoCompoundDelegations, AddGet, Delegator};
use frame_support::dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo};
use frame_support::ensure;
use frame_support::traits::Get;
use frame_support::BoundedVec;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{traits::Saturating, RuntimeDebug};

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

		let actual_weight =
			<T as Config>::WeightInfo::schedule_revoke_delegation(scheduled_requests.len() as u32);

		ensure!(
			!scheduled_requests
				.iter()
				.any(|req| req.delegator == delegator),
			DispatchErrorWithPostInfo {
				post_info: Some(actual_weight).into(),
				error: <Error<T>>::PendingDelegationRequestAlreadyExists.into(),
			},
		);

		let bonded_amount = state
			.get_bond_amount(&collator)
			.ok_or(<Error<T>>::DelegationDNE)?;
		let now = <Round<T>>::get().current;
		let when = now.saturating_add(T::RevokeDelegationDelay::get());
		scheduled_requests
			.try_push(ScheduledRequest {
				delegator: delegator.clone(),
				action: DelegationAction::Revoke(bonded_amount),
				when_executable: when,
			})
			.map_err(|_| DispatchErrorWithPostInfo {
				post_info: Some(actual_weight).into(),
				error: Error::<T>::ExceedMaxDelegationsPerDelegator.into(),
			})?;
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

		let actual_weight = <T as Config>::WeightInfo::schedule_delegator_bond_less(
			scheduled_requests.len() as u32,
		);

		ensure!(
			!scheduled_requests
				.iter()
				.any(|req| req.delegator == delegator),
			DispatchErrorWithPostInfo {
				post_info: Some(actual_weight).into(),
				error: <Error<T>>::PendingDelegationRequestAlreadyExists.into(),
			},
		);

		let bonded_amount = state
			.get_bond_amount(&collator)
			.ok_or(DispatchErrorWithPostInfo {
				post_info: Some(actual_weight).into(),
				error: <Error<T>>::DelegationDNE.into(),
			})?;
		ensure!(
			bonded_amount > decrease_amount,
			DispatchErrorWithPostInfo {
				post_info: Some(actual_weight).into(),
				error: <Error<T>>::DelegatorBondBelowMin.into(),
			},
		);
		let new_amount: BalanceOf<T> = (bonded_amount.saturating_sub(decrease_amount)).into();
		ensure!(
			new_amount >= T::MinDelegation::get(),
			DispatchErrorWithPostInfo {
				post_info: Some(actual_weight).into(),
				error: <Error<T>>::DelegationBelowMin.into(),
			},
		);

		// Net Total is total after pending orders are executed
		let net_total = state.total().saturating_sub(state.less_total);
		// Net Total is always >= MinDelegation
		let max_subtracted_amount = net_total.saturating_sub(T::MinDelegation::get().into());
		ensure!(
			decrease_amount <= max_subtracted_amount,
			DispatchErrorWithPostInfo {
				post_info: Some(actual_weight).into(),
				error: <Error<T>>::DelegatorBondBelowMin.into(),
			},
		);

		let now = <Round<T>>::get().current;
		let when = now.saturating_add(T::DelegationBondLessDelay::get());
		scheduled_requests
			.try_push(ScheduledRequest {
				delegator: delegator.clone(),
				action: DelegationAction::Decrease(decrease_amount),
				when_executable: when,
			})
			.map_err(|_| DispatchErrorWithPostInfo {
				post_info: Some(actual_weight).into(),
				error: Error::<T>::ExceedMaxDelegationsPerDelegator.into(),
			})?;
		state.less_total = state.less_total.saturating_add(decrease_amount);
		<DelegationScheduledRequests<T>>::insert(collator.clone(), scheduled_requests);
		<DelegatorState<T>>::insert(delegator.clone(), state);

		Self::deposit_event(Event::DelegationDecreaseScheduled {
			delegator,
			candidate: collator,
			amount_to_decrease: decrease_amount,
			execute_round: when,
		});
		Ok(Some(actual_weight).into())
	}

	/// Cancels the delegator's existing [ScheduledRequest] towards a given collator.
	pub(crate) fn delegation_cancel_request(
		collator: T::AccountId,
		delegator: T::AccountId,
	) -> DispatchResultWithPostInfo {
		let mut state = <DelegatorState<T>>::get(&delegator).ok_or(<Error<T>>::DelegatorDNE)?;
		let mut scheduled_requests = <DelegationScheduledRequests<T>>::get(&collator);
		let actual_weight =
			<T as Config>::WeightInfo::cancel_delegation_request(scheduled_requests.len() as u32);

		let request =
			Self::cancel_request_with_state(&delegator, &mut state, &mut scheduled_requests)
				.ok_or(DispatchErrorWithPostInfo {
					post_info: Some(actual_weight).into(),
					error: <Error<T>>::PendingDelegationRequestDNE.into(),
				})?;

		<DelegationScheduledRequests<T>>::insert(collator.clone(), scheduled_requests);
		<DelegatorState<T>>::insert(delegator.clone(), state);

		Self::deposit_event(Event::CancelledDelegationRequest {
			delegator,
			collator,
			cancelled_request: request.into(),
		});
		Ok(Some(actual_weight).into())
	}

	fn cancel_request_with_state(
		delegator: &T::AccountId,
		state: &mut Delegator<T::AccountId, BalanceOf<T>>,
		scheduled_requests: &mut BoundedVec<
			ScheduledRequest<T::AccountId, BalanceOf<T>>,
			AddGet<T::MaxTopDelegationsPerCandidate, T::MaxBottomDelegationsPerCandidate>,
		>,
	) -> Option<ScheduledRequest<T::AccountId, BalanceOf<T>>> {
		let request_idx = scheduled_requests
			.iter()
			.position(|req| &req.delegator == delegator)?;

		let request = scheduled_requests.remove(request_idx);
		let amount = request.action.amount();
		state.less_total = state.less_total.saturating_sub(amount);
		Some(request)
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
			.position(|req| req.delegator == delegator)
			.ok_or(<Error<T>>::PendingDelegationRequestDNE)?;
		let request = &scheduled_requests[request_idx];

		let now = <Round<T>>::get().current;
		ensure!(
			request.when_executable <= now,
			<Error<T>>::PendingDelegationRequestNotDueYet
		);

		match request.action {
			DelegationAction::Revoke(amount) => {
				let actual_weight =
					<T as Config>::WeightInfo::execute_delegator_revoke_delegation_worst();

				// revoking last delegation => leaving set of delegators
				let leaving = if state.delegations.0.len() == 1usize {
					true
				} else {
					ensure!(
						state.total().saturating_sub(T::MinDelegation::get().into()) >= amount,
						DispatchErrorWithPostInfo {
							post_info: Some(actual_weight).into(),
							error: <Error<T>>::DelegatorBondBelowMin.into(),
						}
					);
					false
				};

				// remove from pending requests
				let amount = scheduled_requests.remove(request_idx).action.amount();
				state.less_total = state.less_total.saturating_sub(amount);

				// remove delegation from delegator state
				state.rm_delegation::<T>(&collator);

				// remove delegation from auto-compounding info
				<AutoCompoundDelegations<T>>::remove_auto_compound(&collator, &delegator);

				// remove delegation from collator state delegations
				Self::delegator_leaves_candidate(collator.clone(), delegator.clone(), amount)
					.map_err(|err| DispatchErrorWithPostInfo {
						post_info: Some(actual_weight).into(),
						error: err,
					})?;
				Self::deposit_event(Event::DelegationRevoked {
					delegator: delegator.clone(),
					candidate: collator.clone(),
					unstaked_amount: amount,
				});

				<DelegationScheduledRequests<T>>::insert(collator, scheduled_requests);
				if leaving {
					<DelegatorState<T>>::remove(&delegator);
					Self::deposit_event(Event::DelegatorLeft {
						delegator,
						unstaked_amount: amount,
					});
				} else {
					<DelegatorState<T>>::insert(&delegator, state);
				}
				Ok(Some(actual_weight).into())
			}
			DelegationAction::Decrease(_) => {
				let actual_weight =
					<T as Config>::WeightInfo::execute_delegator_revoke_delegation_worst();

				// remove from pending requests
				let amount = scheduled_requests.remove(request_idx).action.amount();
				state.less_total = state.less_total.saturating_sub(amount);

				// decrease delegation
				for bond in &mut state.delegations.0 {
					if bond.owner == collator {
						return if bond.amount > amount {
							let amount_before: BalanceOf<T> = bond.amount.into();
							bond.amount = bond.amount.saturating_sub(amount);
							let mut collator_info = <CandidateInfo<T>>::get(&collator)
								.ok_or(<Error<T>>::CandidateDNE)
								.map_err(|err| DispatchErrorWithPostInfo {
									post_info: Some(actual_weight).into(),
									error: err.into(),
								})?;

							state
								.total_sub_if::<T, _>(amount, |total| {
									let new_total: BalanceOf<T> = total.into();
									ensure!(
										new_total >= T::MinDelegation::get(),
										<Error<T>>::DelegationBelowMin
									);

									Ok(())
								})
								.map_err(|err| DispatchErrorWithPostInfo {
									post_info: Some(actual_weight).into(),
									error: err,
								})?;

							// need to go into decrease_delegation
							let in_top = collator_info
								.decrease_delegation::<T>(
									&collator,
									delegator.clone(),
									amount_before,
									amount,
								)
								.map_err(|err| DispatchErrorWithPostInfo {
									post_info: Some(actual_weight).into(),
									error: err,
								})?;
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
							Ok(Some(actual_weight).into())
						} else {
							// must rm entire delegation if bond.amount <= less or cancel request
							Err(DispatchErrorWithPostInfo {
								post_info: Some(actual_weight).into(),
								error: <Error<T>>::DelegationBelowMin.into(),
							})
						};
					}
				}
				Err(DispatchErrorWithPostInfo {
					post_info: Some(actual_weight).into(),
					error: <Error<T>>::DelegationDNE.into(),
				})
			}
		}
	}

	/// Removes the delegator's existing [ScheduledRequest] towards a given collator, if exists.
	/// The state needs to be persisted by the caller of this function.
	pub(crate) fn delegation_remove_request_with_state(
		collator: &T::AccountId,
		delegator: &T::AccountId,
		state: &mut Delegator<T::AccountId, BalanceOf<T>>,
	) {
		let mut scheduled_requests = <DelegationScheduledRequests<T>>::get(collator);

		let maybe_request_idx = scheduled_requests
			.iter()
			.position(|req| &req.delegator == delegator);

		if let Some(request_idx) = maybe_request_idx {
			let request = scheduled_requests.remove(request_idx);
			let amount = request.action.amount();
			state.less_total = state.less_total.saturating_sub(amount);
			<DelegationScheduledRequests<T>>::insert(collator, scheduled_requests);
		}
	}

	/// Returns true if a [ScheduledRequest] exists for a given delegation
	pub fn delegation_request_exists(collator: &T::AccountId, delegator: &T::AccountId) -> bool {
		<DelegationScheduledRequests<T>>::get(collator)
			.iter()
			.any(|req| &req.delegator == delegator)
	}

	/// Returns true if a [DelegationAction::Revoke] [ScheduledRequest] exists for a given delegation
	pub fn delegation_request_revoke_exists(
		collator: &T::AccountId,
		delegator: &T::AccountId,
	) -> bool {
		<DelegationScheduledRequests<T>>::get(collator)
			.iter()
			.any(|req| {
				&req.delegator == delegator && matches!(req.action, DelegationAction::Revoke(_))
			})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{mock::Test, set::OrderedSet, Bond};

	#[test]
	fn test_cancel_request_with_state_removes_request_for_correct_delegator_and_updates_state() {
		let mut state = Delegator {
			id: 1,
			delegations: OrderedSet::from(vec![Bond {
				amount: 100,
				owner: 2,
			}]),
			total: 100,
			less_total: 100,
			status: crate::DelegatorStatus::Active,
		};
		let mut scheduled_requests = vec![
			ScheduledRequest {
				delegator: 1,
				when_executable: 1,
				action: DelegationAction::Revoke(100),
			},
			ScheduledRequest {
				delegator: 2,
				when_executable: 1,
				action: DelegationAction::Decrease(50),
			},
		]
		.try_into()
		.expect("must succeed");
		let removed_request =
			<Pallet<Test>>::cancel_request_with_state(&1, &mut state, &mut scheduled_requests);

		assert_eq!(
			removed_request,
			Some(ScheduledRequest {
				delegator: 1,
				when_executable: 1,
				action: DelegationAction::Revoke(100),
			})
		);
		assert_eq!(
			scheduled_requests,
			vec![ScheduledRequest {
				delegator: 2,
				when_executable: 1,
				action: DelegationAction::Decrease(50),
			},]
		);
		assert_eq!(
			state,
			Delegator {
				id: 1,
				delegations: OrderedSet::from(vec![Bond {
					amount: 100,
					owner: 2,
				}]),
				total: 100,
				less_total: 0,
				status: crate::DelegatorStatus::Active,
			}
		);
	}

	#[test]
	fn test_cancel_request_with_state_does_nothing_when_request_does_not_exist() {
		let mut state = Delegator {
			id: 1,
			delegations: OrderedSet::from(vec![Bond {
				amount: 100,
				owner: 2,
			}]),
			total: 100,
			less_total: 100,
			status: crate::DelegatorStatus::Active,
		};
		let mut scheduled_requests = vec![ScheduledRequest {
			delegator: 2,
			when_executable: 1,
			action: DelegationAction::Decrease(50),
		}]
		.try_into()
		.expect("must succeed");
		let removed_request =
			<Pallet<Test>>::cancel_request_with_state(&1, &mut state, &mut scheduled_requests);

		assert_eq!(removed_request, None,);
		assert_eq!(
			scheduled_requests,
			vec![ScheduledRequest {
				delegator: 2,
				when_executable: 1,
				action: DelegationAction::Decrease(50),
			},]
		);
		assert_eq!(
			state,
			Delegator {
				id: 1,
				delegations: OrderedSet::from(vec![Bond {
					amount: 100,
					owner: 2,
				}]),
				total: 100,
				less_total: 100,
				status: crate::DelegatorStatus::Active,
			}
		);
	}
}
