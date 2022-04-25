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

use super::*;

pub fn shares_to_stake<T: Config>(
	candidate: &T::AccountId,
	shares: &BalanceOf<T>,
) -> Result<BalanceOf<T>, Error<T>> {
	shares
		.checked_mul(&ManualClaimSharesTotalStaked::<T>::get(candidate))
		.ok_or(Error::MathOverflow)?
		.checked_div(&ManualClaimSharesSupply::<T>::get(candidate))
		.ok_or(Error::NoOneIsStaking)
}

pub fn shares_to_stake_or_init<T: Config>(
	candidate: &T::AccountId,
	shares: &BalanceOf<T>,
) -> Result<BalanceOf<T>, Error<T>> {
	if Zero::is_zero(&ManualClaimSharesSupply::<T>::get(&candidate)) {
		shares
			.checked_mul(&T::InitialManualClaimShareValue::get())
			.ok_or(Error::MathOverflow)
	} else {
		shares_to_stake(candidate, shares)
	}
}

pub fn stake_to_shares<T: Config>(
	candidate: &T::AccountId,
	stake: &BalanceOf<T>,
) -> Result<BalanceOf<T>, Error<T>> {
	stake
		.checked_mul(&ManualClaimSharesSupply::<T>::get(candidate))
		.ok_or(Error::MathOverflow)?
		.checked_div(&ManualClaimSharesTotalStaked::<T>::get(candidate))
		.ok_or(Error::NoOneIsStaking)
}

pub fn stake_to_shares_or_init<T: Config>(
	candidate: &T::AccountId,
	stake: &BalanceOf<T>,
) -> Result<BalanceOf<T>, Error<T>> {
	if Zero::is_zero(&ManualClaimSharesSupply::<T>::get(&candidate)) {
		stake
			.checked_div(&T::InitialManualClaimShareValue::get())
			.ok_or(Error::<T>::InvalidPalletSetting)
	} else {
		stake_to_shares(candidate, stake)
	}
}

pub fn add_shares<T: Config>(
	candidate: T::AccountId,
	delegator: T::AccountId,
	shares: BalanceOf<T>,
) -> Result<BalanceOf<T>, Error<T>> {
	ensure!(!Zero::is_zero(&shares), Error::StakeMustBeNonZero);

	let stake = shares_to_stake_or_init(&candidate, &shares)?;

	super::add_staked::<
		T,
		ManualClaimSharesSupply<T>,
		ManualClaimShares<T>,
		ManualClaimSharesTotalStaked<T>,
	>(&candidate, &delegator, shares, stake)?;

	Pallet::<T>::deposit_event(Event::<T>::StakedManualClaim {
		candidate,
		delegator,
		shares,
		stake,
	});

	Ok(stake)
}

pub fn sub_shares<T: Config>(
	candidate: T::AccountId,
	delegator: T::AccountId,
	shares: BalanceOf<T>,
) -> Result<BalanceOf<T>, Error<T>> {
	ensure!(!Zero::is_zero(&shares), Error::StakeMustBeNonZero);

	let stake = shares_to_stake(&candidate, &shares)?;

	super::sub_staked::<
		T,
		ManualClaimSharesSupply<T>,
		ManualClaimShares<T>,
		ManualClaimSharesTotalStaked<T>,
	>(&candidate, &delegator, shares, stake)?;

	Pallet::<T>::deposit_event(Event::<T>::UnstakedManualClaim {
		candidate,
		delegator,
		shares,
		stake,
	});

	Ok(stake)
}

pub fn shares<T: Config>(candidate: &T::AccountId, delegator: &T::AccountId) -> BalanceOf<T> {
	ManualClaimShares::<T>::get(candidate, delegator)
}

pub fn stake<T: Config>(
	candidate: &T::AccountId,
	delegator: &T::AccountId,
) -> Result<BalanceOf<T>, Error<T>> {
	let shares = shares::<T>(candidate, delegator);

	if shares.is_zero() {
		return Ok(Zero::zero());
	}

	shares_to_stake(candidate, &shares)
}

pub fn pending_rewards<T: Config>(
	candidate: &T::AccountId,
	delegator: &T::AccountId,
) -> Result<BalanceOf<T>, Error<T>> {
	let shares = ManualClaimShares::<T>::get(candidate, delegator);

	if Zero::is_zero(&shares) {
		return Ok(0_u32.into());
	}

	// TODO: Should be safe to wrap around.
	let checkpoint = ManualClaimSharesRewardCheckpoint::<T>::get(candidate, delegator);
	let diff = ManualClaimSharesRewardCounter::<T>::get(candidate)
		.checked_sub(&checkpoint)
		.ok_or(Error::MathUnderflow)?;

	diff.checked_mul(&shares).ok_or(Error::MathOverflow)
}

pub fn claim_rewards<T: Config>(
	candidate: T::AccountId,
	delegator: T::AccountId,
) -> Result<BalanceOf<T>, Error<T>> {
	let shares = ManualClaimShares::<T>::get(&candidate, &delegator);
	let rewards_counter = ManualClaimSharesRewardCounter::<T>::get(&candidate);

	if Zero::is_zero(&shares) {
		ManualClaimSharesRewardCheckpoint::<T>::insert(&candidate, &delegator, rewards_counter);
		return Ok(0_u32.into());
	}

	// TODO: Should be safe to wrap around.
	let checkpoint = ManualClaimSharesRewardCheckpoint::<T>::get(&candidate, &delegator);
	let diff = ManualClaimSharesRewardCounter::<T>::get(&candidate)
		.checked_sub(&checkpoint)
		.ok_or(Error::MathUnderflow)?;

	if Zero::is_zero(&diff) {
		return Ok(0_u32.into());
	}

	let rewards = diff.checked_mul(&shares).ok_or(Error::MathOverflow)?;

	ManualClaimSharesRewardCheckpoint::<T>::insert(&candidate, &delegator, rewards_counter);

	Pallet::<T>::deposit_event(Event::<T>::ClaimedManualRewards {
		candidate,
		delegator,
		rewards,
	});

	Ok(rewards)
}
