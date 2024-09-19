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

//! Types for parachain-staking

use crate::{
	auto_compound::AutoCompoundDelegations, set::OrderedSet, BalanceOf, BottomDelegations,
	CandidateInfo, Config, DelegatorState, Error, Event, Pallet, Round, RoundIndex, TopDelegations,
	Total, COLLATOR_LOCK_ID, DELEGATOR_LOCK_ID,
};
use frame_support::{
	pallet_prelude::*,
	traits::{tokens::WithdrawReasons, LockableCurrency},
};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Saturating, Zero},
	Perbill, Percent, RuntimeDebug,
};
use sp_std::{cmp::Ordering, collections::btree_map::BTreeMap, prelude::*};

pub struct CountedDelegations<T: Config> {
	pub uncounted_stake: BalanceOf<T>,
	pub rewardable_delegations: Vec<Bond<T::AccountId, BalanceOf<T>>>,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Bond<AccountId, Balance> {
	pub owner: AccountId,
	pub amount: Balance,
}

impl<A: Decode, B: Default> Default for Bond<A, B> {
	fn default() -> Bond<A, B> {
		Bond {
			owner: A::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
				.expect("infinite length input; no invalid inputs for type; qed"),
			amount: B::default(),
		}
	}
}

impl<A, B: Default> Bond<A, B> {
	pub fn from_owner(owner: A) -> Self {
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

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct BondWithAutoCompound<AccountId, Balance> {
	pub owner: AccountId,
	pub amount: Balance,
	pub auto_compound: Percent,
}

impl<A: Decode, B: Default> Default for BondWithAutoCompound<A, B> {
	fn default() -> BondWithAutoCompound<A, B> {
		BondWithAutoCompound {
			owner: A::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
				.expect("infinite length input; no invalid inputs for type; qed"),
			amount: B::default(),
			auto_compound: Percent::zero(),
		}
	}
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
/// Snapshot of collator state at the start of the round for which they are selected
pub struct CollatorSnapshot<AccountId, Balance> {
	/// The total value locked by the collator.
	pub bond: Balance,

	/// The rewardable delegations. This list is a subset of total delegators, where certain
	/// delegators are adjusted based on their scheduled
	/// [DelegationChange::Revoke] or [DelegationChange::Decrease] action.
	pub delegations: Vec<BondWithAutoCompound<AccountId, Balance>>,

	/// The total counted value locked for the collator, including the self bond + total staked by
	/// top delegators.
	pub total: Balance,
}

impl<A: PartialEq, B: PartialEq> PartialEq for CollatorSnapshot<A, B> {
	fn eq(&self, other: &Self) -> bool {
		let must_be_true = self.bond == other.bond && self.total == other.total;
		if !must_be_true {
			return false;
		}
		for (
			BondWithAutoCompound {
				owner: o1,
				amount: a1,
				auto_compound: c1,
			},
			BondWithAutoCompound {
				owner: o2,
				amount: a2,
				auto_compound: c2,
			},
		) in self.delegations.iter().zip(other.delegations.iter())
		{
			if o1 != o2 || a1 != a2 || c1 != c2 {
				return false;
			}
		}
		true
	}
}

impl<A, B: Default> Default for CollatorSnapshot<A, B> {
	fn default() -> CollatorSnapshot<A, B> {
		CollatorSnapshot {
			bond: B::default(),
			delegations: Vec::new(),
			total: B::default(),
		}
	}
}

#[derive(Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
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
/// DEPRECATED, replaced by `CandidateMetadata` and two storage instances of `Delegations`
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

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
/// Type for top and bottom delegation storage item
pub struct Delegations<AccountId, Balance> {
	pub delegations: Vec<Bond<AccountId, Balance>>,
	pub total: Balance,
}

impl<A, B: Default> Default for Delegations<A, B> {
	fn default() -> Delegations<A, B> {
		Delegations {
			delegations: Vec::new(),
			total: B::default(),
		}
	}
}

impl<AccountId, Balance: Copy + Ord + sp_std::ops::AddAssign + Zero + Saturating>
	Delegations<AccountId, Balance>
{
	pub fn sort_greatest_to_least(&mut self) {
		self.delegations.sort_by(|a, b| b.amount.cmp(&a.amount));
	}
	/// Insert sorted greatest to least and increase .total accordingly
	/// Insertion respects first come first serve so new delegations are pushed after existing
	/// delegations if the amount is the same
	pub fn insert_sorted_greatest_to_least(&mut self, delegation: Bond<AccountId, Balance>) {
		self.total = self.total.saturating_add(delegation.amount);
		// if delegations nonempty && last_element == delegation.amount => push input and return
		if !self.delegations.is_empty() {
			// if last_element == delegation.amount => push the delegation and return early
			if self.delegations[self.delegations.len() - 1].amount == delegation.amount {
				self.delegations.push(delegation);
				// early return
				return;
			}
		}
		// else binary search insertion
		match self
			.delegations
			.binary_search_by(|x| delegation.amount.cmp(&x.amount))
		{
			// sorted insertion on sorted vec
			// enforces first come first serve for equal bond amounts
			Ok(i) => {
				let mut new_index = i + 1;
				while new_index <= (self.delegations.len() - 1) {
					if self.delegations[new_index].amount == delegation.amount {
						new_index = new_index.saturating_add(1);
					} else {
						self.delegations.insert(new_index, delegation);
						return;
					}
				}
				self.delegations.push(delegation)
			}
			Err(i) => self.delegations.insert(i, delegation),
		}
	}
	/// Return the capacity status for top delegations
	pub fn top_capacity<T: Config>(&self) -> CapacityStatus {
		match &self.delegations {
			x if x.len() as u32 >= T::MaxTopDelegationsPerCandidate::get() => CapacityStatus::Full,
			x if x.is_empty() => CapacityStatus::Empty,
			_ => CapacityStatus::Partial,
		}
	}
	/// Return the capacity status for bottom delegations
	pub fn bottom_capacity<T: Config>(&self) -> CapacityStatus {
		match &self.delegations {
			x if x.len() as u32 >= T::MaxBottomDelegationsPerCandidate::get() => {
				CapacityStatus::Full
			}
			x if x.is_empty() => CapacityStatus::Empty,
			_ => CapacityStatus::Partial,
		}
	}
	/// Return last delegation amount without popping the delegation
	pub fn lowest_delegation_amount(&self) -> Balance {
		self.delegations
			.last()
			.map(|x| x.amount)
			.unwrap_or(Balance::zero())
	}
	/// Return highest delegation amount
	pub fn highest_delegation_amount(&self) -> Balance {
		self.delegations
			.first()
			.map(|x| x.amount)
			.unwrap_or(Balance::zero())
	}
}

#[derive(PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
/// Capacity status for top or bottom delegations
pub enum CapacityStatus {
	/// Reached capacity
	Full,
	/// Empty aka contains no delegations
	Empty,
	/// Partially full (nonempty and not full)
	Partial,
}

#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
/// All candidate info except the top and bottom delegations
pub struct CandidateMetadata<Balance> {
	/// This candidate's self bond amount
	pub bond: Balance,
	/// Total number of delegations to this candidate
	pub delegation_count: u32,
	/// Self bond + sum of top delegations
	pub total_counted: Balance,
	/// The smallest top delegation amount
	pub lowest_top_delegation_amount: Balance,
	/// The highest bottom delegation amount
	pub highest_bottom_delegation_amount: Balance,
	/// The smallest bottom delegation amount
	pub lowest_bottom_delegation_amount: Balance,
	/// Capacity status for top delegations
	pub top_capacity: CapacityStatus,
	/// Capacity status for bottom delegations
	pub bottom_capacity: CapacityStatus,
	/// Maximum 1 pending request to decrease candidate self bond at any given time
	pub request: Option<CandidateBondLessRequest<Balance>>,
	/// Current status of the collator
	pub status: CollatorStatus,
}

impl<
		Balance: Copy
			+ Zero
			+ PartialOrd
			+ sp_std::ops::AddAssign
			+ sp_std::ops::SubAssign
			+ sp_std::ops::Sub<Output = Balance>
			+ sp_std::fmt::Debug
			+ Saturating,
	> CandidateMetadata<Balance>
{
	pub fn new(bond: Balance) -> Self {
		CandidateMetadata {
			bond,
			delegation_count: 0u32,
			total_counted: bond,
			lowest_top_delegation_amount: Zero::zero(),
			highest_bottom_delegation_amount: Zero::zero(),
			lowest_bottom_delegation_amount: Zero::zero(),
			top_capacity: CapacityStatus::Empty,
			bottom_capacity: CapacityStatus::Empty,
			request: None,
			status: CollatorStatus::Active,
		}
	}
	pub fn is_active(&self) -> bool {
		matches!(self.status, CollatorStatus::Active)
	}
	pub fn is_leaving(&self) -> bool {
		matches!(self.status, CollatorStatus::Leaving(_))
	}
	pub fn schedule_leave<T: Config>(&mut self) -> Result<(RoundIndex, RoundIndex), DispatchError> {
		ensure!(!self.is_leaving(), Error::<T>::CandidateAlreadyLeaving);
		let now = <Round<T>>::get().current;
		let when = now + T::LeaveCandidatesDelay::get();
		self.status = CollatorStatus::Leaving(when);
		Ok((now, when))
	}
	pub fn can_leave<T: Config>(&self) -> DispatchResult {
		if let CollatorStatus::Leaving(when) = self.status {
			ensure!(
				<Round<T>>::get().current >= when,
				Error::<T>::CandidateCannotLeaveYet
			);
			Ok(())
		} else {
			Err(Error::<T>::CandidateNotLeaving.into())
		}
	}
	pub fn go_offline(&mut self) {
		self.status = CollatorStatus::Idle;
	}
	pub fn go_online(&mut self) {
		self.status = CollatorStatus::Active;
	}
	pub fn bond_more<T: Config>(&mut self, who: T::AccountId, more: Balance) -> DispatchResult
	where
		BalanceOf<T>: From<Balance>,
	{
		ensure!(
			<Pallet<T>>::get_collator_stakable_free_balance(&who) >= more.into(),
			Error::<T>::InsufficientBalance
		);
		let new_total = <Total<T>>::get().saturating_add(more.into());
		<Total<T>>::put(new_total);
		self.bond = self.bond.saturating_add(more);
		T::Currency::set_lock(
			COLLATOR_LOCK_ID,
			&who.clone(),
			self.bond.into(),
			WithdrawReasons::all(),
		);
		self.total_counted = self.total_counted.saturating_add(more);
		<Pallet<T>>::deposit_event(Event::CandidateBondedMore {
			candidate: who.clone(),
			amount: more.into(),
			new_total_bond: self.bond.into(),
		});
		Ok(())
	}

	pub fn bond_less<T: Config>(&mut self, who: T::AccountId, amount: Balance)
	where
		BalanceOf<T>: From<Balance>,
	{
		let new_total_staked = <Total<T>>::get().saturating_sub(amount.into());
		<Total<T>>::put(new_total_staked);
		self.bond = self.bond.saturating_sub(amount);
		if self.bond.is_zero() {
			T::Currency::remove_lock(COLLATOR_LOCK_ID, &who);
		} else {
			T::Currency::set_lock(
				COLLATOR_LOCK_ID,
				&who,
				self.bond.into(),
				WithdrawReasons::all(),
			);
		}
		self.total_counted = self.total_counted.saturating_sub(amount);
		let event = Event::CandidateBondedLess {
			candidate: who.clone(),
			amount: amount.into(),
			new_bond: self.bond.into(),
		};
		// update candidate pool value because it must change if self bond changes
		if self.is_active() {
			Pallet::<T>::update_active(who, self.total_counted.into());
		}
		Pallet::<T>::deposit_event(event);
	}

	/// Schedule executable decrease of collator candidate self bond
	/// Returns the round at which the collator can execute the pending request
	pub fn schedule_bond_less<T: Config>(
		&mut self,
		less: Balance,
	) -> Result<RoundIndex, DispatchError>
	where
		BalanceOf<T>: Into<Balance>,
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
	/// Execute pending request to decrease the collator self bond
	/// Returns the event to be emitted
	pub fn execute_bond_less<T: Config>(&mut self, who: T::AccountId) -> DispatchResult
	where
		BalanceOf<T>: From<Balance>,
	{
		let request = self
			.request
			.ok_or(Error::<T>::PendingCandidateRequestsDNE)?;
		ensure!(
			request.when_executable <= <Round<T>>::get().current,
			Error::<T>::PendingCandidateRequestNotDueYet
		);
		self.bond_less::<T>(who.clone(), request.amount);
		// reset s.t. no pending request
		self.request = None;
		Ok(())
	}

	/// Cancel candidate bond less request
	pub fn cancel_bond_less<T: Config>(&mut self, who: T::AccountId) -> DispatchResult
	where
		BalanceOf<T>: From<Balance>,
	{
		let request = self
			.request
			.ok_or(Error::<T>::PendingCandidateRequestsDNE)?;
		let event = Event::CancelledCandidateBondLess {
			candidate: who.clone().into(),
			amount: request.amount.into(),
			execute_round: request.when_executable,
		};
		self.request = None;
		Pallet::<T>::deposit_event(event);
		Ok(())
	}
	/// Reset top delegations metadata
	pub fn reset_top_data<T: Config>(
		&mut self,
		candidate: T::AccountId,
		top_delegations: &Delegations<T::AccountId, BalanceOf<T>>,
	) where
		BalanceOf<T>: Into<Balance> + From<Balance>,
	{
		self.lowest_top_delegation_amount = top_delegations.lowest_delegation_amount().into();
		self.top_capacity = top_delegations.top_capacity::<T>();
		let old_total_counted = self.total_counted;
		self.total_counted = self.bond.saturating_add(top_delegations.total.into());
		// CandidatePool value for candidate always changes if top delegations total changes
		// so we moved the update into this function to deduplicate code and patch a bug that
		// forgot to apply the update when increasing top delegation
		if old_total_counted != self.total_counted && self.is_active() {
			Pallet::<T>::update_active(candidate, self.total_counted.into());
		}
	}
	/// Reset bottom delegations metadata
	pub fn reset_bottom_data<T: Config>(
		&mut self,
		bottom_delegations: &Delegations<T::AccountId, BalanceOf<T>>,
	) where
		BalanceOf<T>: Into<Balance>,
	{
		self.lowest_bottom_delegation_amount = bottom_delegations.lowest_delegation_amount().into();
		self.highest_bottom_delegation_amount =
			bottom_delegations.highest_delegation_amount().into();
		self.bottom_capacity = bottom_delegations.bottom_capacity::<T>();
	}
	/// Add delegation
	/// Returns whether delegator was added and an optional negative total counted remainder
	/// for if a bottom delegation was kicked
	/// MUST ensure no delegation exists for this candidate in the `DelegatorState` before call
	pub fn add_delegation<T: Config>(
		&mut self,
		candidate: &T::AccountId,
		delegation: Bond<T::AccountId, BalanceOf<T>>,
	) -> Result<(DelegatorAdded<Balance>, Option<Balance>), DispatchError>
	where
		BalanceOf<T>: Into<Balance> + From<Balance>,
	{
		let mut less_total_staked = None;
		let delegator_added = match self.top_capacity {
			CapacityStatus::Full => {
				// top is full, insert into top iff the lowest_top < amount
				if self.lowest_top_delegation_amount < delegation.amount.into() {
					// bumps lowest top to the bottom inside this function call
					less_total_staked = self.add_top_delegation::<T>(candidate, delegation);
					DelegatorAdded::AddedToTop {
						new_total: self.total_counted,
					}
				} else {
					// if bottom is full, only insert if greater than lowest bottom (which will
					// be bumped out)
					if matches!(self.bottom_capacity, CapacityStatus::Full) {
						ensure!(
							delegation.amount.into() > self.lowest_bottom_delegation_amount,
							Error::<T>::CannotDelegateLessThanOrEqualToLowestBottomWhenFull
						);
						// need to subtract from total staked
						less_total_staked = Some(self.lowest_bottom_delegation_amount);
					}
					// insert into bottom
					self.add_bottom_delegation::<T>(false, candidate, delegation);
					DelegatorAdded::AddedToBottom
				}
			}
			// top is either empty or partially full
			_ => {
				self.add_top_delegation::<T>(candidate, delegation);
				DelegatorAdded::AddedToTop {
					new_total: self.total_counted,
				}
			}
		};
		Ok((delegator_added, less_total_staked))
	}
	/// Add delegation to top delegation
	/// Returns Option<negative_total_staked_remainder>
	/// Only call if lowest top delegation is less than delegation.amount || !top_full
	pub fn add_top_delegation<T: Config>(
		&mut self,
		candidate: &T::AccountId,
		delegation: Bond<T::AccountId, BalanceOf<T>>,
	) -> Option<Balance>
	where
		BalanceOf<T>: Into<Balance> + From<Balance>,
	{
		let mut less_total_staked = None;
		let mut top_delegations = <TopDelegations<T>>::get(candidate)
			.expect("CandidateInfo existence => TopDelegations existence");
		let max_top_delegations_per_candidate = T::MaxTopDelegationsPerCandidate::get();
		if top_delegations.delegations.len() as u32 == max_top_delegations_per_candidate {
			// pop lowest top delegation
			let new_bottom_delegation = top_delegations.delegations.pop().expect("");
			top_delegations.total = top_delegations
				.total
				.saturating_sub(new_bottom_delegation.amount);
			if matches!(self.bottom_capacity, CapacityStatus::Full) {
				less_total_staked = Some(self.lowest_bottom_delegation_amount);
			}
			self.add_bottom_delegation::<T>(true, candidate, new_bottom_delegation);
		}
		// insert into top
		top_delegations.insert_sorted_greatest_to_least(delegation);
		// update candidate info
		self.reset_top_data::<T>(candidate.clone(), &top_delegations);
		if less_total_staked.is_none() {
			// only increment delegation count if we are not kicking a bottom delegation
			self.delegation_count = self.delegation_count.saturating_add(1u32);
		}
		<TopDelegations<T>>::insert(&candidate, top_delegations);
		less_total_staked
	}
	/// Add delegation to bottom delegations
	/// Check before call that if capacity is full, inserted delegation is higher than lowest
	/// bottom delegation (and if so, need to adjust the total storage item)
	/// CALLER MUST ensure(lowest_bottom_to_be_kicked.amount < delegation.amount)
	pub fn add_bottom_delegation<T: Config>(
		&mut self,
		bumped_from_top: bool,
		candidate: &T::AccountId,
		delegation: Bond<T::AccountId, BalanceOf<T>>,
	) where
		BalanceOf<T>: Into<Balance> + From<Balance>,
	{
		let mut bottom_delegations = <BottomDelegations<T>>::get(candidate)
			.expect("CandidateInfo existence => BottomDelegations existence");
		// if bottom is full, kick the lowest bottom (which is expected to be lower than input
		// as per check)
		let increase_delegation_count = if bottom_delegations.delegations.len() as u32
			== T::MaxBottomDelegationsPerCandidate::get()
		{
			let lowest_bottom_to_be_kicked = bottom_delegations
				.delegations
				.pop()
				.expect("if at full capacity (>0), then >0 bottom delegations exist; qed");
			// EXPECT lowest_bottom_to_be_kicked.amount < delegation.amount enforced by caller
			// if lowest_bottom_to_be_kicked.amount == delegation.amount, we will still kick
			// the lowest bottom to enforce first come first served
			bottom_delegations.total = bottom_delegations
				.total
				.saturating_sub(lowest_bottom_to_be_kicked.amount);
			// update delegator state
			// total staked is updated via propagation of lowest bottom delegation amount prior
			// to call
			let mut delegator_state = <DelegatorState<T>>::get(&lowest_bottom_to_be_kicked.owner)
				.expect("Delegation existence => DelegatorState existence");
			let leaving = delegator_state.delegations.0.len() == 1usize;
			delegator_state.rm_delegation::<T>(candidate);
			<Pallet<T>>::delegation_remove_request_with_state(
				&candidate,
				&lowest_bottom_to_be_kicked.owner,
				&mut delegator_state,
			);
			<AutoCompoundDelegations<T>>::remove_auto_compound(
				&candidate,
				&lowest_bottom_to_be_kicked.owner,
			);

			Pallet::<T>::deposit_event(Event::DelegationKicked {
				delegator: lowest_bottom_to_be_kicked.owner.clone(),
				candidate: candidate.clone(),
				unstaked_amount: lowest_bottom_to_be_kicked.amount,
			});
			if leaving {
				<DelegatorState<T>>::remove(&lowest_bottom_to_be_kicked.owner);
				Pallet::<T>::deposit_event(Event::DelegatorLeft {
					delegator: lowest_bottom_to_be_kicked.owner,
					unstaked_amount: lowest_bottom_to_be_kicked.amount,
				});
			} else {
				<DelegatorState<T>>::insert(&lowest_bottom_to_be_kicked.owner, delegator_state);
			}
			false
		} else {
			!bumped_from_top
		};
		// only increase delegation count if new bottom delegation (1) doesn't come from top &&
		// (2) doesn't pop the lowest delegation from the bottom
		if increase_delegation_count {
			self.delegation_count = self.delegation_count.saturating_add(1u32);
		}
		bottom_delegations.insert_sorted_greatest_to_least(delegation);
		self.reset_bottom_data::<T>(&bottom_delegations);
		<BottomDelegations<T>>::insert(candidate, bottom_delegations);
	}
	/// Remove delegation
	/// Removes from top if amount is above lowest top or top is not full
	/// Return Ok(if_total_counted_changed)
	pub fn rm_delegation_if_exists<T: Config>(
		&mut self,
		candidate: &T::AccountId,
		delegator: T::AccountId,
		amount: Balance,
	) -> Result<bool, DispatchError>
	where
		BalanceOf<T>: Into<Balance> + From<Balance>,
	{
		let amount_geq_lowest_top = amount >= self.lowest_top_delegation_amount;
		let top_is_not_full = !matches!(self.top_capacity, CapacityStatus::Full);
		let lowest_top_eq_highest_bottom =
			self.lowest_top_delegation_amount == self.highest_bottom_delegation_amount;
		let delegation_dne_err: DispatchError = Error::<T>::DelegationDNE.into();
		if top_is_not_full || (amount_geq_lowest_top && !lowest_top_eq_highest_bottom) {
			self.rm_top_delegation::<T>(candidate, delegator)
		} else if amount_geq_lowest_top && lowest_top_eq_highest_bottom {
			let result = self.rm_top_delegation::<T>(candidate, delegator.clone());
			if result == Err(delegation_dne_err) {
				// worst case removal
				self.rm_bottom_delegation::<T>(candidate, delegator)
			} else {
				result
			}
		} else {
			self.rm_bottom_delegation::<T>(candidate, delegator)
		}
	}
	/// Remove top delegation, bumps top bottom delegation if exists
	pub fn rm_top_delegation<T: Config>(
		&mut self,
		candidate: &T::AccountId,
		delegator: T::AccountId,
	) -> Result<bool, DispatchError>
	where
		BalanceOf<T>: Into<Balance> + From<Balance>,
	{
		let old_total_counted = self.total_counted;
		// remove top delegation
		let mut top_delegations = <TopDelegations<T>>::get(candidate)
			.expect("CandidateInfo exists => TopDelegations exists");
		let mut actual_amount_option: Option<BalanceOf<T>> = None;
		top_delegations.delegations = top_delegations
			.delegations
			.clone()
			.into_iter()
			.filter(|d| {
				if d.owner != delegator {
					true
				} else {
					actual_amount_option = Some(d.amount);
					false
				}
			})
			.collect();
		let actual_amount = actual_amount_option.ok_or(Error::<T>::DelegationDNE)?;
		top_delegations.total = top_delegations.total.saturating_sub(actual_amount);
		// if bottom nonempty => bump top bottom to top
		if !matches!(self.bottom_capacity, CapacityStatus::Empty) {
			let mut bottom_delegations =
				<BottomDelegations<T>>::get(candidate).expect("bottom is nonempty as just checked");
			// expect already stored greatest to least by bond amount
			let highest_bottom_delegation = bottom_delegations.delegations.remove(0);
			bottom_delegations.total = bottom_delegations
				.total
				.saturating_sub(highest_bottom_delegation.amount);
			self.reset_bottom_data::<T>(&bottom_delegations);
			<BottomDelegations<T>>::insert(candidate, bottom_delegations);
			// insert highest bottom into top delegations
			top_delegations.insert_sorted_greatest_to_least(highest_bottom_delegation);
		}
		// update candidate info
		self.reset_top_data::<T>(candidate.clone(), &top_delegations);
		self.delegation_count = self.delegation_count.saturating_sub(1u32);
		<TopDelegations<T>>::insert(candidate, top_delegations);
		// return whether total counted changed
		Ok(old_total_counted == self.total_counted)
	}
	/// Remove bottom delegation
	/// Returns if_total_counted_changed: bool
	pub fn rm_bottom_delegation<T: Config>(
		&mut self,
		candidate: &T::AccountId,
		delegator: T::AccountId,
	) -> Result<bool, DispatchError>
	where
		BalanceOf<T>: Into<Balance>,
	{
		// remove bottom delegation
		let mut bottom_delegations = <BottomDelegations<T>>::get(candidate)
			.expect("CandidateInfo exists => BottomDelegations exists");
		let mut actual_amount_option: Option<BalanceOf<T>> = None;
		bottom_delegations.delegations = bottom_delegations
			.delegations
			.clone()
			.into_iter()
			.filter(|d| {
				if d.owner != delegator {
					true
				} else {
					actual_amount_option = Some(d.amount);
					false
				}
			})
			.collect();
		let actual_amount = actual_amount_option.ok_or(Error::<T>::DelegationDNE)?;
		bottom_delegations.total = bottom_delegations.total.saturating_sub(actual_amount);
		// update candidate info
		self.reset_bottom_data::<T>(&bottom_delegations);
		self.delegation_count = self.delegation_count.saturating_sub(1u32);
		<BottomDelegations<T>>::insert(candidate, bottom_delegations);
		Ok(false)
	}
	/// Increase delegation amount
	pub fn increase_delegation<T: Config>(
		&mut self,
		candidate: &T::AccountId,
		delegator: T::AccountId,
		bond: BalanceOf<T>,
		more: BalanceOf<T>,
	) -> Result<bool, DispatchError>
	where
		BalanceOf<T>: Into<Balance> + From<Balance>,
	{
		let lowest_top_eq_highest_bottom =
			self.lowest_top_delegation_amount == self.highest_bottom_delegation_amount;
		let bond_geq_lowest_top = bond.into() >= self.lowest_top_delegation_amount;
		let delegation_dne_err: DispatchError = Error::<T>::DelegationDNE.into();
		if bond_geq_lowest_top && !lowest_top_eq_highest_bottom {
			// definitely in top
			self.increase_top_delegation::<T>(candidate, delegator.clone(), more)
		} else if bond_geq_lowest_top && lowest_top_eq_highest_bottom {
			// update top but if error then update bottom (because could be in bottom because
			// lowest_top_eq_highest_bottom)
			let result = self.increase_top_delegation::<T>(candidate, delegator.clone(), more);
			if result == Err(delegation_dne_err) {
				self.increase_bottom_delegation::<T>(candidate, delegator, bond, more)
			} else {
				result
			}
		} else {
			self.increase_bottom_delegation::<T>(candidate, delegator, bond, more)
		}
	}
	/// Increase top delegation
	pub fn increase_top_delegation<T: Config>(
		&mut self,
		candidate: &T::AccountId,
		delegator: T::AccountId,
		more: BalanceOf<T>,
	) -> Result<bool, DispatchError>
	where
		BalanceOf<T>: Into<Balance> + From<Balance>,
	{
		let mut top_delegations = <TopDelegations<T>>::get(candidate)
			.expect("CandidateInfo exists => TopDelegations exists");
		let mut in_top = false;
		top_delegations.delegations = top_delegations
			.delegations
			.clone()
			.into_iter()
			.map(|d| {
				if d.owner != delegator {
					d
				} else {
					in_top = true;
					let new_amount = d.amount.saturating_add(more);
					Bond {
						owner: d.owner,
						amount: new_amount,
					}
				}
			})
			.collect();
		ensure!(in_top, Error::<T>::DelegationDNE);
		top_delegations.total = top_delegations.total.saturating_add(more);
		top_delegations.sort_greatest_to_least();
		self.reset_top_data::<T>(candidate.clone(), &top_delegations);
		<TopDelegations<T>>::insert(candidate, top_delegations);
		Ok(true)
	}
	/// Increase bottom delegation
	pub fn increase_bottom_delegation<T: Config>(
		&mut self,
		candidate: &T::AccountId,
		delegator: T::AccountId,
		bond: BalanceOf<T>,
		more: BalanceOf<T>,
	) -> Result<bool, DispatchError>
	where
		BalanceOf<T>: Into<Balance> + From<Balance>,
	{
		let mut bottom_delegations =
			<BottomDelegations<T>>::get(candidate).ok_or(Error::<T>::CandidateDNE)?;
		let mut delegation_option: Option<Bond<T::AccountId, BalanceOf<T>>> = None;
		let in_top_after = if (bond.saturating_add(more)).into() > self.lowest_top_delegation_amount
		{
			// bump it from bottom
			bottom_delegations.delegations = bottom_delegations
				.delegations
				.clone()
				.into_iter()
				.filter(|d| {
					if d.owner != delegator {
						true
					} else {
						delegation_option = Some(Bond {
							owner: d.owner.clone(),
							amount: d.amount.saturating_add(more),
						});
						false
					}
				})
				.collect();
			let delegation = delegation_option.ok_or(Error::<T>::DelegationDNE)?;
			bottom_delegations.total = bottom_delegations.total.saturating_sub(bond);
			// add it to top
			let mut top_delegations = <TopDelegations<T>>::get(candidate)
				.expect("CandidateInfo existence => TopDelegations existence");
			// if top is full, pop lowest top
			if matches!(top_delegations.top_capacity::<T>(), CapacityStatus::Full) {
				// pop lowest top delegation
				let new_bottom_delegation = top_delegations
					.delegations
					.pop()
					.expect("Top capacity full => Exists at least 1 top delegation");
				top_delegations.total = top_delegations
					.total
					.saturating_sub(new_bottom_delegation.amount);
				bottom_delegations.insert_sorted_greatest_to_least(new_bottom_delegation);
			}
			// insert into top
			top_delegations.insert_sorted_greatest_to_least(delegation);
			self.reset_top_data::<T>(candidate.clone(), &top_delegations);
			<TopDelegations<T>>::insert(candidate, top_delegations);
			true
		} else {
			let mut in_bottom = false;
			// just increase the delegation
			bottom_delegations.delegations = bottom_delegations
				.delegations
				.clone()
				.into_iter()
				.map(|d| {
					if d.owner != delegator {
						d
					} else {
						in_bottom = true;
						Bond {
							owner: d.owner,
							amount: d.amount.saturating_add(more),
						}
					}
				})
				.collect();
			ensure!(in_bottom, Error::<T>::DelegationDNE);
			bottom_delegations.total = bottom_delegations.total.saturating_add(more);
			bottom_delegations.sort_greatest_to_least();
			false
		};
		self.reset_bottom_data::<T>(&bottom_delegations);
		<BottomDelegations<T>>::insert(candidate, bottom_delegations);
		Ok(in_top_after)
	}
	/// Decrease delegation
	pub fn decrease_delegation<T: Config>(
		&mut self,
		candidate: &T::AccountId,
		delegator: T::AccountId,
		bond: Balance,
		less: BalanceOf<T>,
	) -> Result<bool, DispatchError>
	where
		BalanceOf<T>: Into<Balance> + From<Balance>,
	{
		let lowest_top_eq_highest_bottom =
			self.lowest_top_delegation_amount == self.highest_bottom_delegation_amount;
		let bond_geq_lowest_top = bond >= self.lowest_top_delegation_amount;
		let delegation_dne_err: DispatchError = Error::<T>::DelegationDNE.into();
		if bond_geq_lowest_top && !lowest_top_eq_highest_bottom {
			// definitely in top
			self.decrease_top_delegation::<T>(candidate, delegator.clone(), bond.into(), less)
		} else if bond_geq_lowest_top && lowest_top_eq_highest_bottom {
			// update top but if error then update bottom (because could be in bottom because
			// lowest_top_eq_highest_bottom)
			let result =
				self.decrease_top_delegation::<T>(candidate, delegator.clone(), bond.into(), less);
			if result == Err(delegation_dne_err) {
				self.decrease_bottom_delegation::<T>(candidate, delegator, less)
			} else {
				result
			}
		} else {
			self.decrease_bottom_delegation::<T>(candidate, delegator, less)
		}
	}
	/// Decrease top delegation
	pub fn decrease_top_delegation<T: Config>(
		&mut self,
		candidate: &T::AccountId,
		delegator: T::AccountId,
		bond: BalanceOf<T>,
		less: BalanceOf<T>,
	) -> Result<bool, DispatchError>
	where
		BalanceOf<T>: Into<Balance> + From<Balance>,
	{
		// The delegation after the `decrease-delegation` will be strictly less than the
		// highest bottom delegation
		let bond_after_less_than_highest_bottom =
			bond.saturating_sub(less).into() < self.highest_bottom_delegation_amount;
		// The top delegations is full and the bottom delegations has at least one delegation
		let full_top_and_nonempty_bottom = matches!(self.top_capacity, CapacityStatus::Full)
			&& !matches!(self.bottom_capacity, CapacityStatus::Empty);
		let mut top_delegations =
			<TopDelegations<T>>::get(candidate).ok_or(Error::<T>::CandidateDNE)?;
		let in_top_after = if bond_after_less_than_highest_bottom && full_top_and_nonempty_bottom {
			let mut delegation_option: Option<Bond<T::AccountId, BalanceOf<T>>> = None;
			// take delegation from top
			top_delegations.delegations = top_delegations
				.delegations
				.clone()
				.into_iter()
				.filter(|d| {
					if d.owner != delegator {
						true
					} else {
						top_delegations.total = top_delegations.total.saturating_sub(d.amount);
						delegation_option = Some(Bond {
							owner: d.owner.clone(),
							amount: d.amount.saturating_sub(less),
						});
						false
					}
				})
				.collect();
			let delegation = delegation_option.ok_or(Error::<T>::DelegationDNE)?;
			// pop highest bottom by reverse and popping
			let mut bottom_delegations = <BottomDelegations<T>>::get(candidate)
				.expect("CandidateInfo existence => BottomDelegations existence");
			let highest_bottom_delegation = bottom_delegations.delegations.remove(0);
			bottom_delegations.total = bottom_delegations
				.total
				.saturating_sub(highest_bottom_delegation.amount);
			// insert highest bottom into top
			top_delegations.insert_sorted_greatest_to_least(highest_bottom_delegation);
			// insert previous top into bottom
			bottom_delegations.insert_sorted_greatest_to_least(delegation);
			self.reset_bottom_data::<T>(&bottom_delegations);
			<BottomDelegations<T>>::insert(candidate, bottom_delegations);
			false
		} else {
			// keep it in the top
			let mut is_in_top = false;
			top_delegations.delegations = top_delegations
				.delegations
				.clone()
				.into_iter()
				.map(|d| {
					if d.owner != delegator {
						d
					} else {
						is_in_top = true;
						Bond {
							owner: d.owner,
							amount: d.amount.saturating_sub(less),
						}
					}
				})
				.collect();
			ensure!(is_in_top, Error::<T>::DelegationDNE);
			top_delegations.total = top_delegations.total.saturating_sub(less);
			top_delegations.sort_greatest_to_least();
			true
		};
		self.reset_top_data::<T>(candidate.clone(), &top_delegations);
		<TopDelegations<T>>::insert(candidate, top_delegations);
		Ok(in_top_after)
	}
	/// Decrease bottom delegation
	pub fn decrease_bottom_delegation<T: Config>(
		&mut self,
		candidate: &T::AccountId,
		delegator: T::AccountId,
		less: BalanceOf<T>,
	) -> Result<bool, DispatchError>
	where
		BalanceOf<T>: Into<Balance>,
	{
		let mut bottom_delegations = <BottomDelegations<T>>::get(candidate)
			.expect("CandidateInfo exists => BottomDelegations exists");
		let mut in_bottom = false;
		bottom_delegations.delegations = bottom_delegations
			.delegations
			.clone()
			.into_iter()
			.map(|d| {
				if d.owner != delegator {
					d
				} else {
					in_bottom = true;
					Bond {
						owner: d.owner,
						amount: d.amount.saturating_sub(less),
					}
				}
			})
			.collect();
		ensure!(in_bottom, Error::<T>::DelegationDNE);
		bottom_delegations.sort_greatest_to_least();
		self.reset_bottom_data::<T>(&bottom_delegations);
		<BottomDelegations<T>>::insert(candidate, bottom_delegations);
		Ok(false)
	}
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
	pub fn is_active(&self) -> bool {
		self.state == CollatorStatus::Active
	}
}

impl<A: Clone, B: Copy> From<CollatorCandidate<A, B>> for CollatorSnapshot<A, B> {
	fn from(other: CollatorCandidate<A, B>) -> CollatorSnapshot<A, B> {
		CollatorSnapshot {
			bond: other.bond,
			delegations: other
				.top_delegations
				.into_iter()
				.map(|d| BondWithAutoCompound {
					owner: d.owner,
					amount: d.amount,
					auto_compound: Percent::zero(),
				})
				.collect(),
			total: other.total_counted,
		}
	}
}

#[allow(deprecated)]
#[derive(Clone, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum DelegatorStatus {
	/// Active with no scheduled exit
	Active,
	/// Schedule exit to revoke all ongoing delegations
	#[deprecated(note = "must only be used for backwards compatibility reasons")]
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
	/// Sum of pending revocation amounts + bond less amounts
	pub less_total: Balance,
	/// Status for this delegator
	pub status: DelegatorStatus,
}

// Temporary manual implementation for migration testing purposes
impl<A: PartialEq, B: PartialEq> PartialEq for Delegator<A, B> {
	fn eq(&self, other: &Self) -> bool {
		let must_be_true = self.id == other.id
			&& self.total == other.total
			&& self.less_total == other.less_total
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
		AccountId: Ord + Clone,
		Balance: Copy
			+ sp_std::ops::AddAssign
			+ sp_std::ops::Add<Output = Balance>
			+ sp_std::ops::SubAssign
			+ sp_std::ops::Sub<Output = Balance>
			+ Ord
			+ Zero
			+ Default
			+ Saturating,
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
			less_total: Balance::zero(),
			status: DelegatorStatus::Active,
		}
	}

	pub fn default_with_total(id: AccountId, amount: Balance) -> Self {
		Delegator {
			id,
			total: amount,
			delegations: OrderedSet::from(vec![]),
			less_total: Balance::zero(),
			status: DelegatorStatus::Active,
		}
	}

	pub fn total(&self) -> Balance {
		self.total
	}

	pub fn total_sub_if<T, F>(&mut self, amount: Balance, check: F) -> DispatchResult
	where
		T: Config,
		T::AccountId: From<AccountId>,
		BalanceOf<T>: From<Balance>,
		F: Fn(Balance) -> DispatchResult,
	{
		let total = self.total.saturating_sub(amount);
		check(total)?;
		self.total = total;
		self.adjust_bond_lock::<T>(BondAdjust::Decrease)?;
		Ok(())
	}

	pub fn total_add<T, F>(&mut self, amount: Balance) -> DispatchResult
	where
		T: Config,
		T::AccountId: From<AccountId>,
		BalanceOf<T>: From<Balance>,
	{
		self.total = self.total.saturating_add(amount);
		self.adjust_bond_lock::<T>(BondAdjust::Increase(amount))?;
		Ok(())
	}

	pub fn total_sub<T>(&mut self, amount: Balance) -> DispatchResult
	where
		T: Config,
		T::AccountId: From<AccountId>,
		BalanceOf<T>: From<Balance>,
	{
		self.total = self.total.saturating_sub(amount);
		self.adjust_bond_lock::<T>(BondAdjust::Decrease)?;
		Ok(())
	}

	pub fn is_active(&self) -> bool {
		matches!(self.status, DelegatorStatus::Active)
	}

	pub fn add_delegation(&mut self, bond: Bond<AccountId, Balance>) -> bool {
		let amt = bond.amount;
		if self.delegations.insert(bond) {
			self.total = self.total.saturating_add(amt);
			true
		} else {
			false
		}
	}
	// Return Some(remaining balance), must be more than MinDelegation
	// Return None if delegation not found
	pub fn rm_delegation<T: Config>(&mut self, collator: &AccountId) -> Option<Balance>
	where
		BalanceOf<T>: From<Balance>,
		T::AccountId: From<AccountId>,
	{
		let mut amt: Option<Balance> = None;
		let delegations = self
			.delegations
			.0
			.iter()
			.filter_map(|x| {
				if &x.owner == collator {
					amt = Some(x.amount);
					None
				} else {
					Some(x.clone())
				}
			})
			.collect();
		if let Some(balance) = amt {
			self.delegations = OrderedSet::from(delegations);
			self.total_sub::<T>(balance)
				.expect("Decreasing lock cannot fail, qed");
			Some(self.total)
		} else {
			None
		}
	}

	/// Increases the delegation amount and returns `true` if the delegation is part of the
	/// TopDelegations set, `false` otherwise.
	pub fn increase_delegation<T: Config>(
		&mut self,
		candidate: AccountId,
		amount: Balance,
	) -> Result<bool, sp_runtime::DispatchError>
	where
		BalanceOf<T>: From<Balance>,
		T::AccountId: From<AccountId>,
		Delegator<T::AccountId, BalanceOf<T>>: From<Delegator<AccountId, Balance>>,
	{
		let delegator_id: T::AccountId = self.id.clone().into();
		let candidate_id: T::AccountId = candidate.clone().into();
		let balance_amt: BalanceOf<T> = amount.into();
		// increase delegation
		for x in &mut self.delegations.0 {
			if x.owner == candidate {
				let before_amount: BalanceOf<T> = x.amount.into();
				x.amount = x.amount.saturating_add(amount);
				self.total = self.total.saturating_add(amount);
				self.adjust_bond_lock::<T>(BondAdjust::Increase(amount))?;

				// update collator state delegation
				let mut collator_state =
					<CandidateInfo<T>>::get(&candidate_id).ok_or(Error::<T>::CandidateDNE)?;
				let before = collator_state.total_counted;
				let in_top = collator_state.increase_delegation::<T>(
					&candidate_id,
					delegator_id.clone(),
					before_amount,
					balance_amt,
				)?;
				let after = collator_state.total_counted;
				if collator_state.is_active() && (before != after) {
					Pallet::<T>::update_active(candidate_id.clone(), after);
				}
				<CandidateInfo<T>>::insert(&candidate_id, collator_state);
				let new_total_staked = <Total<T>>::get().saturating_add(balance_amt);
				<Total<T>>::put(new_total_staked);
				let nom_st: Delegator<T::AccountId, BalanceOf<T>> = self.clone().into();
				<DelegatorState<T>>::insert(&delegator_id, nom_st);
				return Ok(in_top);
			}
		}
		Err(Error::<T>::DelegationDNE.into())
	}

	/// Updates the bond locks for this delegator.
	///
	/// This will take the current self.total and ensure that a lock of the same amount is applied
	/// and when increasing the bond lock will also ensure that the account has enough free balance.
	///
	/// `additional_required_balance` should reflect the change to the amount that should be locked if
	/// positive, 0 otherwise (e.g. `min(0, change_in_total_bond)`). This is necessary because it is
	/// not possible to query the amount that is locked for a given lock id.
	pub fn adjust_bond_lock<T: Config>(
		&mut self,
		additional_required_balance: BondAdjust<Balance>,
	) -> DispatchResult
	where
		BalanceOf<T>: From<Balance>,
		T::AccountId: From<AccountId>,
	{
		match additional_required_balance {
			BondAdjust::Increase(amount) => {
				ensure!(
					<Pallet<T>>::get_delegator_stakable_balance(&self.id.clone().into())
						>= amount.into(),
					Error::<T>::InsufficientBalance,
				);

				// additional sanity check: shouldn't ever want to lock more than total
				if amount > self.total {
					log::warn!("LOGIC ERROR: request to reserve more than bond total");
					return Err(DispatchError::Other("Invalid additional_required_balance"));
				}
			}
			BondAdjust::Decrease => (), // do nothing on decrease
		};

		if self.total.is_zero() {
			T::Currency::remove_lock(DELEGATOR_LOCK_ID, &self.id.clone().into());
		} else {
			T::Currency::set_lock(
				DELEGATOR_LOCK_ID,
				&self.id.clone().into(),
				self.total.into(),
				WithdrawReasons::all(),
			);
		}
		Ok(())
	}

	/// Retrieves the bond amount that a delegator has provided towards a collator.
	/// Returns `None` if missing.
	pub fn get_bond_amount(&self, collator: &AccountId) -> Option<Balance> {
		self.delegations
			.0
			.iter()
			.find(|b| &b.owner == collator)
			.map(|b| b.amount)
	}
}

pub mod deprecated {
	#![allow(deprecated)]

	use super::*;

	#[deprecated(note = "use DelegationAction")]
	#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Changes requested by the delegator
	/// - limit of 1 ongoing change per delegation
	pub enum DelegationChange {
		Revoke,
		Decrease,
	}

	#[deprecated(note = "use ScheduledRequest")]
	#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
	pub struct DelegationRequest<AccountId, Balance> {
		pub collator: AccountId,
		pub amount: Balance,
		pub when_executable: RoundIndex,
		pub action: DelegationChange,
	}

	#[deprecated(note = "use DelegationScheduledRequests storage item")]
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
				+ sp_std::ops::Sub<Output = B>
				+ Saturating,
		> PendingDelegationRequests<A, B>
	{
		/// New default (empty) pending requests
		pub fn new() -> Self {
			Self::default()
		}
	}

	#[deprecated(note = "use new crate::types::Delegator struct")]
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

	// CollatorSnapshot

	#[deprecated(note = "use CollatorSnapshot with BondWithAutoCompound delegations")]
	#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
	/// Snapshot of collator state at the start of the round for which they are selected
	pub struct CollatorSnapshot<AccountId, Balance> {
		/// The total value locked by the collator.
		pub bond: Balance,

		/// The rewardable delegations. This list is a subset of total delegators, where certain
		/// delegators are adjusted based on their scheduled
		/// [DelegationChange::Revoke] or [DelegationChange::Decrease] action.
		pub delegations: Vec<Bond<AccountId, Balance>>,

		/// The total counted value locked for the collator, including the self bond + total staked by
		/// top delegators.
		pub total: Balance,
	}

	impl<A: PartialEq, B: PartialEq> PartialEq for CollatorSnapshot<A, B> {
		fn eq(&self, other: &Self) -> bool {
			let must_be_true = self.bond == other.bond && self.total == other.total;
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
			) in self.delegations.iter().zip(other.delegations.iter())
			{
				if o1 != o2 || a1 != a2 {
					return false;
				}
			}
			true
		}
	}

	impl<A, B: Default> Default for CollatorSnapshot<A, B> {
		fn default() -> CollatorSnapshot<A, B> {
			CollatorSnapshot {
				bond: B::default(),
				delegations: Vec::new(),
				total: B::default(),
			}
		}
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

// /// Temporary function to migrate state
// pub(crate) fn migrate_nominator_to_delegator_state<T: Config>(
// 	id: T::AccountId,
// 	nominator: Nominator2<T::AccountId, BalanceOf<T>>,
// ) -> Delegator<T::AccountId, BalanceOf<T>> {
// 	Delegator {
// 		id,
// 		delegations: nominator.delegations,
// 		total: nominator.total,
// 		requests: PendingDelegationRequests::new(),
// 		status: nominator.status,
// 	}
// }

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
/// The current round index and transition information
pub struct RoundInfo<BlockNumber> {
	/// Current round index
	pub current: RoundIndex,
	/// The first block of the current round
	pub first: BlockNumber,
	/// The length of the current round in number of blocks
	pub length: u32,
	/// The first slot of the current round
	pub first_slot: u64,
}
impl<
		B: Copy + sp_std::ops::Add<Output = B> + sp_std::ops::Sub<Output = B> + From<u32> + PartialOrd,
	> RoundInfo<B>
{
	pub fn new(current: RoundIndex, first: B, length: u32, first_slot: u64) -> RoundInfo<B> {
		RoundInfo {
			current,
			first,
			length,
			first_slot,
		}
	}
	/// Check if the round should be updated
	pub fn should_update(&self, now: B) -> bool {
		now - self.first >= self.length.into()
	}
	/// New round
	pub fn update(&mut self, now: B, now_slot: u64) {
		self.current = self.current.saturating_add(1u32);
		self.first = now;
		self.first_slot = now_slot;
	}
}
impl<
		B: Copy + sp_std::ops::Add<Output = B> + sp_std::ops::Sub<Output = B> + From<u32> + PartialOrd,
	> Default for RoundInfo<B>
{
	fn default() -> RoundInfo<B> {
		RoundInfo::new(1u32, 1u32.into(), 20u32, 0)
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum InflationDistributionConfigId {
	ParachainBondReserve,
	Treasury,
	Other([u8; 32]),
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
/// Reserve information { account, percent_of_inflation }
pub struct InflationDistributionConfig<AccountId> {
	/// Account which receives funds intended for parachain bond
	pub account: AccountId,
	/// Percent of inflation set aside for parachain bond account
	pub percent: Percent,
}

pub enum BondAdjust<Balance> {
	Increase(Balance),
	Decrease,
}
