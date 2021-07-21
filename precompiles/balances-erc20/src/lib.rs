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
	traits::{Get, StorageInstance},
	transactional, Blake2_128Concat,
};
use pallet_balances::pallet::{
	Instance1, Instance10, Instance11, Instance12, Instance13, Instance14, Instance15, Instance16,
	Instance2, Instance3, Instance4, Instance5, Instance6, Instance7, Instance8, Instance9,
};
use pallet_evm::{AddressMapping, GasWeightMapping, Log, Precompile};
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

/// Storage type used to store approvals.
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
		const SELECTOR_SIZE_BYTES: usize = 4;

		if input.len() < 4 {
			return Err(ExitError::Other("input length less than 4 bytes".into()));
		}

		match input[0..SELECTOR_SIZE_BYTES] {
			[0x7c, 0x80, 0xaa, 0x9f] => Self::total_supply(&input[SELECTOR_SIZE_BYTES..]),
			[0x70, 0xa0, 0x82, 0x31] => Self::balance_of(&input[SELECTOR_SIZE_BYTES..]),
			[0xdd, 0x62, 0xed, 0x3e] => Self::allowance(&input[SELECTOR_SIZE_BYTES..]),
			[0x09, 0x5e, 0xa7, 0xb3] => Self::approve(context, &input[SELECTOR_SIZE_BYTES..]),
			[0xa9, 0x05, 0x9c, 0xbb] => {
				Self::transfer(context, &input[SELECTOR_SIZE_BYTES..], target_gas)
			}
			[0x0c, 0x41, 0xb0, 0x33] => {
				Self::transfer_from(context, &input[SELECTOR_SIZE_BYTES..], target_gas)
			}
			_ => Err(ExitError::Other(
				"No staking wrapper method at selector given selector".into(),
			)),
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
	/// Dispatch a call from provided origin.
	/// Will make sure the call will not consume more than the target gas.
	fn dispatch_call(
		origin: <Runtime::Call as Dispatchable>::Origin,
		call: Runtime::Call,
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		let info = call.get_dispatch_info();

		// Make sure enough gas
		if let Some(gas_limit) = target_gas {
			let required_gas = Runtime::GasWeightMapping::weight_to_gas(info.weight);
			if required_gas > gas_limit {
				return Err(ExitError::OutOfGas);
			}
		}

		// Dispatch that call.
		match call.dispatch(origin) {
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
			Err(_) => Err(ExitError::Other("ERC20 wrapper call via EVM failed".into())),
		}
	}

	fn total_supply(input: &[u8]) -> Result<PrecompileOutput, ExitError> {
		if !input.is_empty() {
			return Err(ExitError::Other("Incorrect input lenght".into()));
		}

		let amount = pallet_balances::Pallet::<Runtime, Instance>::total_issuance();

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		let mut output = [0u8; 32];
		U256::from(amount).to_big_endian(&mut output);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output.to_vec(),
			logs: Default::default(),
		})
	}

	fn balance_of(input: &[u8]) -> Result<PrecompileOutput, ExitError> {
		if input.len() != 32 {
			return Err(ExitError::Other("Incorrect input lenght".into()));
		}

		let address = H160::from_slice(&input[12..32]);

		let amount = pallet_balances::Pallet::<Runtime, Instance>::usable_balance(&address.into());

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		let mut output = [0u8; 32];
		U256::from(amount).to_big_endian(&mut output);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output.to_vec(),
			logs: Default::default(),
		})
	}

	fn allowance(input: &[u8]) -> Result<PrecompileOutput, ExitError> {
		if input.len() != 64 {
			return Err(ExitError::Other("Incorrect input lenght".into()));
		}

		let owner = H160::from_slice(&input[12..32]);
		let spender = H160::from_slice(&input[44..64]);

		let owner_id: Runtime::AccountId = owner.into();
		let spender_id: Runtime::AccountId = spender.into();

		let amount =
			ApprovesStorage::<Runtime, Instance>::get(owner_id, spender_id).unwrap_or_default();

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().read,
		);

		let mut output = [0u8; 32];
		U256::from(amount).to_big_endian(&mut output);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: output.to_vec(),
			logs: Default::default(),
		})
	}

	fn approve(context: &Context, input: &[u8]) -> Result<PrecompileOutput, ExitError> {
		if input.len() != 64 {
			return Err(ExitError::Other("Incorrect input lenght".into()));
		}

		let spender = H160::from_slice(&input[12..32]);
		let amount = Self::parse_amount(&input[32..64])?;

		let caller_id: Runtime::AccountId = context.caller.into();
		let spender_id: Runtime::AccountId = spender.into();

		ApprovesStorage::<Runtime, Instance>::insert(caller_id, spender_id, amount);

		let gas_consumed = <Runtime as pallet_evm::Config>::GasWeightMapping::weight_to_gas(
			<Runtime as frame_system::Config>::DbWeight::get().write,
		);

		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gas_consumed,
			output: vec![],
			logs: vec![Log {
				address: context.address,
				data: input[32..64].to_vec(),
				topics: vec![
					SELECTOR_LOG_APPROVAL.into(),
					context.caller.into(),
					spender.into(),
				],
			}],
		})
	}

	fn transfer(
		context: &Context,
		input: &[u8],
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		// Parse arguments.
		if input.len() != 64 {
			return Err(ExitError::Other("Incorrect input lenght".into()));
		}

		let to = H160::from_slice(&input[12..32]);
		let amount = Self::parse_amount(&input[32..64])?;

		// Build call with origin.
		let origin = Runtime::AddressMapping::into_account_id(context.caller);
		let call = pallet_balances::Call::<Runtime, Instance>::transfer(
			Runtime::Lookup::unlookup(to.into()),
			amount,
		);

		// Dispatch call (if enough gas).
		let mut output = Self::dispatch_call(Some(origin).into(), call.into(), target_gas)?;

		// Add transfer log.
		output.logs.push(Log {
			address: context.address,
			data: input[32..64].to_vec(),
			topics: vec![
				SELECTOR_LOG_TRANSFER.into(),
				context.caller.into(),
				to.into(),
			],
		});

		Ok(output)
	}

	// This function is annotated with transactional.
	// This is to ensure that if the substrate call fails, the change in allowance is reverted.
	#[transactional]
	fn transfer_from(
		context: &Context,
		input: &[u8],
		target_gas: Option<u64>,
	) -> Result<PrecompileOutput, ExitError> {
		// Parse arguments.
		if input.len() != 96 {
			return Err(ExitError::Other("Incorrect input lenght".into()));
		}

		let from = H160::from_slice(&input[12..32]);
		let to = H160::from_slice(&input[44..64]);
		let amount = Self::parse_amount(&input[64..96])?;

		// If caller is "from", it can spend as much as it wants.
		if context.caller != from {
			let owner_id: Runtime::AccountId = from.into();
			let spender_id: Runtime::AccountId = context.caller.into();

			ApprovesStorage::<Runtime, Instance>::mutate(owner_id, spender_id, |entry| {
				// Get current value, exit if None.
				let value = entry.ok_or(ExitError::Other("Not allowed".into()))?;

				// Remove "amount" from allowed, exit if underflow.
				let new_value = value.checked_sub(&amount).ok_or_else(|| {
					ExitError::Other("Requesting to spend more than allowed".into())
				})?;

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
		let mut output = Self::dispatch_call(Some(origin).into(), call.into(), target_gas)?;

		// Add transfer log.
		output.logs.push(Log {
			address: context.address,
			data: input[64..96].to_vec(), // amount
			topics: vec![SELECTOR_LOG_TRANSFER.into(), from.into(), to.into()],
		});

		Ok(output)
	}

	/// Parses an amount of ether from a 256 bit (32 byte) slice. The balance type is generic.
	fn parse_amount(input: &[u8]) -> Result<BalanceOf<Runtime, Instance>, ExitError> {
		Self::parse_uint256(input)?
			.try_into()
			.map_err(|_| ExitError::Other("Amount is too large for provided balance type".into()))
	}

	/// Parses a uint256 value
	fn parse_uint256(input: &[u8]) -> Result<U256, ExitError> {
		// In solidity all values are encoded to this width
		const SIZE_BYTES: usize = 32;

		if input.len() != SIZE_BYTES {
			return Err(ExitError::Other(
				"Incorrect input length for uint256 parsing".into(),
			));
		}

		Ok(U256::from_big_endian(&input[0..SIZE_BYTES]))
	}
}
