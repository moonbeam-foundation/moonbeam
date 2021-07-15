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

//! Precompile to call parachain-staking runtime methods via the EVM

#![cfg_attr(not(feature = "std"), no_std)]

use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::Currency;
use pallet_evm::AddressMapping;
use pallet_evm::GasWeightMapping;
use pallet_evm::Precompile;
use sp_core::{H160, U256};
use sp_std::convert::{TryFrom, TryInto};
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;

type BalanceOf<Runtime> = <<Runtime as pallet_crowdloan_rewards::Config>::RewardCurrency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

/// A precompile to wrap the functionality from pallet_crowdloan_rewards.
///
/// EXAMPLE USECASE:
/// A simple example usecase is a contract that allows donors to donate, and stakes all the funds
/// toward one fixed address chosen by the deployer.
/// Such a contract could be deployed by a collator candidate, and the deploy address distributed to
/// supporters who want to donate toward a perpetual nomination fund.
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
		log::trace!(target: "crowdloan-rewards-precompile", "In crowdloan rewards wrapper");

		// Basic sanity checking for length
		// https://solidity-by-example.org/primitives/

		const SELECTOR_SIZE_BYTES: usize = 4;

		if input.len() < 4 {
			return Err(ExitError::Other("input length less than 4 bytes".into()));
		}

		log::trace!(target: "crowdloan-rewards-precompile", "Made it past preliminary length check");
		log::trace!(target: "crowdloan-rewards-precompile", "context.caller is {:?}", context.caller);

		// Parse the function selector
		// These are the four-byte function selectors calculated from the StakingInterface.sol
		// according to the solidity specification
		// https://docs.soliditylang.org/en/v0.8.0/abi-spec.html#function-selector
		let inner_call = match input[0..SELECTOR_SIZE_BYTES] {
			// Check for accessor methods first. These return results immediately
	/* 		[0x8e, 0x50, 0x80, 0xe7] => {
				return Self::is_nominator(&input[SELECTOR_SIZE_BYTES..]);
			}
			[0x85, 0x45, 0xc8, 0x33] => {
				return Self::is_candidate(&input[SELECTOR_SIZE_BYTES..]);
			}
			[0x8f, 0x6d, 0x27, 0xc7] => {
				return Self::is_selected_candidate(&input[SELECTOR_SIZE_BYTES..]);
			}
			[0xc9, 0xf5, 0x93, 0xb2] => {
				//TODO Do we need to verify that there were no additional bytes passed in here?
				return Self::min_nomination();
			}
			[0x97, 0x99, 0xb4, 0xe7] => {
				return Self::points(&input[SELECTOR_SIZE_BYTES..]);
			}*/

			// If not an accessor, check for dispatchables. These calls ready for dispatch below.
			[0x4e, 0x71, 0xd9, 0x2d] =>  Self::claim()?,
			_ => {
				log::trace!(
					target: "crowdloan-rewards-precompile",
					"Failed to match function selector in crowdloan rewards precompile"
				);
				return Err(ExitError::Other(
					"No crowdloan rewards wrapper method at selector given selector".into(),
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

	fn claim() -> Result<pallet_crowdloan_rewards::Call<Runtime>, ExitError> {
		Ok(pallet_crowdloan_rewards::Call::<Runtime>::claim())
	}
}