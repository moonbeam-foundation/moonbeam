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

//! XCM configuration for Moonbase.
//!

use super::{
	governance, AccountId, AssetId, AssetManager, Balance, Balances, EmergencyParaXcm,
	Erc20XcmBridge, EvmForeignAssets, MaintenanceMode, MessageQueue, OpenTechCommitteeInstance,
	ParachainInfo, ParachainSystem, Perbill, PolkadotXcm, Runtime, RuntimeBlockWeights,
	RuntimeCall, RuntimeEvent, RuntimeOrigin, Treasury, XcmpQueue,
};

use super::moonbeam_weights;
use frame_support::{
	parameter_types,
	traits::{EitherOf, EitherOfDiverse, Everything, Nothing, PalletInfoAccess, TransformOrigin},
};
use moonkit_xcm_primitives::AccountIdAssetIdConversion;
use sp_runtime::{
	traits::{Hash as THash, MaybeEquivalence, PostDispatchInfoOf},
	DispatchErrorWithPostInfo,
};
use sp_weights::Weight;

use frame_system::{EnsureRoot, RawOrigin};
use sp_core::{ConstU32, H160, H256};

use xcm_builder::{
	AccountKey20Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, Case, ConvertedConcreteId, DescribeAllTerminal, DescribeFamily,
	EnsureXcmOrigin, FungibleAdapter as XcmCurrencyAdapter, FungiblesAdapter, HashedDescription,
	NoChecking, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountKey20AsNative, SovereignSignedViaLocation,
	TakeWeightCredit, TrailingSetTopicAsId, WeightInfoBounds, WithComputedOrigin, WithUniqueTopic,
};

use parachains_common::message_queue::{NarrowOriginToSibling, ParaIdToSibling};
use xcm::{
	latest::prelude::{
		AllOf, Asset, AssetFilter, GlobalConsensus, InteriorLocation, Junction, Location,
		NetworkId, PalletInstance, Parachain, Wild, WildFungible,
	},
	IntoVersion,
};

use xcm_executor::traits::{CallDispatcher, ConvertLocation, JustTry};

use cumulus_primitives_core::{AggregateMessageOrigin, ParaId};
use pallet_xcm::EnsureXcm;
use xcm_primitives::{
	AbsoluteAndRelativeReserve, AccountIdToCurrencyId, AccountIdToLocation, AsAssetType,
	IsBridgedConcreteAssetFrom, MultiNativeAsset, SignedToAccountId20, UtilityAvailableCalls,
	UtilityEncodeCall, XcmTransact,
};

use crate::governance::referenda::{FastGeneralAdminOrRoot, GeneralAdminOrRoot};
use crate::runtime_params::dynamic_params;
use moonbeam_runtime_common::xcm_origins::AllowSiblingParachains;
use pallet_moonbeam_foreign_assets::{MapSuccessToGovernance, MapSuccessToXcm};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::Get;
use sp_std::{
	convert::{From, Into, TryFrom},
	prelude::*,
};

parameter_types! {
	// The network Id of the relay
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;
	// The relay chain Origin type
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub UniversalLocation: InteriorLocation =
		[GlobalConsensus(RelayNetwork::get()), Parachain(ParachainInfo::parachain_id().into())].into();
	// Self Reserve location, defines the multilocation identifiying the self-reserve currency
	// This is used to match it also against our Balances pallet when we receive such
	// a Location: (Self Balances pallet index)
	// We use the RELATIVE multilocation
	pub SelfReserve: Location = Location {
		parents:0,
		interior: [
			PalletInstance(<Balances as PalletInfoAccess>::index() as u8)
		].into()
	};
}

/// Type for specifying how a `Location` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<polkadot_parachain::primitives::Sibling, AccountId>,
	// If we receive a Location of type AccountKey20, just generate a native account
	AccountKey20Aliases<RelayNetwork, AccountId>,
	// Generate remote accounts according to polkadot standards
	HashedDescription<AccountId, DescribeFamily<DescribeAllTerminal>>,
);

/// Wrapper type around `LocationToAccountId` to convert an `AccountId` to type `H160`.
pub struct LocationToH160;
impl ConvertLocation<H160> for LocationToH160 {
	fn convert_location(location: &Location) -> Option<H160> {
		<LocationToAccountId as ConvertLocation<AccountId>>::convert_location(location)
			.map(Into::into)
	}
}

// The non-reserve fungible transactor type
// It will use pallet-assets, and the Id will be matched against AsAssetType
pub type ForeignFungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	super::Assets,
	// Use this currency when it is a fungible asset matching the given location or name:
	(
		ConvertedConcreteId<
			AssetId,
			Balance,
			AsAssetType<AssetId, AssetType, AssetManager>,
			JustTry,
		>,
	),
	// Do a simple punn to convert an AccountId20 Location into a native chain account ID:
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

// We use all transactors
pub type AssetTransactors = (
	LocalAssetTransactor,
	EvmForeignAssets,
	ForeignFungiblesTransactor,
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

pub type XcmBarrier = TrailingSetTopicAsId<(
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
)>;

parameter_types! {
	/// Xcm fees will go to the treasury account
	pub XcmFeesAccount: AccountId = Treasury::account_id();
}

pub struct SafeCallFilter;
impl frame_support::traits::Contains<RuntimeCall> for SafeCallFilter {
	fn contains(_call: &RuntimeCall) -> bool {
		// TODO review
		// This needs to be addressed at EVM level
		true
	}
}

parameter_types! {
	 /// Location of Asset Hub
	pub AssetHubLocation: Location = Location::new(1, [Parachain(1000)]);
	pub const RelayLocation: Location = Location::parent();
	pub RelayLocationFilter: AssetFilter = Wild(AllOf {
		fun: WildFungible,
		id: xcm::prelude::AssetId(RelayLocation::get()),
	});
	pub RelayChainNativeAssetFromAssetHub: (AssetFilter, Location) = (
		RelayLocationFilter::get(),
		AssetHubLocation::get()
	);
	pub const MaxAssetsIntoHolding: u32 = xcm_primitives::MAX_ASSETS;
}

type Reserves = (
	// Assets bridged from different consensus systems held in reserve on Asset Hub.
	IsBridgedConcreteAssetFrom<AssetHubLocation>,
	// Relaychain (DOT) from Asset Hub
	Case<RelayChainNativeAssetFromAssetHub>,
	// Assets which the reserve is the same as the origin.
	MultiNativeAsset<AbsoluteAndRelativeReserve<SelfLocationAbsolute>>,
);

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
	type IsReserve = Reserves;
	type IsTeleporter = (); // No teleport
	type UniversalLocation = UniversalLocation;
	type Barrier = XcmBarrier;
	type Weigher = XcmWeigher;
	// As trader we use the XcmWeightTrader pallet.
	// For each foreign asset, the fee is computed based on its relative price (also
	// stored in the XcmWeightTrader pallet) against the native asset.
	// For the native asset fee is computed using WeightToFee implementation.
	type Trader = pallet_xcm_weight_trader::Trader<Runtime>;
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
	type TransactionalProcessor = pallet_ethereum_xcm::XcmEthTransactionalProcessor;
	type HrmpNewChannelOpenRequestHandler = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type XcmRecorder = PolkadotXcm;
}

type XcmExecutor = pallet_erc20_xcm_bridge::XcmExecutorWrapper<
	XcmExecutorConfig,
	xcm_executor::XcmExecutor<XcmExecutorConfig>,
>;

// Converts a Signed Local Origin into a Location
pub type LocalOriginToLocation = SignedToAccountId20<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = WithUniqueTopic<(
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
)>;

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
	type WeightInfo = moonbeam_weights::pallet_xcm::WeightInfo<Runtime>;
	type AdminOrigin = EnsureRoot<AccountId>;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
	type MaxInboundSuspended = sp_core::ConstU32<1_000>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = moonbeam_weights::cumulus_pallet_xcmp_queue::WeightInfo<Runtime>;
	type PriceForSiblingDelivery = polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery<
		cumulus_primitives_core::ParaId,
	>;
	type MaxActiveOutboundChannels = ConstU32<128>;
	// Most on-chain HRMP channels are configured to use 102400 bytes of max message size, so we
	// need to set the page size larger than that until we reduce the channel size on-chain.
	type MaxPageSize = MessageQueueHeapSize;
}

parameter_types! {
	pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
}

parameter_types! {
	/// The amount of weight (if any) which should be provided to the message queue for
	/// servicing enqueued items.
	///
	/// This may be legitimately `None` in the case that you will call
	/// `ServiceQueues::service_queues` manually.
	pub MessageQueueServiceWeight: Weight =
		Perbill::from_percent(25) * RuntimeBlockWeights::get().max_block;
	/// The maximum number of stale pages (i.e. of overweight messages) allowed before culling
	/// can happen. Once there are more stale pages than this, then historical pages may be
	/// dropped, even if they contain unprocessed overweight messages.
	pub const MessageQueueMaxStale: u32 = 8;
	/// The size of the page; this implies the maximum message size which can be sent.
	///
	/// A good value depends on the expected message sizes, their weights, the weight that is
	/// available for processing them and the maximal needed message size. The maximal message
	/// size is slightly lower than this as defined by [`MaxMessageLenOf`].
	pub const MessageQueueHeapSize: u32 = 103 * 1024;
}

impl pallet_message_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	#[cfg(feature = "runtime-benchmarks")]
	type MessageProcessor = pallet_message_queue::mock_helpers::NoopMessageProcessor<
		cumulus_primitives_core::AggregateMessageOrigin,
	>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MessageProcessor = pallet_ethereum_xcm::MessageProcessorWrapper<
		xcm_builder::ProcessXcmMessage<AggregateMessageOrigin, XcmExecutor, RuntimeCall>,
	>;
	type Size = u32;
	type HeapSize = MessageQueueHeapSize;
	type MaxStale = MessageQueueMaxStale;
	type ServiceWeight = MessageQueueServiceWeight;
	// The XCMP queue pallet is only ever able to handle the `Sibling(ParaId)` origin:
	type QueueChangeHandler = NarrowOriginToSibling<XcmpQueue>;
	// NarrowOriginToSibling calls XcmpQueue's is_paused if Origin is sibling. Allows all other origins
	type QueuePausedQuery = EmergencyParaXcm;
	type WeightInfo = moonbeam_weights::pallet_message_queue::WeightInfo<Runtime>;
	type IdleMaxServiceWeight = MessageQueueServiceWeight;
}

pub type FastAuthorizeUpgradeOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, OpenTechCommitteeInstance, 5, 9>,
>;

pub type ResumeXcmOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionAtLeast<AccountId, OpenTechCommitteeInstance, 5, 9>,
>;

impl pallet_emergency_para_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type CheckAssociatedRelayNumber =
		cumulus_pallet_parachain_system::RelayNumberMonotonicallyIncreases;
	type QueuePausedQuery = (MaintenanceMode, NarrowOriginToSibling<XcmpQueue>);
	type PausedThreshold = ConstU32<300>;
	type FastAuthorizeUpgradeOrigin = FastAuthorizeUpgradeOrigin;
	type PausedToNormalOrigin = ResumeXcmOrigin;
}

// Our AssetType. For now we only handle Xcm Assets
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum AssetType {
	Xcm(xcm::v3::Location),
}
impl Default for AssetType {
	fn default() -> Self {
		Self::Xcm(xcm::v3::Location::here())
	}
}

impl From<xcm::v3::Location> for AssetType {
	fn from(location: xcm::v3::Location) -> Self {
		Self::Xcm(location)
	}
}

// This can be removed once we fully adopt xcm::v5 everywhere
impl TryFrom<Location> for AssetType {
	type Error = ();

	fn try_from(location: Location) -> Result<Self, Self::Error> {
		// Convert the V5 location to a V3 location
		match xcm::VersionedLocation::V5(location).into_version(xcm::v3::VERSION) {
			Ok(xcm::VersionedLocation::V3(loc)) => Ok(AssetType::Xcm(loc.into())),
			// Any other version or conversion error returns an error
			_ => Err(()),
		}
	}
}

impl Into<Option<xcm::v3::Location>> for AssetType {
	fn into(self) -> Option<xcm::v3::Location> {
		match self {
			Self::Xcm(location) => Some(location),
		}
	}
}

impl Into<Option<Location>> for AssetType {
	fn into(self) -> Option<Location> {
		match self {
			Self::Xcm(location) => {
				let versioned = xcm::VersionedLocation::V3(location);
				match versioned.into_version(xcm::latest::VERSION) {
					Ok(xcm::VersionedLocation::V5(loc)) => Some(loc),
					_ => None,
				}
			}
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
				Some((_prefix, asset_id)) => CurrencyId::ForeignAsset(asset_id),
				// If no known prefix is identified, we consider that it's a "real" erc20 token
				// (i.e. managed by a real smart contract)
				None => CurrencyId::Erc20 {
					contract_address: account.into(),
				},
			},
		})
	}
}
// How to convert from CurrencyId to Location
pub struct CurrencyIdToLocation<AssetXConverter>(sp_std::marker::PhantomData<AssetXConverter>);
impl<AssetXConverter> sp_runtime::traits::Convert<CurrencyId, Option<Location>>
	for CurrencyIdToLocation<AssetXConverter>
where
	AssetXConverter: MaybeEquivalence<Location, AssetId>,
{
	fn convert(currency: CurrencyId) -> Option<Location> {
		match currency {
			CurrencyId::SelfReserve => {
				let multi: Location = SelfReserve::get();
				Some(multi)
			}
			CurrencyId::ForeignAsset(asset) => AssetXConverter::convert_back(&asset),
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
	pub SelfLocation: Location = Location::here();
	// We need this to be able to catch when someone is trying to execute a non-
	// cross-chain transfer in xtokens through the absolute path way
	pub SelfLocationAbsolute: Location = Location {
		parents:1,
		interior: [
			Parachain(ParachainInfo::parachain_id().into())
		].into()
	};
}

// 1 DOT should be enough
parameter_types! {
	pub MaxHrmpRelayFee: Asset = (Location::parent(), 1_000_000_000_000u128).into();
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
	fn destination(self) -> Location {
		match self {
			Transactors::Relay => Location::parent(),
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
	type AccountIdToLocation = AccountIdToLocation<AccountId>;
	type CurrencyIdToLocation = CurrencyIdToLocation<(
		EvmForeignAssets,
		AsAssetType<AssetId, AssetType, AssetManager>,
	)>;
	type XcmSender = XcmRouter;
	type SelfLocation = SelfLocation;
	type Weigher = XcmWeigher;
	type UniversalLocation = UniversalLocation;
	type BaseXcmWeight = BaseXcmWeight;
	type AssetTransactor = AssetTransactors;
	type ReserveProvider = AbsoluteAndRelativeReserve<SelfLocationAbsolute>;
	type WeightInfo = moonbeam_weights::pallet_xcm_transactor::WeightInfo<Runtime>;
	type HrmpManipulatorOrigin = GeneralAdminOrRoot;
	type HrmpOpenOrigin = FastGeneralAdminOrRoot;
	type MaxHrmpFee = xcm_builder::Case<MaxHrmpRelayFee>;
}

parameter_types! {
	// This is the relative view of erc20 assets.
	// Identified by this prefix + AccountKey20(contractAddress)
	// We use the RELATIVE multilocation
	pub Erc20XcmBridgePalletLocation: Location = Location {
		parents:0,
		interior: [
			PalletInstance(<Erc20XcmBridge as PalletInfoAccess>::index() as u8)
		].into()
	};

	// To be able to support almost all erc20 implementations,
	// we provide a sufficiently hight gas limit.
	pub Erc20XcmBridgeTransferGasLimit: u64 = 400_000;
}

impl pallet_erc20_xcm_bridge::Config for Runtime {
	type AccountIdConverter = LocationToH160;
	type Erc20MultilocationPrefix = Erc20XcmBridgePalletLocation;
	type Erc20TransferGasLimit = Erc20XcmBridgeTransferGasLimit;
	type EvmRunner = EvmRunnerPrecompileOrEthXcm<MoonbeamCall, Self>;
}

pub struct AccountIdToH160;
impl sp_runtime::traits::Convert<AccountId, H160> for AccountIdToH160 {
	fn convert(account_id: AccountId) -> H160 {
		account_id.into()
	}
}

pub struct EvmForeignAssetIdFilter;
impl frame_support::traits::Contains<AssetId> for EvmForeignAssetIdFilter {
	fn contains(asset_id: &AssetId) -> bool {
		use xcm_primitives::AssetTypeGetter as _;
		// We should return true only if the AssetId doesn't exist in AssetManager
		AssetManager::get_asset_type(*asset_id).is_none()
	}
}

pub type ForeignAssetManagerOrigin = EitherOf<
	MapSuccessToXcm<EnsureXcm<AllowSiblingParachains>>,
	MapSuccessToGovernance<
		EitherOf<
			EnsureRoot<AccountId>,
			EitherOf<
				pallet_collective::EnsureProportionMoreThan<
					AccountId,
					OpenTechCommitteeInstance,
					5,
					9,
				>,
				EitherOf<
					governance::custom_origins::FastGeneralAdmin,
					governance::custom_origins::GeneralAdmin,
				>,
			>,
		>,
	>,
>;

impl pallet_moonbeam_foreign_assets::Config for Runtime {
	type AccountIdToH160 = AccountIdToH160;
	type AssetIdFilter = EvmForeignAssetIdFilter;
	type EvmRunner = EvmRunnerPrecompileOrEthXcm<MoonbeamCall, Self>;
	type ConvertLocation =
		SiblingParachainConvertsVia<polkadot_parachain::primitives::Sibling, AccountId>;
	type ForeignAssetCreatorOrigin = ForeignAssetManagerOrigin;
	type ForeignAssetFreezerOrigin = ForeignAssetManagerOrigin;
	type ForeignAssetModifierOrigin = ForeignAssetManagerOrigin;
	type ForeignAssetUnfreezerOrigin = ForeignAssetManagerOrigin;
	type OnForeignAssetCreated = ();
	type MaxForeignAssets = ConstU32<256>;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = moonbeam_weights::pallet_moonbeam_foreign_assets::WeightInfo<Runtime>;
	type XcmLocationToH160 = LocationToH160;
	type ForeignAssetCreationDeposit = dynamic_params::xcm_config::ForeignAssetCreationDeposit;
	type Balance = Balance;
	type Currency = Balances;
}

pub struct AssetFeesFilter;
impl frame_support::traits::Contains<Location> for AssetFeesFilter {
	fn contains(location: &Location) -> bool {
		location.parent_count() > 0
			&& location.first_interior() != Erc20XcmBridgePalletLocation::get().first_interior()
	}
}

pub type AddAndEditSupportedAssetOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EitherOfDiverse<
		pallet_collective::EnsureProportionMoreThan<AccountId, OpenTechCommitteeInstance, 5, 9>,
		EitherOf<
			governance::custom_origins::GeneralAdmin,
			governance::custom_origins::FastGeneralAdmin,
		>,
	>,
>;

pub type RemoveSupportedAssetOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<AccountId, OpenTechCommitteeInstance, 5, 9>,
>;

impl pallet_xcm_weight_trader::Config for Runtime {
	type AccountIdToLocation = AccountIdToLocation<AccountId>;
	type AddSupportedAssetOrigin = AddAndEditSupportedAssetOrigin;
	type AssetLocationFilter = AssetFeesFilter;
	type AssetTransactor = AssetTransactors;
	type AssetIdentifier = EvmForeignAssets;
	type Balance = Balance;
	type EditSupportedAssetOrigin = AddAndEditSupportedAssetOrigin;
	type NativeLocation = SelfReserve;
	type PauseSupportedAssetOrigin = AddAndEditSupportedAssetOrigin;
	type ResumeSupportedAssetOrigin = AddAndEditSupportedAssetOrigin;
	type RemoveSupportedAssetOrigin = RemoveSupportedAssetOrigin;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = moonbeam_weights::pallet_xcm_weight_trader::WeightInfo<Runtime>;
	type WeightToFee = <Runtime as pallet_transaction_payment::Config>::WeightToFee;
	type XcmFeesAccount = XcmFeesAccount;
	#[cfg(feature = "runtime-benchmarks")]
	type NotFilteredLocation = RelayLocation;
	#[cfg(feature = "runtime-benchmarks")]
	type AssetCreator = EvmForeignAssets;
}

#[cfg(feature = "runtime-benchmarks")]
mod testing {
	use super::*;

	/// This From exists for benchmarking purposes. It has the potential side-effect of calling
	/// AssetManager::set_asset_type_asset_id() and should NOT be used in any production code.
	impl From<Location> for CurrencyId {
		fn from(location: Location) -> CurrencyId {
			use xcm_primitives::AssetTypeGetter;

			// If it does not exist, for benchmarking purposes, we create the association
			let asset_id = if let Some(asset_id) =
				AsAssetType::<AssetId, AssetType, AssetManager>::convert_location(&location)
			{
				asset_id
			} else {
				let asset_type: AssetType = location
					.try_into()
					.expect("Location convertion to AssetType should succeed");
				let asset_id: AssetId = asset_type.clone().into();
				AssetManager::set_asset_type_asset_id(asset_type, asset_id);
				asset_id
			};

			CurrencyId::ForeignAsset(asset_id)
		}
	}
}
