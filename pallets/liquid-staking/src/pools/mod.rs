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
	frame_support::pallet_prelude::*,
	frame_support::{ensure, traits::Get, StorageDoubleMap, StorageMap},
	sp_runtime::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Zero},
};

pub mod auto_compounding;
pub mod candidates;
pub mod leaving;

// It is important to automatically claim rewards before updating
// the amount of shares since pending rewards are stored per share.
pub mod manual_claim;

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

pub fn check_candidate_consistency(
	candidate: &T::AccountId,
) -> Result<(), Error<T>> {
	let total0 = CandidatesStake::<T>::get(&candidate);

	let auto = AutoCompoundingSharesTotalStaked::<T>::get(&candidate);
	let manual = ManualCompoundingSharesTotalStaked::<T>::get(&candidate);
	let leaving = LeavingSharesTotalStaked::<T>::get(&candidate);

	let total1 = auto
		.checked_add(manual)
		.ok_or(Error::InconsistentState)?
		.checked_add(leaving)
		.ok_or(Error::InconsistentState)?;

	ensure!(total0 == total1, Error::InconsistentState);

	Ok(())
}