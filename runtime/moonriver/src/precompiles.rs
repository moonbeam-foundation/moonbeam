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

use crate::asset_config::{ForeignAssetInstance, LocalAssetInstance};
use crowdloan_rewards_precompiles::CrowdloanRewardsWrapper;
use fp_evm::PrecompileHandle;
use moonbeam_relay_encoder::kusama::KusamaEncoder;
use pallet_author_mapping_precompiles::AuthorMappingWrapper;
use pallet_democracy_precompiles::DemocracyWrapper;
use pallet_evm::{AddressMapping, Precompile, PrecompileResult, PrecompileSet};
use pallet_evm_precompile_assets_erc20::{Erc20AssetsPrecompileSet, IsForeign, IsLocal};
use pallet_evm_precompile_balances_erc20::{Erc20BalancesPrecompile, Erc20Metadata};
use pallet_evm_precompile_batch::BatchPrecompile;
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use parachain_staking_precompiles::ParachainStakingWrapper;
use precompile_utils::{precompileset::BuilderParams, PrecompileSetBuilderExt};
use relay_encoder_precompiles::RelayEncoderWrapper;
use sp_core::H160;
use sp_std::{boxed::Box, fmt::Debug, marker::PhantomData};
use xcm_transactor_precompiles::XcmTransactorWrapper;
use xtokens_precompiles::XtokensWrapper;

pub struct NativeErc20Metadata;

/// ERC20 metadata for the native token.
impl Erc20Metadata for NativeErc20Metadata {
	/// Returns the name of the token.
	fn name() -> &'static str {
		"MOVR token"
	}

	/// Returns the symbol of the token.
	fn symbol() -> &'static str {
		"MOVR"
	}

	/// Returns the decimals places of the token.
	fn decimals() -> u8 {
		18
	}

	/// Must return `true` only if it represents the main native currency of
	/// the network. It must be the currency used in `pallet_evm`.
	fn is_native_currency() -> bool {
		true
	}
}

/// The asset precompile address prefix. Addresses that match against this prefix will be routed
/// to Erc20AssetsPrecompileSet
pub const FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[255u8; 4];
pub const LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[255u8, 255u8, 255u8, 254u8];

type ChainedPrecompileSet<R>
where
	Dispatch<R>: Precompile,
	ParachainStakingWrapper<R>: Precompile,
	CrowdloanRewardsWrapper<R>: Precompile,
	Erc20BalancesPrecompile<R, NativeErc20Metadata>: Precompile,
	Erc20AssetsPrecompileSet<R, IsForeign, ForeignAssetInstance>: PrecompileSet,
	Erc20AssetsPrecompileSet<R, IsLocal, LocalAssetInstance>: PrecompileSet,
	XtokensWrapper<R>: Precompile,
	RelayEncoderWrapper<R, KusamaEncoder>: Precompile,
	XcmTransactorWrapper<R>: Precompile,
	DemocracyWrapper<R>: Precompile,
	AuthorMappingWrapper<R>: Precompile,
	R: pallet_evm::Config,
= impl PrecompileSet + Clone;

pub fn moonriver_precompiles<R>() -> ChainedPrecompileSet<R>
where
	Dispatch<R>: Precompile,
	ParachainStakingWrapper<R>: Precompile,
	CrowdloanRewardsWrapper<R>: Precompile,
	Erc20BalancesPrecompile<R, NativeErc20Metadata>: Precompile,
	Erc20AssetsPrecompileSet<R, IsForeign, ForeignAssetInstance>: PrecompileSet,
	Erc20AssetsPrecompileSet<R, IsLocal, LocalAssetInstance>: PrecompileSet,
	XtokensWrapper<R>: Precompile,
	RelayEncoderWrapper<R, KusamaEncoder>: Precompile,
	XcmTransactorWrapper<R>: Precompile,
	DemocracyWrapper<R>: Precompile,
	AuthorMappingWrapper<R>: Precompile,
	R: pallet_evm::Config,
{
	// Ethereum precompiles:
	().add_precompile::<ECRecover>(hash(1), BuilderParams::new().allow_delegatecall())
		.add_precompile::<Sha256>(hash(2), BuilderParams::new().allow_delegatecall())
		.add_precompile::<Ripemd160>(hash(3), BuilderParams::new().allow_delegatecall())
		.add_precompile::<Identity>(hash(4), BuilderParams::new().allow_delegatecall())
		.add_precompile::<Modexp>(hash(5), BuilderParams::new().allow_delegatecall())
		.add_precompile::<Bn128Add>(hash(6), BuilderParams::new().allow_delegatecall())
		.add_precompile::<Bn128Add>(hash(6), BuilderParams::new().allow_delegatecall())
		.add_precompile::<Bn128Mul>(hash(7), BuilderParams::new().allow_delegatecall())
		.add_precompile::<Bn128Pairing>(hash(8), BuilderParams::new().allow_delegatecall())
		.add_precompile::<Blake2F>(hash(9), BuilderParams::new().allow_delegatecall())
		// Non-Moonbeam specific nor Ethereum precompiles :
		.add_precompile::<Sha3FIPS256>(hash(1024), BuilderParams::new())
		.add_precompile::<Dispatch<R>>(hash(1025), BuilderParams::new())
		.add_precompile::<ECRecoverPublicKey>(hash(1026), BuilderParams::new())
		// Moonbeam specific precompiles :
		.add_precompile::<ParachainStakingWrapper<R>>(hash(2048), BuilderParams::new())
		.add_precompile::<CrowdloanRewardsWrapper<R>>(hash(2049), BuilderParams::new())
		.add_precompile::<Erc20BalancesPrecompile<R, NativeErc20Metadata>>(
			hash(2050),
			BuilderParams::new(),
		)
		.add_precompile::<DemocracyWrapper<R>>(hash(2051), BuilderParams::new())
		.add_precompile::<XtokensWrapper<R>>(hash(2052), BuilderParams::new())
		.add_precompile::<RelayEncoderWrapper<R, KusamaEncoder>>(hash(2053), BuilderParams::new())
		.add_precompile::<XcmTransactorWrapper<R>>(hash(2054), BuilderParams::new())
		.add_precompile::<AuthorMappingWrapper<R>>(hash(2055), BuilderParams::new())
		.add_precompile::<BatchPrecompile<R>>(
			hash(2056),
			BuilderParams::new().with_max_recursion(Some(3)),
		)
		.add_precompile_set(
			FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
			Erc20AssetsPrecompileSet::<R, IsForeign, ForeignAssetInstance>::new(),
		)
		.add_precompile_set(
			LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX,
			Erc20AssetsPrecompileSet::<R, IsLocal, LocalAssetInstance>::new(),
		)
}

/// The PrecompileSet installed in the Moonriver runtime.
/// We include the nine Istanbul precompiles
/// (https://github.com/ethereum/go-ethereum/blob/3c46f557/core/vm/contracts.go#L69)
/// as well as a special precompile for dispatching Substrate extrinsics
#[derive(Clone)]
pub struct MoonriverPrecompiles<R>
where
	Dispatch<R>: Precompile,
	ParachainStakingWrapper<R>: Precompile,
	CrowdloanRewardsWrapper<R>: Precompile,
	Erc20BalancesPrecompile<R, NativeErc20Metadata>: Precompile,
	Erc20AssetsPrecompileSet<R, IsForeign, ForeignAssetInstance>: PrecompileSet,
	Erc20AssetsPrecompileSet<R, IsLocal, LocalAssetInstance>: PrecompileSet,
	XtokensWrapper<R>: Precompile,
	RelayEncoderWrapper<R, KusamaEncoder>: Precompile,
	XcmTransactorWrapper<R>: Precompile,
	DemocracyWrapper<R>: Precompile,
	AuthorMappingWrapper<R>: Precompile,
	R: pallet_evm::Config,
{
	inner: ChainedPrecompileSet<R>,
}

impl<R> MoonriverPrecompiles<R>
where
	Dispatch<R>: Precompile,
	ParachainStakingWrapper<R>: Precompile,
	CrowdloanRewardsWrapper<R>: Precompile,
	Erc20BalancesPrecompile<R, NativeErc20Metadata>: Precompile,
	Erc20AssetsPrecompileSet<R, IsForeign, ForeignAssetInstance>: PrecompileSet,
	Erc20AssetsPrecompileSet<R, IsLocal, LocalAssetInstance>: PrecompileSet,
	XtokensWrapper<R>: Precompile,
	RelayEncoderWrapper<R, KusamaEncoder>: Precompile,
	XcmTransactorWrapper<R>: Precompile,
	DemocracyWrapper<R>: Precompile,
	AuthorMappingWrapper<R>: Precompile,
	R: pallet_evm::Config,
{
	pub fn new() -> Self {
		Self {
			inner: moonriver_precompiles::<R>(),
		}
	}

	/// Return all addresses that contain precompiles. This can be used to populate dummy code
	/// under the precompile.
	pub fn used_addresses() -> impl Iterator<Item = R::AccountId> {
		sp_std::vec![
			1, 2, 3, 4, 5, 6, 7, 8, 9, 1024, 1025, 1026, 2048, 2049, 2050, 2051, 2052, 2053, 2054,
			2055, 2056
		]
		.into_iter()
		.map(|x| R::AddressMapping::into_account_id(hash(x)))
	}
}

/// The following distribution has been decided for the precompiles
/// 0-1023: Ethereum Mainnet Precompiles
/// 1024-2047 Precompiles that are not in Ethereum Mainnet but are neither Moonbeam specific
/// 2048-4095 Moonbeam specific precompiles
impl<R> PrecompileSet for MoonriverPrecompiles<R>
where
	Dispatch<R>: Precompile,
	ParachainStakingWrapper<R>: Precompile,
	CrowdloanRewardsWrapper<R>: Precompile,
	Erc20BalancesPrecompile<R, NativeErc20Metadata>: Precompile,
	Erc20AssetsPrecompileSet<R, IsForeign, ForeignAssetInstance>: PrecompileSet,
	Erc20AssetsPrecompileSet<R, IsLocal, LocalAssetInstance>: PrecompileSet,
	XtokensWrapper<R>: Precompile,
	RelayEncoderWrapper<R, KusamaEncoder>: Precompile,
	XcmTransactorWrapper<R>: Precompile,
	DemocracyWrapper<R>: Precompile,
	AuthorMappingWrapper<R>: Precompile,
	R: pallet_evm::Config,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		self.inner.execute(handle)
	}
	fn is_precompile(&self, address: H160) -> bool {
		self.inner.is_precompile(address)
	}
}

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}
