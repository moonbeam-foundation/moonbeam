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

use crowdloan_rewards_precompiles::CrowdloanRewardsWrapper;
use evm::{executor::PrecompileOutput, Context, ExitError};
use pallet_democracy_precompiles::DemocracyWrapper;
use pallet_evm::{AddressMapping, Precompile, PrecompileSet};
use pallet_evm_precompile_assets_erc20::{AccountIdToAssetId, Erc20AssetsPrecompile};
use pallet_evm_precompile_balances_erc20::Erc20BalancesPrecompile;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use parachain_staking_precompiles::ParachainStakingWrapper;
use sp_core::H160;
use sp_runtime::traits::Zero;
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;

/// The PrecompileSet installed in the Moonbase runtime.
/// We include the nine Istanbul precompiles
/// (https://github.com/ethereum/go-ethereum/blob/3c46f557/core/vm/contracts.go#L69)
/// as well as a special precompile for dispatching Substrate extrinsics
#[derive(Debug, Clone, Copy)]
pub struct MoonbasePrecompiles<R>(PhantomData<R>);

impl<R> AccountIdToAssetId<R::AccountId, R::AssetId> for MoonbasePrecompiles<R>
where
	R: pallet_assets::Config,
	R::AccountId: Into<H160>,
	R::AssetId: From<u128>,
{
	/// The way to convert an account to assetId is by ensuring that the prefix is 0XFFFFFFFF
	/// and by taking the lowest 128 bits as the assetId
	fn account_to_asset_id(account: R::AccountId) -> Option<R::AssetId> {
		let h160_account: H160 = account.into();
		let mut data = [0u8; 16];
		let (prefix_part, id_part) = h160_account.as_fixed_bytes().split_at(4);
		if prefix_part == &[255u8; 4] {
			data.copy_from_slice(id_part);
			let asset_id: R::AssetId = u128::from_be_bytes(data).into();
			Some(asset_id)
		} else {
			None
		}
	}
}

impl<R> MoonbasePrecompiles<R>
where
	R: pallet_evm::Config,
{
	/// Return all addresses that contain precompiles. This can be used to populate dummy code
	/// under the precompile.
	pub fn used_addresses() -> impl Iterator<Item = R::AccountId> {
		sp_std::vec![1, 2, 3, 4, 5, 6, 7, 8, 1024, 1025, 1026, 2048, 2049, 2050]
			.into_iter()
			.map(|x| R::AddressMapping::into_account_id(hash(x)))
	}
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

impl<R> PrecompileSet for MoonbasePrecompiles<R>
where
	// TODO remove this first trait bound once https://github.com/paritytech/frontier/pull/472 lands
	R: pallet_evm::Config,
	// Needed for AccountIdToAssetId
	R: pallet_assets::Config,
	Dispatch<R>: Precompile,
	ParachainStakingWrapper<R>: Precompile,
	CrowdloanRewardsWrapper<R>: Precompile,
	Erc20BalancesPrecompile<R>: Precompile,
	Erc20AssetsPrecompile<R>: Precompile,
	DemocracyWrapper<R>: Precompile,
	// Ensure we can convert from accountId to assetId
	R::Precompiles: AccountIdToAssetId<
		<R as frame_system::Config>::AccountId,
		<R as pallet_assets::Config>::AssetId,
	>,
{
	fn execute(
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
	) -> Option<Result<PrecompileOutput, ExitError>> {
		match address {
			// Ethereum precompiles :
			a if a == hash(1) => Some(ECRecover::execute(input, target_gas, context)),
			a if a == hash(2) => Some(Sha256::execute(input, target_gas, context)),
			a if a == hash(3) => Some(Ripemd160::execute(input, target_gas, context)),
			a if a == hash(5) => Some(Modexp::execute(input, target_gas, context)),
			a if a == hash(4) => Some(Identity::execute(input, target_gas, context)),
			a if a == hash(6) => Some(Bn128Add::execute(input, target_gas, context)),
			a if a == hash(7) => Some(Bn128Mul::execute(input, target_gas, context)),
			a if a == hash(8) => Some(Bn128Pairing::execute(input, target_gas, context)),
			// Non-Moonbeam specific nor Ethereum precompiles :
			a if a == hash(1024) => Some(Sha3FIPS256::execute(input, target_gas, context)),
			a if a == hash(1025) => Some(Dispatch::<R>::execute(input, target_gas, context)),
			a if a == hash(1026) => Some(ECRecoverPublicKey::execute(input, target_gas, context)),
			// Moonbeam specific precompiles :
			a if a == hash(2048) => Some(ParachainStakingWrapper::<R>::execute(
				input, target_gas, context,
			)),
			a if a == hash(2049) => Some(CrowdloanRewardsWrapper::<R>::execute(
				input, target_gas, context,
			)),
			a if a == hash(2050) => Some(Erc20BalancesPrecompile::<R>::execute(
				input, target_gas, context,
			)),
			a if a == hash(2051) => {
				Some(DemocracyWrapper::<R>::execute(input, target_gas, context))
			}
			_ => {
				// If address starts with 0XFFFFFFFF
				if let Some(asset_id) =
					R::Precompiles::account_to_asset_id(R::AddressMapping::into_account_id(address))
				{
					// If the assetId has non-zero supply
					// "total_supply" returns both 0 if the assetId does not exist or if the supply is 0
					// The assumption I am making here is that a 0 supply asset is not interesting from
					// the perspective of the precompiles. Once pallet-assets has more publicly accesible
					// storage we can use another function for this, like check_asset_existence.
					// The other options is to check the asset existence in pallet-asset-manager, but
					// this makes the precompiles dependent on such a pallet, which is not ideal
					if !pallet_assets::Pallet::<R>::total_supply(asset_id).is_zero() {
						return Some(Erc20AssetsPrecompile::<R>::execute(
							input, target_gas, context,
						));
					}
				}
				None
			}
		}
	}
}

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}
