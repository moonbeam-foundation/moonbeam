// Copyright 2019-2021 PureStake Inc.
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
#![feature(assert_matches)]

use fp_evm::{Context, ExitSucceed, PrecompileOutput};
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::Currency,
};
use pallet_evm::{AddressMapping, Precompile};
use precompile_utils::{
	Address, EvmDataReader, EvmDataWriter, EvmResult, Gasometer, RuntimeHelper,
};

use sp_core::{H160, U256};
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

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	IsContributor = "is_contributor(address)",
	RewardInfo = "reward_info(address)",
	Claim = "claim()",
	UpdateRewardAddress = "update_reward_address(address)",
}

/// A precompile to wrap the functionality from pallet_crowdloan_rewards.
pub struct CrowdloanRewardsWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for CrowdloanRewardsWrapper<Runtime>
where
	Runtime: pallet_crowdloan_rewards::Config + pallet_evm::Config,
	BalanceOf<Runtime>: TryFrom<U256> + Debug,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_crowdloan_rewards::Call<Runtime>>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
		_is_static: bool,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		let (mut input, selector) = EvmDataReader::new_with_selector(&mut gasometer, input)?;

		match selector {
			// Check for accessor methods first. These return results immediately
			Action::IsContributor => Self::is_contributor(&mut input, &mut gasometer),
			Action::RewardInfo => Self::reward_info(&mut input, &mut gasometer),
			Action::Claim => Self::claim(&mut gasometer, context),
			Action::UpdateRewardAddress => {
				Self::update_reward_address(&mut input, &mut gasometer, context)
			}
		}
	}
}

impl<Runtime> CrowdloanRewardsWrapper<Runtime>
where
	Runtime: pallet_crowdloan_rewards::Config + pallet_evm::Config + frame_system::Config,
	BalanceOf<Runtime>: TryFrom<U256> + TryInto<u128> + Debug,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_crowdloan_rewards::Call<Runtime>>,
{
	// The accessors are first. They directly return their result.
	fn is_contributor(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?; // accounts_payable

		// Bound check
		input.expect_arguments(gasometer, 1)?;

		// parse the address
		let contributor: H160 = input.read::<Address>(gasometer)?.into();

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

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(is_contributor).build(),
			logs: Default::default(),
		})
	}

	fn reward_info(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?; // accounts_payable

		// Bound check
		input.expect_arguments(gasometer, 1)?;

		// parse the address
		let contributor: H160 = input.read::<Address>(gasometer)?.into();

		let account = Runtime::AddressMapping::into_account_id(contributor);

		log::trace!(
			target: "crowdloan-rewards-precompile",
			"Checking reward info for {:?}",
			contributor
		);

		// fetch data from pallet
		let reward_info = pallet_crowdloan_rewards::Pallet::<Runtime>::accounts_payable(account);

		let (total, claimed): (U256, U256) =
			if let Some(reward_info) = reward_info {
				let total_reward: u128 = reward_info.total_reward.try_into().map_err(|_| {
					gasometer.revert("Amount is too large for provided balance type")
				})?;
				let claimed_reward: u128 = reward_info.claimed_reward.try_into().map_err(|_| {
					gasometer.revert("Amount is too large for provided balance type")
				})?;

				(total_reward.into(), claimed_reward.into())
			} else {
				(0u128.into(), 0u128.into())
			};

		log::trace!(
			target: "crowldoan-rewards-precompile", "Result from pallet is {:?}  {:?}",
			total, claimed
		);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(total).write(claimed).build(),
			logs: Default::default(),
		})
	}

	fn claim(gasometer: &mut Gasometer, context: &Context) -> EvmResult<PrecompileOutput> {
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = pallet_crowdloan_rewards::Call::<Runtime>::claim {};

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}

	fn update_reward_address(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		log::trace!(
			target: "crowdloan-rewards-precompile",
			"In update_reward_address dispatchable wrapper"
		);

		// Bound check
		input.expect_arguments(gasometer, 1)?;

		// parse the address
		let new_address: H160 = input.read::<Address>(gasometer)?.into();

		let new_reward_account = Runtime::AddressMapping::into_account_id(new_address);

		log::trace!(target: "crowdloan-rewards-precompile", "New account is {:?}", new_address);

		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call =
			pallet_crowdloan_rewards::Call::<Runtime>::update_reward_address { new_reward_account };

		RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, gasometer)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Stopped,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: Default::default(),
		})
	}
}
