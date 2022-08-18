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

//! Precompile to interact with randomness through an evm precompile.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(assert_matches)]

extern crate alloc;

use fp_evm::{
	Context, ExitReason, ExitSucceed, Log, Precompile, PrecompileHandle, PrecompileOutput,
};
use frame_support::traits::Get;
use pallet_randomness::{
	BalanceOf, GetBabeData, Pallet, Request, RequestInfo, RequestState, RequestType,
};
use precompile_utils::{costs::call_cost, prelude::*};
use sp_core::{H160, H256, U256};
use sp_std::{fmt::Debug, marker::PhantomData, vec, vec::Vec};

// #[cfg(test)]
// mod mock;
mod solidity_types;
#[cfg(test)]
mod tests;
use solidity_types::*;

#[generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	RelayEpochIndex = "relayEpochIndex()",
	RequiredDeposit = "requiredDeposit()",
	GetRequestStatus = "getRequestStatus(uint256)",
	GetRequest = "getRequest(uint256)",
	RequestRelayBabeEpochRandomWords =
		"requestRelayBabeEpochRandomWords(address,uint256,uint64,bytes32,uint8)",
	RequestLocalVRFRandomWords =
		"requestLocalVRFRandomWords(address,uint256,uint64,bytes32,uint8,uint64)",
	FulfillRequest = "fulfillRequest(uint256)",
	IncreaseRequestFee = "increaseRequestFee(uint256,uint256)",
	PurgeExpiredRequest = "purgeExpiredRequest(uint256)",
}

// Tests to verify equal to weight_to_gas(weight) in runtime integration tests
pub const REQUEST_RANDOMNESS_ESTIMATED_COST: u64 = 26325;
pub const FULFILLMENT_OVERHEAD_ESTIMATED_COST: u64 = 24545;
pub const INCREASE_REQUEST_FEE_ESTIMATED_COST: u64 = 16718;
pub const EXECUTE_EXPIRATION_ESTIMATED_COST: u64 = 21989;
pub const LOG_FULFILLMENT_SUCCEEDED: [u8; 32] = keccak256!("FulFillmentSucceeded()");
pub const LOG_FULFILLMENT_FAILED: [u8; 32] = keccak256!("FulFillmentFailed()");

pub fn log_fulfillment_succeeded(address: impl Into<H160>) -> Log {
	log1(address, LOG_FULFILLMENT_SUCCEEDED, vec![])
}

pub fn log_fulfillment_failed(address: impl Into<H160>) -> Log {
	log1(address, LOG_FULFILLMENT_FAILED, vec![])
}

/// Reverts if fees and gas_limit are not sufficient to make subcall and cleanup
fn ensure_can_provide_randomness<Runtime>(
	code_address: H160,
	gas_limit: u64,
	request_fee: BalanceOf<Runtime>,
	clean_up_cost: u64,
) -> EvmResult<()>
where
	Runtime: pallet_randomness::Config + pallet_evm::Config + pallet_base_fee::Config,
	BalanceOf<Runtime>: Into<U256>,
{
	// assert fee > gasLimit * base_fee
	let gas_limit_as_u256: U256 = gas_limit.into();
	if let Some(gas_limit_times_base_fee) =
		gas_limit_as_u256.checked_mul(pallet_base_fee::Pallet::<Runtime>::base_fee_per_gas())
	{
		if gas_limit_times_base_fee >= request_fee.into() {
			return Err(revert(
				"Gas limit at current price must be less than fees allotted",
			));
		}
	} else {
		return Err(revert("Gas limit times base fee overflowed U256"));
	}
	let log_cost = log_fulfillment_failed(code_address)
		.compute_cost()
		.map_err(|_| revert("failed to compute log cost"))?;
	// Cost of the call itself that the batch precompile must pay.
	let call_cost = call_cost(U256::zero(), <Runtime as pallet_evm::Config>::config());
	// assert gasLimit > overhead cost
	let overhead = call_cost + log_cost + clean_up_cost;
	if gas_limit <= overhead {
		return Err(revert("Gas limit must exceed overhead call cost"));
	}
	Ok(())
}

/// Subcall to provide randomness
/// caller must call `ensure_can_provide_randomness` before calling this function
fn provide_randomness(
	handle: &mut impl PrecompileHandle,
	request_id: u64,
	gas_limit: u64,
	contract: H160,
	randomness: Vec<H256>,
) -> EvmResult<()> {
	let (reason, _) = handle.call(
		contract,
		None,
		// callback function selector: keccak256("rawFulfillRandomWords(uint256,uint256[])")
		EvmDataWriter::new_with_selector(0x1fe543e3_u32)
			.write(request_id)
			.write(randomness)
			.build(),
		Some(gas_limit),
		false,
		&Context {
			caller: handle.context().address,
			address: contract,
			apparent_value: U256::zero(),
		},
	);
	// Logs
	// We reserved enough gas so this should not OOG.
	match reason {
		ExitReason::Revert(_) | ExitReason::Error(_) => {
			let log = log_fulfillment_failed(handle.code_address());
			handle.record_log_costs(&[&log])?;
			log.record(handle)?
		}
		ExitReason::Succeed(_) => {
			let log = log_fulfillment_succeeded(handle.code_address());
			handle.record_log_costs(&[&log])?;
			log.record(handle)?
		}
		_ => (),
	}
	Ok(())
}

/// A precompile to wrap the functionality from pallet-randomness
pub struct RandomnessWrapper<Runtime>(PhantomData<Runtime>);

impl<Runtime> Precompile for RandomnessWrapper<Runtime>
where
	Runtime: pallet_randomness::Config + pallet_evm::Config + pallet_base_fee::Config,
	<Runtime as frame_system::Config>::BlockNumber: From<u32>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256>,
{
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		log::trace!(target: "randomness-precompile", "In randomness wrapper");

		let selector = handle.read_selector()?;

		handle.check_function_modifier(match selector {
			Action::RelayEpochIndex
			| Action::GetRequestStatus
			| Action::GetRequest
			| Action::RequiredDeposit => FunctionModifier::View,
			_ => FunctionModifier::NonPayable,
		})?;

		match selector {
			Action::RelayEpochIndex => Self::relay_epoch_index(handle),
			Action::RequiredDeposit => Self::required_deposit(handle),
			Action::GetRequestStatus => Self::get_request_status(handle),
			Action::GetRequest => Self::get_request(handle),
			Action::RequestRelayBabeEpochRandomWords => Self::request_babe_randomness(handle),
			Action::RequestLocalVRFRandomWords => Self::request_local_randomness(handle),
			Action::FulfillRequest => Self::fulfill_request(handle),
			Action::IncreaseRequestFee => Self::increase_request_fee(handle),
			Action::PurgeExpiredRequest => Self::purge_expired_request(handle),
		}
	}
}

impl<Runtime> RandomnessWrapper<Runtime>
where
	Runtime: pallet_randomness::Config + pallet_evm::Config + pallet_base_fee::Config,
	<Runtime as frame_system::Config>::BlockNumber: TryInto<u64> + TryFrom<u64>,
	BalanceOf<Runtime>: TryFrom<U256> + Into<U256>,
{
	fn relay_epoch_index(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let relay_epoch_index =
			<Runtime as pallet_randomness::Config>::BabeDataGetter::get_epoch_index();
		Ok(succeed(
			EvmDataWriter::new().write(relay_epoch_index).build(),
		))
	}
	fn required_deposit(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		let required_deposit: u128 = <Runtime as pallet_randomness::Config>::Deposit::get()
			.try_into()
			.map_err(|_| revert("Amount is too large for provided balance type"))?;
		Ok(succeed(
			EvmDataWriter::new().write(required_deposit).build(),
		))
	}
	fn get_request_status(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { request_id: u64 });
		// record cost of 2 DB reads
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost() * 2)?;
		let status =
			if let Some(RequestState { request, .. }) = Pallet::<Runtime>::requests(request_id) {
				if request.is_expired() {
					RequestStatus::Expired
				} else if request.can_be_fulfilled() {
					RequestStatus::Ready
				} else {
					RequestStatus::Pending
				}
			} else {
				RequestStatus::DoesNotExist
			};
		Ok(succeed(EvmDataWriter::new().write(status).build()))
	}
	fn get_request(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { request_id: u64 });

		// record cost of 2 DB reads
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost() * 2)?;
		let RequestState { request, .. } =
			Pallet::<Runtime>::requests(request_id).ok_or(revert("Request Does Not Exist"))?;
		let status = if request.is_expired() {
			RequestStatus::Expired
		} else if request.can_be_fulfilled() {
			RequestStatus::Ready
		} else {
			RequestStatus::Pending
		};
		let (
			randomness_source,
			fulfillment_block,
			fulfillment_epoch,
			expiration_block,
			expiration_epoch,
			request_status,
		) = match request.info {
			RequestInfo::BabeEpoch(epoch_due, epoch_expired) => (
				RandomnessSource::RelayBabeEpoch,
				0u64,
				epoch_due,
				0u64,
				epoch_expired,
				status,
			),
			RequestInfo::Local(block_due, block_expired) => (
				RandomnessSource::LocalVRF,
				block_due
					.try_into()
					.map_err(|_| revert("block number overflowed u32"))?,
				0u64,
				block_expired
					.try_into()
					.map_err(|_| revert("block number overflowed u32"))?,
				0u64,
				status,
			),
		};
		let (refund_address, contract_address, fee): (Address, Address, U256) = (
			request.refund_address.into(),
			request.contract_address.into(),
			request.fee.into(),
		);
		let writer = EvmDataWriter::new()
			.write(request_id)
			.write(refund_address)
			.write(contract_address)
			.write(fee)
			.write(request.gas_limit)
			.write(request.salt)
			.write(request.num_words)
			.write(randomness_source)
			.write(fulfillment_block)
			.write(fulfillment_epoch)
			.write(expiration_block)
			.write(expiration_epoch)
			.write(request_status)
			.build();
		Ok(succeed(writer))
	}
	/// Make request for babe randomness one epoch ago
	fn request_babe_randomness(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(
			REQUEST_RANDOMNESS_ESTIMATED_COST + RuntimeHelper::<Runtime>::db_read_gas_cost(),
		)?;

		read_args!(handle, {
			refund_address: Address,
			fee: U256,
			gas_limit: u64,
			salt: H256,
			num_words: u8
		});
		let refund_address: H160 = refund_address.into();
		let fee: BalanceOf<Runtime> = fee
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").in_field("fee"))?;

		let contract_address = handle.context().caller;

		let two_epochs_later =
			<Runtime as pallet_randomness::Config>::BabeDataGetter::get_epoch_index()
				.checked_add(2u64)
				.ok_or(revert("Epoch Index (u64) overflowed"))?;

		let request = Request {
			refund_address,
			contract_address,
			fee,
			gas_limit,
			num_words,
			salt,
			info: RequestType::BabeEpoch(two_epochs_later),
		};

		let request_id: U256 = Pallet::<Runtime>::request_randomness(request)
			.map_err(|e| revert(alloc::format!("Error in pallet_randomness: {:?}", e)))?
			.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: EvmDataWriter::new().write(request_id).build(),
		})
	}
	/// Make request for local VRF randomness
	fn request_local_randomness(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(
			REQUEST_RANDOMNESS_ESTIMATED_COST + RuntimeHelper::<Runtime>::db_read_gas_cost(),
		)?;

		read_args!(handle, {
			refund_address: Address,
			fee: U256,
			gas_limit: u64,
			salt: H256,
			num_words: u8,
			delay: u64
		});
		let refund_address: H160 = refund_address.into();
		let fee: BalanceOf<Runtime> = fee
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").in_field("fee"))?;

		let contract_address = handle.context().caller;

		let current_block_number: u64 = <frame_system::Pallet<Runtime>>::block_number()
			.try_into()
			.map_err(|_| revert("block number overflowed u64"))?;

		let requested_block_number = delay
			.checked_add(current_block_number)
			.ok_or(revert("addition result overflowed u64"))?
			.try_into()
			.map_err(|_| revert("u64 addition result overflowed block number type"))?;

		let request = Request {
			refund_address,
			contract_address,
			fee,
			gas_limit,
			num_words,
			salt,
			info: RequestType::Local(requested_block_number),
		};

		let request_id: U256 = Pallet::<Runtime>::request_randomness(request)
			.map_err(|e| revert(alloc::format!("Error in pallet_randomness: {:?}", e)))?
			.into();

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: EvmDataWriter::new().write(request_id).build(),
		})
	}
	/// Fulfill a randomness request due to be fulfilled
	fn fulfill_request(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		read_args!(handle, { request_id: u64 });

		// read all the inputs
		let pallet_randomness::FulfillArgs {
			request,
			deposit,
			randomness,
		} = Pallet::<Runtime>::prepare_fulfillment(request_id)
			.map_err(|e| revert(alloc::format!("{:?}", e)))?;
		// check that randomness can be provided
		ensure_can_provide_randomness::<Runtime>(
			handle.code_address(),
			request.gas_limit,
			request.fee,
			FULFILLMENT_OVERHEAD_ESTIMATED_COST,
		)?;
		// get gas before subcall
		let before_remaining_gas = handle.remaining_gas();
		provide_randomness(
			handle,
			request_id,
			request.gas_limit,
			request.contract_address.clone().into(),
			randomness.into_iter().map(|x| H256(x)).collect(),
		)?;
		// get gas after subcall
		let after_remaining_gas = handle.remaining_gas();
		let gas_used: U256 = before_remaining_gas
			.checked_sub(after_remaining_gas)
			.ok_or(revert("Before remaining gas < After remaining gas"))?
			.into();
		// cost of execution is before_remaining_gas less after_remaining_gas
		let cost_of_execution: BalanceOf<Runtime> = gas_used
			.checked_mul(pallet_base_fee::Pallet::<Runtime>::base_fee_per_gas())
			.ok_or(revert("Multiply gas used by base fee overflowed"))?
			.try_into()
			.map_err(|_| revert("amount is too large for provided balance type"))?;
		// Finish fulfillment to
		// refund cost of execution to caller
		// refund excess fee to the refund_address
		// remove request state
		Pallet::<Runtime>::finish_fulfillment(
			request_id,
			request,
			deposit,
			&handle.context().caller,
			cost_of_execution,
		);
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: Default::default(),
		})
	}
	/// Increase the fee used to refund fulfillment of the request
	fn increase_request_fee(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(INCREASE_REQUEST_FEE_ESTIMATED_COST)?;

		read_args!(handle, {
			request_id: u64,
			fee_increase: U256
		});
		let fee_increase: BalanceOf<Runtime> = fee_increase.try_into().map_err(|_| {
			RevertReason::value_is_too_large("balance type").in_field("feeIncrease")
		})?;

		Pallet::<Runtime>::increase_request_fee(&handle.context().caller, request_id, fee_increase)
			.map_err(|e| revert(alloc::format!("{:?}", e)))?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: Default::default(),
		})
	}
	/// Execute request expiration to remove the request from storage
	/// Transfers `fee` to caller and `deposit` back to `contract_address`
	fn purge_expired_request(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(EXECUTE_EXPIRATION_ESTIMATED_COST)?;

		read_args!(handle, { request_id: u64 });

		Pallet::<Runtime>::execute_request_expiration(&handle.context().caller, request_id)
			.map_err(|e| revert(alloc::format!("{:?}", e)))?;
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			output: Default::default(),
		})
	}
}
