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

use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::{Currency, Get},
};
use pallet_evm::{AddressMapping, GasWeightMapping, Precompile};
use precompile_utils::{error, InputReader, OutputBuilder};

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

type BalanceOf<Runtime> =
	<<Runtime as pallet_crowdloan_rewards::Config>::RewardCurrency as Currency<
		<Runtime as frame_system::Config>::AccountId,
	>>::Balance;

/// A precompile to wrap the functionality from pallet_crowdloan_rewards.
pub struct CrowdloanRewardsWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for CrowdloanRewardsWrapper<Runtime>
where
	Runtime: pallet_crowdloan_rewards::Config + pallet_evm::Config,
	Runtime::AccountId: From<H160>,
	BalanceOf<Runtime>: TryFrom<U256> + Debug,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_crowdloan_rewards::Call<Runtime>>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let input = InputReader::new(input)?;

		// Parse the function selector
		// These are the four-byte function selectors calculated from the CrowdloanInterface.sol
		// according to the solidity specification
		// https://docs.soliditylang.org/en/v0.8.0/abi-spec.html#function-selector
		let inner_call = match input.selector() {
			// Check for accessor methods first. These return results immediately
			[0x53, 0x44, 0x0c, 0x90] => {
				return Self::is_contributor(input, target_gas);
			}
			[0x76, 0xf7, 0x02, 0x49] => {
				return Self::reward_info(input, target_gas);
			}
			[0x4e, 0x71, 0xd9, 0x2d] => Self::claim()?,

			[0xaa, 0xac, 0x61, 0xd6] => Self::update_reward_address(input)?,
			_ => {
				log::trace!(
					target: "crowdloan-rewards-precompile",
					"Failed to match function selector in crowdloan rewards precompile"
				);
				return Err(error(
					"No crowdloan rewards wrapper method at given selector".into(),
				));
			}
		};

		let outer_call: Runtime::Call = inner_call.into();
		let info = outer_call.get_dispatch_info();

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			let required_gas = Runtime::GasWeightMapping::weight_to_gas(info.weight);
			if required_gas > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}
		log::trace!(target: "crowdloan-rewards-precompile", "Made it past gas check");

		// Dispatch that call
		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		log::trace!(target: "crowdloan-rewards-precompile", "Gonna call with origin {:?}", origin);

		match outer_call.dispatch(Some(origin).into()) {
			Ok(post_info) => {
				let gas_used = Runtime::GasWeightMapping::weight_to_gas(
					post_info.actual_weight.unwrap_or(info.weight),
				);
				Ok(PrecompileOutput {
					exit_status: ExitSucceed::Stopped,
					cost: gas_used,
					output: Default::default(),
					logs: Default::default(),
				})
			}
			Err(e) => {
				log::trace!(
					target: "crowdloan-rewards-precompile",
					"Crowdloan rewards call via evm failed {:?}",
					e
				);
				Err(ExitError::Other(
					"Crowdloan rewards call via EVM failed".into(),
				))
			}
		}
	}
}

impl<Runtime> CrowdloanRewardsWrapper<Runtime>
where
	Runtime: pallet_crowdloan_rewards::Config + pallet_evm::Config + frame_system::Config,
	Runtime::AccountId: From<H160>,
	BalanceOf<Runtime>: TryFrom<U256> + TryInto<u128> + Debug,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<pallet_crowdloan_rewards::Call<Runtime>>,
{
	// The accessors are first. They directly return their result.
	fn is_contributor(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		// Bound check
		input.expect_arguments(1)?;

		// parse the address
		let contributor = input.read_address::<Runtime::AccountId>()?;

		log::trace!(
			target: "crowdloan-rewards-precompile",
			"Checking whether {:?} is a contributor",
			contributor
		);

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			if gas_consumed > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}

		// fetch data from pallet
		let is_contributor =
			pallet_crowdloan_rewards::Pallet::<Runtime>::accounts_payable(contributor).is_some();

		log::trace!(target: "crowldoan-rewards-precompile", "Result from pallet is {:?}", is_contributor);

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: OutputBuilder::new().write_bool(is_contributor).build(),
			logs: Default::default(),
		})
	}

	fn reward_info(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		// Bound check
		input.expect_arguments(1)?;

		// parse the address
		let contributor = input.read_address::<Runtime::AccountId>()?;

		log::trace!(
			target: "crowdloan-rewards-precompile",
			"Checking reward info for {:?}",
			contributor
		);

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			if gas_consumed > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}

		// fetch data from pallet
		let reward_info =
			pallet_crowdloan_rewards::Pallet::<Runtime>::accounts_payable(contributor);

		let (total, claimed): (U256, U256) = if let Some(reward_info) = reward_info {
			let total_reward: u128 = reward_info.total_reward.try_into().map_err(|_| {
				ExitError::Other("Amount is too large for provided balance type".into())
			})?;
			let claimed_reward: u128 = reward_info.claimed_reward.try_into().map_err(|_| {
				ExitError::Other("Amount is too large for provided balance type".into())
			})?;

			(total_reward.into(), claimed_reward.into())
		} else {
			(0u128.into(), 0u128.into())
		};

		log::trace!(
			target: "crowldoan-rewards-precompile", "Result from pallet is {:?}  {:?}",
			total, claimed
		);

		let mut output = OutputBuilder::new().write_u256(total).build();
		output.extend(OutputBuilder::new().write_u256(claimed).build());

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output,
			logs: Default::default(),
		})
	}

	fn claim() -> Result<pallet_crowdloan_rewards::Call<Runtime>, ExitError> {
		Ok(pallet_crowdloan_rewards::Call::<Runtime>::claim())
	}

	fn update_reward_address(
		mut input: InputReader,
	) -> Result<pallet_crowdloan_rewards::Call<Runtime>, ExitError> {
		log::trace!(
			target: "crowdloan-rewards-precompile",
			"In update_reward_address dispatchable wrapper"
		);

		// Bound check
		input.expect_arguments(1)?;

		// parse the address
		let new_address = input.read_address::<Runtime::AccountId>()?;

		log::trace!(target: "crowdloan-rewards-precompile", "New account is {:?}", new_address);

		Ok(pallet_crowdloan_rewards::Call::<Runtime>::update_reward_address(new_address))
	}
}
