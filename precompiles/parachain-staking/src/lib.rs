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
use frame_support::traits::{Currency, Get};
use pallet_evm::AddressMapping;
use pallet_evm::GasWeightMapping;
use pallet_evm::Precompile;
use sp_core::{H160, U256};
use sp_std::convert::{TryFrom, TryInto};
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;
use sp_std::vec::Vec;

type BalanceOf<Runtime> = <<Runtime as parachain_staking::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

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
	Runtime::AccountId: From<H160>,
	BalanceOf<Runtime>: TryFrom<U256> + Debug,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<parachain_staking::Call<Runtime>>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		log::trace!(target: "staking-precompile", "In parachain staking wrapper");

		// Basic sanity checking for length
		// https://solidity-by-example.org/primitives/

		const SELECTOR_SIZE_BYTES: usize = 4;

		if input.len() < 4 {
			return Err(ExitError::Other("input length less than 4 bytes".into()));
		}

		log::trace!(target: "staking-precompile", "Made it past preliminary length check");
		log::trace!(target: "staking-precompile", "context.caller is {:?}", context.caller);

		// Parse the function selector
		// These are the four-byte function selectors calculated from the StakingInterface.sol
		// according to the solidity specification
		// https://docs.soliditylang.org/en/v0.8.0/abi-spec.html#function-selector
		let inner_call = match input[0..SELECTOR_SIZE_BYTES] {
			// Check for accessor methods first. These return results immediately
			[0x8e, 0x50, 0x80, 0xe7] => {
				return Self::is_nominator(&input[SELECTOR_SIZE_BYTES..]);
			}
			[0x85, 0x45, 0xc8, 0x33] => {
				return Self::is_candidate(&input[SELECTOR_SIZE_BYTES..]);
			}
			[0x8f, 0x6d, 0x27, 0xc7] => {
				return Self::is_selected_candidate(&input[SELECTOR_SIZE_BYTES..]);
			}
			[0xc9, 0xf5, 0x93, 0xb2] => {
				return Self::min_nomination();
			}
			[0x97, 0x99, 0xb4, 0xe7] => {
				return Self::points(&input[SELECTOR_SIZE_BYTES..]);
			}
			[0x4b, 0x1c, 0x4c, 0x29] => {
				return Self::candidate_count();
			}
			[0x0a, 0xd6, 0xa7, 0xbe] => {
				return Self::collator_nomination_count(&input[SELECTOR_SIZE_BYTES..]);
			}
			[0xda, 0xe5, 0x65, 0x9b] => {
				return Self::nominator_nomination_count(&input[SELECTOR_SIZE_BYTES..]);
			}

			// If not an accessor, check for dispatchables. These calls ready for dispatch below.
			[0x0a, 0x1b, 0xff, 0x60] => Self::join_candidates(&input[SELECTOR_SIZE_BYTES..])?,
			[0x72, 0xb0, 0x2a, 0x31] => Self::leave_candidates(&input[SELECTOR_SIZE_BYTES..])?,
			[0x76, 0x7e, 0x04, 0x50] => Self::go_offline()?,
			[0xd2, 0xf7, 0x3c, 0xeb] => Self::go_online()?,
			[0x28, 0x9b, 0x6b, 0xa7] => Self::candidate_bond_less(&input[SELECTOR_SIZE_BYTES..])?,
			[0xc5, 0x7b, 0xd3, 0xa8] => Self::candidate_bond_more(&input[SELECTOR_SIZE_BYTES..])?,
			[0x49, 0xdf, 0x6e, 0xb3] => Self::nominate(&input[SELECTOR_SIZE_BYTES..])?,
			[0xb7, 0x1d, 0x21, 0x53] => Self::leave_nominators(&input[SELECTOR_SIZE_BYTES..])?,
			[0x4b, 0x65, 0xc3, 0x4b] => Self::revoke_nomination(&input[SELECTOR_SIZE_BYTES..])?,
			[0xf6, 0xa5, 0x25, 0x69] => Self::nominator_bond_less(&input[SELECTOR_SIZE_BYTES..])?,
			[0x97, 0x1d, 0x44, 0xc8] => Self::nominator_bond_more(&input[SELECTOR_SIZE_BYTES..])?,
			_ => {
				log::trace!(
					target: "staking-precompile",
					"Failed to match function selector in staking wrapper precompile"
				);
				return Err(ExitError::Other(
					"No staking wrapper method at selector given selector".into(),
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
		log::trace!(target: "staking-precompile", "Made it past gas check");

		// Dispatch that call
		let origin = Runtime::AddressMapping::into_account_id(context.caller);

		log::trace!(target: "staking-precompile", "Gonna call with origin {:?}", origin);

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
					target: "staking-precompile",
					"Parachain staking call via evm failed {:?}",
					e
				);
				Err(ExitError::Other(
					"Parachain staking call via EVM failed".into(),
				))
			}
		}
	}
}

/// Parses an H160 account address from a 256 bit (32 byte) buffer. Only the last 20 bytes are used.
fn parse_account(input: &[u8]) -> Result<H160, ExitError> {
	const PADDING_SIZE_BYTES: usize = 12;
	const ACCOUNT_SIZE_BYTES: usize = 20;
	const TOTAL_SIZE_BYTES: usize = PADDING_SIZE_BYTES + ACCOUNT_SIZE_BYTES;

	if input.len() != TOTAL_SIZE_BYTES {
		log::trace!(target: "staking-precompile",
			"Unable to parse address. Got {} bytes, expected {}",
			input.len(),
			TOTAL_SIZE_BYTES,
		);
		return Err(ExitError::Other(
			"Incorrect input length for account parsing".into(),
		));
	}

	Ok(H160::from_slice(
		&input[PADDING_SIZE_BYTES..TOTAL_SIZE_BYTES],
	))
}

/// Parses an amount of ether from a 256 bit (32 byte) slice. The balance type is generic.
fn parse_amount<Balance: TryFrom<U256>>(input: &[u8]) -> Result<Balance, ExitError> {
	Ok(parse_uint256(input)?
		.try_into()
		.map_err(|_| ExitError::Other("Amount is too large for provided balance type".into()))?)
}

/// Parses a uint256 value
fn parse_uint256(input: &[u8]) -> Result<U256, ExitError> {
	// In solidity all values are encoded to this width
	const SIZE_BYTES: usize = 32;

	if input.len() != SIZE_BYTES {
		log::trace!(target: "staking-precompile",
			"Unable to parse uint256. Got {} bytes, expected {}",
			input.len(),
			SIZE_BYTES,
		);
		return Err(ExitError::Other(
			"Incorrect input length for uint256 parsing".into(),
		));
	}

	Ok(U256::from_big_endian(&input[0..SIZE_BYTES]))
}

/// Parses Weight Hint: u32 from a 256 bit (32 byte) slice.
fn parse_weight_hint(input: &[u8]) -> Result<u32, ExitError> {
	const WEIGHT_HINT_SIZE_BYTES: usize = 32;

	if input.len() != WEIGHT_HINT_SIZE_BYTES {
		log::trace!(target: "staking-precompile",
			"Unable to parse weight hint. Got {} bytes, expected {}",
			input.len(),
			WEIGHT_HINT_SIZE_BYTES,
		);
		return Err(ExitError::Other(
			"Incorrect input length for weight hint parsing".into(),
		));
	}

	let weight_hint: u32 = U256::from_big_endian(&input[0..WEIGHT_HINT_SIZE_BYTES])
		.try_into()
		.map_err(|_| ExitError::Other("Weight hint is too large for u32".into()))?;
	Ok(weight_hint)
}

impl<Runtime> ParachainStakingWrapper<Runtime>
where
	Runtime: parachain_staking::Config + pallet_evm::Config + frame_system::Config,
	Runtime::AccountId: From<H160>,
	BalanceOf<Runtime>: TryFrom<U256> + TryInto<u128> + Debug,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Runtime::Call: From<parachain_staking::Call<Runtime>>,
{
	// The accessors are first. They directly return their result.

	fn is_nominator(input: &[u8]) -> Result<PrecompileOutput, ExitError> {
		// parse the address
		let nominator = H160::from_slice(&input[12..32]);

		log::trace!(
			target: "staking-precompile",
			"Checking whether {:?} is a nominator",
			nominator
		);

		// fetch data from pallet
		let is_nominator = parachain_staking::Pallet::<Runtime>::is_nominator(&nominator.into());

		log::trace!(target: "staking-precompile", "Result from pallet is {:?}", is_nominator);

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: bool_to_solidity_bytes(is_nominator),
			logs: Default::default(),
		})
	}

	fn is_candidate(input: &[u8]) -> Result<PrecompileOutput, ExitError> {
		// parse the address
		let candidate = H160::from_slice(&input[12..32]);

		log::trace!(
			target: "staking-precompile",
			"Checking whether {:?} is a collator candidate",
			candidate
		);

		// fetch data from pallet
		let is_candidate = parachain_staking::Pallet::<Runtime>::is_candidate(&candidate.into());

		log::trace!(target: "staking-precompile", "Result from pallet is {:?}", is_candidate);

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: bool_to_solidity_bytes(is_candidate),
			logs: Default::default(),
		})
	}

	fn is_selected_candidate(input: &[u8]) -> Result<PrecompileOutput, ExitError> {
		// parse the address
		let candidate = H160::from_slice(&input[12..32]);

		log::trace!(
			target: "staking-precompile",
			"Checking whether {:?} is a selected collator",
			candidate
		);

		// fetch data from pallet
		let is_selected =
			parachain_staking::Pallet::<Runtime>::is_selected_candidate(&candidate.into());

		log::trace!(target: "staking-precompile", "Result from pallet is {:?}", is_selected);

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: bool_to_solidity_bytes(is_selected),
			logs: Default::default(),
		})
	}

	fn min_nomination() -> Result<PrecompileOutput, ExitError> {
		// fetch data from pallet
		let raw_min_nomination: u128 = <
			<Runtime as parachain_staking::Config>::MinNomination
				as Get<BalanceOf<Runtime>>
			>::get().try_into()
				.map_err(|_|
					ExitError::Other("Amount is too large for provided balance type".into())
				)?;
		let min_nomination: U256 = raw_min_nomination.into();

		log::trace!(target: "staking-precompile", "Result from pallet is {:?}", min_nomination);

		// TODO find cost of Config associated type read
		// For now assume it is as bad as a storage read in the worst case
		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		let mut buffer = [0u8; 32];
		min_nomination.to_big_endian(&mut buffer);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: buffer.to_vec(),
			logs: Default::default(),
		})
	}

	fn points(input: &[u8]) -> Result<PrecompileOutput, ExitError> {
		let round_u256 = parse_uint256(input)?;

		// Make sure the round number fits in a u32
		if round_u256.leading_zeros() < 256 - 32 {
			return Err(ExitError::Other(
				"Round is too large. 32 bit maximum".into(),
			));
		}
		let round: u32 = round_u256.low_u32();

		log::trace!(target: "staking-precompile", "🥩round is {}", round);
		// Read the point value and format it for Solidity
		let points: u32 = parachain_staking::Pallet::<Runtime>::points(round);
		log::trace!(target: "staking-precompile", "🥩points is {}", points);
		let mut output = [0u8; 32];
		U256::from(points).to_big_endian(&mut output);
		log::trace!(target: "staking-precompile", "🥩output is {:?}", output);

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output.to_vec(),
			logs: Default::default(),
		})
	}

	fn candidate_count() -> Result<PrecompileOutput, ExitError> {
		// fetch data from pallet
		let raw_candidate_count: u32 = <parachain_staking::Pallet<Runtime>>::candidate_pool()
			.0
			.len() as u32;
		let candidate_count: U256 = raw_candidate_count.into();

		log::trace!(target: "staking-precompile", "Result from pallet is {:?}", candidate_count);

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		let mut buffer = [0u8; 32];
		candidate_count.to_big_endian(&mut buffer);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: buffer.to_vec(),
			logs: Default::default(),
		})
	}

	fn collator_nomination_count(input: &[u8]) -> Result<PrecompileOutput, ExitError> {
		// TODO: check length and same for all of the others, can panic if len < 32
		let collator: Runtime::AccountId = parse_account(&input[..32])?.into();
		let mut buffer = [0u8; 32];
		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);
		if let Some(state) = <parachain_staking::Pallet<Runtime>>::collator_state2(&collator) {
			let raw_collator_nomination_count: u32 = state.nominators.0.len() as u32;
			let collator_nomination_count: U256 = raw_collator_nomination_count.into();

			log::trace!(
				target: "staking-precompile",
				"Result from pallet is {:?}",
				raw_collator_nomination_count
			);

			collator_nomination_count.to_big_endian(&mut buffer);
		} else {
			log::trace!(
				target: "staking-precompile",
				"Collator {:?} not found, so nomination count is 0",
				collator
			);
		}

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: buffer.to_vec(),
			logs: Default::default(),
		})
	}

	fn nominator_nomination_count(input: &[u8]) -> Result<PrecompileOutput, ExitError> {
		let nominator: Runtime::AccountId = parse_account(&input[..32])?.into();
		let mut buffer = [0u8; 32];
		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);
		if let Some(state) = <parachain_staking::Pallet<Runtime>>::nominator_state(&nominator) {
			let raw_nominator_nomination_count: u32 = state.nominations.0.len() as u32;
			let nominator_nomination_count: U256 = raw_nominator_nomination_count.into();

			log::trace!(
				target: "staking-precompile",
				"Result from pallet is {:?}",
				raw_nominator_nomination_count
			);

			nominator_nomination_count.to_big_endian(&mut buffer);
		} else {
			log::trace!(
				target: "staking-precompile",
				"Nominator {:?} not found, so nomination count is 0",
				nominator
			);
		}

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: buffer.to_vec(),
			logs: Default::default(),
		})
	}

	// The dispatchable wrappers are next. They return a substrate inner Call ready for dispatch.

	fn join_candidates(input: &[u8]) -> Result<parachain_staking::Call<Runtime>, ExitError> {
		let amount = parse_amount::<BalanceOf<Runtime>>(&input[..32])?;
		let collator_candidate_count = parse_weight_hint(&input[32..])?;

		log::trace!(target: "staking-precompile", "Collator stake amount is {:?}", amount);
		log::trace!(
			target: "staking-precompile",
			"Weight Hint: collator candidate count is {:?}",
			collator_candidate_count
		);

		Ok(parachain_staking::Call::<Runtime>::join_candidates(
			amount,
			collator_candidate_count,
		))
	}

	fn leave_candidates(input: &[u8]) -> Result<parachain_staking::Call<Runtime>, ExitError> {
		let collator_candidate_count = parse_weight_hint(input)?;
		Ok(parachain_staking::Call::<Runtime>::leave_candidates(
			collator_candidate_count,
		))
	}

	fn go_offline() -> Result<parachain_staking::Call<Runtime>, ExitError> {
		Ok(parachain_staking::Call::<Runtime>::go_offline())
	}

	fn go_online() -> Result<parachain_staking::Call<Runtime>, ExitError> {
		Ok(parachain_staking::Call::<Runtime>::go_online())
	}

	fn candidate_bond_more(input: &[u8]) -> Result<parachain_staking::Call<Runtime>, ExitError> {
		let amount = parse_amount::<BalanceOf<Runtime>>(input)?;

		log::trace!(target: "staking-precompile", "Collator bond increment is {:?}", amount);

		Ok(parachain_staking::Call::<Runtime>::candidate_bond_more(
			amount,
		))
	}

	fn candidate_bond_less(input: &[u8]) -> Result<parachain_staking::Call<Runtime>, ExitError> {
		let amount = parse_amount::<BalanceOf<Runtime>>(input)?;

		log::trace!(target: "staking-precompile", "Collator bond decrement is {:?}", amount);

		Ok(parachain_staking::Call::<Runtime>::candidate_bond_less(
			amount,
		))
	}

	fn nominate(input: &[u8]) -> Result<parachain_staking::Call<Runtime>, ExitError> {
		log::trace!(target: "staking-precompile", "In nominate dispatchable wrapper");
		log::trace!(target: "staking-precompile", "input is {:?}", input);
		let collator = parse_account(&input[..32])?;
		let amount = parse_amount::<BalanceOf<Runtime>>(&input[32..64])?;
		let collator_nomination_count = parse_weight_hint(&input[64..96])?;
		let nominator_nomination_count = parse_weight_hint(&input[96..])?;

		log::trace!(target: "staking-precompile", "Collator account is {:?}", collator);
		log::trace!(target: "staking-precompile", "Nomination amount is {:?}", amount);
		log::trace!(
			target: "staking-precompile",
			"Weight Hint: collator nominations count is {:?}",
			collator_nomination_count
		);
		log::trace!(
			target: "staking-precompile",
			"Weight Hint: nominator nominations count is {:?}",
			nominator_nomination_count
		);

		Ok(parachain_staking::Call::<Runtime>::nominate(
			collator.into(),
			amount,
			collator_nomination_count,
			nominator_nomination_count,
		))
	}

	fn leave_nominators(input: &[u8]) -> Result<parachain_staking::Call<Runtime>, ExitError> {
		let nomination_count = parse_weight_hint(&input[..])?;
		Ok(parachain_staking::Call::<Runtime>::leave_nominators(
			nomination_count,
		))
	}

	fn revoke_nomination(input: &[u8]) -> Result<parachain_staking::Call<Runtime>, ExitError> {
		log::trace!(target: "staking-precompile", "In revoke nomination dispatchable wrapper");
		let collator = parse_account(&input[..32])?;

		log::trace!(target: "staking-precompile", "Collator account is {:?}", collator);

		Ok(parachain_staking::Call::<Runtime>::revoke_nomination(
			collator.into(),
		))
	}

	fn nominator_bond_more(input: &[u8]) -> Result<parachain_staking::Call<Runtime>, ExitError> {
		let collator = parse_account(&input[..32])?;
		let amount = parse_amount::<BalanceOf<Runtime>>(&input[32..])?;

		log::trace!(target: "staking-precompile", "Collator account is {:?}", collator);
		log::trace!(target: "staking-precompile", "Nomination increment is {:?}", amount);

		Ok(parachain_staking::Call::<Runtime>::nominator_bond_more(
			collator.into(),
			amount,
		))
	}

	fn nominator_bond_less(input: &[u8]) -> Result<parachain_staking::Call<Runtime>, ExitError> {
		let collator = parse_account(&input[..32])?;
		let amount = parse_amount::<BalanceOf<Runtime>>(&input[32..])?;

		log::trace!(target: "staking-precompile", "Collator account is {:?}", collator);
		log::trace!(target: "staking-precompile", "Nomination decrement is {:?}", amount);

		Ok(parachain_staking::Call::<Runtime>::nominator_bond_less(
			collator.into(),
			amount,
		))
	}
}

// Solidity's bool type is 256 bits as shown by these examples
// https://docs.soliditylang.org/en/v0.8.0/abi-spec.html
// This utility function converts a Rust bool into the corresponding Solidity type
fn bool_to_solidity_bytes(b: bool) -> Vec<u8> {
	let mut result_bytes = [0u8; 32];

	if b {
		result_bytes[31] = 1;
	}

	result_bytes.to_vec()
}
