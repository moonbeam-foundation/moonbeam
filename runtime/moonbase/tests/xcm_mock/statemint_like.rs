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

//! Relay chain runtime mock.

use frame_support::{
	construct_runtime, parameter_types,
	traits::{AsEnsureOriginWithArg, Contains, ContainsPair, Everything, Get, Nothing},
	weights::Weight,
};
use frame_system::{EnsureRoot, EnsureSigned};
use polkadot_core_primitives::BlockNumber as RelayBlockNumber;
use sp_core::H256;
use sp_runtime::{
	traits::{ConstU32, Hash, IdentityLookup},
	AccountId32,
};

use polkadot_parachain::primitives::Id as ParaId;
use polkadot_parachain::primitives::Sibling;
use sp_std::convert::TryFrom;
use xcm::latest::prelude::*;
use xcm::VersionedXcm;
use xcm_builder::{
	AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, AsPrefixedGeneralIndex,
	ConvertedConcreteId, EnsureXcmOrigin, FixedRateOfFungible, FixedWeightBounds, FungibleAdapter,
	FungiblesAdapter, IsConcrete, NoChecking, ParentAsSuperuser, ParentIsPreset,
	RelayChainAsNative, SiblingParachainAsNative, SiblingParachainConvertsVia,
	SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit,
};
use xcm_executor::{traits::JustTry, Config, XcmExecutor};
use xcm_simulator::{
	DmpMessageHandlerT as DmpMessageHandler, XcmpMessageFormat,
	XcmpMessageHandlerT as XcmpMessageHandler,
};
pub type AccountId = AccountId32;
pub type Balance = u128;
pub type AssetId = u128;

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
	type Lookup = IdentityLookup<Self::AccountId>;
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
	pub ExistentialDeposit: Balance = 1;
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
	pub const AssetDeposit: Balance = 0; // 1 UNIT deposit to create asset
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;
	/// Key = 32 bytes, Value = 36 bytes (32+1+1+1+1)
	// https://github.com/paritytech/substrate/blob/069917b/frame/assets/src/lib.rs#L257L271
	pub const MetadataDepositBase: Balance = 0;
	pub const MetadataDepositPerByte: Balance = 0;
	pub const ExecutiveBody: BodyId = BodyId::Executive;
	pub const AssetAccountDeposit: Balance = 0;
}

impl pallet_assets::Config for Runtime {
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
	type WeightInfo = ();
	type RemoveItemsLimit = ConstU32<656>;
	type AssetIdParameter = AssetId;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type CallbackHandle = ();
	pallet_assets::runtime_benchmarks_enabled! {
		type BenchmarkHelper = BenchmarkHelper;
	}
}

parameter_types! {
	pub const KsmLocation: Location = Location::parent();
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub UniversalLocation: InteriorLocation =
		[GlobalConsensus(RelayNetwork::get()), Parachain(MsgQueue::parachain_id().into())].into();
	pub Local: Location = Here.into();
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
	pub KsmPerSecond: (xcm::latest::prelude::AssetId, u128, u128) =
		(AssetId(KsmLocation::get()), 1, 1);
}

/// Type for specifying how a `Location` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

/// Means for transacting the native currency on this chain.
pub type CurrencyTransactor = FungibleAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<KsmLocation>,
	// Convert an XCM Location into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports of `Balances`.
	(),
>;

/// Means for transacting assets besides the native currency on this chain.
pub type FungiblesTransactor = FungiblesAdapter<
	// Use this fungibles implementation:
	Assets,
	// Use this currency when it is a fungible asset matching the given location or name:
	ConvertedConcreteId<
		AssetId,
		Balance,
		AsPrefixedGeneralIndex<PrefixChanger, AssetId, JustTry>,
		JustTry,
	>,
	// Convert an XCM Location into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We only want to allow teleports of known assets. We use non-zero issuance as an indication
	// that this asset is known.
	NoChecking,
	// The account to use for tracking teleports.
	CheckingAccount,
>;
/// Means for transacting assets on this chain.
pub type AssetTransactors = (CurrencyTransactor, FungiblesTransactor);

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will convert to a `Relay` origin when
	// recognised.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	ParentAsSuperuser<RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `RuntimeOrigin::signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	pallet_xcm::XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = Weight::from_parts(100u64, 100u64);
	pub const MaxInstructions: u32 = 100;
}

pub struct ParentOrParentsExecutivePlurality;
impl Contains<Location> for ParentOrParentsExecutivePlurality {
	fn contains(location: &Location) -> bool {
		matches!(
			location.unpack(),
			(1, [])
				| (
					1,
					[Plurality {
						id: BodyId::Executive,
						..
					}]
				)
		)
	}
}

pub struct ParentOrSiblings;
impl Contains<Location> for ParentOrSiblings {
	fn contains(location: &Location) -> bool {
		matches!(location.unpack(), (1, []) | (1, [_]))
	}
}

pub type Barrier = (
	TakeWeightCredit,
	AllowTopLevelPaidExecutionFrom<Everything>,
	// Parent and its exec plurality get free execution
	AllowUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
	// Expected responses are OK.
	AllowKnownQueryResponses<PolkadotXcm>,
	// Subscriptions for version tracking are OK.
	AllowSubscriptionsFrom<ParentOrSiblings>,
);

parameter_types! {
	pub MatcherLocation: Location = Location::here();
	pub const MaxAssetsIntoHolding: u32 = 64;
	pub const RelayTokenLocation: Location = Location::parent();
}

// Copied from:
//
// https://github.com/paritytech/polkadot-sdk/blob/f4eb41773611008040c9d4d8a8e6b7323eccfca1/cumulus
// /parachains/common/src/xcm_config.rs#L118
//
// The difference with the original "ConcreteAssetFromSystem" (which is used by AssetHub),
// is that in our tests we only need to check if the asset matches the relay one.
pub struct ConcreteAssetFromRelay<AssetLocation>(sp_std::marker::PhantomData<AssetLocation>);
impl<AssetLocation: Get<Location>> ContainsPair<Asset, Location>
	for ConcreteAssetFromRelay<AssetLocation>
{
	fn contains(asset: &Asset, origin: &Location) -> bool {
		let is_relay = match origin.unpack() {
			// The Relay Chain
			(1, []) => true,
			// Others
			_ => false,
		};
		asset.id.0 == AssetLocation::get() && is_relay
	}
}

pub type TrustedTeleporters = (ConcreteAssetFromRelay<RelayTokenLocation>,);

pub struct XcmConfig;
impl Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	type AssetTransactor = AssetTransactors;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve =
		orml_xcm_support::MultiNativeAsset<orml_traits::location::RelativeReserveProvider>;
	type IsTeleporter = TrustedTeleporters;
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = FixedRateOfFungible<KsmPerSecond, ()>;
	type ResponseHandler = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type CallDispatcher = RuntimeCall;
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

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

pub type XcmRouter = super::ParachainXcmRouter<MsgQueue>;

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = Nothing;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = Everything;
	type XcmReserveTransferFilter = Everything;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
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

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
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
impl mock_msg_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

// Pallet to cover test cases for change https://github.com/paritytech/cumulus/pull/831
#[frame_support::pallet]
pub mod mock_statemint_prefix {
	use super::*;
	use frame_support::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn current_prefix)]
	pub(super) type CurrentPrefix<T: Config> = StorageValue<_, Location, ValueQuery>;

	impl<T: Config> Get<Location> for Pallet<T> {
		fn get() -> Location {
			Self::current_prefix()
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Changed Prefix
		PrefixChanged(Location),
	}

	impl<T: Config> Pallet<T> {
		pub fn set_prefix(prefix: Location) {
			CurrentPrefix::<T>::put(&prefix);
			Self::deposit_event(Event::PrefixChanged(prefix));
		}
	}
}

impl mock_statemint_prefix::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

type Block = frame_system::mocking::MockBlockU32<Runtime>;
construct_runtime!(
	pub enum Runtime	{
		System: frame_system,
		Balances: pallet_balances,
		PolkadotXcm: pallet_xcm,
		CumulusXcm: cumulus_pallet_xcm,
		MsgQueue: mock_msg_queue,
		Assets: pallet_assets,
		PrefixChanger: mock_statemint_prefix,

	}
);
