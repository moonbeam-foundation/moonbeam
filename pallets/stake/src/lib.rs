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

//! # Stake Pallet
//!
//! Minimal viable stake pallet, drop-in replacement for pallet-staking for substrate chains. Each
//! nominator can choose at most one validator to share profit and slashing risk.
//!
//! **Goals**:
//! * enable nominators to share profit and slashing risk with validators
//! * choose validator set for each session from validator candidates
//!
//! These goals are 3 distinct requirements:
//! 1. choosing canonical validator set
//! 2. paying rewards
//! 3. enforcing punishment
//!
//! ### Rules
//!
//! **Nominator Rules**
//! * nominators may bond in support of validator candidates (`nominate`)
//!     * if a candidate becomes a validator, its nominators share profit and slashing risk
//! * to become a nominator, an account must not be a validator candidate nor an existing nominator
//!
//! **Validator Rules**
//! * all validators are/were validator candidates at some time
//! * validators are selected at the beginning of each Round from the viable candidates and are only
//! defined as being validators in the context of the specific `Round`
//! * to become a validator candidate, an account must not be a validator candidate nor an existing
//! nominator
//!
//! ### Choosing Canonical Validator Set
//!
//! Implement `SessionManager`, this is the only API to query new validator sets and allow the
//! validator set to be rewarded once the era ends.
//!
//! There is a new round every `BlocksPerRound` and, upon every round change, a new validator set
//! is selected from the candidates stored in `Candidates: T::AccountId => Option<ValState>`.
//! Validator selection and insertion is executed in `qualified_candidates_become_validators`.
//!
//! ### Paying Rewards
//! * every validator takes a fee off the top and there is a maximum fee for the whole module
//! * preventing double-withdrawals
//! * weighting relative points vs relative stake for reward distributions
//!
//! ### Enforcing Punishment (Slashing)
//!
//! Bad behavior:
//! 1. not producing blocks (offline without calling `go_offline`)
//! 2. equivocation (signing two different blocks at the same height)
//!
//! Could charge per-round fee if validator does not produce blocks and/or goes offline (1)
//!
//! (2) is a big deal and should result in a large penalty, possibly forced immediate removal from the validator set

#![recursion_limit = "256"]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure,
	traits::{
		Currency, EnsureOrigin, EstimateNextNewSession, ExistenceRequirement, Get, Imbalance,
		LockableCurrency, ReservableCurrency,
	},
};
use frame_system::{ensure_signed, Config as System};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{
	traits::{AccountIdConversion, AtLeast32BitUnsigned, Convert, Zero},
	DispatchResult, ModuleId, Perbill, RuntimeDebug,
};
use sp_std::prelude::*;
mod substrate;
pub use substrate::*;
#[cfg(test)]
pub(crate) mod mock;
#[cfg(test)]
mod tests;

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
/// Destination set by payee for receiving rewards
pub enum Destination<AccountId> {
	/// Pay into the stash account, not increasing the amount at stake.
	Stash,
	/// Pay into a specified account.
	Account(AccountId),
}

impl<AccountId> Default for Destination<AccountId> {
	fn default() -> Self {
		Destination::Stash
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
/// Wrapper around destination for configurable reward splitting
pub enum Reward<Dest> {
	/// Pay into single RewardDestination
	One(Dest),
	/// Pay (Perbill * Total) to 1st Dest, ((1-Perbill) * Total) to 2nd Dest
	Two(Dest, Perbill, Dest),
}

impl<Dest: Default> Default for Reward<Dest> {
	fn default() -> Reward<Dest> {
		Reward::One(Dest::default())
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
/// Validator preferences
pub struct ValPrefs<Balance> {
	/// The fee this validator takes on all profits before returning rewards in proportion to stake weight (which may also include the validator)
	pub fee: Perbill,
	/// Minimum nomination amount accepted by this validator
	pub min: Balance,
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct Nomination<AccountId, Balance> {
	pub owner: AccountId,
	pub amount: Balance,
}

impl<A, B> Nomination<A, B> {
	pub fn new(owner: A, amount: B) -> Nomination<A, B> {
		Nomination { owner, amount }
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
/// Slash status of the validator
pub enum Slash {
	/// Should be removed from the validator set ASAP and slashed
	Remove,
	/// Has strikes and will be removed, slashed if strikes exceeds MaxStrikes
	Strike(u8),
}

impl Default for Slash {
	fn default() -> Slash {
		Slash::Strike(0u8)
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub enum ValStatus {
	Active,
	Idle,
}

impl Default for ValStatus {
	fn default() -> ValStatus {
		ValStatus::Active
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct ValState<AccountId, Balance> {
	pub validator: AccountId,
	pub prefs: ValPrefs<Balance>,
	pub nominations: Vec<Nomination<AccountId, Balance>>,
	pub total: Balance,
	pub state: ValStatus,
	pub slash: Slash,
}

impl<
		A: Ord + Clone,
		B: AtLeast32BitUnsigned + Ord + Copy + sp_std::ops::AddAssign + sp_std::ops::SubAssign,
	> ValState<A, B>
{
	pub fn new(validator: A, prefs: ValPrefs<B>, bond: B) -> ValState<A, B> {
		ValState {
			validator: validator.clone(),
			prefs,
			nominations: vec![Nomination::new(validator, bond)],
			total: bond,
			state: ValStatus::default(), // default active
			slash: Slash::default(),     // default innocent
		}
	}
	pub fn innocent(&self) -> bool {
		self.slash == Slash::Strike(0u8)
	}
	pub fn has_strikes(&self) -> bool {
		if let Slash::Strike(_) = self.slash {
			true
		} else {
			false
		}
	}
	pub fn cannot_return(&self) -> bool {
		self.slash == Slash::Remove
	}
	pub fn is_active(&self) -> bool {
		self.state == ValStatus::Active
	}
	pub fn can_validate(&self) -> bool {
		self.is_active() && self.has_strikes()
	}
	pub fn remove(&mut self) {
		self.slash = Slash::Remove;
		self.go_offline();
	}
	pub fn add_strike(&mut self) {
		if let Slash::Strike(c) = self.slash {
			self.slash = Slash::Strike(c + 1u8);
		}
	}
	pub fn reset_strikes(&mut self) {
		self.slash = Slash::Strike(0u8)
	}
	pub fn go_offline(&mut self) {
		self.state = ValStatus::Idle;
	}
	pub fn activate(&mut self) {
		self.state = ValStatus::Active;
	}
	/// Adds new nomination (assumes nomination does not already exist for nominator A)
	pub fn add_nomination(&mut self, nominator: A, amount: B) {
		self.nominations.push(Nomination::new(nominator, amount));
		self.total += amount;
	}
	/// Remove the entire nomination and, if removal successful, return unstaked amount
	pub fn rm_nomination(&mut self, nominator: A) -> Option<B> {
		let mut ret: Option<B> = None;
		let nominations = self
			.nominations
			.clone()
			.into_iter()
			.filter_map(|x| {
				if x.owner == nominator {
					ret = Some(x.amount);
					Some(x)
				} else {
					None
				}
			})
			.collect();
		self.nominations = nominations;
		ret
	}
}

type RoundIndex = u32;
type RewardPoint = u32;
type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as System>::AccountId>>::Balance;
type PositiveImbalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as System>::AccountId>>::PositiveImbalance;
type ValidatorState<T> = ValState<<T as System>::AccountId, BalanceOf<T>>;
type RewardPolicy<T> = Reward<Destination<<T as System>::AccountId>>;

pub trait Config: System {
	/// The overarching event type
	type Event: From<Event<Self>> + Into<<Self as System>::Event>;
	/// The currency type
	type Currency: ReservableCurrency<Self::AccountId>
		+ LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
	// ~~ SESSION STUFF ~~
	/// Interface for interacting with a session module.
	type SessionInterface: substrate::SessionInterface<Self::AccountId>;
	/// Something that can estimate the next session change, accurately or as a best effort guess.
	type NextNewSession: EstimateNextNewSession<Self::BlockNumber>;
	// ~~ CONSTANTS ~~
	/// Maximum number of validators for any given round
	type MaxValidators: Get<u32>;
	/// Maximum individual nominators for all validators
	type MaxNominatorsPerValidator: Get<usize>;
	/// Minimum individual nominators for all validators
	type MinNominatorsPerValidator: Get<usize>;
	/// Minimum stake for any registered on-chain account to become a validator candidate
	type MinCandidateBond: Get<BalanceOf<Self>>;
	/// Total minimum backed stake for any candidate to become a validator
	type MinValidatorBond: Get<BalanceOf<Self>>;
	/// Minimum stake for any registered on-chain account to become a nominator
	type MinNominatorBond: Get<BalanceOf<Self>>;
	/// Maximum fee a validator can charge (taken off the top of revenue, before stake-weighted payouts)
	type MaxValidatorFee: Get<Perbill>;
	/// Origin from which all slashes come from
	type SlashOrigin: EnsureOrigin<Self::Origin>;
	/// Maximum slash strike count a validator can incur before they are removed
	type MaxStrikes: Get<u8>;
	/// Percentage of collateral slashed
	type SlashPct: Get<Perbill>;
	/// Length of each round in blocks
	type BlocksPerRound: Get<Self::BlockNumber>;
	/// Weighting to determine reward payouts as Perbill * PtsPercent + (1-Perbill)* StakePercent
	type Pts2StakeRewardRatio: Get<Perbill>;
	/// Maximum reward (per Round)
	type Reward: Get<BalanceOf<Self>>;
	/// The treasury's module id, used for deriving its sovereign account ID.
	type Treasury: Get<ModuleId>;
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as System>::AccountId,
		Balance = BalanceOf<T>,
		BlockNumber = <T as System>::BlockNumber,
	{
		// Account, Amount Locked
		CandidateJoined(AccountId, Balance),
		ValidatorOffline(RoundIndex, AccountId),
		ValidatorActivated(RoundIndex, AccountId),
		// Round, Validator Account, Total Exposed Amount (includes all nominations)
		ValidatorChosen(RoundIndex, AccountId, Balance),
		// Account, Amount Unlocked, New Total Amt Locked
		ValidatorLeft(AccountId, Balance, Balance),
		// Nominator, Amount Locked, Validator, New Total Amt Locked
		ValidatorNominated(AccountId, Balance, AccountId, Balance),
		NominationRevoked(AccountId, Balance, AccountId, Balance),
		Rewarded(AccountId, Balance),
		// Validator, Nominator, Balance Slashed
		Slashed(AccountId, AccountId, Balance),
		NewRound(BlockNumber, RoundIndex, u32, Balance),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		// Nominator Does Not Exist
		NominatorDNE,
		CandidateDNE,
		ValidatorDNE,
		NominatorExists,
		ValidatorExists,
		CandidateBondBelowMin,
		NominatorBondBelowMin,
		FeeExceedsMaxValidatorFee,
		CannotRmNomNotFound,
		TooManyNomForVal,
		AlreadyActive,
		AlreadyOffline,
		NoPointsNoReward,
		CurrentRndRewardsUnClaimable,
		CannotRevokeIfValidatorAwaitingSlash,
	}
}

decl_storage! {
	trait Store for Module<T: Config> as Stake {
		/// Current round, incremented every `BlocksPerRound` in `fn on_finalize`
		pub Round get(fn round): RoundIndex;
		/// Nominators with their validator
		pub Nominators get(fn nominators): map
			hasher(blake2_128_concat) T::AccountId => Option<T::AccountId>;
		/// Validator candidates with nomination state (includes all current validators by default)
		pub Candidates get(fn candidates): map
			hasher(blake2_128_concat) T::AccountId => Option<ValidatorState<T>>;
		/// Total locked
		pub TotalLocked get(fn total_locked): BalanceOf<T>;
		/// Total at stake per round
		pub TotalStake get(fn total_stake): map
			hasher(blake2_128_concat) RoundIndex => BalanceOf<T>;
		/// Stake exposure per round, per account (canonical validator set for round)
		pub AtStake get(fn at_stake): double_map
			hasher(blake2_128_concat) RoundIndex,
			hasher(blake2_128_concat) T::AccountId => Exposure<T::AccountId,BalanceOf<T>>;
		/// Total points awarded in this round
		pub Points get(fn points): map
			hasher(blake2_128_concat) RoundIndex => RewardPoint;
		/// Validator set for the given round, stores individual points accrued each round per validator
		pub ValidatorPts get(fn validator_pts): double_map
			hasher(blake2_128_concat) RoundIndex,
			hasher(blake2_128_concat) T::AccountId => RewardPoint;
		/// Track recipient preferences for receiving rewards
		pub Payee get(fn payee): map
			hasher(blake2_128_concat) T::AccountId => RewardPolicy<T>;
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
						Default::default(),
					)
				} else {
					<Module<T>>::join_candidates(
						T::Origin::from(Some(actor.clone()).into()),
						balance,
						Perbill::from_percent(2), // default fee
						T::MinNominatorBond::get(), // default minimum nominator bond
						Default::default(),
					)
				};
			}
			// starts with Round 1 at Block 0
			<Module<T>>::qualified_candidates_become_validators(T::BlockNumber::zero(),1u32);
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
			stake: BalanceOf<T>,
			fee: Perbill,
			min: BalanceOf<T>,
			policy: RewardPolicy<T>,
		) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			ensure!(!Self::is_nominator(&acc),Error::<T>::NominatorExists);
			ensure!(!Self::is_candidate(&acc),Error::<T>::ValidatorExists);
			ensure!(stake >= T::MinCandidateBond::get(),Error::<T>::CandidateBondBelowMin);
			ensure!(min >= T::MinNominatorBond::get(),Error::<T>::NominatorBondBelowMin);
			ensure!(fee <= T::MaxValidatorFee::get(),Error::<T>::FeeExceedsMaxValidatorFee);
			T::Currency::reserve(&acc,stake)?;
			let state: ValidatorState<T> = ValState::new(acc.clone(),ValPrefs{fee,min},stake);
			<Candidates<T>>::insert(&acc,state);
			<Payee<T>>::insert(&acc,policy);
			Self::deposit_event(RawEvent::CandidateJoined(acc,stake));
			Ok(())
		}
		#[weight = 0]
		fn go_offline(origin) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			let mut validator = <Candidates<T>>::get(&acc).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(validator.is_active(),Error::<T>::AlreadyOffline);
			validator.go_offline();
			<Candidates<T>>::insert(&acc,validator);
			Self::deposit_event(RawEvent::ValidatorOffline(<Round>::get(),acc));
			Ok(())
		}
		#[weight = 0]
		fn activate(origin) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			let mut validator = <Candidates<T>>::get(&acc).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!validator.is_active(),Error::<T>::AlreadyActive);
			validator.activate();
			<Candidates<T>>::insert(&acc,validator);
			Self::deposit_event(RawEvent::ValidatorActivated(<Round>::get(),acc));
			Ok(())
		}
		#[weight = 0]
		fn exit(origin) -> DispatchResult {
			Self::return_nominations(ensure_signed(origin)?)
		}
		#[weight = 0]
		fn nominate(
			origin,
			validator: T::AccountId,
			amount: BalanceOf<T>,
			payee: RewardPolicy<T>,
		) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			ensure!(!Self::is_candidate(&acc),Error::<T>::ValidatorExists);
			ensure!(!Self::is_nominator(&acc),Error::<T>::NominatorExists);
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(amount >= state.prefs.min,Error::<T>::NominatorBondBelowMin);
			state.add_nomination(acc.clone(),amount);
			ensure!(state.nominations.len() <= T::MaxNominatorsPerValidator::get(), Error::<T>::TooManyNomForVal);
			T::Currency::reserve(&acc,amount)?;
			let new_total = state.total;
			let new_total_locked = <TotalLocked<T>>::get() + amount;
			<TotalLocked<T>>::put(new_total_locked);
			<Nominators<T>>::insert(&acc,validator.clone());
			<Candidates<T>>::insert(&validator,state);
			<Payee<T>>::insert(&acc,payee);
			Self::deposit_event(RawEvent::ValidatorNominated(acc,amount,validator,new_total));
			Ok(())
		}
		#[weight = 0]
		fn revoke_nomination(origin) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			let validator = <Nominators<T>>::get(&acc).ok_or(Error::<T>::NominatorDNE)?;
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(state.innocent(), Error::<T>::CannotRevokeIfValidatorAwaitingSlash);
			let amt_unstaked = state.rm_nomination(acc.clone()).ok_or(Error::<T>::CannotRmNomNotFound)?;
			let new_total = state.total;
			let new_total_locked = <TotalLocked<T>>::get() - amt_unstaked;
			<TotalLocked<T>>::put(new_total_locked);
			<Candidates<T>>::insert(&validator,state);
			<Nominators<T>>::remove(&acc);
			Self::deposit_event(RawEvent::NominationRevoked(acc,amt_unstaked,validator,new_total));
			Ok(())
		}
		#[weight = 0]
		fn pay_stakers(
			origin,
			validator: T::AccountId,
			round: RoundIndex,
		) -> DispatchResult {
			ensure_signed(origin)?;
			Self::pay_staker(validator,round)
		}
		#[weight = 0]
		fn report_offline(
			origin,
			validator: T::AccountId,
			round: RoundIndex
		) -> DispatchResult {
			T::SlashOrigin::ensure_origin(origin)?;
			Self::slash_staker(validator,round,Slash::Strike(1u8))
		}
		#[weight = 0]
		fn report_equivocation(
			origin,
			validator: T::AccountId,
			round: RoundIndex
		) -> DispatchResult {
			T::SlashOrigin::ensure_origin(origin)?;
			Self::slash_staker(validator,round,Slash::Remove)
		}
		fn on_finalize(n: T::BlockNumber) {
			if (n % T::BlocksPerRound::get()).is_zero() {
				let last = <Round>::get();
				let next = last + 1;
				// insert exposure for next validator set
				Self::qualified_candidates_become_validators(n,next);
			}
		}
	}
}

impl<T: Config> Module<T> {
	pub fn treasury() -> T::AccountId {
		T::Treasury::get().into_account()
	}
	pub fn is_nominator(acc: &T::AccountId) -> bool {
		<Nominators<T>>::get(acc).is_some()
	}
	pub fn is_candidate(acc: &T::AccountId) -> bool {
		<Candidates<T>>::get(acc).is_some()
	}
	pub fn is_validator(round: RoundIndex, acc: &T::AccountId) -> bool {
		<AtStake<T>>::get(round, acc) != Exposure::default()
	}
	fn qualified_candidates_become_validators(block: T::BlockNumber, round: RoundIndex) {
		let mut val_count = 0u32;
		let mut total_at_stake = BalanceOf::<T>::zero();
		let (min_bond, min_noms, max_noms, max_vals) = (
			T::MinValidatorBond::get(),
			T::MinNominatorsPerValidator::get(),
			T::MaxNominatorsPerValidator::get(),
			T::MaxValidators::get(),
		);
		// apply slash and remove from validator set
		let apply_slash = |acc: T::AccountId, info: ValidatorState<T>| {
			let slash_pct = T::SlashPct::get();
			let treasury = Self::treasury();
			for Nomination { owner, amount } in info.nominations {
				let slashed = slash_pct * amount;
				T::Currency::unreserve(&owner, amount);
				if let Ok(_) = T::Currency::transfer(
					&owner,
					&treasury,
					slashed,
					ExistenceRequirement::KeepAlive,
				) {
					Self::deposit_event(RawEvent::Slashed(acc.clone(), owner.clone(), slashed));
				}
				<Nominators<T>>::remove(&owner);
			}
			<Candidates<T>>::remove(&acc);
			// make sure to disable validator till the end of this session
			let _ = T::SessionInterface::disable_validator(&acc);
		};
		// apply queued slashes and choose next validator set
		for (acc, info) in <Candidates<T>>::iter() {
			// First, apply all slashes
			match info.slash {
				Slash::Remove => {
					// apply slash and remove from validator set
					apply_slash(acc.clone(), info.clone());
				}
				Slash::Strike(count) => {
					if count > T::MaxStrikes::get() {
						// apply slash and remove from validator set
						apply_slash(acc.clone(), info.clone());
					}
				}
			}
			if val_count >= max_vals {
				continue; // skip validator selection if so
			}
			// Next, choose the next validator set
			// it is all not offline validators above minimum validator capital threshold
			// -> these validators are exposed to slashing risk for next round
			let num_noms = info.nominations.len();
			let total = info.total;
			let qualified_validator: bool = info.can_validate()
				&& total >= min_bond
				&& num_noms >= min_noms
				&& num_noms <= max_noms;
			if qualified_validator {
				// convert from ValState to Exposure
				let exposure: Exposure<T::AccountId, BalanceOf<T>> = info.into();
				<AtStake<T>>::insert(round, &acc, exposure);
				val_count += 1u32;
				total_at_stake += total;
				Self::deposit_event(RawEvent::ValidatorChosen(round, acc.clone(), total));
			}
		}
		// start the next round
		<Round>::put(round);
		<TotalStake<T>>::insert(round, total_at_stake);
		Self::deposit_event(RawEvent::NewRound(block, round, val_count, total_at_stake));
	}
	fn return_nominations(validator: T::AccountId) -> DispatchResult {
		let state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
		ensure!(
			state.innocent(),
			Error::<T>::CannotRevokeIfValidatorAwaitingSlash
		);
		for Nomination { owner, amount } in state.nominations {
			// return stake
			let _ = T::Currency::unreserve(&owner, amount);
		}
		let new_total_locked = <TotalLocked<T>>::get() - state.total;
		<TotalLocked<T>>::put(new_total_locked);
		<Candidates<T>>::remove(&validator);
		Self::deposit_event(RawEvent::ValidatorLeft(
			validator,
			state.total,
			new_total_locked,
		));
		Ok(())
	}
	fn slash_staker(validator: T::AccountId, round: RoundIndex, ty: Slash) -> DispatchResult {
		let mut val = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
		let at_stake = <AtStake<T>>::get(round, &validator);
		ensure!(at_stake != Exposure::default(), Error::<T>::ValidatorDNE);
		match ty {
			Slash::Remove => {
				val.remove(); // sets offline by default to keep staker out of future validator sets
				<Candidates<T>>::insert(&validator, val);
				Ok(())
			}
			Slash::Strike(_) => {
				val.add_strike();
				<Candidates<T>>::insert(&validator, val);
				Ok(())
			}
		}
	}
	/// Pay validator for points awarded in the given round
	fn pay_staker(validator: T::AccountId, round: RoundIndex) -> DispatchResult {
		ensure!(
			<Round>::get() > round,
			Error::<T>::CurrentRndRewardsUnClaimable
		);
		let points = <ValidatorPts<T>>::get(round, &validator);
		ensure!(points > Zero::zero(), Error::<T>::NoPointsNoReward);
		let val = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
		let at_stake = <AtStake<T>>::get(round, &validator);
		ensure!(at_stake != Exposure::default(), Error::<T>::ValidatorDNE);
		let all_pts = <Points>::get(round);
		let pts_pct = Perbill::from_rational_approximation(points, all_pts) * 100u32;
		let all_stake = <TotalStake<T>>::get(round);
		let stake_pct = Perbill::from_rational_approximation(at_stake.total, all_stake) * 100u32;
		let ratio = T::Pts2StakeRewardRatio::get();
		let ratio_as_u32 = ratio * 100u32;
		let inv = Perbill::from_percent(100u32 - ratio_as_u32);
		let pct_for_val = (ratio * pts_pct) + (inv * stake_pct);
		let all = Perbill::from_percent(pct_for_val) * T::Reward::get();
		let fee = val.prefs.fee * all;
		if let Some(imbalance) = Self::make_payout(&validator, fee) {
			Self::deposit_event(RawEvent::Rewarded(validator.clone(), imbalance.peek()));
		}
		let remaining = all - fee;
		for Nomination { owner, amount } in val.nominations {
			let percent = Perbill::from_rational_approximation(amount, val.total);
			let for_nom = percent * remaining;
			if let Some(imbalance) = Self::make_payout(&owner, for_nom) {
				Self::deposit_event(RawEvent::Rewarded(owner.clone(), imbalance.peek()));
			}
		}
		let remaining_pts = all_pts - points;
		<Points>::insert(round, remaining_pts);
		<ValidatorPts<T>>::remove(round, &validator);
		Ok(())
	}
	/// Pay specific account
	fn make_payout(stash: &T::AccountId, amount: BalanceOf<T>) -> Option<PositiveImbalanceOf<T>> {
		let policy = Self::payee(stash);
		let payout = |dest: Destination<T::AccountId>,
		              amount: BalanceOf<T>|
		 -> Option<PositiveImbalanceOf<T>> {
			match dest {
				Destination::Stash => T::Currency::deposit_into_existing(stash, amount).ok(),
				Destination::Account(dest_account) => {
					Some(T::Currency::deposit_creating(&dest_account, amount))
				}
			}
		};
		match policy {
			Reward::One(destination) => payout(destination, amount),
			Reward::Two(dest1, pct, dest2) => {
				if pct.is_one() {
					return payout(dest1, amount);
				}
				if pct.is_zero() {
					return payout(dest2, amount);
				}
				let first_amt = pct * amount;
				let remaining = amount - first_amt;
				if let Some(payout1) = payout(dest1, first_amt) {
					Some(payout1.maybe_merge(payout(dest2, remaining)))
				} else {
					payout(dest2, remaining)
				}
			}
		}
	}
	fn new_session() -> Option<Vec<T::AccountId>> {
		let (min_bond, min_noms, max_noms, max_vals) = (
			T::MinValidatorBond::get(),
			T::MinNominatorsPerValidator::get(),
			T::MaxNominatorsPerValidator::get(),
			T::MaxValidators::get(),
		);
		let mut all_vals = 0u32;
		let ret: Vec<T::AccountId> = <Candidates<T>>::iter()
			.filter_map(|(acc, info)| {
				let num_noms = info.nominations.len();
				let qualified_validator: bool = info.can_validate()
					&& info.total >= min_bond
					&& num_noms >= min_noms
					&& num_noms <= max_noms
					&& all_vals < max_vals;
				if qualified_validator {
					all_vals += 1u32;
					Some(acc)
				} else {
					None
				}
			})
			.collect();
		if ret.is_empty() {
			None
		} else {
			Some(ret)
		}
	}
	fn start_session(index: RoundIndex) {
		if index > <Round>::get() {
			<Round>::put(index);
		}
	}
}

impl<T: Config> pallet_session::SessionManager<T::AccountId> for Module<T> {
	fn new_session(_new_index: RoundIndex) -> Option<Vec<T::AccountId>> {
		Self::new_session()
	}
	fn start_session(start_index: RoundIndex) {
		Self::start_session(start_index)
	}
	fn end_session(_end_index: RoundIndex) {}
}

impl<T: Config>
	pallet_session::historical::SessionManager<T::AccountId, Exposure<T::AccountId, BalanceOf<T>>>
	for Module<T>
{
	fn new_session(
		new_index: RoundIndex,
	) -> Option<Vec<(T::AccountId, Exposure<T::AccountId, BalanceOf<T>>)>> {
		<Self as pallet_session::SessionManager<_>>::new_session(new_index).map(|validators| {
			let current_era = Self::round();
			validators
				.into_iter()
				.map(|v| {
					let exposure = Self::at_stake(current_era, &v);
					(v, exposure)
				})
				.collect()
		})
	}
	fn start_session(start_index: RoundIndex) {
		<Self as pallet_session::SessionManager<_>>::start_session(start_index)
	}
	fn end_session(end_index: RoundIndex) {
		<Self as pallet_session::SessionManager<_>>::end_session(end_index)
	}
}

/// Add reward points to block authors:
/// * 20 points to the block producer for producing a (non-uncle) block in the relay chain,
/// * 2 points to the block producer for each reference to a previously unreferenced uncle, and
/// * 1 point to the producer of each referenced uncle block.
impl<T> pallet_authorship::EventHandler<T::AccountId, T::BlockNumber> for Module<T>
where
	T: Config + pallet_authorship::Config + pallet_session::Config,
{
	fn note_author(author: T::AccountId) {
		let now = <Round>::get();
		let score_plus_20 = <ValidatorPts<T>>::get(now, &author) + 20;
		<ValidatorPts<T>>::insert(now, author, score_plus_20);
		<Points>::mutate(now, |x| *x += 20);
	}
	fn note_uncle(author: T::AccountId, _age: T::BlockNumber) {
		let now = <Round>::get();
		let p_auth = <pallet_authorship::Module<T>>::author();
		let score_plus_2 = <ValidatorPts<T>>::get(now, &p_auth) + 2;
		let score_plus_1 = <ValidatorPts<T>>::get(now, &author) + 1;
		<ValidatorPts<T>>::insert(now, p_auth, score_plus_2);
		<ValidatorPts<T>>::insert(now, author, score_plus_1);
		<Points>::mutate(now, |x| *x += 3);
	}
}

/// A typed conversion from stash account ID to the active exposure of nominators
/// on that account.
pub struct ExposureOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Convert<T::AccountId, Option<Exposure<T::AccountId, BalanceOf<T>>>>
	for ExposureOf<T>
{
	fn convert(validator: T::AccountId) -> Option<Exposure<T::AccountId, BalanceOf<T>>> {
		Some(<Module<T>>::at_stake(<Round>::get(), &validator))
	}
}

/// A `Convert` implementation that finds the stash of the given controller account,
/// if any.
pub struct StashOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Convert<T::AccountId, Option<T::AccountId>> for StashOf<T> {
	fn convert(controller: T::AccountId) -> Option<T::AccountId> {
		// our module only has one account per user so we just return the input if it is a validator or nominator
		if <Module<T>>::is_nominator(&controller) || <Module<T>>::is_candidate(&controller) {
			Some(controller)
		} else {
			None
		}
	}
}
