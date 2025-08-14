// Copyright 2019-2025 PureStake Inc.
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

//! Precompile to call pallet-crowdloan-rewards runtime methods via the EVM

#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::PrecompileHandle;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	traits::Currency,
};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;

use sp_core::{H160, U256};
use sp_runtime::traits::Dispatchable;
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	marker::PhantomData,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub type BalanceOf<Runtime> =
	<<Runtime as pallet_crowdloan_rewards::Config>::RewardCurrency as Currency<
		<Runtime as frame_system::Config>::AccountId,
	>>::Balance;

/// A precompile to wrap the functionality from pallet_crowdloan_rewards.
pub struct CrowdloanRewardsPrecompile<Runtime>(PhantomData<Runtime>);

#[precompile_utils::precompile]
impl<Runtime> CrowdloanRewardsPrecompile<Runtime>
where
	Runtime: pallet_crowdloan_rewards::Config + pallet_evm::Config + frame_system::Config,
	BalanceOf<Runtime>: TryFrom<U256> + TryInto<u128> + Debug,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	Runtime::RuntimeCall: From<pallet_crowdloan_rewards::Call<Runtime>>,
	<Runtime as pallet_evm::Config>::AddressMapping: AddressMapping<Runtime::AccountId>,
{
	// The accessors are first.
	#[precompile::public("isContributor(address)")]
	#[precompile::public("is_contributor(address)")]
	#[precompile::view]
	fn is_contributor(handle: &mut impl PrecompileHandle, contributor: Address) -> EvmResult<bool> {
		// AccountsPayable: Blake2128(16) + 20 + RewardInfo(16 + 16 + UnBoundedVec<AccountId32(32)>)
		// TODO RewardInfo.contributed_relay_addresses is unbounded, we set a safe length of 100.
		handle.record_db_read::<Runtime>(3268)?;

		let contributor: H160 = contributor.into();

		let account = Runtime::AddressMapping::into_account_id(contributor);

		log::trace!(
			target: "crowdloan-rewards-precompile",
			"Checking whether {:?} is a contributor",
			contributor
		);

		// fetch data from pallet
		let is_contributor: bool =
			pallet_crowdloan_rewards::Pallet::<Runtime>::accounts_payable(account).is_some();

		log::trace!(target: "crowldoan-rewards-precompile", "Result from pallet is {:?}", is_contributor);

		Ok(is_contributor)
	}

	#[precompile::public("rewardInfo(address)")]
	#[precompile::public("reward_info(address)")]
	#[precompile::view]
	fn reward_info(
		handle: &mut impl PrecompileHandle,
		contributor: Address,
	) -> EvmResult<(U256, U256)> {
		// AccountsPayable: Blake2128(16) + 20 + RewardInfo(16 + 16 + UnBoundedVec<AccountId32(32)>)
		// TODO RewardInfo.contributed_relay_addresses is unbounded, we set a safe length of 100.
		handle.record_db_read::<Runtime>(3268)?;

		let contributor: H160 = contributor.into();

		let account = Runtime::AddressMapping::into_account_id(contributor);

		log::trace!(
			target: "crowdloan-rewards-precompile",
			"Checking reward info for {:?}",
			contributor
		);

		// fetch data from pallet
		let reward_info = pallet_crowdloan_rewards::Pallet::<Runtime>::accounts_payable(account);

		let (total, claimed): (U256, U256) = if let Some(reward_info) = reward_info {
			let total_reward: u128 = reward_info
				.total_reward
				.try_into()
				.map_err(|_| RevertReason::value_is_too_large("balance type"))?;
			let claimed_reward: u128 = reward_info
				.claimed_reward
				.try_into()
				.map_err(|_| RevertReason::value_is_too_large("balance type"))?;

			(total_reward.into(), claimed_reward.into())
		} else {
			(0u128.into(), 0u128.into())
		};

		log::trace!(
			target: "crowldoan-rewards-precompile", "Result from pallet is {:?}  {:?}",
			total, claimed
		);

		Ok((total, claimed))
	}

	#[precompile::public("claim()")]
	fn claim(handle: &mut impl PrecompileHandle) -> EvmResult {
		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call = pallet_crowdloan_rewards::Call::<Runtime>::claim {};

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}

	#[precompile::public("updateRewardAddress(address)")]
	#[precompile::public("update_reward_address(address)")]
	fn update_reward_address(
		handle: &mut impl PrecompileHandle,
		new_address: Address,
	) -> EvmResult {
		log::trace!(
			target: "crowdloan-rewards-precompile",
			"In update_reward_address dispatchable wrapper"
		);

		let new_address: H160 = new_address.into();

		let new_reward_account = Runtime::AddressMapping::into_account_id(new_address);

		log::trace!(target: "crowdloan-rewards-precompile", "New account is {:?}", new_address);

		let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
		let call =
			pallet_crowdloan_rewards::Call::<Runtime>::update_reward_address { new_reward_account };

		RuntimeHelper::<Runtime>::try_dispatch(handle, Some(origin).into(), call, 0)?;

		Ok(())
	}
}
