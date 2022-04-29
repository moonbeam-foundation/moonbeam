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
	shares: BalanceOf<T>,
) -> Result<BalanceOf<T>, Error<T>> {
	let total_staked = AutoCompoundingSharesTotalStaked::<T>::get(candidate);
	let supply = AutoCompoundingSharesSupply::<T>::get(candidate);
	ensure!(!supply.is_zero(), Error::NoOneIsStaking);

	shares
		.mul_div(total_staked, supply)
		.ok_or(Error::MathOverflow)
}

pub fn shares_to_stake_or_init<T: Config>(
	candidate: &T::AccountId,
	shares: BalanceOf<T>,
) -> Result<BalanceOf<T>, Error<T>> {
	if Zero::is_zero(&AutoCompoundingSharesSupply::<T>::get(&candidate)) {
		shares
			.checked_mul(&T::InitialAutoCompoundingShareValue::get())
			.ok_or(Error::MathOverflow)
	} else {
		shares_to_stake(candidate, shares)
	}
}

pub fn stake_to_shares<T: Config>(
	candidate: &T::AccountId,
	stake: BalanceOf<T>,
) -> Result<BalanceOf<T>, Error<T>> {
	let total_staked = AutoCompoundingSharesTotalStaked::<T>::get(candidate);
	let supply = AutoCompoundingSharesSupply::<T>::get(candidate);
	ensure!(!total_staked.is_zero(), Error::NoOneIsStaking);

	stake
		.mul_div(supply, total_staked)
		.ok_or(Error::MathOverflow)
}

pub fn stake_to_shares_or_init<T: Config>(
	candidate: &T::AccountId,
	stake: BalanceOf<T>,
) -> Result<BalanceOf<T>, Error<T>> {
	if Zero::is_zero(&AutoCompoundingSharesSupply::<T>::get(&candidate)) {
		stake
			.checked_div(&T::InitialAutoCompoundingShareValue::get())
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

	let stake = shares_to_stake_or_init(&candidate, shares)?;

	super::add_staked::<
		T,
		AutoCompoundingSharesSupply<T>,
		AutoCompoundingShares<T>,
		AutoCompoundingSharesTotalStaked<T>,
	>(&candidate, &delegator, shares, stake)?;

	Pallet::<T>::deposit_event(Event::<T>::StakedAutoCompounding {
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

	let stake = shares_to_stake(&candidate, shares)?;

	super::sub_staked::<
		T,
		AutoCompoundingSharesSupply<T>,
		AutoCompoundingShares<T>,
		AutoCompoundingSharesTotalStaked<T>,
	>(&candidate, &delegator, shares, stake)?;

	Pallet::<T>::deposit_event(Event::<T>::UnstakedAutoCompounding {
		candidate,
		delegator,
		shares,
		stake,
	});

	Ok(stake)
}

pub fn shares<T: Config>(candidate: &T::AccountId, delegator: &T::AccountId) -> BalanceOf<T> {
	AutoCompoundingShares::<T>::get(candidate, delegator)
}

pub fn stake<T: Config>(
	candidate: &T::AccountId,
	delegator: &T::AccountId,
) -> Result<BalanceOf<T>, Error<T>> {
	let shares = shares::<T>(candidate, delegator);

	if shares.is_zero() {
		return Ok(Zero::zero());
	}

	shares_to_stake(candidate, shares)
}
