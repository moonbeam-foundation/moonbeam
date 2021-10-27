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

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::{Currency, Get};
use pallet_evm::AddressMapping;
use pallet_evm::Precompile;
use precompile_utils::{
	error, Address, EvmData, EvmDataReader, EvmDataWriter, Gasometer, RuntimeHelper,
};
use sp_std::convert::TryInto;
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;
use sp_std::vec;

type BalanceOf<Runtime> = <<Runtime as parachain_staking::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, num_enum::TryFromPrimitive)]
enum Action {
	MinNomination = "min_nomination()",
	Points = "points(uint256)",
	CandidateCount = "candidate_count()",
	CollatorNominationCount = "collator_nomination_count(address)",
	NominatorNominationCount = "nominator_nomination_count(address)",
	IsNominator = "is_nominator(address)",
	IsCandidate = "is_candidate(address)",
	IsSelectedCandidate = "is_selected_candidate(address)",
	JoinCandidates = "join_candidates(uint256,uint256)",
	LeaveCandidates = "leave_candidates(uint256)",
	GoOffline = "go_offline()",
	GoOnline = "go_online()",
	CandidateBondLess = "candidate_bond_less(uint256)",
	CandidateBondMore = "candidate_bond_more(uint256)",
	Nominate = "nominate(address,uint256,uint256,uint256)",
	LeaveNominators = "leave_nominators(uint256)",
	RevokeNomination = "revoke_nomination(address)",
	NominatorBondLess = "nominator_bond_less(address,uint256)",
	NominatorBondMore = "nominator_bond_more(address,uint256)",
}

/// A precompile to wrap the functionality from parachain_staking.
///
/// EXAMPLE USECASE:
/// A simple example usecase is a contract that allows donors to donate, and stakes all the funds
/// toward one fixed address chosen by the deployer.
/// Such a contract could be deployed by a collator candidate, and the deploy address distributed to
/// supporters who want to donate toward a perpetual nomination fund.
pub struct ParachainStakingWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for ParachainStakingWrapper<Runtime>
where
	Runtime: parachain_staking::Config + pallet_evm::Config,
	BalanceOf<Runtime>: EvmData,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<parachain_staking::Call<Runtime>>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let (input, selector) = EvmDataReader::new_with_selector(input)?;

		// Return early if storage getter; return (origin, call) if dispatchable
		let (origin, call) = match selector {
			// constants
			Action::MinNomination => return Self::min_nomination(target_gas),
			// storage getters
			Action::Points => return Self::points(input, target_gas),
			Action::CandidateCount => return Self::candidate_count(target_gas),
			Action::CollatorNominationCount => {
				return Self::collator_nomination_count(input, target_gas)
			}
			Action::NominatorNominationCount => {
				return Self::nominator_nomination_count(input, target_gas)
			}
			// role verifiers
			Action::IsNominator => return Self::is_nominator(input, target_gas),
			Action::IsCandidate => return Self::is_candidate(input, target_gas),
			Action::IsSelectedCandidate => return Self::is_selected_candidate(input, target_gas),
			// runtime methods (dispatchables)
			Action::JoinCandidates => Self::join_candidates(input, context)?,
			Action::LeaveCandidates => Self::leave_candidates(input, context)?,
			Action::GoOffline => Self::go_offline(context)?,
			Action::GoOnline => Self::go_online(context)?,
			Action::CandidateBondLess => Self::candidate_bond_less(input, context)?,
			Action::CandidateBondMore => Self::candidate_bond_more(input, context)?,
			Action::Nominate => Self::nominate(input, context)?,
			Action::LeaveNominators => Self::leave_nominators(input, context)?,
			Action::RevokeNomination => Self::revoke_nomination(input, context)?,
			Action::NominatorBondLess => Self::nominator_bond_less(input, context)?,
			Action::NominatorBondMore => Self::nominator_bond_more(input, context)?,
		};
		// Initialize gasometer
		let mut gasometer = Gasometer::new(target_gas);
		// Dispatch call (if enough gas).
		let used_gas =
			RuntimeHelper::<Runtime>::try_dispatch(origin, call, gasometer.remaining_gas()?)?;
		gasometer.record_cost(used_gas)?;
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
			logs: vec![],
		})
	}
}

impl<Runtime> ParachainStakingWrapper<Runtime>
where
	Runtime: parachain_staking::Config + pallet_evm::Config,
	BalanceOf<Runtime>: EvmData,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<parachain_staking::Call<Runtime>>,
{
	// Constants

	fn min_nomination(target_gas: Option<u64>) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let min_nomination: u128 = <<Runtime as parachain_staking::Config>::MinNomination as Get<
			BalanceOf<Runtime>,
		>>::get()
		.try_into()
		.map_err(|_| error("Amount is too large for provided balance type"))?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(min_nomination).build(),
			logs: vec![],
		})
	}

	// Storage Getters

	fn points(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;
		let round = input.read::<u32>()?;

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let points: u32 = parachain_staking::Pallet::<Runtime>::points(round);

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(points).build(),
			logs: vec![],
		})
	}

	fn candidate_count(target_gas: Option<u64>) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let candidate_count: u32 = <parachain_staking::Pallet<Runtime>>::candidate_pool()
			.0
			.len() as u32;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(candidate_count).build(),
			logs: vec![],
		})
	}

	fn collator_nomination_count(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;
		let address = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let result =
			if let Some(state) = <parachain_staking::Pallet<Runtime>>::collator_state2(&address) {
				let collator_nomination_count: u32 = state.nominators.0.len() as u32;

				log::trace!(
					target: "staking-precompile",
					"Result from pallet is {:?}",
					collator_nomination_count
				);
				collator_nomination_count
			} else {
				log::trace!(
					target: "staking-precompile",
					"Collator {:?} not found, so nomination count is 0",
					address
				);
				0u32
			};

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(result).build(),
			logs: vec![],
		})
	}

	fn nominator_nomination_count(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;
		let address = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let result =
			if let Some(state) = <parachain_staking::Pallet<Runtime>>::nominator_state2(&address) {
				let nominator_nomination_count: u32 = state.nominations.0.len() as u32;

				log::trace!(
					target: "staking-precompile",
					"Result from pallet is {:?}",
					nominator_nomination_count
				);

				nominator_nomination_count
			} else {
				log::trace!(
					target: "staking-precompile",
					"Nominator {:?} not found, so nomination count is 0",
					address
				);
				0u32
			};

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(result).build(),
			logs: vec![],
		})
	}

	// Role Verifiers

	fn is_nominator(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;
		let address = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_nominator = parachain_staking::Pallet::<Runtime>::is_nominator(&address);

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(is_nominator).build(),
			logs: vec![],
		})
	}

	fn is_candidate(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;
		let address = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_candidate = parachain_staking::Pallet::<Runtime>::is_candidate(&address);

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(is_candidate).build(),
			logs: vec![],
		})
	}

	fn is_selected_candidate(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);

		// Read input.
		input.expect_arguments(1)?;
		let address = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Fetch info.
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let is_selected = parachain_staking::Pallet::<Runtime>::is_selected_candidate(&address);

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(is_selected).build(),
			logs: vec![],
		})
	}

	// Runtime Methods (dispatchables)

	fn join_candidates(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(2)?;
		let bond: BalanceOf<Runtime> = input.read()?;
		let candidate_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::join_candidates {
			bond,
			candidate_count,
		};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn leave_candidates(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(1)?;
		let candidate_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::leave_candidates { candidate_count };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn go_offline(
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::go_offline {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn go_online(
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::go_online {};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn candidate_bond_more(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(1)?;
		let more: BalanceOf<Runtime> = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::candidate_bond_more { more };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn candidate_bond_less(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(1)?;
		let less: BalanceOf<Runtime> = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::candidate_bond_less { less };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn nominate(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(4)?;
		let collator = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let amount: BalanceOf<Runtime> = input.read()?;
		let collator_nominator_count = input.read()?;
		let nomination_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::nominate {
			collator,
			amount,
			collator_nominator_count,
			nomination_count,
		};

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn leave_nominators(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(1)?;
		let nomination_count = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::leave_nominators { nomination_count };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn revoke_nomination(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(1)?;
		let collator = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::revoke_nomination { collator };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn nominator_bond_more(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(2)?;
		let candidate = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let more: BalanceOf<Runtime> = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::nominator_bond_more { candidate, more };

		// Return call information
		Ok((Some(origin).into(), call))
	}

	fn nominator_bond_less(
		mut input: EvmDataReader,
		context: &Context,
	) -> Result<
		(
			<Runtime::Call as Dispatchable>::Origin,
			parachain_staking::Call<Runtime>,
		),
		ExitError,
	> {
		// Read input.
		input.expect_arguments(2)?;
		let candidate = Runtime::AddressMapping::into_account_id(input.read::<Address>()?.0);
		let less: BalanceOf<Runtime> = input.read()?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = parachain_staking::Call::<Runtime>::nominator_bond_less { candidate, less };

		// Return call information
		Ok((Some(origin).into(), call))
	}
}
