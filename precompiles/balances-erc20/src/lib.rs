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

//! Precompile to interact with pallet_balances instances using the ERC20 interface standard.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(test, feature(assert_matches))]

use fp_evm::{Context, ExitSucceed, PrecompileOutput};
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	sp_runtime::traits::{Bounded, CheckedSub, StaticLookup},
	storage::types::{StorageDoubleMap, StorageMap, ValueQuery},
	traits::StorageInstance,
	Blake2_128Concat,
};
use pallet_balances::pallet::{
	Instance1, Instance10, Instance11, Instance12, Instance13, Instance14, Instance15, Instance16,
	Instance2, Instance3, Instance4, Instance5, Instance6, Instance7, Instance8, Instance9,
};
use pallet_evm::{AddressMapping, Precompile};
use precompile_utils::{
	keccak256, Address, Bytes, EvmDataReader, EvmDataWriter, EvmResult, FunctionModifier,
	Gasometer, LogsBuilder, RuntimeHelper,
};
use sp_core::{H160, U256};
use sp_std::{
	convert::{TryFrom, TryInto},
	marker::PhantomData,
	vec,
};

mod eip2612;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_TRANSFER: [u8; 32] = keccak256!("Transfer(address,address,uint256)");

/// Solidity selector of the Approval log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_APPROVAL: [u8; 32] = keccak256!("Approval(address,address,uint256)");

/// Solidity selector of the Deposit log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_DEPOSIT: [u8; 32] = keccak256!("Deposit(address,uint256)");

/// Solidity selector of the Withdraw log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_WITHDRAWAL: [u8; 32] = keccak256!("Withdrawal(address,uint256)");

/// Associates pallet Instance to a prefix used for the Approves storage.
/// This trait is implemented for () and the 16 substrate Instance.
pub trait InstanceToPrefix {
	/// Prefix used for the Approves storage.
	type ApprovesPrefix: StorageInstance;

	/// Prefix used for the Approves storage.
	type NoncesPrefix: StorageInstance;
}

// We use a macro to implement the trait for () and the 16 substrate Instance.
macro_rules! impl_prefix {
	($instance:ident, $name:literal) => {
		// Using `paste!` we generate a dedicated module to avoid collisions
		// between each instance `Approves` struct.
		paste::paste! {
			mod [<_impl_prefix_ $instance:snake>] {
				use super::*;

				pub struct Approves;

				impl StorageInstance for Approves {
					const STORAGE_PREFIX: &'static str = "Approves";

					fn pallet_prefix() -> &'static str {
						$name
					}
				}

				pub struct Nonces;

				impl StorageInstance for Nonces {
					const STORAGE_PREFIX: &'static str = "Nonces";

					fn pallet_prefix() -> &'static str {
						$name
					}
				}

				impl InstanceToPrefix for $instance {
					type ApprovesPrefix = Approves;
					type NoncesPrefix = Nonces;
				}
			}
		}
	};
}

// Since the macro expect a `ident` to be used with `paste!` we cannot provide `()` directly.
type Instance0 = ();

impl_prefix!(Instance0, "Erc20Instance0Balances");
impl_prefix!(Instance1, "Erc20Instance1Balances");
impl_prefix!(Instance2, "Erc20Instance2Balances");
impl_prefix!(Instance3, "Erc20Instance3Balances");
impl_prefix!(Instance4, "Erc20Instance4Balances");
impl_prefix!(Instance5, "Erc20Instance5Balances");
impl_prefix!(Instance6, "Erc20Instance6Balances");
impl_prefix!(Instance7, "Erc20Instance7Balances");
impl_prefix!(Instance8, "Erc20Instance8Balances");
impl_prefix!(Instance9, "Erc20Instance9Balances");
impl_prefix!(Instance10, "Erc20Instance10Balances");
impl_prefix!(Instance11, "Erc20Instance11Balances");
impl_prefix!(Instance12, "Erc20Instance12Balances");
impl_prefix!(Instance13, "Erc20Instance13Balances");
impl_prefix!(Instance14, "Erc20Instance14Balances");
impl_prefix!(Instance15, "Erc20Instance15Balances");
impl_prefix!(Instance16, "Erc20Instance16Balances");

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
	BalanceOf<Runtime, Instance>,
>;

/// Storage type used to store EIP2612 nonces.
pub type NoncesStorage<Instance> = StorageMap<
	<Instance as InstanceToPrefix>::NoncesPrefix,
	// Owner
	Blake2_128Concat,
	H160,
	// Nonce
	U256,
	ValueQuery,
>;

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq)]
pub enum Action {
	TotalSupply = "totalSupply()",
	BalanceOf = "balanceOf(address)",
	Allowance = "allowance(address,address)",
	Transfer = "transfer(address,uint256)",
	Approve = "approve(address,uint256)",
	TransferFrom = "transferFrom(address,address,uint256)",
	Name = "name()",
	Symbol = "symbol()",
	Decimals = "decimals()",
	Deposit = "deposit()",
	Withdraw = "withdraw(uint256)",
	// EIP 2612
	Eip2612Permit = "permit(address,address,uint256,uint256,uint8,bytes32,bytes32)",
	Eip2612Nonces = "nonces(address)",
	Eip2612DomainSeparator = "DOMAIN_SEPARATOR()",
}

/// Metadata of an ERC20 token.
pub trait Erc20Metadata {
	/// Returns the name of the token.
	fn name() -> &'static str;

	/// Returns the symbol of the token.
	fn symbol() -> &'static str;

	/// Returns the decimals places of the token.
	fn decimals() -> u8;

	/// Must return `true` only if it represents the main native currency of
	/// the network. It must be the currency used in `pallet_evm`.
	fn is_native_currency() -> bool;
}

/// Precompile exposing a pallet_balance as an ERC20.
/// Multiple precompiles can support instances of pallet_balance.
/// The precompile uses an additional storage to store approvals.
pub struct Erc20BalancesPrecompile<Runtime, Metadata: Erc20Metadata, Instance: 'static = ()>(
	PhantomData<(Runtime, Metadata, Instance)>,
);

impl<Runtime, Metadata, Instance> Precompile
	for Erc20BalancesPrecompile<Runtime, Metadata, Instance>
where
	Metadata: Erc20Metadata,
	Instance: InstanceToPrefix + 'static,
	Runtime: pallet_balances::Config<Instance> + pallet_evm::Config + pallet_timestamp::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_balances::Call<Runtime, Instance>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256>,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		let gasometer = &mut gasometer;

		let (mut input, selector) = EvmDataReader::new_with_selector(gasometer, input)
			.unwrap_or_else(|_| (EvmDataReader::new(input), Action::Deposit));
		let input = &mut input;

		gasometer.check_function_modifier(
			context,
			is_static,
			match selector {
				Action::Approve | Action::Transfer | Action::TransferFrom | Action::Withdraw => {
					FunctionModifier::NonPayable
				}
				Action::Deposit => FunctionModifier::Payable,
				_ => FunctionModifier::View,
			},
		)?;

		match selector {
			Action::TotalSupply => Self::total_supply(input, gasometer),
			Action::BalanceOf => Self::balance_of(input, gasometer),
			Action::Allowance => Self::allowance(input, gasometer),
			Action::Approve => Self::approve(input, gasometer, context),
			Action::Transfer => Self::transfer(input, gasometer, context),
			Action::TransferFrom => Self::transfer_from(input, gasometer, context),
			Action::Name => Self::name(input, gasometer, context),
			Action::Symbol => Self::symbol(input, gasometer, context),
			Action::Decimals => Self::decimals(input, gasometer, context),
			Action::Deposit => Self::deposit(input, gasometer, context),
			Action::Withdraw => Self::withdraw(input, gasometer, context),
			Action::Eip2612Permit => {
				eip2612::Eip2612::<Runtime, Metadata, Instance>::permit(input, gasometer, context)
			}
			Action::Eip2612Nonces => {
				eip2612::Eip2612::<Runtime, Metadata, Instance>::nonces(input, gasometer)
			}
			Action::Eip2612DomainSeparator => {
				eip2612::Eip2612::<Runtime, Metadata, Instance>::domain_separator(
					gasometer, context,
				)
			}
		}
	}
}

impl<Runtime, Metadata, Instance> Erc20BalancesPrecompile<Runtime, Metadata, Instance>
where
	Metadata: Erc20Metadata,
	Instance: InstanceToPrefix + 'static,
	Runtime: pallet_balances::Config<Instance> + pallet_evm::Config + pallet_timestamp::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_balances::Call<Runtime, Instance>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256>,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
{
	fn total_supply(
		input: &EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input.
		input.expect_arguments(gasometer, 0)?;

		// Fetch info.
		let amount: U256 = pallet_balances::Pallet::<Runtime, Instance>::total_issuance().into();

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(amount).build(),
			logs: vec![],
		})
	}

	fn balance_of(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Read input.
		input.expect_arguments(gasometer, 1)?;

		let owner: H160 = input.read::<Address>(gasometer)?.into();

		// Fetch info.
		let amount: U256 = {
			let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
			pallet_balances::Pallet::<Runtime, Instance>::usable_balance(&owner).into()
		};

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(amount).build(),
			logs: vec![],
		})
	}

	fn allowance(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Read input.
		input.expect_arguments(gasometer, 2)?;

		let owner: H160 = input.read::<Address>(gasometer)?.into();
		let spender: H160 = input.read::<Address>(gasometer)?.into();

		// Fetch info.
		let amount: U256 = {
			let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
			let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);

			ApprovesStorage::<Runtime, Instance>::get(owner, spender)
				.unwrap_or_default()
				.into()
		};

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(amount).build(),
			logs: vec![],
		})
	}

	fn approve(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		gasometer.record_log_costs_manual(3, 32)?;

		// Parse input.
		input.expect_arguments(gasometer, 2)?;

		let spender: H160 = input.read::<Address>(gasometer)?.into();
		let amount: U256 = input.read(gasometer)?;

		// Write into storage.
		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);
			// Amount saturate if too high.
			let amount = Self::u256_to_amount(&mut gasometer.clone(), amount)
				.unwrap_or_else(|_| Bounded::max_value());

			ApprovesStorage::<Runtime, Instance>::insert(caller, spender, amount);
		}

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(true).build(),
			logs: LogsBuilder::new(context.address)
				.log3(
					SELECTOR_LOG_APPROVAL,
					context.caller,
					spender,
					EvmDataWriter::new().write(amount).build(),
				)
				.build(),
		})
	}

	fn transfer(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		// Parse input.
		input.expect_arguments(gasometer, 2)?;

		let to: H160 = input.read::<Address>(gasometer)?.into();
		let amount: U256 = input.read(gasometer)?;

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(context.caller);
			let to = Runtime::AddressMapping::into_account_id(to);
			let amount = Self::u256_to_amount(gasometer, amount)?;

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_balances::Call::<Runtime, Instance>::transfer {
					dest: Runtime::Lookup::unlookup(to),
					value: amount,
				},
				gasometer,
			)?;
		}

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(true).build(),
			logs: LogsBuilder::new(context.address)
				.log3(
					SELECTOR_LOG_TRANSFER,
					context.caller,
					to,
					EvmDataWriter::new().write(amount).build(),
				)
				.build(),
		})
	}

	fn transfer_from(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		gasometer.record_log_costs_manual(3, 32)?;

		// Parse input.
		input.expect_arguments(gasometer, 3)?;
		let from: H160 = input.read::<Address>(gasometer)?.into();
		let to: H160 = input.read::<Address>(gasometer)?.into();
		let amount: U256 = input.read(gasometer)?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
			let amount = Self::u256_to_amount(gasometer, amount)?;

			// If caller is "from", it can spend as much as it wants.
			if caller != from {
				ApprovesStorage::<Runtime, Instance>::mutate(from.clone(), caller, |entry| {
					// Get current value, exit if None.
					let value = entry.ok_or(gasometer.revert("spender not allowed"))?;

					// Remove "amount" from allowed, exit if underflow.
					let new_value = value
						.checked_sub(&amount)
						.ok_or_else(|| gasometer.revert("trying to spend more than allowed"))?;

					// Update value.
					*entry = Some(new_value);

					Ok(())
				})?;
			}

			// Build call with origin. Here origin is the "from"/owner field.
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(from).into(),
				pallet_balances::Call::<Runtime, Instance>::transfer {
					dest: Runtime::Lookup::unlookup(to),
					value: amount,
				},
				gasometer,
			)?;
		}

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(true).build(),
			logs: LogsBuilder::new(context.address)
				.log3(
					SELECTOR_LOG_TRANSFER,
					from,
					to,
					EvmDataWriter::new().write(amount).build(),
				)
				.build(),
		})
	}

	fn name(
		_: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		_: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new()
				.write::<Bytes>(Metadata::name().into())
				.build(),
			logs: Default::default(),
		})
	}

	fn symbol(
		_: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		_: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new()
				.write::<Bytes>(Metadata::symbol().into())
				.build(),
			logs: Default::default(),
		})
	}

	fn decimals(
		_: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		_: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(Metadata::decimals()).build(),
			logs: Default::default(),
		})
	}

	fn deposit(
		_: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Deposit only makes sense for the native currency.
		if !Metadata::is_native_currency() {
			return Err(gasometer.revert("unknown selector"));
		}

		let caller: Runtime::AccountId = Runtime::AddressMapping::into_account_id(context.caller);
		let precompile = Runtime::AddressMapping::into_account_id(context.address);
		let amount = Self::u256_to_amount(gasometer, context.apparent_value)?;

		if amount.into() == U256::from(0u32) {
			return Err(gasometer.revert("deposited amount must be non-zero"));
		}

		gasometer.record_log_costs_manual(2, 32)?;

		// Send back funds received by the precompile.
		RuntimeHelper::<Runtime>::try_dispatch(
			Some(precompile).into(),
			pallet_balances::Call::<Runtime, Instance>::transfer {
				dest: Runtime::Lookup::unlookup(caller),
				value: amount,
			},
			gasometer,
		)?;

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: LogsBuilder::new(context.address)
				.log2(
					SELECTOR_LOG_DEPOSIT,
					context.caller,
					EvmDataWriter::new().write(context.apparent_value).build(),
				)
				.build(),
		})
	}

	fn withdraw(
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		// Withdraw only makes sense for the native currency.
		if !Metadata::is_native_currency() {
			return Err(gasometer.revert("unknown selector"));
		}

		gasometer.record_log_costs_manual(2, 32)?;

		let withdrawn_amount: U256 = input.read(gasometer)?;

		let account_amount: U256 = {
			let owner: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			pallet_balances::Pallet::<Runtime, Instance>::usable_balance(&owner).into()
		};

		if withdrawn_amount > account_amount {
			return Err(gasometer.revert("trying to withdraw more than owned"));
		}

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: Default::default(),
			logs: LogsBuilder::new(context.address)
				.log2(
					SELECTOR_LOG_WITHDRAWAL,
					context.caller,
					EvmDataWriter::new().write(withdrawn_amount).build(),
				)
				.build(),
		})
	}

	fn u256_to_amount(
		gasometer: &mut Gasometer,
		value: U256,
	) -> EvmResult<BalanceOf<Runtime, Instance>> {
		value
			.try_into()
			.map_err(|_| gasometer.revert("amount is too large for provided balance type"))
	}
}
