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
	traits::{All, Get, OriginTrait},
	weights::{constants::WEIGHT_PER_SECOND, Weight},
};
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{Hash, IdentityLookup},
	AccountId32,
};

use frame_system::{EnsureOneOf, EnsureRoot};
use sp_std::{
	convert::{TryFrom, TryInto},
	prelude::*,
};

use pallet_xcm::XcmPassthrough;
use polkadot_core_primitives::BlockNumber as RelayBlockNumber;
use polkadot_parachain::primitives::{Id as ParaId, Sibling};
use xcm::{
	v0::{
		Error as XcmError, ExecuteXcm,
		Junction::{Parachain, Parent},
		MultiAsset,
		MultiLocation::{self, X1},
		NetworkId, Outcome, Xcm,
	},
	VersionedXcm,
};
use xcm_builder::{
	AccountId32Aliases, AllowUnpaidExecutionFrom, CurrencyAdapter as XcmCurrencyAdapter,
	EnsureXcmOrigin, FixedRateOfConcreteFungible, FixedWeightBounds, IsConcrete, LocationInverter,
	NativeAsset, ParentIsDefault, SiblingParachainConvertsVia, SignedAccountId32AsNative,
	SignedToAccountId32, SovereignSignedViaLocation,
};
use xcm_executor::{Config, XcmExecutor};
use xcm_simulator::{
	DmpMessageHandlerT as DmpMessageHandler, XcmpMessageFormat,
	XcmpMessageHandlerT as XcmpMessageHandler,
};

pub type AccountId = moonbeam_core_primitives::AccountId;
pub type Balance = u128;

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
	type BaseCallFilter = ();
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

pub type AssetId = u32;

parameter_types! {
	pub const AssetDeposit: Balance = 0; // Does not really matter as this is forbidden
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

parameter_types! {
	pub const ReservedXcmpWeight: Weight = WEIGHT_PER_SECOND / 4;
	pub const ReservedDmpWeight: Weight = WEIGHT_PER_SECOND / 4;
}

parameter_types! {
	pub const KsmLocation: MultiLocation = MultiLocation::X1(Parent);
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	pub MoonbeamNetwork: NetworkId = NetworkId::Named("moon".into());
	pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = Parachain(MsgQueue::parachain_id().into()).into();
	pub SelfAssetLocation: MultiLocation = MultiLocation::X3(Parent, Parachain(MsgQueue::parachain_id().into()).into(), Junction::GeneralIndex{ id: 0}.into()).into();
	pub Local = MultiLocation::Null;
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the default `AccountId`.
	xcm_builder::ParentIsDefault<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	xcm_builder::SiblingParachainConvertsVia<polkadot_parachain::primitives::Sibling, AccountId>,
	xcm_builder::AccountKey20Aliases<RelayNetwork, AccountId>,
);

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	xcm_builder::SovereignSignedViaLocation<LocationToAccountId, Origin>,
	// Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
	// recognised.
	xcm_builder::RelayChainAsNative<RelayChainOrigin, Origin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognised.
	xcm_builder::SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
	// Superuser converter for the Relay-chain (Parent) location. This will allow it to issue a
	// transaction from the Root origin.
	xcm_builder::ParentAsSuperuser<Origin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	pallet_xcm::XcmPassthrough<Origin>,
	xcm_builder::SignedAccountKey20AsNative<RelayNetwork, Origin>,
);

parameter_types! {
	pub const UnitWeightCost: Weight = 1;
	pub KsmPerSecond: (MultiLocation, u128) = (X1(Parent), 1);
}

/// Converter struct implementing `AssetIdConversion` converting a numeric asset ID (must be `TryFrom/TryInto<u128>`) into
/// a Parachain junction, prefixed by some `MultiLocation` value.
use sp_std::borrow::Borrow;
use xcm::v0::Junction;
pub struct AsParachainId;
impl xcm_executor::traits::Convert<MultiLocation, AssetId> for AsParachainId {
	fn convert_ref(id: impl Borrow<MultiLocation>) -> Result<AssetId, ()> {
		println!("LOCATION IS {:?}", id.borrow());
		match id.borrow() {
			MultiLocation::X1(Junction::Parent) => {
				println!("I AM HER 3");
				Ok(0u32.into())
			}
			MultiLocation::X3(
				Junction::Parent,
				Junction::Parachain(para_id),
				Junction::GeneralIndex { id },
			) => {
				if *id == 0u128 {
					Ok(*para_id)
				} else {
					Err(())
				}
			}
			_ => {
				println!("THIS LOCATION IS UNHANDLED");
				Err(())
			}
		}
	}
	fn reverse_ref(what: impl Borrow<AssetId>) -> Result<MultiLocation, ()> {
		match what.borrow() {
			0u32 => Ok(MultiLocation::X1(Junction::Parent)),
			1000u32 => Ok(MultiLocation::X1(Junction::Parachain(1000))),
			1001u32 => Ok(MultiLocation::X1(Junction::Parachain(1001))),
			para_id => Ok(MultiLocation::X3(
				Junction::Parent,
				Junction::Parachain(*para_id),
				Junction::GeneralIndex { id: 0 },
			)),
		}
	}
}

pub type FungiblesTransactor = xcm_builder::FungiblesAdapter<
	// Use this fungibles implementation:
	Assets,
	// Use this currency when it is a fungible asset matching the given location or name:
	(
	/* 	xcm_builder::ConvertedConcreteAssetId<
			AssetId,
			Balance,
			AsParachainId,
			xcm_executor::traits::JustTry,
		>,*/
	xcm_builder::ConvertedConcreteAssetId<AssetId, Balance, AsPrefixedGeneralIndex<Local, AssetId, JustTry>, JustTry>,
	),
	// Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We dont allow teleports.
	(),
	// We dont track any teleports
	(),
>;

/// The transactor for our own chain currency.
pub type LocalAssetTransactor = xcm_builder::CurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	xcm_builder::IsConcrete<SelfAssetLocation>,
	// We can convert the MultiLocations with our converter above:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We track our teleports in/out to keep total issuance correct.
	(),
>;

pub type AssetTransactors = (LocalAssetTransactor, FungiblesTransactor);

pub type XcmRouter = super::ParachainXcmRouter<MsgQueue>;
pub type Barrier = AllowUnpaidExecutionFrom<All<MultiLocation>>;

use xcm_executor::traits::WeightTrader;
pub struct MyWeightTrader;
impl WeightTrader for MyWeightTrader {
	fn new() -> Self {
		MyWeightTrader
	}
	fn buy_weight(
		&mut self,
		_: Weight,
		a: xcm_executor::Assets,
	) -> Result<xcm_executor::Assets, XcmError> {
		Ok(a)
	}
}

use xcm_executor::traits::FilterAssetLocation;
// Change
pub struct MultiNativeAsset;
impl FilterAssetLocation for MultiNativeAsset {
	fn filter_asset_location(asset: &MultiAsset, origin: &MultiLocation) -> bool {
		return true;
	}
}

pub struct XcmConfig;
impl Config for XcmConfig {
	type Call = Call;
	type XcmSender = XcmRouter;
	type AssetTransactor = AssetTransactors;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = orml_xcm_support::MultiNativeAsset;
	type IsTeleporter = ();
	type LocationInverter = LocationInverter<Ancestry>;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
	//type Trader = FixedRateOfConcreteFungible<KsmPerSecond, ()>;
	type Trader = MyWeightTrader;
	type ResponseHandler = ();
}

impl cumulus_pallet_xcm::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

pub struct AssetIdtoMultiLocation<AssetXConverter>(sp_std::marker::PhantomData<AssetXConverter>);
impl<AssetXConverter> sp_runtime::traits::Convert<AssetId, Option<MultiLocation>>
	for AssetIdtoMultiLocation<AssetXConverter>
where
	AssetXConverter: xcm_executor::traits::Convert<MultiLocation, AssetId>,
{
	fn convert(asset: AssetId) -> Option<MultiLocation> {
		AssetXConverter::reverse_ref(asset).ok()
	}
}

pub struct AccountIdToMultiLocation;
impl sp_runtime::traits::Convert<AccountId, MultiLocation> for AccountIdToMultiLocation {
	fn convert(account: AccountId) -> MultiLocation {
		MultiLocation::X1(Junction::AccountKey20 {
			network: NetworkId::Any,
			key: account.into(),
		})
	}
}

parameter_types! {
	pub const BaseXcmWeight: Weight = 1;
	pub SelfLocation: MultiLocation = MultiLocation::X2(Parent, Parachain(MsgQueue::parachain_id().into()).into());
}

impl orml_xtokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type CurrencyId = AssetId;
	type AccountIdToMultiLocation = AccountIdToMultiLocation;
	type CurrencyIdConvert = AssetIdtoMultiLocation<AsParachainId>;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type SelfLocation = SelfLocation;
	type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, Call>;
	type BaseXcmWeight = BaseXcmWeight;
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
			println!("HELLOOOOOOOO");

			let (result, event) = match Xcm::<T::Call>::try_from(xcm) {
				Ok(xcm) => {
					let location = (Parent, Parachain(sender.into()));
					match T::XcmExecutor::execute_xcm(
						location.clone().into(),
						xcm.clone(),
						max_weight,
					) {
						Outcome::Error(e) => (Err(e.clone()), Event::Fail(Some(hash), e)),
						Outcome::Complete(w) => {
							println!("SUCCESS");
							println!("Location is {:?}", location);
							println!("XCM is {:?}", xcm);
							(Ok(w), Event::Success(Some(hash)))
						}
						// As far as the caller is concerned, this was dispatched without error, so
						// we just report the weight used.
						Outcome::Incomplete(w, e) => {
							println!("Error is {:?}", e);
							println!("Location is {:?}", location);
							println!("XCM is {:?}", xcm);
							(Ok(w), Event::Fail(Some(hash), e))
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
						println!("XCM is {:?}", x);
						let outcome = T::XcmExecutor::execute_xcm(Parent.into(), x, limit);
						Self::deposit_event(Event::ExecutedDownward(id, outcome));
					}
				}
			}
			limit
		}
	}
}

impl mock_msg_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

pub struct SignedToAccountId20<Origin, AccountId, Network>(
	sp_std::marker::PhantomData<(Origin, AccountId, Network)>,
);
impl<Origin: OriginTrait + Clone, AccountId: Into<[u8; 20]>, Network: Get<NetworkId>>
	xcm_executor::traits::Convert<Origin, MultiLocation>
	for SignedToAccountId20<Origin, AccountId, Network>
where
	Origin::PalletsOrigin: From<frame_system::RawOrigin<AccountId>>
		+ TryInto<frame_system::RawOrigin<AccountId>, Error = Origin::PalletsOrigin>,
{
	fn convert(o: Origin) -> Result<MultiLocation, Origin> {
		o.try_with_caller(|caller| match caller.try_into() {
			Ok(frame_system::RawOrigin::Signed(who)) => Ok(Junction::AccountKey20 {
				key: who.into(),
				network: Network::get(),
			}
			.into()),
			Ok(other) => Err(other.into()),
			Err(other) => Err(other),
		})
	}
}

pub type LocalOriginToLocation = SignedToAccountId20<Origin, AccountId, RelayNetwork>;

impl pallet_xcm::Config for Runtime {
	type Event = Event;
	type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmExecuteFilter = All<(MultiLocation, Xcm<Call>)>;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = ();
	type XcmReserveTransferFilter = All<(MultiLocation, Vec<MultiAsset>)>;
	type Weigher = FixedWeightBounds<UnitWeightCost, Call>;
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
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
		Assets: pallet_assets::{Pallet, Call, Storage, Event<T>},
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin},
		XTokens: orml_xtokens::{Pallet, Call, Storage, Event<T>}
	}
);
