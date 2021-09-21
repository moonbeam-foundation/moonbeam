// Copyright 2019-2021 PureStake Inc.
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
//! uses direct delegation. Nominators choose exactly who they nominate and with what stake.
//! This is different from `frame/pallet-staking` where you approval vote and then run Phragmen.
//!
//! ### Rules
//! There is a new round every `<Round<T>>::get().length` blocks.
//!
//! At the start of every round,
//! * issuance is distributed to collators (and their nominators) for block authoring
//! `T::RewardPaymentDelay` rounds ago
//! * queued collator and nominator exits are executed
//! * a new set of collators is chosen from the candidates
//!
//! To join the set of candidates, call `join_candidates` with `bond >= MinCollatorCandidateStk`.
//!
//! To leave the set of candidates, call `leave_candidates`. If the call succeeds,
//! the collator is removed from the pool of candidates so they cannot be selected for future
//! collator sets, but they are not unstaked until `T::LeaveCandidatesDelay` rounds later.
//! The exit request is stored in the `ExitQueue` and processed `T::LeaveCandidatesDelay` rounds
//! later to unstake the collator and all of its delegations.
//!
//! To join the set of nominators, call `nominate` and pass in an account that is
//! already a collator candidate and `bond >= MinNominatorStk`. Each nominator can nominate up to
//! `T::MaxCollatorsPerNominator` collator candidates by calling `nominate`.
//!
//! To revoke a nomination, call `revoke_nomination` with the collator candidate's account.
//! To leave the set of nominators and revoke all delegations, call `leave_nominators`.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;
mod inflation;
#[cfg(test)]
mod mock;
mod set;
#[cfg(test)]
mod tests;

pub mod weights;
use weights::WeightInfo;

use frame_support::pallet;
pub use inflation::{InflationInfo, Range};

pub use pallet::*;

#[pallet]
pub mod pallet {
	use crate::{set::OrderedSet, InflationInfo, Range, WeightInfo};
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, Get, Imbalance, ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::{Decode, Encode};
	use sp_runtime::{
		traits::{AtLeast32BitUnsigned, Saturating, Zero},
		Perbill, Percent, RuntimeDebug,
	};
	use sp_std::{cmp::Ordering, collections::btree_map::BTreeMap, prelude::*};

	/// Pallet for parachain staking
	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[derive(Default, Clone, Encode, Decode, RuntimeDebug)]
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

	#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
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

	#[derive(Default, Encode, Decode, RuntimeDebug)]
	/// Snapshot of collator state at the start of the round for which they are selected
	pub struct CollatorSnapshot<AccountId, Balance> {
		pub bond: Balance,
		pub delegations: Vec<Bond<AccountId, Balance>>,
		pub total: Balance,
	}

	#[derive(Encode, Decode, RuntimeDebug)]
	/// DEPRECATED
	/// Collator state with commission fee, bonded stake, and delegations
	pub struct Collator2<AccountId, Balance> {
		/// The account of this collator
		pub id: AccountId,
		/// This collator's self stake.
		pub bond: Balance,
		/// Set of all nominator AccountIds (to prevent >1 nomination per AccountId)
		pub nominators: OrderedSet<AccountId>,
		/// Top T::MaxNominatorsPerCollator::get() nominators, ordered greatest to least
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

	#[derive(PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug)]
	/// Actions allowed by an active collator candidate
	pub enum CandidateBondAction {
		Increase,
		Decrease,
	}

	#[derive(PartialEq, Clone, Copy, Encode, Decode, RuntimeDebug)]
	/// Request scheduled to change the collator candidate self-bond
	pub struct CandidateBondChange<Balance> {
		pub amount: Balance,
		pub change: CandidateBondAction,
		pub when: RoundIndex,
	}

	impl<B> CandidateBondChange<B> {
		pub fn new(
			change: CandidateBondAction,
			amount: B,
			when: RoundIndex,
		) -> CandidateBondChange<B> {
			CandidateBondChange {
				amount,
				change,
				when,
			}
		}
	}

	#[derive(Encode, Decode, RuntimeDebug)]
	/// Collator candidate state with self bond + delegations
	pub struct CollatorCandidate<AccountId, Balance> {
		/// The account of this collator
		pub id: AccountId,
		/// This collator's self stake.
		pub bond: Balance,
		/// Set of all delegator AccountIds (to prevent >1 delegation per AccountId)
		pub delegators: OrderedSet<AccountId>,
		/// Top T::MaxNominatorsPerCollator::get() delegations, ordered greatest to least
		pub top_delegations: Vec<Bond<AccountId, Balance>>,
		/// Bottom delegations (unbounded), ordered least to greatest
		pub bottom_delegations: Vec<Bond<AccountId, Balance>>,
		/// Sum of top delegations + self.bond
		pub total_counted: Balance,
		/// Sum of all delegations + self.bond = (total_counted + uncounted)
		pub total_backing: Balance,
		/// Maximum 1 pending request to adjust candidate self bond at any given time
		pub request: Option<CandidateBondChange<Balance>>,
		/// Current status of the collator
		pub state: CollatorStatus,
	} // TODO: impl From<Collator2 for CollatorCandidate

	/// Convey relevant information describing if a delegator was added to the top or bottom
	/// Delegations added to the top yield a new total
	#[derive(Clone, Copy, PartialEq, Encode, Decode, RuntimeDebug)]
	pub enum DelegatorAdded<B> {
		AddedToTop { new_total: B },
		AddedToBottom,
	}

	impl<
			A: Ord + Clone,
			B: AtLeast32BitUnsigned + Ord + Copy + sp_std::ops::AddAssign + sp_std::ops::SubAssign,
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
		pub fn can_leave<T: Config>(&self) -> Result<bool, DispatchError> {
			if let CollatorStatus::Leaving(when) = self.state {
				Ok(<Round<T>>::get().current >= when)
			} else {
				Err(Error::<T>::CandidateNotLeaving.into())
			}
		}
		/// Schedule executable increase of collator candidate self bond
		/// Returns the round at which the collator can execute the pending request
		pub fn schedule_bond_more<T: Config>(
			&mut self,
			more: B,
		) -> Result<RoundIndex, DispatchError> {
			// ensure no pending request
			ensure!(
				self.request.is_none(),
				Error::<T>::PendingCollatorRequestAlreadyExists
			);
			let when = <Round<T>>::get().current + T::CandidateBondDelay::get();
			self.request = Some(CandidateBondChange::new(
				CandidateBondAction::Increase,
				more,
				when,
			));
			Ok(when)
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
				Error::<T>::PendingCollatorRequestAlreadyExists
			);
			// ensure bond above min after decrease (TODO: change error?)
			ensure!(self.bond > less, Error::<T>::CollatorBondBelowMin);
			ensure!(
				self.bond - less > T::MinCollatorCandidateStk::get().into(),
				Error::<T>::CollatorBondBelowMin
			);
			let when = <Round<T>>::get().current + T::CandidateBondDelay::get();
			self.request = Some(CandidateBondChange::new(
				CandidateBondAction::Decrease,
				less,
				when,
			));
			Ok(when)
		}
		/// Execute pending request to change the collator self bond
		/// Returns the event to be emitted
		pub fn execute_pending_request<T: Config>(&mut self) -> Result<Event<T>, DispatchError>
		where
			BalanceOf<T>: From<B>,
			T::AccountId: From<A>,
		{
			ensure!(!self.is_leaving(), Error::<T>::CannotActBecauseLeaving);
			let request = self.request.ok_or(Error::<T>::PendingCollatorRequestDNE)?;
			ensure!(
				request.when <= <Round<T>>::get().current,
				Error::<T>::PendingCollatorRequestNotDueYet
			);
			let caller: T::AccountId = self.id.clone().into();
			let event = match request.change {
				CandidateBondAction::Increase => {
					T::Currency::reserve(&caller, request.amount.into())?;
					let new_total = <Total<T>>::get().saturating_add(request.amount.into());
					<Total<T>>::put(new_total);
					let before = self.bond;
					self.bond += request.amount;
					self.total_counted += request.amount;
					self.total_backing += request.amount;
					Event::CollatorBondedMore(
						self.id.clone().into(),
						before.into(),
						self.bond.into(),
					)
				}
				CandidateBondAction::Decrease => {
					T::Currency::unreserve(&caller, request.amount.into());
					let new_total_staked = <Total<T>>::get().saturating_sub(request.amount.into());
					<Total<T>>::put(new_total_staked);
					// Arithmetic assumptions are self.bond > less && self.bond - less > CollatorMinBond
					// (assumptions enforced by `schedule_bond_less`; if storage corrupts, must re-verify)
					let before = self.bond;
					self.bond -= request.amount;
					self.total_counted -= request.amount;
					self.total_backing -= request.amount;
					Event::CollatorBondedLess(
						self.id.clone().into(),
						before.into(),
						self.bond.into(),
					)
				}
			};
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
			CandidateBondChange<BalanceOf<T>>: From<CandidateBondChange<B>>,
			T::AccountId: From<A>,
		{
			ensure!(!self.is_leaving(), Error::<T>::CannotActBecauseLeaving);
			let request = self.request.ok_or(Error::<T>::PendingCollatorRequestDNE)?;
			let event = Event::CancelledCollatorBondChange(self.id.clone().into(), request.into());
			self.request = None;
			Ok(event)
		}
		/// Infallible sorted insertion
		/// caller must verify !self.delegators.contains(nominator.owner) before call
		pub fn add_top_nominator(&mut self, nominator: Bond<A, B>) {
			match self
				.top_delegations
				.binary_search_by(|x| nominator.amount.cmp(&x.amount))
			{
				Ok(i) => self.top_delegations.insert(i, nominator),
				Err(i) => self.top_delegations.insert(i, nominator),
			}
		}
		/// Infallible sorted insertion
		/// caller must verify !self.delegators.contains(nominator.owner) before call
		pub fn add_bottom_nominator(&mut self, nominator: Bond<A, B>) {
			match self
				.bottom_delegations
				.binary_search_by(|x| x.amount.cmp(&nominator.amount))
			{
				Ok(i) => self.bottom_delegations.insert(i, nominator),
				Err(i) => self.bottom_delegations.insert(i, nominator),
			}
		}
		/// Sort top nominators from greatest to least
		pub fn sort_top_nominators(&mut self) {
			self.top_delegations
				.sort_unstable_by(|a, b| b.amount.cmp(&a.amount));
		}
		/// Sort bottom nominators from least to greatest
		pub fn sort_bottom_nominators(&mut self) {
			self.bottom_delegations
				.sort_unstable_by(|a, b| a.amount.cmp(&b.amount));
		}
		/// Bond a new account as a nominator, and make a first nomination. If successful,
		/// the return value indicates whether the nomination is top for the candidate.
		pub fn add_nominator<T: Config>(
			&mut self,
			acc: A,
			amount: B,
		) -> Result<DelegatorAdded<B>, DispatchError> {
			ensure!(
				self.delegators.insert(acc.clone()),
				Error::<T>::NominatorExists
			);
			self.total_backing += amount;
			if (self.top_delegations.len() as u32) < T::MaxNominatorsPerCollator::get() {
				self.add_top_nominator(Bond { owner: acc, amount });
				self.total_counted += amount;
				Ok(DelegatorAdded::AddedToTop {
					new_total: self.total_counted,
				})
			} else {
				// >pop requires push to reset in case isn't pushed to bottom
				let last_nomination_in_top = self
					.top_delegations
					.pop()
					.expect("self.top_delegations.len() >= T::Max exists >= 1 element in top");
				if amount > last_nomination_in_top.amount {
					// update total_counted with positive difference
					self.total_counted += amount - last_nomination_in_top.amount;
					// last nomination already popped from top_nominators
					// insert new nominator into top_nominators
					self.add_top_nominator(Bond { owner: acc, amount });
					self.add_bottom_nominator(last_nomination_in_top);
					Ok(DelegatorAdded::AddedToTop {
						new_total: self.total_counted,
					})
				} else {
					// >required push to previously popped last nomination into top_nominators
					self.top_delegations.push(last_nomination_in_top);
					self.add_bottom_nominator(Bond { owner: acc, amount });
					Ok(DelegatorAdded::AddedToBottom)
				}
			}
		}
		/// Return Ok((if_total_counted_changed, nominator's stake))
		pub fn rm_delegator<T: Config>(
			&mut self,
			nominator: A,
		) -> Result<(bool, B), DispatchError> {
			ensure!(
				self.delegators.remove(&nominator),
				Error::<T>::NominatorDNEInNominatorSet
			);
			let mut nominator_stake: Option<B> = None;
			self.top_delegations = self
				.top_delegations
				.clone()
				.into_iter()
				.filter_map(|nom| {
					if nom.owner != nominator {
						Some(nom)
					} else {
						nominator_stake = Some(nom.amount);
						None
					}
				})
				.collect();
			// item removed from the top => highest bottom is popped from bottom and pushed to top
			if let Some(s) = nominator_stake {
				// last element has largest amount as per ordering
				if let Some(last) = self.bottom_delegations.pop() {
					self.total_counted -= s - last.amount;
					self.add_top_nominator(last);
				} else {
					// no item in bottom nominators so no item from bottom to pop and push up
					self.total_counted -= s;
				}
				self.total_backing -= s;
				return Ok((true, s));
			}
			// else (no item removed from the top)
			self.bottom_delegations = self
				.bottom_delegations
				.clone()
				.into_iter()
				.filter_map(|nom| {
					if nom.owner != nominator {
						Some(nom)
					} else {
						nominator_stake = Some(nom.amount);
						None
					}
				})
				.collect();
			// if err, no item with account exists in top || bottom
			let stake = nominator_stake.ok_or(Error::<T>::NominatorDNEinTopNorBottom)?;
			self.total_backing -= stake;
			Ok((false, stake))
		}
		/// Return true if in_top after call
		/// Caller must verify before call that account is a nominator
		pub fn inc_nominator(&mut self, nominator: A, more: B) -> bool {
			let mut in_top = false;
			for x in &mut self.top_delegations {
				if x.owner == nominator {
					x.amount += more;
					self.total_counted += more;
					self.total_backing += more;
					in_top = true;
					break;
				}
			}
			// if nominator was increased in top nominators
			if in_top {
				self.sort_top_nominators();
				return true;
			}
			// else nominator to increase must exist in bottom
			// >pop requires push later on to reset in case it isn't used
			let lowest_top = self
				.top_delegations
				.pop()
				.expect("any bottom nominators => exists T::Max top nominators");
			let mut move_2_top = false;
			for x in &mut self.bottom_delegations {
				if x.owner == nominator {
					x.amount += more;
					self.total_backing += more;
					move_2_top = x.amount > lowest_top.amount;
					break;
				}
			}
			if move_2_top {
				self.sort_bottom_nominators();
				let highest_bottom = self.bottom_delegations.pop().expect("updated => exists");
				self.total_counted += highest_bottom.amount - lowest_top.amount;
				self.add_top_nominator(highest_bottom);
				self.add_bottom_nominator(lowest_top);
				true
			} else {
				// >required push to reset top_nominators from earlier pop
				self.top_delegations.push(lowest_top);
				self.sort_bottom_nominators();
				false
			}
		}
		/// Return true if in_top after call
		pub fn dec_nominator(&mut self, nominator: A, less: B) -> bool {
			let mut in_top = false;
			let mut new_lowest_top: Option<Bond<A, B>> = None;
			for x in &mut self.top_delegations {
				if x.owner == nominator {
					x.amount -= less;
					// if there is at least 1 nominator in bottom nominators, compare it to check
					// if it should be swapped with lowest top nomination and put in top
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
				self.sort_top_nominators();
				if let Some(highest_bottom) = new_lowest_top {
					// pop last in top to swap it with top bottom
					let lowest_top = self
						.top_delegations
						.pop()
						.expect("must have >1 item to update, assign in_top = true");
					self.total_counted -= lowest_top.amount + less;
					self.total_counted += highest_bottom.amount;
					self.total_backing -= less;
					self.add_top_nominator(highest_bottom);
					self.add_bottom_nominator(lowest_top);
					return false;
				} else {
					// no existing bottom nominators so update both counters the same magnitude
					self.total_counted -= less;
					self.total_backing -= less;
					return true;
				}
			}
			for x in &mut self.bottom_delegations {
				if x.owner == nominator {
					x.amount -= less;
					self.total_backing -= less;
					break;
				}
			}
			self.sort_bottom_nominators();
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

	#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug)]
	pub enum DelegatorStatus {
		/// Active with no scheduled exit
		Active,
		/// Schedule exit to revoke all ongoing delegations
		Leaving(RoundIndex),
	}

	#[derive(Clone, Encode, Decode, RuntimeDebug)]
	/// Delegator state
	pub struct Delegator<AccountId, Balance> {
		/// Delegator account
		pub id: AccountId,
		/// All current delegations
		pub delegations: OrderedSet<Bond<AccountId, Balance>>,
		/// Total balance locked for this delegator
		pub total: Balance,
		/// Requests to change delegations, relevant iff active
		pub requests: PendingNominationRequests<AccountId, Balance>,
		/// Status for this delegator
		pub status: DelegatorStatus,
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
				requests: PendingNominationRequests::new(),
				status: DelegatorStatus::Active,
			}
		}
		pub fn is_active(&self) -> bool {
			matches!(self.status, DelegatorStatus::Active)
		}
		pub fn is_leaving(&self) -> bool {
			matches!(self.status, DelegatorStatus::Leaving(_))
		}
		/// Can only leave if the current round is less than or equal to scheduled execution round
		/// - returns None if not in leaving state
		pub fn can_leave<T: Config>(&self) -> Result<bool, DispatchError> {
			if let DelegatorStatus::Leaving(when) = self.status {
				Ok(<Round<T>>::get().current >= when)
			} else {
				Err(Error::<T>::NominatorNotLeaving.into())
			}
		}
		/// Set nominator status to leaving (prevent any changes until exit or cancellation)
		pub fn leave<T: Config>(&mut self) -> (RoundIndex, RoundIndex) {
			let now = <Round<T>>::get().current;
			let when = now + T::LeaveNominatorsDelay::get();
			self.status = DelegatorStatus::Leaving(when);
			(now, when)
		}
		/// Set nominator status to active
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
		// Return Some(remaining balance), must be more than MinNominatorStk
		// Return None if nomination not found
		pub fn rm_nomination(&mut self, collator: AccountId) -> Option<Balance> {
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
		/// Schedule increase delegation
		pub fn schedule_increase_delegation<T: Config>(
			&mut self,
			collator: AccountId,
			more: Balance,
		) -> Result<RoundIndex, DispatchError> {
			ensure!(
				&self.delegations.0.iter().any(|x| x.owner == collator),
				Error::<T>::NominationDNE
			);
			let when = <Round<T>>::get().current + T::NominatorBondDelay::get();
			self.requests.bond_more::<T>(collator, more, when)?;
			Ok(when)
		}
		/// Schedule decrease delegation
		pub fn schedule_decrease_delegation<T: Config>(
			&mut self,
			collator: AccountId,
			less: Balance,
		) -> Result<RoundIndex, DispatchError> {
			ensure!(
				&self.delegations.0.iter().any(|x| x.owner == collator),
				Error::<T>::NominatorDNE
			);
			let when = <Round<T>>::get().current + T::NominatorBondDelay::get();
			self.requests.bond_less::<T>(collator, less, when)?;
			Ok(when)
		}
		/// Schedule revocation for the given collator
		pub fn schedule_revoke<T: Config>(
			&mut self,
			collator: AccountId,
		) -> Result<Option<(Balance, RoundIndex, RoundIndex)>, DispatchError>
		where
			BalanceOf<T>: Into<Balance>,
		{
			// TODO: need a revocations_count variable as well for this exact purpose
			if self.delegations.0.len() == 1usize {
				return Ok(None); // leave set of nominator instead of revoking bc only 1 remaining
			}
			let now = <Round<T>>::get().current;
			let when = now + T::RevokeNominationDelay::get();
			// get nomination amount
			let mut nomination_amt: Option<Balance> = None;
			for Bond { owner, amount } in &self.delegations.0 {
				if owner == &collator {
					nomination_amt = Some(*amount);
					break;
				}
			}
			let amount = nomination_amt.ok_or(Error::<T>::NominationDNE)?;
			// Net Total is total after pending orders are executed
			let net_total = self.total - self.requests.less_total; // + self.requests.more_total
													   // calculate max amount allowed to be revoked for this nominator to not fall below min
													   // if this subtraction underflows, then \exists inconsistency
			let max_revocation_amount = net_total - T::MinNominatorStk::get().into();
			ensure!(
				amount <= max_revocation_amount,
				Error::<T>::NominatorBondBelowMin
			);
			let new_expected_total = self.total - amount;
			// add revocation to pending requests
			self.requests.revoke::<T>(collator, amount, when)?;
			Ok(Some((new_expected_total, now, when)))
		}
		/// Execute pending nomination change request
		pub fn execute_pending_request<T: Config>(
			&mut self,
			candidate: AccountId,
		) -> Result<Event<T>, DispatchError>
		where
			BalanceOf<T>: From<Balance>,
			T::AccountId: From<AccountId>,
		{
			ensure!(self.is_active(), Error::<T>::CannotActBecauseLeaving);
			let now = <Round<T>>::get().current;
			let NominationRequest {
				amount,
				action,
				when,
				..
			} = self
				.requests
				.requests
				.remove(&candidate)
				.ok_or(Error::<T>::PendingNominationRequestDNE)?
				.clone();
			ensure!(when <= now, Error::<T>::PendingNominationRequestNotDueYet);
			let (balance_amt, candidate_id, nominator_id): (
				BalanceOf<T>,
				T::AccountId,
				T::AccountId,
			) = (
				amount.into(),
				candidate.clone().into(),
				self.id.clone().into(),
			);
			match action {
				NominationChange::Revoke => {
					// remove from pending requests
					self.requests.less_total -= amount;
					// remove nomination from nominator state
					self.rm_nomination(candidate.clone());
					// remove nomination from collator state delegations
					Pallet::<T>::delegator_leaves_collator(
						nominator_id.clone(),
						candidate_id.clone(),
					)?;
					Ok(Event::NominationRevoked(
						nominator_id,
						candidate_id,
						balance_amt,
					))
				}
				NominationChange::Increase => {
					// remove from pending requests
					self.requests.more_total -= amount;
					// increase delegation
					for x in &mut self.delegations.0 {
						if x.owner == candidate {
							x.amount += amount;
							self.total += amount;
							// update collator state nomination
							let mut collator_state = <CandidateState<T>>::get(&candidate_id)
								.ok_or(Error::<T>::CandidateDNE)?;
							T::Currency::reserve(&self.id.clone().into(), balance_amt)?;
							let before = collator_state.total_counted;
							let in_top =
								collator_state.inc_nominator(self.id.clone().into(), balance_amt);
							let after = collator_state.total_counted;
							if collator_state.is_active() && (before != after) {
								Pallet::<T>::update_active(candidate_id.clone(), after);
							}
							<CandidateState<T>>::insert(&candidate_id, collator_state);
							let new_total_staked = <Total<T>>::get().saturating_add(balance_amt);
							<Total<T>>::put(new_total_staked);
							return Ok(Event::NominationIncreased(
								nominator_id,
								candidate_id,
								self.total.into(),
								in_top,
							));
						}
					}
					Err(Error::<T>::NominationDNE.into())
				}
				NominationChange::Decrease => {
					// remove from pending requests
					self.requests.less_total -= amount;
					// decrease nomination
					for x in &mut self.delegations.0 {
						if x.owner == candidate {
							if x.amount > amount {
								x.amount -= amount;
								self.total -= amount;
								let new_total: BalanceOf<T> = self.total.into();
								ensure!(
									new_total >= T::MinNomination::get(),
									Error::<T>::NominationBelowMin
								);
								ensure!(
									new_total >= T::MinNominatorStk::get(),
									Error::<T>::NominatorBondBelowMin
								);
								let mut collator = <CandidateState<T>>::get(&candidate_id)
									.ok_or(Error::<T>::CandidateDNE)?;
								T::Currency::unreserve(&nominator_id, balance_amt);
								let before = collator.total_counted;
								let in_top =
									collator.dec_nominator(candidate_id.clone(), balance_amt);
								let after = collator.total_counted;
								if collator.is_active() && (before != after) {
									Pallet::<T>::update_active(candidate_id.clone(), after);
								}
								<CandidateState<T>>::insert(&candidate_id, collator);
								let new_total_staked =
									<Total<T>>::get().saturating_sub(balance_amt);
								<Total<T>>::put(new_total_staked);
								return Ok(Event::NominationDecreased(
									nominator_id,
									candidate_id,
									balance_amt,
									in_top,
								));
							} else {
								// must rm entire nomination if x.amount <= less
								return Err(Error::<T>::NominationBelowMin.into());
							}
						}
					}
					Err(Error::<T>::NominationDNE.into())
				}
			}
		}
		/// Cancel pending nomination change request
		pub fn cancel_pending_request<T: Config>(
			&mut self,
			candidate: AccountId,
		) -> Result<NominationRequest<AccountId, Balance>, DispatchError> {
			ensure!(self.is_active(), Error::<T>::CannotActBecauseLeaving);
			let order = self
				.requests
				.requests
				.remove(&candidate)
				.ok_or(Error::<T>::PendingNominationRequestDNE)?;
			match order.action {
				NominationChange::Revoke => {
					self.requests.less_total -= order.amount;
				}
				NominationChange::Decrease => {
					self.requests.less_total -= order.amount;
				}
				NominationChange::Increase => {
					self.requests.more_total -= order.amount;
				}
			}
			Ok(order)
		}
	}

	#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
	/// Changes requested by the nominator
	/// - limit of 1 ongoing change per nomination
	/// - no changes allowed if nominator is leaving
	pub enum NominationChange {
		Revoke,
		Increase,
		Decrease,
	}

	#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
	pub struct NominationRequest<AccountId, Balance> {
		pub collator: AccountId,
		pub amount: Balance,
		pub when: RoundIndex,
		pub action: NominationChange,
	}

	#[derive(Clone, Encode, Decode, RuntimeDebug)]
	/// Pending requests to mutate delegations for each nominator
	pub struct PendingNominationRequests<AccountId, Balance> {
		/// Map from collator -> Request (enforces at most 1 pending request per nomination)
		pub requests: BTreeMap<AccountId, NominationRequest<AccountId, Balance>>,
		/// Sum of pending revocation amounts + bond less amounts
		pub less_total: Balance,
		/// Sum of pending bond more amounts
		pub more_total: Balance,
	}

	impl<A: Ord, B: Zero> Default for PendingNominationRequests<A, B> {
		fn default() -> PendingNominationRequests<A, B> {
			PendingNominationRequests {
				requests: BTreeMap::new(),
				less_total: B::zero(),
				more_total: B::zero(),
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
		> PendingNominationRequests<A, B>
	{
		/// New default (empty) pending requests
		pub fn new() -> PendingNominationRequests<A, B> {
			PendingNominationRequests::default()
		}
		/// Add bond more order to pending requests
		pub fn bond_more<T: Config>(
			&mut self,
			collator: A,
			amount: B,
			when: RoundIndex,
		) -> DispatchResult {
			ensure!(
				self.requests.get(&collator).is_none(),
				Error::<T>::PendingNominationRequestAlreadyExists
			);
			self.requests.insert(
				collator.clone(),
				NominationRequest {
					collator,
					amount,
					when,
					action: NominationChange::Increase,
				},
			);
			self.more_total += amount;
			Ok(())
		}
		/// Add bond less order to pending requests, only succeeds if returns true
		/// - limit is the maximum amount allowed that can be subtracted from the nomination
		/// before it would be below the minimum nomination amount
		pub fn bond_less<T: Config>(
			&mut self,
			collator: A,
			amount: B,
			when: RoundIndex,
		) -> DispatchResult {
			ensure!(
				self.requests.get(&collator).is_none(),
				Error::<T>::PendingNominationRequestAlreadyExists
			);
			self.requests.insert(
				collator.clone(),
				NominationRequest {
					collator,
					amount,
					when,
					action: NominationChange::Decrease,
				},
			);
			self.less_total += amount;
			Ok(())
		}
		/// Add revoke order to pending requests
		/// - limit is the maximum amount allowed that can be subtracted from the nomination
		/// before it would be below the minimum nomination amount
		pub fn revoke<T: Config>(
			&mut self,
			collator: A,
			amount: B,
			when: RoundIndex,
		) -> DispatchResult {
			ensure!(
				self.requests.get(&collator).is_none(),
				Error::<T>::PendingNominationRequestAlreadyExists
			);
			self.requests.insert(
				collator.clone(),
				NominationRequest {
					collator,
					amount,
					when,
					action: NominationChange::Revoke,
				},
			);
			self.less_total += amount;
			Ok(())
		}
	}

	#[derive(Clone, Encode, Decode, RuntimeDebug)]
	/// DEPRECATED in favor of Delegator
	/// Nominator state
	pub struct Nominator2<AccountId, Balance> {
		/// All current delegations
		pub delegations: OrderedSet<Bond<AccountId, Balance>>,
		/// Nominations scheduled to be revoked
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

	#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
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

	#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
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

	#[derive(Encode, Decode, RuntimeDebug, Default)]
	/// Store and process all delayed exits by collators and nominators
	pub struct ExitQ<AccountId> {
		/// Candidate exit set
		pub candidates: OrderedSet<AccountId>,
		/// Nominator exit set (does not include nominators that made `revoke` requests)
		pub nominators_leaving: OrderedSet<AccountId>,
		/// [Candidate, Round to Exit]
		pub candidate_schedule: Vec<(AccountId, RoundIndex)>,
		/// [Nominator, Some(ValidatorId) || None => All Nominations, Round To Exit]
		pub nominator_schedule: Vec<(AccountId, Option<AccountId>, RoundIndex)>,
	}

	type RoundIndex = u32;
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
		/// Number of rounds that collators remain bonded before exit request is executable
		#[pallet::constant]
		type LeaveCandidatesDelay: Get<RoundIndex>;
		/// Number of rounds that collator requests to adjust self-bond must wait to be executable
		#[pallet::constant]
		type CandidateBondDelay: Get<RoundIndex>;
		/// Number of rounds that nominators remain bonded before exit request is executable
		#[pallet::constant]
		type LeaveNominatorsDelay: Get<RoundIndex>;
		/// Number of rounds that delegations remain bonded before revocation request is executable
		#[pallet::constant]
		type RevokeNominationDelay: Get<RoundIndex>;
		/// Number of rounds that nominator bond {more, less} requests must wait before executable
		#[pallet::constant]
		type NominatorBondDelay: Get<RoundIndex>;
		/// Number of rounds after which block authors are rewarded
		#[pallet::constant]
		type RewardPaymentDelay: Get<RoundIndex>;
		/// Minimum number of selected candidates every round
		#[pallet::constant]
		type MinSelectedCandidates: Get<u32>;
		/// Maximum nominators counted per collator
		#[pallet::constant]
		type MaxNominatorsPerCollator: Get<u32>;
		/// Maximum collators per nominator
		#[pallet::constant]
		type MaxCollatorsPerNominator: Get<u32>;
		/// Default commission due to collators, set at genesis
		#[pallet::constant]
		type DefaultCollatorCommission: Get<Perbill>;
		/// Default percent of inflation set aside for parachain bond account
		#[pallet::constant]
		type DefaultParachainBondReservePercent: Get<Percent>;
		/// Minimum stake required for any account to be in `SelectedCandidates` for the round
		#[pallet::constant]
		type MinCollatorStk: Get<BalanceOf<Self>>;
		/// Minimum stake required for any account to be a collator candidate
		#[pallet::constant]
		type MinCollatorCandidateStk: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to nominate
		#[pallet::constant]
		type MinNomination: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to become a nominator
		#[pallet::constant]
		type MinNominatorStk: Get<BalanceOf<Self>>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		NominatorDNE,
		NominatorDNEinTopNorBottom,
		NominatorDNEInNominatorSet,
		CandidateDNE,
		NominationDNE,
		NominatorExists,
		CandidateExists,
		CollatorBondBelowMin,
		NominatorBondBelowMin, // TODO: change to Delegator
		NominationBelowMin,
		AlreadyOffline,
		AlreadyActive,
		NominatorAlreadyLeaving,
		NominatorNotLeaving,
		NominatorCannotLeaveYet,
		NominationAlreadyRevoked,
		CandidateAlreadyLeaving,
		CandidateNotLeaving,
		CandidateCannotLeaveYet,
		CannotActBecauseLeaving,
		CannotActBecauseRevoking,
		ExceedMaxCollatorsPerNom,
		AlreadyNominatedCollator,
		InvalidSchedule,
		CannotSetBelowMin,
		NoWritingSameValue,
		TooLowCandidateCountWeightHintJoinCandidates,
		TooLowCandidateCountWeightHintCancelLeaveCandidates,
		TooLowCollatorCandidateCountToLeaveCandidates,
		TooLowNominationCountToNominate,
		TooLowCollatorNominationCountToNominate,
		TooLowNominationCountToLeaveNominators,
		PendingCollatorRequestDoesNotMatchCall,
		PendingCollatorRequestDNE,
		PendingCollatorRequestAlreadyExists,
		PendingCollatorRequestNotDueYet,
		PendingNominationRequestDoesNotMatchCall,
		PendingNominationRequestDNE,
		PendingNominationRequestAlreadyExists,
		PendingNominationRequestNotDueYet,
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
		/// Collator Account, Amount To Increase, Round at which request can be executed by caller
		CollatorBondMoreRequested(T::AccountId, BalanceOf<T>, RoundIndex),
		/// Collator Account, Amount To Decrease, Round at which request can be executed by caller
		CollatorBondLessRequested(T::AccountId, BalanceOf<T>, RoundIndex),
		/// Collator Account, Old Bond, New Bond
		CollatorBondedMore(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Collator Account, Old Bond, New Bond
		CollatorBondedLess(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		CollatorWentOffline(RoundIndex, T::AccountId),
		CollatorBackOnline(RoundIndex, T::AccountId),
		/// Round, Collator Account, Scheduled Exit
		CollatorScheduledExit(RoundIndex, T::AccountId, RoundIndex),
		/// Collator Account
		CancelledCandidateExit(T::AccountId),
		/// Collator Account, Cancelled Request
		CancelledCollatorBondChange(T::AccountId, CandidateBondChange<BalanceOf<T>>),
		/// Account, Amount Unlocked, New Total Amt Locked
		CollatorLeft(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Nominator, Collator, Amount to be increased, Round at which can be executed
		NominationIncreaseScheduled(T::AccountId, T::AccountId, BalanceOf<T>, RoundIndex),
		/// Nominator, Collator, Amount to be decreased, Round at which can be executed
		NominationDecreaseScheduled(T::AccountId, T::AccountId, BalanceOf<T>, RoundIndex),
		// Nominator, Collator, Amount, If in top delegations for collator after increase
		NominationIncreased(T::AccountId, T::AccountId, BalanceOf<T>, bool),
		// Nominator, Collator, Amount, If in top delegations for collator after decrease
		NominationDecreased(T::AccountId, T::AccountId, BalanceOf<T>, bool),
		/// Round, Nominator, Scheduled Exit
		NominatorExitScheduled(RoundIndex, T::AccountId, RoundIndex),
		/// Round, Nominator, Collator, Scheduled Exit
		NominationRevocationScheduled(RoundIndex, T::AccountId, T::AccountId, RoundIndex),
		/// Nominator, Amount Unstaked
		NominatorLeft(T::AccountId, BalanceOf<T>),
		/// Nominator, Collator, Amount Unstaked
		NominationRevoked(T::AccountId, T::AccountId, BalanceOf<T>),
		/// Nominator
		NominatorExitCancelled(T::AccountId),
		/// Nominator, Cancelled Request
		CancelledNominationRequest(T::AccountId, NominationRequest<T::AccountId, BalanceOf<T>>),
		/// Nominator, Amount Locked, Collator, Nominator Position with New Total Counted if in Top
		Nomination(
			T::AccountId,
			BalanceOf<T>,
			T::AccountId,
			DelegatorAdded<BalanceOf<T>>,
		),
		/// Nominator, Collator, Amount Unstaked, New Total Amt Staked for Collator
		NominatorLeftCollator(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Paid the account (nominator or collator) the balance as liquid rewards
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
			let mut round = <Round<T>>::get();
			if round.should_update(n) {
				// mutate round
				round.update(n);
				// pay all stakers for T::RewardPaymentDelay rounds ago
				Self::pay_stakers(round.current);
				// select top collator candidates for next round
				let (collator_count, nomination_count, total_staked) =
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
				T::WeightInfo::active_on_initialize(collator_count, nomination_count)
			} else {
				T::WeightInfo::passive_on_initialize()
			}
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
	type Round<T: Config> = StorageValue<_, RoundInfo<T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nominator_state2)]
	/// DEPRECATED in favor of DelegatorState
	/// Get nominator state associated with an account if account is nominating else None
	type NominatorState2<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Nominator2<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn delegator_state)]
	/// Get delegator state associated with an account if account is delegating else None
	type DelegatorState<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Delegator<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn candidate_state)]
	/// Get collator state associated with an account if account is collating else None
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
	/// DEPRECATED: TODO remove this code
	/// A queue of collators and nominators awaiting exit
	type ExitQueue2<T: Config> = StorageValue<_, ExitQ<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn at_stake)]
	/// Snapshot of collator nomination stake at the start of the round
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
	#[pallet::getter(fn staked)]
	/// Total backing stake for selected candidates in the round
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
					T::Currency::free_balance(&candidate) >= balance,
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
			let mut col_nominator_count: BTreeMap<T::AccountId, u32> = BTreeMap::new();
			let mut nom_nominator_count: BTreeMap<T::AccountId, u32> = BTreeMap::new();
			// Initialize the delegations
			for &(ref nominator, ref target, balance) in &self.delegations {
				assert!(
					T::Currency::free_balance(&nominator) >= balance,
					"Account does not have enough balance to place nomination."
				);
				let cn_count = if let Some(x) = col_nominator_count.get(&target) {
					*x
				} else {
					0u32
				};
				let nn_count = if let Some(x) = nom_nominator_count.get(&nominator) {
					*x
				} else {
					0u32
				};
				if let Err(error) = <Pallet<T>>::nominate(
					T::Origin::from(Some(nominator.clone()).into()),
					target.clone(),
					balance,
					cn_count,
					nn_count,
				) {
					log::warn!("Join nominators failed in genesis with error {:?}", error);
				} else {
					if let Some(x) = col_nominator_count.get_mut(&target) {
						*x += 1u32;
					} else {
						col_nominator_count.insert(target.clone(), 1u32);
					};
					if let Some(x) = nom_nominator_count.get_mut(&nominator) {
						*x += 1u32;
					} else {
						nom_nominator_count.insert(nominator.clone(), 1u32);
					};
				}
			}
			// Set collator commission to default config
			<CollatorCommission<T>>::put(T::DefaultCollatorCommission::get());
			// Set parachain bond config to default config
			<ParachainBondInfo<T>>::put(ParachainBondConfig {
				// must be set soon; if not => due inflation will be sent to collators/nominators
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
			ensure!(!Self::is_nominator(&acc), Error::<T>::NominatorExists);
			ensure!(
				bond >= T::MinCollatorCandidateStk::get(),
				Error::<T>::CollatorBondBelowMin
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
		#[pallet::weight(<T as Config>::WeightInfo::leave_candidates(*candidate_count))]
		/// Request to leave the set of candidates. If successful, the account is immediately
		/// removed from the candidate pool to prevent selection as a collator.
		pub fn leave_candidates(
			origin: OriginFor<T>,
			candidate_count: u32,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let (now, when) = state.leave::<T>()?;
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidate_count >= candidates.0.len() as u32,
				Error::<T>::TooLowCollatorCandidateCountToLeaveCandidates
			);
			if candidates.remove(&Bond::from_owner(collator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorScheduledExit(now, collator, when));
			Ok(().into())
		}
		#[pallet::weight(0)]
		/// Execute leave candidates request
		pub fn execute_leave_candidates(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			// TODO: should we let anyone call this by adding arg `collator: AccountId`
			let collator = ensure_signed(origin)?;
			let state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(state.can_leave::<T>()?, Error::<T>::CandidateCannotLeaveYet);
			let return_stake = |bond: Bond<T::AccountId, BalanceOf<T>>| {
				T::Currency::unreserve(&bond.owner, bond.amount);
				// remove nomination from nominator state
				let mut nominator = DelegatorState::<T>::get(&bond.owner).expect(
					"Collator state and nominator state are consistent. 
						Collator state has a record of this nomination. Therefore, 
						Nominator state also has a record. qed.",
				);
				if let Some(remaining) = nominator.rm_nomination(collator.clone()) {
					if remaining.is_zero() {
						// TODO: consider NominatorLeft event here? or logging it
						<DelegatorState<T>>::remove(&bond.owner);
					} else {
						<DelegatorState<T>>::insert(&bond.owner, nominator);
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
			<CandidateState<T>>::remove(&collator);
			let new_total_staked = <Total<T>>::get().saturating_sub(state.total_backing);
			<Total<T>>::put(new_total_staked);
			Self::deposit_event(Event::CollatorLeft(
				collator,
				state.total_backing,
				new_total_staked,
			));
			Ok(().into())
		}
		#[pallet::weight(0)]
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
			Self::deposit_event(Event::CollatorWentOffline(
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
			ensure!(!state.is_leaving(), Error::<T>::CannotActBecauseLeaving);
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
			Self::deposit_event(Event::CollatorBackOnline(
				<Round<T>>::get().current,
				collator,
			));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::candidate_bond_more())]
		/// Request by collator candidate to increase self bond by `more`
		pub fn candidate_bond_more(
			origin: OriginFor<T>,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(), Error::<T>::CannotActBecauseLeaving);
			let when = state.schedule_bond_more::<T>(more)?;
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorBondMoreRequested(collator, more, when));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::candidate_bond_less())]
		/// Request by collator candidate to decrease self bond by `less`
		pub fn candidate_bond_less(
			origin: OriginFor<T>,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(), Error::<T>::CannotActBecauseLeaving);
			let when = state.schedule_bond_less::<T>(less)?;
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorBondLessRequested(collator, less, when));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::candidate_bond_more())]
		/// Execute pending request to adjust the collator candidate self bond
		pub fn execute_candidate_bond_request(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let event = state.execute_pending_request::<T>()?;
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(event);
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::candidate_bond_more())]
		/// Cancel pending request to adjust the collator candidate self bond
		pub fn cancel_candidate_bond_request(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let event = state.cancel_pending_request::<T>()?;
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(event);
			Ok(().into())
		}
		#[pallet::weight(
			<T as Config>::WeightInfo::nominate(
				*collator_nominator_count,
				*nomination_count
			)
		)]
		/// If caller is not a nominator and not a collator, then join the set of nominators
		/// If caller is a nominator, then makes nomination to change their nomination state
		pub fn nominate(
			origin: OriginFor<T>,
			collator: T::AccountId,
			amount: BalanceOf<T>,
			collator_nominator_count: u32,
			nomination_count: u32,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			let nominator = if let Some(mut state) = <DelegatorState<T>>::get(&acc) {
				ensure!(state.is_active(), Error::<T>::CannotActBecauseLeaving);
				// nomination after first
				ensure!(
					amount >= T::MinNomination::get(),
					Error::<T>::NominationBelowMin
				);
				ensure!(
					nomination_count >= state.delegations.0.len() as u32,
					Error::<T>::TooLowNominationCountToNominate
				);
				ensure!(
					(state.delegations.0.len() as u32) < T::MaxCollatorsPerNominator::get(),
					Error::<T>::ExceedMaxCollatorsPerNom
				);
				ensure!(
					state.add_delegation(Bond {
						owner: collator.clone(),
						amount
					}),
					Error::<T>::AlreadyNominatedCollator
				);
				state
			} else {
				// first nomination
				ensure!(
					amount >= T::MinNominatorStk::get(),
					Error::<T>::NominatorBondBelowMin
				);
				ensure!(!Self::is_candidate(&acc), Error::<T>::CandidateExists);
				Delegator::new(acc.clone(), collator.clone(), amount)
			};
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(
				collator_nominator_count >= state.delegators.0.len() as u32,
				Error::<T>::TooLowCollatorNominationCountToNominate
			);
			let nominator_position = state.add_nominator::<T>(acc.clone(), amount)?;
			T::Currency::reserve(&acc, amount)?;
			if let DelegatorAdded::AddedToTop { new_total } = nominator_position {
				if state.is_active() {
					// collator in candidate pool
					Self::update_active(collator.clone(), new_total);
				}
			}
			let new_total_locked = <Total<T>>::get() + amount;
			<Total<T>>::put(new_total_locked);
			<CandidateState<T>>::insert(&collator, state);
			<DelegatorState<T>>::insert(&acc, nominator);
			Self::deposit_event(Event::Nomination(acc, amount, collator, nominator_position));
			Ok(().into())
		}
		#[pallet::weight(0)]
		/// Request to leave the set of nominators. If successful, the nominator is scheduled
		/// to be allowed to exit. Success forbids future nominator actions until the request is
		/// invoked or cancelled.
		pub fn leave_nominators(
			origin: OriginFor<T>,
			nomination_count: u32,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			let mut state = <DelegatorState<T>>::get(&acc).ok_or(Error::<T>::NominatorDNE)?;
			ensure!(!state.is_leaving(), Error::<T>::NominatorAlreadyLeaving);
			ensure!(
				nomination_count >= (state.delegations.0.len() as u32),
				Error::<T>::TooLowNominationCountToLeaveNominators
			);
			let (now, when) = state.leave::<T>();
			<DelegatorState<T>>::insert(&acc, state);
			Self::deposit_event(Event::NominatorExitScheduled(now, acc, when));
			Ok(().into())
		}
		#[pallet::weight(0)]
		/// Execute the right to exit the set of nominators and revoke all ongoing delegations.
		pub fn execute_leave_nominators(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let nominator = ensure_signed(origin)?;
			let state = <DelegatorState<T>>::get(&nominator).ok_or(Error::<T>::NominatorDNE)?;
			ensure!(state.can_leave::<T>()?, Error::<T>::NominatorCannotLeaveYet);
			for bond in state.delegations.0 {
				if let Err(error) =
					Self::delegator_leaves_collator(nominator.clone(), bond.owner.clone())
				{
					log::warn!("Nominator exit collator failed with error: {:?}", error);
				}
			}
			<DelegatorState<T>>::remove(&nominator);
			Self::deposit_event(Event::NominatorLeft(nominator, state.total));
			Ok(().into())
		}
		#[pallet::weight(0)]
		/// Cancel a pending request to exit the set of delegators. Success clears the pending exit
		/// request (thereby resetting the delay upon another `leave_nominators` call).
		pub fn cancel_leave_nominators(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let nominator = ensure_signed(origin)?;
			// ensure delegator state exists
			let mut state = <DelegatorState<T>>::get(&nominator).ok_or(Error::<T>::NominatorDNE)?;
			// ensure state is leaving
			ensure!(state.is_leaving(), Error::<T>::NominatorDNE);
			// cancel exit request
			state.cancel_leave();
			<DelegatorState<T>>::insert(&nominator, state);
			Self::deposit_event(Event::NominatorExitCancelled(nominator));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::revoke_nomination())]
		/// Request to revoke an existing delegation. If successful, the delegation is scheduled
		/// to be allowed to be revoked via the `execute_delegation_request` extrinsic.
		pub fn revoke_nomination(
			origin: OriginFor<T>,
			collator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let nominator = ensure_signed(origin)?;
			let mut state = <DelegatorState<T>>::get(&nominator).ok_or(Error::<T>::NominatorDNE)?;
			ensure!(state.is_active(), Error::<T>::CannotActBecauseLeaving);
			// if >1 delegations then only revoke 1, else schedule to leave set of delegators
			if let Some((remaining_future_total, now, when)) =
				state.schedule_revoke::<T>(collator.clone())?
			{
				// schedule revocation iff remaining total is not below min delegator stake
				ensure!(
					remaining_future_total >= T::MinNominatorStk::get(),
					Error::<T>::NominatorBondBelowMin
				);
				<DelegatorState<T>>::insert(&nominator, state);
				Self::deposit_event(Event::NominationRevocationScheduled(
					now, nominator, collator, when,
				));
			} else {
				// was last delegation so will exit instead
				// ensure the last delegation matches revocation to trigger scheduled exit
				ensure!(
					state.delegations.0[0].owner == collator,
					Error::<T>::NominationDNE
				);
				let (now, when) = state.leave::<T>();
				<DelegatorState<T>>::insert(&nominator, state);
				Self::deposit_event(Event::NominatorExitScheduled(now, nominator, when));
			}
			Ok(().into())
		}
		#[pallet::weight(0)]
		/// Request to bond more for delegators wrt a specific collator candidate.
		pub fn delegator_bond_more(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let nominator = ensure_signed(origin)?;
			let mut state = <DelegatorState<T>>::get(&nominator).ok_or(Error::<T>::NominatorDNE)?;
			ensure!(state.is_active(), Error::<T>::CannotActBecauseLeaving);
			let when = state.schedule_increase_delegation::<T>(candidate.clone(), more)?;
			<DelegatorState<T>>::insert(&nominator, state);
			Self::deposit_event(Event::NominationIncreaseScheduled(
				nominator, candidate, more, when,
			));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::nominator_bond_less())]
		/// Request bond less for delegators wrt a specific collator candidate.
		pub fn delegator_bond_less(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			let mut state = <DelegatorState<T>>::get(&caller).ok_or(Error::<T>::NominatorDNE)?;
			ensure!(state.is_active(), Error::<T>::CannotActBecauseLeaving);
			let when = state.schedule_decrease_delegation::<T>(candidate.clone(), less)?;
			<DelegatorState<T>>::insert(&caller, state);
			Self::deposit_event(Event::NominationDecreaseScheduled(
				caller, candidate, less, when,
			));
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::nominator_bond_less())]
		/// Execute pending request to change an existing delegation
		pub fn execute_delegation_request(
			origin: OriginFor<T>,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let caller = ensure_signed(origin)?;
			let mut state = <DelegatorState<T>>::get(&caller).ok_or(Error::<T>::NominatorDNE)?;
			let event = state.execute_pending_request::<T>(candidate.clone())?;
			<DelegatorState<T>>::insert(&caller, state);
			Self::deposit_event(event);
			Ok(().into())
		}
		#[pallet::weight(<T as Config>::WeightInfo::revoke_nomination())]
		/// Cancel request to change an existing delegation.
		pub fn cancel_delegation_request(
			origin: OriginFor<T>,
			collator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let nominator = ensure_signed(origin)?;
			let mut state = <DelegatorState<T>>::get(&nominator).ok_or(Error::<T>::NominatorDNE)?;
			let request = state.cancel_pending_request::<T>(collator.clone())?;
			<DelegatorState<T>>::insert(&nominator, state);
			Self::deposit_event(Event::CancelledNominationRequest(nominator, request));
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		// TODO: change to is_delegator
		pub fn is_nominator(acc: &T::AccountId) -> bool {
			<DelegatorState<T>>::get(acc).is_some()
		}
		pub fn is_candidate(acc: &T::AccountId) -> bool {
			<CandidateState<T>>::get(acc).is_some()
		}
		pub fn is_selected_candidate(acc: &T::AccountId) -> bool {
			<SelectedCandidates<T>>::get().binary_search(acc).is_ok()
		}
		// ensure candidate is active before calling
		fn update_active(candidate: T::AccountId, total: BalanceOf<T>) {
			let mut candidates = <CandidatePool<T>>::get();
			candidates.remove(&Bond::from_owner(candidate.clone()));
			candidates.insert(Bond {
				owner: candidate,
				amount: total,
			});
			<CandidatePool<T>>::put(candidates);
		}
		// Calculate round issuance based on total staked for the given round
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
		pub(crate) fn delegator_leaves_collator(
			nominator: T::AccountId,
			collator: T::AccountId,
		) -> DispatchResult {
			let mut state = <CandidateState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let (total_changed, nominator_stake) = state.rm_delegator::<T>(nominator.clone())?;
			T::Currency::unreserve(&nominator, nominator_stake);
			if state.is_active() && total_changed {
				Self::update_active(collator.clone(), state.total_counted);
			}
			let new_total_locked = <Total<T>>::get() - nominator_stake;
			<Total<T>>::put(new_total_locked);
			let new_total = state.total_counted;
			<CandidateState<T>>::insert(&collator, state);
			Self::deposit_event(Event::NominatorLeftCollator(
				nominator,
				collator,
				nominator_stake,
				new_total,
			));
			Ok(())
		}
		fn pay_stakers(next: RoundIndex) {
			// payout is next - duration rounds ago => next - duration > 0 else return early
			let duration = T::RewardPaymentDelay::get();
			if next <= duration {
				return;
			}
			let round_to_payout = next - duration;
			let total = <Points<T>>::get(round_to_payout);
			if total.is_zero() {
				return;
			}
			let total_staked = <Staked<T>>::get(round_to_payout);
			let mut issuance = Self::compute_issuance(total_staked);
			// reserve portion of issuance for parachain bond account
			let bond_config = <ParachainBondInfo<T>>::get();
			let parachain_bond_reserve = bond_config.percent * issuance;
			if let Ok(imb) =
				T::Currency::deposit_into_existing(&bond_config.account, parachain_bond_reserve)
			{
				// update round issuance iff transfer succeeds
				issuance -= imb.peek();
				Self::deposit_event(Event::ReservedForParachainBond(
					bond_config.account,
					imb.peek(),
				));
			}
			let mint = |amt: BalanceOf<T>, to: T::AccountId| {
				if let Ok(imb) = T::Currency::deposit_into_existing(&to, amt) {
					Self::deposit_event(Event::Rewarded(to.clone(), imb.peek()));
				}
			};
			let collator_fee = <CollatorCommission<T>>::get();
			for (val, pts) in <AwardedPts<T>>::drain_prefix(round_to_payout) {
				let pct_due = Perbill::from_rational(pts, total);
				let mut amt_due = pct_due * issuance;
				// Take the snapshot of block author and delegations
				let state = <AtStake<T>>::take(round_to_payout, &val);
				if state.delegations.is_empty() {
					// solo collator with no nominators
					mint(amt_due, val.clone());
				} else {
					// pay collator first; commission + due_portion
					let val_pct = Perbill::from_rational(state.bond, state.total);
					let commission = collator_fee * amt_due;
					amt_due -= commission;
					let val_due = (val_pct * amt_due) + commission;
					mint(val_due, val.clone());
					// pay nominators due portion
					for Bond { owner, amount } in state.delegations {
						let percent = Perbill::from_rational(amount, state.total);
						let due = percent * amt_due;
						mint(due, owner);
					}
				}
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
			candidates
				.into_iter()
				.rev()
				.take(top_n)
				.filter(|x| x.amount >= T::MinCollatorStk::get())
				.map(|x| x.owner)
				.collect::<Vec<T::AccountId>>()
		}
		/// Best as in most cumulatively supported in terms of stake
		/// Returns [collator_count, nomination_count, total staked]
		fn select_top_candidates(next: RoundIndex) -> (u32, u32, BalanceOf<T>) {
			let (mut collator_count, mut nomination_count, mut total) =
				(0u32, 0u32, BalanceOf::<T>::zero());
			// choose the top TotalSelected qualified candidates, ordered by stake
			let mut collators = Self::compute_top_candidates();
			// snapshot exposure for round for weighting reward distribution
			for account in collators.iter() {
				let state = <CandidateState<T>>::get(&account)
					.expect("all members of CandidateQ must be candidates");
				collator_count += 1u32;
				nomination_count += state.delegators.0.len() as u32;
				let amount = state.total_counted;
				total += amount;
				let exposure: CollatorSnapshot<T::AccountId, BalanceOf<T>> = state.into();
				<AtStake<T>>::insert(next, account, exposure);
				Self::deposit_event(Event::CollatorChosen(next, account.clone(), amount));
			}
			collators.sort();
			// insert canonical collator set
			<SelectedCandidates<T>>::put(collators);
			(collator_count, nomination_count, total)
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
