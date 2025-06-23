// Copyright 2019-2025 PureStake Inc.
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
//! * a new set of collators is chosen from the candidates
//!
//! Immediately following a round change, payments are made once-per-block until all payments have
//! been made. In each such block, one collator is chosen for a rewards payment and is paid along
//! with each of its top `T::MaxTopDelegationsPerCandidate` delegators.
//!
//! To join the set of candidates, call `join_candidates` with `bond >= MinCandidateStk`.
//! To leave the set of candidates, call `schedule_leave_candidates`. If the call succeeds,
//! the collator is removed from the pool of candidates so they cannot be selected for future
//! collator sets, but they are not unbonded until their exit request is executed. Any signed
//! account may trigger the exit `T::LeaveCandidatesDelay` rounds after the round in which the
//! original request was made.
//!
//! To join the set of delegators, call `delegate` and pass in an account that is
//! already a collator candidate and `bond >= MinDelegation`. Each delegator can delegate up to
//! `T::MaxDelegationsPerDelegator` collator candidates by calling `delegate`.
//!
//! To revoke a delegation, call `revoke_delegation` with the collator candidate's account.
//! To leave the set of delegators and revoke all delegations, call `leave_delegators`.

#![cfg_attr(not(feature = "std"), no_std)]

mod auto_compound;
mod delegation_requests;
pub mod inflation;
pub mod migrations;
pub mod traits;
pub mod types;
pub mod weights;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;
#[cfg(test)]
mod mock;
mod set;
#[cfg(test)]
mod tests;

use frame_support::pallet;
pub use inflation::{InflationInfo, Range};
pub use weights::WeightInfo;

pub use auto_compound::{AutoCompoundConfig, AutoCompoundDelegations};
pub use delegation_requests::{CancelledScheduledRequest, DelegationAction, ScheduledRequest};
pub use pallet::*;
pub use traits::*;
pub use types::*;
pub use RoundIndex;

#[pallet]
pub mod pallet {
	use crate::delegation_requests::{
		CancelledScheduledRequest, DelegationAction, ScheduledRequest,
	};
	use crate::{set::BoundedOrderedSet, traits::*, types::*, InflationInfo, Range, WeightInfo};
	use crate::{AutoCompoundConfig, AutoCompoundDelegations};
	use frame_support::pallet_prelude::*;
	use frame_support::traits::{
		fungible::{Balanced, Inspect, Mutate, MutateFreeze},
		Currency, Get, LockIdentifier, LockableCurrency, ReservableCurrency,
	};
	use frame_system::pallet_prelude::*;
	use sp_consensus_slots::Slot;
	use sp_runtime::{
		traits::{Saturating, Zero},
		DispatchErrorWithPostInfo, Perbill, Percent,
	};
	use sp_std::{collections::btree_map::BTreeMap, prelude::*};

	/// Pallet for parachain staking
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	pub type RoundIndex = u32;
	type RewardPoint = u32;
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

	// DEPRECATED: Remove after applying migration `MigrateLocksToFreezes`
	pub const COLLATOR_LOCK_ID: LockIdentifier = *b"stkngcol";
	pub const DELEGATOR_LOCK_ID: LockIdentifier = *b"stkngdel";

	/// A hard limit for weight computation purposes for the max candidates that _could_
	/// theoretically exist.
	pub const MAX_CANDIDATES: u32 = 200;

	/// Maximum number of accounts (delegators and candidates) that can be migrated at once in the `migrate_locks_to_freezes_batch` extrinsic.
	pub(crate) const MAX_ACCOUNTS_PER_MIGRATION_BATCH: u32 = 100;

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The fungible type for handling balances
		type Currency: Inspect<Self::AccountId>
			+ Mutate<Self::AccountId>
			+ MutateFreeze<Self::AccountId, Id = Self::RuntimeFreezeReason>
			+ Balanced<Self::AccountId>
			// DEPRECATED: Remove traits below after applying migration `MigrateLocksToFreezes`
			+ Currency<Self::AccountId, Balance = BalanceOf<Self>>
			+ ReservableCurrency<Self::AccountId>
			+ LockableCurrency<Self::AccountId>;
		/// The overarching freeze identifier type.
		type RuntimeFreezeReason: From<FreezeReason>;
		/// The origin for monetary governance
		type MonetaryGovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Minimum number of blocks per round
		#[pallet::constant]
		type MinBlocksPerRound: Get<u32>;
		/// If a collator doesn't produce any block on this number of rounds, it is notified as inactive.
		/// This value must be less than or equal to RewardPaymentDelay.
		#[pallet::constant]
		type MaxOfflineRounds: Get<u32>;
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
		/// Maximum top delegations counted per candidate
		#[pallet::constant]
		type MaxTopDelegationsPerCandidate: Get<u32>;
		/// Maximum bottom delegations (not counted) per candidate
		#[pallet::constant]
		type MaxBottomDelegationsPerCandidate: Get<u32>;
		/// Maximum delegations per delegator
		#[pallet::constant]
		type MaxDelegationsPerDelegator: Get<u32>;
		/// Minimum stake required for any account to be a collator candidate
		#[pallet::constant]
		type MinCandidateStk: Get<BalanceOf<Self>>;
		/// Minimum stake for any registered on-chain account to delegate
		#[pallet::constant]
		type MinDelegation: Get<BalanceOf<Self>>;
		/// Get the current block author
		type BlockAuthor: Get<Self::AccountId>;
		/// Handler to notify the runtime when a collator is paid.
		/// If you don't need it, you can specify the type `()`.
		type OnCollatorPayout: OnCollatorPayout<Self::AccountId, BalanceOf<Self>>;
		/// Handler to distribute a collator's reward.
		/// To use the default implementation of minting rewards, specify the type `()`.
		type PayoutCollatorReward: PayoutCollatorReward<Self>;
		/// Handler to notify the runtime when a collator is inactive.
		/// The default behavior is to mark the collator as offline.
		/// If you need to use the default implementation, specify the type `()`.
		type OnInactiveCollator: OnInactiveCollator<Self>;
		/// Handler to notify the runtime when a new round begin.
		/// If you don't need it, you can specify the type `()`.
		type OnNewRound: OnNewRound;
		/// Get the current slot number
		type SlotProvider: Get<Slot>;
		/// Get the slot duration in milliseconds
		#[pallet::constant]
		type SlotDuration: Get<u64>;
		/// Get the average time beetween 2 blocks in milliseconds
		#[pallet::constant]
		type BlockTime: Get<u64>;
		/// Maximum candidates
		#[pallet::constant]
		type MaxCandidates: Get<u32>;
		/// Threshold after which inflation become linear
		/// If you don't want to use it, set it to `()`
		#[pallet::constant]
		type LinearInflationThreshold: Get<Option<BalanceOf<Self>>>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	/// The reason for freezing funds.
	#[pallet::composite_enum]
	pub enum FreezeReason {
		/// Funds frozen for staking as a collator
		StakingCollator,
		/// Funds frozen for staking as a delegator
		StakingDelegator,
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
		RoundLengthMustBeGreaterThanTotalSelectedCollators,
		NoWritingSameValue,
		TotalInflationDistributionPercentExceeds100,
		TooLowCandidateCountWeightHintJoinCandidates,
		TooLowCandidateCountWeightHintCancelLeaveCandidates,
		TooLowCandidateCountToLeaveCandidates,
		TooLowDelegationCountToDelegate,
		TooLowCandidateDelegationCountToDelegate,
		TooLowCandidateDelegationCountToLeaveCandidates,
		TooLowDelegationCountToLeaveDelegators,
		PendingCandidateRequestsDNE,
		PendingCandidateRequestAlreadyExists,
		PendingCandidateRequestNotDueYet,
		PendingDelegationRequestDNE,
		PendingDelegationRequestAlreadyExists,
		PendingDelegationRequestNotDueYet,
		CannotDelegateLessThanOrEqualToLowestBottomWhenFull,
		PendingDelegationRevoke,
		TooLowDelegationCountToAutoCompound,
		TooLowCandidateAutoCompoundingDelegationCountToAutoCompound,
		TooLowCandidateAutoCompoundingDelegationCountToDelegate,
		TooLowCollatorCountToNotifyAsInactive,
		CannotBeNotifiedAsInactive,
		TooLowCandidateAutoCompoundingDelegationCountToLeaveCandidates,
		TooLowCandidateCountWeightHint,
		TooLowCandidateCountWeightHintGoOffline,
		CandidateLimitReached,
		CannotSetAboveMaxCandidates,
		MarkingOfflineNotEnabled,
		CurrentRoundTooLow,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Started new round.
		NewRound {
			starting_block: BlockNumberFor<T>,
			round: RoundIndex,
			selected_collators_number: u32,
			total_balance: BalanceOf<T>,
		},
		/// Account joined the set of collator candidates.
		JoinedCollatorCandidates {
			account: T::AccountId,
			amount_locked: BalanceOf<T>,
			new_total_amt_locked: BalanceOf<T>,
		},
		/// Candidate selected for collators. Total Exposed Amount includes all delegations.
		CollatorChosen {
			round: RoundIndex,
			collator_account: T::AccountId,
			total_exposed_amount: BalanceOf<T>,
		},
		/// Candidate requested to decrease a self bond.
		CandidateBondLessRequested {
			candidate: T::AccountId,
			amount_to_decrease: BalanceOf<T>,
			execute_round: RoundIndex,
		},
		/// Candidate has increased a self bond.
		CandidateBondedMore {
			candidate: T::AccountId,
			amount: BalanceOf<T>,
			new_total_bond: BalanceOf<T>,
		},
		/// Candidate has decreased a self bond.
		CandidateBondedLess {
			candidate: T::AccountId,
			amount: BalanceOf<T>,
			new_bond: BalanceOf<T>,
		},
		/// Candidate temporarily leave the set of collator candidates without unbonding.
		CandidateWentOffline { candidate: T::AccountId },
		/// Candidate rejoins the set of collator candidates.
		CandidateBackOnline { candidate: T::AccountId },
		/// Candidate has requested to leave the set of candidates.
		CandidateScheduledExit {
			exit_allowed_round: RoundIndex,
			candidate: T::AccountId,
			scheduled_exit: RoundIndex,
		},
		/// Cancelled request to leave the set of candidates.
		CancelledCandidateExit { candidate: T::AccountId },
		/// Cancelled request to decrease candidate's bond.
		CancelledCandidateBondLess {
			candidate: T::AccountId,
			amount: BalanceOf<T>,
			execute_round: RoundIndex,
		},
		/// Candidate has left the set of candidates.
		CandidateLeft {
			ex_candidate: T::AccountId,
			unlocked_amount: BalanceOf<T>,
			new_total_amt_locked: BalanceOf<T>,
		},
		/// Delegator requested to decrease a bond for the collator candidate.
		DelegationDecreaseScheduled {
			delegator: T::AccountId,
			candidate: T::AccountId,
			amount_to_decrease: BalanceOf<T>,
			execute_round: RoundIndex,
		},
		// Delegation increased.
		DelegationIncreased {
			delegator: T::AccountId,
			candidate: T::AccountId,
			amount: BalanceOf<T>,
			in_top: bool,
		},
		// Delegation decreased.
		DelegationDecreased {
			delegator: T::AccountId,
			candidate: T::AccountId,
			amount: BalanceOf<T>,
			in_top: bool,
		},
		/// Delegator requested to leave the set of delegators.
		DelegatorExitScheduled {
			round: RoundIndex,
			delegator: T::AccountId,
			scheduled_exit: RoundIndex,
		},
		/// Delegator requested to revoke delegation.
		DelegationRevocationScheduled {
			round: RoundIndex,
			delegator: T::AccountId,
			candidate: T::AccountId,
			scheduled_exit: RoundIndex,
		},
		/// Delegator has left the set of delegators.
		DelegatorLeft {
			delegator: T::AccountId,
			unstaked_amount: BalanceOf<T>,
		},
		/// Delegation revoked.
		DelegationRevoked {
			delegator: T::AccountId,
			candidate: T::AccountId,
			unstaked_amount: BalanceOf<T>,
		},
		/// Delegation kicked.
		DelegationKicked {
			delegator: T::AccountId,
			candidate: T::AccountId,
			unstaked_amount: BalanceOf<T>,
		},
		/// Cancelled a pending request to exit the set of delegators.
		DelegatorExitCancelled { delegator: T::AccountId },
		/// Cancelled request to change an existing delegation.
		CancelledDelegationRequest {
			delegator: T::AccountId,
			cancelled_request: CancelledScheduledRequest<BalanceOf<T>>,
			collator: T::AccountId,
		},
		/// New delegation (increase of the existing one).
		Delegation {
			delegator: T::AccountId,
			locked_amount: BalanceOf<T>,
			candidate: T::AccountId,
			delegator_position: DelegatorAdded<BalanceOf<T>>,
			auto_compound: Percent,
		},
		/// Delegation from candidate state has been remove.
		DelegatorLeftCandidate {
			delegator: T::AccountId,
			candidate: T::AccountId,
			unstaked_amount: BalanceOf<T>,
			total_candidate_staked: BalanceOf<T>,
		},
		/// Paid the account (delegator or collator) the balance as liquid rewards.
		Rewarded {
			account: T::AccountId,
			rewards: BalanceOf<T>,
		},
		/// Transferred to account which holds funds reserved for parachain bond.
		InflationDistributed {
			index: u32,
			account: T::AccountId,
			value: BalanceOf<T>,
		},
		InflationDistributionConfigUpdated {
			old: InflationDistributionConfig<T::AccountId>,
			new: InflationDistributionConfig<T::AccountId>,
		},
		/// Annual inflation input (first 3) was used to derive new per-round inflation (last 3)
		InflationSet {
			annual_min: Perbill,
			annual_ideal: Perbill,
			annual_max: Perbill,
			round_min: Perbill,
			round_ideal: Perbill,
			round_max: Perbill,
		},
		/// Staking expectations set.
		StakeExpectationsSet {
			expect_min: BalanceOf<T>,
			expect_ideal: BalanceOf<T>,
			expect_max: BalanceOf<T>,
		},
		/// Set total selected candidates to this value.
		TotalSelectedSet { old: u32, new: u32 },
		/// Set collator commission to this value.
		CollatorCommissionSet { old: Perbill, new: Perbill },
		/// Set blocks per round
		BlocksPerRoundSet {
			current_round: RoundIndex,
			first_block: BlockNumberFor<T>,
			old: u32,
			new: u32,
			new_per_round_inflation_min: Perbill,
			new_per_round_inflation_ideal: Perbill,
			new_per_round_inflation_max: Perbill,
		},
		/// Auto-compounding reward percent was set for a delegation.
		AutoCompoundSet {
			candidate: T::AccountId,
			delegator: T::AccountId,
			value: Percent,
		},
		/// Compounded a portion of rewards towards the delegation.
		Compounded {
			candidate: T::AccountId,
			delegator: T::AccountId,
			amount: BalanceOf<T>,
		},
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			let mut weight = <T as Config>::WeightInfo::base_on_initialize();

			let mut round = <Round<T>>::get();
			if round.should_update(n) {
				// fetch current slot number
				let current_slot: u64 = T::SlotProvider::get().into();

				// account for SlotProvider read
				weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));

				// Compute round duration in slots
				let round_duration = (current_slot.saturating_sub(round.first_slot))
					.saturating_mul(T::SlotDuration::get());

				// mutate round
				round.update(n, current_slot);
				// notify that new round begin
				weight = weight.saturating_add(T::OnNewRound::on_new_round(round.current));
				// pay all stakers for T::RewardPaymentDelay rounds ago
				weight =
					weight.saturating_add(Self::prepare_staking_payouts(round, round_duration));
				// select top collator candidates for next round
				let (extra_weight, collator_count, _delegation_count, total_staked) =
					Self::select_top_candidates(round.current);
				weight = weight.saturating_add(extra_weight);
				// start next round
				<Round<T>>::put(round);
				Self::deposit_event(Event::NewRound {
					starting_block: round.first,
					round: round.current,
					selected_collators_number: collator_count,
					total_balance: total_staked,
				});
				// record inactive collators
				weight = weight.saturating_add(Self::mark_collators_as_inactive(round.current));
				// account for Round write
				weight = weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));
			} else {
				weight = weight.saturating_add(Self::handle_delayed_payouts(round.current));
			}

			// add on_finalize weight
			//   read:  Author, Points, AwardedPts, WasInactive
			//   write: Points, AwardedPts, WasInactive
			weight = weight.saturating_add(T::DbWeight::get().reads_writes(4, 3));
			weight
		}
		fn on_finalize(_n: BlockNumberFor<T>) {
			Self::award_points_to_block_author();
			Self::cleanup_inactive_collator_info();
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn collator_commission)]
	/// Commission percent taken off of rewards for all collators
	type CollatorCommission<T: Config> = StorageValue<_, Perbill, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_selected)]
	/// The total candidates selected every round
	pub(crate) type TotalSelected<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn inflation_distribution_info)]
	/// Inflation distribution configuration, including accounts that should receive inflation
	/// before it is distributed to collators and delegators.
	///
	/// The sum of the distribution percents must be less than or equal to 100.
	pub(crate) type InflationDistributionInfo<T: Config> =
		StorageValue<_, InflationDistributionConfig<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn round)]
	/// Current round index and next round scheduled transition
	pub type Round<T: Config> = StorageValue<_, RoundInfo<BlockNumberFor<T>>, ValueQuery>;

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
	#[pallet::getter(fn candidate_info)]
	/// Get collator candidate info associated with an account if account is candidate else None
	pub(crate) type CandidateInfo<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, CandidateMetadata<BalanceOf<T>>, OptionQuery>;

	pub struct AddGet<T, R> {
		_phantom: PhantomData<(T, R)>,
	}
	impl<T, R> Get<u32> for AddGet<T, R>
	where
		T: Get<u32>,
		R: Get<u32>,
	{
		fn get() -> u32 {
			T::get() + R::get()
		}
	}

	/// Stores outstanding delegation requests per collator.
	#[pallet::storage]
	#[pallet::getter(fn delegation_scheduled_requests)]
	pub(crate) type DelegationScheduledRequests<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<
			ScheduledRequest<T::AccountId, BalanceOf<T>>,
			AddGet<T::MaxTopDelegationsPerCandidate, T::MaxBottomDelegationsPerCandidate>,
		>,
		ValueQuery,
	>;

	/// Stores auto-compounding configuration per collator.
	#[pallet::storage]
	#[pallet::getter(fn auto_compounding_delegations)]
	pub(crate) type AutoCompoundingDelegations<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<
			AutoCompoundConfig<T::AccountId>,
			AddGet<T::MaxTopDelegationsPerCandidate, T::MaxBottomDelegationsPerCandidate>,
		>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn top_delegations)]
	/// Top delegations for collator candidate
	pub(crate) type TopDelegations<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Delegations<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn bottom_delegations)]
	/// Bottom delegations for collator candidate
	pub(crate) type BottomDelegations<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Delegations<T::AccountId, BalanceOf<T>>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn selected_candidates)]
	/// The collator candidates selected for the current round
	type SelectedCandidates<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxCandidates>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total)]
	/// Total capital locked by this staking pallet
	pub(crate) type Total<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn candidate_pool)]
	/// The pool of collator candidates, each with their total backing stake
	pub(crate) type CandidatePool<T: Config> = StorageValue<
		_,
		BoundedOrderedSet<Bond<T::AccountId, BalanceOf<T>>, T::MaxCandidates>,
		ValueQuery,
	>;

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
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn was_inactive)]
	/// Records collators' inactivity.
	/// Data persists for MaxOfflineRounds + 1 rounds before being pruned.
	pub type WasInactive<T: Config> =
		StorageDoubleMap<_, Twox64Concat, RoundIndex, Twox64Concat, T::AccountId, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn delayed_payouts)]
	/// Delayed payouts
	pub type DelayedPayouts<T: Config> =
		StorageMap<_, Twox64Concat, RoundIndex, DelayedPayout<BalanceOf<T>>, OptionQuery>;

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

	#[pallet::storage]
	#[pallet::getter(fn marking_offline)]
	/// Killswitch to enable/disable marking offline feature.
	pub type EnableMarkingOffline<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	/// Temporary storage to track candidates that have been migrated from locks to freezes.
	/// This storage should be removed after all accounts have been migrated.
	pub type MigratedCandidates<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (), OptionQuery>;

	#[pallet::storage]
	/// Temporary storage to track delegators that have been migrated from locks to freezes.
	/// This storage should be removed after all accounts have been migrated.
	pub type MigratedDelegators<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (), OptionQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// Initialize balance and register all as collators: `(collator AccountId, balance Amount)`
		pub candidates: Vec<(T::AccountId, BalanceOf<T>)>,
		/// Initialize balance and make delegations:
		/// `(delegator AccountId, collator AccountId, delegation Amount, auto-compounding Percent)`
		pub delegations: Vec<(T::AccountId, T::AccountId, BalanceOf<T>, Percent)>,
		/// Inflation configuration
		pub inflation_config: InflationInfo<BalanceOf<T>>,
		/// Default fixed percent a collator takes off the top of due rewards
		pub collator_commission: Perbill,
		/// Default percent of inflation set aside for parachain bond every round
		pub parachain_bond_reserve_percent: Percent,
		/// Default number of blocks in a round
		pub blocks_per_round: u32,
		/// Number of selected candidates every round. Cannot be lower than MinSelectedCandidates
		pub num_selected_candidates: u32,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				candidates: vec![],
				delegations: vec![],
				inflation_config: Default::default(),
				collator_commission: Default::default(),
				parachain_bond_reserve_percent: Default::default(),
				blocks_per_round: 1u32,
				num_selected_candidates: T::MinSelectedCandidates::get(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			assert!(self.blocks_per_round > 0, "Blocks per round must be > 0");
			<InflationConfig<T>>::put(self.inflation_config.clone());
			let mut candidate_count = 0u32;
			// Initialize the candidates
			for &(ref candidate, balance) in &self.candidates {
				assert!(
					<Pallet<T>>::get_collator_stakable_free_balance(candidate) >= balance,
					"Account does not have enough balance to bond as a candidate."
				);
				if let Err(error) = <Pallet<T>>::join_candidates(
					T::RuntimeOrigin::from(Some(candidate.clone()).into()),
					balance,
					candidate_count,
				) {
					log::warn!("Join candidates failed in genesis with error {:?}", error);
				} else {
					candidate_count = candidate_count.saturating_add(1u32);
				}
			}

			let mut col_delegator_count: BTreeMap<T::AccountId, u32> = BTreeMap::new();
			let mut col_auto_compound_delegator_count: BTreeMap<T::AccountId, u32> =
				BTreeMap::new();
			let mut del_delegation_count: BTreeMap<T::AccountId, u32> = BTreeMap::new();
			// Initialize the delegations
			for &(ref delegator, ref target, balance, auto_compound) in &self.delegations {
				assert!(
					<Pallet<T>>::get_delegator_stakable_balance(delegator) >= balance,
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
				let cd_auto_compound_count = col_auto_compound_delegator_count
					.get(target)
					.cloned()
					.unwrap_or_default();
				if let Err(error) = <Pallet<T>>::delegate_with_auto_compound(
					T::RuntimeOrigin::from(Some(delegator.clone()).into()),
					target.clone(),
					balance,
					auto_compound,
					cd_count,
					cd_auto_compound_count,
					dd_count,
				) {
					log::warn!("Delegate failed in genesis with error {:?}", error);
				} else {
					if let Some(x) = col_delegator_count.get_mut(target) {
						*x = x.saturating_add(1u32);
					} else {
						col_delegator_count.insert(target.clone(), 1u32);
					};
					if let Some(x) = del_delegation_count.get_mut(delegator) {
						*x = x.saturating_add(1u32);
					} else {
						del_delegation_count.insert(delegator.clone(), 1u32);
					};
					if !auto_compound.is_zero() {
						col_auto_compound_delegator_count
							.entry(target.clone())
							.and_modify(|x| *x = x.saturating_add(1))
							.or_insert(1);
					}
				}
			}
			// Set collator commission to default config
			<CollatorCommission<T>>::put(self.collator_commission);
			// Set parachain bond config to default config
			let pbr = InflationDistributionAccount {
				// must be set soon; if not => due inflation will be sent to collators/delegators
				account: T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
					.expect("infinite length input; no invalid inputs for type; qed"),
				percent: self.parachain_bond_reserve_percent,
			};
			let zeroed_account = InflationDistributionAccount {
				account: T::AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
					.expect("infinite length input; no invalid inputs for type; qed"),
				percent: Percent::zero(),
			};
			<InflationDistributionInfo<T>>::put::<InflationDistributionConfig<T::AccountId>>(
				[pbr, zeroed_account].into(),
			);
			// Set total selected candidates to value from config
			assert!(
				self.num_selected_candidates >= T::MinSelectedCandidates::get(),
				"{:?}",
				Error::<T>::CannotSetBelowMin
			);
			assert!(
				self.num_selected_candidates <= T::MaxCandidates::get(),
				"{:?}",
				Error::<T>::CannotSetAboveMaxCandidates
			);
			<TotalSelected<T>>::put(self.num_selected_candidates);
			// Choose top TotalSelected collator candidates
			let (_, v_count, _, total_staked) = <Pallet<T>>::select_top_candidates(1u32);
			// Start Round 1 at Block 0
			let round: RoundInfo<BlockNumberFor<T>> =
				RoundInfo::new(1u32, Zero::zero(), self.blocks_per_round, 0);
			<Round<T>>::put(round);
			<Pallet<T>>::deposit_event(Event::NewRound {
				starting_block: Zero::zero(),
				round: 1u32,
				selected_collators_number: v_count,
				total_balance: total_staked,
			});
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the expectations for total staked. These expectations determine the issuance for
		/// the round according to logic in `fn compute_issuance`
		#[pallet::call_index(0)]
		#[pallet::weight(<T as Config>::WeightInfo::set_staking_expectations())]
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
			Self::deposit_event(Event::StakeExpectationsSet {
				expect_min: config.expect.min,
				expect_ideal: config.expect.ideal,
				expect_max: config.expect.max,
			});
			<InflationConfig<T>>::put(config);
			Ok(().into())
		}

		/// Set the annual inflation rate to derive per-round inflation
		#[pallet::call_index(1)]
		#[pallet::weight(<T as Config>::WeightInfo::set_inflation())]
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
			Self::deposit_event(Event::InflationSet {
				annual_min: config.annual.min,
				annual_ideal: config.annual.ideal,
				annual_max: config.annual.max,
				round_min: config.round.min,
				round_ideal: config.round.ideal,
				round_max: config.round.max,
			});
			<InflationConfig<T>>::put(config);
			Ok(().into())
		}

		/// Deprecated: please use `set_inflation_distribution_config` instead.
		///
		///  Set the account that will hold funds set aside for parachain bond
		#[pallet::call_index(2)]
		#[pallet::weight(<T as Config>::WeightInfo::set_parachain_bond_account())]
		pub fn set_parachain_bond_account(
			origin: OriginFor<T>,
			new: T::AccountId,
		) -> DispatchResultWithPostInfo {
			T::MonetaryGovernanceOrigin::ensure_origin(origin.clone())?;
			let old = <InflationDistributionInfo<T>>::get().0;
			let new = InflationDistributionAccount {
				account: new,
				percent: old[0].percent.clone(),
			};
			Pallet::<T>::set_inflation_distribution_config(origin, [new, old[1].clone()].into())
		}

		/// Deprecated: please use `set_inflation_distribution_config` instead.
		///
		/// Set the percent of inflation set aside for parachain bond
		#[pallet::call_index(3)]
		#[pallet::weight(<T as Config>::WeightInfo::set_parachain_bond_reserve_percent())]
		pub fn set_parachain_bond_reserve_percent(
			origin: OriginFor<T>,
			new: Percent,
		) -> DispatchResultWithPostInfo {
			T::MonetaryGovernanceOrigin::ensure_origin(origin.clone())?;
			let old = <InflationDistributionInfo<T>>::get().0;
			let new = InflationDistributionAccount {
				account: old[0].account.clone(),
				percent: new,
			};
			Pallet::<T>::set_inflation_distribution_config(origin, [new, old[1].clone()].into())
		}

		/// Set the total number of collator candidates selected per round
		/// - changes are not applied until the start of the next round
		#[pallet::call_index(4)]
		#[pallet::weight(<T as Config>::WeightInfo::set_total_selected())]
		pub fn set_total_selected(origin: OriginFor<T>, new: u32) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			ensure!(
				new >= T::MinSelectedCandidates::get(),
				Error::<T>::CannotSetBelowMin
			);
			ensure!(
				new <= T::MaxCandidates::get(),
				Error::<T>::CannotSetAboveMaxCandidates
			);
			let old = <TotalSelected<T>>::get();
			ensure!(old != new, Error::<T>::NoWritingSameValue);
			ensure!(
				new < <Round<T>>::get().length,
				Error::<T>::RoundLengthMustBeGreaterThanTotalSelectedCollators,
			);
			<TotalSelected<T>>::put(new);
			Self::deposit_event(Event::TotalSelectedSet { old, new });
			Ok(().into())
		}

		/// Set the commission for all collators
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::set_collator_commission())]
		pub fn set_collator_commission(
			origin: OriginFor<T>,
			new: Perbill,
		) -> DispatchResultWithPostInfo {
			frame_system::ensure_root(origin)?;
			let old = <CollatorCommission<T>>::get();
			ensure!(old != new, Error::<T>::NoWritingSameValue);
			<CollatorCommission<T>>::put(new);
			Self::deposit_event(Event::CollatorCommissionSet { old, new });
			Ok(().into())
		}

		/// Set blocks per round
		/// - if called with `new` less than length of current round, will transition immediately
		/// in the next block
		/// - also updates per-round inflation config
		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::set_blocks_per_round())]
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
				new > <TotalSelected<T>>::get(),
				Error::<T>::RoundLengthMustBeGreaterThanTotalSelectedCollators,
			);
			round.length = new;
			// update per-round inflation given new rounds per year
			let mut inflation_config = <InflationConfig<T>>::get();
			inflation_config.reset_round::<T>(new);
			<Round<T>>::put(round);
			Self::deposit_event(Event::BlocksPerRoundSet {
				current_round: now,
				first_block: first,
				old: old,
				new: new,
				new_per_round_inflation_min: inflation_config.round.min,
				new_per_round_inflation_ideal: inflation_config.round.ideal,
				new_per_round_inflation_max: inflation_config.round.max,
			});
			<InflationConfig<T>>::put(inflation_config);
			Ok(().into())
		}

		/// Join the set of collator candidates
		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::join_candidates(*candidate_count))]
		pub fn join_candidates(
			origin: OriginFor<T>,
			bond: BalanceOf<T>,
			candidate_count: u32,
		) -> DispatchResultWithPostInfo {
			let acc = ensure_signed(origin)?;
			ensure!(
				bond >= T::MinCandidateStk::get(),
				Error::<T>::CandidateBondBelowMin
			);
			Self::join_candidates_inner(acc, bond, candidate_count)
		}

		/// Request to leave the set of candidates. If successful, the account is immediately
		/// removed from the candidate pool to prevent selection as a collator.
		#[pallet::call_index(8)]
		#[pallet::weight(<T as Config>::WeightInfo::schedule_leave_candidates(*candidate_count))]
		pub fn schedule_leave_candidates(
			origin: OriginFor<T>,
			candidate_count: u32,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateInfo<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let (now, when) = state.schedule_leave::<T>()?;
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidate_count >= candidates.0.len() as u32,
				Error::<T>::TooLowCandidateCountToLeaveCandidates
			);
			if candidates.remove(&Bond::from_owner(collator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<CandidateInfo<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateScheduledExit {
				exit_allowed_round: now,
				candidate: collator,
				scheduled_exit: when,
			});
			Ok(().into())
		}

		/// Execute leave candidates request
		#[pallet::call_index(9)]
		#[pallet::weight(
			<T as Config>::WeightInfo::execute_leave_candidates_worst_case(*candidate_delegation_count)
		)]
		pub fn execute_leave_candidates(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			candidate_delegation_count: u32,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			let state = <CandidateInfo<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(
				state.delegation_count <= candidate_delegation_count,
				Error::<T>::TooLowCandidateDelegationCountToLeaveCandidates
			);
			<Pallet<T>>::execute_leave_candidates_inner(candidate)
		}

		/// Cancel open request to leave candidates
		/// - only callable by collator account
		/// - result upon successful call is the candidate is active in the candidate pool
		#[pallet::call_index(10)]
		#[pallet::weight(<T as Config>::WeightInfo::cancel_leave_candidates(*candidate_count))]
		pub fn cancel_leave_candidates(
			origin: OriginFor<T>,
			candidate_count: u32,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateInfo<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			ensure!(state.is_leaving(), Error::<T>::CandidateNotLeaving);
			state.go_online();
			let mut candidates = <CandidatePool<T>>::get();
			ensure!(
				candidates.0.len() as u32 <= candidate_count,
				Error::<T>::TooLowCandidateCountWeightHintCancelLeaveCandidates
			);
			let maybe_inserted_candidate = candidates
				.try_insert(Bond {
					owner: collator.clone(),
					amount: state.total_counted,
				})
				.map_err(|_| Error::<T>::CandidateLimitReached)?;
			ensure!(maybe_inserted_candidate, Error::<T>::AlreadyActive);
			<CandidatePool<T>>::put(candidates);
			<CandidateInfo<T>>::insert(&collator, state);
			Self::deposit_event(Event::CancelledCandidateExit {
				candidate: collator,
			});
			Ok(().into())
		}

		/// Temporarily leave the set of collator candidates without unbonding
		#[pallet::call_index(11)]
		#[pallet::weight(<T as Config>::WeightInfo::go_offline(MAX_CANDIDATES))]
		pub fn go_offline(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			<Pallet<T>>::go_offline_inner(collator)
		}

		/// Rejoin the set of collator candidates if previously had called `go_offline`
		#[pallet::call_index(12)]
		#[pallet::weight(<T as Config>::WeightInfo::go_online(MAX_CANDIDATES))]
		pub fn go_online(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			<Pallet<T>>::go_online_inner(collator)
		}

		/// Increase collator candidate self bond by `more`
		#[pallet::call_index(13)]
		#[pallet::weight(<T as Config>::WeightInfo::candidate_bond_more(MAX_CANDIDATES))]
		pub fn candidate_bond_more(
			origin: OriginFor<T>,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let candidate = ensure_signed(origin)?;
			<Pallet<T>>::candidate_bond_more_inner(candidate, more)
		}

		/// Request by collator candidate to decrease self bond by `less`
		#[pallet::call_index(14)]
		#[pallet::weight(<T as Config>::WeightInfo::schedule_candidate_bond_less())]
		pub fn schedule_candidate_bond_less(
			origin: OriginFor<T>,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateInfo<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let when = state.schedule_bond_less::<T>(less)?;
			<CandidateInfo<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateBondLessRequested {
				candidate: collator,
				amount_to_decrease: less,
				execute_round: when,
			});
			Ok(().into())
		}

		/// Execute pending request to adjust the collator candidate self bond
		#[pallet::call_index(15)]
		#[pallet::weight(<T as Config>::WeightInfo::execute_candidate_bond_less(MAX_CANDIDATES))]
		pub fn execute_candidate_bond_less(
			origin: OriginFor<T>,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?; // we may want to reward this if caller != candidate
			<Pallet<T>>::execute_candidate_bond_less_inner(candidate)
		}

		/// Cancel pending request to adjust the collator candidate self bond
		#[pallet::call_index(16)]
		#[pallet::weight(<T as Config>::WeightInfo::cancel_candidate_bond_less())]
		pub fn cancel_candidate_bond_less(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let collator = ensure_signed(origin)?;
			let mut state = <CandidateInfo<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			state.cancel_bond_less::<T>(collator.clone())?;
			<CandidateInfo<T>>::insert(&collator, state);
			Ok(().into())
		}

		/// If caller is not a delegator and not a collator, then join the set of delegators
		/// If caller is a delegator, then makes delegation to change their delegation state
		/// Sets the auto-compound config for the delegation
		#[pallet::call_index(18)]
		#[pallet::weight(
			<T as Config>::WeightInfo::delegate_with_auto_compound(
				*candidate_delegation_count,
				*candidate_auto_compounding_delegation_count,
				*delegation_count,
			)
		)]
		pub fn delegate_with_auto_compound(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			amount: BalanceOf<T>,
			auto_compound: Percent,
			candidate_delegation_count: u32,
			candidate_auto_compounding_delegation_count: u32,
			delegation_count: u32,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			<AutoCompoundDelegations<T>>::delegate_with_auto_compound(
				candidate,
				delegator,
				amount,
				auto_compound,
				candidate_delegation_count,
				candidate_auto_compounding_delegation_count,
				delegation_count,
			)
		}

		/// Request to revoke an existing delegation. If successful, the delegation is scheduled
		/// to be allowed to be revoked via the `execute_delegation_request` extrinsic.
		/// The delegation receives no rewards for the rounds while a revoke is pending.
		/// A revoke may not be performed if any other scheduled request is pending.
		#[pallet::call_index(22)]
		#[pallet::weight(<T as Config>::WeightInfo::schedule_revoke_delegation(
			T::MaxTopDelegationsPerCandidate::get() + T::MaxBottomDelegationsPerCandidate::get()
		))]
		pub fn schedule_revoke_delegation(
			origin: OriginFor<T>,
			collator: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			Self::delegation_schedule_revoke(collator, delegator)
		}

		/// Bond more for delegators wrt a specific collator candidate.
		#[pallet::call_index(23)]
		#[pallet::weight(<T as Config>::WeightInfo::delegator_bond_more(
			T::MaxTopDelegationsPerCandidate::get() + T::MaxBottomDelegationsPerCandidate::get()
		))]
		pub fn delegator_bond_more(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			let (in_top, weight) = Self::delegation_bond_more_without_event(
				delegator.clone(),
				candidate.clone(),
				more.clone(),
			)?;
			Pallet::<T>::deposit_event(Event::DelegationIncreased {
				delegator,
				candidate,
				amount: more,
				in_top,
			});

			Ok(Some(weight).into())
		}

		/// Request bond less for delegators wrt a specific collator candidate. The delegation's
		/// rewards for rounds while the request is pending use the reduced bonded amount.
		/// A bond less may not be performed if any other scheduled request is pending.
		#[pallet::call_index(24)]
		#[pallet::weight(<T as Config>::WeightInfo::schedule_delegator_bond_less(
			T::MaxTopDelegationsPerCandidate::get() + T::MaxBottomDelegationsPerCandidate::get()
		))]
		pub fn schedule_delegator_bond_less(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			less: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			Self::delegation_schedule_bond_decrease(candidate, delegator, less)
		}

		/// Execute pending request to change an existing delegation
		#[pallet::call_index(25)]
		#[pallet::weight(<T as Config>::WeightInfo::execute_delegator_revoke_delegation_worst())]
		pub fn execute_delegation_request(
			origin: OriginFor<T>,
			delegator: T::AccountId,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?; // we may want to reward caller if caller != delegator
			Self::delegation_execute_scheduled_request(candidate, delegator)
		}

		/// Cancel request to change an existing delegation.
		#[pallet::call_index(26)]
		#[pallet::weight(<T as Config>::WeightInfo::cancel_delegation_request(350))]
		pub fn cancel_delegation_request(
			origin: OriginFor<T>,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			Self::delegation_cancel_request(candidate, delegator)
		}

		/// Sets the auto-compounding reward percentage for a delegation.
		#[pallet::call_index(27)]
		#[pallet::weight(<T as Config>::WeightInfo::set_auto_compound(
			*candidate_auto_compounding_delegation_count_hint,
			*delegation_count_hint,
		))]
		pub fn set_auto_compound(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			value: Percent,
			candidate_auto_compounding_delegation_count_hint: u32,
			delegation_count_hint: u32,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			<AutoCompoundDelegations<T>>::set_auto_compound(
				candidate,
				delegator,
				value,
				candidate_auto_compounding_delegation_count_hint,
				delegation_count_hint,
			)
		}

		/// Hotfix to remove existing empty entries for candidates that have left.
		#[pallet::call_index(28)]
		#[pallet::weight(
			T::DbWeight::get().reads_writes(2 * candidates.len() as u64, candidates.len() as u64)
		)]
		pub fn hotfix_remove_delegation_requests_exited_candidates(
			origin: OriginFor<T>,
			candidates: Vec<T::AccountId>,
		) -> DispatchResult {
			ensure_signed(origin)?;
			ensure!(candidates.len() < 100, <Error<T>>::InsufficientBalance);
			for candidate in &candidates {
				ensure!(
					<CandidateInfo<T>>::get(&candidate).is_none(),
					<Error<T>>::CandidateNotLeaving
				);
				ensure!(
					<DelegationScheduledRequests<T>>::get(&candidate).is_empty(),
					<Error<T>>::CandidateNotLeaving
				);
			}

			for candidate in candidates {
				<DelegationScheduledRequests<T>>::remove(candidate);
			}

			Ok(().into())
		}

		/// Notify a collator is inactive during MaxOfflineRounds
		#[pallet::call_index(29)]
		#[pallet::weight(<T as Config>::WeightInfo::notify_inactive_collator())]
		pub fn notify_inactive_collator(
			origin: OriginFor<T>,
			collator: T::AccountId,
		) -> DispatchResult {
			ensure!(
				<EnableMarkingOffline<T>>::get(),
				<Error<T>>::MarkingOfflineNotEnabled
			);
			ensure_signed(origin)?;

			let mut collators_len = 0usize;
			let max_collators = <TotalSelected<T>>::get();

			if let Some(len) = <SelectedCandidates<T>>::decode_len() {
				collators_len = len;
			};

			// Check collators length is not below or eq to 66% of max_collators.
			// We use saturating logic here with (2/3)
			// as it is dangerous to use floating point numbers directly.
			ensure!(
				collators_len * 3 > (max_collators * 2) as usize,
				<Error<T>>::TooLowCollatorCountToNotifyAsInactive
			);

			let round_info = <Round<T>>::get();
			let max_offline_rounds = T::MaxOfflineRounds::get();

			ensure!(
				round_info.current > max_offline_rounds,
				<Error<T>>::CurrentRoundTooLow
			);

			// Have rounds_to_check = [8,9]
			// in case we are in round 10 for instance
			// with MaxOfflineRounds = 2
			let first_round_to_check = round_info.current.saturating_sub(max_offline_rounds);
			let rounds_to_check = first_round_to_check..round_info.current;

			// If this counter is eq to max_offline_rounds,
			// the collator should be notified as inactive
			let mut inactive_counter: RoundIndex = 0u32;

			// Iter rounds and check whether the collator has been inactive
			for r in rounds_to_check {
				if <WasInactive<T>>::get(r, &collator).is_some() {
					inactive_counter = inactive_counter.saturating_add(1);
				}
			}

			if inactive_counter == max_offline_rounds {
				let _ = T::OnInactiveCollator::on_inactive_collator(
					collator.clone(),
					round_info.current.saturating_sub(1),
				);
			} else {
				return Err(<Error<T>>::CannotBeNotifiedAsInactive.into());
			}

			Ok(().into())
		}

		/// Enable/Disable marking offline feature
		#[pallet::call_index(30)]
		#[pallet::weight(
			Weight::from_parts(3_000_000u64, 4_000u64)
				.saturating_add(T::DbWeight::get().writes(1u64))
		)]
		pub fn enable_marking_offline(origin: OriginFor<T>, value: bool) -> DispatchResult {
			ensure_root(origin)?;
			<EnableMarkingOffline<T>>::set(value);
			Ok(())
		}

		/// Force join the set of collator candidates.
		/// It will skip the minimum required bond check.
		#[pallet::call_index(31)]
		#[pallet::weight(<T as Config>::WeightInfo::join_candidates(*candidate_count))]
		pub fn force_join_candidates(
			origin: OriginFor<T>,
			account: T::AccountId,
			bond: BalanceOf<T>,
			candidate_count: u32,
		) -> DispatchResultWithPostInfo {
			T::MonetaryGovernanceOrigin::ensure_origin(origin.clone())?;
			Self::join_candidates_inner(account, bond, candidate_count)
		}

		/// Set the inflation distribution configuration.
		#[pallet::call_index(32)]
		#[pallet::weight(<T as Config>::WeightInfo::set_inflation_distribution_config())]
		pub fn set_inflation_distribution_config(
			origin: OriginFor<T>,
			new: InflationDistributionConfig<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			T::MonetaryGovernanceOrigin::ensure_origin(origin)?;
			let old = <InflationDistributionInfo<T>>::get().0;
			let new = new.0;
			ensure!(old != new, Error::<T>::NoWritingSameValue);
			let total_percent = new.iter().fold(0, |acc, x| acc + x.percent.deconstruct());
			ensure!(
				total_percent <= 100,
				Error::<T>::TotalInflationDistributionPercentExceeds100,
			);
			<InflationDistributionInfo<T>>::put::<InflationDistributionConfig<T::AccountId>>(
				new.clone().into(),
			);
			Self::deposit_event(Event::InflationDistributionConfigUpdated {
				old: old.into(),
				new: new.into(),
			});
			Ok(().into())
		}
		/// Batch migrate locks to freezes for a list of accounts.
		///
		/// This function allows migrating multiple accounts from the old lock-based
		/// staking to the new freeze-based staking in a single transaction.
		///
		/// Parameters:
		/// - `accounts`: List of tuples containing (account_id, is_collator)
		///   where is_collator indicates if the account is a collator (true) or delegator (false)
		///
		/// The maximum number of accounts that can be migrated in one batch is 100.
		#[pallet::call_index(33)]
		#[pallet::weight({
			T::WeightInfo::migrate_locks_to_freezes_batch_delegators(MAX_ACCOUNTS_PER_MIGRATION_BATCH).max(T::WeightInfo::migrate_locks_to_freezes_batch_candidates(MAX_ACCOUNTS_PER_MIGRATION_BATCH))
		})]
		pub fn migrate_locks_to_freezes_batch(
			origin: OriginFor<T>,
			accounts: BoundedVec<(T::AccountId, bool), ConstU32<MAX_ACCOUNTS_PER_MIGRATION_BATCH>>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			for (account, is_collator) in accounts.iter() {
				// Attempt migration, ignoring any errors to allow partial migration
				Self::check_and_migrate_lock(account, *is_collator)?;
			}

			Ok(())
		}
	}

	/// Represents a payout made via `pay_one_collator_reward`.
	pub(crate) enum RewardPayment {
		/// A collator was paid
		Paid,
		/// A collator was skipped for payment. This can happen if they haven't been awarded any
		/// points, that is, they did not produce any blocks.
		Skipped,
		/// All collator payments have been processed.
		Finished,
	}

	impl<T: Config> Pallet<T> {
		/// Check if an account has been migrated from lock to freeze.
		///
		/// Returns `true` if migration was performed, `false` if already migrated or is not a collator/delegator
		///
		/// `is_collator` determines whether the account is a collator or delegator
		fn check_and_migrate_lock(
			account: &T::AccountId,
			is_collator: bool,
		) -> Result<bool, DispatchError> {
			use frame_support::traits::{fungible::MutateFreeze, LockableCurrency};

			// Check if already migrated
			if is_collator {
				if <MigratedCandidates<T>>::contains_key(account) {
					return Ok(false);
				}
			} else {
				if <MigratedDelegators<T>>::contains_key(account) {
					return Ok(false);
				}
			}

			// Not migrated yet, proceed with migration
			let (lock_id, freeze_reason) = if is_collator {
				(COLLATOR_LOCK_ID, FreezeReason::StakingCollator)
			} else {
				(DELEGATOR_LOCK_ID, FreezeReason::StakingDelegator)
			};

			// Get the amount that should be locked/frozen from storage
			let amount = if is_collator {
				// For collators, get the bond amount from storage
				match <CandidateInfo<T>>::get(account) {
					Some(info) => info.bond,
					None => return Ok(false),
				}
			} else {
				// For delegators, get the total delegated amount from storage
				match <DelegatorState<T>>::get(account) {
					Some(state) => state.total,
					None => return Ok(false),
				}
			};

			if amount > BalanceOf::<T>::zero() {
				// Remove any existing lock
				T::Currency::remove_lock(lock_id, account);

				// Set the freeze
				T::Currency::set_freeze(&freeze_reason.into(), account, amount)?;
			}

			if is_collator {
				<MigratedCandidates<T>>::insert(account, ());
			} else {
				<MigratedDelegators<T>>::insert(account, ());
			}

			Ok(true)
		}

		/// Set freeze with lazy migration support
		/// This will check for existing locks and migrate them before setting the freeze
		///
		/// `is_collator` determines whether the account is a collator or delegator
		pub(crate) fn freeze_extended(
			account: &T::AccountId,
			amount: BalanceOf<T>,
			is_collator: bool,
		) -> DispatchResult {
			use frame_support::traits::fungible::MutateFreeze;

			// First check and migrate any existing lock
			let _ = Self::check_and_migrate_lock(account, is_collator)?;

			// Now set the freeze
			let freeze_reason = if is_collator {
				FreezeReason::StakingCollator
			} else {
				FreezeReason::StakingDelegator
			};

			T::Currency::set_freeze(&freeze_reason.into(), account, amount)
		}

		/// Thaw with lazy migration support
		/// This will check for existing locks and remove them before thawing
		///
		/// `is_collator` determines whether the account is a collator or delegator
		pub(crate) fn thaw_extended(account: &T::AccountId, is_collator: bool) -> DispatchResult {
			use frame_support::traits::{fungible::MutateFreeze, LockableCurrency};

			// First check and remove any existing lock
			let lock_id = if is_collator {
				COLLATOR_LOCK_ID
			} else {
				DELEGATOR_LOCK_ID
			};

			// Remove the lock if it exists
			T::Currency::remove_lock(lock_id, account);

			// Now thaw the freeze
			let freeze_reason = if is_collator {
				FreezeReason::StakingCollator
			} else {
				FreezeReason::StakingDelegator
			};

			let _ = T::Currency::thaw(&freeze_reason.into(), account);
			Ok(())
		}

		/// Get frozen balance with lazy migration support
		/// This will check for existing locks and migrate them before returning the frozen balance
		///
		/// `is_collator` determines whether the account is a collator or delegator
		pub(crate) fn balance_frozen_extended(
			account: &T::AccountId,
			is_collator: bool,
		) -> Option<BalanceOf<T>> {
			// First check and migrate any existing lock
			// We ignore the result as we want to return the frozen balance regardless
			let _ = Self::check_and_migrate_lock(account, is_collator);

			// Now return the frozen balance
			if is_collator {
				<CandidateInfo<T>>::get(account).map(|info| info.bond)
			} else {
				<DelegatorState<T>>::get(account).map(|state| state.total)
			}
		}

		pub fn set_candidate_bond_to_zero(acc: &T::AccountId) -> Weight {
			let actual_weight =
				<T as Config>::WeightInfo::set_candidate_bond_to_zero(T::MaxCandidates::get());
			if let Some(mut state) = <CandidateInfo<T>>::get(&acc) {
				state.bond_less::<T>(acc.clone(), state.bond);
				<CandidateInfo<T>>::insert(&acc, state);
			}
			actual_weight
		}

		pub fn is_delegator(acc: &T::AccountId) -> bool {
			<DelegatorState<T>>::get(acc).is_some()
		}

		pub fn is_candidate(acc: &T::AccountId) -> bool {
			<CandidateInfo<T>>::get(acc).is_some()
		}

		pub fn is_selected_candidate(acc: &T::AccountId) -> bool {
			<SelectedCandidates<T>>::get().binary_search(acc).is_ok()
		}

		pub fn join_candidates_inner(
			acc: T::AccountId,
			bond: BalanceOf<T>,
			candidate_count: u32,
		) -> DispatchResultWithPostInfo {
			ensure!(!Self::is_candidate(&acc), Error::<T>::CandidateExists);
			ensure!(!Self::is_delegator(&acc), Error::<T>::DelegatorExists);
			let mut candidates = <CandidatePool<T>>::get();
			let old_count = candidates.0.len() as u32;
			ensure!(
				candidate_count >= old_count,
				Error::<T>::TooLowCandidateCountWeightHintJoinCandidates
			);
			let maybe_inserted_candidate = candidates
				.try_insert(Bond {
					owner: acc.clone(),
					amount: bond,
				})
				.map_err(|_| Error::<T>::CandidateLimitReached)?;
			ensure!(maybe_inserted_candidate, Error::<T>::CandidateExists);

			ensure!(
				Self::get_collator_stakable_free_balance(&acc) >= bond,
				Error::<T>::InsufficientBalance,
			);
			Self::freeze_extended(&acc, bond, true).map_err(|_| Error::<T>::InsufficientBalance)?;
			let candidate = CandidateMetadata::new(bond);
			<CandidateInfo<T>>::insert(&acc, candidate);
			let empty_delegations: Delegations<T::AccountId, BalanceOf<T>> = Default::default();
			// insert empty top delegations
			<TopDelegations<T>>::insert(&acc, empty_delegations.clone());
			// insert empty bottom delegations
			<BottomDelegations<T>>::insert(&acc, empty_delegations);
			<CandidatePool<T>>::put(candidates);
			let new_total = <Total<T>>::get().saturating_add(bond);
			<Total<T>>::put(new_total);
			Self::deposit_event(Event::JoinedCollatorCandidates {
				account: acc,
				amount_locked: bond,
				new_total_amt_locked: new_total,
			});
			Ok(().into())
		}

		pub fn go_offline_inner(collator: T::AccountId) -> DispatchResultWithPostInfo {
			let mut state = <CandidateInfo<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let mut candidates = <CandidatePool<T>>::get();
			let actual_weight = <T as Config>::WeightInfo::go_offline(candidates.0.len() as u32);

			ensure!(
				state.is_active(),
				DispatchErrorWithPostInfo {
					post_info: Some(actual_weight).into(),
					error: <Error<T>>::AlreadyOffline.into(),
				}
			);
			state.go_offline();

			if candidates.remove(&Bond::from_owner(collator.clone())) {
				<CandidatePool<T>>::put(candidates);
			}
			<CandidateInfo<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateWentOffline {
				candidate: collator,
			});
			Ok(Some(actual_weight).into())
		}

		pub fn go_online_inner(collator: T::AccountId) -> DispatchResultWithPostInfo {
			let mut state = <CandidateInfo<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let mut candidates = <CandidatePool<T>>::get();
			let actual_weight = <T as Config>::WeightInfo::go_online(candidates.0.len() as u32);

			ensure!(
				!state.is_active(),
				DispatchErrorWithPostInfo {
					post_info: Some(actual_weight).into(),
					error: <Error<T>>::AlreadyActive.into(),
				}
			);
			ensure!(
				!state.is_leaving(),
				DispatchErrorWithPostInfo {
					post_info: Some(actual_weight).into(),
					error: <Error<T>>::CannotGoOnlineIfLeaving.into(),
				}
			);
			state.go_online();

			let maybe_inserted_candidate = candidates
				.try_insert(Bond {
					owner: collator.clone(),
					amount: state.total_counted,
				})
				.map_err(|_| Error::<T>::CandidateLimitReached)?;
			ensure!(
				maybe_inserted_candidate,
				DispatchErrorWithPostInfo {
					post_info: Some(actual_weight).into(),
					error: <Error<T>>::AlreadyActive.into(),
				},
			);

			<CandidatePool<T>>::put(candidates);
			<CandidateInfo<T>>::insert(&collator, state);
			Self::deposit_event(Event::CandidateBackOnline {
				candidate: collator,
			});
			Ok(Some(actual_weight).into())
		}

		pub fn candidate_bond_more_inner(
			collator: T::AccountId,
			more: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let mut state = <CandidateInfo<T>>::get(&collator).ok_or(Error::<T>::CandidateDNE)?;
			let actual_weight =
				<T as Config>::WeightInfo::candidate_bond_more(T::MaxCandidates::get());

			state
				.bond_more::<T>(collator.clone(), more)
				.map_err(|err| DispatchErrorWithPostInfo {
					post_info: Some(actual_weight).into(),
					error: err,
				})?;
			let (is_active, total_counted) = (state.is_active(), state.total_counted);
			<CandidateInfo<T>>::insert(&collator, state);
			if is_active {
				Self::update_active(collator, total_counted);
			}
			Ok(Some(actual_weight).into())
		}

		pub fn execute_candidate_bond_less_inner(
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let mut state = <CandidateInfo<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			let actual_weight =
				<T as Config>::WeightInfo::execute_candidate_bond_less(T::MaxCandidates::get());

			state
				.execute_bond_less::<T>(candidate.clone())
				.map_err(|err| DispatchErrorWithPostInfo {
					post_info: Some(actual_weight).into(),
					error: err,
				})?;
			<CandidateInfo<T>>::insert(&candidate, state);
			Ok(Some(actual_weight).into())
		}

		pub fn execute_leave_candidates_inner(
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let state = <CandidateInfo<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			let actual_auto_compound_delegation_count =
				<AutoCompoundingDelegations<T>>::decode_len(&candidate).unwrap_or_default() as u32;

			// TODO use these to return actual weight used via `execute_leave_candidates_ideal`
			let actual_delegation_count = state.delegation_count;
			let actual_weight = <T as Config>::WeightInfo::execute_leave_candidates_ideal(
				actual_delegation_count,
				actual_auto_compound_delegation_count,
			);

			state
				.can_leave::<T>()
				.map_err(|err| DispatchErrorWithPostInfo {
					post_info: Some(actual_weight).into(),
					error: err,
				})?;
			let return_stake = |bond: Bond<T::AccountId, BalanceOf<T>>| {
				// remove delegation from delegator state
				let mut delegator = DelegatorState::<T>::get(&bond.owner).expect(
					"Collator state and delegator state are consistent.
						Collator state has a record of this delegation. Therefore,
						Delegator state also has a record. qed.",
				);

				if let Some(remaining) = delegator.rm_delegation::<T>(&candidate) {
					Self::delegation_remove_request_with_state(
						&candidate,
						&bond.owner,
						&mut delegator,
					);
					<AutoCompoundDelegations<T>>::remove_auto_compound(&candidate, &bond.owner);

					if remaining.is_zero() {
						// we do not remove the scheduled delegation requests from other collators
						// since it is assumed that they were removed incrementally before only the
						// last delegation was left.
						<DelegatorState<T>>::remove(&bond.owner);
						// Thaw all frozen funds for delegator
						let _ = Self::thaw_extended(&bond.owner, false);
					} else {
						<DelegatorState<T>>::insert(&bond.owner, delegator);
					}
				} else {
					// TODO: review. we assume here that this delegator has no remaining staked
					// balance, so we ensure the funds are freed
					let _ = Self::thaw_extended(&bond.owner, false);
				}
			};
			// total backing stake is at least the candidate self bond
			let mut total_backing = state.bond;
			// return all top delegations
			let top_delegations =
				<TopDelegations<T>>::take(&candidate).expect("CandidateInfo existence checked");
			for bond in top_delegations.delegations {
				return_stake(bond);
			}
			total_backing = total_backing.saturating_add(top_delegations.total);
			// return all bottom delegations
			let bottom_delegations =
				<BottomDelegations<T>>::take(&candidate).expect("CandidateInfo existence checked");
			for bond in bottom_delegations.delegations {
				return_stake(bond);
			}
			total_backing = total_backing.saturating_add(bottom_delegations.total);
			// Thaw all frozen funds for collator
			let _ = Self::thaw_extended(&candidate, true);
			<CandidateInfo<T>>::remove(&candidate);
			<DelegationScheduledRequests<T>>::remove(&candidate);
			<AutoCompoundingDelegations<T>>::remove(&candidate);
			<TopDelegations<T>>::remove(&candidate);
			<BottomDelegations<T>>::remove(&candidate);
			let new_total_staked = <Total<T>>::get().saturating_sub(total_backing);
			<Total<T>>::put(new_total_staked);
			Self::deposit_event(Event::CandidateLeft {
				ex_candidate: candidate,
				unlocked_amount: total_backing,
				new_total_amt_locked: new_total_staked,
			});
			Ok(Some(actual_weight).into())
		}

		/// Returns an account's stakable balance (including the reserved) which is not frozen in delegation staking
		pub fn get_delegator_stakable_balance(acc: &T::AccountId) -> BalanceOf<T> {
			let total_balance =
				T::Currency::balance(acc).saturating_add(T::Currency::reserved_balance(acc));
			if let Some(frozen_balance) = Self::balance_frozen_extended(acc, false) {
				return total_balance.saturating_sub(frozen_balance);
			}
			total_balance
		}

		/// Returns an account's free balance which is not frozen in collator staking
		pub fn get_collator_stakable_free_balance(acc: &T::AccountId) -> BalanceOf<T> {
			let total_balance = T::Currency::balance(acc);
			if let Some(frozen_balance) = Self::balance_frozen_extended(acc, true) {
				return total_balance.saturating_sub(frozen_balance);
			}
			total_balance
		}

		/// Returns a delegations auto-compound value.
		pub fn delegation_auto_compound(
			candidate: &T::AccountId,
			delegator: &T::AccountId,
		) -> Percent {
			<AutoCompoundDelegations<T>>::auto_compound(candidate, delegator)
		}

		/// Caller must ensure candidate is active before calling
		pub(crate) fn update_active(candidate: T::AccountId, total: BalanceOf<T>) {
			let mut candidates = <CandidatePool<T>>::get();
			candidates.remove(&Bond::from_owner(candidate.clone()));
			candidates
				.try_insert(Bond {
					owner: candidate,
					amount: total,
				})
				.expect(
					"the candidate is removed in previous step so the length cannot increase; qed",
				);
			<CandidatePool<T>>::put(candidates);
		}

		/// Compute round issuance based on duration of the given round
		fn compute_issuance(round_duration: u64, round_length: u32) -> BalanceOf<T> {
			let ideal_duration: BalanceOf<T> = round_length
				.saturating_mul(T::BlockTime::get() as u32)
				.into();
			let config = <InflationConfig<T>>::get();
			let round_issuance = crate::inflation::round_issuance_range::<T>(config.round);

			// Initial formula: (round_duration / ideal_duration) * ideal_issuance
			// We multiply before the division to reduce rounding effects
			BalanceOf::<T>::from(round_duration as u32).saturating_mul(round_issuance.ideal)
				/ (ideal_duration)
		}

		/// Remove delegation from candidate state
		/// Amount input should be retrieved from delegator and it informs the storage lookups
		pub(crate) fn delegator_leaves_candidate(
			candidate: T::AccountId,
			delegator: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let mut state = <CandidateInfo<T>>::get(&candidate).ok_or(Error::<T>::CandidateDNE)?;
			state.rm_delegation_if_exists::<T>(&candidate, delegator.clone(), amount)?;
			let new_total_locked = <Total<T>>::get().saturating_sub(amount);
			<Total<T>>::put(new_total_locked);
			let new_total = state.total_counted;
			<CandidateInfo<T>>::insert(&candidate, state);
			Self::deposit_event(Event::DelegatorLeftCandidate {
				delegator: delegator,
				candidate: candidate,
				unstaked_amount: amount,
				total_candidate_staked: new_total,
			});
			Ok(())
		}

		pub(crate) fn prepare_staking_payouts(
			round_info: RoundInfo<BlockNumberFor<T>>,
			round_duration: u64,
		) -> Weight {
			let RoundInfo {
				current: now,
				length: round_length,
				..
			} = round_info;

			// This function is called right after the round index increment,
			// and the goal is to compute the payout informations for the round that just ended.
			// We don't need to saturate here because the genesis round is 1.
			let prepare_payout_for_round = now - 1;

			// Return early if there is no blocks for this round
			if <Points<T>>::get(prepare_payout_for_round).is_zero() {
				return Weight::zero();
			}

			// Compute total issuance based on round duration
			let total_issuance = Self::compute_issuance(round_duration, round_length);
			// reserve portion of issuance for parachain bond account
			let mut left_issuance = total_issuance;

			let configs = <InflationDistributionInfo<T>>::get().0;
			for (index, config) in configs.iter().enumerate() {
				if config.percent.is_zero() {
					continue;
				}
				let reserve = config.percent * total_issuance;
				if frame_system::Pallet::<T>::account_exists(&config.account) {
					if let Ok(minted) = T::Currency::mint_into(&config.account, reserve) {
						// update round issuance if minting succeeds
						left_issuance = left_issuance.saturating_sub(minted);
						Self::deposit_event(Event::InflationDistributed {
							index: index as u32,
							account: config.account.clone(),
							value: minted,
						});
					}
				}
			}

			let payout = DelayedPayout {
				round_issuance: total_issuance,
				total_staking_reward: left_issuance,
				collator_commission: <CollatorCommission<T>>::get(),
			};

			<DelayedPayouts<T>>::insert(prepare_payout_for_round, payout);

			<T as Config>::WeightInfo::prepare_staking_payouts()
		}

		/// Wrapper around pay_one_collator_reward which handles the following logic:
		/// * whether or not a payout needs to be made
		/// * cleaning up when payouts are done
		/// * returns the weight consumed by pay_one_collator_reward if applicable
		fn handle_delayed_payouts(now: RoundIndex) -> Weight {
			let delay = T::RewardPaymentDelay::get();

			// don't underflow uint
			if now < delay {
				return Weight::from_parts(0u64, 0);
			}

			let paid_for_round = now.saturating_sub(delay);

			if let Some(payout_info) = <DelayedPayouts<T>>::get(paid_for_round) {
				let result = Self::pay_one_collator_reward(paid_for_round, payout_info);

				// clean up storage items that we no longer need
				if matches!(result.0, RewardPayment::Finished) {
					<DelayedPayouts<T>>::remove(paid_for_round);
					<Points<T>>::remove(paid_for_round);
				}
				result.1 // weight consumed by pay_one_collator_reward
			} else {
				Weight::from_parts(0u64, 0)
			}
		}

		/// Payout a single collator from the given round.
		///
		/// Returns an optional tuple of (Collator's AccountId, total paid)
		/// or None if there were no more payouts to be made for the round.
		pub(crate) fn pay_one_collator_reward(
			paid_for_round: RoundIndex,
			payout_info: DelayedPayout<BalanceOf<T>>,
		) -> (RewardPayment, Weight) {
			// 'early_weight' tracks weight used for reads/writes done early in this fn before its
			// early-exit codepaths.
			let mut early_weight = Weight::zero();

			// TODO: it would probably be optimal to roll Points into the DelayedPayouts storage
			// item so that we do fewer reads each block
			let total_points = <Points<T>>::get(paid_for_round);
			early_weight = early_weight.saturating_add(T::DbWeight::get().reads_writes(1, 0));

			if total_points.is_zero() {
				// TODO: this case is obnoxious... it's a value query, so it could mean one of two
				// different logic errors:
				// 1. we removed it before we should have
				// 2. we called pay_one_collator_reward when we were actually done with deferred
				//    payouts
				log::warn!("pay_one_collator_reward called with no <Points<T>> for the round!");
				return (RewardPayment::Finished, early_weight);
			}

			let collator_fee = payout_info.collator_commission;
			let collator_issuance = collator_fee * payout_info.round_issuance;
			if let Some((collator, state)) =
				<AtStake<T>>::iter_prefix(paid_for_round).drain().next()
			{
				// read and kill AtStake
				early_weight = early_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

				// Take the awarded points for the collator
				let pts = <AwardedPts<T>>::take(paid_for_round, &collator);
				// read and kill AwardedPts
				early_weight = early_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
				if pts == 0 {
					return (RewardPayment::Skipped, early_weight);
				}

				// 'extra_weight' tracks weight returned from fns that we delegate to which can't be
				// known ahead of time.
				let mut extra_weight = Weight::zero();
				let pct_due = Perbill::from_rational(pts, total_points);
				let total_paid = pct_due * payout_info.total_staking_reward;
				let mut amt_due = total_paid;

				let num_delegators = state.delegations.len();
				let mut num_paid_delegations = 0u32;
				let mut num_auto_compounding = 0u32;
				let num_scheduled_requests =
					<DelegationScheduledRequests<T>>::decode_len(&collator).unwrap_or_default();
				if state.delegations.is_empty() {
					// solo collator with no delegators
					extra_weight = extra_weight
						.saturating_add(T::PayoutCollatorReward::payout_collator_reward(
							paid_for_round,
							collator.clone(),
							amt_due,
						))
						.saturating_add(T::OnCollatorPayout::on_collator_payout(
							paid_for_round,
							collator.clone(),
							amt_due,
						));
				} else {
					// pay collator first; commission + due_portion
					let collator_pct = Perbill::from_rational(state.bond, state.total);
					let commission = pct_due * collator_issuance;
					amt_due = amt_due.saturating_sub(commission);
					let collator_reward = (collator_pct * amt_due).saturating_add(commission);
					extra_weight = extra_weight
						.saturating_add(T::PayoutCollatorReward::payout_collator_reward(
							paid_for_round,
							collator.clone(),
							collator_reward,
						))
						.saturating_add(T::OnCollatorPayout::on_collator_payout(
							paid_for_round,
							collator.clone(),
							collator_reward,
						));

					// pay delegators due portion
					for BondWithAutoCompound {
						owner,
						amount,
						auto_compound,
					} in state.delegations
					{
						let percent = Perbill::from_rational(amount, state.total);
						let due = percent * amt_due;
						if !due.is_zero() {
							num_auto_compounding += if auto_compound.is_zero() { 0 } else { 1 };
							num_paid_delegations += 1u32;
							Self::mint_and_compound(
								due,
								auto_compound.clone(),
								collator.clone(),
								owner.clone(),
							);
						}
					}
				}

				extra_weight = extra_weight.saturating_add(
					<T as Config>::WeightInfo::pay_one_collator_reward_best(
						num_paid_delegations,
						num_auto_compounding,
						num_scheduled_requests as u32,
					),
				);

				(
					RewardPayment::Paid,
					<T as Config>::WeightInfo::pay_one_collator_reward(num_delegators as u32)
						.saturating_add(extra_weight),
				)
			} else {
				// Note that we don't clean up storage here; it is cleaned up in
				// handle_delayed_payouts()
				(RewardPayment::Finished, Weight::from_parts(0u64, 0))
			}
		}

		/// Compute the top `TotalSelected` candidates in the CandidatePool and return
		/// a vec of their AccountIds (sorted by AccountId).
		///
		/// If the returned vec is empty, the previous candidates should be used.
		pub fn compute_top_candidates() -> Vec<T::AccountId> {
			let top_n = <TotalSelected<T>>::get() as usize;
			if top_n == 0 {
				return vec![];
			}

			let candidates = <CandidatePool<T>>::get().0;

			// If the number of candidates is greater than top_n, select the candidates with higher
			// amount. Otherwise, return all the candidates.
			if candidates.len() > top_n {
				// Partially sort candidates such that element at index `top_n - 1` is sorted, and
				// all the elements in the range 0..top_n are the top n elements.
				let sorted_candidates = candidates
					.try_mutate(|inner| {
						inner.select_nth_unstable_by(top_n - 1, |a, b| {
							// Order by amount, then owner. The owner is needed to ensure a stable order
							// when two accounts have the same amount.
							a.amount
								.cmp(&b.amount)
								.then_with(|| a.owner.cmp(&b.owner))
								.reverse()
						});
					})
					.expect("sort cannot increase item count; qed");

				let mut collators = sorted_candidates
					.into_iter()
					.take(top_n)
					.map(|x| x.owner)
					.collect::<Vec<_>>();

				// Sort collators by AccountId
				collators.sort();

				collators
			} else {
				// Return all candidates
				// The candidates are already sorted by AccountId, so no need to sort again
				candidates.into_iter().map(|x| x.owner).collect::<Vec<_>>()
			}
		}
		/// Best as in most cumulatively supported in terms of stake
		/// Returns [collator_count, delegation_count, total staked]
		pub(crate) fn select_top_candidates(now: RoundIndex) -> (Weight, u32, u32, BalanceOf<T>) {
			let (mut collator_count, mut delegation_count, mut total) =
				(0u32, 0u32, BalanceOf::<T>::zero());
			// choose the top TotalSelected qualified candidates, ordered by stake
			let collators = Self::compute_top_candidates();
			if collators.is_empty() {
				// SELECTION FAILED TO SELECT >=1 COLLATOR => select collators from previous round
				let last_round = now.saturating_sub(1u32);
				let mut total_per_candidate: BTreeMap<T::AccountId, BalanceOf<T>> = BTreeMap::new();
				// set this round AtStake to last round AtStake
				for (account, snapshot) in <AtStake<T>>::iter_prefix(last_round) {
					collator_count = collator_count.saturating_add(1u32);
					delegation_count =
						delegation_count.saturating_add(snapshot.delegations.len() as u32);
					total = total.saturating_add(snapshot.total);
					total_per_candidate.insert(account.clone(), snapshot.total);
					<AtStake<T>>::insert(now, account, snapshot);
				}
				// `SelectedCandidates` remains unchanged from last round
				// emit CollatorChosen event for tools that use this event
				for candidate in <SelectedCandidates<T>>::get() {
					let snapshot_total = total_per_candidate
						.get(&candidate)
						.expect("all selected candidates have snapshots");
					Self::deposit_event(Event::CollatorChosen {
						round: now,
						collator_account: candidate,
						total_exposed_amount: *snapshot_total,
					})
				}
				let weight = <T as Config>::WeightInfo::select_top_candidates(0, 0);
				return (weight, collator_count, delegation_count, total);
			}

			// snapshot exposure for round for weighting reward distribution
			for account in collators.iter() {
				let state = <CandidateInfo<T>>::get(account)
					.expect("all members of CandidateQ must be candidates");

				collator_count = collator_count.saturating_add(1u32);
				delegation_count = delegation_count.saturating_add(state.delegation_count);
				total = total.saturating_add(state.total_counted);
				let CountedDelegations {
					uncounted_stake,
					rewardable_delegations,
				} = Self::get_rewardable_delegators(&account);
				let total_counted = state.total_counted.saturating_sub(uncounted_stake);

				let auto_compounding_delegations = <AutoCompoundingDelegations<T>>::get(&account)
					.into_iter()
					.map(|x| (x.delegator, x.value))
					.collect::<BTreeMap<_, _>>();
				let rewardable_delegations = rewardable_delegations
					.into_iter()
					.map(|d| BondWithAutoCompound {
						owner: d.owner.clone(),
						amount: d.amount,
						auto_compound: auto_compounding_delegations
							.get(&d.owner)
							.cloned()
							.unwrap_or_else(|| Percent::zero()),
					})
					.collect();

				let snapshot = CollatorSnapshot {
					bond: state.bond,
					delegations: rewardable_delegations,
					total: total_counted,
				};
				<AtStake<T>>::insert(now, account, snapshot);
				Self::deposit_event(Event::CollatorChosen {
					round: now,
					collator_account: account.clone(),
					total_exposed_amount: state.total_counted,
				});
			}
			// insert canonical collator set
			<SelectedCandidates<T>>::put(
				BoundedVec::try_from(collators)
					.expect("subset of collators is always less than or equal to max candidates"),
			);

			let avg_delegator_count = delegation_count.checked_div(collator_count).unwrap_or(0);
			let weight = <T as Config>::WeightInfo::select_top_candidates(
				collator_count,
				avg_delegator_count,
			);
			(weight, collator_count, delegation_count, total)
		}

		/// Apply the delegator intent for revoke and decrease in order to build the
		/// effective list of delegators with their intended bond amount.
		///
		/// This will:
		/// - if [DelegationChange::Revoke] is outstanding, set the bond amount to 0.
		/// - if [DelegationChange::Decrease] is outstanding, subtract the bond by specified amount.
		/// - else, do nothing
		///
		/// The intended bond amounts will be used while calculating rewards.
		pub(crate) fn get_rewardable_delegators(collator: &T::AccountId) -> CountedDelegations<T> {
			let requests = <DelegationScheduledRequests<T>>::get(collator)
				.into_iter()
				.map(|x| (x.delegator, x.action))
				.collect::<BTreeMap<_, _>>();
			let mut uncounted_stake = BalanceOf::<T>::zero();
			let rewardable_delegations = <TopDelegations<T>>::get(collator)
				.expect("all members of CandidateQ must be candidates")
				.delegations
				.into_iter()
				.map(|mut bond| {
					bond.amount = match requests.get(&bond.owner) {
						None => bond.amount,
						Some(DelegationAction::Revoke(_)) => {
							uncounted_stake = uncounted_stake.saturating_add(bond.amount);
							BalanceOf::<T>::zero()
						}
						Some(DelegationAction::Decrease(amount)) => {
							uncounted_stake = uncounted_stake.saturating_add(*amount);
							bond.amount.saturating_sub(*amount)
						}
					};

					bond
				})
				.collect();
			CountedDelegations {
				uncounted_stake,
				rewardable_delegations,
			}
		}

		/// This function exists as a helper to delegator_bond_more & auto_compound functionality.
		/// Any changes to this function must align with both user-initiated bond increases and
		/// auto-compounding bond increases.
		/// Any feature-specific preconditions should be validated before this function is invoked.
		/// Any feature-specific events must be emitted after this function is invoked.
		pub fn delegation_bond_more_without_event(
			delegator: T::AccountId,
			candidate: T::AccountId,
			more: BalanceOf<T>,
		) -> Result<
			(bool, Weight),
			DispatchErrorWithPostInfo<frame_support::dispatch::PostDispatchInfo>,
		> {
			let mut state = <DelegatorState<T>>::get(&delegator).ok_or(Error::<T>::DelegatorDNE)?;
			ensure!(
				!Self::delegation_request_revoke_exists(&candidate, &delegator),
				Error::<T>::PendingDelegationRevoke
			);

			let actual_weight = <T as Config>::WeightInfo::delegator_bond_more(
				<DelegationScheduledRequests<T>>::decode_len(&candidate).unwrap_or_default() as u32,
			);
			let in_top = state
				.increase_delegation::<T>(candidate.clone(), more)
				.map_err(|err| DispatchErrorWithPostInfo {
					post_info: Some(actual_weight).into(),
					error: err,
				})?;

			Ok((in_top, actual_weight))
		}

		/// Mint a specified reward amount to the beneficiary account. Emits the [Rewarded] event.
		pub fn mint(amt: BalanceOf<T>, to: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
			// Mint rewards to the account
			let minted = T::Currency::mint_into(&to, amt)?;
			Self::deposit_event(Event::Rewarded {
				account: to.clone(),
				rewards: minted,
			});
			Ok(minted)
		}

		/// Mint a specified reward amount to the collator's account. Emits the [Rewarded] event.
		pub fn mint_collator_reward(
			_paid_for_round: RoundIndex,
			collator_id: T::AccountId,
			amt: BalanceOf<T>,
		) -> Weight {
			// Mint rewards to the collator
			if let Err(e) = Self::mint(amt, &collator_id) {
				log::warn!(
					"Failed to mint collator reward for {:?}: {:?}",
					collator_id,
					e
				);
			}

			<T as Config>::WeightInfo::mint_collator_reward()
		}

		/// Mint and compound delegation rewards. The function mints the amount towards the
		/// delegator and tries to compound a specified percent of it back towards the delegation.
		/// If a scheduled delegation revoke exists, then the amount is only minted, and nothing is
		/// compounded. Emits the [Compounded] event.
		pub fn mint_and_compound(
			amt: BalanceOf<T>,
			compound_percent: Percent,
			candidate: T::AccountId,
			delegator: T::AccountId,
		) {
			// Mint rewards to the delegator
			if frame_system::Pallet::<T>::account_exists(&delegator) {
				if let Ok(minted) = Self::mint(amt.clone(), &delegator) {
					let compound_amount = compound_percent.mul_ceil(minted);
					if compound_amount.is_zero() {
						return;
					}

					if let Err(err) = Self::delegation_bond_more_without_event(
						delegator.clone(),
						candidate.clone(),
						compound_amount.clone(),
					) {
						log::debug!(
							"skipped compounding staking reward towards candidate '{:?}' for delegator '{:?}': {:?}",
							candidate,
							delegator,
							err
						);
						return;
					};

					Pallet::<T>::deposit_event(Event::Compounded {
						delegator,
						candidate,
						amount: compound_amount.clone(),
					});
				};
			}
		}

		/// Add reward points to block authors:
		/// * 20 points to the block producer for producing a block in the chain
		fn award_points_to_block_author() {
			let author = T::BlockAuthor::get();
			let now = <Round<T>>::get().current;
			let score_plus_20 = <AwardedPts<T>>::get(now, &author).saturating_add(20);
			<AwardedPts<T>>::insert(now, author, score_plus_20);
			<Points<T>>::mutate(now, |x| *x = x.saturating_add(20));
		}

		/// Marks collators as inactive for the previous round if they received zero awarded points.
		pub fn mark_collators_as_inactive(cur: RoundIndex) -> Weight {
			// This function is called after round index increment,
			// We don't need to saturate here because the genesis round is 1.
			let prev = cur - 1;

			let mut collators_at_stake_count = 0u32;
			for (account, _) in <AtStake<T>>::iter_prefix(prev) {
				collators_at_stake_count = collators_at_stake_count.saturating_add(1u32);
				if <AwardedPts<T>>::get(prev, &account).is_zero() {
					<WasInactive<T>>::insert(prev, account, ());
				}
			}

			<T as Config>::WeightInfo::mark_collators_as_inactive(collators_at_stake_count)
		}

		/// Cleans up historical staking information that is older than MaxOfflineRounds
		/// by removing entries from the WasIactive storage map.
		fn cleanup_inactive_collator_info() {
			let now = <Round<T>>::get().current;
			let minimum_rounds_required = T::MaxOfflineRounds::get() + 1;

			if now < minimum_rounds_required {
				return;
			}

			let _ = <WasInactive<T>>::iter_prefix(now - minimum_rounds_required)
				.drain()
				.next();
		}
	}

	impl<T: Config> nimbus_primitives::CanAuthor<T::AccountId> for Pallet<T> {
		fn can_author(account: &T::AccountId, _slot: &u32) -> bool {
			Self::is_selected_candidate(account)
		}
	}

	impl<T: Config> Get<Vec<T::AccountId>> for Pallet<T> {
		fn get() -> Vec<T::AccountId> {
			Self::selected_candidates().into_inner()
		}
	}
}
