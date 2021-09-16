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
use frame_support::pallet_prelude::NMapKey;
use frame_support::pallet_prelude::StorageNMap;
use frame_support::traits::fungibles::Inspect;
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	sp_runtime::traits::{CheckedSub, StaticLookup},
	traits::StorageInstance,
	transactional, Blake2_128Concat,
};
use pallet_assets::pallet::{
	Instance1, Instance10, Instance11, Instance12, Instance13, Instance14, Instance15, Instance16,
	Instance2, Instance3, Instance4, Instance5, Instance6, Instance7, Instance8, Instance9,
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

impl_prefix!(ApprovesPrefix0, (), "Erc20Instance0Assets");
impl_prefix!(ApprovesPrefix1, Instance1, "Erc20Instance1Assets");
impl_prefix!(ApprovesPrefix2, Instance2, "Erc20Instance2Assets");
impl_prefix!(ApprovesPrefix3, Instance3, "Erc20Instance3Assets");
impl_prefix!(ApprovesPrefix4, Instance4, "Erc20Instance4Assets");
impl_prefix!(ApprovesPrefix5, Instance5, "Erc20Instance5Assets");
impl_prefix!(ApprovesPrefix6, Instance6, "Erc20Instance6Assets");
impl_prefix!(ApprovesPrefix7, Instance7, "Erc20Instance7Assets");
impl_prefix!(ApprovesPrefix8, Instance8, "Erc20Instance8Assets");
impl_prefix!(ApprovesPrefix9, Instance9, "Erc20Instance9Assets");
impl_prefix!(ApprovesPrefix10, Instance10, "Erc20Instance10Assets");
impl_prefix!(ApprovesPrefix11, Instance11, "Erc20Instance11Assets");
impl_prefix!(ApprovesPrefix12, Instance12, "Erc20Instance12Assets");
impl_prefix!(ApprovesPrefix13, Instance13, "Erc20Instance13Assets");
impl_prefix!(ApprovesPrefix14, Instance14, "Erc20Instance14Assets");
impl_prefix!(ApprovesPrefix15, Instance15, "Erc20Instance15Assets");
impl_prefix!(ApprovesPrefix16, Instance16, "Erc20Instance16Assets");

/// Alias for the Balance type for the provided Runtime and Instance.
pub type BalanceOf<Runtime, Instance = ()> = <Runtime as pallet_assets::Config<Instance>>::Balance;

/// Alias for the Asset Id type for the provided Runtime and Instance.
pub type AssetIdOf<Runtime, Instance = ()> = <Runtime as pallet_assets::Config<Instance>>::AssetId;

#[derive(Default, Clone, Encode, Decode)]
pub struct ApprovalFromTo<Runtime: frame_system::Config> {
	from: <Runtime as frame_system::Config>::AccountId,
	to: <Runtime as frame_system::Config>::AccountId,
}

/// Storage type used to store approvals, since `pallet_assets` doesn't
/// handle this behavior.
/// (Owner => Allowed => Amount)
pub type ApprovesStorage<Runtime, Instance> = StorageNMap<
	<Instance as InstanceToPrefix>::ApprovesPrefix,
	(
		NMapKey<Blake2_128Concat, AssetIdOf<Runtime, Instance>>,
		NMapKey<Blake2_128Concat, <Runtime as frame_system::Config>::AccountId>, // owner
		NMapKey<Blake2_128Concat, <Runtime as frame_system::Config>::AccountId>, // delegate
	),
	<Runtime as pallet_assets::Config<Instance>>::Balance,
>;

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

pub trait AccountIdToAssetId<Account, AssetId> {
	// Get assetId from account
	fn account_to_asset_id(account: Account) -> Option<AssetId>;
}

/// Precompile exposing a pallet_balance as an ERC20.
/// Multiple precompiles can support instances of pallet_balance.
/// The precompile uses an additional storage to store approvals.
pub struct Erc20AssetsPrecompile<Runtime, Instance: 'static = ()>(PhantomData<(Runtime, Instance)>);

impl<Runtime, Instance> Precompile for Erc20AssetsPrecompile<Runtime, Instance>
where
	Instance: InstanceToPrefix + 'static,
	Runtime: pallet_assets::Config<Instance> + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_assets::Call<Runtime, Instance>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256>,
	Runtime::Precompiles: AccountIdToAssetId<Runtime::AccountId, AssetIdOf<Runtime, Instance>>,
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
	Instance: InstanceToPrefix + 'static,
	Runtime: pallet_assets::Config<Instance> + pallet_evm::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_assets::Call<Runtime, Instance>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256>,
	Runtime::Precompiles: AccountIdToAssetId<Runtime::AccountId, AssetIdOf<Runtime, Instance>>,
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

	fn allowance(
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

			ApprovesStorage::<Runtime, Instance>::insert((asset_id, caller, spender), amount);
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

	// This function is annotated with transactional.
	// This is to ensure that if the substrate call fails, the change in allowance is reverted.
	#[transactional]
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
				ApprovesStorage::<Runtime, Instance>::mutate(
					(&asset_id, &from, &caller),
					|entry| {
						// Get current value, exit if None.
						let value = entry.ok_or(error("spender not allowed"))?;

						// Remove "amount" from allowed, exit if underflow.
						let new_value = value
							.checked_sub(&amount)
							.ok_or_else(|| error("trying to spend more than allowed"))?;

						// Update value.
						*entry = Some(new_value);

						Ok(())
					},
				)?;
			}

			// Build call with origin. Here origin is the "from"/owner field.
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
