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

//! # Parachain Staking
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

#![cfg_attr(not(feature = "std"), no_std)]

mod inflation;
#[cfg(test)]
mod mock;
mod set;
#[cfg(test)]
mod tests;
use frame_support::pallet;
pub use inflation::{InflationInfo, Range};

pub use pallet::*;

#[pallet]
pub mod pallet {
	use super::{InflationInfo, Range};
	use crate::set::OrderedSet;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, Get, Imbalance, ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::{Decode, Encode};
	use sp_runtime::{
		traits::{AtLeast32BitUnsigned, Zero},
		DispatchResult, Perbill, RuntimeDebug,
	};
	use sp_std::{cmp::Ordering, prelude::*};

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
	/// The activity status of the validator
	pub enum ValidatorStatus {
		/// Committed to be online and producing valid blocks (not equivocating)
		Active,
		/// Temporarily inactive and excused for inactivity
		Idle,
		/// Bonded until the inner round
		Leaving(RoundIndex),
	}

	impl Default for ValidatorStatus {
		fn default() -> ValidatorStatus {
			ValidatorStatus::Active
		}
	}

	#[derive(Default, Encode, Decode, RuntimeDebug)]
	/// Snapshot of validator state at the start of the round for which they are selected
	pub struct ValidatorSnapshot<AccountId, Balance> {
		pub fee: Perbill,
		pub bond: Balance,
		pub nominators: Vec<Bond<AccountId, Balance>>,
		pub total: Balance,
	}

	#[derive(Encode, Decode, RuntimeDebug)]
	/// Global validator state with commission fee, bonded stake, and nominations
	pub struct Validator<AccountId, Balance> {
		pub id: AccountId,
		pub fee: Perbill,
		pub bond: Balance,
		pub nominators: OrderedSet<Bond<AccountId, Balance>>,
		pub total: Balance,
		pub state: ValidatorStatus,
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
			matches!(self.state, ValidatorStatus::Leaving(_))
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
				.map(|x| {
					if x.owner == nominator {
						// new amount must be greater or will underflow
						difference = amount - x.amount;
						Bond {
							owner: x.owner.clone(),
							amount,
						}
					} else {
						x.clone()
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
		pub fn leave_candidates(&mut self, round: RoundIndex) {
			self.state = ValidatorStatus::Leaving(round);
		}
	}

	impl<A: Clone, B: Copy> From<Validator<A, B>> for ValidatorSnapshot<A, B> {
		fn from(other: Validator<A, B>) -> ValidatorSnapshot<A, B> {
			ValidatorSnapshot {
				fee: other.fee,
				bond: other.bond,
				nominators: other.nominators.0,
				total: other.total,
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
					owner: validator,
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
				None
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
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	type Candidate<T> = Validator<<T as frame_system::Config>::T::AccountId, BalanceOf<T>>;

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency type
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// Number of blocks per round
		type BlocksPerRound: Get<u32>;
		/// Number of rounds that validators remain bonded before exit request is executed
		type BondDuration: Get<RoundIndex>;
		/// Maximum validators per round
		type MaxValidators: Get<u32>;
		/// Maximum nominators per validator
		type MaxNominatorsPerValidator: Get<u32>;
		/// Maximum validators per nominator
		type MaxValidatorsPerNominator: Get<u32>;
		/// Maximum fee for any validator
		type MaxFee: Get<Perbill>;
		/// Minimum stake for any registered on-chain account to become a validator
		type MinValidatorStk: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to nominate
		type MinNomination: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to become a nominator
		type MinNominatorStk: Get<BalanceOf<Self>>;
	}

	#[pallet::error]
	pub enum Error<T> {
		// Nominator Does Not Exist
		NominatorDNE,
		CandidateDNE,
		NominatorExists,
		CandidateExists,
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
		NominationDNE,
		Underflow,
		CannotSwitchToSameNomination,
		InvalidSchedule,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Starting Block, Round, Number of Validators, Total Balance
		NewRound(T::BlockNumber, RoundIndex, u32, BalanceOf<T>),
		/// Account, Amount Locked, New Total Amt Locked
		JoinedValidatorCandidates(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Round, Validator Account, Total Exposed Amount (includes all nominations)
		ValidatorChosen(RoundIndex, T::AccountId, BalanceOf<T>),
		/// Validator Account, Old Bond, New Bond
		ValidatorBondedMore(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Validator Account, Old Bond, New Bond
		ValidatorBondedLess(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		ValidatorWentOffline(RoundIndex, T::AccountId),
		ValidatorBackOnline(RoundIndex, T::AccountId),
		/// Round, Validator Account, Scheduled Exit
		ValidatorScheduledExit(RoundIndex, T::AccountId, RoundIndex),
		/// Account, Amount Unlocked, New Total Amt Locked
		ValidatorLeft(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		// Nominator, Validator, Old Nomination, New Nomination
		NominationIncreased(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		// Nominator, Validator, Old Nomination, New Nomination
		NominationDecreased(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		// Nominator, Swapped Amount, Old Nominator, New Nominator
		NominationSwapped(T::AccountId, BalanceOf<T>, T::AccountId, T::AccountId),
		/// Nominator, Amount Staked
		NominatorJoined(T::AccountId, BalanceOf<T>),
		/// Nominator, Amount Unstaked
		NominatorLeft(T::AccountId, BalanceOf<T>),
		/// Nominator, Amount Locked, Validator, New Total Amt Locked
		ValidatorNominated(T::AccountId, BalanceOf<T>, T::AccountId, BalanceOf<T>),
		/// Nominator, Validator, Amount Unstaked, New Total Amt Staked for Validator
		NominatorLeftValidator(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Paid the account (nominator or validator) the balance as liquid rewards
		Rewarded(T::AccountId, BalanceOf<T>),
		/// Round inflation range set with the provided annual inflation range
		RoundInflationSet(Perbill, Perbill, Perbill),
		/// Staking expectations set
		StakeExpectationsSet(BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: T::BlockNumber) {
			if (n % T::BlocksPerRound::get().into()).is_zero() {
				let next = <Round<T>>::get() + 1;
				// pay all stakers for T::BondDuration rounds ago
				Self::pay_stakers(next);
				// execute all delayed validator exits
				Self::execute_delayed_validator_exits(next);
				// insert exposure for next validator set
				let (validator_count, total_staked) = Self::best_candidates_become_validators(next);
				// start next round
				<Round<T>>::put(next);
				// snapshot total stake
				<Staked<T>>::insert(next, <Total<T>>::get());
				Self::deposit_event(Event::NewRound(n, next, validator_count, total_staked));
			}
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn round)]
	type Round<T: Config> = StorageValue<_, RoundIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nominators)]
	type Nominators<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Nominator<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn candidates)]
	type Candidates<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Candidate<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn validators)]
	type Validators<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total)]
	type Total<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn candidate_pool)]
	type CandidatePool<T: Config> =
		StorageValue<_, OrderedSet<Bond<T::AccountId, BalanceOf<T>>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn exit_queue)]
	type ExitQueue<T: Config> =
		StorageValue<_, OrderedSet<Bond<T::AccountId, RoundIndex>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn at_stake)]
	pub type AtStake<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		RoundIndex,
		Twox64Concat,
		T::AccountId,
		ValidatorSnapshot<T::AccountId, BalanceOf<T>>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn staked)]
	pub type Staked<T: Config> = StorageMap<_, Twox64Concat, RoundIndex, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn inflation_config)]
	pub type InflationConfig<T: Config> = StorageValue<_, InflationInfo<BalanceOf<T>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn points)]
	pub type Points<T: Config> = StorageMap<_, Twox64Concat, RoundIndex, RewardPoint, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn awarded_pts)]
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
		pub stakers: Vec<(T::AccountId, Option<T::AccountId>, BalanceOf<T>)>,
		pub inflation_config: InflationInfo<BalanceOf<T>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				stakers: vec![],
				..Default::default()
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<InflationConfig<T>>::put(self.inflation_config.clone());
			for &(ref actor, ref opt_val, balance) in &self.stakers {
				assert!(
					T::Currency::free_balance(&actor) >= balance,
					"Account does not have enough balance to bond."
				);
				let _ = if let Some(nominated_val) = opt_val {
					<Pallet<T>>::join_nominators(
						T::Origin::from(Some(actor.clone()).into()),
						nominated_val.clone(),
						balance,
					)
				} else {
					<Pallet<T>>::join_candidates(
						T::Origin::from(Some(actor.clone()).into()),
						Perbill::zero(), // default fee for validators registered at genesis is 0%
						balance,
					)
				};
			}
			// Choose top `MaxValidator`s from validator candidates
			let (v_count, total_staked) = <Pallet<T>>::best_candidates_become_validators(1u32);
			// Start Round 1 at Block 0
			<Round<T>>::put(1u32);
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
		/// Set the expectations for total staked. These expectations determine the issuance for
		/// the round according to logic in `fn compute_issuance`
		#[pallet::weight(0)]
		pub fn set_staking_expectations(
			origin: OriginFor<T>,
			expectations: Range<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(expectations.is_valid(), Error::<T>::InvalidSchedule);
			let mut config = <InflationConfig<T>>::get();
			config.set_expectations(expectations);
			Self::deposit_event(Event::StakeExpectationsSet(
				config.expect.min,
				config.expect.ideal,
				config.expect.max,
			));
			<InflationConfig<T>>::put(config);
			Ok(().into())
		}
		#[pallet::weight(0)]
		pub fn set_inflation(
			origin: OriginFor<T>,
			schedule: Range<Perbill>,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(schedule.is_valid(), Error::<T>::InvalidSchedule);
			let mut config = <InflationConfig<T>>::get();
			config.set_annual_rate::<T>(schedule);
			Self::deposit_event(Event::RoundInflationSet(
				config.round.min,
				config.round.ideal,
				config.round.max,
			));
			<InflationConfig<T>>::put(config);
			Ok(().into())
		}
		#[pallet::weight(0)]
		pub fn join_candidates(
			origin: OriginFor<T>,
			fee: Perbill,
			bond: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			ensure!(!Self::is_candidate(&acc), Error::<T>::CandidateExists);
			ensure!(!Self::is_nominator(&acc), Error::<T>::NominatorExists);
			ensure!(fee <= T::MaxFee::get(), Error::<T>::FeeOverMax);
			ensure!(
				bond >= T::MinValidatorStk::get(),
				Error::<T>::ValBondBelowMin
			);
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidates.insert(Bond {
					owner: acc.clone(),
					amount: bond
				}),
				Error::<T>::CandidateExists
			);
			T::Currency::reserve(&acc, bond)?;
			let candidate: Candidate<T> = Validator::new(acc.clone(), fee, bond);
			let new_total = <Total<T>>::get() + bond;
			<Total<T>>::put(new_total);
			<Candidates<T>>::insert(&acc, candidate);
			<CandidatePool<T>>::put(candidates);
			Self::deposit_event(Event::JoinedValidatorCandidates(acc, bond, new_total));
			Ok(().into())
		}
		/// Request to leave the set of candidates. If successful, the account is immediately
		/// removed from the candidate pool to prevent selection as a validator, but unbonding is
		/// executed with a delay of `BondDuration` rounds.
		#[pallet::weight(0)]
		pub fn leave_candidates(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let validator = ensure_signed(origin)?;
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(), Error::<T>::AlreadyLeaving);
			let mut exits = <ExitQueue<T>>::get();
			let now = <Round<T>>::get();
			let when = now + T::BondDuration::get();
			ensure!(
				exits.insert(Bond {
					owner: validator.clone(),
					amount: when
				}),
				Error::<T>::AlreadyLeaving
			);
			state.leave_candidates(when);
			let mut candidates = <CandidatePool<T>>::get();
			if candidates.remove(&Bond::from_owner(validator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<ExitQueue<T>>::put(exits);
			<Candidates<T>>::insert(&validator, state);
			Self::deposit_event(Event::ValidatorScheduledExit(now, validator, when));
			Ok(().into())
		}
		/// Temporarily leave the set of validator candidates without unbonding
		#[pallet::weight(0)]
		pub fn go_offline(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let validator = ensure_signed(origin)?;
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(state.is_active(), Error::<T>::AlreadyOffline);
			state.go_offline();
			let mut candidates = <CandidatePool<T>>::get();
			// TODO: investigate possible bug in this next line
			if candidates.remove(&Bond::from_owner(validator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<Candidates<T>>::insert(&validator, state);
			Self::deposit_event(Event::ValidatorWentOffline(<Round<T>>::get(), validator));
			Ok(().into())
		}
		/// Rejoin the set of validator candidates if previously had called `go_offline`
		#[pallet::weight(0)]
		pub fn go_online(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let validator = ensure_signed(origin)?;
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_active(), Error::<T>::AlreadyActive);
			ensure!(!state.is_leaving(), Error::<T>::CannotActivateIfLeaving);
			state.go_online();
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidates.insert(Bond {
					owner: validator.clone(),
					amount: state.total
				}),
				Error::<T>::AlreadyActive
			);
			<CandidatePool<T>>::put(candidates);
			<Candidates<T>>::insert(&validator, state);
			Self::deposit_event(Event::ValidatorBackOnline(<Round<T>>::get(), validator));
			Ok(().into())
		}
		/// Bond more for validator candidates
		#[pallet::weight(0)]
		pub fn candidate_bond_more(
			origin: OriginFor<T>,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let validator = ensure_signed(origin)?;
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(), Error::<T>::CannotActivateIfLeaving);
			T::Currency::reserve(&validator, more)?;
			let before = state.bond;
			state.bond_more(more);
			let after = state.bond;
			if state.is_active() {
				Self::update_active(validator.clone(), state.total);
			}
			<Candidates<T>>::insert(&validator, state);
			Self::deposit_event(Event::ValidatorBondedMore(validator, before, after));
			Ok(().into())
		}
		/// Bond less for validator candidates
		#[pallet::weight(0)]
		pub fn candidate_bond_less(
			origin: OriginFor<T>,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let validator = ensure_signed(origin)?;
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(), Error::<T>::CannotActivateIfLeaving);
			let before = state.bond;
			let after = state.bond_less(less).ok_or(Error::<T>::Underflow)?;
			ensure!(
				after >= T::MinValidatorStk::get(),
				Error::<T>::ValBondBelowMin
			);
			T::Currency::unreserve(&validator, less);
			if state.is_active() {
				Self::update_active(validator.clone(), state.total);
			}
			<Candidates<T>>::insert(&validator, state);
			Self::deposit_event(Event::ValidatorBondedLess(validator, before, after));
			Ok(().into())
		}
		/// Join the set of nominators
		#[pallet::weight(0)]
		pub fn join_nominators(
			origin: OriginFor<T>,
			validator: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			ensure!(
				amount >= T::MinNominatorStk::get(),
				Error::<T>::NomBondBelowMin
			);
			ensure!(!Self::is_nominator(&acc), Error::<T>::NominatorExists);
			ensure!(!Self::is_candidate(&acc), Error::<T>::CandidateExists);
			Self::nominator_joins_validator(acc.clone(), amount, validator.clone())?;
			<Nominators<T>>::insert(&acc, Nominator::new(validator, amount));
			Self::deposit_event(Event::NominatorJoined(acc, amount));
			Ok(().into())
		}
		/// Leave the set of nominators and, by implication, revoke all ongoing nominations
		#[pallet::weight(0)]
		pub fn leave_nominators(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			let nominator = <Nominators<T>>::get(&acc).ok_or(Error::<T>::NominatorDNE)?;
			for bond in nominator.nominations.0 {
				Self::nominator_leaves_validator(acc.clone(), bond.owner.clone())?;
			}
			<Nominators<T>>::remove(&acc);
			Self::deposit_event(Event::NominatorLeft(acc, nominator.total));
			Ok(().into())
		}
		/// Nominate a new validator candidate if already nominating
		#[pallet::weight(0)]
		pub fn nominate_new(
			origin: OriginFor<T>,
			validator: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			ensure!(
				amount >= T::MinNomination::get(),
				Error::<T>::NominationBelowMin
			);
			let mut nominator = <Nominators<T>>::get(&acc).ok_or(Error::<T>::NominatorDNE)?;
			ensure!(
				(nominator.nominations.0.len() as u32) < T::MaxValidatorsPerNominator::get(),
				Error::<T>::ExceedMaxValidatorsPerNom
			);
			let mut state = <Candidates<T>>::get(&validator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(
				nominator.add_nomination(Bond {
					owner: validator.clone(),
					amount
				}),
				Error::<T>::AlreadyNominatedValidator
			);
			let nomination = Bond {
				owner: acc.clone(),
				amount,
			};
			ensure!(
				(state.nominators.0.len() as u32) < T::MaxNominatorsPerValidator::get(),
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
			Self::deposit_event(Event::ValidatorNominated(acc, amount, validator, new_total));
			Ok(().into())
		}
		/// Revoke an existing nomination
		#[pallet::weight(0)]
		pub fn revoke_nomination(
			origin: OriginFor<T>,
			validator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			Self::nominator_revokes_validator(ensure_signed(origin)?, validator)
		}
		/// Bond more for nominators with respect to a specific validator candidate
		#[pallet::weight(0)]
		pub fn nominator_bond_more(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let nominator = ensure_signed(origin)?;
			let mut nominations =
				<Nominators<T>>::get(&nominator).ok_or(Error::<T>::NominatorDNE)?;
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
			Self::deposit_event(Event::NominationIncreased(
				nominator, candidate, before, after,
			));
			Ok(().into())
		}
		/// Bond less for nominators with respect to a specific nominator candidate
		#[pallet::weight(0)]
		pub fn nominator_bond_less(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let nominator = ensure_signed(origin)?;
			let mut nominations =
				<Nominators<T>>::get(&nominator).ok_or(Error::<T>::NominatorDNE)?;
			let mut validator = <Candidates<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			let remaining = nominations
				.dec_nomination(candidate.clone(), less)
				.ok_or(Error::<T>::NominationDNE)?
				.ok_or(Error::<T>::Underflow)?;
			ensure!(
				remaining >= T::MinNomination::get(),
				Error::<T>::NominationBelowMin
			);
			ensure!(
				nominations.total >= T::MinNominatorStk::get(),
				Error::<T>::NomBondBelowMin
			);
			T::Currency::unreserve(&nominator, less);
			let before = validator.total;
			validator.dec_nominator(nominator.clone(), less);
			let after = validator.total;
			if validator.is_active() {
				Self::update_active(candidate.clone(), validator.total);
			}
			<Candidates<T>>::insert(&candidate, validator);
			<Nominators<T>>::insert(&nominator, nominations);
			Self::deposit_event(Event::NominationDecreased(
				nominator, candidate, before, after,
			));
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
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
				owner: candidate,
				amount: total,
			});
			<CandidatePool<T>>::put(candidates);
		}
		// Calculate round issuance based on total staked for the given round
		fn compute_issuance(staked: BalanceOf<T>) -> BalanceOf<T> {
			let config = <InflationConfig<T>>::get();
			let round_issuance = crate::inflation::round_issuance_range::<T>(config.round);
			if staked < config.expect.min {
				return round_issuance.min;
			} else if staked > config.expect.max {
				return round_issuance.max;
			} else {
				// TODO: split up into 3 branches
				// 1. min < staked < ideal
				// 2. ideal < staked < max
				// 3. staked == ideal
				return round_issuance.ideal;
			}
		}
		fn nominator_joins_validator(
			nominator: T::AccountId,
			amount: BalanceOf<T>,
			validator: T::AccountId,
		) -> DispatchResultWithPostInfo {
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
				(state.nominators.0.len() as u32) <= T::MaxNominatorsPerValidator::get(),
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
			Self::deposit_event(Event::ValidatorNominated(
				nominator, amount, validator, new_total,
			));
			Ok(().into())
		}
		fn nominator_revokes_validator(
			acc: T::AccountId,
			validator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let mut nominator = <Nominators<T>>::get(&acc).ok_or(Error::<T>::NominatorDNE)?;
			let remaining = nominator
				.rm_nomination(validator.clone())
				.ok_or(Error::<T>::NominationDNE)?;
			ensure!(
				remaining >= T::MinNominatorStk::get(),
				Error::<T>::NomBondBelowMin
			);
			Self::nominator_leaves_validator(acc.clone(), validator)?;
			<Nominators<T>>::insert(&acc, nominator);
			Ok(().into())
		}
		fn nominator_leaves_validator(
			nominator: T::AccountId,
			validator: T::AccountId,
		) -> DispatchResultWithPostInfo {
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
			Self::deposit_event(Event::NominatorLeftValidator(
				nominator,
				validator,
				nominator_stake,
				new_total,
			));
			Ok(().into())
		}
		fn pay_stakers(next: RoundIndex) {
			let mint = |amt: BalanceOf<T>, to: T::AccountId| {
				if amt > T::Currency::minimum_balance() {
					if let Ok(imb) = T::Currency::deposit_into_existing(&to, amt) {
						Self::deposit_event(Event::Rewarded(to.clone(), imb.peek()));
					}
				}
			};
			let duration = T::BondDuration::get();
			if next > duration {
				let round_to_payout = next - duration;
				let total = <Points<T>>::get(round_to_payout);
				let total_staked = <Staked<T>>::get(round_to_payout);
				let issuance = Self::compute_issuance(total_staked);
				for (val, pts) in <AwardedPts<T>>::drain_prefix(round_to_payout) {
					let pct_due = Perbill::from_rational_approximation(pts, total);
					let mut amt_due = pct_due * issuance;
					if amt_due <= T::Currency::minimum_balance() {
						continue;
					}
					// Take the snapshot of block author and nominations
					let state = <AtStake<T>>::take(round_to_payout, &val);
					if state.nominators.is_empty() {
						// solo validator with no nominators
						mint(amt_due, val.clone());
					} else {
						// pay validator first; commission + due_portion
						let val_pct = Perbill::from_rational_approximation(state.bond, state.total);
						let commission = state.fee * amt_due;
						let val_due = if commission > T::Currency::minimum_balance() {
							amt_due -= commission;
							(val_pct * amt_due) + commission
						} else {
							// commission is negligible so not applied
							val_pct * amt_due
						};
						mint(val_due, val.clone());
						// pay nominators due portion
						for Bond { owner, amount } in state.nominators {
							let percent = Perbill::from_rational_approximation(amount, state.total);
							let due = percent * amt_due;
							mint(due, owner);
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
									if let Some(remaining) =
										nominator.rm_nomination(x.owner.clone())
									{
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
							Self::deposit_event(Event::ValidatorLeft(
								x.owner,
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
			// snapshot exposure for round for weighting reward distribution
			for account in validators.iter() {
				let state = <Candidates<T>>::get(&account)
					.expect("all members of CandidateQ must be candidates");
				let amount = state.total;
				let exposure: ValidatorSnapshot<T::AccountId, BalanceOf<T>> = state.into();
				<AtStake<T>>::insert(next, account, exposure);
				all_validators += 1u32;
				total += amount;
				Self::deposit_event(Event::ValidatorChosen(next, account.clone(), amount));
			}
			validators.sort();
			// insert canonical validator set
			<Validators<T>>::put(validators);
			(all_validators, total)
		}
	}
	/// Add reward points to block authors:
	/// * 20 points to the block producer for producing a block in the chain
	impl<T: Config> author_inherent::EventHandler<T::AccountId> for Pallet<T> {
		fn note_author(author: T::AccountId) {
			let now = <Round<T>>::get();
			let score_plus_20 = <AwardedPts<T>>::get(now, &author) + 20;
			<AwardedPts<T>>::insert(now, author, score_plus_20);
			<Points<T>>::mutate(now, |x| *x += 20);
		}
	}

	impl<T: Config> author_inherent::CanAuthor<T::AccountId> for Pallet<T> {
		fn can_author(account: &T::AccountId) -> bool {
			Self::is_validator(account)
		}
	}
}
