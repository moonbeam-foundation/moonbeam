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

use evm::{executor::PrecompileOutput, Context, ExitError, ExitSucceed};
use frame_support::traits::fungibles::Inspect;
use frame_support::traits::OriginTrait;
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	sp_runtime::traits::StaticLookup,
};
use sp_runtime::traits::Zero;

use pallet_evm::{AddressMapping, Precompile, PrecompileSet};
use precompile_utils::{
	error, Address, EvmData, EvmDataReader, EvmDataWriter, EvmResult, Gasometer, LogsBuilder,
	RuntimeHelper,
};

use slices::u8_slice;
use sp_core::{H160, U256};
use sp_std::{convert::TryFrom, marker::PhantomData, vec};

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

#[precompile_utils::generate_function_selector]
#[derive(Debug, PartialEq, num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
pub enum Action {
	TotalSupply = "totalSupply()",
	BalanceOf = "balanceOf(address)",
	Allowance = "allowance(address,address)",
	Transfer = "transfer(address,uint256)",
	Approve = "approve(address,uint256)",
	TransferFrom = "transferFrom(address,address,uint256)",
}

/// This trait ensure we can convert AccountIds to AssetIds
/// We will require Runtime to have this trait implemented
pub trait AccountIdToAssetId<Account, AssetId> {
	// Get assetId from account
	fn account_to_asset_id(account: Account) -> Option<AssetId>;
}

/// The following distribution has been decided for the precompiles
/// 0-1023: Ethereum Mainnet Precompiles
/// 1024-2047 Precompiles that are not in Ethereum Mainnet but are neither Moonbeam specific
/// 2048-4095 Moonbeam specific precompiles
/// Asset precompiles can only fall between
/// 	0xFFFFFFFF00000000000000000000000000000000 - 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
/// The precompile for AssetId X, where X is a u128 (i.e.16 bytes), if 0XFFFFFFFF + Bytes(AssetId)
/// In order to route the address to Erc20AssetsPrecompile<R>, we first check whether the AssetId
/// exists in pallet-assets
/// We cannot do this right now, so instead we check whether the total supply is zero. If so, we
/// do not route to the precompiles

/// This means that every address that starts with 0xFFFFFFFF will go through an additional db read,
/// but the probability for this to happen is 2^-32 for random addresses
pub struct Erc20AssetsPrecompileSet<Runtime, Instance: 'static = ()>(
	PhantomData<(Runtime, Instance)>,
);

impl<Runtime, Instance> PrecompileSet for Erc20AssetsPrecompileSet<Runtime, Instance>
where
	Instance: 'static,
	Erc20AssetsPrecompile<Runtime, Instance>: Precompile,
	Runtime: pallet_assets::Config<Instance> + pallet_evm::Config,
	Runtime: AccountIdToAssetId<Runtime::AccountId, AssetIdOf<Runtime, Instance>>,
{
	fn execute(
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
	) -> Option<Result<PrecompileOutput, ExitError>> {
		if let Some(asset_id) =
			Runtime::account_to_asset_id(Runtime::AddressMapping::into_account_id(address))
		{
			// If the assetId has non-zero supply
			// "total_supply" returns both 0 if the assetId does not exist or if the supply is 0
			// The assumption I am making here is that a 0 supply asset is not interesting from
			// the perspective of the precompiles. Once pallet-assets has more publicly accesible
			// storage we can use another function for this, like check_asset_existence.
			// The other options is to check the asset existence in pallet-asset-manager, but
			// this makes the precompiles dependent on such a pallet, which is not ideal
			if !pallet_assets::Pallet::<Runtime, Instance>::total_supply(asset_id).is_zero() {
				return Some(
					<Erc20AssetsPrecompile<Runtime, Instance> as Precompile>::execute(
						input, target_gas, context,
					),
				);
			}
		}
		None
	}
}

pub struct Erc20AssetsPrecompile<Runtime, Instance: 'static = ()>(PhantomData<(Runtime, Instance)>);

impl<Runtime, Instance> Precompile for Erc20AssetsPrecompile<Runtime, Instance>
where
	Instance: 'static,
	Runtime: pallet_assets::Config<Instance> + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_assets::Call<Runtime, Instance>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256> + EvmData,
	Runtime: AccountIdToAssetId<Runtime::AccountId, AssetIdOf<Runtime, Instance>>,
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
			Action::Allowance => Self::allowance(input, target_gas, context),
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
	BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256> + EvmData,
	Runtime: AccountIdToAssetId<Runtime::AccountId, AssetIdOf<Runtime, Instance>>,
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
			Runtime::account_to_asset_id(execution_address).ok_or(error("non-assetId address"))?;

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
				Runtime::account_to_asset_id(execution_address)
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
	fn allowance(
		mut _input: EvmDataReader,
		_target_gas: Option<u64>,
		_context: &Context,
	) -> EvmResult<PrecompileOutput> {
		Err(error("unimplemented"))
		/*	let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Read input.
		input.expect_arguments(2)?;

		let owner: H160 = input.read::<Address>()?.into();
		let spender: H160 = input.read::<Address>()?.into();

		// Fetch info.
		let amount: U256 = {
			let execution_address = Runtime::AddressMapping::into_account_id(context.address);
			let asset_id: AssetIdOf<Runtime, Instance> =
				Runtime::account_to_asset_id(execution_address)
					.ok_or(error("non-assetId address"))?;

			let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
			let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);

			// Fetch info.
			pallet_assets::Pallet::<Runtime, Instance>::allowance(asset_id, owner, spender).into()
		};

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(amount).build(),
			logs: vec![],
		})*/
	}

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
		let amount = input.read::<BalanceOf<Runtime, Instance>>()?;

		{
			let execution_address = Runtime::AddressMapping::into_account_id(context.address);
			let asset_id: AssetIdOf<Runtime, Instance> =
				Runtime::account_to_asset_id(execution_address)
					.ok_or(error("non-assetId address"))?;

			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);

			// Dispatch call (if enough gas).
			// We first cancel any existing approvals
			// Since we cannot check storage, we need to execute this call without knowing whether
			// another approval exists already.
			// But we know that if no approval exists we should get "Unknown"
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
					// One DB read for checking the approval did not exist
					if e.contains("Unknown") {
						Ok(RuntimeHelper::<Runtime>::db_read_gas_cost())
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
		let amount = input.read::<BalanceOf<Runtime, Instance>>()?;

		// Build call with origin.
		{
			let execution_address = Runtime::AddressMapping::into_account_id(context.address);
			let asset_id: AssetIdOf<Runtime, Instance> =
				Runtime::account_to_asset_id(execution_address)
					.ok_or(error("non-assetId address"))?;

			let origin = Runtime::AddressMapping::into_account_id(context.caller);
			let to = Runtime::AddressMapping::into_account_id(to);

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
		let amount = input.read::<BalanceOf<Runtime, Instance>>()?;

		{
			let execution_address = Runtime::AddressMapping::into_account_id(context.address);
			let asset_id: AssetIdOf<Runtime, Instance> =
				Runtime::account_to_asset_id(execution_address)
					.ok_or(error("non-assetId address"))?;
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from.clone());
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);

			// If caller is "from", it can spend as much as it wants from its own balance.
			let used_gas = if caller != from {
				// Dispatch call (if enough gas).
				RuntimeHelper::<Runtime>::try_dispatch(
					Some(caller).into(),
					pallet_assets::Call::<Runtime, Instance>::transfer_approved(
						asset_id,
						Runtime::Lookup::unlookup(from),
						Runtime::Lookup::unlookup(to),
						amount,
					),
					gasometer.remaining_gas()?,
				)
			} else {
				// Dispatch call (if enough gas).
				RuntimeHelper::<Runtime>::try_dispatch(
					Some(from).into(),
					pallet_assets::Call::<Runtime, Instance>::transfer(
						asset_id,
						Runtime::Lookup::unlookup(to),
						amount,
					),
					gasometer.remaining_gas()?,
				)
			}?;
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
					from,
					to,
					EvmDataWriter::new().write(amount).build(),
				)
				.build(),
		})
	}
}
