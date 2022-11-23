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

pub(crate) mod calls;
pub mod mul_div;
pub mod pools;
pub mod rewards;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

use frame_support::pallet;

#[pallet]
pub mod pallet {
	use {
		crate::{calls::Calls, mul_div, pools, rewards},
		frame_support::{
			pallet_prelude::*,
			storage::types::Key,
			traits::{tokens::Balance, Currency, ReservableCurrency},
			transactional,
		},
		frame_system::pallet_prelude::*,
		sp_runtime::Perbill,
		sp_std::{collections::btree_set::BTreeSet, convert::TryInto},
	};

	#[cfg(feature = "std")]
	use serde::{Deserialize, Serialize};

	// Type aliases for better readability.
	pub type Generation = u32;
	pub type Candidate<T> = <T as frame_system::Config>::AccountId;
	pub type CandidateGen<T> = GenId<Candidate<T>>;
	pub type Delegator<T> = <T as frame_system::Config>::AccountId;

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

	/// A generational ID.
	/// A slashed candidate get its generation increased, and previous generations
	/// are dead tokens that can only be used to retreive staked currency.
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(RuntimeDebug, PartialEq, Eq, Encode, Decode, Clone, TypeInfo)]
	pub struct GenId<T: Sized> {
		pub id: T,
		pub generation: Generation,
	}

	/// Identifier used when executing a pending leaving request.
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(RuntimeDebug, PartialEq, Eq, Encode, Decode, Clone, TypeInfo)]
	pub struct ExecuteLeavingQuery<C, D, B> {
		pub candidate: C,
		pub delegator: D,
		pub at_block: B,
	}

	/// Identifier used when canceling a pending leaving request.
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(RuntimeDebug, PartialEq, Eq, Encode, Decode, Clone, TypeInfo)]
	pub struct CancelLeavingQuery<C, B> {
		pub candidate: C,
		pub at_block: B,
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
		type Currency: Currency<Self::AccountId, Balance = Self::Balance>
			+ ReservableCurrency<Self::AccountId, Balance = Self::Balance>;

		/// Same as Currency::Balance. Must impl `MulDiv` which perform
		/// multiplication followed by division using a bigger type to avoid
		/// overflows.
		type Balance: Balance + mul_div::MulDiv;

		/// Account holding Currency of all delegators.
		type StakingAccount: Get<Self::AccountId>;
		/// Account of the reserve.
		type ReserveAccount: Get<Self::AccountId>;

		/// When creating the first Shares for a candidate the supply can be arbitrary.
		/// Picking a value too low will make an higher supply, which means each share will get
		/// less rewards, and rewards calculations will have more impactful rounding errors.
		/// Picking a value too high is a barrier of entry for staking.
		type InitialManualClaimShareValue: Get<Self::Balance>;
		/// When creating the first Shares for a candidate the supply can arbitrary.
		/// Picking a value too high is a barrier of entry for staking, which will increase overtime
		/// as the value of each share will increase due to auto compounding.
		type InitialAutoCompoundingShareValue: Get<Self::Balance>;
		/// Minimum amount of stake a Candidate must delegate (stake) towards itself. Not reaching
		/// this minimum prevents from being elected.
		type MinimumSelfDelegation: Get<Self::Balance>;

		/// When leaving staking the stake is put into leaving pools, and the share of this pool
		/// is stored alongside the current BlockNumber. The user will be able to withdraw the stake
		/// represented by those shares once LeavingDelay has passed.
		/// Shares are used here to allow slashing, as while leaving stake is no longer used for
		/// elections and rewards they must still be at stake in case the candidate misbehave.
		type LeavingDelay: Get<Self::BlockNumber>;

		/// Inflation determines how much is minted as rewards every block.
		type BlockInflation: Get<Perbill>;

		/// Part of the rewards that will be sent to the reserve.
		type RewardsReserveCommission: Get<Perbill>;

		/// Part of the rewards that will be sent exclusively to the collator.
		type RewardsCollatorCommission: Get<Perbill>;

		/// Mapping to distribute collator rewards to an account.
		/// Made to support Moonbeam orbiters program.
		/// `()` gives rewards to the collator's account.
		type CollatorRewardsMapping: rewards::CollatorRewardsMapping<Self>;
	}

	// /// Part of the rewards that will be sent to the reserve.
	// #[pallet::storage]
	// pub type RewardsReserveCommission<T: Config> = StorageValue<_, Perbill, ValueQuery>;

	// /// Part of the rewards that will be sent exclusively to the collator.
	// #[pallet::storage]
	// pub type RewardsCollatorCommission<T: Config> = StorageValue<_, Perbill, ValueQuery>;

	/// Collator set.
	#[pallet::storage]
	pub type CollatorSet<T: Config> = StorageValue<_, BTreeSet<Candidate<T>>, ValueQuery>;

	/// Max collator set size.
	#[pallet::storage]
	pub type MaxCollatorSetSize<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Sorted list of eligible candidates.
	#[pallet::storage]
	pub type SortedEligibleCandidates<T: Config> =
		StorageValue<_, Vec<pools::candidates::Candidate<Candidate<T>, T::Balance>>, ValueQuery>;

	/// Generation of each Candidate.
	/// Being slashed increase the generation by 1, effectively kicking the candidate until
	/// delegators stake towards the new generation.
	#[pallet::storage]
	pub type CandidatesGeneration<T: Config> =
		StorageMap<_, Twox64Concat, Candidate<T>, Generation, ValueQuery>;

	/// Total stake of each candidate, regarless of which share type is held.
	#[pallet::storage]
	pub type CandidatesStake<T: Config> =
		StorageMap<_, Twox64Concat, CandidateGen<T>, T::Balance, ValueQuery>;

	/// Total amount of Currency staked.
	#[pallet::storage]
	pub type CandidatesTotalStaked<T: Config> = StorageValue<_, T::Balance, ValueQuery>;

	/// AutoCompounding Shares.
	#[pallet::storage]
	pub type AutoCompoundingShares<T: Config> = StorageDoubleMap<
		_,
		// Key1: Candidate ID
		Twox64Concat,
		CandidateGen<T>,
		// Key2: Delegator ID
		Twox64Concat,
		Delegator<T>,
		// Value: Amount of shares of that Delegator towards that Candidate.
		T::Balance,
		ValueQuery,
	>;

	/// Total amount of AutoCompounding Shares for each Candidate.
	#[pallet::storage]
	pub type AutoCompoundingSharesSupply<T: Config> =
		StorageMap<_, Twox64Concat, CandidateGen<T>, T::Balance, ValueQuery>;

	/// Amount of stake that represents all AutoCompounding Shares of a Candidate.
	#[pallet::storage]
	pub type AutoCompoundingSharesTotalStaked<T: Config> =
		StorageMap<_, Twox64Concat, CandidateGen<T>, T::Balance, ValueQuery>;

	/// ManualClaim Shares.
	#[pallet::storage]
	pub type ManualClaimShares<T: Config> = StorageDoubleMap<
		_,
		// Key1: Candidate ID
		Twox64Concat,
		CandidateGen<T>,
		// Key2: Delegator ID
		Twox64Concat,
		Delegator<T>,
		// Value: Amount of shares of that Delegator towards that Candidate.
		T::Balance,
		ValueQuery,
	>;

	/// Total amount of ManualClaim Shares for each Candidate.
	#[pallet::storage]
	pub type ManualClaimSharesSupply<T: Config> =
		StorageMap<_, Twox64Concat, CandidateGen<T>, T::Balance, ValueQuery>;

	/// Amount of stake that represents all ManualClaim Shares of a Candidate.
	#[pallet::storage]
	pub type ManualClaimSharesTotalStaked<T: Config> =
		StorageMap<_, Twox64Concat, CandidateGen<T>, T::Balance, ValueQuery>;

	/// Counter that represents to cumulated rewards per share generated by a Candidate since genesis.
	#[pallet::storage]
	pub type ManualClaimSharesRewardCounter<T: Config> =
		StorageMap<_, Twox64Concat, CandidateGen<T>, T::Balance, ValueQuery>;

	/// Value of the counter the last time the Delegator claimed its rewards for a Candidate.
	/// The difference between the checkpoint and the counter is the amount of claimable reward per
	/// share of that Delegator.
	#[pallet::storage]
	pub type ManualClaimSharesRewardCheckpoint<T: Config> = StorageDoubleMap<
		_,
		// Key1: Candidate ID
		Twox64Concat,
		CandidateGen<T>,
		// Key2: Delegator ID
		Twox64Concat,
		Delegator<T>,
		// Value: Reward checkpoint for that Delegator with this Candidate.
		T::Balance,
		ValueQuery,
	>;

	/// Shares among delegators leaving that Candidate.
	#[pallet::storage]
	pub type LeavingShares<T: Config> = StorageDoubleMap<
		_,
		// Key1: Candidate ID
		Twox64Concat,
		CandidateGen<T>,
		// Key2: Delegator ID
		Twox64Concat,
		Delegator<T>,
		// Value: Amount of shares among delegators leaving that Candidate.
		T::Balance,
		ValueQuery,
	>;

	/// Total amount of Leaving Shares for each Candidate.
	#[pallet::storage]
	pub type LeavingSharesSupply<T: Config> =
		StorageMap<_, Twox64Concat, CandidateGen<T>, T::Balance, ValueQuery>;

	/// Amount of stake that represents all Leaving Shares of a Candidate.
	#[pallet::storage]
	pub type LeavingSharesTotalStaked<T: Config> =
		StorageMap<_, Twox64Concat, CandidateGen<T>, T::Balance, ValueQuery>;

	/// Requests for leaving.
	#[pallet::storage]
	pub type LeavingRequests<T: Config> = StorageNMap<
		_,
		(
			// Candidate
			Key<Twox64Concat, CandidateGen<T>>,
			// Delegator
			Key<Twox64Concat, Delegator<T>>,
			// Block at which the request was emited
			Key<Twox64Concat, T::BlockNumber>,
		),
		// Number of shares requested for leaving at that block.
		T::Balance,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Stake of that Candidate increased.
		UpdatedCandidatePosition {
			candidate: CandidateGen<T>,
			stake: T::Balance,
			self_delegation: T::Balance,
			before: Option<u32>,
			after: Option<u32>,
		},
		/// Stake of that Candidate increased.
		IncreasedStake {
			candidate: CandidateGen<T>,
			stake: T::Balance,
		},
		/// Stake of that Candidate decreased.
		DecreasedStake {
			candidate: CandidateGen<T>,
			stake: T::Balance,
		},
		/// Delegator staked towards a Candidate for AutoCompounding Shares.
		StakedAutoCompounding {
			candidate: CandidateGen<T>,
			delegator: Delegator<T>,
			shares: T::Balance,
			stake: T::Balance,
		},
		/// Delegator unstaked towards a candidate with AutoCompounding Shares.
		UnstakedAutoCompounding {
			candidate: CandidateGen<T>,
			delegator: Delegator<T>,
			shares: T::Balance,
			stake: T::Balance,
		},
		/// Delegator staked towards a candidate for ManualClaim Shares.
		StakedManualClaim {
			candidate: CandidateGen<T>,
			delegator: Delegator<T>,
			shares: T::Balance,
			stake: T::Balance,
		},
		/// Delegator unstaked towards a candidate with ManualClaim Shares.
		UnstakedManualClaim {
			candidate: CandidateGen<T>,
			delegator: Delegator<T>,
			shares: T::Balance,
			stake: T::Balance,
		},
		/// Collator has been rewarded.
		RewardedCollator {
			collator: CandidateGen<T>,
			auto_compounding_rewards: T::Balance,
			manual_claim_rewards: T::Balance,
		},
		/// Delegators have been rewarded.
		RewardedDelegators {
			collator: CandidateGen<T>,
			auto_compounding_rewards: T::Balance,
			manual_claim_rewards: T::Balance,
		},
		/// Rewards manually claimed.
		ClaimedManualRewards {
			candidate: CandidateGen<T>,
			delegator: Delegator<T>,
			rewards: T::Balance,
		},
		/// Registered delayed leaving from staking towards this candidate.
		RegisteredLeaving {
			candidate: CandidateGen<T>,
			delegator: Delegator<T>,
			stake: T::Balance,
			leaving_shares: T::Balance,
			total_leaving_shares: T::Balance,
		},
		/// Executed delayed leaving from staking towards this candidate.
		ExecutedLeaving {
			candidate: CandidateGen<T>,
			delegator: Delegator<T>,
			stake: T::Balance,
			leaving_shares: T::Balance,
			requested_at: T::BlockNumber,
		},
		/// Canceled delayed leaving from staking towards this candidate.
		CanceledLeaving {
			candidate: CandidateGen<T>,
			delegator: Delegator<T>,
			stake: T::Balance,
			leaving_shares: T::Balance,
			requested_at: T::BlockNumber,
		},
		/// Transfered AutoCompounding shares to another account.
		TransferedAutoCompounding {
			candidate: CandidateGen<T>,
			sender: Delegator<T>,
			recipient: Delegator<T>,
			shares: T::Balance,
		},
		/// Transfered ManualClaim shares to another account.
		TransferedManualClaim {
			candidate: CandidateGen<T>,
			sender: Delegator<T>,
			recipient: Delegator<T>,
			shares: T::Balance,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidPalletSetting,
		DisabledFeature,
		NoOneIsStaking,
		StakeMustBeNonZero,
		RewardsMustBeNonZero,
		MathUnderflow,
		MathOverflow,
		NotEnoughShares,
		TryingToLeaveTooSoon,
		InconsistentState,
		UnsufficientSharesForTransfer,
		CandidateTransferingOwnSharesForbidden,
		WrongCandidateGeneration,
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
			candidate: CandidateGen<T>,
			quantity: SharesOrStake<T::Balance>,
		) -> DispatchResultWithPostInfo {
			Calls::<T>::stake_manual_claim(origin, candidate, quantity)
		}

		/// Unstake towards candidate the provided amount of stake (Currency) in "Manual Claim" mode.
		/// Remove "Manual Claim" shares of this candidate from the origin.
		/// Automatically claims pending rewards.
		#[pallet::weight(0)]
		#[transactional]
		pub fn unstake_manual_claim(
			origin: OriginFor<T>,
			candidate: CandidateGen<T>,
			quantity: SharesOrStake<T::Balance>,
		) -> DispatchResultWithPostInfo {
			Calls::<T>::unstake_manual_claim(origin, candidate, quantity)
		}

		/// Stake towards candidate the provided amount of stake (Currency) in "Auto Compounding"
		/// mode. Add "Auto Compounding" shares of this candidate to the origin.
		#[pallet::weight(0)]
		#[transactional]
		pub fn stake_auto_compounding(
			origin: OriginFor<T>,
			candidate: CandidateGen<T>,
			quantity: SharesOrStake<T::Balance>,
		) -> DispatchResultWithPostInfo {
			Calls::<T>::stake_auto_compounding(origin, candidate, quantity)
		}

		/// Untake towards candidate the provided amount of stake (Currency) in "Auto Compounding"
		/// mode. Remove "Auto Compounding" shares of this candidate from the origin.
		#[pallet::weight(0)]
		#[transactional]
		pub fn unstake_auto_compounding(
			origin: OriginFor<T>,
			candidate: CandidateGen<T>,
			quantity: SharesOrStake<T::Balance>,
		) -> DispatchResultWithPostInfo {
			Calls::<T>::unstake_auto_compounding(origin, candidate, quantity)
		}

		/// Convert ManualClaim shares to AutoCompounding shares.
		/// Due to rounding while converting back and forth between stake and shares, some "dust"
		/// stake will not be converted, and will be distributed among all AutoCompound share holders.
		#[pallet::weight(0)]
		#[transactional]
		pub fn convert_manual_claim_to_auto_compounding(
			origin: OriginFor<T>,
			candidate: CandidateGen<T>,
			quantity: SharesOrStake<T::Balance>,
		) -> DispatchResultWithPostInfo {
			Calls::<T>::convert_manual_claim_to_auto_compounding(origin, candidate, quantity)
		}

		/// Convert AutoCompounding shares to ManualClaim shares.
		/// Due to rounding while converting back and forth between stake and shares, some "dust"
		/// stake will not be converted, and will be distributed among all AutoCompound share holders.
		#[pallet::weight(0)]
		#[transactional]
		pub fn convert_auto_compounding_to_manual_claim(
			origin: OriginFor<T>,
			candidate: CandidateGen<T>,
			quantity: SharesOrStake<T::Balance>,
		) -> DispatchResultWithPostInfo {
			Calls::<T>::convert_auto_compounding_to_manual_claim(origin, candidate, quantity)
		}

		/// Claim pending manual rewards for this candidate.
		#[pallet::weight(0)]
		#[transactional]
		pub fn claim_manual_rewards(
			origin: OriginFor<T>,
			candidate: CandidateGen<T>,
		) -> DispatchResultWithPostInfo {
			Calls::<T>::claim_manual_rewards(origin, candidate)
		}

		/// Claim pending manual rewards for this candidate.
		#[pallet::weight(0)]
		pub fn batch_claim_manual_rewards(
			origin: OriginFor<T>,
			candidates: Vec<CandidateGen<T>>,
		) -> DispatchResultWithPostInfo {
			Calls::<T>::batch_claim_manual_rewards(origin, candidates)
		}

		/// Execute leaving requests if the Leaving delay have elapsed.
		/// Anyone can execute anyones
		#[pallet::weight(0)]
		#[transactional]
		pub fn execute_leaving(
			origin: OriginFor<T>,
			requests: Vec<ExecuteLeavingQuery<CandidateGen<T>, Delegator<T>, T::BlockNumber>>,
		) -> DispatchResultWithPostInfo {
			Calls::<T>::execute_leaving(origin, requests)
		}

		/// Cancel leaving requests that have not been executed yet.
		/// `put_in_auto_compound` specifies if the funds are put back in AutoCompound or
		/// ManualClaim pools. Dust is shared among AutoCompound share holders.
		#[pallet::weight(0)]
		#[transactional]
		pub fn cancel_leaving(
			origin: OriginFor<T>,
			requests: Vec<CancelLeavingQuery<CandidateGen<T>, T::BlockNumber>>,
			put_in_auto_compound: bool,
		) -> DispatchResultWithPostInfo {
			Calls::<T>::cancel_leaving(origin, requests, put_in_auto_compound)
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn transfer_auto_compounding(
			origin: OriginFor<T>,
			candidate: CandidateGen<T>,
			recipient: Delegator<T>,
			shares: T::Balance,
		) -> DispatchResultWithPostInfo {
			Calls::<T>::transfer_auto_compounding(origin, candidate, recipient, shares)
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn transfer_manual_claim(
			origin: OriginFor<T>,
			candidate: CandidateGen<T>,
			recipient: Delegator<T>,
			shares: T::Balance,
		) -> DispatchResultWithPostInfo {
			Calls::<T>::transfer_manual_claim(origin, candidate, recipient, shares)
		}
	}

	impl<T: Config> nimbus_primitives::EventHandler<T::AccountId> for Pallet<T> {
		fn note_author(author: T::AccountId) {
			let circulating = T::Currency::total_issuance();
			let rewards = T::BlockInflation::get() * circulating;

			let generation = CandidatesGeneration::<T>::get(&author);
			let candidate = CandidateGen::<T> {
				id: author,
				generation,
			};

			if let Err(err) = rewards::distribute_rewards::<T>(candidate, rewards) {
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

	pub trait CandidateExt {
		fn with_gen(self, generation: Generation) -> GenId<Self>
		where
			Self: Sized,
		{
			GenId {
				id: self,
				generation,
			}
		}
	}

	impl<T> CandidateExt for T {}
}
