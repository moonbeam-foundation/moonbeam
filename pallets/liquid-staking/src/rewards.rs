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
	super::*,
	sp_runtime::traits::{CheckedMul, CheckedSub, Zero},
};

pub fn pending_rewards<T: Config>(
	candidate: &T::AccountId,
	staker: &T::AccountId,
) -> Result<BalanceOf<T>, Error<T>> {
	let shares = ManualClaimShares::<T>::get(candidate, staker);

	if Zero::is_zero(&shares) {
		return Ok(0_u32.into());
	}

	let checkpoint = ManualClaimSharesRewardCheckpoint::<T>::get(candidate, staker);
	let diff = ManualClaimSharesRewardCounter::<T>::get(candidate)
		.checked_sub(&checkpoint)
		.ok_or(Error::MathUnderflow)?;

	diff.checked_mul(&shares).ok_or(Error::MathOverflow)
}

pub fn claim_rewards<T: Config>(
	candidate: T::AccountId,
	staker: T::AccountId,
) -> Result<BalanceOf<T>, Error<T>> {
	let shares = ManualClaimShares::<T>::get(&candidate, &staker);
	let rewards_counter = ManualClaimSharesRewardCounter::<T>::get(&candidate);

	if Zero::is_zero(&shares) {
		ManualClaimSharesRewardCheckpoint::<T>::insert(&candidate, &staker, rewards_counter);
		return Ok(0_u32.into());
	}

	let checkpoint = ManualClaimSharesRewardCheckpoint::<T>::get(&candidate, &staker);
	let diff = ManualClaimSharesRewardCounter::<T>::get(&candidate)
		.checked_sub(&checkpoint)
		.ok_or(Error::MathUnderflow)?;

	if Zero::is_zero(&diff) {
		return Ok(0_u32.into());
	}

	let rewards = diff.checked_mul(&shares).ok_or(Error::MathOverflow)?;

	ManualClaimSharesRewardCheckpoint::<T>::insert(&candidate, &staker, rewards_counter);

	Pallet::<T>::deposit_event(Event::<T>::ClaimedManualRewards {
		candidate,
		staker,
		rewards,
	});

	Ok(rewards)
}

// TODO : Reward distribution
