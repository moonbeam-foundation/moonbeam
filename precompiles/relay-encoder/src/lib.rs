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
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::Precompile;
use pallet_staking::RewardDestination;
use precompile_utils::{
	error, Bytes, EvmDataReader, EvmDataWriter, EvmResult, Gasometer, RuntimeHelper,
};
use sp_core::{H256, U256};
use sp_runtime::AccountId32;
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
		let mut input = EvmDataReader::new(input);
		// Parse the function selector
		// These are the four-byte function selectors calculated from the CrowdloanInterface.sol
		// according to the solidity specification
		// https://docs.soliditylang.org/en/v0.8.0/abi-spec.html#function-selector

		match &input.read_selector()? {
			// Check for accessor methods first. These return results immediately
			[0xbe, 0x3e, 0x04, 0x00] => {
				return Self::encode_bond(input, target_gas);
			}
			[0x49, 0xde, 0xf3, 0x26] => {
				return Self::encode_bond_extra(input, target_gas);
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
			[0x3d, 0xa4, 0x87, 0x67] => {
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
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(4)?;

		let address: [u8; 32] = input.read::<H256>()?.into();
		let amount: U256 = input.read()?;
		let relay_amount = u256_to_relay_amount(amount)?;

		let option: u8 = input.read()?;
		let reward_address: [u8; 32] = input.read::<H256>()?.into();

		let reward_destination = parse_reward_destination(option, reward_address.into())?;
		let encoded: Bytes = RelayRuntime::encode_call(AvailableStakeCalls::Bond(
			address.into(),
			relay_amount,
			reward_destination,
		))
		.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_bond_extra(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(1)?;
		let amount: U256 = input.read()?;
		let relay_amount = u256_to_relay_amount(amount)?;
		let encoded: Bytes =
			RelayRuntime::encode_call(AvailableStakeCalls::BondExtra(relay_amount)).into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_unbond(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(1)?;

		let amount: U256 = input.read()?;
		let relay_amount = u256_to_relay_amount(amount)?;

		let encoded: Bytes =
			RelayRuntime::encode_call(AvailableStakeCalls::Unbond(relay_amount)).into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_withdraw_unbonded(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(1)?;

		let num_slashing_spans: u32 = input.read()?;
		let encoded: Bytes =
			RelayRuntime::encode_call(AvailableStakeCalls::WithdrawUnbonded(num_slashing_spans))
				.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_validate(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(2)?;

		let parst_per_billion: u32 = input.read()?;
		let blocked: bool = input.read()?;
		let fraction = Perbill::from_parts(parst_per_billion);
		let encoded: Bytes = RelayRuntime::encode_call(AvailableStakeCalls::Validate(
			pallet_staking::ValidatorPrefs {
				commission: fraction,
				blocked: blocked,
			},
		))
		.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_nominate(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let nominated_as_h256: Vec<H256> = input.read()?;

		let nominated: Vec<AccountId32> = nominated_as_h256
			.iter()
			.map(|&add| {
				let as_bytes: [u8; 32] = add.into();
				as_bytes.into()
			})
			.collect();
		let encoded: Bytes =
			RelayRuntime::encode_call(AvailableStakeCalls::Nominate(nominated)).into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_chill(
		input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(0)?;

		let encoded: Bytes = RelayRuntime::encode_call(AvailableStakeCalls::Chill).into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_set_payee(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(2)?;

		let option: u8 = input.read()?;
		let reward_address: [u8; 32] = input.read::<H256>()?.into();

		let reward_destination = parse_reward_destination(option, reward_address.into())?;

		let encoded: Bytes =
			RelayRuntime::encode_call(AvailableStakeCalls::SetPayee(reward_destination)).into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_set_controller(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let controller: [u8; 32] = input.read::<H256>()?.into();

		let encoded: Bytes =
			RelayRuntime::encode_call(AvailableStakeCalls::SetController(controller.into())).into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_rebond(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(1)?;

		let amount: U256 = input.read()?;
		let relay_amount = u256_to_relay_amount(amount)?;
		let encoded: Bytes =
			RelayRuntime::encode_call(AvailableStakeCalls::Rebond(relay_amount)).into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}
}

pub fn u256_to_relay_amount(value: U256) -> EvmResult<relay_chain::Balance> {
	value
		.try_into()
		.map_err(|_| error("amount is too large for provided balance type"))
}

pub fn parse_reward_destination(
	option: u8,
	address: AccountId32,
) -> Result<pallet_staking::RewardDestination<relay_chain::AccountId>, ExitError> {
	match option {
		0u8 => Ok(RewardDestination::Staked),
		1u8 => Ok(RewardDestination::Stash),
		2u8 => Ok(RewardDestination::Controller),
		3u8 => Ok(RewardDestination::Account(address)),
		4u8 => Ok(RewardDestination::None),
		_ => Err(error("Not available enum")),
	}
}
