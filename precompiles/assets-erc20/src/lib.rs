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

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(test, feature(assert_matches))]

use core::fmt::Display;
use fp_evm::PrecompileHandle;
use frame_support::traits::fungibles::approvals::Inspect as ApprovalInspect;
use frame_support::traits::fungibles::metadata::Inspect as MetadataInspect;
use frame_support::traits::fungibles::Inspect;
use frame_support::traits::{ConstBool, Get, OriginTrait};
use frame_support::{
	dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo},
	sp_runtime::traits::StaticLookup,
};
use pallet_evm::AddressMapping;
use precompile_utils::prelude::*;
use sp_runtime::traits::Bounded;
use sp_std::vec::Vec;

use sp_core::{H160, H256, U256};
use sp_std::{
	convert::{TryFrom, TryInto},
	marker::PhantomData,
};

mod eip2612;
use eip2612::Eip2612;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// Solidity selector of the Transfer log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_TRANSFER: [u8; 32] = keccak256!("Transfer(address,address,uint256)");

/// Solidity selector of the Approval log, which is the Keccak of the Log signature.
pub const SELECTOR_LOG_APPROVAL: [u8; 32] = keccak256!("Approval(address,address,uint256)");

/// Length limit of strings (symbol and name).
type GetAssetsStringLimit<R, I> = <R as pallet_assets::Config<I>>::StringLimit;

/// Alias for the Balance type for the provided Runtime and Instance.
pub type BalanceOf<Runtime, Instance = ()> = <Runtime as pallet_assets::Config<Instance>>::Balance;

/// Alias for the Asset Id type for the provided Runtime and Instance.
pub type AssetIdOf<Runtime, Instance = ()> = <Runtime as pallet_assets::Config<Instance>>::AssetId;

/// Public types to use with the PrecompileSet
pub type IsLocal = ConstBool<true>;
pub type IsForeign = ConstBool<false>;

/// This trait ensure we can convert AccountIds to AssetIds
/// We will require Runtime to have this trait implemented
pub trait AccountIdAssetIdConversion<Account, AssetId> {
	// Get assetId and prefix from account
	fn account_to_asset_id(account: Account) -> Option<(Vec<u8>, AssetId)>;

	// Get AccountId from AssetId and prefix
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

impl<T, U, V> Clone for Erc20AssetsPrecompileSet<T, U, V> {
	fn clone(&self) -> Self {
		Self(PhantomData)
	}
}

impl<T, U, V> Default for Erc20AssetsPrecompileSet<T, U, V> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<Runtime, IsLocal, Instance> Erc20AssetsPrecompileSet<Runtime, IsLocal, Instance> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

#[precompile_utils::precompile]
#[precompile::precompile_set]
#[precompile::test_concrete_types(mock::Runtime, IsForeign, pallet_assets::Instance1)]
impl<Runtime, IsLocal, Instance> Erc20AssetsPrecompileSet<Runtime, IsLocal, Instance>
where
	Instance: eip2612::InstanceToPrefix + 'static,
	Runtime: pallet_assets::Config<Instance>
		+ pallet_evm::Config
		+ frame_system::Config
		+ pallet_timestamp::Config,
	Runtime::RuntimeCall: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	Runtime::RuntimeCall: From<pallet_assets::Call<Runtime, Instance>>,
	<Runtime::RuntimeCall as Dispatchable>::RuntimeOrigin: From<Option<Runtime::AccountId>>,
	BalanceOf<Runtime, Instance>: TryFrom<U256> + Into<U256> + EvmData,
	Runtime: AccountIdAssetIdConversion<Runtime::AccountId, AssetIdOf<Runtime, Instance>>,
	<<Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin: OriginTrait,
	IsLocal: Get<bool>,
	<Runtime as pallet_timestamp::Config>::Moment: Into<U256>,
	AssetIdOf<Runtime, Instance>: Display,
{
	/// PrecompileSet discrimiant. Allows to knows if the address maps to an asset id,
	/// and if this is the case which one.
	#[precompile::discriminant]
	fn discriminant(address: H160) -> Option<AssetIdOf<Runtime, Instance>> {
		let account_id = Runtime::AddressMapping::into_account_id(address);
		let asset_id = match Runtime::account_to_asset_id(account_id) {
			Some((_, asset_id)) => asset_id,
			None => return None,
		};

		if pallet_assets::Pallet::<Runtime, Instance>::maybe_total_supply(asset_id).is_some() {
			Some(asset_id)
		} else {
			None
		}
	}

	#[precompile::public("totalSupply()")]
	#[precompile::view]
	fn total_supply(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<U256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		Ok(pallet_assets::Pallet::<Runtime, Instance>::total_issuance(asset_id).into())
	}

	#[precompile::public("balanceOf(address)")]
	#[precompile::view]
	fn balance_of(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		who: Address,
	) -> EvmResult<U256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let who: H160 = who.into();

		// Fetch info.
		let amount: U256 = {
			let who: Runtime::AccountId = Runtime::AddressMapping::into_account_id(who);
			pallet_assets::Pallet::<Runtime, Instance>::balance(asset_id, &who).into()
		};

		// Build output.
		Ok(amount)
	}

	#[precompile::public("allowance(address,address)")]
	#[precompile::view]
	fn allowance(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		owner: Address,
		spender: Address,
	) -> EvmResult<U256> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let owner: H160 = owner.into();
		let spender: H160 = spender.into();

		// Fetch info.
		let amount: U256 = {
			let owner: Runtime::AccountId = Runtime::AddressMapping::into_account_id(owner);
			let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);

			// Fetch info.
			pallet_assets::Pallet::<Runtime, Instance>::allowance(asset_id, &owner, &spender).into()
		};

		// Build output.
		Ok(amount)
	}

	#[precompile::public("approve(address,uint256)")]
	fn approve(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		spender: Address,
		value: U256,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let spender: H160 = spender.into();

		Self::approve_inner(asset_id, handle, handle.context().caller, spender, value)?;

		log3(
			handle.context().address,
			SELECTOR_LOG_APPROVAL,
			handle.context().caller,
			spender,
			EvmDataWriter::new().write(value).build(),
		)
		.record(handle)?;

		// Build output.
		Ok(true)
	}

	fn approve_inner(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		owner: H160,
		spender: H160,
		value: U256,
	) -> EvmResult {
		let owner = Runtime::AddressMapping::into_account_id(owner);
		let spender: Runtime::AccountId = Runtime::AddressMapping::into_account_id(spender);
		// Amount saturate if too high.
		let amount: BalanceOf<Runtime, Instance> =
			value.try_into().unwrap_or_else(|_| Bounded::max_value());

		// Allowance read
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		// If previous approval exists, we need to clean it
		if pallet_assets::Pallet::<Runtime, Instance>::allowance(asset_id, &owner, &spender)
			!= 0u32.into()
		{
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(owner.clone()).into(),
				pallet_assets::Call::<Runtime, Instance>::cancel_approval {
					id: asset_id,
					delegate: Runtime::Lookup::unlookup(spender.clone()),
				},
			)?;
		}
		// Dispatch call (if enough gas).
		RuntimeHelper::<Runtime>::try_dispatch(
			handle,
			Some(owner).into(),
			pallet_assets::Call::<Runtime, Instance>::approve_transfer {
				id: asset_id,
				delegate: Runtime::Lookup::unlookup(spender),
				amount,
			},
		)?;

		Ok(())
	}

	#[precompile::public("transfer(address,uint256)")]
	fn transfer(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		to: Address,
		value: U256,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let to: H160 = to.into();
		let value = Self::u256_to_amount(value).in_field("value")?;

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			let to = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::transfer {
					id: asset_id,
					target: Runtime::Lookup::unlookup(to),
					amount: value,
				},
			)?;
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_TRANSFER,
			handle.context().caller,
			to,
			EvmDataWriter::new().write(value).build(),
		)
		.record(handle)?;

		Ok(true)
	}

	#[precompile::public("transferFrom(address,address,uint256)")]
	fn transfer_from(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		from: Address,
		to: Address,
		value: U256,
	) -> EvmResult<bool> {
		handle.record_log_costs_manual(3, 32)?;

		let from: H160 = from.into();
		let to: H160 = to.into();
		let value = Self::u256_to_amount(value).in_field("value")?;

		{
			let caller: Runtime::AccountId =
				Runtime::AddressMapping::into_account_id(handle.context().caller);
			let from: Runtime::AccountId = Runtime::AddressMapping::into_account_id(from.clone());
			let to: Runtime::AccountId = Runtime::AddressMapping::into_account_id(to);

			// If caller is "from", it can spend as much as it wants from its own balance.
			if caller != from {
				// Dispatch call (if enough gas).
				RuntimeHelper::<Runtime>::try_dispatch(
					handle,
					Some(caller).into(),
					pallet_assets::Call::<Runtime, Instance>::transfer_approved {
						id: asset_id,
						owner: Runtime::Lookup::unlookup(from),
						destination: Runtime::Lookup::unlookup(to),
						amount: value,
					},
				)?;
			} else {
				// Dispatch call (if enough gas).
				RuntimeHelper::<Runtime>::try_dispatch(
					handle,
					Some(from).into(),
					pallet_assets::Call::<Runtime, Instance>::transfer {
						id: asset_id,
						target: Runtime::Lookup::unlookup(to),
						amount: value,
					},
				)?;
			}
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_TRANSFER,
			from,
			to,
			EvmDataWriter::new().write(value).build(),
		)
		.record(handle)?;

		// Build output.
		Ok(true)
	}

	#[precompile::public("name()")]
	#[precompile::view]
	fn name(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<UnboundedBytes> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let name = pallet_assets::Pallet::<Runtime, Instance>::name(asset_id)
			.as_slice()
			.into();

		Ok(name)
	}

	#[precompile::public("symbol()")]
	#[precompile::view]
	fn symbol(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<UnboundedBytes> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		let symbol = pallet_assets::Pallet::<Runtime, Instance>::symbol(asset_id)
			.as_slice()
			.into();

		Ok(symbol)
	}

	#[precompile::public("decimals()")]
	#[precompile::view]
	fn decimals(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<u8> {
		handle.record_cost(RuntimeHelper::<Runtime>::db_read_gas_cost())?;

		Ok(pallet_assets::Pallet::<Runtime, Instance>::decimals(
			asset_id,
		))
	}

	// From here: only for locals, we need to check whether we are in local assets otherwise fail
	#[precompile::public("mint(address,uint256)")]
	fn mint(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		to: Address,
		value: U256,
	) -> EvmResult<bool> {
		if !IsLocal::get() {
			return Err(RevertReason::UnknownSelector.into());
		}

		handle.record_log_costs_manual(3, 32)?;

		let to: H160 = to.into();
		let value = Self::u256_to_amount(value).in_field("value")?;

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			let to = Runtime::AddressMapping::into_account_id(to);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::mint {
					id: asset_id,
					beneficiary: Runtime::Lookup::unlookup(to),
					amount: value,
				},
			)?;
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_TRANSFER,
			H160::default(),
			to,
			EvmDataWriter::new().write(value).build(),
		)
		.record(handle)?;

		Ok(true)
	}

	#[precompile::public("burn(address,uint256)")]
	fn burn(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		from: Address,
		value: U256,
	) -> EvmResult<bool> {
		if !IsLocal::get() {
			return Err(RevertReason::UnknownSelector.into());
		}

		handle.record_log_costs_manual(3, 32)?;

		let from: H160 = from.into();
		let value = Self::u256_to_amount(value).in_field("value")?;

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			let from = Runtime::AddressMapping::into_account_id(from);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::burn {
					id: asset_id,
					who: Runtime::Lookup::unlookup(from),
					amount: value,
				},
			)?;
		}

		log3(
			handle.context().address,
			SELECTOR_LOG_TRANSFER,
			from,
			H160::default(),
			EvmDataWriter::new().write(value).build(),
		)
		.record(handle)?;

		Ok(true)
	}

	#[precompile::public("freeze(address)")]
	fn freeze(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		account: Address,
	) -> EvmResult<bool> {
		if !IsLocal::get() {
			return Err(RevertReason::UnknownSelector.into());
		}

		let account: H160 = account.into();

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			let account = Runtime::AddressMapping::into_account_id(account);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::freeze {
					id: asset_id,
					who: Runtime::Lookup::unlookup(account),
				},
			)?;
		}

		Ok(true)
	}

	#[precompile::public("thaw(address)")]
	fn thaw(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		account: Address,
	) -> EvmResult<bool> {
		if !IsLocal::get() {
			return Err(RevertReason::UnknownSelector.into());
		}

		let account: H160 = account.into();

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			let account = Runtime::AddressMapping::into_account_id(account);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::thaw {
					id: asset_id,
					who: Runtime::Lookup::unlookup(account),
				},
			)?;
		}

		Ok(true)
	}

	#[precompile::public("freezeAsset()")]
	#[precompile::public("freeze_asset()")]
	fn freeze_asset(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<bool> {
		if !IsLocal::get() {
			return Err(RevertReason::UnknownSelector.into());
		}

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::freeze_asset { id: asset_id },
			)?;
		}

		Ok(true)
	}

	#[precompile::public("thawAsset()")]
	#[precompile::public("thaw_asset()")]
	fn thaw_asset(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<bool> {
		if !IsLocal::get() {
			return Err(RevertReason::UnknownSelector.into());
		}

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::thaw_asset { id: asset_id },
			)?;
		}

		// Build output.
		Ok(true)
	}

	#[precompile::public("transferOwnership(address)")]
	#[precompile::public("transfer_ownership(address)")]
	fn transfer_ownership(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		owner: Address,
	) -> EvmResult<bool> {
		if !IsLocal::get() {
			return Err(RevertReason::UnknownSelector.into());
		}

		let owner: H160 = owner.into();

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			let owner = Runtime::AddressMapping::into_account_id(owner);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::transfer_ownership {
					id: asset_id,
					owner: Runtime::Lookup::unlookup(owner),
				},
			)?;
		}

		Ok(true)
	}

	#[precompile::public("setTeam(address,address,address)")]
	#[precompile::public("set_team(address,address,address)")]
	fn set_team(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		issuer: Address,
		admin: Address,
		freezer: Address,
	) -> EvmResult<bool> {
		if !IsLocal::get() {
			return Err(RevertReason::UnknownSelector.into());
		}

		let issuer: H160 = issuer.into();
		let admin: H160 = admin.into();
		let freezer: H160 = freezer.into();

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);
			let issuer = Runtime::AddressMapping::into_account_id(issuer);
			let admin = Runtime::AddressMapping::into_account_id(admin);
			let freezer = Runtime::AddressMapping::into_account_id(freezer);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::set_team {
					id: asset_id,
					issuer: Runtime::Lookup::unlookup(issuer),
					admin: Runtime::Lookup::unlookup(admin),
					freezer: Runtime::Lookup::unlookup(freezer),
				},
			)?;
		}

		Ok(true)
	}

	#[precompile::public("setMetadata(string,string,uint8)")]
	#[precompile::public("set_metadata(string,string,uint8)")]
	fn set_metadata(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		name: BoundedString<GetAssetsStringLimit<Runtime, Instance>>,
		symbol: BoundedString<GetAssetsStringLimit<Runtime, Instance>>,
		decimals: u8,
	) -> EvmResult<bool> {
		if !IsLocal::get() {
			return Err(RevertReason::UnknownSelector.into());
		}

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::set_metadata {
					id: asset_id,
					name: name.into(),
					symbol: symbol.into(),
					decimals,
				},
			)?;
		}

		Ok(true)
	}

	#[precompile::public("clearMetadata()")]
	#[precompile::public("clear_metadata()")]
	fn clear_metadata(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<bool> {
		if !IsLocal::get() {
			return Err(RevertReason::UnknownSelector.into());
		}

		// Build call with origin.
		{
			let origin = Runtime::AddressMapping::into_account_id(handle.context().caller);

			// Dispatch call (if enough gas).
			RuntimeHelper::<Runtime>::try_dispatch(
				handle,
				Some(origin).into(),
				pallet_assets::Call::<Runtime, Instance>::clear_metadata { id: asset_id },
			)?;
		}

		Ok(true)
	}

	#[precompile::public("permit(address,address,uint256,uint256,uint8,bytes32,bytes32)")]
	fn eip2612_permit(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		owner: Address,
		spender: Address,
		value: U256,
		deadline: U256,
		v: u8,
		r: H256,
		s: H256,
	) -> EvmResult {
		<Eip2612<Runtime, IsLocal, Instance>>::permit(
			asset_id, handle, owner, spender, value, deadline, v, r, s,
		)
	}

	#[precompile::public("nonces(address)")]
	#[precompile::view]
	fn eip2612_nonces(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
		owner: Address,
	) -> EvmResult<U256> {
		<Eip2612<Runtime, IsLocal, Instance>>::nonces(asset_id, handle, owner)
	}

	#[precompile::public("DOMAIN_SEPARATOR()")]
	#[precompile::view]
	fn eip2612_domain_separator(
		asset_id: AssetIdOf<Runtime, Instance>,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<H256> {
		<Eip2612<Runtime, IsLocal, Instance>>::domain_separator(asset_id, handle)
	}

	fn u256_to_amount(value: U256) -> MayRevert<BalanceOf<Runtime, Instance>> {
		value
			.try_into()
			.map_err(|_| RevertReason::value_is_too_large("balance type").into())
	}
}
