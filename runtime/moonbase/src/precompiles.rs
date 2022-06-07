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
use moonbeam_relay_encoder::westend::WestendEncoder;
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
use precompile_utils::revert;
use relay_encoder_precompiles::RelayEncoderWrapper;
use sp_core::H160;
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;
use xcm_transactor_precompiles::XcmTransactorWrapper;
use xtokens_precompiles::XtokensWrapper;

/// ERC20 metadata for the native token.
pub struct NativeErc20Metadata;

impl Erc20Metadata for NativeErc20Metadata {
	/// Returns the name of the token.
	fn name() -> &'static str {
		"DEV token"
	}

	/// Returns the symbol of the token.
	fn symbol() -> &'static str {
		"DEV"
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
/// to Erc20AssetsPrecompileSet being marked as foreign
pub const FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[255u8; 4];
/// The asset precompile address prefix. Addresses that match against this prefix will be routed
/// to Erc20AssetsPrecompileSet being marked as local
pub const LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[255u8, 255u8, 255u8, 254u8];

/// The PrecompileSet installed in the Moonbase runtime.
/// We include the nine Istanbul precompiles
/// (https://github.com/ethereum/go-ethereum/blob/3c46f557/core/vm/contracts.go#L69)
/// as well as a special precompile for dispatching Substrate extrinsics
#[derive(Debug, Clone, Copy)]
pub struct MoonbasePrecompiles<R>(PhantomData<R>);

impl<R> MoonbasePrecompiles<R>
where
	R: pallet_evm::Config,
{
	pub fn new() -> Self {
		Self(Default::default())
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

impl<R> PrecompileSet for MoonbasePrecompiles<R>
where
	Dispatch<R>: Precompile,
	ParachainStakingWrapper<R>: Precompile,
	CrowdloanRewardsWrapper<R>: Precompile,
	Erc20BalancesPrecompile<R, NativeErc20Metadata>: Precompile,
	// We require PrecompileSet here because indeed we are dealing with a set of precompiles
	// This precompile set does additional checks, e.g., total supply not being 0
	Erc20AssetsPrecompileSet<R, IsForeign, ForeignAssetInstance>: PrecompileSet,
	Erc20AssetsPrecompileSet<R, IsLocal, LocalAssetInstance>: PrecompileSet,
	DemocracyWrapper<R>: Precompile,
	XtokensWrapper<R>: Precompile,
	RelayEncoderWrapper<R, WestendEncoder>: Precompile,
	XcmTransactorWrapper<R>: Precompile,
	AuthorMappingWrapper<R>: Precompile,
	R: pallet_evm::Config,
{
	fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
		// Filter known precompile addresses except Ethereum officials
		if self.is_precompile(handle.code_address())
			&& handle.code_address() > hash(9)
			&& handle.code_address() != handle.context().address
		{
			return Some(Err(revert(
				"cannot be called with DELEGATECALL or CALLCODE",
			)));
		}

		match handle.code_address() {
			// Ethereum precompiles :
			a if a == hash(1) => Some(ECRecover::execute(handle)),
			a if a == hash(2) => Some(Sha256::execute(handle)),
			a if a == hash(3) => Some(Ripemd160::execute(handle)),
			a if a == hash(5) => Some(Modexp::execute(handle)),
			a if a == hash(4) => Some(Identity::execute(handle)),
			a if a == hash(6) => Some(Bn128Add::execute(handle)),
			a if a == hash(7) => Some(Bn128Mul::execute(handle)),
			a if a == hash(8) => Some(Bn128Pairing::execute(handle)),
			a if a == hash(9) => Some(Blake2F::execute(handle)),

			// Non-Moonbeam specific nor Ethereum precompiles :
			a if a == hash(1024) => Some(Sha3FIPS256::execute(handle)),
			a if a == hash(1025) => Some(Dispatch::<R>::execute(handle)),
			a if a == hash(1026) => Some(ECRecoverPublicKey::execute(handle)),

			// Moonbeam specific precompiles :
			a if a == hash(2048) => Some(ParachainStakingWrapper::<R>::execute(handle)),
			a if a == hash(2049) => Some(CrowdloanRewardsWrapper::<R>::execute(handle)),
			a if a == hash(2050) => Some(
				Erc20BalancesPrecompile::<R, NativeErc20Metadata>::execute(handle),
			),
			a if a == hash(2051) => Some(DemocracyWrapper::<R>::execute(handle)),
			a if a == hash(2052) => Some(XtokensWrapper::<R>::execute(handle)),
			a if a == hash(2053) => Some(RelayEncoderWrapper::<R, WestendEncoder>::execute(handle)),
			a if a == hash(2054) => Some(XcmTransactorWrapper::<R>::execute(handle)),
			a if a == hash(2055) => Some(AuthorMappingWrapper::<R>::execute(handle)),
			a if a == hash(2056) => Some(BatchPrecompile::<R>::execute(handle)),
			// If the address matches asset prefix, the we route through the asset precompile set
			a if &a.to_fixed_bytes()[0..4] == FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX => {
				Erc20AssetsPrecompileSet::<R, IsForeign, ForeignAssetInstance>::new()
					.execute(handle)
			}
			// If the address matches asset prefix, the we route through the asset precompile set
			a if &a.to_fixed_bytes()[0..4] == LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX => {
				Erc20AssetsPrecompileSet::<R, IsLocal, LocalAssetInstance>::new().execute(handle)
			}
			_ => None,
		}
	}
	fn is_precompile(&self, address: H160) -> bool {
		Self::used_addresses().any(|x| x == R::AddressMapping::into_account_id(address))
			|| Erc20AssetsPrecompileSet::<R, IsForeign, ForeignAssetInstance>::new()
				.is_precompile(address)
			|| Erc20AssetsPrecompileSet::<R, IsLocal, LocalAssetInstance>::new()
				.is_precompile(address)
	}
}

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}
