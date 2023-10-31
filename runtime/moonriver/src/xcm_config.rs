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

//! XCM configuration for Moonbase.
//!

use super::{
	governance, AccountId, AssetId, AssetManager, Assets, Balance, Balances, DealWithFees,
	Erc20XcmBridge, LocalAssets, ParachainInfo, ParachainSystem, PolkadotXcm, Runtime, RuntimeCall,
	RuntimeEvent, RuntimeOrigin, Treasury, XcmpQueue, FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX,
};

use frame_support::{
	parameter_types,
	traits::{EitherOfDiverse, Everything, Nothing, PalletInfoAccess},
};
use moonbeam_runtime_common::weights as moonbeam_weights;
use pallet_evm_precompileset_assets_erc20::AccountIdAssetIdConversion;
use sp_runtime::{
	traits::{Hash as THash, MaybeEquivalence, PostDispatchInfoOf},
	DispatchErrorWithPostInfo,
};
use sp_weights::Weight;

use frame_system::{EnsureRoot, RawOrigin};
use sp_core::{ConstU32, H160, H256};

use xcm_builder::{
	AccountKey20Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, AsPrefixedGeneralIndex, ConvertedConcreteId,
	CurrencyAdapter as XcmCurrencyAdapter, EnsureXcmOrigin, FungiblesAdapter, NoChecking,
	ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
	SignedAccountKey20AsNative, SovereignSignedViaLocation, TakeWeightCredit, UsingComponents,
	WeightInfoBounds, WithComputedOrigin,
};

use xcm::latest::prelude::*;
use xcm_executor::traits::{CallDispatcher, ConvertLocation, JustTry};

use orml_xcm_support::MultiNativeAsset;
use xcm_primitives::{
	AbsoluteAndRelativeReserve, AccountIdToCurrencyId, AccountIdToMultiLocation, AsAssetType,
	FirstAssetTrader, SignedToAccountId20, UtilityAvailableCalls, UtilityEncodeCall, XcmTransact,
};

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

use sp_std::{
	convert::{From, Into, TryFrom},
	prelude::*,
};

use orml_traits::parameter_type_with_key;

use crate::governance::referenda::GeneralAdminOrRoot;

parameter_types! {
	// The network Id of the relay
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	// The relay chain Origin type
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	// The universal location within the global consensus system
	pub UniversalLocation: InteriorMultiLocation =
		X2(GlobalConsensus(RelayNetwork::get()), Parachain(ParachainInfo::parachain_id().into()));

	// Self Reserve location, defines the multilocation identifiying the self-reserve currency
	// This is used to match it also against our Balances pallet when we receive such
	// a MultiLocation: (Self Balances pallet index)
	// We use the RELATIVE multilocation
	pub SelfReserve: MultiLocation = MultiLocation {
		parents:0,
		interior: Junctions::X1(
			PalletInstance(<Balances as PalletInfoAccess>::index() as u8)
		)
	};

	// This is the relative view of our local assets.
	// Indentified by thix prefix + generalIndex(assetId)
	// We use the RELATIVE multilocation
	pub LocalAssetsPalletLocation: MultiLocation = MultiLocation {
		parents:0,
		interior: Junctions::X1(
			PalletInstance(<LocalAssets as PalletInfoAccess>::index() as u8)
		)
	};
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<polkadot_parachain::primitives::Sibling, AccountId>,
	// If we receive a MultiLocation of type AccountKey20, just generate a native account
	AccountKey20Aliases<RelayNetwork, AccountId>,
	// Generate remote accounts according to polkadot standards
	xcm_builder::HashedDescriptionDescribeFamilyAllTerminal<AccountId>,
);

/// Wrapper type around `LocationToAccountId` to convert an `AccountId` to type `H160`.
pub struct LocationToH160;
impl ConvertLocation<H160> for LocationToH160 {
	fn convert_location(location: &MultiLocation) -> Option<H160> {
		<LocationToAccountId as ConvertLocation<AccountId>>::convert_location(location)
			.map(Into::into)
	}
}

// The non-reserve fungible transactor type
// It will use pallet-assets, and the Id will be matched against AsAssetType
pub type ForeignFungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	Assets,
	// Use this currency when it is a fungible asset matching the given location or name:
	(
		ConvertedConcreteId<
			AssetId,
			Balance,
			AsAssetType<AssetId, AssetType, AssetManager>,
			JustTry,
		>,
	),
	// Do a simple punn to convert an AccountId20 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We dont allow teleports.
	NoChecking,
	// We dont track any teleports
	(),
>;

/// The transactor for our own chain currency.
pub type LocalAssetTransactor = XcmCurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching any of the locations in
	// SelfReserveRepresentations
	xcm_builder::IsConcrete<SelfReserve>,
	// We can convert the MultiLocations with our converter above:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We dont allow teleport
	(),
>;

/// Means for transacting local assets besides the native currency on this chain.
pub type LocalFungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	LocalAssets,
	// Use this currency when it is a fungible asset matching the given location or name:
	(
		ConvertedConcreteId<
			AssetId,
			Balance,
			// This just tells to convert an assetId into a GeneralIndex junction prepended
			// by LocalAssetsPalletLocationNewReanchor
			AsPrefixedGeneralIndex<LocalAssetsPalletLocation, AssetId, JustTry>,
			JustTry,
		>,
	),
	// Convert an XCM MultiLocation into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We dont want to allow teleporting assets
	NoChecking,
	// The account to use for tracking teleports.
	(),
>;

// We use all transactors
// These correspond to
// SelfReserve asset, both pre and post 0.9.16
// Foreign assets
// Local assets, both pre and post 0.9.16
// We can remove the Old reanchor once
// we import https://github.com/open-web3-stack/open-runtime-module-library/pull/708
pub type AssetTransactors = (
	LocalAssetTransactor,
	ForeignFungiblesTransactor,
	LocalFungiblesTransactor,
	Erc20XcmBridge,
);

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognised.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	pallet_xcm::XcmPassthrough<RuntimeOrigin>,
	// Xcm Origins defined by a Multilocation of type AccountKey20 can be converted to a 20 byte-
	// account local origin
	SignedAccountKey20AsNative<RelayNetwork, RuntimeOrigin>,
);

parameter_types! {
	/// The amount of weight an XCM operation takes. This is safe overestimate.
	pub UnitWeightCost: Weight = Weight::from_parts(200_000_000u64, 0);
	/// Maximum number of instructions in a single XCM fragment. A sanity check against
	/// weight caculations getting too crazy.
	pub MaxInstructions: u32 = 100;
}

/// Xcm Weigher shared between multiple Xcm-related configs.
pub type XcmWeigher = WeightInfoBounds<
	moonbeam_xcm_benchmarks::weights::XcmWeight<Runtime, RuntimeCall>,
	RuntimeCall,
	MaxInstructions,
>;

pub type XcmBarrier = (
	// Weight that is paid for may be consumed.
	TakeWeightCredit,
	// Expected responses are OK.
	AllowKnownQueryResponses<PolkadotXcm>,
	WithComputedOrigin<
		(
			// If the message is one that immediately attemps to pay for execution, then allow it.
			AllowTopLevelPaidExecutionFrom<Everything>,
			// Subscriptions for version tracking are OK.
			AllowSubscriptionsFrom<Everything>,
		),
		UniversalLocation,
		ConstU32<8>,
	>,
);

parameter_types! {
	/// Xcm fees will go to the treasury account
	pub XcmFeesAccount: AccountId = Treasury::account_id();
}

/// This is the struct that will handle the revenue from xcm fees
/// We do not burn anything because we want to mimic exactly what
/// the sovereign account has
pub type XcmFeesToAccount = xcm_primitives::XcmFeesToAccount<
	Assets,
	(
		ConvertedConcreteId<
			AssetId,
			Balance,
			AsAssetType<AssetId, AssetType, AssetManager>,
			JustTry,
		>,
	),
	AccountId,
	XcmFeesAccount,
>;

pub struct SafeCallFilter;
impl frame_support::traits::Contains<RuntimeCall> for SafeCallFilter {
	fn contains(_call: &RuntimeCall) -> bool {
		// TODO review
		// This needs to be addressed at EVM level
		true
	}
}

parameter_types! {
	pub const MaxAssetsIntoHolding: u32 = xcm_primitives::MAX_ASSETS;
}

// Our implementation of the Moonbeam Call
// Attachs the right origin in case the call is made to pallet-ethereum-xcm
#[cfg(not(feature = "evm-tracing"))]
moonbeam_runtime_common::impl_moonbeam_xcm_call!();
#[cfg(feature = "evm-tracing")]
moonbeam_runtime_common::impl_moonbeam_xcm_call_tracing!();

moonbeam_runtime_common::impl_evm_runner_precompile_or_eth_xcm!();

pub struct XcmExecutorConfig;
impl xcm_executor::Config for XcmExecutorConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = AssetTransactors;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	// Filter to the reserve withdraw operations
	// Whenever the reserve matches the relative or absolute value
	// of our chain, we always return the relative reserve
	type IsReserve = MultiNativeAsset<AbsoluteAndRelativeReserve<SelfLocationAbsolute>>;
	type IsTeleporter = (); // No teleport
	type UniversalLocation = UniversalLocation;
	type Barrier = XcmBarrier;
	type Weigher = XcmWeigher;
	// We use two traders
	// When we receive the relative representation of the self-reserve asset,
	// we use UsingComponents and the local way of handling fees
	// When we receive a non-reserve asset, we use AssetManager to fetch how many
	// units per second we should charge
	type Trader = (
		UsingComponents<
			<Runtime as pallet_transaction_payment::Config>::WeightToFee,
			SelfReserve,
			AccountId,
			Balances,
			DealWithFees<Runtime>,
		>,
		FirstAssetTrader<AssetType, AssetManager, XcmFeesToAccount>,
	);
	type ResponseHandler = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type AssetTrap = pallet_erc20_xcm_bridge::AssetTrapWrapper<PolkadotXcm, Runtime>;
	type AssetClaims = PolkadotXcm;
	type CallDispatcher = MoonbeamCall;
	type PalletInstancesInfo = crate::AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type AssetLocker = ();
	type AssetExchanger = ();
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type SafeCallFilter = SafeCallFilter;
	type Aliasers = Nothing;
}

type XcmExecutor = pallet_erc20_xcm_bridge::XcmExecutorWrapper<
	RuntimeCall,
	xcm_executor::XcmExecutor<XcmExecutorConfig>,
>;

// Converts a Signed Local Origin into a MultiLocation
pub type LocalOriginToLocation = SignedToAccountId20<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Nothing;
	type XcmExecutor = XcmExecutor;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type Weigher = XcmWeigher;
	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = LocationToAccountId;
	type MaxLockers = ConstU32<8>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	// TODO pallet-xcm weights
	type WeightInfo = moonbeam_weights::pallet_xcm::WeightInfo<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
	type AdminOrigin = EnsureRoot<AccountId>;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = moonbeam_weights::cumulus_pallet_xcmp_queue::WeightInfo<Runtime>;
	type PriceForSiblingDelivery = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

// Our AssetType. For now we only handle Xcm Assets
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum AssetType {
	Xcm(MultiLocation),
}
impl Default for AssetType {
	fn default() -> Self {
		Self::Xcm(MultiLocation::here())
	}
}

impl From<MultiLocation> for AssetType {
	fn from(location: MultiLocation) -> Self {
		Self::Xcm(location)
	}
}
impl Into<Option<MultiLocation>> for AssetType {
	fn into(self) -> Option<MultiLocation> {
		match self {
			Self::Xcm(location) => Some(location),
		}
	}
}

// Implementation on how to retrieve the AssetId from an AssetType
// We simply hash the AssetType and take the lowest 128 bits
impl From<AssetType> for AssetId {
	fn from(asset: AssetType) -> AssetId {
		match asset {
			AssetType::Xcm(id) => {
				let mut result: [u8; 16] = [0u8; 16];
				let hash: H256 = id.using_encoded(<Runtime as frame_system::Config>::Hashing::hash);
				result.copy_from_slice(&hash.as_fixed_bytes()[0..16]);
				u128::from_le_bytes(result)
			}
		}
	}
}

// Our currencyId. We distinguish for now between SelfReserve, and Others, defined by their Id.
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum CurrencyId {
	// Our native token
	SelfReserve,
	// Assets representing other chains native tokens
	ForeignAsset(AssetId),
	// Our local assets
	LocalAssetReserve(AssetId),
	// Erc20 token
	Erc20 { contract_address: H160 },
}

impl AccountIdToCurrencyId<AccountId, CurrencyId> for Runtime {
	fn account_to_currency_id(account: AccountId) -> Option<CurrencyId> {
		Some(match account {
			// the self-reserve currency is identified by the pallet-balances address
			a if a == H160::from_low_u64_be(2050).into() => CurrencyId::SelfReserve,
			// the rest of the currencies, by their corresponding erc20 address
			_ => match Runtime::account_to_asset_id(account) {
				// We distinguish by prefix, and depending on it we create either
				// Foreign or Local
				Some((prefix, asset_id)) => {
					if prefix == FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX.to_vec() {
						CurrencyId::ForeignAsset(asset_id)
					} else {
						CurrencyId::LocalAssetReserve(asset_id)
					}
				}
				// If no known prefix is identified, we consider that it's a "real" erc20 token
				// (i.e. managed by a real smart contract)
				None => CurrencyId::Erc20 {
					contract_address: account.into(),
				},
			},
		})
	}
}

// How to convert from CurrencyId to MultiLocation
pub struct CurrencyIdtoMultiLocation<AssetXConverter>(sp_std::marker::PhantomData<AssetXConverter>);
impl<AssetXConverter> sp_runtime::traits::Convert<CurrencyId, Option<MultiLocation>>
	for CurrencyIdtoMultiLocation<AssetXConverter>
where
	AssetXConverter: MaybeEquivalence<MultiLocation, AssetId>,
{
	fn convert(currency: CurrencyId) -> Option<MultiLocation> {
		match currency {
			// For now and until Xtokens is adapted to handle 0.9.16 version we use
			// the old anchoring here
			// This is not a problem in either cases, since the view of the destination
			// chain does not change
			// TODO! change this to NewAnchoringSelfReserve once xtokens is adapted for it
			CurrencyId::SelfReserve => {
				let multi: MultiLocation = SelfReserve::get();
				Some(multi)
			}
			CurrencyId::ForeignAsset(asset) => AssetXConverter::convert_back(&asset),
			CurrencyId::LocalAssetReserve(asset) => {
				let mut location = LocalAssetsPalletLocation::get();
				location.push_interior(Junction::GeneralIndex(asset)).ok();
				Some(location)
			}
			CurrencyId::Erc20 { contract_address } => {
				let mut location = Erc20XcmBridgePalletLocation::get();
				location
					.push_interior(Junction::AccountKey20 {
						key: contract_address.0,
						network: None,
					})
					.ok();
				Some(location)
			}
		}
	}
}

parameter_types! {
	pub const BaseXcmWeight: Weight = Weight::from_parts(200_000_000u64, 0);
	pub const MaxAssetsForTransfer: usize = 2;

	// This is how we are going to detect whether the asset is a Reserve asset
	// This however is the chain part only
	pub SelfLocation: MultiLocation = MultiLocation::here();
	// We need this to be able to catch when someone is trying to execute a non-
	// cross-chain transfer in xtokens through the absolute path way
	pub SelfLocationAbsolute: MultiLocation = MultiLocation {
		parents:1,
		interior: Junctions::X1(
			Parachain(ParachainInfo::parachain_id().into())
		)
	};
}

parameter_type_with_key! {
	pub ParachainMinFee: |location: MultiLocation| -> Option<u128> {
		match (location.parents, location.first_interior()) {
			// Kusama AssetHub fee
			(1, Some(Parachain(1000u32))) => Some(50_000_000u128),
			_ => None,
		}
	};
}

impl orml_xtokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = AccountIdToMultiLocation<AccountId>;
	type CurrencyIdConvert =
		CurrencyIdtoMultiLocation<AsAssetType<AssetId, AssetType, AssetManager>>;
	type XcmExecutor = XcmExecutor;
	type SelfLocation = SelfLocation;
	type Weigher = XcmWeigher;
	type BaseXcmWeight = BaseXcmWeight;
	type UniversalLocation = UniversalLocation;
	type MaxAssetsForTransfer = MaxAssetsForTransfer;
	type MinXcmFee = ParachainMinFee;
	type MultiLocationsFilter = Everything;
	type ReserveProvider = AbsoluteAndRelativeReserve<SelfLocationAbsolute>;
}

// 1 KSM should be enough
parameter_types! {
	pub MaxHrmpRelayFee: MultiAsset = (MultiLocation::parent(), 1_000_000_000_000u128).into();
}

// For now we only allow to transact in the relay, although this might change in the future
// Transactors just defines the chains in which we allow transactions to be issued through
// xcm
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum Transactors {
	Relay,
}

// Default for benchmarking
#[cfg(feature = "runtime-benchmarks")]
impl Default for Transactors {
	fn default() -> Self {
		Transactors::Relay
	}
}

impl TryFrom<u8> for Transactors {
	type Error = ();
	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			0u8 => Ok(Transactors::Relay),
			_ => Err(()),
		}
	}
}

impl UtilityEncodeCall for Transactors {
	fn encode_call(self, call: UtilityAvailableCalls) -> Vec<u8> {
		match self {
			Transactors::Relay => pallet_xcm_transactor::Pallet::<Runtime>::encode_call(
				pallet_xcm_transactor::Pallet(sp_std::marker::PhantomData::<Runtime>),
				call,
			),
		}
	}
}

impl XcmTransact for Transactors {
	fn destination(self) -> MultiLocation {
		match self {
			Transactors::Relay => MultiLocation::parent(),
		}
	}
}

pub type DerivativeAddressRegistrationOrigin =
	EitherOfDiverse<EnsureRoot<AccountId>, governance::custom_origins::GeneralAdmin>;

impl pallet_xcm_transactor::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Transactor = Transactors;
	type DerivativeAddressRegistrationOrigin = DerivativeAddressRegistrationOrigin;
	type SovereignAccountDispatcherOrigin = EnsureRoot<AccountId>;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = AccountIdToMultiLocation<AccountId>;
	type CurrencyIdToMultiLocation =
		CurrencyIdtoMultiLocation<AsAssetType<AssetId, AssetType, AssetManager>>;
	type XcmSender = XcmRouter;
	type SelfLocation = SelfLocation;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type BaseXcmWeight = BaseXcmWeight;
	type AssetTransactor = AssetTransactors;
	type ReserveProvider = AbsoluteAndRelativeReserve<SelfLocationAbsolute>;
	type WeightInfo = moonbeam_weights::pallet_xcm_transactor::WeightInfo<Runtime>;
	type HrmpManipulatorOrigin = GeneralAdminOrRoot;
	type MaxHrmpFee = xcm_builder::Case<MaxHrmpRelayFee>;
}

parameter_types! {
	// This is the relative view of erc20 assets.
	// Identified by this prefix + AccountKey20(contractAddress)
	// We use the RELATIVE multilocation
	pub Erc20XcmBridgePalletLocation: MultiLocation = MultiLocation {
		parents:0,
		interior: Junctions::X1(
			PalletInstance(<Erc20XcmBridge as PalletInfoAccess>::index() as u8)
		)
	};

	// To be able to support almost all erc20 implementations,
	// we provide a sufficiently hight gas limit.
	pub Erc20XcmBridgeTransferGasLimit: u64 = 200_000;
}

impl pallet_erc20_xcm_bridge::Config for Runtime {
	type AccountIdConverter = LocationToH160;
	type Erc20MultilocationPrefix = Erc20XcmBridgePalletLocation;
	type Erc20TransferGasLimit = Erc20XcmBridgeTransferGasLimit;
	type EvmRunner = EvmRunnerPrecompileOrEthXcm<MoonbeamCall, Self>;
}

#[cfg(feature = "runtime-benchmarks")]
mod testing {
	use super::*;

	/// This From exists for benchmarking purposes. It has the potential side-effect of calling
	/// AssetManager::set_asset_type_asset_id() and should NOT be used in any production code.
	impl From<MultiLocation> for CurrencyId {
		fn from(location: MultiLocation) -> CurrencyId {
			use xcm_primitives::AssetTypeGetter;

			// If it does not exist, for benchmarking purposes, we create the association
			let asset_id = if let Some(asset_id) =
				AsAssetType::<AssetId, AssetType, AssetManager>::convert_location(&location)
			{
				asset_id
			} else {
				let asset_type = AssetType::Xcm(location);
				let asset_id: AssetId = asset_type.clone().into();
				AssetManager::set_asset_type_asset_id(asset_type, asset_id);
				asset_id
			};

			CurrencyId::ForeignAsset(asset_id)
		}
	}
}
