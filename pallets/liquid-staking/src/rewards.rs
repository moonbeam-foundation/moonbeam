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
		PerThing, Perbill,
	},
	substrate_fixed::{
		transcendental::pow as floatpow,
		types::{I32F32, I64F64},
	},
};

#[transactional]
pub(crate) fn distribute_rewards<T: Config>(
	collator: T::AccountId,
	value: T::Balance,
) -> Result<(), Error<T>> {
	// Compute rewards distribution.
	let reserve_rewards = T::RewardsReserveCommission::get() * value;
	let shared_rewards = value
		.checked_sub(&reserve_rewards)
		.ok_or(Error::MathUnderflow)?;
	let collator_rewards = T::RewardsCollatorCommission::get() * value;
	let delegators_rewards = shared_rewards
		.checked_sub(&collator_rewards)
		.ok_or(Error::MathUnderflow)?;

	// Mint new currency for reserve.
	T::Currency::deposit_creating(&T::ReserveAccount::get(), reserve_rewards);

	// Compute staking rewards.
	// All rewards must be computed before being distributed for calculations
	// to not be impacted by other distributions.
	let (delegators_mc_rewards, delegators_rewards_per_share, delegators_ac_rewards) =
		compute_delegators_rewards::<T>(collator.clone(), delegators_rewards)?;

	let (collator_mc_rewards, collator_ac_rewards_in_shares) =
		compute_collator_rewards::<T>(collator.clone(), collator_rewards)?;

	// Distribute staking rewards.
	// Distributing collator AC rewards must be done in shares and thus
	// the total staked should not have been changed yet, which is changed when
	// distributing delegators AC rewards.

	// - Collator manual claim
	// Rewards are directly minted in collator account.
	if !collator_mc_rewards.is_zero() {
		T::Currency::deposit_creating(&collator, collator_mc_rewards);
	}

	// - Collator auto compounding
	let collator_ac_rewards = if collator_ac_rewards_in_shares.is_zero() {
		Zero::zero()
	} else {
		let collator_ac_rewards = pools::auto_compounding::add_shares::<T>(
			collator.clone(),
			collator.clone(),
			collator_ac_rewards_in_shares,
		)?;
		T::Currency::deposit_creating(&T::StakingAccount::get(), collator_ac_rewards);
		collator_ac_rewards
	};

	// All delegators rewards are managed by the staking pallet:
	// - Auto compounding : increase total staked
	// - Manual claim : in manual claim system handled by the pallet
	T::Currency::deposit_creating(&T::StakingAccount::get(), delegators_rewards);

	// - Delegators manual claim
	// TODO: Should be safe to wrap around.
	if !delegators_rewards_per_share.is_zero() {
		let counter = ManualClaimSharesRewardCounter::<T>::get(&collator);
		let counter = counter
			.checked_add(&delegators_rewards_per_share)
			.ok_or(Error::MathOverflow)?;
		ManualClaimSharesRewardCounter::<T>::insert(&collator, counter);
	}

	// - Delegators auto compounding
	if !delegators_ac_rewards.is_zero() {
		pools::auto_compounding::share_stake_among_holders(&collator, delegators_ac_rewards)?;
	}

	// - Update candidate stake
	let additional_stake = collator_ac_rewards
		.checked_add(&delegators_ac_rewards)
		.ok_or(Error::MathOverflow)?;
	if !additional_stake.is_zero() {
		pools::candidates::add_stake::<T>(collator.clone(), additional_stake)?;
	}

	pools::check_candidate_consistency::<T>(&collator)?;

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

fn compute_delegators_rewards<T: Config>(
	collator: T::AccountId,
	value: T::Balance,
) -> Result<(T::Balance, T::Balance, T::Balance), Error<T>> {
	// Rewards must be split according to repartition between
	// AutoCompounding and ManualClaim shares.
	//
	// Distributing the manual claim rewards will lead to rounding since rewards are
	// stored per share, while auto compounding rewards will not be rounded since we
	// simply increase the total staked by auto compounding shares.
	//
	// Thus, we compute first the manual claim rewards and give the rounding
	// error as auto compounding rewards.

	let total_stake = pools::candidates::stake::<T>(&collator);
	let mc_stake = ManualClaimSharesTotalStaked::<T>::get(&collator);

	if mc_stake.is_zero() {
		Ok((Zero::zero(), Zero::zero(), value))
	} else {
		let mc_rewards = value
			.mul_div(mc_stake, total_stake)
			.ok_or(Error::MathOverflow)?;

		let rewards_per_share = mc_rewards
			.checked_div(&ManualClaimSharesSupply::<T>::get(&collator))
			.ok_or(Error::NoOneIsStaking)?;

		let mc_rewards = rewards_per_share
			.checked_mul(&ManualClaimSharesSupply::<T>::get(&collator))
			.ok_or(Error::MathOverflow)?;

		let ac_rewards = value.checked_sub(&mc_rewards).ok_or(Error::MathUnderflow)?;

		Ok((mc_rewards, rewards_per_share, ac_rewards))
	}
}

fn compute_collator_rewards<T: Config>(
	collator: T::AccountId,
	value: T::Balance,
) -> Result<(T::Balance, T::Balance), Error<T>> {
	// Rewards must be split according to repartition between
	// AutoCompounding and ManualClaim shares.
	//
	// Since manual claim rewards will be minted directly to the collator account
	// then there is no rounding. However distributing their auto compounding
	// rewards require to mint new shares, which may lead to rounding to have
	// an integer amount of shares.
	//
	// Thus, we compute first the auto compounding rewards and give the rounding
	// error as manual claim rewards.

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
		.mul_div(ac_stake, sum_stake)
		.ok_or(Error::MathOverflow)?;

	let (ac_shares, ac_rewards) = if !ac_rewards.is_zero() {
		// Rewards are staked automatically.
		// Not staked dust is moved to manual rewards distribution.
		let ac_shares = pools::auto_compounding::stake_to_shares::<T>(&collator, ac_rewards)?;

		if !ac_shares.is_zero() {
			(
				ac_shares,
				pools::auto_compounding::shares_to_stake::<T>(&collator, ac_shares)?,
			)
		} else {
			(Zero::zero(), Zero::zero())
		}
	} else {
		(Zero::zero(), Zero::zero())
	};

	let mc_rewards = value.checked_sub(&ac_rewards).ok_or(Error::MathUnderflow)?;

	Ok((mc_rewards, ac_shares))
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
