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

#![cfg_attr(not(feature = "std"), no_std)]

use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	sp_runtime::traits::{CheckedSub, StaticLookup},
	storage::types::StorageDoubleMap,
	traits::StorageInstance,
	transactional, Blake2_128Concat,
};
use pallet_balances::pallet::{
	Instance1, Instance10, Instance11, Instance12, Instance13, Instance14, Instance15, Instance16,
	Instance2, Instance3, Instance4, Instance5, Instance6, Instance7, Instance8, Instance9,
};
use pallet_evm::{AddressMapping, Precompile};
use precompile_utils::{error, EvmResult, InputReader, LogsBuilder, OutputBuilder, RuntimeHelper};
use slices::u8_slice;
use sp_core::{H160, U256};
use sp_std::{convert::TryInto, marker::PhantomData, vec};

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
const SELECTOR_LOG_TRANSFER: &[u8; 32] =
	u8_slice!("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

/// Solidity selector of the Approval log, which is the Keccak of the Log signature.
const SELECTOR_LOG_APPROVAL: &[u8; 32] =
	u8_slice!("0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925");

/// Associates pallet Instance to a prefix used for the Approves storage.
/// This trait is implemented for () and the 16 substrate Instance.
pub trait InstanceToPrefix {
	/// Prefix used for the Approves storage.
	type ApprovesPrefix: StorageInstance;
}

// We use a macro to implement the trait for () and the 16 substrate Instance.
macro_rules! impl_prefix {
	($prefix:ident, $instance:ty, $name:literal) => {
		pub struct $prefix;

		impl StorageInstance for $prefix {
			const STORAGE_PREFIX: &'static str = "Approves";

			fn pallet_prefix() -> &'static str {
				$name
			}
		}

		impl InstanceToPrefix for $instance {
			type ApprovesPrefix = $prefix;
		}
	};
}

impl_prefix!(ApprovesPrefix0, (), "Erc20Instance0Balances");
impl_prefix!(ApprovesPrefix1, Instance1, "Erc20Instance1Balances");
impl_prefix!(ApprovesPrefix2, Instance2, "Erc20Instance2Balances");
impl_prefix!(ApprovesPrefix3, Instance3, "Erc20Instance3Balances");
impl_prefix!(ApprovesPrefix4, Instance4, "Erc20Instance4Balances");
impl_prefix!(ApprovesPrefix5, Instance5, "Erc20Instance5Balances");
impl_prefix!(ApprovesPrefix6, Instance6, "Erc20Instance6Balances");
impl_prefix!(ApprovesPrefix7, Instance7, "Erc20Instance7Balances");
impl_prefix!(ApprovesPrefix8, Instance8, "Erc20Instance8Balances");
impl_prefix!(ApprovesPrefix9, Instance9, "Erc20Instance9Balances");
impl_prefix!(ApprovesPrefix10, Instance10, "Erc20Instance10Balances");
impl_prefix!(ApprovesPrefix11, Instance11, "Erc20Instance11Balances");
impl_prefix!(ApprovesPrefix12, Instance12, "Erc20Instance12Balances");
impl_prefix!(ApprovesPrefix13, Instance13, "Erc20Instance13Balances");
impl_prefix!(ApprovesPrefix14, Instance14, "Erc20Instance14Balances");
impl_prefix!(ApprovesPrefix15, Instance15, "Erc20Instance15Balances");
impl_prefix!(ApprovesPrefix16, Instance16, "Erc20Instance16Balances");

/// Alias for the Balance type for the provided Runtime and Instance.
pub type BalanceOf<Runtime, Instance = ()> =
	<Runtime as pallet_balances::Config<Instance>>::Balance;

/// Storage type used to store approvals, since `pallet_balances` doesn't
/// handle this behavior.
/// (Owner => Allowed => Amount)
pub type ApprovesStorage<Runtime, Instance> = StorageDoubleMap<
	<Instance as InstanceToPrefix>::ApprovesPrefix,
	Blake2_128Concat,
	<Runtime as frame_system::Config>::AccountId,
	Blake2_128Concat,
	<Runtime as frame_system::Config>::AccountId,
	<Runtime as pallet_balances::Config<Instance>>::Balance,
>;

/// Precompile exposing a pallet_balance as an ERC20.
/// Multiple precompiles can support instances of pallet_balance.
/// The precompile used an additional storage to store approvals.
pub struct Erc20BalancesPrecompile<Runtime, Instance: 'static = ()>(
	PhantomData<(Runtime, Instance)>,
);

impl<Runtime, Instance> Precompile for Erc20BalancesPrecompile<Runtime, Instance>
where
	Runtime: pallet_balances::Config<Instance> + pallet_evm::Config,
	Runtime::AccountId: From<H160>,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_balances::Call<Runtime, Instance>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Instance: InstanceToPrefix + 'static,
	U256: From<BalanceOf<Runtime, Instance>> + TryInto<BalanceOf<Runtime, Instance>>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let input = InputReader::new(input)?;

		match input.selector() {
			[0x7c, 0x80, 0xaa, 0x9f] => Self::total_supply(input),
			[0x70, 0xa0, 0x82, 0x31] => Self::balance_of(input),
			[0xdd, 0x62, 0xed, 0x3e] => Self::allowance(input),
			[0x09, 0x5e, 0xa7, 0xb3] => Self::approve(input, context),
			[0xa9, 0x05, 0x9c, 0xbb] => Self::transfer(input, context, target_gas),
			[0x0c, 0x41, 0xb0, 0x33] => Self::transfer_from(input, context, target_gas),
			_ => Err(error("unknown selector")),
		}
	}
}

impl<Runtime, Instance> Erc20BalancesPrecompile<Runtime, Instance>
where
	Runtime: pallet_balances::Config<Instance> + pallet_evm::Config,
	Runtime::AccountId: From<H160>,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_balances::Call<Runtime, Instance>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	Instance: InstanceToPrefix + 'static,
	U256: From<BalanceOf<Runtime, Instance>> + TryInto<BalanceOf<Runtime, Instance>>,
{
	fn total_supply(input: InputReader) -> EvmResult<PrecompileOutput> {
		// Parse input.
		input.expect_arguments(0)?;

		// Fetch info.
		let amount = pallet_balances::Pallet::<Runtime, Instance>::total_issuance();

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: RuntimeHelper::<Runtime>::db_read_gas_cost(),
			output: OutputBuilder::new().write_u256(amount).build(),
			logs: vec![],
		})
	}

	fn balance_of(mut input: InputReader) -> EvmResult<PrecompileOutput> {
		// Read input.
		input.expect_arguments(1)?;

		let address = input.read_address()?;

		// Fetch info.
		let amount = pallet_balances::Pallet::<Runtime, Instance>::usable_balance(&address.into());

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: RuntimeHelper::<Runtime>::db_read_gas_cost(),
			output: OutputBuilder::new().write_u256(amount).build(),
			logs: vec![],
		})
	}

	fn allowance(mut input: InputReader) -> EvmResult<PrecompileOutput> {
		// Read input.
		input.expect_arguments(2)?;

		let owner_id: Runtime::AccountId = input.read_address()?.into();
		let spender_id: Runtime::AccountId = input.read_address()?.into();

		// Fetch info.
		let amount =
			ApprovesStorage::<Runtime, Instance>::get(owner_id, spender_id).unwrap_or_default();

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: RuntimeHelper::<Runtime>::db_read_gas_cost(),
			output: OutputBuilder::new().write_u256(amount).build(),
			logs: vec![],
		})
	}

	fn approve(mut input: InputReader, context: &Context) -> EvmResult<PrecompileOutput> {
		// Parse input.
		input.expect_arguments(2)?;

		let spender = input.read_address()?;
		let amount = Self::u256_to_amount(input.read_u256()?)?;

		let spender_id: Runtime::AccountId = spender.into();
		let caller_id: Runtime::AccountId = context.caller.into();

		// Write into storage.
		ApprovesStorage::<Runtime, Instance>::insert(caller_id, spender_id, amount);

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: RuntimeHelper::<Runtime>::db_read_gas_cost(),
			output: vec![],
			logs: LogsBuilder::new(context.address)
				.log3(
					SELECTOR_LOG_APPROVAL,
					context.caller,
					spender,
					OutputBuilder::new().write_u256(amount).build(),
				)
				.build(),
		})
	}

	fn transfer(
		mut input: InputReader,
		context: &Context,
		target_gas: Option<u64>,
	) -> EvmResult<PrecompileOutput> {
		// Parse input.
		input.expect_arguments(2)?;

		let to = input.read_address()?;
		let amount = Self::u256_to_amount(input.read_u256()?)?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = pallet_balances::Call::<Runtime, Instance>::transfer(
			Runtime::Lookup::unlookup(to.into()),
			amount,
		);

		// Dispatch call (if enough gas).
		let used_gas =
			RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, target_gas)?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: used_gas,
			output: vec![],
			logs: LogsBuilder::new(context.address)
				.log3(
					SELECTOR_LOG_TRANSFER,
					context.caller,
					to,
					OutputBuilder::new().write_u256(amount).build(),
				)
				.build(),
		})
	}

	// This function is annotated with transactional.
	// This is to ensure that if the substrate call fails, the change in allowance is reverted.
	#[transactional]
	fn transfer_from(
		mut input: InputReader,
		context: &Context,
		target_gas: Option<u64>,
	) -> EvmResult<PrecompileOutput> {
		// Parse input.
		input.expect_arguments(3)?;

		let from = input.read_address()?;
		let to = input.read_address()?;
		let amount = Self::u256_to_amount(input.read_u256()?)?;

		// If caller is "from", it can spend as much as it wants.
		if context.caller != from {
			let owner_id: Runtime::AccountId = from.into();
			let spender_id: Runtime::AccountId = context.caller.into();

			ApprovesStorage::<Runtime, Instance>::mutate(owner_id, spender_id, |entry| {
				// Get current value, exit if None.
				let value = entry.ok_or(error("spender not allowed"))?;

				// Remove "amount" from allowed, exit if underflow.
				let new_value = value
					.checked_sub(&amount)
					.ok_or_else(|| error("trying to spend more than allowed"))?;

				// Update value.
				*entry = Some(new_value);

				Ok(())
			})?;
		}

		// Build call with origin. Here origin is the "from" field.
		let origin = Runtime::AddressMapping::into_account_id(from);
		let call = pallet_balances::Call::<Runtime, Instance>::transfer(
			Runtime::Lookup::unlookup(to.into()),
			amount,
		);

		// Dispatch call (if enough gas).
		let used_gas =
			RuntimeHelper::<Runtime>::try_dispatch(Some(origin).into(), call, target_gas)?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: used_gas,
			output: vec![],
			logs: LogsBuilder::new(context.address)
				.log3(
					SELECTOR_LOG_TRANSFER,
					from,
					to,
					OutputBuilder::new().write_u256(amount).build(),
				)
				.build(),
		})
	}

	fn u256_to_amount(value: U256) -> EvmResult<BalanceOf<Runtime, Instance>> {
		value
			.try_into()
			.map_err(|_| error("amount is too large for provided balance type"))
	}
}
