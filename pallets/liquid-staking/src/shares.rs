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
	frame_support::{ensure, traits::Get},
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
		staker: T::AccountId,
		shares: BalanceOf<T>,
	) -> Result<BalanceOf<T>, Error<T>> {
		ensure!(!Zero::is_zero(&shares), Error::StakeMustBeNonZero);

		let stake = shares_to_stake_or_init(&candidate, &shares)?;

		let new_shares_supply = ManualClaimSharesSupply::<T>::get(&candidate)
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

	pub fn sub_shares<T: Config>(
		candidate: T::AccountId,
		staker: T::AccountId,
		shares: BalanceOf<T>,
	) -> Result<BalanceOf<T>, Error<T>> {
		ensure!(!Zero::is_zero(&shares), Error::StakeMustBeNonZero);

		let stake = shares_to_stake(&candidate, &shares)?;

		let new_shares_supply = ManualClaimSharesSupply::<T>::get(&candidate)
			.checked_sub(&shares)
			.ok_or(Error::MathUnderflow)?;

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

		Ok(stake)
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

	pub fn shares_to_stake_or_init<T: Config>(
		candidate: &T::AccountId,
		shares: &BalanceOf<T>,
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

	pub fn stake_to_shares_or_init<T: Config>(
		candidate: &T::AccountId,
		stake: &BalanceOf<T>,
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
		staker: T::AccountId,
		shares: BalanceOf<T>,
	) -> Result<BalanceOf<T>, Error<T>> {
		ensure!(!Zero::is_zero(&shares), Error::StakeMustBeNonZero);

		let stake = shares_to_stake_or_init(&candidate, &shares)?;

		let new_shares_supply = AutoCompoundingSharesSupply::<T>::get(&candidate)
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

		Ok(stake)
	}

	pub fn sub_shares<T: Config>(
		candidate: T::AccountId,
		staker: T::AccountId,
		shares: BalanceOf<T>,
	) -> Result<BalanceOf<T>, Error<T>> {
		ensure!(!Zero::is_zero(&shares), Error::StakeMustBeNonZero);

		let stake = shares_to_stake(&candidate, &shares)?;

		let new_shares_supply = AutoCompoundingSharesSupply::<T>::get(&candidate)
			.checked_sub(&shares)
			.ok_or(Error::MathUnderflow)?;

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

		Ok(stake)
	}
}

pub mod leaving {
	use super::*;

	pub fn shares_to_stake<T: Config>(
		candidate: &T::AccountId,
		shares: &BalanceOf<T>,
	) -> Result<BalanceOf<T>, Error<T>> {
		shares
			.checked_mul(&LeavingSharesTotalStaked::<T>::get(candidate))
			.ok_or(Error::MathOverflow)?
			.checked_div(&LeavingSharesSupply::<T>::get(candidate))
			.ok_or(Error::NoOneIsStaking)
	}

	pub fn stake_to_shares<T: Config>(
		candidate: &T::AccountId,
		stake: &BalanceOf<T>,
	) -> Result<BalanceOf<T>, Error<T>> {
		stake
			.checked_mul(&LeavingSharesSupply::<T>::get(candidate))
			.ok_or(Error::MathOverflow)?
			.checked_div(&LeavingSharesTotalStaked::<T>::get(candidate))
			.ok_or(Error::NoOneIsStaking)
	}

	/// Add stake in the leaving pool of this Candidate.
	/// Accept stake instead of shares since we want to deal with rounding.
	/// Returns the amount of shares created.
	fn add_stake<T: Config>(
		candidate: T::AccountId,
		staker: T::AccountId,
		stake: BalanceOf<T>,
	) -> Result<BalanceOf<T>, Error<T>> {
		ensure!(!Zero::is_zero(&stake), Error::StakeMustBeNonZero);

		let shares_supply = LeavingSharesSupply::<T>::get(&candidate);

		let shares = if Zero::is_zero(&shares_supply) {
			stake // By default 1 share = 1 coin
		} else {
			// Number of shares might be rounded down / corresponds to slightly less stake.
			// But since we want to put all stake in the leaving state, we will not correct this
			// and spread the rounding among all leaving stakers.
			stake_to_shares(&candidate, &stake)?
		};

		let new_shares_supply = shares_supply
			.checked_add(&shares)
			.ok_or(Error::MathOverflow)?;

		let new_shares = LeavingShares::<T>::get(&candidate, &staker)
			.checked_add(&shares)
			.ok_or(Error::MathOverflow)?;

		let new_total_stake = LeavingSharesTotalStaked::<T>::get(&candidate)
			.checked_add(&stake)
			.ok_or(Error::MathOverflow)?;

		LeavingSharesSupply::<T>::insert(&candidate, new_shares_supply);
		LeavingShares::<T>::insert(&candidate, &staker, new_shares);
		LeavingSharesTotalStaked::<T>::insert(&candidate, new_total_stake);

		Ok(shares)
	}

	/// Remove shares from the leaving pool of this Candidate.
	/// Accept shares since the leaving queue deal with shares to support slashing.
	/// Returns value of removed shares.
	fn sub_shares<T: Config>(
		candidate: T::AccountId,
		staker: T::AccountId,
		shares: BalanceOf<T>,
	) -> Result<BalanceOf<T>, Error<T>> {
		ensure!(!Zero::is_zero(&shares), Error::StakeMustBeNonZero);

		let stake = shares_to_stake(&candidate, &shares)?;

		let new_shares_supply = LeavingSharesSupply::<T>::get(&candidate)
			.checked_sub(&shares)
			.ok_or(Error::MathUnderflow)?;

		let new_shares = LeavingShares::<T>::get(&candidate, &staker)
			.checked_sub(&shares)
			.ok_or(Error::MathUnderflow)?;

		let new_total_staked = LeavingSharesTotalStaked::<T>::get(&candidate)
			.checked_sub(&stake)
			.ok_or(Error::MathUnderflow)?;

		LeavingSharesSupply::<T>::insert(&candidate, new_shares_supply);
		LeavingShares::<T>::insert(&candidate, &staker, new_shares);
		LeavingSharesTotalStaked::<T>::insert(&candidate, new_total_staked);

		// Pallet::<T>::deposit_event(Event::<T>::UnstakedManualClaim {
		// 	candidate,
		// 	staker,
		// 	shares,
		// 	stake,
		// });

		// TODO: Event?

		Ok(stake)
	}

	pub fn register_leaving<T: Config>(
		candidate: T::AccountId,
		staker: T::AccountId,
		stake: BalanceOf<T>,
	) -> Result<(), Error<T>> {
		let leaving_shares = add_stake::<T>(candidate.clone(), staker.clone(), stake)?;

		let block_number = frame_system::Pallet::<T>::block_number();

		let already_leaving_shares = LeavingRequests::<T>::get((&candidate, &staker, block_number));

		let new_leaving_shares = already_leaving_shares
			.checked_add(&leaving_shares)
			.ok_or(Error::MathOverflow)?;

		LeavingRequests::<T>::insert((&candidate, &staker, block_number), new_leaving_shares);

		Pallet::<T>::deposit_event(Event::<T>::RegisteredLeaving {
			candidate,
			staker,
			stake,
			leaving_shares,
			total_leaving_shares: new_leaving_shares,
		});

		Ok(())
	}

	pub fn execute_leaving<T: Config>(
		candidate: T::AccountId,
		staker: T::AccountId,
		at_block: T::BlockNumber,
	) -> Result<BalanceOf<T>, Error<T>> {
		let block_number = frame_system::Pallet::<T>::block_number();

		let release_block_number = at_block
			.checked_add(&T::LeavingDelay::get())
			.ok_or(Error::MathOverflow)?;

		ensure!(
			block_number >= release_block_number,
			Error::TryingToLeaveTooSoon
		);

		let shares = LeavingRequests::<T>::get((&candidate, &staker, at_block));
		let stake = sub_shares(candidate.clone(), staker.clone(), shares)?;

		LeavingRequests::<T>::remove((&candidate, &staker, at_block));

		Pallet::<T>::deposit_event(Event::<T>::ExecutedLeaving {
			candidate,
			staker,
			stake,
			leaving_shares: shares,
			requested_at: at_block,
		});

		Ok(stake)
	}
}
