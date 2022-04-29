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
	frame_support::{
		traits::{tokens::currency::Currency, Get},
		transactional,
	},
	sp_runtime::{
		traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Zero},
		PerThing, Perbill,
	},
	substrate_fixed::{
		transcendental::pow as floatpow,
		types::{I32F32, I64F64},
	},
};

#[transactional]
pub fn distribute_rewards<T: Config>(
	collator: T::AccountId,
	value: BalanceOf<T>,
) -> Result<(), Error<T>> {
	// Compute rewards distribution.
	let reserve_rewards = RewardsReserveCommission::<T>::get() * value;
	let shared_rewards = value
		.checked_sub(&reserve_rewards)
		.ok_or(Error::MathUnderflow)?;
	let collator_rewards = RewardsCollatorCommission::<T>::get() * value;
	let delegators_rewards = shared_rewards
		.checked_sub(&collator_rewards)
		.ok_or(Error::MathUnderflow)?;

	// Mint new currency for reserve.
	T::Currency::deposit_creating(&T::ReserveAccount::get(), reserve_rewards);

	// Distribute staking rewards.
	reward_delegators::<T>(collator.clone(), delegators_rewards)?;
	reward_collator::<T>(collator, collator_rewards)?;

	Ok(())
}

fn reward_delegators<T: Config>(
	collator: T::AccountId,
	value: BalanceOf<T>,
) -> Result<(), Error<T>> {
	// All rewards are part of staking design.
	T::Currency::deposit_creating(&T::StakingAccount::get(), value);

	// Rewards must be split according to repartition between
	// AutoCompounding and ManualClaim shares.
	let total_stake = pools::candidates::stake::<T>(&collator);
	let mc_stake = ManualClaimSharesTotalStaked::<T>::get(&collator);

	// ManualClaim rewards
	let mc_rewards = value
		.checked_mul(&mc_stake)
		.ok_or(Error::MathOverflow)?
		.checked_div(&total_stake)
		.ok_or(Error::RewardsMustBeNonZero)?;

	if !mc_rewards.is_zero() {
		// Should not fail. If rewards/total staked is non zero then
		// the supply should be non-zero.
		let rewards_per_share = mc_rewards
			.checked_div(&ManualClaimSharesSupply::<T>::get(&collator))
			.ok_or(Error::NoOneIsStaking)?;

		// TODO: Should be safe to wrap around.
		let counter = ManualClaimSharesRewardCounter::<T>::get(&collator);
		let counter = counter
			.checked_add(&rewards_per_share)
			.ok_or(Error::MathOverflow)?;
		ManualClaimSharesRewardCounter::<T>::insert(&collator, counter);
	}

	// AutoCompounnding rewards.
	// This is done simply by increasing the stake of the collator.
	let ac_rewards = value.checked_sub(&mc_rewards).ok_or(Error::MathUnderflow)?;
	pools::candidates::add_stake(collator.clone(), ac_rewards)?;

	Pallet::<T>::deposit_event(Event::<T>::RewardedDelegators {
		collator,
		auto_compounding_rewards: ac_rewards,
		manual_claim_rewards: mc_rewards,
	});

	Ok(())
}

fn reward_collator<T: Config>(collator: T::AccountId, value: BalanceOf<T>) -> Result<(), Error<T>> {
	// Rewards must be split according to repartition between
	// AutoCompounding and ManualClaim shares.

	let mc_stake = if !ManualClaimSharesSupply::<T>::get(&collator).is_zero() {
		pools::manual_claim::stake(&collator, &collator)?
	} else {
		Zero::zero()
	};

	let ac_stake = if !AutoCompoundingSharesSupply::<T>::get(&collator).is_zero() {
		pools::auto_compounding::stake(&collator, &collator)?
	} else {
		Zero::zero()
	};

	let sum_stake = mc_stake.checked_add(&ac_stake).ok_or(Error::MathOverflow)?;

	let ac_rewards = value
		.checked_mul(&ac_stake)
		.ok_or(Error::MathOverflow)?
		.checked_div(&sum_stake)
		.ok_or(Error::StakeMustBeNonZero)?;

	let ac_rewards = if !ac_rewards.is_zero() {
		// Rewards are staked automatically.
		// Not staked dust is moved to manual rewards distribution.
		let shares = pools::auto_compounding::stake_to_shares::<T>(&collator, ac_rewards)?;

		if !shares.is_zero() {
			let stake = pools::auto_compounding::add_shares::<T>(
				collator.clone(),
				collator.clone(),
				shares,
			)?;
			pools::candidates::add_stake::<T>(collator.clone(), stake)?;
			T::Currency::deposit_creating(&T::StakingAccount::get(), stake);

			stake
		} else {
			Zero::zero()
		}
	} else {
		Zero::zero()
	};

	let mc_rewards = value.checked_sub(&ac_rewards).ok_or(Error::MathUnderflow)?;

	if !mc_rewards.is_zero() {
		// Rewards are directly minted in collator account.
		T::Currency::deposit_creating(&collator, mc_rewards);
	}

	Pallet::<T>::deposit_event(Event::<T>::RewardedCollator {
		collator,
		auto_compounding_rewards: ac_rewards,
		manual_claim_rewards: mc_rewards,
	});

	Ok(())
}

/// Convert an annual inflation to a block inflation
/// round = (1+annual)^(1/blocks_per_year) - 1
pub fn annual_to_block_inflation(annual_inflation: Perbill, sec_per_block: u32) -> Perbill {
	const SECONDS_PER_YEAR: u32 = 31557600;
	let blocks_per_year = SECONDS_PER_YEAR / sec_per_block;

	let exponent = I32F32::from_num(1) / I32F32::from_num(blocks_per_year);

	let x = I32F32::from_num(annual_inflation.deconstruct()) / I32F32::from_num(Perbill::ACCURACY);
	let y: I64F64 = floatpow(I32F32::from_num(1) + x, exponent)
		.expect("Cannot overflow since blocks_per_year is u32 so worst case 0; QED");
	Perbill::from_parts(
		((y - I64F64::from_num(1)) * I64F64::from_num(Perbill::ACCURACY))
			.ceil()
			.to_num::<u32>(),
	)
}
