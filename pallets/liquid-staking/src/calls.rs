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

use {
	crate::{pallet::*, pools},
	frame_support::{
		pallet_prelude::*,
		traits::{tokens::ExistenceRequirement, Currency},
	},
	frame_system::pallet_prelude::*,
	sp_runtime::traits::{CheckedAdd, CheckedSub, Zero},
};

pub struct Calls<T>(PhantomData<T>);

impl<T: Config> Calls<T> {
	/// Stake towards candidate the provided amount of stake (Currency) in "Manual Claim"
	/// mode. Add "Manual Claim" shares of this candidate to the origin.
	/// Automatically claims pending rewards.
	pub fn stake_manual_claim(
		origin: OriginFor<T>,
		candidate: T::AccountId,
		quantity: SharesOrStake<T::Balance>,
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

		pools::check_candidate_consistency::<T>(&candidate)?;

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
	pub fn unstake_manual_claim(
		origin: OriginFor<T>,
		candidate: T::AccountId,
		quantity: SharesOrStake<T::Balance>,
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
		pools::leaving::register_leaving::<T>(candidate.clone(), delegator, stake)?;

		pools::check_candidate_consistency::<T>(&candidate)?;

		Ok(().into())
	}

	/// Stake towards candidate the provided amount of stake (Currency) in "Auto Compounding"
	/// mode. Add "Auto Compounding" shares of this candidate to the origin.
	pub fn stake_auto_compounding(
		origin: OriginFor<T>,
		candidate: T::AccountId,
		quantity: SharesOrStake<T::Balance>,
	) -> DispatchResultWithPostInfo {
		let delegator = ensure_signed(origin)?;

		let shares = match quantity {
			SharesOrStake::Shares(shares) => shares,
			SharesOrStake::Stake(stake) => {
				pools::auto_compounding::stake_to_shares_or_init::<T>(&candidate, stake)?
			}
		};

		ensure!(!Zero::is_zero(&shares), Error::<T>::StakeMustBeNonZero);

		let stake =
			pools::auto_compounding::add_shares::<T>(candidate.clone(), delegator.clone(), shares)?;
		pools::candidates::add_stake::<T>(candidate.clone(), stake)?;

		pools::check_candidate_consistency::<T>(&candidate)?;

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
	pub fn unstake_auto_compounding(
		origin: OriginFor<T>,
		candidate: T::AccountId,
		quantity: SharesOrStake<T::Balance>,
	) -> DispatchResultWithPostInfo {
		let delegator = ensure_signed(origin)?;

		let shares = match quantity {
			SharesOrStake::Shares(shares) => shares,
			SharesOrStake::Stake(stake) => {
				pools::auto_compounding::stake_to_shares::<T>(&candidate, stake)?
			}
		};

		ensure!(!Zero::is_zero(&shares), Error::<T>::StakeMustBeNonZero);

		let stake =
			pools::auto_compounding::sub_shares::<T>(candidate.clone(), delegator.clone(), shares)?;
		// Leaving still count as staked.
		// pools::candidates::sub_stake::<T>(candidate.clone(), stake)?;
		pools::leaving::register_leaving::<T>(candidate, delegator, stake)?;

		Ok(().into())
	}

	/// Convert ManualClaim shares to AutoCompounding shares.
	/// Due to rounding while converting back and forth between stake and shares, some "dust"
	/// stake will not be converted, and will be distributed among all AutoCompound share holders.
	pub fn convert_manual_claim_to_auto_compounding(
		origin: OriginFor<T>,
		candidate: T::AccountId,
		quantity: SharesOrStake<T::Balance>,
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
		let mc_stake =
			pools::manual_claim::sub_shares::<T>(candidate.clone(), delegator.clone(), mc_shares)?;

		let ac_shares =
			pools::auto_compounding::stake_to_shares_or_init::<T>(&candidate, mc_stake)?;
		let ac_stake = pools::auto_compounding::add_shares::<T>(
			candidate.clone(),
			delegator.clone(),
			ac_shares,
		)?;

		// Deal with dust, which is shared among all AutoCompound share holders.
		let diff_stake = mc_stake
			.checked_sub(&ac_stake)
			.ok_or(Error::<T>::MathUnderflow)?;
		pools::auto_compounding::share_stake_among_holders::<T>(&candidate, diff_stake)?;

		pools::check_candidate_consistency::<T>(&candidate)?;

		Ok(().into())
	}

	/// Convert AutoCompounding shares to ManualClaim shares.
	/// Due to rounding while converting back and forth between stake and shares, some "dust"
	/// stake will not be converted, and will be distributed among all AutoCompound share holders.
	pub fn convert_auto_compounding_to_manual_claim(
		origin: OriginFor<T>,
		candidate: T::AccountId,
		quantity: SharesOrStake<T::Balance>,
	) -> DispatchResultWithPostInfo {
		let delegator = ensure_signed(origin)?;

		let ac_shares = match quantity {
			SharesOrStake::Shares(shares) => shares,
			SharesOrStake::Stake(stake) => {
				pools::auto_compounding::stake_to_shares::<T>(&candidate, stake)?
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
		let ac_stake =
			pools::manual_claim::sub_shares::<T>(candidate.clone(), delegator.clone(), ac_shares)?;

		let mc_shares =
			pools::auto_compounding::stake_to_shares_or_init::<T>(&candidate, ac_stake)?;
		let mc_stake = pools::auto_compounding::add_shares::<T>(
			candidate.clone(),
			delegator.clone(),
			mc_shares,
		)?;

		// Deal with dust, which is shared among all AutoCompound share holders.
		let diff_stake = mc_stake
			.checked_sub(&ac_stake)
			.ok_or(Error::<T>::MathUnderflow)?;

		pools::auto_compounding::share_stake_among_holders::<T>(&candidate, diff_stake)?;

		pools::check_candidate_consistency::<T>(&candidate)?;

		Ok(().into())
	}

	/// Claim pending manual rewards for this candidate.
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
	pub fn execute_leaving(
		_origin: OriginFor<T>,
		requests: Vec<ExecuteLeavingQuery<T::AccountId, T::BlockNumber>>,
	) -> DispatchResultWithPostInfo {
		for request in requests {
			let released = pools::leaving::execute_leaving::<T>(
				request.candidate.clone(),
				request.delegator.clone(),
				request.at_block,
			)?;

			pools::candidates::sub_stake::<T>(request.candidate.clone(), released)?;

			T::Currency::transfer(
				&T::StakingAccount::get(),
				&request.delegator,
				released,
				ExistenceRequirement::KeepAlive,
			)?;

			pools::check_candidate_consistency::<T>(&request.candidate)?;
		}

		Ok(().into())
	}

	/// Cancel leaving requests that have not been executed yet.
	/// `put_in_auto_compound` specifies if the funds are put back in AutoCompound or
	/// ManualClaim pools. Dust is shared among AutoCompound share holders.
	pub fn cancel_leaving(
		origin: OriginFor<T>,
		requests: Vec<CancelLeavingQuery<T::AccountId, T::BlockNumber>>,
		put_in_auto_compound: bool,
	) -> DispatchResultWithPostInfo {
		let delegator = ensure_signed(origin)?;

		for request in requests {
			let canceled_stake = pools::leaving::cancel_leaving::<T>(
				request.candidate.clone(),
				delegator.clone(),
				request.at_block,
			)?;

			if canceled_stake.is_zero() {
				continue;
			}

			let inserted_stake = if put_in_auto_compound {
				let shares = pools::auto_compounding::stake_to_shares_or_init::<T>(
					&request.candidate,
					canceled_stake,
				)?;

				if !Zero::is_zero(&shares) {
					pools::auto_compounding::add_shares::<T>(
						request.candidate.clone(),
						delegator.clone(),
						shares,
					)?
				} else {
					Zero::zero()
				}
			} else {
				let shares = pools::manual_claim::stake_to_shares_or_init::<T>(
					&request.candidate,
					&canceled_stake,
				)?;

				if !Zero::is_zero(&shares) {
					pools::manual_claim::add_shares::<T>(
						request.candidate.clone(),
						delegator.clone(),
						shares,
					)?
				} else {
					Zero::zero()
				}
			};

			let dust_stake = canceled_stake
				.checked_sub(&inserted_stake)
				.ok_or(Error::<T>::MathUnderflow)?;

			pools::auto_compounding::share_stake_among_holders::<T>(
				&request.candidate,
				dust_stake,
			)?;

			pools::check_candidate_consistency::<T>(&request.candidate)?;
		}

		Ok(().into())
	}

	pub fn transfer_auto_compounding(
		origin: OriginFor<T>,
		candidate: T::AccountId,
		recipient: T::AccountId,
		shares: T::Balance,
	) -> DispatchResultWithPostInfo {
		ensure!(
			cfg!(feature = "transferable-shares"),
			Error::<T>::DisabledFeature
		);

		let sender = ensure_signed(origin)?;
		ensure!(!Zero::is_zero(&shares), Error::<T>::StakeMustBeNonZero);

		ensure!(
			sender != candidate,
			Error::<T>::CandidateTransferingOwnSharesForbidden
		);

		let sender_new_shares = AutoCompoundingShares::<T>::get(&candidate, &sender)
			.checked_sub(&shares)
			.ok_or(Error::<T>::UnsufficientSharesForTransfer)?;

		let recipient_new_shares = AutoCompoundingShares::<T>::get(&candidate, &recipient)
			.checked_add(&shares)
			.ok_or(Error::<T>::MathOverflow)?;

		AutoCompoundingShares::<T>::insert(&candidate, &sender, sender_new_shares);
		AutoCompoundingShares::<T>::insert(&candidate, &recipient, recipient_new_shares);

		Pallet::<T>::deposit_event(Event::TransferedAutoCompounding {
			candidate,
			sender,
			recipient,
			shares,
		});

		Ok(().into())
	}

	pub fn transfer_manual_claim(
		origin: OriginFor<T>,
		candidate: T::AccountId,
		recipient: T::AccountId,
		shares: T::Balance,
	) -> DispatchResultWithPostInfo {
		ensure!(
			cfg!(feature = "transferable-shares"),
			Error::<T>::DisabledFeature
		);

		let sender = ensure_signed(origin)?;
		ensure!(!Zero::is_zero(&shares), Error::<T>::StakeMustBeNonZero);

		ensure!(
			sender != candidate,
			Error::<T>::CandidateTransferingOwnSharesForbidden
		);

		let sender_new_shares = ManualClaimShares::<T>::get(&candidate, &sender)
			.checked_sub(&shares)
			.ok_or(Error::<T>::UnsufficientSharesForTransfer)?;

		let recipient_new_shares = ManualClaimShares::<T>::get(&candidate, &recipient)
			.checked_add(&shares)
			.ok_or(Error::<T>::MathOverflow)?;

		// It is important to automatically claim rewards before updating
		// the amount of shares since pending rewards are stored per share.
		let sender_rewards =
			pools::manual_claim::claim_rewards::<T>(candidate.clone(), sender.clone())?;
		if !Zero::is_zero(&sender_rewards) {
			T::Currency::transfer(
				&T::StakingAccount::get(),
				&sender,
				sender_rewards,
				ExistenceRequirement::KeepAlive,
			)?;
		}

		let recipient_rewards =
			pools::manual_claim::claim_rewards::<T>(candidate.clone(), recipient.clone())?;
		if !Zero::is_zero(&recipient_rewards) {
			T::Currency::transfer(
				&T::StakingAccount::get(),
				&recipient,
				recipient_rewards,
				ExistenceRequirement::KeepAlive,
			)?;
		}

		ManualClaimShares::<T>::insert(&candidate, &sender, sender_new_shares);
		ManualClaimShares::<T>::insert(&candidate, &recipient, recipient_new_shares);

		Pallet::<T>::deposit_event(Event::TransferedManualClaim {
			candidate,
			sender,
			recipient,
			shares,
		});

		Ok(().into())
	}
}
