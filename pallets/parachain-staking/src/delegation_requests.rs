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

//! Scheduled requests functionality for delegators

use crate::pallet::{
	BalanceOf, CandidateInfo, Config, DelegationScheduledRequests,
	DelegationScheduledRequestsPerCollator, DelegatorState, Error, Event, Pallet, Round,
	RoundIndex, Total,
};
use crate::weights::WeightInfo;
use crate::{auto_compound::AutoCompoundDelegations, Delegator};
use frame_support::dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo};
use frame_support::ensure;
use frame_support::traits::Get;
use frame_support::BoundedVec;
use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{Saturating, Zero},
	RuntimeDebug,
};

/// An action that can be performed upon a delegation
#[derive(
	Clone,
	Eq,
	PartialEq,
	Encode,
	Decode,
	RuntimeDebug,
	TypeInfo,
	PartialOrd,
	Ord,
	DecodeWithMemTracking,
)]
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
#[derive(
	Clone,
	Eq,
	PartialEq,
	Encode,
	Decode,
	RuntimeDebug,
	TypeInfo,
	PartialOrd,
	Ord,
	DecodeWithMemTracking,
)]
pub struct ScheduledRequest<AccountId, Balance> {
	pub delegator: AccountId,
	pub when_executable: RoundIndex,
	pub action: DelegationAction<Balance>,
}

/// Represents a cancelled scheduled request for emitting an event.
#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, DecodeWithMemTracking)]
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
		let mut scheduled_requests = <DelegationScheduledRequests<T>>::get(&collator, &delegator);

		let actual_weight =
			<T as Config>::WeightInfo::schedule_revoke_delegation(scheduled_requests.len() as u32);

		let is_new_delegator = scheduled_requests.is_empty();

		ensure!(
			is_new_delegator,
			DispatchErrorWithPostInfo {
				post_info: Some(actual_weight).into(),
				error: <Error<T>>::PendingDelegationRequestAlreadyExists.into(),
			},
		);

		// This is the first scheduled request for this delegator towards this collator,
		// ensure we do not exceed the maximum number of delegators that can have pending
		// requests for the collator.
		let current = <DelegationScheduledRequestsPerCollator<T>>::get(&collator);
		if current >= Pallet::<T>::max_delegators_per_candidate() {
			return Err(DispatchErrorWithPostInfo {
				post_info: Some(actual_weight).into(),
				error: Error::<T>::ExceedMaxDelegationsPerDelegator.into(),
			});
		}

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
		if is_new_delegator {
			<DelegationScheduledRequestsPerCollator<T>>::mutate(&collator, |c| {
				*c = c.saturating_add(1);
			});
		}
		<DelegationScheduledRequests<T>>::insert(
			collator.clone(),
			delegator.clone(),
			scheduled_requests,
		);
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
		let mut scheduled_requests = <DelegationScheduledRequests<T>>::get(&collator, &delegator);

		let actual_weight = <T as Config>::WeightInfo::schedule_delegator_bond_less(
			scheduled_requests.len() as u32,
		);

		// If this is the first scheduled request for this delegator towards this collator,
		// ensure we do not exceed the maximum number of delegators that can have pending
		// requests for the collator.
		let is_new_delegator =
			!<DelegationScheduledRequests<T>>::contains_key(&collator, &delegator);
		if is_new_delegator {
			let current = <DelegationScheduledRequestsPerCollator<T>>::get(&collator);
			let max_delegators = Pallet::<T>::max_delegators_per_candidate();
			if current >= max_delegators {
				return Err(DispatchErrorWithPostInfo {
					post_info: Some(actual_weight).into(),
					error: Error::<T>::ExceedMaxDelegationsPerDelegator.into(),
				});
			}
		}

		ensure!(
			!scheduled_requests
				.iter()
				.any(|req| matches!(req.action, DelegationAction::Revoke(_))),
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
		let new_amount: BalanceOf<T> = (bonded_amount - decrease_amount).into();
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
		if is_new_delegator {
			<DelegationScheduledRequestsPerCollator<T>>::mutate(&collator, |c| {
				*c = c.saturating_add(1);
			});
		}
		<DelegationScheduledRequests<T>>::insert(
			collator.clone(),
			delegator.clone(),
			scheduled_requests,
		);
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
		let mut scheduled_requests = <DelegationScheduledRequests<T>>::get(&collator, &delegator);
		let actual_weight =
			<T as Config>::WeightInfo::cancel_delegation_request(scheduled_requests.len() as u32);

		let request = Self::cancel_request_with_state(&mut state, &mut scheduled_requests).ok_or(
			DispatchErrorWithPostInfo {
				post_info: Some(actual_weight).into(),
				error: <Error<T>>::PendingDelegationRequestDNE.into(),
			},
		)?;

		if scheduled_requests.is_empty() {
			<DelegationScheduledRequestsPerCollator<T>>::mutate(&collator, |c| {
				*c = c.saturating_sub(1);
			});
			<DelegationScheduledRequests<T>>::remove(&collator, &delegator);
		} else {
			<DelegationScheduledRequests<T>>::insert(
				collator.clone(),
				delegator.clone(),
				scheduled_requests,
			);
		}
		<DelegatorState<T>>::insert(delegator.clone(), state);

		Self::deposit_event(Event::CancelledDelegationRequest {
			delegator,
			collator,
			cancelled_request: request.into(),
		});
		Ok(Some(actual_weight).into())
	}

	fn cancel_request_with_state(
		state: &mut Delegator<T::AccountId, BalanceOf<T>>,
		scheduled_requests: &mut BoundedVec<
			ScheduledRequest<T::AccountId, BalanceOf<T>>,
			T::MaxScheduledRequestsPerDelegator,
		>,
	) -> Option<ScheduledRequest<T::AccountId, BalanceOf<T>>> {
		if scheduled_requests.is_empty() {
			return None;
		}

		// `BoundedVec::remove` can panic, but we make sure it will not happen by
		// checking above that `scheduled_requests` is not empty.
		let request = scheduled_requests.remove(0);
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
		let mut scheduled_requests = <DelegationScheduledRequests<T>>::get(&collator, &delegator);
		let request = scheduled_requests
			.first()
			.ok_or(<Error<T>>::PendingDelegationRequestDNE)?;

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
				// `BoundedVec::remove` can panic, but we make sure it will not happen by checking above that `scheduled_requests` is not empty.
				let amount = scheduled_requests.remove(0).action.amount();
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
				if scheduled_requests.is_empty() {
					<DelegationScheduledRequests<T>>::remove(&collator, &delegator);
					<DelegationScheduledRequestsPerCollator<T>>::mutate(&collator, |c| {
						*c = c.saturating_sub(1);
					});
				} else {
					<DelegationScheduledRequests<T>>::insert(
						collator.clone(),
						delegator.clone(),
						scheduled_requests,
					);
				}
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
				// `BoundedVec::remove` can panic, but we make sure it will not happen by checking above that `scheduled_requests` is not empty.
				let amount = scheduled_requests.remove(0).action.amount();
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

							if scheduled_requests.is_empty() {
								<DelegationScheduledRequests<T>>::remove(&collator, &delegator);
								<DelegationScheduledRequestsPerCollator<T>>::mutate(
									&collator,
									|c| {
										*c = c.saturating_sub(1);
									},
								);
							} else {
								<DelegationScheduledRequests<T>>::insert(
									collator.clone(),
									delegator.clone(),
									scheduled_requests,
								);
							}
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
		let scheduled_requests = <DelegationScheduledRequests<T>>::get(collator, delegator);

		if scheduled_requests.is_empty() {
			return;
		}

		// Calculate total amount across all scheduled requests
		let total_amount: BalanceOf<T> = scheduled_requests
			.iter()
			.map(|request| request.action.amount())
			.fold(BalanceOf::<T>::zero(), |acc, amount| {
				acc.saturating_add(amount)
			});

		state.less_total = state.less_total.saturating_sub(total_amount);
		<DelegationScheduledRequests<T>>::remove(collator, delegator);
		<DelegationScheduledRequestsPerCollator<T>>::mutate(collator, |c| {
			*c = c.saturating_sub(1);
		});
	}

	/// Returns true if a [ScheduledRequest] exists for a given delegation
	pub fn delegation_request_exists(collator: &T::AccountId, delegator: &T::AccountId) -> bool {
		!<DelegationScheduledRequests<T>>::get(collator, delegator).is_empty()
	}

	/// Returns true if a [DelegationAction::Revoke] [ScheduledRequest] exists for a given delegation
	pub fn delegation_request_revoke_exists(
		collator: &T::AccountId,
		delegator: &T::AccountId,
	) -> bool {
		<DelegationScheduledRequests<T>>::get(collator, delegator)
			.iter()
			.any(|req| matches!(req.action, DelegationAction::Revoke(_)))
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
			less_total: 150,
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
			<Pallet<Test>>::cancel_request_with_state(&mut state, &mut scheduled_requests);

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
			state.less_total, 50,
			"less_total should be reduced by the amount of the cancelled request"
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
		let mut scheduled_requests: BoundedVec<
			ScheduledRequest<u64, u128>,
			<Test as crate::pallet::Config>::MaxScheduledRequestsPerDelegator,
		> = BoundedVec::default();
		let removed_request =
			<Pallet<Test>>::cancel_request_with_state(&mut state, &mut scheduled_requests);

		assert_eq!(removed_request, None,);
		assert_eq!(
			scheduled_requests.len(),
			0,
			"scheduled_requests should remain empty"
		);
		assert_eq!(
			state.less_total, 100,
			"less_total should remain unchanged when there is nothing to cancel"
		);
	}
}
