// Copyright 2019-2022 PureStake Inc.
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

use fp_evm::{Context, ExitSucceed, PrecompileOutput};
use frame_support::traits::fungibles::approvals::Inspect as ApprovalInspect;
use frame_support::traits::fungibles::metadata::Inspect as MetadataInspect;
use frame_support::traits::fungibles::Inspect;
use frame_support::traits::{Get, OriginTrait};
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	sp_runtime::traits::StaticLookup,
};
use pallet_evm::{AddressMapping, PrecompileSet};
use precompile_utils::{
	keccak256, Address, Bytes, EvmData, EvmDataReader, EvmDataWriter, EvmResult, FunctionModifier,
	Gasometer, LogsBuilder, RuntimeHelper,
};
use sp_runtime::traits::{Bounded, Zero};
use sp_std::vec::Vec;

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
pub const SELECTOR_LOG_TRANSFER: [u8; 32] = keccak256!("Transfer(address,address,uint256)");

/// Solidity selector of the Approval log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_APPROVAL: [u8; 32] = keccak256!("Approval(address,address,uint256)");

/// Alias for the Balance type for the provided Runtime and Instance.
pub type BalanceOf<Runtime, Instance = ()> = <Runtime as pallet_assets::Config<Instance>>::Balance;

/// Alias for the Asset Id type for the provided Runtime and Instance.
pub type AssetIdOf<Runtime, Instance = ()> = <Runtime as pallet_assets::Config<Instance>>::AssetId;

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
	Mint = "mint(address,uint256)",
	Burn = "burn(address,uint256)",
	Freeze = "freeze(address)",
	Thaw = "thaw(address)",
	FreezeAsset = "freeze_asset()",
	ThawAsset = "thaw_asset()",
	TransferOwnership = "transfer_ownership(address)",
	SetTeam = "set_team(address,address,address)",
	SetMetadata = "set_metadata(string,string,uint8)",
	ClearMetadata = "clear_metadata()",
}

/// This trait ensure we can convert AccountIds to AssetIds
/// We will require Runtime to have this trait implemented
pub trait AccountIdAssetIdConversion<Account, AssetId> {
	// Get assetId and prefix from account
	fn account_to_asset_id(account: Account) -> Option<(Vec<u8>, AssetId)>;

	// Get AccountId from AssetId
	fn asset_id_to_account(prefix: &[u8], asset_id: AssetId) -> Account;
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
pub struct Erc20AssetsPrecompileSet<Runtime, IsLocal, Instance: 'static = ()>(
	PhantomData<(Runtime, IsLocal, Instance)>,
);

impl<Runtime, IsLocal, Instance> PrecompileSet
	for Erc20AssetsPrecompileSet<Runtime, IsLocal, Instance>
where
	Instance: 'static,
	Runtime: pallet_assets::Config<Instance> + pallet_evm::Config + frame_system::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_assets::Call<Runtime, Instance>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256> + EvmData,
	Runtime: AccountIdAssetIdConversion<Runtime::AccountId, AssetIdOf<Runtime, Instance>>,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin: OriginTrait,
	IsLocal: Get<bool>,
{
	fn execute(
		&self,
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
		is_static: bool,
	) -> Option<EvmResult<PrecompileOutput>> {
		if let Some((_, asset_id)) =
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
					let mut gasometer = Gasometer::new(target_gas);
					let gasometer = &mut gasometer;

					let (mut input, selector) =
						match EvmDataReader::new_with_selector(gasometer, input) {
							Ok((input, selector)) => (input, selector),
							Err(e) => return Some(Err(e)),
						};
					let input = &mut input;

					if let Err(err) = gasometer.check_function_modifier(
						context,
						is_static,
						match selector {
							Action::Approve | Action::Transfer | Action::TransferFrom => {
								FunctionModifier::NonPayable
							}
							_ => FunctionModifier::View,
						},
					) {
						return Some(Err(err));
					}

					match selector {
						// Local and Foreign common
						Action::TotalSupply => Self::total_supply(asset_id, input, gasometer),
						Action::BalanceOf => Self::balance_of(asset_id, input, gasometer),
						Action::Allowance => Self::allowance(asset_id, input, gasometer),
						Action::Approve => Self::approve(asset_id, input, gasometer, context),
						Action::Transfer => Self::transfer(asset_id, input, gasometer, context),
						Action::TransferFrom => {
							Self::transfer_from(asset_id, input, gasometer, context)
						}
						Action::Name => Self::name(asset_id, gasometer),
						Action::Symbol => Self::symbol(asset_id, gasometer),
						Action::Decimals => Self::decimals(asset_id, gasometer),
						Action::Mint => Self::mint(asset_id, input, gasometer, context),
						Action::Burn => Self::burn(asset_id, input, gasometer, context),
						Action::Freeze => Self::freeze(asset_id, input, gasometer, context),
						Action::Thaw => Self::thaw(asset_id, input, gasometer, context),
						Action::FreezeAsset => Self::freeze_asset(asset_id, gasometer, context),
						Action::ThawAsset => Self::thaw_asset(asset_id, gasometer, context),
						Action::TransferOwnership => {
							Self::transfer_ownership(asset_id, input, gasometer, context)
						}
						Action::SetTeam => Self::set_team(asset_id, input, gasometer, context),
						Action::SetMetadata => {
							Self::set_metadata(asset_id, input, gasometer, context)
						}
						Action::ClearMetadata => Self::clear_metadata(asset_id, gasometer, context),
					}
				};
				return Some(result);
			}
		}
		None
	}

	fn is_precompile(&self, address: H160) -> bool {
		if let Some((_, asset_id)) =
			Runtime::account_to_asset_id(Runtime::AddressMapping::into_account_id(address))
		{
			// If the assetId has non-zero supply
			// "total_supply" returns both 0 if the assetId does not exist or if the supply is 0
			// The assumption I am making here is that a 0 supply asset is not interesting from
			// the perspective of the precompiles. Once pallet-assets has more publicly accesible
			// storage we can use another function for this, like check_asset_existence.
			// The other options is to check the asset existence in pallet-asset-manager, but
			// this makes the precompiles dependent on such a pallet, which is not ideal
			!pallet_assets::Pallet::<Runtime, Instance>::total_supply(asset_id).is_zero()
		} else {
			false
		}
	}
}

impl<Runtime, IsLocal, Instance> Erc20AssetsPrecompileSet<Runtime, IsLocal, Instance> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<Runtime, IsLocal, Instance> Erc20AssetsPrecompileSet<Runtime, IsLocal, Instance>
where
	Instance: 'static,
	Runtime: pallet_assets::Config<Instance> + pallet_evm::Config + frame_system::Config,
	Runtime::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::Call: From<pallet_assets::Call<Runtime, Instance>>,
	<Runtime::Call as Dispatchable>::Origin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256> + EvmData,
	Runtime: AccountIdAssetIdConversion<Runtime::AccountId, AssetIdOf<Runtime, Instance>>,
	<<Runtime as frame_system::Config>::Call as Dispatchable>::Origin: OriginTrait,
	IsLocal: Get<bool>,
{
	fn total_supply(
		asset_id: AssetIdOf<Runtime, Instance>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Parse input.
		input.expect_arguments(gasometer, 0)?;

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
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		// Parse input.
		input.expect_arguments(gasometer, 2)?;

		let spender: H160 = input.read::<Address>(gasometer)?.into();
		let amount: U256 = input.read(gasometer)?;

		{
			let origin = Runtime::AddressMapping::into_account_id(context.caller);
			let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);
			// Amount saturate if too high.
			let amount: BalanceOf<Runtime, Instance> =
				amount.try_into().unwrap_or_else(|_| Bounded::max_value());

			// Allowance read
			gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

			// If previous approval exists, we need to clean it
			if pallet_assets::Pallet::<Runtime, Instance>::allowance(asset_id, &origin, &spender)
				!= 0u32.into()
			{
				RuntimeHelper::<Runtime>::try_dispatch(
					Some(origin.clone()).into(),
					pallet_assets::Call::<Runtime, Instance>::cancel_approval {
						id: asset_id,
						delegate: Runtime::Lookup::unlookup(spender.clone()),
					},
					gasometer,
				)?;
			}
			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::approve_transfer {
					id: asset_id,
					delegate: Runtime::Lookup::unlookup(spender),
					amount,
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
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		// Parse input.
		input.expect_arguments(gasometer, 2)?;

		let to: H160 = input.read::<Address>(gasometer)?.into();
		let amount = input.read::<BalanceOf<Runtime, Instance>>(gasometer)?;

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(context.caller);
			let to = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::transfer {
					id: asset_id,
					target: Runtime::Lookup::unlookup(to),
					amount,
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
		asset_id: AssetIdOf<Runtime, Instance>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_log_costs_manual(3, 32)?;

		// Parse input.
		input.expect_arguments(gasometer, 3)?;
		let from: H160 = input.read::<Address>(gasometer)?.into();
		let to: H160 = input.read::<Address>(gasometer)?.into();
		let amount = input.read::<BalanceOf<Runtime, Instance>>(gasometer)?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(context.caller);
			let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from.clone());
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);

			// If caller is "from", it can spend as much as it wants from its own balance.
			if caller != from {
				// Dispatch call (if enough gas).
				RuntimeHelper::<Runtime>::try_dispatch(
					Some(caller).into(),
					pallet_assets::Call::<Runtime, Instance>::transfer_approved {
						id: asset_id,
						owner: Runtime::Lookup::unlookup(from),
						destination: Runtime::Lookup::unlookup(to),
						amount,
					},
					gasometer,
				)?;
			} else {
				// Dispatch call (if enough gas).
				RuntimeHelper::<Runtime>::try_dispatch(
					Some(from).into(),
					pallet_assets::Call::<Runtime, Instance>::transfer {
						id: asset_id,
						target: Runtime::Lookup::unlookup(to),
						amount,
					},
					gasometer,
				)?;
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
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
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
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
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
		gasometer: &mut Gasometer,
	) -> EvmResult<PrecompileOutput> {
		gasometer.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new()
				.write::<u8>(pallet_assets::Pallet::<Runtime, Instance>::decimals(
					asset_id,
				))
				.build(),
			logs: Default::default(),
		})
	}

	// From here: only for locals, we need to check whether we are in local assets otherwise fail
	fn mint(
		asset_id: AssetIdOf<Runtime, Instance>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		if !IsLocal::get() {
			return Err(gasometer.revert("unknown selector"));
		}

		gasometer.record_log_costs_manual(3, 32)?;

		// Parse input.
		input.expect_arguments(gasometer, 2)?;

		let to: H160 = input.read::<Address>(gasometer)?.into();
		let amount = input.read::<BalanceOf<Runtime, Instance>>(gasometer)?;

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(context.caller);
			let to = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::mint {
					id: asset_id,
					beneficiary: Runtime::Lookup::unlookup(to),
					amount,
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
					H160::default(),
					to,
					EvmDataWriter::new().write(amount).build(),
				)
				.build(),
		})
	}

	fn burn(
		asset_id: AssetIdOf<Runtime, Instance>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		if !IsLocal::get() {
			return Err(gasometer.revert("unknown selector"));
		}

		gasometer.record_log_costs_manual(3, 32)?;

		// Parse input.
		input.expect_arguments(gasometer, 2)?;

		let to: H160 = input.read::<Address>(gasometer)?.into();
		let amount = input.read::<BalanceOf<Runtime, Instance>>(gasometer)?;

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(context.caller);
			let to = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::burn {
					id: asset_id,
					who: Runtime::Lookup::unlookup(to),
					amount,
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
					to,
					H160::default(),
					EvmDataWriter::new().write(amount).build(),
				)
				.build(),
		})
	}

	fn freeze(
		asset_id: AssetIdOf<Runtime, Instance>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		if !IsLocal::get() {
			return Err(gasometer.revert("unknown selector"));
		}

		// Parse input.
		input.expect_arguments(gasometer, 1)?;

		let to: H160 = input.read::<Address>(gasometer)?.into();

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(context.caller);
			let to = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::freeze {
					id: asset_id,
					who: Runtime::Lookup::unlookup(to),
				},
				gasometer,
			)?;
		}

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(true).build(),
			logs: Default::default(),
		})
	}

	fn thaw(
		asset_id: AssetIdOf<Runtime, Instance>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		if !IsLocal::get() {
			return Err(gasometer.revert("unknown selector"));
		}

		// Parse input.
		input.expect_arguments(gasometer, 1)?;

		let to: H160 = input.read::<Address>(gasometer)?.into();

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(context.caller);
			let to = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::thaw {
					id: asset_id,
					who: Runtime::Lookup::unlookup(to),
				},
				gasometer,
			)?;
		}

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(true).build(),
			logs: Default::default(),
		})
	}

	fn freeze_asset(
		asset_id: AssetIdOf<Runtime, Instance>,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		if !IsLocal::get() {
			return Err(gasometer.revert("unknown selector"));
		}

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(context.caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::freeze_asset { id: asset_id },
				gasometer,
			)?;
		}

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(true).build(),
			logs: Default::default(),
		})
	}

	fn thaw_asset(
		asset_id: AssetIdOf<Runtime, Instance>,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		if !IsLocal::get() {
			return Err(gasometer.revert("unknown selector"));
		}

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(context.caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::thaw_asset { id: asset_id },
				gasometer,
			)?;
		}

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(true).build(),
			logs: Default::default(),
		})
	}

	fn transfer_ownership(
		asset_id: AssetIdOf<Runtime, Instance>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		if !IsLocal::get() {
			return Err(gasometer.revert("unknown selector"));
		}

		// Parse input.
		input.expect_arguments(gasometer, 1)?;

		let owner: H160 = input.read::<Address>(gasometer)?.into();

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(context.caller);
			let owner = Runtime::AddressMapping::into_account_id(owner);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::transfer_ownership {
					id: asset_id,
					owner: Runtime::Lookup::unlookup(owner),
				},
				gasometer,
			)?;
		}

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(true).build(),
			logs: Default::default(),
		})
	}

	fn set_team(
		asset_id: AssetIdOf<Runtime, Instance>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		if !IsLocal::get() {
			return Err(gasometer.revert("unknown selector"));
		}

		// Parse input.
		input.expect_arguments(gasometer, 3)?;

		let issuer: H160 = input.read::<Address>(gasometer)?.into();
		let admin: H160 = input.read::<Address>(gasometer)?.into();
		let freezer: H160 = input.read::<Address>(gasometer)?.into();

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(context.caller);
			let issuer = Runtime::AddressMapping::into_account_id(issuer);
			let admin = Runtime::AddressMapping::into_account_id(admin);
			let freezer = Runtime::AddressMapping::into_account_id(freezer);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::set_team {
					id: asset_id,
					issuer: Runtime::Lookup::unlookup(issuer),
					admin: Runtime::Lookup::unlookup(admin),
					freezer: Runtime::Lookup::unlookup(freezer),
				},
				gasometer,
			)?;
		}

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(true).build(),
			logs: Default::default(),
		})
	}

	fn set_metadata(
		asset_id: AssetIdOf<Runtime, Instance>,
		input: &mut EvmDataReader,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		if !IsLocal::get() {
			return Err(gasometer.revert("unknown selector"));
		}

		// Parse input.
		input.expect_arguments(gasometer, 3)?;

		let name: Bytes = input.read::<Bytes>(gasometer)?.into();
		let symbol: Bytes = input.read::<Bytes>(gasometer)?.into();
		let decimals: u8 = input.read::<u8>(gasometer)?.into();

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(context.caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::set_metadata {
					id: asset_id,
					name: name.into(),
					symbol: symbol.into(),
					decimals,
				},
				gasometer,
			)?;
		}

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(true).build(),
			logs: Default::default(),
		})
	}

	fn clear_metadata(
		asset_id: AssetIdOf<Runtime, Instance>,
		gasometer: &mut Gasometer,
		context: &Context,
	) -> EvmResult<PrecompileOutput> {
		if !IsLocal::get() {
			return Err(gasometer.revert("unknown selector"));
		}

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(context.caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::clear_metadata { id: asset_id },
				gasometer,
			)?;
		}

		// Build output.
		Ok(PrecompileOutput {
			exit_status: ExitSucceed::Returned,
			cost: gasometer.used_gas(),
			output: EvmDataWriter::new().write(true).build(),
			logs: Default::default(),
		})
	}
}
