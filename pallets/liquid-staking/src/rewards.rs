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
	mul_div::MulDiv,
	sp_runtime::{
		traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Zero},
		DispatchError, PerThing, Perbill,
	},
	substrate_fixed::{
		transcendental::pow as floatpow,
		types::{I32F32, I64F64},
	},
};

// We need to return a `DispatchError` since `#[transactional]` itself can return an
// error if there is too much transaction nesting.
#[transactional]
pub(crate) fn distribute_rewards<T: Config>(
	collator: CandidateGen<T>,
	value: T::Balance,
) -> Result<(), DispatchError> {
	// Rewards distribution is done in the following order :
	// 1. Distribute to reserve.
	// 2. Distribute delegator MC rewards, since it implies some rounding.
	// 3. Distribute delegator AC rewards with dust from 2.
	// 4. Distribute collator AC rewards, since it implies some rounding.
	//    It is done after 2 as we want rewards to be distributed among all delegators
	//    according to the pre-reward repartition.
	// 5. Distribute collator MC rewards, with dust from 4.

	// Compute rewards distribution between collator and delegators.
	let reserve_rewards = T::RewardsReserveCommission::get() * value;
	let shared_rewards = value
		.checked_sub(&reserve_rewards)
		.ok_or(Error::<T>::MathUnderflow)?;
	let collator_rewards = T::RewardsCollatorCommission::get() * value;
	let delegators_rewards = shared_rewards
		.checked_sub(&collator_rewards)
		.ok_or(Error::<T>::MathUnderflow)?;

	// 1. Distribute to reserve.
	T::Currency::deposit_creating(&T::ReserveAccount::get(), reserve_rewards);

	// 2 + 3. All rewards are held by the staking pallet.
	T::Currency::deposit_creating(&T::StakingAccount::get(), delegators_rewards);

	// 2. Distribute delegator MC rewards, since it implies some rounding.
	let delegators_mc_rewards =
		compute_delegators_mc_rewards_before_rounding::<T>(&collator, delegators_rewards)?;
	let delegators_mc_rewards =
		distribute_delegators_mc_rewards::<T>(&collator, delegators_mc_rewards)?;

	// 3. Distribute delegator AC rewards with dust from 2.
	let delegators_ac_rewards = delegators_rewards
		.checked_sub(&delegators_mc_rewards)
		.ok_or(Error::<T>::MathUnderflow)?;

	distribute_delegators_ac_rewards::<T>(&collator, delegators_ac_rewards)?;

	// 4. Distribute collator AC rewards, since it implies some rounding.
	//    It is done after 2 as we want rewards to be distributed among all delegators
	//    according to the pre-reward repartition.
	let collator_ac_rewards =
		compute_collator_ac_rewards_before_rounding::<T>(&collator, collator_rewards)?;
	let collator_ac_rewards = distribute_collator_ac_rewards::<T>(&collator, collator_ac_rewards)?;

	// 5. Distribute collator MC rewards, with dust from 4.
	let collator_mc_rewards = collator_rewards
		.checked_sub(&collator_ac_rewards)
		.ok_or(Error::<T>::MathUnderflow)?;

	distribute_collator_mc_rewards::<T>(&collator, collator_mc_rewards)?;

	// Update candidate stake
	let additional_stake = collator_ac_rewards
		.checked_add(&delegators_ac_rewards)
		.ok_or(Error::<T>::MathOverflow)?;
	if !additional_stake.is_zero() {
		pools::candidates::add_stake::<T>(collator.clone(), additional_stake)?;
	}

	pools::check_candidate_consistency::<T>(&collator)?;

	// Emit final events.
	Pallet::<T>::deposit_event(Event::<T>::RewardedCollator {
		collator: collator.clone(),
		auto_compounding_rewards: collator_ac_rewards,
		manual_claim_rewards: collator_mc_rewards,
	});

	Pallet::<T>::deposit_event(Event::<T>::RewardedDelegators {
		collator,
		auto_compounding_rewards: delegators_ac_rewards,
		manual_claim_rewards: delegators_mc_rewards,
	});

	Ok(())
}

// Split delegators rewards between MC and AC (before MC rounding)
fn compute_delegators_mc_rewards_before_rounding<T: Config>(
	collator: &CandidateGen<T>,
	delegators_rewards: T::Balance,
) -> Result<T::Balance, Error<T>> {
	let total_stake = pools::candidates::stake::<T>(collator);
	let mc_stake = ManualClaimSharesTotalStaked::<T>::get(collator);

	if mc_stake.is_zero() {
		Ok(Zero::zero())
	} else {
		Ok(delegators_rewards
			.mul_div(mc_stake, total_stake)
			.ok_or(Error::MathOverflow)?)
	}
}

// Distribute delegators MC rewards.
// Return the amount really distributed (after rounding).
fn distribute_delegators_mc_rewards<T: Config>(
	collator: &CandidateGen<T>,
	mc_rewards: T::Balance,
) -> Result<T::Balance, Error<T>> {
	if mc_rewards.is_zero() {
		Ok(Zero::zero())
	} else {
		let rewards_per_share = mc_rewards
			.checked_div(&ManualClaimSharesSupply::<T>::get(collator))
			.ok_or(Error::NoOneIsStaking)?;

		// Compute total after per-share rounding.
		let mc_rewards = rewards_per_share
			.checked_mul(&ManualClaimSharesSupply::<T>::get(collator))
			.ok_or(Error::MathOverflow)?;

		// TODO: Should be safe to wrap around.
		if !rewards_per_share.is_zero() {
			let counter = ManualClaimSharesRewardCounter::<T>::get(collator);
			let counter = counter
				.checked_add(&rewards_per_share)
				.ok_or(Error::MathOverflow)?;
			ManualClaimSharesRewardCounter::<T>::insert(collator, counter);
		}

		Ok(mc_rewards)
	}
}

// Distribute delegators AC rewards.
fn distribute_delegators_ac_rewards<T: Config>(
	collator: &CandidateGen<T>,
	ac_rewards: T::Balance,
) -> Result<(), Error<T>> {
	if !ac_rewards.is_zero() {
		pools::auto_compounding::share_stake_among_holders(collator, ac_rewards)?;
	}

	Ok(())
}

// Split collator rewards between MC and AC (before AC rounding)
fn compute_collator_ac_rewards_before_rounding<T: Config>(
	collator: &CandidateGen<T>,
	collator_rewards: T::Balance,
) -> Result<T::Balance, Error<T>> {
	let mc_stake = if !ManualClaimSharesSupply::<T>::get(collator).is_zero() {
		pools::manual_claim::stake(collator, &collator.id)?
	} else {
		Zero::zero()
	};

	let ac_stake = if !AutoCompoundingSharesSupply::<T>::get(collator).is_zero() {
		pools::auto_compounding::stake(collator, &collator.id)?
	} else {
		Zero::zero()
	};

	let sum_stake = mc_stake.checked_add(&ac_stake).ok_or(Error::MathOverflow)?;

	if sum_stake.is_zero() {
		Ok(Zero::zero())
	} else {
		collator_rewards
			.mul_div(ac_stake, sum_stake)
			.ok_or(Error::MathOverflow)
	}
}

// Distribute collator AC rewards.
// Return the amount really distributed (after rounding).
fn distribute_collator_ac_rewards<T: Config>(
	collator: &CandidateGen<T>,
	ac_rewards: T::Balance,
) -> Result<T::Balance, Error<T>> {
	if ac_rewards.is_zero() {
		Ok(Zero::zero())
	} else {
		let ac_shares = pools::auto_compounding::stake_to_shares::<T>(collator, ac_rewards)?;

		let ac_rewards = pools::auto_compounding::add_shares::<T>(
			collator.clone(),
			collator.id.clone(),
			ac_shares,
		)?;
		T::Currency::deposit_creating(&T::StakingAccount::get(), ac_rewards);
		Ok(ac_rewards)
	}
}

// Distribute collator MC rewards.
fn distribute_collator_mc_rewards<T: Config>(
	collator: &CandidateGen<T>,
	mc_rewards: T::Balance,
) -> Result<(), Error<T>> {
	if !mc_rewards.is_zero() {
		T::Currency::deposit_creating(&collator.id, mc_rewards);
	}

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
