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
	AccountId, AssetId, AssetManager, Assets, Balance, Balances, Call, DealWithFees, Event,
	LocalAssets, Origin, ParachainInfo, ParachainSystem, PolkadotXcm, Runtime, Treasury,
	WeightToFee, XcmpQueue, FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX, MAXIMUM_BLOCK_WEIGHT,
};

use pallet_evm_precompile_assets_erc20::AccountIdAssetIdConversion;
use sp_runtime::traits::Hash as THash;

use frame_support::{
	parameter_types,
	traits::{Everything, Nothing, PalletInfoAccess},
	weights::Weight,
};

use frame_system::EnsureRoot;
use sp_core::{H160, H256};

use xcm_builder::{
	AccountKey20Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, AsPrefixedGeneralIndex, ConvertedConcreteAssetId,
	CurrencyAdapter as XcmCurrencyAdapter, EnsureXcmOrigin, FixedWeightBounds, FungiblesAdapter,
	LocationInverter, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountKey20AsNative, SovereignSignedViaLocation,
	TakeWeightCredit, UsingComponents,
};

use xcm::latest::prelude::*;
use xcm_executor::traits::JustTry;

use xcm_primitives::{
	AccountIdToCurrencyId, AccountIdToMultiLocation, AsAssetType, FirstAssetTrader,
	MultiNativeAsset, SignedToAccountId20, UtilityAvailableCalls, UtilityEncodeCall, XcmTransact,
};

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

use sp_std::{
	convert::{From, Into, TryFrom},
	prelude::*,
};

use orml_traits::{location::AbsoluteReserveProvider, parameter_type_with_key};

parameter_types! {
	// The network Id of the relay
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	// The relay chain Origin type
	pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
	// The ancestry, defines the multilocation describing this consensus system
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
	// Old Self Reserve location, defines the multilocation identifiying the self-reserve currency
	// This is used to match it against our Balances pallet when we receive such a MultiLocation
	// (Parent, Self Para Id, Self Balances pallet index)
	// This is the old anchoring way
	pub OldAnchoringSelfReserve: MultiLocation = MultiLocation {
		parents:1,
		interior: Junctions::X2(
			Parachain(ParachainInfo::parachain_id().into()),
			PalletInstance(<Balances as PalletInfoAccess>::index() as u8)
		)
	};
	// New Self Reserve location, defines the multilocation identifiying the self-reserve currency
	// This is used to match it also against our Balances pallet when we receive such
	// a MultiLocation: (Self Balances pallet index)
	// This is the new anchoring way
	pub NewAnchoringSelfReserve: MultiLocation = MultiLocation {
		parents:0,
		interior: Junctions::X1(
			PalletInstance(<Balances as PalletInfoAccess>::index() as u8)
		)
	};

	// The Locations we accept to refer to our own currency. We need to support both pre and
	// post 0.9.16 versions, hence the reason for this being a Vec
	pub SelfReserveRepresentations: Vec<MultiLocation> = vec![
		OldAnchoringSelfReserve::get(),
		NewAnchoringSelfReserve::get()
	];

	// Old reanchor logic location for pallet assets
	// We need to support both in case we talk to a chain not in 0.9.16
	// Or until we import https://github.com/open-web3-stack/open-runtime-module-library/pull/708
	// We will be able to remove this once we import the aforementioned change
	// Indentified by thix prefix + generalIndex(assetId)
	pub LocalAssetsPalletLocationOldReanchor: MultiLocation = MultiLocation {
		parents:1,
		interior: Junctions::X2(
			Parachain(ParachainInfo::parachain_id().into()),
			PalletInstance(<LocalAssets as PalletInfoAccess>::index() as u8)
		)
	};

	// New reanchor logic location for pallet assets
	// This is the relative view of our local assets. This is the representation that will
	// be considered canonical after we import
	// https://github.com/open-web3-stack/open-runtime-module-library/pull/708
	// Indentified by thix prefix + generalIndex(assetId)
	pub LocalAssetsPalletLocationNewReanchor: MultiLocation = MultiLocation {
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
);

// The non-reserve fungible transactor type
// It will use pallet-assets, and the Id will be matched against AsAssetType
pub type ForeignFungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	Assets,
	// Use this currency when it is a fungible asset matching the given location or name:
	(
		ConvertedConcreteAssetId<
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
	Nothing,
	// We dont track any teleports
	(),
>;

/// The transactor for our own chain currency.
pub type LocalAssetTransactor = XcmCurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching any of the locations in
	// SelfReserveRepresentations
	xcm_primitives::MultiIsConcrete<SelfReserveRepresentations>,
	// We can convert the MultiLocations with our converter above:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We dont allow teleport
	(),
>;

/// Means for transacting local assets that are not the native currency
/// This transactor uses the old reanchor logic
/// Remove once we import https://github.com/open-web3-stack/open-runtime-module-library/pull/708
pub type LocalFungiblesTransactorOldReanchor = FungiblesAdapter<
	// Use this fungibles implementation:
	LocalAssets,
	(
		ConvertedConcreteAssetId<
			AssetId,
			Balance,
			// This just tells to convert an assetId into a GeneralIndex junction prepended
			// by LocalAssetsPalletLocationOldReanchor
			AsPrefixedGeneralIndex<LocalAssetsPalletLocationOldReanchor, AssetId, JustTry>,
			JustTry,
		>,
	),
	// Convert an XCM MultiLocation into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We dont want to allow teleporting assets
	Nothing,
	// The account to use for tracking teleports.
	(),
>;

/// Means for transacting local assets besides the native currency on this chain.
pub type LocalFungiblesTransactorNewReanchor = FungiblesAdapter<
	// Use this fungibles implementation:
	LocalAssets,
	// Use this currency when it is a fungible asset matching the given location or name:
	(
		ConvertedConcreteAssetId<
			AssetId,
			Balance,
			// This just tells to convert an assetId into a GeneralIndex junction prepended
			// by LocalAssetsPalletLocationNewReanchor
			AsPrefixedGeneralIndex<LocalAssetsPalletLocationNewReanchor, AssetId, JustTry>,
			JustTry,
		>,
	),
	// Convert an XCM MultiLocation into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We dont want to allow teleporting assets
	Nothing,
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
	LocalFungiblesTransactorOldReanchor,
	LocalFungiblesTransactorNewReanchor,
);

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, Origin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognised.
	RelayChainAsNative<RelayChainOrigin, Origin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	pallet_xcm::XcmPassthrough<Origin>,
	// Xcm Origins defined by a Multilocation of type AccountKey20 can be converted to a 20 byte-
	// account local origin
	SignedAccountKey20AsNative<RelayNetwork, Origin>,
);

parameter_types! {
	/// The amount of weight an XCM operation takes. This is safe overestimate.
	pub UnitWeightCost: Weight = 200_000_000;
	/// Maximum number of instructions in a single XCM fragment. A sanity check against
	/// weight caculations getting too crazy.
	pub MaxInstructions: u32 = 100;
}

/// Xcm Weigher shared between multiple Xcm-related configs.
pub type XcmWeigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;

// Allow paid executions
pub type XcmBarrier = (
	TakeWeightCredit,
	AllowTopLevelPaidExecutionFrom<Everything>,
	// Expected responses are OK.
	AllowKnownQueryResponses<PolkadotXcm>,
	// Subscriptions for version tracking are OK.
	AllowSubscriptionsFrom<Everything>,
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
		ConvertedConcreteAssetId<
			AssetId,
			Balance,
			AsAssetType<AssetId, AssetType, AssetManager>,
			JustTry,
		>,
	),
	AccountId,
	XcmFeesAccount,
>;

pub struct XcmExecutorConfig;
impl xcm_executor::Config for XcmExecutorConfig {
	type Call = Call;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = AssetTransactors;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	// Filter to the reserve withdraw operations
	type IsReserve = MultiNativeAsset;
	type IsTeleporter = (); // No teleport
	type LocationInverter = LocationInverter<Ancestry>;
	type Barrier = XcmBarrier;
	type Weigher = XcmWeigher;
	// We use three traders
	// When we receive either representation of the self-reserve asset,
	// we use UsingComponents and the local way of handling fees
	// When we receive a non-reserve asset, we use AssetManager to fetch how many
	// units per second we should charge
	type Trader = (
		UsingComponents<
			WeightToFee,
			OldAnchoringSelfReserve,
			AccountId,
			Balances,
			DealWithFees<Runtime>,
		>,
		UsingComponents<
			WeightToFee,
			NewAnchoringSelfReserve,
			AccountId,
			Balances,
			DealWithFees<Runtime>,
		>,
		FirstAssetTrader<AssetType, AssetManager, XcmFeesToAccount>,
	);
	type ResponseHandler = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
}

type XcmExecutor = xcm_executor::XcmExecutor<XcmExecutorConfig>;

parameter_types! {
	pub const MaxDownwardMessageWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 10;
}

// Converts a Signed Local Origin into a MultiLocation
pub type LocalOriginToLocation = SignedToAccountId20<Origin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
	type Event = Event;
	type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmExecuteFilter = Nothing;
	type XcmExecutor = XcmExecutor;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type Weigher = XcmWeigher;
	type LocationInverter = LocationInverter<Ancestry>;
	type Origin = Origin;
	type Call = Call;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = cumulus_pallet_xcmp_queue::weights::SubstrateWeight<Self>;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

parameter_types! {
	// Statemine ParaId in kusama
	pub StatemineParaId: u32 = 1000;
	// Assets Pallet instance in Statemine kusama
	pub StatemineAssetPalletInstance: u8 = 50;
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
		match location {
			// Change https://github.com/paritytech/cumulus/pull/831
			// This avoids interrumption once they upgrade
			// We map the previous location to the new one so that the assetId is well retrieved
			MultiLocation {
				parents: 1,
				interior: X2(Parachain(id), GeneralIndex(index)),
			} if id == StatemineParaId::get() => Self::Xcm(MultiLocation {
				parents: 1,
				interior: X3(
					Parachain(id),
					PalletInstance(StatemineAssetPalletInstance::get()),
					GeneralIndex(index),
				),
			}),
			_ => Self::Xcm(location),
		}
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
}

impl AccountIdToCurrencyId<AccountId, CurrencyId> for Runtime {
	fn account_to_currency_id(account: AccountId) -> Option<CurrencyId> {
		match account {
			// the self-reserve currency is identified by the pallet-balances address
			a if a == H160::from_low_u64_be(2050).into() => Some(CurrencyId::SelfReserve),
			// the rest of the currencies, by their corresponding erc20 address
			_ => Runtime::account_to_asset_id(account).map(|(prefix, asset_id)| {
				if prefix == FOREIGN_ASSET_PRECOMPILE_ADDRESS_PREFIX.to_vec() {
					CurrencyId::ForeignAsset(asset_id)
				} else {
					CurrencyId::LocalAssetReserve(asset_id)
				}
			}),
		}
	}
}
// How to convert from CurrencyId to MultiLocation

pub struct CurrencyIdtoMultiLocation<AssetXConverter>(sp_std::marker::PhantomData<AssetXConverter>);
impl<AssetXConverter> sp_runtime::traits::Convert<CurrencyId, Option<MultiLocation>>
	for CurrencyIdtoMultiLocation<AssetXConverter>
where
	AssetXConverter: xcm_executor::traits::Convert<MultiLocation, AssetId>,
{
	fn convert(currency: CurrencyId) -> Option<MultiLocation> {
		match currency {
			// For now and until Xtokens is adapted to handle 0.9.16 version we use
			// the old anchoring here
			// This is not a problem in either cases, since the view of the destination
			// chain does not change
			// TODO! change this to NewAnchoringSelfReserve once xtokens is adapted for it
			CurrencyId::SelfReserve => {
				let multi: MultiLocation = OldAnchoringSelfReserve::get();
				Some(multi)
			}
			CurrencyId::ForeignAsset(asset) => AssetXConverter::reverse_ref(asset).ok(),
			CurrencyId::LocalAssetReserve(asset) => {
				let mut location = LocalAssetsPalletLocationOldReanchor::get();
				location.push_interior(Junction::GeneralIndex(asset)).ok();
				Some(location)
			}
		}
	}
}

parameter_types! {
	pub const BaseXcmWeight: Weight = 100_000_000;
	pub const MaxAssetsForTransfer: usize = 2;

	// This is how we are going to detect whether the asset is a Reserve asset
	// This however is the chain part only
	pub SelfLocation: MultiLocation = MultiLocation {
		parents:1,
		interior: Junctions::X1(
			Parachain(ParachainInfo::parachain_id().into())
		)
	};
}

parameter_type_with_key! {
	pub ParachainMinFee: |_location: MultiLocation| -> u128 {
		u128::MAX
	};
}

impl orml_xtokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = AccountIdToMultiLocation<AccountId>;
	type CurrencyIdConvert =
		CurrencyIdtoMultiLocation<AsAssetType<AssetId, AssetType, AssetManager>>;
	type XcmExecutor = XcmExecutor;
	type SelfLocation = SelfLocation;
	type Weigher = XcmWeigher;
	type BaseXcmWeight = BaseXcmWeight;
	type LocationInverter = LocationInverter<Ancestry>;
	type MaxAssetsForTransfer = MaxAssetsForTransfer;
	type MinXcmFee = ParachainMinFee;
	type MultiLocationsFilter = Everything;
	type ReserveProvider = AbsoluteReserveProvider;
}

// For now we only allow to transact in the relay, although this might change in the future
// Transactors just defines the chains in which we allow transactions to be issued through
// xcm
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum Transactors {
	Relay,
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
			// Shall we use westend for moonbase? The tests are probably based on rococo
			// but moonbase-alpha is attached to westend-runtime I think
			Transactors::Relay => moonbeam_relay_encoder::kusama::KusamaEncoder.encode_call(call),
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

impl xcm_transactor::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Transactor = Transactors;
	type DerivativeAddressRegistrationOrigin = EnsureRoot<AccountId>;
	type SovereignAccountDispatcherOrigin = EnsureRoot<AccountId>;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = AccountIdToMultiLocation<AccountId>;
	type CurrencyIdToMultiLocation =
		CurrencyIdtoMultiLocation<AsAssetType<AssetId, AssetType, AssetManager>>;
	type XcmSender = XcmRouter;
	type SelfLocation = SelfLocation;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type LocationInverter = LocationInverter<Ancestry>;
	type BaseXcmWeight = BaseXcmWeight;
	type AssetTransactor = AssetTransactors;
	type WeightInfo = xcm_transactor::weights::SubstrateWeight<Runtime>;
}
