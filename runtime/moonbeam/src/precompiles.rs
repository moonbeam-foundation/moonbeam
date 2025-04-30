// Copyright 2019-2025 PureStake Inc.
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

use super::moonbeam_weights;
use crate::{
	asset_config::ForeignAssetInstance,
	xcm_config::{AssetType, XcmExecutorConfig},
	AccountId, AssetId, AssetManager, Balances, Erc20XcmBridge, EvmForeignAssets,
	OpenTechCommitteeInstance, Runtime, TreasuryCouncilInstance, H160,
};
use frame_support::parameter_types;
use moonkit_xcm_primitives::{
	location_matcher::{Erc20PalletMatcher, ForeignAssetMatcher, SingleAddressMatcher},
	AccountIdAssetIdConversion,
};
use pallet_evm_precompile_author_mapping::AuthorMappingPrecompile;
use pallet_evm_precompile_balances_erc20::{Erc20BalancesPrecompile, Erc20Metadata};
use pallet_evm_precompile_batch::BatchPrecompile;
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_call_permit::CallPermitPrecompile;
use pallet_evm_precompile_collective::CollectivePrecompile;
use pallet_evm_precompile_conviction_voting::ConvictionVotingPrecompile;
use pallet_evm_precompile_crowdloan_rewards::CrowdloanRewardsPrecompile;
use pallet_evm_precompile_gmp::GmpPrecompile;
use pallet_evm_precompile_identity::IdentityPrecompile;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_p256verify::P256Verify;
use pallet_evm_precompile_parachain_staking::ParachainStakingPrecompile;
use pallet_evm_precompile_preimage::PreimagePrecompile;
use pallet_evm_precompile_proxy::{OnlyIsProxyAndProxy, ProxyPrecompile};
use pallet_evm_precompile_randomness::RandomnessPrecompile;
use pallet_evm_precompile_referenda::ReferendaPrecompile;
use pallet_evm_precompile_registry::PrecompileRegistry;
use pallet_evm_precompile_relay_encoder::RelayEncoderPrecompile;
use pallet_evm_precompile_relay_verifier::RelayDataVerifierPrecompile;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use pallet_evm_precompile_xcm::PalletXcmPrecompile;
use pallet_evm_precompile_xcm_transactor::{
	v1::XcmTransactorPrecompileV1, v2::XcmTransactorPrecompileV2, v3::XcmTransactorPrecompileV3,
};
use pallet_evm_precompile_xcm_utils::XcmUtilsPrecompile;
use pallet_evm_precompile_xtokens::XtokensPrecompile;
use pallet_evm_precompileset_assets_erc20::Erc20AssetsPrecompileSet;
use pallet_precompile_benchmarks::WeightInfo;
use precompile_utils::precompile_set::*;
use sp_std::prelude::*;
use xcm_primitives::AsAssetType;

parameter_types! {
	pub P256VerifyWeight: frame_support::weights::Weight =
		moonbeam_weights::pallet_precompile_benchmarks::WeightInfo::<Runtime>::p256_verify();
}

pub struct NativeErc20Metadata;

/// ERC20 metadata for the native token.
impl Erc20Metadata for NativeErc20Metadata {
	/// Returns the name of the token.
	fn name() -> &'static str {
		"GLMR token"
	}

	/// Returns the symbol of the token.
	fn symbol() -> &'static str {
		"GLMR"
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

/// Const to identify ERC20_BALANCES_PRECOMPILE address
pub const ERC20_BALANCES_PRECOMPILE: u64 = 2050;

parameter_types! {
	pub ForeignAssetPrefix: &'static [u8] = FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX;
	pub LocalAssetPrefix: &'static [u8] = LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX;
}

type EthereumPrecompilesChecks = (AcceptDelegateCall, CallableByContract, CallableByPrecompile);

// Pallet-xcm precompile types.
// Type that converts AssetId into Location
type AssetIdToLocationManager = (
	AsAssetType<AssetId, AssetType, AssetManager>,
	EvmForeignAssets,
);

// The pallet-balances address is identified by ERC20_BALANCES_PRECOMPILE const
type SingleAddressMatch = SingleAddressMatcher<AccountId, ERC20_BALANCES_PRECOMPILE, Balances>;

// Type that matches an AccountId with a foreign asset address (if any)
type ForeignAssetMatch = ForeignAssetMatcher<AccountId, AssetId, Runtime, AssetIdToLocationManager>;

// Erc20XcmBridge pallet is used to match ERC20s
type Erc20Match = Erc20PalletMatcher<AccountId, Erc20XcmBridge>;

#[precompile_utils::precompile_name_from_address]
type MoonbeamPrecompilesAt<R> = (
	// Ethereum precompiles:
	// We allow DELEGATECALL to stay compliant with Ethereum behavior.
	PrecompileAt<AddressU64<1>, ECRecover, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<2>, Sha256, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<3>, Ripemd160, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<4>, Identity, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<5>, Modexp, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<6>, Bn128Add, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<7>, Bn128Mul, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<8>, Bn128Pairing, EthereumPrecompilesChecks>,
	PrecompileAt<AddressU64<9>, Blake2F, EthereumPrecompilesChecks>,
	// (0x100 => 256) https://github.com/ethereum/RIPs/blob/master/RIPS/rip-7212.md
	PrecompileAt<AddressU64<256>, P256Verify<P256VerifyWeight>, EthereumPrecompilesChecks>,
	// Non-Moonbeam specific nor Ethereum precompiles :
	PrecompileAt<AddressU64<1024>, Sha3FIPS256, (CallableByContract, CallableByPrecompile)>,
	RemovedPrecompileAt<AddressU64<1025>>, // Dispatch<R>
	PrecompileAt<AddressU64<1026>, ECRecoverPublicKey, (CallableByContract, CallableByPrecompile)>,
	// Moonbeam specific precompiles:
	PrecompileAt<
		AddressU64<2048>,
		ParachainStakingPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2049>,
		CrowdloanRewardsPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<ERC20_BALANCES_PRECOMPILE>,
		Erc20BalancesPrecompile<R, NativeErc20Metadata>,
		(CallableByContract, CallableByPrecompile),
	>,
	RemovedPrecompileAt<AddressU64<2051>>, // Democracy
	PrecompileAt<
		AddressU64<2052>,
		XtokensPrecompile<R>,
		(
			SubcallWithMaxNesting<1>,
			CallableByContract,
			CallableByPrecompile,
		),
	>,
	PrecompileAt<
		AddressU64<2053>,
		RelayEncoderPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2054>,
		XcmTransactorPrecompileV1<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2055>,
		AuthorMappingPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2056>,
		BatchPrecompile<R>,
		(
			SubcallWithMaxNesting<2>,
			// Batch is the only precompile allowed to call Batch.
			CallableByPrecompile<OnlyFrom<AddressU64<2056>>>,
		),
	>,
	PrecompileAt<
		AddressU64<2057>,
		RandomnessPrecompile<R>,
		(SubcallWithMaxNesting<0>, CallableByContract),
	>,
	PrecompileAt<
		AddressU64<2058>,
		CallPermitPrecompile<R>,
		(SubcallWithMaxNesting<0>, CallableByContract),
	>,
	PrecompileAt<
		AddressU64<2059>,
		ProxyPrecompile<R>,
		(
			CallableByContract<OnlyIsProxyAndProxy<R>>,
			SubcallWithMaxNesting<0>,
			// Batch is the only precompile allowed to call Proxy.
			CallableByPrecompile<OnlyFrom<AddressU64<2056>>>,
		),
	>,
	PrecompileAt<
		AddressU64<2060>,
		XcmUtilsPrecompile<R, XcmExecutorConfig>,
		CallableByContract<
			pallet_evm_precompile_xcm_utils::AllExceptXcmExecute<R, XcmExecutorConfig>,
		>,
	>,
	PrecompileAt<
		AddressU64<2061>,
		XcmTransactorPrecompileV2<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	RemovedPrecompileAt<AddressU64<2062>>, //CouncilInstance
	RemovedPrecompileAt<AddressU64<2063>>, // TechCommitteeInstance
	PrecompileAt<
		AddressU64<2064>,
		CollectivePrecompile<R, TreasuryCouncilInstance>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2065>,
		ReferendaPrecompile<R, crate::governance::custom_origins::Origin>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2066>,
		ConvictionVotingPrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2067>,
		PreimagePrecompile<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2068>,
		CollectivePrecompile<R, OpenTechCommitteeInstance>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2069>,
		PrecompileRegistry<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<AddressU64<2070>, GmpPrecompile<R>, SubcallWithMaxNesting<0>>,
	PrecompileAt<
		AddressU64<2071>,
		XcmTransactorPrecompileV3<R>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2072>,
		IdentityPrecompile<R, crate::MaxAdditionalFields>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2073>,
		RelayDataVerifierPrecompile<
			R,
			moonbeam_weights::pallet_precompile_benchmarks::WeightInfo<Runtime>,
		>,
		(CallableByContract, CallableByPrecompile),
	>,
	PrecompileAt<
		AddressU64<2074>,
		PalletXcmPrecompile<R, (SingleAddressMatch, ForeignAssetMatch, Erc20Match)>,
		(
			CallableByContract,
			CallableByPrecompile,
			SubcallWithMaxNesting<1>,
		),
	>,
);

pub struct DisabledLocalAssets<Runtime>(sp_std::marker::PhantomData<Runtime>);

impl<Runtime> sp_core::Get<Vec<H160>> for DisabledLocalAssets<Runtime>
where
	Runtime: frame_system::Config,
	Runtime::AccountId: Into<H160>,
	Runtime: AccountIdAssetIdConversion<Runtime::AccountId, AssetId>,
{
	fn get() -> Vec<H160> {
		vec![
			337110116006454532607322340792629567158u128,
			278750993613512357835566279094880339619,
			228256396637196286254896220398224702687,
			270195117769614861929703564202131636628,
		]
		.iter()
		.map(|id| Runtime::asset_id_to_account(LOCAL_ASSET_PRECOMPILE_ADDRESS_PREFIX, *id).into())
		.collect()
	}
}

/// The PrecompileSet installed in the Moonbeam runtime.
/// We include the nine Istanbul precompiles
/// (https://github.com/ethereum/go-ethereum/blob/3c46f557/core/vm/contracts.go#L69)
/// as well as a special precompile for dispatching Substrate extrinsics
/// The following distribution has been decided for the precompiles
/// 0-1023: Ethereum Mainnet Precompiles
/// 1024-2047 Precompiles that are not in Ethereum Mainnet but are neither Moonbeam specific
/// 2048-4095 Moonbeam specific precompiles
pub type MoonbeamPrecompiles<R> = PrecompileSetBuilder<
	R,
	(
		// Skip precompiles if out of range.
		PrecompilesInRangeInclusive<(AddressU64<1>, AddressU64<4095>), MoonbeamPrecompilesAt<R>>,
		// Prefixed precompile sets (XC20)
		PrecompileSetStartingWith<
			ForeignAssetPrefix,
			Erc20AssetsPrecompileSet<R, ForeignAssetInstance>,
			(CallableByContract, CallableByPrecompile),
		>,
		RemovedPrecompilesAt<DisabledLocalAssets<R>>,
	),
>;
