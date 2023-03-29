// Copyright 2021 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Parachain runtime mock.

use frame_support::{
	construct_runtime,
	dispatch::GetDispatchInfo,
	parameter_types,
	traits::{AsEnsureOriginWithArg, Everything, Get, Nothing, PalletInfoAccess},
	weights::Weight,
	PalletId,
};
use frame_system::{EnsureNever, EnsureRoot};
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, ConstU32, Hash, IdentityLookup},
	Permill,
};
use sp_std::{convert::TryFrom, prelude::*};
use xcm::{latest::prelude::*, Version as XcmVersion, VersionedXcm};

use cumulus_primitives_core::relay_chain::HrmpChannelId;
use orml_traits::parameter_type_with_key;
use polkadot_core_primitives::BlockNumber as RelayBlockNumber;
use polkadot_parachain::primitives::{Id as ParaId, Sibling};
use xcm::latest::{
	AssetId as XcmAssetId, Error as XcmError, ExecuteXcm,
	Junction::{PalletInstance, Parachain},
	Junctions, MultiLocation, NetworkId, Outcome, Xcm,
};
use xcm_builder::{
	AccountKey20Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, AsPrefixedGeneralIndex, ConvertedConcreteId,
	CurrencyAdapter as XcmCurrencyAdapter, EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds,
	FungiblesAdapter, IsConcrete, NoChecking, ParentAsSuperuser, ParentIsPreset,
	RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
	SignedAccountKey20AsNative, SovereignSignedViaLocation, TakeWeightCredit,
};
use xcm_executor::{traits::JustTry, Config, XcmExecutor};

use scale_info::TypeInfo;
use xcm_simulator::{
	DmpMessageHandlerT as DmpMessageHandler, XcmpMessageFormat,
	XcmpMessageHandlerT as XcmpMessageHandler,
};

pub type AccountId = moonbeam_core_primitives::AccountId;
pub type Balance = u128;
pub type AssetId = u128;
pub type BlockNumber = u32;

parameter_types! {
	pub const BlockHashCount: u32 = 250;
}

impl frame_system::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<AccountId>;
	type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = Nothing;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub ExistentialDeposit: Balance = 0;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

pub type ForeignAssetInstance = ();
pub type LocalAssetInstance = pallet_assets::Instance1;

parameter_types! {
	pub const AssetDeposit: Balance = 1; // Does not really matter as this will be only called by root
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 0;
	pub const MetadataDepositPerByte: Balance = 0;
	pub const AssetAccountDeposit: Balance = 0;
}

impl pallet_assets::Config<ForeignAssetInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type AssetAccountDeposit = AssetAccountDeposit;
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	type RemoveItemsLimit = ConstU32<656>;
	type AssetIdParameter = AssetId;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureNever<AccountId>>;
	type CallbackHandle = ();
}

impl pallet_assets::Config<LocalAssetInstance> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = AssetDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = AssetsStringLimit;
	type Freezer = ();
	type Extra = ();
	type AssetAccountDeposit = AssetAccountDeposit;
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	type RemoveItemsLimit = ConstU32<656>;
	type AssetIdParameter = AssetId;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureNever<AccountId>>;
	type CallbackHandle = ();
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	AccountKey20Aliases<RelayNetwork, AccountId>,
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
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	pallet_xcm::XcmPassthrough<RuntimeOrigin>,
	SignedAccountKey20AsNative<RelayNetwork, RuntimeOrigin>,
);

parameter_types! {
	pub const UnitWeightCost: Weight = Weight::from_parts(1u64, 1u64);
	pub MaxInstructions: u32 = 100;
}

// Instructing how incoming xcm assets will be handled
pub type ForeignFungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	Assets,
	// Use this currency when it is a fungible asset matching the given location or name:
	(
		ConvertedConcreteId<
			AssetId,
			Balance,
			xcm_primitives::AsAssetType<AssetId, AssetType, AssetManager>,
			JustTry,
		>,
	),
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We dont allow teleports.
	NoChecking,
	// We dont track any teleports
	(),
>;

pub type LocalAssetTransactor = XcmCurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching any of the locations in
	// SelfReserveRepresentations
	IsConcrete<SelfReserve>,
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

// We use both transactors
pub type AssetTransactors = (
	LocalAssetTransactor,
	ForeignFungiblesTransactor,
	LocalFungiblesTransactor,
);

pub type XcmRouter = super::ParachainXcmRouter<MsgQueue>;

pub type Barrier = (
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
pub type XcmFeesToAccount_ = xcm_primitives::XcmFeesToAccount<
	Assets,
	(
		ConvertedConcreteId<
			AssetId,
			Balance,
			xcm_primitives::AsAssetType<AssetId, AssetType, AssetManager>,
			JustTry,
		>,
	),
	AccountId,
	XcmFeesAccount,
>;

parameter_types! {
	// We cannot skip the native trader for some specific tests, so we will have to work with
	// a native trader that charges same number of units as weight
	pub ParaTokensPerSecond: (XcmAssetId, u128, u128) = (
		Concrete(SelfReserve::get()),
		1000000000000,
		0,
	);
}

parameter_types! {
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub UniversalLocation: InteriorMultiLocation =
		X2(GlobalConsensus(RelayNetwork::get()), Parachain(MsgQueue::parachain_id().into()));

	pub LocalAssetsPalletLocation: MultiLocation = MultiLocation {
		parents:0,
		interior: Junctions::X1(
			PalletInstance(<LocalAssets as PalletInfoAccess>::index() as u8)
		)
	};

	// This is used to match it against our Balances pallet when we receive such a MultiLocation
	// (Parent, Self Para Id, Self Balances pallet index)
	pub SelfReserve: MultiLocation = MultiLocation {
		parents:0,
		interior: Junctions::X1(
			PalletInstance(<Balances as PalletInfoAccess>::index() as u8)
		)
	};
	pub const MaxAssetsIntoHolding: u32 = 64;
}

pub struct XcmConfig;
impl Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	type AssetTransactor = AssetTransactors;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = orml_xcm_support::MultiNativeAsset<
		xcm_primitives::AbsoluteAndRelativeReserve<SelfLocationAbsolute>,
	>;
	type IsTeleporter = ();
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	// We use three traders
	// When we receive either representation of the self-reserve asset,
	// When we receive a non-reserve asset, we use AssetManager to fetch how many
	// units per second we should charge
	type Trader = (
		FixedRateOfFungible<ParaTokensPerSecond, ()>,
		xcm_primitives::FirstAssetTrader<AssetType, AssetManager, XcmFeesToAccount_>,
	);

	type ResponseHandler = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
	type CallDispatcher = RuntimeCall;
	type AssetLocker = ();
	type AssetExchanger = ();
	type PalletInstancesInfo = ();
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type SafeCallFilter = Everything;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

// Our currencyId. We distinguish for now between SelfReserve, and Others, defined by their Id.
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum CurrencyId {
	SelfReserve,
	ForeignAsset(AssetId),
	LocalAssetReserve(AssetId),
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
			CurrencyId::SelfReserve => {
				// For now and until Xtokens is adapted to handle 0.9.16 version we use
				// the old anchoring here
				// This is not a problem in either cases, since the view of the destination
				// chain does not change
				// TODO! change this to NewAnchoringSelfReserve once xtokens is adapted for it
				let multi: MultiLocation = SelfReserve::get();
				Some(multi)
			}
			CurrencyId::ForeignAsset(asset) => AssetXConverter::reverse_ref(asset).ok(),
			CurrencyId::LocalAssetReserve(asset) => {
				let mut location = LocalAssetsPalletLocation::get();
				location.push_interior(Junction::GeneralIndex(asset)).ok();
				Some(location)
			}
		}
	}
}

parameter_types! {
	pub const BaseXcmWeight: Weight = Weight::from_parts(100u64, 100u64);
	pub const MaxAssetsForTransfer: usize = 2;
	pub SelfLocation: MultiLocation = MultiLocation::here();
	pub SelfLocationAbsolute: MultiLocation = MultiLocation {
		parents:1,
		interior: Junctions::X1(
			Parachain(MsgQueue::parachain_id().into())
		)
	};
}

parameter_type_with_key! {
	pub ParachainMinFee: |_location: MultiLocation| -> Option<u128> {
		Some(u128::MAX)
	};
}

// The XCM message wrapper wrapper
impl orml_xtokens::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = xcm_primitives::AccountIdToMultiLocation<AccountId>;
	type CurrencyIdConvert =
		CurrencyIdtoMultiLocation<xcm_primitives::AsAssetType<AssetId, AssetType, AssetManager>>;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type SelfLocation = SelfLocation;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type BaseXcmWeight = BaseXcmWeight;
	type UniversalLocation = UniversalLocation;
	type MaxAssetsForTransfer = MaxAssetsForTransfer;
	type MinXcmFee = ParachainMinFee;
	type MultiLocationsFilter = Everything;
	type ReserveProvider = xcm_primitives::AbsoluteAndRelativeReserve<SelfLocationAbsolute>;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 0;
	pub const SpendPeriod: u32 = 0;
	pub const TreasuryId: PalletId = PalletId(*b"pc/trsry");
	pub const MaxApprovals: u32 = 100;
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRoot<AccountId>;
	type RejectOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = ();
	type BurnDestination = ();
	type MaxApprovals = MaxApprovals;
	type WeightInfo = ();
	type SpendFunds = ();
	type ProposalBondMaximum = ();
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<Balance>; // Same as Polkadot
}

#[frame_support::pallet]
pub mod mock_msg_queue {
	use super::*;
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type XcmExecutor: ExecuteXcm<Self::RuntimeCall>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn parachain_id)]
	pub(super) type ParachainId<T: Config> = StorageValue<_, ParaId, ValueQuery>;

	impl<T: Config> Get<ParaId> for Pallet<T> {
		fn get() -> ParaId {
			Self::parachain_id()
		}
	}

	pub type MessageId = [u8; 32];

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// XCMP
		/// Some XCM was executed OK.
		Success(Option<T::Hash>),
		/// Some XCM failed.
		Fail(Option<T::Hash>, XcmError),
		/// Bad XCM version used.
		BadVersion(Option<T::Hash>),
		/// Bad XCM format used.
		BadFormat(Option<T::Hash>),

		// DMP
		/// Downward message is invalid XCM.
		InvalidFormat(MessageId),
		/// Downward message is unsupported version of XCM.
		UnsupportedVersion(MessageId),
		/// Downward message executed with the given outcome.
		ExecutedDownward(MessageId, Outcome),
	}

	impl<T: Config> Pallet<T> {
		pub fn set_para_id(para_id: ParaId) {
			ParachainId::<T>::put(para_id);
		}

		fn handle_xcmp_message(
			sender: ParaId,
			_sent_at: RelayBlockNumber,
			xcm: VersionedXcm<T::RuntimeCall>,
			max_weight: Weight,
		) -> Result<Weight, XcmError> {
			let hash = Encode::using_encoded(&xcm, T::Hashing::hash);
			let (result, event) = match Xcm::<T::RuntimeCall>::try_from(xcm) {
				Ok(xcm) => {
					let location = MultiLocation::new(1, Junctions::X1(Parachain(sender.into())));
					let mut id = [0u8; 32];
					id.copy_from_slice(hash.as_ref());
					match T::XcmExecutor::execute_xcm(location, xcm, id, max_weight) {
						Outcome::Error(e) => (Err(e.clone()), Event::Fail(Some(hash), e)),
						Outcome::Complete(w) => (Ok(w), Event::Success(Some(hash))),
						// As far as the caller is concerned, this was dispatched without error, so
						// we just report the weight used.
						Outcome::Incomplete(w, e) => (Ok(w), Event::Fail(Some(hash), e)),
					}
				}
				Err(()) => (
					Err(XcmError::UnhandledXcmVersion),
					Event::BadVersion(Some(hash)),
				),
			};
			Self::deposit_event(event);
			result
		}
	}

	impl<T: Config> XcmpMessageHandler for Pallet<T> {
		fn handle_xcmp_messages<'a, I: Iterator<Item = (ParaId, RelayBlockNumber, &'a [u8])>>(
			iter: I,
			max_weight: Weight,
		) -> Weight {
			for (sender, sent_at, data) in iter {
				let mut data_ref = data;
				let _ = XcmpMessageFormat::decode(&mut data_ref)
					.expect("Simulator encodes with versioned xcm format; qed");

				let mut remaining_fragments = &data_ref[..];
				while !remaining_fragments.is_empty() {
					if let Ok(xcm) =
						VersionedXcm::<T::RuntimeCall>::decode(&mut remaining_fragments)
					{
						let _ = Self::handle_xcmp_message(sender, sent_at, xcm, max_weight);
					} else {
						debug_assert!(false, "Invalid incoming XCMP message data");
					}
				}
			}
			max_weight
		}
	}

	impl<T: Config> DmpMessageHandler for Pallet<T> {
		fn handle_dmp_messages(
			iter: impl Iterator<Item = (RelayBlockNumber, Vec<u8>)>,
			limit: Weight,
		) -> Weight {
			for (_i, (_sent_at, data)) in iter.enumerate() {
				let id = sp_io::hashing::blake2_256(&data[..]);
				let maybe_msg = VersionedXcm::<T::RuntimeCall>::decode(&mut &data[..])
					.map(Xcm::<T::RuntimeCall>::try_from);
				match maybe_msg {
					Err(_) => {
						Self::deposit_event(Event::InvalidFormat(id));
					}
					Ok(Err(())) => {
						Self::deposit_event(Event::UnsupportedVersion(id));
					}
					Ok(Ok(x)) => {
						let outcome = T::XcmExecutor::execute_xcm(Parent, x, id, limit);

						Self::deposit_event(Event::ExecutedDownward(id, outcome));
					}
				}
			}
			limit
		}
	}
}

// Pallet to provide the version, used to test runtime upgrade version changes
#[frame_support::pallet]
pub mod mock_version_changer {
	use super::*;
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn current_version)]
	pub(super) type CurrentVersion<T: Config> = StorageValue<_, XcmVersion, ValueQuery>;

	impl<T: Config> Get<XcmVersion> for Pallet<T> {
		fn get() -> XcmVersion {
			Self::current_version()
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// XCMP
		/// Some XCM was executed OK.
		VersionChanged(XcmVersion),
	}

	impl<T: Config> Pallet<T> {
		pub fn set_version(version: XcmVersion) {
			CurrentVersion::<T>::put(version);
			Self::deposit_event(Event::VersionChanged(version));
		}
	}
}

impl mock_msg_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl mock_version_changer::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

pub type LocalOriginToLocation =
	xcm_primitives::SignedToAccountId20<RuntimeOrigin, AccountId, RelayNetwork>;

parameter_types! {
	pub MatcherLocation: MultiLocation = MultiLocation::here();
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = frame_support::traits::Nothing;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	// Do not allow teleports
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	// We use a custom one to test runtime ugprades
	type AdvertisedXcmVersion = XcmVersioner;
	type Currency = Balances;
	type CurrencyMatcher = IsConcrete<MatcherLocation>;
	type TrustedLockers = ();
	type SovereignAccountOf = ();
	type MaxLockers = ConstU32<8>;
	type WeightInfo = pallet_xcm::TestWeightInfo;
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

// We instruct how to register the Assets
// In this case, we tell it to Create an Asset in pallet-assets
pub struct AssetRegistrar;
use frame_support::pallet_prelude::DispatchResult;
impl pallet_asset_manager::AssetRegistrar<Runtime> for AssetRegistrar {
	fn create_foreign_asset(
		asset: AssetId,
		min_balance: Balance,
		metadata: AssetMetadata,
		is_sufficient: bool,
	) -> DispatchResult {
		Assets::force_create(
			RuntimeOrigin::root(),
			asset,
			AssetManager::account_id(),
			is_sufficient,
			min_balance,
		)?;

		Assets::force_set_metadata(
			RuntimeOrigin::root(),
			asset,
			metadata.name,
			metadata.symbol,
			metadata.decimals,
			false,
		)
	}

	fn create_local_asset(
		asset: AssetId,
		_creator: AccountId,
		min_balance: Balance,
		is_sufficient: bool,
		owner: AccountId,
	) -> DispatchResult {
		LocalAssets::force_create(
			RuntimeOrigin::root(),
			asset,
			owner,
			is_sufficient,
			min_balance,
		)?;

		// TODO uncomment when we feel comfortable
		/*
		// The asset has been created. Let's put the revert code in the precompile address
		let precompile_address = Runtime::asset_id_to_account(ASSET_PRECOMPILE_ADDRESS_PREFIX, asset);
		pallet_evm::AccountCodes::<Runtime>::insert(
			precompile_address,
			vec![0x60, 0x00, 0x60, 0x00, 0xfd],
		);*/
		Ok(())
	}
	fn destroy_foreign_asset(asset: AssetId) -> DispatchResult {
		// Mark the asset as destroying
		Assets::start_destroy(RuntimeOrigin::root(), asset)?;

		Ok(())
	}

	fn destroy_local_asset(asset: AssetId) -> DispatchResult {
		// Mark the asset as destroying
		LocalAssets::start_destroy(RuntimeOrigin::root(), asset)?;

		Ok(())
	}

	fn destroy_asset_dispatch_info_weight(asset: AssetId) -> Weight {
		RuntimeCall::Assets(
			pallet_assets::Call::<Runtime, ForeignAssetInstance>::start_destroy { id: asset },
		)
		.get_dispatch_info()
		.weight
	}
}

#[derive(Clone, Default, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub struct AssetMetadata {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
}
pub struct LocalAssetIdCreator;
impl pallet_asset_manager::LocalAssetIdCreator<Runtime> for LocalAssetIdCreator {
	fn create_asset_id_from_metadata(local_asset_counter: u128) -> AssetId {
		// Our means of converting a creator to an assetId
		// We basically hash (local asset counter)
		let mut result: [u8; 16] = [0u8; 16];
		let hash: H256 =
			local_asset_counter.using_encoded(<Runtime as frame_system::Config>::Hashing::hash);
		result.copy_from_slice(&hash.as_fixed_bytes()[0..16]);
		u128::from_le_bytes(result)
	}
}

impl pallet_asset_manager::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type AssetRegistrarMetadata = AssetMetadata;
	type ForeignAssetType = AssetType;
	type AssetRegistrar = AssetRegistrar;
	type ForeignAssetModifierOrigin = EnsureRoot<AccountId>;
	type LocalAssetModifierOrigin = EnsureRoot<AccountId>;
	type LocalAssetIdCreator = LocalAssetIdCreator;
	type Currency = Balances;
	type LocalAssetDeposit = AssetDeposit;
	type WeightInfo = ();
}

// 1 KSM should be enough
parameter_types! {
	pub MaxHrmpRelayFee: MultiAsset = (MultiLocation::parent(), 1_000_000_000_000u128).into();
}

impl pallet_xcm_transactor::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Transactor = MockTransactors;
	type DerivativeAddressRegistrationOrigin = EnsureRoot<AccountId>;
	type SovereignAccountDispatcherOrigin = frame_system::EnsureRoot<AccountId>;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = xcm_primitives::AccountIdToMultiLocation<AccountId>;
	type CurrencyIdToMultiLocation =
		CurrencyIdtoMultiLocation<xcm_primitives::AsAssetType<AssetId, AssetType, AssetManager>>;
	type SelfLocation = SelfLocation;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type XcmSender = XcmRouter;
	type BaseXcmWeight = BaseXcmWeight;
	type AssetTransactor = AssetTransactors;
	type ReserveProvider = xcm_primitives::AbsoluteAndRelativeReserve<SelfLocationAbsolute>;
	type WeightInfo = ();
	type HrmpManipulatorOrigin = EnsureRoot<AccountId>;
	type MaxHrmpFee = xcm_builder::Case<MaxHrmpRelayFee>;
	type HrmpEncoder = MockHrmpEncoder;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 1000;
}
impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

use sp_core::U256;

parameter_types! {
	pub BlockGasLimit: U256 = U256::max_value();
	pub WeightPerGas: Weight = Weight::from_ref_time(1);
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;

	type CallOrigin = pallet_evm::EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<AccountId>;

	type AddressMapping = moonbeam_runtime_common::IntoAddressMapping;
	type Currency = Balances;
	type Runner = pallet_evm::runner::stack::Runner<Self>;

	type RuntimeEvent = RuntimeEvent;
	type PrecompilesType = ();
	type PrecompilesValue = ();
	type ChainId = ();
	type BlockGasLimit = BlockGasLimit;
	type OnChargeTransaction = ();
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
	type OnCreate = ();
}

pub struct NormalFilter;
impl frame_support::traits::Contains<RuntimeCall> for NormalFilter {
	fn contains(c: &RuntimeCall) -> bool {
		match c {
			_ => true,
		}
	}
}

// We need to use the encoding from the relay mock runtime
#[derive(Encode, Decode)]
pub enum RelayCall {
	#[codec(index = 5u8)]
	// the index should match the position of the module in `construct_runtime!`
	Utility(UtilityCall),
	#[codec(index = 6u8)]
	// the index should match the position of the module in `construct_runtime!`
	Hrmp(HrmpCall),
}

#[derive(Encode, Decode)]
pub enum UtilityCall {
	#[codec(index = 1u8)]
	AsDerivative(u16),
}

// HRMP call encoding, needed for xcm transactor pallet
#[derive(Encode, Decode)]
pub enum HrmpCall {
	#[codec(index = 0u8)]
	InitOpenChannel(ParaId, u32, u32),
	#[codec(index = 1u8)]
	AcceptOpenChannel(ParaId),
	#[codec(index = 2u8)]
	CloseChannel(HrmpChannelId),
	#[codec(index = 6u8)]
	CancelOpenRequest(HrmpChannelId, u32),
}

#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum MockTransactors {
	Relay,
}

impl xcm_primitives::XcmTransact for MockTransactors {
	fn destination(self) -> MultiLocation {
		match self {
			MockTransactors::Relay => MultiLocation::parent(),
		}
	}
}

impl xcm_primitives::UtilityEncodeCall for MockTransactors {
	fn encode_call(self, call: xcm_primitives::UtilityAvailableCalls) -> Vec<u8> {
		match self {
			MockTransactors::Relay => match call {
				xcm_primitives::UtilityAvailableCalls::AsDerivative(a, b) => {
					let mut call =
						RelayCall::Utility(UtilityCall::AsDerivative(a.clone())).encode();
					call.append(&mut b.clone());
					call
				}
			},
		}
	}
}

pub struct MockHrmpEncoder;
impl xcm_primitives::HrmpEncodeCall for MockHrmpEncoder {
	fn hrmp_encode_call(
		call: xcm_primitives::HrmpAvailableCalls,
	) -> Result<Vec<u8>, xcm::latest::Error> {
		match call {
			xcm_primitives::HrmpAvailableCalls::InitOpenChannel(a, b, c) => Ok(RelayCall::Hrmp(
				HrmpCall::InitOpenChannel(a.clone(), b.clone(), c.clone()),
			)
			.encode()),
			xcm_primitives::HrmpAvailableCalls::AcceptOpenChannel(a) => {
				Ok(RelayCall::Hrmp(HrmpCall::AcceptOpenChannel(a.clone())).encode())
			}
			xcm_primitives::HrmpAvailableCalls::CloseChannel(a) => {
				Ok(RelayCall::Hrmp(HrmpCall::CloseChannel(a.clone())).encode())
			}
			xcm_primitives::HrmpAvailableCalls::CancelOpenRequest(a, b) => {
				Ok(RelayCall::Hrmp(HrmpCall::CancelOpenRequest(a.clone(), b.clone())).encode())
			}
		}
	}
}

impl pallet_ethereum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		MsgQueue: mock_msg_queue::{Pallet, Storage, Event<T>},
		XcmVersioner: mock_version_changer::{Pallet, Storage, Event<T>},

		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin},
		XTokens: orml_xtokens::{Pallet, Call, Storage, Event<T>},
		AssetManager: pallet_asset_manager::{Pallet, Call, Storage, Event<T>},
		XcmTransactor: pallet_xcm_transactor::{Pallet, Call, Storage, Event<T>},
		Treasury: pallet_treasury::{Pallet, Storage, Config, Event<T>, Call},
		LocalAssets: pallet_assets::<Instance1>::{Pallet, Call, Storage, Event<T>},

		Timestamp: pallet_timestamp::{Pallet, Call, Storage},
		EVM: pallet_evm::{Pallet, Call, Storage, Config, Event<T>},
		Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Origin, Config},
	}
);

pub(crate) fn para_events() -> Vec<RuntimeEvent> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| Some(e))
		.collect::<Vec<_>>()
}

use frame_support::traits::{OnFinalize, OnInitialize, OnRuntimeUpgrade};
pub(crate) fn on_runtime_upgrade() {
	PolkadotXcm::on_runtime_upgrade();
}

pub(crate) fn para_roll_to(n: BlockNumber) {
	while System::block_number() < n {
		PolkadotXcm::on_finalize(System::block_number());
		Balances::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		Balances::on_initialize(System::block_number());
		PolkadotXcm::on_initialize(System::block_number());
	}
}
