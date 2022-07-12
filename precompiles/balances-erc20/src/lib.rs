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

use fp_evm::{Precompile, PrecompileHandle, PrecompileOutput};
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
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_core::{H160, U256};
use sp_std::{
	convert::{TryFrom, TryInto},
	marker::PhantomData,
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

#[generate_function_selector]
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
	fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		let selector = handle.read_selector().unwrap_or_else(|_| Action::Deposit);

		handle.check_function_modifier(match selector {
			Action::Approve | Action::Transfer | Action::TransferFrom | Action::Withdraw => {
				FunctionModifier::NonPayable
			}
			Action::Deposit => FunctionModifier::Payable,
			_ => FunctionModifier::View,
		})?;

		match selector {
			Action::TotalSupply => Self::total_supply(handle),
			Action::BalanceOf => Self::balance_of(handle),
			Action::Allowance => Self::allowance(handle),
			Action::Approve => Self::approve(handle),
			Action::Transfer => Self::transfer(handle),
			Action::TransferFrom => Self::transfer_from(handle),
			Action::Name => Self::name(),
			Action::Symbol => Self::symbol(),
			Action::Decimals => Self::decimals(),
			Action::Deposit => Self::deposit(handle),
			Action::Withdraw => Self::withdraw(handle),
			Action::Eip2612Permit => {
				eip2612::Eip2612::<Runtime, Metadata, Instance>::permit(handle)
			}
			Action::Eip2612Nonces => {
				eip2612::Eip2612::<Runtime, Metadata, Instance>::nonces(handle)
			}
			Action::Eip2612DomainSeparator => {
				eip2612::Eip2612::<Runtime, Metadata, Instance>::domain_separator(handle)
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
	fn total_supply(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input.
		let input = handle.read_input()?;
		input.expect_arguments(0)?;

		// Fetch info.
		let amount: U256 = pallet_balances::Pallet::<Runtime, Instance>::total_issuance().into();

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(amount).build()))
	}

	fn balance_of(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Read input.
		let mut input = handle.read_input()?;
		input.expect_arguments(1)?;

		let owner: H160 = input.read::<Address>()?.into();

		// Fetch info.
		let amount: U256 = {
			let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
			pallet_balances::Pallet::<Runtime, Instance>::usable_balance(&owner).into()
		};

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(amount).build()))
	}

	fn allowance(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Read input.
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let owner: H160 = input.read::<Address>()?.into();
		let spender: H160 = input.read::<Address>()?.into();

		// Fetch info.
		let amount: U256 = {
			let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
			let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);

			ApprovesStorage::<Runtime, Instance>::get(owner, spender)
				.unwrap_or_default()
				.into()
		};

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(amount).build()))
	}

	fn approve(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		handle.record_log_costs_manual(3, 32)?;

		// Parse input.
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let spender: H160 = input.read::<Address>()?.into();
		let amount: U256 = input.read()?;

		// Write into storage.
		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);
			// Amount saturate if too high.
			let amount = Self::u256_to_amount(amount).unwrap_or_else(|_| Bounded::max_value());

			ApprovesStorage::<Runtime, Instance>::insert(caller, spender, amount);
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_APPROVAL,
			handle.context().caller,
			spender,
			EvmDataWriter::new().write(amount).build(),
		)
		.record(handle)?;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn transfer(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_log_costs_manual(3, 32)?;

		// Parse input.
		let mut input = handle.read_input()?;
		input.expect_arguments(2)?;

		let to: H160 = input.read::<Address>()?.into();
		let amount: U256 = input.read()?;

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			let to = Runtime::AddressMapping::into_account_id(to);
			let amount = Self::u256_to_amount(amount)?;

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_balances::Call::<Runtime, Instance>::transfer {
					dest: Runtime::Lookup::unlookup(to),
					value: amount,
				},
			)?;
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_TRANSFER,
			handle.context().caller,
			to,
			EvmDataWriter::new().write(amount).build(),
		)
		.record(handle)?;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn transfer_from(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		handle.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		handle.record_log_costs_manual(3, 32)?;

		// Parse input.
		let mut input = handle.read_input()?;
		input.expect_arguments(3)?;
		let from: H160 = input.read::<Address>()?.into();
		let to: H160 = input.read::<Address>()?.into();
		let amount: U256 = input.read()?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from);
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
			let amount = Self::u256_to_amount(amount)?;

			// If caller is "from", it can spend as much as it wants.
			if caller != from {
				ApprovesStorage::<Runtime, Instance>::mutate(from.clone(), caller, |entry| {
					// Get current value, exit if None.
					let value = entry.ok_or(revert("spender not allowed"))?;

					// Remove "amount" from allowed, exit if underflow.
					let new_value = value
						.checked_sub(&amount)
						.ok_or_else(|| revert("trying to spend more than allowed"))?;

					// Update value.
					*entry = Some(new_value);

					EvmResult::Ok(())
				})?;
			}

			// Build call with origin. Here origin is the "from"/owner field.
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(from).into(),
				pallet_balances::Call::<Runtime, Instance>::transfer {
					dest: Runtime::Lookup::unlookup(to),
					value: amount,
				},
			)?;
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_TRANSFER,
			from,
			to,
			EvmDataWriter::new().write(amount).build(),
		)
		.record(handle)?;

		// Build output.
		Ok(succeed(EvmDataWriter::new().write(true).build()))
	}

	fn name() -> EvmResult<PrecompileOutput> {
		// Build output.
		Ok(succeed(
			EvmDataWriter::new()
				.write::<Bytes>(Metadata::name().into())
				.build(),
		))
	}

	fn symbol() -> EvmResult<PrecompileOutput> {
		// Build output.
		Ok(succeed(
			EvmDataWriter::new()
				.write::<Bytes>(Metadata::symbol().into())
				.build(),
		))
	}

	fn decimals() -> EvmResult<PrecompileOutput> {
		// Build output.
		Ok(succeed(
			EvmDataWriter::new().write(Metadata::decimals()).build(),
		))
	}

	fn deposit(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		// Deposit only makes sense for the native currency.
		if !Metadata::is_native_currency() {
			return Err(revert("unknown selector"));
		}

		let caller: Runtime::AccountId =
			Runtime::AddressMapping::into_account_id(handle.context().caller);
		let precompile = Runtime::AddressMapping::into_account_id(handle.context().address);
		let amount = Self::u256_to_amount(handle.context().apparent_value)?;

		if amount.into() == U256::from(0u32) {
			return Err(revert("deposited amount must be non-zero"));
		}

		handle.record_log_costs_manual(2, 32)?;

		// Send back funds received by the precompile.
		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(precompile).into(),
			pallet_balances::Call::<Runtime, Instance>::transfer {
				dest: Runtime::Lookup::unlookup(caller),
				value: amount,
			},
		)?;

		log2(
			handle.context().address,
			SELECTOR_LOG_DEPOSIT,
			handle.context().caller,
			EvmDataWriter::new()
				.write(handle.context().apparent_value)
				.build(),
		)
		.record(handle)?;

		Ok(succeed([]))
	}

	fn withdraw(handle: &mut impl PrecompileHandle) -> EvmResult<PrecompileOutput> {
		// Withdraw only makes sense for the native currency.
		if !Metadata::is_native_currency() {
			return Err(revert("unknown selector"));
		}

		handle.record_log_costs_manual(2, 32)?;

		let mut input = handle.read_input()?;
		let withdrawn_amount: U256 = input.read()?;

		let account_amount: U256 = {
			let owner: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			pallet_balances::Pallet::<Runtime, Instance>::usable_balance(&owner).into()
		};

		if withdrawn_amount > account_amount {
			return Err(revert("trying to withdraw more than owned"));
		}

		log2(
			handle.context().address,
			SELECTOR_LOG_WITHDRAWAL,
			handle.context().caller,
			EvmDataWriter::new().write(withdrawn_amount).build(),
		)
		.record(handle)?;

		Ok(succeed([]))
	}

	fn u256_to_amount(value: U256) -> EvmResult<BalanceOf<Runtime, Instance>> {
		value
			.try_into()
			.map_err(|_| revert("amount is too large for provided balance type"))
	}
}
