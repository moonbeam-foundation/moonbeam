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

use cumulus_primitives_core::relay_chain;
use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	traits::Get,
};
use pallet_evm::{GasWeightMapping, Precompile};
use pallet_staking::RewardDestination;
use precompile_utils::{error, InputReader, OutputBuilder, RuntimeHelper};
use sp_runtime::Perbill;
use sp_std::vec::Vec;
use sp_std::{convert::TryInto, marker::PhantomData};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test_relay_runtime;
#[cfg(test)]
mod tests;
pub enum AvailableStakeCalls {
	Bond(
		relay_chain::AccountId,
		relay_chain::Balance,
		pallet_staking::RewardDestination<relay_chain::AccountId>,
	),
	BondExtra(relay_chain::Balance),
	Unbond(relay_chain::Balance),
	WithdrawUnbonded(u32),
	Validate(pallet_staking::ValidatorPrefs),
	Nominate(Vec<relay_chain::AccountId>),
	Chill,
	SetPayee(pallet_staking::RewardDestination<relay_chain::AccountId>),
	SetController(relay_chain::AccountId),
	Rebond(relay_chain::Balance),
}

pub trait StakeEncodeCall {
	/// Encode call from the relay.
	fn encode_call(call: AvailableStakeCalls) -> Vec<u8>;
}

/// A precompile to wrap the functionality from pallet_crowdloan_rewards.
pub struct RelayEncoderWrapper<Runtime, RelayRuntime>(PhantomData<(Runtime, RelayRuntime)>);

impl<Runtime, RelayRuntime> Precompile for RelayEncoderWrapper<Runtime, RelayRuntime>
where
	RelayRuntime: StakeEncodeCall,
	Runtime: pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
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

		match input.selector() {
			// Check for accessor methods first. These return results immediately
			[0xf4, 0xb9, 0x13, 0xdd] => {
				return Self::encode_bond(input, target_gas);
			}
			[0xea, 0x02, 0x9c, 0xbb] => {
				return Self::encode_bond_more(input, target_gas);
			}
			[0xbc, 0x4b, 0x21, 0x87] => {
				return Self::encode_chill(input, target_gas);
			}
			[0xa7, 0xcb, 0x12, 0x4b] => {
				return Self::encode_nominate(input, target_gas);
			}
			[0xad, 0xd6, 0xb3, 0xbf] => {
				return Self::encode_rebond(input, target_gas);
			}
			[0x7a, 0x8f, 0x48, 0xc2] => {
				return Self::encode_set_controller(input, target_gas);
			}
			[0x7c, 0x85, 0x56, 0x9d] => {
				return Self::encode_set_payee(input, target_gas);
			}
			[0x2c, 0xd6, 0x12, 0x17] => {
				return Self::encode_unbond(input, target_gas);
			}
			[0x3a, 0x0d, 0x80, 0x3a] => {
				return Self::encode_validate(input, target_gas);
			}
			[0x2d, 0x22, 0x03, 0x31] => {
				return Self::encode_withdraw_unbonded(input, target_gas);
			}
			_ => {
				log::trace!(
					target: "relay-encoder-precompile",
					"Failed to match function selector in crowdloan rewards precompile"
				);
				return Err(error("No relay wrapper method at given selector".into()));
			}
		};
	}
}

impl<Runtime, RelayRuntime> RelayEncoderWrapper<Runtime, RelayRuntime>
where
	RelayRuntime: StakeEncodeCall,
	Runtime: pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
{
	// The accessors are first. They directly return their result.
	fn encode_bond(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		input.expect_arguments(4)?;

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			if gas_consumed > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}

		let address = input.read_relay_address()?;
		let amount = parse_relay_amount(&mut input)?;
		let reward_destination = parse_reward_destination(&mut input)?;
		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::Bond(
			address,
			amount,
			reward_destination,
		));

		let gas_consumed = RuntimeHelper::<Runtime>::db_read_gas_cost();

		let mut output = OutputBuilder::new().write_u256(32u32).build();
		output.extend(OutputBuilder::new().write_u256(encoded.len()).build());
		output.extend(OutputBuilder::new().write_bytes(encoded).build());

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output,
			logs: Default::default(),
		})
	}

	fn encode_bond_more(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			if gas_consumed > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}

		input.expect_arguments(1)?;
		let amount = parse_relay_amount(&mut input)?;
		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::BondExtra(amount));

		let mut output = OutputBuilder::new().write_u256(32u32).build();
		output.extend(OutputBuilder::new().write_u256(encoded.len()).build());
		output.extend(OutputBuilder::new().write_bytes(encoded).build());

		let gas_consumed = RuntimeHelper::<Runtime>::db_read_gas_cost();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output,
			logs: Default::default(),
		})
	}

	fn encode_unbond(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		input.expect_arguments(1)?;

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			if gas_consumed > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}

		let amount = parse_relay_amount(&mut input)?;
		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::Unbond(amount));

		let mut output = OutputBuilder::new().write_u256(32u32).build();
		output.extend(OutputBuilder::new().write_u256(encoded.len()).build());
		output.extend(OutputBuilder::new().write_bytes(encoded).build());

		let gas_consumed = RuntimeHelper::<Runtime>::db_read_gas_cost();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output,
			logs: Default::default(),
		})
	}

	fn encode_withdraw_unbonded(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		input.expect_arguments(1)?;

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			if gas_consumed > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}

		let num_slashing_spans = input.read_u32()?;
		let encoded =
			RelayRuntime::encode_call(AvailableStakeCalls::WithdrawUnbonded(num_slashing_spans));

		let mut output = OutputBuilder::new().write_u256(32u32).build();
		output.extend(OutputBuilder::new().write_u256(encoded.len()).build());
		output.extend(OutputBuilder::new().write_bytes(encoded).build());

		let gas_consumed = RuntimeHelper::<Runtime>::db_read_gas_cost();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output,
			logs: Default::default(),
		})
	}

	fn encode_validate(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		input.expect_arguments(2)?;

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			if gas_consumed > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}

		let parst_per_billion = input.read_u32()?;
		let blocked = input.read_bool()?;
		let fraction = Perbill::from_parts(parst_per_billion);
		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::Validate(
			pallet_staking::ValidatorPrefs {
				commission: fraction,
				blocked: blocked,
			},
		));

		let mut output = OutputBuilder::new().write_u256(32u32).build();
		output.extend(OutputBuilder::new().write_u256(encoded.len()).build());
		output.extend(OutputBuilder::new().write_bytes(encoded).build());

		let gas_consumed = RuntimeHelper::<Runtime>::db_read_gas_cost();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output,
			logs: Default::default(),
		})
	}

	fn encode_nominate(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		input.expect_minimum_arguments(2)?;

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			if gas_consumed > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}

		// This points to the offset at which the vector starts. In this case, should be immediate.
		let _ = input.read_u256()?;

		// The next thing is to read the length of the vector
		let length = input.read_u32()?;

		input.expect_arguments(2 + length as usize)?;

		let mut nominated = Vec::new();

		for _ in 0u32..length {
			nominated.push(input.read_relay_address()?)
		}

		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::Nominate(nominated));

		let mut output = OutputBuilder::new().write_u256(32u32).build();
		output.extend(OutputBuilder::new().write_u256(encoded.len()).build());
		output.extend(OutputBuilder::new().write_bytes(encoded).build());

		let gas_consumed = RuntimeHelper::<Runtime>::db_read_gas_cost();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output,
			logs: Default::default(),
		})
	}

	fn encode_chill(
		input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		input.expect_arguments(0)?;

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			if gas_consumed > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}

		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::Chill);

		let gas_consumed = RuntimeHelper::<Runtime>::db_read_gas_cost();

		let mut output = OutputBuilder::new().write_u256(32u32).build();
		output.extend(OutputBuilder::new().write_u256(encoded.len()).build());
		output.extend(OutputBuilder::new().write_bytes(encoded).build());

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output,
			logs: Default::default(),
		})
	}

	fn encode_set_payee(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		input.expect_arguments(2)?;
		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			if gas_consumed > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}

		let reward_destination = parse_reward_destination(&mut input)?;

		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::SetPayee(reward_destination));

		let mut output = OutputBuilder::new().write_u256(32u32).build();
		output.extend(OutputBuilder::new().write_u256(encoded.len()).build());
		output.extend(OutputBuilder::new().write_bytes(encoded).build());

		let gas_consumed = RuntimeHelper::<Runtime>::db_read_gas_cost();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output,
			logs: Default::default(),
		})
	}

	fn encode_set_controller(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		input.expect_arguments(1)?;
		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			if gas_consumed > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}

		let controller = input.read_relay_address()?;

		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::SetController(controller));

		let mut output = OutputBuilder::new().write_u256(32u32).build();
		output.extend(OutputBuilder::new().write_u256(encoded.len()).build());
		output.extend(OutputBuilder::new().write_bytes(encoded).build());

		let gas_consumed = RuntimeHelper::<Runtime>::db_read_gas_cost();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output,
			logs: Default::default(),
		})
	}

	fn encode_rebond(
		mut input: InputReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		input.expect_arguments(1)?;
		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			if gas_consumed > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}

		let amount = parse_relay_amount(&mut input)?;

		let encoded = RelayRuntime::encode_call(AvailableStakeCalls::Rebond(amount));

		let mut output = OutputBuilder::new().write_u256(32u32).build();
		output.extend(OutputBuilder::new().write_u256(encoded.len()).build());
		output.extend(OutputBuilder::new().write_bytes(encoded).build());

		let gas_consumed = RuntimeHelper::<Runtime>::db_read_gas_cost();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output,
			logs: Default::default(),
		})
	}
}

pub fn parse_relay_amount(input: &mut InputReader) -> Result<relay_chain::Balance, ExitError> {
	let a = input.read_u256()?;
	TryInto::<relay_chain::Balance>::try_into(a)
		.map_err(|_| ExitError::Other("Amount is too large for provided balance type".into()))
}

pub fn parse_reward_destination(
	input: &mut InputReader,
) -> Result<pallet_staking::RewardDestination<relay_chain::AccountId>, ExitError> {
	let option: u128 = input
		.read_u256()?
		.try_into()
		.map_err(|_| ExitError::Other("Amount is too large for provided balance type".into()))?;
	let address = input.read_relay_address()?;

	match option {
		0u128 => Ok(RewardDestination::Staked),
		1u128 => Ok(RewardDestination::Stash),
		2u128 => Ok(RewardDestination::Controller),
		3u128 => Ok(RewardDestination::Account(address)),
		4u128 => Ok(RewardDestination::None),
		_ => Err(error("Not available enum")),
	}
}
