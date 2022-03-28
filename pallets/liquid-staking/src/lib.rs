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

#![cfg_attr(not(feature = "std"), no_std)]

mod rewards;
mod shares;

#[cfg(test)]
mod mock;

pub use pallet::*;

use frame_support::pallet;

#[pallet]
pub mod pallet {
	use {
		super::{rewards, shares},
		frame_support::{
			pallet_prelude::*,
			storage::types::Key,
			traits::{tokens::ExistenceRequirement, Currency, ReservableCurrency},
			transactional,
		},
		frame_system::pallet_prelude::*,
		sp_runtime::traits::{CheckedAdd, CheckedDiv, Zero},
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
		/// Account holding Currency of all stakers.
		type StakingAccount: Get<Self::AccountId>;
		/// When creating the first Shares for a candidate the supply can be arbitrary.
		/// Picking a value too low will make an higher supply, which means each share will get
		/// less rewards, and rewards calculations will have more impactful rounding errors.
		/// Picking a value too high is a barrier of entry for staking.
		type InitialManualClaimShareValue: Get<BalanceOf<Self>>;
		/// When creating the first Shares for a candidate the supply can arbitrary.
		/// Picking a value too high is a barrier of entry for staking, which will increase overtime
		/// as the value of each share will increase due to auto compounding.
		type InitialAutoCompoundingShareValue: Get<BalanceOf<Self>>;
		/// When leaving staking the stake is put into leaving pools, and the share of this pool
		/// is stored alongside the current BlockNumber. The user will be able to withdraw the stake
		/// represented by those shares once LeavingDelay has passed.
		/// Shares are used here to allow slashing, as while leaving stake is no longer used for
		/// elections and rewards they must still be at stake in case the candidate misbehave.
		type LeavingDelay: Get<Self::BlockNumber>;
	}

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
		// Key2: Staker ID
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

	/// ManualClaim Shares.
	#[pallet::storage]
	pub type ManualClaimShares<T: Config> = StorageDoubleMap<
		_,
		// Key1: Candidate ID
		Twox64Concat,
		T::AccountId,
		// Key2: Staker ID
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
		// Key2: Staker ID
		Twox64Concat,
		T::AccountId,
		// Value: Reward checkpoint for that Staker with this Candidate.
		BalanceOf<T>,
		ValueQuery,
	>;

	/// Shares among stakers leaving that Candidate.
	#[pallet::storage]
	pub type LeavingShares<T: Config> = StorageDoubleMap<
		_,
		// Key1: Candidate ID
		Twox64Concat,
		T::AccountId,
		// Key2: Staker ID
		Twox64Concat,
		T::AccountId,
		// Value: Amount of shares among stakers leaving that Candidate.
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
			// Staker
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
			staker: T::AccountId,
			shares: BalanceOf<T>,
			stake: BalanceOf<T>,
		},
		/// Staker unstaked towards a candidate with AutoCompounding Shares.
		UnstakedAutoCompounding {
			candidate: T::AccountId,
			staker: T::AccountId,
			shares: BalanceOf<T>,
			stake: BalanceOf<T>,
		},
		/// Staker staked towards a candidate for ManualClaim Shares.
		StakedManualClaim {
			candidate: T::AccountId,
			staker: T::AccountId,
			shares: BalanceOf<T>,
			stake: BalanceOf<T>,
		},
		/// Staker unstaked towards a candidate with ManualClaim Shares.
		UnstakedManualClaim {
			candidate: T::AccountId,
			staker: T::AccountId,
			shares: BalanceOf<T>,
			stake: BalanceOf<T>,
		},
		/// Candidate has been rewarded.
		RewardedCandidate {
			candidate: T::AccountId,
			auto_compounding_rewards: BalanceOf<T>,
			manual_claim_rewards: BalanceOf<T>,
		},
		/// Stakers have been rewarded.
		RewardedStakers {
			candidate: T::AccountId,
			auto_compounding_rewards: BalanceOf<T>,
			manual_claim_rewards: BalanceOf<T>,
		},
		/// Rewards manually claimed.
		ClaimedManualRewards {
			candidate: T::AccountId,
			staker: T::AccountId,
			rewards: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidPalletSetting,
		NoOneIsStaking,
		StakeMustBeNonZero,
		MathUnderflow,
		MathOverflow,
		NotEnoughShares,
		TryingToLeaveTooSoon,
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
			let staker = ensure_signed(origin)?;

			let shares = match quantity {
				SharesOrStake::Shares(shares) => shares,
				SharesOrStake::Stake(stake) => {
					let shares_supply = ManualClaimSharesSupply::<T>::get(&candidate);

					if Zero::is_zero(&shares_supply) {
						stake
							.checked_div(&T::InitialManualClaimShareValue::get())
							.ok_or(Error::<T>::InvalidPalletSetting)?
					} else {
						shares::manual_claim::stake_to_shares::<T>(&candidate, &stake)?
					}
				}
			};

			ensure!(!Zero::is_zero(&shares), Error::<T>::StakeMustBeNonZero);

			// It is important to automatically claim rewards before updating
			// the amount of shares since pending rewards are stored per share.
			let rewards = rewards::claim_rewards::<T>(candidate.clone(), staker.clone())?;
			if !Zero::is_zero(&rewards) {
				T::Currency::transfer(
					&T::StakingAccount::get(),
					&staker,
					rewards,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			let stake =
				shares::manual_claim::add_shares::<T>(candidate.clone(), staker.clone(), shares)?;
			shares::candidates::add_stake::<T>(candidate.clone(), stake)?;

			T::Currency::transfer(
				&staker,
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
			let staker = ensure_signed(origin)?;

			let shares = match quantity {
				SharesOrStake::Shares(shares) => shares,
				SharesOrStake::Stake(stake) => {
					shares::manual_claim::stake_to_shares::<T>(&candidate, &stake)?
				}
			};

			ensure!(!Zero::is_zero(&shares), Error::<T>::StakeMustBeNonZero);

			// It is important to automatically claim rewards before updating
			// the amount of shares since pending rewards are stored per share.
			let rewards = rewards::claim_rewards::<T>(candidate.clone(), staker.clone())?;
			if !Zero::is_zero(&rewards) {
				T::Currency::transfer(
					&T::StakingAccount::get(),
					&staker,
					rewards,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			let stake =
				shares::manual_claim::sub_shares::<T>(candidate.clone(), staker.clone(), shares)?;
			shares::candidates::sub_stake::<T>(candidate.clone(), stake)?;
			shares::leaving::register_leaving::<T>(candidate, staker, stake)?;

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
			let staker = ensure_signed(origin)?;

			let shares = match quantity {
				SharesOrStake::Shares(shares) => shares,
				SharesOrStake::Stake(stake) => {
					let shares_supply = AutoCompoundingSharesSupply::<T>::get(&candidate);

					if Zero::is_zero(&shares_supply) {
						stake
							.checked_div(&T::InitialAutoCompoundingShareValue::get())
							.ok_or(Error::<T>::InvalidPalletSetting)?
					} else {
						shares::auto_compounding::stake_to_shares::<T>(&candidate, &stake)?
					}
				}
			};

			ensure!(!Zero::is_zero(&shares), Error::<T>::StakeMustBeNonZero);

			let stake = shares::auto_compounding::add_shares::<T>(
				candidate.clone(),
				staker.clone(),
				shares,
			)?;
			shares::candidates::add_stake::<T>(candidate.clone(), stake)?;

			T::Currency::transfer(
				&staker,
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
			let staker = ensure_signed(origin)?;

			let shares = match quantity {
				SharesOrStake::Shares(shares) => shares,
				SharesOrStake::Stake(stake) => {
					shares::auto_compounding::stake_to_shares::<T>(&candidate, &stake)?
				}
			};

			ensure!(!Zero::is_zero(&shares), Error::<T>::StakeMustBeNonZero);

			let stake = shares::auto_compounding::sub_shares::<T>(
				candidate.clone(),
				staker.clone(),
				shares,
			)?;
			shares::candidates::sub_stake::<T>(candidate.clone(), stake)?;
			shares::leaving::register_leaving::<T>(candidate, staker, stake)?;

			Ok(().into())
		}

		/// Claim pending manual rewards for this candidate.
		#[pallet::weight(0)]
		#[transactional]
		pub fn claim_manual_rewards(
			origin: OriginFor<T>,
			candidate: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let staker = ensure_signed(origin)?;
			let rewards = rewards::claim_rewards::<T>(candidate.clone(), staker.clone())?;

			if !Zero::is_zero(&rewards) {
				T::Currency::transfer(
					&T::StakingAccount::get(),
					&staker,
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
		#[pallet::weight(0)]
		#[transactional]
		pub fn execute_leaving(
			origin: OriginFor<T>,
			requests: Vec<LeavingQuery<T::AccountId, T::BlockNumber>>,
		) -> DispatchResultWithPostInfo {
			let staker = ensure_signed(origin)?;
			let mut stake_sum: BalanceOf<T> = Zero::zero();

			for request in requests {
				let released = shares::leaving::execute_leaving::<T>(
					request.candidate,
					staker.clone(),
					request.at_block,
				)?;

				stake_sum = stake_sum
					.checked_add(&released)
					.ok_or(Error::<T>::MathOverflow)?;
			}

			T::Currency::transfer(
				&T::StakingAccount::get(),
				&staker,
				stake_sum,
				ExistenceRequirement::KeepAlive,
			)?;

			Ok(().into())
		}
	}
}
