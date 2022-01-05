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

//! Precompile to encode relay staking calls via the EVM

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(test, feature(assert_matches))]

use cumulus_primitives_core::relay_chain;
use fp_evm::{Context, ExitSucceed, PrecompileOutput};
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	ensure,
};
use pallet_evm::Precompile;
use pallet_staking::RewardDestination;
use precompile_utils::{
	Bytes, EvmData, EvmDataReader, EvmDataWriter, EvmResult, Gasometer, RuntimeHelper,
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

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
enum Action {
	EncodeBond = "encode_bond(uint256,uint256,bytes)",
	EncodeBondExtra = "encode_bond_extra(uint256)",
	EncodeUnbond = "encode_unbond(uint256)",
	EncodeWithdrawUnbonded = "encode_withdraw_unbonded(uint32)",
	EncodeValidate = "encode_validate(uint256,bool)",
	EncodeNominate = "encode_nominate(uint256[])",
	EncodeChill = "encode_chill()",
	EncodeSetPayee = "encode_set_payee(bytes)",
	EncodeSetController = "encode_set_controller(uint256)",
	EncodeRebond = "encode_rebond(uint256)",
}

/// A precompile to provide relay stake calls encoding through evm
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
		_context: &Context,
		_is_static: bool,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		let gasometer = &mut gasometer;

		let (mut input, selector) = EvmDataReader::new_with_selector(gasometer, input)?;
		let input = &mut input;

		// Parse the function selector
		// These are the four-byte function selectors calculated from the RelayEncoder.sol
		// according to the solidity specification
		// https://docs.soliditylang.org/en/v0.8.0/abi-spec.html#function-selector
		match selector {
			// Storage Accessors
			Action::EncodeBond => Self::encode_bond(input, gasometer),
			Action::EncodeBondExtra => Self::encode_bond_extra(input, gasometer),
			Action::EncodeUnbond => Self::encode_unbond(input, gasometer),
			Action::EncodeWithdrawUnbonded => Self::encode_withdraw_unbonded(input, gasometer),
			Action::EncodeValidate => Self::encode_validate(input, gasometer),
			Action::EncodeNominate => Self::encode_nominate(input, gasometer),
			Action::EncodeChill => Self::encode_chill(input, gasometer),
			Action::EncodeSetPayee => Self::encode_set_payee(input, gasometer),
			Action::EncodeSetController => Self::encode_set_controller(input, gasometer),
			Action::EncodeRebond => Self::encode_rebond(input, gasometer),
		}
	}
}

impl<Runtime, RelayRuntime> RelayEncoderWrapper<Runtime, RelayRuntime>
where
	RelayRuntime: StakeEncodeCall,
	Runtime: pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
{
	fn encode_bond(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 4)?;

		let address: [u8; 32] = input.read::<H256>(gasometer)?.into();
		let amount: U256 = input.read(gasometer)?;
		let relay_amount = u256_to_relay_amount(gasometer, amount)?;

		let reward_destination = input.read::<RewardDestinationWrapper>(gasometer)?.into();
		let encoded: Bytes = RelayRuntime::encode_call(AvailableStakeCalls::Bond(
			address.into(),
			relay_amount,
			reward_destination,
		))
		.as_slice()
		.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_bond_extra(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 1)?;
		let amount: U256 = input.read(gasometer)?;
		let relay_amount = u256_to_relay_amount(gasometer, amount)?;
		let encoded: Bytes =
			RelayRuntime::encode_call(AvailableStakeCalls::BondExtra(relay_amount))
				.as_slice()
				.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_unbond(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 1)?;

		let amount: U256 = input.read(gasometer)?;
		let relay_amount = u256_to_relay_amount(gasometer, amount)?;

		let encoded: Bytes = RelayRuntime::encode_call(AvailableStakeCalls::Unbond(relay_amount))
			.as_slice()
			.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_withdraw_unbonded(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 1)?;

		let num_slashing_spans: u32 = input.read(gasometer)?;
		let encoded: Bytes =
			RelayRuntime::encode_call(AvailableStakeCalls::WithdrawUnbonded(num_slashing_spans))
				.as_slice()
				.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_validate(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 2)?;

		let parst_per_billion: u32 = input.read(gasometer)?;
		let blocked: bool = input.read(gasometer)?;
		let fraction = Perbill::from_parts(parst_per_billion);
		let encoded: Bytes = RelayRuntime::encode_call(AvailableStakeCalls::Validate(
			pallet_staking::ValidatorPrefs {
				commission: fraction,
				blocked: blocked,
			},
		))
		.as_slice()
		.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_nominate(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let nominated_as_h256: Vec<H256> = input.read(gasometer)?;

		let nominated: Vec<AccountId32> = nominated_as_h256
			.iter()
			.map(|&add| {
				let as_bytes: [u8; 32] = add.into();
				as_bytes.into()
			})
			.collect();
		let encoded: Bytes = RelayRuntime::encode_call(AvailableStakeCalls::Nominate(nominated))
			.as_slice()
			.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_chill(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 0)?;

		let encoded: Bytes = RelayRuntime::encode_call(AvailableStakeCalls::Chill)
			.as_slice()
			.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_set_payee(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 2)?;

		let reward_destination = input.read::<RewardDestinationWrapper>(gasometer)?.into();

		let encoded: Bytes =
			RelayRuntime::encode_call(AvailableStakeCalls::SetPayee(reward_destination))
				.as_slice()
				.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_set_controller(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let controller: [u8; 32] = input.read::<H256>(gasometer)?.into();

		let encoded: Bytes =
			RelayRuntime::encode_call(AvailableStakeCalls::SetController(controller.into()))
				.as_slice()
				.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}

	fn encode_rebond(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		input.expect_arguments(gasometer, 1)?;

		let amount: U256 = input.read(gasometer)?;
		let relay_amount = u256_to_relay_amount(gasometer, amount)?;
		let encoded: Bytes = RelayRuntime::encode_call(AvailableStakeCalls::Rebond(relay_amount))
			.as_slice()
			.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(encoded).build(),
			logs: Default::default(),
		})
	}
}

pub fn u256_to_relay_amount(
	gasometer: &mut Gasometer,
	value: U256,
) -> EvmResult<relay_chain::Balance> {
	value
		.try_into()
		.map_err(|_| gasometer.revert("amount is too large for provided balance type"))
}

// A wrapper to be able to implement here the EvmData reader
#[derive(Clone, Eq, PartialEq)]
pub struct RewardDestinationWrapper(RewardDestination<AccountId32>);

impl From<RewardDestination<AccountId32>> for RewardDestinationWrapper {
	fn from(reward_dest: RewardDestination<AccountId32>) -> Self {
		RewardDestinationWrapper(reward_dest)
	}
}

impl Into<RewardDestination<AccountId32>> for RewardDestinationWrapper {
	fn into(self) -> RewardDestination<AccountId32> {
		self.0
	}
}

impl EvmData for RewardDestinationWrapper {
	fn read(reader: &mut EvmDataReader, gasometer: &mut Gasometer) -> EvmResult<Self> {
		let reward_destination = reader.read::<Bytes>(gasometer)?;
		let reward_destination_bytes = reward_destination.as_bytes();
		ensure!(
			reward_destination_bytes.len() > 0,
			gasometer.revert("Reward destinations cannot be empty")
		);
		// For simplicity we use an EvmReader here
		let mut encoded_reward_destination = EvmDataReader::new(&reward_destination_bytes);

		// We take the first byte
		let enum_selector = encoded_reward_destination.read_raw_bytes(gasometer, 1)?;
		// The firs byte selects the enum variant
		match enum_selector[0] {
			0u8 => Ok(RewardDestinationWrapper(RewardDestination::Staked)),
			1u8 => Ok(RewardDestinationWrapper(RewardDestination::Stash)),
			2u8 => Ok(RewardDestinationWrapper(RewardDestination::Controller)),
			3u8 => {
				let address = encoded_reward_destination.read::<H256>(gasometer)?;
				Ok(RewardDestinationWrapper(RewardDestination::Account(
					address.as_fixed_bytes().clone().into(),
				)))
			}
			4u8 => Ok(RewardDestinationWrapper(RewardDestination::None)),
			_ => Err(gasometer.revert("Not available enum")),
		}
	}

	fn write(writer: &mut EvmDataWriter, value: Self) {
		let mut encoded: Vec<u8> = Vec::new();
		let encoded_bytes: Bytes = match value.0 {
			RewardDestination::Staked => {
				encoded.push(0);
				encoded.as_slice().into()
			}
			RewardDestination::Stash => {
				encoded.push(1);
				encoded.as_slice().into()
			}
			RewardDestination::Controller => {
				encoded.push(2);
				encoded.as_slice().into()
			}
			RewardDestination::Account(address) => {
				encoded.push(3);
				let address_bytes: [u8; 32] = address.into();
				encoded.append(&mut address_bytes.to_vec());
				encoded.as_slice().into()
			}
			RewardDestination::None => {
				encoded.push(4);
				encoded.as_slice().into()
			}
		};
		EvmData::write(writer, encoded_bytes);
	}
}
