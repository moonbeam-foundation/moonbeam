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
//! To join the set of nominators, an account must not be a validator candidate nor an existing
//! nominator. To join the set of nominators, an account must call `join_nominators` with
//! stake >= `MinNominatorStk`.

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
pub struct CandidateState<AccountId, Balance, RoundIndex> {
	pub validator: AccountId,
	pub fee: Perbill,
	pub nominators: OrderedSet<Bond<AccountId, Balance>>,
	pub total: Balance,
	pub state: ValidatorStatus<RoundIndex>,
}

impl<
		A: Ord + Clone,
		B: AtLeast32BitUnsigned + Ord + Copy + sp_std::ops::AddAssign + sp_std::ops::SubAssign,
		C: Ord + Copy,
	> CandidateState<A, B, C>
{
	pub fn new(validator: A, fee: Perbill, bond: B) -> Self {
		let nominators = OrderedSet::from(vec![Bond {
			owner: validator.clone(),
			amount: bond,
		}]);
		let total = bond;
		CandidateState {
			validator,
			fee,
			nominators,
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
	pub fn go_offline(&mut self) {
		self.state = ValidatorStatus::Idle;
	}
	pub fn go_online(&mut self) {
		self.state = ValidatorStatus::Active;
	}
	pub fn leave_candidates(&mut self, block: C) {
		self.state = ValidatorStatus::Leaving(block);
	}
}

impl<A: PartialEq, B: HasCompact + Zero, C> Into<Exposure<A, B>> for CandidateState<A, B, C> {
	fn into(self) -> Exposure<A, B> {
		let mut others = Vec::<IndividualExposure<A, B>>::new();
		let mut own = Zero::zero();
		for Bond { owner, amount } in self.nominators.0 {
			if owner == self.validator {
				own = amount;
			} else {
				others.push(Bond { owner, amount }.into());
			}
		}
		Exposure {
			total: self.total,
			own,
			others,
		}
	}
}

type RoundIndex = u32;
type RewardPoint = u32;
type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as System>::AccountId>>::Balance;
type Candidate<T> = CandidateState<<T as System>::AccountId, BalanceOf<T>, RoundIndex>;

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
	/// Balance issued as rewards per round (constant issuance)
	type IssuancePerRound: Get<BalanceOf<Self>>;
	/// Maximum fee for any validator
	type MaxFee: Get<Perbill>;
	/// Minimum stake for any registered on-chain account to become a validator
	type MinValidatorStk: Get<BalanceOf<Self>>;
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
		ValidatorWentOffline(RoundIndex, AccountId),
		ValidatorBackOnline(RoundIndex, AccountId),
		/// Round, Validator Account, Scheduled Exit
		ValidatorScheduledExit(RoundIndex, AccountId, RoundIndex),
		/// Account, Amount Unlocked, New Total Amt Locked
		ValidatorLeft(AccountId, Balance, Balance),
		/// Nominator, Validator, Amount Unstaked, New Total Amt Staked for Validator
		NominatorLeft(AccountId, AccountId, Balance, Balance),
		/// Nominator, Amount Locked, Validator, New Total Amt Locked
		ValidatorNominated(AccountId, Balance, AccountId, Balance),
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
		AlreadyOffline,
		AlreadyActive,
		AlreadyLeaving,
		TooManyNominators,
		CannotActivateIfLeaving,
	}
}

decl_storage! {
	trait Store for Module<T: Config> as Stake {
		/// Current round, incremented every `BlocksPerRound` in `fn on_finalize`
		Round: RoundIndex;
		/// Current nominators with their validator
		Nominators: map hasher(blake2_128_concat) T::AccountId => Option<T::AccountId>;
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
					<Module<T>>::nominate(
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
			let candidate: Candidate<T> = CandidateState::new(acc.clone(),fee,bond);
			let new_total = <Total<T>>::get() + bond;
			<Total<T>>::put(new_total);
			<Candidates<T>>::insert(&acc,candidate);
			<CandidatePool<T>>::put(candidates);
			Self::deposit_event(RawEvent::JoinedValidatorCandidates(acc,bond,new_total));
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
		fn nominate(
			origin,
			validator: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			ensure!(!Self::is_nominator(&acc),Error::<T>::NominatorExists);
			ensure!(!Self::is_candidate(&acc),Error::<T>::CandidateExists);
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(amount >= T::MinNominatorStk::get(), Error::<T>::NomBondBelowMin);
			let nomination = Bond {
				owner: acc.clone(),
				amount,
			};
			ensure!(state.nominators.insert(nomination),Error::<T>::NominatorExists);
			ensure!(
				state.nominators.0.len() <= T::MaxNominatorsPerValidator::get(),
				Error::<T>::TooManyNominators
			);
			T::Currency::reserve(&acc,amount)?;
			let new_total = state.total + amount;
			if state.is_active() {
				Self::update_active_candidate(validator.clone(),new_total);
			}
			let new_total_locked = <Total<T>>::get() + amount;
			<Total<T>>::put(new_total_locked);
			<Nominators<T>>::insert(&acc,validator.clone());
			state.total = new_total;
			<Candidates<T>>::insert(&validator,state);
			Self::deposit_event(RawEvent::ValidatorNominated(acc,amount,validator,new_total));
			Ok(())
		}
		#[weight = 0]
		fn leave_nominators(origin) -> DispatchResult {
			let nominator = ensure_signed(origin)?;
			let validator = <Nominators<T>>::get(&nominator).ok_or(Error::<T>::NominatorDNE)?;
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			let mut exists: Option<BalanceOf<T>> = None;
			let noms = state.nominators.0.into_iter().filter_map(|nom| {
				if nom.owner != nominator {
					Some(nom)
				} else {
					exists = Some(nom.amount);
					None
				}
			}).collect();
			let nominators = OrderedSet::from(noms);
			let nominator_stake = exists.ok_or(Error::<T>::NominatorDNE)?;
			T::Currency::unreserve(&nominator,nominator_stake);
			state.nominators = nominators;
			let new_total = state.total - nominator_stake;
			if state.is_active() {
				Self::update_active_candidate(validator.clone(),new_total);
			}
			state.total = new_total;
			let new_total_locked = <Total<T>>::get() - nominator_stake;
			<Total<T>>::put(new_total_locked);
			<Candidates<T>>::insert(&validator,state);
			<Nominators<T>>::remove(&nominator);
			Self::deposit_event(
				RawEvent::NominatorLeft(nominator,validator,nominator_stake,new_total)
			);
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
	fn update_active_candidate(candidate: T::AccountId, new_total: BalanceOf<T>) {
		let mut candidates = <CandidatePool<T>>::get();
		candidates.remove(&Bond::from_owner(candidate.clone()));
		candidates.insert(Bond {
			owner: candidate.clone(),
			amount: new_total,
		});
		<CandidatePool<T>>::put(candidates);
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
					if state.nominators.0.len() == 1usize {
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
							// return funds to nominator
							T::Currency::unreserve(&bond.owner, bond.amount);
						}
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
