// Copyright 2019-2020 PureStake Inc.
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

//! # Stake
//! Minimal staking pallet that implements ordered validator selection by total amount at stake
//!
//! ### Rules
//! There is a new round every `BlocksPerRound` blocks.
//!
//! At the start of every round,
//! * `IssuancePerRound` is distributed to validators for `BondDuration` rounds ago
//! in proportion to the points they received in that round (for authoring blocks)
//! * queued validator exits are executed
//! * a new set of validators is chosen from the candidates
//!
//! To join the set of candidates, an account must call `join_candidates` with
//! stake >= `MinValidatorStk` and fee <= `MaxFee`. The fee is taken off the top
//! of any rewards for the validator before the remaining rewards are distributed
//! in proportion to stake to all nominators (including the validator, who always
//! self-nominates).
//!
//! To leave the set of candidates, the validator calls `leave_candidates`. If the call succeeds,
//! the validator is removed from the pool of candidates so they cannot be selected for future
//! validator sets, but they are not unstaked until `BondDuration` rounds later. The exit request is
//! stored in the `ExitQueue` and processed `BondDuration` rounds later to unstake the validator
//! and all of its nominators.
//!
//! To join the set of nominators, an account must call `join_nominators` with
//! stake >= `MinNominatorStk`. There are also runtime methods for nominating additional validators
//! and revoking nominations.

#![recursion_limit = "256"]
#![cfg_attr(not(feature = "std"), no_std)]

mod set;
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure,
	storage::IterableStorageDoubleMap,
	traits::{Currency, Get, Imbalance, ReservableCurrency},
};
use frame_system::{ensure_signed, Config as System};
use pallet_staking::{Exposure, IndividualExposure};
use parity_scale_codec::{Decode, Encode, HasCompact};
use set::OrderedSet;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Zero},
	DispatchResult, Perbill, RuntimeDebug,
};
use sp_std::{cmp::Ordering, prelude::*};
#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

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

impl<A, B: HasCompact> Into<IndividualExposure<A, B>> for Bond<A, B> {
	fn into(self) -> IndividualExposure<A, B> {
		IndividualExposure {
			who: self.owner,
			value: self.amount,
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
/// The activity status of the validator
pub enum ValidatorStatus<BlockNumber> {
	/// Committed to be online and producing valid blocks (not equivocating)
	Active,
	/// Temporarily inactive and excused for inactivity
	Idle,
	/// Bonded until the wrapped block
	Leaving(BlockNumber),
}

impl<B> Default for ValidatorStatus<B> {
	fn default() -> ValidatorStatus<B> {
		ValidatorStatus::Active
	}
}

#[derive(Encode, Decode, RuntimeDebug)]
pub struct Validator<AccountId, Balance> {
	pub id: AccountId,
	pub fee: Perbill,
	pub bond: Balance,
	pub nominators: OrderedSet<Bond<AccountId, Balance>>,
	pub total: Balance,
	pub state: ValidatorStatus<RoundIndex>,
}

impl<
		A: Ord + Clone,
		B: AtLeast32BitUnsigned + Ord + Copy + sp_std::ops::AddAssign + sp_std::ops::SubAssign,
	> Validator<A, B>
{
	pub fn new(id: A, fee: Perbill, bond: B) -> Self {
		let total = bond;
		Validator {
			id,
			fee,
			bond,
			nominators: OrderedSet::new(),
			total,
			state: ValidatorStatus::default(), // default active
		}
	}
	pub fn is_active(&self) -> bool {
		self.state == ValidatorStatus::Active
	}
	pub fn is_leaving(&self) -> bool {
		if let ValidatorStatus::Leaving(_) = self.state {
			true
		} else {
			false
		}
	}
	pub fn bond_more(&mut self, more: B) {
		self.bond += more;
		self.total += more;
	}
	// Returns None if underflow or less == self.bond (in which case validator should leave instead)
	pub fn bond_less(&mut self, less: B) -> Option<B> {
		if self.bond > less {
			self.bond -= less;
			self.total -= less;
			Some(self.bond)
		} else {
			None
		}
	}
	// infallible so nominator must exist before calling
	pub fn rm_nominator(&mut self, nominator: A) -> B {
		let mut total = self.total;
		let nominators = self
			.nominators
			.0
			.iter()
			.filter_map(|x| {
				if x.owner == nominator {
					total -= x.amount;
					None
				} else {
					Some(x.clone())
				}
			})
			.collect();
		self.nominators = OrderedSet::from(nominators);
		self.total = total;
		total
	}
	// infallible so nominator dne before calling
	pub fn add_nominator(&mut self, owner: A, amount: B) -> B {
		self.nominators.insert(Bond { owner, amount });
		self.total += amount;
		self.total
	}
	// only call with an amount larger than existing amount
	pub fn update_nominator(&mut self, nominator: A, amount: B) -> B {
		let mut difference: B = 0u32.into();
		let nominators = self
			.nominators
			.0
			.iter()
			.filter_map(|x| {
				if x.owner == nominator {
					// new amount must be greater or will underflow
					difference = amount - x.amount;
					Some(Bond {
						owner: x.owner.clone(),
						amount,
					})
				} else {
					Some(x.clone())
				}
			})
			.collect();
		self.nominators = OrderedSet::from(nominators);
		self.total += difference;
		self.total
	}
	pub fn inc_nominator(&mut self, nominator: A, more: B) {
		for x in &mut self.nominators.0 {
			if x.owner == nominator {
				x.amount += more;
				self.total += more;
				return;
			}
		}
	}
	pub fn dec_nominator(&mut self, nominator: A, less: B) {
		for x in &mut self.nominators.0 {
			if x.owner == nominator {
				x.amount -= less;
				self.total -= less;
				return;
			}
		}
	}
	pub fn go_offline(&mut self) {
		self.state = ValidatorStatus::Idle;
	}
	pub fn go_online(&mut self) {
		self.state = ValidatorStatus::Active;
	}
	pub fn leave_candidates(&mut self, block: RoundIndex) {
		self.state = ValidatorStatus::Leaving(block);
	}
}

impl<A: PartialEq, B: HasCompact + Zero> Into<Exposure<A, B>> for Validator<A, B> {
	fn into(self) -> Exposure<A, B> {
		Exposure {
			total: self.total,
			own: self.bond,
			others: self.nominators.0.into_iter().map(|x| x.into()).collect(),
		}
	}
}

#[derive(Encode, Decode, RuntimeDebug)]
pub struct Nominator<AccountId, Balance> {
	pub nominations: OrderedSet<Bond<AccountId, Balance>>,
	pub total: Balance,
}

impl<
		AccountId: Ord + Clone,
		Balance: Copy
			+ sp_std::ops::AddAssign
			+ sp_std::ops::Add<Output = Balance>
			+ sp_std::ops::SubAssign
			+ PartialOrd,
	> Nominator<AccountId, Balance>
{
	pub fn new(validator: AccountId, nomination: Balance) -> Self {
		Nominator {
			nominations: OrderedSet::from(vec![Bond {
				owner: validator.clone(),
				amount: nomination,
			}]),
			total: nomination,
		}
	}
	pub fn add_nomination(&mut self, bond: Bond<AccountId, Balance>) -> bool {
		let amt = bond.amount;
		if self.nominations.insert(bond) {
			self.total += amt;
			true
		} else {
			false
		}
	}
	// Returns Some(remaining balance), must be more than MinNominatorStk
	// Returns None if nomination not found
	pub fn rm_nomination(&mut self, validator: AccountId) -> Option<Balance> {
		let mut amt: Option<Balance> = None;
		let nominations = self
			.nominations
			.0
			.iter()
			.filter_map(|x| {
				if x.owner == validator {
					amt = Some(x.amount);
					None
				} else {
					Some(x.clone())
				}
			})
			.collect();
		if let Some(balance) = amt {
			self.nominations = OrderedSet::from(nominations);
			self.total -= balance;
			Some(self.total)
		} else {
			None
		}
	}
	// Returns Some(new balances) if old was nominated and None if it wasn't nominated
	pub fn swap_nomination(
		&mut self,
		old: AccountId,
		new: AccountId,
	) -> Option<(Balance, Balance)> {
		let mut amt: Option<Balance> = None;
		let nominations = self
			.nominations
			.0
			.iter()
			.filter_map(|x| {
				if x.owner == old {
					amt = Some(x.amount);
					None
				} else {
					Some(x.clone())
				}
			})
			.collect();
		if let Some(swapped_amt) = amt {
			let mut old_new_amt: Option<Balance> = None;
			let nominations2 = self
				.nominations
				.0
				.iter()
				.filter_map(|x| {
					if x.owner == new {
						old_new_amt = Some(x.amount);
						None
					} else {
						Some(x.clone())
					}
				})
				.collect();
			let new_amount = if let Some(old_amt) = old_new_amt {
				// update existing nomination
				self.nominations = OrderedSet::from(nominations2);
				let new_amt = old_amt + swapped_amt;
				self.nominations.insert(Bond {
					owner: new,
					amount: new_amt,
				});
				new_amt
			} else {
				// insert completely new nomination
				self.nominations = OrderedSet::from(nominations);
				self.nominations.insert(Bond {
					owner: new,
					amount: swapped_amt,
				});
				swapped_amt
			};
			Some((swapped_amt, new_amount))
		} else {
			return None;
		}
	}
	// Returns None if nomination not found
	pub fn inc_nomination(&mut self, validator: AccountId, more: Balance) -> Option<Balance> {
		for x in &mut self.nominations.0 {
			if x.owner == validator {
				x.amount += more;
				self.total += more;
				return Some(x.amount);
			}
		}
		None
	}
	// Returns Some(Some(balance)) if successful
	// None if nomination not found
	// Some(None) if underflow
	pub fn dec_nomination(
		&mut self,
		validator: AccountId,
		less: Balance,
	) -> Option<Option<Balance>> {
		for x in &mut self.nominations.0 {
			if x.owner == validator {
				if x.amount > less {
					x.amount -= less;
					self.total -= less;
					return Some(Some(x.amount));
				} else {
					// underflow error; should rm entire nomination if x.amount == validator
					return Some(None);
				}
			}
		}
		None
	}
}

type RoundIndex = u32;
type RewardPoint = u32;
type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as System>::AccountId>>::Balance;
type Candidate<T> = Validator<<T as System>::AccountId, BalanceOf<T>>;

pub trait Config: System {
	/// The overarching event type
	type Event: From<Event<Self>> + Into<<Self as System>::Event>;
	/// The currency type
	type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	/// Blocks per round
	type BlocksPerRound: Get<Self::BlockNumber>;
	/// Number of rounds that validators remain bonded before exit request is executed
	type BondDuration: Get<RoundIndex>;
	/// Maximum validators per round
	type MaxValidators: Get<u32>;
	/// Maximum nominators per validator
	type MaxNominatorsPerValidator: Get<usize>;
	/// Maximum validators per nominator
	type MaxValidatorsPerNominator: Get<usize>;
	/// Balance issued as rewards per round (constant issuance)
	type IssuancePerRound: Get<BalanceOf<Self>>;
	/// Maximum fee for any validator
	type MaxFee: Get<Perbill>;
	/// Minimum stake for any registered on-chain account to become a validator
	type MinValidatorStk: Get<BalanceOf<Self>>;
	/// Minimum stake for any registered on-chain account to nominate
	type MinNomination: Get<BalanceOf<Self>>;
	/// Minimum stake for any registered on-chain account to become a nominator
	type MinNominatorStk: Get<BalanceOf<Self>>;
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as System>::AccountId,
		Balance = BalanceOf<T>,
		BlockNumber = <T as System>::BlockNumber,
	{
		/// Starting Block, Round, Number of Validators, Total Balance
		NewRound(BlockNumber, RoundIndex, u32, Balance),
		/// Account, Amount Locked, New Total Amt Locked
		JoinedValidatorCandidates(AccountId, Balance, Balance),
		/// Round, Validator Account, Total Exposed Amount (includes all nominations)
		ValidatorChosen(RoundIndex, AccountId, Balance),
		/// Validator Account, Old Bond, New Bond
		ValidatorBondedMore(AccountId, Balance, Balance),
		/// Validator Account, Old Bond, New Bond
		ValidatorBondedLess(AccountId, Balance, Balance),
		ValidatorWentOffline(RoundIndex, AccountId),
		ValidatorBackOnline(RoundIndex, AccountId),
		/// Round, Validator Account, Scheduled Exit
		ValidatorScheduledExit(RoundIndex, AccountId, RoundIndex),
		/// Account, Amount Unlocked, New Total Amt Locked
		ValidatorLeft(AccountId, Balance, Balance),
		// Nominator, Validator, Old Nomination, New Nomination
		NominationIncreased(AccountId, AccountId, Balance, Balance),
		// Nominator, Validator, Old Nomination, New Nomination
		NominationDecreased(AccountId, AccountId, Balance, Balance),
		// Nominator, Swapped Amount, Old Nominator, New Nominator
		NominationSwapped(AccountId, Balance, AccountId, AccountId),
		/// Nominator, Amount Staked
		NominatorJoined(AccountId, Balance),
		/// Nominator, Amount Unstaked
		NominatorLeft(AccountId, Balance),
		/// Nominator, Amount Locked, Validator, New Total Amt Locked
		ValidatorNominated(AccountId, Balance, AccountId, Balance),
		/// Nominator, Validator, Amount Unstaked, New Total Amt Staked for Validator
		NominatorLeftValidator(AccountId, AccountId, Balance, Balance),
		Rewarded(AccountId, Balance),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		// Nominator Does Not Exist
		NominatorDNE,
		CandidateDNE,
		ValidatorDNE,
		NominatorExists,
		CandidateExists,
		ValidatorExists,
		FeeOverMax,
		ValBondBelowMin,
		NomBondBelowMin,
		NominationBelowMin,
		AlreadyOffline,
		AlreadyActive,
		AlreadyLeaving,
		TooManyNominators,
		CannotActivateIfLeaving,
		ExceedMaxValidatorsPerNom,
		AlreadyNominatedValidator,
		MustNominateAtLeastOne,
		NominationDNE,
		Underflow,
	}
}

decl_storage! {
	trait Store for Module<T: Config> as Stake {
		/// Current round, incremented every `BlocksPerRound` in `fn on_finalize`
		Round: RoundIndex;
		/// Current nominators with their validator
		Nominators: map
			hasher(blake2_128_concat) T::AccountId => Option<Nominator<T::AccountId, BalanceOf<T>>>;
		/// Current candidates with associated state
		Candidates: map hasher(blake2_128_concat) T::AccountId => Option<Candidate<T>>;
		/// Current validator set
		Validators: Vec<T::AccountId>;
		/// Total Locked
		Total: BalanceOf<T>;
		/// Pool of candidates, ordered by account id
		CandidatePool: OrderedSet<Bond<T::AccountId,BalanceOf<T>>>;
		/// Queue of validator exit requests, ordered by account id
		ExitQueue: OrderedSet<Bond<T::AccountId,RoundIndex>>;
		/// Exposure at stake per round, per validator
		AtStake: double_map
			hasher(blake2_128_concat) RoundIndex,
			hasher(blake2_128_concat) T::AccountId => Exposure<T::AccountId,BalanceOf<T>>;
		/// Total points awarded in this round
		Points: map
			hasher(blake2_128_concat) RoundIndex => RewardPoint;
		/// Individual points accrued each round per validator
		AwardedPts: double_map
			hasher(blake2_128_concat) RoundIndex,
			hasher(blake2_128_concat) T::AccountId => RewardPoint;
	}
	add_extra_genesis {
		config(stakers):
			Vec<(T::AccountId,Option<T::AccountId>,BalanceOf<T>)>;
		build(|config: &GenesisConfig<T>| {
			for &(ref actor, ref opt_val, balance) in &config.stakers {
				assert!(
					T::Currency::free_balance(&actor) >= balance,
					"Stash does not have enough balance to bond."
				);
				let _ = if let Some(nominated_val) = opt_val {
					<Module<T>>::join_nominators(
						T::Origin::from(Some(actor.clone()).into()),
						nominated_val.clone(),
						balance,
					)
				} else {
					<Module<T>>::join_candidates(
						T::Origin::from(Some(actor.clone()).into()),
						Perbill::from_percent(2),// default fee for validators set at genesis is 2%
						balance,
					)
				};
			}
			let (v_count, total_staked) = <Module<T>>::best_candidates_become_validators(1u32);
			// start Round 1 at Block 0
			<Round>::put(1u32);
			<Module<T>>::deposit_event(
				RawEvent::NewRound(T::BlockNumber::zero(), 1u32, v_count, total_staked)
			);
		});
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		#[weight = 0]
		fn join_candidates(
			origin,
			fee: Perbill,
			bond: BalanceOf<T>,
		) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			ensure!(!Self::is_candidate(&acc),Error::<T>::CandidateExists);
			ensure!(!Self::is_nominator(&acc),Error::<T>::NominatorExists);
			ensure!(fee <= T::MaxFee::get(),Error::<T>::FeeOverMax);
			ensure!(bond >= T::MinValidatorStk::get(),Error::<T>::ValBondBelowMin);
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidates.insert(Bond{owner: acc.clone(), amount: bond}),
				Error::<T>::CandidateExists
			);
			T::Currency::reserve(&acc,bond)?;
			let candidate: Candidate<T> = Validator::new(acc.clone(),fee,bond);
			let new_total = <Total<T>>::get() + bond;
			<Total<T>>::put(new_total);
			<Candidates<T>>::insert(&acc,candidate);
			<CandidatePool<T>>::put(candidates);
			Self::deposit_event(RawEvent::JoinedValidatorCandidates(acc,bond,new_total));
			Ok(())
		}
		#[weight = 0]
		fn leave_candidates(origin) -> DispatchResult {
			let validator = ensure_signed(origin)?;
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(),Error::<T>::AlreadyLeaving);
			let mut exits = <ExitQueue<T>>::get();
			let now = <Round>::get();
			let when = now + T::BondDuration::get();
			ensure!(
				exits.insert(Bond{owner:validator.clone(),amount:when}),
				Error::<T>::AlreadyLeaving
			);
			state.leave_candidates(when);
			let mut candidates = <CandidatePool<T>>::get();
			if candidates.remove(&Bond::from_owner(validator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<ExitQueue<T>>::put(exits);
			<Candidates<T>>::insert(&validator,state);
			Self::deposit_event(RawEvent::ValidatorScheduledExit(now,validator,when));
			Ok(())
		}
		#[weight = 0]
		fn go_offline(origin) -> DispatchResult {
			let validator = ensure_signed(origin)?;
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(state.is_active(),Error::<T>::AlreadyOffline);
			state.go_offline();
			let mut candidates = <CandidatePool<T>>::get();
			if candidates.remove(&Bond::from_owner(validator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<Candidates<T>>::insert(&validator,state);
			Self::deposit_event(RawEvent::ValidatorWentOffline(<Round>::get(),validator));
			Ok(())
		}
		#[weight = 0]
		fn go_online(origin) -> DispatchResult {
			let validator = ensure_signed(origin)?;
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_active(),Error::<T>::AlreadyActive);
			ensure!(!state.is_leaving(),Error::<T>::CannotActivateIfLeaving);
			state.go_online();
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidates.insert(Bond{owner:validator.clone(),amount:state.total}),
				Error::<T>::AlreadyActive
			);
			<CandidatePool<T>>::put(candidates);
			<Candidates<T>>::insert(&validator,state);
			Self::deposit_event(RawEvent::ValidatorBackOnline(<Round>::get(),validator));
			Ok(())
		}
		#[weight = 0]
		fn candidate_bond_more(origin, more: BalanceOf<T>) -> DispatchResult {
			let validator = ensure_signed(origin)?;
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(),Error::<T>::CannotActivateIfLeaving);
			T::Currency::reserve(&validator, more)?;
			let before = state.bond;
			state.bond_more(more);
			let after = state.bond;
			if state.is_active() {
				Self::update_active(validator.clone(), state.total);
			}
			<Candidates<T>>::insert(&validator,state);
			Self::deposit_event(RawEvent::ValidatorBondedMore(validator, before, after));
			Ok(())
		}
		#[weight = 0]
		fn candidate_bond_less(origin, less: BalanceOf<T>) -> DispatchResult {
			let validator = ensure_signed(origin)?;
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(),Error::<T>::CannotActivateIfLeaving);
			let before = state.bond;
			let after = state.bond_less(less).ok_or(Error::<T>::Underflow)?;
			ensure!(after >= T::MinValidatorStk::get(), Error::<T>::ValBondBelowMin);
			T::Currency::unreserve(&validator, less);
			if state.is_active() {
				Self::update_active(validator.clone(), state.total);
			}
			<Candidates<T>>::insert(&validator, state);
			Self::deposit_event(RawEvent::ValidatorBondedLess(validator, before, after));
			Ok(())
		}
		#[weight = 0]
		fn join_nominators(
			origin,
			validator: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			ensure!(amount >= T::MinNominatorStk::get(), Error::<T>::NomBondBelowMin);
			ensure!(!Self::is_nominator(&acc),Error::<T>::NominatorExists);
			ensure!(!Self::is_candidate(&acc),Error::<T>::CandidateExists);
			Self::nominator_joins_validator(acc.clone(), amount, validator.clone())?;
			<Nominators<T>>::insert(&acc, Nominator::new(validator.clone(),amount));
			Self::deposit_event(RawEvent::NominatorJoined(acc,amount));
			Ok(())
		}
		#[weight = 0]
		fn leave_nominators(origin) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			let nominator = <Nominators<T>>::get(&acc).ok_or(Error::<T>::NominatorDNE)?;
			for bond in nominator.nominations.0 {
				Self::nominator_leaves_validator(acc.clone(), bond.owner.clone())?;
			}
			<Nominators<T>>::remove(&acc);
			Self::deposit_event(RawEvent::NominatorLeft(acc.clone(), nominator.total));
			Ok(())
		}
		#[weight = 0]
		fn nominate_new(
			origin,
			validator: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			ensure!(amount >= T::MinNomination::get(), Error::<T>::NominationBelowMin);
			let mut nominator = <Nominators<T>>::get(&acc).ok_or(Error::<T>::NominatorDNE)?;
			ensure!(
				nominator.nominations.0.len() < T::MaxValidatorsPerNominator::get(),
				Error::<T>::ExceedMaxValidatorsPerNom
			);
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(
				nominator.add_nomination(Bond{owner:validator.clone(), amount}),
				Error::<T>::AlreadyNominatedValidator
			);
			let nomination = Bond {
				owner: acc.clone(),
				amount,
			};
			ensure!(
				state.nominators.0.len() < T::MaxNominatorsPerValidator::get(),
				Error::<T>::TooManyNominators
			);
			ensure!(
				state.nominators.insert(nomination),
				Error::<T>::NominatorExists
			);
			T::Currency::reserve(&acc, amount)?;
			let new_total = state.total + amount;
			if state.is_active() {
				Self::update_active(validator.clone(), new_total);
			}
			let new_total_locked = <Total<T>>::get() + amount;
			<Total<T>>::put(new_total_locked);
			state.total = new_total;
			<Candidates<T>>::insert(&validator, state);
			<Nominators<T>>::insert(&acc, nominator);
			Self::deposit_event(RawEvent::ValidatorNominated(
				acc, amount, validator, new_total,
			));
			Ok(())
		}
		#[weight = 0]
		fn switch_nomination(origin, old: T::AccountId, new: T::AccountId) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			let mut nominator = <Nominators<T>>::get(&acc).ok_or(Error::<T>::NominatorDNE)?;
			let mut old_validator = <Candidates<T>>::get(&old).ok_or(Error::<T>::CandidateDNE)?;
			let mut new_validator = <Candidates<T>>::get(&new).ok_or(Error::<T>::CandidateDNE)?;
			let (swapped_amt, new_amt) = nominator
				.swap_nomination(old.clone(), new.clone())
				.ok_or(Error::<T>::NominationDNE)?;
			let (new_old, new_new) = if new_amt > swapped_amt {
				(old_validator.rm_nominator(acc.clone()), new_validator.update_nominator(acc.clone(), new_amt))
			} else {
				(old_validator.rm_nominator(acc.clone()), new_validator.add_nominator(acc.clone(), swapped_amt))
			};
			if old_validator.is_active() {
				Self::update_active(old.clone(), new_old);
			}
			if new_validator.is_active() {
				Self::update_active(new.clone(), new_new);
			}
			<Candidates<T>>::insert(&old, old_validator);
			<Candidates<T>>::insert(&new, new_validator);
			<Nominators<T>>::insert(&acc, nominator);
			Self::deposit_event(RawEvent::NominationSwapped(acc, swapped_amt, old, new));
			Ok(())
		}
		#[weight = 0]
		fn revoke_nomination(origin, validator: T::AccountId) -> DispatchResult {
			Self::nominator_revokes_validator(ensure_signed(origin)?, validator.clone())
		}
		#[weight = 0]
		fn nominator_bond_more(
			origin,
			candidate: T::AccountId,
			more: BalanceOf<T>
		) -> DispatchResult {
			let nominator = ensure_signed(origin)?;
			let mut nominations = <Nominators<T>>::get(&nominator).ok_or(Error::<T>::NominatorDNE)?;
			let mut validator = <Candidates<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			let _ = nominations
				.inc_nomination(candidate.clone(), more)
				.ok_or(Error::<T>::NominationDNE)?;
			T::Currency::reserve(&nominator, more)?;
			let before = validator.total;
			validator.inc_nominator(nominator.clone(), more);
			let after = validator.total;
			if validator.is_active() {
				Self::update_active(candidate.clone(), validator.total);
			}
			<Candidates<T>>::insert(&candidate, validator);
			<Nominators<T>>::insert(&nominator, nominations);
			Self::deposit_event(RawEvent::NominationIncreased(nominator, candidate, before, after));
			Ok(())
		}
		#[weight = 0]
		fn nominator_bond_less(
			origin,
			candidate: T::AccountId,
			less: BalanceOf<T>
		) -> DispatchResult {
			let nominator = ensure_signed(origin)?;
			let mut nominations = <Nominators<T>>::get(&nominator).ok_or(Error::<T>::NominatorDNE)?;
			let mut validator = <Candidates<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			let remaining = nominations
				.dec_nomination(candidate.clone(), less)
				.ok_or(Error::<T>::NominationDNE)?
				.ok_or(Error::<T>::Underflow)?;
			ensure!(remaining >= T::MinNomination::get(), Error::<T>::NominationBelowMin);
			ensure!(nominations.total >= T::MinNominatorStk::get(), Error::<T>::NomBondBelowMin);
			T::Currency::unreserve(&nominator, less);
			let before = validator.total;
			validator.dec_nominator(nominator.clone(), less);
			let after = validator.total;
			if validator.is_active() {
				Self::update_active(candidate.clone(), validator.total);
			}
			<Candidates<T>>::insert(&candidate, validator);
			<Nominators<T>>::insert(&nominator, nominations);
			Self::deposit_event(RawEvent::NominationDecreased(nominator, candidate, before, after));
			Ok(())
		}
		fn on_finalize(n: T::BlockNumber) {
			if (n % T::BlocksPerRound::get()).is_zero() {
				let next = <Round>::get() + 1;
				// pay all stakers for T::BondDuration rounds ago
				Self::pay_stakers(next);
				// execute all delayed validator exits
				Self::execute_delayed_validator_exits(next);
				// insert exposure for next validator set
				let (validator_count, total_staked) = Self::best_candidates_become_validators(next);
				// start next round
				<Round>::put(next);
				Self::deposit_event(RawEvent::NewRound(n, next, validator_count, total_staked));
			}
		}
	}
}

impl<T: Config> Module<T> {
	pub fn is_nominator(acc: &T::AccountId) -> bool {
		<Nominators<T>>::get(acc).is_some()
	}
	pub fn is_candidate(acc: &T::AccountId) -> bool {
		<Candidates<T>>::get(acc).is_some()
	}
	pub fn is_validator(acc: &T::AccountId) -> bool {
		<Validators<T>>::get().binary_search(acc).is_ok()
	}
	// ensure candidate is active before calling
	fn update_active(candidate: T::AccountId, total: BalanceOf<T>) {
		let mut candidates = <CandidatePool<T>>::get();
		candidates.remove(&Bond::from_owner(candidate.clone()));
		candidates.insert(Bond {
			owner: candidate.clone(),
			amount: total,
		});
		<CandidatePool<T>>::put(candidates);
	}
	fn nominator_joins_validator(
		nominator: T::AccountId,
		amount: BalanceOf<T>,
		validator: T::AccountId,
	) -> DispatchResult {
		let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
		let nomination = Bond {
			owner: nominator.clone(),
			amount,
		};
		ensure!(
			state.nominators.insert(nomination),
			Error::<T>::NominatorExists
		);
		ensure!(
			state.nominators.0.len() <= T::MaxNominatorsPerValidator::get(),
			Error::<T>::TooManyNominators
		);
		T::Currency::reserve(&nominator, amount)?;
		let new_total = state.total + amount;
		if state.is_active() {
			Self::update_active(validator.clone(), new_total);
		}
		let new_total_locked = <Total<T>>::get() + amount;
		<Total<T>>::put(new_total_locked);
		state.total = new_total;
		<Candidates<T>>::insert(&validator, state);
		Self::deposit_event(RawEvent::ValidatorNominated(
			nominator, amount, validator, new_total,
		));
		Ok(())
	}
	fn nominator_revokes_validator(acc: T::AccountId, validator: T::AccountId) -> DispatchResult {
		let mut nominator = <Nominators<T>>::get(&acc).ok_or(Error::<T>::NominatorDNE)?;
		let remaining = nominator
			.rm_nomination(validator.clone())
			.ok_or(Error::<T>::NominationDNE)?;
		ensure!(
			remaining >= T::MinNominatorStk::get(),
			Error::<T>::NomBondBelowMin
		);
		Self::nominator_leaves_validator(acc.clone(), validator.clone())?;
		<Nominators<T>>::insert(&acc, nominator);
		Ok(())
	}
	fn nominator_leaves_validator(
		nominator: T::AccountId,
		validator: T::AccountId,
	) -> DispatchResult {
		let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
		let mut exists: Option<BalanceOf<T>> = None;
		let noms = state
			.nominators
			.0
			.into_iter()
			.filter_map(|nom| {
				if nom.owner != nominator {
					Some(nom)
				} else {
					exists = Some(nom.amount);
					None
				}
			})
			.collect();
		let nominators = OrderedSet::from(noms);
		let nominator_stake = exists.ok_or(Error::<T>::NominatorDNE)?;
		T::Currency::unreserve(&nominator, nominator_stake);
		state.nominators = nominators;
		state.total -= nominator_stake;
		if state.is_active() {
			Self::update_active(validator.clone(), state.total);
		}
		let new_total_locked = <Total<T>>::get() - nominator_stake;
		<Total<T>>::put(new_total_locked);
		let new_total = state.total;
		<Candidates<T>>::insert(&validator, state);
		Self::deposit_event(RawEvent::NominatorLeftValidator(
			nominator,
			validator,
			nominator_stake,
			new_total,
		));
		Ok(())
	}
	fn pay_stakers(next: RoundIndex) {
		let duration = T::BondDuration::get();
		if next > duration {
			let round_to_payout = next - duration;
			let total = <Points>::get(round_to_payout);
			if total == 0u32 {
				return;
			}
			let issuance = T::IssuancePerRound::get();
			for (val, pts) in <AwardedPts<T>>::drain_prefix(round_to_payout) {
				let pct_due = Perbill::from_rational_approximation(pts, total);
				let mut amt_due = pct_due * issuance;
				if amt_due < T::Currency::minimum_balance() {
					continue;
				}
				if let Some(state) = <Candidates<T>>::get(&val) {
					if state.nominators.0.len() == 0usize {
						// solo validator with no nominators
						if let Some(imb) = T::Currency::deposit_into_existing(&val, amt_due).ok() {
							Self::deposit_event(RawEvent::Rewarded(val.clone(), imb.peek()));
						}
					} else {
						let fee = state.fee * amt_due;
						if let Some(imb) = T::Currency::deposit_into_existing(&val, fee).ok() {
							Self::deposit_event(RawEvent::Rewarded(val.clone(), imb.peek()));
						}
						amt_due -= fee;
						for Bond { owner, amount } in state.nominators.0 {
							let percent = Perbill::from_rational_approximation(amount, state.total);
							let due = percent * amt_due;
							if let Some(imb) = T::Currency::deposit_into_existing(&owner, due).ok()
							{
								Self::deposit_event(RawEvent::Rewarded(owner.clone(), imb.peek()));
							}
						}
						let pct = Perbill::from_rational_approximation(state.bond, state.total);
						let due = pct * amt_due;
						if let Some(imb) = T::Currency::deposit_into_existing(&state.id, due).ok() {
							Self::deposit_event(RawEvent::Rewarded(state.id.clone(), imb.peek()));
						}
					}
				}
			}
		}
	}
	fn execute_delayed_validator_exits(next: RoundIndex) {
		let remain_exits = <ExitQueue<T>>::get()
			.0
			.into_iter()
			.filter_map(|x| {
				if x.amount > next {
					Some(x)
				} else {
					if let Some(state) = <Candidates<T>>::get(&x.owner) {
						for bond in state.nominators.0 {
							// return stake to nominator
							T::Currency::unreserve(&bond.owner, bond.amount);
							// remove nomination from nominator state
							if let Some(mut nominator) = <Nominators<T>>::get(&bond.owner) {
								if let Some(remaining) = nominator.rm_nomination(x.owner.clone()) {
									if remaining.is_zero() {
										<Nominators<T>>::remove(&bond.owner);
									} else {
										<Nominators<T>>::insert(&bond.owner, nominator);
									}
								}
							}
						}
						// return stake to validator
						T::Currency::unreserve(&state.id, state.bond);
						let new_total = <Total<T>>::get() - state.total;
						<Total<T>>::put(new_total);
						<Candidates<T>>::remove(&x.owner);
						Self::deposit_event(RawEvent::ValidatorLeft(
							x.owner.clone(),
							state.total,
							new_total,
						));
					}
					None
				}
			})
			.collect::<Vec<Bond<T::AccountId, RoundIndex>>>();
		<ExitQueue<T>>::put(OrderedSet::from(remain_exits));
	}
	/// Best as in most cumulatively supported in terms of stake
	fn best_candidates_become_validators(next: RoundIndex) -> (u32, BalanceOf<T>) {
		let (mut all_validators, mut total) = (0u32, BalanceOf::<T>::zero());
		let mut candidates = <CandidatePool<T>>::get().0;
		// order candidates by stake (least to greatest so requires `rev()`)
		candidates.sort_unstable_by(|a, b| a.amount.partial_cmp(&b.amount).unwrap());
		let max_validators = T::MaxValidators::get() as usize;
		// choose the top MaxValidators qualified candidates, ordered by stake
		let mut validators = candidates
			.into_iter()
			.rev()
			.take(max_validators)
			.map(|x| x.owner)
			.collect::<Vec<T::AccountId>>();
		// snapshot exposure for round
		for account in validators.iter() {
			let state = <Candidates<T>>::get(&account)
				.expect("all members of CandidateQ must be viable candidates by construction; qed");
			let amount = state.total;
			let exposure: Exposure<T::AccountId, BalanceOf<T>> = state.into();
			<AtStake<T>>::insert(next, account, exposure);
			all_validators += 1u32;
			total += amount;
			Self::deposit_event(RawEvent::ValidatorChosen(next, account.clone(), amount));
		}
		validators.sort();
		// insert canonical validator set
		<Validators<T>>::put(validators);
		(all_validators, total)
	}
}

/// Add reward points to block authors:
/// * 20 points to the block producer for producing a block in the chain
impl<T> author_inherent::EventHandler<T::AccountId> for Module<T>
where
	T: Config + author_inherent::Config,
{
	fn note_author(author: T::AccountId) {
		let now = <Round>::get();
		let score_plus_20 = <AwardedPts<T>>::get(now, &author) + 20;
		<AwardedPts<T>>::insert(now, author, score_plus_20);
		<Points>::mutate(now, |x| *x += 20);
	}
}

impl<T> author_inherent::CanAuthor<T::AccountId> for Module<T>
where
	T: Config + author_inherent::Config,
{
	fn can_author(account: &T::AccountId) -> bool {
		Self::is_validator(account)
	}
}
