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
	frame_support::ensure,
	sp_runtime::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Zero},
};

pub mod candidates {
	use super::*;

	pub fn add_stake<T: Config>(
		candidate: T::AccountId,
		stake: BalanceOf<T>,
	) -> Result<(), Error<T>> {
		ensure!(!Zero::is_zero(&stake), Error::StakeMustBeNonZero);


		let new_stake = CandidatesStake::<T>::get(&candidate)
			.checked_add(&stake)
			.ok_or(Error::MathOverflow)?;

		let new_total_staked = CandidatesTotalStaked::<T>::get()
			.checked_add(&stake)
			.ok_or(Error::MathOverflow)?;

		CandidatesStake::<T>::insert(&candidate, new_stake);
		CandidatesTotalStaked::<T>::set(new_total_staked);

		Pallet::<T>::deposit_event(Event::<T>::IncreasedStake { candidate, stake });

		Ok(())
	}

	pub fn sub_stake<T: Config>(
		candidate: T::AccountId,
		stake: BalanceOf<T>,
	) -> Result<(), Error<T>> {
		ensure!(!Zero::is_zero(&stake), Error::StakeMustBeNonZero);

		let new_stake = CandidatesStake::<T>::get(&candidate)
			.checked_sub(&stake)
			.ok_or(Error::MathUnderflow)?;

		let new_total_staked = CandidatesTotalStaked::<T>::get()
			.checked_sub(&stake)
			.ok_or(Error::MathUnderflow)?;

		CandidatesStake::<T>::insert(&candidate, new_stake);
		CandidatesTotalStaked::<T>::set(new_total_staked);

		Pallet::<T>::deposit_event(Event::<T>::DecreasedStake { candidate, stake });

		Ok(())
	}
}

// It is important to automatically claim rewards before updating
// the amount of shares since pending rewards are stored per share.
pub mod manual_claim {
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

	/// Create new shares for that candidate and staker.
	/// Since provided stake might not be a multiple of the value of
	/// a share, the stake will be rounded down to the closest value, which
	/// is returned by the function.
	pub fn add_shares<T: Config>(
		candidate: T::AccountId,
		staker: T::AccountId,
		stake: BalanceOf<T>,
	) -> Result<BalanceOf<T>, Error<T>> {
		ensure!(!Zero::is_zero(&stake), Error::StakeMustBeNonZero);

		let shares_supply = ManualClaimSharesSupply::<T>::get(&candidate);
		
		let (stake, shares) = if Zero::is_zero(&shares_supply) {
			let value_per_share = T::InitialManualClaimShareValue::get();
			let shares = stake
				.checked_div(&value_per_share)
				.ok_or(Error::InvalidPalletSetting)?;

			let stake = shares
				.checked_mul(&value_per_share)
				.ok_or(Error::MathOverflow)?;

			(stake, shares)
		} else {
			let shares = stake_to_shares::<T>(&candidate, &stake)?;
			let stake = shares_to_stake::<T>(&candidate, &shares)?;

			(stake, shares)
		};

		let new_shares_supply = shares_supply
			.checked_add(&shares)
			.ok_or(Error::MathOverflow)?;

		let new_shares = ManualClaimShares::<T>::get(&candidate, &staker)
			.checked_add(&shares)
			.ok_or(Error::MathOverflow)?;

		let new_total_staked = ManualClaimSharesTotalStaked::<T>::get(&candidate)
			.checked_add(&stake)
			.ok_or(Error::MathOverflow)?;

		ManualClaimSharesSupply::<T>::insert(&candidate, new_shares_supply);
		ManualClaimShares::<T>::insert(&candidate, &staker, new_shares);
		ManualClaimSharesTotalStaked::<T>::insert(&candidate, new_total_staked);

		Pallet::<T>::deposit_event(Event::<T>::StakedManualClaim {
			candidate,
			staker,
			shares,
			stake,
		});

		Ok(stake)
	}

	/// Remove shares for that candidate and staker.
	/// Since provided stake might not be a multiple of the value of
	/// a share, the stake will be rounded down to the closest value, which
	/// is returned by the function.
	pub fn sub_shares<T: Config>(
		candidate: T::AccountId,
		staker: T::AccountId,
		stake: BalanceOf<T>,
	) -> Result<BalanceOf<T>, Error<T>> {
		ensure!(!Zero::is_zero(&stake), Error::StakeMustBeNonZero);

		let shares = stake_to_shares::<T>(&candidate, &stake)?;
		let stake = shares_to_stake::<T>(&candidate, &shares)?;

		let new_shares_supply = ManualClaimSharesSupply::<T>::get(&candidate)
			.checked_sub(&shares)
			.ok_or(Error::NotEnoughShares)?;

		let new_shares = ManualClaimShares::<T>::get(&candidate, &staker)
			.checked_sub(&shares)
			.ok_or(Error::MathUnderflow)?;

		let new_total_staked = ManualClaimSharesTotalStaked::<T>::get(&candidate)
			.checked_sub(&stake)
			.ok_or(Error::MathUnderflow)?;

		ManualClaimSharesSupply::<T>::insert(&candidate, new_shares_supply);
		ManualClaimShares::<T>::insert(&candidate, &staker, new_shares);
		ManualClaimSharesTotalStaked::<T>::insert(&candidate, new_total_staked);

		Pallet::<T>::deposit_event(Event::<T>::UnstakedManualClaim {
			candidate,
			staker,
			shares,
			stake,
		});

		Ok(shares)
	}
}

pub mod auto_compounding {
	use super::*;

	pub fn shares_to_stake<T: Config>(
		candidate: &T::AccountId,
		shares: &BalanceOf<T>,
	) -> Result<BalanceOf<T>, Error<T>> {
		let candidate_stake = CandidatesStake::<T>::get(candidate);
		let auto_compounded_stake = candidate_stake
			.checked_sub(&ManualClaimSharesTotalStaked::<T>::get(candidate))
			.ok_or(Error::MathUnderflow)?;

		shares
			.checked_mul(&auto_compounded_stake)
			.ok_or(Error::MathOverflow)?
			.checked_div(&AutoCompoundingSharesSupply::<T>::get(candidate))
			.ok_or(Error::NoOneIsStaking)
	}

	pub fn stake_to_shares<T: Config>(
		candidate: &T::AccountId,
		stake: &BalanceOf<T>,
	) -> Result<BalanceOf<T>, Error<T>> {
		let candidate_stake = CandidatesStake::<T>::get(candidate);
		let auto_compounded_stake = candidate_stake
			.checked_sub(&ManualClaimSharesTotalStaked::<T>::get(candidate))
			.ok_or(Error::MathUnderflow)?;

		stake
			.checked_mul(&AutoCompoundingSharesSupply::<T>::get(candidate))
			.ok_or(Error::MathOverflow)?
			.checked_div(&auto_compounded_stake)
			.ok_or(Error::NoOneIsStaking)
	}

	pub fn add_shares<T: Config>(
		candidate: T::AccountId,
		staker: T::AccountId,
		stake: BalanceOf<T>,
	) -> Result<BalanceOf<T>, Error<T>> {
		ensure!(!Zero::is_zero(&stake), Error::StakeMustBeNonZero);

		let shares_supply = AutoCompoundingSharesSupply::<T>::get(&candidate);
		let shares = if Zero::is_zero(&shares_supply) {
			// TODO : Better choice of value ?
			// Value is only relevant to reduce rounding errors when converting between stake
			// and shares, or when computing rewards.
			10_000_000_u32.into()
		} else {
			stake_to_shares::<T>(&candidate, &stake)?
		};

		let new_shares_supply = shares_supply
			.checked_add(&shares)
			.ok_or(Error::MathOverflow)?;

		let new_shares = AutoCompoundingShares::<T>::get(&candidate, &staker)
			.checked_add(&shares)
			.ok_or(Error::MathOverflow)?;

		AutoCompoundingSharesSupply::<T>::insert(&candidate, new_shares_supply);
		AutoCompoundingShares::<T>::insert(&candidate, &staker, new_shares);

		Pallet::<T>::deposit_event(Event::<T>::StakedAutoCompounding {
			candidate,
			staker,
			shares,
			stake,
		});

		Ok(shares)
	}

	pub fn sub_shares<T: Config>(
		candidate: T::AccountId,
		staker: T::AccountId,
		stake: BalanceOf<T>,
	) -> Result<BalanceOf<T>, Error<T>> {
		ensure!(!Zero::is_zero(&stake), Error::StakeMustBeNonZero);

		let shares = stake_to_shares::<T>(&candidate, &stake)?;

		let new_shares_supply = AutoCompoundingSharesSupply::<T>::get(&candidate)
			.checked_sub(&shares)
			.ok_or(Error::NotEnoughShares)?;

		let new_shares = AutoCompoundingShares::<T>::get(&candidate, &staker)
			.checked_sub(&shares)
			.ok_or(Error::MathUnderflow)?;

		AutoCompoundingSharesSupply::<T>::insert(&candidate, new_shares_supply);
		AutoCompoundingShares::<T>::insert(&candidate, &staker, new_shares);

		Pallet::<T>::deposit_event(Event::<T>::UnstakedAutoCompounding {
			candidate,
			staker,
			shares,
			stake,
		});

		Ok(shares)
	}
}
