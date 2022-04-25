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

// TODOs :
// - Leaving cancelation, user can choose which kind of shares they want back.
// - Shares convertion should not make leaving requests.

#![cfg_attr(not(feature = "std"), no_std)]

mod pools;
mod rewards;

#[cfg(test)]
mod mock;

pub use pallet::*;

use frame_support::pallet;

#[pallet]
pub mod pallet {
	use {
		super::{pools, rewards},
		frame_support::{
			pallet_prelude::*,
			storage::types::Key,
			traits::{tokens::ExistenceRequirement, Currency, ReservableCurrency},
			transactional,
		},
		frame_system::pallet_prelude::*,
		sp_runtime::{
			traits::{CheckedSub, Zero},
			Perbill,
		},
		sp_std::collections::btree_set::BTreeSet,
	};

	#[cfg(feature = "std")]
	use serde::{Deserialize, Serialize};

	/// Type of balances of the staked Currency and shares.
	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Allow calls to be performed using either share amounts or stake.
	/// When providing stake, calls will convert them into share amounts that are
	/// worth up to the provided stake. The amount of stake thus will be at most the provided
	/// amount.
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(RuntimeDebug, PartialEq, Eq, Encode, Decode, Clone, TypeInfo)]
	pub enum SharesOrStake<T> {
		Shares(T),
		Stake(T),
	}

	/// Identifier used when executing a pending leaving request.
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(RuntimeDebug, PartialEq, Eq, Encode, Decode, Clone, TypeInfo)]
	pub struct LeavingQuery<C, B> {
		candidate: C,
		delegator: C,
		at_block: B,
	}

	/// Liquid Staking pallet.
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency type.
		/// Shares will use the same Balance type.
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// Account holding Currency of all delegators.
		type StakingAccount: Get<Self::AccountId>;
		/// Account of the reserve.
		type ReserveAccount: Get<Self::AccountId>;

		/// When creating the first Shares for a candidate the supply can be arbitrary.
		/// Picking a value too low will make an higher supply, which means each share will get
		/// less rewards, and rewards calculations will have more impactful rounding errors.
		/// Picking a value too high is a barrier of entry for staking.
		type InitialManualClaimShareValue: Get<BalanceOf<Self>>;
		/// When creating the first Shares for a candidate the supply can arbitrary.
		/// Picking a value too high is a barrier of entry for staking, which will increase overtime
		/// as the value of each share will increase due to auto compounding.
		type InitialAutoCompoundingShareValue: Get<BalanceOf<Self>>;
		/// Minimum amount of stake a Candidate must delegate (stake) towards itself. Not reaching
		/// this minimum prevents from being elected.
		type MinimumSelfDelegation: Get<BalanceOf<Self>>;

		/// When leaving staking the stake is put into leaving pools, and the share of this pool
		/// is stored alongside the current BlockNumber. The user will be able to withdraw the stake
		/// represented by those shares once LeavingDelay has passed.
		/// Shares are used here to allow slashing, as while leaving stake is no longer used for
		/// elections and rewards they must still be at stake in case the candidate misbehave.
		type LeavingDelay: Get<Self::BlockNumber>;

		/// Inflation determines how much is minted as rewards every block.
		type BlockInflation: Get<Perbill>;
	}

	/// Part of the rewards that will be sent to the reserve.
	#[pallet::storage]
	pub type RewardsReserveCommission<T: Config> = StorageValue<_, Perbill, ValueQuery>;

	/// Part of the rewards that will be sent exclusively to the collator.
	#[pallet::storage]
	pub type RewardsCollatorCommission<T: Config> = StorageValue<_, Perbill, ValueQuery>;

	/// Collator set.
	#[pallet::storage]
	pub type CollatorSet<T: Config> = StorageValue<_, BTreeSet<T::AccountId>, ValueQuery>;

	/// Max collator set size.
	#[pallet::storage]
	pub type MaxCollatorSetSize<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Sorted list of eligible candidates.
	#[pallet::storage]
	pub type SortedEligibleCandidates<T: Config> =
		StorageValue<_, Vec<pools::candidates::Candidate<T::AccountId, BalanceOf<T>>>, ValueQuery>;

	/// Stake of each candidate.
	/// Updated by (un)staking either in AutoCompounding or ManualClaim Shares.
	#[pallet::storage]
	pub type CandidatesStake<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	/// Total amount of Currency staked.
	#[pallet::storage]
	pub type CandidatesTotalStaked<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// AutoCompounding Shares.
	#[pallet::storage]
	pub type AutoCompoundingShares<T: Config> = StorageDoubleMap<
		_,
		// Key1: Candidate ID
		Twox64Concat,
		T::AccountId,
		// Key2: Delegator ID
		Twox64Concat,
		T::AccountId,
		// Value: Amount of shares of that Staker towards that Candidate.
		BalanceOf<T>,
		ValueQuery,
	>;

	/// Total amount of AutoCompounding Shares for each Candidate.
	#[pallet::storage]
	pub type AutoCompoundingSharesSupply<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	/// Amount of stake that represents all AutoCompounding Shares of a Candidate.
	#[pallet::storage]
	pub type AutoCompoundingSharesTotalStaked<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	/// ManualClaim Shares.
	#[pallet::storage]
	pub type ManualClaimShares<T: Config> = StorageDoubleMap<
		_,
		// Key1: Candidate ID
		Twox64Concat,
		T::AccountId,
		// Key2: Delegator ID
		Twox64Concat,
		T::AccountId,
		// Value: Amount of shares of that Staker towards that Candidate.
		BalanceOf<T>,
		ValueQuery,
	>;

	/// Total amount of ManualClaim Shares for each Candidate.
	#[pallet::storage]
	pub type ManualClaimSharesSupply<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	/// Amount of stake that represents all ManualClaim Shares of a Candidate.
	#[pallet::storage]
	pub type ManualClaimSharesTotalStaked<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	/// Counter that represents to cumulated rewards per share generated by a Candidate since genesis.
	#[pallet::storage]
	pub type ManualClaimSharesRewardCounter<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	/// Value of the counter the last time the Staker claimed its rewards for a Candidate.
	/// The difference between the checkpoint and the counter is the amount of claimable reward per
	/// share of that Staker.
	#[pallet::storage]
	pub type ManualClaimSharesRewardCheckpoint<T: Config> = StorageDoubleMap<
		_,
		// Key1: Candidate ID
		Twox64Concat,
		T::AccountId,
		// Key2: Delegator ID
		Twox64Concat,
		T::AccountId,
		// Value: Reward checkpoint for that Staker with this Candidate.
		BalanceOf<T>,
		ValueQuery,
	>;

	/// Shares among delegators leaving that Candidate.
	#[pallet::storage]
	pub type LeavingShares<T: Config> = StorageDoubleMap<
		_,
		// Key1: Candidate ID
		Twox64Concat,
		T::AccountId,
		// Key2: Staker ID
		Twox64Concat,
		T::AccountId,
		// Value: Amount of shares among delegators leaving that Candidate.
		BalanceOf<T>,
		ValueQuery,
	>;

	/// Total amount of Leaving Shares for each Candidate.
	#[pallet::storage]
	pub type LeavingSharesSupply<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	/// Amount of stake that represents all Leaving Shares of a Candidate.
	#[pallet::storage]
	pub type LeavingSharesTotalStaked<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	/// Requests for leaving.
	#[pallet::storage]
	pub type LeavingRequests<T: Config> = StorageNMap<
		_,
		(
			// Candidate
			Key<Twox64Concat, T::AccountId>,
			// Delegator
			Key<Twox64Concat, T::AccountId>,
			// Block at which the request was emited
			Key<Twox64Concat, T::BlockNumber>,
		),
		// Number of shares requested for leaving at that block.
		BalanceOf<T>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Stake of that Candidate increased.
		UpdatedCandidatePosition {
			candidate: T::AccountId,
			stake: BalanceOf<T>,
			self_delegation: BalanceOf<T>,
			before: Option<u32>,
			after: Option<u32>,
		},
		/// Stake of that Candidate increased.
		IncreasedStake {
			candidate: T::AccountId,
			stake: BalanceOf<T>,
		},
		/// Stake of that Candidate decreased.
		DecreasedStake {
			candidate: T::AccountId,
			stake: BalanceOf<T>,
		},
		/// Staker staked towards a Candidate for AutoCompounding Shares.
		StakedAutoCompounding {
			candidate: T::AccountId,
			delegator: T::AccountId,
			shares: BalanceOf<T>,
			stake: BalanceOf<T>,
		},
		/// Staker unstaked towards a candidate with AutoCompounding Shares.
		UnstakedAutoCompounding {
			candidate: T::AccountId,
			delegator: T::AccountId,
			shares: BalanceOf<T>,
			stake: BalanceOf<T>,
		},
		/// Staker staked towards a candidate for ManualClaim Shares.
		StakedManualClaim {
			candidate: T::AccountId,
			delegator: T::AccountId,
			shares: BalanceOf<T>,
			stake: BalanceOf<T>,
		},
		/// Staker unstaked towards a candidate with ManualClaim Shares.
		UnstakedManualClaim {
			candidate: T::AccountId,
			delegator: T::AccountId,
			shares: BalanceOf<T>,
			stake: BalanceOf<T>,
		},
		/// Collator has been rewarded.
		RewardedCollator {
			collator: T::AccountId,
			auto_compounding_rewards: BalanceOf<T>,
			manual_claim_rewards: BalanceOf<T>,
		},
		/// Delegators have been rewarded.
		RewardedDelegators {
			collator: T::AccountId,
			auto_compounding_rewards: BalanceOf<T>,
			manual_claim_rewards: BalanceOf<T>,
		},
		/// Rewards manually claimed.
		ClaimedManualRewards {
			candidate: T::AccountId,
			delegator: T::AccountId,
			rewards: BalanceOf<T>,
		},
		/// Registered delayed leaving from staking towards this candidate.
		RegisteredLeaving {
			candidate: T::AccountId,
			delegator: T::AccountId,
			stake: BalanceOf<T>,
			leaving_shares: BalanceOf<T>,
			total_leaving_shares: BalanceOf<T>,
		},
		/// Executed delayed leaving from staking towards this candidate.
		ExecutedLeaving {
			candidate: T::AccountId,
			delegator: T::AccountId,
			stake: BalanceOf<T>,
			leaving_shares: BalanceOf<T>,
			requested_at: T::BlockNumber,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidPalletSetting,
		NoOneIsStaking,
		StakeMustBeNonZero,
		RewardsMustBeNonZero,
		MathUnderflow,
		MathOverflow,
		NotEnoughShares,
		TryingToLeaveTooSoon,
		InconsistentState,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Stake towards candidate the provided amount of stake (Currency) in "Manual Claim"
		/// mode. Add "Manual Claim" shares of this candidate to the origin.
		/// Automatically claims pending rewards.
		#[pallet::weight(0)]
		#[transactional]
		pub fn stake_manual_claim(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			quantity: SharesOrStake<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;

			let shares = match quantity {
				SharesOrStake::Shares(shares) => shares,
				SharesOrStake::Stake(stake) => {
					pools::manual_claim::stake_to_shares_or_init::<T>(&candidate, &stake)?
				}
			};

			ensure!(!Zero::is_zero(&shares), Error::<T>::StakeMustBeNonZero);

			// It is important to automatically claim rewards before updating
			// the amount of shares since pending rewards are stored per share.
			let rewards =
				pools::manual_claim::claim_rewards::<T>(candidate.clone(), delegator.clone())?;
			if !Zero::is_zero(&rewards) {
				T::Currency::transfer(
					&T::StakingAccount::get(),
					&delegator,
					rewards,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			let stake =
				pools::manual_claim::add_shares::<T>(candidate.clone(), delegator.clone(), shares)?;
			pools::candidates::add_stake::<T>(candidate.clone(), stake)?;

			pools::check_candidate_consistency(&candidate)?;

			T::Currency::transfer(
				&delegator,
				&T::StakingAccount::get(),
				stake,
				ExistenceRequirement::KeepAlive,
			)?;

			Ok(().into())
		}

		/// Unstake towards candidate the provided amount of stake (Currency) in "Manual Claim" mode.
		/// Remove "Manual Claim" shares of this candidate from the origin.
		/// Automatically claims pending rewards.
		#[pallet::weight(0)]
		#[transactional]
		pub fn unstake_manual_claim(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			quantity: SharesOrStake<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;

			let shares = match quantity {
				SharesOrStake::Shares(shares) => shares,
				SharesOrStake::Stake(stake) => {
					pools::manual_claim::stake_to_shares::<T>(&candidate, &stake)?
				}
			};

			ensure!(!Zero::is_zero(&shares), Error::<T>::StakeMustBeNonZero);

			// It is important to automatically claim rewards before updating
			// the amount of shares since pending rewards are stored per share.
			let rewards =
				pools::manual_claim::claim_rewards::<T>(candidate.clone(), delegator.clone())?;
			if !Zero::is_zero(&rewards) {
				T::Currency::transfer(
					&T::StakingAccount::get(),
					&delegator,
					rewards,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			let stake =
				pools::manual_claim::sub_shares::<T>(candidate.clone(), delegator.clone(), shares)?;
			// Leaving still count as staked.
			// pools::candidates::sub_stake::<T>(candidate.clone(), stake)?;
			pools::leaving::register_leaving::<T>(candidate, delegator, stake)?;

			pools::check_candidate_consistency(&candidate)?;

			Ok(().into())
		}

		/// Stake towards candidate the provided amount of stake (Currency) in "Auto Compounding"
		/// mode. Add "Auto Compounding" shares of this candidate to the origin.
		#[pallet::weight(0)]
		#[transactional]
		pub fn stake_auto_compounding(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			quantity: SharesOrStake<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;

			let shares = match quantity {
				SharesOrStake::Shares(shares) => shares,
				SharesOrStake::Stake(stake) => {
					pools::auto_compounding::stake_to_shares_or_init::<T>(&candidate, &stake)?
				}
			};

			ensure!(!Zero::is_zero(&shares), Error::<T>::StakeMustBeNonZero);

			let stake = pools::auto_compounding::add_shares::<T>(
				candidate.clone(),
				delegator.clone(),
				shares,
			)?;
			pools::candidates::add_stake::<T>(candidate.clone(), stake)?;

			pools::check_candidate_consistency(&candidate)?;

			T::Currency::transfer(
				&delegator,
				&T::StakingAccount::get(),
				stake,
				ExistenceRequirement::KeepAlive,
			)?;

			Ok(().into())
		}

		/// Untake towards candidate the provided amount of stake (Currency) in "Auto Compounding"
		/// mode. Remove "Auto Compounding" shares of this candidate from the origin.
		#[pallet::weight(0)]
		#[transactional]
		pub fn unstake_auto_compounding(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			quantity: SharesOrStake<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;

			let shares = match quantity {
				SharesOrStake::Shares(shares) => shares,
				SharesOrStake::Stake(stake) => {
					pools::auto_compounding::stake_to_shares::<T>(&candidate, &stake)?
				}
			};

			ensure!(!Zero::is_zero(&shares), Error::<T>::StakeMustBeNonZero);

			let stake = pools::auto_compounding::sub_shares::<T>(
				candidate.clone(),
				delegator.clone(),
				shares,
			)?;
			// Leaving still count as staked.
			// pools::candidates::sub_stake::<T>(candidate.clone(), stake)?;
			pools::leaving::register_leaving::<T>(candidate, delegator, stake)?;

			Ok(().into())
		}

		/// Convert ManualClaim shares to AutoCompounding shares.
		/// Due to rounding while converting back and forth between stake and shares, some "dust"
		/// stake will not be converted, and will be unstaked instead.
		#[pallet::weight(0)]
		#[transactional]
		pub fn convert_manual_claim_to_auto_compounding(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			quantity: SharesOrStake<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;

			let mc_shares = match quantity {
				SharesOrStake::Shares(shares) => shares,
				SharesOrStake::Stake(stake) => {
					pools::manual_claim::stake_to_shares::<T>(&candidate, &stake)?
				}
			};

			ensure!(!Zero::is_zero(&mc_shares), Error::<T>::StakeMustBeNonZero);

			// It is important to automatically claim rewards before updating
			// the amount of shares since pending rewards are stored per share.
			let rewards =
				pools::manual_claim::claim_rewards::<T>(candidate.clone(), delegator.clone())?;
			if !Zero::is_zero(&rewards) {
				T::Currency::transfer(
					&T::StakingAccount::get(),
					&delegator,
					rewards,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			// Shares convertion.
			let mc_stake = pools::manual_claim::sub_shares::<T>(
				candidate.clone(),
				delegator.clone(),
				mc_shares,
			)?;

			let ac_shares =
				pools::auto_compounding::stake_to_shares_or_init::<T>(&candidate, &mc_stake)?;
			let ac_stake = pools::auto_compounding::add_shares::<T>(
				candidate.clone(),
				delegator.clone(),
				ac_shares,
			)?;

			// Deal with dust.
			let diff_stake = mc_stake
				.checked_sub(&ac_stake)
				.ok_or(Error::<T>::MathUnderflow)?;
			// Leaving still count as staked.
			// pools::candidates::sub_stake::<T>(candidate.clone(), diff_stake)?;
			pools::leaving::register_leaving::<T>(candidate, delegator, diff_stake)?;

			pools::check_candidate_consistency(&candidate)?;

			Ok(().into())
		}

		/// Convert AutoCompounding shares to ManualClaim shares.
		/// Due to rounding while converting back and forth between stake and shares, some "dust"
		/// stake will not be converted, and will be unstaked instead.
		#[pallet::weight(0)]
		#[transactional]
		pub fn convert_auto_compounding_to_manual_claim(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			quantity: SharesOrStake<BalanceOf<T>>,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;

			let ac_shares = match quantity {
				SharesOrStake::Shares(shares) => shares,
				SharesOrStake::Stake(stake) => {
					pools::auto_compounding::stake_to_shares::<T>(&candidate, &stake)?
				}
			};

			ensure!(!Zero::is_zero(&ac_shares), Error::<T>::StakeMustBeNonZero);

			// It is important to automatically claim rewards before updating
			// the amount of shares since pending rewards are stored per share.
			let rewards =
				pools::manual_claim::claim_rewards::<T>(candidate.clone(), delegator.clone())?;
			if !Zero::is_zero(&rewards) {
				T::Currency::transfer(
					&T::StakingAccount::get(),
					&delegator,
					rewards,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			// Shares convertion.
			let ac_stake = pools::manual_claim::sub_shares::<T>(
				candidate.clone(),
				delegator.clone(),
				ac_shares,
			)?;

			let mc_shares =
				pools::auto_compounding::stake_to_shares_or_init::<T>(&candidate, &ac_stake)?;
			let mc_stake = pools::auto_compounding::add_shares::<T>(
				candidate.clone(),
				delegator.clone(),
				mc_shares,
			)?;

			// Deal with dust.
			let diff_stake = ac_stake
				.checked_sub(&mc_stake)
				.ok_or(Error::<T>::MathUnderflow)?;
			// Leaving still count as staked.
			// pools::candidates::sub_stake::<T>(candidate.clone(), diff_stake)?;
			pools::leaving::register_leaving::<T>(candidate, delegator, diff_stake)?;

			pools::check_candidate_consistency(&candidate)?;

			Ok(().into())
		}

		/// Claim pending manual rewards for this candidate.
		#[pallet::weight(0)]
		#[transactional]
		pub fn claim_manual_rewards(
			origin: OriginFor<T>,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let delegator = ensure_signed(origin)?;
			let rewards =
				pools::manual_claim::claim_rewards::<T>(candidate.clone(), delegator.clone())?;

			if !Zero::is_zero(&rewards) {
				T::Currency::transfer(
					&T::StakingAccount::get(),
					&delegator,
					rewards,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			Ok(().into())
		}

		/// Claim pending manual rewards for this candidate.
		#[pallet::weight(0)]
		pub fn batch_claim_manual_rewards(
			origin: OriginFor<T>,
			candidates: Vec<T::AccountId>,
		) -> DispatchResultWithPostInfo {
			for candidate in candidates {
				// Each claim is transactional, but it is not important if
				// some claims succeed then another one fails.
				Self::claim_manual_rewards(origin.clone(), candidate)?;
			}

			Ok(().into())
		}

		/// Execute leaving requests if the Leaving delay have elapsed.
		/// Anyone can execute anyones
		#[pallet::weight(0)]
		#[transactional]
		pub fn execute_leaving(
			_origin: OriginFor<T>,
			requests: Vec<LeavingQuery<T::AccountId, T::BlockNumber>>,
		) -> DispatchResultWithPostInfo {
			for request in requests {
				let released = pools::leaving::execute_leaving::<T>(
					request.candidate.clone(),
					request.delegator.clone(),
					request.at_block,
				)?;

				pools::candidates::sub_stake::<T>(request.candidate, released)?;

				T::Currency::transfer(
					&T::StakingAccount::get(),
					&request.delegator,
					released,
					ExistenceRequirement::KeepAlive,
				)?;

				pools::check_candidate_consistency(&candidate)?;
			}

			Ok(().into())
		}
	}

	impl<T: Config> nimbus_primitives::EventHandler<T::AccountId> for Pallet<T> {
		fn note_author(author: T::AccountId) {
			let circulating = T::Currency::total_issuance();
			let rewards = T::BlockInflation::get() * circulating;

			if let Err(err) = rewards::distribute_rewards::<T>(author, rewards) {
				log::error!("Failed to distribute rewards: {:?}", err);
			}
		}
	}

	impl<T: Config> nimbus_primitives::CanAuthor<T::AccountId> for Pallet<T> {
		fn can_author(account: &T::AccountId, _slot: &u32) -> bool {
			CollatorSet::<T>::get().contains(account)
		}
	}

	impl<T: Config> Get<Vec<T::AccountId>> for Pallet<T> {
		fn get() -> Vec<T::AccountId> {
			CollatorSet::<T>::get().iter().cloned().collect()
		}
	}
}
