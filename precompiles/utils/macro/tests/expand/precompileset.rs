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

use {
	core::marker::PhantomData,
	precompile_utils::{EvmResult, prelude::*, testing::PrecompileTesterExt},
	sp_core::H160
};

// Based on Erc20AssetsPrecompileSet with stripped code.

struct PrecompileSet<Runtime>(PhantomData<Runtime>);

type Discriminant = u32;
type GetAssetsStringLimit<R> = R;
type MockRuntime = ConstU32<42>;

#[precompile_utils_macro::precompile]
#[precompile::precompile_set]
#[precompile::test_concrete_types(MockRuntime)]
impl<Runtime> PrecompileSet<Runtime>
where
	Runtime: Get<u32>
{
	/// PrecompileSet discrimiant. Allows to knows if the address maps to an asset id,
	/// and if this is the case which one.
	#[precompile::discriminant]
	fn discriminant(address: H160) -> Option<Discriminant> {
		todo!("discriminant")
	}

	#[precompile::public("totalSupply()")]
	fn total_supply(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<U256> {
		todo!("total_supply")
	}

	#[precompile::public("balanceOf(address)")]
	fn balance_of(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
		who: Address,
	) -> EvmResult<U256> {
		todo!("balance_of")
	}

	#[precompile::public("allowance(address,address)")]
	fn allowance(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
		owner: Address,
		spender: Address,
	) -> EvmResult<U256> {
		todo!("allowance")
	}

	#[precompile::public("approve(address,uint256)")]
	fn approve(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
		spender: Address,
		value: U256,
	) -> EvmResult<bool> {
		todo!("approve")
	}

	fn approve_inner(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
		owner: H160,
		spender: H160,
		value: U256,
	) -> EvmResult {
		todo!("approve_inner")
	}

	#[precompile::public("transfer(address,uint256)")]
	fn transfer(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
		to: Address,
		value: U256,
	) -> EvmResult<bool> {
		todo!("transfer")
	}

	#[precompile::public("transferFrom(address,address,uint256)")]
	fn transfer_from(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
		from: Address,
		to: Address,
		value: U256,
	) -> EvmResult<bool> {
		todo!("transfer_from")
	}

	#[precompile::public("name()")]
	fn name(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<UnboundedBytes> {
		todo!("name")
	}

	#[precompile::public("symbol()")]
	fn symbol(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<UnboundedBytes> {
		todo!("symbol")
	}

	#[precompile::public("decimals()")]
	fn decimals(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<u8> {
		todo!("decimals")
	}

	// From here: only for locals, we need to check whether we are in local assets otherwise fail
	#[precompile::public("mint(address,uint256)")]
	fn mint(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
		to: Address,
		value: U256,
	) -> EvmResult<bool> {
		todo!("mint")
	}

	#[precompile::public("burn(address,uint256)")]
	fn burn(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
		from: Address,
		value: U256,
	) -> EvmResult<bool> {
		todo!("burn")
	}

	#[precompile::public("freeze(address)")]
	fn freeze(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
		account: Address,
	) -> EvmResult<bool> {
		todo!("freeze")
	}

	#[precompile::public("thaw(address)")]
	fn thaw(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
		account: Address,
	) -> EvmResult<bool> {
		todo!("thaw")
	}

	#[precompile::public("freezeAsset()")]
	#[precompile::public("freeze_asset()")]
	fn freeze_asset(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<bool> {
		todo!("freeze_asset")
	}

	#[precompile::public("thawAsset()")]
	#[precompile::public("thaw_asset()")]
	fn thaw_asset(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<bool> {
		todo!("thaw_asset")
	}

	#[precompile::public("transferOwnership(address)")]
	#[precompile::public("transfer_ownership(address)")]
	fn transfer_ownership(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
		owner: Address,
	) -> EvmResult<bool> {
		todo!("transfer_ownership")
	}

	#[precompile::public("setTeam(address,address,address)")]
	#[precompile::public("set_team(address,address,address)")]
	fn set_team(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
		issuer: Address,
		admin: Address,
		freezer: Address,
	) -> EvmResult<bool> {
		todo!("set_team")
	}

	#[precompile::public("setMetadata(string,string,uint8)")]
	#[precompile::public("set_metadata(string,string,uint8)")]
	fn set_metadata(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
		name: BoundedString<GetAssetsStringLimit<Runtime>>,
		symbol: BoundedString<GetAssetsStringLimit<Runtime>>,
		decimals: u8,
	) -> EvmResult<bool> {
		todo!("set_metadata")
	}

	#[precompile::public("clearMetadata()")]
	#[precompile::public("clear_metadata()")]
	fn clear_metadata(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<bool> {
		todo!("clear_metadata")
	}

	#[precompile::public("permit(address,address,uint256,uint256,uint8,bytes32,bytes32)")]
	fn eip2612_permit(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
		owner: Address,
		spender: Address,
		value: U256,
		deadline: U256,
		v: u8,
		r: H256,
		s: H256,
	) -> EvmResult {
		todo!("eip2612_permit")
	}

	#[precompile::public("nonces(address)")]
	#[precompile::view]
	fn eip2612_nonces(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
		owner: Address,
	) -> EvmResult<U256> {
		todo!("eip2612_nonces")
	}

	#[precompile::public("DOMAIN_SEPARATOR()")]
	#[precompile::view]
	fn eip2612_domain_separator(
		asset_id: Discriminant,
		handle: &mut impl PrecompileHandle,
	) -> EvmResult<H256> {
		todo!("eip2612_domain_separator")
	}
}