use frame_support::ensure;
use frame_support::traits::{Get, ReservableCurrency};
use frame_support::{dispatch::DispatchResultWithPostInfo, RuntimeDebug};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::traits::Saturating;

use crate::pallet::{
	BalanceOf, CandidateInfo, CollatorId, Config, DelegatorId,
	DelegatorScheduledRequestDecreaseAmount, DelegatorScheduledRequests,
	DelegatorScheduledRevokeRequestCount, DelegatorState, Error, Event, Pallet, Round, RoundIndex,
	Total,
};

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum DelegationAction<Balance> {
	Revoke(Balance),
	Decrease(Balance),
}

impl<Balance> DelegationAction<Balance>
where
	Balance: Copy,
{
	pub fn amount(&self) -> Balance {
		match self {
			DelegationAction::Revoke(amount) => *amount,
			DelegationAction::Decrease(amount) => *amount,
		}
	}
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ScheduledRequest<Balance> {
	pub when_executable: RoundIndex,
	pub action: DelegationAction<Balance>,
}

impl<T> Pallet<T>
where
	T: Config,
{
	pub(crate) fn delegator_schedule_revoke(
		delegator: DelegatorId<T>,
		collator: CollatorId<T>,
	) -> DispatchResultWithPostInfo {
		let state = <DelegatorState<T>>::get(&delegator).ok_or(<Error<T>>::DelegatorDNE)?;
		ensure!(
			<DelegatorScheduledRequests<T>>::get(&delegator, &collator).is_none(),
			<Error<T>>::PendingDelegationRequestAlreadyExists,
		);

		let bonded_amount = state
			.get_bond_amount(&collator)
			.ok_or(<Error<T>>::DelegationDNE)?;
		let now = <Round<T>>::get().current;
		let when = now + T::RevokeDelegationDelay::get();

		Self::delegator_scheduled_requests_state_add(
			delegator.clone(),
			collator.clone(),
			ScheduledRequest {
				action: DelegationAction::Revoke(bonded_amount),
				when_executable: when,
			},
		);

		Self::deposit_event(Event::DelegationRevocationScheduled {
			round: now,
			delegator,
			candidate: collator,
			scheduled_exit: when,
		});
		Ok(().into())
	}

	pub(crate) fn delegator_schedule_bond_decrease(
		delegator: DelegatorId<T>,
		collator: CollatorId<T>,
		decrease_amount: BalanceOf<T>,
	) -> DispatchResultWithPostInfo {
		let state = <DelegatorState<T>>::get(&delegator).ok_or(<Error<T>>::DelegatorDNE)?;
		ensure!(
			<DelegatorScheduledRequests<T>>::get(&delegator, &collator).is_none(),
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
		let total_scheduled_decrease_amount =
			<DelegatorScheduledRequestDecreaseAmount<T>>::get(&delegator);
		let net_total = state.total.saturating_sub(total_scheduled_decrease_amount);

		// Net Total is always >= MinDelegatorStk
		let max_subtracted_amount = net_total.saturating_sub(T::MinDelegatorStk::get().into());
		ensure!(
			decrease_amount <= max_subtracted_amount,
			<Error<T>>::DelegatorBondBelowMin
		);

		let now = <Round<T>>::get().current;
		let when = now + T::RevokeDelegationDelay::get();

		Self::delegator_scheduled_requests_state_add(
			delegator.clone(),
			collator.clone(),
			ScheduledRequest {
				action: DelegationAction::Decrease(decrease_amount),
				when_executable: when,
			},
		);

		Self::deposit_event(Event::DelegationDecreaseScheduled {
			delegator,
			candidate: collator,
			amount_to_decrease: decrease_amount,
			execute_round: when,
		});
		Ok(().into())
	}

	pub(crate) fn delegator_cancel_request(
		delegator: DelegatorId<T>,
		collator: CollatorId<T>,
	) -> DispatchResultWithPostInfo {
		let request = <DelegatorScheduledRequests<T>>::get(&delegator, &collator)
			.ok_or(<Error<T>>::PendingDelegationRequestDNE)?;

		Self::delegator_scheduled_requests_state_remove(&delegator, &collator);
		Self::deposit_event(Event::CancelledDelegationRequest {
			delegator,
			collator,
			cancelled_request: request,
		});
		Ok(().into())
	}

	pub(crate) fn delegator_execute_scheduled_request(
		delegator: DelegatorId<T>,
		collator: CollatorId<T>,
	) -> DispatchResultWithPostInfo {
		let mut state = <DelegatorState<T>>::get(&delegator).ok_or(<Error<T>>::DelegatorDNE)?;
		let request = <DelegatorScheduledRequests<T>>::get(&delegator, &collator)
			.ok_or(<Error<T>>::PendingDelegationRequestDNE)?;

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
				Self::delegator_scheduled_requests_state_remove(&delegator, &collator);
				// remove delegation from delegator state
				state.rm_delegation(&collator);

				// remove delegation from collator state delegations
				Self::delegator_leaves_candidate(collator.clone(), delegator.clone(), amount)?;
				Self::deposit_event(Event::DelegationRevoked {
					delegator: delegator.clone(),
					candidate: collator,
					unstaked_amount: amount,
				});
				if leaving {
					<DelegatorState<T>>::remove(&delegator);
					Self::deposit_event(Event::DelegatorLeft {
						delegator: delegator,
						unstaked_amount: amount,
					});
				} else {
					<DelegatorState<T>>::insert(&delegator, state);
				}
				Ok(().into())
			}
			DelegationAction::Decrease(amount) => {
				// remove from pending requests
				Self::delegator_scheduled_requests_state_remove(&delegator, &collator);

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

							<DelegatorState<T>>::insert(&delegator, state);
							Self::deposit_event(Event::DelegationDecreased {
								delegator,
								candidate: collator,
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

	pub(crate) fn delegator_scheduled_requests_state_add(
		delegator: DelegatorId<T>,
		collator: CollatorId<T>,
		request: ScheduledRequest<BalanceOf<T>>,
	) {
		let amount = request.action.amount();
		let is_revoke = matches!(&request.action, &DelegationAction::Revoke(_));

		<DelegatorScheduledRequests<T>>::insert(delegator.clone(), collator, request);
		if is_revoke {
			<DelegatorScheduledRevokeRequestCount<T>>::mutate(&delegator, |x| {
				*x = x.saturating_add(1)
			});
		}
		<DelegatorScheduledRequestDecreaseAmount<T>>::mutate(&delegator, |x| {
			*x = x.saturating_add(amount)
		});
	}

	pub(crate) fn delegator_scheduled_requests_state_remove(
		delegator: &DelegatorId<T>,
		collator: &CollatorId<T>,
	) {
		if let Some(request) = <DelegatorScheduledRequests<T>>::take(delegator, collator) {
			if matches!(request.action, DelegationAction::Revoke(_)) {
				<DelegatorScheduledRevokeRequestCount<T>>::mutate(&delegator, |x| {
					*x = x.saturating_sub(1)
				});
			}
			<DelegatorScheduledRequestDecreaseAmount<T>>::mutate(&delegator, |x| {
				*x = x.saturating_sub(request.action.amount())
			});
		}
	}
}
