// Copyright 2019-2021 PureStake Inc.
// This file is 	part of Moonbeam.

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
#![cfg_attr(test, feature(assert_matches))]

use codec::{Decode, Encode};
use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use frame_support::traits::fungibles::Inspect;
use frame_support::traits::OriginTrait;
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	sp_runtime::traits::StaticLookup,
};

use pallet_evm::{AddressMapping, Precompile};
use precompile_utils::{
	error, Address, EvmDataReader, EvmDataWriter, EvmResult, Gasometer, LogsBuilder, RuntimeHelper,
};

use slices::u8_slice;
use sp_core::{H160, U256};
use sp_std::{
	convert::{TryFrom, TryInto},
	marker::PhantomData,
	vec,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_TRANSFER: &[u8; 32] =
	u8_slice!("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

/// Solidity selector of the Approval log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_APPROVAL: &[u8; 32] =
	u8_slice!("0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925");

/// Alias for the Balance type for the provided Runtime and Instance.
pub type BalanceOf<Runtime, Instance = ()> = <Runtime as pallet_assets::Config<Instance>>::Balance;

/// Alias for the Asset Id type for the provided Runtime and Instance.
pub type AssetIdOf<Runtime, Instance = ()> = <Runtime as pallet_assets::Config<Instance>>::AssetId;

#[derive(Default, Clone, Encode, Decode)]
pub struct ApprovalFromTo<Runtime: frame_system::Config> {
	from: <Runtime as frame_system::Config>::AccountId,
	to: <Runtime as frame_system::Config>::AccountId,
}

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
pub enum Action {
	TotalSupply = "totalSupply()",
	BalanceOf = "balanceOf(address)",
	//	Allowance = "allowance(address,address)",
	Transfer = "transfer(address,uint256)",
	Approve = "approve(address,uint256)",
	TransferFrom = "transferFrom(address,address,uint256)",
}

/// This trait ensure we can convert AccountIds to AssetIds
/// We will require Runtime::Precompiles to have this trait implemented
pub trait AccountIdToAssetId<Account, AssetId> {
	// Get assetId from account
	fn account_to_asset_id(account: Account) -> Option<AssetId>;
}

/// Precompile exposing a pallet_assets as an ERC20.
/// Multiple precompiles can support instances of pallet_assetss.
/// The precompile uses an additional storage to store approvals.
pub struct Erc20AssetsPrecompile<Runtime, Instance: 'static = ()>(PhantomData<(Runtime, Instance)>);

impl<Runtime, Instance> Precompile for Erc20AssetsPrecompile<Runtime, Instance>
where
	Instance: 'static,
	Runtime: pallet_assets::Config<Instance> + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_assets::Call<Runtime, Instance>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256>,
	Runtime::Precompiles: AccountIdToAssetId<Runtime::AccountId, AssetIdOf<Runtime, Instance>>,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin: OriginTrait,
{
	fn execute(
		input: &[u8], //Reminder this is big-endian
		target_gas: Option<u64>,
		context: &Context,
	) -> Result<PrecompileOutput, ExitError> {
		let mut input = EvmDataReader::new(input);

		match &input.read_selector()? {
			Action::TotalSupply => Self::total_supply(input, target_gas, context),
			Action::BalanceOf => Self::balance_of(input, target_gas, context),
			//			Action::Allowance => Self::allowance(input, target_gas, context),
			Action::Approve => Self::approve(input, target_gas, context),
			Action::Transfer => Self::transfer(input, target_gas, context),
			Action::TransferFrom => Self::transfer_from(input, target_gas, context),
		}
	}
}

impl<Runtime, Instance> Erc20AssetsPrecompile<Runtime, Instance>
where
	Instance: 'static,
	Runtime: pallet_assets::Config<Instance> + pallet_evm::Config + frame_system::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_assets::Call<Runtime, Instance>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256>,
	Runtime::Precompiles: AccountIdToAssetId<Runtime::AccountId, AssetIdOf<Runtime, Instance>>,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin: OriginTrait,
{
	fn total_supply(
		input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let execution_address = Runtime::AddressMapping::into_account_id(context.address);

		// Parse input.
		input.expect_arguments(0)?;

		let asset_id: AssetIdOf<Runtime, Instance> =
			Runtime::Precompiles::account_to_asset_id(execution_address)
				.ok_or(error("non-assetId address"))?;

		// Fetch info.
		let amount: U256 =
			pallet_assets::Pallet::<Runtime, Instance>::total_issuance(asset_id).into();

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(amount).build(),
			logs: vec![],
		})
	}

	fn balance_of(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Read input.
		input.expect_arguments(1)?;

		let owner: H160 = input.read::<Address>()?.into();

		// Fetch info.
		let amount: U256 = {
			let execution_address = Runtime::AddressMapping::into_account_id(context.address);

			let asset_id: AssetIdOf<Runtime, Instance> =
				Runtime::Precompiles::account_to_asset_id(execution_address)
					.ok_or(error("non-assetId address"))?;

			let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
			pallet_assets::Pallet::<Runtime, Instance>::balance(asset_id, &owner).into()
		};

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(amount).build(),
			logs: vec![],
		})
	}

	// This should be added once https://github.com/paritytech/substrate/pull/9757 is merged.
	/*fn allowance(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Read input.
		input.expect_arguments(2)?;

		let owner: H160 = input.read::<Address>()?.into();
		let spender: H160 = input.read::<Address>()?.into();

		// Fetch info.
		let amount: U256 = {
			let execution_address = Runtime::AddressMapping::into_account_id(context.address);
			let asset_id: AssetIdOf<Runtime, Instance> =
				Runtime::Precompiles::account_to_asset_id(execution_address)
					.ok_or(error("non-assetId address"))?;

			let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
			let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);

			ApprovesStorage::<Runtime, Instance>::get((asset_id, owner, spender))
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
	}*/

	fn approve(
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		gasometer.record_log_costs_manual(3, 32)?;

		// Parse input.
		input.expect_arguments(2)?;

		let spender: H160 = input.read::<Address>()?.into();
		let amount: U256 = input.read()?;

		// Write into storage.
		{
			let execution_address = Runtime::AddressMapping::into_account_id(context.address);
			let asset_id: AssetIdOf<Runtime, Instance> =
				Runtime::Precompiles::account_to_asset_id(execution_address)
					.ok_or(error("non-assetId address"))?;

			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);
			let amount = Self::u256_to_amount(amount)?;

			// Dispatch call (if enough gas).
			// We first cancel any exusting approvals
			// Since we cannot check storage, we need to execute this call without knowing whether
			// another approval exists already.
			// Allowance() should be checked instead of doing this Result matching
			let used_gas = match RuntimeHelper::<Runtime>::try_dispatch(
				<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin::root(),
				pallet_assets::Call::<Runtime, Instance>::force_cancel_approval(
					asset_id,
					Runtime::Lookup::unlookup(caller),
					Runtime::Lookup::unlookup(spender.clone()),
				),
				gasometer.remaining_gas()?,
			) {
				Ok(gas_used) => Ok(gas_used),
				Err(ExitError::Other(e)) => {
					// This would mean there is not an existing approval
					// We convert this case to 0 gas used
					// We could also convert it to one DB read, as this would be
					if e.contains("Unknown") {
						Ok(0)
					} else {
						Err(ExitError::Other(e))
					}
				}
				Err(e) => Err(e),
			}?;
			gasometer.record_cost(used_gas)?;

			let origin = Runtime::AddressMapping::into_account_id(context.caller);

			// Dispatch call (if enough gas).
			let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::approve_transfer(
					asset_id,
					Runtime::Lookup::unlookup(spender),
					amount,
				),
				gasometer.remaining_gas()?,
			)?;
			gasometer.record_cost(used_gas)?;
		}

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
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
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_log_costs_manual(3, 32)?;

		// Parse input.
		input.expect_arguments(2)?;

		let to: H160 = input.read::<Address>()?.into();
		let amount: U256 = input.read()?;

		// Build call with origin.
		{
			let execution_address = Runtime::AddressMapping::into_account_id(context.address);
			let asset_id: AssetIdOf<Runtime, Instance> =
				Runtime::Precompiles::account_to_asset_id(execution_address)
					.ok_or(error("non-assetId address"))?;

			let origin = Runtime::AddressMapping::into_account_id(context.caller);
			let to = Runtime::AddressMapping::into_account_id(to);
			let amount = Self::u256_to_amount(amount)?;

			// Dispatch call (if enough gas).
			let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::transfer(
					asset_id,
					Runtime::Lookup::unlookup(to),
					amount,
				),
				gasometer.remaining_gas()?,
			)?;
			gasometer.record_cost(used_gas)?;
		}

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
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
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_write_gas_cost())?;
		gasometer.record_log_costs_manual(3, 32)?;

		// Parse input.
		input.expect_arguments(3)?;
		let from: H160 = input.read::<Address>()?.into();
		let to: H160 = input.read::<Address>()?.into();
		let amount: U256 = input.read()?;

		{
			let execution_address = Runtime::AddressMapping::into_account_id(context.address);
			let asset_id: AssetIdOf<Runtime, Instance> =
				Runtime::Precompiles::account_to_asset_id(execution_address)
					.ok_or(error("non-assetId address"))?;
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from.clone());
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
			let amount = Self::u256_to_amount(amount)?;

			// If caller is "from", it can spend as much as it wants.
			if caller != from {
				// Dispatch call (if enough gas).
				let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
					Some(caller).into(),
					pallet_assets::Call::<Runtime, Instance>::transfer_approved(
						asset_id,
						Runtime::Lookup::unlookup(from),
						Runtime::Lookup::unlookup(to),
						amount,
					),
					gasometer.remaining_gas()?,
				)?;
				gasometer.record_cost(used_gas)?;
			} else {
				// Dispatch call (if enough gas).
				let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
					Some(from).into(),
					pallet_assets::Call::<Runtime, Instance>::transfer(
						asset_id,
						Runtime::Lookup::unlookup(to),
						amount,
					),
					gasometer.remaining_gas()?,
				)?;
				gasometer.record_cost(used_gas)?;
			}
		}

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: vec![],
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

	fn u256_to_amount(value: U256) -> EvmResult<BalanceOf<Runtime, Instance>> {
		value
			.try_into()
			.map_err(|_| error("amount is too large for provided balance type"))
	}
}
