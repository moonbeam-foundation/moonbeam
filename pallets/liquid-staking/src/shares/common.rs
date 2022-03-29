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

pub fn add<T, Supply, Shares>(
	candidate: &T::AccountId,
	delegator: &T::AccountId,
	shares: BalanceOf<T>,
) -> Result<(), Error<T>>
where
	T: Config,
	Supply: StorageMap<T::AccountId, BalanceOf<T>, Query = BalanceOf<T>>,
	Shares: StorageDoubleMap<T::AccountId, T::AccountId, BalanceOf<T>, Query = BalanceOf<T>>,
{
	let new_shares_supply = Supply::get(&candidate)
		.checked_add(&shares)
		.ok_or(Error::MathOverflow)?;

	let new_shares = Shares::get(&candidate, &delegator)
		.checked_add(&shares)
		.ok_or(Error::MathOverflow)?;

	Supply::insert(&candidate, new_shares_supply);
	Shares::insert(&candidate, &delegator, new_shares);

	Ok(())
}

pub fn sub<T, Supply, Shares>(
	candidate: &T::AccountId,
	delegator: &T::AccountId,
	shares: BalanceOf<T>,
) -> Result<(), Error<T>>
where
	T: Config,
	Supply: StorageMap<T::AccountId, BalanceOf<T>, Query = BalanceOf<T>>,
	Shares: StorageDoubleMap<T::AccountId, T::AccountId, BalanceOf<T>, Query = BalanceOf<T>>,
{
	let new_shares_supply = Supply::get(&candidate)
		.checked_sub(&shares)
		.ok_or(Error::MathUnderflow)?;

	let new_shares = Shares::get(&candidate, &delegator)
		.checked_sub(&shares)
		.ok_or(Error::MathUnderflow)?;

	Supply::insert(&candidate, new_shares_supply);
	Shares::insert(&candidate, &delegator, new_shares);

	Ok(())
}

pub fn add_staked<T, Supply, Shares, Staked>(
	candidate: &T::AccountId,
	delegator: &T::AccountId,
	shares: BalanceOf<T>,
	stake: BalanceOf<T>,
) -> Result<(), Error<T>>
where
	T: Config,
	Supply: StorageMap<T::AccountId, BalanceOf<T>, Query = BalanceOf<T>>,
	Shares: StorageDoubleMap<T::AccountId, T::AccountId, BalanceOf<T>, Query = BalanceOf<T>>,
	Staked: StorageMap<T::AccountId, BalanceOf<T>, Query = BalanceOf<T>>,
{
	let new_shares_supply = Supply::get(&candidate)
		.checked_add(&shares)
		.ok_or(Error::MathOverflow)?;

	let new_shares = Shares::get(&candidate, &delegator)
		.checked_add(&shares)
		.ok_or(Error::MathOverflow)?;

	let new_total_stake = Staked::get(&candidate)
		.checked_add(&stake)
		.ok_or(Error::MathOverflow)?;

	Supply::insert(&candidate, new_shares_supply);
	Shares::insert(&candidate, &delegator, new_shares);
	Staked::insert(&candidate, new_total_stake);

	Ok(())
}

pub fn sub_staked<T, Supply, Shares, Staked>(
	candidate: &T::AccountId,
	delegator: &T::AccountId,
	shares: BalanceOf<T>,
	stake: BalanceOf<T>,
) -> Result<(), Error<T>>
where
	T: Config,
	Supply: StorageMap<T::AccountId, BalanceOf<T>, Query = BalanceOf<T>>,
	Shares: StorageDoubleMap<T::AccountId, T::AccountId, BalanceOf<T>, Query = BalanceOf<T>>,
	Staked: StorageMap<T::AccountId, BalanceOf<T>, Query = BalanceOf<T>>,
{
	let new_shares_supply = Supply::get(&candidate)
		.checked_sub(&shares)
		.ok_or(Error::MathUnderflow)?;

	let new_shares = Shares::get(&candidate, &delegator)
		.checked_sub(&shares)
		.ok_or(Error::MathUnderflow)?;

	let new_total_stake = Staked::get(&candidate)
		.checked_sub(&stake)
		.ok_or(Error::MathUnderflow)?;

	Supply::insert(&candidate, new_shares_supply);
	Shares::insert(&candidate, &delegator, new_shares);
	Staked::insert(&candidate, new_total_stake);

	Ok(())
}
