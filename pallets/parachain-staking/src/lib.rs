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

//! # Parachain Staking
//! Minimal staking pallet that implements collator selection by total backed stake.
//! The main difference between this pallet and `frame/pallet-staking` is that this pallet
//! uses direct delegation. Delegators choose exactly who they delegate and with what stake.
//! This is different from `frame/pallet-staking` where delegators approval vote and run Phragmen.
//!
//! ### Rules
//! There is a new round every `<Round<T>>::get().length` blocks.
//!
//! At the start of every round,
//! * issuance is calculated for collators (and their delegators) for block authoring
//! `T::RewardPaymentDelay` rounds ago
//! * queued collator and delegator exits are executed
//! * a new set of collators is chosen from the candidates
//!
//! Immediately following a round change, payments are made once-per-block until all payments have
//! been made. In each such block, one collator is chosen for a rewards payment and is paid along
//! with each of its "top" delegators.
//!
//! To join the set of candidates, call `join_candidates` with `bond >= MinCandidateStk`.
//! To leave the set of candidates, call `schedule_leave_candidates`. If the call succeeds,
//! the collator is removed from the pool of candidates so they cannot be selected for future
//! collator sets, but they are not unbonded until their exit request is executed. Any signed
//! account may trigger the exit `T::LeaveCandidatesDelay` rounds after the round in which the
//! original request was made.
//!
//! To join the set of delegators, call `delegate` and pass in an account that is
//! already a collator candidate and `bond >= MinDelegatorStk`. Each delegator can delegate up to
//! `T::MaxDelegationsPerDelegator` collator candidates by calling `delegate`.
//!
//! To revoke a delegation, call `revoke_delegation` with the collator candidate's account.
//! To leave the set of delegators and revoke all delegations, call `leave_delegators`.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;
pub mod inflation;
pub mod migrations;
#[cfg(test)]
mod mock;
mod set;
#[cfg(test)]
mod tests;
pub mod weights;

use frame_support::pallet;
pub use inflation::{InflationInfo, Range};
use weights::WeightInfo;

pub use pallet::*;

#[pallet]
pub mod pallet {
	use crate::{set::OrderedSet, InflationInfo, Range, WeightInfo};
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, Get, Imbalance, ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::{Decode, Encode};
	use scale_info::TypeInfo;
	use sp_runtime::{
		traits::{AtLeast32BitUnsigned, Saturating, Zero},
		Perbill, Percent, RuntimeDebug,
	};
	use sp_std::{cmp::Ordering, collections::btree_map::BTreeMap, prelude::*};

	/// Pallet for parachain staking
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[derive(Default, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct Bond<AccountId, Balance> {
		pub owner: AccountId,
		pub amount: Balance,
	}

	impl<A, B: Default> Bond<A, B> {
		fn from_owner(owner: A) -> Self {
			Bond {
				owner,
				amount: B::default(),
			}
		}
	}

	impl<AccountId: Ord, Balance> Eq for Bond<AccountId, Balance> {}

	impl<AccountId: Ord, Balance> Ord for Bond<AccountId, Balance> {
		fn cmp(&self, other: &Self) -> Ordering {
			self.owner.cmp(&other.owner)
		}
	}

	impl<AccountId: Ord, Balance> PartialOrd for Bond<AccountId, Balance> {
		fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
			Some(self.cmp(other))
		}
	}

	impl<AccountId: Ord, Balance> PartialEq for Bond<AccountId, Balance> {
		fn eq(&self, other: &Self) -> bool {
			self.owner == other.owner
		}
	}

	#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// The activity status of the collator
	pub enum CollatorStatus {
		/// Committed to be online and producing valid blocks (not equivocating)
		Active,
		/// Temporarily inactive and excused for inactivity
		Idle,
		/// Bonded until the inner round
		Leaving(RoundIndex),
	}

	impl Default for CollatorStatus {
		fn default() -> CollatorStatus {
			CollatorStatus::Active
		}
	}

	#[derive(Default, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Snapshot of collator state at the start of the round for which they are selected
	pub struct CollatorSnapshot<AccountId, Balance> {
		pub bond: Balance,
		pub delegations: Vec<Bond<AccountId, Balance>>,
		pub total: Balance,
	}

	#[derive(Default, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Info needed to make delayed payments to stakers after round end
	pub struct DelayedPayout<Balance> {
		/// Total round reward (result of compute_issuance() at round end)
		pub round_issuance: Balance,
		/// The total inflation paid this round to stakers (e.g. less parachain bond fund)
		pub total_staking_reward: Balance,
		/// Snapshot of collator commission rate at the end of the round
		pub collator_commission: Perbill,
	}

	#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
	/// DEPRECATED
	/// Collator state with commission fee, bonded stake, and delegations
	pub struct Collator2<AccountId, Balance> {
		/// The account of this collator
		pub id: AccountId,
		/// This collator's self stake.
		pub bond: Balance,
		/// Set of all nominator AccountIds (to prevent >1 nomination per AccountId)
		pub nominators: OrderedSet<AccountId>,
		/// Top T::MaxDelegatorsPerCollator::get() nominators, ordered greatest to least
		pub top_nominators: Vec<Bond<AccountId, Balance>>,
		/// Bottom nominators (unbounded), ordered least to greatest
		pub bottom_nominators: Vec<Bond<AccountId, Balance>>,
		/// Sum of top delegations + self.bond
		pub total_counted: Balance,
		/// Sum of all delegations + self.bond = (total_counted + uncounted)
		pub total_backing: Balance,
		/// Current status of the collator
		pub state: CollatorStatus,
	}

	impl<A, B> From<Collator2<A, B>> for CollatorCandidate<A, B> {
		fn from(other: Collator2<A, B>) -> CollatorCandidate<A, B> {
			CollatorCandidate {
				id: other.id,
				bond: other.bond,
				delegators: other.nominators,
				top_delegations: other.top_nominators,
				bottom_delegations: other.bottom_nominators,
				total_counted: other.total_counted,
				total_backing: other.total_backing,
				request: None,
				state: other.state,
			}
		}
	}

	#[derive(PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Request scheduled to change the collator candidate self-bond
	pub struct CandidateBondLessRequest<Balance> {
		pub amount: Balance,
		pub when_executable: RoundIndex,
	}

	#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Collator candidate state with self bond + delegations
	pub struct CollatorCandidate<AccountId, Balance> {
		/// The account of this collator
		pub id: AccountId,
		/// This collator's self stake.
		pub bond: Balance,
		/// Set of all delegator AccountIds (to prevent >1 delegation per AccountId)
		pub delegators: OrderedSet<AccountId>,
		/// Top T::MaxDelegatorsPerCollator::get() delegations, ordered greatest to least
		pub top_delegations: Vec<Bond<AccountId, Balance>>,
		/// Bottom delegations (unbounded), ordered least to greatest
		pub bottom_delegations: Vec<Bond<AccountId, Balance>>,
		/// Sum of top delegations + self.bond
		pub total_counted: Balance,
		/// Sum of all delegations + self.bond = (total_counted + uncounted)
		pub total_backing: Balance,
		/// Maximum 1 pending request to decrease candidate self bond at any given time
		pub request: Option<CandidateBondLessRequest<Balance>>,
		/// Current status of the collator
		pub state: CollatorStatus,
	}

	// Temporary manual implementation for migration testing purposes
	impl<A: PartialEq, B: PartialEq> PartialEq for CollatorCandidate<A, B> {
		fn eq(&self, other: &Self) -> bool {
			let must_be_true = self.id == other.id
				&& self.bond == other.bond
				&& self.total_counted == other.total_counted
				&& self.total_backing == other.total_backing
				&& self.request == other.request
				&& self.state == other.state;
			if !must_be_true {
				return false;
			}
			for (x, y) in self.delegators.0.iter().zip(other.delegators.0.iter()) {
				if x != y {
					return false;
				}
			}
			for (
				Bond {
					owner: o1,
					amount: a1,
				},
				Bond {
					owner: o2,
					amount: a2,
				},
			) in self
				.top_delegations
				.iter()
				.zip(other.top_delegations.iter())
			{
				if o1 != o2 || a1 != a2 {
					return false;
				}
			}
			for (
				Bond {
					owner: o1,
					amount: a1,
				},
				Bond {
					owner: o2,
					amount: a2,
				},
			) in self
				.bottom_delegations
				.iter()
				.zip(other.bottom_delegations.iter())
			{
				if o1 != o2 || a1 != a2 {
					return false;
				}
			}
			true
		}
	}

	/// Convey relevant information describing if a delegator was added to the top or bottom
	/// Delegations added to the top yield a new total
	#[derive(Clone, Copy, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub enum DelegatorAdded<B> {
		AddedToTop { new_total: B },
		AddedToBottom,
	}

	impl<
			A: Ord + Clone + sp_std::fmt::Debug,
			B: AtLeast32BitUnsigned
				+ Ord
				+ Copy
				+ sp_std::ops::AddAssign
				+ sp_std::ops::SubAssign
				+ sp_std::fmt::Debug,
		> CollatorCandidate<A, B>
	{
		pub fn new(id: A, bond: B) -> Self {
			CollatorCandidate {
				id,
				bond,
				delegators: OrderedSet::new(),
				top_delegations: Vec::new(),
				bottom_delegations: Vec::new(),
				total_counted: bond,
				total_backing: bond,
				request: None,
				state: CollatorStatus::default(), // default active
			}
		}
		pub fn is_active(&self) -> bool {
			self.state == CollatorStatus::Active
		}
		pub fn is_leaving(&self) -> bool {
			matches!(self.state, CollatorStatus::Leaving(_))
		}
		pub fn can_leave<T: Config>(&self) -> DispatchResult {
			if let CollatorStatus::Leaving(when) = self.state {
				ensure!(
					<Round<T>>::get().current >= when,
					Error::<T>::CandidateCannotLeaveYet
				);
				Ok(())
			} else {
				Err(Error::<T>::CandidateNotLeaving.into())
			}
		}
		pub fn bond_more<T: Config>(&mut self, more: B) -> DispatchResult
		where
			BalanceOf<T>: From<B>,
			T::AccountId: From<A>,
			CollatorCandidate<T::AccountId, BalanceOf<T>>: From<Self>,
		{
			let caller: T::AccountId = self.id.clone().into();
			T::Currency::reserve(&caller, more.into())?;
			let new_total = <Total<T>>::get().saturating_add(more.into());
			<Total<T>>::put(new_total);
			self.bond += more;
			self.total_counted += more;
			self.total_backing += more;
			<Pallet<T>>::deposit_event(Event::CandidateBondedMore(
				self.id.clone().into(),
				more.into(),
				self.bond.into(),
			));
			Ok(())
		}
		/// Schedule executable decrease of collator candidate self bond
		/// Returns the round at which the collator can execute the pending request
		pub fn schedule_bond_less<T: Config>(
			&mut self,
			less: B,
		) -> Result<RoundIndex, DispatchError>
		where
			BalanceOf<T>: Into<B>,
		{
			// ensure no pending request
			ensure!(
				self.request.is_none(),
				Error::<T>::PendingCandidateRequestAlreadyExists
			);
			// ensure bond above min after decrease
			ensure!(self.bond > less, Error::<T>::CandidateBondBelowMin);
			ensure!(
				self.bond - less >= T::MinCandidateStk::get().into(),
				Error::<T>::CandidateBondBelowMin
			);
			let when_executable = <Round<T>>::get().current + T::CandidateBondLessDelay::get();
			self.request = Some(CandidateBondLessRequest {
				amount: less,
				when_executable,
			});
			Ok(when_executable)
		}
		/// Execute pending request to change the collator self bond
		/// Returns the event to be emitted
		pub fn execute_pending_request<T: Config>(&mut self) -> Result<Event<T>, DispatchError>
		where
			BalanceOf<T>: From<B>,
			T::AccountId: From<A>,
		{
			let request = self
				.request
				.ok_or(Error::<T>::PendingCandidateRequestsDNE)?;
			ensure!(
				request.when_executable <= <Round<T>>::get().current,
				Error::<T>::PendingCandidateRequestNotDueYet
			);
			let caller: T::AccountId = self.id.clone().into();
			T::Currency::unreserve(&caller, request.amount.into());
			let new_total_staked = <Total<T>>::get().saturating_sub(request.amount.into());
			<Total<T>>::put(new_total_staked);
			// Arithmetic assumptions are self.bond > less && self.bond - less > CollatorMinBond
			// (assumptions enforced by `schedule_bond_less`; if storage corrupts, must re-verify)
			self.bond -= request.amount;
			self.total_counted -= request.amount;
			self.total_backing -= request.amount;
			let event = Event::CandidateBondedLess(
				self.id.clone().into(),
				request.amount.into(),
				self.bond.into(),
			);
			// reset s.t. no pending request
			self.request = None;
			// update candidate pool value because it must change if self bond changes
			if self.is_active() {
				Pallet::<T>::update_active(self.id.clone().into(), self.total_counted.into());
			}
			Ok(event)
		}
		/// Cancel pending request to change the collator self bond
		pub fn cancel_pending_request<T: Config>(&mut self) -> Result<Event<T>, DispatchError>
		where
			CandidateBondLessRequest<BalanceOf<T>>: From<CandidateBondLessRequest<B>>,
			T::AccountId: From<A>,
			BalanceOf<T>: From<B>,
		{
			let request = self
				.request
				.ok_or(Error::<T>::PendingCandidateRequestsDNE)?;
			let event = Event::CancelledCandidateBondLess(
				self.id.clone().into(),
				request.amount.into(),
				request.when_executable,
			);
			self.request = None;
			Ok(event)
		}
		/// Infallible sorted insertion
		/// caller must verify !self.delegators.contains(delegation.owner) before call
		pub fn add_top_delegation(&mut self, delegation: Bond<A, B>) {
			match self
				.top_delegations
				.binary_search_by(|x| delegation.amount.cmp(&x.amount))
			{
				Ok(i) => self.top_delegations.insert(i, delegation),
				Err(i) => self.top_delegations.insert(i, delegation),
			}
		}
		/// Infallible sorted insertion
		/// caller must verify !self.delegators.contains(delegation.owner) before call
		pub fn add_bottom_delegation(&mut self, delegation: Bond<A, B>) {
			match self
				.bottom_delegations
				.binary_search_by(|x| x.amount.cmp(&delegation.amount))
			{
				Ok(i) => self.bottom_delegations.insert(i, delegation),
				Err(i) => self.bottom_delegations.insert(i, delegation),
			}
		}
		/// Sort top delegations from greatest to least
		pub fn sort_top_delegations(&mut self) {
			self.top_delegations
				.sort_unstable_by(|a, b| b.amount.cmp(&a.amount));
		}
		/// Sort bottom delegations from least to greatest
		pub fn sort_bottom_delegations(&mut self) {
			self.bottom_delegations
				.sort_unstable_by(|a, b| a.amount.cmp(&b.amount));
		}
		/// Bond account and add delegation. If successful, the return value indicates whether the
		/// delegation is top for the candidate.
		pub fn add_delegation<T: Config>(
			&mut self,
			acc: A,
			amount: B,
		) -> Result<DelegatorAdded<B>, DispatchError> {
			ensure!(
				self.delegators.insert(acc.clone()),
				Error::<T>::DelegatorExists
			);
			self.total_backing += amount;
			if (self.top_delegations.len() as u32) < T::MaxDelegatorsPerCandidate::get() {
				self.add_top_delegation(Bond { owner: acc, amount });
				self.total_counted += amount;
				Ok(DelegatorAdded::AddedToTop {
					new_total: self.total_counted,
				})
			} else {
				// >pop requires push to reset in case isn't pushed to bottom
				let last_delegation_in_top = self
					.top_delegations
					.pop()
					.expect("self.top_delegations.len() >= T::Max exists >= 1 element in top");
				if amount > last_delegation_in_top.amount {
					// update total_counted with positive difference
					self.total_counted += amount - last_delegation_in_top.amount;
					// last delegation already popped from top_delegations
					// insert new delegation into top_delegations
					self.add_top_delegation(Bond { owner: acc, amount });
					self.add_bottom_delegation(last_delegation_in_top);
					Ok(DelegatorAdded::AddedToTop {
						new_total: self.total_counted,
					})
				} else {
					// >required push to previously popped last delegation into top_delegations
					self.top_delegations.push(last_delegation_in_top);
					self.add_bottom_delegation(Bond { owner: acc, amount });
					Ok(DelegatorAdded::AddedToBottom)
				}
			}
		}
		/// Return Ok((if_total_counted_changed, delegation_amount))
		pub fn rm_delegator<T: Config>(
			&mut self,
			delegator: A,
		) -> Result<(bool, B), DispatchError> {
			ensure!(
				self.delegators.remove(&delegator),
				Error::<T>::DelegatorDNEInDelegatorSet
			);
			let mut delegation_amt: Option<B> = None;
			self.top_delegations = self
				.top_delegations
				.clone()
				.into_iter()
				.filter_map(|d| {
					if d.owner != delegator {
						Some(d)
					} else {
						delegation_amt = Some(d.amount);
						None
					}
				})
				.collect();
			// item removed from the top => highest bottom is popped from bottom and pushed to top
			if let Some(amount) = delegation_amt {
				// last element has largest amount as per ordering
				if let Some(last) = self.bottom_delegations.pop() {
					self.total_counted -= amount - last.amount;
					self.add_top_delegation(last);
				} else {
					// no item in bottom delegations so no item from bottom to pop and push up
					self.total_counted -= amount;
				}
				self.total_backing -= amount;
				return Ok((true, amount));
			}
			// else (no item removed from the top)
			self.bottom_delegations = self
				.bottom_delegations
				.clone()
				.into_iter()
				.filter_map(|d| {
					if d.owner != delegator {
						Some(d)
					} else {
						delegation_amt = Some(d.amount);
						None
					}
				})
				.collect();
			// if err, no item with account exists in top || bottom
			let amount = delegation_amt.ok_or(Error::<T>::DelegatorDNEinTopNorBottom)?;
			self.total_backing -= amount;
			Ok((false, amount))
		}
		/// Return true if in_top after call
		/// Caller must verify before call that account is a delegator
		fn increase_delegation(&mut self, delegator: A, more: B) -> bool {
			let mut in_top = false;
			for x in &mut self.top_delegations {
				if x.owner == delegator {
					x.amount += more;
					self.total_counted += more;
					self.total_backing += more;
					in_top = true;
					break;
				}
			}
			// if delegator was increased in top delegations
			if in_top {
				self.sort_top_delegations();
				return true;
			}
			// else delegator to increase must exist in bottom
			// >pop requires push later on to reset in case it isn't used
			let lowest_top = self
				.top_delegations
				.pop()
				.expect("any bottom delegations => must exist max top delegations");
			let mut move_2_top = false;
			for x in &mut self.bottom_delegations {
				if x.owner == delegator {
					x.amount += more;
					self.total_backing += more;
					move_2_top = x.amount > lowest_top.amount;
					break;
				}
			}
			if move_2_top {
				self.sort_bottom_delegations();
				let highest_bottom = self.bottom_delegations.pop().expect("updated => exists");
				self.total_counted += highest_bottom.amount - lowest_top.amount;
				self.add_top_delegation(highest_bottom);
				self.add_bottom_delegation(lowest_top);
				true
			} else {
				// >required push to reset top_delegations from earlier pop
				self.top_delegations.push(lowest_top);
				self.sort_bottom_delegations();
				false
			}
		}
		/// Return true if in_top after call
		pub fn decrease_delegation(&mut self, delegator: A, less: B) -> bool {
			let mut in_top = false;
			let mut new_lowest_top: Option<Bond<A, B>> = None;
			for x in &mut self.top_delegations {
				if x.owner == delegator {
					x.amount -= less;
					// if there is at least 1 delegator in bottom delegators, compare it to check
					// if it should be swapped with lowest top delegation and put in top
					// >pop requires push later on to reset in case it isn't used
					if let Some(highest_bottom) = self.bottom_delegations.pop() {
						if highest_bottom.amount > x.amount {
							new_lowest_top = Some(highest_bottom);
						} else {
							// >required push to reset self.bottom_delegations
							self.bottom_delegations.push(highest_bottom);
						}
					}
					in_top = true;
					break;
				}
			}
			if in_top {
				self.sort_top_delegations();
				if let Some(highest_bottom) = new_lowest_top {
					// pop last in top to swap it with top bottom
					let lowest_top = self
						.top_delegations
						.pop()
						.expect("must have >1 item to update, assign in_top = true");
					self.total_counted -= lowest_top.amount + less;
					self.total_counted += highest_bottom.amount;
					self.total_backing -= less;
					self.add_top_delegation(highest_bottom);
					self.add_bottom_delegation(lowest_top);
					return false;
				} else {
					// no existing bottom delegators so update both counters the same magnitude
					self.total_counted -= less;
					self.total_backing -= less;
					return true;
				}
			}
			for x in &mut self.bottom_delegations {
				if x.owner == delegator {
					x.amount -= less;
					self.total_backing -= less;
					break;
				}
			}
			self.sort_bottom_delegations();
			false
		}
		pub fn go_offline(&mut self) {
			self.state = CollatorStatus::Idle;
		}
		pub fn go_online(&mut self) {
			self.state = CollatorStatus::Active;
		}
		pub fn leave<T: Config>(&mut self) -> Result<(RoundIndex, RoundIndex), DispatchError> {
			ensure!(!self.is_leaving(), Error::<T>::CandidateAlreadyLeaving);
			let now = <Round<T>>::get().current;
			let when = now + T::LeaveCandidatesDelay::get();
			self.state = CollatorStatus::Leaving(when);
			Ok((now, when))
		}
	}

	impl<A: Clone, B: Copy> From<CollatorCandidate<A, B>> for CollatorSnapshot<A, B> {
		fn from(other: CollatorCandidate<A, B>) -> CollatorSnapshot<A, B> {
			CollatorSnapshot {
				bond: other.bond,
				delegations: other.top_delegations,
				total: other.total_counted,
			}
		}
	}

	#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub enum DelegatorStatus {
		/// Active with no scheduled exit
		Active,
		/// Schedule exit to revoke all ongoing delegations
		Leaving(RoundIndex),
	}

	#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Delegator state
	pub struct Delegator<AccountId, Balance> {
		/// Delegator account
		pub id: AccountId,
		/// All current delegations
		pub delegations: OrderedSet<Bond<AccountId, Balance>>,
		/// Total balance locked for this delegator
		pub total: Balance,
		/// Requests to change delegations, relevant iff active
		pub requests: PendingDelegationRequests<AccountId, Balance>,
		/// Status for this delegator
		pub status: DelegatorStatus,
	}

	// Temporary manual implementation for migration testing purposes
	impl<A: PartialEq, B: PartialEq> PartialEq for Delegator<A, B> {
		fn eq(&self, other: &Self) -> bool {
			let must_be_true = self.id == other.id
				&& self.total == other.total
				&& self.requests == other.requests
				&& self.status == other.status;
			if !must_be_true {
				return false;
			}
			for (
				Bond {
					owner: o1,
					amount: a1,
				},
				Bond {
					owner: o2,
					amount: a2,
				},
			) in self.delegations.0.iter().zip(other.delegations.0.iter())
			{
				if o1 != o2 || a1 != a2 {
					return false;
				}
			}
			true
		}
	}

	impl<
			AccountId: Ord + Clone + Default,
			Balance: Copy
				+ sp_std::ops::AddAssign
				+ sp_std::ops::Add<Output = Balance>
				+ sp_std::ops::SubAssign
				+ sp_std::ops::Sub<Output = Balance>
				+ Ord
				+ Zero
				+ Default,
		> Delegator<AccountId, Balance>
	{
		pub fn new(id: AccountId, collator: AccountId, amount: Balance) -> Self {
			Delegator {
				id,
				delegations: OrderedSet::from(vec![Bond {
					owner: collator,
					amount,
				}]),
				total: amount,
				requests: PendingDelegationRequests::new(),
				status: DelegatorStatus::Active,
			}
		}
		pub fn requests(&self) -> BTreeMap<AccountId, DelegationRequest<AccountId, Balance>> {
			self.requests.requests.clone()
		}
		pub fn is_active(&self) -> bool {
			matches!(self.status, DelegatorStatus::Active)
		}
		pub fn is_leaving(&self) -> bool {
			matches!(self.status, DelegatorStatus::Leaving(_))
		}
		/// Can only leave if the current round is less than or equal to scheduled execution round
		/// - returns None if not in leaving state
		pub fn can_execute_leave<T: Config>(&self, delegation_weight_hint: u32) -> DispatchResult {
			ensure!(
				delegation_weight_hint >= (self.delegations.0.len() as u32),
				Error::<T>::TooLowDelegationCountToLeaveDelegators
			);
			if let DelegatorStatus::Leaving(when) = self.status {
				ensure!(
					<Round<T>>::get().current >= when,
					Error::<T>::DelegatorCannotLeaveYet
				);
				Ok(())
			} else {
				Err(Error::<T>::DelegatorNotLeaving.into())
			}
		}
		/// Set status to leaving
		pub(crate) fn set_leaving(&mut self, when: RoundIndex) {
			self.status = DelegatorStatus::Leaving(when);
		}
		/// Schedule status to exit
		pub fn schedule_leave<T: Config>(&mut self) -> (RoundIndex, RoundIndex) {
			let now = <Round<T>>::get().current;
			let when = now + T::LeaveDelegatorsDelay::get();
			self.set_leaving(when);
			(now, when)
		}
		/// Set delegator status to active
		pub fn cancel_leave(&mut self) {
			self.status = DelegatorStatus::Active
		}
		pub fn add_delegation(&mut self, bond: Bond<AccountId, Balance>) -> bool {
			let amt = bond.amount;
			if self.delegations.insert(bond) {
				self.total += amt;
				true
			} else {
				false
			}
		}
		// Return Some(remaining balance), must be more than MinDelegatorStk
		// Return None if delegation not found
		pub fn rm_delegation(&mut self, collator: AccountId) -> Option<Balance> {
			let mut amt: Option<Balance> = None;
			let delegations = self
				.delegations
				.0
				.iter()
				.filter_map(|x| {
					if x.owner == collator {
						amt = Some(x.amount);
						None
					} else {
						Some(x.clone())
					}
				})
				.collect();
			if let Some(balance) = amt {
				self.delegations = OrderedSet::from(delegations);
				self.total -= balance;
				Some(self.total)
			} else {
				None
			}
		}
		pub fn increase_delegation<T: Config>(
			&mut self,
			candidate: AccountId,
			amount: Balance,
		) -> DispatchResult
		where
			BalanceOf<T>: From<Balance>,
			T::AccountId: From<AccountId>,
			Delegator<T::AccountId, BalanceOf<T>>: From<Delegator<AccountId, Balance>>,
		{
			let delegator_id = self.id.clone().into();
			let candidate_id: T::AccountId = candidate.clone().into();
			let balance_amt: BalanceOf<T> = amount.into();
			// increase delegation
			for x in &mut self.delegations.0 {
				if x.owner == candidate {
					x.amount += amount;
					self.total += amount;
					// update collator state delegation
					let mut collator_state =
						<CandidateState<T>>::get(&candidate_id).ok_or(Error::<T>::CandidateDNE)?;
					T::Currency::reserve(&self.id.clone().into(), balance_amt)?;
					let before = collator_state.total_counted;
					let in_top =
						collator_state.increase_delegation(self.id.clone().into(), balance_amt);
					let after = collator_state.total_counted;
					if collator_state.is_active() && (before != after) {
						Pallet::<T>::update_active(candidate_id.clone(), after);
					}
					<CandidateState<T>>::insert(&candidate_id, collator_state);
					let new_total_staked = <Total<T>>::get().saturating_add(balance_amt);
					<Total<T>>::put(new_total_staked);
					let nom_st: Delegator<T::AccountId, BalanceOf<T>> = self.clone().into();
					<DelegatorState<T>>::insert(&delegator_id, nom_st);
					Pallet::<T>::deposit_event(Event::DelegationIncreased(
						delegator_id,
						candidate_id,
						balance_amt,
						in_top,
					));
					return Ok(());
				}
			}
			Err(Error::<T>::DelegationDNE.into())
		}
		/// Schedule decrease delegation
		pub fn schedule_decrease_delegation<T: Config>(
			&mut self,
			collator: AccountId,
			less: Balance,
		) -> Result<RoundIndex, DispatchError>
		where
			BalanceOf<T>: Into<Balance> + From<Balance>,
		{
			// get delegation amount
			let Bond { amount, .. } = self
				.delegations
				.0
				.iter()
				.find(|b| b.owner == collator)
				.ok_or(Error::<T>::DelegationDNE)?;
			ensure!(*amount > less, Error::<T>::DelegatorBondBelowMin);
			let expected_amt: BalanceOf<T> = (*amount - less).into();
			ensure!(
				expected_amt >= T::MinDelegation::get(),
				Error::<T>::DelegationBelowMin
			);
			// Net Total is total after pending orders are executed
			let net_total = self.total - self.requests.less_total;
			// Net Total is always >= MinDelegatorStk
			let max_subtracted_amount = net_total - T::MinDelegatorStk::get().into();
			ensure!(
				less <= max_subtracted_amount,
				Error::<T>::DelegatorBondBelowMin
			);
			let when = <Round<T>>::get().current + T::DelegationBondLessDelay::get();
			self.requests.bond_less::<T>(collator, less, when)?;
			Ok(when)
		}
		/// Temporary function to migrate revocations
		pub fn hotfix_set_revoke<T: Config>(&mut self, collator: AccountId, when: RoundIndex) {
			// get delegation amount
			let maybe_bond = self.delegations.0.iter().find(|b| b.owner == collator);
			if let Some(Bond { amount, .. }) = maybe_bond {
				// add revocation to pending requests
				if let Err(e) = self.requests.revoke::<T>(collator, *amount, when) {
					log::warn!("Migrate revocation request failed with error: {:?}", e);
				}
			} else {
				log::warn!("Migrate revocation request failed because delegation DNE");
			}
		}
		/// Schedule revocation for the given collator
		pub fn schedule_revoke<T: Config>(
			&mut self,
			collator: AccountId,
		) -> Result<(RoundIndex, RoundIndex), DispatchError>
		where
			BalanceOf<T>: Into<Balance>,
		{
			// get delegation amount
			let Bond { amount, .. } = self
				.delegations
				.0
				.iter()
				.find(|b| b.owner == collator)
				.ok_or(Error::<T>::DelegationDNE)?;
			let now = <Round<T>>::get().current;
			let when = now + T::RevokeDelegationDelay::get();
			// add revocation to pending requests
			self.requests.revoke::<T>(collator, *amount, when)?;
			Ok((now, when))
		}
		/// Execute pending delegation change request
		pub fn execute_pending_request<T: Config>(&mut self, candidate: AccountId) -> DispatchResult
		where
			BalanceOf<T>: From<Balance> + Into<Balance>,
			T::AccountId: From<AccountId>,
			Delegator<T::AccountId, BalanceOf<T>>: From<Delegator<AccountId, Balance>>,
		{
			let now = <Round<T>>::get().current;
			let DelegationRequest {
				amount,
				action,
				when_executable,
				..
			} = self
				.requests
				.requests
				.remove(&candidate)
				.ok_or(Error::<T>::PendingDelegationRequestDNE)?;
			ensure!(
				when_executable <= now,
				Error::<T>::PendingDelegationRequestNotDueYet
			);
			let (balance_amt, candidate_id, delegator_id): (
				BalanceOf<T>,
				T::AccountId,
				T::AccountId,
			) = (
				amount.into(),
				candidate.clone().into(),
				self.id.clone().into(),
			);
			match action {
				DelegationChange::Revoke => {
					// revoking last delegation => leaving set of delegators
					let leaving = if self.delegations.0.len() == 1usize {
						true
					} else {
						ensure!(
							self.total - T::MinDelegatorStk::get().into() >= amount,
							Error::<T>::DelegatorBondBelowMin
						);
						false
					};
					// remove from pending requests
					self.requests.less_total -= amount;
					self.requests.revocations_count -= 1u32;
					// remove delegation from delegator state
					self.rm_delegation(candidate.clone());
					// remove delegation from collator state delegations
					Pallet::<T>::delegator_leaves_collator(
						delegator_id.clone(),
						candidate_id.clone(),
					)?;
					Pallet::<T>::deposit_event(Event::DelegationRevoked(
						delegator_id.clone(),
						candidate_id,
						balance_amt,
					));
					if leaving {
						<DelegatorState<T>>::remove(&delegator_id);
						Pallet::<T>::deposit_event(Event::DelegatorLeft(delegator_id, balance_amt));
					} else {
						let nom_st: Delegator<T::AccountId, BalanceOf<T>> = self.clone().into();
						<DelegatorState<T>>::insert(&delegator_id, nom_st);
					}
					Ok(())
				}
				DelegationChange::Decrease => {
					// remove from pending requests
					self.requests.less_total -= amount;
					// decrease delegation
					for x in &mut self.delegations.0 {
						if x.owner == candidate {
							if x.amount > amount {
								x.amount -= amount;
								self.total -= amount;
								let new_total: BalanceOf<T> = self.total.into();
								ensure!(
									new_total >= T::MinDelegation::get(),
									Error::<T>::DelegationBelowMin
								);
								ensure!(
									new_total >= T::MinDelegatorStk::get(),
									Error::<T>::DelegatorBondBelowMin
								);
								let mut collator = <CandidateState<T>>::get(&candidate_id)
									.ok_or(Error::<T>::CandidateDNE)?;
								T::Currency::unreserve(&delegator_id, balance_amt);
								let before = collator.total_counted;
								// need to go into decrease_delegation
								let in_top =
									collator.decrease_delegation(delegator_id.clone(), balance_amt);
								let after = collator.total_counted;
								if collator.is_active() && (before != after) {
									Pallet::<T>::update_active(candidate_id.clone(), after);
								}
								<CandidateState<T>>::insert(&candidate_id, collator);
								let new_total_staked =
									<Total<T>>::get().saturating_sub(balance_amt);
								<Total<T>>::put(new_total_staked);
								let nom_st: Delegator<T::AccountId, BalanceOf<T>> =
									self.clone().into();
								<DelegatorState<T>>::insert(&delegator_id, nom_st);
								Pallet::<T>::deposit_event(Event::DelegationDecreased(
									delegator_id,
									candidate_id,
									balance_amt,
									in_top,
								));
								return Ok(());
							} else {
								// must rm entire delegation if x.amount <= less or cancel request
								return Err(Error::<T>::DelegationBelowMin.into());
							}
						}
					}
					Err(Error::<T>::DelegationDNE.into())
				}
			}
		}
		/// Cancel pending delegation change request
		pub fn cancel_pending_request<T: Config>(
			&mut self,
			candidate: AccountId,
		) -> Result<DelegationRequest<AccountId, Balance>, DispatchError> {
			let order = self
				.requests
				.requests
				.remove(&candidate)
				.ok_or(Error::<T>::PendingDelegationRequestDNE)?;
			match order.action {
				DelegationChange::Revoke => {
					self.requests.revocations_count -= 1u32;
					self.requests.less_total -= order.amount;
				}
				DelegationChange::Decrease => {
					self.requests.less_total -= order.amount;
				}
			}
			Ok(order)
		}
	}

	#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Changes requested by the delegator
	/// - limit of 1 ongoing change per delegation
	pub enum DelegationChange {
		Revoke,
		Decrease,
	}

	#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct DelegationRequest<AccountId, Balance> {
		pub collator: AccountId,
		pub amount: Balance,
		pub when_executable: RoundIndex,
		pub action: DelegationChange,
	}

	#[derive(Clone, Encode, PartialEq, Decode, RuntimeDebug, TypeInfo)]
	/// Pending requests to mutate delegations for each delegator
	pub struct PendingDelegationRequests<AccountId, Balance> {
		/// Number of pending revocations (necessary for determining whether revoke is exit)
		pub revocations_count: u32,
		/// Map from collator -> Request (enforces at most 1 pending request per delegation)
		pub requests: BTreeMap<AccountId, DelegationRequest<AccountId, Balance>>,
		/// Sum of pending revocation amounts + bond less amounts
		pub less_total: Balance,
	}

	impl<A: Ord, B: Zero> Default for PendingDelegationRequests<A, B> {
		fn default() -> PendingDelegationRequests<A, B> {
			PendingDelegationRequests {
				revocations_count: 0u32,
				requests: BTreeMap::new(),
				less_total: B::zero(),
			}
		}
	}

	impl<
			A: Ord + Clone,
			B: Zero
				+ Ord
				+ Copy
				+ Clone
				+ sp_std::ops::AddAssign
				+ sp_std::ops::Add<Output = B>
				+ sp_std::ops::SubAssign
				+ sp_std::ops::Sub<Output = B>,
		> PendingDelegationRequests<A, B>
	{
		/// New default (empty) pending requests
		pub fn new() -> PendingDelegationRequests<A, B> {
			PendingDelegationRequests::default()
		}
		/// Add bond less order to pending requests, only succeeds if returns true
		/// - limit is the maximum amount allowed that can be subtracted from the delegation
		/// before it would be below the minimum delegation amount
		pub fn bond_less<T: Config>(
			&mut self,
			collator: A,
			amount: B,
			when_executable: RoundIndex,
		) -> DispatchResult {
			ensure!(
				self.requests.get(&collator).is_none(),
				Error::<T>::PendingDelegationRequestAlreadyExists
			);
			self.requests.insert(
				collator.clone(),
				DelegationRequest {
					collator,
					amount,
					when_executable,
					action: DelegationChange::Decrease,
				},
			);
			self.less_total += amount;
			Ok(())
		}
		/// Add revoke order to pending requests
		/// - limit is the maximum amount allowed that can be subtracted from the delegation
		/// before it would be below the minimum delegation amount
		pub fn revoke<T: Config>(
			&mut self,
			collator: A,
			amount: B,
			when_executable: RoundIndex,
		) -> DispatchResult {
			ensure!(
				self.requests.get(&collator).is_none(),
				Error::<T>::PendingDelegationRequestAlreadyExists
			);
			self.requests.insert(
				collator.clone(),
				DelegationRequest {
					collator,
					amount,
					when_executable,
					action: DelegationChange::Revoke,
				},
			);
			self.revocations_count += 1u32;
			self.less_total += amount;
			Ok(())
		}
	}

	#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// DEPRECATED in favor of Delegator
	/// Nominator state
	pub struct Nominator2<AccountId, Balance> {
		/// All current delegations
		pub delegations: OrderedSet<Bond<AccountId, Balance>>,
		/// Delegations scheduled to be revoked
		pub revocations: OrderedSet<AccountId>,
		/// Total balance locked for this nominator
		pub total: Balance,
		/// Total number of revocations scheduled to be executed
		pub scheduled_revocations_count: u32,
		/// Total amount to be unbonded once revocations are executed
		pub scheduled_revocations_total: Balance,
		/// Status for this nominator
		pub status: DelegatorStatus,
	}

	/// Temporary function to migrate state
	pub(crate) fn migrate_nominator_to_delegator_state<T: Config>(
		id: T::AccountId,
		nominator: Nominator2<T::AccountId, BalanceOf<T>>,
	) -> Delegator<T::AccountId, BalanceOf<T>> {
		Delegator {
			id,
			delegations: nominator.delegations,
			total: nominator.total,
			requests: PendingDelegationRequests::new(),
			status: nominator.status,
		}
	}

	#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// The current round index and transition information
	pub struct RoundInfo<BlockNumber> {
		/// Current round index
		pub current: RoundIndex,
		/// The first block of the current round
		pub first: BlockNumber,
		/// The length of the current round in number of blocks
		pub length: u32,
	}
	impl<
			B: Copy
				+ sp_std::ops::Add<Output = B>
				+ sp_std::ops::Sub<Output = B>
				+ From<u32>
				+ PartialOrd,
		> RoundInfo<B>
	{
		pub fn new(current: RoundIndex, first: B, length: u32) -> RoundInfo<B> {
			RoundInfo {
				current,
				first,
				length,
			}
		}
		/// Check if the round should be updated
		pub fn should_update(&self, now: B) -> bool {
			now - self.first >= self.length.into()
		}
		/// New round
		pub fn update(&mut self, now: B) {
			self.current += 1u32;
			self.first = now;
		}
	}
	impl<
			B: Copy
				+ sp_std::ops::Add<Output = B>
				+ sp_std::ops::Sub<Output = B>
				+ From<u32>
				+ PartialOrd,
		> Default for RoundInfo<B>
	{
		fn default() -> RoundInfo<B> {
			RoundInfo::new(1u32, 1u32.into(), 20u32)
		}
	}

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Reserve information { account, percent_of_inflation }
	pub struct ParachainBondConfig<AccountId> {
		/// Account which receives funds intended for parachain bond
		pub account: AccountId,
		/// Percent of inflation set aside for parachain bond account
		pub percent: Percent,
	}
	impl<A: Default> Default for ParachainBondConfig<A> {
		fn default() -> ParachainBondConfig<A> {
			ParachainBondConfig {
				account: A::default(),
				percent: Percent::zero(),
			}
		}
	}

	#[derive(Encode, Decode, RuntimeDebug, Default, PartialEq, Eq, TypeInfo)]
	/// DEPRECATED
	/// Store and process all delayed exits by collators and nominators
	pub struct ExitQ<AccountId> {
		/// Candidate exit set
		pub candidates: OrderedSet<AccountId>,
		/// Nominator exit set (does not include nominators that made `revoke` requests)
		pub nominators_leaving: OrderedSet<AccountId>,
		/// [Candidate, Round to Exit]
		pub candidate_schedule: Vec<(AccountId, RoundIndex)>,
		/// [Nominator, Some(ValidatorId) || None => All Delegations, Round To Exit]
		pub nominator_schedule: Vec<(AccountId, Option<AccountId>, RoundIndex)>,
	}

	pub(crate) type RoundIndex = u32;
	type RewardPoint = u32;
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency type
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// The origin for monetary governance
		type MonetaryGovernanceOrigin: EnsureOrigin<Self::Origin>;
		/// Minimum number of blocks per round
		#[pallet::constant]
		type MinBlocksPerRound: Get<u32>;
		/// Default number of blocks per round at genesis
		#[pallet::constant]
		type DefaultBlocksPerRound: Get<u32>;
		/// Number of rounds that candidates remain bonded before exit request is executable
		#[pallet::constant]
		type LeaveCandidatesDelay: Get<RoundIndex>;
		/// Number of rounds candidate requests to decrease self-bond must wait to be executable
		#[pallet::constant]
		type CandidateBondLessDelay: Get<RoundIndex>;
		/// Number of rounds that delegators remain bonded before exit request is executable
		#[pallet::constant]
		type LeaveDelegatorsDelay: Get<RoundIndex>;
		/// Number of rounds that delegations remain bonded before revocation request is executable
		#[pallet::constant]
		type RevokeDelegationDelay: Get<RoundIndex>;
		/// Number of rounds that delegation less requests must wait before executable
		#[pallet::constant]
		type DelegationBondLessDelay: Get<RoundIndex>;
		/// Number of rounds after which block authors are rewarded
		#[pallet::constant]
		type RewardPaymentDelay: Get<RoundIndex>;
		/// Minimum number of selected candidates every round
		#[pallet::constant]
		type MinSelectedCandidates: Get<u32>;
		/// Maximum delegators counted per candidate
		#[pallet::constant]
		type MaxDelegatorsPerCandidate: Get<u32>;
		/// Maximum delegations per delegator
		#[pallet::constant]
		type MaxDelegationsPerDelegator: Get<u32>;
		/// Default commission due to collators, is `CollatorCommission` storage value in genesis
		#[pallet::constant]
		type DefaultCollatorCommission: Get<Perbill>;
		/// Default percent of inflation set aside for parachain bond account
		#[pallet::constant]
		type DefaultParachainBondReservePercent: Get<Percent>;
		/// Minimum stake required for any candidate to be in `SelectedCandidates` for the round
		#[pallet::constant]
		type MinCollatorStk: Get<BalanceOf<Self>>;
		/// Minimum stake required for any account to be a collator candidate
		#[pallet::constant]
		type MinCandidateStk: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to delegate
		#[pallet::constant]
		type MinDelegation: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to be a delegator
		#[pallet::constant]
		type MinDelegatorStk: Get<BalanceOf<Self>>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		DelegatorDNE,
		DelegatorDNEinTopNorBottom,
		DelegatorDNEInDelegatorSet,
		CandidateDNE,
		DelegationDNE,
		DelegatorExists,
		CandidateExists,
		CandidateBondBelowMin,
		InsufficientBalance,
		DelegatorBondBelowMin,
		DelegationBelowMin,
		AlreadyOffline,
		AlreadyActive,
		DelegatorAlreadyLeaving,
		DelegatorNotLeaving,
		DelegatorCannotLeaveYet,
		CannotDelegateIfLeaving,
		CandidateAlreadyLeaving,
		CandidateNotLeaving,
		CandidateCannotLeaveYet,
		CannotGoOnlineIfLeaving,
		ExceedMaxDelegationsPerDelegator,
		AlreadyDelegatedCandidate,
		InvalidSchedule,
		CannotSetBelowMin,
		RoundLengthMustBeAtLeastTotalSelectedCollators,
		NoWritingSameValue,
		TooLowCandidateCountWeightHintJoinCandidates,
		TooLowCandidateCountWeightHintCancelLeaveCandidates,
		TooLowCandidateCountToLeaveCandidates,
		TooLowDelegationCountToDelegate,
		TooLowCandidateDelegationCountToDelegate,
		TooLowDelegationCountToLeaveDelegators,
		PendingCandidateRequestsDNE,
		PendingCandidateRequestAlreadyExists,
		PendingCandidateRequestNotDueYet,
		PendingDelegationRequestDNE,
		PendingDelegationRequestAlreadyExists,
		PendingDelegationRequestNotDueYet,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Starting Block, Round, Number of Collators Selected, Total Balance
		NewRound(T::BlockNumber, RoundIndex, u32, BalanceOf<T>),
		/// Account, Amount Locked, New Total Amt Locked
		JoinedCollatorCandidates(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Round, Collator Account, Total Exposed Amount (includes all delegations)
		CollatorChosen(RoundIndex, T::AccountId, BalanceOf<T>),
		/// Candidate, Amount To Decrease, Round at which request can be executed by caller
		CandidateBondLessRequested(T::AccountId, BalanceOf<T>, RoundIndex),
		/// Candidate, Amount, New Bond Total
		CandidateBondedMore(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Candidate, Amount, New Bond
		CandidateBondedLess(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Round Offline, Candidate
		CandidateWentOffline(RoundIndex, T::AccountId),
		/// Round Online, Candidate
		CandidateBackOnline(RoundIndex, T::AccountId),
		/// Round At Which Exit Is Allowed, Candidate, Scheduled Exit
		CandidateScheduledExit(RoundIndex, T::AccountId, RoundIndex),
		/// Candidate
		CancelledCandidateExit(T::AccountId),
		/// Candidate, Amount, Round at which could be executed
		CancelledCandidateBondLess(T::AccountId, BalanceOf<T>, RoundIndex),
		/// Ex-Candidate, Amount Unlocked, New Total Amt Locked
		CandidateLeft(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Delegator, Candidate, Amount to be decreased, Round at which can be executed
		DelegationDecreaseScheduled(T::AccountId, T::AccountId, BalanceOf<T>, RoundIndex),
		// Delegator, Candidate, Amount, If in top delegations for candidate after increase
		DelegationIncreased(T::AccountId, T::AccountId, BalanceOf<T>, bool),
		// Delegator, Candidate, Amount, If in top delegations for candidate after decrease
		DelegationDecreased(T::AccountId, T::AccountId, BalanceOf<T>, bool),
		/// Round, Delegator, Scheduled Exit
		DelegatorExitScheduled(RoundIndex, T::AccountId, RoundIndex),
		/// Round, Delegator, Candidate, Scheduled Exit
		DelegationRevocationScheduled(RoundIndex, T::AccountId, T::AccountId, RoundIndex),
		/// Delegator, Amount Unstaked
		DelegatorLeft(T::AccountId, BalanceOf<T>),
		/// Delegator, Candidate, Amount Unstaked
		DelegationRevoked(T::AccountId, T::AccountId, BalanceOf<T>),
		/// Delegator
		DelegatorExitCancelled(T::AccountId),
		/// Delegator, Cancelled Request
		CancelledDelegationRequest(T::AccountId, DelegationRequest<T::AccountId, BalanceOf<T>>),
		/// Delegator, Amount Locked, Candidate, Delegator Position with New Total Counted if in Top
		Delegation(
			T::AccountId,
			BalanceOf<T>,
			T::AccountId,
			DelegatorAdded<BalanceOf<T>>,
		),
		/// Delegator, Candidate, Amount Unstaked, New Total Amt Staked for Candidate
		DelegatorLeftCandidate(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Paid the account (delegator or collator) the balance as liquid rewards
		Rewarded(T::AccountId, BalanceOf<T>),
		/// Transferred to account which holds funds reserved for parachain bond
		ReservedForParachainBond(T::AccountId, BalanceOf<T>),
		/// Account (re)set for parachain bond treasury [old, new]
		ParachainBondAccountSet(T::AccountId, T::AccountId),
		/// Percent of inflation reserved for parachain bond (re)set [old, new]
		ParachainBondReservePercentSet(Percent, Percent),
		/// Annual inflation input (first 3) was used to derive new per-round inflation (last 3)
		InflationSet(Perbill, Perbill, Perbill, Perbill, Perbill, Perbill),
		/// Staking expectations set
		StakeExpectationsSet(BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
		/// Set total selected candidates to this value [old, new]
		TotalSelectedSet(u32, u32),
		/// Set collator commission to this value [old, new]
		CollatorCommissionSet(Perbill, Perbill),
		/// Set blocks per round [current_round, first_block, old, new, new_per_round_inflation]
		BlocksPerRoundSet(
			RoundIndex,
			T::BlockNumber,
			u32,
			u32,
			Perbill,
			Perbill,
			Perbill,
		),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> Weight {
			let mut weight = T::WeightInfo::base_on_initialize();

			let mut round = <Round<T>>::get();
			if round.should_update(n) {
				// mutate round
				round.update(n);
				// pay all stakers for T::RewardPaymentDelay rounds ago
				Self::prepare_staking_payouts(round.current);
				// select top collator candidates for next round
				let (collator_count, delegation_count, total_staked) =
					Self::select_top_candidates(round.current);
				// start next round
				<Round<T>>::put(round);
				// snapshot total stake
				<Staked<T>>::insert(round.current, <Total<T>>::get());
				Self::deposit_event(Event::NewRound(
					round.first,
					round.current,
					collator_count,
					total_staked,
				));
				weight +=
					T::WeightInfo::round_transition_on_initialize(collator_count, delegation_count);
			}

			weight += Self::handle_delayed_payouts(round.current);

			weight
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn collator_commission)]
	/// Commission percent taken off of rewards for all collators
	type CollatorCommission<T: Config> = StorageValue<_, Perbill, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_selected)]
	/// The total candidates selected every round
	type TotalSelected<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn parachain_bond_info)]
	/// Parachain bond config info { account, percent_of_inflation }
	type ParachainBondInfo<T: Config> =
		StorageValue<_, ParachainBondConfig<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn round)]
	/// Current round index and next round scheduled transition
	pub(crate) type Round<T: Config> = StorageValue<_, RoundInfo<T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nominator_state2)]
	/// DEPRECATED in favor of DelegatorState
	/// Get nominator state associated with an account if account is nominating else None
	pub(crate) type NominatorState2<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Nominator2<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn delegator_state)]
	/// Get delegator state associated with an account if account is delegating else None
	pub(crate) type DelegatorState<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Delegator<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn candidate_state)]
	/// Get collator candidate state associated with an account if account is a candidate else None
	pub(crate) type CandidateState<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		CollatorCandidate<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn collator_state2)]
	/// DEPRECATED in favor of CandidateState
	/// Get collator state associated with an account if account is collating else None
	pub(crate) type CollatorState2<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Collator2<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn selected_candidates)]
	/// The collator candidates selected for the current round
	type SelectedCandidates<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total)]
	/// Total capital locked by this staking pallet
	type Total<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn candidate_pool)]
	/// The pool of collator candidates, each with their total backing stake
	type CandidatePool<T: Config> =
		StorageValue<_, OrderedSet<Bond<T::AccountId, BalanceOf<T>>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn exit_queue2)]
	/// DEPRECATED, to be removed in future runtime upgrade but necessary for runtime migration
	/// A queue of collators and nominators awaiting exit
	pub type ExitQueue2<T: Config> = StorageValue<_, ExitQ<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn at_stake)]
	/// Snapshot of collator delegation stake at the start of the round
	pub type AtStake<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		RoundIndex,
		Twox64Concat,
		T::AccountId,
		CollatorSnapshot<T::AccountId, BalanceOf<T>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn delayed_payouts)]
	/// Delayed payouts
	pub type DelayedPayouts<T: Config> =
		StorageMap<_, Twox64Concat, RoundIndex, DelayedPayout<BalanceOf<T>>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn staked)]
	/// Total counted stake for selected candidates in the round
	pub type Staked<T: Config> = StorageMap<_, Twox64Concat, RoundIndex, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn inflation_config)]
	/// Inflation configuration
	pub type InflationConfig<T: Config> = StorageValue<_, InflationInfo<BalanceOf<T>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn points)]
	/// Total points awarded to collators for block production in the round
	pub type Points<T: Config> = StorageMap<_, Twox64Concat, RoundIndex, RewardPoint, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn awarded_pts)]
	/// Points for each collator per round
	pub type AwardedPts<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		RoundIndex,
		Twox64Concat,
		T::AccountId,
		RewardPoint,
		ValueQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub candidates: Vec<(T::AccountId, BalanceOf<T>)>,
		/// Vec of tuples of the format (delegator AccountId, collator AccountId, delegation Amount)
		pub delegations: Vec<(T::AccountId, T::AccountId, BalanceOf<T>)>,
		pub inflation_config: InflationInfo<BalanceOf<T>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				candidates: vec![],
				delegations: vec![],
				..Default::default()
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<InflationConfig<T>>::put(self.inflation_config.clone());
			let mut candidate_count = 0u32;
			// Initialize the candidates
			for &(ref candidate, balance) in &self.candidates {
				assert!(
					T::Currency::free_balance(candidate) >= balance,
					"Account does not have enough balance to bond as a candidate."
				);
				candidate_count += 1u32;
				if let Err(error) = <Pallet<T>>::join_candidates(
					T::Origin::from(Some(candidate.clone()).into()),
					balance,
					candidate_count,
				) {
					log::warn!("Join candidates failed in genesis with error {:?}", error);
				} else {
					candidate_count += 1u32;
				}
			}
			let mut col_delegator_count: BTreeMap<T::AccountId, u32> = BTreeMap::new();
			let mut del_delegation_count: BTreeMap<T::AccountId, u32> = BTreeMap::new();
			// Initialize the delegations
			for &(ref delegator, ref target, balance) in &self.delegations {
				assert!(
					T::Currency::free_balance(delegator) >= balance,
					"Account does not have enough balance to place delegation."
				);
				let cd_count = if let Some(x) = col_delegator_count.get(target) {
					*x
				} else {
					0u32
				};
				let dd_count = if let Some(x) = del_delegation_count.get(delegator) {
					*x
				} else {
					0u32
				};
				if let Err(error) = <Pallet<T>>::delegate(
					T::Origin::from(Some(delegator.clone()).into()),
					target.clone(),
					balance,
					cd_count,
					dd_count,
				) {
					log::warn!("Delegate failed in genesis with error {:?}", error);
				} else {
					if let Some(x) = col_delegator_count.get_mut(target) {
						*x += 1u32;
					} else {
						col_delegator_count.insert(target.clone(), 1u32);
					};
					if let Some(x) = del_delegation_count.get_mut(delegator) {
						*x += 1u32;
					} else {
						del_delegation_count.insert(delegator.clone(), 1u32);
					};
				}
			}
			// Set collator commission to default config
			<CollatorCommission<T>>::put(T::DefaultCollatorCommission::get());
			// Set parachain bond config to default config
			<ParachainBondInfo<T>>::put(ParachainBondConfig {
				// must be set soon; if not => due inflation will be sent to collators/delegators
				account: T::AccountId::default(),
				percent: T::DefaultParachainBondReservePercent::get(),
			});
			// Set total selected candidates to minimum config
			<TotalSelected<T>>::put(T::MinSelectedCandidates::get());
			// Choose top TotalSelected collator candidates
			let (v_count, _, total_staked) = <Pallet<T>>::select_top_candidates(1u32);
			// Start Round 1 at Block 0
			let round: RoundInfo<T::BlockNumber> =
				RoundInfo::new(1u32, 0u32.into(), T::DefaultBlocksPerRound::get());
			<Round<T>>::put(round);
			// Snapshot total stake
			<Staked<T>>::insert(1u32, <Total<T>>::get());
			<Pallet<T>>::deposit_event(Event::NewRound(
				T::BlockNumber::zero(),
				1u32,
				v_count,
				total_staked,
			));
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)] // TODO: update weight
		/// Hotfix patch to correct and update CandidatePool value for candidates that have
		/// called candidate_bond_more when it did not update the CandidatePool value
		pub fn hotfix_update_candidate_pool_value(
			origin: OriginFor<T>,
			candidates: Vec<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			for candidate in candidates {
				if let Some(state) = <CandidateState<T>>::get(&candidate) {
					Self::update_active(candidate, state.total_counted);
				} // else candidate is not a candidate so no update needed
			}
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::set_staking_expectations())]
		/// Set the expectations for total staked. These expectations determine the issuance for
		/// the round according to logic in `fn compute_issuance`
		pub fn set_staking_expectations(
			origin: OriginFor<T>,
			expectations: Range<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			T::MonetaryGovernanceOrigin::ensure_origin(origin)?;
			ensure!(expectations.is_valid(), Error::<T>::InvalidSchedule);
			let mut config = <InflationConfig<T>>::get();
			ensure!(
				config.expect != expectations,
				Error::<T>::NoWritingSameValue
			);
			config.set_expectations(expectations);
			Self::deposit_event(Event::StakeExpectationsSet(
				config.expect.min,
				config.expect.ideal,
				config.expect.max,
			));
			<InflationConfig<T>>::put(config);
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::set_inflation())]
		/// Set the annual inflation rate to derive per-round inflation
		pub fn set_inflation(
			origin: OriginFor<T>,
			schedule: Range<Perbill>,
		) -> DispatchResultWithPostInfo {
			T::MonetaryGovernanceOrigin::ensure_origin(origin)?;
			ensure!(schedule.is_valid(), Error::<T>::InvalidSchedule);
			let mut config = <InflationConfig<T>>::get();
			ensure!(config.annual != schedule, Error::<T>::NoWritingSameValue);
			config.annual = schedule;
			config.set_round_from_annual::<T>(schedule);
			Self::deposit_event(Event::InflationSet(
				config.annual.min,
				config.annual.ideal,
				config.annual.max,
				config.round.min,
				config.round.ideal,
				config.round.max,
			));
			<InflationConfig<T>>::put(config);
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::set_parachain_bond_account())]
		/// Set the account that will hold funds set aside for parachain bond
		pub fn set_parachain_bond_account(
			origin: OriginFor<T>,
			new: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::MonetaryGovernanceOrigin::ensure_origin(origin)?;
			let ParachainBondConfig {
				account: old,
				percent,
			} = <ParachainBondInfo<T>>::get();
			ensure!(old != new, Error::<T>::NoWritingSameValue);
			<ParachainBondInfo<T>>::put(ParachainBondConfig {
				account: new.clone(),
				percent,
			});
			Self::deposit_event(Event::ParachainBondAccountSet(old, new));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::set_parachain_bond_reserve_percent())]
		/// Set the percent of inflation set aside for parachain bond
		pub fn set_parachain_bond_reserve_percent(
			origin: OriginFor<T>,
			new: Percent,
		) -> DispatchResultWithPostInfo {
			T::MonetaryGovernanceOrigin::ensure_origin(origin)?;
			let ParachainBondConfig {
				account,
				percent: old,
			} = <ParachainBondInfo<T>>::get();
			ensure!(old != new, Error::<T>::NoWritingSameValue);
			<ParachainBondInfo<T>>::put(ParachainBondConfig {
				account,
				percent: new,
			});
			Self::deposit_event(Event::ParachainBondReservePercentSet(old, new));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::set_total_selected())]
		/// Set the total number of collator candidates selected per round
		/// - changes are not applied until the start of the next round
		pub fn set_total_selected(origin: OriginFor<T>, new: u32) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(
				new >= T::MinSelectedCandidates::get(),
				Error::<T>::CannotSetBelowMin
			);
			let old = <TotalSelected<T>>::get();
			ensure!(old != new, Error::<T>::NoWritingSameValue);
			ensure!(
				new <= <Round<T>>::get().length,
				Error::<T>::RoundLengthMustBeAtLeastTotalSelectedCollators,
			);
			<TotalSelected<T>>::put(new);
			Self::deposit_event(Event::TotalSelectedSet(old, new));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::set_collator_commission())]
		/// Set the commission for all collators
		pub fn set_collator_commission(
			origin: OriginFor<T>,
			new: Perbill,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			let old = <CollatorCommission<T>>::get();
			ensure!(old != new, Error::<T>::NoWritingSameValue);
			<CollatorCommission<T>>::put(new);
			Self::deposit_event(Event::CollatorCommissionSet(old, new));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::set_blocks_per_round())]
		/// Set blocks per round
		/// - if called with `new` less than length of current round, will transition immediately
		/// in the next block
		/// - also updates per-round inflation config
		pub fn set_blocks_per_round(origin: OriginFor<T>, new: u32) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(
				new >= T::MinBlocksPerRound::get(),
				Error::<T>::CannotSetBelowMin
			);
			let mut round = <Round<T>>::get();
			let (now, first, old) = (round.current, round.first, round.length);
			ensure!(old != new, Error::<T>::NoWritingSameValue);
			ensure!(
				new >= <TotalSelected<T>>::get(),
				Error::<T>::RoundLengthMustBeAtLeastTotalSelectedCollators,
			);
			round.length = new;
			// update per-round inflation given new rounds per year
			let mut inflation_config = <InflationConfig<T>>::get();
			inflation_config.reset_round(new);
			<Round<T>>::put(round);
			Self::deposit_event(Event::BlocksPerRoundSet(
				now,
				first,
				old,
				new,
				inflation_config.round.min,
				inflation_config.round.ideal,
				inflation_config.round.max,
			));
			<InflationConfig<T>>::put(inflation_config);
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::join_candidates(*candidate_count))]
		/// Join the set of collator candidates
		pub fn join_candidates(
			origin: OriginFor<T>,
			bond: BalanceOf<T>,
			candidate_count: u32,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			ensure!(!Self::is_candidate(&acc), Error::<T>::CandidateExists);
			ensure!(!Self::is_delegator(&acc), Error::<T>::DelegatorExists);
			ensure!(
				bond >= T::MinCandidateStk::get(),
				Error::<T>::CandidateBondBelowMin
			);
			let mut candidates = <CandidatePool<T>>::get();
			let old_count = candidates.0.len() as u32;
			ensure!(
				candidate_count >= old_count,
				Error::<T>::TooLowCandidateCountWeightHintJoinCandidates
			);
			ensure!(
				candidates.insert(Bond {
					owner: acc.clone(),
					amount: bond
				}),
				Error::<T>::CandidateExists
			);
			T::Currency::reserve(&acc, bond)?;
			let candidate = CollatorCandidate::new(acc.clone(), bond);
			<CandidateState<T>>::insert(&acc, candidate);
			<CandidatePool<T>>::put(candidates);
			let new_total = <Total<T>>::get().saturating_add(bond);
			<Total<T>>::put(new_total);
			Self::deposit_event(Event::JoinedCollatorCandidates(acc, bond, new_total));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::schedule_leave_candidates(*candidate_count))]
		/// Request to leave the set of candidates. If successful, the account is immediately
		/// removed from the candidate pool to prevent selection as a collator.
		pub fn schedule_leave_candidates(
			origin: OriginFor<T>,
			candidate_count: u32,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let (now, when) = state.leave::<T>()?;
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidate_count >= candidates.0.len() as u32,
				Error::<T>::TooLowCandidateCountToLeaveCandidates
			);
			if candidates.remove(&Bond::from_owner(collator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateScheduledExit(now, collator, when));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::execute_leave_candidates())]
		/// Execute leave candidates request
		pub fn execute_leave_candidates(
			origin: OriginFor<T>,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			let state = <CandidateState<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			state.can_leave::<T>()?;
			let return_stake = |bond: Bond<T::AccountId, BalanceOf<T>>| {
				T::Currency::unreserve(&bond.owner, bond.amount);
				// remove delegation from delegator state
				let mut delegator = DelegatorState::<T>::get(&bond.owner).expect(
					"Collator state and delegator state are consistent. 
						Collator state has a record of this delegation. Therefore, 
						Delegator state also has a record. qed.",
				);
				if let Some(remaining) = delegator.rm_delegation(candidate.clone()) {
					if remaining.is_zero() {
						<DelegatorState<T>>::remove(&bond.owner);
					} else {
						<DelegatorState<T>>::insert(&bond.owner, delegator);
					}
				}
			};
			// return all top delegations
			for bond in state.top_delegations {
				return_stake(bond);
			}
			// return all bottom delegations
			for bond in state.bottom_delegations {
				return_stake(bond);
			}
			// return stake to collator
			T::Currency::unreserve(&state.id, state.bond);
			<CandidateState<T>>::remove(&candidate);
			let new_total_staked = <Total<T>>::get().saturating_sub(state.total_backing);
			<Total<T>>::put(new_total_staked);
			Self::deposit_event(Event::CandidateLeft(
				candidate,
				state.total_backing,
				new_total_staked,
			));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::cancel_leave_candidates(*candidate_count))]
		/// Cancel open request to leave candidates
		/// - only callable by collator account
		/// - result upon successful call is the candidate is active in the candidate pool
		pub fn cancel_leave_candidates(
			origin: OriginFor<T>,
			candidate_count: u32,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(state.is_leaving(), Error::<T>::CandidateNotLeaving);
			state.go_online();
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidates.0.len() as u32 <= candidate_count,
				Error::<T>::TooLowCandidateCountWeightHintCancelLeaveCandidates
			);
			ensure!(
				candidates.insert(Bond {
					owner: collator.clone(),
					amount: state.total_counted
				}),
				Error::<T>::AlreadyActive
			);
			<CandidatePool<T>>::put(candidates);
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CancelledCandidateExit(collator));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::go_offline())]
		/// Temporarily leave the set of collator candidates without unbonding
		pub fn go_offline(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(state.is_active(), Error::<T>::AlreadyOffline);
			state.go_offline();
			let mut candidates = <CandidatePool<T>>::get();
			if candidates.remove(&Bond::from_owner(collator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateWentOffline(
				<Round<T>>::get().current,
				collator,
			));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::go_online())]
		/// Rejoin the set of collator candidates if previously had called `go_offline`
		pub fn go_online(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_active(), Error::<T>::AlreadyActive);
			ensure!(!state.is_leaving(), Error::<T>::CannotGoOnlineIfLeaving);
			state.go_online();
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidates.insert(Bond {
					owner: collator.clone(),
					amount: state.total_counted
				}),
				Error::<T>::AlreadyActive
			);
			<CandidatePool<T>>::put(candidates);
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateBackOnline(
				<Round<T>>::get().current,
				collator,
			));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::candidate_bond_more())]
		/// Increase collator candidate self bond by `more`
		pub fn candidate_bond_more(
			origin: OriginFor<T>,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			state.bond_more::<T>(more)?;
			if state.is_active() {
				Self::update_active(collator.clone(), state.total_counted);
			}
			<CandidateState<T>>::insert(&collator, state);
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::schedule_candidate_bond_less())]
		/// Request by collator candidate to decrease self bond by `less`
		pub fn schedule_candidate_bond_less(
			origin: OriginFor<T>,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let when = state.schedule_bond_less::<T>(less)?;
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateBondLessRequested(collator, less, when));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::execute_candidate_bond_less())]
		/// Execute pending request to adjust the collator candidate self bond
		pub fn execute_candidate_bond_less(
			origin: OriginFor<T>,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?; // we may want to reward this if caller != candidate
			let mut state = <CandidateState<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			let event = state.execute_pending_request::<T>()?;
			<CandidateState<T>>::insert(&candidate, state);
			Self::deposit_event(event);
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::cancel_candidate_bond_less())]
		/// Cancel pending request to adjust the collator candidate self bond
		pub fn cancel_candidate_bond_less(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let event = state.cancel_pending_request::<T>()?;
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(event);
			Ok(().into())
		}
		#[pallet::weight(
			<T as Config>::WeightInfo::delegate(
				*candidate_delegation_count,
				*delegation_count
			)
		)]
		/// If caller is not a delegator and not a collator, then join the set of delegators
		/// If caller is a delegator, then makes delegation to change their delegation state
		pub fn delegate(
			origin: OriginFor<T>,
			collator: T::AccountId,
			amount: BalanceOf<T>,
			candidate_delegation_count: u32,
			delegation_count: u32,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			let delegator_state = if let Some(mut state) = <DelegatorState<T>>::get(&acc) {
				ensure!(state.is_active(), Error::<T>::CannotDelegateIfLeaving);
				// delegation after first
				ensure!(
					amount >= T::MinDelegation::get(),
					Error::<T>::DelegationBelowMin
				);
				ensure!(
					delegation_count >= state.delegations.0.len() as u32,
					Error::<T>::TooLowDelegationCountToDelegate
				);
				ensure!(
					(state.delegations.0.len() as u32) < T::MaxDelegationsPerDelegator::get(),
					Error::<T>::ExceedMaxDelegationsPerDelegator
				);
				ensure!(
					state.add_delegation(Bond {
						owner: collator.clone(),
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
				ensure!(!Self::is_candidate(&acc), Error::<T>::CandidateExists);
				Delegator::new(acc.clone(), collator.clone(), amount)
			};
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(
				candidate_delegation_count >= state.delegators.0.len() as u32,
				Error::<T>::TooLowCandidateDelegationCountToDelegate
			);
			let delegator_position = state.add_delegation::<T>(acc.clone(), amount)?;
			T::Currency::reserve(&acc, amount)?;
			if let DelegatorAdded::AddedToTop { new_total } = delegator_position {
				if state.is_active() {
					// collator in candidate pool
					Self::update_active(collator.clone(), new_total);
				}
			}
			let new_total_locked = <Total<T>>::get() + amount;
			<Total<T>>::put(new_total_locked);
			<CandidateState<T>>::insert(&collator, state);
			<DelegatorState<T>>::insert(&acc, delegator_state);
			Self::deposit_event(Event::Delegation(acc, amount, collator, delegator_position));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::schedule_leave_delegators())]
		/// Request to leave the set of delegators. If successful, the caller is scheduled
		/// to be allowed to exit. Success forbids future delegator actions until the request is
		/// invoked or cancelled.
		pub fn schedule_leave_delegators(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			let mut state = <DelegatorState<T>>::get(&acc).ok_or(Error::<T>::DelegatorDNE)?;
			ensure!(!state.is_leaving(), Error::<T>::DelegatorAlreadyLeaving);
			let (now, when) = state.schedule_leave::<T>();
			<DelegatorState<T>>::insert(&acc, state);
			Self::deposit_event(Event::DelegatorExitScheduled(now, acc, when));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::execute_leave_delegators(*delegation_count))]
		/// Execute the right to exit the set of delegators and revoke all ongoing delegations.
		pub fn execute_leave_delegators(
			origin: OriginFor<T>,
			delegator: T::AccountId,
			delegation_count: u32,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			let state = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			state.can_execute_leave::<T>(delegation_count)?;
			for bond in state.delegations.0 {
				if let Err(error) =
					Self::delegator_leaves_collator(delegator.clone(), bond.owner.clone())
				{
					log::warn!(
						"STORAGE CORRUPTED \nDelegator leaving collator failed with error: {:?}",
						error
					);
				}
			}
			<DelegatorState<T>>::remove(&delegator);
			Self::deposit_event(Event::DelegatorLeft(delegator, state.total));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::cancel_leave_delegators())]
		/// Cancel a pending request to exit the set of delegators. Success clears the pending exit
		/// request (thereby resetting the delay upon another `leave_delegators` call).
		pub fn cancel_leave_delegators(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			// ensure delegator state exists
			let mut state = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			// ensure state is leaving
			ensure!(state.is_leaving(), Error::<T>::DelegatorDNE);
			// cancel exit request
			state.cancel_leave();
			<DelegatorState<T>>::insert(&delegator, state);
			Self::deposit_event(Event::DelegatorExitCancelled(delegator));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::schedule_revoke_delegation())]
		/// Request to revoke an existing delegation. If successful, the delegation is scheduled
		/// to be allowed to be revoked via the `execute_delegation_request` extrinsic.
		pub fn schedule_revoke_delegation(
			origin: OriginFor<T>,
			collator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			let mut state = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			let (now, when) = state.schedule_revoke::<T>(collator.clone())?;
			<DelegatorState<T>>::insert(&delegator, state);
			Self::deposit_event(Event::DelegationRevocationScheduled(
				now, delegator, collator, when,
			));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::delegator_bond_more())]
		/// Bond more for delegators wrt a specific collator candidate.
		pub fn delegator_bond_more(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			let mut state = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			state.increase_delegation::<T>(candidate.clone(), more)?;
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::schedule_delegator_bond_less())]
		/// Request bond less for delegators wrt a specific collator candidate.
		pub fn schedule_delegator_bond_less(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			let mut state = <DelegatorState<T>>::get(&caller).ok_or(Error::<T>::DelegatorDNE)?;
			let when = state.schedule_decrease_delegation::<T>(candidate.clone(), less)?;
			<DelegatorState<T>>::insert(&caller, state);
			Self::deposit_event(Event::DelegationDecreaseScheduled(
				caller, candidate, less, when,
			));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::execute_delegator_bond_less())]
		/// Execute pending request to change an existing delegation
		pub fn execute_delegation_request(
			origin: OriginFor<T>,
			delegator: T::AccountId,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?; // we may want to reward caller if caller != delegator
			let mut state = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			state.execute_pending_request::<T>(candidate)?;
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::cancel_delegator_bond_less())]
		/// Cancel request to change an existing delegation.
		pub fn cancel_delegation_request(
			origin: OriginFor<T>,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			let mut state = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			let request = state.cancel_pending_request::<T>(candidate)?;
			<DelegatorState<T>>::insert(&delegator, state);
			Self::deposit_event(Event::CancelledDelegationRequest(delegator, request));
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn is_delegator(acc: &T::AccountId) -> bool {
			<DelegatorState<T>>::get(acc).is_some()
		}
		pub fn is_candidate(acc: &T::AccountId) -> bool {
			<CandidateState<T>>::get(acc).is_some()
		}
		pub fn is_selected_candidate(acc: &T::AccountId) -> bool {
			<SelectedCandidates<T>>::get().binary_search(acc).is_ok()
		}
		/// Caller must ensure candidate is active before calling
		pub(crate) fn update_active(candidate: T::AccountId, total: BalanceOf<T>) {
			let mut candidates = <CandidatePool<T>>::get();
			candidates.remove(&Bond::from_owner(candidate.clone()));
			candidates.insert(Bond {
				owner: candidate,
				amount: total,
			});
			<CandidatePool<T>>::put(candidates);
		}
		/// Compute round issuance based on total staked for the given round
		fn compute_issuance(staked: BalanceOf<T>) -> BalanceOf<T> {
			let config = <InflationConfig<T>>::get();
			let round_issuance = crate::inflation::round_issuance_range::<T>(config.round);
			// TODO: consider interpolation instead of bounded range
			if staked < config.expect.min {
				round_issuance.min
			} else if staked > config.expect.max {
				round_issuance.max
			} else {
				round_issuance.ideal
			}
		}
		fn delegator_leaves_collator(
			delegator: T::AccountId,
			collator: T::AccountId,
		) -> DispatchResult {
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let (total_changed, delegator_stake) = state.rm_delegator::<T>(delegator.clone())?;
			T::Currency::unreserve(&delegator, delegator_stake);
			if state.is_active() && total_changed {
				Self::update_active(collator.clone(), state.total_counted);
			}
			let new_total_locked = <Total<T>>::get() - delegator_stake;
			<Total<T>>::put(new_total_locked);
			let new_total = state.total_counted;
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::DelegatorLeftCandidate(
				delegator,
				collator,
				delegator_stake,
				new_total,
			));
			Ok(())
		}
		fn prepare_staking_payouts(now: RoundIndex) {
			// payout is now - delay rounds ago => now - delay > 0 else return early
			let delay = T::RewardPaymentDelay::get();
			if now <= delay {
				return;
			}
			let round_to_payout = now - delay;
			let total_points = <Points<T>>::get(round_to_payout);
			if total_points.is_zero() {
				return;
			}
			let total_staked = <Staked<T>>::take(round_to_payout);
			let total_issuance = Self::compute_issuance(total_staked);
			let mut left_issuance = total_issuance;
			// reserve portion of issuance for parachain bond account
			let bond_config = <ParachainBondInfo<T>>::get();
			let parachain_bond_reserve = bond_config.percent * total_issuance;
			if let Ok(imb) =
				T::Currency::deposit_into_existing(&bond_config.account, parachain_bond_reserve)
			{
				// update round issuance iff transfer succeeds
				left_issuance -= imb.peek();
				Self::deposit_event(Event::ReservedForParachainBond(
					bond_config.account,
					imb.peek(),
				));
			}

			let payout = DelayedPayout {
				round_issuance: total_issuance,
				total_staking_reward: left_issuance,
				collator_commission: <CollatorCommission<T>>::get(),
			};

			<DelayedPayouts<T>>::insert(round_to_payout, payout);
		}

		/// Wrapper around pay_one_collator_reward which handles the following logic:
		/// * whether or not a payout needs to be made
		/// * cleaning up when payouts are done
		/// * returns the weight consumed by pay_one_collator_reward if applicable
		fn handle_delayed_payouts(now: RoundIndex) -> Weight {
			let delay = T::RewardPaymentDelay::get();

			// don't underflow uint
			if now < delay {
				return 0u64.into();
			}

			let paid_for_round = now - delay;

			if let Some(payout_info) = <DelayedPayouts<T>>::get(paid_for_round) {
				let result = Self::pay_one_collator_reward(paid_for_round, payout_info);
				if result.0.is_none() {
					// result.0 indicates whether or not a payout was made
					// clean up storage items that we no longer need
					<DelayedPayouts<T>>::remove(paid_for_round);
					<Points<T>>::remove(paid_for_round);
				}
				result.1 // weight consumed by pay_one_collator_reward
			} else {
				0u64.into()
			}
		}

		/// Payout a single collator from the given round.
		///
		/// Returns an optional tuple of (Collator's AccountId, total paid)
		/// or None if there were no more payouts to be made for the round.
		pub(crate) fn pay_one_collator_reward(
			paid_for_round: RoundIndex,
			payout_info: DelayedPayout<BalanceOf<T>>,
		) -> (Option<(T::AccountId, BalanceOf<T>)>, Weight) {
			// TODO: it would probably be optimal to roll Points into the DelayedPayouts storage item
			// so that we do fewer reads each block
			let total_points = <Points<T>>::get(paid_for_round);
			if total_points.is_zero() {
				// TODO: this case is obnoxious... it's a value query, so it could mean one of two
				// different logic errors:
				// 1. we removed it before we should have
				// 2. we called pay_one_collator_reward when we were actually done with deferred
				//    payouts
				log::warn!("pay_one_collator_reward called with no <Points<T>> for the round!");
				return (None, 0u64.into());
			}

			let mint = |amt: BalanceOf<T>, to: T::AccountId| {
				if let Ok(amount_transferred) = T::Currency::deposit_into_existing(&to, amt) {
					Self::deposit_event(Event::Rewarded(to.clone(), amount_transferred.peek()));
				}
			};

			let collator_fee = payout_info.collator_commission;
			let collator_issuance = collator_fee * payout_info.round_issuance;

			if let Some((collator, pts)) =
				<AwardedPts<T>>::iter_prefix(paid_for_round).drain().next()
			{
				let pct_due = Perbill::from_rational(pts, total_points);
				let total_paid = pct_due * payout_info.total_staking_reward;
				let mut amt_due = total_paid;
				// Take the snapshot of block author and delegations
				let state = <AtStake<T>>::take(paid_for_round, &collator);
				let num_delegators = state.delegations.len();
				if state.delegations.is_empty() {
					// solo collator with no delegators
					mint(amt_due, collator.clone());
				} else {
					// pay collator first; commission + due_portion
					let collator_pct = Perbill::from_rational(state.bond, state.total);
					let commission = pct_due * collator_issuance;
					amt_due -= commission;
					let collator_reward = (collator_pct * amt_due) + commission;
					mint(collator_reward, collator.clone());
					// pay delegators due portion
					for Bond { owner, amount } in state.delegations {
						let percent = Perbill::from_rational(amount, state.total);
						let due = percent * amt_due;
						mint(due, owner.clone());
					}
				}

				return (
					Some((collator, total_paid)),
					T::WeightInfo::pay_one_collator_reward(num_delegators as u32),
				);
			} else {
				// Note that we don't clean up storage here; it is cleaned up in handle_delayed_payouts()
				return (None, 0u64.into());
			}
		}

		/// Compute the top `TotalSelected` candidates in the CandidatePool and return
		/// a vec of their AccountIds (in the order of selection)
		pub fn compute_top_candidates() -> Vec<T::AccountId> {
			let mut candidates = <CandidatePool<T>>::get().0;
			// order candidates by stake (least to greatest so requires `rev()`)
			candidates.sort_unstable_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap());
			let top_n = <TotalSelected<T>>::get() as usize;
			// choose the top TotalSelected qualified candidates, ordered by stake
			let mut collators = candidates
				.into_iter()
				.rev()
				.take(top_n)
				.filter(|x| x.amount >= T::MinCollatorStk::get())
				.map(|x| x.owner)
				.collect::<Vec<T::AccountId>>();
			collators.sort();
			collators
		}
		/// Best as in most cumulatively supported in terms of stake
		/// Returns [collator_count, delegation_count, total staked]
		fn select_top_candidates(now: RoundIndex) -> (u32, u32, BalanceOf<T>) {
			let (mut collator_count, mut delegation_count, mut total) =
				(0u32, 0u32, BalanceOf::<T>::zero());
			// choose the top TotalSelected qualified candidates, ordered by stake
			let collators = Self::compute_top_candidates();
			// snapshot exposure for round for weighting reward distribution
			for account in collators.iter() {
				let state = <CandidateState<T>>::get(&account)
					.expect("all members of CandidateQ must be candidates");
				collator_count += 1u32;
				delegation_count += state.delegators.0.len() as u32;
				let amount = state.total_counted;
				total += amount;
				let exposure: CollatorSnapshot<T::AccountId, BalanceOf<T>> = state.into();
				<AtStake<T>>::insert(now, account, exposure);
				Self::deposit_event(Event::CollatorChosen(now, account.clone(), amount));
			}
			// insert canonical collator set
			<SelectedCandidates<T>>::put(collators);
			(collator_count, delegation_count, total)
		}
	}

	/// Add reward points to block authors:
	/// * 20 points to the block producer for producing a block in the chain
	impl<T: Config> nimbus_primitives::EventHandler<T::AccountId> for Pallet<T> {
		fn note_author(author: T::AccountId) {
			let now = <Round<T>>::get().current;
			let score_plus_20 = <AwardedPts<T>>::get(now, &author) + 20;
			<AwardedPts<T>>::insert(now, author, score_plus_20);
			<Points<T>>::mutate(now, |x| *x += 20);
		}
	}

	impl<T: Config> nimbus_primitives::CanAuthor<T::AccountId> for Pallet<T> {
		fn can_author(account: &T::AccountId, _slot: &u32) -> bool {
			Self::is_selected_candidate(account)
		}
	}

	impl<T: Config> Get<Vec<T::AccountId>> for Pallet<T> {
		fn get() -> Vec<T::AccountId> {
			Self::selected_candidates()
		}
	}
}
