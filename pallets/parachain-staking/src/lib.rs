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
//! Minimal staking pallet that implements collator selection by total backed stake.
//! The main difference between this pallet and `frame/pallet-staking` is that this pallet
//! uses direct delegation. Nominators choose exactly who they nominate and with what stake.
//! This is different from `frame/pallet-staking` where you approval vote and then run Phragmen.
//!
//! ### Rules
//! There is a new round every `BlocksPerRound` blocks.
//!
//! At the start of every round,
//! * issuance is distributed to collators for `BondDuration` rounds ago
//! in proportion to the points they received in that round (for authoring blocks)
//! * queued collator exits are executed
//! * a new set of collators is chosen from the candidates
//!
//! To join the set of candidates, an account must call `join_candidates` with
//! stake >= `MinCollatorCandidateStk` and fee <= `MaxFee`. The fee is taken off the top
//! of any rewards for the collator before the remaining rewards are distributed
//! in proportion to stake to all nominators (including the collator, who always
//! self-nominates).
//!
//! To leave the set of candidates, the collator calls `leave_candidates`. If the call succeeds,
//! the collator is removed from the pool of candidates so they cannot be selected for future
//! collator sets, but they are not unstaked until `BondDuration` rounds later. The exit request is
//! stored in the `ExitQueue` and processed `BondDuration` rounds later to unstake the collator
//! and all of its nominators.
//!
//! To join the set of nominators, an account must call `join_nominators` with
//! stake >= `MinNominatorStk`. There are also runtime methods for nominating additional collators
//! and revoking nominations.

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
	use super::*;
	use crate::set::OrderedSet;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{Currency, Get, Imbalance, ReservableCurrency};
	use frame_system::pallet_prelude::*;
	use parity_scale_codec::{Decode, Encode};
	use sp_runtime::{
		traits::{AtLeast32BitUnsigned, Zero},
		Perbill, RuntimeDebug,
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
		pub nominators: Vec<Bond<AccountId, Balance>>,
		pub total: Balance,
	}

	#[derive(Encode, Decode, RuntimeDebug)]
	/// Global collator state with commission fee, bonded stake, and nominations
	pub struct Collator<AccountId, Balance> {
		pub id: AccountId,
		pub bond: Balance,
		pub nominators: OrderedSet<Bond<AccountId, Balance>>,
		pub total: Balance,
		pub state: CollatorStatus,
	}

	impl<
			A: Ord + Clone,
			B: AtLeast32BitUnsigned + Ord + Copy + sp_std::ops::AddAssign + sp_std::ops::SubAssign,
		> Collator<A, B>
	{
		pub fn new(id: A, bond: B) -> Self {
			let total = bond;
			Collator {
				id,
				bond,
				nominators: OrderedSet::new(),
				total,
				state: CollatorStatus::default(), // default active
			}
		}
		pub fn is_active(&self) -> bool {
			matches!(self.state, CollatorStatus::Active)
		}
		pub fn is_leaving(&self) -> bool {
			matches!(self.state, CollatorStatus::Leaving(_))
		}
		pub fn bond_more(&mut self, more: B) {
			self.bond += more;
			self.total += more;
		}
		// Returns None if underflow or less == self.bond (in which case collator should leave)
		pub fn bond_less(&mut self, less: B) -> Option<B> {
			if self.bond > less {
				self.bond -= less;
				self.total -= less;
				Some(self.bond)
			} else {
				None
			}
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
			self.state = CollatorStatus::Idle;
		}
		pub fn go_online(&mut self) {
			self.state = CollatorStatus::Active;
		}
		pub fn leave_candidates(&mut self, round: RoundIndex) {
			self.state = CollatorStatus::Leaving(round);
		}
	}

	impl<A: Clone, B: Copy> From<Collator<A, B>> for CollatorSnapshot<A, B> {
		fn from(other: Collator<A, B>) -> CollatorSnapshot<A, B> {
			CollatorSnapshot {
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
		pub fn new(collator: AccountId, amount: Balance) -> Self {
			Nominator {
				nominations: OrderedSet::from(vec![Bond {
					owner: collator,
					amount,
				}]),
				total: amount,
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
		pub fn rm_nomination(&mut self, collator: AccountId) -> Option<Balance> {
			let mut amt: Option<Balance> = None;
			let nominations = self
				.nominations
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
				self.nominations = OrderedSet::from(nominations);
				self.total -= balance;
				Some(self.total)
			} else {
				None
			}
		}
		// Returns None if nomination not found
		pub fn inc_nomination(&mut self, collator: AccountId, more: Balance) -> Option<Balance> {
			for x in &mut self.nominations.0 {
				if x.owner == collator {
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
			collator: AccountId,
			less: Balance,
		) -> Option<Option<Balance>> {
			for x in &mut self.nominations.0 {
				if x.owner == collator {
					if x.amount > less {
						x.amount -= less;
						self.total -= less;
						return Some(Some(x.amount));
					} else {
						// underflow error; should rm entire nomination if x.amount == collator
						return Some(None);
					}
				}
			}
			None
		}
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
			RoundInfo::new(1u32, 1u32.into(), 20u32.into())
		}
	}

	#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, Default)]
	/// Total collator candidates (NOT active collators, for this see TotalSelected storage item)
	pub struct CollatorCount {
		pub candidate_count: u32,
		pub max_collator_candidates: u32,
	}

	impl CollatorCount {
		pub fn new(candidate_count: u32, max_collator_candidates: u32) -> Self {
			CollatorCount {
				candidate_count,
				max_collator_candidates,
			}
		}

		pub fn can_increment(&self) -> bool {
			self.candidate_count < self.max_collator_candidates
				|| self.max_collator_candidates == 0u32
		}

		pub fn increment<T: Config>(&mut self) {
			if self.max_collator_candidates == 0u32 {
				// must initialize if not initialized in genesis build b/c derived default is 0u32
				self.max_collator_candidates = T::MaxCollatorCandidates::get();
			}
			self.candidate_count += 1u32;
		}

		pub fn decrement(&mut self) {
			self.candidate_count -= 1u32;
		}
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
		/// Maximum Collator Candidates, bound collator candidate pool storage item
		type MaxCollatorCandidates: Get<u32>;
		/// Minimum number of blocks per round
		type MinBlocksPerRound: Get<u32>;
		/// Default number of blocks per round at genesis
		type DefaultBlocksPerRound: Get<u32>;
		/// Number of rounds that collators remain bonded before exit request is executed
		type BondDuration: Get<RoundIndex>;
		/// Minimum number of selected candidates every round
		type MinSelectedCandidates: Get<u32>;
		/// Maximum nominators per collator
		type MaxNominatorsPerCollator: Get<u32>;
		/// Maximum collators per nominator
		type MaxCollatorsPerNominator: Get<u32>;
		/// Commission due to collators, set at genesis
		type DefaultCollatorCommission: Get<Perbill>;
		/// Minimum stake required for any account to be in `SelectedCandidates` for the round
		type MinCollatorStk: Get<BalanceOf<Self>>;
		/// Minimum stake required for any account to be a collator candidate
		type MinCollatorCandidateStk: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to nominate
		type MinNomination: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to become a nominator
		type MinNominatorStk: Get<BalanceOf<Self>>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		CannotSetMaxCandidatesBelowTotalSelected,
		MaxCollatorCandidatesOnlyIncreases,
		NominatorDNE,
		CandidateDNE,
		NominatorExists,
		CandidateExists,
		ValBondBelowMin,
		NomBondBelowMin,
		NominationBelowMin,
		AlreadyOffline,
		AlreadyActive,
		AlreadyLeaving,
		TooManyNominators,
		CannotActivateIfLeaving,
		ExceedsMaxCollatorCandidates,
		ExceedMaxCollatorsPerNom,
		AlreadyNominatedCollator,
		NominationDNE,
		Underflow,
		InvalidSchedule,
		CannotSetBelowMin,
		TooLowCollatorCandidateCountToJoinCandidates,
		TooLowCollatorCandidateCountToLeaveCandidates,
		TooLowNominationCountToNominate,
		TooLowCollatorCountToNominate,
		TooLowNominatorCountToLeaveCandidates,
		TooLowNominationCountToLeaveNominators,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Starting Block, Round, Number of Collators Selected, Total Balance
		NewRound(T::BlockNumber, RoundIndex, u32, BalanceOf<T>),
		/// Account, Amount Locked, New Total Amt Locked
		JoinedCollatorCandidates(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Round, Collator Account, Total Exposed Amount (includes all nominations)
		CollatorChosen(RoundIndex, T::AccountId, BalanceOf<T>),
		/// Collator Account, Old Bond, New Bond
		CollatorBondedMore(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Collator Account, Old Bond, New Bond
		CollatorBondedLess(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		CollatorWentOffline(RoundIndex, T::AccountId),
		CollatorBackOnline(RoundIndex, T::AccountId),
		/// Round, Collator Account, Scheduled Exit
		CollatorScheduledExit(RoundIndex, T::AccountId, RoundIndex),
		/// Account, Amount Unlocked, New Total Amt Locked
		CollatorLeft(T::AccountId, BalanceOf<T>, BalanceOf<T>),
		// Nominator, Collator, Old Nomination, New Nomination
		NominationIncreased(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		// Nominator, Collator, Old Nomination, New Nomination
		NominationDecreased(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Nominator, Amount Unstaked
		NominatorLeft(T::AccountId, BalanceOf<T>),
		/// Nominator, Amount Locked, Collator, New Total Amt backing Collator
		Nomination(T::AccountId, BalanceOf<T>, T::AccountId, BalanceOf<T>),
		/// Nominator, Collator, Amount Unstaked, New Total Amt Staked for Collator
		NominatorLeftCollator(T::AccountId, T::AccountId, BalanceOf<T>, BalanceOf<T>),
		/// Paid the account (nominator or collator) the balance as liquid rewards
		Rewarded(T::AccountId, BalanceOf<T>),
		/// Annual inflation input (first 3) was used to derive new per-round inflation (last 3)
		InflationSet(Perbill, Perbill, Perbill, Perbill, Perbill, Perbill),
		/// Staking expectations set
		StakeExpectationsSet(BalanceOf<T>, BalanceOf<T>, BalanceOf<T>),
		/// Set total selected candidates to this value [old, new]
		TotalSelectedSet(u32, u32),
		/// Set maximum collator candidates to this value [old, new]
		MaxCollatorCandidatesSet(u32, u32),
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
		fn on_finalize(n: T::BlockNumber) {
			let mut round = <Round<T>>::get();
			if round.should_update(n) {
				// mutate round
				round.update(n);
				// pay all stakers for T::BondDuration rounds ago
				Self::pay_stakers(round.current);
				// execute all delayed collator exits
				Self::execute_delayed_collator_exits(round.current);
				// select top collator candidates for next round
				let (collator_count, total_staked) = Self::select_top_candidates(round.current);
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
	#[pallet::getter(fn round)]
	/// Current round index and next round scheduled transition
	type Round<T: Config> = StorageValue<_, RoundInfo<T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn nominator_state)]
	/// Get nominator state associated with an account if account is nominating else None
	type NominatorState<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Nominator<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn collator_state)]
	/// Get collator state associated with an account if account is collating else None
	type CollatorState<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Collator<T::AccountId, BalanceOf<T>>,
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
	#[pallet::getter(fn candidate_count)]
	/// Total number of collator candidates, TODO: replace with OrderedSet + BoundedVec instead
	type CandidateCount<T: Config> = StorageValue<_, CollatorCount, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn exit_queue)]
	/// A queue of collators awaiting exit `BondDuration` delay after request
	type ExitQueue<T: Config> =
		StorageValue<_, OrderedSet<Bond<T::AccountId, RoundIndex>>, ValueQuery>;

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
			// Set initial collator candidate count
			<CandidateCount<T>>::put(CollatorCount::new(0u32, T::MaxCollatorCandidates::get()));
			let mut collator_count = 0u32;
			let mut col_nominator_count: BTreeMap<T::AccountId, u32> = BTreeMap::new();
			let mut nom_nominator_count: BTreeMap<T::AccountId, u32> = BTreeMap::new();
			for &(ref actor, ref opt_val, balance) in &self.stakers {
				assert!(
					T::Currency::free_balance(&actor) >= balance,
					"Account does not have enough balance to bond."
				);
				if let Some(nominated_val) = opt_val {
					let cn_count = if let Some(x) = col_nominator_count.get(&nominated_val) {
						*x
					} else {
						0u32
					};
					let nn_count = if let Some(x) = nom_nominator_count.get(&actor) {
						*x
					} else {
						0u32
					};
					if let Ok(_) = <Pallet<T>>::nominate(
						T::Origin::from(Some(actor.clone()).into()),
						nominated_val.clone(),
						balance,
						cn_count,
						nn_count,
					) {
						if let Some(x) = col_nominator_count.get_mut(&nominated_val) {
							*x += 1u32;
						} else {
							col_nominator_count.insert(nominated_val.clone(), 1u32);
						};
						if let Some(x) = nom_nominator_count.get_mut(&actor) {
							*x += 1u32;
						} else {
							nom_nominator_count.insert(actor.clone(), 1u32);
						};
					}
				} else {
					collator_count += 1u32;
					let _ = <Pallet<T>>::join_candidates(
						T::Origin::from(Some(actor.clone()).into()),
						balance,
						collator_count - 1u32,
					);
				}
			}
			// Set collator commission to default config
			<CollatorCommission<T>>::put(T::DefaultCollatorCommission::get());
			// Set total selected candidates to minimum config
			<TotalSelected<T>>::put(T::MinSelectedCandidates::get());
			// Choose top TotalSelected collator candidates
			let (v_count, total_staked) = <Pallet<T>>::select_top_candidates(1u32);
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
		/// Set the expectations for total staked. These expectations determine the issuance for
		/// the round according to logic in `fn compute_issuance`
		#[pallet::weight(T::WeightInfo::set_staking_expectations())]
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
		/// Set the annual inflation rate to derive per-round inflation
		#[pallet::weight(T::WeightInfo::set_inflation())]
		pub fn set_inflation(
			origin: OriginFor<T>,
			schedule: Range<Perbill>,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(schedule.is_valid(), Error::<T>::InvalidSchedule);
			let mut config = <InflationConfig<T>>::get();
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
		#[pallet::weight(T::WeightInfo::set_total_selected())]
		/// Set the total number of collator candidates selected per round
		/// - changes are not applied until the start of the next round
		pub fn set_total_selected(origin: OriginFor<T>, new: u32) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(
				new >= T::MinSelectedCandidates::get(),
				Error::<T>::CannotSetBelowMin
			);
			let old = <TotalSelected<T>>::get();
			<TotalSelected<T>>::put(new);
			Self::deposit_event(Event::TotalSelectedSet(old, new));
			Ok(().into())
		}
		#[pallet::weight(T::WeightInfo::set_max_collator_candidates())]
		/// Set the maximum number of collator candidates to be allowed in the CandidatePool
		/// - current implementation requires new_max > old_max
		pub fn set_max_collator_candidates(
			origin: OriginFor<T>,
			new: u32,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(
				new >= <TotalSelected<T>>::get(),
				Error::<T>::CannotSetMaxCandidatesBelowTotalSelected
			);
			let mut count = <CandidateCount<T>>::get();
			let old = count.max_collator_candidates;
			// TODO: add path to set lower max collator candidates by immediately kicking lowest
			// new_max - old_max of existing candidates if old candidate pool is full
			ensure!(new > old, Error::<T>::MaxCollatorCandidatesOnlyIncreases);
			count.max_collator_candidates = new;
			<CandidateCount<T>>::put(count);
			Self::deposit_event(Event::MaxCollatorCandidatesSet(old, new));
			Ok(().into())
		}
		#[pallet::weight(T::WeightInfo::set_collator_commission())]
		/// Set the commission for all collators
		pub fn set_collator_commission(
			origin: OriginFor<T>,
			pct: Perbill,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			let old = <CollatorCommission<T>>::get();
			<CollatorCommission<T>>::put(pct);
			Self::deposit_event(Event::CollatorCommissionSet(old, pct));
			Ok(().into())
		}
		#[pallet::weight(T::WeightInfo::set_blocks_per_round())]
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
		#[pallet::weight(T::WeightInfo::force_leave_candidates(
			*collator_count,
			*nominator_count,
		))]
		/// Root dispatchable to force a collator candidate's immediate exit
		/// - `collator_count` weight hint is the number of collator candidates in CandidatePool
		/// - `nominator_count` weight hint is the number of nominators for the collator leaving
		pub fn force_leave_candidates(
			origin: OriginFor<T>,
			who: T::AccountId,
			collator_count: u32,
			nominator_count: u32,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			Self::collator_exit(who, true, Some(collator_count), Some(nominator_count))
		}
		/// Join the set of collator candidates
		/// - `collator_count` weight hint is the number of collator candidates in CandidatePool
		#[pallet::weight(T::WeightInfo::join_candidates(*collator_count))]
		pub fn join_candidates(
			origin: OriginFor<T>,
			bond: BalanceOf<T>,
			collator_count: u32,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			ensure!(!Self::is_candidate(&acc), Error::<T>::CandidateExists);
			ensure!(!Self::is_nominator(&acc), Error::<T>::NominatorExists);
			ensure!(
				bond >= T::MinCollatorCandidateStk::get(),
				Error::<T>::ValBondBelowMin
			);
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				collator_count >= candidates.0.len() as u32,
				Error::<T>::TooLowCollatorCandidateCountToJoinCandidates
			);
			ensure!(
				candidates.insert(Bond {
					owner: acc.clone(),
					amount: bond
				}),
				Error::<T>::CandidateExists
			);
			let mut count = <CandidateCount<T>>::get();
			ensure!(
				count.can_increment(), // count.candidate_count < count.max_collator_candidates
				Error::<T>::ExceedsMaxCollatorCandidates
			);
			count.increment::<T>();
			T::Currency::reserve(&acc, bond)?;
			let new_total = <Total<T>>::get() + bond;
			<Total<T>>::put(new_total);
			<CandidateCount<T>>::put(count);
			<CollatorState<T>>::insert(&acc, Collator::new(acc.clone(), bond));
			<CandidatePool<T>>::put(candidates);
			Self::deposit_event(Event::JoinedCollatorCandidates(acc, bond, new_total));
			Ok(().into())
		}
		/// Schedule exit from the set of candidates. If successful, the account is immediately
		/// removed from the candidate pool to prevent selection as a collator, but unbonding is
		/// executed after `BondDuration` rounds.
		/// - `collator_count` weight hint is the number of collator candidates in CandidatePool
		/// - `nominator_count` weight hint is the number of nominators for the collator leaving
		#[pallet::weight(
			T::WeightInfo::leave_candidates(*collator_count) +
			T::WeightInfo::force_leave_candidates(
				*collator_count,
				*nominator_count,
			)
		)]
		pub fn leave_candidates(
			origin: OriginFor<T>,
			collator_count: u32,
			nominator_count: u32,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(), Error::<T>::AlreadyLeaving);
			ensure!(
				nominator_count >= state.nominators.0.len() as u32,
				Error::<T>::TooLowNominationCountToLeaveNominators,
			);
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				collator_count >= candidates.0.len() as u32,
				Error::<T>::TooLowCollatorCandidateCountToLeaveCandidates
			);
			let mut exits = <ExitQueue<T>>::get();
			let now = <Round<T>>::get().current;
			let when = now + T::BondDuration::get();
			ensure!(
				exits.insert(Bond {
					owner: collator.clone(),
					amount: when
				}),
				Error::<T>::AlreadyLeaving
			);
			state.leave_candidates(when);
			if candidates.remove(&Bond::from_owner(collator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<ExitQueue<T>>::put(exits);
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorScheduledExit(now, collator, when));
			Ok(().into())
		}
		/// Temporarily leave the set of collator candidates without unbonding
		#[pallet::weight(T::WeightInfo::go_offline())]
		pub fn go_offline(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(state.is_active(), Error::<T>::AlreadyOffline);
			state.go_offline();
			let mut candidates = <CandidatePool<T>>::get();
			// TODO: investigate possible bug in this next line
			if candidates.remove(&Bond::from_owner(collator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorWentOffline(
				<Round<T>>::get().current,
				collator,
			));
			Ok(().into())
		}
		/// Rejoin the set of collator candidates if previously had called `go_offline`
		#[pallet::weight(T::WeightInfo::go_online())]
		pub fn go_online(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_active(), Error::<T>::AlreadyActive);
			ensure!(!state.is_leaving(), Error::<T>::CannotActivateIfLeaving);
			state.go_online();
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidates.insert(Bond {
					owner: collator.clone(),
					amount: state.total
				}),
				Error::<T>::AlreadyActive
			);
			<CandidatePool<T>>::put(candidates);
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorBackOnline(
				<Round<T>>::get().current,
				collator,
			));
			Ok(().into())
		}
		/// Bond more for collator candidates
		#[pallet::weight(T::WeightInfo::candidate_bond_more())]
		pub fn candidate_bond_more(
			origin: OriginFor<T>,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(), Error::<T>::CannotActivateIfLeaving);
			T::Currency::reserve(&collator, more)?;
			let before = state.bond;
			state.bond_more(more);
			let after = state.bond;
			if state.is_active() {
				Self::update_active(collator.clone(), state.total);
			}
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorBondedMore(collator, before, after));
			Ok(().into())
		}
		/// Bond less for collator candidates
		#[pallet::weight(T::WeightInfo::candidate_bond_less())]
		pub fn candidate_bond_less(
			origin: OriginFor<T>,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(!state.is_leaving(), Error::<T>::CannotActivateIfLeaving);
			let before = state.bond;
			let after = state.bond_less(less).ok_or(Error::<T>::Underflow)?;
			ensure!(
				after >= T::MinCollatorCandidateStk::get(),
				Error::<T>::ValBondBelowMin
			);
			T::Currency::unreserve(&collator, less);
			if state.is_active() {
				Self::update_active(collator.clone(), state.total);
			}
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::CollatorBondedLess(collator, before, after));
			Ok(().into())
		}
		/// If caller is not a nominator, then join the set of nominators
		/// If caller is a nominator, then makes nomination to change their nomination state
		/// - `nomination_count` weight hint is the current number of nominations for the caller
		#[pallet::weight(T::WeightInfo::nominate(*collator_nominator_count, *nomination_count))]
		pub fn nominate(
			origin: OriginFor<T>,
			collator: T::AccountId,
			amount: BalanceOf<T>,
			collator_nominator_count: u32,
			nomination_count: u32,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			let nominator = if let Some(mut nom) = <NominatorState<T>>::get(&acc) {
				// nomination after first
				ensure!(
					amount >= T::MinNomination::get(),
					Error::<T>::NominationBelowMin
				);
				ensure!(
					nomination_count >= nom.nominations.0.len() as u32,
					Error::<T>::TooLowNominationCountToNominate
				);
				ensure!(
					(nom.nominations.0.len() as u32) < T::MaxCollatorsPerNominator::get(),
					Error::<T>::ExceedMaxCollatorsPerNom
				);
				ensure!(
					nom.add_nomination(Bond {
						owner: collator.clone(),
						amount
					}),
					Error::<T>::AlreadyNominatedCollator
				);
				nom
			} else {
				// first nomination
				ensure!(
					amount >= T::MinNominatorStk::get(),
					Error::<T>::NomBondBelowMin
				);
				ensure!(!Self::is_candidate(&acc), Error::<T>::CandidateExists);
				Nominator::new(collator.clone(), amount)
			};
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(
				collator_nominator_count >= state.nominators.0.len() as u32,
				Error::<T>::TooLowCollatorCountToNominate
			);
			ensure!(
				(state.nominators.0.len() as u32) < T::MaxNominatorsPerCollator::get(),
				Error::<T>::TooManyNominators
			);
			ensure!(
				state.nominators.insert(Bond {
					owner: acc.clone(),
					amount,
				}),
				Error::<T>::NominatorExists
			);
			T::Currency::reserve(&acc, amount)?;
			let new_total = state.total + amount;
			if state.is_active() {
				Self::update_active(collator.clone(), new_total);
			}
			let new_total_locked = <Total<T>>::get() + amount;
			<Total<T>>::put(new_total_locked);
			state.total = new_total;
			<CollatorState<T>>::insert(&collator, state);
			<NominatorState<T>>::insert(&acc, nominator);
			Self::deposit_event(Event::Nomination(acc, amount, collator, new_total));
			Ok(().into())
		}
		/// Leave the set of nominators and, by implication, revoke all ongoing nominations
		/// - `nomination_count` is the number of nominations for the account in question
		#[pallet::weight(T::WeightInfo::leave_nominators(*nomination_count))]
		pub fn leave_nominators(
			origin: OriginFor<T>,
			nomination_count: u32,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			let nominator = <NominatorState<T>>::get(&acc).ok_or(Error::<T>::NominatorDNE)?;
			ensure!(
				(nominator.nominations.0.len() as u32) <= nomination_count,
				Error::<T>::TooLowNominationCountToLeaveNominators
			);
			for bond in nominator.nominations.0 {
				Self::nominator_leaves_collator(acc.clone(), bond.owner.clone())?;
			}
			<NominatorState<T>>::remove(&acc);
			Self::deposit_event(Event::NominatorLeft(acc, nominator.total));
			Ok(().into())
		}
		/// Revoke a single existing nomination
		#[pallet::weight(T::WeightInfo::revoke_nomination())]
		pub fn revoke_nomination(
			origin: OriginFor<T>,
			collator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			Self::nominator_revokes_collator(ensure_signed(origin)?, collator)
		}
		/// Bond more for nominators with respect to a specific collator candidate
		#[pallet::weight(T::WeightInfo::nominator_bond_more())]
		pub fn nominator_bond_more(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let nominator = ensure_signed(origin)?;
			let mut nominations =
				<NominatorState<T>>::get(&nominator).ok_or(Error::<T>::NominatorDNE)?;
			let mut collator =
				<CollatorState<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			let _ = nominations
				.inc_nomination(candidate.clone(), more)
				.ok_or(Error::<T>::NominationDNE)?;
			T::Currency::reserve(&nominator, more)?;
			let before = collator.total;
			collator.inc_nominator(nominator.clone(), more);
			let after = collator.total;
			if collator.is_active() {
				Self::update_active(candidate.clone(), collator.total);
			}
			<CollatorState<T>>::insert(&candidate, collator);
			<NominatorState<T>>::insert(&nominator, nominations);
			Self::deposit_event(Event::NominationIncreased(
				nominator, candidate, before, after,
			));
			Ok(().into())
		}
		/// Bond less for nominators with respect to a specific nominator candidate
		#[pallet::weight(T::WeightInfo::nominator_bond_less())]
		pub fn nominator_bond_less(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let nominator = ensure_signed(origin)?;
			let mut nominations =
				<NominatorState<T>>::get(&nominator).ok_or(Error::<T>::NominatorDNE)?;
			let mut collator =
				<CollatorState<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
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
			let before = collator.total;
			collator.dec_nominator(nominator.clone(), less);
			let after = collator.total;
			if collator.is_active() {
				Self::update_active(candidate.clone(), collator.total);
			}
			<CollatorState<T>>::insert(&candidate, collator);
			<NominatorState<T>>::insert(&nominator, nominations);
			Self::deposit_event(Event::NominationDecreased(
				nominator, candidate, before, after,
			));
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn is_nominator(acc: &T::AccountId) -> bool {
			<NominatorState<T>>::get(acc).is_some()
		}
		pub fn is_candidate(acc: &T::AccountId) -> bool {
			<CollatorState<T>>::get(acc).is_some()
		}
		pub fn is_selected_candidate(acc: &T::AccountId) -> bool {
			<SelectedCandidates<T>>::get().binary_search(acc).is_ok()
		}
		/// Update a collator candidate's balance
		/// - caller must ensure candidate is active before calling
		fn update_active(candidate: T::AccountId, total: BalanceOf<T>) {
			let mut candidates = <CandidatePool<T>>::get();
			candidates.remove(&Bond::from_owner(candidate.clone()));
			candidates.insert(Bond {
				owner: candidate,
				amount: total,
			});
			<CandidatePool<T>>::put(candidates);
		}
		/// Calculate round issuance based on total staked for the given round
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

		/// Executes collator exit
		/// - for the collator `who`
		/// - `clear_exit` => exit queue entry exists and must be removed as well
		/// - `collator_count` is a weight hint for the number of active collator candidates
		/// - `nominator_count` is a weight hint for the number of nominations for the collator
		fn collator_exit(
			who: T::AccountId,
			clear_exit: bool,
			collator_count: Option<u32>,
			nominator_count: Option<u32>,
		) -> DispatchResultWithPostInfo {
			let state = <CollatorState<T>>::get(&who).ok_or(Error::<T>::CandidateDNE)?;
			if let Some(count) = nominator_count {
				ensure!(
					count >= state.nominators.0.len() as u32,
					Error::<T>::TooLowNominatorCountToLeaveCandidates
				);
			}
			let mut candidates: Option<OrderedSet<Bond<T::AccountId, BalanceOf<T>>>> = None;
			if let Some(count) = collator_count {
				let candidate_pool = <CandidatePool<T>>::get();
				ensure!(
					count >= candidate_pool.0.len() as u32,
					Error::<T>::TooLowCollatorCandidateCountToLeaveCandidates
				);
				candidates = Some(candidate_pool);
			}
			if state.is_active() {
				let mut collator_candidates = if let Some(pool) = candidates {
					pool
				} else {
					<CandidatePool<T>>::get()
				};
				if collator_candidates.remove(&Bond::from_owner(who.clone())) {
					<CandidatePool<T>>::put(collator_candidates);
				}
			}
			// return all nominator bonds
			for bond in &state.nominators.0 {
				// return stake to nominator
				T::Currency::unreserve(&bond.owner, bond.amount);
				// remove nomination from nominator state
				if let Some(mut nominator) = <NominatorState<T>>::get(&bond.owner) {
					if let Some(remaining) = nominator.rm_nomination(who.clone()) {
						if remaining.is_zero() {
							<NominatorState<T>>::remove(&bond.owner);
						} else {
							<NominatorState<T>>::insert(&bond.owner, nominator);
						}
					}
				}
			}
			if state.is_leaving() && clear_exit {
				// collator was already in exit queue but exit was expedited
				// => need to remove from the exit queue before executing exit
				let remain_exits = <ExitQueue<T>>::get()
					.0
					.into_iter()
					.filter_map(|x| if x.owner == who { None } else { Some(x) })
					.collect::<Vec<Bond<T::AccountId, RoundIndex>>>();
				<ExitQueue<T>>::put(OrderedSet::from(remain_exits));
			}
			// return collator bond
			T::Currency::unreserve(&state.id, state.bond);
			let mut new_count = <CandidateCount<T>>::get();
			new_count.decrement();
			let new_total = <Total<T>>::get() - state.total;
			<Total<T>>::put(new_total);
			<CandidateCount<T>>::put(new_count);
			<CollatorState<T>>::remove(&who);
			Self::deposit_event(Event::CollatorLeft(who, state.total, new_total));
			Ok(().into())
		}
		fn nominator_revokes_collator(
			acc: T::AccountId,
			collator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let mut nominator = <NominatorState<T>>::get(&acc).ok_or(Error::<T>::NominatorDNE)?;
			let old_total = nominator.total;
			let remaining = nominator
				.rm_nomination(collator.clone())
				.ok_or(Error::<T>::NominationDNE)?;
			// edge case; if no nominations remaining, leave set of nominators
			if nominator.nominations.0.len().is_zero() {
				// leave the set of nominators because no nominations left
				Self::nominator_leaves_collator(acc.clone(), collator)?;
				<NominatorState<T>>::remove(&acc);
				Self::deposit_event(Event::NominatorLeft(acc, old_total));
				return Ok(().into());
			}
			ensure!(
				remaining >= T::MinNominatorStk::get(),
				Error::<T>::NomBondBelowMin
			);
			Self::nominator_leaves_collator(acc.clone(), collator)?;
			<NominatorState<T>>::insert(&acc, nominator);
			Ok(().into())
		}
		fn nominator_leaves_collator(
			nominator: T::AccountId,
			collator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let mut state = <CollatorState<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
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
			let nominator_stake = exists.ok_or(Error::<T>::NominatorDNE)?;
			let nominators = OrderedSet::from(noms);
			T::Currency::unreserve(&nominator, nominator_stake);
			state.nominators = nominators;
			state.total -= nominator_stake;
			if state.is_active() {
				Self::update_active(collator.clone(), state.total);
			}
			let new_total_locked = <Total<T>>::get() - nominator_stake;
			<Total<T>>::put(new_total_locked);
			let new_total = state.total;
			<CollatorState<T>>::insert(&collator, state);
			Self::deposit_event(Event::NominatorLeftCollator(
				nominator,
				collator,
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
			let collator_fee = <CollatorCommission<T>>::get();
			if next > duration {
				let round_to_payout = next - duration;
				let total = <Points<T>>::get(round_to_payout);
				let total_staked = <Staked<T>>::get(round_to_payout);
				let issuance = Self::compute_issuance(total_staked);
				for (val, pts) in <AwardedPts<T>>::drain_prefix(round_to_payout) {
					let pct_due = Perbill::from_rational(pts, total);
					let mut amt_due = pct_due * issuance;
					if amt_due <= T::Currency::minimum_balance() {
						continue;
					}
					// Take the snapshot of block author and nominations
					let state = <AtStake<T>>::take(round_to_payout, &val);
					if state.nominators.is_empty() {
						// solo collator with no nominators
						mint(amt_due, val.clone());
					} else {
						// pay collator first; commission + due_portion
						let val_pct = Perbill::from_rational(state.bond, state.total);
						let commission = collator_fee * amt_due;
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
							let percent = Perbill::from_rational(amount, state.total);
							let due = percent * amt_due;
							mint(due, owner);
						}
					}
				}
			}
		}
		fn execute_delayed_collator_exits(next: RoundIndex) {
			let remain_exits = <ExitQueue<T>>::get()
				.0
				.into_iter()
				.filter_map(|x| {
					if x.amount > next {
						Some(x)
					} else {
						// TODO: handle error path explicitly, log something at least
						let _ = Self::collator_exit(x.owner.clone(), false, None, None);
						None
					}
				})
				.collect::<Vec<Bond<T::AccountId, RoundIndex>>>();
			<ExitQueue<T>>::put(OrderedSet::from(remain_exits));
		}
		/// Best as in most cumulatively supported in terms of stake
		fn select_top_candidates(next: RoundIndex) -> (u32, BalanceOf<T>) {
			let (mut all_collators, mut total) = (0u32, BalanceOf::<T>::zero());
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
			// snapshot exposure for round for weighting reward distribution
			for account in collators.iter() {
				let state = <CollatorState<T>>::get(&account)
					.expect("all members of CandidateQ must be candidates");
				let amount = state.total;
				let exposure: CollatorSnapshot<T::AccountId, BalanceOf<T>> = state.into();
				<AtStake<T>>::insert(next, account, exposure);
				all_collators += 1u32;
				total += amount;
				Self::deposit_event(Event::CollatorChosen(next, account.clone(), amount));
			}
			collators.sort();
			// insert canonical collator set
			<SelectedCandidates<T>>::put(collators);
			(all_collators, total)
		}
	}
	/// Add reward points to block authors:
	/// * 20 points to the block producer for producing a block in the chain
	impl<T: Config> pallet_author_inherent::EventHandler<T::AccountId> for Pallet<T> {
		fn note_author(author: T::AccountId) {
			let now = <Round<T>>::get().current;
			let score_plus_20 = <AwardedPts<T>>::get(now, &author) + 20;
			<AwardedPts<T>>::insert(now, author, score_plus_20);
			<Points<T>>::mutate(now, |x| *x += 20);
		}
	}

	impl<T: Config> pallet_author_inherent::CanAuthor<T::AccountId> for Pallet<T> {
		fn can_author(account: &T::AccountId) -> bool {
			Self::is_selected_candidate(account)
		}
	}

	//TODO this is for coupling with the author slot filter.
	// Nimbus should introduce its own trait for exhaustive sets
	// and then use that here.
	impl<T: Config> Get<Vec<T::AccountId>> for Pallet<T> {
		fn get() -> Vec<T::AccountId> {
			Self::selected_candidates()
		}
	}
}
