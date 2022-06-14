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
use frame_support::parameter_types;
use moonbeam_relay_encoder::kusama::KusamaEncoder;
use pallet_author_mapping_precompiles::AuthorMappingWrapper;
use pallet_democracy_precompiles::DemocracyWrapper;
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
use precompile_utils::precompile_set::*;
use relay_encoder_precompiles::RelayEncoderWrapper;
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
/// to Erc20AssetsPrecompileSet being marked as foreign
pub const FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[255u8; 4];
/// The asset precompile address prefix. Addresses that match against this prefix will be routed
/// to Erc20AssetsPrecompileSet being marked as local
pub const LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX: &[u8] = &[255u8, 255u8, 255u8, 254u8];

parameter_types! {
	pub ForeignAssetPrefix: &'static [u8] = FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX;
	pub LocalAssetPrefix: &'static [u8] = LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX;
}

/// The PrecompileSet installed in the Moonriver runtime.
/// We include the nine Istanbul precompiles
/// (https://github.com/ethereum/go-ethereum/blob/3c46f557/core/vm/contracts.go#L69)
/// as well as a special precompile for dispatching Substrate extrinsics
/// The following distribution has been decided for the precompiles
/// 0-1023: Ethereum Mainnet Precompiles
/// 1024-2047 Precompiles that are not in Ethereum Mainnet but are neither Moonbeam specific
/// 2048-4095 Moonbeam specific precompiles
pub type MoonriverPrecompiles<R> = PrecompileSetFromTuple<
	R,
	(
		// Ethereum precompiles:
		// We allow DELEGATECALL to stay compliant with Ethereum behavior.
		// They don't do any recursion, so NoRecursionLimit is safe to use to skip recursion check.
		StatelessPrecompile<AddressU64<1>, ECRecover, NoRecursionLimit, AllowDelegateCall>,
		StatelessPrecompile<AddressU64<2>, Sha256, NoRecursionLimit, AllowDelegateCall>,
		StatelessPrecompile<AddressU64<3>, Ripemd160, NoRecursionLimit, AllowDelegateCall>,
		StatelessPrecompile<AddressU64<4>, Modexp, NoRecursionLimit, AllowDelegateCall>,
		StatelessPrecompile<AddressU64<5>, Identity, NoRecursionLimit, AllowDelegateCall>,
		StatelessPrecompile<AddressU64<6>, Bn128Add, NoRecursionLimit, AllowDelegateCall>,
		StatelessPrecompile<AddressU64<7>, Bn128Mul, NoRecursionLimit, AllowDelegateCall>,
		StatelessPrecompile<AddressU64<8>, Bn128Pairing, NoRecursionLimit, AllowDelegateCall>,
		StatelessPrecompile<AddressU64<9>, Blake2F, NoRecursionLimit, AllowDelegateCall>,
		// Non-Moonbeam specific nor Ethereum precompiles :
		StatelessPrecompile<AddressU64<1024>, Sha3FIPS256>,
		StatelessPrecompile<AddressU64<1025>, Dispatch<R>>,
		StatelessPrecompile<AddressU64<1026>, ECRecoverPublicKey>,
		// Moonbeam specific precompiles:
		StatelessPrecompile<AddressU64<2048>, ParachainStakingWrapper<R>>,
		StatelessPrecompile<AddressU64<2049>, CrowdloanRewardsWrapper<R>>,
		StatelessPrecompile<AddressU64<2050>, Erc20BalancesPrecompile<R, NativeErc20Metadata>>,
		StatelessPrecompile<AddressU64<2051>, DemocracyWrapper<R>>,
		StatelessPrecompile<AddressU64<2052>, XtokensWrapper<R>>,
		StatelessPrecompile<AddressU64<2053>, RelayEncoderWrapper<R, KusamaEncoder>>,
		StatelessPrecompile<AddressU64<2054>, XcmTransactorWrapper<R>>,
		StatelessPrecompile<AddressU64<2055>, AuthorMappingWrapper<R>>,
		StatelessPrecompile<AddressU64<2056>, BatchPrecompile<R>, LimitRecursionTo<3>>,
		// Prefixed precompile sets (XC20)
		PrefixedPrecompileSet<
			ForeignAssetPrefix,
			Erc20AssetsPrecompileSet<R, IsForeign, ForeignAssetInstance>,
		>,
		PrefixedPrecompileSet<
			LocalAssetPrefix,
			Erc20AssetsPrecompileSet<R, IsLocal, LocalAssetInstance>,
		>,
	),
>;
