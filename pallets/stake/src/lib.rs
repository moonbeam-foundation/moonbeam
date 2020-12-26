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

//! # Minimal Staking Pallet
//!
//! Minimal viable staking pallet, drop-in replacement for pallet-staking for substrate chains. Each
//! nominator can choose at most one validator to share profit and slashing risk.

#![recursion_limit = "256"]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure,
	traits::{
		Currency, EstimateNextNewSession, Get, Imbalance, LockableCurrency, ReservableCurrency,
	},
};
use frame_system::{ensure_signed, Config as System};
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
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
pub enum ValStatus {
	Active,
	Chill,
}

#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug)]
pub struct ValState<AccountId, Balance> {
	pub validator: AccountId,
	pub prefs: ValPrefs<Balance>,
	pub nominations: Vec<Nomination<AccountId, Balance>>,
	pub total: Balance,
	pub status: ValStatus,
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
			status: ValStatus::Active,
		}
	}
	pub fn chill(&mut self) {
		self.status = ValStatus::Chill;
	}
	pub fn activate(&mut self) {
		self.status = ValStatus::Active;
	}
	pub fn is_active(&self) -> bool {
		self.status == ValStatus::Active
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

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Staker<AccountId> {
	Validator(AccountId),
	Nominator(AccountId, AccountId),
}
impl<AccountId: Clone> Staker<AccountId> {
	fn account(self) -> AccountId {
		match self {
			Staker::Validator(acc) => acc.clone(),
			Staker::Nominator(acc, _) => acc.clone(),
		}
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
	/// Timer for triggering periodic tasks in `on_finalize`
	type BlocksPerRound: Get<Self::BlockNumber>;
	/// Number of rounds kept in-memory for retroactive rewards/penalties
	type HistoryDepth: Get<u32>;
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
		ValidatorChilled(RoundIndex, AccountId),
		ValidatorActivated(RoundIndex, AccountId),
		// Round, Validator Account, Total Exposed Amount (includes all nominations)
		ValidatorChosen(RoundIndex, AccountId, Balance),
		// Account, Amount Unlocked, New Total Amt Locked
		ValidatorLeft(AccountId, Balance, Balance),
		// Nominator, Amount Locked, Validator, New Total Amt Locked
		ValidatorNominated(AccountId, Balance, AccountId, Balance),
		NominationRevoked(AccountId, Balance, AccountId, Balance),
		Rewarded(AccountId, Balance),
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
		AlreadyChill,
		NoPointsNoReward,
		CurrentRndRewardsUnClaimable,
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
			Vec<(Staker<T::AccountId>,BalanceOf<T>)>;
		build(|config: &GenesisConfig<T>| {
			for &(ref staker, balance) in &config.stakers {
				assert!(
					T::Currency::free_balance(&staker.clone().account()) >= balance,
					"Stash does not have enough balance to bond."
				);
				let _ = match staker {
					Staker::Validator(acc) => {
						<Module<T>>::join_candidates(
							T::Origin::from(Some(acc.clone()).into()),
							balance,
							Perbill::from_percent(2), // default fee
							T::MinNominatorBond::get(), // default minimum nominator bond
							Default::default(),
						)
					},
					Staker::Nominator(acc,val) => {
						<Module<T>>::nominate(
							T::Origin::from(Some(acc.clone()).into()),
							val.clone(),
							balance,
							Default::default(),
						)
					},
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
		fn chill(origin) -> DispatchResult {
			let acc = ensure_signed(origin)?;
			let mut validator = <Candidates<T>>::get(&acc).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(validator.is_active(),Error::<T>::AlreadyChill);
			validator.chill();
			<Candidates<T>>::insert(&acc,validator);
			Self::deposit_event(RawEvent::ValidatorChilled(<Round>::get(),acc));
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
		fn on_finalize(n: T::BlockNumber) {
			if (n % T::BlocksPerRound::get()).is_zero() {
				let last = <Round>::get();
				let next = last + 1;
				// pay remaining validators in next-HistoryDepth and remove exposure for all validators
				if next > T::HistoryDepth::get() {
					let round_to_delete = next - T::HistoryDepth::get();
					<ValidatorPts<T>>::iter_prefix(round_to_delete).for_each(|(val,_)| {
						// pay stakers that haven't claimed payment
						let _ = Self::pay_staker(val.clone(),round_to_delete);
					});
					// remove exposure for all validators in this round
					<AtStake<T>>::remove_prefix(round_to_delete);
				}
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
		// insert exposure for next validator set
		for (acc, info) in <Candidates<T>>::iter() {
			if val_count >= max_vals {
				break;
			}
			// next validator set is all unchilled validators above minimum validator capital threshold
			// -> these validators are by defn exposed to slashing risk for this round because no
			// transaction that called `exit` was processed before this point in time (tacit consent)
			let num_noms = info.nominations.len();
			let total = info.total;
			let qualified_validator: bool = info.is_active()
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
			// start the next round
			<Round>::put(round);
			<TotalStake<T>>::insert(round, total_at_stake);
			Self::deposit_event(RawEvent::NewRound(block, round, val_count, total_at_stake));
		}
	}
	fn return_nominations(validator: T::AccountId) -> DispatchResult {
		let state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
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
				let qualified_validator: bool = info.is_active()
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

/// Minimal implementation that ensures Round is strictly monotonically increasing.
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
