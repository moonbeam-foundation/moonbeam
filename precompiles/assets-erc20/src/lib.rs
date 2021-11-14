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

use fp_evm::{Context, ExitError, ExitSucceed, PrecompileOutput};
use frame_support::traits::fungibles::approvals::Inspect as ApprovalInspect;
use frame_support::traits::fungibles::metadata::Inspect as MetadataInspect;
use frame_support::traits::fungibles::Inspect;
use frame_support::traits::OriginTrait;
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	sp_runtime::traits::StaticLookup,
};
use pallet_evm::{AddressMapping, PrecompileSet};
use precompile_utils::{
	keccak256, Address, Bytes, EvmData, EvmDataReader, EvmDataWriter, EvmResult, Gasometer,
	LogsBuilder, RuntimeHelper,
};
use sp_runtime::traits::Zero;

use sp_core::{H160, U256};
use sp_std::{convert::TryFrom, marker::PhantomData, vec};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_TRANSFER: [u8; 32] = keccak256!("Transfer(address,address,uint256)");

/// Solidity selector of the Approval log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_APPROVAL: [u8; 32] = keccak256!("Approval(address,address,uint256)");

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
	Name = "name()",
	Symbol = "symbol()",
	Decimals = "decimals()",
}

/// This trait ensure we can convert AccountIds to AssetIds
/// We will require Runtime to have this trait implemented
pub trait AccountIdAssetIdConversion<Account, AssetId> {
	// Get assetId from account
	fn account_to_asset_id(account: Account) -> Option<AssetId>;

	// Get AccountId from AssetId
	fn asset_id_to_account(asset_id: AssetId) -> Account;
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
	Runtime: pallet_assets::Config<Instance> + pallet_evm::Config + frame_system::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_assets::Call<Runtime, Instance>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256> + EvmData,
	Runtime: AccountIdAssetIdConversion<Runtime::AccountId, AssetIdOf<Runtime, Instance>>,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin: OriginTrait,
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
				let result = {
					let (input, selector) = match EvmDataReader::new_with_selector(input) {
						Ok((input, selector)) => (input, selector),
						Err(e) => return Some(Err(e)),
					};

					match selector {
						Action::TotalSupply => Self::total_supply(asset_id, input, target_gas),
						Action::BalanceOf => Self::balance_of(asset_id, input, target_gas),
						Action::Allowance => Self::allowance(asset_id, input, target_gas),
						Action::Approve => Self::approve(asset_id, input, target_gas, context),
						Action::Transfer => Self::transfer(asset_id, input, target_gas, context),
						Action::TransferFrom => {
							Self::transfer_from(asset_id, input, target_gas, context)
						}
						Action::Name => Self::name(asset_id, target_gas),
						Action::Symbol => Self::symbol(asset_id, target_gas),
						Action::Decimals => Self::decimals(asset_id, target_gas),
					}
				};
				return Some(result);
			}
		}
		None
	}
}

impl<Runtime, Instance> Erc20AssetsPrecompileSet<Runtime, Instance>
where
	Instance: 'static,
	Runtime: pallet_assets::Config<Instance> + pallet_evm::Config + frame_system::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_assets::Call<Runtime, Instance>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256> + EvmData,
	Runtime: AccountIdAssetIdConversion<Runtime::AccountId, AssetIdOf<Runtime, Instance>>,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin: OriginTrait,
{
	fn total_supply(
		asset_id: AssetIdOf<Runtime, Instance>,
		input: EvmDataReader,
		target_gas: Option<u64>,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input.
		input.expect_arguments(0)?;

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
		asset_id: AssetIdOf<Runtime, Instance>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Read input.
		input.expect_arguments(1)?;

		let owner: H160 = input.read::<Address>()?.into();

		// Fetch info.
		let amount: U256 = {
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
		asset_id: AssetIdOf<Runtime, Instance>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Read input.
		input.expect_arguments(2)?;

		let owner: H160 = input.read::<Address>()?.into();
		let spender: H160 = input.read::<Address>()?.into();

		// Fetch info.
		let amount: U256 = {
			let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
			let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);

			// Fetch info.
			pallet_assets::Pallet::<Runtime, Instance>::allowance(asset_id, &owner, &spender).into()
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
		asset_id: AssetIdOf<Runtime, Instance>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_log_costs_manual(3, 32)?;

		// Parse input.
		input.expect_arguments(2)?;

		let spender: H160 = input.read::<Address>()?.into();
		let amount = input.read::<BalanceOf<Runtime, Instance>>()?;

		{
			let origin = Runtime::AddressMapping::into_account_id(context.caller);

			let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);

			// Allowance read
			gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

			// If previous approval exists, we need to clean it
			if pallet_assets::Pallet::<Runtime, Instance>::allowance(asset_id, &origin, &spender)
				!= 0u32.into()
			{
				let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
					Some(origin.clone()).into(),
					pallet_assets::Call::<Runtime, Instance>::cancel_approval {
						id: asset_id,
						delegate: Runtime::Lookup::unlookup(spender.clone()),
					},
					gasometer.remaining_gas()?,
				)?;
				gasometer.record_cost(used_gas)?;
			}
			// Dispatch call (if enough gas).
			let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::approve_transfer {
					id: asset_id,
					delegate: Runtime::Lookup::unlookup(spender),
					amount,
				},
				gasometer.remaining_gas()?,
			)?;
			gasometer.record_cost(used_gas)?;
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
		asset_id: AssetIdOf<Runtime, Instance>,
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
			let origin = Runtime::AddressMapping::into_account_id(context.caller);
			let to = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::transfer {
					id: asset_id,
					target: Runtime::Lookup::unlookup(to),
					amount,
				},
				gasometer.remaining_gas()?,
			)?;
			gasometer.record_cost(used_gas)?;
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
		asset_id: AssetIdOf<Runtime, Instance>,
		mut input: EvmDataReader,
		target_gas: Option<u64>,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_log_costs_manual(3, 32)?;

		// Parse input.
		input.expect_arguments(3)?;
		let from: H160 = input.read::<Address>()?.into();
		let to: H160 = input.read::<Address>()?.into();
		let amount = input.read::<BalanceOf<Runtime, Instance>>()?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from.clone());
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);
			if caller != from {
				let used_gas = if caller != from {
					// Dispatch call (if enough gas).
					RuntimeHelper::<Runtime>::try_dispatch(
						Some(caller).into(),
						pallet_assets::Call::<Runtime, Instance>::transfer_approved {
							id: asset_id,
							owner: Runtime::Lookup::unlookup(from),
							destination: Runtime::Lookup::unlookup(to),
							amount,
						},
						gasometer.remaining_gas()?,
					)
				} else {
					// Dispatch call (if enough gas).
					RuntimeHelper::<Runtime>::try_dispatch(
						Some(from).into(),
						pallet_assets::Call::<Runtime, Instance>::transfer {
							id: asset_id,
							target: Runtime::Lookup::unlookup(to),
							amount,
						},
						gasometer.remaining_gas()?,
					)
				}?;
				gasometer.record_cost(used_gas)?;
			}
			// caller == from, we use regular transfer
			else {
				// Dispatch call (if enough gas).
				let used_gas = RuntimeHelper::<Runtime>::try_dispatch(
					Some(caller).into(),
					pallet_assets::Call::<Runtime, Instance>::transfer {
						id: asset_id,
						target: Runtime::Lookup::unlookup(to),
						amount,
					},
					gasometer.remaining_gas()?,
				)?;
				gasometer.record_cost(used_gas)?;
			}
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
		asset_id: AssetIdOf<Runtime, Instance>,
		target_gas: Option<u64>,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: 0,
			output: EvmDataWriter::new()
				.write::<Bytes>(
					pallet_assets::Pallet::<Runtime, Instance>::name(asset_id)
						.as_slice()
						.into(),
				)
				.build(),
			logs: Default::default(),
		})
	}

	fn symbol(
		asset_id: AssetIdOf<Runtime, Instance>,
		target_gas: Option<u64>,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;
		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: 0,
			output: EvmDataWriter::new()
				.write::<Bytes>(
					pallet_assets::Pallet::<Runtime, Instance>::symbol(asset_id)
						.as_slice()
						.into(),
				)
				.build(),
			logs: Default::default(),
		})
	}

	fn decimals(
		asset_id: AssetIdOf<Runtime, Instance>,
		target_gas: Option<u64>,
	) -> EvmResult<PrecompileOutput> {
		let mut gasometer = Gasometer::new(target_gas);
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: 0,
			output: EvmDataWriter::new()
				.write::<u8>(pallet_assets::Pallet::<Runtime, Instance>::decimals(
					asset_id,
				))
				.build(),
			logs: Default::default(),
		})
	}
}
