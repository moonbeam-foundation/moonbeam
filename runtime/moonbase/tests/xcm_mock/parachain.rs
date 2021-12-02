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
	construct_runtime, parameter_types,
	traits::{Everything, Get, Nothing, PalletInfo as PalletInfoTrait},
	weights::Weight,
	PalletId,
};

use frame_system::EnsureRoot;
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{Hash, IdentityLookup},
	Permill,
};
use sp_std::{convert::TryFrom, prelude::*};
use xcm::{latest::prelude::*, Version as XcmVersion, VersionedXcm};

use polkadot_core_primitives::BlockNumber as RelayBlockNumber;
use polkadot_parachain::primitives::{Id as ParaId, Sibling};
use xcm::latest::{
	AssetId as XcmAssetId, Error as XcmError, ExecuteXcm,
	Junction::{PalletInstance, Parachain},
	Junctions, MultiLocation, NetworkId, Outcome, Xcm,
};
use xcm_builder::{
	AccountKey20Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, ConvertedConcreteAssetId,
	CurrencyAdapter as XcmCurrencyAdapter, EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds,
	FungiblesAdapter, IsConcrete, LocationInverter, ParentAsSuperuser, ParentIsDefault,
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

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Runtime {
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<AccountId>;
	type Header = Header;
	type Event = Event;
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
}

parameter_types! {
	pub ExistentialDeposit: Balance = 1;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const AssetDeposit: Balance = 0; // Does not really matter as this will be only called by root
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 0;
	pub const MetadataDepositPerByte: Balance = 0;
	pub const ExecutiveBody: xcm::v0::BodyId = xcm::v0::BodyId::Executive;
}

impl pallet_assets::Config for Runtime {
	type Event = Event;
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
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsDefault<AccountId>,
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
	SovereignSignedViaLocation<LocationToAccountId, Origin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognised.
	RelayChainAsNative<RelayChainOrigin, Origin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<Origin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	pallet_xcm::XcmPassthrough<Origin>,
	SignedAccountKey20AsNative<RelayNetwork, Origin>,
);

parameter_types! {
	pub const UnitWeightCost: Weight = 1;
	pub MaxInstructions: u32 = 100;
}

// Instructing how incoming xcm assets will be handled
pub type FungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	Assets,
	// Use this currency when it is a fungible asset matching the given location or name:
	(
		ConvertedConcreteAssetId<
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
	Nothing,
	// We dont track any teleports
	(),
>;

/// The transactor for our own chain currency.
pub type LocalAssetTransactor = XcmCurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<SelfReserve>,
	// We can convert the MultiLocations with our converter above:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We track our teleports in/out to keep total issuance correct.
	(),
>;

// These will be our transactors
pub type AssetTransactors = (LocalAssetTransactor, FungiblesTransactor);
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
		ConvertedConcreteAssetId<
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
	pub ParaTokensPerSecond: (XcmAssetId, u128) = (Concrete(SelfReserve::get()), 1000000000000);
}

parameter_types! {
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;
	pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = Parachain(MsgQueue::parachain_id().into()).into();
		pub SelfReserve: MultiLocation = MultiLocation {
		parents:1,
		interior: Junctions::X2(
			Parachain(MsgQueue::parachain_id().into()),
			PalletInstance(<Runtime as frame_system::Config>::PalletInfo::index::<Balances>().unwrap() as u8)
		)
	};
}
pub struct XcmConfig;
impl Config for XcmConfig {
	type Call = Call;
	type XcmSender = XcmRouter;
	type AssetTransactor = AssetTransactors;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = xcm_primitives::MultiNativeAsset;
	type IsTeleporter = ();
	type LocationInverter = LocationInverter<Ancestry>;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type Trader = (
		FixedRateOfFungible<ParaTokensPerSecond, ()>,
		xcm_primitives::FirstAssetTrader<AssetId, AssetType, AssetManager, XcmFeesToAccount_>,
	);

	type ResponseHandler = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

// Our currencyId. We distinguish for now between SelfReserve, and Others, defined by their Id.
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub enum CurrencyId {
	SelfReserve,
	OtherReserve(AssetId),
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
				let multi: MultiLocation = SelfReserve::get();
				Some(multi)
			}
			CurrencyId::OtherReserve(asset) => AssetXConverter::reverse_ref(asset).ok(),
		}
	}
}

parameter_types! {
	pub const BaseXcmWeight: Weight = 100;
	pub SelfLocation: MultiLocation = MultiLocation {
		parents:1,
		interior: Junctions::X1(
			Parachain(MsgQueue::parachain_id().into())
		)
	};
}

// The XCM message wrapper wrapper
impl orml_xtokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = xcm_primitives::AccountIdToMultiLocation<AccountId>;
	type CurrencyIdConvert =
		CurrencyIdtoMultiLocation<xcm_primitives::AsAssetType<AssetId, AssetType, AssetManager>>;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type SelfLocation = SelfLocation;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type BaseXcmWeight = BaseXcmWeight;
	type LocationInverter = LocationInverter<Ancestry>;
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 0;
	pub const SpendPeriod: u64 = 0;
	pub const TreasuryId: PalletId = PalletId(*b"pc/trsry");
	pub const MaxApprovals: u32 = 100;
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryId;
	type Currency = Balances;
	type ApproveOrigin = EnsureRoot<AccountId>;
	type RejectOrigin = EnsureRoot<AccountId>;
	type Event = Event;
	type OnSlash = Treasury;
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type SpendPeriod = SpendPeriod;
	type Burn = ();
	type BurnDestination = ();
	type MaxApprovals = MaxApprovals;
	type WeightInfo = ();
	type SpendFunds = ();
}

#[frame_support::pallet]
pub mod mock_msg_queue {
	use super::*;
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type XcmExecutor: ExecuteXcm<Self::Call>;
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
			xcm: VersionedXcm<T::Call>,
			max_weight: Weight,
		) -> Result<Weight, XcmError> {
			let hash = Encode::using_encoded(&xcm, T::Hashing::hash);
			let (result, event) = match Xcm::<T::Call>::try_from(xcm) {
				Ok(xcm) => {
					let location = MultiLocation::new(1, Junctions::X1(Parachain(sender.into())));
					match T::XcmExecutor::execute_xcm(location, xcm, max_weight) {
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
					if let Ok(xcm) = VersionedXcm::<T::Call>::decode(&mut remaining_fragments) {
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
				let maybe_msg =
					VersionedXcm::<T::Call>::decode(&mut &data[..]).map(Xcm::<T::Call>::try_from);
				match maybe_msg {
					Err(_) => {
						Self::deposit_event(Event::InvalidFormat(id));
					}
					Ok(Err(())) => {
						Self::deposit_event(Event::UnsupportedVersion(id));
					}
					Ok(Ok(x)) => {
						let outcome = T::XcmExecutor::execute_xcm(Parent, x, limit);

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
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
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
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl mock_version_changer::Config for Runtime {
	type Event = Event;
}

pub type LocalOriginToLocation =
	xcm_primitives::SignedToAccountId20<Origin, AccountId, RelayNetwork>;

impl pallet_xcm::Config for Runtime {
	type Event = Event;
	type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmExecuteFilter = frame_support::traits::Nothing;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	// Do not allow teleports
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type LocationInverter = xcm_builder::LocationInverter<Ancestry>;
	type Origin = Origin;
	type Call = Call;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	// We use a custom one to test runtime ugprades
	type AdvertisedXcmVersion = XcmVersioner;
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
	fn into(self: Self) -> Option<MultiLocation> {
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
	fn create_asset(
		asset: AssetId,
		min_balance: Balance,
		metadata: AssetMetadata,
	) -> DispatchResult {
		Assets::force_create(
			Origin::root(),
			asset,
			AssetManager::account_id(),
			true,
			min_balance,
		)?;

		Assets::force_set_metadata(
			Origin::root(),
			asset,
			metadata.name,
			metadata.symbol,
			metadata.decimals,
			false,
		)
	}
}

#[derive(Clone, Default, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode, TypeInfo)]
pub struct AssetMetadata {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
}

impl pallet_asset_manager::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type AssetId = AssetId;
	type AssetRegistrarMetadata = AssetMetadata;
	type AssetType = AssetType;
	type AssetRegistrar = AssetRegistrar;
	type AssetModifierOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ();
}

impl xcm_transactor::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Transactor = MockTransactors;
	type DerivativeAddressRegistrationOrigin = EnsureRoot<AccountId>;
	type SovereignAccountDispatcherOrigin = frame_system::EnsureRoot<AccountId>;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = xcm_primitives::AccountIdToMultiLocation<AccountId>;
	type CurrencyIdToMultiLocation =
		CurrencyIdtoMultiLocation<xcm_primitives::AsAssetType<AssetId, AssetType, AssetManager>>;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type SelfLocation = SelfLocation;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type LocationInverter = LocationInverter<Ancestry>;
	type XcmSender = XcmRouter;
	type BaseXcmWeight = BaseXcmWeight;
}

pub struct NormalFilter;
impl frame_support::traits::Contains<Call> for NormalFilter {
	fn contains(c: &Call) -> bool {
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
}

#[derive(Encode, Decode)]
pub enum UtilityCall {
	#[codec(index = 1u8)]
	AsDerivative(u16),
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
		Assets: pallet_assets::{Pallet, Storage, Event<T>},
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin},
		XTokens: orml_xtokens::{Pallet, Call, Storage, Event<T>},
		AssetManager: pallet_asset_manager::{Pallet, Call, Storage, Event<T>},
		XcmTransactor: xcm_transactor::{Pallet, Call, Storage, Event<T>},
		Treasury: pallet_treasury::{Pallet, Storage, Config, Event<T>, Call}
	}
);

pub(crate) fn para_events() -> Vec<Event> {
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

pub(crate) fn para_roll_to(n: u64) {
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
