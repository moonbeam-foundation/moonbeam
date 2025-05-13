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
	ensure, parameter_types,
	traits::{
		AsEnsureOriginWithArg, ConstU32, Everything, Get, InstanceFilter, Nothing, PalletInfoAccess,
	},
	weights::Weight,
	PalletId,
};

use frame_system::{pallet_prelude::BlockNumberFor, EnsureNever, EnsureRoot};
use pallet_xcm::migration::v1::VersionUncheckedMigrateToV1;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, Hash, IdentityLookup, MaybeEquivalence, Zero},
	Permill,
};
use sp_std::{convert::TryFrom, prelude::*};
use xcm::{latest::prelude::*, Version as XcmVersion, VersionedXcm};

use cumulus_primitives_core::relay_chain::HrmpChannelId;
use pallet_ethereum::PostLogContent;
use polkadot_core_primitives::BlockNumber as RelayBlockNumber;
use polkadot_parachain::primitives::{Id as ParaId, Sibling};
use xcm::latest::{
	Error as XcmError, ExecuteXcm,
	Junction::{PalletInstance, Parachain},
	Location, NetworkId, Outcome, Xcm,
};
use xcm_builder::{
	AccountKey20Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, Case, ConvertedConcreteId, EnsureXcmOrigin, FixedWeightBounds,
	FungibleAdapter as XcmCurrencyAdapter, FungiblesAdapter, IsConcrete, NoChecking,
	ParentAsSuperuser, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountKey20AsNative, SovereignSignedViaLocation,
	TakeWeightCredit, WithComputedOrigin,
};
use xcm_executor::{traits::JustTry, Config, XcmExecutor};

pub use moonbase_runtime::xcm_config::AssetType;
#[cfg(feature = "runtime-benchmarks")]
use moonbeam_runtime_common::benchmarking::BenchmarkHelper as ArgumentsBenchmarkHelper;
use scale_info::TypeInfo;
use xcm_simulator::{
	DmpMessageHandlerT as DmpMessageHandler, XcmpMessageFormat,
	XcmpMessageHandlerT as XcmpMessageHandler,
};

pub type AccountId = moonbeam_core_primitives::AccountId;
pub type Balance = u128;
pub type AssetId = u128;
pub type BlockNumber = BlockNumberFor<Runtime>;

parameter_types! {
	pub const BlockHashCount: u32 = 250;
}

impl frame_system::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeTask = RuntimeTask;
	type Nonce = u64;
	type Block = Block;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<AccountId>;
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
	type BaseCallFilter = Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
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
	type RuntimeHoldReason = ();
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeFreezeReason = ();
}

pub type ForeignAssetInstance = ();

// Required for runtime benchmarks
pallet_assets::runtime_benchmarks_enabled! {
	pub struct BenchmarkHelper;
	impl<AssetIdParameter> pallet_assets::BenchmarkHelper<AssetIdParameter> for BenchmarkHelper
	where
		AssetIdParameter: From<u128>,
	{
		fn create_asset_id_parameter(id: u32) -> AssetIdParameter {
			(id as u128).into()
		}
	}
}

parameter_types! {
	pub const AssetDeposit: Balance = 10; // Does not really matter as this will be only called by root
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
	pallet_assets::runtime_benchmarks_enabled! {
		type BenchmarkHelper = BenchmarkHelper;
	}
}

/// Type for specifying how a `Location` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	AccountKey20Aliases<RelayNetwork, AccountId>,
	// The rest of multilocations convert via hashing it
	xcm_builder::HashedDescription<
		AccountId,
		xcm_builder::DescribeFamily<xcm_builder::DescribeAllTerminal>,
	>,
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
	// Use this currency when it is a fungible asset matching any of the locations in
	// SelfReserveRepresentations
	(
		ConvertedConcreteId<
			AssetId,
			Balance,
			xcm_primitives::AsAssetType<AssetId, AssetType, AssetManager>,
			JustTry,
		>,
	),
	// Do a simple punn to convert an AccountId32 Location into a native chain account ID:
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
	IsConcrete<SelfReserve>,
	// We can convert the Locations with our converter above:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We dont allow teleport
	(),
>;

// These will be our transactors
// We use both transactors
pub type AssetTransactors = (LocalAssetTransactor, ForeignFungiblesTransactor);

pub type XcmRouter = super::ParachainXcmRouter<MsgQueue>;

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
	/// Parachain token units per second of execution
	pub ParaTokensPerSecond: u128 = 1000000000000;
}

pub struct WeightToFee;
impl sp_weights::WeightToFee for WeightToFee {
	type Balance = Balance;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		use sp_runtime::SaturatedConversion as _;
		Self::Balance::saturated_from(weight.ref_time())
			.saturating_mul(ParaTokensPerSecond::get())
			.saturating_div(frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND as u128)
	}
}

parameter_types! {
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub UniversalLocation: InteriorLocation =
		[GlobalConsensus(RelayNetwork::get()), Parachain(MsgQueue::parachain_id().into())].into();

	// New Self Reserve location, defines the multilocation identifiying the self-reserve currency
	// This is used to match it also against our Balances pallet when we receive such
	// a Location: (Self Balances pallet index)
	pub SelfReserve: Location = Location {
		parents:0,
		interior: [
			PalletInstance(<Balances as PalletInfoAccess>::index() as u8)
		].into()
	};
	pub const MaxAssetsIntoHolding: u32 = 64;

	pub AssetHubLocation: Location = Location::new(1, [Parachain(1000)]);
	pub RelayLocationFilter: AssetFilter = Wild(AllOf {
		fun: WildFungible,
		id: xcm::prelude::AssetId(Location::parent()),
	});

	pub RelayChainNativeAssetFromAssetHub: (AssetFilter, Location) = (
		RelayLocationFilter::get(),
		AssetHubLocation::get()
	);
}

use frame_system::RawOrigin;
use sp_runtime::traits::PostDispatchInfoOf;
use sp_runtime::DispatchErrorWithPostInfo;
use xcm_executor::traits::CallDispatcher;
moonbeam_runtime_common::impl_moonbeam_xcm_call!();

type Reserves = (
	// Relaychain (DOT) from Asset Hub
	Case<RelayChainNativeAssetFromAssetHub>,
	// Assets which the reserve is the same as the origin.
	xcm_primitives::MultiNativeAsset<
		xcm_primitives::AbsoluteAndRelativeReserve<SelfLocationAbsolute>,
	>,
);

pub struct XcmConfig;
impl Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	type AssetTransactor = AssetTransactors;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = Reserves;
	type IsTeleporter = ();
	type UniversalLocation = UniversalLocation;
	type Barrier = XcmBarrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = pallet_xcm_weight_trader::Trader<Runtime>;

	type ResponseHandler = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
	type CallDispatcher = MoonbeamCall;
	type AssetLocker = ();
	type AssetExchanger = ();
	type PalletInstancesInfo = ();
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type SafeCallFilter = Everything;
	type Aliasers = Nothing;
	type TransactionalProcessor = ();
	type HrmpNewChannelOpenRequestHandler = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type XcmRecorder = PolkadotXcm;
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
				// For now and until Xtokens is adapted to handle 0.9.16 version we use
				// the old anchoring here
				// This is not a problem in either cases, since the view of the destination
				// chain does not change
				// TODO! change this to NewAnchoringSelfReserve once xtokens is adapted for it
				let multi: Location = SelfReserve::get();
				Some(multi)
			}
			CurrencyId::ForeignAsset(asset) => AssetXConverter::convert_back(&asset),
		}
	}
}

parameter_types! {
	pub const BaseXcmWeight: Weight = Weight::from_parts(100u64, 100u64);
	pub const MaxAssetsForTransfer: usize = 2;
	pub SelfLocation: Location = Location::here();
	pub SelfLocationAbsolute: Location = Location {
		parents:1,
		interior: [
			Parachain(MsgQueue::parachain_id().into())
		].into()
	};
}

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 0;
	pub const SpendPeriod: u32 = 0;
	pub const TreasuryId: PalletId = PalletId(*b"pc/trsry");
	pub const MaxApprovals: u32 = 100;
	pub TreasuryAccount: AccountId = Treasury::account_id();
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryId;
	type Currency = Balances;
	type RejectOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type SpendPeriod = SpendPeriod;
	type Burn = ();
	type BurnDestination = ();
	type MaxApprovals = MaxApprovals;
	type WeightInfo = ();
	type SpendFunds = ();
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<Balance>; // Same as Polkadot
	type AssetKind = ();
	type Beneficiary = AccountId;
	type BeneficiaryLookup = IdentityLookup<AccountId>;
	type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
	type BalanceConverter = UnityAssetBalanceConversion;
	type PayoutPeriod = ConstU32<0>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ArgumentsBenchmarkHelper;
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
					let location = Location::new(1, [Parachain(sender.into())]);
					let mut id = [0u8; 32];
					id.copy_from_slice(hash.as_ref());
					match T::XcmExecutor::prepare_and_execute(
						location,
						xcm,
						&mut id,
						max_weight,
						Weight::zero(),
					) {
						Outcome::Error { error } => {
							(Err(error.clone()), Event::Fail(Some(hash), error))
						}
						Outcome::Complete { used } => (Ok(used), Event::Success(Some(hash))),
						// As far as the caller is concerned, this was dispatched without error, so
						// we just report the weight used.
						Outcome::Incomplete { used, error } => {
							(Ok(used), Event::Fail(Some(hash), error))
						}
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
				let mut id = sp_io::hashing::blake2_256(&data[..]);
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
						let outcome = T::XcmExecutor::prepare_and_execute(
							Parent,
							x,
							&mut id,
							limit,
							Weight::zero(),
						);

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
	pub MatcherLocation: Location = Location::here();
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
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	type AdminOrigin = frame_system::EnsureRoot<AccountId>;
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

	fn destroy_foreign_asset(asset: AssetId) -> DispatchResult {
		// Mark the asset as destroying
		Assets::start_destroy(RuntimeOrigin::root(), asset.into())?;

		Ok(())
	}

	fn destroy_asset_dispatch_info_weight(asset: AssetId) -> Weight {
		RuntimeCall::Assets(
			pallet_assets::Call::<Runtime, ForeignAssetInstance>::start_destroy {
				id: asset.into(),
			},
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

impl pallet_asset_manager::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = AssetId;
	type AssetRegistrarMetadata = AssetMetadata;
	type ForeignAssetType = AssetType;
	type AssetRegistrar = AssetRegistrar;
	type ForeignAssetModifierOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ();
}

// 1 ROC/WND should be enough
parameter_types! {
	pub MaxHrmpRelayFee: Asset = (Location::parent(), 1_000_000_000_000u128).into();
}

impl pallet_xcm_transactor::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Transactor = MockTransactors;
	type DerivativeAddressRegistrationOrigin = EnsureRoot<AccountId>;
	type SovereignAccountDispatcherOrigin = frame_system::EnsureRoot<AccountId>;
	type CurrencyId = CurrencyId;
	type AccountIdToLocation = xcm_primitives::AccountIdToLocation<AccountId>;
	type CurrencyIdToLocation = CurrencyIdToLocation<(
		EvmForeignAssets,
		AsAssetType<moonbeam_core_primitives::AssetId, AssetType, AssetManager>,
	)>;
	type SelfLocation = SelfLocation;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type XcmSender = XcmRouter;
	type BaseXcmWeight = BaseXcmWeight;
	type AssetTransactor = AssetTransactors;
	type ReserveProvider = xcm_primitives::AbsoluteAndRelativeReserve<SelfLocationAbsolute>;
	type WeightInfo = ();
	type HrmpManipulatorOrigin = EnsureRoot<AccountId>;
	type HrmpOpenOrigin = EnsureRoot<AccountId>;
	type MaxHrmpFee = xcm_builder::Case<MaxHrmpRelayFee>;
}

parameter_types! {
	pub RelayLocation: Location = Location::parent();
}

impl pallet_xcm_weight_trader::Config for Runtime {
	type AccountIdToLocation = xcm_primitives::AccountIdToLocation<AccountId>;
	type AddSupportedAssetOrigin = EnsureRoot<AccountId>;
	type AssetLocationFilter = Everything;
	type AssetTransactor = AssetTransactors;
	type Balance = Balance;
	type EditSupportedAssetOrigin = EnsureRoot<AccountId>;
	type NativeLocation = SelfReserve;
	type PauseSupportedAssetOrigin = EnsureRoot<AccountId>;
	type RemoveSupportedAssetOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type ResumeSupportedAssetOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ();
	type WeightToFee = WeightToFee;
	type XcmFeesAccount = XcmFeesAccount;
	#[cfg(feature = "runtime-benchmarks")]
	type NotFilteredLocation = RelayLocation;
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
	pub BlockGasLimit: U256 = U256::from(u64::MAX);
	pub WeightPerGas: Weight = Weight::from_parts(1, 0);
	pub GasLimitPovSizeRatio: u64 = {
		let block_gas_limit = BlockGasLimit::get().min(u64::MAX.into()).low_u64();
		block_gas_limit.saturating_div(MAX_POV_SIZE as u64)
	};
	pub GasLimitStorageGrowthRatio: u64 =
		BlockGasLimit::get().min(u64::MAX.into()).low_u64().saturating_div(BLOCK_STORAGE_LIMIT);
}

pub struct RandomnessProvider;
impl
	frame_support::traits::Randomness<
		<Runtime as frame_system::Config>::Hash,
		BlockNumberFor<Runtime>,
	> for RandomnessProvider
{
	fn random(
		subject: &[u8],
	) -> (
		<Runtime as frame_system::Config>::Hash,
		BlockNumberFor<Runtime>,
	) {
		let output = <Runtime as frame_system::Config>::Hashing::hash(subject);
		let block_number = frame_system::Pallet::<Runtime>::block_number();
		(output, block_number)
	}
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;

	type CallOrigin = pallet_evm::EnsureAddressRoot<AccountId>;
	type WithdrawOrigin = pallet_evm::EnsureAddressNever<AccountId>;

	type AddressMapping = pallet_evm::IdentityAddressMapping;
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
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type SuicideQuickClearLimit = ConstU32<0>;
	type GasLimitStorageGrowthRatio = GasLimitStorageGrowthRatio;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Runtime>;
	type AccountProvider = FrameSystemAccountProvider<Runtime>;
	type RandomnessProvider = RandomnessProvider;
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
	fn destination(self) -> Location {
		match self {
			MockTransactors::Relay => Location::parent(),
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

parameter_types! {
	pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot =
		pallet_ethereum::IntermediateStateRoot<<Runtime as frame_system::Config>::Version>;
	type PostLogContent = PostBlockAndTxnHashes;
	type ExtraDataLength = ConstU32<30>;
}
parameter_types! {
	pub ReservedXcmpWeight: Weight = Weight::from_parts(u64::max_value(), 0);
}

#[derive(
	Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, MaxEncodedLen, TypeInfo,
)]
pub enum ProxyType {
	NotAllowed = 0,
	Any = 1,
}

impl pallet_evm_precompile_proxy::EvmProxyCallFilter for ProxyType {}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, _c: &RuntimeCall) -> bool {
		match self {
			ProxyType::NotAllowed => false,
			ProxyType::Any => true,
		}
	}
	fn is_superset(&self, _o: &Self) -> bool {
		false
	}
}

impl Default for ProxyType {
	fn default() -> Self {
		Self::NotAllowed
	}
}

parameter_types! {
	pub const ProxyCost: u64 = 1;
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyCost;
	type ProxyDepositFactor = ProxyCost;
	type MaxProxies = ConstU32<32>;
	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
	type MaxPending = ConstU32<32>;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = ProxyCost;
	type AnnouncementDepositFactor = ProxyCost;
}

pub struct EthereumXcmEnsureProxy;
impl xcm_primitives::EnsureProxy<AccountId> for EthereumXcmEnsureProxy {
	fn ensure_ok(delegator: AccountId, delegatee: AccountId) -> Result<(), &'static str> {
		// The EVM implicitely contains an Any proxy, so we only allow for "Any" proxies
		let def: pallet_proxy::ProxyDefinition<AccountId, ProxyType, BlockNumber> =
			pallet_proxy::Pallet::<Runtime>::find_proxy(
				&delegator,
				&delegatee,
				Some(ProxyType::Any),
			)
			.map_err(|_| "proxy error: expected `ProxyType::Any`")?;
		// We only allow to use it for delay zero proxies, as the call will iMmediatly be executed
		ensure!(def.delay.is_zero(), "proxy delay is Non-zero`");
		Ok(())
	}
}

impl pallet_ethereum_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type InvalidEvmTransactionError = pallet_ethereum::InvalidTransactionWrapper;
	type ValidatedTransaction = pallet_ethereum::ValidatedTransaction<Self>;
	type XcmEthereumOrigin = pallet_ethereum_xcm::EnsureXcmEthereumTransaction;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type EnsureProxy = EthereumXcmEnsureProxy;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ForceOrigin = EnsureRoot<AccountId>;
}

type Block = frame_system::mocking::MockBlockU32<Runtime>;

construct_runtime!(
	pub enum Runtime	{
		System: frame_system,
		Balances: pallet_balances,
		MsgQueue: mock_msg_queue,
		XcmVersioner: mock_version_changer,

		PolkadotXcm: pallet_xcm,
		Assets: pallet_assets,
		CumulusXcm: cumulus_pallet_xcm,
		AssetManager: pallet_asset_manager,
		XcmTransactor: pallet_xcm_transactor,
		XcmWeightTrader: pallet_xcm_weight_trader,
		Treasury: pallet_treasury,
		Proxy: pallet_proxy,

		Timestamp: pallet_timestamp,
		EVM: pallet_evm,
		Ethereum: pallet_ethereum,
		EthereumXcm: pallet_ethereum_xcm,
	}
);

pub(crate) fn para_events() -> Vec<RuntimeEvent> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| Some(e))
		.collect::<Vec<_>>()
}

use frame_support::traits::tokens::{PayFromAccount, UnityAssetBalanceConversion};
use frame_support::traits::{OnFinalize, OnInitialize, UncheckedOnRuntimeUpgrade};
use moonbase_runtime::{EvmForeignAssets, BLOCK_STORAGE_LIMIT, MAX_POV_SIZE};
use pallet_evm::FrameSystemAccountProvider;
use xcm_primitives::AsAssetType;

pub(crate) fn on_runtime_upgrade() {
	VersionUncheckedMigrateToV1::<Runtime>::on_runtime_upgrade();
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
